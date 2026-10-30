//! Bounded party combat over public P2 records; no simulation lives in Core.
use ge4g_pentomino::p2::{
    ActionId, ActionType, ActionValue, CoreContext, CoreDescriptor, CorePlugin, CoreTransaction,
    EntityRef, Error, ErrorCode, FieldType, ObjectRef, ReadFrame, Record, RecordKey, RecordSchema,
    RefKind, SceneRef, SchemaId, Selection, Value,
};
use ge4g_pentomino::{Bindings, CapabilityId, PluginId, Version};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const SCENE_ID: &str = "p5_arena";
const STATUS: &str = "status";
fn invalid(detail: impl Into<String>) -> Error {
    Error::new(ErrorCode::InvalidInput, detail)
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FighterConfig {
    pub id: String,
    pub team: u32,
    pub hp: i64,
    pub resource: i64,
    pub attack: i64,
    pub defense: i64,
    pub speed: i64,
    pub position: [i64; 2],
    pub color: [u8; 4],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    SelfOnly,
    Ally,
    Enemy,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    Poison,
    Regen,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Damage,
    Heal,
    Resource,
    Effect,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionConfig {
    pub id: String,
    pub kind: ActionKind,
    pub target: Target,
    pub cost: i64,
    pub power: i64,
    pub effect: Option<Effect>,
    pub duration: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameConfig {
    pub version: u32,
    pub width: u32,
    pub height: u32,
    pub tick_hz: u32,
    pub fighters: Vec<FighterConfig>,
    pub actions: Vec<ActionConfig>,
}
impl GameConfig {
    pub fn validate(&self) -> Result<(), Error> {
        if self.version != 2 {
            return Err(Error::new(
                ErrorCode::VersionMismatch,
                "P5 config version must be 2",
            ));
        }
        if !(32..=512).contains(&self.width)
            || !(32..=512).contains(&self.height)
            || !(1..=240).contains(&self.tick_hz)
            || !(2..=4).contains(&self.fighters.len())
            || !(1..=16).contains(&self.actions.len())
        {
            return Err(invalid("config dimensions/roster/actions exceed bounds"));
        }
        let mut ids = BTreeSet::new();
        let mut teams = BTreeSet::new();
        for f in &self.fighters {
            if !identifier(&f.id)
                || !ids.insert(&f.id)
                || f.team > 1
                || !(1..=10000).contains(&f.hp)
                || [f.resource, f.attack, f.defense, f.speed]
                    .into_iter()
                    .any(|n| !(0..=10000).contains(&n))
            {
                return Err(invalid("invalid fighter identity/team/stats"));
            }
            if f.position[0] < 8
                || f.position[0] > i64::from(self.width) - 8
                || f.position[1] < 32
                || f.position[1] > i64::from(self.height)
            {
                return Err(invalid("fighter geometry must fit viewport"));
            }
            teams.insert(f.team);
        }
        if teams.len() != 2 {
            return Err(invalid("exactly two live teams required"));
        }
        ids.clear();
        for a in &self.actions {
            if !identifier(&a.id)
                || !ids.insert(&a.id)
                || !(0..=10000).contains(&a.cost)
                || !(0..=10000).contains(&a.power)
            {
                return Err(invalid("invalid action identifier/cost/power"));
            }
            if a.kind == ActionKind::Effect {
                if a.effect.is_none() || a.power == 0 || !(1..=100).contains(&a.duration) {
                    return Err(invalid("effect requires type/power/duration"));
                }
            } else if a.effect.is_some() || a.duration != 0 {
                return Err(invalid("non-effect has effect fields"));
            }
        }
        Ok(())
    }
    fn order(&self) -> Vec<usize> {
        let mut v: Vec<_> = (0..self.fighters.len()).collect();
        v.sort_by_key(|&i| (std::cmp::Reverse(self.fighters[i].speed), i));
        v
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub power: i64,
    pub turns: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FighterState {
    pub hp: i64,
    pub resource: i64,
    pub poison: Status,
    pub regen: Status,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleState {
    pub fighters: Vec<FighterState>,
    pub turn: usize,
    pub winner: i64,
    pub command_count: i64,
    pub last_command_id: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub id: i64,
    pub actor: usize,
    pub action: usize,
    pub target: usize,
}
impl BattleState {
    pub fn initial(config: &GameConfig) -> Result<Self, Error> {
        config.validate()?;
        Ok(Self {
            fighters: config
                .fighters
                .iter()
                .map(|f| FighterState {
                    hp: f.hp,
                    resource: f.resource,
                    poison: Status::default(),
                    regen: Status::default(),
                })
                .collect(),
            turn: config.order()[0],
            winner: -1,
            command_count: 0,
            last_command_id: 0,
        })
    }
    fn outcome(&self, config: &GameConfig) -> i64 {
        let teams: BTreeSet<_> = self
            .fighters
            .iter()
            .enumerate()
            .filter(|(_, f)| f.hp > 0)
            .map(|(i, _)| config.fighters[i].team)
            .collect();
        match teams.len() {
            0 => -2,
            1 => i64::from(*teams.first().expect("one team")),
            _ => -1,
        }
    }
    fn validate(&self, config: &GameConfig) -> Result<(), Error> {
        if self.fighters.len() != config.fighters.len()
            || self.turn >= self.fighters.len()
            || self.command_count < 0
            || self.last_command_id < self.command_count
            || (self.command_count == 0) != (self.last_command_id == 0)
        {
            return Err(invalid("invalid battle metadata"));
        }
        for (f, c) in self.fighters.iter().zip(&config.fighters) {
            if !(0..=c.hp).contains(&f.hp) || !(0..=c.resource).contains(&f.resource) {
                return Err(invalid("HP/resource out of bounds"));
            }
            for effect in [&f.poison, &f.regen] {
                if !(0..=10000).contains(&effect.power)
                    || effect.turns > 100
                    || (effect.power == 0) != (effect.turns == 0)
                    || (f.hp == 0 && effect.turns != 0)
                {
                    return Err(invalid("invalid status effect"));
                }
            }
        }
        if self.winner != self.outcome(config)
            || (self.winner == -1 && self.fighters[self.turn].hp == 0)
        {
            return Err(invalid("inconsistent outcome/turn"));
        }
        if self.command_count == 0 && self != &Self::initial(config)? {
            return Err(invalid("noninitial state has no commands"));
        }
        Ok(())
    }
    fn fields(&self) -> BTreeMap<String, Value> {
        let mut fields = BTreeMap::from([
            ("turn".into(), Value::I64(self.turn as i64)),
            ("winner".into(), Value::I64(self.winner)),
            ("command_count".into(), Value::I64(self.command_count)),
            ("last_command_id".into(), Value::I64(self.last_command_id)),
        ]);
        for (i, f) in self.fighters.iter().enumerate() {
            for (name, value) in [
                ("hp", f.hp),
                ("resource", f.resource),
                ("poison_power", f.poison.power),
                ("poison_turns", i64::from(f.poison.turns)),
                ("regen_power", f.regen.power),
                ("regen_turns", i64::from(f.regen.turns)),
            ] {
                fields.insert(format!("f{i}_{name}"), Value::I64(value));
            }
        }
        fields
    }
}
pub fn state_owner() -> PluginId {
    PluginId::new("p5.state").expect("static ID")
}
pub fn rules_owner() -> PluginId {
    PluginId::new("p5.turn_combat").expect("static ID")
}
pub fn capability() -> CapabilityId {
    CapabilityId::new("p5.battle_state").expect("static ID")
}
pub fn position_schema() -> SchemaId {
    SchemaId::new("p5.state.position").expect("static ID")
}
fn status_schema() -> SchemaId {
    SchemaId::new("p5.state.status").expect("static ID")
}
fn decision_schema() -> SchemaId {
    SchemaId::new("p5.turn_combat.decision").expect("static ID")
}
fn rule_action(field: &str) -> ActionId {
    ActionId::new(&format!("p5.turn_combat.{field}")).expect("static ID")
}
fn update_action(field: &str) -> ActionId {
    ActionId::new(&format!("p5.state.{field}")).expect("bounded field ID")
}
fn version() -> Version {
    Version {
        major: 1,
        minor: 0,
        patch: 0,
    }
}
pub fn bindings() -> Bindings {
    BTreeMap::from([(capability(), state_owner())])
}
pub fn scene_local() -> String {
    format!("s_{:x}", Sha256::digest(SCENE_ID.as_bytes()))
}
pub fn selection(include_rules: bool) -> Selection {
    let mut owners = BTreeSet::from([state_owner()]);
    if include_rules {
        owners.insert(rules_owner());
    }
    Selection {
        owners,
        schemas: BTreeSet::new(),
        include_objects: true,
        include_history: true,
    }
}
fn scalar(record: &Record, key: &str) -> Result<i64, Error> {
    match record.fields.get(key) {
        Some(Value::I64(n)) => Ok(*n),
        _ => Err(invalid(format!("missing scalar {key}"))),
    }
}
fn from_record(
    record: &Record,
    schema: SchemaId,
    config: &GameConfig,
) -> Result<BattleState, Error> {
    if record.schema != schema || record.fields.len() != config.fighters.len() * 6 + 4 {
        return Err(invalid("wrong status schema/field count"));
    }
    let mut fighters = Vec::new();
    for i in 0..config.fighters.len() {
        let get = |f: &str| scalar(record, &format!("f{i}_{f}"));
        let status = |name: &str| -> Result<Status, Error> {
            Ok(Status {
                power: get(&format!("{name}_power"))?,
                turns: u32::try_from(get(&format!("{name}_turns"))?)
                    .map_err(|_| invalid("negative effect duration"))?,
            })
        };
        fighters.push(FighterState {
            hp: get("hp")?,
            resource: get("resource")?,
            poison: status("poison")?,
            regen: status("regen")?,
        });
    }
    let state = BattleState {
        fighters,
        turn: usize::try_from(scalar(record, "turn")?).map_err(|_| invalid("negative turn"))?,
        winner: scalar(record, "winner")?,
        command_count: scalar(record, "command_count")?,
        last_command_id: scalar(record, "last_command_id")?,
    };
    state.validate(config)?;
    Ok(state)
}
pub fn scene(read: &ReadFrame) -> Result<SceneRef, Error> {
    let scenes: Vec<_> = read
        .scenes
        .iter()
        .filter(|s| s.reference.owner == state_owner())
        .collect();
    if scenes.len() != 1 || scenes[0].reference.local != scene_local() {
        return Err(invalid("missing/unexpected arena scene"));
    }
    Ok(scenes[0].reference.clone())
}
pub fn fighter_entities(read: &ReadFrame) -> Result<Vec<EntityRef>, Error> {
    let mut result = Vec::new();
    for i in 0..4 {
        let Some(record) = read.records.get(&RecordKey {
            owner: state_owner(),
            local: format!("position_{i}"),
        }) else {
            break;
        };
        let Some(Value::Ref(ObjectRef::Entity(entity))) = record.fields.get("entity") else {
            return Err(invalid("missing position entity ref"));
        };
        if entity.owner != state_owner()
            || !read.entities.iter().any(|e| e.reference == *entity)
            || result.contains(entity)
        {
            return Err(invalid("missing/duplicate live entity"));
        }
        result.push(entity.clone());
    }
    if !(2..=4).contains(&result.len())
        || read
            .entities
            .iter()
            .filter(|e| e.reference.owner == state_owner())
            .count()
            != result.len()
    {
        return Err(invalid("fighter entity count"));
    }
    Ok(result)
}
pub fn battle_state(read: &ReadFrame, config: &GameConfig) -> Result<BattleState, Error> {
    config.validate()?;
    let record = read
        .records
        .get(&RecordKey {
            owner: state_owner(),
            local: STATUS.into(),
        })
        .ok_or_else(|| invalid("missing status"))?;
    let state = from_record(record, status_schema(), config)?;
    let scene = scene(read)?;
    let entities = fighter_entities(read)?;
    if entities.len() != config.fighters.len() || read.records.len() != config.fighters.len() + 1 {
        return Err(invalid("authority record/roster count differs"));
    }
    for (i, (entity, f)) in entities.iter().zip(&config.fighters).enumerate() {
        let p = &read.records[&RecordKey {
            owner: state_owner(),
            local: format!("position_{i}"),
        }];
        if entity.local != f.id
            || p.schema != position_schema()
            || p.fields.len() != 4
            || scalar(p, "x")? != f.position[0]
            || scalar(p, "y")? != -f.position[1]
            || scalar(p, "z")? != 0
            || read
                .entities
                .iter()
                .find(|e| e.reference == *entity)
                .is_none_or(|e| e.scene != scene)
        {
            return Err(invalid("position/config identity mismatch"));
        }
    }
    Ok(state)
}
pub fn command_actions(
    command: &Command,
    config: &GameConfig,
) -> Result<Vec<(ActionId, ActionValue)>, Error> {
    config.validate()?;
    if command.id <= 0
        || command.actor >= config.fighters.len()
        || command.target >= config.fighters.len()
        || command.action >= config.actions.len()
    {
        return Err(invalid("command indices/identity invalid"));
    }
    Ok(BTreeMap::from([
        (rule_action("id"), ActionValue::I64(command.id)),
        (rule_action("actor"), ActionValue::I64(command.actor as i64)),
        (
            rule_action("action"),
            ActionValue::I64(command.action as i64),
        ),
        (
            rule_action("target"),
            ActionValue::I64(command.target as i64),
        ),
    ])
    .into_iter()
    .collect())
}
pub fn decide(
    state: &BattleState,
    command: &Command,
    config: &GameConfig,
) -> Result<BattleState, Error> {
    command_actions(command, config)?;
    state.validate(config)?;
    if command.id <= state.last_command_id {
        return Err(invalid("duplicate/stale command identity"));
    }
    if state.winner != -1
        || command.actor != state.turn
        || state.fighters[command.actor].hp == 0
        || state.fighters[command.target].hp == 0
    {
        return Err(invalid("finished battle/wrong actor/dead target"));
    }
    let action = &config.actions[command.action];
    let same_team = config.fighters[command.actor].team == config.fighters[command.target].team;
    let target_ok = match action.target {
        Target::SelfOnly => command.actor == command.target,
        Target::Ally => same_team,
        Target::Enemy => !same_team,
    };
    if !target_ok || state.fighters[command.actor].resource < action.cost {
        return Err(invalid("target relation/resource cost invalid"));
    }
    let mut next = state.clone();
    next.fighters[command.actor].resource -= action.cost;
    let target = &mut next.fighters[command.target];
    match action.kind {
        ActionKind::Damage => {
            let amount = (config.fighters[command.actor].attack + action.power
                - config.fighters[command.target].defense)
                .max(1);
            target.hp = (target.hp - amount).max(0);
        }
        ActionKind::Heal => {
            target.hp = (target.hp + action.power).min(config.fighters[command.target].hp)
        }
        ActionKind::Resource => {
            target.resource =
                (target.resource + action.power).min(config.fighters[command.target].resource)
        }
        ActionKind::Effect => {
            let effect = Status {
                power: action.power,
                turns: action.duration,
            };
            match action.effect.expect("validated effect") {
                Effect::Poison => target.poison = effect,
                Effect::Regen => target.regen = effect,
            };
        }
    }
    let actor = &mut next.fighters[command.actor];
    if actor.hp > 0 && actor.poison.turns > 0 {
        actor.hp = (actor.hp - actor.poison.power).max(0);
        actor.poison.turns -= 1;
        if actor.poison.turns == 0 {
            actor.poison.power = 0;
        }
    }
    if actor.hp > 0 && actor.regen.turns > 0 {
        actor.hp = (actor.hp + actor.regen.power).min(config.fighters[command.actor].hp);
        actor.regen.turns -= 1;
        if actor.regen.turns == 0 {
            actor.regen.power = 0;
        }
    }
    for f in &mut next.fighters {
        if f.hp == 0 {
            f.poison = Status::default();
            f.regen = Status::default();
        }
    }
    next.command_count = state
        .command_count
        .checked_add(1)
        .ok_or_else(|| Error::new(ErrorCode::Overflow, "command counter"))?;
    next.last_command_id = command.id;
    next.winner = next.outcome(config);
    if next.winner == -1 {
        let order = config.order();
        let pos = order
            .iter()
            .position(|i| *i == command.actor)
            .expect("valid actor");
        next.turn = (1..=order.len())
            .map(|offset| order[(pos + offset) % order.len()])
            .find(|&i| next.fighters[i].hp > 0)
            .ok_or_else(|| invalid("running battle has no living fighter"))?;
    }
    next.validate(config)?;
    Ok(next)
}
fn scalar_schema(config: &GameConfig) -> RecordSchema {
    RecordSchema {
        fields: BattleState::initial(config)
            .expect("validated config")
            .fields()
            .keys()
            .map(|f| {
                let (min, max) = if f == "winner" {
                    (-2, 1)
                } else if f == "turn" {
                    (0, config.fighters.len() as i64 - 1)
                } else if f == "command_count" || f == "last_command_id" {
                    (0, i64::MAX)
                } else if f.ends_with("_turns") {
                    (0, 100)
                } else {
                    (0, 10000)
                };
                (f.clone(), FieldType::I64 { min, max })
            })
            .collect(),
    }
}
fn descriptor(id: PluginId) -> CoreDescriptor {
    CoreDescriptor {
        id,
        release: Version {
            major: 0,
            minor: 3,
            patch: 0,
        },
        contract: Version {
            major: 2,
            minor: 0,
            patch: 0,
        },
        provides: BTreeMap::new(),
        requires: BTreeMap::new(),
        schemas: BTreeMap::new(),
        event_kinds: BTreeMap::new(),
        actions: BTreeMap::new(),
    }
}
pub struct StateProvider {
    config: GameConfig,
}
impl StateProvider {
    pub fn new(config: GameConfig) -> Result<Self, Error> {
        config.validate()?;
        Ok(Self { config })
    }
}
impl CorePlugin for StateProvider {
    fn descriptor(&self) -> CoreDescriptor {
        let mut d = descriptor(state_owner());
        d.provides.insert(capability(), version());
        d.schemas
            .insert(status_schema(), scalar_schema(&self.config));
        d.schemas.insert(
            position_schema(),
            RecordSchema {
                fields: BTreeMap::from([
                    (
                        "entity".into(),
                        FieldType::Ref {
                            kind: RefKind::Entity,
                        },
                    ),
                    (
                        "x".into(),
                        FieldType::I64 {
                            min: -512,
                            max: 512,
                        },
                    ),
                    (
                        "y".into(),
                        FieldType::I64 {
                            min: -512,
                            max: 512,
                        },
                    ),
                    ("z".into(), FieldType::I64 { min: 0, max: 0 }),
                ]),
            },
        );
        for (f, ty) in scalar_schema(&self.config).fields {
            let FieldType::I64 { min, max } = ty else {
                unreachable!()
            };
            d.actions
                .insert(update_action(&f), ActionType::I64 { min, max });
        }
        d.event_kinds.insert("applied".into(), status_schema());
        d
    }
    fn initialize(&self, _: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error> {
        let scene = tx.create_scene(&scene_local())?;
        for (i, f) in self.config.fighters.iter().enumerate() {
            let entity = tx.create_entity(&f.id, &scene)?;
            tx.set(
                &format!("position_{i}"),
                Record {
                    schema: position_schema(),
                    fields: BTreeMap::from([
                        ("entity".into(), Value::Ref(ObjectRef::Entity(entity))),
                        ("x".into(), Value::I64(f.position[0])),
                        ("y".into(), Value::I64(-f.position[1])),
                        ("z".into(), Value::I64(0)),
                    ]),
                },
            )?;
        }
        tx.set(
            STATUS,
            Record {
                schema: status_schema(),
                fields: BattleState::initial(&self.config)?.fields(),
            },
        )
    }
    fn tick(&self, ctx: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error> {
        let schema = scalar_schema(&self.config);
        let mut fields = BTreeMap::new();
        for f in schema.fields.keys() {
            if let Some(ActionValue::I64(n)) = ctx.action(&update_action(f))? {
                fields.insert(f.clone(), Value::I64(*n));
            }
        }
        if fields.is_empty() {
            return Ok(());
        }
        if fields.len() != schema.fields.len() {
            return Err(invalid("state update requires complete scalar batch"));
        }
        let record = Record {
            schema: status_schema(),
            fields,
        };
        tx.set(STATUS, record.clone())?;
        tx.emit("applied", record)
    }
}
pub struct TurnCombat {
    config: GameConfig,
}
impl TurnCombat {
    pub fn new(config: GameConfig) -> Result<Self, Error> {
        config.validate()?;
        Ok(Self { config })
    }
}
impl CorePlugin for TurnCombat {
    fn descriptor(&self) -> CoreDescriptor {
        let mut d = descriptor(rules_owner());
        d.requires.insert(capability(), version());
        d.schemas
            .insert(decision_schema(), scalar_schema(&self.config));
        d.event_kinds.insert("decision".into(), decision_schema());
        for f in ["id", "actor", "action", "target"] {
            let max = match f {
                "id" => i64::MAX,
                "action" => self.config.actions.len() as i64 - 1,
                _ => self.config.fighters.len() as i64 - 1,
            };
            d.actions.insert(
                rule_action(f),
                ActionType::I64 {
                    min: if f == "id" { 1 } else { 0 },
                    max,
                },
            );
        }
        d
    }
    fn initialize(&self, _: &CoreContext, _: &mut CoreTransaction<'_>) -> Result<(), Error> {
        Ok(())
    }
    fn tick(&self, ctx: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error> {
        let mut values = BTreeMap::new();
        for f in ["id", "actor", "action", "target"] {
            if let Some(ActionValue::I64(n)) = ctx.action(&rule_action(f))? {
                values.insert(f, *n);
            }
        }
        if values.is_empty() {
            return Ok(());
        }
        if values.len() != 4 {
            return Err(invalid("command requires four scalar actions"));
        }
        let command = Command {
            id: values["id"],
            actor: values["actor"] as usize,
            action: values["action"] as usize,
            target: values["target"] as usize,
        };
        let record = ctx
            .read(&state_owner(), STATUS)?
            .ok_or_else(|| invalid("missing battle state"))?;
        let state = from_record(record, status_schema(), &self.config)?;
        let next = decide(&state, &command, &self.config)?;
        tx.emit(
            "decision",
            Record {
                schema: decision_schema(),
                fields: next.fields(),
            },
        )
    }
}
pub fn decision_actions(
    read: &ReadFrame,
    tick: u64,
    expected: &Command,
    config: &GameConfig,
) -> Result<Vec<(ActionId, ActionValue)>, Error> {
    if read.tick != tick {
        return Err(invalid("decision tick mismatch"));
    }
    let state = battle_state(read, config)?;
    let next = decide(&state, expected, config)?;
    let history = read
        .histories
        .get(&rules_owner())
        .ok_or_else(|| invalid("missing rule history"))?;
    if history.pending.len() != 1 {
        return Err(invalid("exactly one decision required"));
    }
    let event = &history.pending[0];
    if event.owner != rules_owner()
        || event.tick != tick
        || event.kind != "decision"
        || from_record(&event.record, decision_schema(), config)? != next
    {
        return Err(invalid("decision differs from expected command result"));
    }
    Ok(next
        .fields()
        .into_iter()
        .map(|(f, v)| {
            let Value::I64(n) = v else { unreachable!() };
            (update_action(&f), ActionValue::I64(n))
        })
        .collect())
}

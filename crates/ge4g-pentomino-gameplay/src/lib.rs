//! Replaceable, trusted native turn rules over a persistent P2 record provider.
//!
//! One attack uses two fixed Core ticks: the rules emit a decision, then the
//! host translates that pending event into the provider's scalar update actions.
//! Neither plugin mutates another owner's records or stores hidden mutable state.
pub mod v2;

use ge4g_pentomino::p2::{
    ActionId, ActionType, ActionValue, CoreContext, CoreDescriptor, CorePlugin, CoreTransaction,
    EntityRef, Error, ErrorCode, FieldType, ObjectRef, ReadFrame, Record, RecordKey, RecordSchema,
    RefKind, SceneRef, SchemaId, Selection, Value,
};
use ge4g_pentomino::{Bindings, CapabilityId, PluginId, Version};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const SCENE_ID: &str = "p4_arena";
const STATUS: &str = "status";
const FIELDS: [&str; 5] = ["hp0", "hp1", "turn", "winner", "attack_count"];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FighterConfig {
    pub id: String,
    pub hp: i64,
    pub damage: i64,
    /// Pixel feet, with positive Y downward. Position records negate Y.
    pub position: [i64; 2],
    pub color: [u8; 4],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameConfig {
    pub version: u32,
    pub width: u32,
    pub height: u32,
    pub tick_hz: u32,
    pub attack_control: String,
    pub fighters: [FighterConfig; 2],
}

fn invalid(detail: impl Into<String>) -> Error {
    Error::new(ErrorCode::InvalidRecord, detail)
}

impl GameConfig {
    pub fn validate(&self) -> Result<(), Error> {
        if self.version != 1 {
            return Err(Error::new(
                ErrorCode::VersionMismatch,
                "gameplay config version must be 1",
            ));
        }
        if !(32..=512).contains(&self.width)
            || !(32..=512).contains(&self.height)
            || !(1..=240).contains(&self.tick_hz)
        {
            return Err(invalid("viewport must be 32..512 and tick_hz 1..240"));
        }
        if self.attack_control.is_empty()
            || self.attack_control.len() > 64
            || !self
                .attack_control
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            return Err(invalid(
                "attack_control must be a bounded ASCII control name",
            ));
        }
        for fighter in &self.fighters {
            // P2 local keys have the single-segment identifier grammar.
            if fighter.id.is_empty()
                || fighter.id.len() > 128
                || !fighter
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
                || !(1..=10000).contains(&fighter.hp)
                || !(1..=10000).contains(&fighter.damage)
            {
                return Err(invalid("invalid fighter identifier, HP or damage"));
            }
            let [x, y] = fighter.position;
            if x < 8 || x > i64::from(self.width) - 8 || y < 32 || y > i64::from(self.height) {
                return Err(invalid(
                    "fighter 16x24 geometry and HP bar must fit viewport",
                ));
            }
        }
        if self.fighters[0].id == self.fighters[1].id {
            return Err(invalid("fighter identifiers must be distinct"));
        }
        Ok(())
    }
}

pub fn state_owner() -> PluginId {
    PluginId::new("p4.state").expect("static valid plugin ID")
}
pub fn rules_owner() -> PluginId {
    PluginId::new("p4.turn_combat").expect("static valid plugin ID")
}
pub fn capability() -> CapabilityId {
    CapabilityId::new("p4.battle_state").expect("static valid capability ID")
}
pub fn attack_action() -> ActionId {
    ActionId::new("p4.turn_combat.attack").expect("static valid action ID")
}
pub fn position_schema() -> SchemaId {
    SchemaId::new("p4.state.position").expect("static valid schema ID")
}
fn status_schema() -> SchemaId {
    SchemaId::new("p4.state.status").expect("static valid schema ID")
}
fn decision_schema() -> SchemaId {
    SchemaId::new("p4.turn_combat.decision").expect("static valid schema ID")
}
fn update_action(field: &str) -> ActionId {
    ActionId::new(&format!("p4.state.{field}")).expect("static valid scalar field")
}
fn version(major: u32, minor: u32, patch: u32) -> Version {
    Version {
        major,
        minor,
        patch,
    }
}
pub fn bindings() -> Bindings {
    BTreeMap::from([(capability(), state_owner())])
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
pub fn scene_local() -> String {
    format!("s_{:x}", Sha256::digest(SCENE_ID.as_bytes()))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BattleState {
    pub hp: [i64; 2],
    pub turn: i64,
    /// -1 while running, otherwise the winning fighter index.
    pub winner: i64,
    pub attack_count: i64,
}
impl BattleState {
    fn fields(&self) -> BTreeMap<String, Value> {
        BTreeMap::from([
            ("hp0".into(), Value::I64(self.hp[0])),
            ("hp1".into(), Value::I64(self.hp[1])),
            ("turn".into(), Value::I64(self.turn)),
            ("winner".into(), Value::I64(self.winner)),
            ("attack_count".into(), Value::I64(self.attack_count)),
        ])
    }
    fn validate(&self, config: &GameConfig) -> Result<(), Error> {
        if !(0..=1).contains(&self.turn)
            || !(-1..=1).contains(&self.winner)
            || self.attack_count < 0
            || self
                .hp
                .iter()
                .enumerate()
                .any(|(i, hp)| *hp < 0 || *hp > config.fighters[i].hp)
        {
            return Err(invalid("battle state bounds"));
        }
        // Configuration bounds keep these calculations below 20001. Derive
        // the unique reachable status instead of trusting schema-valid scalars.
        let hits_to_defeat_1 =
            (config.fighters[1].hp + config.fighters[0].damage - 1) / config.fighters[0].damage;
        let hits_to_defeat_0 =
            (config.fighters[0].hp + config.fighters[1].damage - 1) / config.fighters[1].damage;
        let terminal_count = (2 * hits_to_defeat_1 - 1).min(2 * hits_to_defeat_0);
        if self.attack_count > terminal_count {
            return Err(invalid("attack count exceeds first battle termination"));
        }
        let hits_by_0 = (self.attack_count + 1) / 2;
        let hits_by_1 = self.attack_count / 2;
        let expected_hp = [
            (config.fighters[0].hp - hits_by_1 * config.fighters[1].damage).max(0),
            (config.fighters[1].hp - hits_by_0 * config.fighters[0].damage).max(0),
        ];
        let expected_winner = if self.attack_count == terminal_count {
            (self.attack_count - 1) % 2
        } else {
            -1
        };
        if self.hp != expected_hp || self.winner != expected_winner {
            return Err(invalid(
                "battle status is unreachable under configured rules",
            ));
        }
        if self.winner == -1 {
            if self.hp.contains(&0) || self.turn != self.attack_count % 2 {
                return Err(invalid("running battle state is inconsistent"));
            }
        } else {
            let winner = self.winner as usize;
            if self.attack_count == 0
                || self.turn != self.winner
                || self.winner != (self.attack_count - 1) % 2
                || self.hp[winner] == 0
                || self.hp[1 - winner] != 0
            {
                return Err(invalid("finished battle state is inconsistent"));
            }
        }
        Ok(())
    }
}
fn read_scalar(record: &Record, field: &str) -> Result<i64, Error> {
    match record.fields.get(field) {
        Some(Value::I64(value)) => Ok(*value),
        _ => Err(invalid(format!("missing or noninteger {field}"))),
    }
}
fn from_record(record: &Record, schema: SchemaId) -> Result<BattleState, Error> {
    if record.schema != schema || record.fields.len() != FIELDS.len() {
        return Err(invalid("unexpected battle record schema/field count"));
    }
    Ok(BattleState {
        hp: [read_scalar(record, "hp0")?, read_scalar(record, "hp1")?],
        turn: read_scalar(record, "turn")?,
        winner: read_scalar(record, "winner")?,
        attack_count: read_scalar(record, "attack_count")?,
    })
}
pub fn battle_state(read: &ReadFrame, config: &GameConfig) -> Result<BattleState, Error> {
    config.validate()?;
    let record = read
        .records
        .get(&RecordKey {
            owner: state_owner(),
            local: STATUS.into(),
        })
        .ok_or_else(|| invalid("missing persistent battle status"))?;
    let state = from_record(record, status_schema())?;
    state.validate(config)?;
    // Restores are also checked against the immutable configured scene/positions.
    let scene = scene(read)?;
    let entities = fighter_entities(read)?;
    for (i, entity) in entities.iter().enumerate() {
        if entity.local != config.fighters[i].id {
            return Err(invalid("restored fighter identity mismatch"));
        }
        let key = RecordKey {
            owner: state_owner(),
            local: format!("position_{i}"),
        };
        let record = read
            .records
            .get(&key)
            .ok_or_else(|| invalid("missing position record"))?;
        if record.schema != position_schema()
            || record.fields.len() != 4
            || record.fields.get("entity") != Some(&Value::Ref(ObjectRef::Entity(entity.clone())))
            || read_scalar(record, "x")? != config.fighters[i].position[0]
            || read_scalar(record, "y")? != -config.fighters[i].position[1]
            || read_scalar(record, "z")? != 0
            || read
                .entities
                .iter()
                .find(|e| e.reference == *entity)
                .is_none_or(|e| e.scene != scene)
        {
            return Err(invalid("restored position mismatch"));
        }
    }
    Ok(state)
}
pub fn scene(read: &ReadFrame) -> Result<SceneRef, Error> {
    let scenes: Vec<_> = read
        .scenes
        .iter()
        .filter(|s| s.reference.owner == state_owner())
        .collect();
    if scenes.len() != 1 || scenes[0].reference.local != scene_local() {
        return Err(invalid("expected exactly one battle scene"));
    }
    Ok(scenes[0].reference.clone())
}
pub fn fighter_entities(read: &ReadFrame) -> Result<[EntityRef; 2], Error> {
    let mut result = Vec::new();
    for i in 0..2 {
        let record = read
            .records
            .get(&RecordKey {
                owner: state_owner(),
                local: format!("position_{i}"),
            })
            .ok_or_else(|| invalid("missing fighter position"))?;
        let Some(Value::Ref(ObjectRef::Entity(entity))) = record.fields.get("entity") else {
            return Err(invalid("position entity reference"));
        };
        if entity.owner != state_owner() || !read.entities.iter().any(|e| e.reference == *entity) {
            return Err(invalid("missing live fighter entity"));
        }
        result.push(entity.clone());
    }
    if result[0] == result[1]
        || read
            .entities
            .iter()
            .filter(|e| e.reference.owner == state_owner())
            .count()
            != 2
    {
        return Err(invalid("expected exactly two distinct fighters"));
    }
    result.try_into().map_err(|_| invalid("fighter count"))
}

fn scalar_schema() -> RecordSchema {
    RecordSchema {
        fields: FIELDS
            .into_iter()
            .map(|field| {
                let (min, max) = match field {
                    "hp0" | "hp1" => (0, 10000),
                    "turn" => (0, 1),
                    "winner" => (-1, 1),
                    _ => (0, i64::MAX),
                };
                (field.into(), FieldType::I64 { min, max })
            })
            .collect(),
    }
}
fn descriptor(owner: PluginId) -> CoreDescriptor {
    CoreDescriptor {
        id: owner,
        release: version(0, 3, 0),
        contract: version(2, 0, 0),
        provides: BTreeMap::new(),
        requires: BTreeMap::new(),
        schemas: BTreeMap::new(),
        event_kinds: BTreeMap::new(),
        actions: BTreeMap::new(),
    }
}

/// Persistent generic scalar-record authority; no damage, turn or victory rules.
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
        d.provides.insert(capability(), version(1, 0, 0));
        d.schemas.insert(status_schema(), scalar_schema());
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
        for (field, ty) in scalar_schema().fields {
            let FieldType::I64 { min, max } = ty else {
                unreachable!("scalar schema")
            };
            d.actions
                .insert(update_action(&field), ActionType::I64 { min, max });
        }
        d.event_kinds.insert("applied".into(), status_schema());
        d
    }
    fn initialize(&self, _: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error> {
        let scene = tx.create_scene(&scene_local())?;
        for (i, fighter) in self.config.fighters.iter().enumerate() {
            let entity = tx.create_entity(&fighter.id, &scene)?;
            tx.set(
                &format!("position_{i}"),
                Record {
                    schema: position_schema(),
                    fields: BTreeMap::from([
                        ("entity".into(), Value::Ref(ObjectRef::Entity(entity))),
                        ("x".into(), Value::I64(fighter.position[0])),
                        ("y".into(), Value::I64(-fighter.position[1])),
                        ("z".into(), Value::I64(0)),
                    ]),
                },
            )?;
        }
        tx.set(
            STATUS,
            Record {
                schema: status_schema(),
                fields: BattleState {
                    hp: [self.config.fighters[0].hp, self.config.fighters[1].hp],
                    turn: 0,
                    winner: -1,
                    attack_count: 0,
                }
                .fields(),
            },
        )
    }
    fn tick(&self, ctx: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error> {
        let mut values = BTreeMap::new();
        for field in FIELDS {
            if let Some(value) = ctx.action(&update_action(field))? {
                let ActionValue::I64(value) = value else {
                    return Err(invalid("scalar update type"));
                };
                values.insert(field.into(), Value::I64(*value));
            }
        }
        if values.is_empty() {
            return Ok(());
        }
        if values.len() != FIELDS.len() {
            return Err(Error::new(
                ErrorCode::InvalidInput,
                "scalar update requires all five fields",
            ));
        }
        let record = Record {
            schema: status_schema(),
            fields: values,
        };
        tx.set(STATUS, record.clone())?;
        tx.emit("applied", record)
    }
}

/// Unloading this plugin removes decisions/actions only, preserving StateProvider.
pub struct TurnCombat {
    config: GameConfig,
}
impl TurnCombat {
    pub fn new(config: GameConfig) -> Result<Self, Error> {
        config.validate()?;
        Ok(Self { config })
    }
}
fn attack(state: &BattleState, config: &GameConfig) -> Result<Option<BattleState>, Error> {
    state.validate(config)?;
    if state.winner != -1 {
        return Ok(None);
    }
    let mut next = state.clone();
    let actor = state.turn as usize;
    let target = 1 - actor;
    next.hp[target] = (state.hp[target] - config.fighters[actor].damage).max(0);
    next.attack_count = state
        .attack_count
        .checked_add(1)
        .ok_or_else(|| Error::new(ErrorCode::Overflow, "attack count"))?;
    if next.hp[target] == 0 {
        next.winner = state.turn;
    } else {
        next.turn = target as i64;
    }
    Ok(Some(next))
}
impl CorePlugin for TurnCombat {
    fn descriptor(&self) -> CoreDescriptor {
        let mut d = descriptor(rules_owner());
        d.requires.insert(capability(), version(1, 0, 0));
        d.schemas.insert(decision_schema(), scalar_schema());
        d.event_kinds.insert("decision".into(), decision_schema());
        d.actions.insert(attack_action(), ActionType::Bool);
        d
    }
    fn initialize(&self, _: &CoreContext, _: &mut CoreTransaction<'_>) -> Result<(), Error> {
        Ok(())
    }
    fn tick(&self, ctx: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error> {
        if ctx.action(&attack_action())? != Some(&ActionValue::Bool(true)) {
            return Ok(());
        }
        let record = ctx
            .read(&state_owner(), STATUS)?
            .ok_or_else(|| invalid("missing battle provider state"))?;
        let state = from_record(record, status_schema())?;
        if let Some(next) = attack(&state, &self.config)? {
            tx.emit(
                "decision",
                Record {
                    schema: decision_schema(),
                    fields: next.fields(),
                },
            )?;
        }
        Ok(())
    }
}

/// Consume the actual current-tick pending decision and translate its fields.
/// A stale/duplicate/malformed/missing/unexpected decision is an explicit error.
pub fn decision_actions(
    read: &ReadFrame,
    tick: u64,
    expected_attack: bool,
    config: &GameConfig,
) -> Result<Vec<(ActionId, ActionValue)>, Error> {
    if read.tick != tick {
        return Err(invalid("decision read tick mismatch"));
    }
    let state = battle_state(read, config)?;
    let expected = if expected_attack {
        attack(&state, config)?
    } else {
        None
    };
    let history = read
        .histories
        .get(&rules_owner())
        .ok_or_else(|| invalid("missing rules event projection"))?;
    let Some(expected) = expected else {
        if !history.pending.is_empty() {
            return Err(invalid("unexpected pending decision"));
        }
        return Ok(Vec::new());
    };
    if history.pending.len() != 1 {
        return Err(invalid("expected exactly one pending decision"));
    }
    let event = &history.pending[0];
    if event.owner != rules_owner()
        || event.tick != tick
        || event.kind != "decision"
        || from_record(&event.record, decision_schema())? != expected
    {
        return Err(invalid("invalid or inconsistent pending decision"));
    }
    FIELDS
        .into_iter()
        .map(|field| {
            Ok((
                update_action(field),
                ActionValue::I64(read_scalar(&event.record, field)?),
            ))
        })
        .collect()
}

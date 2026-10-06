//! FlatLand v2 authoring: maps are static data; actors own behavior.
use crate::gameplay::{Atlas, QuestStatus, music_volume, one};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Map {
    pub cell: u32,
    pub rows: Vec<String>,
    pub tiles: BTreeMap<String, Tile>,
    #[serde(default)]
    pub portals: Vec<crate::gameplay::Portal>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Tile {
    pub color: [u8; 4],
    #[serde(default)]
    pub solid: bool,
    #[serde(default)]
    pub spawn: Option<String>,
    #[serde(default)]
    pub planes: Vec<i32>,
}
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum BodyMode {
    #[default]
    Pass,
    Fixed,
    Push,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Actor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub building: Option<crate::building::Building>,
    #[serde(default)]
    pub step_walk: bool,
    #[serde(default)]
    pub body: BodyMode,
    #[serde(default)]
    pub grid: bool,
    #[serde(default)]
    pub plane: i32,
    #[serde(default)]
    pub ai: Option<Ai>,
    #[serde(default)]
    pub hp: Option<i64>,
    #[serde(default)]
    pub pickup: Vec<Action>,
    #[serde(default)]
    pub animation: Option<Animation>,
    #[serde(default)]
    pub depth: bool,
    #[serde(default)]
    pub z: i32,
    #[serde(default)]
    pub visual_size: Option<[u32; 2]>,
    #[serde(default)]
    pub health_bar: bool,
    #[serde(default)]
    pub anchor: [i32; 2],
    #[serde(default)]
    pub team: String,
    #[serde(default)]
    pub defense: i64,
    #[serde(default)]
    pub attack: Option<String>,
    #[serde(default)]
    pub drops: Vec<Action>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Ai {
    pub speed: i64,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub corner: [i64; 2],
    #[serde(default)]
    pub flee_timer: Option<String>,
    #[serde(default)]
    pub mode: AiMode,
    #[serde(default)]
    pub patrol: Vec<[i64; 2]>,
    #[serde(default = "default_sight")]
    pub sight: u32,
}
fn default_sight() -> u32 {
    256
}
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AiMode {
    #[default]
    Chase,
    Idle,
    Patrol,
    Guard,
    Attack,
    Return,
    Flee,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Animation {
    pub frames: Vec<String>,
    pub ticks: u32,
    #[serde(default)]
    pub directions: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub rotate: bool,
    #[serde(default)]
    pub alternate_timer: Option<String>,
    #[serde(default)]
    pub alternate: Vec<String>,
    #[serde(default)]
    pub once: bool,
    #[serde(default)]
    pub atlas: Option<Atlas>,
    #[serde(default)]
    pub markers: BTreeMap<u32, Vec<Action>>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Condition {
    #[default]
    Always,
    State {
        key: String,
        eq: Value,
    },
    Timer {
        id: String,
    },
    Vulnerable {
        entity: String,
    },
    Remaining {
        tag: String,
        count: usize,
    },
    Hp {
        entity: String,
        le: i64,
    },
    HasItem {
        item: String,
        #[serde(default = "one")]
        count: i64,
    },
    QuestIs {
        quest: String,
        status: QuestStatus,
    },
    ObjectivesComplete {
        quest: String,
    },
    Any {
        conditions: Vec<Condition>,
    },
    All {
        conditions: Vec<Condition>,
    },
    Not {
        condition: Box<Condition>,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    ViewReset,
    CombatantHeal {
        combatant: String,
        amount: i64,
    },
    CombatantReset {
        combatant: String,
    },
    View {
        mode: Option<String>,
    },
    Pace {
        entity: String,
        speed: i64,
    },
    PlayClip {
        entity: String,
        clip: String,
    },
    Heal {
        entity: String,
        amount: i64,
    },
    Give {
        item: String,
        count: i64,
    },
    Take {
        item: String,
        count: i64,
    },
    Use {
        item: String,
    },
    Equip {
        slot: String,
        item: String,
    },
    Quest {
        quest: String,
        status: QuestStatus,
    },
    Objective {
        quest: String,
        objective: String,
        #[serde(default = "one")]
        count: i64,
    },
    Attack {
        entity: String,
        attack: String,
    },
    EventScene {
        event: String,
    },
    MapPatch {
        at: [usize; 2],
        tile: String,
    },
    Plane {
        entity: String,
        plane: i32,
    },
    Elevate {
        entity: String,
        z: i32,
    },
    Move {
        entity: String,
        at: [i64; 2],
    },
    Spawn {
        entity: String,
    },
    Animate {
        entity: String,
        frames: Vec<String>,
        ticks: u32,
        #[serde(default)]
        once: bool,
    },
    Roll {
        key: String,
        min: i64,
        max: i64,
    },
    Music {
        #[serde(default)]
        cue: Option<String>,
        #[serde(default = "music_volume")]
        volume: u32,
    },
    Set {
        key: String,
        value: Value,
    },
    Add {
        key: String,
        value: i64,
    },
    Timer {
        id: String,
        ticks: u64,
    },
    Say {
        text: String,
    },
    Remove {
        entity: String,
    },
    Damage {
        entity: String,
        amount: i64,
        #[serde(default)]
        immunity: u64,
    },
    Respawn {
        entity: String,
        #[serde(default)]
        immunity: u64,
    },
    Face {
        entity: String,
        vector: [i64; 2],
    },
    Goto {
        scene: String,
        spawn: String,
    },
    Stop {},
    Sound {
        cue: String,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub id: String,
    pub on: String,
    #[serde(default)]
    pub target_tag: Option<String>,
    #[serde(default)]
    pub when: Condition,
    #[serde(default)]
    pub once: bool,
    pub actions: Vec<Action>,
}

use super::{Entity, Error, Project, Result, Scene, StateStore, valid_id};

pub(crate) fn expand_scene(mut value: Value) -> Result<Value> {
    if value.get("schema_version").and_then(Value::as_u64) != Some(2) {
        if value.get("schema_version").and_then(Value::as_u64) == Some(1) {
            for e in value
                .get("entities")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if e.get("position").is_none() || e.get("size").is_none() {
                    return Err(Error("schema 1 entity requires position and size".into()));
                }
            }
        }
        return Ok(value);
    }
    let prefabs = value
        .get("prefabs")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let entities = value
        .get_mut("entities")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| Error("v2 scene needs entities".into()))?;
    for entity in entities.iter_mut() {
        if let Some(name) = entity
            .get("prefab")
            .and_then(Value::as_str)
            .map(str::to_owned)
        {
            let mut base = prefabs
                .get(&name)
                .cloned()
                .ok_or_else(|| Error(format!("unknown prefab {name}")))?;
            if base.get("prefab").is_some() {
                return Err(Error("nested prefab inheritance is unsupported".into()));
            }
            entity
                .as_object_mut()
                .ok_or_else(|| Error("entity must be an object".into()))?
                .remove("prefab");
            merge(&mut base, entity.clone());
            *entity = base;
        }
    }
    if let Some(map_value) = value.get("map") {
        let map: Map =
            serde_json::from_value(map_value.clone()).map_err(|e| Error(format!("map: {e}")))?;
        if map.rows.len() > 128
            || map.rows.iter().any(|r| !r.is_ascii() || r.len() > 128)
            || !(1..=128).contains(&map.cell)
        {
            return Err(Error(
                "map must fit 128x128 ASCII cells; cell must be 1..128".into(),
            ));
        }
        let mut spawned = Vec::new();
        for (y, row) in map.rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                if let Some(name) = map
                    .tiles
                    .get(&ch.to_string())
                    .and_then(|t| t.spawn.as_ref())
                {
                    let mut entity = prefabs
                        .get(name)
                        .cloned()
                        .ok_or_else(|| Error(format!("map tile: unknown prefab {name}")))?;
                    if entity.get("prefab").is_some() {
                        return Err(Error("nested prefab inheritance is unsupported".into()));
                    }
                    let size = entity.get("size").and_then(Value::as_array);
                    let w = size
                        .and_then(|s| s.first())
                        .and_then(Value::as_i64)
                        .unwrap_or(16);
                    let h = size
                        .and_then(|s| s.get(1))
                        .and_then(Value::as_i64)
                        .unwrap_or(16);
                    let o = entity
                        .as_object_mut()
                        .ok_or_else(|| Error("prefab must be an object".into()))?;
                    o.insert("id".into(), Value::from(format!("tile_{x}_{y}")));
                    o.insert(
                        "position".into(),
                        serde_json::json!([
                            x as i64 * i64::from(map.cell) + (i64::from(map.cell) - w) / 2,
                            y as i64 * i64::from(map.cell) + (i64::from(map.cell) - h) / 2
                        ]),
                    );
                    spawned.push(entity);
                }
            }
        }
        value
            .get_mut("entities")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .extend(spawned);
    }
    Ok(value)
}
pub(crate) fn merge(base: &mut Value, patch: Value) {
    if let (Some(a), Some(b)) = (base.as_object_mut(), patch.as_object()) {
        for (key, v) in b {
            if let Some(old) = a.get_mut(key) {
                merge(old, v.clone());
            } else {
                a.insert(key.clone(), v.clone());
            }
        }
    } else {
        *base = patch;
    }
}
impl Map {
    pub fn open(&self, x: i64, y: i64) -> bool {
        self.open_on_plane(x, y, 0)
    }
    pub fn open_on_plane(&self, x: i64, y: i64, plane: i32) -> bool {
        if x < 0 || y < 0 {
            return false;
        }
        self.rows
            .get(y as usize)
            .and_then(|r| r.as_bytes().get(x as usize))
            .and_then(|ch| self.tiles.get(&(*ch as char).to_string()))
            .is_some_and(|t| !t.solid || (!t.planes.is_empty() && !t.planes.contains(&plane)))
    }
}
impl Project {
    pub(crate) fn validate_flatland(&self, scene: &Scene) -> Result<()> {
        let fail = |s: &str| Error(format!("scene {}: {s}", scene.id));
        if scene.schema_version == 1 {
            if scene.map.is_some()
                || !scene.prefabs.is_empty()
                || !scene.rules.is_empty()
                || scene.script.is_some()
                || !scene.sounds.is_empty()
                || scene
                    .entities
                    .iter()
                    .any(|e| e.flatland.is_some() || e.prefab.is_some())
            {
                return Err(fail("FlatLand fields require schema_version 2"));
            }
            return Ok(());
        }
        if let Some(map) = &scene.map {
            let width = map.rows.first().map_or(0, String::len);
            if width == 0
                || width > 128
                || map.rows.is_empty()
                || map.rows.len() > 128
                || !(1..=128).contains(&map.cell)
                || map.rows.iter().any(|r| {
                    !r.is_ascii()
                        || r.len() != width
                        || r.chars().any(|ch| !map.tiles.contains_key(&ch.to_string()))
                })
                || map.tiles.keys().any(|k| k.len() != 1 || !k.is_ascii())
            {
                return Err(fail(
                    "map needs equal ASCII rows, defined single-character tiles and cell 1..128",
                ));
            }
            if map.tiles.values().any(|t| t.solid && t.spawn.is_some()) {
                return Err(fail("solid map tile cannot spawn an actor"));
            }
        }
        for entity in &scene.entities {
            if entity.player.as_ref().is_some_and(|p| p.speed > 6000) {
                return Err(fail("FlatLand player speed exceeds 6000"));
            }
            if let Some(a) = &entity.flatland {
                if entity.collider.is_some() {
                    return Err(fail("v2 actor uses body policy instead of collider"));
                }
                if a.hp.is_some_and(|hp| !(1..=1_000_000).contains(&hp)) {
                    return Err(fail("HP must be 1..1000000"));
                }
                if a.grid || a.step_walk || a.ai.is_some() {
                    let map = scene
                        .map
                        .as_ref()
                        .ok_or_else(|| fail("grid controller/AI needs a map"))?;
                    let cell = i64::from(map.cell);
                    if entity.position.iter().any(|v| v % cell != 0)
                        || entity.size.iter().any(|v| *v != map.cell)
                    {
                        return Err(fail(
                            "grid actors must align to cells and have cell-sized bodies",
                        ));
                    }
                }
                if let Some(ai) = &a.ai
                    && (!(1..=6000).contains(&ai.speed)
                        || ai
                            .target
                            .as_ref()
                            .is_some_and(|id| !scene.entities.iter().any(|e| &e.id == id)))
                {
                    return Err(fail("invalid AI speed or target"));
                }
                if let Some(anim) = &a.animation
                    && ((anim.frames.is_empty() && anim.atlas.is_none())
                        || anim.frames.len() > 128
                        || anim.alternate.len() > 128
                        || anim.directions.iter().any(|(key, frames)| {
                            !matches!(key.as_str(), "up" | "down" | "left" | "right")
                                || frames.is_empty()
                                || frames.len() > 128
                        })
                        || anim.ticks == 0)
                {
                    return Err(fail("animation needs 1..128 frames and positive ticks"));
                }
                for action in &a.pickup {
                    self.validate_action(scene, action)?;
                }
                if a.pickup.len() > 64 {
                    return Err(fail("pickup action limit is 64"));
                }
            }
            if entity.player.is_some()
                || entity
                    .flatland
                    .as_ref()
                    .is_some_and(|a| a.body != BodyMode::Pass)
            {
                self.check_map_position(scene, entity, entity.position)?;
            }
        }
        if let Some(player) = scene.entities.iter().find(|e| e.player.is_some()) {
            for position in scene.spawns.values() {
                self.check_map_position(scene, player, *position)?;
            }
        }
        self.validate_gameplay(scene)?;
        let mut ids = std::collections::BTreeSet::new();
        if scene.rules.len() > 256 {
            return Err(fail("rule limit is 256"));
        }
        for rule in &scene.rules {
            if !valid_id(&rule.id)
                || !ids.insert(&rule.id)
                || rule.actions.len() > 64
                || (![
                    "tick",
                    "contact",
                    "pickup",
                    "interact",
                    "start",
                    "hit",
                    "death",
                    "battle_win",
                    "battle_loss",
                ]
                .contains(&rule.on.as_str())
                    && !rule.on.starts_with("action."))
            {
                return Err(fail("invalid rule ID, trigger or action count"));
            }
            self.validate_condition(&rule.when, 0)?;
            for action in &rule.actions {
                self.validate_action(scene, action)?;
            }
        }
        Ok(())
    }
    fn check_map_position(&self, scene: &Scene, entity: &Entity, position: [i64; 2]) -> Result<()> {
        let Some(map) = &scene.map else {
            return Ok(());
        };
        let cell = i64::from(map.cell);
        for y in position[1].div_euclid(cell)
            ..=(position[1] + i64::from(entity.size[1]) - 1).div_euclid(cell)
        {
            for x in position[0].div_euclid(cell)
                ..=(position[0] + i64::from(entity.size[0]) - 1).div_euclid(cell)
            {
                if !map.open_on_plane(x, y, entity.flatland.as_ref().map_or(0, |a| a.plane)) {
                    return Err(Error(format!(
                        "scene {} entity {} overlaps map wall/boundary at {x},{y}",
                        scene.id, entity.id
                    )));
                }
            }
        }
        Ok(())
    }
    pub(crate) fn validate_condition(&self, condition: &Condition, depth: usize) -> Result<()> {
        if depth > 16 {
            return Err(Error("condition nesting exceeds 16".into()));
        }
        match condition {
            Condition::HasItem { item, count }
                if !(1..=1_000_000).contains(count)
                    || !self
                        .scenes
                        .values()
                        .any(|s| s.gameplay.items.contains_key(item)) =>
            {
                return Err(Error("condition references unknown item/count".into()));
            }
            Condition::QuestIs { quest, .. } | Condition::ObjectivesComplete { quest }
                if !self
                    .scenes
                    .values()
                    .any(|s| s.gameplay.quests.contains_key(quest)) =>
            {
                return Err(Error("condition references unknown quest".into()));
            }
            Condition::State { key, eq } => {
                let mut s = StateStore::new(self.manifest.state.clone())?;
                s.set(key, eq.clone())?;
            }
            Condition::All { conditions } | Condition::Any { conditions } => {
                if conditions.len() > 64 {
                    return Err(Error("condition list exceeds 64".into()));
                }
                for c in conditions {
                    self.validate_condition(c, depth + 1)?;
                }
            }
            Condition::Not { condition } => self.validate_condition(condition, depth + 1)?,
            _ => {}
        }
        Ok(())
    }
    pub fn validate_action(&self, scene: &Scene, action: &Action) -> Result<()> {
        self.validate_system_action(scene, action)?;
        match action {
            Action::Goto {
                scene: target,
                spawn,
            } => self.validate_transition(
                &super::Transition {
                    scene: target.clone(),
                    spawn: spawn.clone(),
                },
                &format!("scene {}", scene.id),
            )?,
            Action::Respawn { immunity, .. } if *immunity > 1_000_000 => {
                return Err(Error("respawn immunity exceeds tick limit".into()));
            }
            Action::Set { key, value } => {
                let mut s = StateStore::new(self.manifest.state.clone())?;
                s.set(key, value.clone())?;
            }
            Action::Add { key, .. } => {
                if self
                    .manifest
                    .state
                    .get(key)
                    .is_none_or(|d| d.kind != ge4g_core::StateType::Integer)
                {
                    return Err(Error(format!("add requires declared integer state {key}")));
                }
            }
            Action::Timer { id, ticks } => {
                if !valid_id(id) || *ticks > 1_000_000 {
                    return Err(Error("invalid timer ID/duration".into()));
                }
            }
            Action::Say { text } => {
                if text.len() > 8192 {
                    return Err(Error("dialogue exceeds 8192 bytes".into()));
                }
            }
            Action::Sound { cue } => {
                if !scene.sounds.contains_key(cue) {
                    return Err(Error(format!("unknown audio cue {cue}")));
                }
            }
            Action::Damage {
                amount, immunity, ..
            } if (!(0..=1_000_000).contains(amount) || *immunity > 1_000_000) => {
                return Err(Error("invalid damage/immunity".into()));
            }
            _ => {}
        }
        let reference = match action {
            Action::Remove { entity }
            | Action::Damage { entity, .. }
            | Action::Respawn { entity, .. }
            | Action::Face { entity, .. } => Some(entity),
            _ => None,
        };
        if let Some(id) = reference
            && !["$target", "$player"].contains(&id.as_str())
            && !scene.entities.iter().any(|e| &e.id == id)
        {
            return Err(Error(format!("action references missing entity {id}")));
        }
        Ok(())
    }
}

//! Optional FlatLand systems. Defaults keep previously authored v2 games unchanged.
use crate::flatland::{Action, Condition};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
fn seed() -> u64 {
    1
}
pub fn one() -> i64 {
    1
}
pub fn music_volume() -> u32 {
    40
}
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum QuestStatus {
    #[default]
    Inactive,
    Active,
    Completed,
    Failed,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Gameplay {
    #[serde(default = "seed")]
    pub seed: u64,
    #[serde(default)]
    pub combatants: BTreeMap<String, Fighter>,
    #[serde(default)]
    pub items: BTreeMap<String, Item>,
    #[serde(default)]
    pub quests: BTreeMap<String, Quest>,
    #[serde(default)]
    pub attacks: BTreeMap<String, Attack>,
    #[serde(default)]
    pub events: BTreeMap<String, Vec<Instruction>>,
    #[serde(default)]
    pub clips: BTreeMap<String, Clip>,
    #[serde(default)]
    pub modules: BTreeMap<String, String>,
    #[serde(default)]
    pub camera: Option<Camera>,
    #[serde(default)]
    pub views: BTreeMap<String, View>,
    #[serde(default)]
    pub default_view: Option<String>,
    #[serde(default)]
    pub lua_events: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct View {
    pub label: String,
    #[serde(default = "hundred")]
    pub zoom: i64,
    #[serde(default = "hundred")]
    pub tilt: i64,
    #[serde(default)]
    pub shear: i64,
}
fn hundred() -> i64 {
    100
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BattleMove {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fx: Option<BattleFxPreset>,
    pub id: String,
    pub name: String,
    #[serde(default = "hundred")]
    pub power: i64,
    #[serde(default = "default_pp")]
    pub pp: u32,
}
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum BattleFxPreset {
    #[default]
    Strike,
    Slash,
    Projectile,
    Burst,
    Heal,
    Guard,
}
fn default_pp() -> u32 {
    20
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BattleStage {
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub hold_result: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub name: String,
    #[serde(default = "one")]
    pub stack: i64,
    #[serde(default)]
    pub use_actions: Vec<Action>,
    #[serde(default)]
    pub slot: Option<String>,
    #[serde(default)]
    pub attack_bonus: i64,
    #[serde(default)]
    pub defense_bonus: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Quest {
    pub name: String,
    #[serde(default)]
    pub requires: Condition,
    pub objectives: BTreeMap<String, i64>,
    #[serde(default)]
    pub objective_labels: BTreeMap<String, String>,
    #[serde(default)]
    pub rewards: Vec<Action>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Attack {
    pub damage: i64,
    pub range: u32,
    #[serde(default)]
    pub startup: u32,
    #[serde(default = "one_tick")]
    pub active: u32,
    #[serde(default)]
    pub recovery: u32,
    #[serde(default)]
    pub immunity: u64,
    #[serde(default)]
    pub knockback: u32,
    #[serde(default)]
    pub projectile_speed: u32,
    #[serde(default = "projectile_life")]
    pub lifetime: u32,
    #[serde(default)]
    pub on_hit: Vec<Action>,
}
fn one_tick() -> u32 {
    1
}
fn projectile_life() -> u32 {
    60
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Atlas {
    pub file: String,
    pub cell: [u32; 2],
    pub indices: Vec<u32>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Camera {
    pub target: String,
    pub bounds: [i64; 4],
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub when: Condition,
    pub next: usize,
    #[serde(default)]
    pub actions: Vec<Action>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Fighter {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combatant: Option<String>,
    #[serde(default)]
    pub sprite: Option<String>,
    #[serde(default)]
    pub back_sprite: Option<String>,
    #[serde(default)]
    pub moves: Vec<BattleMove>,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default, alias = "max_hp")]
    pub hp: i64,
    #[serde(default)]
    pub attack: i64,
    #[serde(default)]
    pub defense: i64,
    #[serde(default)]
    pub speed: i64,
    #[serde(default)]
    pub enemy: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Instruction {
    Do {
        actions: Vec<Action>,
    },
    Wait {
        ticks: u32,
    },
    Say {
        text: String,
    },
    SayBubble {
        actor: String,
        text: String,
    },
    Choice {
        text: String,
        options: Vec<Choice>,
    },
    Branch {
        when: Condition,
        yes: usize,
        no: usize,
    },
    Jump {
        next: usize,
    },
    Camera {
        at: [i64; 2],
    },
    Call {
        event: String,
    },
    Battle {
        #[serde(default)]
        stage: Option<BattleStage>,
        fighters: Vec<Fighter>,
        #[serde(default)]
        victory: Vec<Action>,
        #[serde(default)]
        defeat: Vec<Action>,
    },
    Return {
        #[serde(default)]
        retain_view: bool,
    },
}

use crate::{Error, Project, Result, Scene, valid_id};
impl Gameplay {
    pub fn fighter(&self, authored: &Fighter) -> Result<Fighter> {
        if let Some(id) = &authored.combatant {
            let mut f = self
                .combatants
                .get(id)
                .cloned()
                .ok_or_else(|| Error(format!("unknown combatant {id}")))?;
            if f.combatant.is_some() {
                return Err(Error("nested combatant reference".into()));
            }
            f.id = if authored.id.is_empty() {
                id.clone()
            } else {
                authored.id.clone()
            };
            f.enemy = authored.enemy;
            f.combatant = Some(id.clone());
            Ok(f)
        } else {
            Ok(authored.clone())
        }
    }
}
impl Project {
    pub(crate) fn validate_gameplay(&self, scene: &Scene) -> Result<()> {
        let g = &scene.gameplay;
        if g.lua_events.len() > 32 || g.lua_events.iter().any(|e| e.is_empty() || e.len() > 64) {
            return Err(Error("invalid lua_events filter".into()));
        }
        if g.views.len() > 8
            || g.views.iter().any(|(id, v)| {
                !valid_id(id)
                    || v.label.is_empty()
                    || v.label.len() > 128
                    || !(100..=160).contains(&v.zoom)
                    || !(60..=100).contains(&v.tilt)
                    || v.shear.unsigned_abs() > 25
            })
            || g.default_view
                .as_ref()
                .is_some_and(|v| !g.views.contains_key(v))
        {
            return Err(Error("invalid view preset".into()));
        }
        if let Some(map) = &scene.map {
            let mut ids = std::collections::BTreeSet::new();
            if map.portals.len() > 64 {
                return Err(Error("portal limit 64".into()));
            }
            for p in &map.portals {
                if !valid_id(&p.id)
                    || !ids.insert(&p.id)
                    || p.from == p.to
                    || !map.open_on_plane(p.at[0], p.at[1], p.from)
                    || !map.open_on_plane(p.destination[0], p.destination[1], p.to)
                {
                    return Err(Error("invalid portal/reference/clearance".into()));
                }
            }
        }
        let fail = |s: &str| Error(format!("scene {} gameplay: {s}", scene.id));
        if g.combatants.len() > 256 {
            return Err(fail("combatant limit 256"));
        }
        for (id, f) in &g.combatants {
            if !valid_id(id)
                || f.combatant.is_some()
                || !(1..=1_000_000).contains(&f.hp)
                || !(1..=10000).contains(&f.attack)
                || !(0..=10000).contains(&f.defense)
                || !(0..=10000).contains(&f.speed)
                || f.moves.len() > 4
                || f.moves.iter().any(|m| {
                    !valid_id(&m.id)
                        || m.name.is_empty()
                        || !(1..=300).contains(&m.power)
                        || m.pp > 100
                })
                || f.moves
                    .iter()
                    .map(|m| &m.id)
                    .collect::<std::collections::BTreeSet<_>>()
                    .len()
                    != f.moves.len()
                || [&f.sprite, &f.back_sprite]
                    .into_iter()
                    .flatten()
                    .any(|t| !self.textures.contains_key(t))
            {
                return Err(fail("invalid persistent combatant"));
            }
            for other in self
                .scenes
                .values()
                .filter_map(|s| s.gameplay.combatants.get(id))
            {
                if serde_json::to_value(f).unwrap() != serde_json::to_value(other).unwrap() {
                    return Err(fail("combatant definition differs across scenes"));
                }
            }
        }

        if [
            g.items.len(),
            g.quests.len(),
            g.attacks.len(),
            g.events.len(),
            g.modules.len(),
        ]
        .iter()
        .any(|n| *n > 256)
        {
            return Err(fail("resource limit 256"));
        }
        for id in g
            .items
            .keys()
            .chain(g.quests.keys())
            .chain(g.attacks.keys())
            .chain(g.events.keys())
            .chain(g.modules.keys())
        {
            if !valid_id(id) {
                return Err(fail("invalid resource ID"));
            }
        }
        let check = |actions: &[Action]| -> Result<()> {
            if actions.len() > 64 {
                return Err(fail("action limit 64"));
            }
            for a in actions {
                self.validate_action(scene, a)?;
            }
            Ok(())
        };
        for c in g.clips.values() {
            if c.frames.is_empty()
                || c.frames.len() > 128
                || c.ticks == 0
                || c.frames.iter().any(|f| !self.textures.contains_key(f))
            {
                return Err(fail("invalid named clip"));
            }
        }
        for i in g.items.values() {
            if !(0..=10000).contains(&i.attack_bonus)
                || !(0..=10000).contains(&i.defense_bonus)
                || !(1..=1_000_000).contains(&i.stack)
                || i.name.len() > 256
            {
                return Err(fail("invalid item"));
            }
            check(&i.use_actions)?;
        }
        for q in g.quests.values() {
            if q.objectives.is_empty()
                || q.objectives.len() > 64
                || q.objectives
                    .iter()
                    .any(|(id, n)| !valid_id(id) || !(1..=1_000_000).contains(n))
            {
                return Err(fail("invalid quest objectives"));
            }
            self.validate_condition(&q.requires, 0)?;
            check(&q.rewards)?;
        }
        for a in g.attacks.values() {
            if !(1..=1_000_000).contains(&a.damage)
                || a.range > 4096
                || a.active == 0
                || a.startup > 10000
                || a.active > 10000
                || a.recovery > 10000
                || a.immunity > 10000
                || a.knockback > 1024
                || a.projectile_speed > 6000
                || !(1..=10000).contains(&a.lifetime)
            {
                return Err(fail("invalid attack bounds"));
            }
            check(&a.on_hit)?;
        }
        for steps in g.events.values() {
            if steps.is_empty() || steps.len() > 1024 {
                return Err(fail("event needs 1..1024 instructions"));
            }
            for step in steps {
                match step {
                    Instruction::Do { actions } => check(actions)?,
                    Instruction::Wait { ticks } if *ticks > 10000 => {
                        return Err(fail("wait exceeds 10000 ticks"));
                    }
                    Instruction::Say { text }
                    | Instruction::SayBubble { text, .. }
                    | Instruction::Choice { text, .. }
                        if text.len() > 8192 =>
                    {
                        return Err(fail("event text too long"));
                    }
                    Instruction::SayBubble { actor, .. }
                        if !scene.entities.iter().any(|e| &e.id == actor) =>
                    {
                        return Err(fail("say_bubble actor must exist in the scene"));
                    }
                    Instruction::Choice { options, .. } => {
                        if options.is_empty() || options.len() > 16 {
                            return Err(fail("choice needs 1..16 options"));
                        }
                        let mut ids = std::collections::BTreeSet::new();
                        for c in options {
                            if !valid_id(&c.id)
                                || !ids.insert(&c.id)
                                || c.next >= steps.len()
                                || c.text.len() > 256
                            {
                                return Err(fail("invalid choice"));
                            }
                            self.validate_condition(&c.when, 0)?;
                            check(&c.actions)?;
                        }
                    }
                    Instruction::Branch { when, yes, no } => {
                        self.validate_condition(when, 0)?;
                        if *yes >= steps.len() || *no >= steps.len() {
                            return Err(fail("branch outside event"));
                        }
                    }
                    Instruction::Jump { next } if *next >= steps.len() => {
                        return Err(fail("jump outside event"));
                    }
                    Instruction::Call { event } if !g.events.contains_key(event) => {
                        return Err(fail("unknown event call"));
                    }
                    Instruction::Battle {
                        fighters,
                        victory,
                        defeat,
                        stage,
                    } => {
                        if let Some(bg) = stage.as_ref().and_then(|s| s.background.as_ref()) {
                            self.path(bg)?;
                            if !self.textures.contains_key(bg) {
                                return Err(fail("missing battle background"));
                            }
                        }
                        let fighters: Vec<_> = fighters
                            .iter()
                            .map(|f| g.fighter(f))
                            .collect::<Result<_>>()?;
                        if fighters.len() < 2
                            || fighters.len() > 16
                            || !fighters.iter().any(|f| f.enemy)
                            || fighters.iter().filter(|f| !f.enemy).count() != 1
                        {
                            return Err(fail("battle needs one hero and 1..15 enemies"));
                        }
                        let mut ids = std::collections::BTreeSet::new();
                        for f in &fighters {
                            if f.moves.len() > 4
                                || f.moves.iter().any(|m| {
                                    !valid_id(&m.id)
                                        || m.name.is_empty()
                                        || !(1..=300).contains(&m.power)
                                        || m.pp > 100
                                })
                                || f.moves
                                    .iter()
                                    .map(|m| &m.id)
                                    .collect::<std::collections::BTreeSet<_>>()
                                    .len()
                                    != f.moves.len()
                            {
                                return Err(fail("invalid battle moves"));
                            }
                            for file in [&f.sprite, &f.back_sprite].into_iter().flatten() {
                                if !self.textures.contains_key(file) {
                                    return Err(fail("missing fighter sprite"));
                                }
                            }
                            if !valid_id(&f.id)
                                || !ids.insert(&f.id)
                                || !(1..=1_000_000).contains(&f.hp)
                                || !(1..=10000).contains(&f.attack)
                                || !(0..=10000).contains(&f.defense)
                                || !(0..=10000).contains(&f.speed)
                            {
                                return Err(fail("invalid battle participant"));
                            }
                        }
                        check(victory)?;
                        check(defeat)?;
                    }
                    _ => {}
                }
            }
        }
        if let Some(c) = &g.camera
            && (!scene.entities.iter().any(|e| e.id == c.target)
                || c.bounds[0] > c.bounds[2]
                || c.bounds[1] > c.bounds[3]
                || c.bounds.iter().any(|n| n.unsigned_abs() > 1_000_000))
        {
            return Err(fail("invalid follow camera"));
        }
        for e in &scene.entities {
            if let Some(a) = &e.flatland {
                if let Some(b) = &a.building
                    && (!b.valid() || b.surfaces().any(|t| !self.textures.contains_key(t)))
                {
                    return Err(fail(
                        "invalid building presentation (0.2 supports facing=south only)",
                    ));
                }
                if a.visual_size
                    .is_some_and(|size| size.iter().any(|n| !(1..=1024).contains(n)))
                    || a.anchor.iter().any(|n| n.unsigned_abs() > 4096)
                    || a.z.unsigned_abs() > 4096
                    || !(0..=10000).contains(&a.defense)
                    || a.attack
                        .as_ref()
                        .is_some_and(|id| !g.attacks.contains_key(id))
                {
                    return Err(fail("invalid actor elevation/defense/attack"));
                }
                check(&a.drops)?;
                if let Some(anim) = &a.animation {
                    for actions in anim.markers.values() {
                        check(actions)?;
                    }
                    if let Some(atlas) = &anim.atlas {
                        let t = self
                            .textures
                            .get(&atlas.file)
                            .ok_or_else(|| fail("missing atlas"))?;
                        if atlas.cell.contains(&0)
                            || atlas.indices.is_empty()
                            || atlas.indices.len() > 128
                            || atlas.cell[0] > t.width
                            || atlas.cell[1] > t.height
                            || atlas.indices.iter().any(|i| {
                                *i >= (t.width / atlas.cell[0]) * (t.height / atlas.cell[1])
                            })
                        {
                            return Err(fail("invalid atlas frame"));
                        }
                    }
                }
            }
        }
        Ok(())
    }
    pub(crate) fn validate_system_action(&self, scene: &Scene, a: &Action) -> Result<()> {
        let g = &scene.gameplay;
        let fail = |s: &str| Error(format!("scene {} action: {s}", scene.id));
        match a {
            Action::CombatantHeal { combatant, amount }
                if !g.combatants.contains_key(combatant) || !(0..=1_000_000).contains(amount) =>
            {
                return Err(fail("invalid combatant heal"));
            }
            Action::CombatantReset { combatant } if !g.combatants.contains_key(combatant) => {
                return Err(fail("unknown combatant"));
            }
            Action::Heal { amount, .. } if !(0..=1_000_000).contains(amount) => {
                return Err(fail("invalid healing amount"));
            }
            Action::Give { item, count } | Action::Take { item, count } => {
                if !g.items.contains_key(item) || !(1..=1_000_000).contains(count) {
                    return Err(fail("invalid item/count"));
                }
            }
            Action::Pace { speed, .. } if !(12..=6000).contains(speed) => {
                return Err(Error("pace speed 12..6000".into()));
            }
            Action::View { mode } if mode.as_ref().is_some_and(|id| !g.views.contains_key(id)) => {
                return Err(Error("unknown view preset".into()));
            }
            Action::PlayClip { clip, .. } if !g.clips.contains_key(clip) => {
                return Err(fail("unknown clip"));
            }
            Action::Use { item } | Action::Equip { item, .. } if !g.items.contains_key(item) => {
                return Err(fail("unknown item"));
            }
            Action::Quest { quest, .. } if !g.quests.contains_key(quest) => {
                return Err(fail("unknown quest"));
            }
            Action::Objective {
                quest,
                objective,
                count,
            } => {
                if !g
                    .quests
                    .get(quest)
                    .is_some_and(|q| q.objectives.contains_key(objective))
                    || !(1..=1_000_000).contains(count)
                {
                    return Err(fail("invalid objective"));
                }
            }
            Action::Attack { attack, .. } if !g.attacks.contains_key(attack) => {
                return Err(fail("unknown attack"));
            }
            Action::EventScene { event } if !g.events.contains_key(event) => {
                return Err(fail("unknown event scene"));
            }
            Action::MapPatch { at, tile } => {
                if !scene.map.as_ref().is_some_and(|m| {
                    at[1] < m.rows.len() && at[0] < m.rows[0].len() && m.tiles.contains_key(tile)
                }) {
                    return Err(fail("invalid map patch"));
                }
            }
            Action::Elevate { z, .. } if z.unsigned_abs() > 4096 => {
                return Err(fail("elevation exceeds 4096"));
            }
            Action::Move { at, .. } if at.iter().any(|n| n.unsigned_abs() > 1_000_000) => {
                return Err(fail("move outside bounds"));
            }
            Action::Animate { frames, ticks, .. }
                if frames.is_empty()
                    || frames.len() > 128
                    || *ticks == 0
                    || frames.iter().any(|f| !self.textures.contains_key(f)) =>
            {
                return Err(fail("invalid animation override"));
            }
            Action::Roll { key, min, max } => {
                if min > max
                    || min.unsigned_abs() > 1_000_000
                    || max.unsigned_abs() > 1_000_000
                    || self
                        .manifest
                        .state
                        .get(key)
                        .is_none_or(|d| d.kind != ge4g_core::StateType::Integer)
                {
                    return Err(fail("invalid integer random range/state"));
                }
            }
            Action::Music { cue, volume }
                if *volume > 100 || cue.as_ref().is_some_and(|c| !scene.sounds.contains_key(c)) =>
            {
                return Err(fail("invalid music cue/volume"));
            }
            _ => {}
        }
        let entity = match a {
            Action::Pace { entity, .. }
            | Action::PlayClip { entity, .. }
            | Action::Heal { entity, .. }
            | Action::Attack { entity, .. }
            | Action::Plane { entity, .. }
            | Action::Elevate { entity, .. }
            | Action::Move { entity, .. }
            | Action::Spawn { entity }
            | Action::Animate { entity, .. } => Some(entity),
            _ => None,
        };
        if entity.is_some_and(|id| {
            !["$player", "$target"].contains(&id.as_str())
                && !scene.entities.iter().any(|e| &e.id == id)
        }) {
            return Err(fail("unknown actor"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Portal {
    pub id: String,
    pub at: [i64; 2],
    pub from: i32,
    pub to: i32,
    pub destination: [i64; 2],
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Clip {
    pub frames: Vec<String>,
    pub ticks: u32,
    #[serde(default)]
    pub once: bool,
}

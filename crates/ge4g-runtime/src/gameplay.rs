//! Serializable gameplay controllers; authoritative commands, never presentation callbacks.
use super::*;
use ge4g_project::flatland::{Action, BodyMode, Condition, Map};
use ge4g_project::gameplay::{Fighter, Instruction, QuestStatus};
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct QuestProgress {
    pub status: QuestStatus,
    pub objectives: BTreeMap<String, i64>,
    pub rewarded: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AttackInstance {
    pub owner: String,
    pub preset: String,
    pub start: u64,
    pub hit: BTreeSet<String>,
    pub origin: Vec2,
    pub direction: [i64; 2],
    pub plane: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnimationState {
    #[serde(default)]
    pub event_serial: u64,
    pub frames: Vec<String>,
    pub ticks: u32,
    pub once: bool,
    pub start: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MusicState {
    pub cue: String,
    pub file: String,
    pub volume: u32,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Systems {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub pace: BTreeMap<String, i64>,
    pub rng: u64,
    #[serde(default)]
    pub event_serial: u64,
    #[serde(default)]
    pub paused_ticks: u64,
    pub inventory: BTreeMap<String, i64>,
    pub equipment: BTreeMap<String, String>,
    pub quests: BTreeMap<String, QuestProgress>,
    pub attacks: Vec<AttackInstance>,
    #[serde(default)]
    pub pending_deaths: BTreeSet<String>,
    pub cooldowns: BTreeMap<String, u64>,
    pub planes: BTreeMap<String, i32>,
    pub elevation: BTreeMap<String, i32>,
    pub patches: BTreeMap<String, String>,
    pub animations: BTreeMap<String, AnimationState>,
    pub patrol: BTreeMap<String, usize>,
    #[serde(default)]
    pub portal_latches: BTreeMap<String, [i64; 3]>,
    pub events: Vec<EventFrame>,
    pub music: Option<MusicState>,
    pub camera: Option<[i64; 2]>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EventFrame {
    pub serial: u64,
    pub event: String,
    pub pc: usize,
    pub wait: u64,
    pub waiting: bool,
    pub battle: Option<BattleState>,
    pub parent: BTreeMap<String, LiveEntity>,
    #[serde(default)]
    pub parent_view: Option<String>,
    #[serde(default)]
    pub parent_pace: BTreeMap<String, i64>,
    pub parent_camera: Option<[i64; 2]>,
    pub parent_planes: BTreeMap<String, i32>,
    pub parent_elevation: BTreeMap<String, i32>,
    pub parent_animations: BTreeMap<String, AnimationState>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BattleState {
    #[serde(default)]
    pub turn_tick: u64,
    #[serde(default)]
    pub previous_hp: BTreeMap<String, i64>,
    #[serde(default)]
    pub result: Option<bool>,
    pub fighters: Vec<Fighter>,
    pub turn: u64,
    pub log: String,
}
impl World {
    pub(super) fn systems(&mut self) -> &mut Systems {
        let seed = self.project.scenes[&self.scene].gameplay.seed.max(1);
        self.flatland.systems.get_or_insert_with(|| Systems {
            rng: seed,
            ..Systems::default()
        })
    }
    pub fn inventory(&self, item: &str) -> i64 {
        self.flatland
            .systems
            .as_ref()
            .and_then(|s| s.inventory.get(item))
            .copied()
            .unwrap_or(0)
    }
    pub(super) fn actor_plane(&self, id: &str) -> i32 {
        self.flatland
            .systems
            .as_ref()
            .and_then(|s| s.planes.get(id))
            .copied()
            .unwrap_or_else(|| {
                self.entities
                    .get(id)
                    .and_then(|e| e.spec.flatland.as_ref())
                    .map_or(0, |a| a.plane)
            })
    }
    fn patched_map(&self) -> Option<Map> {
        let mut m = self.project.scenes[&self.scene].map.clone()?;
        if let Some(s) = &self.flatland.systems {
            for (key, tile) in &s.patches {
                let prefix = format!("{}:", self.scene);
                if let Some(at) = key.strip_prefix(&prefix) {
                    let a: Vec<_> = at
                        .split(':')
                        .filter_map(|x| x.parse::<usize>().ok())
                        .collect();
                    if a.len() == 2 && a[1] < m.rows.len() && a[0] < m.rows[a[1]].len() {
                        m.rows[a[1]].replace_range(a[0]..a[0] + 1, tile);
                    }
                }
            }
        }
        Some(m)
    }
    pub(super) fn effective_map(&self, plane: i32) -> Option<Map> {
        let mut m = self.patched_map()?;
        for t in m.tiles.values_mut() {
            if !t.planes.is_empty() && !t.planes.contains(&plane) {
                t.solid = false;
            }
            t.planes.clear();
        }
        Some(m)
    }
    pub(super) fn actor_clear(&self, id: &str, at: Vec2, plane: i32) -> bool {
        let Some(e) = self.entities.get(id) else {
            return false;
        };
        let body = Aabb {
            position: at,
            ..e.aabb()
        };
        if self.effective_map(plane).as_ref().is_some_and(|m| {
            let cell = i64::from(m.cell) * SUBPIXELS;
            ((body.position.y.div_euclid(cell))
                ..=(body.position.y + body.size.y - 1).div_euclid(cell))
                .any(|y| {
                    ((body.position.x.div_euclid(cell))
                        ..=(body.position.x + body.size.x - 1).div_euclid(cell))
                        .any(|x| !m.open(x, y))
                })
        }) {
            return false;
        }
        !self.entities.iter().any(|(other, e)| {
            other != id
                && self.actor_plane(other) == plane
                && e.spec
                    .flatland
                    .as_ref()
                    .is_some_and(|a| a.body != BodyMode::Pass)
                && body.overlaps(e.aabb())
        })
    }
    pub(super) fn random(&mut self, min: i64, max: i64) -> Result<i64> {
        if min > max || min.unsigned_abs() > 1_000_000 || max.unsigned_abs() > 1_000_000 {
            return Err(Error("random range -1000000..1000000".into()));
        }
        let s = self.systems();
        let mut x = s.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.rng = x;
        Ok(min + (x % ((max - min + 1) as u64)) as i64)
    }
    pub(super) fn system_condition(&self, c: &Condition) -> bool {
        match c {
            Condition::HasItem { item, count } => self.inventory(item) >= *count,
            Condition::ObjectivesComplete { quest } => self.project.scenes[&self.scene]
                .gameplay
                .quests
                .get(quest)
                .is_some_and(|q| {
                    self.flatland
                        .systems
                        .as_ref()
                        .and_then(|s| s.quests.get(quest))
                        .is_some_and(|progress| {
                            q.objectives
                                .iter()
                                .all(|(id, n)| progress.objectives.get(id).unwrap_or(&0) >= n)
                        })
                }),
            Condition::QuestIs { quest, status } => {
                self.flatland
                    .systems
                    .as_ref()
                    .and_then(|s| s.quests.get(quest))
                    .map_or(QuestStatus::Inactive, |q| q.status)
                    == *status
            }
            _ => false,
        }
    }
    pub(super) fn system_action(&mut self, a: &Action, target: Option<&str>) -> Result<()> {
        match a {
            Action::PlayClip { entity, clip } => {
                let c = self.project.scenes[&self.scene].gameplay.clips[clip].clone();
                self.actions(
                    &[Action::Animate {
                        entity: entity.clone(),
                        frames: c.frames,
                        ticks: c.ticks,
                        once: c.once,
                    }],
                    target,
                )?;
            }
            Action::Give { item, count } | Action::Take { item, count } => {
                let limit = self.project.scenes[&self.scene].gameplay.items[item].stack;
                let old = self.inventory(item);
                let value = if matches!(a, Action::Give { .. }) {
                    old.checked_add(*count)
                } else {
                    old.checked_sub(*count)
                }
                .ok_or_else(|| Error("item quantity overflow".into()))?;
                if value < 0 || value > limit {
                    return Err(Error(format!("item {item} quantity outside 0..{limit}")));
                }
                self.systems().inventory.insert(item.clone(), value);
                if value == 0 {
                    self.systems().equipment.retain(|_, id| id != item);
                }
                self.emit("inventory", None, json!({"item":item,"count":value}));
            }
            Action::Use { item } => {
                let effects = self.project.scenes[&self.scene].gameplay.items[item]
                    .use_actions
                    .clone();
                if effects.is_empty() {
                    return Err(Error("item is not usable".into()));
                }
                self.actions(
                    &[Action::Take {
                        item: item.clone(),
                        count: 1,
                    }],
                    target,
                )?;
                self.actions(&effects, target)?;
            }
            Action::Equip { slot, item } => {
                if self.inventory(item) < 1
                    || self.project.scenes[&self.scene].gameplay.items[item]
                        .slot
                        .as_ref()
                        != Some(slot)
                {
                    return Err(Error("item/slot cannot equip".into()));
                }
                self.systems().equipment.insert(slot.clone(), item.clone());
                self.emit("equipped", None, json!({"slot":slot,"item":item}));
            }
            Action::Quest { quest, status } => {
                let definition = self.project.scenes[&self.scene].gameplay.quests[quest].clone();
                let old = self
                    .systems()
                    .quests
                    .get(quest)
                    .map_or(QuestStatus::Inactive, |q| q.status);
                if old == *status {
                    return Ok(());
                }
                if matches!(old, QuestStatus::Completed | QuestStatus::Failed) {
                    return Err(Error("quest terminal transition".into()));
                }
                if *status == QuestStatus::Active && !self.condition(&definition.requires, target) {
                    return Err(Error("quest prerequisites unmet".into()));
                }
                let progress =
                    self.systems()
                        .quests
                        .entry(quest.clone())
                        .or_insert(QuestProgress {
                            status: QuestStatus::Inactive,
                            objectives: BTreeMap::new(),
                            rewarded: false,
                        });
                if *status == QuestStatus::Completed
                    && (old != QuestStatus::Active
                        || definition
                            .objectives
                            .iter()
                            .any(|(id, n)| progress.objectives.get(id).unwrap_or(&0) < n))
                {
                    return Err(Error("quest objectives incomplete".into()));
                }
                progress.status = *status;
                let reward = *status == QuestStatus::Completed && !progress.rewarded;
                if reward {
                    progress.rewarded = true;
                }
                self.emit("quest", None, json!({"quest":quest,"status":status}));
                if reward {
                    self.actions(&definition.rewards, target)?;
                }
            }
            Action::Objective {
                quest,
                objective,
                count,
            } => {
                let max =
                    self.project.scenes[&self.scene].gameplay.quests[quest].objectives[objective];
                let q = self
                    .systems()
                    .quests
                    .get_mut(quest)
                    .ok_or_else(|| Error("quest not active".into()))?;
                if q.status != QuestStatus::Active {
                    return Err(Error("objective requires active quest".into()));
                }
                let old = *q.objectives.get(objective).unwrap_or(&0);
                let n = old
                    .checked_add(*count)
                    .ok_or_else(|| Error("objective overflow".into()))?
                    .min(max);
                q.objectives.insert(objective.clone(), n);
                self.emit(
                    "objective",
                    None,
                    json!({"quest":quest,"objective":objective,"count":n}),
                );
            }
            Action::Attack { entity, attack } => {
                let id = self.ref_id(entity, target)?;
                self.begin_attack(&id, attack)?;
            }
            Action::EventScene { event } => self.begin_event(event)?,
            Action::MapPatch { at, tile } => {
                let key = format!("{}:{}:{}", self.scene, at[0], at[1]);
                self.systems().patches.insert(key, tile.clone());
                for (id, e) in &self.entities {
                    if !self.actor_clear(id, e.position, self.actor_plane(id)) {
                        return Err(Error("map patch would enclose an actor".into()));
                    }
                }
                self.emit("map_patch", None, json!({"at":at,"tile":tile}));
            }
            Action::Plane { entity, plane } => {
                let id = self.ref_id(entity, target)?;
                if !self.actor_clear(&id, self.entities[&id].position, *plane) {
                    return Err(Error("plane destination blocked".into()));
                }
                self.systems().planes.insert(id.clone(), *plane);
                self.emit("plane", Some(&id), json!({"plane":plane}));
            }
            Action::View { mode } => {
                self.systems().view = mode.clone();
                self.emit("view_changed", None, json!({"mode":mode}));
            }
            Action::Pace { entity, speed } => {
                let id = self.ref_id(entity, target)?;
                self.systems().pace.insert(id, *speed);
            }
            Action::Elevate { entity, z } => {
                let id = self.ref_id(entity, target)?;
                self.systems().elevation.insert(id, *z);
            }
            Action::Move { entity, at } => {
                let id = self.ref_id(entity, target)?;
                let pos = Vec2::pixels(at[0], at[1]);
                if !self.actor_clear(&id, pos, self.actor_plane(&id)) {
                    return Err(Error("scripted move destination blocked".into()));
                }
                self.entities
                    .get_mut(&id)
                    .ok_or_else(|| Error("missing mover".into()))?
                    .position = pos;
            }
            Action::Spawn { entity } => {
                let id = self.ref_id(entity, target)?;
                if self.entities.contains_key(&id) {
                    return Err(Error("actor already present".into()));
                }
                let spec = self.project.scenes[&self.scene]
                    .entities
                    .iter()
                    .find(|e| e.id == id)
                    .cloned()
                    .ok_or_else(|| Error("spawn actor undefined".into()))?;
                let live = LiveEntity {
                    actor: flatland::ActorState::new(&spec),
                    position: Vec2::pixels(spec.position[0], spec.position[1]),
                    spec,
                };
                self.entities.insert(id.clone(), live);
                if !self.actor_clear(&id, self.entities[&id].position, self.actor_plane(&id)) {
                    return Err(Error("spawn destination blocked".into()));
                }
                self.emit("spawn", Some(&id), json!({}));
            }
            Action::Animate {
                entity,
                frames,
                ticks,
                once,
            } => {
                let id = self.ref_id(entity, target)?;
                let start = self.tick;
                let event_serial = self
                    .flatland
                    .systems
                    .as_ref()
                    .and_then(|s| s.events.last())
                    .map_or(0, |f| f.serial);
                self.systems().animations.insert(
                    id,
                    AnimationState {
                        event_serial,
                        frames: frames.clone(),
                        ticks: *ticks,
                        once: *once,
                        start,
                    },
                );
            }
            Action::Roll { key, min, max } => {
                let value = self.random(*min, *max)?;
                self.set_state(&BTreeMap::from([(key.clone(), json!(value))]), target)?;
            }
            Action::Music { cue, volume } => {
                let music = cue.as_ref().map(|c| MusicState {
                    cue: c.clone(),
                    file: self.project.scenes[&self.scene].sounds[c].clone(),
                    volume: *volume,
                });
                self.systems().music = music.clone();
                self.emit("music", None, json!(music));
            }
            _ => return Err(Error("unsupported system action".into())),
        }
        Ok(())
    }
    fn equipment_bonus(&self, id: &str, attack: bool) -> i64 {
        if !self
            .entities
            .get(id)
            .is_some_and(|e| e.spec.player.is_some())
        {
            return 0;
        }
        self.flatland
            .systems
            .as_ref()
            .map(|s| {
                s.equipment
                    .values()
                    .filter_map(|id| {
                        self.project
                            .scenes
                            .values()
                            .find_map(|scene| scene.gameplay.items.get(id))
                    })
                    .map(|i| {
                        if attack {
                            i.attack_bonus
                        } else {
                            i.defense_bonus
                        }
                    })
                    .sum()
            })
            .unwrap_or(0)
    }
    fn begin_attack(&mut self, id: &str, preset: &str) -> Result<()> {
        let e = self
            .entities
            .get(id)
            .ok_or_else(|| Error("attacker missing".into()))?;
        if e.actor.hp == Some(0) {
            return Ok(());
        }
        if self
            .flatland
            .systems
            .as_ref()
            .and_then(|s| s.cooldowns.get(id))
            .is_some_and(|t| *t > self.tick)
        {
            return Ok(());
        }
        let a = self.project.scenes[&self.scene]
            .gameplay
            .attacks
            .get(preset)
            .ok_or_else(|| Error("attack preset missing".into()))?
            .clone();
        let instance = AttackInstance {
            owner: id.into(),
            preset: preset.into(),
            start: self.tick,
            hit: BTreeSet::new(),
            origin: e.position,
            direction: e.actor.facing,
            plane: self.actor_plane(id),
        };
        let end = self.tick + u64::from(a.startup + a.active + a.recovery);
        let s = self.systems();
        if s.attacks.len() >= 128 {
            return Err(Error("attack instance limit".into()));
        }
        s.cooldowns.insert(id.into(), end);
        s.attacks.push(instance);
        self.emit("attack", Some(id), json!({"preset":preset}));
        Ok(())
    }
    pub(super) fn ai_decision(
        &mut self,
        id: &str,
        ai: &ge4g_project::flatland::Ai,
        map: &Map,
        at: [i64; 2],
    ) -> Result<[i64; 2]> {
        use ge4g_project::flatland::AiMode;
        if ai.mode == AiMode::Idle {
            return Ok([0, 0]);
        }
        let e = &self.entities[id];
        let cell = i64::from(map.cell) * SUBPIXELS;
        let home = [
            e.spec.position[0] / i64::from(map.cell),
            e.spec.position[1] / i64::from(map.cell),
        ];
        let target_id = ai
            .target
            .as_ref()
            .filter(|id| self.entities.contains_key(*id));
        let visible = target_id.is_some_and(|target| {
            let t = &self.entities[target];
            self.actor_plane(id) == self.actor_plane(target)
                && (t.position.x - e.position.x)
                    .abs()
                    .max((t.position.y - e.position.y).abs())
                    <= i64::from(ai.sight) * SUBPIXELS
                && self.line_open(e.position, t.position, self.actor_plane(id))
        });
        let mut target = target_id
            .map(|id| {
                [
                    self.entities[id].position.x / cell,
                    self.entities[id].position.y / cell,
                ]
            })
            .unwrap_or(ai.corner);
        if ai.mode == AiMode::Patrol {
            if ai.patrol.is_empty() {
                return Ok([0, 0]);
            }
            let idx = self
                .flatland
                .systems
                .as_ref()
                .and_then(|s| s.patrol.get(id))
                .copied()
                .unwrap_or(0)
                % ai.patrol.len();
            let idx = if at == ai.patrol[idx] {
                (idx + 1) % ai.patrol.len()
            } else {
                idx
            };
            self.systems().patrol.insert(id.into(), idx);
            target = ai.patrol[idx];
        } else if ai.mode == AiMode::Return
            || (matches!(ai.mode, AiMode::Guard | AiMode::Attack) && !visible)
        {
            target = home;
        }
        if ai.mode == AiMode::Attack && visible {
            let attack = self.entities[id]
                .spec
                .flatland
                .as_ref()
                .and_then(|a| a.attack.clone());
            if let (Some(target), Some(attack)) = (target_id.cloned(), attack) {
                let range = self.project.scenes[&self.scene].gameplay.attacks[&attack].range;
                if self.in_front(id, &target, range)? {
                    self.begin_attack(id, &attack)?;
                    return Ok([0, 0]);
                }
            }
        }
        if let Some(target_id) = target_id {
            let target_plane = self.actor_plane(target_id);
            if target_plane != self.actor_plane(id) {
                return self.portal_direction(id, at, target, target_plane);
            }
        }
        if target == at {
            return Ok([0, 0]);
        }
        Ok(super::flatland::next_direction(
            map,
            at,
            target,
            self.entities[id].actor.direction,
            ai.mode == AiMode::Flee
                || ai.flee_timer.as_ref().is_some_and(|t| {
                    self.flatland
                        .timers
                        .get(t)
                        .is_some_and(|end| *end > self.tick)
                }),
        ))
    }
    fn line_open(&self, from: Vec2, to: Vec2, plane: i32) -> bool {
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let steps = (dx.abs().max(dy.abs()) / SUBPIXELS).clamp(1, 4096);
        (0..=steps).all(|i| {
            self.point_open(
                Vec2 {
                    x: from.x + dx * i / steps,
                    y: from.y + dy * i / steps,
                },
                plane,
            )
        })
    }
    pub(super) fn combat_tick(&mut self) -> Result<()> {
        let pending = self.systems_if_present_attacks();
        let mut keep = Vec::new();
        for mut hit in pending {
            let Some(owner) = self.entities.get(&hit.owner) else {
                continue;
            };
            if owner.actor.hp == Some(0) {
                continue;
            }
            let Some(a) = self.project.scenes[&self.scene]
                .gameplay
                .attacks
                .get(&hit.preset)
                .cloned()
            else {
                continue;
            };
            let age = self.tick.saturating_sub(hit.start);
            let life = if a.projectile_speed > 0 {
                a.startup + a.lifetime
            } else {
                a.startup + a.active
            };
            if age >= u64::from(life) {
                continue;
            }
            if age >= u64::from(a.startup) {
                let team = owner
                    .spec
                    .flatland
                    .as_ref()
                    .map(|a| a.team.clone())
                    .unwrap_or_default();
                let travel =
                    age.saturating_sub(u64::from(a.startup)) as i64 * i64::from(a.projectile_speed);
                let projectile = Vec2 {
                    x: hit.origin.x + travel * hit.direction[0],
                    y: hit.origin.y + travel * hit.direction[1],
                };
                let before = Vec2 {
                    x: projectile.x - i64::from(a.projectile_speed) * hit.direction[0],
                    y: projectile.y - i64::from(a.projectile_speed) * hit.direction[1],
                };
                let shot = Aabb {
                    position: before,
                    size: Vec2::pixels(4, 4),
                }
                .swept_bounds(projectile);
                if a.projectile_speed > 0 && !self.line_open(before, projectile, hit.plane) {
                    continue;
                }
                let targets: Vec<_> = self
                    .entities
                    .iter()
                    .filter(|(id, e)| {
                        *id != &hit.owner
                            && !hit.hit.contains(*id)
                            && e.actor.hp.is_some_and(|hp| hp > 0)
                            && self.actor_plane(id) == hit.plane
                            && e.spec
                                .flatland
                                .as_ref()
                                .is_none_or(|other| other.team != team || team.is_empty())
                    })
                    .filter(|(id, e)| {
                        if a.projectile_speed > 0 {
                            shot.overlaps(e.aabb())
                        } else {
                            (self.entities[&hit.owner].aabb().overlaps(e.aabb())
                                || self.in_front(&hit.owner, id, a.range).unwrap_or(false))
                                && self.line_open(
                                    self.entities[&hit.owner].position,
                                    e.position,
                                    hit.plane,
                                )
                        }
                    })
                    .map(|(id, _)| id.clone())
                    .collect();
                for id in targets {
                    hit.hit.insert(id.clone());
                    let defense = self.entities[&id]
                        .spec
                        .flatland
                        .as_ref()
                        .map_or(0, |a| a.defense);
                    let attack_bonus = self.equipment_bonus(&hit.owner, true);
                    let defense_bonus = self.equipment_bonus(&id, false);
                    let damage = (a.damage + attack_bonus - defense - defense_bonus).max(1);
                    let old = self.entities[&id].actor.hp;
                    self.actions(
                        &[Action::Damage {
                            entity: id.clone(),
                            amount: damage,
                            immunity: a.immunity,
                        }],
                        Some(&id),
                    )?;
                    if old != self.entities.get(&id).and_then(|e| e.actor.hp) {
                        for (axis, v) in [(true, hit.direction[0]), (false, hit.direction[1])] {
                            self.move_axis(&id, axis, v * i64::from(a.knockback) * SUBPIXELS)?;
                        }
                        self.actions(&a.on_hit, Some(&id))?;
                        self.flat_event("hit", Some(&id))?;
                    }
                    if a.projectile_speed > 0 {
                        break;
                    }
                }
            }
            keep.push(hit);
        }
        if let Some(s) = &mut self.flatland.systems {
            s.attacks = keep;
        }
        Ok(())
    }
    fn systems_if_present_attacks(&mut self) -> Vec<AttackInstance> {
        self.flatland
            .systems
            .as_mut()
            .map(|s| std::mem::take(&mut s.attacks))
            .unwrap_or_default()
    }
    fn point_open(&self, pos: Vec2, plane: i32) -> bool {
        self.effective_map(plane).is_none_or(|m| {
            let c = i64::from(m.cell) * SUBPIXELS;
            m.open(pos.x.div_euclid(c), pos.y.div_euclid(c))
        })
    }
    pub(super) fn begin_event(&mut self, event: &str) -> Result<()> {
        if !self.project.scenes[&self.scene]
            .gameplay
            .events
            .contains_key(event)
        {
            return Err(Error("unknown event scene".into()));
        }
        if self.systems().events.len() >= 8 {
            return Err(Error("event stack depth exceeds 8".into()));
        }
        let parent = self.entities.clone();
        let camera = self.systems().camera;
        let planes = self.systems().planes.clone();
        let elevation = self.systems().elevation.clone();
        let animations = self.systems().animations.clone();
        let s = self.systems();
        s.event_serial += 1;
        let serial = s.event_serial;
        let parent_view = self.systems().view.clone();
        let parent_pace = self.systems().pace.clone();
        self.systems().events.push(EventFrame {
            serial,
            event: event.into(),
            pc: 0,
            wait: 0,
            waiting: false,
            battle: None,
            parent,
            parent_view,
            parent_pace,
            parent_camera: camera,
            parent_planes: planes,
            parent_elevation: elevation,
            parent_animations: animations,
        });
        self.release_inputs();
        self.flatland.popup = None;
        self.emit("event_enter", None, json!({"event":event}));
        Ok(())
    }
    pub fn waiting(&self) -> Option<Value> {
        let frame = self.flatland.systems.as_ref()?.events.last()?;
        if !frame.waiting {
            return None;
        }
        let id = format!(
            "{}:{}:{}:{}",
            frame.event,
            frame.serial,
            frame.pc,
            frame.battle.as_ref().map_or(0, |b| b.turn)
        );
        if let Some(b) = &frame.battle {
            let definitions =
                match &self.project.scenes[&self.scene].gameplay.events[&frame.event][frame.pc] {
                    Instruction::Battle { fighters, .. } => fighters,
                    _ => unreachable!(),
                };
            let hero = b.fighters.iter().find(|f| !f.enemy).unwrap();
            let mut options = Vec::new();
            if b.result.is_some() {
                options.push(
                    json!({"id":"battle_continue","text":"마을로 돌아가기","group":"result"}),
                );
            } else {
                for enemy in b.fighters.iter().filter(|f| f.enemy && f.hp > 0) {
                    if hero.moves.is_empty() {
                        options.push(json!({"id":format!("attack_{}",enemy.id),"text":"공격","target":enemy.id,"group":"moves"}));
                    } else {
                        for m in hero.moves.iter().filter(|m| m.pp > 0) {
                            options.push(json!({"id":format!("move:{}:{}",m.id,enemy.id),"text":m.name,"pp":m.pp,"target":enemy.id,"group":"moves"}));
                        }
                    }
                }
                options.push(json!({"id":"guard","text":"방어","group":"moves"}));
                options.extend(self.inventory_battle_choices());
            }
            let fighters: Vec<_> = b
                .fighters
                .iter()
                .map(|f| {
                    let mut v = serde_json::to_value(f).unwrap();
                    v["hp_max"] = json!(definitions.iter().find(|d| d.id == f.id).unwrap().hp);
                    v
                })
                .collect();
            return Some(
                json!({"id":id,"kind":"battle","turn":b.turn+1,"text":b.log,"options":options,"fighters":fighters,"result":b.result,"animation_ticks":self.tick.saturating_sub(b.turn_tick)}),
            );
        }
        match &self.project.scenes[&self.scene].gameplay.events[&frame.event][frame.pc] {
            Instruction::Say { text } => Some(
                json!({"id":id,"kind":"dialogue","text":text,"options":[{"id":"continue","text":"계속 / CONTINUE"}]}),
            ),
            Instruction::Choice { text, options } => Some(
                json!({"id":id,"kind":"choice","text":text,"options":options.iter().filter(|c|self.condition(&c.when,None)).map(|c|json!({"id":c.id,"text":c.text})).collect::<Vec<_>>()}),
            ),
            _ => None,
        }
    }
    fn inventory_battle_choices(&self) -> Vec<Value> {
        self.project.scenes[&self.scene].gameplay.items.iter().filter(|(id,i)|self.inventory(id)>0&&!i.use_actions.is_empty()).map(|(id,i)|json!({"id":format!("item_{id}"),"text":format!("{} ×{}",i.name,self.inventory(id)),"group":"items"})).collect()
    }
    pub fn choose(&mut self, choice: &str) -> Result<()> {
        if choice.len() > 128 {
            return Err(Error("choice ID exceeds 128".into()));
        }
        let backup = (
            self.tick,
            self.entities.clone(),
            self.state.clone(),
            self.flatland.clone(),
            self.events.clone(),
            self.events_dropped,
        );
        let result = self.choose_inner(choice);
        if result.is_err() {
            (
                self.tick,
                self.entities,
                self.state,
                self.flatland,
                self.events,
                self.events_dropped,
            ) = backup;
        }
        if result.is_ok() {
            self.release_inputs();
        }
        result
    }
    fn choose_inner(&mut self, choice: &str) -> Result<()> {
        self.command_budget = 0;
        let frame = self
            .flatland
            .systems
            .as_ref()
            .and_then(|s| s.events.last())
            .cloned()
            .ok_or_else(|| Error("no event choice".into()))?;
        if !frame.waiting {
            return Err(Error("event is not waiting".into()));
        }
        if frame.battle.is_some() {
            self.battle_choice(choice)?;
        } else {
            let step =
                self.project.scenes[&self.scene].gameplay.events[&frame.event][frame.pc].clone();
            let next = match step {
                Instruction::Say { .. } if choice == "continue" => frame.pc + 1,
                Instruction::Choice { options, .. } => {
                    let c = options
                        .iter()
                        .find(|c| c.id == choice && self.condition(&c.when, None))
                        .ok_or_else(|| Error("unknown/disabled choice".into()))?;
                    self.actions(&c.actions, None)?;
                    c.next
                }
                _ => return Err(Error("invalid choice".into())),
            };
            let frame = self.systems().events.last_mut().unwrap();
            frame.pc = next;
            frame.waiting = false;
        }
        self.emit("choice", None, json!({"id":choice}));
        self.event_tick()?;
        Ok(())
    }
    pub fn skip_event_wait(&mut self) -> Result<()> {
        let before = self.clone();
        let result = self.skip_wait_inner();
        if result.is_err() {
            *self = before;
        }
        result
    }
    fn skip_wait_inner(&mut self) -> Result<()> {
        if let Some(frame) = self.systems().events.last_mut() {
            frame.wait = 0;
        } else {
            return Err(Error("no event to skip".into()));
        }
        self.event_tick()
    }
    pub(super) fn event_tick(&mut self) -> Result<()> {
        for _ in 0..128 {
            let Some(frame) = self
                .flatland
                .systems
                .as_ref()
                .and_then(|s| s.events.last())
                .cloned()
            else {
                return Ok(());
            };
            if frame.waiting || frame.wait > self.tick {
                return Ok(());
            }
            let instructions = &self.project.scenes[&self.scene].gameplay.events[&frame.event];
            let step = instructions
                .get(frame.pc)
                .cloned()
                .unwrap_or(Instruction::Return { retain_view: false });
            match step {
                Instruction::Return { retain_view } => {
                    let frame = self.systems().events.pop().unwrap();
                    self.entities = frame.parent;
                    self.systems().camera = frame.parent_camera;
                    if !retain_view {
                        self.systems().view = frame.parent_view;
                    }
                    self.systems().pace = frame.parent_pace;
                    if !retain_view {
                        self.systems().planes = frame.parent_planes;
                        self.systems().elevation = frame.parent_elevation;
                    }
                    self.systems().animations = frame.parent_animations;
                    self.flatland.popup = None;
                    self.emit("event_return", None, json!({"event":frame.event}));
                }
                Instruction::Say { .. } | Instruction::Choice { .. } => {
                    self.systems().events.last_mut().unwrap().waiting = true;
                    if self
                        .waiting()
                        .is_none_or(|w| w["options"].as_array().is_none_or(|a| a.is_empty()))
                    {
                        return Err(Error("event has no available choice/fallback".into()));
                    }
                    return Ok(());
                }
                Instruction::Battle { fighters, .. } => {
                    let tick = self.tick;
                    let f = self.systems().events.last_mut().unwrap();
                    f.battle = Some(BattleState {
                        fighters,
                        turn: 0,
                        log: "전투 시작".into(),
                        turn_tick: tick,
                        previous_hp: BTreeMap::new(),
                        result: None,
                    });
                    f.waiting = true;
                    return Ok(());
                }
                Instruction::Wait { ticks } => {
                    let until = self.tick + u64::from(ticks);
                    let f = self.systems().events.last_mut().unwrap();
                    f.pc += 1;
                    f.wait = until;
                    if ticks > 0 {
                        return Ok(());
                    }
                }
                Instruction::Do { actions } => {
                    self.systems().events.last_mut().unwrap().pc += 1;
                    self.actions(&actions, None)?;
                }
                Instruction::Camera { at } => {
                    self.systems().camera = Some(at);
                    self.systems().events.last_mut().unwrap().pc += 1;
                }
                Instruction::Branch { when, yes, no } => {
                    let next = if self.condition(&when, None) { yes } else { no };
                    self.systems().events.last_mut().unwrap().pc = next;
                }
                Instruction::Jump { next } => self.systems().events.last_mut().unwrap().pc = next,
                Instruction::Call { event } => {
                    self.systems().events.last_mut().unwrap().pc += 1;
                    self.begin_event(&event)?;
                }
            }
        }
        Err(Error(
            "event instruction budget exceeds 128; possible jump cycle".into(),
        ))
    }
    fn battle_choice(&mut self, choice: &str) -> Result<()> {
        let frame = self.systems().events.last().unwrap().clone();
        let mut battle = frame.battle.unwrap();
        if let Some(won) = battle.result {
            if choice != "battle_continue" {
                return Err(Error("battle ended".into()));
            }
            return self.finish_battle(won);
        }
        battle.previous_hp = battle
            .fighters
            .iter()
            .map(|f| (f.id.clone(), f.hp))
            .collect();
        battle.turn_tick = self.tick;
        let hero = battle.fighters.iter().position(|f| !f.enemy).unwrap();
        let selected_move =
            battle.fighters[hero]
                .moves
                .iter()
                .enumerate()
                .find_map(|(index, m)| {
                    battle
                        .fighters
                        .iter()
                        .find(|f| {
                            f.enemy
                                && f.hp > 0
                                && choice == format!("move:{}:{}", m.id, f.id)
                                && m.pp > 0
                        })
                        .map(|f| (index, f.id.clone()))
                });
        let move_power = selected_move
            .as_ref()
            .map_or(100, |(index, _)| battle.fighters[hero].moves[*index].power);
        let move_name = selected_move
            .as_ref()
            .map_or("공격".to_owned(), |(index, _)| {
                battle.fighters[hero].moves[*index].name.clone()
            });
        let move_target = selected_move.as_ref().map(|(_, id)| id.as_str());
        let legacy_target = if battle.fighters[hero].moves.is_empty() {
            choice.strip_prefix("attack_")
        } else {
            None
        };
        let target = if let Some(id) = legacy_target.or(move_target) {
            Some(
                battle
                    .fighters
                    .iter()
                    .position(|f| f.id == id && f.enemy && f.hp > 0)
                    .ok_or_else(|| Error("invalid battle target".into()))?,
            )
        } else {
            None
        };
        let item = choice.strip_prefix("item_");
        if target.is_none() && choice != "guard" && item.is_none() {
            return Err(Error("unknown battle action".into()));
        }
        if let Some(item) = item
            && self.inventory(item) < 1
        {
            return Err(Error("battle item missing".into()));
        }
        let mut order: Vec<usize> = (0..battle.fighters.len()).collect();
        order.sort_by_key(|i| (-battle.fighters[*i].speed, battle.fighters[*i].id.clone()));
        let mut log = Vec::new();
        for i in order {
            if battle.fighters[i].hp == 0 {
                continue;
            }
            if i == hero {
                if let Some(item) = item {
                    let definition = self.project.scenes[&self.scene]
                        .gameplay
                        .items
                        .get(item)
                        .cloned()
                        .ok_or_else(|| Error("battle item undefined".into()))?;
                    let healing = definition
                        .use_actions
                        .iter()
                        .filter_map(|a| {
                            if let Action::Heal { amount, .. } = a {
                                Some(*amount)
                            } else {
                                None
                            }
                        })
                        .sum::<i64>();
                    if healing <= 0 {
                        return Err(Error("battle item needs a heal effect".into()));
                    }
                    self.actions(
                        &[Action::Take {
                            item: item.into(),
                            count: 1,
                        }],
                        None,
                    )?;
                    let max = match &self.project.scenes[&self.scene].gameplay.events[&frame.event]
                        [frame.pc]
                    {
                        Instruction::Battle { fighters, .. } => fighters[hero].hp,
                        _ => unreachable!(),
                    };
                    battle.fighters[hero].hp = (battle.fighters[hero].hp + healing).min(max);
                    log.push(format!("{}: +{} HP", definition.name, healing));
                } else if let Some(t) = target
                    && battle.fighters[t].hp > 0
                {
                    if let Some((index, _)) = &selected_move {
                        battle.fighters[hero].moves[*index].pp -= 1;
                    }
                    let damage = (battle.fighters[i].attack * move_power / 100
                        - battle.fighters[t].defense
                        + self.random(0, 2)?)
                    .max(1);
                    battle.fighters[t].hp = (battle.fighters[t].hp - damage).max(0);
                    log.push(format!(
                        "{}의 {} → {}: {} 피해",
                        battle.fighters[i].name, move_name, battle.fighters[t].name, damage
                    ));
                }
            } else if battle.fighters[hero].hp > 0 {
                let damage = (battle.fighters[i].attack - battle.fighters[hero].defense
                    + self.random(0, 2)?)
                .max(1);
                let damage = if choice == "guard" {
                    (damage / 2).max(1)
                } else {
                    damage
                };
                battle.fighters[hero].hp = (battle.fighters[hero].hp - damage).max(0);
                log.push(format!(
                    "{} → {}: {}",
                    battle.fighters[i].name, battle.fighters[hero].name, damage
                ));
            }
        }
        battle.turn += 1;
        battle.log = log.join("\n");
        let won = !battle.fighters.iter().any(|f| f.enemy && f.hp > 0);
        let lost = battle.fighters[hero].hp == 0;
        self.emit(
            "battle_turn",
            None,
            json!({"turn":battle.turn,"fighters":battle.fighters,"log":battle.log}),
        );
        if won || lost {
            let hold = matches!(&self.project.scenes[&self.scene].gameplay.events[&frame.event][frame.pc],Instruction::Battle{stage:Some(stage),..} if stage.hold_result);
            if hold {
                battle.result = Some(won);
                battle.log = if won {
                    "승리! 실험을 완료했습니다.".into()
                } else {
                    "패배했습니다. 다시 도전할 수 있습니다.".into()
                };
                self.systems().events.last_mut().unwrap().battle = Some(battle);
            } else {
                self.finish_battle(won)?;
            }
        } else {
            self.systems().events.last_mut().unwrap().battle = Some(battle);
        }
        Ok(())
    }
    fn finish_battle(&mut self, won: bool) -> Result<()> {
        let frame = self.systems().events.last().unwrap().clone();
        let effects =
            match &self.project.scenes[&self.scene].gameplay.events[&frame.event][frame.pc] {
                Instruction::Battle {
                    victory, defeat, ..
                } => {
                    if won {
                        victory.clone()
                    } else {
                        defeat.clone()
                    }
                }
                _ => unreachable!(),
            };
        let f = self.systems().events.last_mut().unwrap();
        f.battle = None;
        f.waiting = false;
        f.pc += 1;
        self.actions(&effects, None)?;
        self.emit("battle_result", None, json!({"victory":won}));
        self.flat_event(if won { "battle_win" } else { "battle_loss" }, None)
    }
    pub(super) fn freeze_world_tick(&mut self) {
        for end in self.flatland.timers.values_mut() {
            *end += 1;
        }
        for e in self.entities.values_mut() {
            if e.actor.immune_until > 0 {
                e.actor.immune_until += 1;
            }
        }
        if let Some(s) = &mut self.flatland.systems {
            s.paused_ticks += 1;
            for a in &mut s.attacks {
                a.start += 1;
            }
            for t in s.cooldowns.values_mut() {
                *t += 1;
            }
            let current = s.events.last().map_or(0, |f| f.serial);
            for a in s.animations.values_mut() {
                if a.event_serial != current {
                    a.start += 1;
                }
            }
            for f in &mut s.events {
                for e in f.parent.values_mut() {
                    if e.actor.immune_until > 0 {
                        e.actor.immune_until += 1;
                    }
                }
                for a in f.parent_animations.values_mut() {
                    a.start += 1;
                }
            }
        }
    }
    pub fn command(&mut self, actions: &[Action]) -> Result<()> {
        let backup = (
            self.tick,
            self.scene.clone(),
            self.entities.clone(),
            self.state.clone(),
            self.flatland.clone(),
            self.events.clone(),
            self.events_dropped,
            self.triggers.clone(),
            self.held_actions.clone(),
            self.interact_held,
        );
        self.command_budget = 0;
        let result = self
            .actions(actions, None)
            .and_then(|_| self.death_phase())
            .and_then(|_| self.event_tick());
        if result.is_err() {
            (
                self.tick,
                self.scene,
                self.entities,
                self.state,
                self.flatland,
                self.events,
                self.events_dropped,
                self.triggers,
                self.held_actions,
                self.interact_held,
            ) = backup;
        }
        result
    }
    pub(super) fn death_phase(&mut self) -> Result<()> {
        let ids = self
            .flatland
            .systems
            .as_mut()
            .map(|s| std::mem::take(&mut s.pending_deaths))
            .unwrap_or_default();
        for id in ids {
            if let Some(e) = self.entities.get(&id).cloned() {
                if e.actor.hp != Some(0) {
                    continue;
                }
                let drops = e
                    .spec
                    .flatland
                    .as_ref()
                    .map(|a| a.drops.clone())
                    .unwrap_or_default();
                self.flat_event("death", Some(&id))?;
                self.actions(&drops, Some(&id))?;
                self.emit("death", Some(&id), json!({}));
                if e.spec.player.is_none() {
                    self.entities.remove(&id);
                }
            }
        }
        Ok(())
    }
    pub(super) fn animation_markers(&mut self) -> Result<()> {
        let clips: Vec<_> = self
            .entities
            .iter()
            .filter_map(|(id, e)| {
                e.spec
                    .flatland
                    .as_ref()
                    .and_then(|a| a.animation.as_ref())
                    .map(|a| (id.clone(), a.clone()))
            })
            .collect();
        let phase = self
            .tick
            .saturating_sub(self.flatland.systems.as_ref().map_or(0, |s| s.paused_ticks));
        for (id, a) in clips {
            if a.ticks == 0 || !phase.is_multiple_of(u64::from(a.ticks)) {
                continue;
            }
            let n = a.atlas.as_ref().map_or(a.frames.len(), |a| a.indices.len());
            let index = phase / u64::from(a.ticks);
            if a.once && index >= n as u64 {
                continue;
            }
            if let Some(actions) = a.markers.get(&((index as usize % n) as u32)) {
                self.actions(actions, Some(&id))?;
                self.emit(
                    "animation_marker",
                    Some(&id),
                    json!({"frame":index as usize%n}),
                );
            }
        }
        Ok(())
    }
    pub(super) fn camera_position(&self) -> [i64; 2] {
        if let Some(at) = self.flatland.systems.as_ref().and_then(|s| s.camera) {
            return at;
        }
        if let Some(c) = &self.project.scenes[&self.scene].gameplay.camera
            && let Some(e) = self.entities.get(&c.target)
        {
            return [
                (e.position.x / SUBPIXELS - i64::from(self.project.manifest.window.width) / 2)
                    .clamp(c.bounds[0], c.bounds[2]),
                (e.position.y / SUBPIXELS - i64::from(self.project.manifest.window.height) / 2)
                    .clamp(c.bounds[1], c.bounds[3]),
            ];
        }
        self.project.scenes[&self.scene].camera
    }
}
impl PartialEq for Systems {
    fn eq(&self, other: &Self) -> bool {
        serde_json::to_value(self).expect("system JSON")
            == serde_json::to_value(other).expect("system JSON")
    }
}
impl Eq for Systems {}
impl World {
    pub(super) fn validate_systems(&self) -> Result<()> {
        let Some(s) = &self.flatland.systems else {
            return Ok(());
        };
        let bad = || Error("invalid saved FlatLand systems".into());
        let g = &self.project.scenes[&self.scene].gameplay;
        if s.view.as_ref().is_some_and(|id| !g.views.contains_key(id))
            || s.pace
                .iter()
                .any(|(id, speed)| !self.entities.contains_key(id) || !(12..=6000).contains(speed))
        {
            return Err(bad());
        }
        if s.rng == 0
            || s.paused_ticks > self.tick
            || s.events.len() > 8
            || s.event_serial > 1_000_000
            || s.attacks.len() > 128
        {
            return Err(bad());
        }
        let item = |id: &str| {
            self.project
                .scenes
                .values()
                .find_map(|scene| scene.gameplay.items.get(id))
        };
        let quest = |id: &str| {
            self.project
                .scenes
                .values()
                .find_map(|scene| scene.gameplay.quests.get(id))
        };
        for (id, n) in &s.inventory {
            if item(id).is_none_or(|i| *n < 0 || *n > i.stack) {
                return Err(bad());
            }
        }
        for (slot, id) in &s.equipment {
            if self.inventory(id) < 1 || item(id).is_none_or(|i| i.slot.as_ref() != Some(slot)) {
                return Err(bad());
            }
        }
        for (id, q) in &s.quests {
            let definition = quest(id).ok_or_else(bad)?;
            if q.rewarded != (q.status == QuestStatus::Completed)
                || q.objectives.iter().any(|(id, n)| {
                    definition
                        .objectives
                        .get(id)
                        .is_none_or(|max| *n < 0 || n > max)
                })
                || q.status == QuestStatus::Completed
                    && definition
                        .objectives
                        .iter()
                        .any(|(id, n)| q.objectives.get(id).unwrap_or(&0) < n)
            {
                return Err(bad());
            }
        }
        let authored = |id: &str| {
            self.project.scenes[&self.scene]
                .entities
                .iter()
                .any(|e| e.id == id)
        };
        if s.planes
            .keys()
            .chain(s.elevation.keys())
            .chain(s.animations.keys())
            .chain(s.cooldowns.keys())
            .chain(s.patrol.keys())
            .any(|id| !authored(id))
            || s.elevation.values().any(|z| z.unsigned_abs() > 4096)
            || s.cooldowns.values().any(|end| *end > self.tick + 1_000_000)
        {
            return Err(bad());
        }
        for a in s.animations.values() {
            if a.frames.is_empty()
                || a.frames.len() > 128
                || a.frames
                    .iter()
                    .any(|f| !self.project.textures.contains_key(f))
                || a.ticks == 0
                || a.start > self.tick
                || a.event_serial > s.event_serial
            {
                return Err(bad());
            }
        }
        for a in &s.attacks {
            if !authored(&a.owner)
                || !g.attacks.contains_key(&a.preset)
                || a.start > self.tick
                || a.hit.iter().any(|id| !authored(id))
                || a.direction.iter().any(|v| !(-1..=1).contains(v))
                || a.direction == [0, 0]
                || [a.origin.x, a.origin.y]
                    .iter()
                    .any(|v| v.unsigned_abs() > 60_000_000)
            {
                return Err(bad());
            }
        }
        for (key, tile) in &s.patches {
            let parts: Vec<_> = key.split(':').collect();
            if parts.len() != 3 {
                return Err(bad());
            }
            let map = self
                .project
                .scenes
                .get(parts[0])
                .and_then(|scene| scene.map.as_ref())
                .ok_or_else(bad)?;
            let x = parts[1].parse::<usize>().map_err(|_| bad())?;
            let y = parts[2].parse::<usize>().map_err(|_| bad())?;
            if y >= map.rows.len() || x >= map.rows[y].len() || !map.tiles.contains_key(tile) {
                return Err(bad());
            }
        }
        if s.camera
            .is_some_and(|at| at.iter().any(|n| n.unsigned_abs() > 1_000_000))
        {
            return Err(bad());
        }
        if let Some(m) = &s.music
            && (m.volume > 100
                || !self
                    .project
                    .scenes
                    .values()
                    .any(|scene| scene.sounds.get(&m.cue) == Some(&m.file)))
        {
            return Err(bad());
        }
        let mut previous = 0;
        for f in &s.events {
            let steps = g.events.get(&f.event).ok_or_else(bad)?;
            if f.serial <= previous
                || f.serial > s.event_serial
                || f.pc > steps.len()
                || f.wait > self.tick + 10000
            {
                return Err(bad());
            }
            previous = f.serial;
            self.validate_saved_actors(&f.parent)?;
            if f.parent_view
                .as_ref()
                .is_some_and(|id| !g.views.contains_key(id))
                || f.parent_pace
                    .iter()
                    .any(|(id, speed)| !f.parent.contains_key(id) || !(12..=6000).contains(speed))
                || f.parent_planes
                    .keys()
                    .chain(f.parent_elevation.keys())
                    .chain(f.parent_animations.keys())
                    .any(|id| !f.parent.contains_key(id))
                || f.parent_elevation.values().any(|z| z.unsigned_abs() > 4096)
                || f.parent_camera
                    .is_some_and(|at| at.iter().any(|v| v.unsigned_abs() > 1_000_000))
            {
                return Err(bad());
            }
            for a in f.parent_animations.values() {
                if a.frames.is_empty()
                    || a.ticks == 0
                    || a.start > self.tick
                    || a.event_serial > f.serial
                    || a.frames
                        .iter()
                        .any(|file| !self.project.textures.contains_key(file))
                {
                    return Err(bad());
                }
            }
            let mut parent_world = self.clone();
            parent_world.entities = f.parent.clone();
            if let Some(parent_systems) = &mut parent_world.flatland.systems {
                parent_systems.planes = f.parent_planes.clone();
            }
            if parent_world.entities.iter().any(|(id, e)| {
                (e.spec.player.is_some()
                    || e.spec
                        .flatland
                        .as_ref()
                        .is_some_and(|a| a.body != BodyMode::Pass))
                    && !parent_world.actor_clear(id, e.position, parent_world.actor_plane(id))
            }) {
                return Err(bad());
            }
            if f.waiting
                && !matches!(
                    steps.get(f.pc),
                    Some(
                        Instruction::Say { .. }
                            | Instruction::Choice { .. }
                            | Instruction::Battle { .. }
                    )
                )
            {
                return Err(bad());
            }
            if let Some(b) = &f.battle {
                let Some(Instruction::Battle { fighters, .. }) = steps.get(f.pc) else {
                    return Err(bad());
                };
                if !f.waiting
                    || b.fighters.len() != fighters.len()
                    || b.turn > 1_000_000
                    || b.log.len() > 8192
                    || b.turn_tick > self.tick
                    || b.previous_hp.iter().any(|(id, hp)| {
                        fighters
                            .iter()
                            .find(|f| &f.id == id)
                            .is_none_or(|f| *hp < 0 || *hp > f.hp)
                    })
                {
                    return Err(bad());
                }
                let hero_alive = b.fighters.iter().any(|f| !f.enemy && f.hp > 0);
                let enemy_alive = b.fighters.iter().any(|f| f.enemy && f.hp > 0);
                if b.result
                    .is_some_and(|won| won != (hero_alive && !enemy_alive))
                    || (b.result.is_some() && hero_alive && enemy_alive)
                {
                    return Err(bad());
                }
                for (actual, definition) in b.fighters.iter().zip(fighters) {
                    let mut expected = definition.clone();
                    if actual.hp < 0 || actual.hp > definition.hp {
                        return Err(bad());
                    }
                    expected.hp = actual.hp;
                    if actual.moves.len() != definition.moves.len() {
                        return Err(bad());
                    }
                    for (actual_move, expected_move) in actual.moves.iter().zip(&mut expected.moves)
                    {
                        if actual_move.pp > expected_move.pp {
                            return Err(bad());
                        }
                        expected_move.pp = actual_move.pp;
                    }
                    if serde_json::to_value(actual).unwrap()
                        != serde_json::to_value(expected).unwrap()
                    {
                        return Err(bad());
                    }
                }
            }
        }
        Ok(())
    }
    fn validate_saved_actors(&self, actors: &BTreeMap<String, LiveEntity>) -> Result<()> {
        let fail = || Error("invalid saved event parent actor".into());
        if actors.values().filter(|e| e.spec.player.is_some()).count() != 1 {
            return Err(fail());
        }
        for (id, e) in actors {
            let spec = self.project.scenes[&self.scene]
                .entities
                .iter()
                .find(|s| &s.id == id)
                .ok_or_else(fail)?;
            let max = spec.flatland.as_ref().and_then(|a| a.hp);
            if e.spec.id != *id
                || serde_json::to_value(spec).unwrap() != serde_json::to_value(&e.spec).unwrap()
                || e.actor.hp.is_some() != max.is_some()
                || e.actor.hp.is_some_and(|hp| hp < 0 || hp > max.unwrap_or(0))
                || [e.position.x, e.position.y]
                    .iter()
                    .any(|v| v.unsigned_abs() > 60_000_000)
                || e.actor
                    .facing
                    .iter()
                    .chain(e.actor.direction.iter())
                    .chain(e.actor.queued.iter())
                    .any(|v| !(-1..=1).contains(v))
                || e.actor.immune_until > self.tick + 1_000_000
            {
                return Err(fail());
            }
        }
        Ok(())
    }
}

impl World {
    fn portal_direction(
        &mut self,
        id: &str,
        at: [i64; 2],
        target: [i64; 2],
        target_plane: i32,
    ) -> Result<[i64; 2]> {
        let plane = self.actor_plane(id);
        let map = self.patched_map().unwrap();
        let start = [at[0], at[1], i64::from(plane)];
        let goal = [target[0], target[1], i64::from(target_plane)];
        let mut queue = VecDeque::from([start]);
        let mut previous = BTreeMap::from([(start, start)]);
        let mut found = false;
        while let Some(node) = queue.pop_front() {
            if node == goal {
                found = true;
                break;
            }
            if previous.len() >= 4096 {
                break;
            }
            let mut neighbors = Vec::new();
            for d in [[0, -1], [-1, 0], [0, 1], [1, 0]] {
                let n = [node[0] + d[0], node[1] + d[1], node[2]];
                if map.open_on_plane(n[0], n[1], n[2] as i32) {
                    neighbors.push(n);
                }
            }
            for p in &map.portals {
                if p.from == node[2] as i32 && p.at == [node[0], node[1]] {
                    neighbors.push([p.destination[0], p.destination[1], i64::from(p.to)]);
                }
            }
            for n in neighbors {
                if let std::collections::btree_map::Entry::Vacant(e) = previous.entry(n) {
                    e.insert(node);
                    queue.push_back(n);
                }
            }
        }
        if !found {
            return Ok([0, 0]);
        }
        let mut next = goal;
        while previous[&next] != start {
            next = previous[&next];
        }
        if next[2] != start[2] {
            let cell = i64::from(map.cell);
            let pos = Vec2::pixels(next[0] * cell, next[1] * cell);
            if !self.actor_clear(id, pos, next[2] as i32) {
                return Ok([0, 0]);
            }
            self.entities.get_mut(id).unwrap().position = pos;
            let s = self.systems();
            s.planes.insert(id.into(), next[2] as i32);
            s.portal_latches.insert(id.into(), next);
            self.emit(
                "portal",
                Some(id),
                json!({"plane":next[2],"at":[next[0],next[1]]}),
            );
            return Ok([0, 0]);
        }
        Ok([next[0] - start[0], next[1] - start[1]])
    }
    pub(super) fn portal_tick(&mut self) -> Result<()> {
        let Some(map) = self.project.scenes[&self.scene].map.clone() else {
            return Ok(());
        };
        if map.portals.is_empty() {
            return Ok(());
        }
        let cell = i64::from(map.cell) * SUBPIXELS;
        let ids: Vec<_> = self
            .entities
            .iter()
            .filter(|(_, e)| {
                e.spec.player.is_some() || e.spec.flatland.as_ref().is_some_and(|a| a.ai.is_some())
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in ids {
            let pos = self.entities[&id].position;
            if pos.x % cell != 0 || pos.y % cell != 0 {
                continue;
            }
            let plane = self.actor_plane(&id);
            let node = [pos.x / cell, pos.y / cell, i64::from(plane)];
            if self
                .flatland
                .systems
                .as_ref()
                .and_then(|s| s.portal_latches.get(&id))
                == Some(&node)
            {
                continue;
            }
            if let Some(s) = &mut self.flatland.systems {
                s.portal_latches.remove(&id);
            }
            if let Some(p) = map
                .portals
                .iter()
                .find(|p| p.from == plane && p.at == [node[0], node[1]])
            {
                let destination = Vec2::pixels(
                    p.destination[0] * i64::from(map.cell),
                    p.destination[1] * i64::from(map.cell),
                );
                if !self.actor_clear(&id, destination, p.to) {
                    continue;
                }
                self.entities.get_mut(&id).unwrap().position = destination;
                let s = self.systems();
                s.planes.insert(id.clone(), p.to);
                s.portal_latches.insert(
                    id.clone(),
                    [p.destination[0], p.destination[1], i64::from(p.to)],
                );
                self.emit("portal", Some(&id), json!({"portal":p.id,"plane":p.to}));
            }
        }
        Ok(())
    }
}

impl World {
    pub(super) fn projectile_snapshots(&self) -> Vec<EntitySnapshot> {
        let Some(s) = &self.flatland.systems else {
            return vec![];
        };
        s.attacks
            .iter()
            .enumerate()
            .filter_map(|(index, shot)| {
                let a = self.project.scenes[&self.scene]
                    .gameplay
                    .attacks
                    .get(&shot.preset)?;
                let age = self.tick.saturating_sub(shot.start);
                if a.projectile_speed == 0 || age < u64::from(a.startup) {
                    return None;
                }
                let travel =
                    age.saturating_sub(u64::from(a.startup)) as i64 * i64::from(a.projectile_speed);
                Some(EntitySnapshot {
                    id: format!("$projectile.{}.{}.{}", shot.owner, shot.start, index),
                    position: Vec2 {
                        x: shot.origin.x + travel * shot.direction[0],
                        y: shot.origin.y + travel * shot.direction[1],
                    },
                    size: [4, 4],
                    components: vec!["projectile".into()],
                    tags: vec![],
                    metadata: BTreeMap::new(),
                    color: Some([240, 255, 40, 255]),
                    texture: None,
                    layer: 1,
                    blocking: false,
                    trigger: false,
                    flatland: Some(json!({"plane":shot.plane,"facing":shot.direction})),
                })
            })
            .collect()
    }
}

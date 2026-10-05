//! One authoritative fixed-step simulation shared by every adapter.
mod flatland;
pub use flatland::Resume;
use ge4g_core::{
    Aabb, ENGINE_VERSION, EntitySnapshot, Error, Event, Input, Result, SUBPIXELS, Snapshot,
    StateStore, TICK_HZ, Vec2,
};
use ge4g_project::{Assertion, Entity, Project, Transition, atomic_json, read_text, version};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::Path,
};

pub const EVENT_LIMIT: usize = 4096;
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct LiveEntity {
    pub spec: Entity,
    pub position: Vec2,
    pub actor: flatland::ActorState,
}
impl LiveEntity {
    fn aabb(&self) -> Aabb {
        Aabb {
            position: self.position,
            size: Vec2::pixels(self.spec.size[0].into(), self.spec.size[1].into()),
        }
    }
    fn snapshot(&self) -> EntitySnapshot {
        let mut components = vec!["transform".into()];
        if self.spec.sprite.is_some() {
            components.push("sprite".into());
        }
        if self.spec.collider.is_some() {
            components.push("collider".into());
        }
        if self.spec.player.is_some() {
            components.push("player".into());
        }
        if self.spec.trigger.is_some() {
            components.push("trigger".into());
        }
        if self.spec.interaction.is_some() {
            components.push("interaction".into());
        }
        EntitySnapshot {
            id: self.spec.id.clone(),
            position: self.position,
            size: self.spec.size,
            components,
            tags: self.spec.tags.clone(),
            metadata: self.spec.metadata.clone(),
            color: self.spec.sprite.as_ref().map(|s| s.color),
            texture: self.spec.sprite.as_ref().and_then(|s| s.texture.clone()),
            layer: self.spec.sprite.as_ref().map_or(0, |s| s.layer),
            blocking: self.spec.collider.as_ref().is_some_and(|c| c.blocking),
            trigger: self.spec.trigger.is_some(),
            flatland: self
                .spec
                .flatland
                .as_ref()
                .map(|_| serde_json::to_value(&self.actor).expect("actor JSON")),
        }
    }
}
pub struct World {
    pub project: Project,
    pub tick: u64,
    pub scene: String,
    pub entities: BTreeMap<String, LiveEntity>,
    pub state: StateStore,
    events: VecDeque<Event>,
    pub events_dropped: u64,
    contacts: BTreeSet<String>,
    triggers: BTreeSet<String>,
    interact_held: bool,
    held_actions: BTreeSet<String>,
    pub flatland: flatland::FlatState,
    lua_budget: std::sync::Arc<std::sync::atomic::AtomicU32>,
    command_budget: usize,
}
impl World {
    pub fn new(project: Project) -> Result<Self> {
        Self::with_save(project, None)
    }
    pub fn with_save(project: Project, save: Option<&Path>) -> Result<Self> {
        let mut state = StateStore::new(project.manifest.state.clone())?;
        if let Some(path) = save.filter(|_| project.manifest.schema_version == 1) {
            load_save(path, &project.manifest.name, &mut state)?;
        }
        let initial = project.manifest.start_scene.clone();
        let mut world = Self {
            project,
            tick: 0,
            scene: initial.clone(),
            entities: BTreeMap::new(),
            state,
            events: VecDeque::new(),
            events_dropped: 0,
            contacts: BTreeSet::new(),
            triggers: BTreeSet::new(),
            interact_held: false,
            held_actions: BTreeSet::new(),
            flatland: flatland::FlatState::default(),
            lua_budget: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
            command_budget: 0,
        };
        if save.is_some() {
            world.emit(
                "save_loaded",
                None,
                json!({"project": world.project.manifest.name}),
            );
        }
        world.enter_scene(&initial, None)?;
        if world.project.manifest.schema_version == 2 {
            if let Some(path) = save {
                world.load_flatland(path)?;
            } else {
                world.flat_event("start", None)?;
            }
        }
        Ok(world)
    }
    fn emit(&mut self, kind: &str, entity: Option<&str>, data: Value) {
        if self.events.len() == EVENT_LIMIT {
            self.events.pop_front();
            self.events_dropped += 1;
        }
        self.events.push_back(Event {
            tick: self.tick,
            kind: kind.into(),
            scene: self.scene.clone(),
            entity: entity.map(str::to_owned),
            data,
        });
    }
    fn set_state(&mut self, writes: &BTreeMap<String, Value>, entity: Option<&str>) -> Result<()> {
        for (key, value) in writes {
            let before = self.state.values.get(key).cloned();
            if self.state.set(key, value.clone())? {
                self.emit(
                    "state_changed",
                    entity,
                    json!({"key": key, "before": before, "after": value}),
                );
            }
        }
        Ok(())
    }
    fn enter_scene(&mut self, scene: &str, spawn: Option<&str>) -> Result<()> {
        let spec = self
            .project
            .scenes
            .get(scene)
            .cloned()
            .ok_or_else(|| Error(format!("tick {}: unknown scene {scene}", self.tick)))?;
        let position = spawn
            .map(|name| {
                spec.spawns.get(name).copied().ok_or_else(|| {
                    Error(format!(
                        "tick {} scene {scene}: unknown spawn {name}",
                        self.tick
                    ))
                })
            })
            .transpose()?;
        self.scene = scene.to_owned();
        self.entities.clear();
        self.contacts.clear();
        self.triggers.clear();
        for mut entity in spec.entities {
            if entity.player.is_some()
                && let Some(pos) = position
            {
                entity.position = pos;
            }
            self.entities.insert(
                entity.id.clone(),
                LiveEntity {
                    position: Vec2::pixels(entity.position[0], entity.position[1]),
                    actor: flatland::ActorState::new(&entity),
                    spec: entity,
                },
            );
        }
        self.emit("scene_loaded", None, json!({"spawn": spawn}));
        let ids: Vec<String> = self.entities.keys().cloned().collect();
        for id in ids {
            self.emit(
                "entity_spawned",
                Some(&id),
                json!({"position": self.entities[&id].position}),
            );
        }
        self.set_state(&spec.on_enter, None)
    }
    fn transition(&mut self, transition: &Transition, entity: &str) -> Result<()> {
        self.emit(
            "scene_transition",
            Some(entity),
            json!({"from": self.scene, "to": transition.scene, "spawn": transition.spawn}),
        );
        self.enter_scene(&transition.scene, Some(&transition.spawn))
    }
    pub fn snapshot(&self) -> Snapshot {
        let scene = &self.project.scenes[&self.scene];
        Snapshot {
            schema_version: self.project.manifest.schema_version,
            engine_version: ENGINE_VERSION.into(),
            tick: self.tick,
            tick_hz: TICK_HZ,
            subpixels_per_pixel: SUBPIXELS,
            scene: self.scene.clone(),
            camera: scene.camera,
            background: scene.background,
            entities: self
                .entities
                .values()
                .map(|e| {
                    let mut snapshot = e.snapshot();
                    if let Some(a) = &e.spec.flatland {
                        snapshot.blocking = a.body != ge4g_project::flatland::BodyMode::Pass;
                        if let Some(v) = snapshot.flatland.as_mut().and_then(Value::as_object_mut) {
                            v.insert("depth".into(), json!(a.depth));
                            v.insert("body".into(), json!(a.body));
                            v.insert("plane".into(), json!(a.plane));
                            v.insert(
                                "rotate".into(),
                                json!(a.animation.as_ref().is_some_and(|a| a.rotate)),
                            );
                        }
                        if let Some(anim) = &a.animation {
                            let frames = if anim.alternate_timer.as_ref().is_some_and(|id| {
                                self.flatland
                                    .timers
                                    .get(id)
                                    .is_some_and(|end| *end > self.tick)
                            }) && !anim.alternate.is_empty()
                            {
                                &anim.alternate
                            } else {
                                let direction = match e.actor.facing {
                                    [x, _] if x > 0 => "right",
                                    [x, _] if x < 0 => "left",
                                    [_, y] if y < 0 => "up",
                                    _ => "down",
                                };
                                anim.directions.get(direction).unwrap_or(&anim.frames)
                            };
                            snapshot.texture = Some(
                                frames[(self.tick / u64::from(anim.ticks)) as usize % frames.len()]
                                    .clone(),
                            );
                        }
                    }
                    snapshot
                })
                .collect(),
            state: self.state.values.clone(),
            events: self.events.iter().cloned().collect(),
            events_dropped: self.events_dropped,
            flatland: (self.project.manifest.schema_version == 2)
                .then(|| serde_json::to_value(&self.flatland).expect("world JSON")),
        }
    }
    /// Versioned client adapters may also send named buttons. Standard Basement
    /// Input is unchanged; extra actions are observable and never mutate gameplay
    /// through a separate simulation implementation.
    pub fn step_actions(&mut self, input: &Input, actions: &BTreeSet<String>) -> Result<()> {
        if actions.len() > 32
            || actions.iter().any(|action| {
                action.is_empty()
                    || action.len() > 64
                    || !action
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
            })
        {
            return Err(Error(
                "client actions require at most 32 valid names of 1..64 ASCII characters".into(),
            ));
        }
        self.step(input)?;
        for action in self
            .held_actions
            .difference(actions)
            .cloned()
            .collect::<Vec<_>>()
        {
            self.emit("action_released", None, json!({"action": action}));
        }
        for action in actions
            .difference(&self.held_actions)
            .cloned()
            .collect::<Vec<_>>()
        {
            self.emit("action_pressed", None, json!({"action": action}));
        }
        self.held_actions = actions.clone();
        Ok(())
    }
    pub fn release_inputs(&mut self) {
        self.interact_held = false;
        for action in std::mem::take(&mut self.held_actions) {
            self.emit("action_released", None, json!({"action": action}));
        }
    }
    /// Advance exactly one tick; event timestamps describe the resulting tick.
    pub fn step(&mut self, input: &Input) -> Result<()> {
        if self.project.manifest.schema_version == 2 {
            return self.step_flatland(input);
        }
        if self.tick >= 1_000_000 {
            return Err(Error("runtime tick limit (1,000,000) reached".into()));
        }
        self.tick += 1;
        let id = self
            .entities
            .values()
            .find(|e| e.spec.player.is_some())
            .map(|e| e.spec.id.clone())
            .ok_or_else(|| {
                Error(format!(
                    "scene {} tick {} has no player",
                    self.scene, self.tick
                ))
            })?;
        let mut player = self.entities[&id].clone();
        let mut motion_path = vec![player.aabb()];
        let speed = player.spec.player.as_ref().unwrap().speed;
        let (dx, dy) = input.axes();
        let mut contacts = BTreeSet::new();
        for (x_axis, requested) in [(true, dx * speed), (false, dy * speed)] {
            let start = player.aabb();
            let mut allowed = requested;
            for wall in self
                .entities
                .values()
                .filter(|e| e.spec.id != id && e.spec.collider.as_ref().is_some_and(|c| c.blocking))
            {
                allowed = start.sweep_axis(wall.aabb(), allowed, x_axis);
            }
            if allowed != requested {
                for wall in self.entities.values().filter(|e| {
                    e.spec.id != id && e.spec.collider.as_ref().is_some_and(|c| c.blocking)
                }) {
                    if start.sweep_axis(wall.aabb(), requested, x_axis) == allowed {
                        contacts.insert(wall.spec.id.clone());
                    }
                }
            }
            if x_axis {
                player.position.x += allowed;
            } else {
                player.position.y += allowed;
            }
            motion_path.push(player.aabb());
        }
        for target in contacts
            .difference(&self.contacts)
            .cloned()
            .collect::<Vec<_>>()
        {
            self.emit("collision_started", Some(&id), json!({"target": target}));
        }
        for target in &contacts {
            self.emit(
                "collision_resolved",
                Some(&id),
                json!({"target": target, "position": player.position}),
            );
        }
        for target in self
            .contacts
            .difference(&contacts)
            .cloned()
            .collect::<Vec<_>>()
        {
            self.emit("collision_ended", Some(&id), json!({"target": target}));
        }
        self.contacts = contacts;
        self.entities.insert(id.clone(), player.clone());
        if input.interact && !self.interact_held {
            let candidates: Vec<LiveEntity> = self
                .entities
                .values()
                .filter(|e| e.spec.interaction.is_some())
                .cloned()
                .collect();
            for npc in candidates {
                let behavior = npc.spec.interaction.as_ref().unwrap();
                let range = i64::from(behavior.range) * SUBPIXELS;
                let near = Aabb {
                    position: Vec2 {
                        x: npc.position.x - range,
                        y: npc.position.y - range,
                    },
                    size: Vec2 {
                        x: npc.aabb().size.x + 2 * range,
                        y: npc.aabb().size.y + 2 * range,
                    },
                };
                if player.aabb().overlaps(near) {
                    self.emit(
                        "interaction",
                        Some(&id),
                        json!({"target": npc.spec.id, "dialogue": behavior.dialogue}),
                    );
                    self.set_state(&behavior.set_state, Some(&npc.spec.id))?;
                    self.emit("audio", Some(&npc.spec.id), json!({"cue": "interaction"}));
                    if let Some(t) = &behavior.transition {
                        self.transition(t, &id)?;
                        self.interact_held = input.interact;
                        return Ok(());
                    }
                    break; // One deterministic target: first eligible stable entity id.
                }
            }
        }
        self.interact_held = input.interact;
        let overlaps: BTreeSet<String> = self
            .entities
            .values()
            .filter(|e| {
                e.spec.trigger.is_some()
                    && motion_path.windows(2).any(|segment| {
                        segment[0]
                            .swept_bounds(segment[1].position)
                            .overlaps(e.aabb())
                    })
            })
            .map(|e| e.spec.id.clone())
            .collect();
        for target in self
            .triggers
            .difference(&overlaps)
            .cloned()
            .collect::<Vec<_>>()
        {
            self.emit("trigger_exited", Some(&id), json!({"target": target}));
        }
        if let Some(target) = overlaps.difference(&self.triggers).next().cloned() {
            self.emit("trigger_entered", Some(&id), json!({"target": target}));
            let transition = self.entities[&target]
                .spec
                .trigger
                .as_ref()
                .unwrap()
                .clone();
            self.transition(&transition, &id)?;
            return Ok(());
        }
        self.triggers = overlaps;
        Ok(())
    }
    pub fn save(&mut self, path: &Path) -> Result<()> {
        if self.project.manifest.schema_version == 2 {
            return self.save_flatland(path);
        }
        let save = Save {
            schema_version: 1,
            project: self.project.manifest.name.clone(),
            state: self.state.persistent_values(),
        };
        atomic_json(path, &save)?;
        self.emit(
            "save_written",
            None,
            json!({"path": path.to_string_lossy(), "keys": save.state.len()}),
        );
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Save {
    pub schema_version: u32,
    pub project: String,
    pub state: BTreeMap<String, Value>,
}
pub fn load_save(path: &Path, project: &str, state: &mut StateStore) -> Result<()> {
    let save: Save = serde_json::from_str(&read_text(path)?)
        .map_err(|e| Error(format!("save {}: {e}", path.display())))?;
    if save.schema_version != 1 {
        return Err(Error(format!(
            "save {}: unsupported schema_version {}; v1 games require v1 flag saves",
            path.display(),
            save.schema_version
        )));
    }
    version(save.schema_version, "save")
        .map_err(|e| Error(format!("save {}: {e}", path.display())))?;
    if save.project != project {
        return Err(Error(format!(
            "save {} belongs to project {}, expected {project}",
            path.display(),
            save.project
        )));
    }
    if save.state.keys().collect::<Vec<_>>() != state.persistent_values().keys().collect::<Vec<_>>()
    {
        return Err(Error(format!(
            "save {}: persistent key set differs from project schema",
            path.display()
        )));
    }
    let mut candidate = state.clone();
    for (key, value) in save.state {
        candidate
            .set(&key, value)
            .map_err(|e| Error(format!("save {}: {e}", path.display())))?;
    }
    *state = candidate;
    Ok(())
}
pub fn hash_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn snapshot_hash(snapshot: &Snapshot) -> Result<String> {
    Ok(hash_bytes(
        &serde_json::to_vec(snapshot).map_err(|e| Error(format!("snapshot hash: {e}")))?,
    ))
}
pub fn assert_snapshot(snapshot: &Snapshot, assertion: &Assertion) -> Result<()> {
    let fail = |message: String| Error(format!("assertion at tick {}: {message}", assertion.tick));
    if snapshot.tick != assertion.tick {
        return Err(fail(format!("observed tick {}", snapshot.tick)));
    }
    if let Some(scene) = &assertion.scene
        && &snapshot.scene != scene
    {
        return Err(fail(format!(
            "expected scene {scene}, got {}",
            snapshot.scene
        )));
    }
    if let Some(a) = &assertion.entity {
        let entity = snapshot
            .entities
            .iter()
            .find(|e| e.id == a.id)
            .ok_or_else(|| fail(format!("entity {} missing", a.id)))?;
        if let Some([x, y]) = a.position
            && entity.position != Vec2::pixels(x, y)
        {
            return Err(fail(format!(
                "entity {} expected pixel position [{x},{y}], got subpixels {:?}",
                a.id, entity.position
            )));
        }
        if let Some(max) = a.max_x
            && entity.position.x > max * SUBPIXELS
        {
            return Err(fail(format!("entity {} crossed max_x {max}", a.id)));
        }
    }
    for (key, value) in &assertion.state {
        if snapshot.state.get(key) != Some(value) {
            return Err(fail(format!(
                "state {key} expected {value}, got {:?}",
                snapshot.state.get(key)
            )));
        }
    }
    if let Some(e) = &assertion.event
        && !snapshot.events.iter().any(|event| {
            event.kind == e.kind
                && e.entity
                    .as_ref()
                    .is_none_or(|id| event.entity.as_ref() == Some(id))
                && e.target.as_ref().is_none_or(|target| {
                    event.data.get("target").and_then(Value::as_str) == Some(target)
                })
        })
    {
        return Err(fail(format!(
            "expected event {:?} in retained event stream",
            e
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    fn demo() -> Project {
        Project::load(Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/basement_demo"
        )))
        .unwrap()
    }
    #[test]
    fn movement_collision_and_replay_are_real_and_deterministic() {
        let mut first = World::new(demo()).unwrap();
        let mut second = World::new(demo()).unwrap();
        for _ in 0..100 {
            first
                .step(&Input {
                    right: true,
                    ..Input::default()
                })
                .unwrap();
            second
                .step(&Input {
                    right: true,
                    ..Input::default()
                })
                .unwrap();
        }
        assert_eq!(first.snapshot(), second.snapshot());
        assert_eq!(first.entities["player"].position, Vec2::pixels(84, 40));
        assert!(
            first
                .snapshot()
                .events
                .iter()
                .any(|e| e.kind == "collision_started" && e.data["target"] == "wall")
        );
    }
    #[test]
    fn save_reload_checks_types_without_partial_mutation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("save.json");
        let mut world = World::new(demo()).unwrap();
        world
            .state
            .set("demo.npc.spoken", Value::Bool(true))
            .unwrap();
        world.save(&path).unwrap();
        let restored = World::with_save(demo(), Some(&path)).unwrap();
        assert_eq!(restored.state.values["demo.npc.spoken"], true);
        let original = world.state.values.clone();
        let mut corrupt: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        corrupt["state"]["demo.npc.spoken"] = json!(123);
        fs::write(&path, corrupt.to_string()).unwrap();
        assert!(load_save(&path, &world.project.manifest.name, &mut world.state).is_err());
        assert_eq!(original, world.state.values);
        fs::write(&path, "broken json").unwrap();
        assert!(World::with_save(demo(), Some(&path)).is_err());
    }
    #[test]
    fn high_speed_motion_cannot_skip_walls_or_door_triggers() {
        let mut world = World::new(demo()).unwrap();
        world
            .entities
            .get_mut("player")
            .unwrap()
            .spec
            .player
            .as_mut()
            .unwrap()
            .speed = 60_000;
        world
            .step(&Input {
                right: true,
                ..Input::default()
            })
            .unwrap();
        assert_eq!(world.entities["player"].position, Vec2::pixels(84, 40));
        world.entities.get_mut("player").unwrap().position = Vec2::pixels(84, 130);
        world
            .step(&Input {
                right: true,
                ..Input::default()
            })
            .unwrap();
        assert_eq!(world.scene, "room_b");
        assert_eq!(world.entities["player"].position, Vec2::pixels(24, 40));
        assert!(
            world
                .snapshot()
                .events
                .iter()
                .any(|e| e.kind == "trigger_entered" && e.data["target"] == "door")
        );
    }
    #[test]
    fn interaction_is_edge_triggered_and_state_change_is_not_duplicated() {
        let mut world = World::new(demo()).unwrap();
        world.entities.get_mut("player").unwrap().position = Vec2::pixels(84, 130);
        for _ in 0..3 {
            world
                .step(&Input {
                    interact: true,
                    ..Input::default()
                })
                .unwrap();
        }
        assert_eq!(
            world
                .snapshot()
                .events
                .iter()
                .filter(|e| e.kind == "interaction")
                .count(),
            1
        );
        world.step(&Input::default()).unwrap();
        world
            .step(&Input {
                interact: true,
                ..Input::default()
            })
            .unwrap();
        assert_eq!(
            world
                .snapshot()
                .events
                .iter()
                .filter(|e| e.kind == "interaction")
                .count(),
            2
        );
        assert_eq!(
            world
                .snapshot()
                .events
                .iter()
                .filter(|e| e.kind == "state_changed" && e.data["key"] == "demo.npc.spoken")
                .count(),
            1
        );
    }
    #[test]
    fn save_version_and_key_migrations_are_never_silently_assumed() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("save.json");
        let mut world = World::new(demo()).unwrap();
        world.save(&path).unwrap();
        let original: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        let mut future = original.clone();
        future["schema_version"] = json!(99);
        fs::write(&path, future.to_string()).unwrap();
        assert!(
            World::with_save(demo(), Some(&path))
                .err()
                .unwrap()
                .to_string()
                .contains("schema_version 99")
        );
        let mut missing = original;
        missing["state"]
            .as_object_mut()
            .unwrap()
            .remove("demo.npc.spoken");
        fs::write(&path, missing.to_string()).unwrap();
        assert!(
            World::with_save(demo(), Some(&path))
                .err()
                .unwrap()
                .to_string()
                .contains("key set")
        );
    }
    #[test]
    fn event_memory_is_bounded_and_reports_loss() {
        let mut world = World::new(demo()).unwrap();
        for _ in 0..5000 {
            world
                .step(&Input {
                    right: true,
                    ..Input::default()
                })
                .unwrap();
        }
        assert_eq!(world.snapshot().events.len(), EVENT_LIMIT);
        assert!(world.events_dropped > 0);
    }
}

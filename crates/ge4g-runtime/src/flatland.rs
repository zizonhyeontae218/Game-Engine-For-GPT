//! First FlatLand vertical slice. All gameplay remains in the Rust world.
use super::*;
use ge4g_project::flatland::{Action, BodyMode, Condition, Map};
use mlua::{Lua, LuaOptions, LuaSerdeExt, StdLib};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ActorState {
    pub facing: [i64; 2],
    pub direction: [i64; 2],
    pub queued: [i64; 2],
    pub hp: Option<i64>,
    pub immune_until: u64,
}
impl ActorState {
    pub fn new(entity: &Entity) -> Self {
        Self {
            facing: [1, 0],
            direction: [0, 0],
            queued: [0, 0],
            hp: entity.flatland.as_ref().and_then(|a| a.hp),
            immune_until: 0,
        }
    }
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FlatState {
    pub timers: BTreeMap<String, u64>,
    pub completed: BTreeSet<String>,
    pub stopped: bool,
    pub popup: Option<Popup>,
    pub popup_serial: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Popup {
    pub id: u64,
    pub text: String,
}
#[derive(Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Resume {
    schema_version: u32,
    project: String,
    content: String,
    tick: u64,
    scene: String,
    state: BTreeMap<String, Value>,
    actors: BTreeMap<String, LiveEntity>,
    runtime: FlatState,
}
impl World {
    pub fn facing(&self, id: &str) -> Result<[i64; 2]> {
        self.entities
            .get(id)
            .map(|e| e.actor.facing)
            .ok_or_else(|| Error(format!("unknown actor {id}")))
    }
    pub fn face(&mut self, id: &str, vector: [i64; 2]) -> Result<()> {
        if vector != [0, 0] {
            self.entities
                .get_mut(id)
                .ok_or_else(|| Error(format!("unknown actor {id}")))?
                .actor
                .facing = [vector[0].signum(), vector[1].signum()];
        }
        Ok(())
    }
    pub fn toward(&self, a: &str, b: &str) -> Result<[i64; 2]> {
        let a = self
            .entities
            .get(a)
            .ok_or_else(|| Error("missing source actor".into()))?;
        let b = self
            .entities
            .get(b)
            .ok_or_else(|| Error("missing target actor".into()))?;
        Ok([
            (b.position.x - a.position.x).signum(),
            (b.position.y - a.position.y).signum(),
        ])
    }
    pub fn in_front(&self, a: &str, b: &str, range: u32) -> Result<bool> {
        let facing = self.facing(a)?;
        let a = self.entities.get(a).unwrap();
        let b = self
            .entities
            .get(b)
            .ok_or_else(|| Error("missing target actor".into()))?;
        let x = b.position.x + b.aabb().size.x / 2 - a.position.x - a.aabb().size.x / 2;
        let y = b.position.y + b.aabb().size.y / 2 - a.position.y - a.aabb().size.y / 2;
        let dot = x * facing[0] + y * facing[1];
        let cross = x * facing[1] - y * facing[0];
        Ok(dot > 0 && cross.abs() <= dot && x.abs().max(y.abs()) <= i64::from(range) * SUBPIXELS)
    }
    fn timer(&self, id: &str) -> bool {
        self.flatland.timers.get(id).is_some_and(|t| *t > self.tick)
    }
    fn condition(&self, c: &Condition, target: Option<&str>) -> bool {
        match c {
            Condition::Always => true,
            Condition::Vulnerable { entity } => self
                .ref_id(entity, target)
                .ok()
                .and_then(|id| self.entities.get(&id))
                .is_some_and(|e| e.actor.immune_until <= self.tick),
            Condition::State { key, eq } => self.state.values.get(key) == Some(eq),
            Condition::Timer { id } => self.timer(id),
            Condition::Remaining { tag, count } => {
                self.entities
                    .values()
                    .filter(|e| e.spec.tags.contains(tag))
                    .count()
                    == *count
            }
            Condition::Hp { entity, le } => self
                .entities
                .get(entity)
                .and_then(|e| e.actor.hp)
                .is_some_and(|hp| hp <= *le),
            Condition::All { conditions } => conditions.iter().all(|c| self.condition(c, target)),
            Condition::Not { condition } => !self.condition(condition, target),
        }
    }
    fn ref_id(&self, id: &str, target: Option<&str>) -> Result<String> {
        match id {
            "$target" => target
                .map(str::to_owned)
                .ok_or_else(|| Error("action needs event target".into())),
            "$player" => self
                .entities
                .values()
                .find(|e| e.spec.player.is_some())
                .map(|e| e.spec.id.clone())
                .ok_or_else(|| Error("no player".into())),
            _ => Ok(id.into()),
        }
    }
    fn actions(&mut self, actions: &[Action], target: Option<&str>) -> Result<()> {
        self.command_budget += actions.len();
        if self.command_budget > 4096 {
            return Err(Error("tick command budget exceeded".into()));
        }
        if actions.len() > 64 {
            return Err(Error("action batch exceeds 64".into()));
        }
        for action in actions {
            self.project
                .validate_action(&self.project.scenes[&self.scene], action)?;
            match action {
                Action::Set { key, value } => {
                    self.set_state(&BTreeMap::from([(key.clone(), value.clone())]), target)?
                }
                Action::Add { key, value } => {
                    let old = self
                        .state
                        .values
                        .get(key)
                        .and_then(Value::as_i64)
                        .ok_or_else(|| Error(format!("not an integer state {key}")))?;
                    let next = old
                        .checked_add(*value)
                        .ok_or_else(|| Error(format!("state overflow {key}")))?;
                    self.set_state(&BTreeMap::from([(key.clone(), json!(next))]), target)?;
                }
                Action::Timer { id, ticks } => {
                    self.flatland.timers.insert(id.clone(), self.tick + ticks);
                }
                Action::Say { text } => {
                    self.flatland.popup_serial += 1;
                    self.flatland.popup = Some(Popup {
                        id: self.flatland.popup_serial,
                        text: text.clone(),
                    });
                    self.emit(
                        "dialogue",
                        target,
                        json!({"text":text,"id":self.flatland.popup_serial}),
                    );
                }
                Action::Remove { entity } => {
                    let id = self.ref_id(entity, target)?;
                    if self
                        .entities
                        .get(&id)
                        .is_some_and(|e| e.spec.player.is_some())
                    {
                        return Err(Error("cannot remove player".into()));
                    }
                    self.entities
                        .remove(&id)
                        .ok_or_else(|| Error(format!("missing actor {id}")))?;
                    self.emit("entity_removed", Some(&id), json!({}));
                }
                Action::Damage {
                    entity,
                    amount,
                    immunity,
                } => {
                    let id = self.ref_id(entity, target)?;
                    let e = self
                        .entities
                        .get_mut(&id)
                        .ok_or_else(|| Error(format!("missing actor {id}")))?;
                    if e.actor.immune_until <= self.tick {
                        let hp = e
                            .actor
                            .hp
                            .ok_or_else(|| Error(format!("actor {id} has no HP")))?;
                        e.actor.hp = Some((hp - amount).max(0));
                        e.actor.immune_until = self.tick + immunity;
                        self.emit(
                            "hit",
                            Some(&id),
                            json!({"damage":amount,"hp":self.entities[&id].actor.hp}),
                        );
                    }
                }
                Action::Respawn { entity, immunity } => {
                    let id = self.ref_id(entity, target)?;
                    let e = self
                        .entities
                        .get_mut(&id)
                        .ok_or_else(|| Error(format!("missing actor {id}")))?;
                    e.position = Vec2::pixels(e.spec.position[0], e.spec.position[1]);
                    e.actor.immune_until = e.actor.immune_until.max(self.tick + immunity);
                    e.actor.direction = [0, 0];
                    e.actor.queued = [0, 0];
                }
                Action::Face { entity, vector } => {
                    let id = self.ref_id(entity, target)?;
                    self.face(&id, *vector)?;
                }
                Action::Goto { scene, spawn } => {
                    let id = self.ref_id("$player", None)?;
                    self.transition(
                        &Transition {
                            scene: scene.clone(),
                            spawn: spawn.clone(),
                        },
                        &id,
                    )?;
                    return Ok(()); // No stale actions after a scene boundary.
                }
                Action::Stop {} => self.flatland.stopped = true,
                Action::Sound { cue } => self.emit(
                    "audio",
                    target,
                    json!({"cue":cue,"file":self.project.scenes[&self.scene].sounds[cue]}),
                ),
            }
        }
        Ok(())
    }
    pub(super) fn flat_event(&mut self, on: &str, target: Option<&str>) -> Result<()> {
        let origin = self.scene.clone();
        let rules = self.project.scenes[&self.scene].rules.clone();
        let tags = target
            .and_then(|id| self.entities.get(id))
            .map(|e| e.spec.tags.clone())
            .unwrap_or_default();
        for r in rules {
            if r.on == on
                && (!r.once
                    || !self
                        .flatland
                        .completed
                        .contains(&format!("{}:{}", self.scene, r.id)))
                && r.target_tag.as_ref().is_none_or(|tag| tags.contains(tag))
                && self.condition(&r.when, target)
            {
                self.actions(&r.actions, target)?;
                if self.scene != origin {
                    return Ok(());
                }
                if r.once {
                    self.flatland
                        .completed
                        .insert(format!("{}:{}", self.scene, r.id));
                }
                self.emit("rule_fired", target, json!({"rule":r.id}));
            }
        }
        if let Some(source) = self.project.scripts.get(&self.scene).cloned() {
            let lua = Lua::new_with(
                StdLib::TABLE | StdLib::STRING | StdLib::UTF8,
                LuaOptions::default(),
            )
            .map_err(lua_error)?;
            for key in [
                "dofile",
                "loadfile",
                "load",
                "require",
                "print",
                "pairs",
                "next",
                "pcall",
                "xpcall",
                "collectgarbage",
                "warn",
            ] {
                lua.globals()
                    .set(key, mlua::Value::Nil)
                    .map_err(lua_error)?;
            }
            let strings: mlua::Table = lua.globals().get("string").map_err(lua_error)?;
            for key in ["dump", "format", "find", "match", "gmatch", "gsub"] {
                strings.set(key, mlua::Value::Nil).map_err(lua_error)?;
            }
            lua.globals()
                .set(
                    "tostring",
                    lua.create_function(|_, value: mlua::Value| match value {
                        mlua::Value::String(s) => Ok(s.to_str()?.to_owned()),
                        mlua::Value::Integer(i) => Ok(i.to_string()),
                        mlua::Value::Boolean(b) => Ok(b.to_string()),
                        mlua::Value::Nil => Ok("nil".into()),
                        _ => Err(mlua::Error::RuntimeError(
                            "tostring accepts only string, integer, boolean or nil".into(),
                        )),
                    })
                    .map_err(lua_error)?,
                )
                .map_err(lua_error)?;
            lua.set_memory_limit(2 * 1024 * 1024).map_err(lua_error)?;
            let count = std::cell::Cell::new(0);
            let budget = self.lua_budget.clone();
            lua.set_hook(
                mlua::HookTriggers::new().every_nth_instruction(1000),
                move |_, _| {
                    count.set(count.get() + 1000);
                    let total = budget.fetch_add(1000, std::sync::atomic::Ordering::Relaxed) + 1000;
                    if count.get() > 20_000 || total > 100_000 {
                        Err(mlua::Error::RuntimeError(
                            "Lua instruction budget exceeded".into(),
                        ))
                    } else {
                        Ok(mlua::VmState::Continue)
                    }
                },
            )
            .map_err(lua_error)?;
            let facing: BTreeMap<String, [i64; 2]> = self
                .entities
                .iter()
                .map(|(id, e)| (id.clone(), e.actor.facing))
                .collect();
            lua.globals()
                .set(
                    "facing",
                    lua.create_function(move |_, id: String| {
                        facing
                            .get(&id)
                            .map(|v| (v[0], v[1]))
                            .ok_or_else(|| mlua::Error::RuntimeError(format!("unknown actor {id}")))
                    })
                    .map_err(lua_error)?,
                )
                .map_err(lua_error)?;
            let ctx = lua
                .to_value(
                    &json!({"event":on,"target":target,"tick":self.tick,"state":self.state.values}),
                )
                .map_err(lua_error)?;
            let hook: mlua::Function = lua
                .load(&source)
                .set_name(
                    self.project.scenes[&self.scene]
                        .script
                        .as_deref()
                        .unwrap_or("scene.lua"),
                )
                .eval()
                .map_err(lua_error)?;
            let result: mlua::Value = hook.call(ctx).map_err(lua_error)?;
            if result != mlua::Value::Nil {
                let actions: Vec<Action> = lua.from_value(result).map_err(lua_error)?;
                self.actions(&actions, target)?;
            }
        }
        Ok(())
    }
    pub(super) fn step_flatland(&mut self, input: &Input) -> Result<()> {
        // Roll back authoritative state and events if any authored action/Lua hook fails.
        let backup_triggers = self.triggers.clone();
        let backup = (
            self.tick,
            self.scene.clone(),
            self.entities.clone(),
            self.state.clone(),
            self.flatland.clone(),
            self.events.clone(),
            self.events_dropped,
            self.interact_held,
        );
        let result = self.flat_tick(input);
        if result.is_err() {
            self.triggers = backup_triggers;
            (
                self.tick,
                self.scene,
                self.entities,
                self.state,
                self.flatland,
                self.events,
                self.events_dropped,
                self.interact_held,
            ) = backup;
        }
        result
    }
    fn flat_tick(&mut self, input: &Input) -> Result<()> {
        if self.tick >= 1_000_000 {
            return Err(Error("runtime tick limit reached".into()));
        }
        self.lua_budget
            .store(0, std::sync::atomic::Ordering::Relaxed);
        self.command_budget = 0;
        self.tick += 1;
        if self.flatland.stopped {
            return Ok(());
        }
        let player_id = self.ref_id("$player", None)?;
        let mut paths = Vec::new();
        let player_before = self.entities[&player_id].aabb();
        let mut actor_paths: BTreeMap<String, Vec<(Aabb, Aabb)>> = BTreeMap::new();
        let ids: Vec<String> = self
            .entities
            .iter()
            .filter(|(_, e)| {
                e.spec.player.is_some() || e.spec.flatland.as_ref().is_some_and(|a| a.ai.is_some())
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in ids {
            let e = &self.entities[&id];
            let ai = e.spec.flatland.as_ref().and_then(|a| a.ai.clone());
            let speed = ai.as_ref().map_or_else(
                || e.spec.player.as_ref().map_or(0, |p| p.speed),
                |a| a.speed,
            );
            let grid = e
                .spec
                .flatland
                .as_ref()
                .is_some_and(|a| a.grid || a.ai.is_some());
            let (x, y) = if ai.is_some() { (0, 0) } else { input.axes() };
            if grid {
                if x != 0 || y != 0 {
                    self.entities.get_mut(&id).unwrap().actor.queued =
                        if x != 0 { [x, 0] } else { [0, y] };
                }
                let map = self.project.scenes[&self.scene]
                    .map
                    .as_ref()
                    .unwrap()
                    .clone();
                let cell = i64::from(map.cell) * SUBPIXELS;
                let mut budget = speed;
                for _ in 0..128 {
                    let e = &self.entities[&id];
                    let pos = e.position;
                    if pos.x % cell == 0 && pos.y % cell == 0 {
                        let at = [pos.x / cell, pos.y / cell];
                        let desired = if let Some(a) = &ai {
                            let target = a
                                .target
                                .as_ref()
                                .and_then(|id| self.entities.get(id))
                                .map(|e| [e.position.x / cell, e.position.y / cell])
                                .unwrap_or(a.corner);
                            next_direction(
                                &map,
                                at,
                                target,
                                e.actor.direction,
                                a.flee_timer.as_ref().is_some_and(|t| self.timer(t)),
                            )
                        } else {
                            e.actor.queued
                        };
                        let open =
                            |d: [i64; 2]| d != [0, 0] && map.open(at[0] + d[0], at[1] + d[1]);
                        let dir = if open(desired) {
                            desired
                        } else if open(e.actor.direction) {
                            e.actor.direction
                        } else {
                            [0, 0]
                        };
                        self.entities.get_mut(&id).unwrap().actor.direction = dir;
                    }
                    let dir = self.entities[&id].actor.direction;
                    if dir == [0, 0] || budget == 0 {
                        break;
                    }
                    let pos = self.entities[&id].position;
                    let coordinate = if dir[0] != 0 { pos.x } else { pos.y };
                    let sign = if dir[0] != 0 { dir[0] } else { dir[1] };
                    let rem = coordinate.rem_euclid(cell);
                    let distance = if sign > 0 {
                        cell - rem
                    } else if rem == 0 {
                        cell
                    } else {
                        rem
                    };
                    let step = budget.min(distance);
                    let moved = self.move_axis(&id, dir[0] != 0, step * sign)?;
                    actor_paths.entry(id.clone()).or_default().push(moved);
                    if id == player_id {
                        paths.push(moved);
                    }
                    if moved.0.position != moved.1.position {
                        self.face(&id, dir)?;
                    } else {
                        self.entities.get_mut(&id).unwrap().actor.direction = [0, 0];
                        break;
                    }
                    budget -= step;
                }
            } else {
                for (axis, delta) in [(true, x * speed), (false, y * speed)] {
                    let moved = self.move_axis(&id, axis, delta)?;
                    actor_paths.entry(id.clone()).or_default().push(moved);
                    if id == player_id {
                        paths.push(moved);
                    }
                    if moved.0.position != moved.1.position {
                        self.face(&id, [x, y])?;
                    }
                }
            }
        }
        let player = self.entities[&player_id].aabb();
        let touched = |box_: Aabb| {
            player.overlaps(box_)
                || paths
                    .iter()
                    .any(|(a, b)| a.swept_bounds(b.position).overlaps(box_))
        };
        let targets: Vec<String> = self
            .entities
            .iter()
            .filter(|(id, e)| {
                *id != &player_id
                    && (touched(e.aabb())
                        || actor_paths.get(*id).is_some_and(|segments| {
                            segments
                                .iter()
                                .any(|(a, b)| a.swept_bounds(b.position).overlaps(player_before))
                        }))
            })
            .map(|(id, _)| id.clone())
            .collect();
        for target in targets {
            if !self.entities.contains_key(&target) {
                continue;
            }
            let pickup = self.entities[&target]
                .spec
                .flatland
                .as_ref()
                .map(|a| a.pickup.clone())
                .unwrap_or_default();
            if !pickup.is_empty() {
                self.emit("pickup", Some(&player_id), json!({"target":target}));
                self.actions(&pickup, Some(&target))?;
                self.flat_event("pickup", Some(&target))?;
                self.entities.remove(&target);
            } else {
                self.flat_event("contact", Some(&target))?;
            }
        }
        if input.interact && !self.interact_held {
            let candidates: Vec<String> = self
                .entities
                .iter()
                .filter(|(_, e)| e.spec.interaction.is_some())
                .map(|(id, _)| id.clone())
                .collect();
            for target in candidates {
                let e = &self.entities[&target];
                let i = e.spec.interaction.clone().unwrap();
                let range = i64::from(i.range) * SUBPIXELS;
                let near = Aabb {
                    position: Vec2 {
                        x: e.position.x - range,
                        y: e.position.y - range,
                    },
                    size: Vec2 {
                        x: e.aabb().size.x + 2 * range,
                        y: e.aabb().size.y + 2 * range,
                    },
                };
                if player.overlaps(near) {
                    if !i.dialogue.is_empty() {
                        self.actions(&[Action::Say { text: i.dialogue }], Some(&target))?;
                    }
                    self.set_state(&i.set_state, Some(&target))?;
                    self.flat_event("interact", Some(&target))?;
                    if let Some(t) = i.transition {
                        self.transition(&t, &player_id)?;
                    }
                    break;
                }
            }
        }
        self.interact_held = input.interact;
        let doors: Vec<String> = self
            .entities
            .iter()
            .filter(|(_, e)| e.spec.trigger.is_some() && touched(e.aabb()))
            .map(|(id, _)| id.clone())
            .collect();
        if let Some(id) = doors.iter().find(|id| !self.triggers.contains(*id)) {
            let t = self.entities[id].spec.trigger.clone().unwrap();
            self.transition(&t, &player_id)?;
            return Ok(());
        }
        self.triggers = doors.into_iter().collect();
        self.flat_event("tick", None)?;
        Ok(())
    }
    fn move_axis(&mut self, id: &str, x: bool, delta: i64) -> Result<(Aabb, Aabb)> {
        let start = self.entities[id].aabb();
        if delta == 0 {
            return Ok((start, start));
        }
        let plane = self.entities[id]
            .spec
            .flatland
            .as_ref()
            .map_or(0, |a| a.plane);
        let mut candidate = self.entities.clone();
        if !push_axis(
            &self.project.scenes[&self.scene].map,
            &mut candidate,
            id,
            x,
            delta,
            plane,
            &mut BTreeSet::new(),
        ) {
            // Clamp blocked movers up to contact, preserving free sliding.
            let mut allowed = delta;
            for wall in map_boxes(&self.project.scenes[&self.scene].map, start, delta, x) {
                allowed = start.sweep_axis(wall, allowed, x);
            }
            for (other, e) in &self.entities {
                if other != id && blocks(e, plane) {
                    allowed = start.sweep_axis(e.aabb(), allowed, x);
                }
            }
            let e = self.entities.get_mut(id).unwrap();
            if x {
                e.position.x += allowed;
            } else {
                e.position.y += allowed;
            }
        } else {
            self.entities = candidate;
        }
        Ok((start, self.entities[id].aabb()))
    }
    fn content_hash(&self) -> Result<String> {
        let mut bytes = Vec::new();
        for name in &self.project.files_checked {
            bytes.extend(name.as_bytes());
            bytes.push(0);
            bytes.extend(
                std::fs::read(self.project.path(name)?)
                    .map_err(|e| Error(format!("content hash: {e}")))?,
            );
            bytes.push(0);
        }
        Ok(hash_bytes(&bytes))
    }
    pub(super) fn save_flatland(&mut self, path: &Path) -> Result<()> {
        let save = Resume {
            schema_version: 2,
            project: self.project.manifest.name.clone(),
            content: self.content_hash()?,
            tick: self.tick,
            scene: self.scene.clone(),
            state: self.state.values.clone(),
            actors: self.entities.clone(),
            runtime: self.flatland.clone(),
        };
        atomic_json(path, &save)?;
        self.emit(
            "save_written",
            None,
            json!({"path":path.to_string_lossy(),"resume":true}),
        );
        Ok(())
    }
    pub(super) fn load_flatland(&mut self, path: &Path) -> Result<()> {
        let save: Resume =
            serde_json::from_str(&read_text(path)?).map_err(|e| Error(format!("resume: {e}")))?;
        if save.schema_version != 2
            || save.project != self.project.manifest.name
            || save.content != self.content_hash()?
            || save.tick > 1_000_000
            || !self.project.scenes.contains_key(&save.scene)
        {
            return Err(Error(
                "resume version/project/content/scene mismatch".into(),
            ));
        }
        if save.state.keys().ne(self.state.values.keys()) {
            return Err(Error("resume state key mismatch".into()));
        }
        let mut state = self.state.clone();
        for (key, value) in save.state {
            state.set(&key, value)?;
        }
        let scene = &self.project.scenes[&save.scene];
        if save
            .actors
            .values()
            .filter(|e| e.spec.player.is_some())
            .count()
            != 1
        {
            return Err(Error("resume requires one player".into()));
        }
        for (id, e) in &save.actors {
            let spec = scene
                .entities
                .iter()
                .find(|s| &s.id == id)
                .ok_or_else(|| Error(format!("unknown saved actor {id}")))?;
            if serde_json::to_value(spec).unwrap() != serde_json::to_value(&e.spec).unwrap()
                || e.spec.id != *id
                || [e.position.x, e.position.y]
                    .iter()
                    .any(|v| v.unsigned_abs() > (1_000_000 * SUBPIXELS) as u64)
                || e.actor.facing.iter().any(|v| !(-1..=1).contains(v))
                || e.actor.direction.iter().any(|v| !(-1..=1).contains(v))
                || e.actor.queued.iter().any(|v| !(-1..=1).contains(v))
                || e.actor.hp.is_some() != spec.flatland.as_ref().and_then(|a| a.hp).is_some()
                || e.actor.immune_until > save.tick + 1_000_000
                || e.actor.hp.is_some_and(|hp| {
                    hp < 0 || hp > spec.flatland.as_ref().and_then(|a| a.hp).unwrap_or(0)
                })
            {
                return Err(Error(format!("invalid saved actor {id}")));
            }
        }
        if save
            .runtime
            .timers
            .values()
            .any(|end| *end > save.tick + 1_000_000)
            || save
                .runtime
                .popup
                .as_ref()
                .is_some_and(|p| p.text.len() > 8192 || p.id > save.runtime.popup_serial)
        {
            return Err(Error("invalid saved timer/popup".into()));
        }
        for e in save.actors.values().filter(|e| {
            e.spec.player.is_some()
                || e.spec
                    .flatland
                    .as_ref()
                    .is_some_and(|a| a.body != BodyMode::Pass)
        }) {
            if map_boxes(&scene.map, e.aabb(), 0, true)
                .iter()
                .any(|wall| e.aabb().overlaps(*wall))
            {
                return Err(Error("saved actor overlaps map wall".into()));
            }
        }
        self.tick = save.tick;
        self.scene = save.scene;
        self.entities = save.actors;
        self.state = state;
        self.flatland = save.runtime;
        self.events.clear();
        self.events_dropped = 0;
        self.interact_held = false;
        self.held_actions.clear();
        self.emit("save_loaded", None, json!({"resume":true}));
        Ok(())
    }
}
fn lua_error(e: mlua::Error) -> Error {
    Error(format!("Lua: {e}"))
}
fn blocks(e: &LiveEntity, plane: i32) -> bool {
    e.spec
        .flatland
        .as_ref()
        .is_some_and(|a| a.plane == plane && a.body != BodyMode::Pass)
        || e.spec.collider.as_ref().is_some_and(|c| c.blocking)
}
fn map_boxes(map: &Option<Map>, body: Aabb, delta: i64, x: bool) -> Vec<Aabb> {
    let Some(map) = map else {
        return vec![];
    };
    let destination = Vec2 {
        x: body.position.x + if x { delta } else { 0 },
        y: body.position.y + if x { 0 } else { delta },
    };
    let bounds = body.swept_bounds(destination);
    let cell = i64::from(map.cell) * SUBPIXELS;
    let mut boxes = Vec::new();
    let width = map.rows[0].len() as i64;
    let height = map.rows.len() as i64;
    for cy in bounds.position.y.div_euclid(cell).max(-1)
        ..=(bounds.position.y + bounds.size.y - 1)
            .div_euclid(cell)
            .min(height)
    {
        for cx in bounds.position.x.div_euclid(cell).max(-1)
            ..=(bounds.position.x + bounds.size.x - 1)
                .div_euclid(cell)
                .min(width)
        {
            if !map.open(cx, cy) {
                boxes.push(Aabb {
                    position: Vec2 {
                        x: cx * cell,
                        y: cy * cell,
                    },
                    size: Vec2 { x: cell, y: cell },
                });
            }
        }
    }
    boxes
}
fn push_axis(
    map: &Option<Map>,
    entities: &mut BTreeMap<String, LiveEntity>,
    id: &str,
    x: bool,
    delta: i64,
    plane: i32,
    visited: &mut BTreeSet<String>,
) -> bool {
    if visited.len() >= 32 || !visited.insert(id.into()) {
        return false;
    }
    let start = entities[id].aabb();
    if map_boxes(map, start, delta, x)
        .into_iter()
        .any(|w| start.sweep_axis(w, delta, x) != delta)
    {
        visited.remove(id);
        return false;
    }
    let others: Vec<String> = entities
        .iter()
        .filter(|(other, e)| {
            *other != id && blocks(e, plane) && start.sweep_axis(e.aabb(), delta, x) != delta
        })
        .map(|(id, _)| id.clone())
        .collect();
    for other in others {
        let gap = start.sweep_axis(entities[&other].aabb(), delta, x);
        if entities[&other]
            .spec
            .flatland
            .as_ref()
            .is_none_or(|a| a.body != BodyMode::Push)
            || !push_axis(map, entities, &other, x, delta - gap, plane, visited)
        {
            visited.remove(id);
            return false;
        }
    }
    let e = entities.get_mut(id).unwrap();
    if x {
        e.position.x += delta;
    } else {
        e.position.y += delta;
    }
    visited.remove(id);
    true
}
fn next_direction(
    map: &Map,
    at: [i64; 2],
    goal: [i64; 2],
    previous: [i64; 2],
    flee: bool,
) -> [i64; 2] {
    // One bounded BFS per intersection; tie order is up/left/down/right.
    let directions = [[0, -1], [-1, 0], [0, 1], [1, 0]];
    let mut distances = BTreeMap::new();
    let mut queue = VecDeque::new();
    if map.open(goal[0], goal[1]) {
        distances.insert(goal, 0i64);
        queue.push_back(goal);
    }
    while let Some(p) = queue.pop_front() {
        let d = distances[&p];
        for v in directions {
            let q = [p[0] + v[0], p[1] + v[1]];
            if map.open(q[0], q[1]) && !distances.contains_key(&q) {
                distances.insert(q, d + 1);
                queue.push_back(q);
            }
        }
    }
    let mut choices: Vec<_> = directions
        .into_iter()
        .filter(|v| map.open(at[0] + v[0], at[1] + v[1]))
        .collect();
    if choices.len() > 1 {
        choices.retain(|v| *v != [-previous[0], -previous[1]]);
    }
    choices
        .into_iter()
        .min_by_key(|v| {
            let q = [at[0] + v[0], at[1] + v[1]];
            let d = distances
                .get(&q)
                .copied()
                .unwrap_or((q[0] - goal[0]).abs() + (q[1] - goal[1]).abs() + 16384);
            if flee { -d } else { d }
        })
        .unwrap_or([0, 0])
}

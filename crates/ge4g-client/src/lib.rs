//! Version 1 native client ABI; the existing World and CPU renderer stay authoritative.
use ge4g_core::{Error, Input, Result};
use ge4g_project::Project;
use ge4g_render2d::{Frame, render};
use ge4g_runtime::World;
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{CStr, CString, c_char},
    panic::{AssertUnwindSafe, catch_unwind},
    path::Path,
    sync::{Mutex, OnceLock},
};

pub const ABI_VERSION: u32 = 1;
struct Session {
    world: World,
    frame: Frame,
    debug: bool,
}
#[derive(Default)]
struct Registry {
    next: u64,
    sessions: BTreeMap<u64, Session>,
}
static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
fn registry() -> &'static Mutex<Registry> {
    REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClientInput {
    #[serde(default)]
    direction: Option<[i64; 2]>,
    #[serde(default)]
    left: bool,
    #[serde(default)]
    right: bool,
    #[serde(default)]
    up: bool,
    #[serde(default)]
    down: bool,
    #[serde(default)]
    interact: bool,
    #[serde(default)]
    actions: BTreeSet<String>,
}
impl ClientInput {
    fn core(&self) -> Input {
        Input {
            direction: self.direction,
            left: self.left,
            right: self.right,
            up: self.up,
            down: self.down,
            interact: self.interact,
        }
    }
}
#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
enum Request {
    Replay {
        session: u64,
        replay: ge4g_project::Replay,
    },
    Choose {
        session: u64,
        choice: String,
    },
    Command {
        session: u64,
        actions: Vec<ge4g_project::flatland::Action>,
    },
    Skip {
        session: u64,
    },
    Validate {
        project: String,
    },
    Open {
        project: String,
        #[serde(default)]
        load: Option<String>,
    },
    Advance {
        session: u64,
        ticks: u32,
        #[serde(default)]
        input: ClientInput,
    },
    Observe {
        session: u64,
        #[serde(default)]
        compact: bool,
        #[serde(default)]
        after: Option<u64>,
        #[serde(default)]
        entity: Option<String>,
    },
    Release {
        session: u64,
    },
    Debug {
        session: u64,
        enabled: bool,
    },
    Save {
        session: u64,
        path: String,
    },
    Close {
        session: u64,
    },
}
fn status(session: u64, live: &Session) -> Value {
    let events = live.world.events();
    let events_dropped = live.world.event_offset();
    let dialogue = events
        .iter()
        .rev()
        .find(|e| e.kind == "interaction")
        .and_then(|e| e.data.get("dialogue"))
        .cloned();
    let action = events
        .iter()
        .rev()
        .find(|e| e.kind == "action_pressed")
        .and_then(|e| e.data.get("action"))
        .cloned();
    let mut response = json!({"abi_version": ABI_VERSION, "ok": true, "session": session, "tick": live.world.tick, "scene": live.world.scene, "width": live.frame.width, "height": live.frame.height, "frame_bytes": live.frame.rgba.len(), "state": live.world.state.values, "dialogue": dialogue, "last_action": action});
    response["waiting"] = live.world.waiting().unwrap_or(Value::Null);
    if response["waiting"]["kind"] == "bubble" {
        let actor = response["waiting"]["actor"].as_str().unwrap_or("");
        if let Some(anchor) = ge4g_render2d::actor_screen_anchor(
            &live.world.project,
            &live.world.render_snapshot(),
            actor,
        ) {
            response["waiting"]["screen_anchor"] = json!(anchor);
        }
    }
    response["systems"]=live.world.flatland.systems.as_ref().map(|s|json!({"combatants":s.combatants,"gameplay_view":s.gameplay_view,"inventory":s.inventory,"equipment":s.equipment,"quests":s.quests,"music":s.music,"view":s.view,"pace":s.pace,"planes":s.planes,"event":s.events.last().map(|f|json!({"id":f.event,"pc":f.pc}))})).unwrap_or(Value::Null);
    response["catalog"] = json!({"items":live.world.project.scenes[&live.world.scene].gameplay.items.iter().map(|(id,i)|(id,json!({"name":i.name,"usable":!i.use_actions.is_empty(),"slot":i.slot}))).collect::<BTreeMap<_,_>>(),"quests":live.world.project.scenes[&live.world.scene].gameplay.quests.iter().map(|(id,q)|(id,json!({"name":q.name,"objectives":q.objectives,"objective_labels":q.objective_labels}))).collect::<BTreeMap<_,_>>()});
    response["view_label"] = live
        .world
        .flatland
        .systems
        .as_ref()
        .and_then(|s| s.view.as_ref())
        .and_then(|v| {
            live.world.project.scenes[&live.world.scene]
                .gameplay
                .views
                .get(v)
        })
        .map(|v| json!(v.label))
        .unwrap_or(Value::Null);
    response["game_schema"] = json!(live.world.project.manifest.schema_version);
    response["popup"] = if let Some(p) = &live.world.flatland.popup {
        json!(p)
    } else {
        events.iter().rev().find(|e|e.kind=="interaction").map(|e|json!({"id":format!("{}:{}",e.tick,e.entity.as_deref().unwrap_or("")),"text":e.data["dialogue"]})).unwrap_or(Value::Null)
    };
    response["actors"] = json!(
        live.world
            .entities
            .iter()
            .filter(|(_, e)| e.spec.player.is_some()
                || e.spec.flatland.as_ref().is_some_and(|a| a.ai.is_some()))
            .map(|(id, e)| (id.clone(), json!({"hp":e.actor.hp,"facing":e.actor.facing})))
            .collect::<BTreeMap<_, _>>()
    );
    response["timers"] = json!(
        live.world
            .flatland
            .timers
            .iter()
            .map(|(id, end)| (id.clone(), end.saturating_sub(live.world.tick)))
            .collect::<BTreeMap<_, _>>()
    );
    response["stopped"] = json!(live.world.flatland.stopped);
    let end = events_dropped + events.len() as u64;
    response["event_cursor"] = json!(end);
    response["audio"] = json!(
        events
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, e)| e.kind == "audio")
            .take(32)
            .map(|(i, e)| json!({"id":events_dropped+i as u64+1,"file":e.data.get("file")}))
            .collect::<Vec<_>>()
    );
    response
}
pub fn request_json(request: &str) -> Value {
    match catch_unwind(AssertUnwindSafe(|| dispatch(request))) {
        Ok(Ok(value)) => value,
        Ok(Err(error)) => {
            json!({"abi_version": ABI_VERSION, "ok": false, "error": error.to_string()})
        }
        Err(_) => {
            json!({"abi_version": ABI_VERSION, "ok": false, "error": "native runtime panic was contained; close and reopen the game"})
        }
    }
}
fn dispatch(text: &str) -> Result<Value> {
    if text.len() > 1024 * 1024 {
        return Err(Error("client request exceeds 1 MiB".into()));
    }
    let request: Request =
        serde_json::from_str(text).map_err(|e| Error(format!("client request: {e}")))?;
    if let Request::Validate { project } = &request {
        let report = Project::validate(Path::new(project));
        return Ok(
            json!({"abi_version": ABI_VERSION, "ok": report.ok, "errors": report.errors, "warnings": report.warnings, "files_checked": report.files_checked}),
        );
    }
    let mut registry = registry()
        .lock()
        .map_err(|_| Error("native session registry is poisoned; restart the client".into()))?;
    match request {
        Request::Replay { session, replay } => {
            replay.validate()?;
            if replay.ticks > 10000 {
                return Err(Error("native replay limit 10000 ticks".into()));
            }
            let live = registry
                .sessions
                .get_mut(&session)
                .ok_or_else(|| Error("unknown session".into()))?;
            for _ in 0..replay.ticks {
                live.world.step_replay(&replay)?;
            }
            live.frame = render(
                &live.world.project,
                &live.world.render_snapshot(),
                live.debug,
            )?;
            Ok(status(session, live))
        }
        Request::Choose { session, choice } => {
            let live = registry
                .sessions
                .get_mut(&session)
                .ok_or_else(|| Error("unknown session".into()))?;
            live.world.choose(&choice)?;
            live.frame = render(
                &live.world.project,
                &live.world.render_snapshot(),
                live.debug,
            )?;
            Ok(status(session, live))
        }
        Request::Command { session, actions } => {
            let live = registry
                .sessions
                .get_mut(&session)
                .ok_or_else(|| Error("unknown session".into()))?;
            live.world.command(&actions)?;
            live.frame = render(
                &live.world.project,
                &live.world.render_snapshot(),
                live.debug,
            )?;
            Ok(status(session, live))
        }
        Request::Skip { session } => {
            let live = registry
                .sessions
                .get_mut(&session)
                .ok_or_else(|| Error("unknown session".into()))?;
            live.world.skip_event_wait()?;
            live.frame = render(
                &live.world.project,
                &live.world.render_snapshot(),
                live.debug,
            )?;
            Ok(status(session, live))
        }
        Request::Open { project, load } => {
            if registry.sessions.len() >= 8 {
                return Err(Error(
                    "close an existing game before opening another (8 session limit)".into(),
                ));
            }
            let world = World::with_save(
                Project::load(Path::new(&project))?,
                load.as_deref().map(Path::new),
            )?;
            let frame = render(&world.project, &world.render_snapshot(), false)?;
            registry.next = registry
                .next
                .checked_add(1)
                .ok_or_else(|| Error("session id exhausted".into()))?;
            let id = registry.next;
            let live = Session {
                world,
                frame,
                debug: false,
            };
            let response = status(id, &live);
            registry.sessions.insert(id, live);
            Ok(response)
        }
        Request::Close { session } => {
            if registry.sessions.remove(&session).is_none() {
                return Err(Error(format!("unknown client session {session}")));
            }
            Ok(json!({"abi_version": ABI_VERSION, "ok": true}))
        }
        Request::Advance {
            session,
            ticks,
            input,
        } => {
            if ticks > 15 {
                return Err(Error(
                    "client advance allows 0..15 ticks per request".into(),
                ));
            }
            let live = registry
                .sessions
                .get_mut(&session)
                .ok_or_else(|| Error(format!("unknown client session {session}")))?;
            let popup = live.world.flatland.popup_serial;
            let waiting = live
                .world
                .waiting()
                .and_then(|w| w["id"].as_str().map(str::to_owned));
            for _ in 0..ticks {
                live.world.step_actions(&input.core(), &input.actions)?;
                if live.world.flatland.popup_serial != popup
                    || live
                        .world
                        .waiting()
                        .and_then(|w| w["id"].as_str().map(str::to_owned))
                        != waiting
                {
                    break;
                }
            }
            live.frame = render(
                &live.world.project,
                &live.world.render_snapshot(),
                live.debug,
            )?;
            Ok(status(session, live))
        }
        Request::Observe {
            session,
            compact,
            after,
            entity,
        } => {
            let live = registry
                .sessions
                .get(&session)
                .ok_or_else(|| Error(format!("unknown client session {session}")))?;
            let snapshot = live.world.snapshot();
            if let Some(id) = entity {
                let e = snapshot
                    .entities
                    .iter()
                    .find(|e| e.id == id)
                    .ok_or_else(|| Error(format!("unknown entity {id}")))?;
                return Ok(
                    json!({"abi_version":ABI_VERSION,"ok":true,"tick":snapshot.tick,"entity":e}),
                );
            }
            if compact || after.is_some() {
                let start = snapshot.events_dropped;
                let end = start + snapshot.events.len() as u64;
                let requested = after.unwrap_or(start);
                if requested > end {
                    return Err(Error("event cursor is ahead of this session".into()));
                }
                let events: Vec<_> = snapshot
                    .events
                    .iter()
                    .skip(requested.saturating_sub(start) as usize)
                    .take(256)
                    .collect();
                let cursor = requested.max(start) + events.len() as u64;
                return Ok(
                    json!({"abi_version":ABI_VERSION,"ok":true,"tick":snapshot.tick,"scene":snapshot.scene,"state":snapshot.state,"events":events,"cursor":cursor,"more":cursor<end,"reset_required":requested<start,"entity_count":snapshot.entities.len()}),
                );
            }
            Ok(json!({"abi_version": ABI_VERSION, "ok": true, "snapshot": snapshot}))
        }
        Request::Release { session } => {
            let live = registry
                .sessions
                .get_mut(&session)
                .ok_or_else(|| Error(format!("unknown client session {session}")))?;
            live.world.release_inputs();
            Ok(status(session, live))
        }
        Request::Debug { session, enabled } => {
            let live = registry
                .sessions
                .get_mut(&session)
                .ok_or_else(|| Error(format!("unknown client session {session}")))?;
            live.debug = enabled;
            live.frame = render(&live.world.project, &live.world.snapshot(), enabled)?;
            Ok(status(session, live))
        }
        Request::Save { session, path } => {
            let live = registry
                .sessions
                .get_mut(&session)
                .ok_or_else(|| Error(format!("unknown client session {session}")))?;
            live.world.save(Path::new(&path))?;
            Ok(status(session, live))
        }
        Request::Validate { .. } => unreachable!(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ge4g_abi_version() -> u32 {
    ABI_VERSION
}

/// # Safety
/// `request` is null or points to a valid NUL-terminated UTF-8 C string for this call.
/// Free the returned string exactly once using `ge4g_free_string`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ge4g_request_json(request: *const c_char) -> *mut c_char {
    let value = if request.is_null() {
        json!({"abi_version": 1, "ok": false, "error": "null client request"})
    } else {
        match unsafe { CStr::from_ptr(request) }.to_str() {
            Ok(text) => request_json(text),
            Err(_) => {
                json!({"abi_version": 1, "ok": false, "error": "client request must be UTF-8"})
            }
        }
    };
    CString::new(value.to_string())
        .expect("JSON serialization escapes NUL")
        .into_raw()
}
/// # Safety
/// `string` is null or an unfreed pointer returned by `ge4g_request_json`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ge4g_free_string(string: *mut c_char) {
    if !string.is_null() {
        drop(unsafe { CString::from_raw(string) });
    }
}
/// Copy the canonical RGBA frame. Returns byte count, -1 for invalid/short
/// destination, or -2 for an unknown/poisoned session. No borrowed pointers escape.
/// # Safety
/// A non-null destination must point to `capacity` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ge4g_frame_copy(
    session: u64,
    destination: *mut u8,
    capacity: usize,
) -> i64 {
    if destination.is_null() {
        return -1;
    }
    let Ok(registry) = registry().lock() else {
        return -2;
    };
    let Some(live) = registry.sessions.get(&session) else {
        return -2;
    };
    let bytes = &live.frame.rgba;
    if capacity < bytes.len() {
        return -1;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), destination, bytes.len());
    }
    bytes.len() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    fn demo() -> &'static str {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/basement_demo")
    }
    #[test]
    fn client_replay_and_frame_equal_the_authoritative_headless_runtime() {
        let opened = request_json(&json!({"op":"open","project":demo()}).to_string());
        assert_eq!(opened["ok"], true);
        let session = opened["session"].as_u64().unwrap();
        let project = Project::load(Path::new(demo())).unwrap();
        let replay =
            ge4g_project::Replay::load(&Path::new(demo()).join("replays/journey.json")).unwrap();
        let mut expected = World::new(project.clone()).unwrap();
        for tick in 0..replay.ticks {
            let input = replay.input_at(tick);
            expected.step(&input).unwrap();
            let result = request_json(
                &json!({"op":"advance","session":session,"ticks":1,"input": input}).to_string(),
            );
            assert_eq!(result["ok"], true);
        }
        let observed = request_json(&json!({"op":"observe","session":session}).to_string());
        assert_eq!(
            observed["snapshot"],
            serde_json::to_value(expected.snapshot()).unwrap()
        );
        let expected_frame = render(&project, &expected.snapshot(), false).unwrap();
        let mut actual = vec![0; expected_frame.rgba.len()];
        assert_eq!(
            unsafe { ge4g_frame_copy(session, actual.as_mut_ptr(), actual.len()) },
            actual.len() as i64
        );
        assert_eq!(actual, expected_frame.rgba);
        assert_eq!(
            unsafe { ge4g_frame_copy(session, actual.as_mut_ptr(), 1) },
            -1
        );
        assert_eq!(
            request_json(&json!({"op":"close","session":session}).to_string())["ok"],
            true
        );
    }
    #[test]
    fn c_strings_errors_and_named_button_release_are_real() {
        let request = CString::new("{broken").unwrap();
        let response = unsafe { ge4g_request_json(request.as_ptr()) };
        let json: Value =
            serde_json::from_str(unsafe { CStr::from_ptr(response) }.to_str().unwrap()).unwrap();
        assert_eq!(json["ok"], false);
        unsafe {
            ge4g_free_string(response);
            ge4g_free_string(std::ptr::null_mut());
        }
        let opened = request_json(&json!({"op":"open","project":demo()}).to_string());
        let session = opened["session"].as_u64().unwrap();
        request_json(
            &json!({"op":"advance","session":session,"ticks":1,"input":{"actions":["x","space"]}})
                .to_string(),
        );
        request_json(&json!({"op":"release","session":session}).to_string());
        let observed = request_json(&json!({"op":"observe","session":session}).to_string());
        assert!(
            observed["snapshot"]["events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["kind"] == "action_pressed" && e["data"]["action"] == "x")
        );
        assert!(
            observed["snapshot"]["events"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["kind"] == "action_released" && e["data"]["action"] == "space")
        );
        request_json(&json!({"op":"close","session":session}).to_string());
        assert_eq!(
            request_json(&json!({"op":"observe","session":session}).to_string())["ok"],
            false
        );
    }
    #[test]
    fn flatland_native_replay_selective_observation_and_resume_match_rust() {
        let path = Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/flatland_pacman"
        ));
        let opened = request_json(&json!({"op":"open","project":path}).to_string());
        assert_eq!(opened["game_schema"], 2);
        assert!(opened["popup"]["text"].as_str().unwrap().contains("MAZE"));
        let id = opened["session"].as_u64().unwrap();
        let mut expected = World::new(Project::load(path).unwrap()).unwrap();
        let input = Input {
            left: true,
            ..Input::default()
        };
        for _ in 0..30 {
            expected.step(&input).unwrap();
            assert_eq!(
                request_json(
                    &json!({"op":"advance","session":id,"ticks":1,"input":input}).to_string()
                )["ok"],
                true
            );
        }
        let result = request_json(&json!({"op":"observe","session":id}).to_string());
        assert_eq!(
            result["snapshot"],
            serde_json::to_value(expected.snapshot()).unwrap()
        );
        let partial =
            request_json(&json!({"op":"observe","session":id,"entity":"player"}).to_string());
        assert_eq!(partial["entity"]["flatland"]["facing"], json!([-1, 0]));
        let delta = request_json(
            &json!({"op":"observe","session":id,"compact":true,"after":opened["event_cursor"]})
                .to_string(),
        );
        assert_eq!(delta["reset_required"], false);
        assert!(!delta["events"].as_array().unwrap().is_empty());
        assert!(delta.get("snapshot").is_none());
        let frame = render(&expected.project, &expected.snapshot(), false).unwrap();
        let mut actual = vec![0; frame.rgba.len()];
        assert_eq!(
            unsafe { ge4g_frame_copy(id, actual.as_mut_ptr(), actual.len()) },
            actual.len() as i64
        );
        assert_eq!(actual, frame.rgba);
        let dir = std::env::temp_dir().join(format!("ge4g-ffi-{id}"));
        std::fs::create_dir_all(&dir).unwrap();
        let save = dir.join("save.json");
        assert_eq!(
            request_json(&json!({"op":"save","session":id,"path":save}).to_string())["ok"],
            true
        );
        request_json(&json!({"op":"close","session":id}).to_string());
        let restored = request_json(&json!({"op":"open","project":path,"load":save}).to_string());
        assert_eq!(restored["tick"], 30);
        assert_eq!(restored["state"]["game.score"], 30);
        request_json(&json!({"op":"close","session":restored["session"]}).to_string());
        std::fs::remove_dir_all(dir).unwrap();
    }
}

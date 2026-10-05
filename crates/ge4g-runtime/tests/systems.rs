use ge4g_core::{Input, Vec2};
use ge4g_project::{Project, flatland::Action};
use ge4g_runtime::World;
use serde_json::{Value, json};
use std::fs;
fn fixture(mut extra: Value) -> (tempfile::TempDir, World) {
    let d = tempfile::tempdir().unwrap();
    fs::write(
        d.path().join("ge4g.toml"),
        r#"schema_version=2
name="systems"
start_scene="room"
[window]
width=160
height=100
[scenes]
room="room.json5"
[state."test.reward"]
kind="integer"
default=0
[state."test.roll"]
kind="integer"
default=0
"#,
    )
    .unwrap();
    let mut scene = json!({"schema_version":2,"id":"room","background":[0,0,0,255],"spawns":{"start":[16,16]},"entities":[{"id":"player","position":[16,16],"size":[16,16],"player":{"speed":120},"flatland":{"hp":20,"team":"hero"}},{"id":"enemy","position":[48,16],"size":[16,16],"flatland":{"hp":5,"team":"enemy","drops":[{"op":"give","item":"coin","count":1}]}}],"gameplay":{"seed":42,"items":{"coin":{"name":"Coin","stack":99},"potion":{"name":"Potion","stack":9,"use_actions":[{"op":"heal","entity":"$player","amount":5}]}},"quests":{"signal":{"name":"Signal","objectives":{"key":1},"rewards":[{"op":"add","key":"test.reward","value":1}]}},"attacks":{"sword":{"damage":3,"range":48,"active":3,"recovery":2},"shot":{"damage":3,"range":48,"projectile_speed":120,"lifetime":60}},"events":{}}});
    if let Some(g) = extra.get_mut("gameplay").and_then(Value::as_object_mut) {
        for (k, v) in std::mem::take(g) {
            scene["gameplay"][k] = v;
        }
    }
    for (k, v) in extra.as_object().unwrap() {
        if k != "gameplay" {
            scene[k] = v.clone();
        }
    }
    fs::write(d.path().join("room.json5"), scene.to_string()).unwrap();
    let w = World::new(Project::load(d.path()).unwrap()).unwrap();
    (d, w)
}
fn actions(w: &mut World, value: Value) {
    let a: Vec<Action> = serde_json::from_value(value).unwrap();
    w.command(&a).unwrap();
}
#[test]
fn inventory_quest_reward_is_once_and_bad_batches_roll_back() {
    let (_d, mut w) = fixture(json!({}));
    actions(
        &mut w,
        json!([{"op":"quest","quest":"signal","status":"active"},{"op":"objective","quest":"signal","objective":"key"},{"op":"quest","quest":"signal","status":"completed"}]),
    );
    actions(
        &mut w,
        json!([{"op":"quest","quest":"signal","status":"completed"}]),
    );
    assert_eq!(w.state.values["test.reward"], 1);
    let before = w.snapshot();
    let bad = serde_json::from_value::<Vec<Action>>(
        json!([{"op":"give","item":"coin","count":2},{"op":"take","item":"potion","count":1}]),
    )
    .unwrap();
    assert!(w.command(&bad).is_err());
    assert_eq!(w.snapshot(), before);
}
#[test]
fn melee_hits_once_per_attack_then_drops_once() {
    let (_d, mut w) = fixture(json!({}));
    actions(
        &mut w,
        json!([{"op":"attack","entity":"player","attack":"sword"}]),
    );
    for _ in 0..3 {
        w.step(&Input::default()).unwrap();
    }
    assert_eq!(w.entities["enemy"].actor.hp, Some(2));
    assert_eq!(w.inventory("coin"), 0);
    for _ in 0..2 {
        w.step(&Input::default()).unwrap();
    }
    actions(
        &mut w,
        json!([{"op":"attack","entity":"player","attack":"sword"}]),
    );
    w.step(&Input::default()).unwrap();
    assert!(!w.entities.contains_key("enemy"));
    assert_eq!(w.inventory("coin"), 1);
}
#[test]
fn choice_wait_and_battle_resume_to_exact_parent() {
    let events = json!({"intro":[{"op":"do","actions":[{"op":"move","entity":"player","at":[16,40]}]},{"op":"wait","ticks":3},{"op":"choice","text":"Accept?","options":[{"id":"yes","text":"Yes","next":3}]},{"op":"battle","fighters":[{"id":"hero","name":"Hero","hp":10,"attack":9,"speed":5},{"id":"bot","name":"Bot","hp":4,"attack":2,"enemy":true}],"victory":[{"op":"give","item":"coin","count":2}]},{"op":"return"}]});
    let (d, mut w) = fixture(json!({"gameplay":{"events":events}}));
    let pos = w.entities["player"].position;
    actions(&mut w, json!([{"op":"event_scene","event":"intro"}]));
    for _ in 0..3 {
        w.step(&Input {
            right: true,
            ..Input::default()
        })
        .unwrap();
    }
    assert_eq!(w.entities["player"].position, Vec2::pixels(16, 40));
    assert_eq!(w.waiting().unwrap()["kind"], "choice");
    let path = d.path().join("save.json");
    w.save(&path).unwrap();
    let mut resumed = World::with_save(w.project.clone(), Some(&path)).unwrap();
    w.choose("yes").unwrap();
    resumed.choose("yes").unwrap();
    assert_eq!(w.waiting(), resumed.waiting());
    let battle = d.path().join("battle.json");
    resumed.save(&battle).unwrap();
    let mut second = World::with_save(w.project.clone(), Some(&battle)).unwrap();
    for game in [&mut w, &mut resumed, &mut second] {
        game.choose("attack_bot").unwrap();
        assert!(game.waiting().is_none());
        assert_eq!(game.entities["player"].position, pos);
        assert_eq!(game.inventory("coin"), 2);
    }
    assert_eq!(w.flatland, resumed.flatland);
    assert_eq!(w.flatland, second.flatland);
}
#[test]
fn invalid_choices_and_event_cycles_are_atomic() {
    let (_d, mut w) = fixture(
        json!({"gameplay":{"events":{"choice":[{"op":"choice","text":"?","options":[{"id":"ok","text":"OK","next":1}]},{"op":"return"}],"loop":[{"op":"do","actions":[{"op":"give","item":"coin","count":1}]},{"op":"jump","next":0}]}}}),
    );
    actions(&mut w, json!([{"op":"event_scene","event":"choice"}]));
    let before = w.snapshot();
    assert!(w.choose("bad").is_err());
    assert_eq!(before, w.snapshot());
    w.choose("ok").unwrap();
    let before = w.snapshot();
    let a = serde_json::from_value::<Vec<Action>>(json!([{"op":"event_scene","event":"loop"}]))
        .unwrap();
    assert!(w.command(&a).is_err());
    assert_eq!(before, w.snapshot());
}
#[test]
fn rng_repeats_after_resume_and_lua_fault_restores_it() {
    let (d, mut w) = fixture(json!({}));
    actions(
        &mut w,
        json!([{"op":"roll","key":"test.roll","min":1,"max":20}]),
    );
    let path = d.path().join("rng.json");
    w.save(&path).unwrap();
    let mut r = World::with_save(w.project.clone(), Some(&path)).unwrap();
    for _ in 0..8 {
        for game in [&mut w, &mut r] {
            actions(
                game,
                json!([{"op":"roll","key":"test.roll","min":1,"max":20}]),
            );
        }
        assert_eq!(w.state.values, r.state.values);
    }
    w.project.scripts.insert(
        "room".into(),
        "return function(c) local x=random(1,9); error('fail') end".into(),
    );
    let before = w.snapshot();
    assert!(w.step(&Input::default()).is_err());
    assert_eq!(w.snapshot(), before);
}
#[test]
fn projectile_hits_swept_target_but_not_through_wall_or_plane() {
    let map = json!({"cell":16,"rows":["......","......","......","......"],"tiles":{".":{"color":[0,0,0,255]},"#":{"solid":true,"color":[90,90,90,255]}}});
    let (_d, mut w) = fixture(json!({"map":map}));
    w.face("player", [1, 0]).unwrap();
    actions(
        &mut w,
        json!([{"op":"attack","entity":"player","attack":"shot"}]),
    );
    assert!(
        w.snapshot()
            .entities
            .iter()
            .any(|e| e.components.contains(&"projectile".into()))
    );
    for _ in 0..20 {
        w.step(&Input::default()).unwrap();
    }
    assert_eq!(w.entities["enemy"].actor.hp, Some(2));
    let map = json!({"cell":16,"rows":["......","..#...","......","......"],"tiles":{".":{"color":[0,0,0,255]},"#":{"solid":true,"color":[90,90,90,255],"planes":[0]}}});
    let (_d, mut blocked) = fixture(json!({"map":map}));
    blocked.face("player", [1, 0]).unwrap();
    actions(
        &mut blocked,
        json!([{"op":"attack","entity":"player","attack":"shot"}]),
    );
    for _ in 0..20 {
        blocked.step(&Input::default()).unwrap();
    }
    assert_eq!(blocked.entities["enemy"].actor.hp, Some(5));
    actions(
        &mut blocked,
        json!([{"op":"plane","entity":"player","plane":1},{"op":"elevate","entity":"player","z":8}]),
    );
    actions(
        &mut blocked,
        json!([{"op":"attack","entity":"player","attack":"shot"}]),
    );
    for _ in 0..20 {
        blocked.step(&Input::default()).unwrap();
    }
    assert_eq!(blocked.entities["enemy"].actor.hp, Some(5));
    assert_eq!(
        blocked
            .snapshot()
            .entities
            .iter()
            .find(|e| e.id == "player")
            .unwrap()
            .flatland
            .as_ref()
            .unwrap()["z"],
        8
    );
}
#[test]
fn patch_revision_conflicts_and_invalid_changes_preserve_authored_bytes() {
    let (d, w) = fixture(json!({}));
    let before = fs::read(d.path().join("room.json5")).unwrap();
    let revision = ge4g_project::patch::inspect(&w.project, "room.json5", "entities/enemy")
        .unwrap()["revision"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        ge4g_project::patch::apply(
            &w.project,
            "room.json5",
            "entities/enemy",
            "old",
            json!({"flatland":{"hp":9}})
        )
        .is_err()
    );
    assert!(
        ge4g_project::patch::apply(
            &w.project,
            "room.json5",
            "entities/enemy",
            &revision,
            json!({"flatland":{"hp":-1}})
        )
        .is_err()
    );
    assert_eq!(fs::read(d.path().join("room.json5")).unwrap(), before);
    ge4g_project::patch::apply(
        &w.project,
        "room.json5",
        "entities/enemy",
        &revision,
        json!({"flatland":{"hp":9}}),
    )
    .unwrap();
    assert_eq!(
        World::new(Project::load(d.path()).unwrap())
            .unwrap()
            .entities["enemy"]
            .actor
            .hp,
        Some(9)
    );
}
#[test]
fn replay_command_and_failed_tick_roll_back_together() {
    let (_d, mut w) = fixture(json!({}));
    w.project.scripts.insert(
        "room".into(),
        "return function(ctx) if ctx.event=='tick' then error('bad tick') end return {} end".into(),
    );
    let replay:ge4g_project::Replay=serde_json::from_value(json!({"schema_version":2,"ticks":1,"inputs":[],"commands":[{"tick":0,"op":"do","actions":[{"op":"give","item":"coin","count":2}]}]})).unwrap();
    let before = w.snapshot();
    assert!(w.step_replay(&replay).is_err());
    assert_eq!(w.snapshot(), before);
}
#[test]
fn full_signal_yard_corpus_is_deterministic_and_saves_at_every_choice() {
    let project = Project::load(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/flatland_signal_yard"),
    )
    .unwrap();
    let replay = ge4g_project::Replay::load(&project.root.join("replays/journey.json")).unwrap();
    let mut a = World::new(project.clone()).unwrap();
    let mut b = World::new(project).unwrap();
    let d = tempfile::tempdir().unwrap();
    for _ in 0..replay.ticks {
        if b.waiting().is_some() {
            let path = d.path().join("save.json");
            b.save(&path).unwrap();
            b = World::with_save(b.project.clone(), Some(&path)).unwrap();
        }
        a.step_replay(&replay).unwrap();
        b.step_replay(&replay).unwrap();
        assert_eq!(a.state.values, b.state.values);
        assert_eq!(
            serde_json::to_value(&a.entities).unwrap(),
            serde_json::to_value(&b.entities).unwrap()
        );
        assert_eq!(a.flatland, b.flatland);
    }
    assert_eq!(a.state.values["yard.complete"], true);
    assert_eq!(a.inventory("coin"), 13);
    assert!(a.waiting().is_none());
    assert_eq!(a.entities["player"].position, Vec2::pixels(80, 240));
    assert_eq!(
        a.flatland.systems.as_ref().unwrap().quests["signal"].status,
        ge4g_project::gameplay::QuestStatus::Completed
    );
}
#[test]
fn cutscene_clip_advances_while_parent_is_frozen_and_restores_atlas() {
    let project = Project::load(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/flatland_signal_yard"),
    )
    .unwrap();
    let mut w = World::new(project).unwrap();
    for choice in ["continue", "accept", "continue"] {
        w.choose(choice).unwrap();
    }
    let actors = serde_json::to_value(&w.entities).unwrap();
    let camera = w.snapshot().camera;
    actions(&mut w, json!([{"op":"event_scene","event":"cutscene"}]));
    let first = w
        .snapshot()
        .entities
        .iter()
        .find(|e| e.id == "guide")
        .unwrap()
        .texture
        .clone();
    for _ in 0..8 {
        w.step(&Input {
            right: true,
            ..Input::default()
        })
        .unwrap();
    }
    let next = w
        .snapshot()
        .entities
        .iter()
        .find(|e| e.id == "guide")
        .unwrap()
        .texture
        .clone();
    assert_ne!(first, next);
    assert_eq!(w.entities["player"].position, Vec2::pixels(48, 80));
    for _ in 8..45 {
        w.step(&Input::default()).unwrap();
    }
    w.choose("continue").unwrap();
    assert_eq!(serde_json::to_value(&w.entities).unwrap(), actors);
    assert_eq!(w.snapshot().camera, camera);
    assert!(
        w.snapshot()
            .entities
            .iter()
            .find(|e| e.id == "guide")
            .unwrap()
            .flatland
            .as_ref()
            .unwrap()
            .get("atlas")
            .is_some()
    );
}
#[test]
fn closing_map_on_an_actor_and_corrupt_event_parent_are_rejected() {
    let map = json!({"cell":16,"rows":["......","......","......","......"],"tiles":{".":{"color":[0,0,0,255]},"#":{"solid":true,"color":[90,90,90,255]}}});
    let (d, mut w) = fixture(
        json!({"map":map,"gameplay":{"events":{"test":[{"op":"wait","ticks":10},{"op":"return"}]}}}),
    );
    let before = w.snapshot();
    let patch: Vec<Action> =
        serde_json::from_value(json!([{"op":"map_patch","at":[1,1],"tile":"#"}])).unwrap();
    assert!(w.command(&patch).is_err());
    assert_eq!(w.snapshot(), before);
    actions(&mut w, json!([{"op":"event_scene","event":"test"}]));
    let path = d.path().join("bad.json");
    w.save(&path).unwrap();
    let mut save: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    save["runtime"]["systems"]["events"][0]["parent_elevation"] = json!({"player":99999});
    fs::write(&path, save.to_string()).unwrap();
    assert!(World::with_save(w.project.clone(), Some(&path)).is_err());
}

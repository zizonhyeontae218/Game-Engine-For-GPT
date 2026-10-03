use ge4g_core::{Input, Vec2};
use ge4g_project::Project;
use ge4g_runtime::World;
use serde_json::{Value, json};
use std::{fs, path::Path};

fn fixture(entities: Value, rules: Value, script: Option<&str>) -> (tempfile::TempDir, Project) {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("ge4g.toml"),
        r#"schema_version=2
name="test"
start_scene="room"
[window]
width=160
height=100
[scenes]
room="room.json5"
[state."test.count"]
kind="integer"
default=0
"#,
    )
    .unwrap();
    let mut scene = json!({"schema_version":2,"id":"room","background":[0,0,0,255],"spawns":{"start":[0,0]},"entities":entities,"rules":rules});
    if let Some(s) = script {
        fs::write(dir.path().join("script.lua"), s).unwrap();
        scene["script"] = json!("script.lua");
    }
    fs::write(dir.path().join("room.json5"), scene.to_string()).unwrap();
    let project = Project::load(dir.path()).unwrap();
    (dir, project)
}
fn player() -> Value {
    json!({"id":"player","position":[0,0],"size":[10,10],"player":{"speed":600},"flatland":{"hp":3}})
}
fn body(id: &str, x: i64, mode: &str) -> Value {
    json!({"id":id,"position":[x,0],"size":[10,10],"flatland":{"body":mode}})
}
fn right() -> Input {
    Input {
        right: true,
        ..Input::default()
    }
}
#[test]
fn transactional_push_chains_and_pass_fixed_policies() {
    let (_dir, p) = fixture(
        json!([
            player(),
            body("a", 10, "push"),
            body("b", 20, "push"),
            body("wall", 30, "fixed")
        ]),
        json!([]),
        None,
    );
    let mut w = World::new(p).unwrap();
    w.step(&right()).unwrap();
    assert_eq!(w.entities["player"].position, Vec2::pixels(0, 0));
    assert_eq!(w.entities["a"].position, Vec2::pixels(10, 0));
    assert_eq!(w.entities["b"].position, Vec2::pixels(20, 0));
    let (_dir, p) = fixture(
        json!([
            player(),
            body("a", 10, "push"),
            body("b", 20, "push"),
            body("decoration", 30, "pass")
        ]),
        json!([]),
        None,
    );
    let mut w = World::new(p).unwrap();
    w.step(&right()).unwrap();
    assert_eq!(w.entities["player"].position, Vec2::pixels(10, 0));
    assert_eq!(w.entities["a"].position, Vec2::pixels(20, 0));
    assert_eq!(w.entities["b"].position, Vec2::pixels(30, 0));
    w.face("player", [0, -4]).unwrap();
    w.step(&Input::default()).unwrap();
    assert_eq!(w.facing("player").unwrap(), [0, -1]);
    assert_eq!(w.toward("player", "a").unwrap(), [1, 0]);
    assert!(!w.in_front("player", "a", 100).unwrap());
}
#[test]
fn lua_failures_roll_back_motion_state_tick_and_events() {
    for script in [
        "return function(c) return {{op='add',key='test.count',value=1.5}} end",
        "return function(c) while true do end end",
        "return function(c) return {{op='add',key='test.count',value=1},{op='remove',entity='missing'}} end",
    ] {
        let (dir, p) = fixture(json!([player()]), json!([]), None);
        let mut w = World::new(p).unwrap();
        w.project.scripts.insert("room".into(), script.into());
        let before = w.snapshot();
        let error = w.step(&right()).unwrap_err().to_string();
        assert!(
            error.contains("Lua") || error.contains("missing"),
            "{error}"
        );
        assert_eq!(w.snapshot(), before);
        drop(dir);
    }
    let (_dir, p) = fixture(
        json!([player()]),
        json!([]),
        Some(
            "return function(c) assert(io==nil and os==nil and dofile==nil and loadfile==nil and require==nil and print==nil and pcall==nil and collectgarbage==nil); if c.event=='tick' then return {{op='add',key='test.count',value=1}} end return {} end",
        ),
    );
    let mut w = World::new(p).unwrap();
    w.step(&right()).unwrap();
    assert_eq!(w.state.values["test.count"], 1);
}
#[test]
fn conditions_popup_and_once_rewards_survive_exact_resume() {
    let rules = json!([{"id":"reward","on":"tick","once":true,"when":{"op":"state","key":"test.count","eq":0},"actions":[{"op":"add","key":"test.count","value":7},{"op":"say","text":"Quest completed"}]}]);
    let (dir, p) = fixture(json!([player()]), rules, None);
    let mut w = World::new(p.clone()).unwrap();
    w.step(&right()).unwrap();
    w.face("player", [-1, 0]).unwrap();
    let save = dir.path().join("resume.json");
    w.save(&save).unwrap();
    let mut resumed = World::with_save(p, Some(&save)).unwrap();
    assert_eq!(resumed.tick, 1);
    assert_eq!(resumed.entities["player"].position, Vec2::pixels(10, 0));
    assert_eq!(resumed.facing("player").unwrap(), [-1, 0]);
    assert_eq!(resumed.flatland, w.flatland);
    w.step(&right()).unwrap();
    resumed.step(&right()).unwrap();
    assert_eq!(w.state.values, resumed.state.values);
    assert_eq!(resumed.state.values["test.count"], 7);
    assert_eq!(
        w.entities["player"].position,
        resumed.entities["player"].position
    );
    assert_eq!(
        resumed.flatland.popup.as_ref().unwrap().text,
        "Quest completed"
    );
}
fn pacman() -> Project {
    Project::load(Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/flatland_pacman"
    )))
    .unwrap()
}
#[test]
fn maze_walls_are_not_entities_and_ghosts_move_deterministically() {
    let p = pacman();
    assert!(p.scenes["maze"].map.is_some());
    assert!(
        p.scenes["maze"]
            .entities
            .iter()
            .all(|e| e.flatland.is_some())
    );
    let mut a = World::new(p.clone()).unwrap();
    let mut b = World::new(p).unwrap();
    let initial = a.entities["blinky"].position;
    for _ in 0..30 {
        let i = Input {
            left: true,
            ..Input::default()
        };
        a.step(&i).unwrap();
        b.step(&i).unwrap();
    }
    assert_eq!(a.snapshot(), b.snapshot());
    assert_ne!(a.entities["blinky"].position, initial);
    assert_eq!(a.state.values["game.score"], 30);
    assert_eq!(a.state.values["game.pickups"], 3);
    assert_eq!(a.facing("player").unwrap(), [-1, 0]);
}
#[test]
fn power_contact_timer_expiry_lives_and_clear_have_real_outcomes() {
    let mut w = World::new(pacman()).unwrap();
    let power = w
        .entities
        .iter()
        .find(|(_, e)| e.spec.tags.iter().any(|t| t == "power"))
        .unwrap()
        .1
        .position;
    w.entities.get_mut("player").unwrap().position = power;
    w.step(&Input::default()).unwrap();
    assert_eq!(w.state.values["game.score"], 50);
    assert!(w.flatland.timers["power"] > w.tick);
    let player = w.entities["player"].position;
    w.entities.get_mut("blinky").unwrap().position = player;
    w.step(&Input::default()).unwrap();
    assert_eq!(w.state.values["game.score"], 250);
    assert_eq!(w.entities["player"].actor.hp, Some(3));
    w.flatland.timers.insert("power".into(), w.tick + 1);
    w.entities.get_mut("player").unwrap().actor.hp = Some(1);
    let player = w.entities["player"].position;
    w.entities.get_mut("blinky").unwrap().position = player;
    w.step(&Input::default()).unwrap();
    assert_eq!(w.entities["player"].actor.hp, Some(0));
    assert!(w.flatland.stopped);
    assert_eq!(w.state.values["game.result"], "lost");
    let mut w = World::new(pacman()).unwrap();
    w.entities
        .retain(|_, e| !e.spec.tags.iter().any(|t| t == "food"));
    w.step(&Input::default()).unwrap();
    assert!(w.flatland.stopped);
    assert_eq!(w.state.values["game.result"], "won");
}

#[test]
fn every_authored_food_cell_is_reachable_and_grid_turns_wait_for_intersections() {
    use std::collections::{BTreeSet, VecDeque};
    let p = pacman();
    let map = p.scenes["maze"].map.as_ref().unwrap();
    let mut seen = BTreeSet::from([(9i64, 15i64)]);
    let mut queue = VecDeque::from([(9i64, 15i64)]);
    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let at = (x + dx, y + dy);
            if map.open(at.0, at.1) && seen.insert(at) {
                queue.push_back(at);
            }
        }
    }
    for e in p.scenes["maze"]
        .entities
        .iter()
        .filter(|e| e.tags.iter().any(|t| t == "food"))
    {
        assert!(
            seen.contains(&(e.position[0] / 20, e.position[1] / 20)),
            "unreachable food {}",
            e.id
        );
    }
    let mut w = World::new(p).unwrap();
    w.step(&Input {
        left: true,
        ..Input::default()
    })
    .unwrap();
    let y = w.entities["player"].position.y;
    w.step(&Input {
        up: true,
        ..Input::default()
    })
    .unwrap();
    assert_eq!(w.entities["player"].position.y, y);
    // At x=160 the queued up turn is blocked, so the controller continues left.
    for _ in 0..10 {
        w.step(&Input::default()).unwrap();
    }
    assert_eq!(w.entities["player"].position, Vec2::pixels(160, 300));
}

#[test]
fn a_corrupted_resume_cannot_remove_hp_or_place_the_player_inside_a_wall() {
    let p = pacman();
    let mut w = World::new(p.clone()).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("save.json");
    w.save(&path).unwrap();
    let original: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    for bad in [json!(null), json!(-4)] {
        let mut edited = original.clone();
        edited["actors"]["player"]["actor"]["hp"] = bad;
        fs::write(&path, edited.to_string()).unwrap();
        assert!(World::with_save(p.clone(), Some(&path)).is_err());
    }
    let mut edited = original;
    edited["actors"]["player"]["position"] = json!({"x":0,"y":0});
    fs::write(&path, edited.to_string()).unwrap();
    let error = World::with_save(p, Some(&path))
        .err()
        .expect("wall save must fail");
    assert!(error.to_string().contains("wall"));
}

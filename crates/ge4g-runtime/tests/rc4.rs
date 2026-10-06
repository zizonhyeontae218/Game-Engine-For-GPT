use ge4g_core::{Input, Vec2};
use ge4g_project::{Project, flatland::Action};
use ge4g_render2d::render;
use ge4g_runtime::World;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
fn project() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/flatland_harbor")
}
fn world() -> World {
    let mut w = World::new(Project::load(&project()).unwrap()).unwrap();
    w.choose("continue").unwrap();
    w
}
fn command(w: &mut World, actions: Value) {
    w.command(&serde_json::from_value::<Vec<Action>>(actions).unwrap())
        .unwrap();
}
fn state(w: &World) -> Value {
    let mut v = serde_json::to_value(w.snapshot()).unwrap();
    v.as_object_mut().unwrap().remove("camera");
    v.as_object_mut().unwrap().remove("events");
    v.as_object_mut().unwrap().remove("events_dropped");
    let s = &mut v["flatland"]["systems"];
    for key in ["view", "gameplay_view", "view_initialized"] {
        s.as_object_mut().unwrap().remove(key);
    }
    v
}
fn settle(w: &mut World) {
    while w.waiting().unwrap()["presentation_locked"] == true {
        w.step(&Input::default()).unwrap();
    }
}
fn battle(w: &mut World) {
    command(w, json!([{"op":"event_scene","event":"battle"}]));
}
fn finish(w: &mut World) {
    while w.waiting().unwrap()["result"].is_null() {
        settle(w);
        w.choose("move:scratch:rival").unwrap();
    }
    settle(w);
    w.choose("battle_continue").unwrap();
}
#[test]
fn view_is_simulation_invariant_with_motion_and_collision() {
    let mut w = world();
    w.step(&Input {
        right: true,
        ..Input::default()
    })
    .unwrap();
    let before = state(&w);
    for id in ["depth", "alternate", "top"] {
        command(&mut w, json!([{"op":"view","mode":id}]));
        assert_eq!(before, state(&w));
    }
    let mut a = world();
    let mut b = world();
    command(&mut b, json!([{"op":"view","mode":"depth"}]));
    for _ in 0..40 {
        let i = Input {
            left: true,
            ..Input::default()
        };
        a.step(&i).unwrap();
        b.step(&i).unwrap();
        assert_eq!(state(&a), state(&b));
    }
}
#[test]
fn persistent_view_survives_goto_scoped_camera_save_and_scene_without_local_preset() {
    let mut w = world();
    command(&mut w, json!([{"op":"view","mode":"depth"}]));
    command(&mut w, json!([{"op":"event_scene","event":"tour"}]));
    for _ in 0..46 {
        assert_eq!(
            render(&w.project, &w.snapshot(), false).unwrap(),
            render(&w.project, &w.render_snapshot(), false).unwrap()
        );
        w.step(&Input::default()).unwrap();
    }
    assert_eq!(w.snapshot().flatland.unwrap()["systems"]["view"], "depth");
    command(
        &mut w,
        json!([{"op":"goto","scene":"inside_library","spawn":"entry"}]),
    );
    assert_eq!(w.snapshot().flatland.unwrap()["systems"]["view"], "depth");
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("보관.json");
    w.save(&p).unwrap();
    let mut r = World::with_save(w.project.clone(), Some(&p)).unwrap();
    assert_eq!(r.waiting(), w.waiting());
    assert_eq!(r.snapshot().flatland.unwrap()["systems"]["view"], "depth");
    command(
        &mut r,
        json!([{"op":"goto","scene":"항구","spawn":"entry"}]),
    );
    assert_eq!(r.snapshot().flatland.unwrap()["systems"]["view"], "depth");
}
#[test]
fn step_input_newest_wins_buffers_and_release_does_not_add_a_cell() {
    let mut w = world();
    let start = w.entities["player"].position;
    w.step(&Input {
        right: true,
        ..Input::default()
    })
    .unwrap();
    for _ in 0..15 {
        w.step(&Input {
            right: true,
            up: true,
            ..Input::default()
        })
        .unwrap();
    }
    assert_eq!(w.entities["player"].actor.direction, [0, -1]);
    for _ in 0..20 {
        w.step(&Input::default()).unwrap();
    }
    assert_eq!(
        w.entities["player"].position,
        Vec2::pixels(start.x / 60 + 16, start.y / 60 - 16)
    );
    assert_eq!(w.entities["player"].actor.direction, [0, 0]);
    w.step(&Input {
        left: true,
        right: true,
        direction: Some([-1, 0]),
        ..Input::default()
    })
    .unwrap();
    assert_eq!(w.entities["player"].actor.direction, [-1, 0]);
}
#[test]
fn persistent_hp_pp_resume_fx_lock_rewards_and_fainted_reset() {
    let mut w = world();
    battle(&mut w);
    w.choose("move:scratch:rival").unwrap();
    assert_eq!(w.waiting().unwrap()["presentation_locked"], true);
    assert!(w.choose("move:scratch:rival").is_err());
    let wait = w.waiting().unwrap();
    assert_eq!(wait["fighters"][0]["moves"][0]["pp"], 24);
    assert_eq!(wait["fighters"][1]["display_hp"], 34);
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("battle.json");
    w.save(&p).unwrap();
    let mut r = World::with_save(w.project.clone(), Some(&p)).unwrap();
    assert_eq!(w.waiting(), r.waiting());
    finish(&mut w);
    finish(&mut r);
    assert_eq!(state(&w), state(&r));
    assert_eq!(w.inventory("potion"), r.inventory("potion"));
    let roster = w.snapshot().flatland.unwrap()["systems"]["combatants"].clone();
    assert!(roster["seedling"]["current_hp"].as_i64().unwrap() < 38);
    w.save(&p).unwrap();
    let mut loaded = World::with_save(w.project.clone(), Some(&p)).unwrap();
    assert_eq!(
        roster,
        loaded.snapshot().flatland.unwrap()["systems"]["combatants"]
    );
    battle(&mut loaded);
    assert_eq!(
        loaded.waiting().unwrap()["fighters"][0]["hp"],
        roster["seedling"]["current_hp"]
    );
    assert_eq!(
        loaded.waiting().unwrap()["fighters"][0]["moves"][0]["pp"],
        roster["seedling"]["remaining_pp"]["scratch"]
    );
    // Explicit healing API; no implicit encounter healing.
    command(
        &mut w,
        json!([{"op":"combatant_heal","combatant":"seedling","amount":1}]),
    );
    assert_eq!(
        w.snapshot().flatland.unwrap()["systems"]["combatants"]["seedling"]["current_hp"]
            .as_i64()
            .unwrap(),
        roster["seedling"]["current_hp"].as_i64().unwrap() + 1
    );
    command(
        &mut w,
        json!([{"op":"combatant_reset","combatant":"seedling"}]),
    );
    assert_eq!(
        w.snapshot().flatland.unwrap()["systems"]["combatants"]["seedling"]["remaining_pp"]["scratch"],
        25
    );
}
#[test]
fn cosmetic_skip_does_not_repeat_damage_pp_or_rng() {
    let mut a = world();
    let mut b = world();
    battle(&mut a);
    battle(&mut b);
    a.choose("move:pulse:rival").unwrap();
    b.choose("move:pulse:rival").unwrap();
    let before = a.snapshot().flatland.unwrap()["systems"]["combatants"].clone();
    a.skip_event_wait().unwrap();
    a.skip_event_wait().unwrap();
    settle(&mut b);
    assert_eq!(
        before,
        a.snapshot().flatland.unwrap()["systems"]["combatants"]
    );
    assert_eq!(
        before,
        b.snapshot().flatland.unwrap()["systems"]["combatants"]
    );
    assert_eq!(
        a.snapshot().flatland.unwrap()["systems"]["rng"],
        b.snapshot().flatland.unwrap()["systems"]["rng"]
    );
}
#[test]
fn minimal_buildings_and_actor_render_top_depth_and_follow_anchor() {
    let mut w = world();
    let before = w.entities["player"].position;
    let top = render(&w.project, &w.render_snapshot(), false).unwrap();
    command(&mut w, json!([{"op":"view","mode":"depth"}]));
    let depth = render(&w.project, &w.render_snapshot(), false).unwrap();
    assert_ne!(top, depth);
    assert_eq!(before, w.entities["player"].position);
    assert_eq!(depth, render(&w.project, &w.snapshot(), false).unwrap());
}
#[test]
fn fainted_combatant_stays_zero_until_explicit_reset_and_rewards_are_once() {
    let mut w = world();
    battle(&mut w);
    while w.waiting().unwrap()["result"].is_null() {
        settle(&mut w);
        w.choose("guard").unwrap();
    }
    settle(&mut w);
    assert_eq!(w.waiting().unwrap()["result"], false);
    w.choose("battle_continue").unwrap();
    assert_eq!(w.inventory("potion"), 3);
    battle(&mut w);
    assert_eq!(w.waiting().unwrap()["fighters"][0]["hp"], 0);
    assert_eq!(w.waiting().unwrap()["result"], false);
    w.choose("battle_continue").unwrap();
    command(
        &mut w,
        json!([{"op":"combatant_reset","combatant":"seedling"}]),
    );
    battle(&mut w);
    assert_eq!(w.waiting().unwrap()["fighters"][0]["hp"], 38);
    finish(&mut w);
    assert_eq!(w.inventory("potion"), 4);
    assert!(w.choose("battle_continue").is_err());
    assert_eq!(w.inventory("potion"), 4);
}

use ge4g_core::{Input, Vec2};
use ge4g_project::{Project, Replay, flatland::Action};
use ge4g_render2d::render;
use ge4g_runtime::World;
use serde_json::json;
use std::path::{Path, PathBuf};
fn town() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/flatland_nuvema")
}
fn world() -> World {
    let mut w = World::new(Project::load(&town()).unwrap()).unwrap();
    w.choose("continue").unwrap();
    w
}
fn command(w: &mut World, v: serde_json::Value) {
    let actions: Vec<Action> = serde_json::from_value(v).unwrap();
    w.command(&actions).unwrap();
}
#[test]
fn walking_releases_at_one_cell_faces_blocked_objects_and_uses_authored_pace() {
    let mut w = world();
    let start = w.entities["player"].position;
    w.step(&Input {
        down: true,
        ..Input::default()
    })
    .unwrap();
    for _ in 0..90 {
        w.step(&Input::default()).unwrap();
    }
    assert_eq!(
        w.entities["player"].position,
        Vec2 {
            x: start.x,
            y: start.y + 16 * 60
        }
    );
    assert_eq!(w.entities["player"].actor.direction, [0, 0]);
    command(
        &mut w,
        json!([{"op":"move","entity":"player","at":[224,256]}]),
    );
    w.step(&Input {
        up: true,
        ..Input::default()
    })
    .unwrap();
    assert_eq!(w.entities["player"].position, Vec2::pixels(224, 256));
    assert_eq!(w.facing("player").unwrap(), [0, -1]);
    command(
        &mut w,
        json!([{"op":"move","entity":"player","at":[240,320]},{"op":"pace","entity":"player","speed":72}]),
    );
    w.step(&Input {
        down: true,
        ..Input::default()
    })
    .unwrap();
    assert_eq!(w.entities["player"].position.y, 320 * 60 + 72);
}
#[test]
fn view_choice_retains_projection_plane_and_elevation_and_rejects_occupied_lower_plane() {
    let mut w = world();
    let top = render(&w.project, &w.snapshot(), false).unwrap();
    command(&mut w, json!([{"op":"event_scene","event":"view"}]));
    w.choose("depth").unwrap();
    let depth = render(&w.project, &w.snapshot(), false).unwrap();
    assert_ne!(top.rgba, depth.rgba);
    assert_eq!(
        depth,
        render(&w.project, &w.render_snapshot(), false).unwrap()
    );
    command(&mut w, json!([{"op":"event_scene","event":"view"}]));
    w.choose("balcony").unwrap();
    assert_eq!(
        w.snapshot()
            .entities
            .iter()
            .find(|e| e.id == "player")
            .unwrap()
            .flatland
            .as_ref()
            .unwrap()["plane"],
        1
    );
    command(
        &mut w,
        json!([{"op":"move","entity":"player","at":[304,144]}]),
    );
    let before = w.snapshot();
    let blocked: Vec<Action> =
        serde_json::from_value(json!([{"op":"plane","entity":"player","plane":0}])).unwrap();
    assert!(w.command(&blocked).is_err());
    assert_eq!(before, w.snapshot());
    let dir = tempfile::tempdir().unwrap();
    let save = dir.path().join("save.json");
    w.save(&save).unwrap();
    let resumed = World::with_save(Project::load(&town()).unwrap(), Some(&save)).unwrap();
    assert_eq!(
        render(&w.project, &w.snapshot(), false).unwrap(),
        render(&resumed.project, &resumed.snapshot(), false).unwrap()
    );
}
#[test]
fn battle_has_native_sprites_pp_hit_frames_and_exact_resume_through_result() {
    let mut w = world();
    command(&mut w, json!([{"op":"event_scene","event":"battle_ember"}]));
    assert_eq!(w.waiting().unwrap()["kind"], "battle");
    let first = render(&w.project, &w.snapshot(), false).unwrap();
    assert_eq!(
        first,
        render(&w.project, &w.render_snapshot(), false).unwrap()
    );
    assert!(w.choose("attack_rival").is_err()); // no PP bypass for authored moves
    w.choose("move:burst:rival").unwrap();
    let wait = w.waiting().unwrap();
    assert_eq!(wait["fighters"][0]["moves"][1]["pp"], 9);
    let hit = render(&w.project, &w.snapshot(), false).unwrap();
    assert_ne!(first, hit);
    let dir = tempfile::tempdir().unwrap();
    let save = dir.path().join("save.json");
    w.save(&save).unwrap();
    let mut resumed = World::with_save(Project::load(&town()).unwrap(), Some(&save)).unwrap();
    assert_eq!(w.waiting(), resumed.waiting());
    assert_eq!(
        hit,
        render(&resumed.project, &resumed.snapshot(), false).unwrap()
    );
    for _ in 0..30 {
        w.step(&Input::default()).unwrap();
        resumed.step(&Input::default()).unwrap();
    }
    assert_eq!(w.waiting(), resumed.waiting());
    assert_ne!(hit, render(&w.project, &w.snapshot(), false).unwrap());
    while w.waiting().unwrap()["result"].is_null() {
        w.choose("move:burst:rival").unwrap();
    }
    assert_eq!(w.waiting().unwrap()["result"], true);
    w.save(&save).unwrap();
    resumed = World::with_save(Project::load(&town()).unwrap(), Some(&save)).unwrap();
    resumed.choose("battle_continue").unwrap();
    assert!(resumed.waiting().is_none());
    assert_eq!(resumed.state.values["town.won"], true);
}
#[test]
fn town_journey_completes_every_station_and_minimal_frames_match_full_observation() {
    let mut w = World::new(Project::load(&town()).unwrap()).unwrap();
    let r = Replay::load(&town().join("replays/journey.json")).unwrap();
    while w.tick < r.ticks {
        w.step_replay(&r).unwrap();
        if w.tick.is_multiple_of(60) || w.waiting().is_some() {
            assert_eq!(
                render(&w.project, &w.snapshot(), false).unwrap(),
                render(&w.project, &w.render_snapshot(), false).unwrap()
            );
        }
    }
    assert_eq!(w.state.values["town.completed"], true);
    assert_eq!(w.flatland.systems.as_ref().unwrap().inventory["badge"], 1);
}

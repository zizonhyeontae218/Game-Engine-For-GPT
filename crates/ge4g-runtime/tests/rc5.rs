use ge4g_core::{Input, Vec2};
use ge4g_project::{
    Project,
    flatland::{Action, BodyMode},
};
use ge4g_render2d::render;
use ge4g_runtime::World;
use serde_json::{Value, json};
use std::path::Path;
fn world() -> World {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/flatland_harbor");
    let mut w = World::new(Project::load(&path).unwrap()).unwrap();
    w.choose("continue").unwrap();
    w
}
fn command(w: &mut World, a: Value) {
    w.command(&serde_json::from_value::<Vec<Action>>(a).unwrap())
        .unwrap();
}
#[test]
fn semantic_footprint_defaults_solid_and_view_cannot_grant_roof_access() {
    let mut w = world();
    assert_eq!(w.entities["workshop"].spec.size, [48, 32]);
    assert_eq!(
        w.entities["workshop"].spec.flatland.as_ref().unwrap().body,
        BodyMode::Fixed
    );
    command(
        &mut w,
        json!([{"op":"move","entity":"player","at":[336,272]}]),
    );
    for mode in ["top", "depth", "alternate", "top"] {
        command(&mut w, json!([{"op":"view","mode":mode}]));
        for _ in 0..30 {
            w.step(&Input {
                down: true,
                ..Input::default()
            })
            .unwrap();
        }
        assert_eq!(w.entities["player"].position, Vec2::pixels(336, 272));
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("collision.json");
    w.save(&path).unwrap();
    let mut r = World::with_save(w.project.clone(), Some(&path)).unwrap();
    for _ in 0..30 {
        r.step(&Input {
            down: true,
            ..Input::default()
        })
        .unwrap();
    }
    assert_eq!(r.entities["player"].position, w.entities["player"].position);
    for scene in ["inside_library", "항구"] {
        command(&mut r, json!([{"op":"goto","scene":scene,"spawn":"entry"}]));
    }
    command(
        &mut r,
        json!([{"op":"move","entity":"player","at":[336,272]}]),
    );
    r.step(&Input {
        down: true,
        ..Input::default()
    })
    .unwrap();
    assert_eq!(r.entities["player"].position, Vec2::pixels(336, 272));
}
#[test]
fn truthful_building_semantics_reject_unimplemented_directions() {
    let w = world();
    let mut b = w.entities["workshop"]
        .spec
        .flatland
        .as_ref()
        .unwrap()
        .building
        .clone()
        .unwrap();
    assert!(b.valid());
    for facing in ["north", "east", "west"] {
        b.facing = facing.into();
        assert!(!b.valid());
    }
}
#[test]
fn every_contact_shadow_is_below_opaque_actor_and_depth_occlusion_tracks_feet() {
    let mut w = world();
    w.project.scenes.get_mut(&w.scene).unwrap().gameplay.camera = None;
    w.project.scenes.get_mut(&w.scene).unwrap().camera = [0, 0];
    let mut player = w.entities["player"].clone();
    player.position = Vec2::pixels(100, 100);
    player.spec.sprite.as_mut().unwrap().texture = None;
    player.spec.sprite.as_mut().unwrap().color = [40, 80, 200, 255];
    let a = player.spec.flatland.as_mut().unwrap();
    a.animation = None;
    a.visual_size = None;
    a.anchor = [0, 0];
    a.hp = None;
    let mut npc = w.entities["minimal_actor"].clone();
    npc.position = Vec2::pixels(110, 100);
    npc.spec.sprite = player.spec.sprite.clone();
    npc.spec.sprite.as_mut().unwrap().color = [220, 80, 40, 255];
    w.entities.clear();
    w.entities.insert("player".into(), player);
    w.entities.insert("minimal_actor".into(), npc);
    command(&mut w, json!([{"op":"view","mode":"top"}]));
    let frame = render(&w.project, &w.snapshot(), false).unwrap();
    let at = ((115 * frame.width + 108) * 4) as usize;
    assert_eq!(&frame.rgba[at..at + 4], &[40, 80, 200, 255]);
    let mut behind = w.entities["player"].clone();
    behind.position = Vec2::pixels(110, 92);
    w.entities.insert("player".into(), behind);
    command(&mut w, json!([{"op":"view","mode":"depth"}]));
    let p = ge4g_render2d::actor_screen_anchor(&w.project, &w.snapshot(), "minimal_actor").unwrap();
    let frame = render(&w.project, &w.snapshot(), false).unwrap();
    let at = (((p[1] + 8) * i64::from(frame.width) + p[0]) * 4) as usize;
    assert_eq!(&frame.rgba[at..at + 4], &[220, 80, 40, 255]);
    w.entities.get_mut("player").unwrap().position = Vec2::pixels(110, 108);
    let frame = render(&w.project, &w.snapshot(), false).unwrap();
    let at = (((p[1] + 26) * i64::from(frame.width) + p[0]) * 4) as usize;
    assert_eq!(&frame.rgba[at..at + 4], &[40, 80, 200, 255]);
}
#[test]
fn three_actor_bubbles_resume_once_and_camera_is_scoped_and_eased() {
    let mut w = world();
    command(&mut w, json!([{"op":"view","mode":"depth"}]));
    let positions = w.entities.clone();
    let before = w.snapshot().camera;
    command(&mut w, json!([{"op":"event_scene","event":"tour"}]));
    assert_eq!(w.snapshot().camera, before);
    for _ in 0..6 {
        w.step(&Input::default()).unwrap();
    }
    assert_ne!(w.snapshot().camera, before);
    for _ in 0..6 {
        w.step(&Input::default()).unwrap();
    }
    assert_eq!(w.waiting().unwrap()["kind"], "bubble");
    assert_eq!(w.waiting().unwrap()["actor"], "minimal_actor");
    let first = w.waiting().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bubble.json");
    w.save(&path).unwrap();
    let mut r = World::with_save(w.project.clone(), Some(&path)).unwrap();
    assert_eq!(r.waiting(), w.waiting());
    for actor in positions.values() {
        assert_eq!(r.entities[&actor.spec.id].position, actor.position);
    }
    for _ in 0..3 {
        assert_eq!(r.waiting(), w.waiting());
        r.choose("continue").unwrap();
        w.choose("continue").unwrap();
    }
    assert!(r.waiting().is_none());
    assert!(r.choose("continue").is_err());
    assert!(first["id"].as_str().is_some());
    for _ in 0..12 {
        assert_eq!(
            render(&r.project, &r.snapshot(), false).unwrap(),
            render(&r.project, &r.render_snapshot(), false).unwrap()
        );
        r.step(&Input::default()).unwrap();
        w.step(&Input::default()).unwrap();
    }
    let mut a = serde_json::to_value(r.snapshot()).unwrap();
    let mut b = serde_json::to_value(w.snapshot()).unwrap();
    for value in [&mut a, &mut b] {
        value.as_object_mut().unwrap().remove("events");
        value.as_object_mut().unwrap().remove("events_dropped");
    }
    assert_eq!(a, b);
    assert_eq!(r.snapshot().camera, before);
    assert_eq!(r.snapshot().flatland.unwrap()["systems"]["view"], "depth");
}

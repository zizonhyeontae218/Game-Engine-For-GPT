//! Existing fifth legacy demo is the acceptance fixture. No new content and no
//! gameplay menu commands are used to manufacture a camera-only state change.
use ge4g_core::{SUBPIXELS, Snapshot};
use ge4g_pentomino::PluginId;
use ge4g_pentomino::p2::{CoreHost, EntityRef, ObjectRef, ReadFrame, RecordKey, Selection, Value};
use ge4g_pentomino_view::*;
use ge4g_pentomino_view_legacy::*;
use ge4g_project::Project;
use ge4g_runtime::{World, snapshot_hash};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

fn owner() -> PluginId {
    PluginId::new("fixture.fifth").unwrap()
}
fn vid() -> ViewId {
    ViewId::new("view.fifth").unwrap()
}
fn cid() -> CameraId {
    CameraId::new("camera.fifth").unwrap()
}
fn viewport() -> Viewport {
    Viewport {
        x: 0.0,
        y: 0.0,
        width: 320.0,
        height: 240.0,
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn serialized<T: serde::Serialize>(v: &T) -> Vec<u8> {
    serde_json::to_vec(v).unwrap()
}
fn fifth() -> World {
    World::new(
        Project::load(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/flatland_nuvema"),
        )
        .unwrap(),
    )
    .unwrap()
}
fn select(c: &CoreHost) -> ReadFrame {
    c.select(&Selection {
        owners: BTreeSet::from([owner()]),
        schemas: BTreeSet::new(),
        include_objects: true,
        include_history: true,
    })
    .unwrap()
}
fn local(prefix: &str, s: &str) -> String {
    format!("{prefix}{}", hash(s.as_bytes()))
}
fn entity_source(scene: &str, id: &str) -> String {
    format!("{scene}\0{id}")
}
fn import(w: &World) -> (CoreHost, ReadFrame, RecordBinding) {
    let (plugin, binding) = presentation_plugin(w, owner()).unwrap();
    let mut c = CoreHost::new(123, "existing.fifth.fixture").unwrap();
    c.install(plugin, BTreeMap::new()).unwrap();
    let r = select(&c);
    (c, r, binding)
}
fn view(r: &ReadFrame, binding: RecordBinding, s: &Snapshot) -> ViewHost {
    let mut h = ViewHost::new("existing.fifth.fixture").unwrap();
    h.install(
        Box::new(Classic2DFormat),
        ViewConfig {
            id: vid(),
            scene: r.scenes[0].reference.clone(),
            binding,
            policy: ViewPolicy::classic2d(),
        },
        r,
    )
    .unwrap();
    let t = legacy_camera_target(s, &viewport(), 100.0, 100.0, 0.0).unwrap();
    h.add_camera(CameraConfig::new(cid(), vid(), viewport(), t), r)
        .unwrap();
    h.activate(&cid()).unwrap();
    h
}
fn input(tick: u64) -> ViewInput {
    ViewInput {
        target_tick: tick,
        changes: vec![],
    }
}
fn change(tick: u64, target: CameraTarget, transition: Option<TransitionMode>) -> ViewInput {
    ViewInput {
        target_tick: tick,
        changes: vec![CameraChange {
            camera: cid(),
            target,
            transition,
        }],
    }
}
fn frame(h: &ViewHost, r: &ReadFrame) -> CameraFrame {
    h.frame(r).unwrap().cameras[0].clone()
}
fn ref_for(r: &ReadFrame, scene: &str, id: &str) -> EntityRef {
    let key = RecordKey {
        owner: owner(),
        local: local("p_", &entity_source(scene, id)),
    };
    match &r.records[&key].fields["entity"] {
        Value::Ref(ObjectRef::Entity(e)) => e.clone(),
        _ => panic!("typed entity reference"),
    }
}
fn pixel_feet(s: &Snapshot, id: &str) -> [f64; 2] {
    let e = s.entities.iter().find(|e| e.id == id).unwrap();
    [
        (e.position.x / SUBPIXELS) as f64 + f64::from(e.size[0]) / 2.0,
        (e.position.y / SUBPIXELS) as f64 + f64::from(e.size[1]),
    ]
}

#[test]
fn fifth_content_is_imported_into_actual_core_allocations_with_state_and_feet() {
    let w = fifth();
    let original = snapshot_hash(&w.snapshot()).unwrap();
    let s = w.snapshot();
    assert_eq!(w.scene, "town");
    assert_eq!(w.project.scenes.len(), 5);
    let (mut c, r, binding) = import(&w);
    assert_eq!(r.scenes.len(), 1, "current legacy scene projection");
    assert_eq!(r.entities.len(), s.entities.len());
    assert!(!r.entities.is_empty());
    assert_eq!(binding.owner, owner());
    assert_eq!(binding.units_per_world, SUBPIXELS as f64);
    let scene = r.scenes[0].reference.clone();
    c.scene_handle(&scene).unwrap();
    let scene_key = RecordKey {
        owner: owner(),
        local: local("s_", &s.scene),
    };
    assert_eq!(
        r.records[&scene_key].fields["id"],
        Value::String(s.scene.clone())
    );
    for e in &s.entities {
        let reference = ref_for(&r, &s.scene, &e.id);
        c.entity_handle(&reference).unwrap();
        assert_eq!(
            r.entities
                .iter()
                .find(|e| e.reference == reference)
                .unwrap()
                .scene,
            scene
        );
        let key = RecordKey {
            owner: owner(),
            local: local("p_", &entity_source(&s.scene, &e.id)),
        };
        let fields = &r.records[&key].fields;
        let feet = pixel_feet(&s, &e.id);
        assert_eq!(fields["x"], Value::I64((feet[0] * SUBPIXELS as f64) as i64));
        assert_eq!(
            fields["y"],
            Value::I64((-feet[1] * SUBPIXELS as f64) as i64)
        );
        assert_eq!(fields["z"], Value::I64(0));
        let metadata_key = RecordKey {
            owner: owner(),
            local: local("e_", &entity_source(&s.scene, &e.id)),
        };
        assert_eq!(
            r.records[&metadata_key].fields["id"],
            Value::String(e.id.clone())
        );
    }
    for (key, value) in &s.state {
        let rk = RecordKey {
            owner: owner(),
            local: local("v_", key),
        };
        let record = &r.records[&rk];
        assert_eq!(record.fields["key"], Value::String(key.clone()));
        let expected = if let Some(v) = value.as_bool() {
            Value::Bool(v)
        } else if let Some(v) = value.as_i64() {
            Value::I64(v)
        } else {
            Value::String(value.as_str().unwrap().into())
        };
        assert_eq!(record.fields["value"], expected);
    }
    let save = c.save().unwrap();
    c.restore(&save).unwrap();
    assert_eq!(select(&c), r);
    assert_eq!(snapshot_hash(&w.snapshot()).unwrap(), original);
}

#[test]
fn actual_native_camera_drives_legacy_pixels_and_upright_feet_without_gameplay_mutation() {
    let w = fifth();
    let s = w.render_snapshot();
    let authoritative = snapshot_hash(&w.snapshot()).unwrap();
    let (c, r, b) = import(&w);
    let core_before = c.save().unwrap();
    let mut h = view(&r, b, &s);
    let baseline = frame(&h, &r);
    let output = render_legacy(&w.project, &s, &baseline, false).unwrap();
    assert_eq!((output.width, output.height), (320, 240));
    assert_eq!(output.rgba.len(), 320 * 240 * 4);
    let old = ge4g_render2d::render(&w.project, &s, false).unwrap();
    assert_eq!(
        hash(&output.rgba),
        hash(&old.rgba),
        "plan preset remains the existing rendering"
    );
    let feet = pixel_feet(&s, "player");
    let native = legacy_foot(&baseline, feet, 0.0).unwrap();
    let reference = ge4g_render2d::actor_screen_foot(&w.project, &s, "player").unwrap();
    assert!((native[0] - reference[0] as f64).abs() <= 1.0);
    assert!((native[1] - reference[1] as f64).abs() <= 1.0);
    let goal = legacy_camera_target(&s, &viewport(), 115.0, 75.0, 12.0).unwrap();
    let f = h.update(&r, change(1, goal.clone(), None)).unwrap();
    let first = &f.cameras[0].target;
    assert!(
        first.pose.zoom > 1.0 && first.pose.zoom < goal.pose.zoom,
        "default smooth, not instant"
    );
    for tick in 2..=6 {
        h.update(&r, input(tick)).unwrap();
    }
    let mid = frame(&h, &r);
    let image = render_legacy(&w.project, &s, &mid, false).unwrap();
    assert_ne!(
        hash(&image.rgba),
        hash(&output.rgba),
        "actual native camera state changes pixels"
    );
    let moved = legacy_foot(&mid, feet, 0.0).unwrap();
    assert_ne!(serialized(&moved), serialized(&native));
    let projected = project(
        WorldPoint([feet[0], -feet[1], 0.0]),
        &mid.target,
        &mid.viewport,
    )
    .unwrap();
    assert!((moved[0] - projected.x).abs() < 1e-7 && (moved[1] - projected.y).abs() < 1e-7);
    let lifted = legacy_foot(&mid, feet, 24.0).unwrap();
    assert!((moved[1] - lifted[1] - 24.0 * mid.target.pose.zoom).abs() < 1e-7);
    assert_eq!(c.save().unwrap(), core_before);
    assert_eq!(select(&c), r);
    assert_eq!(snapshot_hash(&w.snapshot()).unwrap(), authoritative);
    assert_eq!(
        serialized(&w.render_snapshot()),
        serialized(&s),
        "input snapshot also unchanged"
    );
}

#[test]
fn fifth_transition_midpoint_restore_deterministic_continuation_and_reinstall_preserve_core() {
    let w = fifth();
    let s = w.render_snapshot();
    let (c, r, b) = import(&w);
    let frozen = c.save().unwrap();
    let source = snapshot_hash(&w.snapshot()).unwrap();
    let mut a = view(&r, b.clone(), &s);
    let mut restored = view(&r, b.clone(), &s);
    let goal = legacy_camera_target(&s, &viewport(), 125.0, 70.0, 18.0).unwrap();
    a.update(&r, change(1, goal, None)).unwrap();
    for tick in 2..=5 {
        a.update(&r, input(tick)).unwrap();
    }
    let saved = a.save().unwrap();
    restored.restore(&saved, &r).unwrap();
    assert_eq!(restored.save().unwrap(), saved);
    for tick in 6..=12 {
        let fa = a.update(&r, input(tick)).unwrap();
        let fb = restored.update(&r, input(tick)).unwrap();
        assert_eq!(serialized(&fa), serialized(&fb));
        assert_eq!(a.save().unwrap(), restored.save().unwrap());
        let ia = render_legacy(&w.project, &s, &fa.cameras[0], false).unwrap();
        let ib = render_legacy(&w.project, &s, &fb.cameras[0], false).unwrap();
        assert_eq!(hash(&ia.rgba), hash(&ib.rgba));
    }
    a.remove(&vid()).unwrap();
    assert!(a.describe().cameras.is_empty());
    a.install(
        Box::new(TopDownFormat),
        ViewConfig {
            id: vid(),
            scene: r.scenes[0].reference.clone(),
            binding: b,
            policy: ViewPolicy::top_down(),
        },
        &r,
    )
    .unwrap();
    let t = legacy_camera_target(&s, &viewport(), 100.0, 100.0, 0.0).unwrap();
    a.add_camera(CameraConfig::new(cid(), vid(), viewport(), t), &r)
        .unwrap();
    a.activate(&cid()).unwrap();
    assert!(render_legacy(&w.project, &s, &frame(&a, &r), false).is_ok());
    assert_eq!(c.save().unwrap(), frozen);
    assert_eq!(select(&c), r);
    assert_eq!(snapshot_hash(&w.snapshot()).unwrap(), source);
}

#[test]
fn compatibility_rejects_unsupported_perspective_rotation_and_unsafe_source_parameters() {
    let w = fifth();
    let s = w.render_snapshot();
    let (_, r, b) = import(&w);
    let h = view(&r, b, &s);
    let mut camera = frame(&h, &r);
    camera.target.projection = Projection::Perspective {
        half_height: 120.0,
        near: 0.1,
        far: 1000.0,
        focus_distance: 100.0,
    };
    let e = render_legacy(&w.project, &s, &camera, false)
        .expect_err("perspective is outside this compositor subset");
    assert_eq!(e.code(), CompatibilityErrorCode::Unsupported);
    camera = frame(&h, &r);
    camera.target.pose.orientation = Quaternion {
        x: 0.0,
        y: 0.0,
        z: 1.0,
        w: 0.0,
    };
    assert_eq!(
        render_legacy(&w.project, &s, &camera, false)
            .expect_err("rotated camera is outside the legacy compositor subset")
            .code(),
        CompatibilityErrorCode::Unsupported
    );
    camera = frame(&h, &r);
    camera.viewport.width = 160.0;
    assert!(render_legacy(&w.project, &s, &camera, false).is_err());
    for bad in [f64::NAN, f64::INFINITY, 0.0, -1.0, 100_001.0] {
        assert!(legacy_camera_target(&s, &viewport(), bad, 100.0, 0.0).is_err());
    }
    let mut unsafe_snapshot = s.clone();
    unsafe_snapshot.entities[0].position.x = i64::MAX;
    assert!(
        render_legacy(&w.project, &unsafe_snapshot, &frame(&h, &r), false).is_err(),
        "checked before legacy i64 math"
    );
    let mut unsafe_world = w.clone();
    unsafe_world
        .entities
        .values_mut()
        .next()
        .unwrap()
        .position
        .x = i64::MAX;
    assert!(
        presentation_plugin(&unsafe_world, owner()).is_err(),
        "import rejects overflow before position record generation"
    );
    assert!(legacy_foot(&frame(&h, &r), [f64::NAN, 0.0], 0.0).is_err());
}

#[test]
fn new_source_importer_can_coexist_with_original_p2_legacy_adapter() {
    let w = fifth();
    let bridge = ge4g_pentomino_legacy::LegacyBridge::new(w.clone()).unwrap();
    let projection = bridge.projection().unwrap();
    let mut c = CoreHost::new(1, "fixture.coexist").unwrap();
    projection
        .install_into(
            &mut c,
            &ge4g_pentomino::p2::InputFrame {
                target_tick: 1,
                actions: vec![],
            },
        )
        .unwrap();
    let (plugin, _) = presentation_plugin(&w, owner()).unwrap();
    c.install(plugin, BTreeMap::new()).unwrap();
    assert_eq!(c.describe().plugins.len(), 2);
    assert!(!select(&c).entities.is_empty());
}

#[test]
fn compatibility_rejects_camera_from_another_scene_and_accepts_zero_weight_blend() {
    let w = fifth();
    let s = w.render_snapshot();
    let (_, r, b) = import(&w);
    let h = view(&r, b, &s);
    let original = frame(&h, &r);
    let mut blended = original.clone();
    let Projection::Orthographic {
        half_height,
        near,
        far,
        focus_distance,
    } = blended.target.projection
    else {
        panic!("initial ortho")
    };
    blended.target.projection = Projection::Blended {
        half_height,
        near,
        far,
        focus_distance,
        perspective_weight: 0.0,
    };
    let baseline = render_legacy(&w.project, &s, &original, false).unwrap();
    let compatible = render_legacy(&w.project, &s, &blended, false).unwrap();
    assert_eq!(hash(&baseline.rgba), hash(&compatible.rgba));
    let mut wrong = original.clone();
    wrong.scene.local = "s_wrongscene".into();
    assert!(
        render_legacy(&w.project, &s, &wrong, false).is_err(),
        "CameraFrame must be for current source scene"
    );
    let mut wrong = original;
    wrong.scene.incarnation = 0;
    assert!(
        render_legacy(&w.project, &s, &wrong, false).is_err(),
        "invalid stable reference is not a valid source scene"
    );
}

#[test]
fn fifth_importer_bounded_state_and_long_owner_fail_without_source_mutation() {
    let mut w = fifth();
    let original = snapshot_hash(&w.snapshot()).unwrap();
    let long = PluginId::new(&format!("a.{}", "b".repeat(126))).unwrap();
    assert!(
        presentation_plugin(&w, long).is_err(),
        "owner-prefixed schema exceedsP2identifier bound"
    );
    assert_eq!(snapshot_hash(&w.snapshot()).unwrap(), original);
    w.state.values.insert(
        "bad.object".into(),
        serde_json::json!({"arbitrary":"object"}),
    );
    let before = snapshot_hash(&w.snapshot()).unwrap();
    assert!(presentation_plugin(&w, owner()).is_err());
    assert_eq!(snapshot_hash(&w.snapshot()).unwrap(), before);
    w.state.values.remove("bad.object");
    for i in 0..257 {
        w.state
            .values
            .insert(format!("oversized.state{i}"), true.into());
    }
    let before = snapshot_hash(&w.snapshot()).unwrap();
    assert!(presentation_plugin(&w, owner()).is_err());
    assert_eq!(snapshot_hash(&w.snapshot()).unwrap(), before);
}

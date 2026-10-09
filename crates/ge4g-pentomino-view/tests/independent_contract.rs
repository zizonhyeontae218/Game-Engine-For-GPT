//! Independent consumer tests derived from the frozen P3 contract, not implementation.
use ge4g_pentomino::p2::{
    self, CoreContext, CoreDescriptor, CoreHost, CorePlugin, CoreTransaction, EntityRef, FieldType,
    InputFrame, ObjectRef, ReadFrame, Record, RecordSchema, RefKind, SceneRef, SchemaId, Selection,
    Value,
};
use ge4g_pentomino::{PluginId, Version};
use ge4g_pentomino_view::*;
use std::collections::{BTreeMap, BTreeSet};

fn owner(name: &str) -> PluginId {
    PluginId::new(name).unwrap()
}
fn schema(name: &str) -> SchemaId {
    SchemaId::new(name).unwrap()
}
fn vid(name: &str) -> ViewId {
    ViewId::new(name).unwrap()
}
fn cid(name: &str) -> CameraId {
    CameraId::new(name).unwrap()
}
fn bytes<T: serde::Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).unwrap()
}
fn close(a: f64, b: f64) {
    assert!(
        (a - b).abs() <= 1e-7 * a.abs().max(b.abs()).max(1.0),
        "{a} != {b}"
    );
}
fn xyz(a: [f64; 3], b: [f64; 3]) {
    for i in 0..3 {
        close(a[i], b[i]);
    }
}
fn identity() -> Quaternion {
    Quaternion {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    }
}
fn unit(q: Quaternion) -> Quaternion {
    let n = (q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w).sqrt();
    Quaternion {
        x: q.x / n,
        y: q.y / n,
        z: q.z / n,
        w: q.w / n,
    }
}
fn target() -> CameraTarget {
    CameraTarget {
        pose: CameraPose {
            position: [0.0, 0.0, 100.0],
            orientation: identity(),
            zoom: 1.0,
        },
        view_transform: ViewTransform {
            origin: [0.0; 3],
            rotation: identity(),
            scale: [1.0; 3],
            shear_xy: 0.0,
        },
        projection: Projection::Orthographic {
            half_height: 100.0,
            near: 0.1,
            far: 1000.0,
            focus_distance: 100.0,
        },
    }
}
fn viewport() -> Viewport {
    Viewport {
        x: 0.0,
        y: 0.0,
        width: 320.0,
        height: 240.0,
    }
}
fn input(tick: u64) -> ViewInput {
    ViewInput {
        target_tick: tick,
        changes: vec![],
    }
}
fn change(tick: u64, camera: &str, goal: CameraTarget, mode: Option<TransitionMode>) -> ViewInput {
    ViewInput {
        target_tick: tick,
        changes: vec![CameraChange {
            camera: cid(camera),
            target: goal,
            transition: mode,
        }],
    }
}
fn expect<T>(result: Result<T, Error>, code: ErrorCode) {
    match result {
        Err(e) => assert_eq!(e.code(), code, "{}", e.detail()),
        Ok(_) => panic!("expected {code:?}"),
    }
}

struct MovingSource {
    name: &'static str,
}
impl MovingSource {
    fn position(&self, entity: EntityRef, x: i64, y: i64) -> Record {
        Record {
            schema: schema(&format!("{}.position", self.name)),
            fields: BTreeMap::from([
                ("entity".into(), Value::Ref(ObjectRef::Entity(entity))),
                ("x".into(), Value::I64(x)),
                ("y".into(), Value::I64(y)),
                ("z".into(), Value::I64(0)),
            ]),
        }
    }
    fn stat(&self, n: i64, rng: i64) -> Record {
        Record {
            schema: schema(&format!("{}.stats", self.name)),
            fields: BTreeMap::from([("n".into(), Value::I64(n)), ("rng".into(), Value::I64(rng))]),
        }
    }
}
impl CorePlugin for MovingSource {
    fn descriptor(&self) -> CoreDescriptor {
        let int = || FieldType::I64 {
            min: i64::MIN,
            max: i64::MAX,
        };
        CoreDescriptor {
            id: owner(self.name),
            release: Version {
                major: 0,
                minor: 3,
                patch: 0,
            },
            contract: Version {
                major: 2,
                minor: 0,
                patch: 0,
            },
            provides: BTreeMap::new(),
            requires: BTreeMap::new(),
            schemas: BTreeMap::from([
                (
                    schema(&format!("{}.position", self.name)),
                    RecordSchema {
                        fields: BTreeMap::from([
                            (
                                "entity".into(),
                                FieldType::Ref {
                                    kind: RefKind::Entity,
                                },
                            ),
                            ("x".into(), int()),
                            ("y".into(), int()),
                            ("z".into(), int()),
                        ]),
                    },
                ),
                (
                    schema(&format!("{}.stats", self.name)),
                    RecordSchema {
                        fields: BTreeMap::from([("n".into(), int()), ("rng".into(), int())]),
                    },
                ),
            ]),
            event_kinds: BTreeMap::from([(
                "pulse".into(),
                schema(&format!("{}.stats", self.name)),
            )]),
            actions: BTreeMap::new(),
        }
    }
    fn initialize(&self, _: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), p2::Error> {
        let s = tx.create_scene("main")?;
        let e = tx.create_entity("actor", &s)?;
        tx.set("position", self.position(e, 10, 20))?;
        let s = tx.create_scene("other")?;
        let e = tx.create_entity("offscene", &s)?;
        tx.set("offscene", self.position(e, 900, 900))?;
        let rng = tx.draw()? as i64;
        let r = self.stat(0, rng);
        tx.set("stats", r.clone())?;
        tx.emit("pulse", r)
    }
    fn tick(&self, ctx: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), p2::Error> {
        let mut r = ctx.own("position")?.unwrap().clone();
        r.fields
            .insert("x".into(), Value::I64(10 * (ctx.tick() as i64 + 1)));
        r.fields
            .insert("y".into(), Value::I64(20 * (ctx.tick() as i64 + 1)));
        tx.set("position", r)?;
        let rng = tx.draw()? as i64;
        let r = self.stat(ctx.tick() as i64, rng);
        tx.set("stats", r.clone())?;
        tx.emit("pulse", r)
    }
}
fn core() -> CoreHost {
    let mut h = CoreHost::new(2026, "p3.consumer").unwrap();
    for name in ["game.source", "independent.plugin"] {
        h.install(Box::new(MovingSource { name }), BTreeMap::new())
            .unwrap();
    }
    h
}
fn read(h: &CoreHost) -> ReadFrame {
    h.select(&Selection {
        owners: BTreeSet::from([owner("game.source"), owner("independent.plugin")]),
        schemas: BTreeSet::new(),
        include_objects: true,
        include_history: true,
    })
    .unwrap()
}
fn scene(r: &ReadFrame) -> SceneRef {
    r.scenes
        .iter()
        .find(|s| s.reference.owner == owner("game.source") && s.reference.local == "main")
        .unwrap()
        .reference
        .clone()
}
fn actor(r: &ReadFrame) -> EntityRef {
    r.entities
        .iter()
        .find(|e| e.reference.owner == owner("game.source") && e.reference.local == "actor")
        .unwrap()
        .reference
        .clone()
}
fn binding() -> RecordBinding {
    RecordBinding {
        owner: owner("game.source"),
        schema: schema("game.source.position"),
        entity_field: "entity".into(),
        x_field: "x".into(),
        y_field: "y".into(),
        z_field: Some("z".into()),
        units_per_world: 1.0,
        representation: Representation::Sprite {
            asset: "consumer.sprite".into(),
            size: [16.0, 24.0],
        },
    }
}
fn config(r: &ReadFrame, name: &str, policy: ViewPolicy) -> ViewConfig {
    ViewConfig {
        id: vid(name),
        scene: scene(r),
        binding: binding(),
        policy,
    }
}
fn follow(r: &ReadFrame) -> FollowBehavior {
    FollowBehavior {
        target: actor(r),
        offset: [0.0; 3],
        dead_zone: [0.0; 3],
        look_ahead_ticks: 0,
        smoothing_ticks: 0,
    }
}
fn host(r: &ReadFrame) -> ViewHost {
    let mut h = ViewHost::new("p3.consumer").unwrap();
    h.install(
        Box::new(TopDownFormat),
        config(r, "view.main", ViewPolicy::top_down()),
        r,
    )
    .unwrap();
    h
}
fn camera(h: &mut ViewHost, r: &ReadFrame, name: &str, view: &str, goal: CameraTarget) {
    h.add_camera(CameraConfig::new(cid(name), vid(view), viewport(), goal), r)
        .unwrap();
    h.activate(&cid(name)).unwrap();
}
fn frame_camera(f: &RenderFrame, name: &str) -> CameraFrame {
    f.cameras
        .iter()
        .find(|c| c.camera == cid(name))
        .unwrap()
        .clone()
}

#[test]
fn four_formats_extract_real_core_records_and_distinct_follow_policies() {
    let c = core();
    let r = read(&c);
    let frozen = c.save().unwrap();
    let cases: Vec<(Box<dyn FormatPlugin>, ViewPolicy, [f64; 3])> = vec![
        (
            Box::new(Classic2DFormat),
            ViewPolicy::classic2d(),
            [10.0, 20.0, 100.0],
        ),
        (
            Box::new(TopDownFormat),
            ViewPolicy::top_down(),
            [10.0, 20.0, 100.0],
        ),
        (Box::new(SideFormat), ViewPolicy::side(), [10.0, 0.0, 100.0]),
        (
            Box::new(VerticalFormat),
            ViewPolicy::vertical(),
            [0.0, 20.0, 100.0],
        ),
    ];
    for (format, policy, expected) in cases {
        let selection = format.extract(&r, &binding(), &scene(&r)).unwrap();
        assert_eq!(
            selection.items.len(),
            1,
            "valid other-scene entities must be filtered"
        );
        assert_eq!(selection.items[0].entity, actor(&r));
        xyz(selection.items[0].world.0, [10.0, 20.0, 0.0]);
        let mut h = ViewHost::new("p3.consumer").unwrap();
        h.install(format, config(&r, "view.main", policy), &r)
            .unwrap();
        let mut cc = CameraConfig::new(cid("cam.main"), vid("view.main"), viewport(), target());
        cc.behaviors.follow = Some(follow(&r));
        h.add_camera(cc, &r).unwrap();
        h.activate(&cid("cam.main")).unwrap();
        let f = h.update(&r, input(1)).unwrap();
        xyz(f.cameras[0].target.pose.position, expected);
        assert_eq!(f.source_tick, r.tick);
        assert_eq!(c.save().unwrap(), frozen);
    }
}

#[test]
fn removal_replacement_and_reinstall_preserve_whole_core_and_independent_camera() {
    let c = core();
    let r = read(&c);
    let frozen = c.save().unwrap();
    let ch = c.hash().unwrap();
    let mut h = host(&r);
    h.install(
        Box::new(SideFormat),
        config(&r, "view.other", ViewPolicy::side()),
        &r,
    )
    .unwrap();
    camera(&mut h, &r, "cam.a", "view.main", target());
    let mut t = target();
    t.pose.position[0] = 99.0;
    camera(&mut h, &r, "cam.b", "view.other", t);
    let f = h.update(&r, input(1)).unwrap();
    let other = bytes(&frame_camera(&f, "cam.b"));
    h.remove_camera(&cid("cam.a")).unwrap();
    assert_eq!(bytes(&frame_camera(&h.frame(&r).unwrap(), "cam.b")), other);
    h.replace(
        &vid("view.main"),
        Box::new(Classic2DFormat),
        config(&r, "view.main", ViewPolicy::classic2d()),
        &r,
    )
    .unwrap();
    camera(&mut h, &r, "cam.a", "view.main", target());
    h.remove(&vid("view.main")).unwrap();
    assert_eq!(h.describe().cameras.len(), 1);
    h.install(
        Box::new(VerticalFormat),
        config(&r, "view.main", ViewPolicy::vertical()),
        &r,
    )
    .unwrap();
    camera(&mut h, &r, "cam.a", "view.main", target());
    assert_eq!(bytes(&frame_camera(&h.frame(&r).unwrap(), "cam.b")), other);
    assert_eq!(c.save().unwrap(), frozen);
    assert_eq!(c.hash().unwrap(), ch);
    assert_eq!(
        read(&c),
        r,
        "identities, records and histories all preserved"
    );
}

#[test]
fn hot_replace_preserves_active_camera_and_transition_while_changing_policy() {
    let c = core();
    let r = read(&c);
    let mut h = host(&r);
    camera(&mut h, &r, "cam.main", "view.main", target());
    let mut goal = target();
    goal.pose.position[0] = 120.0;
    h.update(&r, change(1, "cam.main", goal, None)).unwrap();
    let before = bytes(&h.describe().cameras);
    h.replace(
        &vid("view.main"),
        Box::new(Classic2DFormat),
        config(&r, "view.main", ViewPolicy::classic2d()),
        &r,
    )
    .unwrap();
    assert_eq!(bytes(&h.describe().cameras), before);
    assert!(h.describe().cameras[0].transitioning);
    assert!(h.describe().cameras[0].active);
}

#[test]
fn two_cameras_have_independent_transforms_viewports_and_active_sets() {
    let c = core();
    let r = read(&c);
    let mut h = host(&r);
    camera(&mut h, &r, "cam.a", "view.main", target());
    let mut goal = target();
    goal.view_transform.shear_xy = 0.5;
    goal.pose.zoom = 2.0;
    let mut cc = CameraConfig::new(
        cid("cam.b"),
        vid("view.main"),
        Viewport {
            x: 320.0,
            y: 12.0,
            width: 160.0,
            height: 120.0,
        },
        goal,
    );
    cc.default_transition = TransitionMode::Instant;
    h.add_camera(cc, &r).unwrap();
    h.activate(&cid("cam.b")).unwrap();
    let f = h.update(&r, input(1)).unwrap();
    assert_eq!(f.cameras.len(), 2);
    let a = frame_camera(&f, "cam.a");
    let b = frame_camera(&f, "cam.b");
    assert_ne!(bytes(&a.items[0].screen), bytes(&b.items[0].screen));
    let old_b = bytes(&b);
    let mut goal = target();
    goal.pose.zoom = 3.0;
    h.update(&r, change(2, "cam.a", goal, Some(TransitionMode::Instant)))
        .unwrap();
    assert_eq!(bytes(&frame_camera(&h.frame(&r).unwrap(), "cam.b")), old_b);
    h.set_active(&vid("view.main"), BTreeSet::from([cid("cam.b")]))
        .unwrap();
    assert_eq!(h.frame(&r).unwrap().cameras.len(), 1);
    h.set_active(&vid("view.main"), BTreeSet::new()).unwrap();
    assert!(h.frame(&r).unwrap().cameras.is_empty());
}

#[test]
fn follow_tracks_all_world_axes_before_rotated_sheared_side_policy() {
    let mut c = core();
    let r = read(&c);
    let mut h = ViewHost::new("p3.consumer").unwrap();
    h.install(
        Box::new(SideFormat),
        config(&r, "view.main", ViewPolicy::side()),
        &r,
    )
    .unwrap();
    let mut t = target();
    t.view_transform.shear_xy = 1.7;
    let q = std::f64::consts::FRAC_1_SQRT_2;
    t.view_transform.rotation = Quaternion {
        x: 0.0,
        y: 0.0,
        z: q,
        w: q,
    };
    let mut cc = CameraConfig::new(cid("cam.main"), vid("view.main"), viewport(), t);
    let mut f = follow(&r);
    f.look_ahead_ticks = 2;
    cc.behaviors.follow = Some(f);
    h.add_camera(cc, &r).unwrap();
    h.activate(&cid("cam.main")).unwrap();
    xyz(
        h.update(&r, input(1)).unwrap().cameras[0]
            .target
            .pose
            .position,
        [-20.0, 0.0, 100.0],
    );
    c.step(InputFrame {
        target_tick: 1,
        actions: vec![],
    })
    .unwrap();
    // World point [20,40], velocity [10,20], prediction [40,80], R90*H => [-80,176].
    let r = read(&c);
    let frozen = c.save().unwrap();
    xyz(
        h.update(&r, input(2)).unwrap().cameras[0]
            .target
            .pose
            .position,
        [-80.0, 0.0, 100.0],
    );
    assert_eq!(c.save().unwrap(), frozen);
}

#[test]
fn follow_dead_zone_tracking_filter_and_bounds_are_composed() {
    let mut c = core();
    let r = read(&c);
    let mut h = host(&r);
    let mut cc = CameraConfig::new(cid("cam.main"), vid("view.main"), viewport(), target());
    let mut f = follow(&r);
    f.dead_zone = [5.0; 3];
    f.smoothing_ticks = 2;
    cc.behaviors.follow = Some(f);
    cc.behaviors.bounds = Some(Bounds {
        min: [0.0, 0.0, 1.0],
        max: [13.0, 28.0, 200.0],
    });
    h.add_camera(cc, &r).unwrap();
    h.activate(&cid("cam.main")).unwrap();
    xyz(
        h.update(&r, input(1)).unwrap().cameras[0]
            .target
            .pose
            .position,
        [10.0, 20.0, 100.0],
    );
    c.step(InputFrame {
        target_tick: 1,
        actions: vec![],
    })
    .unwrap();
    xyz(
        h.update(&read(&c), input(2)).unwrap().cameras[0]
            .target
            .pose
            .position,
        [12.5, 27.5, 100.0],
    );
    c.step(InputFrame {
        target_tick: 2,
        actions: vec![],
    })
    .unwrap();
    xyz(
        h.update(&read(&c), input(3)).unwrap().cameras[0]
            .target
            .pose
            .position,
        [13.0, 28.0, 100.0],
    );
}

#[test]
fn world_view_camera_screen_roundtrip_and_real_perspective_depth_scaling() {
    let mut t = target();
    let q = std::f64::consts::FRAC_1_SQRT_2;
    t.view_transform = ViewTransform {
        origin: [1.0, 2.0, 3.0],
        rotation: Quaternion {
            x: 0.0,
            y: 0.0,
            z: q,
            w: q,
        },
        scale: [2.0, 0.5, 3.0],
        shear_xy: 0.37,
    };
    t.pose.orientation = Quaternion {
        x: 0.0,
        y: 0.0,
        z: -q,
        w: q,
    };
    let p = WorldPoint([3.25, -1.5, 2.75]);
    let v = world_to_view(p, &t.view_transform).unwrap();
    xyz(view_to_world(v, &t.view_transform).unwrap().0, p.0);
    let c = view_to_camera(v, &t.pose).unwrap();
    xyz(camera_to_view(c, &t.pose).unwrap().0, v.0);
    for lens in [
        Projection::Orthographic {
            half_height: 100.0,
            near: 0.1,
            far: 1000.0,
            focus_distance: 100.0,
        },
        Projection::Perspective {
            half_height: 100.0,
            near: 0.1,
            far: 1000.0,
            focus_distance: 100.0,
        },
        Projection::Blended {
            half_height: 100.0,
            near: 0.1,
            far: 1000.0,
            focus_distance: 100.0,
            perspective_weight: 0.42,
        },
    ] {
        t.projection = lens;
        let s = project(p, &t, &viewport()).unwrap();
        xyz(unproject(s, &t, &viewport()).unwrap().0, p.0);
    }
    t = target();
    t.projection = Projection::Perspective {
        half_height: 100.0,
        near: 0.1,
        far: 1000.0,
        focus_distance: 100.0,
    };
    let a = project(WorldPoint([10.0, 0.0, 0.0]), &t, &viewport()).unwrap();
    let b = project(WorldPoint([10.0, 0.0, -100.0]), &t, &viewport()).unwrap();
    close(a.x - 160.0, 2.0 * (b.x - 160.0));
    // Offscreen X/Y are algebraically valid; behind-camera points are not.
    assert!(
        project(WorldPoint([10000.0, 0.0, 0.0]), &t, &viewport())
            .unwrap()
            .x
            > 320.0
    );
    expect(
        project(WorldPoint([0.0, 0.0, 101.0]), &t, &viewport()),
        ErrorCode::OutsideDepth,
    );
}

#[test]
fn rejects_nan_infinite_singular_and_extreme_bounds_without_panics() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1e10] {
        assert!(project(WorldPoint([bad, 0.0, 0.0]), &target(), &viewport()).is_err());
        let mut t = target();
        t.pose.position[0] = bad;
        assert!(project(WorldPoint([0.0; 3]), &t, &viewport()).is_err());
    }
    let mut t = target();
    t.view_transform.scale[1] = 0.0;
    expect(
        world_to_view(WorldPoint([1.0; 3]), &t.view_transform),
        ErrorCode::InvalidTransform,
    );
    t = target();
    t.pose.orientation = Quaternion {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    assert!(view_to_camera(ViewPoint([1.0; 3]), &t.pose).is_err());
    t = target();
    t.pose.zoom = 0.0;
    assert!(project(WorldPoint([0.0; 3]), &t, &viewport()).is_err());
    let v = Viewport {
        width: 0.0,
        ..viewport()
    };
    expect(
        project(WorldPoint([0.0; 3]), &target(), &v),
        ErrorCode::InvalidViewport,
    );
    let t = target();
    assert!(
        unproject(
            ScreenPoint {
                x: 0.0,
                y: 0.0,
                depth: 1.1
            },
            &t,
            &viewport()
        )
        .is_err()
    );
    let mut t = target();
    t.pose.zoom = 1000.0;
    t.view_transform.scale = [1000.0; 3];
    t.view_transform.shear_xy = 8.0;
    // A documented extreme need only be finite or explicitly rejected, never NaN/panic.
    if let Ok(p) = project(WorldPoint([1e9, 1e9, 0.0]), &t, &viewport()) {
        assert!(p.x.is_finite() && p.y.is_finite() && p.depth.is_finite());
    }
}

#[test]
fn smooth_is_default_for_pose_viewing_mode_and_projection_and_is_tick_based() {
    let c = core();
    let r = read(&c);
    let mut h = host(&r);
    camera(&mut h, &r, "cam.main", "view.main", target());
    let frozen = c.save().unwrap();
    let mut end = target();
    end.pose.position[0] = 120.0;
    end.pose.zoom = 2.0;
    end.view_transform.origin[0] = 60.0;
    end.view_transform.scale[1] = 0.5;
    end.view_transform.shear_xy = 1.0;
    end.pose.orientation = Quaternion {
        x: 0.0,
        y: 0.0,
        z: 1.0,
        w: 0.0,
    };
    end.view_transform.rotation = Quaternion {
        x: 0.0,
        y: 0.0,
        z: 1.0,
        w: 0.0,
    };
    end.projection = Projection::Perspective {
        half_height: 200.0,
        near: 1.0,
        far: 2000.0,
        focus_distance: 200.0,
    };
    let first = h
        .update(&r, change(1, "cam.main", end.clone(), None))
        .unwrap();
    let u = 1.0_f64 / 12.0;
    let s = u * u * (3.0 - 2.0 * u);
    close(first.cameras[0].target.pose.position[0], 120.0 * s);
    close(first.cameras[0].target.view_transform.origin[0], 60.0 * s);
    close(first.cameras[0].target.pose.zoom, 1.0 + s);
    assert!(matches!(
        first.cameras[0].target.projection,
        Projection::Blended { .. }
    ));
    for tick in 2..=6 {
        h.update(&r, input(tick)).unwrap();
    }
    let mid = h.frame(&r).unwrap();
    let m = &mid.cameras[0].target;
    close(m.pose.position[0], 60.0);
    close(m.view_transform.scale[1], 0.75);
    close(m.pose.orientation.z.abs(), std::f64::consts::FRAC_1_SQRT_2);
    close(
        m.view_transform.rotation.z.abs(),
        std::f64::consts::FRAC_1_SQRT_2,
    );
    let saved = h.save().unwrap();
    assert_eq!(
        h.save().unwrap(),
        saved,
        "frame must not advance transition"
    );
    for tick in 7..=12 {
        h.update(&r, input(tick)).unwrap();
    }
    assert_eq!(bytes(&h.frame(&r).unwrap().cameras[0].target), bytes(&end));
    assert!(!h.describe().cameras[0].transitioning);
    assert_eq!(c.save().unwrap(), frozen);
}

#[test]
fn instant_retarget_and_inactive_freeze_are_explicit() {
    let c = core();
    let r = read(&c);
    let mut h = host(&r);
    camera(&mut h, &r, "cam.main", "view.main", target());
    let mut end = target();
    end.pose.position[0] = 120.0;
    h.update(&r, change(1, "cam.main", end, None)).unwrap();
    for tick in 2..=6 {
        h.update(&r, input(tick)).unwrap();
    }
    close(h.describe().cameras[0].current.pose.position[0], 60.0);
    let mut new = target();
    new.pose.position[0] = -60.0;
    let f = h
        .update(
            &r,
            change(
                7,
                "cam.main",
                new.clone(),
                Some(TransitionMode::Smooth { duration_ticks: 4 }),
            ),
        )
        .unwrap();
    close(f.cameras[0].target.pose.position[0], 41.25); // Retarget from sampled60; smoothstep(1/4)=.15625.
    h.deactivate(&cid("cam.main")).unwrap();
    for tick in 8..=12 {
        h.update(&r, input(tick)).unwrap();
    }
    close(h.describe().cameras[0].current.pose.position[0], 41.25);
    h.activate(&cid("cam.main")).unwrap();
    h.update(&r, input(13)).unwrap();
    close(h.describe().cameras[0].current.pose.position[0], 0.0);
    h.deactivate(&cid("cam.main")).unwrap();
    h.update(
        &r,
        change(14, "cam.main", new.clone(), Some(TransitionMode::Instant)),
    )
    .unwrap();
    assert_eq!(bytes(&h.describe().cameras[0].current), bytes(&new));
}

#[test]
fn midpoint_save_restore_replays_nondecimal_rotation_follow_and_shake_exactly() {
    let mut c = core();
    let r = read(&c);
    let setup = |r: &ReadFrame| {
        let mut h = host(r);
        let mut cc = CameraConfig::new(cid("cam.main"), vid("view.main"), viewport(), target());
        cc.behaviors.follow = Some(follow(r));
        cc.behaviors.shake = Some(ShakeBehavior {
            seed: 891,
            amplitude: [0.25, 0.5, 0.0],
            duration_ticks: 20,
        });
        h.add_camera(cc, r).unwrap();
        h.activate(&cid("cam.main")).unwrap();
        h
    };
    let mut a = setup(&r);
    let mut b = setup(&r);
    let mut t = target();
    t.pose.orientation = Quaternion {
        x: 0.11,
        y: 0.22,
        z: 0.33,
        w: 0.77,
    };
    t.view_transform.rotation = Quaternion {
        x: 0.21,
        y: 0.13,
        z: 0.37,
        w: 0.82,
    };
    t.pose.zoom = 1.23456789012345;
    a.update(&r, change(1, "cam.main", t, None)).unwrap();
    for tick in 2..=5 {
        a.update(&r, input(tick)).unwrap();
    }
    let save = a.save().unwrap();
    b.restore(&save, &r).unwrap();
    assert_eq!(b.save().unwrap(), save);
    for tick in 6..=15 {
        c.step(InputFrame {
            target_tick: tick - 5,
            actions: vec![],
        })
        .unwrap();
        let r = read(&c);
        let fa = a.update(&r, input(tick)).unwrap();
        let fb = b.update(&r, input(tick)).unwrap();
        assert_eq!(bytes(&fa), bytes(&fb), "camera output at tick{tick}");
        assert_eq!(
            a.save().unwrap(),
            b.save().unwrap(),
            "local state at tick{tick}"
        );
    }
}

#[test]
fn invalid_tick_change_follow_reference_and_plugin_failure_roll_back_whole_host() {
    let c = core();
    let r = read(&c);
    let mut h = host(&r);
    camera(&mut h, &r, "cam.a", "view.main", target());
    camera(&mut h, &r, "cam.b", "view.main", target());
    let mut goal = target();
    goal.pose.position[0] = 24.0;
    h.update(&r, change(1, "cam.a", goal, None)).unwrap();
    let old = h.save().unwrap();
    expect(h.update(&r, input(3)), ErrorCode::InputOutOfSequence);
    assert_eq!(h.save().unwrap(), old);
    let mut valid = target();
    valid.pose.zoom = 2.0;
    let mut bad = target();
    bad.pose.zoom = f64::NAN;
    let request = ViewInput {
        target_tick: 2,
        changes: vec![
            CameraChange {
                camera: cid("cam.a"),
                target: valid,
                transition: None,
            },
            CameraChange {
                camera: cid("cam.b"),
                target: bad,
                transition: None,
            },
        ],
    };
    assert!(h.update(&r, request).is_err());
    assert_eq!(h.save().unwrap(), old);
    let missing = ViewInput {
        target_tick: 2,
        changes: vec![CameraChange {
            camera: cid("cam.missing"),
            target: target(),
            transition: None,
        }],
    };
    expect(h.update(&r, missing), ErrorCode::MissingCamera);
    assert_eq!(h.save().unwrap(), old);
    let mut cc = CameraConfig::new(cid("cam.follow"), vid("view.main"), viewport(), target());
    cc.behaviors.follow = Some(follow(&r));
    h.add_camera(cc, &r).unwrap();
    h.activate(&cid("cam.follow")).unwrap();
    let old = h.save().unwrap();
    let mut invalid = r.clone();
    invalid.records.retain(|k, _| k.local != "position");
    expect(h.update(&invalid, input(2)), ErrorCode::InvalidSelection);
    assert_eq!(h.save().unwrap(), old);
}

#[test]
fn malformed_missing_wrong_typed_duplicate_and_stale_selection_are_rejected() {
    let c = core();
    let r = read(&c);
    let f = TopDownFormat;
    let key = r
        .records
        .keys()
        .find(|k| k.owner == owner("game.source") && k.local == "position")
        .unwrap()
        .clone();
    for field in ["entity", "x", "y", "z"] {
        let mut bad = r.clone();
        bad.records.get_mut(&key).unwrap().fields.remove(field);
        expect(
            f.extract(&bad, &binding(), &scene(&r)),
            ErrorCode::InvalidSelection,
        );
    }
    let mut bad = r.clone();
    bad.records
        .get_mut(&key)
        .unwrap()
        .fields
        .insert("x".into(), Value::Bool(true));
    expect(
        f.extract(&bad, &binding(), &scene(&r)),
        ErrorCode::InvalidSelection,
    );
    let mut bad = r.clone();
    let duplicate = bad.records[&key].clone();
    let mut dupkey = key.clone();
    dupkey.local = "duplicate".into();
    bad.records.insert(dupkey, duplicate);
    expect(
        f.extract(&bad, &binding(), &scene(&r)),
        ErrorCode::InvalidSelection,
    );
    let mut bad = r.clone();
    bad.entities.retain(|e| e.reference != actor(&r));
    expect(
        f.extract(&bad, &binding(), &scene(&r)),
        ErrorCode::InvalidSelection,
    );
    let mut b = binding();
    b.units_per_world = 0.0;
    expect(f.extract(&r, &b, &scene(&r)), ErrorCode::InvalidSelection);
}

#[test]
fn canonical_save_rejects_unknown_fields_noncanonical_bytes_and_mismatched_install_atomically() {
    let c = core();
    let r = read(&c);
    let mut h = host(&r);
    camera(&mut h, &r, "cam.main", "view.main", target());
    h.update(&r, input(1)).unwrap();
    let saved = h.save().unwrap();
    let mut json: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    json.as_object_mut()
        .unwrap()
        .insert("unknown".into(), true.into());
    expect(
        h.restore(&serde_json::to_vec(&json).unwrap(), &r),
        ErrorCode::InvalidSave,
    );
    assert_eq!(h.save().unwrap(), saved);
    expect(
        h.restore(
            &serde_json::to_vec_pretty(
                &serde_json::from_slice::<serde_json::Value>(&saved).unwrap(),
            )
            .unwrap(),
            &r,
        ),
        ErrorCode::InvalidSave,
    );
    assert_eq!(h.save().unwrap(), saved);
    let mut bad = saved.clone();
    bad.push(b' ');
    expect(h.restore(&bad, &r), ErrorCode::InvalidSave);
    assert_eq!(h.save().unwrap(), saved);
    let mut other = ViewHost::new("other.content").unwrap();
    other
        .install(
            Box::new(TopDownFormat),
            config(&r, "view.main", ViewPolicy::top_down()),
            &r,
        )
        .unwrap();
    camera(&mut other, &r, "cam.main", "view.main", target());
    expect(other.restore(&saved, &r), ErrorCode::SaveMismatch);
    let mut bad: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    bad["format_version"] = 99.into();
    expect(
        h.restore(&serde_json::to_vec(&bad).unwrap(), &r),
        ErrorCode::VersionMismatch,
    );
    assert_eq!(h.save().unwrap(), saved);
    let too_large = vec![b'x'; h.describe().limits.max_save_bytes + 1];
    assert!(h.restore(&too_large, &r).is_err());
    assert_eq!(h.save().unwrap(), saved);
}

#[test]
fn structured_discovery_is_enough_to_choose_view_projection_and_behaviors() {
    let c = core();
    let r = read(&c);
    let mut h = host(&r);
    camera(&mut h, &r, "cam.main", "view.main", target());
    let d = h.describe();
    assert_eq!(d.contract_version, 1);
    assert_eq!(d.available_formats.len(), 4);
    let families: BTreeSet<_> = d
        .available_formats
        .iter()
        .map(|f| bytes(&f.family))
        .collect();
    assert_eq!(families.len(), 4);
    for word in ["orthographic", "perspective"] {
        assert!(
            d.projections
                .iter()
                .any(|p| p.to_lowercase().contains(word))
        );
    }
    for word in ["follow", "shake", "smooth"] {
        assert!(d.behaviors.iter().any(|b| b.to_lowercase().contains(word)));
    }
    assert_eq!(
        d.installed_views[0].config.binding.owner,
        owner("game.source")
    );
    assert_eq!(
        d.installed_views[0].config.binding.schema,
        schema("game.source.position")
    );
    assert_eq!(d.installed_views[0].cameras, vec![cid("cam.main")]);
    assert!(d.cameras[0].active);
    assert_eq!(d.limits.max_cameras, 32);
    assert_eq!(d.limits.max_views, 8);
    assert!(
        serde_json::from_slice::<serde_json::Value>(&bytes(&d))
            .unwrap()
            .is_object()
    );
}

#[test]
fn budgets_identifiers_and_invalid_configuration_are_atomic() {
    for bad in ["", "has space", "bad/identifier", "한글"] {
        assert!(CameraId::new(bad).is_err());
        assert!(ViewId::new(bad).is_err());
    }
    assert!(CameraId::new(&"x".repeat(129)).is_err());
    let c = core();
    let r = read(&c);
    let mut h = host(&r);
    for i in 0..32 {
        h.add_camera(
            CameraConfig::new(
                cid(&format!("cam.c{i}")),
                vid("view.main"),
                viewport(),
                target(),
            ),
            &r,
        )
        .unwrap();
    }
    let old = h.save().unwrap();
    expect(
        h.add_camera(
            CameraConfig::new(cid("cam.over"), vid("view.main"), viewport(), target()),
            &r,
        ),
        ErrorCode::BudgetExceeded,
    );
    assert_eq!(h.save().unwrap(), old);
    let bad = ViewInput {
        target_tick: 1,
        changes: (0..33)
            .map(|i| CameraChange {
                camera: cid(&format!("cam.c{i}")),
                target: target(),
                transition: None,
            })
            .collect(),
    };
    expect(h.update(&r, bad), ErrorCode::BudgetExceeded);
    assert_eq!(h.save().unwrap(), old);
    let mut h = host(&r);
    let mut cc = CameraConfig::new(cid("cam.bad"), vid("view.main"), viewport(), target());
    cc.behaviors.bounds = Some(Bounds {
        min: [1.0; 3],
        max: [0.0; 3],
    });
    assert!(h.add_camera(cc, &r).is_err());
    assert!(h.describe().cameras.is_empty());
}

struct FailingFormat;
impl FormatPlugin for FailingFormat {
    fn descriptor(&self) -> FormatDescriptor {
        let mut d = TopDownFormat.descriptor();
        d.id = FormatId::new("consumer.failable").unwrap();
        d
    }
    fn extract(
        &self,
        r: &ReadFrame,
        b: &RecordBinding,
        s: &SceneRef,
    ) -> Result<PresentationSelection, Error> {
        if r.tick == 99 {
            return Err(Error::new(
                ErrorCode::PluginFailed,
                "injected deterministic extraction failure",
            ));
        }
        TopDownFormat.extract(r, b, s)
    }
}
struct OversizedFormat;
impl FormatPlugin for OversizedFormat {
    fn descriptor(&self) -> FormatDescriptor {
        let mut d = TopDownFormat.descriptor();
        d.id = FormatId::new("consumer.oversized").unwrap();
        d
    }
    fn extract(
        &self,
        r: &ReadFrame,
        b: &RecordBinding,
        s: &SceneRef,
    ) -> Result<PresentationSelection, Error> {
        let mut selected = TopDownFormat.extract(r, b, s)?;
        if r.tick == 99 {
            selected.items = vec![selected.items[0].clone(); 4097];
        }
        Ok(selected)
    }
}

#[test]
fn pure_plugin_failure_and_oversized_result_rollback_already_staged_cameras() {
    let c = core();
    let r = read(&c);
    for (plugin, expected) in [
        (
            Box::new(FailingFormat) as Box<dyn FormatPlugin>,
            ErrorCode::PluginFailed,
        ),
        (
            Box::new(OversizedFormat) as Box<dyn FormatPlugin>,
            ErrorCode::BudgetExceeded,
        ),
    ] {
        let mut h = host(&r);
        h.install(
            plugin,
            config(&r, "view.failable", ViewPolicy::top_down()),
            &r,
        )
        .unwrap();
        camera(&mut h, &r, "cam.a", "view.main", target());
        camera(&mut h, &r, "cam.z", "view.failable", target());
        let mut end = target();
        end.pose.position[0] = 120.0;
        h.update(&r, change(1, "cam.a", end, None)).unwrap();
        let old = h.save().unwrap();
        let mut invalid = r.clone();
        invalid.tick = 99;
        expect(h.update(&invalid, input(2)), expected);
        assert_eq!(h.save().unwrap(), old);
        assert_eq!(h.describe().tick, 1);
        h.update(&r, input(2)).unwrap();
        close(
            h.describe()
                .cameras
                .iter()
                .find(|s| s.id == cid("cam.a"))
                .unwrap()
                .current
                .pose
                .position[0],
            120.0 * (2.0_f64 / 12.0).powi(2) * (3.0 - 2.0 * 2.0 / 12.0),
        );
    }
}

#[test]
fn replacement_failure_and_cross_view_active_set_leave_exact_old_save() {
    let c = core();
    let r = read(&c);
    let mut h = host(&r);
    h.install(
        Box::new(SideFormat),
        config(&r, "view.other", ViewPolicy::side()),
        &r,
    )
    .unwrap();
    camera(&mut h, &r, "cam.a", "view.main", target());
    camera(&mut h, &r, "cam.b", "view.other", target());
    let old = h.save().unwrap();
    assert!(
        h.set_active(&vid("view.main"), BTreeSet::from([cid("cam.b")]))
            .is_err()
    );
    assert_eq!(h.save().unwrap(), old);
    let mut wrong = config(&r, "view.other", ViewPolicy::classic2d());
    assert!(
        h.replace(
            &vid("view.main"),
            Box::new(Classic2DFormat),
            wrong.clone(),
            &r
        )
        .is_err()
    );
    assert_eq!(h.save().unwrap(), old);
    wrong.id = vid("view.main");
    wrong.binding.x_field = "missing".into();
    expect(
        h.replace(&vid("view.main"), Box::new(Classic2DFormat), wrong, &r),
        ErrorCode::InvalidSelection,
    );
    assert_eq!(h.save().unwrap(), old);
}

#[test]
fn core_rng_and_event_continuation_match_control_after_camera_activity() {
    let mut c = core();
    let mut control = core();
    let r = read(&c);
    let mut h = host(&r);
    let mut cc = CameraConfig::new(cid("cam.main"), vid("view.main"), viewport(), target());
    cc.behaviors.shake = Some(ShakeBehavior {
        seed: 8818,
        amplitude: [1.0, 2.0, 3.0],
        duration_ticks: 100,
    });
    h.add_camera(cc, &r).unwrap();
    h.activate(&cid("cam.main")).unwrap();
    let mut t = target();
    t.pose.position[0] = 77.0;
    h.update(&r, change(1, "cam.main", t, None)).unwrap();
    for tick in 2..=20 {
        h.update(&r, input(tick)).unwrap();
    }
    h.remove(&vid("view.main")).unwrap();
    for tick in 1..=10 {
        c.step(InputFrame {
            target_tick: tick,
            actions: vec![],
        })
        .unwrap();
        control
            .step(InputFrame {
                target_tick: tick,
                actions: vec![],
            })
            .unwrap();
        assert_eq!(
            c.save().unwrap(),
            control.save().unwrap(),
            "entire authoritative continuation at tick{tick}"
        );
    }
}

fn add_unknown_to_pose(json: &mut serde_json::Value) -> bool {
    match json {
        serde_json::Value::Object(fields) => {
            if fields.contains_key("zoom") && fields.contains_key("orientation") {
                fields.insert("nested_unknown".into(), true.into());
                return true;
            }
            fields.values_mut().any(add_unknown_to_pose)
        }
        serde_json::Value::Array(items) => items.iter_mut().any(add_unknown_to_pose),
        _ => false,
    }
}

#[test]
fn recursive_unknown_camera_save_fields_are_rejected_without_losing_progress() {
    let c = core();
    let r = read(&c);
    let mut h = host(&r);
    camera(&mut h, &r, "cam.main", "view.main", target());
    let mut t = target();
    t.pose.zoom = 2.0;
    h.update(&r, change(1, "cam.main", t, None)).unwrap();
    let old = h.save().unwrap();
    let mut bad: serde_json::Value = serde_json::from_slice(&old).unwrap();
    assert!(add_unknown_to_pose(&mut bad));
    expect(h.restore(&bytes(&bad), &r), ErrorCode::InvalidSave);
    assert_eq!(h.save().unwrap(), old);
    h.update(&r, input(2)).unwrap();
    assert!(h.describe().cameras[0].transitioning);
}

#[test]
fn changing_view_transform_does_not_become_entity_world_velocity() {
    let c = core();
    let r = read(&c);
    let mut h = host(&r);
    let mut cc = CameraConfig::new(cid("cam.main"), vid("view.main"), viewport(), target());
    let mut f = follow(&r);
    f.look_ahead_ticks = 5;
    cc.behaviors.follow = Some(f);
    h.add_camera(cc, &r).unwrap();
    h.activate(&cid("cam.main")).unwrap();
    h.update(&r, input(1)).unwrap();
    let mut end = target();
    end.view_transform.origin = [13.0, 19.0, 0.0];
    end.view_transform.shear_xy = 1.7;
    let q = std::f64::consts::FRAC_1_SQRT_2;
    end.view_transform.rotation = Quaternion {
        x: 0.0,
        y: 0.0,
        z: q,
        w: q,
    };
    for tick in 2..=13 {
        let f = if tick == 2 {
            h.update(&r, change(tick, "cam.main", end.clone(), None))
        } else {
            h.update(&r, input(tick))
        }
        .unwrap();
        let camera = &f.cameras[0];
        let expected =
            world_to_view(WorldPoint([10.0, 20.0, 0.0]), &camera.target.view_transform).unwrap();
        // Immutable source world position has velocity0 despite changing viewing mode.
        close(camera.target.pose.position[0], expected.0[0]);
        close(camera.target.pose.position[1], expected.0[1]);
    }
}

#[test]
fn inactive_camera_accepts_smooth_target_without_advancing_progress() {
    let c = core();
    let r = read(&c);
    let mut h = host(&r);
    h.add_camera(
        CameraConfig::new(cid("cam.main"), vid("view.main"), viewport(), target()),
        &r,
    )
    .unwrap();
    let mut end = target();
    end.pose.position[0] = 120.0;
    h.update(&r, change(1, "cam.main", end, None)).unwrap();
    close(h.describe().cameras[0].current.pose.position[0], 0.0);
    assert!(h.describe().cameras[0].transitioning);
    let midpoint = h.save().unwrap();
    let mut restored = host(&r);
    restored
        .add_camera(
            CameraConfig::new(cid("cam.main"), vid("view.main"), viewport(), target()),
            &r,
        )
        .unwrap();
    restored.restore(&midpoint, &r).unwrap();
    assert_eq!(restored.save().unwrap(), midpoint);
    h.activate(&cid("cam.main")).unwrap();
    restored.activate(&cid("cam.main")).unwrap();
    let a = h.update(&r, input(2)).unwrap();
    let b = restored.update(&r, input(2)).unwrap();
    assert_eq!(bytes(&a), bytes(&b));
    let u = 1.0_f64 / 12.0;
    close(
        a.cameras[0].target.pose.position[0],
        120.0 * u * u * (3.0 - 2.0 * u),
    );
}

#[test]
fn arbitrary_mesh_billboard_and_background_descriptors_use_real_perspective_math() {
    let c = core();
    let r = read(&c);
    let mut t = target();
    t.projection = Projection::Perspective {
        half_height: 100.0,
        near: 0.1,
        far: 1000.0,
        focus_distance: 100.0,
    };
    for representation in [
        Representation::Model {
            asset: "model.mesh".into(),
        },
        Representation::Background {
            asset: "background.mesh".into(),
        },
        Representation::Billboard {
            asset: "sprite.billboard".into(),
            size: [10.0, 20.0],
        },
    ] {
        let mut b = binding();
        b.representation = representation.clone();
        let extracted = TopDownFormat.extract(&r, &b, &scene(&r)).unwrap();
        assert_eq!(
            bytes(&extracted.items[0].representation),
            bytes(&representation)
        );
        let screen = project(extracted.items[0].world, &t, &viewport()).unwrap();
        assert!(screen.x.is_finite() && screen.y.is_finite());
    }
}

#[test]
fn seeded_well_conditioned_3d_transform_projection_roundtrips() {
    let mut seed = 0xca5e2026_u64;
    let mut sample = || {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((seed >> 11) as f64) / (1_u64 << 53) as f64
    };
    for i in 0..192 {
        let mut t = target();
        t.view_transform.origin = [sample() * 10.0, sample() * 10.0, sample() * 10.0];
        t.view_transform.scale = [0.5 + sample(), 0.5 + sample(), 0.5 + sample()];
        t.view_transform.shear_xy = sample() - 0.5;
        t.view_transform.rotation = unit(Quaternion {
            x: sample(),
            y: sample(),
            z: sample(),
            w: 1.0,
        });
        t.pose.orientation = unit(Quaternion {
            x: sample() * 0.1,
            y: sample() * 0.1,
            z: sample() * 0.1,
            w: 1.0,
        });
        t.pose.zoom = 0.5 + sample();
        let lens = (50.0 + sample() * 100.0, 0.1, 1000.0, 100.0);
        t.projection = match i % 3 {
            0 => Projection::Orthographic {
                half_height: lens.0,
                near: lens.1,
                far: lens.2,
                focus_distance: lens.3,
            },
            1 => Projection::Perspective {
                half_height: lens.0,
                near: lens.1,
                far: lens.2,
                focus_distance: lens.3,
            },
            _ => Projection::Blended {
                half_height: lens.0,
                near: lens.1,
                far: lens.2,
                focus_distance: lens.3,
                perspective_weight: sample(),
            },
        };
        let p = WorldPoint([sample() * 10.0, sample() * 10.0, sample() * 10.0]);
        let s = project(p, &t, &viewport()).unwrap();
        xyz(unproject(s, &t, &viewport()).unwrap().0, p.0);
    }
}

#[test]
fn smooth_minimum_depth_interval_at_large_near_never_rounds_below_valid_gap() {
    let c = core();
    let r = read(&c);
    let mut h = host(&r);
    let mut a = target();
    a.projection = Projection::Orthographic {
        half_height: 100.0,
        near: 25969279.194258943,
        far: 25969279.195258945,
        focus_distance: 100.0,
    };
    let mut b = target();
    b.projection = Projection::Perspective {
        half_height: 100.0,
        near: 30754240.692758758,
        far: 30754240.69375876,
        focus_distance: 100.0,
    };
    camera(&mut h, &r, "cam.main", "view.main", a);
    for tick in 1..=4096 {
        let result = if tick == 1 {
            h.update(
                &r,
                change(
                    tick,
                    "cam.main",
                    b.clone(),
                    Some(TransitionMode::Smooth {
                        duration_ticks: 4096,
                    }),
                ),
            )
        } else {
            h.update(&r, input(tick))
        };
        let frame = result.unwrap_or_else(|e| {
            panic!("valid minimum-gap transition failed at elapsed{tick}: {e}")
        });
        let (near, far) = match frame.cameras[0].target.projection {
            Projection::Orthographic { near, far, .. }
            | Projection::Perspective { near, far, .. }
            | Projection::Blended { near, far, .. } => (near, far),
        };
        assert!(
            far - near >= 0.001,
            "depth interval lost minimum gap at elapsed{tick}: {}",
            far - near
        );
    }
    assert_eq!(
        bytes(&h.frame(&r).unwrap().cameras[0].target),
        bytes(&b),
        "exact requested endpoint preserved"
    );
}

#[test]
fn equivalent_negative_zero_inputs_have_one_canonical_presentation_image() {
    let c = core();
    let r = read(&c);
    let setup = |zero: f64| {
        let mut h = host(&r);
        let mut t = target();
        t.pose.position[0] = zero;
        t.pose.position[1] = zero;
        t.pose.orientation.x = zero;
        t.pose.orientation.y = zero;
        t.pose.orientation.z = zero;
        t.view_transform.origin = [zero; 3];
        t.view_transform.shear_xy = zero;
        t.view_transform.rotation.x = zero;
        t.view_transform.rotation.y = zero;
        t.view_transform.rotation.z = zero;
        let mut cc = CameraConfig::new(
            cid("cam.main"),
            vid("view.main"),
            Viewport {
                x: zero,
                y: zero,
                ..viewport()
            },
            t,
        );
        let mut f = follow(&r);
        f.offset = [zero; 3];
        f.dead_zone = [zero; 3];
        cc.behaviors.follow = Some(f);
        cc.behaviors.bounds = Some(Bounds {
            min: [zero; 3],
            max: [100.0; 3],
        });
        cc.behaviors.shake = Some(ShakeBehavior {
            seed: 1,
            amplitude: [zero; 3],
            duration_ticks: 10,
        });
        h.add_camera(cc, &r).unwrap();
        h.activate(&cid("cam.main")).unwrap();
        h
    };
    let mut positive = setup(0.0);
    let mut negative = setup(-0.0);
    assert_eq!(
        positive.save().unwrap(),
        negative.save().unwrap(),
        "negative zero must be normalized in all accepted config fields"
    );
    let a = positive.update(&r, input(1)).unwrap();
    let b = negative.update(&r, input(1)).unwrap();
    assert_eq!(bytes(&a), bytes(&b));
    assert_eq!(positive.save().unwrap(), negative.save().unwrap());
}

#[test]
fn replacement_validates_inactive_transition_endpoint_under_new_follow_axes() {
    let c = core();
    let r = read(&c);
    let mut h = ViewHost::new("p3.consumer").unwrap();
    h.install(
        Box::new(VerticalFormat),
        config(&r, "view.main", ViewPolicy::vertical()),
        &r,
    )
    .unwrap();
    let mut cc = CameraConfig::new(cid("cam.main"), vid("view.main"), viewport(), target());
    cc.behaviors.follow = Some(follow(&r));
    h.add_camera(cc, &r).unwrap();
    let mut goal = target();
    goal.pose.position[0] = 1e9;
    // Vertical policy contributes noX; endpoint valid while inactive at sampledX0.
    h.update(&r, change(1, "cam.main", goal, None)).unwrap();
    close(h.describe().cameras[0].current.pose.position[0], 0.0);
    let old = h.save().unwrap();
    // TopDown adds actorWorldX10 at the pendingendpoint, exceeding1e9.
    assert!(
        h.replace(
            &vid("view.main"),
            Box::new(TopDownFormat),
            config(&r, "view.main", ViewPolicy::top_down()),
            &r
        )
        .is_err(),
        "replacement must validate dormant future endpoint, not only sampled current pose"
    );
    assert_eq!(h.save().unwrap(), old);
    assert_eq!(
        bytes(&h.describe().installed_views[0].format.family),
        bytes(&ViewFamily::Vertical)
    );
}

//! P2 public contract fixtures. Normal plugins have no mutable native state.
//! External AtomicBool controls deliberately inject nonconforming callback
//! failures; they are not examples of supported plugin state or rollback.
use ge4g_pentomino::p2::*;
use ge4g_pentomino::{CapabilityId, PluginId, Version};
use sha2::Digest;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

fn id(s: &str) -> PluginId {
    PluginId::new(s).unwrap()
}
fn sid(s: &str) -> SchemaId {
    SchemaId::new(s).unwrap()
}
fn aid(s: &str) -> ActionId {
    ActionId::new(s).unwrap()
}
fn cap(s: &str) -> CapabilityId {
    CapabilityId::new(s).unwrap()
}
fn version() -> Version {
    Version {
        major: 2,
        minor: 0,
        patch: 0,
    }
}
fn int() -> FieldType {
    FieldType::I64 {
        min: i64::MIN,
        max: i64::MAX,
    }
}
fn desc(name: &str) -> CoreDescriptor {
    let schema = sid(&format!("{name}.data"));
    CoreDescriptor {
        id: id(name),
        release: version(),
        contract: version(),
        provides: BTreeMap::new(),
        requires: BTreeMap::new(),
        schemas: BTreeMap::from([(
            schema.clone(),
            RecordSchema {
                fields: BTreeMap::from([("n".into(), int()), ("rng".into(), int())]),
            },
        )]),
        event_kinds: BTreeMap::from([("pulse".into(), schema)]),
        actions: BTreeMap::new(),
    }
}
fn record(d: &CoreDescriptor, n: i64, rng: i64) -> Record {
    Record {
        schema: d.schemas.keys().next().unwrap().clone(),
        fields: BTreeMap::from([("n".into(), Value::I64(n)), ("rng".into(), Value::I64(rng))]),
    }
}
fn err<T>(result: Result<T, Error>, expected: ErrorCode) {
    match result {
        Err(e) => assert_eq!(e.code(), expected, "{}", e.detail()),
        Ok(_) => panic!("expected {expected:?}"),
    }
}
fn host() -> CoreHost {
    CoreHost::new(42, "p2:test-v1").unwrap()
}
fn input(tick: u64) -> InputFrame {
    InputFrame {
        target_tick: tick,
        actions: vec![],
    }
}
fn selection(names: &[&str]) -> Selection {
    Selection {
        owners: names.iter().map(|s| id(s)).collect(),
        schemas: BTreeSet::new(),
        include_objects: true,
        include_history: true,
    }
}
fn view(h: &CoreHost, name: &str) -> ReadFrame {
    h.select(&selection(&[name])).unwrap()
}
fn sr(owner: &str, local: &str, incarnation: u64) -> SceneRef {
    SceneRef {
        owner: id(owner),
        local: local.into(),
        incarnation,
    }
}
fn er(owner: &str, local: &str, incarnation: u64) -> EntityRef {
    EntityRef {
        owner: id(owner),
        local: local.into(),
        incarnation,
    }
}
#[derive(Clone)]
enum Mode {
    Quiet,
    Emit(usize),
    EmitAtTick(u64),
    Objects {
        recreate: bool,
        keep_event: bool,
    },
    Static {
        value: Record,
        swallow: bool,
        emit: bool,
    },
    Fault {
        init: Arc<AtomicBool>,
        tick: Arc<AtomicBool>,
        poison: bool,
    },
    Actions {
        foreign: Option<ActionId>,
    },
    Foreign {
        scene: SceneRef,
        delete: bool,
    },
    RemoveScene(SceneRef),
    ManyRecords {
        each: Record,
        count: usize,
    },
    BoundObjects {
        scenes: usize,
        entities: usize,
    },
    DrawBudget(usize),
    Burst(usize),
    DeleteOnlyScene,
    ReferenceOnTick(SceneRef),
    NestedEventRef,
    GrowRecords(Record),
}
struct Fixture {
    d: CoreDescriptor,
    mode: Mode,
}
impl CorePlugin for Fixture {
    fn descriptor(&self) -> CoreDescriptor {
        self.d.clone()
    }
    fn initialize(&self, _: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error> {
        match &self.mode {
            Mode::GrowRecords(record) => {
                for i in 0..100 {
                    tx.set(&format!("r{i}"), record.clone())?;
                }
            }
            Mode::Quiet
            | Mode::Actions { .. }
            | Mode::DrawBudget(_)
            | Mode::ReferenceOnTick(_)
            | Mode::EmitAtTick(_) => {}
            Mode::DeleteOnlyScene => {
                tx.create_scene("main")?;
            }
            Mode::NestedEventRef => {
                let scene = tx.create_scene("main")?;
                let r = typed_record(
                    &self.d,
                    BTreeMap::from([(
                        "refs".into(),
                        Value::List(vec![Value::List(vec![Value::Ref(ObjectRef::Scene(scene))])]),
                    )]),
                );
                tx.set("refs", r.clone())?;
                tx.emit("pulse", r)?;
            }
            Mode::BoundObjects { scenes, entities } => {
                let mut first = None;
                for i in 0..*scenes {
                    let s = tx.create_scene(&format!("s{i}"))?;
                    if first.is_none() {
                        first = Some(s);
                    }
                }
                for i in 0..*entities {
                    tx.create_entity(&format!("e{i}"), first.as_ref().unwrap())?;
                }
            }
            Mode::Burst(count) => {
                for _ in 0..*count {
                    tx.emit("pulse", record(&self.d, 0, 0))?;
                }
            }
            Mode::RemoveScene(_) => {
                let scene = tx.create_scene("main")?;
                tx.create_entity("actor", &scene)?;
            }
            Mode::Emit(_) => {
                let rng = tx.draw()?;
                tx.set("data", record(&self.d, 0, rng as i64))?;
            }
            Mode::Objects { keep_event, .. } => {
                let scene = tx.create_scene("main")?;
                let entity = tx.create_entity("actor", &scene)?;
                let value = object_record(&self.d, scene, entity);
                tx.set("refs", value.clone())?;
                if *keep_event {
                    tx.emit("pulse", value)?;
                }
            }
            Mode::Static {
                value,
                swallow,
                emit,
            } => {
                let result = if *emit {
                    tx.emit("pulse", value.clone())
                } else {
                    tx.set("data", value.clone())
                };
                if !swallow {
                    result?;
                }
            }
            Mode::Fault { init, .. } => {
                let scene = tx.create_scene("main")?;
                tx.create_entity("actor", &scene)?;
                let rng = tx.draw()?;
                tx.set("data", record(&self.d, 0, rng as i64))?;
                tx.emit("pulse", record(&self.d, 0, 7))?;
                if init.load(Ordering::SeqCst) {
                    return Err(Error::new(ErrorCode::PluginFailed, "injected init fault"));
                }
            }
            Mode::Foreign { scene, delete } => {
                if *delete {
                    tx.remove_scene(scene)?;
                } else {
                    tx.create_entity("foreign_child", scene)?;
                }
            }
            Mode::ManyRecords { each, count } => {
                for i in 0..*count {
                    tx.set(&format!("r{i}"), each.clone())?;
                }
            }
        }
        Ok(())
    }
    fn tick(&self, ctx: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error> {
        match &self.mode {
            Mode::EmitAtTick(target) => {
                if ctx.tick() == *target {
                    tx.emit("pulse", record(&self.d, ctx.tick() as i64, 0))?;
                }
            }
            Mode::GrowRecords(record) => {
                tx.draw()?;
                for i in 0..200 {
                    tx.set(&format!("r{i}"), record.clone())?;
                }
            }
            Mode::DrawBudget(count) => {
                for _ in 0..*count {
                    let _ = tx.draw();
                }
            }
            Mode::Burst(count) => {
                for _ in 0..*count {
                    let _ = tx.emit("pulse", record(&self.d, ctx.tick() as i64, 0));
                }
            }
            Mode::DeleteOnlyScene => {
                tx.remove_scene(&SceneRef {
                    owner: self.d.id.clone(),
                    local: "main".into(),
                    incarnation: 1,
                })?;
            }
            Mode::ReferenceOnTick(scene) => {
                ctx.scene(scene)?;
                let r = typed_record(
                    &self.d,
                    BTreeMap::from([(
                        "refs".into(),
                        Value::List(vec![Value::List(vec![Value::Ref(ObjectRef::Scene(
                            scene.clone(),
                        ))])]),
                    )]),
                );
                tx.set("refs", r)?;
            }
            Mode::NestedEventRef => {
                tx.delete("refs")?;
                tx.remove_scene(&SceneRef {
                    owner: self.d.id.clone(),
                    local: "main".into(),
                    incarnation: 1,
                })?;
            }
            Mode::Emit(count) => {
                let draw = tx.draw()?;
                let r = record(&self.d, ctx.tick() as i64, draw as i64);
                tx.set("data", r.clone())?;
                for _ in 0..*count {
                    tx.emit("pulse", r.clone())?;
                }
            }
            Mode::Objects { recreate, .. } if *recreate => {
                let r = ctx.own("refs")?.unwrap();
                let Value::Ref(ObjectRef::Scene(scene)) = &r.fields["scene"] else {
                    panic!("scene field")
                };
                let Value::Ref(ObjectRef::Entity(entity)) = &r.fields["entity"] else {
                    panic!("entity field")
                };
                tx.delete("refs")?;
                tx.remove_entity(entity)?;
                tx.remove_scene(scene)?;
                let scene = tx.create_scene("main")?;
                let entity = tx.create_entity("actor", &scene)?;
                tx.set("refs", object_record(&self.d, scene, entity))?;
            }
            Mode::Fault { tick, poison, .. } => {
                let scene = tx.create_scene(&format!("s{}", ctx.tick()))?;
                tx.create_entity(&format!("e{}", ctx.tick()), &scene)?;
                let rng = tx.draw()?;
                tx.set("data", record(&self.d, ctx.tick() as i64, rng as i64))?;
                tx.emit("pulse", record(&self.d, ctx.tick() as i64, 99))?;
                if *poison {
                    let _ = tx.set(
                        "bad",
                        Record {
                            schema: sid("absent.schema"),
                            fields: BTreeMap::new(),
                        },
                    );
                }
                if tick.load(Ordering::SeqCst) {
                    return Err(Error::new(ErrorCode::PluginFailed, "injected tick fault"));
                }
            }
            Mode::Actions { foreign } => {
                if let Some(other) = foreign {
                    ctx.action(other)?;
                }
                let flag = ctx.action(&aid(&format!("{}.flag", self.d.id.as_str())))?;
                let number = ctx.action(&aid(&format!("{}.number", self.d.id.as_str())))?;
                let n = match number {
                    Some(ActionValue::I64(n)) => *n,
                    None => -1,
                    _ => panic!("type validated"),
                };
                let rng = match flag {
                    Some(ActionValue::Bool(true)) => 1,
                    Some(ActionValue::Bool(false)) => 0,
                    None => -1,
                    _ => panic!("type validated"),
                };
                tx.set("data", record(&self.d, n, rng))?;
            }
            Mode::RemoveScene(scene) => {
                tx.remove_scene(scene)?;
            }
            _ => {}
        }
        Ok(())
    }
}
fn install(h: &mut CoreHost, name: &str, mode: Mode) {
    h.install(
        Box::new(Fixture {
            d: desc(name),
            mode,
        }),
        BTreeMap::new(),
    )
    .unwrap();
}
fn objects_desc(name: &str) -> CoreDescriptor {
    let mut d = desc(name);
    let schema = sid(&format!("{name}.data"));
    d.schemas.insert(
        schema,
        RecordSchema {
            fields: BTreeMap::from([
                (
                    "scene".into(),
                    FieldType::Ref {
                        kind: RefKind::Scene,
                    },
                ),
                (
                    "entity".into(),
                    FieldType::Ref {
                        kind: RefKind::Entity,
                    },
                ),
            ]),
        },
    );
    d
}
fn object_record(d: &CoreDescriptor, scene: SceneRef, entity: EntityRef) -> Record {
    Record {
        schema: d.schemas.keys().next().unwrap().clone(),
        fields: BTreeMap::from([
            ("scene".into(), Value::Ref(ObjectRef::Scene(scene))),
            ("entity".into(), Value::Ref(ObjectRef::Entity(entity))),
        ]),
    }
}
fn install_objects(h: &mut CoreHost, name: &str, recreate: bool, keep_event: bool) {
    h.install(
        Box::new(Fixture {
            d: objects_desc(name),
            mode: Mode::Objects {
                recreate,
                keep_event,
            },
        }),
        BTreeMap::new(),
    )
    .unwrap();
}
fn replace(bytes: &[u8], old: &str, new: &str) -> Vec<u8> {
    let s = std::str::from_utf8(bytes).unwrap();
    assert!(s.contains(old), "pattern missing {old}");
    s.replacen(old, new, 1).into_bytes()
}
fn reject_restore(h: &mut CoreHost, bytes: &[u8], code: ErrorCode) {
    let before = h.save().unwrap();
    let frame = h
        .select(&Selection {
            owners: h
                .describe()
                .plugins
                .iter()
                .map(|p| p.descriptor.id.clone())
                .collect(),
            schemas: BTreeSet::new(),
            include_objects: true,
            include_history: true,
        })
        .unwrap();
    let tokens: Vec<_> = h
        .describe()
        .plugins
        .iter()
        .map(|p| h.owner(&p.descriptor.id).unwrap())
        .collect();
    let scenes: Vec<_> = frame
        .scenes
        .iter()
        .map(|s| h.scene_handle(&s.reference).unwrap())
        .collect();
    let entities: Vec<_> = frame
        .entities
        .iter()
        .map(|e| h.entity_handle(&e.reference).unwrap())
        .collect();
    err(h.restore(bytes), code);
    assert_eq!(h.save().unwrap(), before);
    for (p, token) in h.describe().plugins.iter().zip(tokens) {
        assert_eq!(h.owner(&p.descriptor.id).unwrap(), token);
    }
    for handle in scenes {
        h.resolve_scene(handle).unwrap();
    }
    for handle in entities {
        h.resolve_entity(handle).unwrap();
    }
}

#[test]
fn owner_full_history_pending_counters_and_rng_are_isolated_past_global_256() {
    let mut combined = host();
    let mut alone = host();
    install(&mut combined, "test.a", Mode::Emit(64));
    install(&mut combined, "test.b", Mode::Emit(64));
    install(&mut alone, "test.b", Mode::Emit(64));
    for tick in 1..=5 {
        combined.step(input(tick)).unwrap();
        alone.step(input(tick)).unwrap();
    }
    let normalize = |mut f: ReadFrame| {
        for h in f.histories.values_mut() {
            for e in h.history.iter_mut().chain(&mut h.pending) {
                e.sequence = 0;
            }
        }
        f
    };
    assert_eq!(
        normalize(view(&combined, "test.b")),
        normalize(view(&alone, "test.b"))
    );
    let b = view(&combined, "test.b");
    let history = &b.histories[&id("test.b")];
    assert_eq!(
        (
            history.emitted,
            history.dropped,
            history.history.len(),
            history.pending.len()
        ),
        (320, 64, 256, 64)
    );
    assert!(
        history
            .history
            .iter()
            .enumerate()
            .all(|(i, e)| e.ordinal == 64 + i as u64)
    );
    let before_b = view(&combined, "test.b");
    combined
        .remove(combined.owner(&id("test.a")).unwrap())
        .unwrap();
    assert_eq!(view(&combined, "test.b"), before_b);
    let bytes = combined.save().unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(json["next_sequence"], 640);
    assert_eq!(json["retired_emitted"], 320);
    assert_eq!(json["purged_retained"], 256);
    assert_eq!(
        json["plugins"][0]["rng_state"],
        serde_json::from_slice::<serde_json::Value>(&alone.save().unwrap()).unwrap()["plugins"][0]
            ["rng_state"]
    );
    combined.restore(&bytes).unwrap();
    combined.step(input(6)).unwrap();
    alone.step(input(6)).unwrap();
    assert_eq!(
        normalize(view(&combined, "test.b")),
        normalize(view(&alone, "test.b"))
    );
}

#[test]
fn deletion_recreation_crosshost_restore_and_abandoned_timeline_handles() {
    let mut h = host();
    install_objects(&mut h, "test.a", true, false);
    let initial = h.save().unwrap();
    let s = sr("test.a", "main", 1);
    let e = er("test.a", "actor", 2);
    let old_scene = h.scene_handle(&s).unwrap();
    let old_entity = h.entity_handle(&e).unwrap();
    let old_owner = h.owner(&id("test.a")).unwrap();
    let mut other = host();
    install_objects(&mut other, "test.a", true, false);
    err(
        other.resolve_scene(old_scene.clone()),
        ErrorCode::StaleHandle,
    );
    err(
        other.resolve_entity(old_entity.clone()),
        ErrorCode::StaleHandle,
    );
    h.step(input(1)).unwrap();
    err(h.resolve_scene(old_scene), ErrorCode::StaleHandle);
    err(h.resolve_entity(old_entity), ErrorCode::StaleHandle);
    let future = view(&h, "test.a");
    assert_eq!(future.scenes[0].reference.incarnation, 3);
    assert_eq!(future.entities[0].reference.incarnation, 4);
    let future_handle = h.scene_handle(&future.scenes[0].reference).unwrap();
    let future_bytes = h.save().unwrap();
    h.restore(&initial).unwrap();
    err(
        h.resolve_scene(future_handle.clone()),
        ErrorCode::StaleHandle,
    );
    err(h.remove(old_owner), ErrorCode::StaleHandle);
    h.step(input(1)).unwrap();
    assert_eq!(h.save().unwrap(), future_bytes);
    assert_eq!(
        view(&h, "test.a").scenes[0].reference,
        future.scenes[0].reference
    );
    err(h.resolve_scene(future_handle), ErrorCode::StaleHandle);
    let current = h.scene_handle(&future.scenes[0].reference).unwrap();
    h.remove(h.owner(&id("test.a")).unwrap()).unwrap();
    err(h.resolve_scene(current), ErrorCode::StaleHandle);
    install_objects(&mut h, "test.a", false, false);
    assert_eq!(view(&h, "test.a").scenes[0].reference.incarnation, 5);
}

#[test]
fn retained_event_references_prevent_delete_and_swallowed_reference_error_is_atomic() {
    let mut h = host();
    install_objects(&mut h, "test.a", true, true);
    let before = h.save().unwrap();
    err(h.step(input(1)), ErrorCode::ReferenceInUse);
    assert_eq!(h.save().unwrap(), before);
    assert_eq!(
        h.resolve_entity(h.entity_handle(&er("test.a", "actor", 2)).unwrap())
            .unwrap()
            .scene,
        sr("test.a", "main", 1)
    );
}

#[test]
fn provider_objects_allow_bound_child_refs_block_removal_and_deny_foreign_writes() {
    let mut h = host();
    let mut provider = desc("test.provider");
    provider.provides.insert(cap("test.objects"), version());
    h.install(
        Box::new(Fixture {
            d: provider,
            mode: Mode::Fault {
                init: Arc::new(AtomicBool::new(false)),
                tick: Arc::new(AtomicBool::new(false)),
                poison: false,
            },
        }),
        BTreeMap::new(),
    )
    .unwrap();
    let foreign = sr("test.provider", "main", 1);
    let mut reader = desc("test.reader");
    reader.requires.insert(cap("test.objects"), version());
    let bindings = BTreeMap::from([(cap("test.objects"), id("test.provider"))]);
    let before = h.save().unwrap();
    err(
        h.install(
            Box::new(Fixture {
                d: reader.clone(),
                mode: Mode::Foreign {
                    scene: foreign.clone(),
                    delete: true,
                },
            }),
            bindings.clone(),
        ),
        ErrorCode::PermissionDenied,
    );
    assert_eq!(h.save().unwrap(), before);
    err(
        h.install(
            Box::new(Fixture {
                d: desc("test.spy"),
                mode: Mode::Foreign {
                    scene: foreign.clone(),
                    delete: false,
                },
            }),
            BTreeMap::new(),
        ),
        ErrorCode::PermissionDenied,
    );
    h.install(
        Box::new(Fixture {
            d: reader,
            mode: Mode::Foreign {
                scene: foreign,
                delete: false,
            },
        }),
        bindings,
    )
    .unwrap();
    assert_eq!(
        view(&h, "test.reader").entities[0].scene,
        sr("test.provider", "main", 1)
    );
    let before = h.hash().unwrap();
    err(
        h.remove(h.owner(&id("test.provider")).unwrap()),
        ErrorCode::DependencyInUse,
    );
    assert_eq!(h.hash().unwrap(), before);
    h.remove(h.owner(&id("test.reader")).unwrap()).unwrap();
    h.remove(h.owner(&id("test.provider")).unwrap()).unwrap();
}

#[test]
fn whole_init_tick_rolls_back_objects_identity_rng_events_and_last_input_retry() {
    let mut h = host();
    let mut clean = host();
    let init_fault = Arc::new(AtomicBool::new(true));
    let tick_fault = Arc::new(AtomicBool::new(true));
    install(&mut h, "test.a", Mode::Emit(1));
    install(&mut clean, "test.a", Mode::Emit(1));
    let before = h.save().unwrap();
    let make = |init: Arc<AtomicBool>, tick: Arc<AtomicBool>| {
        Box::new(Fixture {
            d: action_desc("test.z"),
            mode: Mode::Fault {
                init,
                tick,
                poison: false,
            },
        }) as Box<dyn CorePlugin>
    };
    err(
        h.install(
            make(init_fault.clone(), tick_fault.clone()),
            BTreeMap::new(),
        ),
        ErrorCode::PluginFailed,
    );
    assert_eq!(h.save().unwrap(), before);
    init_fault.store(false, Ordering::SeqCst);
    h.install(make(init_fault, tick_fault.clone()), BTreeMap::new())
        .unwrap();
    clean
        .install(
            make(
                Arc::new(AtomicBool::new(false)),
                Arc::new(AtomicBool::new(false)),
            ),
            BTreeMap::new(),
        )
        .unwrap();
    assert_eq!(h.save().unwrap(), clean.save().unwrap());
    let scene = view(&h, "test.z").scenes[0].reference.clone();
    let handle = h.scene_handle(&scene).unwrap();
    let before = h.save().unwrap();
    let retry = InputFrame {
        target_tick: 1,
        actions: vec![(aid("test.z.flag"), ActionValue::Bool(true))],
    };
    err(h.step(retry.clone()), ErrorCode::PluginFailed);
    assert_eq!(h.save().unwrap(), before);
    h.resolve_scene(handle).unwrap();
    assert_eq!(h.describe().last_input, None);
    tick_fault.store(false, Ordering::SeqCst);
    h.step(retry.clone()).unwrap();
    clean.step(retry.clone()).unwrap();
    assert_eq!(h.describe().last_input, Some(retry));
    assert_eq!(h.save().unwrap(), clean.save().unwrap());
    h.step(input(2)).unwrap();
    clean.step(input(2)).unwrap();
    assert_eq!(h.hash().unwrap(), clean.hash().unwrap());
    let mut poisoned = host();
    install(
        &mut poisoned,
        "test.a",
        Mode::Fault {
            init: Arc::new(AtomicBool::new(false)),
            tick: Arc::new(AtomicBool::new(false)),
            poison: true,
        },
    );
    let before = poisoned.save().unwrap();
    err(poisoned.step(input(1)), ErrorCode::InvalidRecord);
    assert_eq!(poisoned.save().unwrap(), before);
}

fn action_desc(name: &str) -> CoreDescriptor {
    let mut d = desc(name);
    d.actions
        .insert(aid(&format!("{name}.flag")), ActionType::Bool);
    d.actions.insert(
        aid(&format!("{name}.number")),
        ActionType::I64 { min: -4, max: 4 },
    );
    d
}
#[test]
fn action_unique_types_bounds_visibility_sorting_absence_and_saved_frame_replay() {
    let mut h = host();
    h.install(
        Box::new(Fixture {
            d: action_desc("test.a"),
            mode: Mode::Actions { foreign: None },
        }),
        BTreeMap::new(),
    )
    .unwrap();
    let flag = aid("test.a.flag");
    let number = aid("test.a.number");
    for (frame, code) in [
        (input(0), ErrorCode::InputAlreadyCommitted),
        (input(2), ErrorCode::InputOutOfSequence),
        (
            InputFrame {
                target_tick: 1,
                actions: vec![
                    (flag.clone(), ActionValue::Bool(true)),
                    (flag.clone(), ActionValue::Bool(false)),
                ],
            },
            ErrorCode::InvalidInput,
        ),
        (
            InputFrame {
                target_tick: 1,
                actions: vec![(flag.clone(), ActionValue::I64(1))],
            },
            ErrorCode::InvalidInput,
        ),
        (
            InputFrame {
                target_tick: 1,
                actions: vec![(number.clone(), ActionValue::I64(5))],
            },
            ErrorCode::InvalidInput,
        ),
        (
            InputFrame {
                target_tick: 1,
                actions: vec![(aid("test.unknown"), ActionValue::Bool(true))],
            },
            ErrorCode::InvalidInput,
        ),
    ] {
        let before = h.save().unwrap();
        err(h.step(frame), code);
        assert_eq!(h.save().unwrap(), before);
    }
    h.step(InputFrame {
        target_tick: 1,
        actions: vec![
            (number.clone(), ActionValue::I64(-4)),
            (flag.clone(), ActionValue::Bool(true)),
        ],
    })
    .unwrap();
    assert_eq!(
        h.describe().last_input.unwrap().actions,
        vec![
            (flag, ActionValue::Bool(true)),
            (number, ActionValue::I64(-4))
        ]
    );
    let r = &view(&h, "test.a").records[&RecordKey {
        owner: id("test.a"),
        local: "data".into(),
    }];
    assert_eq!(r.fields["n"], Value::I64(-4));
    assert_eq!(r.fields["rng"], Value::I64(1));
    let bytes = h.save().unwrap();
    h.step(input(2)).unwrap();
    let next = h.save().unwrap();
    h.restore(&bytes).unwrap();
    h.step(input(2)).unwrap();
    assert_eq!(h.save().unwrap(), next);
    assert_eq!(
        view(&h, "test.a").records[&RecordKey {
            owner: id("test.a"),
            local: "data".into()
        }]
            .fields["rng"],
        Value::I64(-1)
    );
    let before = h.hash().unwrap();
    err(h.step(input(2)), ErrorCode::InputAlreadyCommitted);
    assert_eq!(h.hash().unwrap(), before);
    let mut spy = desc("test.spy");
    spy.actions = action_desc("test.spy").actions;
    h.install(
        Box::new(Fixture {
            d: spy,
            mode: Mode::Actions {
                foreign: Some(aid("test.a.flag")),
            },
        }),
        BTreeMap::new(),
    )
    .unwrap();
    let before = h.save().unwrap();
    err(h.step(input(3)), ErrorCode::PermissionDenied);
    assert_eq!(h.save().unwrap(), before);
}

#[test]
fn select_empty_flags_schema_filters_and_unknown_ids_obey_host_authority() {
    let mut h = host();
    install_objects(&mut h, "test.a", false, false);
    let empty = h.select(&selection(&[])).unwrap();
    assert!(empty.records.is_empty() && empty.scenes.is_empty() && empty.histories.is_empty());
    let mut selected = selection(&["test.a"]);
    selected.include_history = false;
    selected.include_objects = false;
    selected.schemas.insert(sid("test.a.data"));
    let f = h.select(&selected).unwrap();
    assert_eq!(f.records.len(), 1);
    assert!(f.scenes.is_empty() && f.entities.is_empty() && f.histories.is_empty());
    err(
        h.select(&selection(&["test.missing"])),
        ErrorCode::StaleHandle,
    );
    selected.schemas.insert(sid("absent.schema"));
    err(h.select(&selected), ErrorCode::UndeclaredKey);
}

fn typed_desc(name: &str, fields: BTreeMap<String, FieldType>) -> CoreDescriptor {
    let mut d = desc(name);
    d.schemas.values_mut().next().unwrap().fields = fields;
    d
}
fn typed_record(d: &CoreDescriptor, fields: BTreeMap<String, Value>) -> Record {
    Record {
        schema: d.schemas.keys().next().unwrap().clone(),
        fields,
    }
}
#[test]
fn typed_bool_integer_utf8_bytes_homogeneous_lists_roundtrip_and_reject_exactly() {
    let fields = BTreeMap::from([
        ("bool".into(), FieldType::Bool),
        ("int".into(), FieldType::I64 { min: -4, max: 4 }),
        ("text".into(), FieldType::String { max_bytes: 6 }),
        ("bytes".into(), FieldType::Bytes { max_bytes: 4 }),
        (
            "list".into(),
            FieldType::List {
                item: Box::new(FieldType::Bool),
                max_items: 2,
            },
        ),
    ]);
    let d = typed_desc("test.typed", fields);
    let values = BTreeMap::from([
        ("bool".into(), Value::Bool(true)),
        ("int".into(), Value::I64(-4)),
        ("text".into(), Value::String("항구".into())),
        ("bytes".into(), Value::Bytes(vec![0, 255, 1, 2])),
        (
            "list".into(),
            Value::List(vec![Value::Bool(true), Value::Bool(false)]),
        ),
    ]);
    let r = typed_record(&d, values.clone());
    let mut h = host();
    h.install(
        Box::new(Fixture {
            d: d.clone(),
            mode: Mode::Static {
                value: r.clone(),
                swallow: false,
                emit: false,
            },
        }),
        BTreeMap::new(),
    )
    .unwrap();
    let bytes = h.save().unwrap();
    h.restore(&bytes).unwrap();
    assert_eq!(
        view(&h, "test.typed").records[&RecordKey {
            owner: id("test.typed"),
            local: "data".into()
        }],
        r
    );
    let mut cases = vec![];
    for (field, bad) in [
        ("bool", Value::I64(1)),
        ("int", Value::I64(5)),
        ("text", Value::String("항구x".into())),
        ("bytes", Value::Bytes(vec![0; 5])),
        ("list", Value::List(vec![Value::Bool(true), Value::I64(0)])),
        ("list", Value::List(vec![Value::Bool(true); 3])),
    ] {
        let mut v = values.clone();
        v.insert(field.into(), bad);
        cases.push(v);
    }
    let mut missing = values.clone();
    missing.remove("bool");
    cases.push(missing);
    let mut extra = values;
    extra.insert("extra".into(), Value::Bool(true));
    cases.push(extra);
    for fields in cases {
        for (emit, swallow) in [(false, false), (true, true)] {
            let mut h = host();
            let before = h.save().unwrap();
            err(
                h.install(
                    Box::new(Fixture {
                        d: d.clone(),
                        mode: Mode::Static {
                            value: typed_record(&d, fields.clone()),
                            swallow,
                            emit,
                        },
                    }),
                    BTreeMap::new(),
                ),
                ErrorCode::InvalidRecord,
            );
            assert_eq!(h.save().unwrap(), before);
        }
    }
}

#[test]
fn schema_definition_depth_bounds_duplicate_schema_and_actions_reject_before_init() {
    let list = |item| FieldType::List {
        item: Box::new(item),
        max_items: 1,
    };
    let legal = list(list(list(FieldType::Bool)));
    let mut h = host();
    let d = typed_desc("test.depth", BTreeMap::from([("value".into(), legal)]));
    let value = Value::List(vec![Value::List(vec![Value::List(vec![Value::Bool(
        true,
    )])])]);
    h.install(
        Box::new(Fixture {
            d: d.clone(),
            mode: Mode::Static {
                value: typed_record(&d, BTreeMap::from([("value".into(), value)])),
                emit: false,
                swallow: false,
            },
        }),
        BTreeMap::new(),
    )
    .unwrap();
    for kind in [
        list(list(list(list(FieldType::Bool)))),
        FieldType::String { max_bytes: 0 },
        FieldType::String { max_bytes: 1025 },
        FieldType::Bytes { max_bytes: 4097 },
        FieldType::List {
            item: Box::new(FieldType::Bool),
            max_items: 65,
        },
        FieldType::I64 { min: 2, max: 1 },
    ] {
        let mut h = host();
        let before = h.hash().unwrap();
        let d = typed_desc("test.bad", BTreeMap::from([("field".into(), kind)]));
        err(
            h.install(
                Box::new(Fixture {
                    d,
                    mode: Mode::Quiet,
                }),
                BTreeMap::new(),
            ),
            ErrorCode::InvalidSchema,
        );
        assert_eq!(h.hash().unwrap(), before);
    }
    let mut d = desc("test.other");
    d.schemas = h.describe().plugins[0].descriptor.schemas.clone();
    d.event_kinds.clear();
    let before = h.save().unwrap();
    err(
        h.install(
            Box::new(Fixture {
                d,
                mode: Mode::Quiet,
            }),
            BTreeMap::new(),
        ),
        ErrorCode::DuplicateId,
    );
    assert_eq!(h.save().unwrap(), before);
    let mut a = host();
    a.install(
        Box::new(Fixture {
            d: action_desc("test.a"),
            mode: Mode::Quiet,
        }),
        BTreeMap::new(),
    )
    .unwrap();
    let mut b = desc("test.b");
    b.actions = action_desc("test.a").actions;
    let before = a.save().unwrap();
    err(
        a.install(
            Box::new(Fixture {
                d: b,
                mode: Mode::Quiet,
            }),
            BTreeMap::new(),
        ),
        ErrorCode::DuplicateId,
    );
    assert_eq!(a.save().unwrap(), before);
}

#[test]
fn legal_field_and_list_values_over_64k_reject_before_accepted_state_clone() {
    let d = typed_desc(
        "test.big",
        (0..32)
            .map(|i| (format!("f{i}"), FieldType::String { max_bytes: 1024 }))
            .collect(),
    );
    let r = typed_record(
        &d,
        (0..32)
            .map(|i| (format!("f{i}"), Value::String("\0".repeat(1024))))
            .collect(),
    );
    let list_d = typed_desc(
        "test.list",
        BTreeMap::from([(
            "items".into(),
            FieldType::List {
                item: Box::new(FieldType::String { max_bytes: 1024 }),
                max_items: 64,
            },
        )]),
    );
    let list_r = typed_record(
        &list_d,
        BTreeMap::from([(
            "items".into(),
            Value::List(vec![Value::String("x".repeat(1024)); 64]),
        )]),
    );
    for (d, r) in [(d, r), (list_d, list_r)] {
        for emit in [false, true] {
            let mut h = host();
            let before = h.save().unwrap();
            err(
                h.install(
                    Box::new(Fixture {
                        d: d.clone(),
                        mode: Mode::Static {
                            value: r.clone(),
                            swallow: true,
                            emit,
                        },
                    }),
                    BTreeMap::new(),
                ),
                ErrorCode::BudgetExceeded,
            );
            assert_eq!(h.save().unwrap(), before);
        }
    }
}

#[test]
fn aggregate_over_8mib_install_is_atomic_and_existing_tokens_remain_valid() {
    let mut h = host();
    let make = |name| {
        let d = typed_desc(
            name,
            BTreeMap::from([(
                "items".into(),
                FieldType::List {
                    item: Box::new(FieldType::String { max_bytes: 1000 }),
                    max_items: 48,
                },
            )]),
        );
        let each = typed_record(
            &d,
            BTreeMap::from([(
                "items".into(),
                Value::List(vec![Value::String("x".repeat(1000)); 48]),
            )]),
        );
        Fixture {
            d,
            mode: Mode::ManyRecords { each, count: 100 },
        }
    };
    h.install(Box::new(make("test.a")), BTreeMap::new())
        .unwrap();
    let token = h.owner(&id("test.a")).unwrap();
    let before = h.save().unwrap();
    assert!(before.len() < 8_388_608);
    err(
        h.install(Box::new(make("test.b")), BTreeMap::new()),
        ErrorCode::BudgetExceeded,
    );
    assert_eq!(h.save().unwrap(), before);
    assert_eq!(h.owner(&id("test.a")).unwrap(), token);
}

#[test]
fn canonical_save2_roundtrip_hash_continuation_and_malformed_atomic_rejection() {
    let mut h = host();
    install_objects(&mut h, "test.a", false, false);
    h.step(input(1)).unwrap();
    let bytes = h.save().unwrap();
    assert_eq!(
        h.hash().unwrap(),
        format!("{:x}", sha2::Sha256::digest(&bytes))
    );
    let owner = h.owner(&id("test.a")).unwrap();
    let scene = h.scene_handle(&sr("test.a", "main", 1)).unwrap();
    let entity = h.entity_handle(&er("test.a", "actor", 2)).unwrap();
    h.step(input(2)).unwrap();
    let next = h.save().unwrap();
    h.restore(&bytes).unwrap();
    assert_eq!(h.save().unwrap(), bytes);
    err(h.remove(owner), ErrorCode::StaleHandle);
    err(h.resolve_scene(scene), ErrorCode::StaleHandle);
    err(h.resolve_entity(entity), ErrorCode::StaleHandle);
    h.step(input(2)).unwrap();
    assert_eq!(h.save().unwrap(), next);
    let bytes = h.save().unwrap();
    for malformed in [
        b"{".to_vec(),
        [b" ".as_slice(), bytes.as_slice()].concat(),
        [bytes.as_slice(), b"{}".as_slice()].concat(),
        replace(
            &bytes,
            "\"format_version\":2",
            "\"format_version\":2,\"extra\":1",
        ),
        replace(
            &bytes,
            "\"format_version\":2",
            "\"format_version\":2,\"format_version\":2",
        ),
        replace(&bytes, "\"next_identity\":3", "\"next_identity\":1"),
        replace(&bytes, "\"incarnation\":1", "\"incarnation\":0"),
        replace(&bytes, "\"target_tick\":2", "\"target_tick\":1"),
        replace(&bytes, "\"retired_emitted\":0", "\"retired_emitted\":1"),
        replace(&bytes, "\"purged_retained\":0", "\"purged_retained\":1"),
        vec![b' '; 8_388_609],
    ] {
        reject_restore(&mut h, &malformed, ErrorCode::InvalidSave);
    }
    reject_restore(
        &mut h,
        &replace(&bytes, "\"format_version\":2", "\"format_version\":1"),
        ErrorCode::VersionMismatch,
    );
    reject_restore(
        &mut h,
        &replace(&bytes, "\"seed\":42", "\"seed\":43"),
        ErrorCode::SaveMismatch,
    );
    reject_restore(
        &mut h,
        &replace(
            &bytes,
            "\"content_binding\":\"p2:test-v1\"",
            "\"content_binding\":\"p2:other\"",
        ),
        ErrorCode::SaveMismatch,
    );
}

#[test]
fn save_owner_ordinal_suffix_pending_completeness_global_counts_and_actions_validate() {
    let mut h = host();
    install(&mut h, "test.a", Mode::Emit(1));
    h.step(input(1)).unwrap();
    h.step(input(2)).unwrap();
    let bytes = h.save().unwrap();
    for bad in [
        replace(&bytes, "\"ordinal\":0", "\"ordinal\":1"),
        replace(&bytes, "\"emitted\":2", "\"emitted\":3"),
        replace(&bytes, "\"dropped\":0", "\"dropped\":1"),
        replace(&bytes, "\"next_sequence\":2", "\"next_sequence\":3"),
        replace(&bytes, "\"sequence\":0", "\"sequence\":1"),
    ] {
        reject_restore(&mut h, &bad, ErrorCode::InvalidSave);
    }
    let text = String::from_utf8(bytes.clone()).unwrap();
    let pending_start = text.find("\"pending\":[").unwrap() + "\"pending\":[".len();
    let pending_end = text[pending_start..].find("]}").unwrap() + pending_start;
    let omitted = format!("{}{}", &text[..pending_start], &text[pending_end..]);
    reject_restore(&mut h, omitted.as_bytes(), ErrorCode::InvalidSave);
    // Keep emitted = retained + dropped and global totals equal, while violating
    // the required full newest suffix: two events cannot represent emitted3.
    let forged = replace(
        &replace(
            &replace(&bytes, "\"emitted\":2", "\"emitted\":3"),
            "\"dropped\":0",
            "\"dropped\":1",
        ),
        "\"next_sequence\":2",
        "\"next_sequence\":3",
    );
    reject_restore(&mut h, &forged, ErrorCode::InvalidSave);
    let mut h = host();
    h.install(
        Box::new(Fixture {
            d: action_desc("test.a"),
            mode: Mode::Actions { foreign: None },
        }),
        BTreeMap::new(),
    )
    .unwrap();
    h.step(InputFrame {
        target_tick: 1,
        actions: vec![(aid("test.a.flag"), ActionValue::Bool(true))],
    })
    .unwrap();
    let bytes = h.save().unwrap();
    reject_restore(
        &mut h,
        &replace(
            &bytes,
            "\"actions\":[[\"test.a.flag\",{\"type\":\"bool\",\"value\":true}]]",
            "\"actions\":[[\"test.a.flag\",{\"type\":\"i64\",\"value\":1}]]",
        ),
        ErrorCode::InvalidSave,
    );
    h.remove(h.owner(&id("test.a")).unwrap()).unwrap();
    assert!(h.describe().last_input.unwrap().actions.is_empty());
    let saved = h.save().unwrap();
    h.restore(&saved).unwrap();
}

#[test]
fn nested_record_refs_require_exact_live_kind_incarnation_and_provider_grant() {
    let nested = FieldType::List {
        item: Box::new(FieldType::List {
            item: Box::new(FieldType::Ref {
                kind: RefKind::Scene,
            }),
            max_items: 2,
        }),
        max_items: 2,
    };
    let d = typed_desc("test.reader", BTreeMap::from([("refs".into(), nested)]));
    for (reference, code) in [
        (
            ObjectRef::Scene(sr("test.reader", "main", 1)),
            ErrorCode::InvalidReference,
        ),
        (
            ObjectRef::Entity(er("test.reader", "actor", 2)),
            ErrorCode::InvalidRecord,
        ),
    ] {
        let mut h = host();
        let r = typed_record(
            &d,
            BTreeMap::from([(
                "refs".into(),
                Value::List(vec![Value::List(vec![Value::Ref(reference)])]),
            )]),
        );
        let before = h.save().unwrap();
        err(
            h.install(
                Box::new(Fixture {
                    d: d.clone(),
                    mode: Mode::Static {
                        value: r,
                        swallow: true,
                        emit: false,
                    },
                }),
                BTreeMap::new(),
            ),
            code,
        );
        assert_eq!(h.save().unwrap(), before);
    }
    let mut h = host();
    install_objects(&mut h, "test.a", false, false);
    let bytes = h.save().unwrap();
    let dead = replace(&bytes, "\"incarnation\":1", "\"incarnation\":99");
    reject_restore(&mut h, &dead, ErrorCode::InvalidSave);
}

#[test]
fn scene_with_live_child_cannot_be_removed_without_cascade() {
    let mut h = host();
    install(&mut h, "test.a", Mode::RemoveScene(sr("test.a", "main", 1)));
    let before = h.save().unwrap();
    err(h.step(input(1)), ErrorCode::ReferenceInUse);
    assert_eq!(h.save().unwrap(), before);
}

#[test]
fn scene_entity_record_commands_and_per_owner_event_limits_have_exact_boundaries() {
    for (scenes, entities, okay) in [(64, 256, true), (65, 0, false), (1, 257, false)] {
        let mut h = host();
        let before = h.save().unwrap();
        let result = h.install(
            Box::new(Fixture {
                d: desc("test.a"),
                mode: Mode::BoundObjects { scenes, entities },
            }),
            BTreeMap::new(),
        );
        if okay {
            result.unwrap();
            assert_eq!(
                (
                    view(&h, "test.a").scenes.len(),
                    view(&h, "test.a").entities.len()
                ),
                (64, 256)
            );
        } else {
            err(result, ErrorCode::BudgetExceeded);
            assert_eq!(h.save().unwrap(), before);
        }
    }
    for count in [256, 257] {
        let mut h = host();
        let d = desc("test.a");
        let each = record(&d, 1, 1);
        let before = h.save().unwrap();
        let result = h.install(
            Box::new(Fixture {
                d,
                mode: Mode::ManyRecords { each, count },
            }),
            BTreeMap::new(),
        );
        if count == 256 {
            result.unwrap();
            assert_eq!(view(&h, "test.a").records.len(), 256);
        } else {
            err(result, ErrorCode::BudgetExceeded);
            assert_eq!(h.save().unwrap(), before);
        }
    }
    for count in [4096, 4097] {
        let mut h = host();
        install(&mut h, "test.a", Mode::DrawBudget(count));
        let before = h.save().unwrap();
        if count == 4096 {
            h.step(input(1)).unwrap();
            assert_ne!(h.save().unwrap(), before);
        } else {
            err(h.step(input(1)), ErrorCode::BudgetExceeded);
            assert_eq!(h.save().unwrap(), before);
        }
    }
    let mut h = host();
    install(&mut h, "test.a", Mode::Burst(128));
    install(&mut h, "test.b", Mode::Burst(128));
    h.step(input(1)).unwrap();
    for owner in ["test.a", "test.b"] {
        let f = view(&h, owner);
        assert_eq!(
            (
                f.histories[&id(owner)].history.len(),
                f.histories[&id(owner)].pending.len()
            ),
            (256, 128)
        );
    }
    let mut h = host();
    let before = h.save().unwrap();
    err(
        h.install(
            Box::new(Fixture {
                d: desc("test.a"),
                mode: Mode::Burst(129),
            }),
            BTreeMap::new(),
        ),
        ErrorCode::BudgetExceeded,
    );
    assert_eq!(h.save().unwrap(), before);
}

#[test]
fn late_cross_owner_ref_and_nested_history_ref_cannot_publish_dangling_objects() {
    let nested = FieldType::List {
        item: Box::new(FieldType::List {
            item: Box::new(FieldType::Ref {
                kind: RefKind::Scene,
            }),
            max_items: 2,
        }),
        max_items: 2,
    };
    let mut h = host();
    let mut p = desc("test.a");
    p.provides.insert(cap("test.objects"), version());
    h.install(
        Box::new(Fixture {
            d: p,
            mode: Mode::DeleteOnlyScene,
        }),
        BTreeMap::new(),
    )
    .unwrap();
    let mut reader = typed_desc("test.z", BTreeMap::from([("refs".into(), nested.clone())]));
    reader.requires.insert(cap("test.objects"), version());
    h.install(
        Box::new(Fixture {
            d: reader,
            mode: Mode::ReferenceOnTick(sr("test.a", "main", 1)),
        }),
        BTreeMap::from([(cap("test.objects"), id("test.a"))]),
    )
    .unwrap();
    let before = h.save().unwrap();
    err(h.step(input(1)), ErrorCode::InvalidReference);
    assert_eq!(h.save().unwrap(), before);
    h.scene_handle(&sr("test.a", "main", 1)).unwrap();
    let mut h = host();
    let d = typed_desc("test.a", BTreeMap::from([("refs".into(), nested)]));
    h.install(
        Box::new(Fixture {
            d,
            mode: Mode::NestedEventRef,
        }),
        BTreeMap::new(),
    )
    .unwrap();
    let before = h.save().unwrap();
    err(h.step(input(1)), ErrorCode::ReferenceInUse);
    assert_eq!(h.save().unwrap(), before);
}

#[test]
fn core_limits_and_wire_tag_malformed_records_are_explicit() {
    let h = host();
    let l = h.describe().limits;
    assert_eq!(
        (
            l.max_plugins,
            l.max_provides,
            l.max_requires,
            l.max_schemas,
            l.max_fields,
            l.max_event_kinds,
            l.max_actions
        ),
        (16, 32, 32, 32, 32, 32, 32)
    );
    assert_eq!(
        (
            l.max_records,
            l.max_scenes,
            l.max_entities,
            l.max_string_bytes,
            l.max_bytes,
            l.max_list_items,
            l.max_depth
        ),
        (256, 64, 256, 1024, 4096, 64, 4)
    );
    assert_eq!(
        (
            l.max_record_bytes,
            l.max_frame_actions,
            l.max_id_bytes,
            l.max_content_binding_bytes,
            l.max_error_bytes
        ),
        (65_536, 128, 128, 128, 4096)
    );
    assert_eq!(
        (
            l.max_events_per_commit,
            l.max_pending_events,
            l.max_history_events,
            l.max_commands,
            l.max_save_bytes
        ),
        (128, 128, 256, 4096, 8_388_608)
    );
    let d = typed_desc("test.a", BTreeMap::from([("flag".into(), FieldType::Bool)]));
    let r = typed_record(&d, BTreeMap::from([("flag".into(), Value::Bool(true))]));
    let mut h = host();
    h.install(
        Box::new(Fixture {
            d,
            mode: Mode::Static {
                value: r,
                swallow: false,
                emit: false,
            },
        }),
        BTreeMap::new(),
    )
    .unwrap();
    let bytes = h.save().unwrap();
    for invalid in [
        replace(
            &bytes,
            "\"flag\":{\"type\":\"bool\",\"value\":true}",
            "\"flag\":{\"type\":\"bool\",\"value\":1}",
        ),
        replace(
            &bytes,
            "\"flag\":{\"type\":\"bool\",\"value\":true}",
            "\"flag\":{\"type\":\"bool\",\"value\":true,\"extra\":1}",
        ),
        replace(
            &bytes,
            "\"flag\":{\"type\":\"bool\"}",
            "\"flag\":{\"type\":\"bool\",\"value\":null}",
        ),
    ] {
        reject_restore(&mut h, &invalid, ErrorCode::InvalidSave);
    }
}

#[test]
fn aggregate_tick_growth_over_8mib_rolls_back_rng_records_input_and_owner_token() {
    let mut h = host();
    let d = typed_desc(
        "test.a",
        BTreeMap::from([(
            "items".into(),
            FieldType::List {
                item: Box::new(FieldType::String { max_bytes: 1000 }),
                max_items: 48,
            },
        )]),
    );
    let r = typed_record(
        &d,
        BTreeMap::from([(
            "items".into(),
            Value::List(vec![Value::String("x".repeat(1000)); 48]),
        )]),
    );
    h.install(
        Box::new(Fixture {
            d,
            mode: Mode::GrowRecords(r),
        }),
        BTreeMap::new(),
    )
    .unwrap();
    let token = h.owner(&id("test.a")).unwrap();
    let before = h.save().unwrap();
    err(h.step(input(1)), ErrorCode::BudgetExceeded);
    assert_eq!(h.save().unwrap(), before);
    assert_eq!(h.owner(&id("test.a")).unwrap(), token);
    assert_eq!(h.describe().last_input, None);
}

#[test]
fn aggregate_input_frame128_boundary_and_identity_overflow_are_atomic() {
    let mut h = host();
    let mut actions = vec![];
    for owner in 0..5 {
        let name = format!("test.p{owner}");
        let mut d = desc(&name);
        d.actions = (0..32)
            .map(|i| (aid(&format!("{name}.a{i}")), ActionType::Bool))
            .collect();
        for action in d.actions.keys() {
            actions.push((action.clone(), ActionValue::Bool(true)));
        }
        h.install(
            Box::new(Fixture {
                d,
                mode: Mode::Quiet,
            }),
            BTreeMap::new(),
        )
        .unwrap();
    }
    let before = h.save().unwrap();
    err(
        h.step(InputFrame {
            target_tick: 1,
            actions: actions[..129].to_vec(),
        }),
        ErrorCode::InvalidInput,
    );
    assert_eq!(h.save().unwrap(), before);
    h.step(InputFrame {
        target_tick: 1,
        actions: actions[..128].to_vec(),
    })
    .unwrap();
    assert_eq!(h.describe().last_input.unwrap().actions.len(), 128);
    let mut h = host();
    let bytes = h.save().unwrap();
    let max = replace(
        &bytes,
        "\"next_identity\":1",
        "\"next_identity\":18446744073709551615",
    );
    h.restore(&max).unwrap();
    let before = h.save().unwrap();
    err(
        h.install(
            Box::new(Fixture {
                d: objects_desc("test.a"),
                mode: Mode::Objects {
                    recreate: false,
                    keep_event: false,
                },
            }),
            BTreeMap::new(),
        ),
        ErrorCode::Overflow,
    );
    assert_eq!(h.save().unwrap(), before);
}

#[test]
fn restore_rejects_global_event_sequence_tick_regression_across_owners() {
    let mut h = host();
    install(&mut h, "test.a", Mode::EmitAtTick(1));
    install(&mut h, "test.b", Mode::EmitAtTick(2));
    h.step(input(1)).unwrap();
    h.step(input(2)).unwrap();
    let bytes = h.save().unwrap();
    // Both owners retain one event. Swap global sequences in history AND
    // pending, preserving owner ordinal suffixes, pending equality, unique
    // sequences and lifetime accounting. Global seq0 now occurs at tick2,
    // followed by seq1 at tick1, which no successful host commit can produce.
    let text = std::str::from_utf8(&bytes).unwrap();
    assert!(text.contains("\"sequence\":0") && text.contains("\"sequence\":1"));
    let corrupted = text
        .replace("\"sequence\":0", "\"sequence\":999999")
        .replace("\"sequence\":1", "\"sequence\":0")
        .replace("\"sequence\":999999", "\"sequence\":1");
    reject_restore(&mut h, corrupted.as_bytes(), ErrorCode::InvalidSave);
}

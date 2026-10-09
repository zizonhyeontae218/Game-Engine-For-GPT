//! These tests import immutable legacy projections into CoreHost. Core ticks do
//! not run legacy gameplay; LegacyBridge's World remains authoritative.
use ge4g_core::{StateDefinition, StateType};
use ge4g_pentomino::p2::{
    ActionId, ActionValue, CoreHost, InputFrame, ObjectRef, RecordKey, SchemaId, Selection, Value,
};
use ge4g_pentomino::{PluginId, Version};
use ge4g_pentomino_legacy::{BridgeError, BridgeErrorCode, LegacyBridge};
use ge4g_project::{InputSpan, Project, Replay, ReplayCommand, flatland::Action};
use ge4g_runtime::{World, snapshot_hash};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::Path;
fn id() -> PluginId {
    PluginId::new("legacy.bridge").unwrap()
}
fn action(s: &str) -> ActionId {
    ActionId::new(s).unwrap()
}
fn hash_key(prefix: &str, source: &str) -> String {
    format!("{prefix}{:x}", Sha256::digest(source.as_bytes()))
}
fn world(name: &str) -> World {
    World::new(
        Project::load(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples")
                .join(name),
        )
        .unwrap(),
    )
    .unwrap()
}
fn selection() -> Selection {
    Selection {
        owners: BTreeSet::from([id()]),
        schemas: BTreeSet::new(),
        include_objects: true,
        include_history: true,
    }
}
fn error<T>(result: Result<T, BridgeError>, code: BridgeErrorCode) {
    match result {
        Err(e) => assert_eq!(e.code(), code, "{}", e.detail()),
        Ok(_) => panic!("expected {code:?}"),
    }
}
fn add_state(w: &mut World) {
    for (key, definition) in [
        (
            "test.bool",
            StateDefinition {
                kind: StateType::Bool,
                default: true.into(),
                persistent: true,
            },
        ),
        (
            "test.int",
            StateDefinition {
                kind: StateType::Integer,
                default: (-4_i64).into(),
                persistent: true,
            },
        ),
        (
            "test.text",
            StateDefinition {
                kind: StateType::String,
                default: "항구".to_owned().into(),
                persistent: true,
            },
        ),
    ] {
        let default = definition.default.clone();
        w.state.definitions.insert(key.into(), definition);
        w.state.values.insert(key.into(), default);
    }
}
fn replay(actions: Vec<String>) -> Replay {
    Replay {
        schema_version: 2,
        ticks: 10,
        inputs: vec![InputSpan {
            start: 0,
            end: 2,
            actions,
        }],
        commands: vec![],
    }
}

#[test]
fn four_actual_games_install_typed_scene_entity_state_input_and_roundtrip_save2() {
    for name in [
        "flatland_pacman",
        "basement_demo",
        "flatland_harbor",
        "flatland_signal_yard",
    ] {
        let mut w = world(name);
        add_state(&mut w);
        let legacy_scene = w.scene.clone();
        let legacy_entity_ids: BTreeSet<_> = w.entities.keys().cloned().collect();
        let legacy_state = w.state.values.clone();
        let bridge = LegacyBridge::new(w).unwrap();
        let legacy_before = snapshot_hash(&bridge.legacy_snapshot()).unwrap();
        let projection = bridge.projection().unwrap();
        assert_eq!(
            projection.descriptor.contract,
            Version {
                major: 2,
                minor: 0,
                patch: 0
            }
        );
        assert_eq!(
            projection.descriptor.release,
            Version {
                major: 0,
                minor: 3,
                patch: 0
            }
        );
        assert!(
            projection.descriptor.provides.is_empty()
                && projection.descriptor.requires.is_empty()
                && projection.descriptor.event_kinds.is_empty()
        );
        let frame = bridge
            .input_frame(&replay(vec!["right".into(), "fire".into()]))
            .unwrap();
        assert_eq!(frame.target_tick, bridge.world().tick + 1);
        assert!(frame.actions.windows(2).all(|a| a[0].0 < a[1].0));
        assert!(
            frame
                .actions
                .contains(&(action("legacy.right"), ActionValue::Bool(true)))
        );
        assert!(frame.actions.contains(&(
            action(&hash_key("legacy.button_", "fire")),
            ActionValue::Bool(true)
        )));
        let mut host = CoreHost::new(7, &format!("legacy:{name}")).unwrap();
        projection.install_into(&mut host, &frame).unwrap();
        let selected = host.select(&selection()).unwrap();
        assert_eq!(selected.tick, 0);
        assert_eq!(selected.scenes.len(), 1);
        assert_eq!(selected.entities.len(), legacy_entity_ids.len());
        let scene = &selected.scenes[0].reference;
        assert_eq!(scene.local, hash_key("s_", &legacy_scene));
        let scene_record = &selected.records[&RecordKey {
            owner: id(),
            local: scene.local.clone(),
        }];
        assert_eq!(scene_record.schema, SchemaId::new("legacy.scene").unwrap());
        assert_eq!(
            scene_record.fields["id"],
            Value::String(legacy_scene.clone())
        );
        let projected_ids: BTreeSet<_> = selected
            .records
            .values()
            .filter(|r| r.schema == SchemaId::new("legacy.entity").unwrap())
            .map(|r| {
                assert_eq!(r.fields.len(), 2);
                assert_eq!(
                    r.fields["scene"],
                    Value::Ref(ObjectRef::Scene(scene.clone()))
                );
                let Value::String(source) = &r.fields["id"] else {
                    panic!("entity id type")
                };
                source.clone()
            })
            .collect();
        assert_eq!(projected_ids, legacy_entity_ids);
        for entity in &selected.entities {
            assert_eq!(&entity.scene, scene);
            assert_ne!(entity.reference.incarnation, scene.incarnation);
        }
        for (key, source) in &legacy_state {
            let r = &selected.records[&RecordKey {
                owner: id(),
                local: hash_key("v_", key),
            }];
            assert_eq!(r.fields["key"], Value::String(key.clone()));
            if let Some(value) = source.as_bool() {
                assert_eq!(r.schema, SchemaId::new("legacy.state_bool").unwrap());
                assert_eq!(r.fields["value"], Value::Bool(value));
            } else if let Some(value) = source.as_i64() {
                assert_eq!(r.schema, SchemaId::new("legacy.state_i64").unwrap());
                assert_eq!(r.fields["value"], Value::I64(value));
            } else {
                assert_eq!(r.schema, SchemaId::new("legacy.state_string").unwrap());
                assert_eq!(
                    r.fields["value"],
                    Value::String(source.as_str().unwrap().into())
                );
            }
        }
        if name == "flatland_harbor" {
            assert_eq!(legacy_scene, "항구");
            assert_eq!(scene_record.fields["id"], Value::String("항구".into()));
        }
        host.step(InputFrame {
            target_tick: 1,
            actions: frame.actions,
        })
        .unwrap();
        assert_eq!(
            host.select(&selection()).unwrap().records,
            selected.records,
            "immutable snapshot plugin tick has no legacy authority"
        );
        assert_eq!(
            snapshot_hash(&bridge.legacy_snapshot()).unwrap(),
            legacy_before
        );
        let bytes = host.save().unwrap();
        let state = host.select(&selection()).unwrap();
        let digest = host.hash().unwrap();
        host.restore(&bytes).unwrap();
        assert_eq!(host.select(&selection()).unwrap(), state);
        assert_eq!(host.hash().unwrap(), digest);
        let before = host.save().unwrap();
        error(
            projection.install_into(
                &mut host,
                &InputFrame {
                    target_tick: 2,
                    actions: vec![],
                },
            ),
            BridgeErrorCode::CoreValidation,
        );
        assert_eq!(host.save().unwrap(), before);
    }
}

#[test]
fn bridge_replay_candidates_match_direct_world_advance_for_all_four_games() {
    for name in [
        "flatland_pacman",
        "basement_demo",
        "flatland_harbor",
        "flatland_signal_yard",
    ] {
        let initial = world(name);
        let mut direct = initial.clone();
        let mut bridge = LegacyBridge::new(initial).unwrap();
        let trace = Replay::load(
            &Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../examples")
                .join(name)
                .join("replays/journey.json"),
        )
        .unwrap();
        for _ in 0..3 {
            direct.step_replay(&trace).unwrap();
            let projection = bridge.step_replay(&trace).unwrap();
            assert_eq!(projection.tick, direct.tick);
            assert_eq!(
                snapshot_hash(&bridge.legacy_snapshot()).unwrap(),
                snapshot_hash(&direct.snapshot()).unwrap()
            );
        }
    }
}

#[test]
fn post_tick_projection_limit_and_legacy_error_both_roll_back_authoritative_world() {
    let mut w = world("flatland_harbor");
    w.state.definitions.insert(
        "test.text".into(),
        StateDefinition {
            kind: StateType::String,
            default: "short".to_string().into(),
            persistent: true,
        },
    );
    w.state
        .values
        .insert("test.text".into(), "short".to_string().into());
    w.project
        .manifest
        .state
        .insert("test.text".into(), w.state.definitions["test.text"].clone());
    let mut bridge = LegacyBridge::new(w).unwrap();
    let before = snapshot_hash(&bridge.legacy_snapshot()).unwrap();
    let prior = bridge.projection().unwrap();
    let mut oversize = replay(vec![]);
    oversize.commands.push(ReplayCommand::Do {
        tick: 0,
        actions: vec![Action::Set {
            key: "test.text".into(),
            value: "x".repeat(1025).into(),
        }],
    });
    error(
        bridge.step_replay(&oversize),
        BridgeErrorCode::ProjectionLimit,
    );
    assert_eq!(snapshot_hash(&bridge.legacy_snapshot()).unwrap(), before);
    assert_eq!(bridge.projection().unwrap(), prior);
    assert_eq!(bridge.world().tick, 0);
    let mut invalid = replay(vec![]);
    invalid.commands.push(ReplayCommand::Do {
        tick: 0,
        actions: vec![Action::Set {
            key: "absent.key".into(),
            value: true.into(),
        }],
    });
    error(bridge.step_replay(&invalid), BridgeErrorCode::LegacyRuntime);
    assert_eq!(snapshot_hash(&bridge.legacy_snapshot()).unwrap(), before);
    assert_eq!(bridge.projection().unwrap(), prior);
    bridge.step_replay(&replay(vec![])).unwrap();
    assert_eq!(bridge.world().tick, 1);
}

#[test]
fn malformed_replay_buttons_action_bounds_and_forged_projection_fail_explicitly() {
    let bridge = LegacyBridge::new(world("flatland_harbor")).unwrap();
    let projection = bridge.projection().unwrap();
    let valid = InputFrame {
        target_tick: 1,
        actions: vec![],
    };
    let mut invalid = replay(vec!["illegal label!".into()]);
    error(bridge.input_frame(&invalid), BridgeErrorCode::InvalidInput);
    invalid = replay((0..26).map(|i| format!("button{i}")).collect());
    error(
        bridge.input_frame(&invalid),
        BridgeErrorCode::ProjectionLimit,
    );
    for frame in [
        InputFrame {
            target_tick: 0,
            actions: vec![],
        },
        InputFrame {
            target_tick: 1,
            actions: vec![(action("legacy.right"), ActionValue::I64(1))],
        },
        InputFrame {
            target_tick: 1,
            actions: vec![
                (action("legacy.right"), ActionValue::Bool(true)),
                (action("legacy.right"), ActionValue::Bool(false)),
            ],
        },
        InputFrame {
            target_tick: 1,
            actions: vec![(action("legacy.button_notahash"), ActionValue::Bool(true))],
        },
    ] {
        error(projection.plugin(&frame), BridgeErrorCode::InvalidInput);
    }
    let mut bad = projection.clone();
    bad.owner = PluginId::new("forged.owner").unwrap();
    error(bad.plugin(&valid), BridgeErrorCode::CoreValidation);
    let mut bad = projection.clone();
    bad.records.remove(&RecordKey {
        owner: id(),
        local: "snapshot".into(),
    });
    error(bad.plugin(&valid), BridgeErrorCode::CoreValidation);
    let mut w = world("flatland_harbor");
    w.tick = 1_000_001;
    error(LegacyBridge::new(w), BridgeErrorCode::ProjectionLimit);
    let e = BridgeError::new(
        BridgeErrorCode::InvalidInput,
        format!("{}한", "x".repeat(4095)),
    );
    assert_eq!(e.detail().len(), 4095);
}

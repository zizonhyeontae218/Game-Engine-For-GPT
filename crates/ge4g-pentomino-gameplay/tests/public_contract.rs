//! Independent public P2 integration; no Tiny Core implementation imports.
use ge4g_pentomino::Bindings;
use ge4g_pentomino::p2::{ActionValue, CoreHost, ErrorCode, InputFrame, Value};
use ge4g_pentomino_gameplay::*;

fn config() -> GameConfig {
    serde_json::from_str(
        r#"{
        "version":1,"width":160,"height":96,"tick_hz":60,"attack_control":"attack",
        "fighters":[
            {"id":"hero","hp":30,"damage":9,"position":[40,72],"color":[40,180,240,255]},
            {"id":"rival","hp":25,"damage":7,"position":[120,72],"color":[230,60,90,255]}
        ]
    }"#,
    )
    .unwrap()
}
fn host(seed: u64) -> CoreHost {
    let mut host = CoreHost::new(seed, "p4.gameplay.contract").unwrap();
    host.install(
        Box::new(StateProvider::new(config()).unwrap()),
        Bindings::new(),
    )
    .unwrap();
    host.install(Box::new(TurnCombat::new(config()).unwrap()), bindings())
        .unwrap();
    host
}
fn attack_step(host: &mut CoreHost, pressed: bool) {
    let tick = host.describe().tick + 1;
    host.step(InputFrame {
        target_tick: tick,
        actions: vec![(attack_action(), ActionValue::Bool(pressed))],
    })
    .unwrap();
    let read = host.select(&selection(true)).unwrap();
    let actions = decision_actions(&read, tick, pressed, &config()).unwrap();
    host.step(InputFrame {
        target_tick: tick + 1,
        actions,
    })
    .unwrap();
}
fn state(host: &CoreHost) -> BattleState {
    battle_state(&host.select(&selection(false)).unwrap(), &config()).unwrap()
}

#[test]
fn two_phase_actual_records_actions_events_and_battle_end() {
    let mut host = host(7);
    let initial = state(&host);
    let read = host.select(&selection(true)).unwrap();
    assert_eq!(read.scenes.len(), 1);
    assert_eq!(read.entities.len(), 2);
    assert_eq!(read.records.len(), 3);
    host.step(InputFrame {
        target_tick: 1,
        actions: vec![(attack_action(), ActionValue::Bool(true))],
    })
    .unwrap();
    assert_eq!(state(&host), initial, "rules never mutate foreign state");
    let read = host.select(&selection(true)).unwrap();
    let pending = &read.histories[&rules_owner()].pending;
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].kind, "decision");
    assert_eq!(pending[0].record.fields["hp1"], Value::I64(16));
    let actions = decision_actions(&read, 1, true, &config()).unwrap();
    assert_eq!(actions.len(), 5);
    host.step(InputFrame {
        target_tick: 2,
        actions,
    })
    .unwrap();
    assert_eq!(
        state(&host),
        BattleState {
            hp: [30, 16],
            turn: 1,
            winner: -1,
            attack_count: 1
        }
    );
    assert_eq!(
        host.select(&selection(true)).unwrap().histories[&state_owner()].pending[0].kind,
        "applied"
    );
    for _ in 0..4 {
        attack_step(&mut host, true);
    }
    assert_eq!(
        state(&host),
        BattleState {
            hp: [16, 0],
            turn: 0,
            winner: 0,
            attack_count: 5
        }
    );
    let ended = state(&host);
    attack_step(&mut host, true);
    assert_eq!(state(&host), ended);
    assert!(
        host.select(&selection(true)).unwrap().histories[&rules_owner()]
            .pending
            .is_empty()
    );
}

#[test]
fn deterministic_replay_save_resume_and_rules_unload_keeps_authority() {
    let mut a = host(42);
    let mut b = host(42);
    for pressed in [true, false, true] {
        attack_step(&mut a, pressed);
        attack_step(&mut b, pressed);
    }
    assert_eq!(a.save().unwrap(), b.save().unwrap());
    let midpoint = a.save().unwrap();
    let mut resumed = host(42);
    resumed.restore(&midpoint).unwrap();
    for pressed in [false, true, true, true] {
        attack_step(&mut a, pressed);
        attack_step(&mut resumed, pressed);
    }
    assert_eq!(a.save().unwrap(), resumed.save().unwrap());
    let state_before = b.select(&selection(false)).unwrap();
    let handles: Vec<_> = state_before
        .entities
        .iter()
        .map(|e| b.entity_handle(&e.reference).unwrap())
        .collect();
    let token = b.owner(&rules_owner()).unwrap();
    b.remove(token).unwrap();
    assert_eq!(b.select(&selection(false)).unwrap(), state_before);
    for handle in handles {
        assert!(b.resolve_entity(handle).is_ok());
    }
    let tick = b.describe().tick + 1;
    b.step(InputFrame {
        target_tick: tick,
        actions: vec![],
    })
    .unwrap();
    let survivor = state(&b);
    b.install(Box::new(TurnCombat::new(config()).unwrap()), bindings())
        .unwrap();
    assert_eq!(state(&b), survivor);
    attack_step(&mut b, true);
    assert_eq!(state(&b).attack_count, survivor.attack_count + 1);
}

#[test]
fn pending_decision_save_restore_replays_actual_event_without_hidden_plugin_state() {
    let mut original = host(11);
    attack_step(&mut original, true);
    original
        .step(InputFrame {
            target_tick: 3,
            actions: vec![(attack_action(), ActionValue::Bool(true))],
        })
        .unwrap();
    let save = original.save().unwrap();
    let mut restored = host(11);
    restored.restore(&save).unwrap();
    let a = original.select(&selection(true)).unwrap();
    let b = restored.select(&selection(true)).unwrap();
    assert_eq!(a, b);
    let actions_a = decision_actions(&a, 3, true, &config()).unwrap();
    let actions_b = decision_actions(&b, 3, true, &config()).unwrap();
    assert_eq!(actions_a, actions_b);
    original
        .step(InputFrame {
            target_tick: 4,
            actions: actions_a,
        })
        .unwrap();
    restored
        .step(InputFrame {
            target_tick: 4,
            actions: actions_b,
        })
        .unwrap();
    assert_eq!(original.save().unwrap(), restored.save().unwrap());
    assert_eq!(state(&restored).hp, [23, 16]);
}

#[test]
fn invalid_scalar_batch_and_attack_are_atomic_and_decision_validation_is_strict() {
    let mut host = host(1);
    let save = host.save().unwrap();
    let bad = InputFrame {
        target_tick: 1,
        actions: vec![(
            ge4g_pentomino::p2::ActionId::new("p4.state.hp0").unwrap(),
            ActionValue::I64(0),
        )],
    };
    assert_eq!(host.step(bad).unwrap_err().code(), ErrorCode::InvalidInput);
    assert_eq!(host.save().unwrap(), save);
    assert_eq!(
        host.step(InputFrame {
            target_tick: 1,
            actions: vec![(attack_action(), ActionValue::I64(1))]
        })
        .unwrap_err()
        .code(),
        ErrorCode::InvalidInput
    );
    assert_eq!(host.save().unwrap(), save);
    host.step(InputFrame {
        target_tick: 1,
        actions: vec![(attack_action(), ActionValue::Bool(true))],
    })
    .unwrap();
    let read = host.select(&selection(true)).unwrap();
    assert!(decision_actions(&read, 2, true, &config()).is_err());
    assert!(decision_actions(&read, 1, false, &config()).is_err());
    let mut missing = read.clone();
    missing
        .histories
        .get_mut(&rules_owner())
        .unwrap()
        .pending
        .clear();
    assert!(decision_actions(&missing, 1, true, &config()).is_err());
    let mut duplicate = read.clone();
    let event = duplicate.histories[&rules_owner()].pending[0].clone();
    duplicate
        .histories
        .get_mut(&rules_owner())
        .unwrap()
        .pending
        .push(event);
    assert!(decision_actions(&duplicate, 1, true, &config()).is_err());
    let mut forged = read.clone();
    forged.histories.get_mut(&rules_owner()).unwrap().pending[0]
        .record
        .fields
        .insert("hp1".into(), Value::I64(1));
    assert!(decision_actions(&forged, 1, true, &config()).is_err());
    let mut forged = read.clone();
    forged.histories.get_mut(&rules_owner()).unwrap().pending[0].owner = state_owner();
    assert!(decision_actions(&forged, 1, true, &config()).is_err());
    assert_eq!(
        decision_actions(&read, 1, true, &config()).unwrap().len(),
        5
    );
}

#[test]
fn configuration_and_restored_semantics_reject_unknown_fields_and_invalid_objects() {
    let mut bad = config();
    bad.fighters[1].id = bad.fighters[0].id.clone();
    assert!(bad.validate().is_err());
    let mut bad = config();
    bad.fighters[0].position[1] = 20;
    assert!(bad.validate().is_err());
    let mut json = serde_json::to_value(config()).unwrap();
    json["combat_rules"] = serde_json::json!("private");
    assert!(serde_json::from_value::<GameConfig>(json).is_err());
    let host = host(9);
    let mut read = host.select(&selection(false)).unwrap();
    let key = ge4g_pentomino::p2::RecordKey {
        owner: state_owner(),
        local: "status".into(),
    };
    read.records
        .get_mut(&key)
        .unwrap()
        .fields
        .insert("winner".into(), Value::I64(0));
    assert!(battle_state(&read, &config()).is_err());
    let mut read = host.select(&selection(false)).unwrap();
    read.records
        .get_mut(&key)
        .unwrap()
        .fields
        .insert("hp0".into(), Value::I64(29));
    assert!(
        battle_state(&read, &config()).is_err(),
        "schema-valid HP must still be reachable"
    );
    let mut read = host.select(&selection(false)).unwrap();
    read.records
        .get_mut(&key)
        .unwrap()
        .fields
        .insert("attack_count".into(), Value::I64(i64::MAX));
    assert!(
        battle_state(&read, &config()).is_err(),
        "unreachable extreme count rejects without overflow"
    );
    let mut read = host.select(&selection(false)).unwrap();
    read.entities.clear();
    assert!(battle_state(&read, &config()).is_err());
    let mut read = host.select(&selection(false)).unwrap();
    let key = ge4g_pentomino::p2::RecordKey {
        owner: state_owner(),
        local: "position_0".into(),
    };
    read.records
        .get_mut(&key)
        .unwrap()
        .fields
        .insert("x".into(), Value::I64(1));
    assert!(battle_state(&read, &config()).is_err());
}

//! Public native boundary tests: no private Session, renderer or plugin access.
use ge4g_pentomino_native::{
    CameraRequest, ErrorCode, ExecutionMode, InputRequest, NativeBridge, SaveBundle, SessionHandle,
    ViewFamily,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const GAME: &[u8] = include_bytes!("../../../samples/pentomino_p4/game.json");

fn input(step: u64, attack: bool) -> InputRequest {
    InputRequest {
        version: 1,
        target_step: step,
        controls: if attack {
            vec!["attack".into()]
        } else {
            vec![]
        },
    }
}
fn linked() -> (NativeBridge, SessionHandle) {
    let mut bridge = NativeBridge::new();
    let handle = bridge.create(GAME, 73, ExecutionMode::Linked).unwrap();
    (bridge, handle)
}
fn encode(save: &SaveBundle) -> Vec<u8> {
    serde_json::to_vec(save).unwrap()
}
fn assert_unchanged(
    bridge: &NativeBridge,
    handle: SessionHandle,
    save: &SaveBundle,
    frame: &ge4g_pentomino_native::FrameDto,
) {
    assert_eq!(&bridge.save(handle).unwrap(), save);
    assert_eq!(&bridge.frame(handle).unwrap(), frame);
}

#[test]
fn same_seed_trace_and_headless_have_identical_core_authority() {
    let (mut a, ah) = linked();
    let (mut b, bh) = linked();
    let mut headless = NativeBridge::new();
    let hh = headless.create(GAME, 73, ExecutionMode::Headless).unwrap();
    assert_eq!(headless.frame(hh).unwrap_err().code, ErrorCode::Unsupported);
    for (n, attack) in [true, false, true, true, false, true, true]
        .into_iter()
        .enumerate()
    {
        let step = n as u64 + 1;
        let ao = a.step(ah, input(step, attack)).unwrap();
        let bo = b.step(bh, input(step, attack)).unwrap();
        let ho = headless.step(hh, input(step, attack)).unwrap();
        assert_eq!(ao, bo);
        assert_eq!(ao, ho);
        assert_eq!(ao.core_tick, step * 2);
        assert_eq!(ao.logical_step, step);
        assert_eq!(ao.presentation_tick, step);
        assert_eq!(a.save(ah).unwrap(), b.save(bh).unwrap());
        assert_eq!(
            a.save(ah).unwrap().core_save2,
            headless.save(hh).unwrap().core_save2
        );
        assert_eq!(a.frame(ah).unwrap(), b.frame(bh).unwrap());
    }
    let ended = a.observe(ah).unwrap();
    assert_eq!(ended.battle.hp, [6, 0]);
    assert_eq!(ended.battle.winner, 0);
    assert_eq!(ended.battle.attack_count, 5);
    let old = a.save(ah).unwrap();
    let old_frame = a.frame(ah).unwrap();
    assert!(a.step(ah, input(8, true)).is_err());
    assert_unchanged(&a, ah, &old, &old_frame);
    a.step(ah, input(8, false)).unwrap();
}

#[test]
fn real_frame_observation_rows_and_decision_events_follow_actual_attack() {
    let (mut bridge, h) = linked();
    let before = bridge.frame(h).unwrap();
    assert_eq!((before.width, before.height), (160, 96));
    assert_eq!(before.rgba.len(), 160 * 96 * 4);
    assert_eq!(before.sha256, format!("{:x}", Sha256::digest(&before.rgba)));
    let before_observation = bridge.observe(h).unwrap();
    let observation = bridge.step(h, input(1, true)).unwrap();
    assert_eq!(observation.battle.hp, [12, 7]);
    assert_eq!(observation.battle.turn, 1);
    assert_eq!(observation.battle.attack_count, 1);
    // Static positions and camera prove that changed pixels came from changed HP.
    assert_eq!(before_observation.camera_frames, observation.camera_frames);
    assert_eq!(before_observation.entities, observation.entities);
    let after = bridge.frame(h).unwrap();
    assert_eq!((after.core_tick, after.presentation_tick), (2, 1));
    assert_ne!(before.rgba, after.rgba);
    assert_ne!(before.sha256, after.sha256);
    let colored_pixels = |pixels: &[u8], color: [u8; 4]| {
        pixels
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|pixel| **pixel == color)
            .count()
    };
    // Fighter geometry and camera stay fixed; the engine's HP fill loses
    // pixels while the depleted HP bar gains the corresponding pixels.
    assert!(
        colored_pixels(&before.rgba, [210, 255, 56, 255])
            > colored_pixels(&after.rgba, [210, 255, 56, 255])
    );
    assert!(
        colored_pixels(&before.rgba, [80, 40, 40, 255])
            < colored_pixels(&after.rgba, [80, 40, 40, 255])
    );
    assert_eq!(
        colored_pixels(&before.rgba, [48, 144, 255, 255]),
        colored_pixels(&after.rgba, [48, 144, 255, 255])
    );
    let wire = serde_json::to_value(&observation).unwrap();
    let rows = wire["records"].as_array().unwrap();
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|row| row["owner"] == "p4.state"));
    assert!(
        rows.windows(2)
            .all(|pair| pair[0]["local"].as_str() < pair[1]["local"].as_str())
    );
    assert_eq!(observation.camera_frames.len(), 1);
    assert_eq!(observation.camera_frames[0].items.len(), 2);
    let rules = observation
        .histories
        .iter()
        .find(|row| row.owner.as_str() == "p4.turn_combat")
        .unwrap();
    assert!(rules.history.pending.is_empty());
    assert_eq!(rules.history.history.len(), 1);
    let decision = &rules.history.history[0];
    assert_eq!(decision.tick, 1);
    assert_eq!(decision.kind, "decision");
    assert_eq!(
        decision.record.fields["hp1"],
        ge4g_pentomino::p2::Value::I64(7)
    );
}

#[test]
fn combined_save_restores_midflight_camera_and_deterministic_continuation() {
    let (mut control, ch) = linked();
    control.step(ch, input(1, true)).unwrap();
    control
        .set_camera(
            ch,
            CameraRequest {
                zoom: 1.7,
                origin: [7., 3.],
                transition_ticks: 5,
                ..CameraRequest::default()
            },
        )
        .unwrap();
    control.step(ch, input(2, true)).unwrap();
    assert!(control.describe(ch).unwrap().view.cameras[0].transitioning);
    let midpoint = control.save(ch).unwrap();
    let (mut restored, rh) = linked();
    restored.restore(rh, &encode(&midpoint)).unwrap();
    assert_eq!(restored.save(rh).unwrap(), midpoint);
    assert_eq!(restored.observe(rh).unwrap(), control.observe(ch).unwrap());
    assert_eq!(restored.frame(rh).unwrap(), control.frame(ch).unwrap());
    for step in 3..=7 {
        let attack = step % 2 == 1;
        assert_eq!(
            control.step(ch, input(step, attack)).unwrap(),
            restored.step(rh, input(step, attack)).unwrap()
        );
        assert_eq!(control.save(ch).unwrap(), restored.save(rh).unwrap());
        assert_eq!(control.frame(ch).unwrap(), restored.frame(rh).unwrap());
    }
    assert!(!restored.describe(rh).unwrap().view.cameras[0].transitioning);
    // Execution mode is not authority: the same bundle is usable headlessly.
    let mut headless = NativeBridge::new();
    let hh = headless.create(GAME, 73, ExecutionMode::Headless).unwrap();
    headless.restore(hh, &encode(&midpoint)).unwrap();
    assert_eq!(headless.save(hh).unwrap(), midpoint);
}

#[test]
fn strict_json_and_invalid_inputs_are_atomic_and_retryable() {
    let (mut bridge, h) = linked();
    bridge.step(h, input(1, true)).unwrap();
    let old = bridge.save(h).unwrap();
    let frame = bridge.frame(h).unwrap();
    let invalid_inputs = [
        InputRequest {
            version: 2,
            ..input(2, true)
        },
        input(1, true),
        InputRequest {
            controls: vec!["unknown".into()],
            ..input(2, false)
        },
        InputRequest {
            controls: vec!["attack".into(), "attack".into()],
            ..input(2, false)
        },
    ];
    for request in invalid_inputs {
        assert!(bridge.step(h, request).is_err());
        assert_unchanged(&bridge, h, &old, &frame);
    }
    for request in [
        json!({"version":2,"command":{"op":"observe","session":h.id()}}),
        json!({"version":1,"extra":true,"command":{"op":"observe","session":h.id()}}),
        json!({"version":1,"command":{"op":"step","session":h.id(),"input":{"version":1,"target_step":2,"controls":[true]}}}),
        json!({"version":1,"command":{"op":"step","session":h.id(),"input":{"version":1,"target_step":2,"controls":["attack"],"extra":1}}}),
    ] {
        let response = bridge.request(&serde_json::to_vec(&request).unwrap());
        assert!(!response.ok);
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_unchanged(&bridge, h, &old, &frame);
    }
    let duplicate_field = format!(
        r#"{{"version":1,"command":{{"op":"step","session":{},"input":{{"version":1,"target_step":2,"controls":[],"controls":["attack"]}}}}}}"#,
        h.id()
    );
    assert!(!bridge.request(duplicate_field.as_bytes()).ok);
    assert_unchanged(&bridge, h, &old, &frame);
    bridge.step(h, input(2, true)).unwrap();
    assert_eq!(bridge.observe(h).unwrap().battle.hp, [9, 7]);
}

#[test]
fn declarative_control_mapping_and_content_binding_are_enforced() {
    let (mut original, oh) = linked();
    original.step(oh, input(1, true)).unwrap();
    let original_save = original.save(oh).unwrap();
    let mut config: Value = serde_json::from_slice(GAME).unwrap();
    config["attack_control"] = json!("strike");
    let mut changed = NativeBridge::new();
    let h = changed
        .create(
            &serde_json::to_vec(&config).unwrap(),
            73,
            ExecutionMode::Linked,
        )
        .unwrap();
    let old = changed.save(h).unwrap();
    let frame = changed.frame(h).unwrap();
    assert_ne!(old.content_binding, original_save.content_binding);
    assert_eq!(
        changed
            .restore(h, &encode(&original_save))
            .unwrap_err()
            .code,
        ErrorCode::SaveMismatch
    );
    assert_eq!(
        changed.step(h, input(1, true)).unwrap_err().code,
        ErrorCode::InvalidInput
    );
    assert_unchanged(&changed, h, &old, &frame);
    let mapped = changed
        .step(
            h,
            InputRequest {
                controls: vec!["strike".into()],
                ..input(1, false)
            },
        )
        .unwrap();
    assert_eq!(mapped.battle, original.observe(oh).unwrap().battle);
    assert_eq!(mapped.core_tick, 2);
    let mut another_seed = NativeBridge::new();
    let sh = another_seed
        .create(GAME, 74, ExecutionMode::Linked)
        .unwrap();
    assert_eq!(
        another_seed
            .restore(sh, &encode(&original_save))
            .unwrap_err()
            .code,
        ErrorCode::SaveMismatch
    );
}

#[test]
fn separate_save_failures_and_forged_clocks_do_not_replace_active_session() {
    let (mut bridge, h) = linked();
    bridge.step(h, input(1, true)).unwrap();
    let past = bridge.save(h).unwrap();
    bridge.step(h, input(2, true)).unwrap();
    let current = bridge.save(h).unwrap();
    let frame = bridge.frame(h).unwrap();
    let mut bad = Vec::new();
    let mut save = past.clone();
    save.core_save2 = "{".into();
    bad.push(save);
    let mut save = past.clone();
    save.core_save2.push(' ');
    bad.push(save);
    // Valid old Core is restored in staging before presentation fails.
    let mut save = past.clone();
    save.presentation_save1 = "{".into();
    bad.push(save);
    let mut save = past.clone();
    save.presentation_save1.push(' ');
    bad.push(save);
    let mut save = past.clone();
    save.seed += 1;
    bad.push(save);
    let mut save = past.clone();
    save.version = 2;
    bad.push(save);
    let mut save = past.clone();
    save.logical_step += 1;
    bad.push(save);
    let mut save = past.clone();
    save.presentation_tick += 1;
    bad.push(save);
    let mut save = past.clone();
    save.presentation_tick = 0;
    bad.push(save);
    let mut save = past.clone();
    save.gameplay_attached = false;
    bad.push(save);
    // Canonical structure but an odd Core tick exposes a half-step boundary.
    // Replacing the scalar preserves the public canonical field order.
    let mut save = past.clone();
    assert!(save.core_save2.contains("\"tick\":2,"));
    save.core_save2 = save.core_save2.replacen("\"tick\":2,", "\"tick\":3,", 1);
    bad.push(save);
    for save in bad {
        assert!(bridge.restore(h, &encode(&save)).is_err());
        assert_unchanged(&bridge, h, &current, &frame);
    }
    for bytes in [b"not-json".as_slice(), b"{}".as_slice()] {
        assert!(bridge.restore(h, bytes).is_err());
        assert_unchanged(&bridge, h, &current, &frame);
    }
    let mut unknown = serde_json::to_value(&past).unwrap();
    unknown["undocumented"] = Value::Bool(true);
    assert!(
        bridge
            .restore(h, &serde_json::to_vec(&unknown).unwrap())
            .is_err()
    );
    assert_unchanged(&bridge, h, &current, &frame);
    bridge.step(h, input(3, true)).unwrap();
    assert_eq!(bridge.observe(h).unwrap().battle.hp, [9, 3]);
}

fn state_save(save: &SaveBundle) -> Value {
    let core: Value = serde_json::from_str(&save.core_save2).unwrap();
    core["plugins"]
        .as_array()
        .unwrap()
        .iter()
        .find(|plugin| plugin["descriptor"]["id"] == "p4.state")
        .unwrap()
        .clone()
}

#[test]
fn canonical_but_impossible_status_and_extra_carrier_records_are_atomic() {
    let (mut bridge, h) = linked();
    let initial = bridge.save(h).unwrap();
    let frame = bridge.frame(h).unwrap();
    let mut impossible = initial.clone();
    let count = "\"attack_count\":{\"type\":\"i64\",\"value\":0}";
    assert!(impossible.core_save2.contains(count));
    impossible.core_save2 =
        impossible
            .core_save2
            .replacen(count, "\"attack_count\":{\"type\":\"i64\",\"value\":1}", 1);
    assert!(bridge.restore(h, &encode(&impossible)).is_err());
    assert_unchanged(&bridge, h, &initial, &frame);

    let observation = bridge.observe(h).unwrap();
    let position = &observation
        .records
        .iter()
        .find(|row| row.local == "position_0")
        .unwrap()
        .record;
    let record = serde_json::to_string(position).unwrap();
    let mut extra_record = initial.clone();
    let prefix = "\"records\":{\"position_0\":";
    assert!(extra_record.core_save2.contains(prefix));
    extra_record.core_save2 = extra_record.core_save2.replacen(
        prefix,
        &format!("\"records\":{{\"extra\":{record},\"position_0\":"),
        1,
    );
    assert!(bridge.restore(h, &encode(&extra_record)).is_err());
    assert_unchanged(&bridge, h, &initial, &frame);
    bridge.step(h, input(1, true)).unwrap();
}

#[test]
fn unsupported_canonical_camera_and_future_endpoint_are_rejected_in_both_modes() {
    use ge4g_pentomino_view::CameraTarget;
    for mode in [ExecutionMode::Headless, ExecutionMode::Linked] {
        let mut bridge = NativeBridge::new();
        let h = bridge.create(GAME, 73, mode).unwrap();
        let initial = bridge.save(h).unwrap();
        let frame = (mode == ExecutionMode::Linked).then(|| bridge.frame(h).unwrap());
        let json: Value = serde_json::from_str(&initial.presentation_save1).unwrap();
        let base: CameraTarget =
            serde_json::from_value(json["cameras"]["p4.camera"]["base"].clone()).unwrap();
        let mut unsupported = base.clone();
        unsupported.view_transform.scale[0] = 2.;
        let mut invalid = initial.clone();
        let old = format!("\"base\":{}", serde_json::to_string(&base).unwrap());
        let new = format!("\"base\":{}", serde_json::to_string(&unsupported).unwrap());
        assert!(invalid.presentation_save1.contains(&old));
        invalid.presentation_save1 = invalid.presentation_save1.replacen(&old, &new, 1);
        assert_eq!(
            bridge.restore(h, &encode(&invalid)).unwrap_err().code,
            ErrorCode::Unsupported
        );
        assert_eq!(bridge.save(h).unwrap(), initial);
        if let Some(frame) = &frame {
            assert_eq!(bridge.frame(h).unwrap(), *frame);
        }

        bridge
            .set_camera(
                h,
                CameraRequest {
                    zoom: 2.,
                    transition_ticks: 5,
                    ..CameraRequest::default()
                },
            )
            .unwrap();
        let active = bridge.save(h).unwrap();
        let frame = (mode == ExecutionMode::Linked).then(|| bridge.frame(h).unwrap());
        let json: Value = serde_json::from_str(&active.presentation_save1).unwrap();
        let camera = &json["cameras"]["p4.camera"];
        let base: CameraTarget = serde_json::from_value(camera["base"].clone()).unwrap();
        let start: CameraTarget =
            serde_json::from_value(camera["transition"]["start"].clone()).unwrap();
        let target: CameraTarget =
            serde_json::from_value(camera["transition"]["target"].clone()).unwrap();
        assert_eq!(camera["transition"]["elapsed"], 1);
        let mut unsupported = target.clone();
        unsupported.view_transform.scale[0] = 2.;
        let mut invalid = active.clone();
        // At elapsed=0 the saved base must equal start. This leaves a valid
        // currently supported camera while the future endpoint is unsupported.
        let old = format!("\"base\":{}", serde_json::to_string(&base).unwrap());
        let new = format!("\"base\":{}", serde_json::to_string(&start).unwrap());
        assert!(invalid.presentation_save1.contains(&old));
        invalid.presentation_save1 = invalid.presentation_save1.replacen(&old, &new, 1);
        let old = format!("\"target\":{}", serde_json::to_string(&target).unwrap());
        let new = format!(
            "\"target\":{}",
            serde_json::to_string(&unsupported).unwrap()
        );
        assert!(invalid.presentation_save1.contains(&old));
        invalid.presentation_save1 = invalid.presentation_save1.replacen(&old, &new, 1);
        invalid.presentation_save1 =
            invalid
                .presentation_save1
                .replacen("\"elapsed\":1", "\"elapsed\":0", 1);
        assert_eq!(
            bridge.restore(h, &encode(&invalid)).unwrap_err().code,
            ErrorCode::Unsupported
        );
        assert_eq!(bridge.save(h).unwrap(), active);
        if let Some(frame) = &frame {
            assert_eq!(bridge.frame(h).unwrap(), *frame);
        }
        bridge.step(h, input(1, true)).unwrap();
        assert_eq!(bridge.observe(h).unwrap().battle.hp, [12, 7]);
    }
}

#[test]
fn canonical_even_tick_cannot_claim_an_unapplied_phase_one_decision() {
    let (mut bridge, h) = linked();
    let initial = bridge.observe(h).unwrap();
    let completed = bridge.step(h, input(1, true)).unwrap();
    let saved = bridge.save(h).unwrap();
    let frame = bridge.frame(h).unwrap();
    let mut forged = saved.clone();
    let initial_status = &initial
        .records
        .iter()
        .find(|row| row.local == "status")
        .unwrap()
        .record;
    let applied_status = &completed
        .records
        .iter()
        .find(|row| row.local == "status")
        .unwrap()
        .record;
    let initial_record = serde_json::to_string(initial_status).unwrap();
    let applied_record = serde_json::to_string(applied_status).unwrap();
    let old = format!("\"status\":{applied_record}");
    let new = format!("\"status\":{initial_record}");
    assert!(forged.core_save2.contains(&old));
    forged.core_save2 = forged.core_save2.replacen(&old, &new, 1);
    let carrier_history = &completed
        .histories
        .iter()
        .find(|row| row.owner.as_str() == "p4.state")
        .unwrap()
        .history;
    let empty_history = ge4g_pentomino::p2::OwnerHistory {
        emitted: 0,
        dropped: 0,
        history: vec![],
        pending: vec![],
    };
    let old = format!(
        "\"history\":{}",
        serde_json::to_string(carrier_history).unwrap()
    );
    let new = format!(
        "\"history\":{}",
        serde_json::to_string(&empty_history).unwrap()
    );
    assert!(forged.core_save2.contains(&old));
    forged.core_save2 = forged.core_save2.replacen(&old, &new, 1);
    let last_input = bridge.describe(h).unwrap().core.last_input.unwrap();
    let mut idle_input = last_input.clone();
    idle_input.actions.clear();
    let old = format!(
        "\"last_input\":{}",
        serde_json::to_string(&last_input).unwrap()
    );
    let new = format!(
        "\"last_input\":{}",
        serde_json::to_string(&idle_input).unwrap()
    );
    assert!(forged.core_save2.contains(&old));
    forged.core_save2 = forged.core_save2.replacen(&old, &new, 1);
    assert!(forged.core_save2.contains("\"next_sequence\":2,"));
    forged.core_save2 =
        forged
            .core_save2
            .replacen("\"next_sequence\":2,", "\"next_sequence\":1,", 1);
    // The actual rules decision at tick1 remains, but its application at tick2
    // has been forged away. Even Core tick alone cannot certify a full step.
    assert_eq!(
        bridge.restore(h, &encode(&forged)).unwrap_err().code,
        ErrorCode::InvalidSave
    );
    assert_unchanged(&bridge, h, &saved, &frame);
    bridge.step(h, input(2, true)).unwrap();
}

#[test]
fn literal_gameplay_removal_preserves_carrier_identity_and_reattaches_in_place() {
    let (mut bridge, h) = linked();
    bridge.step(h, input(1, true)).unwrap();
    let before = bridge.observe(h).unwrap();
    let provider = state_save(&bridge.save(h).unwrap());
    bridge.detach_gameplay(h).unwrap();
    let detached = bridge.observe(h).unwrap();
    assert!(!detached.gameplay_attached);
    assert_eq!(detached.battle, before.battle);
    assert_eq!(detached.records, before.records);
    assert_eq!(detached.entities, before.entities);
    assert_eq!(detached.scenes, before.scenes);
    assert_eq!(state_save(&bridge.save(h).unwrap()), provider);
    assert_eq!(bridge.describe(h).unwrap().core.plugins.len(), 1);
    assert!(
        bridge
            .describe(h)
            .unwrap()
            .core
            .plugins
            .iter()
            .all(|p| p.descriptor.id.as_str() != "p4.turn_combat")
    );
    assert!(
        detached
            .histories
            .iter()
            .all(|row| row.owner.as_str() != "p4.turn_combat")
    );
    let old = bridge.save(h).unwrap();
    let frame = bridge.frame(h).unwrap();
    assert_eq!(
        bridge.step(h, input(2, true)).unwrap_err().code,
        ErrorCode::GameplayDetached
    );
    assert_unchanged(&bridge, h, &old, &frame);
    let idle = bridge.step(h, input(2, false)).unwrap();
    assert_eq!(idle.core_tick, 4);
    assert_eq!(idle.battle, before.battle);
    let mut idle_provider = provider.clone();
    // P2 clears pending at each tick, while retained history and RNG persist.
    idle_provider["history"]["pending"] = json!([]);
    assert_eq!(state_save(&bridge.save(h).unwrap()), idle_provider);
    // Detached state is portable and does not implicitly reinstall the rules.
    let (mut restored, rh) = linked();
    restored
        .restore(rh, &encode(&bridge.save(h).unwrap()))
        .unwrap();
    assert_eq!(restored.describe(rh).unwrap().core.plugins.len(), 1);
    bridge.attach_gameplay(h).unwrap();
    assert_eq!(bridge.describe(h).unwrap().core.plugins.len(), 2);
    assert_eq!(state_save(&bridge.save(h).unwrap()), idle_provider);
    let resumed = bridge.step(h, input(3, true)).unwrap();
    assert_eq!(resumed.battle.hp, [9, 7]);
    assert_eq!(resumed.battle.attack_count, 2);
    assert_eq!(resumed.entities, before.entities);
}

#[test]
fn camera_and_view_changes_leave_entire_core_save_unchanged() {
    let (mut bridge, h) = linked();
    bridge.step(h, input(1, true)).unwrap();
    let core = bridge.save(h).unwrap().core_save2;
    let before = bridge.observe(h).unwrap();
    let frame = bridge.frame(h).unwrap();
    bridge
        .set_camera(
            h,
            CameraRequest {
                zoom: 1.3,
                tilt: 0.75,
                shear: 0.1,
                origin: [5., 0.],
                ..CameraRequest::default()
            },
        )
        .unwrap();
    let changed = bridge.observe(h).unwrap();
    assert_eq!(bridge.save(h).unwrap().core_save2, core);
    assert_eq!(changed.battle, before.battle);
    assert_eq!(changed.records, before.records);
    assert_ne!(changed.camera_frames, before.camera_frames);
    assert_ne!(bridge.frame(h).unwrap().rgba, frame.rgba);
    let camera_frame = bridge.frame(h).unwrap();
    let camera_observation = bridge.observe(h).unwrap();
    bridge.set_view(h, ViewFamily::Side).unwrap();
    assert_eq!(bridge.save(h).unwrap().core_save2, core);
    assert_eq!(bridge.observe(h).unwrap().battle, before.battle);
    assert_eq!(bridge.save(h).unwrap().view_family, ViewFamily::Side);
    assert_eq!(
        bridge.describe(h).unwrap().view.installed_views[0]
            .config
            .policy
            .family,
        ViewFamily::Side
    );
    // P3 families change follow policy. With a static camera their projection
    // remains identical; the actual camera request above changes output pixels.
    assert_eq!(
        bridge.observe(h).unwrap().camera_frames,
        camera_observation.camera_frames
    );
    assert_eq!(bridge.frame(h).unwrap().rgba, camera_frame.rgba);
    let old = bridge.save(h).unwrap();
    let frame = bridge.frame(h).unwrap();
    assert!(
        bridge
            .set_camera(
                h,
                CameraRequest {
                    zoom: f64::NAN,
                    ..CameraRequest::default()
                }
            )
            .is_err()
    );
    assert_unchanged(&bridge, h, &old, &frame);
}

#[test]
fn close_invalidates_all_operations_and_foreign_bridge_handles() {
    let (mut bridge, h) = linked();
    let save = bridge.save(h).unwrap();
    let mut foreign = NativeBridge::new();
    let fh = foreign.create(GAME, 73, ExecutionMode::Linked).unwrap();
    assert_ne!(h, fh);
    assert_eq!(
        foreign.observe(h).unwrap_err().code,
        ErrorCode::InvalidSession
    );
    bridge.close(h).unwrap();
    assert_eq!(
        bridge.describe(h).unwrap_err().code,
        ErrorCode::InvalidSession
    );
    assert_eq!(
        bridge.observe(h).unwrap_err().code,
        ErrorCode::InvalidSession
    );
    assert_eq!(bridge.frame(h).unwrap_err().code, ErrorCode::InvalidSession);
    assert_eq!(bridge.save(h).unwrap_err().code, ErrorCode::InvalidSession);
    assert_eq!(
        bridge.restore(h, &encode(&save)).unwrap_err().code,
        ErrorCode::InvalidSession
    );
    assert_eq!(
        bridge.step(h, input(1, true)).unwrap_err().code,
        ErrorCode::InvalidSession
    );
    assert_eq!(
        bridge
            .set_camera(h, CameraRequest::default())
            .unwrap_err()
            .code,
        ErrorCode::InvalidSession
    );
    assert_eq!(
        bridge.set_view(h, ViewFamily::Side).unwrap_err().code,
        ErrorCode::InvalidSession
    );
    assert_eq!(
        bridge.detach_gameplay(h).unwrap_err().code,
        ErrorCode::InvalidSession
    );
    assert_eq!(
        bridge.attach_gameplay(h).unwrap_err().code,
        ErrorCode::InvalidSession
    );
    assert_eq!(bridge.close(h).unwrap_err().code, ErrorCode::InvalidSession);
    let fresh = bridge.create(GAME, 73, ExecutionMode::Linked).unwrap();
    assert_ne!(fresh, h);
    assert_eq!(bridge.observe(fresh).unwrap().logical_step, 0);
    assert_eq!(foreign.observe(fh).unwrap().logical_step, 0);
}

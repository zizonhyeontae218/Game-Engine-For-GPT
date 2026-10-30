//! Public P5 native contracts; hand-calculated combat and replay/rollback evidence.
use ge4g_pentomino_gameplay::v2::Command;
use ge4g_pentomino_native::{
    CameraRequest, ErrorCode, ExecutionMode, SessionHandle, ViewFamily, p5,
};
use serde_json::json;

fn game() -> Vec<u8> {
    serde_json::to_vec(&json!({"version":2,"width":160,"height":96,"tick_hz":60,
        "fighters":[
            {"id":"blue","team":0,"hp":24,"resource":10,"attack":4,"defense":1,"speed":2,"position":[40,72],"color":[48,144,255,255]},
            {"id":"red","team":1,"hp":24,"resource":10,"attack":4,"defense":1,"speed":1,"position":[120,72],"color":[255,80,72,255]}],
        "actions":[
            {"id":"strike","kind":"damage","target":"enemy","cost":0,"power":2,"effect":null,"duration":0},
            {"id":"heal","kind":"heal","target":"ally","cost":2,"power":4,"effect":null,"duration":0},
            {"id":"rest","kind":"resource","target":"self_only","cost":0,"power":3,"effect":null,"duration":0},
            {"id":"toxin","kind":"effect","target":"enemy","cost":3,"power":2,"effect":"poison","duration":2},
            {"id":"renew","kind":"effect","target":"ally","cost":2,"power":3,"effect":"regen","duration":2}]
    })).unwrap()
}
fn command(id: i64, actor: usize, action: usize, target: usize) -> Command {
    Command {
        id,
        actor,
        action,
        target,
    }
}
fn linked() -> (p5::NativeBridge, SessionHandle) {
    let mut bridge = p5::NativeBridge::new();
    let handle = bridge.create(&game(), 73, ExecutionMode::Linked).unwrap();
    (bridge, handle)
}
fn assert_same(
    bridge: &p5::NativeBridge,
    handle: SessionHandle,
    save: &p5::SaveBundle,
    frame: &ge4g_pentomino_native::FrameDto,
) {
    assert_eq!(&bridge.save(handle).unwrap(), save);
    assert_eq!(&bridge.frame(handle).unwrap(), frame);
}
#[test]
fn public_sample_manual_oracle_cost_heal_poison_regen_victory_and_replay() {
    let (mut bridge, h) = linked();
    let mut twin = p5::NativeBridge::new();
    let th = twin.create(&game(), 73, ExecutionMode::Headless).unwrap();
    let commands = [
        command(1, 0, 3, 1),
        command(2, 1, 0, 0),
        command(3, 0, 1, 0),
        command(4, 1, 0, 0),
        command(5, 0, 4, 0),
        command(6, 1, 0, 0),
        command(7, 0, 2, 0),
    ];
    let mut previous_frame = bridge.frame(h).unwrap();
    for (index, c) in commands.into_iter().enumerate() {
        let actual = bridge.act(h, c.clone()).unwrap();
        assert_eq!(actual, twin.act(th, c).unwrap());
        assert_eq!(actual.core_tick, 2 * (index as u64 + 1));
        assert_eq!(actual.logical_step, index as u64 + 1);
        let f = &actual.battle.fighters;
        match index {
            0 => {
                assert_eq!(f[0].resource, 7);
                assert_eq!((f[1].poison.power, f[1].poison.turns), (2, 2));
            }
            1 => {
                assert_eq!((f[0].hp, f[1].hp, f[1].poison.turns), (19, 22, 1));
            }
            2 => {
                assert_eq!((f[0].hp, f[0].resource), (23, 5));
            }
            3 => {
                assert_eq!((f[0].hp, f[1].hp, f[1].poison.turns), (18, 20, 0));
            }
            4 => {
                assert_eq!((f[0].hp, f[0].resource, f[0].regen.turns), (21, 3, 1));
            }
            5 => {
                assert_eq!(f[0].hp, 16);
            }
            6 => {
                assert_eq!((f[0].hp, f[0].resource, f[0].regen.turns), (19, 6, 0));
            }
            _ => unreachable!(),
        }
        let frame = bridge.frame(h).unwrap();
        if index == 1 {
            assert_ne!(frame.rgba, previous_frame.rgba);
        }
        previous_frame = frame;
    }
    let mut id = 8;
    loop {
        let before = bridge.observe(h).unwrap();
        if before.battle.winner != -1 {
            break;
        }
        let actor = before.battle.turn;
        let c = command(id, actor, 0, 1 - actor);
        assert_eq!(bridge.act(h, c.clone()).unwrap(), twin.act(th, c).unwrap());
        id += 1;
        assert!(id < 40);
    }
    let final_state = bridge.observe(h).unwrap();
    assert_eq!(final_state.battle.winner, 1);
    assert_eq!(final_state.battle.fighters[0].hp, 0);
    assert_eq!(final_state.battle.fighters[1].hp, 5);
    let saved = bridge.save(h).unwrap();
    assert_eq!(saved, twin.save(th).unwrap());
    let frame = bridge.frame(h).unwrap();
    bridge
        .restore(h, &serde_json::to_vec(&saved).unwrap())
        .unwrap();
    assert_same(&bridge, h, &saved, &frame);
    assert!(bridge.act(h, command(id, 0, 0, 1)).is_err());
    assert_same(&bridge, h, &saved, &frame);
}
#[test]
fn invalid_duplicate_stale_conflicting_and_detached_commands_preserve_all_domains() {
    let (mut bridge, h) = linked();
    bridge.act(h, command(10, 0, 0, 1)).unwrap();
    let save = bridge.save(h).unwrap();
    let frame = bridge.frame(h).unwrap();
    for c in [
        command(10, 0, 0, 1),
        command(10, 1, 0, 0),
        command(9, 1, 0, 0),
        command(11, 0, 0, 1),
        command(11, 1, 0, 1),
        command(11, 1, 99, 0),
        command(11, 1, 0, 99),
        command(0, 1, 0, 0),
    ] {
        assert_eq!(bridge.act(h, c).unwrap_err().code, ErrorCode::InvalidInput);
        assert_same(&bridge, h, &save, &frame);
    }
    bridge.detach_gameplay(h).unwrap();
    let detached = bridge.save(h).unwrap();
    let detached_frame = bridge.frame(h).unwrap();
    assert_eq!(
        bridge.act(h, command(11, 1, 0, 0)).unwrap_err().code,
        ErrorCode::GameplayDetached
    );
    assert!(bridge.detach_gameplay(h).is_err());
    assert_same(&bridge, h, &detached, &detached_frame);
    bridge
        .restore(h, &serde_json::to_vec(&detached).unwrap())
        .unwrap();
    assert_same(&bridge, h, &detached, &detached_frame);
    bridge.attach_gameplay(h).unwrap();
    assert_eq!(
        bridge.observe(h).unwrap().battle,
        save_battle(&save, &game())
    );
    assert!(bridge.act(h, command(10, 1, 0, 0)).is_err());
    let result = bridge.act(h, command(11, 1, 0, 0)).unwrap();
    assert_eq!(
        (result.battle.command_count, result.battle.last_command_id),
        (2, 11)
    );
}
fn save_battle(save: &p5::SaveBundle, config: &[u8]) -> p5::BattleState {
    let mut b = p5::NativeBridge::new();
    let h = b.create(config, 73, ExecutionMode::Headless).unwrap();
    b.restore(h, &serde_json::to_vec(save).unwrap()).unwrap();
    b.observe(h).unwrap().battle
}
#[test]
fn save_replay_detects_trace_core_clock_and_camera_tampering_without_mutation() {
    let (mut bridge, h) = linked();
    bridge.act(h, command(1, 0, 3, 1)).unwrap();
    bridge
        .set_camera(
            h,
            CameraRequest {
                zoom: 1.5,
                transition_ticks: 3,
                ..Default::default()
            },
        )
        .unwrap();
    bridge.set_view(h, ViewFamily::TopDown).unwrap();
    bridge.detach_gameplay(h).unwrap();
    bridge.attach_gameplay(h).unwrap();
    bridge.act(h, command(2, 1, 0, 0)).unwrap();
    let save = bridge.save(h).unwrap();
    let frame = bridge.frame(h).unwrap();
    let mut variants = Vec::new();
    let mut v = save.clone();
    v.logical_step += 1;
    variants.push(v);
    let mut v = save.clone();
    v.presentation_tick += 1;
    variants.push(v);
    let mut v = save.clone();
    v.gameplay_attached = false;
    variants.push(v);
    let mut v = save.clone();
    v.trace.remove(0);
    variants.push(v);
    let mut v = save.clone();
    v.trace.push(p5::TraceEntry::Gameplay { enabled: true });
    variants.push(v);
    let mut v = save.clone();
    v.core_save2.push(' ');
    variants.push(v);
    let mut v = save.clone();
    v.presentation_save1.push(' ');
    variants.push(v);
    let mut v = save.clone();
    v.version = 1;
    variants.push(v);
    let mut v = save.clone();
    v.seed += 1;
    variants.push(v);
    for bad in variants {
        assert!(
            bridge
                .restore(h, &serde_json::to_vec(&bad).unwrap())
                .is_err()
        );
        assert_same(&bridge, h, &save, &frame);
    }
    bridge
        .restore(h, &serde_json::to_vec(&save).unwrap())
        .unwrap();
    assert_same(&bridge, h, &save, &frame);
    let mut fresh = p5::NativeBridge::new();
    let fh = fresh.create(&game(), 73, ExecutionMode::Headless).unwrap();
    fresh
        .restore(fh, &serde_json::to_vec(&save).unwrap())
        .unwrap();
    assert_eq!(fresh.save(fh).unwrap(), save);
    assert_eq!(fresh.observe(fh).unwrap(), bridge.observe(h).unwrap());
}
#[test]
fn handles_are_foreign_to_p4_and_other_p5_registries_and_never_recycled() {
    let (mut a, h) = linked();
    let mut b = p5::NativeBridge::new();
    assert_eq!(b.observe(h).unwrap_err().code, ErrorCode::InvalidSession);
    let p4 = ge4g_pentomino_native::NativeBridge::new();
    assert_eq!(p4.observe(h).unwrap_err().code, ErrorCode::InvalidSession);
    a.close(h).unwrap();
    assert!(a.close(h).is_err());
    let replacement = b.create(&game(), 73, ExecutionMode::Headless).unwrap();
    assert_ne!(replacement, h);
}

#[test]
fn resource_failure_and_trace_budget_preserve_state_after_core_history_rollover() {
    let mut bridge = p5::NativeBridge::new();
    let h = bridge.create(&game(), 73, ExecutionMode::Headless).unwrap();
    for id in 1..=6 {
        let actor = if id % 2 == 1 { 0 } else { 1 };
        let action = if actor == 0 { 3 } else { 2 };
        let target = 1;
        bridge.act(h, command(id, actor, action, target)).unwrap();
    }
    let before = bridge.save(h).unwrap();
    assert_eq!(bridge.observe(h).unwrap().battle.fighters[0].resource, 1);
    assert!(bridge.act(h, command(7, 0, 3, 1)).is_err());
    assert_eq!(bridge.save(h).unwrap(), before);
    // The rejected ID is reusable. Outlive P2's 256-event history window.
    for id in 7..=270 {
        let actor = bridge.observe(h).unwrap().battle.turn;
        bridge.act(h, command(id, actor, 2, actor)).unwrap();
    }
    let saved = bridge.save(h).unwrap();
    assert_eq!(saved.trace.len(), 270);
    assert_eq!(bridge.observe(h).unwrap().battle.command_count, 270);
    bridge
        .restore(h, &serde_json::to_vec(&saved).unwrap())
        .unwrap();
    assert_eq!(bridge.save(h).unwrap(), saved);
    let mut oversized = saved.clone();
    oversized.trace.resize(
        p5::MAX_TRACE + 1,
        p5::TraceEntry::Gameplay { enabled: false },
    );
    assert_eq!(
        bridge
            .restore(h, &serde_json::to_vec(&oversized).unwrap())
            .unwrap_err()
            .code,
        ErrorCode::BudgetExceeded
    );
    assert_eq!(bridge.save(h).unwrap(), saved);
}

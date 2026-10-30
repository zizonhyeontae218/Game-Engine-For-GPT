//! Public Core integration plus a hand-calculated battle oracle.
use ge4g_pentomino::{
    Bindings,
    p2::{ActionId, ActionValue, CoreHost, CorePlugin, InputFrame},
};
use ge4g_pentomino_gameplay::v2::*;
fn config() -> GameConfig {
    let fighter = |id: &str, team, hp, resource, attack, defense, speed, x| FighterConfig {
        id: id.into(),
        team,
        hp,
        resource,
        attack,
        defense,
        speed,
        position: [x, 72],
        color: [40, 130, 230, 255],
    };
    GameConfig {
        version: 2,
        width: 200,
        height: 96,
        tick_hz: 60,
        fighters: vec![
            fighter("a", 0, 20, 4, 4, 1, 9, 30),
            fighter("b", 0, 15, 3, 3, 1, 5, 70),
            fighter("c", 1, 18, 4, 5, 2, 7, 130),
            fighter("d", 1, 14, 3, 3, 0, 1, 170),
        ],
        actions: vec![
            ActionConfig {
                id: "strike".into(),
                kind: ActionKind::Damage,
                target: Target::Enemy,
                cost: 1,
                power: 3,
                effect: None,
                duration: 0,
            },
            ActionConfig {
                id: "poison".into(),
                kind: ActionKind::Effect,
                target: Target::Enemy,
                cost: 2,
                power: 2,
                effect: Some(Effect::Poison),
                duration: 2,
            },
            ActionConfig {
                id: "heal".into(),
                kind: ActionKind::Heal,
                target: Target::Ally,
                cost: 1,
                power: 5,
                effect: None,
                duration: 0,
            },
            ActionConfig {
                id: "recover".into(),
                kind: ActionKind::Resource,
                target: Target::SelfOnly,
                cost: 0,
                power: 3,
                effect: None,
                duration: 0,
            },
            ActionConfig {
                id: "regen".into(),
                kind: ActionKind::Effect,
                target: Target::SelfOnly,
                cost: 1,
                power: 3,
                effect: Some(Effect::Regen),
                duration: 2,
            },
        ],
    }
}
fn host(c: &GameConfig) -> CoreHost {
    let mut h = CoreHost::new(7, "p5.contract").unwrap();
    h.install(
        Box::new(StateProvider::new(c.clone()).unwrap()),
        Bindings::new(),
    )
    .unwrap();
    h.install(Box::new(TurnCombat::new(c.clone()).unwrap()), bindings())
        .unwrap();
    h
}
fn state(h: &CoreHost, c: &GameConfig) -> BattleState {
    battle_state(&h.select(&selection(false)).unwrap(), c).unwrap()
}
fn act(h: &mut CoreHost, c: &GameConfig, id: i64, actor: usize, action: usize, target: usize) {
    let command = Command {
        id,
        actor,
        action,
        target,
    };
    let tick = h.describe().tick + 1;
    let prior = state(h, c);
    h.step(InputFrame {
        target_tick: tick,
        actions: command_actions(&command, c).unwrap(),
    })
    .unwrap();
    assert_eq!(state(h, c), prior, "decision tick cannot write authority");
    let actions =
        decision_actions(&h.select(&selection(true)).unwrap(), tick, &command, c).unwrap();
    h.step(InputFrame {
        target_tick: tick + 1,
        actions,
    })
    .unwrap();
}
#[test]
fn four_fighter_oracle_costs_targeting_poison_heal_and_recovery() {
    let c = config();
    let mut h = host(&c);
    assert_eq!(
        StateProvider::new(c.clone())
            .unwrap()
            .descriptor()
            .actions
            .len(),
        28
    );
    act(&mut h, &c, 1, 0, 0, 2);
    assert_eq!(
        (
            state(&h, &c).turn,
            state(&h, &c).fighters[2].hp,
            state(&h, &c).fighters[0].resource
        ),
        (2, 13, 3)
    );
    act(&mut h, &c, 2, 2, 1, 0);
    assert_eq!(
        state(&h, &c).fighters[0].poison,
        Status { power: 2, turns: 2 }
    );
    act(&mut h, &c, 3, 1, 2, 0); // capped healing
    act(&mut h, &c, 4, 3, 0, 1);
    assert_eq!(state(&h, &c).fighters[1].hp, 10);
    act(&mut h, &c, 5, 0, 0, 2); // poison ticks only on poisoned actor's turn
    assert_eq!(
        (
            state(&h, &c).fighters[0].hp,
            state(&h, &c).fighters[0].poison.turns
        ),
        (18, 1)
    );
    assert_eq!(state(&h, &c).fighters[2].hp, 8);
    act(&mut h, &c, 6, 2, 3, 2);
    assert_eq!(state(&h, &c).fighters[2].resource, 4);
    act(&mut h, &c, 7, 1, 2, 0);
    assert_eq!(state(&h, &c).fighters[0].hp, 20);
    act(&mut h, &c, 8, 3, 3, 3);
    act(&mut h, &c, 9, 0, 3, 0);
    assert_eq!(
        (
            state(&h, &c).fighters[0].hp,
            state(&h, &c).fighters[0].resource
        ),
        (18, 4)
    );
    assert_eq!(state(&h, &c).fighters[0].poison, Status::default());
    act(&mut h, &c, 10, 2, 4, 2);
    assert_eq!(
        (
            state(&h, &c).fighters[2].hp,
            state(&h, &c).fighters[2].regen.turns
        ),
        (11, 1)
    );
    assert_eq!(state(&h, &c).command_count, 10);
}
#[test]
fn rejected_commands_and_partial_batches_preserve_complete_core_bytes() {
    let c = config();
    let mut h = host(&c);
    act(&mut h, &c, 9, 0, 0, 2);
    for command in [
        Command {
            id: 9,
            actor: 2,
            action: 0,
            target: 0,
        },
        Command {
            id: 8,
            actor: 2,
            action: 0,
            target: 0,
        },
        Command {
            id: 10,
            actor: 0,
            action: 0,
            target: 2,
        },
        Command {
            id: 10,
            actor: 2,
            action: 0,
            target: 3,
        },
        Command {
            id: 10,
            actor: 2,
            action: 3,
            target: 0,
        },
    ] {
        let before = h.save().unwrap();
        assert!(
            h.step(InputFrame {
                target_tick: h.describe().tick + 1,
                actions: command_actions(&command, &c).unwrap()
            })
            .is_err()
        );
        assert_eq!(h.save().unwrap(), before);
    }
    let before = h.save().unwrap();
    for actions in [
        vec![(
            ActionId::new("p5.turn_combat.id").unwrap(),
            ActionValue::I64(10),
        )],
        vec![(ActionId::new("p5.state.turn").unwrap(), ActionValue::I64(0))],
    ] {
        assert!(
            h.step(InputFrame {
                target_tick: h.describe().tick + 1,
                actions
            })
            .is_err()
        );
        assert_eq!(h.save().unwrap(), before);
    }
    // A rejected id is available for the next legal command; gaps are accepted.
    act(&mut h, &c, 10, 2, 0, 0);
    assert_eq!(state(&h, &c).last_command_id, 10);
    act(&mut h, &c, 11, 1, 1, 2);
    act(&mut h, &c, 12, 3, 3, 3);
    act(&mut h, &c, 13, 0, 3, 0);
    act(&mut h, &c, 14, 2, 1, 0);
    act(&mut h, &c, 15, 1, 3, 1);
    act(&mut h, &c, 16, 3, 3, 3);
    act(&mut h, &c, 17, 0, 3, 0);
    // Actor2 has 1 resource after previous cost1+cost2 and cannot pay2.
    let before = h.save().unwrap();
    let command = Command {
        id: 18,
        actor: 2,
        action: 1,
        target: 0,
    };
    assert!(
        h.step(InputFrame {
            target_tick: h.describe().tick + 1,
            actions: command_actions(&command, &c).unwrap()
        })
        .is_err()
    );
    assert_eq!(h.save().unwrap(), before);
}
#[test]
fn actual_core_save_restore_replay_and_rule_reinstall_preserve_dedup_state() {
    let c = config();
    let mut a = host(&c);
    let mut b = host(&c);
    for h in [&mut a, &mut b] {
        act(h, &c, 1, 0, 0, 2);
        act(h, &c, 2, 2, 1, 0);
    }
    assert_eq!(a.save().unwrap(), b.save().unwrap());
    let saved = a.save().unwrap();
    let mut resumed = host(&c);
    resumed.restore(&saved).unwrap();
    for h in [&mut a, &mut resumed] {
        act(h, &c, 3, 1, 2, 0);
        act(h, &c, 4, 3, 0, 1);
    }
    assert_eq!(a.save().unwrap(), resumed.save().unwrap());
    let before = state(&b, &c);
    let token = b.owner(&rules_owner()).unwrap();
    b.remove(token).unwrap();
    assert_eq!(state(&b, &c), before);
    let detached = b.save().unwrap();
    let mut detached_host = host(&c);
    let token = detached_host.owner(&rules_owner()).unwrap();
    detached_host.remove(token).unwrap();
    detached_host.restore(&detached).unwrap();
    detached_host
        .install(Box::new(TurnCombat::new(c.clone()).unwrap()), bindings())
        .unwrap();
    let snapshot = detached_host.save().unwrap();
    let command = Command {
        id: 2,
        actor: 1,
        action: 2,
        target: 0,
    };
    assert!(
        detached_host
            .step(InputFrame {
                target_tick: 5,
                actions: command_actions(&command, &c).unwrap()
            })
            .is_err()
    );
    assert_eq!(detached_host.save().unwrap(), snapshot);
    act(&mut detached_host, &c, 3, 1, 2, 0);
    assert_eq!(state(&detached_host, &c).command_count, 3);
}
#[test]
fn simultaneous_last_enemies_poison_deaths_finish_in_draw_without_dead_turn() {
    let mut c = config();
    c.fighters = vec![c.fighters[0].clone(), c.fighters[2].clone()];
    c.fighters[0].hp = 2;
    c.fighters[1].hp = 2;
    c.fighters[0].resource = 4;
    c.fighters[1].resource = 4;
    let mut h = host(&c);
    act(&mut h, &c, 1, 0, 1, 1);
    act(&mut h, &c, 2, 1, 0, 0);
    let result = state(&h, &c);
    assert_eq!(
        result.fighters.iter().map(|f| f.hp).collect::<Vec<_>>(),
        vec![0, 0]
    );
    assert_eq!(result.winner, -2);
    let before = h.save().unwrap();
    let cmd = Command {
        id: 3,
        actor: 1,
        action: 3,
        target: 1,
    };
    assert!(
        h.step(InputFrame {
            target_tick: 5,
            actions: command_actions(&cmd, &c).unwrap()
        })
        .is_err()
    );
    assert_eq!(h.save().unwrap(), before);
}
#[test]
fn speed_ties_dead_skip_and_config_failures() {
    let mut c = config();
    c.fighters[0].speed = 0;
    c.fighters[1].speed = 0;
    c.fighters[2].speed = 0;
    c.fighters[3].speed = 0;
    c.fighters[2].hp = 1;
    let mut h = host(&c);
    act(&mut h, &c, 1, 0, 0, 2);
    assert_eq!(state(&h, &c).turn, 1);
    act(&mut h, &c, 2, 1, 0, 3);
    assert_eq!(state(&h, &c).turn, 3);
    let mut malformed = c.clone();
    malformed.actions[0].duration = 1;
    assert!(malformed.validate().is_err());
    malformed = c.clone();
    malformed.fighters.push(c.fighters[0].clone());
    assert!(malformed.validate().is_err());
    malformed = c.clone();
    malformed.fighters[1].id = malformed.fighters[0].id.clone();
    assert!(malformed.validate().is_err());
    assert!(
        serde_json::from_str::<Command>(r#"{"id":1,"actor":0,"action":0,"target":2,"extra":true}"#)
            .is_err()
    );
}

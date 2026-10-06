//! Authoritative turn resolution, separate from view and cosmetic effects.
use super::battle_fx::{BattleFxEvent, FX_TICKS};
use super::*;
use ge4g_project::flatland::Action;
use ge4g_project::gameplay::{BattleFxPreset, Fighter, Instruction};
fn is_zero(n: &u64) -> bool {
    *n == 0
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BattleState {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fx: Vec<super::battle_fx::BattleFxEvent>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub resolving_until: u64,
    #[serde(default)]
    pub turn_tick: u64,
    #[serde(default)]
    pub previous_hp: BTreeMap<String, i64>,
    #[serde(default)]
    pub result: Option<bool>,
    pub fighters: Vec<Fighter>,
    pub turn: u64,
    pub log: String,
}
impl World {
    pub(super) fn battle_choice(&mut self, choice: &str) -> Result<()> {
        let frame = self.systems().events.last().unwrap().clone();
        let mut battle = frame.battle.unwrap();
        if self.tick < battle.resolving_until {
            return Err(Error("battle presentation resolving; wait or skip".into()));
        }
        battle.fx.clear();
        let fx_enabled = self
            .project
            .manifest
            .features
            .iter()
            .any(|f| f == "battle_fx");
        if let Some(won) = battle.result {
            if choice != "battle_continue" {
                return Err(Error("battle ended".into()));
            }
            return self.finish_battle(won);
        }
        battle.previous_hp = battle
            .fighters
            .iter()
            .map(|f| (f.id.clone(), f.hp))
            .collect();
        battle.turn_tick = self.tick;
        let hero = battle.fighters.iter().position(|f| !f.enemy).unwrap();
        let selected_move =
            battle.fighters[hero]
                .moves
                .iter()
                .enumerate()
                .find_map(|(index, m)| {
                    battle
                        .fighters
                        .iter()
                        .find(|f| {
                            f.enemy
                                && f.hp > 0
                                && choice == format!("move:{}:{}", m.id, f.id)
                                && m.pp > 0
                        })
                        .map(|f| (index, f.id.clone()))
                });
        let move_power = selected_move
            .as_ref()
            .map_or(100, |(index, _)| battle.fighters[hero].moves[*index].power);
        let move_name = selected_move
            .as_ref()
            .map_or("공격".to_owned(), |(index, _)| {
                battle.fighters[hero].moves[*index].name.clone()
            });
        let move_target = selected_move.as_ref().map(|(_, id)| id.as_str());
        let legacy_target = if battle.fighters[hero].moves.is_empty() {
            choice.strip_prefix("attack_")
        } else {
            None
        };
        let target = if let Some(id) = legacy_target.or(move_target) {
            Some(
                battle
                    .fighters
                    .iter()
                    .position(|f| f.id == id && f.enemy && f.hp > 0)
                    .ok_or_else(|| Error("invalid battle target".into()))?,
            )
        } else {
            None
        };
        let item = choice.strip_prefix("item_");
        if target.is_none() && choice != "guard" && item.is_none() {
            return Err(Error("unknown battle action".into()));
        }
        if let Some(item) = item
            && self.inventory(item) < 1
        {
            return Err(Error("battle item missing".into()));
        }
        let mut order: Vec<usize> = (0..battle.fighters.len()).collect();
        order.sort_by_key(|i| (-battle.fighters[*i].speed, battle.fighters[*i].id.clone()));
        let mut log = Vec::new();
        if choice == "guard" && fx_enabled {
            battle.fx.push(BattleFxEvent {
                actor: battle.fighters[hero].id.clone(),
                target: battle.fighters[hero].id.clone(),
                preset: BattleFxPreset::Guard,
                start_tick: self.tick,
                before_hp: battle.fighters[hero].hp,
                after_hp: battle.fighters[hero].hp,
            });
        }
        for i in order {
            if battle.fighters[i].hp == 0 {
                continue;
            }
            if i == hero {
                if let Some(item) = item {
                    let definition = self.project.scenes[&self.scene]
                        .gameplay
                        .items
                        .get(item)
                        .cloned()
                        .ok_or_else(|| Error("battle item undefined".into()))?;
                    let healing = definition
                        .use_actions
                        .iter()
                        .filter_map(|a| {
                            if let Action::Heal { amount, .. } = a {
                                Some(*amount)
                            } else {
                                None
                            }
                        })
                        .sum::<i64>();
                    if healing <= 0 {
                        return Err(Error("battle item needs a heal effect".into()));
                    }
                    self.actions(
                        &[Action::Take {
                            item: item.into(),
                            count: 1,
                        }],
                        None,
                    )?;
                    let max = match &self.project.scenes[&self.scene].gameplay.events[&frame.event]
                        [frame.pc]
                    {
                        Instruction::Battle { fighters, .. } => {
                            self.project.scenes[&self.scene]
                                .gameplay
                                .fighter(&fighters[hero])?
                                .hp
                        }
                        _ => unreachable!(),
                    };
                    let before_hp = battle.fighters[hero].hp;
                    battle.fighters[hero].hp = (battle.fighters[hero].hp + healing).min(max);
                    if fx_enabled {
                        battle.fx.push(BattleFxEvent {
                            actor: battle.fighters[hero].id.clone(),
                            target: battle.fighters[hero].id.clone(),
                            preset: BattleFxPreset::Heal,
                            start_tick: self.tick + battle.fx.len() as u64 * FX_TICKS,
                            before_hp,
                            after_hp: battle.fighters[hero].hp,
                        });
                    }
                    log.push(format!("{}: +{} HP", definition.name, healing));
                } else if let Some(t) = target
                    && battle.fighters[t].hp > 0
                {
                    if let Some((index, _)) = &selected_move {
                        battle.fighters[hero].moves[*index].pp -= 1;
                    }
                    let damage = (battle.fighters[i].attack * move_power / 100
                        - battle.fighters[t].defense
                        + self.random(0, 2)?)
                    .max(1);
                    let before_hp = battle.fighters[t].hp;
                    battle.fighters[t].hp = (battle.fighters[t].hp - damage).max(0);
                    if fx_enabled {
                        battle.fx.push(BattleFxEvent {
                            actor: battle.fighters[i].id.clone(),
                            target: battle.fighters[t].id.clone(),
                            preset: selected_move
                                .as_ref()
                                .and_then(|(n, _)| battle.fighters[i].moves[*n].fx)
                                .unwrap_or_default(),
                            start_tick: self.tick + battle.fx.len() as u64 * FX_TICKS,
                            before_hp,
                            after_hp: battle.fighters[t].hp,
                        });
                    }
                    log.push(format!(
                        "{}의 {} → {}: {} 피해",
                        battle.fighters[i].name, move_name, battle.fighters[t].name, damage
                    ));
                }
            } else if battle.fighters[hero].hp > 0 {
                let selected = battle.fighters[i].moves.iter().position(|m| m.pp > 0);
                let (power, preset) = selected.map_or((100, BattleFxPreset::Strike), |m| {
                    (
                        battle.fighters[i].moves[m].power,
                        battle.fighters[i].moves[m].fx.unwrap_or_default(),
                    )
                });
                if let Some(m) = selected {
                    battle.fighters[i].moves[m].pp -= 1;
                }
                let damage = (battle.fighters[i].attack * power / 100
                    - battle.fighters[hero].defense
                    + self.random(0, 2)?)
                .max(1);
                let damage = if choice == "guard" {
                    (damage / 2).max(1)
                } else {
                    damage
                };
                let before_hp = battle.fighters[hero].hp;
                battle.fighters[hero].hp = (battle.fighters[hero].hp - damage).max(0);
                if fx_enabled {
                    battle.fx.push(BattleFxEvent {
                        actor: battle.fighters[i].id.clone(),
                        target: battle.fighters[hero].id.clone(),
                        preset,
                        start_tick: self.tick + battle.fx.len() as u64 * FX_TICKS,
                        before_hp,
                        after_hp: battle.fighters[hero].hp,
                    });
                }
                log.push(format!(
                    "{} → {}: {}",
                    battle.fighters[i].name, battle.fighters[hero].name, damage
                ));
            }
        }
        battle.resolving_until = if battle.fx.is_empty() {
            0
        } else {
            self.tick + battle.fx.len() as u64 * FX_TICKS
        };
        self.commit_fighters(&battle.fighters);
        battle.turn += 1;
        battle.log = log.join("\n");
        let won = !battle.fighters.iter().any(|f| f.enemy && f.hp > 0);
        let lost = battle.fighters[hero].hp == 0;
        self.emit(
            "battle_turn",
            None,
            json!({"turn":battle.turn,"fighters":battle.fighters,"log":battle.log}),
        );
        if won || lost {
            let hold = matches!(&self.project.scenes[&self.scene].gameplay.events[&frame.event][frame.pc],Instruction::Battle{stage:Some(stage),..} if stage.hold_result);
            if hold || fx_enabled {
                battle.result = Some(won);
                battle.log = if won {
                    "승리! 실험을 완료했습니다.".into()
                } else {
                    "패배했습니다. 다시 도전할 수 있습니다.".into()
                };
                self.systems().events.last_mut().unwrap().battle = Some(battle);
            } else {
                self.systems().events.last_mut().unwrap().battle = Some(battle);
                self.finish_battle(won)?;
            }
        } else {
            self.systems().events.last_mut().unwrap().battle = Some(battle);
        }
        Ok(())
    }
    pub(super) fn finish_battle(&mut self, won: bool) -> Result<()> {
        let frame = self.systems().events.last().unwrap().clone();
        let effects =
            match &self.project.scenes[&self.scene].gameplay.events[&frame.event][frame.pc] {
                Instruction::Battle {
                    victory, defeat, ..
                } => {
                    if won {
                        victory.clone()
                    } else {
                        defeat.clone()
                    }
                }
                _ => unreachable!(),
            };
        let f = self.systems().events.last_mut().unwrap();
        f.battle = None;
        f.waiting = false;
        f.pc += 1;
        self.actions(&effects, None)?;
        self.emit("battle_result", None, json!({"victory":won}));
        self.flat_event(if won { "battle_win" } else { "battle_loss" }, None)
    }
}

//! Explicit persistent roster. Inline fighters remain ephemeral.
use super::*;
use ge4g_project::gameplay::Fighter;
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CombatantState {
    pub current_hp: i64,
    pub remaining_pp: BTreeMap<String, u32>,
}
impl CombatantState {
    pub fn fresh(f: &Fighter) -> Self {
        Self {
            current_hp: f.hp,
            remaining_pp: f.moves.iter().map(|m| (m.id.clone(), m.pp)).collect(),
        }
    }
    pub fn apply(&self, f: &mut Fighter) {
        f.hp = self.current_hp;
        for m in &mut f.moves {
            m.pp = self.remaining_pp[&m.id];
        }
    }
    pub fn valid(&self, f: &Fighter) -> bool {
        self.current_hp >= 0
            && self.current_hp <= f.hp
            && self.remaining_pp.len() == f.moves.len()
            && f.moves
                .iter()
                .all(|m| self.remaining_pp.get(&m.id).is_some_and(|pp| *pp <= m.pp))
    }
}
impl World {
    pub(super) fn initialize_fighters(&mut self, fighters: Vec<Fighter>) -> Result<Vec<Fighter>> {
        let mut out = Vec::new();
        let mut bound = BTreeSet::new();
        for authored in fighters {
            let mut f = self.project.scenes[&self.scene]
                .gameplay
                .fighter(&authored)?;
            if let Some(id) = &f.combatant {
                if !bound.insert(id.clone()) {
                    return Err(Error("combatant bound twice in one battle".into()));
                }
                let state = self
                    .systems()
                    .combatants
                    .entry(id.clone())
                    .or_insert_with(|| CombatantState::fresh(&f));
                state.apply(&mut f);
            }
            out.push(f);
        }
        Ok(out)
    }
    pub(super) fn commit_fighters(&mut self, fighters: &[Fighter]) {
        for f in fighters {
            if let Some(id) = &f.combatant {
                self.systems()
                    .combatants
                    .insert(id.clone(), CombatantState::fresh(f));
            }
        }
    }
    pub(super) fn combatant_heal(&mut self, id: &str, amount: Option<i64>) -> Result<()> {
        if self
            .flatland
            .systems
            .as_ref()
            .is_some_and(|s| s.events.iter().any(|e| e.battle.is_some()))
        {
            return Err(Error("heal/reset roster outside battle".into()));
        }
        let definition = self.project.scenes[&self.scene]
            .gameplay
            .combatants
            .get(id)
            .cloned()
            .ok_or_else(|| Error("unknown combatant".into()))?;
        let state = self
            .systems()
            .combatants
            .entry(id.into())
            .or_insert_with(|| CombatantState::fresh(&definition));
        if let Some(amount) = amount {
            state.current_hp = (state.current_hp + amount).min(definition.hp);
        } else {
            *state = CombatantState::fresh(&definition);
        }
        Ok(())
    }
}

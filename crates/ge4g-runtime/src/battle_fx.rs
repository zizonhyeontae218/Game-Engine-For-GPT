//! Deterministic cosmetic timeline. Combat has already resolved before these events.
use super::*;
use ge4g_project::gameplay::BattleFxPreset;
pub const FX_TICKS: u64 = 54;
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BattleFxEvent {
    pub actor: String,
    pub target: String,
    pub preset: BattleFxPreset,
    pub start_tick: u64,
    pub before_hp: i64,
    pub after_hp: i64,
}
impl BattleFxEvent {
    pub fn shown_hp(&self, tick: u64) -> i64 {
        let age = tick.saturating_sub(self.start_tick);
        if tick < self.start_tick || age < 26 {
            self.before_hp
        } else {
            self.before_hp
                + (self.after_hp - self.before_hp) * age.saturating_sub(26).min(20) as i64 / 20
        }
    }
}

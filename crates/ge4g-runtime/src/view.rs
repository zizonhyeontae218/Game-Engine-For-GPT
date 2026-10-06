//! Persistent camera presentation. Never owns collision, position, facing or RNG.
use ge4g_project::gameplay::View;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GameplayView {
    pub view_id: String,
    pub zoom: i64,
    pub tilt: i64,
    pub shear: i64,
}
impl GameplayView {
    pub fn new(id: &str, view: &View) -> Self {
        Self {
            view_id: id.into(),
            zoom: view.zoom,
            tilt: view.tilt,
            shear: view.shear,
        }
    }
    pub fn valid(&self) -> bool {
        ge4g_project::valid_id(&self.view_id)
            && (100..=160).contains(&self.zoom)
            && (60..=100).contains(&self.tilt)
            && self.shear.unsigned_abs() <= 25
    }
}

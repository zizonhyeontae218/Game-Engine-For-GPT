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
/// A scoped camera transition uses ticks only for presentation; simulation is untouched.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CameraBlend {
    pub from: [i64; 2],
    pub start_tick: u64,
}
impl super::World {
    pub(super) fn camera_position(&self) -> [i64; 2] {
        let target = self.target_camera_position();
        if let Some(blend) = self
            .flatland
            .systems
            .as_ref()
            .and_then(|s| s.camera_blend.as_ref())
        {
            let t = self.tick.saturating_sub(blend.start_tick).min(12) as i64;
            // Integer smoothstep, 12 ticks / 200ms. Never consulted by collisions or events.
            let weight = t * t * (36 - 2 * t);
            return std::array::from_fn(|i| {
                blend.from[i] + (target[i] - blend.from[i]) * weight / 1728
            });
        }
        target
    }
    pub(super) fn blend_camera(&mut self, target: Option<[i64; 2]>) {
        if self.flatland.systems.as_ref().and_then(|s| s.camera) == target {
            return;
        }
        let from = self.camera_position();
        let enabled = self
            .project
            .manifest
            .features
            .iter()
            .any(|f| f == "cutscene_bubbles");
        let tick = self.tick;
        let s = self.systems();
        s.camera = target;
        s.camera_blend = enabled.then_some(CameraBlend {
            from,
            start_tick: tick,
        });
    }
}

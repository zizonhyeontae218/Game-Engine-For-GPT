//! Compact semantic building presentation; a missing surface uses an engine default.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Building {
    #[serde(default = "footprint")]
    pub footprint: [u32; 2],
    #[serde(default = "height")]
    pub height: u32,
    #[serde(default)]
    pub roof: Roof,
    #[serde(default)]
    pub material: Material,
    #[serde(default = "facing")]
    pub facing: String,
    #[serde(default)]
    pub roof_surface: Option<String>,
    #[serde(default)]
    pub facade_surface: Option<String>,
    #[serde(default)]
    pub side_surface: Option<String>,
}
fn footprint() -> [u32; 2] {
    [48, 32]
}
fn height() -> u32 {
    26
}
fn facing() -> String {
    "south".into()
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Roof {
    #[default]
    Gable,
    Flat,
    Shed,
}
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Material {
    #[default]
    Wood,
    Plaster,
    Brick,
    Stone,
    Metal,
}
impl Building {
    pub fn valid(&self) -> bool {
        self.footprint.iter().all(|n| (8..=256).contains(n))
            && (8..=128).contains(&self.height)
            && ["south", "north", "east", "west"].contains(&self.facing.as_str())
    }
    pub fn surfaces(&self) -> impl Iterator<Item = &String> {
        [&self.roof_surface, &self.facade_surface, &self.side_surface]
            .into_iter()
            .flatten()
    }
}

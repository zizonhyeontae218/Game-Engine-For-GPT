//! Pure, versioned engine types. No window, clock, filesystem or network access.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const SCHEMA_VERSION: u32 = 1;
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const TICK_HZ: u32 = 60;
pub const SUBPIXELS: i64 = 60;

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct Error(pub String);
pub type Result<T> = std::result::Result<T, Error>;

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct Vec2 {
    pub x: i64,
    pub y: i64,
}
impl Vec2 {
    pub fn pixels(x: i64, y: i64) -> Self {
        Self {
            x: x * SUBPIXELS,
            y: y * SUBPIXELS,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Aabb {
    pub position: Vec2,
    pub size: Vec2,
}
impl Aabb {
    pub fn swept_bounds(self, destination: Vec2) -> Self {
        let position = Vec2 {
            x: self.position.x.min(destination.x),
            y: self.position.y.min(destination.y),
        };
        Self {
            position,
            size: Vec2 {
                x: self.size.x + (destination.x - self.position.x).abs(),
                y: self.size.y + (destination.y - self.position.y).abs(),
            },
        }
    }
    pub fn overlaps(self, other: Self) -> bool {
        self.position.x < other.position.x + other.size.x
            && self.position.x + self.size.x > other.position.x
            && self.position.y < other.position.y + other.size.y
            && self.position.y + self.size.y > other.position.y
    }
    /// Clamp a swept movement against a blocking box. Resolve X before Y.
    pub fn sweep_axis(self, wall: Self, requested: i64, x_axis: bool) -> i64 {
        let (p, s, w, ws, cross) = if x_axis {
            (
                self.position.x,
                self.size.x,
                wall.position.x,
                wall.size.x,
                self.position.y < wall.position.y + wall.size.y
                    && self.position.y + self.size.y > wall.position.y,
            )
        } else {
            (
                self.position.y,
                self.size.y,
                wall.position.y,
                wall.size.y,
                self.position.x < wall.position.x + wall.size.x
                    && self.position.x + self.size.x > wall.position.x,
            )
        };
        if !cross {
            return requested;
        }
        if requested > 0 && p + s <= w {
            requested.min(w - p - s)
        } else if requested < 0 && p >= w + ws {
            requested.max(w + ws - p)
        } else {
            requested
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<[i64; 2]>,
    #[serde(default)]
    pub left: bool,
    #[serde(default)]
    pub right: bool,
    #[serde(default)]
    pub up: bool,
    #[serde(default)]
    pub down: bool,
    #[serde(default)]
    pub interact: bool,
}
impl Input {
    pub fn axes(&self) -> (i64, i64) {
        (
            i64::from(self.right) - i64::from(self.left),
            i64::from(self.down) - i64::from(self.up),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StateType {
    Bool,
    Integer,
    String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StateDefinition {
    pub kind: StateType,
    pub default: Value,
    #[serde(default = "yes")]
    pub persistent: bool,
}
fn yes() -> bool {
    true
}
impl StateDefinition {
    pub fn accepts(&self, value: &Value) -> bool {
        match self.kind {
            StateType::Bool => value.is_boolean(),
            StateType::Integer => value.as_i64().is_some(),
            StateType::String => value.is_string(),
        }
    }
}
#[derive(Debug, Clone)]
pub struct StateStore {
    pub definitions: BTreeMap<String, StateDefinition>,
    pub values: BTreeMap<String, Value>,
}
impl StateStore {
    pub fn new(definitions: BTreeMap<String, StateDefinition>) -> Result<Self> {
        for (key, def) in &definitions {
            if !key.contains('.')
                || key.split('.').any(|part| {
                    part.is_empty()
                        || !part
                            .bytes()
                            .all(|c| c.is_ascii_alphanumeric() || b"_-".contains(&c))
                })
                || !def.accepts(&def.default)
            {
                return Err(Error(format!(
                    "state {key}: expected a namespaced key and {:?} default",
                    def.kind
                )));
            }
        }
        let values = definitions
            .iter()
            .map(|(key, def)| (key.clone(), def.default.clone()))
            .collect();
        Ok(Self {
            definitions,
            values,
        })
    }
    pub fn set(&mut self, key: &str, value: Value) -> Result<bool> {
        let def = self
            .definitions
            .get(key)
            .ok_or_else(|| Error(format!("unknown state key {key}")))?;
        if !def.accepts(&value) {
            return Err(Error(format!(
                "state {key}: expected {:?}, received {value}",
                def.kind
            )));
        }
        let changed = self.values.get(key) != Some(&value);
        self.values.insert(key.to_owned(), value);
        Ok(changed)
    }
    pub fn persistent_values(&self) -> BTreeMap<String, Value> {
        self.values
            .iter()
            .filter(|(key, _)| self.definitions[*key].persistent)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub tick: u64,
    pub kind: String,
    pub scene: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<String>,
    pub data: Value,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EntitySnapshot {
    pub id: String,
    pub position: Vec2,
    pub size: [u32; 2],
    pub components: Vec<String>,
    pub tags: Vec<String>,
    pub metadata: BTreeMap<String, Value>,
    pub color: Option<[u8; 4]>,
    pub texture: Option<String>,
    pub layer: i32,
    pub blocking: bool,
    pub trigger: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flatland: Option<Value>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub schema_version: u32,
    pub engine_version: String,
    pub tick: u64,
    pub tick_hz: u32,
    pub subpixels_per_pixel: i64,
    pub scene: String,
    pub camera: [i64; 2],
    pub background: [u8; 4],
    pub entities: Vec<EntitySnapshot>,
    pub state: BTreeMap<String, Value>,
    pub events: Vec<Event>,
    pub events_dropped: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flatland: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn swept_aabb_does_not_tunnel_and_allows_edge_sliding() {
        let body = Aabb {
            position: Vec2::pixels(0, 0),
            size: Vec2::pixels(8, 8),
        };
        let wall = Aabb {
            position: Vec2::pixels(20, 0),
            size: Vec2::pixels(4, 30),
        };
        assert_eq!(body.sweep_axis(wall, 100 * SUBPIXELS, true), 12 * SUBPIXELS);
        let edge = Aabb {
            position: Vec2::pixels(12, 0),
            ..body
        };
        assert!(!edge.overlaps(wall));
        assert_eq!(edge.sweep_axis(wall, 10, false), 10);
    }
    #[test]
    fn state_types_and_namespaces_are_enforced() {
        let defs = BTreeMap::from([(
            "demo.flag".into(),
            StateDefinition {
                kind: StateType::Bool,
                default: Value::Bool(false),
                persistent: true,
            },
        )]);
        let mut state = StateStore::new(defs).unwrap();
        assert!(state.set("demo.flag", Value::Bool(true)).unwrap());
        assert!(!state.set("demo.flag", Value::Bool(true)).unwrap());
        assert!(state.set("demo.flag", Value::from(1)).is_err());
        assert!(state.set("other.flag", Value::Bool(true)).is_err());
    }
    #[test]
    fn opposing_actions_cancel() {
        assert_eq!(
            Input {
                left: true,
                right: true,
                ..Input::default()
            }
            .axes(),
            (0, 0)
        );
    }
}

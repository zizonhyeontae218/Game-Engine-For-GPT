use ge4g_pentomino::{
    PluginId,
    p2::{EntityRef, SceneRef, SchemaId},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    InvalidIdentifier,
    InvalidDescriptor,
    DuplicateId,
    MissingView,
    MissingCamera,
    InvalidSelection,
    InvalidTransform,
    InvalidProjection,
    InvalidViewport,
    InvalidInput,
    InputOutOfSequence,
    OutsideDepth,
    BudgetExceeded,
    Overflow,
    PluginFailed,
    VersionMismatch,
    SaveMismatch,
    InvalidSave,
    Unsupported,
}
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{code:?}: {detail}")]
pub struct Error {
    code: ErrorCode,
    detail: String,
}
impl Error {
    pub fn new(code: ErrorCode, detail: impl Into<String>) -> Self {
        let mut detail = detail.into();
        let mut n = detail.len().min(4096);
        while !detail.is_char_boundary(n) {
            n -= 1;
        }
        detail.truncate(n);
        Self { code, detail }
    }
    pub fn code(&self) -> ErrorCode {
        self.code
    }
    pub fn detail(&self) -> &str {
        &self.detail
    }
}
pub(crate) fn err(code: ErrorCode, detail: &str) -> Error {
    Error::new(code, detail)
}
pub(crate) fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.contains('.')
        && s.split('.').all(|p| {
            !p.is_empty()
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
        })
}
macro_rules! id {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);
        impl $name {
            pub fn new(s: &str) -> Result<Self, Error> {
                if valid_id(s) {
                    Ok(Self(s.into()))
                } else {
                    Err(err(
                        ErrorCode::InvalidIdentifier,
                        "invalid dotted ASCII identifier",
                    ))
                }
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                Self::new(&String::deserialize(d)?).map_err(serde::de::Error::custom)
            }
        }
    };
}
id!(ViewId);
id!(CameraId);
id!(FormatId);
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldPoint(pub [f64; 3]);
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewPoint(pub [f64; 3]);
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraPoint(pub [f64; 3]);
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
    pub depth: f64,
}
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Quaternion {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}
impl Quaternion {
    pub const fn identity() -> Self {
        Self {
            x: 0.,
            y: 0.,
            z: 0.,
            w: 1.,
        }
    }
    pub fn normalized(self) -> Result<Self, Error> {
        crate::math::normalize_quaternion(self)
    }
}
impl Default for Quaternion {
    fn default() -> Self {
        Self::identity()
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewTransform {
    pub origin: [f64; 3],
    pub rotation: Quaternion,
    pub scale: [f64; 3],
    pub shear_xy: f64,
}
impl Default for ViewTransform {
    fn default() -> Self {
        Self {
            origin: [0.; 3],
            rotation: Quaternion::identity(),
            scale: [1.; 3],
            shear_xy: 0.,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraPose {
    pub position: [f64; 3],
    pub orientation: Quaternion,
    pub zoom: f64,
}
impl Default for CameraPose {
    fn default() -> Self {
        Self {
            position: [0., 0., 1.],
            orientation: Quaternion::identity(),
            zoom: 1.,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Viewport {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Projection {
    Orthographic {
        half_height: f64,
        near: f64,
        far: f64,
        focus_distance: f64,
    },
    Perspective {
        half_height: f64,
        near: f64,
        far: f64,
        focus_distance: f64,
    },
    Blended {
        half_height: f64,
        near: f64,
        far: f64,
        focus_distance: f64,
        perspective_weight: f64,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraTarget {
    pub pose: CameraPose,
    pub view_transform: ViewTransform,
    pub projection: Projection,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewFamily {
    Classic2D,
    TopDown,
    Side,
    Vertical,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordBinding {
    pub owner: PluginId,
    pub schema: SchemaId,
    pub entity_field: String,
    pub x_field: String,
    pub y_field: String,
    pub z_field: Option<String>,
    pub units_per_world: f64,
    pub representation: Representation,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Representation {
    Sprite { asset: String, size: [f64; 2] },
    Model { asset: String },
    Background { asset: String },
    Billboard { asset: String, size: [f64; 2] },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationItem {
    pub entity: EntityRef,
    pub world: WorldPoint,
    pub representation: Representation,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PresentationSelection {
    pub scene: SceneRef,
    pub items: Vec<PresentationItem>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormatDescriptor {
    pub id: FormatId,
    pub version: u32,
    pub family: ViewFamily,
    pub required_fields: BTreeMap<String, String>,
    pub representations: Vec<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AxisMask {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub min: [f64; 3],
    pub max: [f64; 3],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewPolicy {
    pub family: ViewFamily,
    pub follow_axes: AxisMask,
}
impl ViewPolicy {
    pub fn classic2d() -> Self {
        Self {
            family: ViewFamily::Classic2D,
            follow_axes: AxisMask {
                x: true,
                y: true,
                z: false,
            },
        }
    }
    pub fn top_down() -> Self {
        Self {
            family: ViewFamily::TopDown,
            follow_axes: AxisMask {
                x: true,
                y: true,
                z: false,
            },
        }
    }
    pub fn side() -> Self {
        Self {
            family: ViewFamily::Side,
            follow_axes: AxisMask {
                x: true,
                y: false,
                z: false,
            },
        }
    }
    pub fn vertical() -> Self {
        Self {
            family: ViewFamily::Vertical,
            follow_axes: AxisMask {
                x: false,
                y: true,
                z: false,
            },
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewConfig {
    pub id: ViewId,
    pub scene: SceneRef,
    pub binding: RecordBinding,
    pub policy: ViewPolicy,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FollowBehavior {
    pub target: EntityRef,
    pub offset: [f64; 3],
    pub dead_zone: [f64; 3],
    pub look_ahead_ticks: u32,
    pub smoothing_ticks: u32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShakeBehavior {
    pub seed: u64,
    pub amplitude: [f64; 3],
    pub duration_ticks: u32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraBehaviors {
    pub follow: Option<FollowBehavior>,
    pub bounds: Option<Bounds>,
    pub shake: Option<ShakeBehavior>,
    pub zoom_range: [f64; 2],
}
impl Default for CameraBehaviors {
    fn default() -> Self {
        Self {
            follow: None,
            bounds: None,
            shake: None,
            zoom_range: [0.001, 1000.],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TransitionMode {
    Instant,
    Smooth { duration_ticks: u32 },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraConfig {
    pub id: CameraId,
    pub view: ViewId,
    pub viewport: Viewport,
    pub target: CameraTarget,
    pub behaviors: CameraBehaviors,
    pub default_transition: TransitionMode,
}
impl CameraConfig {
    pub fn new(id: CameraId, view: ViewId, viewport: Viewport, target: CameraTarget) -> Self {
        Self {
            id,
            view,
            viewport,
            target,
            behaviors: CameraBehaviors::default(),
            default_transition: TransitionMode::Smooth { duration_ticks: 12 },
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraChange {
    pub camera: CameraId,
    pub target: CameraTarget,
    pub transition: Option<TransitionMode>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewInput {
    pub target_tick: u64,
    pub changes: Vec<CameraChange>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderItem {
    pub entity: EntityRef,
    pub world: WorldPoint,
    pub screen: ScreenPoint,
    pub representation: Representation,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraFrame {
    pub camera: CameraId,
    pub view: ViewId,
    pub scene: SceneRef,
    pub viewport: Viewport,
    pub target: CameraTarget,
    pub items: Vec<RenderItem>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RenderFrame {
    pub tick: u64,
    pub source_tick: u64,
    pub cameras: Vec<CameraFrame>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraStatus {
    pub id: CameraId,
    pub view: ViewId,
    pub active: bool,
    pub current: CameraTarget,
    pub transitioning: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewStatus {
    pub config: ViewConfig,
    pub format: FormatDescriptor,
    pub cameras: Vec<CameraId>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewLimits {
    pub max_views: usize,
    pub max_cameras: usize,
    pub max_items_per_view: usize,
    pub max_changes: usize,
    pub max_save_bytes: usize,
    pub max_transition_ticks: u32,
}
impl Default for ViewLimits {
    fn default() -> Self {
        Self {
            max_views: 8,
            max_cameras: 32,
            max_items_per_view: 4096,
            max_changes: 32,
            max_save_bytes: 1_048_576,
            max_transition_ticks: 4096,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewDiscovery {
    pub contract_version: u32,
    pub tick: u64,
    pub available_formats: Vec<FormatDescriptor>,
    pub installed_views: Vec<ViewStatus>,
    pub cameras: Vec<CameraStatus>,
    pub projections: Vec<String>,
    pub behaviors: Vec<String>,
    pub limits: ViewLimits,
}

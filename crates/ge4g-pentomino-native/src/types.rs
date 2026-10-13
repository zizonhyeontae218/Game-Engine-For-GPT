use ge4g_pentomino::{PluginId, p2};
use ge4g_pentomino_gameplay::{BattleState, GameConfig};
use ge4g_pentomino_view::{CameraFrame, ViewFamily};
use serde::{Deserialize, Serialize};

pub const CONTRACT_VERSION: u32 = 1;
pub const MAX_REQUEST_BYTES: usize = 24 * 1024 * 1024;
pub const MAX_CONFIG_BYTES: usize = 65_536;
pub const MAX_SESSIONS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionHandle(pub(crate) u64);
impl SessionHandle {
    pub fn id(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Headless,
    Linked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidSession,
    InvalidConfig,
    InvalidInput,
    InputOutOfSequence,
    GameplayDetached,
    InvalidSave,
    SaveMismatch,
    VersionMismatch,
    Unsupported,
    BudgetExceeded,
    Overflow,
    CoreFailure,
    ViewFailure,
    RenderFailure,
    InternalFailure,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{code:?}: {detail}")]
#[serde(deny_unknown_fields)]
pub struct BridgeError {
    pub code: ErrorCode,
    pub detail: String,
}
impl BridgeError {
    pub fn new(code: ErrorCode, detail: impl Into<String>) -> Self {
        let mut detail = detail.into();
        let mut end = detail.len().min(4096);
        while !detail.is_char_boundary(end) {
            end -= 1;
        }
        detail.truncate(end);
        Self { code, detail }
    }
}
impl From<p2::Error> for BridgeError {
    fn from(e: p2::Error) -> Self {
        Self::new(ErrorCode::CoreFailure, e.to_string())
    }
}
impl From<ge4g_pentomino_view::Error> for BridgeError {
    fn from(e: ge4g_pentomino_view::Error) -> Self {
        Self::new(ErrorCode::ViewFailure, e.to_string())
    }
}
pub type Result<T> = std::result::Result<T, BridgeError>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputRequest {
    pub version: u32,
    pub target_step: u64,
    pub controls: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CameraRequest {
    pub version: u32,
    pub zoom: f64,
    pub tilt: f64,
    pub shear: f64,
    pub origin: [f64; 2],
    pub transition_ticks: u32,
}
impl Default for CameraRequest {
    fn default() -> Self {
        Self {
            version: 1,
            zoom: 1.,
            tilt: 1.,
            shear: 0.,
            origin: [0., 0.],
            transition_ticks: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordRow {
    pub owner: PluginId,
    pub local: String,
    pub record: p2::Record,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryRow {
    pub owner: PluginId,
    pub history: p2::OwnerHistory,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub version: u32,
    pub logical_step: u64,
    pub core_tick: u64,
    pub presentation_tick: u64,
    pub gameplay_attached: bool,
    pub battle: BattleState,
    pub records: Vec<RecordRow>,
    pub scenes: Vec<p2::SceneData>,
    pub entities: Vec<p2::EntityData>,
    pub histories: Vec<HistoryRow>,
    pub camera_frames: Vec<CameraFrame>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrameDto {
    pub version: u32,
    pub core_tick: u64,
    pub presentation_tick: u64,
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub sha256: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveBundle {
    pub version: u32,
    pub content_binding: String,
    pub seed: u64,
    pub logical_step: u64,
    pub presentation_tick: u64,
    pub gameplay_attached: bool,
    pub view_family: ViewFamily,
    /// Exact canonical P2 save2 UTF-8, not reserialized ReadFrame.
    pub core_save2: String,
    /// Exact canonical P3 presentation save1 UTF-8.
    pub presentation_save1: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionInfo {
    pub version: u32,
    pub mode: ExecutionMode,
    pub content_binding: String,
    pub logical_step: u64,
    pub gameplay_attached: bool,
    pub core: p2::CoreDiscovery,
    pub view: ge4g_pentomino_view::ViewDiscovery,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Create {
        config: GameConfig,
        seed: u64,
        mode: ExecutionMode,
    },
    Describe {
        session: u64,
    },
    Step {
        session: u64,
        input: InputRequest,
    },
    Observe {
        session: u64,
    },
    Frame {
        session: u64,
    },
    Save {
        session: u64,
    },
    Restore {
        session: u64,
        save: SaveBundle,
    },
    Gameplay {
        session: u64,
        enabled: bool,
    },
    Camera {
        session: u64,
        request: CameraRequest,
    },
    View {
        session: u64,
        family: ViewFamily,
    },
    Close {
        session: u64,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub version: u32,
    pub command: Command,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    pub version: u32,
    pub ok: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<BridgeError>,
}

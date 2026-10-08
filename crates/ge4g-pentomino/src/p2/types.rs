use crate::{Bindings, CapabilityId, PluginId, Version};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidIdentifier,
    InvalidDescriptor,
    VersionMismatch,
    DuplicateId,
    MissingCapability,
    InvalidBinding,
    DependencyCycle,
    PermissionDenied,
    UndeclaredKey,
    BudgetExceeded,
    StaleHandle,
    DependencyInUse,
    InvalidTick,
    Overflow,
    PluginFailed,
    SaveMismatch,
    InvalidSave,
    InputAlreadyCommitted,
    InputOutOfSequence,
    InvalidInput,
    InvalidSchema,
    InvalidRecord,
    ReferenceInUse,
    InvalidReference,
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
        let mut end = detail.len().min(4096);
        while !detail.is_char_boundary(end) {
            end -= 1;
        }
        detail.truncate(end);
        Self { code, detail }
    }
    pub fn code(&self) -> ErrorCode {
        self.code
    }
    pub fn detail(&self) -> &str {
        &self.detail
    }
}
impl From<crate::Error> for Error {
    fn from(value: crate::Error) -> Self {
        use crate::ErrorCode as Old;
        let code = match value.code() {
            Old::InvalidIdentifier => ErrorCode::InvalidIdentifier,
            Old::InvalidDescriptor => ErrorCode::InvalidDescriptor,
            Old::VersionMismatch => ErrorCode::VersionMismatch,
            Old::DuplicateId => ErrorCode::DuplicateId,
            Old::MissingCapability => ErrorCode::MissingCapability,
            Old::InvalidBinding => ErrorCode::InvalidBinding,
            Old::DependencyCycle => ErrorCode::DependencyCycle,
            Old::PermissionDenied => ErrorCode::PermissionDenied,
            Old::UndeclaredKey => ErrorCode::UndeclaredKey,
            Old::BudgetExceeded => ErrorCode::BudgetExceeded,
            Old::StaleHandle => ErrorCode::StaleHandle,
            Old::DependencyInUse => ErrorCode::DependencyInUse,
            Old::InvalidTick => ErrorCode::InvalidTick,
            Old::Overflow => ErrorCode::Overflow,
            Old::PluginFailed => ErrorCode::PluginFailed,
            Old::SaveMismatch => ErrorCode::SaveMismatch,
            Old::InvalidSave => ErrorCode::InvalidSave,
        };
        Self::new(code, value.detail())
    }
}
pub(super) fn error(code: ErrorCode, detail: &str) -> Error {
    Error::new(code, detail)
}

macro_rules! identifier {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);
        impl $name {
            pub fn new(value: &str) -> Result<Self, Error> {
                if !crate::identifier(value) {
                    return Err(error(
                        ErrorCode::InvalidIdentifier,
                        "invalid dotted identifier",
                    ));
                }
                Ok(Self(value.to_owned()))
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = String::deserialize(deserializer)?;
                Self::new(&value).map_err(serde::de::Error::custom)
            }
        }
    };
}
identifier!(SchemaId);
identifier!(ActionId);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneRef {
    pub owner: PluginId,
    pub local: String,
    pub incarnation: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityRef {
    pub owner: PluginId,
    pub local: String,
    pub incarnation: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ObjectRef {
    Scene(SceneRef),
    Entity(EntityRef),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum RefKind {
    Scene,
    Entity,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum FieldType {
    Bool,
    I64 {
        min: i64,
        max: i64,
    },
    String {
        max_bytes: usize,
    },
    Bytes {
        max_bytes: usize,
    },
    Ref {
        kind: RefKind,
    },
    List {
        item: Box<FieldType>,
        max_items: usize,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Value {
    Bool(bool),
    I64(i64),
    String(String),
    Bytes(Vec<u8>),
    Ref(ObjectRef),
    List(Vec<Value>),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordSchema {
    pub fields: BTreeMap<String, FieldType>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub schema: SchemaId,
    pub fields: BTreeMap<String, Value>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordKey {
    pub owner: PluginId,
    pub local: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ActionType {
    Bool,
    I64 { min: i64, max: i64 },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ActionValue {
    Bool(bool),
    I64(i64),
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InputFrame {
    pub target_tick: u64,
    pub actions: Vec<(ActionId, ActionValue)>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreDescriptor {
    pub id: PluginId,
    pub release: Version,
    pub contract: Version,
    pub provides: BTreeMap<CapabilityId, Version>,
    pub requires: BTreeMap<CapabilityId, Version>,
    pub schemas: BTreeMap<SchemaId, RecordSchema>,
    pub event_kinds: BTreeMap<String, SchemaId>,
    pub actions: BTreeMap<ActionId, ActionType>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SceneData {
    pub reference: SceneRef,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityData {
    pub reference: EntityRef,
    pub scene: SceneRef,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreEvent {
    pub owner: PluginId,
    pub tick: u64,
    pub sequence: u64,
    pub ordinal: u64,
    pub kind: String,
    pub record: Record,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerHistory {
    pub emitted: u64,
    pub dropped: u64,
    pub history: Vec<CoreEvent>,
    pub pending: Vec<CoreEvent>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub owners: BTreeSet<PluginId>,
    pub schemas: BTreeSet<SchemaId>,
    pub include_objects: bool,
    pub include_history: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadFrame {
    pub tick: u64,
    pub records: BTreeMap<RecordKey, Record>,
    pub scenes: Vec<SceneData>,
    pub entities: Vec<EntityData>,
    pub histories: BTreeMap<PluginId, OwnerHistory>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreInstalled {
    pub descriptor: CoreDescriptor,
    pub bindings: Bindings,
    pub removal_blockers: Vec<PluginId>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreDiscovery {
    pub contract_version: Version,
    pub save_version: u32,
    pub scalar_compatibility_version: Version,
    pub limits: CoreLimits,
    pub tick: u64,
    pub plugins: Vec<CoreInstalled>,
    pub last_input: Option<InputFrame>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreLimits {
    pub max_plugins: usize,
    pub max_provides: usize,
    pub max_requires: usize,
    pub max_schemas: usize,
    pub max_fields: usize,
    pub max_event_kinds: usize,
    pub max_actions: usize,
    pub max_records: usize,
    pub max_scenes: usize,
    pub max_entities: usize,
    pub max_string_bytes: usize,
    pub max_bytes: usize,
    pub max_list_items: usize,
    pub max_depth: usize,
    pub max_record_bytes: usize,
    pub max_frame_actions: usize,
    pub max_id_bytes: usize,
    pub max_content_binding_bytes: usize,
    pub max_error_bytes: usize,
    pub max_events_per_commit: usize,
    pub max_pending_events: usize,
    pub max_history_events: usize,
    pub max_commands: usize,
    pub max_save_bytes: usize,
}
impl Default for CoreLimits {
    fn default() -> Self {
        Self {
            max_plugins: 16,
            max_provides: 32,
            max_requires: 32,
            max_schemas: 32,
            max_fields: 32,
            max_event_kinds: 32,
            max_actions: 32,
            max_records: 256,
            max_scenes: 64,
            max_entities: 256,
            max_string_bytes: 1024,
            max_bytes: 4096,
            max_list_items: 64,
            max_depth: 4,
            max_record_bytes: 65_536,
            max_frame_actions: 128,
            max_id_bytes: 128,
            max_content_binding_bytes: 128,
            max_error_bytes: 4096,
            max_events_per_commit: 128,
            max_pending_events: 128,
            max_history_events: 256,
            max_commands: 4096,
            max_save_bytes: 8_388_608,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreOwnerToken {
    pub(super) host: u64,
    pub(super) generation: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneHandle {
    pub(super) host: u64,
    pub(super) generation: u64,
    pub(super) reference: SceneRef,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityHandle {
    pub(super) host: u64,
    pub(super) generation: u64,
    pub(super) reference: EntityRef,
}

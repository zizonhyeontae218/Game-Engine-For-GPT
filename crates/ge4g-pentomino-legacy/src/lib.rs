//! Explicit, read-only Pentomino projection of the authoritative legacy runtime.
//!
//! The bridge advances the existing World, while imported Core plugins are immutable
//! snapshots. Importing or stepping those plugins does not execute legacy gameplay.
//! Presentation and legacy save/resume data remain entirely in the legacy runtime.

use ge4g_pentomino::p2::{
    ActionId, ActionType, ActionValue, CoreContext, CoreDescriptor, CoreHost, CoreOwnerToken,
    CorePlugin, CoreTransaction, EntityData, EntityRef, Error, FieldType, InputFrame, ObjectRef,
    Record, RecordKey, RecordSchema, RefKind, SceneData, SceneRef, SchemaId, Value,
};
use ge4g_pentomino::{Bindings, PluginId, Version};
use ge4g_project::Replay;
use ge4g_runtime::World;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const MAX_STRING: usize = 1024;
const MAX_RECORDS: usize = 256;
const MAX_ENTITIES: usize = 256;
const MAX_ACTIONS: usize = 32;
const MAX_TICK: u64 = 1_000_000;
const COMMON: [&str; 5] = ["left", "right", "up", "down", "interact"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BridgeErrorCode {
    CoreValidation,
    LegacyRuntime,
    ProjectionLimit,
    InvalidInput,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BridgeError {
    code: BridgeErrorCode,
    detail: String,
}
impl BridgeError {
    pub fn new(code: BridgeErrorCode, detail: impl Into<String>) -> Self {
        let mut detail = detail.into();
        let mut end = detail.len().min(4096);
        while !detail.is_char_boundary(end) {
            end -= 1;
        }
        detail.truncate(end);
        Self { code, detail }
    }
    pub fn code(&self) -> BridgeErrorCode {
        self.code
    }
    pub fn detail(&self) -> &str {
        &self.detail
    }
}
impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.detail)
    }
}
impl std::error::Error for BridgeError {}
impl From<Error> for BridgeError {
    fn from(error: Error) -> Self {
        Self::new(BridgeErrorCode::CoreValidation, error.to_string())
    }
}
impl From<ge4g_pentomino::Error> for BridgeError {
    fn from(error: ge4g_pentomino::Error) -> Self {
        Self::new(BridgeErrorCode::CoreValidation, error.to_string())
    }
}
fn invalid(detail: &str) -> BridgeError {
    BridgeError::new(BridgeErrorCode::CoreValidation, detail)
}
fn limit(detail: &str) -> BridgeError {
    BridgeError::new(BridgeErrorCode::ProjectionLimit, detail)
}
fn invalid_input(detail: &str) -> BridgeError {
    BridgeError::new(BridgeErrorCode::InvalidInput, detail)
}
fn bounded_string(source: &str) -> Result<(), BridgeError> {
    if source.len() > MAX_STRING {
        return Err(limit("source UTF-8 string exceeds 1024 bytes"));
    }
    Ok(())
}
fn key(prefix: &str, source: &str) -> String {
    format!("{prefix}{:x}", Sha256::digest(source.as_bytes()))
}
fn entity_key(scene: &str, entity: &str) -> String {
    let mut hash = Sha256::new();
    hash.update(scene.as_bytes());
    hash.update([0]);
    hash.update(entity.as_bytes());
    format!("e_{:x}", hash.finalize())
}
fn owner() -> Result<PluginId, BridgeError> {
    Ok(PluginId::new("legacy.bridge")?)
}
fn schema(name: &str) -> Result<SchemaId, BridgeError> {
    Ok(SchemaId::new(name)?)
}
fn string_type() -> FieldType {
    FieldType::String {
        max_bytes: MAX_STRING,
    }
}
fn descriptor() -> Result<CoreDescriptor, BridgeError> {
    let mut schemas = BTreeMap::new();
    let entries = [
        ("legacy.scene", vec![("id", string_type())]),
        (
            "legacy.entity",
            vec![
                ("id", string_type()),
                (
                    "scene",
                    FieldType::Ref {
                        kind: RefKind::Scene,
                    },
                ),
            ],
        ),
        (
            "legacy.state_bool",
            vec![("key", string_type()), ("value", FieldType::Bool)],
        ),
        (
            "legacy.state_i64",
            vec![
                ("key", string_type()),
                (
                    "value",
                    FieldType::I64 {
                        min: i64::MIN,
                        max: i64::MAX,
                    },
                ),
            ],
        ),
        (
            "legacy.state_string",
            vec![("key", string_type()), ("value", string_type())],
        ),
        (
            "legacy.snapshot",
            vec![(
                "tick",
                FieldType::I64 {
                    min: 0,
                    max: MAX_TICK as i64,
                },
            )],
        ),
    ];
    for (name, fields) in entries {
        schemas.insert(
            schema(name)?,
            RecordSchema {
                fields: fields.into_iter().map(|(k, v)| (k.into(), v)).collect(),
            },
        );
    }
    let mut actions = BTreeMap::new();
    for name in COMMON {
        actions.insert(ActionId::new(&format!("legacy.{name}"))?, ActionType::Bool);
    }
    for name in ["direction_x", "direction_y"] {
        actions.insert(
            ActionId::new(&format!("legacy.{name}"))?,
            ActionType::I64 {
                min: i64::MIN,
                max: i64::MAX,
            },
        );
    }
    Ok(CoreDescriptor {
        id: owner()?,
        release: Version {
            major: 0,
            minor: 3,
            patch: 0,
        },
        contract: Version {
            major: 2,
            minor: 0,
            patch: 0,
        },
        provides: BTreeMap::new(),
        requires: BTreeMap::new(),
        schemas,
        event_kinds: BTreeMap::new(),
        actions,
    })
}
fn record(name: &str, fields: Vec<(&str, Value)>) -> Result<Record, BridgeError> {
    Ok(Record {
        schema: schema(name)?,
        fields: fields
            .into_iter()
            .map(|(key, value)| (key.into(), value))
            .collect(),
    })
}

/// Owns legacy simulation authority; callers cannot mutate it through this bridge.
pub struct LegacyBridge {
    world: World,
}
/// Snapshot-scoped generic projection. Core installation allocates fresh identities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyProjection {
    pub owner: PluginId,
    pub tick: u64,
    pub descriptor: CoreDescriptor,
    pub scenes: Vec<SceneData>,
    pub entities: Vec<EntityData>,
    pub records: BTreeMap<RecordKey, Record>,
}
impl LegacyBridge {
    pub fn new(world: World) -> Result<Self, BridgeError> {
        project(&world)?;
        Ok(Self { world })
    }
    pub fn world(&self) -> &World {
        &self.world
    }
    pub fn projection(&self) -> Result<LegacyProjection, BridgeError> {
        project(&self.world)
    }
    /// Map this World's next replay input; replay commands remain legacy-only.
    pub fn input_frame(&self, replay: &Replay) -> Result<InputFrame, BridgeError> {
        replay
            .validate()
            .map_err(|e| invalid_input(&e.to_string()))?;
        let input = replay.input_at(self.world.tick);
        let buttons = replay.actions_at(self.world.tick);
        if buttons.len() > MAX_ACTIONS - 7 {
            return Err(limit("mapped input exceeds 32 declared actions"));
        }
        let mut actions = BTreeMap::new();
        for (name, value) in COMMON.into_iter().zip([
            input.left,
            input.right,
            input.up,
            input.down,
            input.interact,
        ]) {
            actions.insert(
                ActionId::new(&format!("legacy.{name}"))?,
                ActionValue::Bool(value),
            );
        }
        if let Some([x, y]) = input.direction {
            actions.insert(ActionId::new("legacy.direction_x")?, ActionValue::I64(x));
            actions.insert(ActionId::new("legacy.direction_y")?, ActionValue::I64(y));
        }
        for button in buttons {
            actions.insert(
                ActionId::new(&key("legacy.button_", &button))?,
                ActionValue::Bool(true),
            );
        }
        let target_tick = self
            .world
            .tick
            .checked_add(1)
            .ok_or_else(|| invalid_input("legacy target tick overflow"))?;
        Ok(InputFrame {
            target_tick,
            actions: actions.into_iter().collect(),
        })
    }
    /// Validate the resulting projection before committing any legacy advancement.
    pub fn step_replay(&mut self, replay: &Replay) -> Result<LegacyProjection, BridgeError> {
        replay
            .validate()
            .map_err(|e| invalid_input(&e.to_string()))?;
        let mut candidate = self.world.clone();
        candidate
            .step_replay(replay)
            .map_err(|e| BridgeError::new(BridgeErrorCode::LegacyRuntime, e.to_string()))?;
        let projection = project(&candidate)?;
        self.world = candidate;
        Ok(projection)
    }
    pub fn legacy_snapshot(&self) -> ge4g_core::Snapshot {
        self.world.snapshot()
    }
}
fn project(world: &World) -> Result<LegacyProjection, BridgeError> {
    if world.tick > MAX_TICK {
        return Err(limit("snapshot tick exceeds legacy limit"));
    }
    bounded_string(&world.scene)?;
    if !world.project.scenes.contains_key(&world.scene) {
        return Err(invalid("current legacy scene is absent from project"));
    }
    if world.entities.len() > MAX_ENTITIES
        || world.state.values.len() > MAX_RECORDS - 2
        || world.entities.len() > MAX_RECORDS - 2 - world.state.values.len()
    {
        return Err(limit("legacy projection exceeds object/record bounds"));
    }
    if world.state.values.len() != world.state.definitions.len() {
        return Err(invalid("legacy state definitions and values differ"));
    }
    let owner = owner()?;
    let scene = SceneRef {
        owner: owner.clone(),
        local: key("s_", &world.scene),
        incarnation: 1,
    };
    let mut records = BTreeMap::new();
    records.insert(
        RecordKey {
            owner: owner.clone(),
            local: scene.local.clone(),
        },
        record(
            "legacy.scene",
            vec![("id", Value::String(world.scene.clone()))],
        )?,
    );
    records.insert(
        RecordKey {
            owner: owner.clone(),
            local: "snapshot".into(),
        },
        record(
            "legacy.snapshot",
            vec![("tick", Value::I64(world.tick as i64))],
        )?,
    );
    let mut entities = Vec::new();
    for (source, entity) in &world.entities {
        bounded_string(source)?;
        if source != &entity.spec.id {
            return Err(invalid(
                "legacy entity map key differs from source identity",
            ));
        }
        let reference = EntityRef {
            owner: owner.clone(),
            local: entity_key(&world.scene, source),
            incarnation: 1,
        };
        records.insert(
            RecordKey {
                owner: owner.clone(),
                local: reference.local.clone(),
            },
            record(
                "legacy.entity",
                vec![
                    ("id", Value::String(source.clone())),
                    ("scene", Value::Ref(ObjectRef::Scene(scene.clone()))),
                ],
            )?,
        );
        entities.push(EntityData {
            reference,
            scene: scene.clone(),
        });
    }
    for (source, value) in &world.state.values {
        bounded_string(source)?;
        let definition = world
            .state
            .definitions
            .get(source)
            .ok_or_else(|| invalid("legacy state value has no declaration"))?;
        if !definition.accepts(value) {
            return Err(invalid("legacy state value violates its declared type"));
        }
        let (name, typed) = if let Some(v) = value.as_bool() {
            ("legacy.state_bool", Value::Bool(v))
        } else if let Some(v) = value.as_i64() {
            ("legacy.state_i64", Value::I64(v))
        } else if let Some(v) = value.as_str() {
            bounded_string(v)?;
            ("legacy.state_string", Value::String(v.into()))
        } else {
            return Err(invalid("legacy state value is not bool/i64/UTF-8 string"));
        };
        records.insert(
            RecordKey {
                owner: owner.clone(),
                local: key("v_", source),
            },
            record(
                name,
                vec![("key", Value::String(source.clone())), ("value", typed)],
            )?,
        );
    }
    entities.sort_by(|a, b| a.reference.cmp(&b.reference));
    Ok(LegacyProjection {
        owner,
        tick: world.tick,
        descriptor: descriptor()?,
        scenes: vec![SceneData { reference: scene }],
        entities,
        records,
    })
}

impl LegacyProjection {
    /// Construct a bounded linked snapshot plugin. Supplied Core actions are
    /// validated, declared exactly, and never interpreted as legacy commands.
    pub fn plugin(&self, input: &InputFrame) -> Result<Box<dyn CorePlugin>, BridgeError> {
        self.validate()?;
        let mut descriptor = self.descriptor.clone();
        if input.target_tick == 0 || input.actions.len() > MAX_ACTIONS {
            return Err(invalid_input("invalid mapped input tick/action count"));
        }
        let mut seen = BTreeSet::new();
        for (id, value) in &input.actions {
            if !seen.insert(id) {
                return Err(invalid_input("duplicate mapped action"));
            }
            if let Some(declared) = descriptor.actions.get(id) {
                if !matches!(
                    (declared, value),
                    (ActionType::Bool, ActionValue::Bool(_))
                        | (ActionType::I64 { .. }, ActionValue::I64(_))
                ) {
                    return Err(invalid_input("mapped action has wrong type"));
                }
            } else {
                let suffix = id
                    .as_str()
                    .strip_prefix("legacy.button_")
                    .ok_or_else(|| invalid_input("unknown mapped action"))?;
                if suffix.len() != 64
                    || !suffix
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    || !matches!(value, ActionValue::Bool(true))
                {
                    return Err(invalid_input("invalid mapped button ID/value"));
                }
                descriptor.actions.insert(id.clone(), ActionType::Bool);
            }
        }
        if descriptor.actions.len() > MAX_ACTIONS {
            return Err(limit("mapped action declarations exceed 32"));
        }
        Ok(Box::new(ImportPlugin {
            descriptor,
            projection: self.clone(),
        }))
    }
    pub fn install_into(
        &self,
        host: &mut CoreHost,
        input: &InputFrame,
    ) -> Result<CoreOwnerToken, BridgeError> {
        Ok(host.install(self.plugin(input)?, Bindings::new())?)
    }
    // Public fields permit forged projections. Check exact projected schemas,
    // metadata/key consistency and all refs before allocating/cloning the plugin.
    fn validate(&self) -> Result<(), BridgeError> {
        if self.owner != owner()? || self.descriptor != descriptor()? {
            return Err(invalid(
                "projection descriptor/owner is not the bridge contract",
            ));
        }
        if self.tick > MAX_TICK
            || self.scenes.len() != 1
            || self.entities.len() > MAX_ENTITIES
            || self.records.len() > MAX_RECORDS
        {
            return Err(limit("projection tick/object/record bounds exceeded"));
        }
        let scene = &self.scenes[0].reference;
        if scene.owner != self.owner || scene.incarnation != 1 {
            return Err(invalid("invalid snapshot scene reference"));
        }
        let scene_record = self
            .records
            .get(&RecordKey {
                owner: self.owner.clone(),
                local: scene.local.clone(),
            })
            .ok_or_else(|| invalid("missing scene record"))?;
        let source_scene = string_field(scene_record, "id")?;
        if scene.local != key("s_", source_scene) {
            return Err(invalid("scene identity hash mismatch"));
        }
        let mut expected = BTreeSet::from([scene.local.clone(), "snapshot".into()]);
        for entity in &self.entities {
            if entity.reference.owner != self.owner
                || entity.reference.incarnation != 1
                || entity.scene != *scene
                || !expected.insert(entity.reference.local.clone())
            {
                return Err(invalid("invalid or duplicate snapshot entity reference"));
            }
            let record = self
                .records
                .get(&RecordKey {
                    owner: self.owner.clone(),
                    local: entity.reference.local.clone(),
                })
                .ok_or_else(|| invalid("missing entity record"))?;
            let source = string_field(record, "id")?;
            if entity.reference.local != entity_key(source_scene, source)
                || record.fields.get("scene") != Some(&Value::Ref(ObjectRef::Scene(scene.clone())))
            {
                return Err(invalid("entity identity/reference mismatch"));
            }
        }
        for (record_key, record) in &self.records {
            if record_key.owner != self.owner {
                return Err(invalid("foreign record owner in legacy projection"));
            }
            let declaration = self
                .descriptor
                .schemas
                .get(&record.schema)
                .ok_or_else(|| invalid("undeclared projection schema"))?;
            if record.fields.len() != declaration.fields.len() {
                return Err(invalid("projection fields differ from schema"));
            }
            for (field, ty) in &declaration.fields {
                let value = record
                    .fields
                    .get(field)
                    .ok_or_else(|| invalid("missing projection field"))?;
                match (ty, value) {
                    (FieldType::Bool, Value::Bool(_)) => {}
                    (FieldType::I64 { min, max }, Value::I64(v)) if v >= min && v <= max => {}
                    (FieldType::String { .. }, Value::String(v)) => bounded_string(v)?,
                    (
                        FieldType::Ref {
                            kind: RefKind::Scene,
                        },
                        Value::Ref(ObjectRef::Scene(v)),
                    ) if v == scene => {}
                    _ => return Err(invalid("projection value violates schema")),
                }
            }
            match record.schema.as_str() {
                "legacy.scene" if record_key.local == scene.local => {}
                "legacy.entity"
                    if expected.contains(&record_key.local)
                        && record_key.local.starts_with("e_") => {}
                "legacy.snapshot"
                    if record_key.local == "snapshot"
                        && record.fields.get("tick") == Some(&Value::I64(self.tick as i64)) => {}
                "legacy.state_bool" | "legacy.state_i64" | "legacy.state_string" => {
                    if record_key.local != key("v_", string_field(record, "key")?)
                        || !expected.insert(record_key.local.clone())
                    {
                        return Err(invalid("state record identity hash mismatch"));
                    }
                }
                _ => return Err(invalid("projection record has wrong semantic role")),
            }
        }
        if expected.len() != self.records.len() {
            return Err(invalid("incomplete projection records"));
        }
        Ok(())
    }
}
fn string_field<'a>(record: &'a Record, key: &str) -> Result<&'a str, BridgeError> {
    match record.fields.get(key) {
        Some(Value::String(value)) => {
            bounded_string(value)?;
            Ok(value)
        }
        _ => Err(invalid("projection source identity is not a string")),
    }
}
struct ImportPlugin {
    descriptor: CoreDescriptor,
    projection: LegacyProjection,
}
impl CorePlugin for ImportPlugin {
    fn descriptor(&self) -> CoreDescriptor {
        self.descriptor.clone()
    }
    fn initialize(&self, _ctx: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error> {
        let mut scenes = BTreeMap::new();
        let mut entities = BTreeMap::new();
        for scene in &self.projection.scenes {
            scenes.insert(
                scene.reference.clone(),
                tx.create_scene(&scene.reference.local)?,
            );
        }
        for entity in &self.projection.entities {
            let scene = scenes.get(&entity.scene).ok_or_else(|| {
                Error::new(
                    ge4g_pentomino::p2::ErrorCode::InvalidReference,
                    "missing imported scene",
                )
            })?;
            entities.insert(
                entity.reference.clone(),
                tx.create_entity(&entity.reference.local, scene)?,
            );
        }
        for (key, record) in &self.projection.records {
            let mut remapped = record.clone();
            for value in remapped.fields.values_mut() {
                if let Value::Ref(reference) = value {
                    *reference = match reference {
                        ObjectRef::Scene(source) => {
                            ObjectRef::Scene(scenes.get(source).cloned().ok_or_else(|| {
                                Error::new(
                                    ge4g_pentomino::p2::ErrorCode::InvalidReference,
                                    "missing imported scene reference",
                                )
                            })?)
                        }
                        ObjectRef::Entity(source) => {
                            ObjectRef::Entity(entities.get(source).cloned().ok_or_else(|| {
                                Error::new(
                                    ge4g_pentomino::p2::ErrorCode::InvalidReference,
                                    "missing imported entity reference",
                                )
                            })?)
                        }
                    };
                }
            }
            tx.set(&key.local, remapped)?;
        }
        Ok(())
    }
    fn tick(&self, _ctx: &CoreContext, _tx: &mut CoreTransaction<'_>) -> Result<(), Error> {
        Ok(())
    }
}

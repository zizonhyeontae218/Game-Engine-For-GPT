use super::*;
use serde::Serialize;
use std::io::{self, Write};

/// Reject before extending output; count-only mode never allocates JSON output.
struct CappedWriter {
    limit: usize,
    count: usize,
    bytes: Option<Vec<u8>>,
    exceeded: bool,
}
impl Write for CappedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let Some(next) = self
            .count
            .checked_add(bytes.len())
            .filter(|count| *count <= self.limit)
        else {
            self.exceeded = true;
            return Err(io::Error::other("canonical byte limit exceeded"));
        };
        if let Some(output) = &mut self.bytes {
            output.extend_from_slice(bytes);
        }
        self.count = next;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn serialize<T: Serialize>(
    value: &T,
    limit: usize,
    output: bool,
    code: ErrorCode,
) -> Result<Vec<u8>, Error> {
    let mut writer = CappedWriter {
        limit,
        count: 0,
        bytes: output.then(Vec::new),
        exceeded: false,
    };
    let result = serde_json::to_writer(&mut writer, value);
    if writer.exceeded {
        return Err(error(code, "canonical byte limit exceeded"));
    }
    result.map_err(|e| Error::new(ErrorCode::InvalidSave, e.to_string()))?;
    Ok(writer.bytes.unwrap_or_default())
}
pub(super) fn count_record(record: &Record) -> Result<(), Error> {
    serialize(record, 65_536, false, ErrorCode::BudgetExceeded).map(|_| ())
}
pub(super) fn count_save<T: Serialize>(value: &T) -> Result<(), Error> {
    serialize(value, 8_388_608, false, ErrorCode::BudgetExceeded).map(|_| ())
}
pub(super) fn save_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    serialize(value, 8_388_608, true, ErrorCode::BudgetExceeded)
}

fn field_type(field: &FieldType, depth: usize) -> bool {
    if depth > 4 {
        return false;
    }
    match field {
        FieldType::Bool | FieldType::Ref { .. } => true,
        FieldType::I64 { min, max } => min <= max,
        FieldType::String { max_bytes } => (1..=1024).contains(max_bytes),
        FieldType::Bytes { max_bytes } => (1..=4096).contains(max_bytes),
        FieldType::List { item, max_items } => {
            (1..=64).contains(max_items) && field_type(item, depth + 1)
        }
    }
}
pub(super) fn descriptor(descriptor: &CoreDescriptor) -> Result<(), Error> {
    if descriptor.contract != version() {
        return Err(error(
            ErrorCode::VersionMismatch,
            "unsupported typed contract",
        ));
    }
    if descriptor.provides.len() > 32
        || descriptor.requires.len() > 32
        || descriptor.schemas.len() > 32
        || descriptor.event_kinds.len() > 32
        || descriptor.actions.len() > 32
    {
        return Err(error(
            ErrorCode::BudgetExceeded,
            "descriptor count limit exceeded",
        ));
    }
    for schema in descriptor.schemas.values() {
        if schema.fields.len() > 32
            || schema
                .fields
                .iter()
                .any(|(local, field)| !crate::local_key(local) || !field_type(field, 1))
        {
            return Err(error(
                ErrorCode::InvalidSchema,
                "invalid schema field or bound",
            ));
        }
    }
    for (kind, schema) in &descriptor.event_kinds {
        if !crate::local_key(kind) || !descriptor.schemas.contains_key(schema) {
            return Err(error(
                ErrorCode::InvalidSchema,
                "event kind must name a declared schema",
            ));
        }
    }
    if descriptor
        .actions
        .values()
        .any(|action| matches!(action, ActionType::I64 { min, max } if min > max))
    {
        return Err(error(ErrorCode::InvalidSchema, "invalid action range"));
    }
    Ok(())
}

fn value_shape<F: Fn(&ObjectRef) -> Result<(), Error>>(
    value: &Value,
    field: &FieldType,
    resolve: &F,
) -> Result<(), Error> {
    let valid = match (value, field) {
        (Value::Bool(_), FieldType::Bool) => true,
        (Value::I64(value), FieldType::I64 { min, max }) => value >= min && value <= max,
        (Value::String(value), FieldType::String { max_bytes }) => value.len() <= *max_bytes,
        (Value::Bytes(value), FieldType::Bytes { max_bytes }) => value.len() <= *max_bytes,
        (Value::Ref(reference), FieldType::Ref { kind }) => {
            if matches!(
                (reference, kind),
                (ObjectRef::Scene(_), RefKind::Scene) | (ObjectRef::Entity(_), RefKind::Entity)
            ) {
                resolve(reference)?;
                true
            } else {
                false
            }
        }
        (Value::List(values), FieldType::List { item, max_items }) => {
            if values.len() > *max_items {
                false
            } else {
                for value in values {
                    value_shape(value, item, resolve)?;
                }
                true
            }
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(error(
            ErrorCode::InvalidRecord,
            "record value does not match its field type or bound",
        ))
    }
}
pub(super) fn record<F: Fn(&ObjectRef) -> Result<(), Error>>(
    record: &Record,
    descriptor: &CoreDescriptor,
    resolve: F,
) -> Result<(), Error> {
    let schema = descriptor
        .schemas
        .get(&record.schema)
        .ok_or_else(|| error(ErrorCode::InvalidRecord, "schema is not declared by owner"))?;
    if record.fields.len() != schema.fields.len() || record.fields.keys().ne(schema.fields.keys()) {
        return Err(error(
            ErrorCode::InvalidRecord,
            "record fields must exactly match schema",
        ));
    }
    for (key, value) in &record.fields {
        value_shape(value, &schema.fields[key], &resolve)?;
    }
    count_record(record)
}
fn value_contains(value: &Value, target: &ObjectRef) -> bool {
    match value {
        Value::Ref(reference) => reference == target,
        Value::List(values) => values.iter().any(|value| value_contains(value, target)),
        _ => false,
    }
}
pub(super) fn contains(record: &Record, target: &ObjectRef) -> bool {
    record
        .fields
        .values()
        .any(|value| value_contains(value, target))
}

fn target_owner<'a>(
    own: &'a SavedPlugin,
    owners: &'a BTreeMap<PluginId, SavedPlugin>,
    provider: &PluginId,
) -> Result<&'a SavedPlugin, Error> {
    if !own.permits(provider) {
        return Err(error(
            ErrorCode::PermissionDenied,
            "reference owner is not bound",
        ));
    }
    if provider == &own.descriptor.id {
        Ok(own)
    } else {
        owners
            .get(provider)
            .ok_or_else(|| error(ErrorCode::InvalidReference, "reference owner is absent"))
    }
}

pub(super) fn resolve_scene(
    own: &SavedPlugin,
    owners: &BTreeMap<PluginId, SavedPlugin>,
    reference: &SceneRef,
) -> Result<(), Error> {
    if target_owner(own, owners, &reference.owner)?
        .scene(reference)
        .is_some()
    {
        Ok(())
    } else {
        Err(error(
            ErrorCode::InvalidReference,
            "scene does not target a live exact incarnation",
        ))
    }
}

pub(super) fn resolve(
    own: &SavedPlugin,
    owners: &BTreeMap<PluginId, SavedPlugin>,
    reference: &ObjectRef,
) -> Result<(), Error> {
    let provider = match reference {
        ObjectRef::Scene(reference) => &reference.owner,
        ObjectRef::Entity(reference) => &reference.owner,
    };
    let data = target_owner(own, owners, provider)?;
    let live = match reference {
        ObjectRef::Scene(reference) => data.scene(reference).is_some(),
        ObjectRef::Entity(reference) => data.entity(reference).is_some(),
    };
    if live {
        Ok(())
    } else {
        Err(error(
            ErrorCode::InvalidReference,
            "reference does not target a live exact incarnation",
        ))
    }
}

pub(super) fn bindings(
    descriptor: &CoreDescriptor,
    bindings: &Bindings,
    owners: &BTreeMap<PluginId, SavedPlugin>,
) -> Result<(), Error> {
    if descriptor.requires.len() != bindings.len() || descriptor.requires.keys().ne(bindings.keys())
    {
        return Err(error(
            ErrorCode::InvalidBinding,
            "bindings must exactly cover requirements",
        ));
    }
    for (capability, provider) in bindings {
        if provider == &descriptor.id {
            return Err(error(ErrorCode::DependencyCycle, "self binding"));
        }
        let owner = owners
            .get(provider)
            .ok_or_else(|| error(ErrorCode::MissingCapability, "provider is absent"))?;
        let offered = owner.descriptor.provides.get(capability).ok_or_else(|| {
            error(
                ErrorCode::MissingCapability,
                "provider does not offer capability",
            )
        })?;
        if offered != &descriptor.requires[capability] {
            return Err(error(
                ErrorCode::VersionMismatch,
                "capability version differs",
            ));
        }
    }
    Ok(())
}
pub(super) fn input(
    frame: &InputFrame,
    owners: &BTreeMap<PluginId, SavedPlugin>,
    sorted: bool,
) -> Result<(), Error> {
    if frame.actions.len() > 128 {
        return Err(error(ErrorCode::InvalidInput, "too many input actions"));
    }
    let mut seen = BTreeSet::new();
    let mut previous = None;
    for (id, value) in &frame.actions {
        if !seen.insert(id) || (sorted && previous.is_some_and(|prior| prior >= id)) {
            return Err(error(
                ErrorCode::InvalidInput,
                "actions must be unique and canonically sorted",
            ));
        }
        let declaration = owners
            .values()
            .find_map(|owner| owner.descriptor.actions.get(id))
            .ok_or_else(|| error(ErrorCode::InvalidInput, "action is undeclared"))?;
        let valid = match (value, declaration) {
            (ActionValue::Bool(_), ActionType::Bool) => true,
            (ActionValue::I64(value), ActionType::I64 { min, max }) => value >= min && value <= max,
            _ => false,
        };
        if !valid {
            return Err(error(
                ErrorCode::InvalidInput,
                "action type or range invalid",
            ));
        }
        previous = Some(id);
    }
    Ok(())
}
pub(super) fn order(owners: &BTreeMap<PluginId, SavedPlugin>) -> Result<Vec<PluginId>, Error> {
    let mut remaining: BTreeSet<_> = owners.keys().cloned().collect();
    let mut done = BTreeSet::new();
    let mut order = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        let next = remaining
            .iter()
            .find(|id| {
                owners[*id]
                    .bindings
                    .values()
                    .all(|provider| done.contains(provider))
            })
            .cloned()
            .ok_or_else(|| error(ErrorCode::DependencyCycle, "dependency cycle"))?;
        remaining.remove(&next);
        done.insert(next.clone());
        order.push(next);
    }
    Ok(order)
}

pub(super) fn state(state: &State) -> Result<(), Error> {
    if state.owners.len() > 16 || state.next_identity == 0 {
        return Err(error(
            ErrorCode::InvalidSave,
            "invalid owner/identity count",
        ));
    }
    let mut schemas = BTreeSet::new();
    let mut actions = BTreeSet::new();
    let mut identities = BTreeSet::new();
    let mut sequences = BTreeSet::new();
    let mut global_ticks = BTreeMap::new();
    let mut emitted = state.retired_emitted;
    if state.purged_retained > state.retired_emitted {
        return Err(error(
            ErrorCode::InvalidSave,
            "purged accounting exceeds retired events",
        ));
    }
    for (id, owner) in &state.owners {
        descriptor(&owner.descriptor)?;
        bindings(&owner.descriptor, &owner.bindings, &state.owners)?;
        if id != &owner.descriptor.id
            || owner
                .descriptor
                .schemas
                .keys()
                .any(|schema| !schemas.insert(schema.clone()))
            || owner
                .descriptor
                .actions
                .keys()
                .any(|action| !actions.insert(action.clone()))
        {
            return Err(error(
                ErrorCode::DuplicateId,
                "owner/schema/action identity collision",
            ));
        }
        if owner.records.len() > 256 || owner.scenes.len() > 64 || owner.entities.len() > 256 {
            return Err(error(
                ErrorCode::BudgetExceeded,
                "owned state count exceeds limit",
            ));
        }
        let mut scene_local = BTreeSet::new();
        let mut entity_local = BTreeSet::new();
        let mut previous_scene = None;
        for scene in &owner.scenes {
            let reference = &scene.reference;
            if reference.owner != *id
                || !crate::local_key(&reference.local)
                || reference.incarnation == 0
                || reference.incarnation >= state.next_identity
                || !identities.insert(reference.incarnation)
                || !scene_local.insert(&reference.local)
                || previous_scene.is_some_and(|previous| previous >= reference)
            {
                return Err(error(
                    ErrorCode::InvalidReference,
                    "invalid or duplicate live scene identity",
                ));
            }
            previous_scene = Some(reference);
        }
        let mut previous_entity = None;
        for entity in &owner.entities {
            let reference = &entity.reference;
            if reference.owner != *id
                || !crate::local_key(&reference.local)
                || reference.incarnation == 0
                || reference.incarnation >= state.next_identity
                || !identities.insert(reference.incarnation)
                || !entity_local.insert(&reference.local)
                || previous_entity.is_some_and(|previous| previous >= reference)
            {
                return Err(error(
                    ErrorCode::InvalidReference,
                    "invalid or duplicate live entity identity",
                ));
            }
            resolve_scene(owner, &state.owners, &entity.scene)?;
            previous_entity = Some(reference);
        }
        for (local, value) in &owner.records {
            if !crate::local_key(local) {
                return Err(error(ErrorCode::InvalidRecord, "invalid record key"));
            }
            record(value, &owner.descriptor, |reference| {
                resolve(owner, &state.owners, reference)
            })?;
        }
        let history = &owner.history;
        if history.history.len() != history.emitted.min(256) as usize
            || history.pending.len() > 128
            || history.dropped.checked_add(history.history.len() as u64) != Some(history.emitted)
        {
            return Err(error(
                ErrorCode::InvalidSave,
                "owned history bounds or accounting invalid",
            ));
        }
        emitted = emitted
            .checked_add(history.emitted)
            .ok_or_else(|| error(ErrorCode::Overflow, "global emitted accounting overflow"))?;
        let mut previous = None;
        for (offset, event) in history.history.iter().enumerate() {
            if event.owner != *id
                || event.tick > state.tick
                || event.sequence >= state.next_sequence
                || !sequences.insert(event.sequence)
                || event.ordinal >= history.emitted
                || history.dropped.checked_add(offset as u64) != Some(event.ordinal)
                || previous
                    .is_some_and(|(sequence, tick)| sequence >= event.sequence || tick > event.tick)
            {
                return Err(error(
                    ErrorCode::InvalidSave,
                    "owned history event identity/order/suffix invalid",
                ));
            }
            if owner.descriptor.event_kinds.get(&event.kind) != Some(&event.record.schema) {
                return Err(error(
                    ErrorCode::InvalidRecord,
                    "event schema/kind mismatch",
                ));
            }
            record(&event.record, &owner.descriptor, |reference| {
                resolve(owner, &state.owners, reference)
            })?;
            previous = Some((event.sequence, event.tick));
            global_ticks.insert(event.sequence, event.tick);
        }
        if history
            .history
            .iter()
            .filter(|event| event.tick == state.tick)
            .ne(history.pending.iter())
        {
            return Err(error(
                ErrorCode::InvalidSave,
                "pending is not complete current-tick history",
            ));
        }
    }
    if emitted != state.next_sequence {
        return Err(error(
            ErrorCode::InvalidSave,
            "global event accounting mismatch",
        ));
    }
    let mut previous_tick = None;
    for tick in global_ticks.values() {
        if previous_tick.is_some_and(|previous| previous > *tick) {
            return Err(error(
                ErrorCode::InvalidSave,
                "global event sequence regresses tick",
            ));
        }
        previous_tick = Some(*tick);
    }
    match (&state.last_input, state.tick) {
        (None, 0) => {}
        (Some(frame), tick) if tick > 0 && frame.target_tick == tick => {
            input(frame, &state.owners, true)?
        }
        _ => {
            return Err(error(
                ErrorCode::InvalidInput,
                "last input does not match committed tick",
            ));
        }
    }
    order(&state.owners)?;
    Ok(())
}

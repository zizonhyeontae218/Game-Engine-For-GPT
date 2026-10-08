//! Experimental deterministic scalar plugin host, independent of GE4G adapters.
//! Native callbacks are trusted code; command limits are not a native sandbox.

pub mod p2;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

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

fn err(code: ErrorCode, detail: &str) -> Error {
    Error::new(code, detail)
}
fn local_key(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
fn identifier(s: &str) -> bool {
    s.len() <= 128 && s.contains('.') && s.split('.').all(local_key)
}

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);
        impl $name {
            pub fn new(value: &str) -> Result<Self, Error> {
                if !identifier(value) {
                    return Err(err(
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
id_type!(PluginId);
id_type!(CapabilityId);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
fn contract_version() -> Version {
    Version {
        major: 1,
        minor: 0,
        patch: 0,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PluginDescriptor {
    pub id: PluginId,
    pub release: Version,
    pub contract: Version,
    pub provides: BTreeMap<CapabilityId, Version>,
    pub requires: BTreeMap<CapabilityId, Version>,
    pub resources: BTreeSet<String>,
    pub event_kinds: BTreeSet<String>,
}
pub type Bindings = BTreeMap<CapabilityId, PluginId>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnerToken {
    host: u64,
    generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub tick: u64,
    pub owner: PluginId,
    pub kind: String,
    pub value: i64,
    pub sequence: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub owner: PluginId,
    pub tick: u64,
    pub resources: BTreeMap<String, i64>,
    pub rng_state: u64,
    pub events: Vec<Event>,
    pub next_sequence: u64,
    pub events_dropped: u64,
    pub first_retained_sequence: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstalledPlugin {
    pub descriptor: PluginDescriptor,
    pub bindings: Bindings,
    pub removal_blockers: Vec<PluginId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Limits {
    pub max_plugins: usize,
    pub max_provides: usize,
    pub max_requires: usize,
    pub max_resources: usize,
    pub max_event_kinds: usize,
    pub max_id_bytes: usize,
    pub max_events_per_commit: usize,
    pub max_pending_events: usize,
    pub max_history_events: usize,
    pub max_commands: usize,
    pub max_error_bytes: usize,
    pub max_content_binding_bytes: usize,
    pub max_save_bytes: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_plugins: 16,
            max_provides: 32,
            max_requires: 32,
            max_resources: 64,
            max_event_kinds: 32,
            max_id_bytes: 128,
            max_events_per_commit: 128,
            max_pending_events: 128,
            max_history_events: 256,
            max_commands: 1024,
            max_error_bytes: 4096,
            max_content_binding_bytes: 128,
            max_save_bytes: 1_048_576,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Discovery {
    pub contract_version: Version,
    pub save_version: u32,
    pub limits: Limits,
    pub tick: u64,
    pub plugins: Vec<InstalledPlugin>,
}

pub trait Plugin {
    fn descriptor(&self) -> PluginDescriptor;
    fn initialize(&self, ctx: &Context, tx: &mut Transaction<'_>) -> Result<(), Error>;
    fn tick(&self, ctx: &Context, tx: &mut Transaction<'_>) -> Result<(), Error>;
}

/// Immutable reads from the state committed before a callback begins.
pub struct Context {
    tick: u64,
    own: PluginId,
    readable: BTreeSet<PluginId>,
    records: BTreeMap<PluginId, (BTreeSet<String>, BTreeMap<String, i64>)>,
    events: BTreeMap<PluginId, Vec<Event>>,
}
impl Context {
    pub fn tick(&self) -> u64 {
        self.tick
    }
    pub fn own(&self, key: &str) -> Result<Option<i64>, Error> {
        self.read(&self.own, key)
    }
    pub fn read(&self, provider: &PluginId, key: &str) -> Result<Option<i64>, Error> {
        if !self.readable.contains(provider) {
            return Err(err(ErrorCode::PermissionDenied, "provider is not bound"));
        }
        let (declared, records) = self
            .records
            .get(provider)
            .ok_or_else(|| err(ErrorCode::PermissionDenied, "provider is absent"))?;
        if !declared.contains(key) {
            return Err(err(ErrorCode::UndeclaredKey, "resource is not declared"));
        }
        Ok(records.get(key).copied())
    }
    pub fn events(&self, provider: &PluginId) -> Result<&[Event], Error> {
        if !self.readable.contains(provider) {
            return Err(err(ErrorCode::PermissionDenied, "provider is not bound"));
        }
        Ok(self.events.get(provider).map_or(&[], Vec::as_slice))
    }
}

#[derive(Clone)]
struct ScalarState {
    resources: BTreeMap<String, i64>,
    rng_state: u64,
}

/// Writes remain staged until the host accepts the entire callback/step.
pub struct Transaction<'a> {
    descriptor: &'a PluginDescriptor,
    state: &'a mut ScalarState,
    emitted: Vec<(String, i64)>,
    commands: usize,
    poison: Option<Error>,
}
impl<'a> Transaction<'a> {
    fn new(descriptor: &'a PluginDescriptor, state: &'a mut ScalarState) -> Self {
        Self {
            descriptor,
            state,
            emitted: Vec::new(),
            commands: 0,
            poison: None,
        }
    }
    fn deny<T>(&mut self, error: Error) -> Result<T, Error> {
        if self.poison.is_none() {
            self.poison = Some(error.clone());
        }
        Err(error)
    }
    fn spend(&mut self) -> Result<(), Error> {
        let Some(commands) = self.commands.checked_add(1) else {
            return self.deny(err(ErrorCode::Overflow, "command counter overflow"));
        };
        self.commands = commands;
        if let Some(error) = &self.poison {
            return Err(error.clone());
        }
        if self.commands > 1024 {
            return self.deny(err(
                ErrorCode::BudgetExceeded,
                "callback command budget exceeded",
            ));
        }
        Ok(())
    }
    pub fn set(&mut self, key: &str, value: i64) -> Result<(), Error> {
        self.spend()?;
        if !self.descriptor.resources.contains(key) {
            return self.deny(err(ErrorCode::UndeclaredKey, "resource is not declared"));
        }
        self.state.resources.insert(key.to_owned(), value);
        Ok(())
    }
    pub fn emit(&mut self, kind: &str, value: i64) -> Result<(), Error> {
        self.spend()?;
        if !self.descriptor.event_kinds.contains(kind) {
            return self.deny(err(ErrorCode::UndeclaredKey, "event kind is not declared"));
        }
        if self.emitted.len() >= 128 {
            return self.deny(err(
                ErrorCode::BudgetExceeded,
                "callback event budget exceeded",
            ));
        }
        self.emitted.push((kind.to_owned(), value));
        Ok(())
    }
    pub fn draw(&mut self) -> Result<u64, Error> {
        self.spend()?;
        self.state.rng_state = self.state.rng_state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state.rng_state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        Ok(z ^ (z >> 31))
    }
    fn finish(self, result: Result<(), Error>) -> Result<Vec<(String, i64)>, Error> {
        if let Some(error) = self.poison {
            return Err(error);
        }
        result?;
        Ok(self.emitted)
    }
}

struct Entry {
    plugin: Box<dyn Plugin>,
    descriptor: PluginDescriptor,
    bindings: Bindings,
    state: ScalarState,
    generation: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedPlugin {
    descriptor: PluginDescriptor,
    bindings: Bindings,
    resources: BTreeMap<String, i64>,
    rng_state: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedHost {
    format_version: u32,
    contract_version: Version,
    content_binding: String,
    seed: u64,
    tick: u64,
    plugins: Vec<SavedPlugin>,
    history: Vec<Event>,
    pending: Vec<Event>,
    next_sequence: u64,
    events_dropped: u64,
}

static HOST_NONCE: AtomicU64 = AtomicU64::new(1);

fn reserve_host_nonce() -> Result<u64, Error> {
    let mut current = HOST_NONCE.load(Ordering::Relaxed);
    loop {
        let next = current
            .checked_add(1)
            .ok_or_else(|| err(ErrorCode::Overflow, "host identity exhausted"))?;
        match HOST_NONCE.compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => return Ok(current),
            Err(actual) => current = actual,
        }
    }
}

/// Host-installed linked plugins only. No game package executes native code.
pub struct Host {
    nonce: u64,
    next_generation: u64,
    seed: u64,
    content_binding: String,
    tick: u64,
    plugins: BTreeMap<PluginId, Entry>,
    history: Vec<Event>,
    pending: Vec<Event>,
    next_sequence: u64,
    events_dropped: u64,
}

impl Host {
    pub fn new(seed: u64, content_binding: &str) -> Result<Self, Error> {
        if content_binding.is_empty()
            || content_binding.len() > 128
            || !content_binding
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.:-".contains(&b))
        {
            return Err(err(ErrorCode::InvalidIdentifier, "invalid content binding"));
        }
        let nonce = reserve_host_nonce()?;
        Ok(Self {
            nonce,
            next_generation: 1,
            seed,
            content_binding: content_binding.to_owned(),
            tick: 0,
            plugins: BTreeMap::new(),
            history: Vec::new(),
            pending: Vec::new(),
            next_sequence: 0,
            events_dropped: 0,
        })
    }

    fn validate_descriptor(descriptor: &PluginDescriptor) -> Result<(), Error> {
        if descriptor.contract != contract_version() {
            return Err(err(
                ErrorCode::VersionMismatch,
                "unsupported plugin contract",
            ));
        }
        if descriptor.provides.len() > 32
            || descriptor.requires.len() > 32
            || descriptor.resources.len() > 64
            || descriptor.event_kinds.len() > 32
        {
            return Err(err(ErrorCode::BudgetExceeded, "descriptor limits exceeded"));
        }
        if descriptor
            .resources
            .iter()
            .chain(&descriptor.event_kinds)
            .any(|s| !local_key(s))
        {
            return Err(err(
                ErrorCode::InvalidDescriptor,
                "invalid local declaration",
            ));
        }
        Ok(())
    }

    fn validate_bindings(
        &self,
        descriptor: &PluginDescriptor,
        bindings: &Bindings,
    ) -> Result<(), Error> {
        if descriptor.requires.len() != bindings.len()
            || descriptor.requires.keys().ne(bindings.keys())
        {
            return Err(err(
                ErrorCode::InvalidBinding,
                "bindings must exactly cover requirements",
            ));
        }
        for (capability, provider) in bindings {
            if provider == &descriptor.id {
                return Err(err(ErrorCode::DependencyCycle, "self dependency"));
            }
            let entry = self
                .plugins
                .get(provider)
                .ok_or_else(|| err(ErrorCode::MissingCapability, "provider is not installed"))?;
            let offered = entry.descriptor.provides.get(capability).ok_or_else(|| {
                err(
                    ErrorCode::MissingCapability,
                    "provider does not offer capability",
                )
            })?;
            if offered != &descriptor.requires[capability] {
                return Err(err(
                    ErrorCode::VersionMismatch,
                    "capability version differs",
                ));
            }
        }
        Ok(())
    }

    fn context(&self, descriptor: &PluginDescriptor, bindings: &Bindings, tick: u64) -> Context {
        let mut readable: BTreeSet<_> = bindings.values().cloned().collect();
        readable.insert(descriptor.id.clone());
        let mut records = BTreeMap::new();
        let mut events = BTreeMap::new();
        for id in &readable {
            if let Some(entry) = self.plugins.get(id) {
                records.insert(
                    id.clone(),
                    (
                        entry.descriptor.resources.clone(),
                        entry.state.resources.clone(),
                    ),
                );
            }
            events.insert(
                id.clone(),
                self.pending
                    .iter()
                    .filter(|e| &e.owner == id)
                    .cloned()
                    .collect(),
            );
        }
        // Initialization has no previously committed own records.
        records
            .entry(descriptor.id.clone())
            .or_insert_with(|| (descriptor.resources.clone(), BTreeMap::new()));
        Context {
            tick,
            own: descriptor.id.clone(),
            readable,
            records,
            events,
        }
    }

    fn reserve_generation(&mut self) -> Result<u64, Error> {
        let generation = self.next_generation;
        self.next_generation = generation
            .checked_add(1)
            .ok_or_else(|| err(ErrorCode::Overflow, "owner generation exhausted"))?;
        Ok(generation)
    }

    pub fn install(
        &mut self,
        plugin: Box<dyn Plugin>,
        bindings: Bindings,
    ) -> Result<OwnerToken, Error> {
        let descriptor = plugin.descriptor();
        Self::validate_descriptor(&descriptor)?;
        if self.plugins.contains_key(&descriptor.id) {
            return Err(err(ErrorCode::DuplicateId, "plugin is already installed"));
        }
        if self.plugins.len() >= 16 {
            return Err(err(
                ErrorCode::BudgetExceeded,
                "installed plugin limit exceeded",
            ));
        }
        self.validate_bindings(&descriptor, &bindings)?;
        let generation = self.reserve_generation()?;
        let mut digest = Sha256::new();
        digest.update(self.seed.to_le_bytes());
        digest.update(descriptor.id.as_str().as_bytes());
        let digest = digest.finalize();
        let mut prefix = [0; 8];
        prefix.copy_from_slice(&digest[..8]);
        let mut state = ScalarState {
            resources: BTreeMap::new(),
            rng_state: u64::from_le_bytes(prefix),
        };
        let ctx = self.context(&descriptor, &bindings, self.tick);
        let mut tx = Transaction::new(&descriptor, &mut state);
        let result = plugin.initialize(&ctx, &mut tx);
        let emitted = tx.finish(result)?;
        let events = emitted
            .into_iter()
            .map(|(kind, value)| (descriptor.id.clone(), kind, value))
            .collect();
        let committed = self.stage_events(events, self.tick, true)?;
        self.plugins.insert(
            descriptor.id.clone(),
            Entry {
                plugin,
                descriptor,
                bindings,
                state,
                generation,
            },
        );
        self.commit_events(committed);
        Ok(OwnerToken {
            host: self.nonce,
            generation,
        })
    }

    fn live_id(&self, owner: OwnerToken) -> Result<&PluginId, Error> {
        if owner.host != self.nonce {
            return Err(err(ErrorCode::StaleHandle, "token belongs to another host"));
        }
        self.plugins
            .iter()
            .find(|(_, entry)| entry.generation == owner.generation)
            .map(|(id, _)| id)
            .ok_or_else(|| err(ErrorCode::StaleHandle, "owner token is no longer live"))
    }

    fn blockers(&self, id: &PluginId) -> Vec<PluginId> {
        self.plugins
            .iter()
            .filter(|(_, entry)| entry.bindings.values().any(|provider| provider == id))
            .map(|(other, _)| other.clone())
            .collect()
    }

    pub fn remove(&mut self, owner: OwnerToken) -> Result<(), Error> {
        let id = self.live_id(owner)?.clone();
        if !self.blockers(&id).is_empty() {
            return Err(err(
                ErrorCode::DependencyInUse,
                "dependent plugins are installed",
            ));
        }
        let removed = self
            .history
            .iter()
            .filter(|event| event.owner == id)
            .count();
        let dropped = self
            .events_dropped
            .checked_add(removed as u64)
            .ok_or_else(|| err(ErrorCode::Overflow, "event accounting overflow"))?;
        self.plugins.remove(&id);
        self.history.retain(|event| event.owner != id);
        self.pending.retain(|event| event.owner != id);
        self.events_dropped = dropped;
        Ok(())
    }

    fn tick_order(&self) -> Result<Vec<PluginId>, Error> {
        let mut remaining: BTreeSet<_> = self.plugins.keys().cloned().collect();
        let mut done = BTreeSet::new();
        let mut order = Vec::with_capacity(remaining.len());
        while !remaining.is_empty() {
            let next = remaining
                .iter()
                .find(|id| {
                    self.plugins[*id]
                        .bindings
                        .values()
                        .all(|provider| done.contains(provider))
                })
                .cloned()
                .ok_or_else(|| {
                    err(
                        ErrorCode::DependencyCycle,
                        "dependency graph contains a cycle",
                    )
                })?;
            remaining.remove(&next);
            done.insert(next.clone());
            order.push(next);
        }
        Ok(order)
    }

    pub fn step(&mut self, target_tick: u64) -> Result<(), Error> {
        let expected = self
            .tick
            .checked_add(1)
            .ok_or_else(|| err(ErrorCode::Overflow, "tick exhausted"))?;
        if target_tick != expected {
            return Err(err(ErrorCode::InvalidTick, "target must be the next tick"));
        }
        let mut staged = BTreeMap::new();
        let mut events = Vec::new();
        for id in self.tick_order()? {
            let entry = &self.plugins[&id];
            let ctx = self.context(&entry.descriptor, &entry.bindings, target_tick);
            let mut state = entry.state.clone();
            let mut tx = Transaction::new(&entry.descriptor, &mut state);
            let result = entry.plugin.tick(&ctx, &mut tx);
            for (kind, value) in tx.finish(result)? {
                events.push((id.clone(), kind, value));
            }
            if events.len() > 128 {
                return Err(err(ErrorCode::BudgetExceeded, "step event budget exceeded"));
            }
            staged.insert(id, state);
        }
        let committed = self.stage_events(events, target_tick, false)?;
        for (id, state) in staged {
            self.plugins
                .get_mut(&id)
                .expect("staged installed owner")
                .state = state;
        }
        self.tick = target_tick;
        self.commit_events(committed);
        Ok(())
    }

    pub fn observe(&self, owner: OwnerToken) -> Result<Observation, Error> {
        let id = self.live_id(owner)?;
        let state = &self.plugins[id].state;
        Ok(Observation {
            owner: id.clone(),
            tick: self.tick,
            resources: state.resources.clone(),
            rng_state: state.rng_state,
            events: self
                .history
                .iter()
                .filter(|e| &e.owner == id)
                .cloned()
                .collect(),
            next_sequence: self.next_sequence,
            events_dropped: self.events_dropped,
            first_retained_sequence: self
                .history
                .first()
                .map_or(self.next_sequence, |e| e.sequence),
        })
    }

    pub fn owner(&self, id: &PluginId) -> Result<OwnerToken, Error> {
        self.plugins
            .get(id)
            .map(|entry| OwnerToken {
                host: self.nonce,
                generation: entry.generation,
            })
            .ok_or_else(|| err(ErrorCode::StaleHandle, "plugin is not installed"))
    }

    pub fn describe(&self) -> Discovery {
        Discovery {
            contract_version: contract_version(),
            save_version: 1,
            limits: Limits::default(),
            tick: self.tick,
            plugins: self
                .plugins
                .iter()
                .map(|(id, entry)| InstalledPlugin {
                    descriptor: entry.descriptor.clone(),
                    bindings: entry.bindings.clone(),
                    removal_blockers: self.blockers(id),
                })
                .collect(),
        }
    }

    fn snapshot(&self) -> SavedHost {
        SavedHost {
            format_version: 1,
            contract_version: contract_version(),
            content_binding: self.content_binding.clone(),
            seed: self.seed,
            tick: self.tick,
            plugins: self
                .plugins
                .values()
                .map(|entry| SavedPlugin {
                    descriptor: entry.descriptor.clone(),
                    bindings: entry.bindings.clone(),
                    resources: entry.state.resources.clone(),
                    rng_state: entry.state.rng_state,
                })
                .collect(),
            history: self.history.clone(),
            pending: self.pending.clone(),
            next_sequence: self.next_sequence,
            events_dropped: self.events_dropped,
        }
    }

    pub fn save(&self) -> Result<Vec<u8>, Error> {
        let bytes = serde_json::to_vec(&self.snapshot())
            .map_err(|e| Error::new(ErrorCode::InvalidSave, e.to_string()))?;
        if bytes.len() > 1_048_576 {
            return Err(err(ErrorCode::BudgetExceeded, "save exceeds byte limit"));
        }
        Ok(bytes)
    }

    pub fn hash(&self) -> Result<String, Error> {
        Ok(format!("{:x}", Sha256::digest(self.save()?)))
    }

    pub fn restore(&mut self, bytes: &[u8]) -> Result<(), Error> {
        if bytes.len() > 1_048_576 {
            return Err(err(ErrorCode::InvalidSave, "save exceeds byte limit"));
        }
        let saved: SavedHost = serde_json::from_slice(bytes)
            .map_err(|e| Error::new(ErrorCode::InvalidSave, e.to_string()))?;
        let encoded = serde_json::to_vec(&saved)
            .map_err(|e| Error::new(ErrorCode::InvalidSave, e.to_string()))?;
        if encoded != bytes {
            return Err(err(ErrorCode::InvalidSave, "save is not canonical"));
        }
        self.validate_saved(&saved)?;
        let end = self
            .next_generation
            .checked_add(saved.plugins.len() as u64)
            .ok_or_else(|| err(ErrorCode::Overflow, "owner generation exhausted"))?;
        let generations = (0..saved.plugins.len())
            .map(|offset| {
                self.next_generation
                    .checked_add(offset as u64)
                    .ok_or_else(|| err(ErrorCode::Overflow, "owner generation exhausted"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (record, generation) in saved.plugins.into_iter().zip(generations) {
            let entry = self
                .plugins
                .get_mut(&record.descriptor.id)
                .expect("validated installed owner");
            entry.state = ScalarState {
                resources: record.resources,
                rng_state: record.rng_state,
            };
            entry.generation = generation;
        }
        self.next_generation = end;
        self.tick = saved.tick;
        self.history = saved.history;
        self.pending = saved.pending;
        self.next_sequence = saved.next_sequence;
        self.events_dropped = saved.events_dropped;
        Ok(())
    }

    fn validate_saved(&self, saved: &SavedHost) -> Result<(), Error> {
        if saved.format_version != 1
            || saved.contract_version != contract_version()
            || saved.content_binding != self.content_binding
            || saved.seed != self.seed
        {
            return Err(err(
                ErrorCode::SaveMismatch,
                "save version, content binding or seed differs",
            ));
        }
        if saved.plugins.len() > 16
            || saved
                .plugins
                .windows(2)
                .any(|pair| pair[0].descriptor.id >= pair[1].descriptor.id)
        {
            return Err(err(
                ErrorCode::InvalidSave,
                "plugin records are excessive, duplicate or unsorted",
            ));
        }
        if saved.plugins.len() != self.plugins.len() {
            return Err(err(ErrorCode::SaveMismatch, "installed plugin set differs"));
        }
        for (record, (id, installed)) in saved.plugins.iter().zip(&self.plugins) {
            if &record.descriptor.id != id
                || record.descriptor != installed.descriptor
                || record.bindings != installed.bindings
            {
                return Err(err(
                    ErrorCode::SaveMismatch,
                    "plugin descriptor or binding differs",
                ));
            }
            if record.resources.len() > 64
                || record
                    .resources
                    .keys()
                    .any(|key| !record.descriptor.resources.contains(key))
            {
                return Err(err(
                    ErrorCode::InvalidSave,
                    "undeclared or excessive resource records",
                ));
            }
        }
        if saved.history.len() > 256 || saved.pending.len() > 128 {
            return Err(err(ErrorCode::InvalidSave, "event bounds exceeded"));
        }
        let accounted = saved
            .events_dropped
            .checked_add(saved.history.len() as u64)
            .ok_or_else(|| err(ErrorCode::InvalidSave, "event accounting overflow"))?;
        if accounted != saved.next_sequence {
            return Err(err(ErrorCode::InvalidSave, "event accounting differs"));
        }
        let mut previous = None;
        for event in &saved.history {
            if event.tick > saved.tick
                || event.sequence >= saved.next_sequence
                || previous
                    .is_some_and(|(sequence, tick)| sequence >= event.sequence || tick > event.tick)
            {
                return Err(err(
                    ErrorCode::InvalidSave,
                    "history ordering or tick invalid",
                ));
            }
            let entry = self
                .plugins
                .get(&event.owner)
                .ok_or_else(|| err(ErrorCode::InvalidSave, "event owner is absent"))?;
            if !entry.descriptor.event_kinds.contains(&event.kind) {
                return Err(err(ErrorCode::InvalidSave, "event kind is undeclared"));
            }
            previous = Some((event.sequence, event.tick));
        }
        if saved
            .history
            .iter()
            .filter(|event| event.tick == saved.tick)
            .ne(saved.pending.iter())
        {
            return Err(err(
                ErrorCode::InvalidSave,
                "pending events differ from the complete current-tick history",
            ));
        }
        Ok(())
    }

    fn stage_events(
        &self,
        produced: Vec<(PluginId, String, i64)>,
        tick: u64,
        append_pending: bool,
    ) -> Result<EventCommit, Error> {
        let pending_before = if append_pending {
            self.pending.len()
        } else {
            0
        };
        if produced.len() > 128
            || pending_before
                .checked_add(produced.len())
                .is_none_or(|count| count > 128)
        {
            return Err(err(
                ErrorCode::BudgetExceeded,
                "commit or pending event limit exceeded",
            ));
        }
        let next_sequence = self
            .next_sequence
            .checked_add(produced.len() as u64)
            .ok_or_else(|| err(ErrorCode::Overflow, "event sequence exhausted"))?;
        let mut history = self.history.clone();
        let mut pending = if append_pending {
            self.pending.clone()
        } else {
            Vec::new()
        };
        for (offset, (owner, kind, value)) in produced.into_iter().enumerate() {
            let event = Event {
                tick,
                owner,
                kind,
                value,
                sequence: self.next_sequence + offset as u64,
            };
            history.push(event.clone());
            pending.push(event);
        }
        let removed = history.len().saturating_sub(256);
        let events_dropped = self
            .events_dropped
            .checked_add(removed as u64)
            .ok_or_else(|| err(ErrorCode::Overflow, "event dropped accounting exhausted"))?;
        history.drain(..removed);
        Ok(EventCommit {
            history,
            pending,
            next_sequence,
            events_dropped,
        })
    }

    fn commit_events(&mut self, events: EventCommit) {
        self.history = events.history;
        self.pending = events.pending;
        self.next_sequence = events.next_sequence;
        self.events_dropped = events.events_dropped;
    }
}

struct EventCommit {
    history: Vec<Event>,
    pending: Vec<Event>,
    next_sequence: u64,
    events_dropped: u64,
}

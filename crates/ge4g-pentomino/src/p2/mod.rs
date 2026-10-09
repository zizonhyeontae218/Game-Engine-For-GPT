//! Typed Tiny Core. Linked callbacks are trusted deterministic code, not sandboxed.
//! Runtime handles identify a committed host timeline; serialized references are data.
mod host;
mod transaction;
mod types;
mod validation;

use crate::{Bindings, PluginId, Version};
pub use host::CoreHost;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
pub use transaction::CoreTransaction;
use types::error;
pub use types::*;

pub trait CorePlugin {
    fn descriptor(&self) -> CoreDescriptor;
    fn initialize(&self, ctx: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error>;
    fn tick(&self, ctx: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error>;
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedPlugin {
    descriptor: CoreDescriptor,
    bindings: Bindings,
    records: BTreeMap<String, Record>,
    scenes: Vec<SceneData>,
    entities: Vec<EntityData>,
    rng_state: u64,
    history: OwnerHistory,
}
#[derive(Clone)]
struct State {
    tick: u64,
    next_identity: u64,
    last_input: Option<InputFrame>,
    owners: BTreeMap<PluginId, SavedPlugin>,
    next_sequence: u64,
    retired_emitted: u64,
    purged_retained: u64,
}

impl SavedPlugin {
    fn scene(&self, reference: &SceneRef) -> Option<&SceneData> {
        self.scenes
            .iter()
            .find(|scene| scene.reference == *reference)
    }
    fn entity(&self, reference: &EntityRef) -> Option<&EntityData> {
        self.entities
            .iter()
            .find(|entity| entity.reference == *reference)
    }
    fn permits(&self, owner: &PluginId) -> bool {
        self.descriptor.id == *owner || self.bindings.values().any(|provider| provider == owner)
    }
}

/// Snapshot of committed data with only the caller's bound provider namespaces.
pub struct CoreContext {
    tick: u64,
    own: PluginId,
    owners: BTreeMap<PluginId, SavedPlugin>,
    declared_actions: BTreeSet<ActionId>,
    actions: BTreeMap<ActionId, ActionValue>,
}
impl CoreContext {
    pub fn tick(&self) -> u64 {
        self.tick
    }
    pub fn own(&self, local: &str) -> Result<Option<&Record>, Error> {
        self.read(&self.own, local)
    }
    pub fn read(&self, owner: &PluginId, local: &str) -> Result<Option<&Record>, Error> {
        let data = self
            .owners
            .get(owner)
            .ok_or_else(|| error(ErrorCode::PermissionDenied, "owner is not bound"))?;
        if !crate::local_key(local) {
            return Err(error(ErrorCode::InvalidIdentifier, "invalid record key"));
        }
        Ok(data.records.get(local))
    }
    pub fn scene(&self, reference: &SceneRef) -> Result<&SceneData, Error> {
        self.owners
            .get(&reference.owner)
            .ok_or_else(|| error(ErrorCode::PermissionDenied, "scene owner is not bound"))?
            .scene(reference)
            .ok_or_else(|| error(ErrorCode::StaleHandle, "scene is not live"))
    }
    pub fn entity(&self, reference: &EntityRef) -> Result<&EntityData, Error> {
        self.owners
            .get(&reference.owner)
            .ok_or_else(|| error(ErrorCode::PermissionDenied, "entity owner is not bound"))?
            .entity(reference)
            .ok_or_else(|| error(ErrorCode::StaleHandle, "entity is not live"))
    }
    pub fn events(&self, owner: &PluginId) -> Result<&[CoreEvent], Error> {
        Ok(&self
            .owners
            .get(owner)
            .ok_or_else(|| error(ErrorCode::PermissionDenied, "event owner is not bound"))?
            .history
            .pending)
    }
    pub fn action(&self, action: &ActionId) -> Result<Option<&ActionValue>, Error> {
        if !self.declared_actions.contains(action) {
            return Err(error(
                ErrorCode::PermissionDenied,
                "action is not declared by caller",
            ));
        }
        Ok(self.actions.get(action))
    }
}

fn version() -> Version {
    Version {
        major: 2,
        minor: 0,
        patch: 0,
    }
}

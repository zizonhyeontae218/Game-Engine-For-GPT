use super::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

struct Entry {
    plugin: Box<dyn CorePlugin>,
    generation: u64,
}

#[derive(Serialize)]
struct Envelope<'a> {
    format_version: u32,
    contract_version: Version,
    content_binding: &'a str,
    seed: u64,
    tick: u64,
    next_identity: u64,
    last_input: &'a Option<InputFrame>,
    plugins: Vec<&'a SavedPlugin>,
    next_sequence: u64,
    retired_emitted: u64,
    purged_retained: u64,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SavedEnvelope {
    format_version: u32,
    contract_version: Version,
    content_binding: String,
    seed: u64,
    tick: u64,
    next_identity: u64,
    last_input: Option<InputFrame>,
    plugins: Vec<SavedPlugin>,
    next_sequence: u64,
    retired_emitted: u64,
    purged_retained: u64,
}

/// Owns all authoritative game data. Existing scalar/legacy hosts remain separate.
pub struct CoreHost {
    nonce: u64,
    next_generation: u64,
    seed: u64,
    content_binding: String,
    plugins: BTreeMap<PluginId, Entry>,
    state: State,
}
impl CoreHost {
    pub fn new(seed: u64, content_binding: &str) -> Result<Self, Error> {
        if content_binding.is_empty()
            || content_binding.len() > 128
            || !content_binding
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.:-".contains(&b))
        {
            return Err(error(
                ErrorCode::InvalidIdentifier,
                "invalid content binding",
            ));
        }
        Ok(Self {
            nonce: crate::reserve_host_nonce()?,
            next_generation: 1,
            seed,
            content_binding: content_binding.to_owned(),
            plugins: BTreeMap::new(),
            state: State {
                tick: 0,
                next_identity: 1,
                last_input: None,
                owners: BTreeMap::new(),
                next_sequence: 0,
                retired_emitted: 0,
                purged_retained: 0,
            },
        })
    }
    fn envelope<'a>(&'a self, state: &'a State) -> Envelope<'a> {
        Envelope {
            format_version: 2,
            contract_version: version(),
            content_binding: &self.content_binding,
            seed: self.seed,
            tick: state.tick,
            next_identity: state.next_identity,
            last_input: &state.last_input,
            plugins: state.owners.values().collect(),
            next_sequence: state.next_sequence,
            retired_emitted: state.retired_emitted,
            purged_retained: state.purged_retained,
        }
    }
    fn validate_commit(&self, candidate: &State) -> Result<(), Error> {
        validation::state(candidate)?;
        validation::count_save(&self.envelope(candidate))
    }
    fn generation(&mut self) -> Result<u64, Error> {
        let current = self.next_generation;
        self.next_generation = current
            .checked_add(1)
            .ok_or_else(|| error(ErrorCode::Overflow, "runtime generation exhausted"))?;
        Ok(current)
    }
    fn context(&self, own: &SavedPlugin, tick: u64, input: Option<&InputFrame>) -> CoreContext {
        let mut owners: BTreeMap<_, _> = self
            .state
            .owners
            .iter()
            .filter(|(id, _)| own.permits(id))
            .map(|(id, data)| (id.clone(), data.clone()))
            .collect();
        owners
            .entry(own.descriptor.id.clone())
            .or_insert_with(|| own.clone());
        let actions = input
            .into_iter()
            .flat_map(|frame| frame.actions.iter())
            .filter(|(id, _)| own.descriptor.actions.contains_key(id))
            .cloned()
            .collect();
        CoreContext {
            tick,
            own: own.descriptor.id.clone(),
            owners,
            declared_actions: own.descriptor.actions.keys().cloned().collect(),
            actions,
        }
    }
    fn live_id(&self, owner: CoreOwnerToken) -> Result<&PluginId, Error> {
        if owner.host != self.nonce {
            return Err(error(
                ErrorCode::StaleHandle,
                "owner belongs to a different host",
            ));
        }
        self.plugins
            .iter()
            .find(|(_, entry)| entry.generation == owner.generation)
            .map(|(id, _)| id)
            .ok_or_else(|| error(ErrorCode::StaleHandle, "owner token is no longer live"))
    }
    pub fn owner(&self, id: &PluginId) -> Result<CoreOwnerToken, Error> {
        self.plugins
            .get(id)
            .map(|entry| CoreOwnerToken {
                host: self.nonce,
                generation: entry.generation,
            })
            .ok_or_else(|| error(ErrorCode::StaleHandle, "owner is not installed"))
    }
    fn blockers(&self, id: &PluginId) -> Vec<PluginId> {
        self.state
            .owners
            .iter()
            .filter(|(_, owner)| owner.bindings.values().any(|provider| provider == id))
            .map(|(id, _)| id.clone())
            .collect()
    }
    pub fn install(
        &mut self,
        plugin: Box<dyn CorePlugin>,
        bindings: Bindings,
    ) -> Result<CoreOwnerToken, Error> {
        let descriptor = plugin.descriptor();
        validation::descriptor(&descriptor)?;
        if self.plugins.contains_key(&descriptor.id) {
            return Err(error(ErrorCode::DuplicateId, "owner already installed"));
        }
        if self.plugins.len() >= 16 {
            return Err(error(ErrorCode::BudgetExceeded, "plugin limit exceeded"));
        }
        validation::bindings(&descriptor, &bindings, &self.state.owners)?;
        for owner in self.state.owners.values() {
            if descriptor
                .schemas
                .keys()
                .any(|id| owner.descriptor.schemas.contains_key(id))
                || descriptor
                    .actions
                    .keys()
                    .any(|id| owner.descriptor.actions.contains_key(id))
            {
                return Err(error(
                    ErrorCode::DuplicateId,
                    "schema/action has another declaring owner",
                ));
            }
        }
        let generation = self.generation()?;
        let mut hash = Sha256::new();
        hash.update(self.seed.to_le_bytes());
        hash.update(descriptor.id.as_str().as_bytes());
        let hash = hash.finalize();
        let mut prefix = [0; 8];
        prefix.copy_from_slice(&hash[..8]);
        let mut own = SavedPlugin {
            descriptor,
            bindings,
            records: BTreeMap::new(),
            scenes: Vec::new(),
            entities: Vec::new(),
            rng_state: u64::from_le_bytes(prefix),
            history: OwnerHistory {
                emitted: 0,
                dropped: 0,
                history: Vec::new(),
                pending: Vec::new(),
            },
        };
        let mut candidate = self.state.clone();
        let context = self.context(&own, self.state.tick, None);
        let mut tx =
            CoreTransaction::new(&mut own, &self.state.owners, &mut candidate.next_identity);
        let result = plugin.initialize(&context, &mut tx);
        let emitted = tx.finish(result)?;
        Self::append_events(
            &mut own,
            &mut candidate.next_sequence,
            self.state.tick,
            emitted,
            true,
        )?;
        let id = own.descriptor.id.clone();
        candidate.owners.insert(id.clone(), own);
        self.validate_commit(&candidate)?;
        self.plugins.insert(id, Entry { plugin, generation });
        self.state = candidate;
        Ok(CoreOwnerToken {
            host: self.nonce,
            generation,
        })
    }
    pub fn remove(&mut self, owner: CoreOwnerToken) -> Result<(), Error> {
        let id = self.live_id(owner)?.clone();
        if !self.blockers(&id).is_empty() {
            return Err(error(
                ErrorCode::DependencyInUse,
                "dependent owner remains installed",
            ));
        }
        let mut candidate = self.state.clone();
        let removed = candidate.owners.remove(&id).expect("live installed owner");
        candidate.retired_emitted = candidate
            .retired_emitted
            .checked_add(removed.history.emitted)
            .ok_or_else(|| error(ErrorCode::Overflow, "retired event counter overflow"))?;
        candidate.purged_retained = candidate
            .purged_retained
            .checked_add(removed.history.history.len() as u64)
            .ok_or_else(|| error(ErrorCode::Overflow, "purged event counter overflow"))?;
        if let Some(input) = &mut candidate.last_input {
            input
                .actions
                .retain(|(id, _)| !removed.descriptor.actions.contains_key(id));
        }
        self.validate_commit(&candidate)?;
        self.plugins.remove(&id);
        self.state = candidate;
        Ok(())
    }
    fn append_events(
        own: &mut SavedPlugin,
        next_sequence: &mut u64,
        tick: u64,
        emitted: Vec<(String, Record)>,
        append: bool,
    ) -> Result<(), Error> {
        let before = if append { own.history.pending.len() } else { 0 };
        if emitted.len() > 128
            || before
                .checked_add(emitted.len())
                .is_none_or(|count| count > 128)
        {
            return Err(error(
                ErrorCode::BudgetExceeded,
                "owned commit/pending event limit exceeded",
            ));
        }
        let end_sequence = next_sequence
            .checked_add(emitted.len() as u64)
            .ok_or_else(|| error(ErrorCode::Overflow, "global event sequence overflow"))?;
        let end_ordinal = own
            .history
            .emitted
            .checked_add(emitted.len() as u64)
            .ok_or_else(|| error(ErrorCode::Overflow, "owned event ordinal overflow"))?;
        if !append {
            own.history.pending.clear();
        }
        for (offset, (kind, record)) in emitted.into_iter().enumerate() {
            let event = CoreEvent {
                owner: own.descriptor.id.clone(),
                tick,
                sequence: next_sequence
                    .checked_add(offset as u64)
                    .ok_or_else(|| error(ErrorCode::Overflow, "event sequence overflow"))?,
                ordinal: own
                    .history
                    .emitted
                    .checked_add(offset as u64)
                    .ok_or_else(|| error(ErrorCode::Overflow, "event ordinal overflow"))?,
                kind,
                record,
            };
            own.history.history.push(event.clone());
            own.history.pending.push(event);
        }
        let removed = own.history.history.len().saturating_sub(256);
        own.history.dropped = own
            .history
            .dropped
            .checked_add(removed as u64)
            .ok_or_else(|| error(ErrorCode::Overflow, "owned dropped counter overflow"))?;
        own.history.history.drain(..removed);
        own.history.emitted = end_ordinal;
        *next_sequence = end_sequence;
        Ok(())
    }
    pub fn step(&mut self, mut input: InputFrame) -> Result<(), Error> {
        if input.target_tick <= self.state.tick {
            return Err(error(
                ErrorCode::InputAlreadyCommitted,
                "input tick is already committed",
            ));
        }
        let expected = self
            .state
            .tick
            .checked_add(1)
            .ok_or_else(|| error(ErrorCode::Overflow, "tick exhausted"))?;
        if input.target_tick != expected {
            return Err(error(
                ErrorCode::InputOutOfSequence,
                "input tick skips the next tick",
            ));
        }
        validation::input(&input, &self.state.owners, false)?;
        input.actions.sort_by(|(a, _), (b, _)| a.cmp(b));
        let mut candidate = self.state.clone();
        for id in validation::order(&self.state.owners)? {
            let context = self.context(&self.state.owners[&id], input.target_tick, Some(&input));
            let own = candidate
                .owners
                .get_mut(&id)
                .expect("ordered installed owner");
            let mut tx =
                CoreTransaction::new(own, &self.state.owners, &mut candidate.next_identity);
            let result = self.plugins[&id].plugin.tick(&context, &mut tx);
            let events = tx.finish(result)?;
            Self::append_events(
                own,
                &mut candidate.next_sequence,
                input.target_tick,
                events,
                false,
            )?;
        }
        candidate.tick = input.target_tick;
        candidate.last_input = Some(input);
        self.validate_commit(&candidate)?;
        self.state = candidate;
        Ok(())
    }
    pub fn select(&self, selection: &Selection) -> Result<ReadFrame, Error> {
        if selection
            .owners
            .iter()
            .any(|id| !self.state.owners.contains_key(id))
        {
            return Err(error(ErrorCode::StaleHandle, "selected owner is absent"));
        }
        if selection.schemas.iter().any(|schema| {
            !self
                .state
                .owners
                .values()
                .any(|owner| owner.descriptor.schemas.contains_key(schema))
        }) {
            return Err(error(
                ErrorCode::UndeclaredKey,
                "selected schema is undeclared",
            ));
        }
        let mut frame = ReadFrame {
            tick: self.state.tick,
            records: BTreeMap::new(),
            scenes: Vec::new(),
            entities: Vec::new(),
            histories: BTreeMap::new(),
        };
        for id in &selection.owners {
            let own = &self.state.owners[id];
            for (local, record) in &own.records {
                if selection.schemas.is_empty() || selection.schemas.contains(&record.schema) {
                    frame.records.insert(
                        RecordKey {
                            owner: id.clone(),
                            local: local.clone(),
                        },
                        record.clone(),
                    );
                }
            }
            if selection.include_objects {
                frame.scenes.extend(own.scenes.clone());
                frame.entities.extend(own.entities.clone());
            }
            if selection.include_history {
                frame.histories.insert(id.clone(), own.history.clone());
            }
        }
        Ok(frame)
    }
    pub fn scene_handle(&self, reference: &SceneRef) -> Result<SceneHandle, Error> {
        if self
            .state
            .owners
            .get(&reference.owner)
            .and_then(|owner| owner.scene(reference))
            .is_none()
        {
            return Err(error(ErrorCode::StaleHandle, "scene is not live"));
        }
        Ok(SceneHandle {
            host: self.nonce,
            generation: self.plugins[&reference.owner].generation,
            reference: reference.clone(),
        })
    }
    pub fn entity_handle(&self, reference: &EntityRef) -> Result<EntityHandle, Error> {
        if self
            .state
            .owners
            .get(&reference.owner)
            .and_then(|owner| owner.entity(reference))
            .is_none()
        {
            return Err(error(ErrorCode::StaleHandle, "entity is not live"));
        }
        Ok(EntityHandle {
            host: self.nonce,
            generation: self.plugins[&reference.owner].generation,
            reference: reference.clone(),
        })
    }
    pub fn resolve_scene(&self, handle: SceneHandle) -> Result<SceneData, Error> {
        if handle.host != self.nonce
            || self
                .plugins
                .get(&handle.reference.owner)
                .is_none_or(|owner| owner.generation != handle.generation)
        {
            return Err(error(
                ErrorCode::StaleHandle,
                "scene handle timeline is stale",
            ));
        }
        self.state
            .owners
            .get(&handle.reference.owner)
            .and_then(|owner| owner.scene(&handle.reference))
            .cloned()
            .ok_or_else(|| error(ErrorCode::StaleHandle, "scene is not live"))
    }
    pub fn resolve_entity(&self, handle: EntityHandle) -> Result<EntityData, Error> {
        if handle.host != self.nonce
            || self
                .plugins
                .get(&handle.reference.owner)
                .is_none_or(|owner| owner.generation != handle.generation)
        {
            return Err(error(
                ErrorCode::StaleHandle,
                "entity handle timeline is stale",
            ));
        }
        self.state
            .owners
            .get(&handle.reference.owner)
            .and_then(|owner| owner.entity(&handle.reference))
            .cloned()
            .ok_or_else(|| error(ErrorCode::StaleHandle, "entity is not live"))
    }
    pub fn describe(&self) -> CoreDiscovery {
        CoreDiscovery {
            contract_version: version(),
            save_version: 2,
            scalar_compatibility_version: crate::contract_version(),
            limits: CoreLimits::default(),
            tick: self.state.tick,
            plugins: self
                .state
                .owners
                .iter()
                .map(|(id, owner)| CoreInstalled {
                    descriptor: owner.descriptor.clone(),
                    bindings: owner.bindings.clone(),
                    removal_blockers: self.blockers(id),
                })
                .collect(),
            last_input: self.state.last_input.clone(),
        }
    }
    pub fn save(&self) -> Result<Vec<u8>, Error> {
        validation::save_bytes(&self.envelope(&self.state))
    }
    pub fn hash(&self) -> Result<String, Error> {
        Ok(format!("{:x}", Sha256::digest(self.save()?)))
    }
    pub fn restore(&mut self, bytes: &[u8]) -> Result<(), Error> {
        if bytes.len() > 8_388_608 {
            return Err(error(ErrorCode::InvalidSave, "save2 byte limit exceeded"));
        }
        // Unknown data is skipped without allocation; full decoding below remains strict.
        #[derive(Deserialize)]
        struct VersionProbe {
            format_version: u32,
            contract_version: Version,
        }
        let probe: VersionProbe = serde_json::from_slice(bytes)
            .map_err(|e| Error::new(ErrorCode::InvalidSave, e.to_string()))?;
        if probe.format_version != 2 || probe.contract_version != version() {
            return Err(error(
                ErrorCode::VersionMismatch,
                "save is not version2/contract2",
            ));
        }
        let saved: SavedEnvelope = serde_json::from_slice(bytes)
            .map_err(|e| Error::new(ErrorCode::InvalidSave, e.to_string()))?;
        let canonical = validation::save_bytes(&saved)
            .map_err(|e| Error::new(ErrorCode::InvalidSave, e.detail()))?;
        if canonical != bytes {
            return Err(error(ErrorCode::InvalidSave, "save2 is noncanonical"));
        }
        if saved.content_binding != self.content_binding || saved.seed != self.seed {
            return Err(error(
                ErrorCode::SaveMismatch,
                "immutable content/seed differs",
            ));
        }
        if saved.plugins.len() > 16
            || saved
                .plugins
                .windows(2)
                .any(|pair| pair[0].descriptor.id >= pair[1].descriptor.id)
        {
            return Err(error(
                ErrorCode::InvalidSave,
                "plugins are excessive, duplicate or unsorted",
            ));
        }
        if saved.plugins.len() != self.plugins.len() {
            return Err(error(
                ErrorCode::SaveMismatch,
                "installed owner set differs",
            ));
        }
        for (record, (id, owner)) in saved.plugins.iter().zip(&self.state.owners) {
            if &record.descriptor.id != id
                || record.descriptor != owner.descriptor
                || record.bindings != owner.bindings
            {
                return Err(error(
                    ErrorCode::SaveMismatch,
                    "exact descriptor/binding set differs",
                ));
            }
        }
        let candidate = State {
            tick: saved.tick,
            next_identity: saved.next_identity,
            last_input: saved.last_input,
            owners: saved
                .plugins
                .into_iter()
                .map(|owner| (owner.descriptor.id.clone(), owner))
                .collect(),
            next_sequence: saved.next_sequence,
            retired_emitted: saved.retired_emitted,
            purged_retained: saved.purged_retained,
        };
        validation::state(&candidate)
            .map_err(|e| Error::new(ErrorCode::InvalidSave, e.detail()))?;
        let end = self
            .next_generation
            .checked_add(self.plugins.len() as u64)
            .ok_or_else(|| error(ErrorCode::Overflow, "runtime generation exhausted"))?;
        let generations = (0..self.plugins.len())
            .map(|offset| {
                self.next_generation
                    .checked_add(offset as u64)
                    .ok_or_else(|| error(ErrorCode::Overflow, "runtime generation exhausted"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        for (entry, generation) in self.plugins.values_mut().zip(generations) {
            entry.generation = generation;
        }
        self.next_generation = end;
        self.state = candidate;
        Ok(())
    }
}

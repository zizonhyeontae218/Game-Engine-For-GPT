use super::*;

/// A private own-namespace working copy. Any rejected operation poisons commit.
pub struct CoreTransaction<'a> {
    own: &'a mut SavedPlugin,
    committed: &'a BTreeMap<PluginId, SavedPlugin>,
    next_identity: &'a mut u64,
    emitted: Vec<(String, Record)>,
    commands: usize,
    poison: Option<Error>,
}
impl<'a> CoreTransaction<'a> {
    pub(super) fn new(
        own: &'a mut SavedPlugin,
        committed: &'a BTreeMap<PluginId, SavedPlugin>,
        next_identity: &'a mut u64,
    ) -> Self {
        Self {
            own,
            committed,
            next_identity,
            emitted: Vec::new(),
            commands: 0,
            poison: None,
        }
    }
    fn operation<T>(
        &mut self,
        operation: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let result = self
            .commands
            .checked_add(1)
            .ok_or_else(|| error(ErrorCode::Overflow, "command counter overflow"))
            .and_then(|commands| {
                self.commands = commands;
                if let Some(error) = &self.poison {
                    Err(error.clone())
                } else if commands > 4096 {
                    Err(error(
                        ErrorCode::BudgetExceeded,
                        "callback command budget exceeded",
                    ))
                } else {
                    operation(self)
                }
            });
        if let Err(error) = &result
            && self.poison.is_none()
        {
            self.poison = Some(error.clone());
        }
        result
    }
    fn local(local: &str) -> Result<(), Error> {
        if crate::local_key(local) {
            Ok(())
        } else {
            Err(error(ErrorCode::InvalidIdentifier, "invalid local key"))
        }
    }
    fn identity(&mut self) -> Result<u64, Error> {
        let identity = *self.next_identity;
        *self.next_identity = identity
            .checked_add(1)
            .ok_or_else(|| error(ErrorCode::Overflow, "object identity exhausted"))?;
        Ok(identity)
    }
    pub fn create_scene(&mut self, local: &str) -> Result<SceneRef, Error> {
        self.operation(|tx| {
            Self::local(local)?;
            if tx
                .own
                .scenes
                .iter()
                .any(|scene| scene.reference.local == local)
            {
                return Err(error(ErrorCode::DuplicateId, "scene local is already live"));
            }
            if tx.own.scenes.len() >= 64 {
                return Err(error(ErrorCode::BudgetExceeded, "scene limit exceeded"));
            }
            let reference = SceneRef {
                owner: tx.own.descriptor.id.clone(),
                local: local.to_owned(),
                incarnation: tx.identity()?,
            };
            tx.own.scenes.push(SceneData {
                reference: reference.clone(),
            });
            tx.own.scenes.sort_by(|a, b| a.reference.cmp(&b.reference));
            Ok(reference)
        })
    }
    pub fn create_entity(&mut self, local: &str, scene: &SceneRef) -> Result<EntityRef, Error> {
        self.operation(|tx| {
            Self::local(local)?;
            if tx
                .own
                .entities
                .iter()
                .any(|entity| entity.reference.local == local)
            {
                return Err(error(
                    ErrorCode::DuplicateId,
                    "entity local is already live",
                ));
            }
            if tx.own.entities.len() >= 256 {
                return Err(error(ErrorCode::BudgetExceeded, "entity limit exceeded"));
            }
            validation::resolve_scene(tx.own, tx.committed, scene)?;
            let reference = EntityRef {
                owner: tx.own.descriptor.id.clone(),
                local: local.to_owned(),
                incarnation: tx.identity()?,
            };
            tx.own.entities.push(EntityData {
                reference: reference.clone(),
                scene: scene.clone(),
            });
            tx.own
                .entities
                .sort_by(|a, b| a.reference.cmp(&b.reference));
            Ok(reference)
        })
    }
    fn in_use(&self, reference: &ObjectRef) -> bool {
        let contains = |owner: &SavedPlugin| {
            owner
                .records
                .values()
                .any(|record| validation::contains(record, reference))
                || owner
                    .history
                    .history
                    .iter()
                    .any(|event| validation::contains(&event.record, reference))
                || owner.entities.iter().any(
                    |entity| matches!(reference, ObjectRef::Scene(scene) if entity.scene == *scene),
                )
        };
        contains(self.own)
            || self
                .emitted
                .iter()
                .any(|(_, record)| validation::contains(record, reference))
            || self
                .committed
                .iter()
                .any(|(id, owner)| id != &self.own.descriptor.id && contains(owner))
    }
    pub fn remove_scene(&mut self, reference: &SceneRef) -> Result<(), Error> {
        self.operation(|tx| {
            if reference.owner != tx.own.descriptor.id {
                return Err(error(
                    ErrorCode::PermissionDenied,
                    "cannot remove a foreign scene",
                ));
            }
            if tx.own.scene(reference).is_none() {
                return Err(error(ErrorCode::StaleHandle, "scene is not live"));
            }
            if tx.in_use(&ObjectRef::Scene(reference.clone())) {
                return Err(error(
                    ErrorCode::ReferenceInUse,
                    "scene has surviving references",
                ));
            }
            tx.own.scenes.retain(|scene| scene.reference != *reference);
            Ok(())
        })
    }
    pub fn remove_entity(&mut self, reference: &EntityRef) -> Result<(), Error> {
        self.operation(|tx| {
            if reference.owner != tx.own.descriptor.id {
                return Err(error(
                    ErrorCode::PermissionDenied,
                    "cannot remove a foreign entity",
                ));
            }
            if tx.own.entity(reference).is_none() {
                return Err(error(ErrorCode::StaleHandle, "entity is not live"));
            }
            if tx.in_use(&ObjectRef::Entity(reference.clone())) {
                return Err(error(
                    ErrorCode::ReferenceInUse,
                    "entity has surviving references",
                ));
            }
            tx.own
                .entities
                .retain(|entity| entity.reference != *reference);
            Ok(())
        })
    }
    pub fn set(&mut self, local: &str, record: Record) -> Result<(), Error> {
        self.operation(|tx| {
            Self::local(local)?;
            validation::record(&record, &tx.own.descriptor, |reference| {
                validation::resolve(tx.own, tx.committed, reference)
            })?;
            if !tx.own.records.contains_key(local) && tx.own.records.len() >= 256 {
                return Err(error(ErrorCode::BudgetExceeded, "record limit exceeded"));
            }
            tx.own.records.insert(local.to_owned(), record);
            Ok(())
        })
    }
    pub fn delete(&mut self, local: &str) -> Result<(), Error> {
        self.operation(|tx| {
            Self::local(local)?;
            tx.own.records.remove(local);
            Ok(())
        })
    }
    pub fn emit(&mut self, kind: &str, record: Record) -> Result<(), Error> {
        self.operation(|tx| {
            Self::local(kind)?;
            let schema = tx
                .own
                .descriptor
                .event_kinds
                .get(kind)
                .ok_or_else(|| error(ErrorCode::UndeclaredKey, "event kind is not declared"))?;
            if schema != &record.schema {
                return Err(error(
                    ErrorCode::InvalidRecord,
                    "event kind schema mismatch",
                ));
            }
            validation::record(&record, &tx.own.descriptor, |reference| {
                validation::resolve(tx.own, tx.committed, reference)
            })?;
            if tx.emitted.len() >= 128 {
                return Err(error(
                    ErrorCode::BudgetExceeded,
                    "owned event commit limit exceeded",
                ));
            }
            tx.emitted.push((kind.to_owned(), record));
            Ok(())
        })
    }
    pub fn draw(&mut self) -> Result<u64, Error> {
        self.operation(|tx| {
            tx.own.rng_state = tx.own.rng_state.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = tx.own.rng_state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            Ok(z ^ (z >> 31))
        })
    }
    pub(super) fn finish(self, result: Result<(), Error>) -> Result<Vec<(String, Record)>, Error> {
        if let Some(error) = self.poison {
            return Err(error);
        }
        result?;
        Ok(self.emitted)
    }
}

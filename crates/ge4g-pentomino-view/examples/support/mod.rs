//! Public consumer source fixture. Position semantics are authored by this plugin.
use ge4g_pentomino::p2::*;
use ge4g_pentomino::{PluginId, Version};
use std::collections::{BTreeMap, BTreeSet};
pub struct Source {
    pub owner: PluginId,
    pub count: usize,
}
impl Source {
    pub fn schema(&self) -> SchemaId {
        SchemaId::new(&format!("{}.position", self.owner.as_str())).unwrap()
    }
}
impl CorePlugin for Source {
    fn descriptor(&self) -> CoreDescriptor {
        CoreDescriptor {
            id: self.owner.clone(),
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
            schemas: BTreeMap::from([(
                self.schema(),
                RecordSchema {
                    fields: BTreeMap::from([
                        (
                            "entity".into(),
                            FieldType::Ref {
                                kind: RefKind::Entity,
                            },
                        ),
                        (
                            "x".into(),
                            FieldType::I64 {
                                min: -1_000_000,
                                max: 1_000_000,
                            },
                        ),
                        (
                            "y".into(),
                            FieldType::I64 {
                                min: -1_000_000,
                                max: 1_000_000,
                            },
                        ),
                        (
                            "z".into(),
                            FieldType::I64 {
                                min: -1_000_000,
                                max: 1_000_000,
                            },
                        ),
                    ]),
                },
            )]),
            event_kinds: BTreeMap::new(),
            actions: BTreeMap::new(),
        }
    }
    fn initialize(&self, _: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error> {
        let scene = tx.create_scene("scene")?;
        for i in 0..self.count {
            let key = format!("item_{i}");
            let entity = tx.create_entity(&key, &scene)?;
            tx.set(
                &key,
                Record {
                    schema: self.schema(),
                    fields: BTreeMap::from([
                        ("entity".into(), Value::Ref(ObjectRef::Entity(entity))),
                        ("x".into(), Value::I64((i % 16) as i64 * 10)),
                        ("y".into(), Value::I64((i / 16) as i64 * 10)),
                        ("z".into(), Value::I64(0)),
                    ]),
                },
            )?;
        }
        Ok(())
    }
    fn tick(&self, _: &CoreContext, _: &mut CoreTransaction<'_>) -> Result<(), Error> {
        Ok(())
    }
}
pub fn selection(owners: impl IntoIterator<Item = PluginId>) -> Selection {
    Selection {
        owners: owners.into_iter().collect::<BTreeSet<_>>(),
        schemas: BTreeSet::new(),
        include_objects: true,
        include_history: true,
    }
}
// Kept as an example module for source-free consumers, not a standalone program.
fn main() {}

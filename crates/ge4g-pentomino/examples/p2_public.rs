//! Public-surface P2 example; no engine implementation imports or private access.
use ge4g_pentomino::p2::{
    ActionId, ActionType, ActionValue, CoreContext, CoreDescriptor, CoreHost, CorePlugin,
    CoreTransaction, Error, ErrorCode, FieldType, InputFrame, ObjectRef, Record, RecordSchema,
    RefKind, SchemaId, Selection, Value,
};
use ge4g_pentomino::{Bindings, PluginId, Version};
use std::collections::{BTreeMap, BTreeSet};

struct Sample {
    name: &'static str,
    events: usize,
}
impl Sample {
    fn schema(&self, suffix: &str) -> SchemaId {
        SchemaId::new(&format!("{}.{suffix}", self.name)).expect("static schema ID")
    }
}
impl CorePlugin for Sample {
    fn descriptor(&self) -> CoreDescriptor {
        CoreDescriptor {
            id: PluginId::new(self.name).expect("static plugin ID"),
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
            schemas: BTreeMap::from([
                (
                    self.schema("data"),
                    RecordSchema {
                        fields: BTreeMap::from([
                            ("enabled".into(), FieldType::Bool),
                            (
                                "count".into(),
                                FieldType::I64 {
                                    min: 0,
                                    max: i64::MAX,
                                },
                            ),
                            (
                                "random".into(),
                                FieldType::I64 {
                                    min: 0,
                                    max: i64::MAX,
                                },
                            ),
                            ("label".into(), FieldType::String { max_bytes: 128 }),
                            ("bytes".into(), FieldType::Bytes { max_bytes: 16 }),
                            (
                                "scene".into(),
                                FieldType::Ref {
                                    kind: RefKind::Scene,
                                },
                            ),
                            (
                                "entity".into(),
                                FieldType::Ref {
                                    kind: RefKind::Entity,
                                },
                            ),
                            (
                                "items".into(),
                                FieldType::List {
                                    item: Box::new(FieldType::I64 { min: 0, max: 9 }),
                                    max_items: 4,
                                },
                            ),
                        ]),
                    },
                ),
                (
                    self.schema("event"),
                    RecordSchema {
                        fields: BTreeMap::from([(
                            "count".into(),
                            FieldType::I64 {
                                min: 0,
                                max: i64::MAX,
                            },
                        )]),
                    },
                ),
            ]),
            event_kinds: BTreeMap::from([("changed".into(), self.schema("event"))]),
            actions: BTreeMap::from([(
                ActionId::new(&format!("{}.enabled", self.name)).expect("static action"),
                ActionType::Bool,
            )]),
        }
    }
    fn initialize(&self, _: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error> {
        let scene = tx.create_scene("scope")?;
        let entity = tx.create_entity("item", &scene)?;
        tx.set(
            "data",
            Record {
                schema: self.schema("data"),
                fields: BTreeMap::from([
                    ("enabled".into(), Value::Bool(true)),
                    ("count".into(), Value::I64(0)),
                    ("random".into(), Value::I64(0)),
                    ("label".into(), Value::String("외부 검증".into())),
                    ("bytes".into(), Value::Bytes(vec![1, 2, 3])),
                    ("scene".into(), Value::Ref(ObjectRef::Scene(scene))),
                    ("entity".into(), Value::Ref(ObjectRef::Entity(entity))),
                    (
                        "items".into(),
                        Value::List(vec![Value::I64(1), Value::I64(2)]),
                    ),
                ]),
            },
        )
    }
    fn tick(&self, ctx: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error> {
        let mut record = ctx
            .own("data")?
            .ok_or_else(|| Error::new(ErrorCode::InvalidRecord, "missing data"))?
            .clone();
        if let Some(ActionValue::Bool(enabled)) =
            ctx.action(&ActionId::new(&format!("{}.enabled", self.name))?)?
        {
            record
                .fields
                .insert("enabled".into(), Value::Bool(*enabled));
        }
        if record.fields["enabled"] == Value::Bool(true) {
            let Value::I64(old) = record.fields["count"] else {
                return Err(Error::new(ErrorCode::InvalidRecord, "count type"));
            };
            let count = old
                .checked_add(1)
                .ok_or_else(|| Error::new(ErrorCode::Overflow, "count"))?;
            record.fields.insert("count".into(), Value::I64(count));
            record.fields.insert(
                "random".into(),
                Value::I64((tx.draw()? & i64::MAX as u64) as i64),
            );
            for _ in 0..self.events {
                tx.emit(
                    "changed",
                    Record {
                        schema: self.schema("event"),
                        fields: BTreeMap::from([("count".into(), Value::I64(count))]),
                    },
                )?;
            }
        }
        tx.set("data", record)
    }
}
fn selection(name: &str) -> Selection {
    Selection {
        owners: BTreeSet::from([PluginId::new(name).unwrap()]),
        schemas: BTreeSet::new(),
        include_objects: true,
        include_history: true,
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut host = CoreHost::new(17, "p2.public.example")?;
    let a = host.install(
        Box::new(Sample {
            name: "sample.alpha",
            events: 32,
        }),
        Bindings::new(),
    )?;
    host.install(
        Box::new(Sample {
            name: "sample.beta",
            events: 1,
        }),
        Bindings::new(),
    )?;
    for target_tick in 1..=300 {
        host.step(InputFrame {
            target_tick,
            actions: vec![],
        })?;
    }
    let b = PluginId::new("sample.beta")?;
    let before = host.select(&selection("sample.beta"))?;
    assert_eq!(before.histories[&b].history.len(), 256);
    assert_eq!(before.histories[&b].emitted, 300);
    assert_eq!(before.histories[&b].dropped, 44);
    host.remove(a)?;
    assert_eq!(before, host.select(&selection("sample.beta"))?);
    let handle = host.entity_handle(&before.entities[0].reference)?;
    let save = host.save()?;
    host.restore(&save)?;
    assert_eq!(save, host.save()?);
    assert_eq!(
        host.resolve_entity(handle).unwrap_err().code(),
        ErrorCode::StaleHandle
    );
    let mut control = CoreHost::new(17, "p2.public.example")?;
    control.install(
        Box::new(Sample {
            name: "sample.beta",
            events: 1,
        }),
        Bindings::new(),
    )?;
    control.restore(&save)?;
    let input = InputFrame {
        target_tick: 301,
        actions: vec![(
            ActionId::new("sample.beta.enabled")?,
            ActionValue::Bool(false),
        )],
    };
    host.step(input.clone())?;
    control.step(input)?;
    assert_eq!(host.save()?, control.save()?);
    let final_frame = host.select(&selection("sample.beta"))?;
    assert_eq!(
        final_frame.records.values().next().unwrap().fields["count"],
        Value::I64(300)
    );
    println!(
        "{{\"ok\":true,\"contract\":2,\"ticks\":301,\"per_owner_retained\":256,\"dropped\":44,\"hash\":\"{}\"}}",
        host.hash()?
    );
    Ok(())
}

use crate::{math::*, types::*};
use ge4g_pentomino::p2::{ObjectRef, ReadFrame, SceneRef, Value};
use std::collections::{BTreeMap, BTreeSet};

/// Trusted linked extractor. Implementations must be pure; hidden callback state is not transactional.
pub trait FormatPlugin {
    fn descriptor(&self) -> FormatDescriptor;
    fn extract(
        &self,
        read: &ReadFrame,
        binding: &RecordBinding,
        scene: &SceneRef,
    ) -> Result<PresentationSelection, Error>;
}
pub struct Classic2DFormat;
pub struct TopDownFormat;
pub struct SideFormat;
pub struct VerticalFormat;
macro_rules! builtin {
    ($ty:ident,$family:ident,$id:literal) => {
        impl FormatPlugin for $ty {
            fn descriptor(&self) -> FormatDescriptor {
                descriptor($id, ViewFamily::$family)
            }
            fn extract(
                &self,
                read: &ReadFrame,
                binding: &RecordBinding,
                scene: &SceneRef,
            ) -> Result<PresentationSelection, Error> {
                extract(read, binding, scene)
            }
        }
    };
}
builtin!(Classic2DFormat, Classic2D, "pentomino.classic2d");
builtin!(TopDownFormat, TopDown, "pentomino.top_down");
builtin!(SideFormat, Side, "pentomino.side");
builtin!(VerticalFormat, Vertical, "pentomino.vertical");
fn descriptor(id: &str, family: ViewFamily) -> FormatDescriptor {
    FormatDescriptor {
        id: FormatId::new(id).expect("static valid identifier"),
        version: 1,
        family,
        required_fields: BTreeMap::from([
            ("entity".into(), "ref(entity)".into()),
            ("x".into(), "i64".into()),
            ("y".into(), "i64".into()),
            ("z".into(), "optional:i64".into()),
        ]),
        representations: vec![
            "sprite".into(),
            "model".into(),
            "background".into(),
            "billboard".into(),
        ],
    }
}
pub(crate) fn builtins() -> Vec<FormatDescriptor> {
    vec![
        Classic2DFormat.descriptor(),
        TopDownFormat.descriptor(),
        SideFormat.descriptor(),
        VerticalFormat.descriptor(),
    ]
}
pub(crate) fn validate_descriptor(d: &FormatDescriptor) -> Result<(), Error> {
    if d.version == 0
        || d.required_fields.len() > 32
        || d.representations.is_empty()
        || d.representations.len() > 4
    {
        return Err(err(
            ErrorCode::InvalidDescriptor,
            "descriptor count/version invalid",
        ));
    }
    for (k, v) in &d.required_fields {
        if k.is_empty() || k.len() > 128 || v.is_empty() || v.len() > 128 {
            return Err(err(ErrorCode::InvalidDescriptor, "descriptor field bound"));
        }
    }
    let mut seen = BTreeSet::new();
    for r in &d.representations {
        if !matches!(r.as_str(), "sprite" | "model" | "background" | "billboard") || !seen.insert(r)
        {
            return Err(err(
                ErrorCode::InvalidDescriptor,
                "unsupported/duplicate representation",
            ));
        }
    }
    Ok(())
}
pub(crate) fn validate_representation(r: &Representation) -> Result<(), Error> {
    let (asset, size) = match r {
        Representation::Sprite { asset, size } | Representation::Billboard { asset, size } => {
            (asset, Some(size))
        }
        Representation::Model { asset } | Representation::Background { asset } => (asset, None),
    };
    if asset.is_empty() || asset.len() > 1024 {
        return Err(err(
            ErrorCode::InvalidSelection,
            "asset identifier exceeds bound",
        ));
    }
    if let Some(size) = size {
        for x in size {
            range(*x, 0.001, 1e9, ErrorCode::InvalidSelection)?;
        }
    }
    Ok(())
}
pub(crate) fn validate_binding(b: &RecordBinding) -> Result<(), Error> {
    range(b.units_per_world, 0.001, 1e9, ErrorCode::InvalidSelection)?;
    let fields = [
        Some(&b.entity_field),
        Some(&b.x_field),
        Some(&b.y_field),
        b.z_field.as_ref(),
    ];
    let mut seen = BTreeSet::new();
    for f in fields.into_iter().flatten() {
        if f.is_empty()
            || f.len() > 128
            || !f
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
            || !seen.insert(f)
        {
            return Err(err(
                ErrorCode::InvalidSelection,
                "invalid/duplicate binding field",
            ));
        }
    }
    validate_representation(&b.representation)
}
pub(crate) fn preflight_read(read: &ReadFrame) -> Result<(), Error> {
    // P2 supports at most16owners*256records/entities and16*64scenes.
    // Count before any accepted-state clone or index allocation.
    if read.records.len() > 4096 || read.entities.len() > 4096 || read.scenes.len() > 1024 {
        return Err(err(
            ErrorCode::BudgetExceeded,
            "read selection exceeds P2 object/record bounds",
        ));
    }
    let mut scenes = BTreeSet::new();
    for s in &read.scenes {
        validate_scene_ref(&s.reference)?;
        if !scenes.insert(&s.reference) {
            return Err(err(
                ErrorCode::InvalidSelection,
                "duplicate scene reference",
            ));
        }
    }
    let mut entities = BTreeSet::new();
    for e in &read.entities {
        validate_entity_ref(&e.reference)?;
        validate_scene_ref(&e.scene)?;
        if !entities.insert(&e.reference) || !scenes.contains(&e.scene) {
            return Err(err(
                ErrorCode::InvalidSelection,
                "duplicate entity or missing entity scene",
            ));
        }
    }
    Ok(())
}
fn valid_local(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
pub(crate) fn validate_scene_ref(r: &SceneRef) -> Result<(), Error> {
    if !valid_local(&r.local) || r.incarnation == 0 {
        Err(err(ErrorCode::InvalidSelection, "invalid scene reference"))
    } else {
        Ok(())
    }
}
pub(crate) fn validate_entity_ref(r: &ge4g_pentomino::p2::EntityRef) -> Result<(), Error> {
    if !valid_local(&r.local) || r.incarnation == 0 {
        Err(err(ErrorCode::InvalidSelection, "invalid entity reference"))
    } else {
        Ok(())
    }
}
pub(crate) fn validate_selection(
    read: &ReadFrame,
    s: &PresentationSelection,
    scene: &SceneRef,
) -> Result<(), Error> {
    if s.items.len() > 4096 {
        return Err(err(ErrorCode::BudgetExceeded, "too many selected items"));
    }
    if s.scene != *scene || !read.scenes.iter().any(|x| x.reference == *scene) {
        return Err(err(
            ErrorCode::InvalidSelection,
            "selection scene mismatch/missing",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut last = None;
    for item in &s.items {
        if !seen.insert(&item.entity)
            || last.is_some_and(|previous| previous >= &item.entity)
            || !read
                .entities
                .iter()
                .any(|x| x.reference == item.entity && x.scene == *scene)
        {
            return Err(err(
                ErrorCode::InvalidSelection,
                "unordered/duplicate/dead entity presentation",
            ));
        }
        point(item.world.0).map_err(|e| Error::new(ErrorCode::InvalidSelection, e.detail()))?;
        validate_representation(&item.representation)?;
        last = Some(&item.entity);
    }
    Ok(())
}
fn extract(
    read: &ReadFrame,
    binding: &RecordBinding,
    scene: &SceneRef,
) -> Result<PresentationSelection, Error> {
    preflight_read(read)?;
    validate_binding(binding)?;
    if !read.scenes.iter().any(|s| s.reference == *scene) {
        return Err(err(
            ErrorCode::InvalidSelection,
            "scene absent from read selection",
        ));
    }
    let entities: BTreeMap<_, _> = read
        .entities
        .iter()
        .map(|e| (&e.reference, &e.scene))
        .collect();
    let mut output = BTreeMap::new();
    let mut seen = BTreeSet::new();
    let mut count = 0usize;
    for (key, record) in &read.records {
        if key.owner != binding.owner || record.schema != binding.schema {
            continue;
        }
        count += 1;
        if count > 4096 {
            return Err(err(ErrorCode::BudgetExceeded, "matching record limit"));
        }
        let entity = match record.fields.get(&binding.entity_field) {
            Some(Value::Ref(ObjectRef::Entity(e))) => e,
            _ => {
                return Err(err(
                    ErrorCode::InvalidSelection,
                    "entity slot must be entity reference",
                ));
            }
        };
        let object_scene = entities
            .get(entity)
            .ok_or_else(|| err(ErrorCode::InvalidSelection, "dead entity reference"))?;
        if !seen.insert(entity) {
            return Err(err(ErrorCode::InvalidSelection, "duplicate entity record"));
        }
        let scalar = |field: &str| -> Result<f64, Error> {
            match record.fields.get(field) {
                Some(Value::I64(v)) => Ok(*v as f64 / binding.units_per_world),
                _ => Err(err(
                    ErrorCode::InvalidSelection,
                    "position slot must be i64",
                )),
            }
        };
        let p = [
            scalar(&binding.x_field)?,
            scalar(&binding.y_field)?,
            binding
                .z_field
                .as_ref()
                .map(|f| scalar(f))
                .transpose()?
                .unwrap_or(0.),
        ];
        point(p).map_err(|e| Error::new(ErrorCode::InvalidSelection, e.detail()))?;
        if **object_scene == *scene {
            output.insert(
                entity.clone(),
                PresentationItem {
                    entity: entity.clone(),
                    world: WorldPoint(p),
                    representation: binding.representation.clone(),
                },
            );
        }
    }
    Ok(PresentationSelection {
        scene: scene.clone(),
        items: output.into_values().collect(),
    })
}

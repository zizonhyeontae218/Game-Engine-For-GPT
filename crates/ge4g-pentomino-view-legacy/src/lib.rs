//! Read-only legacy content import and explicitly restricted CPU presentation adapter.
//! Source plugins own immutable imported identities; cameras never own gameplay.
//! Integer compositor parameters round to nearest (ties away from zero).
//! Rounding is at most half a percentage point / half a camera-offset pixel;
//! output tolerance additionally depends on position and integer truncation.
use ge4g_core::{SUBPIXELS, Snapshot};
use ge4g_pentomino::p2::{
    CoreContext, CoreDescriptor, CorePlugin, CoreTransaction, Error as CoreError, FieldType,
    ObjectRef, Record, RecordSchema, RefKind, SchemaId, Value,
};
use ge4g_pentomino::{PluginId, Version};
use ge4g_pentomino_view::{
    CameraFrame, CameraPose, CameraTarget, Projection, Quaternion, RecordBinding, Representation,
    ViewTransform, Viewport, WorldPoint, project,
};
use ge4g_project::Project;
use ge4g_runtime::World;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompatibilityErrorCode {
    InvalidSource,
    CoreValidation,
    Unsupported,
    RenderFailed,
    BudgetExceeded,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompatibilityError {
    code: CompatibilityErrorCode,
    detail: String,
}
impl CompatibilityError {
    pub fn new(code: CompatibilityErrorCode, detail: impl Into<String>) -> Self {
        let mut detail = detail.into();
        let mut n = detail.len().min(4096);
        while !detail.is_char_boundary(n) {
            n -= 1;
        }
        detail.truncate(n);
        Self { code, detail }
    }
    pub fn code(&self) -> CompatibilityErrorCode {
        self.code
    }
    pub fn detail(&self) -> &str {
        &self.detail
    }
}
impl std::fmt::Display for CompatibilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.detail)
    }
}
impl std::error::Error for CompatibilityError {}
impl From<CoreError> for CompatibilityError {
    fn from(e: CoreError) -> Self {
        Self::new(CompatibilityErrorCode::CoreValidation, e.to_string())
    }
}
impl From<ge4g_pentomino::Error> for CompatibilityError {
    fn from(e: ge4g_pentomino::Error) -> Self {
        Self::new(CompatibilityErrorCode::CoreValidation, e.to_string())
    }
}
type Result<T> = std::result::Result<T, CompatibilityError>;
fn invalid(s: &str) -> CompatibilityError {
    CompatibilityError::new(CompatibilityErrorCode::InvalidSource, s)
}
fn unsupported(s: &str) -> CompatibilityError {
    CompatibilityError::new(CompatibilityErrorCode::Unsupported, s)
}
fn budget(s: &str) -> CompatibilityError {
    CompatibilityError::new(CompatibilityErrorCode::BudgetExceeded, s)
}
fn text(s: &str) -> Result<()> {
    if s.len() > 1024 {
        Err(budget("source string exceeds 1024 bytes"))
    } else {
        Ok(())
    }
}
fn key(prefix: &str, source: &str) -> String {
    format!("{prefix}{:x}", Sha256::digest(source.as_bytes()))
}
fn object_key(prefix: &str, scene: &str, entity: &str) -> String {
    let mut h = Sha256::new();
    h.update(scene.as_bytes());
    h.update([0]);
    h.update(entity.as_bytes());
    format!("{prefix}{:x}", h.finalize())
}
fn schema(owner: &PluginId, suffix: &str) -> Result<SchemaId> {
    Ok(SchemaId::new(&format!("{}.{suffix}", owner.as_str()))?)
}
fn str_type() -> FieldType {
    FieldType::String { max_bytes: 1024 }
}
fn int_type() -> FieldType {
    FieldType::I64 {
        min: i64::MIN,
        max: i64::MAX,
    }
}
fn make_record(schema: SchemaId, fields: Vec<(&str, Value)>) -> Record {
    Record {
        schema,
        fields: fields.into_iter().map(|(k, v)| (k.into(), v)).collect(),
    }
}
struct SourceEntity {
    id: String,
    local: String,
    position_local: String,
    feet: [i64; 2],
}
struct SourcePlugin {
    descriptor: CoreDescriptor,
    scene: String,
    scene_local: String,
    tick: i64,
    entities: Vec<SourceEntity>,
    states: Vec<(String, String, SchemaId, Value)>,
}
/// Import the current scene and state once. No legacy commands or camera data enter Core.
pub fn presentation_plugin(
    world: &World,
    owner: PluginId,
) -> Result<(Box<dyn CorePlugin>, RecordBinding)> {
    if world.tick > 1_000_000 {
        return Err(budget("source tick exceeds 1000000"));
    }
    text(&world.scene)?;
    if !world.project.scenes.contains_key(&world.scene) {
        return Err(invalid("current scene missing"));
    }
    let count = world
        .entities
        .len()
        .checked_mul(2)
        .and_then(|v| v.checked_add(world.state.values.len()))
        .and_then(|v| v.checked_add(2));
    if count.is_none_or(|v| v > 256) {
        return Err(budget("2*entities+states+2 exceeds 256 records"));
    }
    if world.state.values.len() != world.state.definitions.len() {
        return Err(invalid("state definitions/values differ"));
    }
    let mut schemas = BTreeMap::new();
    for (suffix, fields) in [
        ("scene", vec![("id", str_type())]),
        (
            "entity",
            vec![
                ("id", str_type()),
                (
                    "scene",
                    FieldType::Ref {
                        kind: RefKind::Scene,
                    },
                ),
            ],
        ),
        (
            "state_bool",
            vec![("key", str_type()), ("value", FieldType::Bool)],
        ),
        (
            "state_i64",
            vec![("key", str_type()), ("value", int_type())],
        ),
        (
            "state_string",
            vec![("key", str_type()), ("value", str_type())],
        ),
        (
            "snapshot",
            vec![(
                "tick",
                FieldType::I64 {
                    min: 0,
                    max: 1_000_000,
                },
            )],
        ),
        (
            "position",
            vec![
                (
                    "entity",
                    FieldType::Ref {
                        kind: RefKind::Entity,
                    },
                ),
                ("x", int_type()),
                ("y", int_type()),
                ("z", int_type()),
            ],
        ),
    ] {
        schemas.insert(
            schema(&owner, suffix)?,
            RecordSchema {
                fields: fields.into_iter().map(|(k, v)| (k.into(), v)).collect(),
            },
        );
    }
    // Validate every source item before storing cloned strings/values.
    for (name, e) in &world.entities {
        text(name)?;
        if name != &e.spec.id {
            return Err(invalid("entity map identity mismatch"));
        }
        for (value, size) in [
            (e.position.x, e.spec.size[0] / 2),
            (e.position.y, e.spec.size[1]),
        ] {
            let feet = (i128::from(value.div_euclid(SUBPIXELS)) + i128::from(size))
                * i128::from(SUBPIXELS);
            if feet.abs() > 1_000_000_000i128 * i128::from(SUBPIXELS)
                || feet < i128::from(i64::MIN) + 1
                || feet > i128::from(i64::MAX)
            {
                return Err(invalid("entity visual feet overflow native bounds"));
            }
        }
    }
    for (name, value) in &world.state.values {
        text(name)?;
        let definition = world
            .state
            .definitions
            .get(name)
            .ok_or_else(|| invalid("state declaration missing"))?;
        if !definition.accepts(value) {
            return Err(invalid("state value violates declaration"));
        }
        if let Some(v) = value.as_str() {
            text(v)?;
        } else if !value.is_boolean() && value.as_i64().is_none() {
            return Err(invalid("state value is not bool/i64/string"));
        }
    }
    let entities = world
        .entities
        .iter()
        .map(|(id, e)| SourceEntity {
            id: id.clone(),
            local: object_key("e_", &world.scene, id),
            position_local: object_key("p_", &world.scene, id),
            feet: [
                (e.position.x.div_euclid(SUBPIXELS) + i64::from(e.spec.size[0]) / 2) * SUBPIXELS,
                -(e.position.y.div_euclid(SUBPIXELS) + i64::from(e.spec.size[1])) * SUBPIXELS,
            ],
        })
        .collect();
    let mut states = Vec::new();
    for (name, value) in &world.state.values {
        let (suffix, v) = if let Some(v) = value.as_bool() {
            ("state_bool", Value::Bool(v))
        } else if let Some(v) = value.as_i64() {
            ("state_i64", Value::I64(v))
        } else {
            (
                "state_string",
                Value::String(value.as_str().ok_or_else(|| invalid("state type"))?.into()),
            )
        };
        states.push((key("v_", name), name.clone(), schema(&owner, suffix)?, v));
    }
    let position_schema = schema(&owner, "position")?;
    let descriptor = CoreDescriptor {
        id: owner.clone(),
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
        actions: BTreeMap::new(),
    };
    let plugin = SourcePlugin {
        descriptor,
        scene: world.scene.clone(),
        scene_local: key("s_", &world.scene),
        tick: world.tick as i64,
        entities,
        states,
    };
    let binding = RecordBinding {
        owner,
        schema: position_schema,
        entity_field: "entity".into(),
        x_field: "x".into(),
        y_field: "y".into(),
        z_field: Some("z".into()),
        units_per_world: SUBPIXELS as f64,
        representation: Representation::Sprite {
            asset: "legacy.current-scene".into(),
            size: [1., 1.],
        },
    };
    Ok((Box::new(plugin), binding))
}
impl CorePlugin for SourcePlugin {
    fn descriptor(&self) -> CoreDescriptor {
        self.descriptor.clone()
    }
    fn initialize(
        &self,
        _: &CoreContext,
        tx: &mut CoreTransaction<'_>,
    ) -> std::result::Result<(), CoreError> {
        let owner = &self.descriptor.id;
        let scene = tx.create_scene(&self.scene_local)?;
        tx.set(
            &self.scene_local,
            make_record(
                self.descriptor
                    .schemas
                    .keys()
                    .find(|s| s.as_str() == format!("{}.scene", owner.as_str()))
                    .expect("constructed scene schema")
                    .clone(),
                vec![("id", Value::String(self.scene.clone()))],
            ),
        )?;
        let get = |suffix: &str| {
            self.descriptor
                .schemas
                .keys()
                .find(|s| s.as_str() == format!("{}.{suffix}", owner.as_str()))
                .expect("constructed schema")
                .clone()
        };
        tx.set(
            "snapshot",
            make_record(get("snapshot"), vec![("tick", Value::I64(self.tick))]),
        )?;
        for e in &self.entities {
            let entity = tx.create_entity(&e.local, &scene)?;
            tx.set(
                &e.local,
                make_record(
                    get("entity"),
                    vec![
                        ("id", Value::String(e.id.clone())),
                        ("scene", Value::Ref(ObjectRef::Scene(scene.clone()))),
                    ],
                ),
            )?;
            tx.set(
                &e.position_local,
                make_record(
                    get("position"),
                    vec![
                        ("entity", Value::Ref(ObjectRef::Entity(entity))),
                        ("x", Value::I64(e.feet[0])),
                        ("y", Value::I64(e.feet[1])),
                        ("z", Value::I64(0)),
                    ],
                ),
            )?;
        }
        for (local, name, schema, value) in &self.states {
            tx.set(
                local,
                make_record(
                    schema.clone(),
                    vec![
                        ("key", Value::String(name.clone())),
                        ("value", value.clone()),
                    ],
                ),
            )?;
        }
        Ok(())
    }
    fn tick(
        &self,
        _: &CoreContext,
        _: &mut CoreTransaction<'_>,
    ) -> std::result::Result<(), CoreError> {
        Ok(())
    }
}
fn identity(q: &Quaternion) -> bool {
    q.x == 0. && q.y == 0. && q.z == 0. && q.w == 1.
}
fn finite_bound(v: f64, bound: f64) -> bool {
    v.is_finite() && v.abs() <= bound
}
/// Build a compatible native target from the legacy pixel camera offset.
pub fn legacy_camera_target(
    snapshot: &Snapshot,
    viewport: &Viewport,
    zoom: f64,
    tilt: f64,
    shear: f64,
) -> Result<CameraTarget> {
    if viewport.x != 0.
        || viewport.y != 0.
        || !finite_bound(viewport.width, 16384.)
        || !finite_bound(viewport.height, 16384.)
        || viewport.width < 1.
        || viewport.height < 1.
    {
        return Err(unsupported(
            "legacy viewport must have zero origin and bounded size",
        ));
    }
    if !zoom.is_finite()
        || !(1. ..=10000.).contains(&zoom)
        || !tilt.is_finite()
        || !(1. ..=10000.).contains(&tilt)
        || !finite_bound(shear, 10000.)
    {
        return Err(unsupported("legacy percentages out of range"));
    }
    let z = zoom / 100.;
    let t = tilt / 100.;
    let sh = -shear / (100. * z * t);
    let ox = snapshot.camera[0] as f64 + viewport.width / 2.;
    let oy = snapshot.camera[1] as f64 + viewport.height / 2.;
    if !finite_bound(ox, 1e6) || !finite_bound(oy, 1e6) || !finite_bound(sh, 8.) {
        return Err(unsupported(
            "legacy origin/shear outside compatible native bounds",
        ));
    }
    let q = Quaternion {
        x: 0.,
        y: 0.,
        z: 0.,
        w: 1.,
    };
    Ok(CameraTarget {
        pose: CameraPose {
            position: [0., 0., 100.],
            orientation: q,
            zoom: z,
        },
        view_transform: ViewTransform {
            origin: [ox, -oy, 0.],
            rotation: q,
            scale: [1., t, 1.],
            shear_xy: sh,
        },
        projection: Projection::Orthographic {
            half_height: viewport.height / 2.,
            near: 0.001,
            far: 1000.,
            focus_distance: 100.,
        },
    })
}
/// Project the ground anchor, then apply upright elevation without tilting the sprite.
pub fn legacy_foot(camera: &CameraFrame, feet: [f64; 2], elevation: f64) -> Result<[f64; 2]> {
    if feet.iter().any(|v| !finite_bound(*v, 1e6)) || !finite_bound(elevation, 1e6) {
        return Err(invalid("feet/elevation bounds"));
    }
    let p = project(
        WorldPoint([feet[0], -feet[1], 0.]),
        &camera.target,
        &camera.viewport,
    )
    .map_err(|e| invalid(&e.to_string()))?;
    let y = p.y - elevation * camera.target.pose.zoom;
    if !finite_bound(p.x, 1e9) || !finite_bound(y, 1e9) {
        return Err(invalid("feet projection overflow"));
    }
    Ok([p.x, y])
}

fn json_bound(value: &serde_json::Value) -> Result<()> {
    let mut pending = vec![(value, 0usize)];
    let mut count = 0usize;
    while let Some((v, depth)) = pending.pop() {
        count += 1;
        if depth > 64 || count > 65536 {
            return Err(budget("JSON source depth/node budget"));
        }
        match v {
            serde_json::Value::Array(a) => {
                if pending.len() + a.len() > 65536 {
                    return Err(budget("JSON source node budget"));
                }
                pending.extend(a.iter().map(|v| (v, depth + 1)));
            }
            serde_json::Value::Object(a) => {
                if pending.len() + a.len() > 65536 {
                    return Err(budget("JSON source node budget"));
                }
                for (k, v) in a {
                    if k.len() > 1024 {
                        return Err(budget("JSON source key budget"));
                    }
                    pending.push((v, depth + 1));
                }
            }
            serde_json::Value::String(s) if s.len() > 1048576 => {
                return Err(budget("JSON source string budget"));
            }
            _ => {}
        }
    }
    Ok(())
}
// Capped writer bounds serialization before any source cloning.
struct Counter {
    left: usize,
}
impl std::io::Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > self.left {
            return Err(std::io::Error::other("source serialization budget"));
        }
        self.left -= bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn capped<T: serde::Serialize>(value: &T, bytes: usize) -> Result<()> {
    serde_json::to_writer(Counter { left: bytes }, value)
        .map_err(|_| budget("source serialization exceeds adapter budget"))
}
fn narrow(v: i128) -> Result<i64> {
    i64::try_from(v).map_err(|_| unsupported("legacy i64 arithmetic overflow"))
}
fn add(a: i64, b: i64) -> Result<i64> {
    narrow(i128::from(a) + i128::from(b))
}
fn sub(a: i64, b: i64) -> Result<i64> {
    narrow(i128::from(a) - i128::from(b))
}
fn mul(a: i64, b: i64) -> Result<i64> {
    narrow(i128::from(a) * i128::from(b))
}
fn coord(v: i64) -> Result<i64> {
    if v.unsigned_abs() > 1_000_000 {
        Err(unsupported("legacy geometry outside 1000000 pixels"))
    } else {
        Ok(v)
    }
}
#[derive(Clone, Copy)]
struct IntegerProjection {
    zoom: i64,
    tilt: i64,
    shear: i64,
    origin: [i64; 2],
    anchor: [i64; 2],
}
impl IntegerProjection {
    fn ground(self, x: i64, y: i64) -> Result<[i64; 2]> {
        coord(x)?;
        coord(y)?;
        let dx = sub(x, self.origin[0])?;
        let dy = sub(y, self.origin[1])?;
        let x = add(
            self.anchor[0],
            add(mul(dx, self.zoom)?, mul(dy, self.shear)?)? / 100,
        )?;
        let y = add(self.anchor[1], mul(mul(dy, self.zoom)?, self.tilt)? / 10000)?;
        Ok([coord(x)?, coord(y)?])
    }
    fn inverse(self, x: i64, y: i64) -> Result<[i64; 2]> {
        let dy = mul(sub(y, self.anchor[1])?, 10000)? / mul(self.zoom, self.tilt)?;
        Ok([
            add(
                self.origin[0],
                sub(mul(sub(x, self.anchor[0])?, 100)?, mul(dy, self.shear)?)? / self.zoom,
            )?,
            add(self.origin[1], dy)?,
        ])
    }
    fn upright(self, feet: [i64; 2], offset: [i64; 2]) -> Result<[i64; 2]> {
        let a = self.ground(feet[0], feet[1])?;
        Ok([
            coord(add(a[0], mul(offset[0], self.zoom)? / 100)?)?,
            coord(add(a[1], mul(offset[1], self.zoom)? / 100)?)?,
        ])
    }
}
fn integer_projection(project: &Project, camera: &CameraFrame) -> Result<IntegerProjection> {
    let v = &camera.viewport;
    let t = &camera.target;
    let tr = &t.view_transform;
    if v.x != 0.
        || v.y != 0.
        || v.width != f64::from(project.manifest.window.width)
        || v.height != f64::from(project.manifest.window.height)
    {
        return Err(unsupported(
            "legacy renderer requires matching zero-origin viewport",
        ));
    }
    let (half_height, near, far, focus_distance) = match t.projection {
        Projection::Orthographic {
            half_height,
            near,
            far,
            focus_distance,
        } => (half_height, near, far, focus_distance),
        Projection::Blended {
            half_height,
            near,
            far,
            focus_distance,
            perspective_weight: 0.,
        } => (half_height, near, far, focus_distance),
        _ => {
            return Err(unsupported(
                "legacy renderer supports orthographic or zero-weight blended lens only",
            ));
        }
    };
    if half_height != v.height / 2.
        || !identity(&t.pose.orientation)
        || !identity(&tr.rotation)
        || tr.scale[0] != 1.
        || tr.scale[2] != 1.
        || tr.origin[2] != 0.
        || !finite_bound(tr.shear_xy, 8.)
        || !tr.scale[1].is_finite()
        || !(0.001..=1000.).contains(&tr.scale[1])
        || !t.pose.zoom.is_finite()
        || !(0.001..=1000.).contains(&t.pose.zoom)
    {
        return Err(unsupported("legacy renderer rotation/scale/lens subset"));
    }
    if t.pose
        .position
        .iter()
        .chain(tr.origin.iter())
        .any(|x| !finite_bound(*x, 1e9))
        || !near.is_finite()
        || !far.is_finite()
        || !focus_distance.is_finite()
        || near < 0.001
        || near > 1e8
        || far - near < 0.001
        || far > 1e9
        || focus_distance < 0.001
        || focus_distance > 1e9
        || t.pose.position[2] < near
        || t.pose.position[2] > far
    {
        return Err(unsupported("legacy target numeric/depth bounds"));
    }
    let zoom = t.pose.zoom * 100.;
    let tilt = tr.scale[1] * 100.;
    let shear = -tr.shear_xy * t.pose.zoom * tr.scale[1] * 100.;
    if !zoom.is_finite()
        || !(1. ..=10000.).contains(&zoom)
        || !tilt.is_finite()
        || !(1. ..=10000.).contains(&tilt)
        || !finite_bound(shear, 10000.)
    {
        return Err(unsupported(
            "legacy rounded presentation percentages out of range",
        ));
    }
    // Invert H*S for camera-local follow displacement; native camera Z does not
    // affect orthographic scale. Source World Y down reverses native View Y.
    let origin = [
        tr.origin[0] + t.pose.position[0] - tr.shear_xy * t.pose.position[1],
        -tr.origin[1] - t.pose.position[1] / tr.scale[1],
    ];
    let half = [v.width / 2., v.height / 2.];
    if origin.iter().any(|x| !finite_bound(*x, 1e6)) {
        return Err(unsupported("legacy effective camera origin bounds"));
    }
    let camera_offset = [
        (origin[0] - half[0]).round() as i64,
        (origin[1] - half[1]).round() as i64,
    ];
    let anchor = [
        i64::from(project.manifest.window.width) / 2,
        i64::from(project.manifest.window.height) / 2,
    ];
    Ok(IntegerProjection {
        zoom: zoom.round() as i64,
        tilt: tilt.round() as i64,
        shear: shear.round() as i64,
        origin: [
            add(camera_offset[0], anchor[0])?,
            add(camera_offset[1], anchor[1])?,
        ],
        anchor,
    })
}
fn array_i64(
    value: Option<&serde_json::Value>,
    name: &str,
    defaults: [i64; 2],
) -> Result<[i64; 2]> {
    match value {
        None => Ok(defaults),
        Some(v) => {
            let a = v
                .as_array()
                .filter(|a| a.len() == 2)
                .ok_or_else(|| invalid(name))?;
            Ok([
                a[0].as_i64().ok_or_else(|| invalid(name))?,
                a[1].as_i64().ok_or_else(|| invalid(name))?,
            ])
        }
    }
}
fn scalar(value: Option<&serde_json::Value>, name: &str) -> Result<i64> {
    match value {
        None => Ok(0),
        Some(v) => v.as_i64().ok_or_else(|| invalid(name)),
    }
}
fn texture(project: &Project, name: &str) -> Result<usize> {
    text(name)?;
    let t = project
        .textures
        .get(name)
        .ok_or_else(|| invalid("missing texture"))?;
    if !(1..=4096).contains(&t.width) || !(1..=4096).contains(&t.height) {
        return Err(unsupported("texture dimensions outside 1..4096"));
    }
    let bytes = u64::from(t.width) * u64::from(t.height) * 4;
    if bytes != t.rgba.len() as u64 {
        return Err(invalid("texture RGBA length mismatch"));
    }
    // Legacy texture indices use u32 throughout, including the final *4.
    if bytes > u64::from(u32::MAX) {
        return Err(unsupported("texture u32 index overflow"));
    }
    Ok(t.rgba.len())
}
fn preflight_building(
    project: &Project,
    b: &ge4g_project::building::Building,
    p: IntegerProjection,
    origin: [i64; 2],
    assets: &mut BTreeSet<String>,
) -> Result<()> {
    if !b.valid() {
        return Err(invalid("invalid building geometry"));
    }
    for s in b.surfaces() {
        texture(project, s)?;
        assets.insert(s.clone());
    }
    let w = i64::from(b.footprint[0]);
    let d = i64::from(b.footprint[1]);
    let h = mul(i64::from(b.height), p.zoom)? / 100;
    let rise = mul(15, p.zoom)? / 100;
    let corners = [
        p.ground(origin[0], origin[1])?,
        p.ground(add(origin[0], w)?, origin[1])?,
        p.ground(origin[0], add(origin[1], d)?)?,
        p.ground(add(origin[0], w)?, add(origin[1], d)?)?,
    ];
    // A conservative envelope covers every facade, roof ridge, overhang, door
    // and fallback outline vertex. Check all edge products before compositor.
    let minx = sub(
        corners
            .iter()
            .map(|a| a[0])
            .min()
            .ok_or_else(|| invalid("building corners"))?,
        4,
    )?;
    let maxx = add(
        corners
            .iter()
            .map(|a| a[0])
            .max()
            .ok_or_else(|| invalid("building corners"))?,
        4,
    )?;
    let miny = sub(
        sub(
            corners
                .iter()
                .map(|a| a[1])
                .min()
                .ok_or_else(|| invalid("building corners"))?,
            h,
        )?,
        rise,
    )?;
    let maxy = *corners
        .iter()
        .map(|a| &a[1])
        .max()
        .ok_or_else(|| invalid("building corners"))?;
    for v in [minx, maxx, miny, maxy] {
        coord(v)?;
    }
    let sx = sub(maxx, minx)?;
    let sy = sub(maxy, miny)?;
    mul(sx, sy)?;
    mul(add(maxx, maxx)?, 1)?;
    mul(h, 3)?;
    for s in b.surfaces() {
        let t = &project.textures[s];
        mul(sx, i64::from(t.width))?;
        mul(sy, i64::from(t.height))?;
    }
    Ok(())
}
fn preflight(
    project: &Project,
    snapshot: &Snapshot,
    p: IntegerProjection,
) -> Result<BTreeSet<String>> {
    if project.manifest.features.len() > 256 {
        return Err(budget("manifest feature count exceeds 256"));
    }
    let width = project.manifest.window.width;
    let height = project.manifest.window.height;
    if !(1..=4096).contains(&width)
        || !(1..=4096).contains(&height)
        || u64::from(width) * u64::from(height) > 4_194_304
    {
        return Err(budget("frame dimensions/4Mi pixel budget"));
    }
    if snapshot.entities.len() > 256 {
        return Err(budget("render entities exceeds 256"));
    }
    text(&snapshot.scene)?;
    text(&project.manifest.name)?;
    if let Some(v) = &snapshot.flatland {
        if !v.is_object() {
            return Err(invalid("flatland not an object"));
        }
        json_bound(v)?;
    }
    for e in &snapshot.entities {
        if let Some(v) = &e.flatland {
            json_bound(v)?;
        }
        for v in e.metadata.values() {
            json_bound(v)?;
        }
    }
    if snapshot.events.len() > 4096 || snapshot.state.len() > 256 {
        return Err(budget("snapshot event/state count"));
    }
    for e in &snapshot.events {
        json_bound(&e.data)?;
    }
    for v in snapshot.state.values() {
        json_bound(v)?;
    }
    capped(snapshot, 8 * 1024 * 1024)?;
    let scene = project
        .scenes
        .get(&snapshot.scene)
        .ok_or_else(|| invalid("snapshot scene missing"))?;
    let systems = snapshot.flatland.as_ref().and_then(|f| f.get("systems"));
    if let Some(s) = systems {
        if !s.is_object() {
            return Err(invalid("systems not an object"));
        }
        if s.get("attacks")
            .is_some_and(|a| !a.is_null() && a.as_array().is_none_or(|a| !a.is_empty()))
            || s.get("events").and_then(|a| a.as_array()).is_some_and(|a| {
                a.iter()
                    .any(|e| e.get("battle").is_some_and(|b| !b.is_null()))
            })
        {
            return Err(unsupported("attack/battle render path is not validated"));
        }
    }
    let mut inverse = Vec::new();
    for x in [0, i64::from(width)] {
        for y in [0, i64::from(height)] {
            inverse.push(p.inverse(x, y)?);
        }
    }
    if let Some(map) = &scene.map {
        if map.cell == 0
            || map.rows.len() > 4096
            || map.tiles.len() > 128
            || map.rows.iter().any(|r| r.len() > 4096 || !r.is_ascii())
        {
            return Err(invalid("map cell/rows require bounded ASCII"));
        }
        if map
            .rows
            .iter()
            .try_fold(0usize, |n, r| n.checked_add(r.len()))
            .is_none_or(|n| n > 1048576)
        {
            return Err(budget("map rows exceed 1MiB"));
        }
        if map.tiles.keys().any(|k| k.len() != 1 || !k.is_ascii()) {
            return Err(invalid("map tile identifiers must be one ASCII byte"));
        }
        for row in &map.rows {
            for c in row.chars() {
                if !map.tiles.contains_key(&c.to_string()) {
                    return Err(invalid("map has undeclared tile"));
                }
            }
        }
        if let Some(patches) = systems.and_then(|s| s.get("patches")) {
            let patches = patches
                .as_object()
                .ok_or_else(|| invalid("patches not an object"))?;
            for (key, value) in patches {
                if let Some(at) = key.strip_prefix(&format!("{}:", snapshot.scene)) {
                    let parts: Vec<_> = at.split(':').collect();
                    if parts.len() != 2 {
                        return Err(invalid("patch coordinate malformed"));
                    }
                    let x = parts[0]
                        .parse::<usize>()
                        .map_err(|_| invalid("patch x malformed"))?;
                    let y = parts[1]
                        .parse::<usize>()
                        .map_err(|_| invalid("patch y malformed"))?;
                    let tile = value
                        .as_str()
                        .ok_or_else(|| invalid("patch tile not a string"))?;
                    if tile.len() != 1
                        || !tile.is_ascii()
                        || !map.tiles.contains_key(tile)
                        || y >= map.rows.len()
                        || x >= map.rows[y].len()
                    {
                        return Err(invalid("patch outside map or invalid tile"));
                    }
                }
            }
        }
    }
    let mut assets = BTreeSet::new();
    let mut ids = BTreeSet::new();
    for e in &snapshot.entities {
        text(&e.id)?;
        if !ids.insert(&e.id) {
            return Err(invalid("duplicate snapshot entity identity"));
        }
        if e.size.iter().any(|n| *n == 0 || *n > 4096) {
            return Err(unsupported("entity size outside 1..4096"));
        }
        let wx = coord(e.position.x.div_euclid(SUBPIXELS))?;
        let wy = coord(e.position.y.div_euclid(SUBPIXELS))?;
        add(e.position.y, mul(i64::from(e.size[1]), SUBPIXELS)?)?;
        let a = e.flatland.as_ref();
        if a.is_some_and(|a| !a.is_object()) {
            return Err(invalid("entity flatland must be an object"));
        }
        let anchor = array_i64(a.and_then(|a| a.get("anchor")), "anchor", [0, 0])?;
        let z = scalar(a.and_then(|a| a.get("z")), "elevation")?;
        for v in [anchor[0], anchor[1], z] {
            coord(v)?;
        }
        let visual = array_i64(
            a.and_then(|a| a.get("visual_size")),
            "visual_size",
            [i64::from(e.size[0]), i64::from(e.size[1])],
        )?;
        if visual.iter().any(|v| !(1..=4096).contains(v)) {
            return Err(unsupported("visual size outside 1..4096"));
        }
        for name in ["hp", "hp_max"] {
            if let Some(v) = a.and_then(|a| a.get(name)).filter(|v| !v.is_null()) {
                let n = v.as_i64().ok_or_else(|| invalid("hp not i64"))?;
                if n.unsigned_abs() > 1_000_000_000 {
                    return Err(unsupported("hp arithmetic bounds"));
                }
            }
        }
        let feet = [
            add(wx, i64::from(e.size[0]) / 2)?,
            add(wy, i64::from(e.size[1]))?,
        ];
        let ox = sub(anchor[0], i64::from(e.size[0]) / 2)?;
        let oy = sub(sub(anchor[1], i64::from(e.size[1]))?, z)?;
        let upright = p.upright(feet, [ox, oy])?;
        let gx = add(wx, anchor[0])?;
        let gy = sub(add(wy, anchor[1])?, z)?;
        for x in [gx, add(gx, visual[0])?] {
            for y in [gy, add(gy, visual[1])?] {
                p.ground(x, y)?;
            }
        }
        let sw = mul(visual[0], p.zoom)? / 100;
        let sh = mul(visual[1], p.zoom)? / 100;
        if sw > 1_000_000 || sh > 1_000_000 {
            return Err(unsupported("scaled visual dimensions"));
        }
        coord(add(upright[0], sw)?)?;
        coord(add(upright[1], sh)?)?;
        sub(upright[1], 3)?;
        for xy in &inverse {
            sub(sub(xy[0], wx)?, anchor[0])?;
            add(sub(sub(xy[1], wy)?, anchor[1])?, z)?;
        }
        mul(sub(i64::from(width), upright[0])?, 100)?;
        mul(sub(i64::from(height), upright[1])?, 100)?;
        // Shadow arithmetic and bounded loop count (building radius can grow).
        let building = a.and_then(|a| a.get("building"));
        let rx = mul(
            if building.is_some() {
                i64::from(e.size[0]) / 2
            } else {
                7
            },
            p.zoom,
        )? / 100;
        if rx > 4096 {
            return Err(unsupported("shadow radius exceeds 4096"));
        }
        mul(mul(mul(rx, rx)?, 4)?, 2)?;
        let fp = p.ground(feet[0], feet[1])?;
        add(fp[0], rx)?;
        sub(fp[0], rx)?;
        if let Some(b) = building {
            capped(b, 16384)?;
            let b: ge4g_project::building::Building =
                serde_json::from_value(b.clone()).map_err(|_| invalid("building shape"))?;
            preflight_building(project, &b, p, [wx, wy], &mut assets)?;
        }
        if let Some(name) = &e.texture {
            texture(project, name)?;
            assets.insert(name.clone());
            let t = &project.textures[name];
            let crop = match a.and_then(|a| a.get("atlas")) {
                None => [0, 0, i64::from(t.width), i64::from(t.height)],
                Some(v) => {
                    let c = v
                        .as_array()
                        .filter(|c| c.len() == 4)
                        .ok_or_else(|| invalid("atlas shape"))?;
                    let mut crop = [0; 4];
                    for (i, v) in c.iter().enumerate() {
                        crop[i] = v.as_i64().ok_or_else(|| invalid("atlas integer"))?;
                    }
                    crop
                }
            };
            if crop[0] < 0
                || crop[1] < 0
                || crop[2] < 1
                || crop[3] < 1
                || add(crop[0], crop[2])? > i64::from(t.width)
                || add(crop[1], crop[3])? > i64::from(t.height)
            {
                return Err(invalid("atlas outside texture"));
            }
            for (size, extent) in [(visual[0], crop[2]), (visual[1], crop[3])] {
                if i128::from(size) * i128::from(extent) > i128::from(u32::MAX) {
                    return Err(unsupported("texture sampling u32 overflow"));
                }
            }
        }
        // Debug rectangles and default avatar squared-distance arithmetic.
        add(wx, i64::from(e.size[0]))?;
        add(wy, i64::from(e.size[1]))?;
        mul(visual[0], visual[0])?;
        mul(visual[1], visual[1])?;
    }
    let mut bytes = 0usize;
    for name in &assets {
        bytes = bytes
            .checked_add(texture(project, name)?)
            .ok_or_else(|| budget("texture budget overflow"))?;
    }
    if assets.len() > 256 || bytes > 64 * 1024 * 1024 {
        return Err(budget("texture count/64MiB budget"));
    }
    Ok(assets)
}
/// Render actual content with native CameraFrame-derived integer presentation.
/// Unsupported native transforms return an error, never an old-camera fallback.
pub fn render_legacy(
    project: &Project,
    snapshot: &Snapshot,
    camera: &CameraFrame,
    debug: bool,
) -> Result<ge4g_render2d::Frame> {
    if camera.scene.local != key("s_", &snapshot.scene) || camera.scene.incarnation == 0 {
        return Err(invalid(
            "CameraFrame scene differs from legacy source snapshot",
        ));
    }
    let p = integer_projection(project, camera)?;
    let assets = preflight(project, snapshot, p)?;
    let scene = project
        .scenes
        .get(&snapshot.scene)
        .ok_or_else(|| invalid("scene missing"))?;
    // Build only fields used by the CPU compositor: arbitrary gameplay programs,
    // metadata and unrelated scenes are never cloned into this consumer.
    let map = scene.map.as_ref().map(|m| ge4g_project::flatland::Map {
        cell: m.cell,
        rows: m.rows.clone(),
        tiles: m
            .tiles
            .iter()
            .map(|(k, t)| {
                (
                    k.clone(),
                    ge4g_project::flatland::Tile {
                        color: t.color,
                        solid: false,
                        spawn: None,
                        planes: Vec::new(),
                    },
                )
            })
            .collect(),
        portals: Vec::new(),
    });
    let scene = ge4g_project::Scene {
        schema_version: scene.schema_version,
        id: snapshot.scene.clone(),
        background: scene.background,
        camera: [0, 0],
        spawns: BTreeMap::new(),
        on_enter: BTreeMap::new(),
        entities: Vec::new(),
        map,
        prefabs: BTreeMap::new(),
        rules: Vec::new(),
        script: None,
        sounds: BTreeMap::new(),
        resources: Vec::new(),
        gameplay: Default::default(),
    };
    let manifest = ge4g_project::Manifest {
        schema_version: project.manifest.schema_version,
        name: project.manifest.name.clone(),
        start_scene: snapshot.scene.clone(),
        window: project.manifest.window.clone(),
        scenes: BTreeMap::new(),
        state: BTreeMap::new(),
        tests: Vec::new(),
        features: if project
            .manifest
            .features
            .iter()
            .any(|f| f == "entity_defaults")
        {
            vec!["entity_defaults".into()]
        } else {
            Vec::new()
        },
    };
    let presentation = Project {
        root: Default::default(),
        manifest,
        scenes: BTreeMap::from([(snapshot.scene.clone(), scene)]),
        textures: assets
            .into_iter()
            .map(|name| {
                let t = project.textures[&name].clone();
                (name, t)
            })
            .collect(),
        files_checked: Vec::new(),
        scripts: BTreeMap::new(),
    };
    // Disable the old followed-actor special origin by installing an explicit
    // camera marker only on this presentation copy. All gameplay input untouched.
    let mut copy = snapshot.clone();
    copy.camera = [
        sub(p.origin[0], p.anchor[0])?,
        sub(p.origin[1], p.anchor[1])?,
    ];
    let flatland = copy.flatland.get_or_insert_with(|| serde_json::json!({}));
    if !flatland.is_object() {
        return Err(invalid("flatland not an object"));
    }
    if flatland.get("systems").is_none() {
        flatland["systems"] = serde_json::json!({});
    }
    flatland["systems"]["camera"] = serde_json::json!({"presentation_adapter":true});
    flatland["systems"]["gameplay_view"] =
        serde_json::json!({"zoom":p.zoom,"tilt":p.tilt,"shear":p.shear});
    // Current scene is complete, its original gameplay camera remains inert due
    // to the explicit marker. No gameplay/world or record ownership is changed.
    ge4g_render2d::render(&presentation, &copy, debug)
        .map_err(|e| CompatibilityError::new(CompatibilityErrorCode::RenderFailed, e.to_string()))
}

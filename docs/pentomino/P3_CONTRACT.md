# P3 Camera / View / Format contract v1

**FROZEN v1 — parent integration after independent Math/Transform and Core
Boundary design reviews. Internal implementation/acceptance are verified by
P3_RUNDOWN.md; external tests PASS by user report (2026-10-09).** Baseline accepted P2 `2159304`
on the stacked `pentomino/p3-camera` branch; main/P2 merge status must remain
explicit in the parent plan. P2 public Core external23/23 is accepted, external
legacy tests additionally accepted by user report (2026-10-09). Both independent design reviews accepted the corrected contract; review
evidence is P3_CONTRACT_REVIEW.md. No Tiny Core change is required.

The observable result is the same authoritative game with a replaceable way
of viewing it. No Tiny Core/P1/P2 adapter/runtime/project/render2d modification
is proposed. There are two new crates:

- `ge4g-pentomino-view`: native read-only Format/View/Camera host, math,
  declarative behavior, render-facing output and canonical presentation save1.
- `ge4g-pentomino-view-legacy`: explicit legacy snapshot extraction and a
  supported-subset render adapter; depends on native view and legacy public
  core/project/runtime/render2d APIs. P2 source-label conventions are reused;
the importer does not need a P2 legacy bridge dependency. No reverse dependency.

The native crate depends on `ge4g-pentomino`, serde/serde_json (with
`float_roundtrip` enabled), sha2 and
thiserror only. Its production surface consumes `&p2::ReadFrame`, never
`&mut CoreHost`, CoreTransaction, World or legacy renderer types. It does not
install itself in the authoritative host. Applications choose a Selection and
call CoreHost::select; changing presentation need not advance Core at all.
Trusted linked Format plugins are pure extractors. This is process architecture,
not sandboxing arbitrary native code or prohibiting an application from holding
unrelated mutable references elsewhere.

## Coordinate model and exact native math

Native world/view/camera coordinates use a right-handed basis. Camera looks
along **negative Z**; view-space up is positive Y, screen Y goes downward.
This declared convention is a presentation convention, not a gameplay genre.
Legacy pixel-world Y down is handled by the compatibility extractor/adapter.
World, View, Camera, Projected and Screen are distinct public point types;
there is no global camera/transform.

All math uses bounded finite f64. Guarantees are deterministic on the same
target/toolchain; arbitrary-platform transcendental/float byte equality is
UNVERIFIED. Normalize negative zero to positive zero at accepted state/output
boundaries. Quaternions normalize and choose sign by the first nonzero member
of `[w,x,y,z]` positive. Reject norm below1e-12 or nonfinite; normalization is
explicit, not accepting arbitrary singular transforms.

```rust
pub struct WorldPoint(pub [f64; 3]);
pub struct ViewPoint(pub [f64; 3]);
pub struct CameraPoint(pub [f64; 3]);
pub struct ScreenPoint { pub x: f64, pub y: f64, pub depth: f64 }
pub struct Quaternion { pub x: f64, pub y: f64, pub z: f64, pub w: f64 }
pub struct ViewTransform {
    pub origin: [f64; 3], pub rotation: Quaternion,
    pub scale: [f64; 3], pub shear_xy: f64,
}
pub struct CameraPose {
    pub position: [f64; 3], pub orientation: Quaternion, pub zoom: f64,
}
pub struct Viewport { pub x: f64, pub y: f64, pub width: f64, pub height: f64 }
pub enum Projection {
    Orthographic { half_height: f64, near: f64, far: f64, focus_distance: f64 },
    Perspective { half_height: f64, near: f64, far: f64, focus_distance: f64 },
    Blended { half_height: f64, near: f64, far: f64,
              focus_distance: f64, perspective_weight: f64 },
}
pub struct CameraTarget {
    pub pose: CameraPose, pub view_transform: ViewTransform, pub projection: Projection,
}
```

World→View is `R * H * S * (world-origin)`, with positive diagonal S and
`H(x,y,z)=(x+shear_xy*y,y,z)`. View→Camera is inverse camera orientation
applied to `(view-camera.position)`. Explicit inverse uses the inverse factors
in reverse order, never a naive lerp of arbitrary invertible matrices.
CameraPoint depth is `d=-z`. The shared lens lowers all projection variants to
`(half_height,near,far,focus_distance,b)` where b is0/1 for the endpoints.
For a point in the inclusive accepted depth interval `[near,far]`:

```
D = (1-b)*focus_distance + b*d
h = half_height / zoom
aspect = viewport.width / viewport.height
ndc_x = camera.x * focus_distance / (h * aspect * D)
ndc_y = camera.y * focus_distance / (h * D)
screen.x = viewport.x + (ndc_x + 1)*viewport.width/2
screen.y = viewport.y + (1 - ndc_y)*viewport.height/2
screen.depth = (d-near)/(far-near)
```

b=0 is genuine orthographic, b=1 is genuine perspective (focal plane extent
at focus_distance), including perspective size change with depth. Blended is
a finite positive-denominator intermediate, not a fake unsupported enum.
Frustum X/Y clipping affects visibility, not algebraic invertibility. Public
project accepts offscreen X/Y and reports the screen point; outside near/far
returns OutsideDepth. `unproject` uses supplied normalized depth to recover d;
screen X/Y alone cannot recover a3D point. Render selection culls offscreen
X/Y and OutsideDepth without turning an entire update into an error.

```rust
pub fn world_to_view(point: WorldPoint, transform: &ViewTransform) -> Result<ViewPoint, Error>;
pub fn view_to_world(point: ViewPoint, transform: &ViewTransform) -> Result<WorldPoint, Error>;
pub fn view_to_camera(point: ViewPoint, pose: &CameraPose) -> Result<CameraPoint, Error>;
pub fn camera_to_view(point: CameraPoint, pose: &CameraPose) -> Result<ViewPoint, Error>;
pub fn project(point: WorldPoint, target: &CameraTarget, viewport: &Viewport) -> Result<ScreenPoint, Error>;
pub fn unproject(point: ScreenPoint, target: &CameraTarget, viewport: &Viewport) -> Result<WorldPoint, Error>;
```

WorldPoint/origin/pose.position absolute component≤1e9; intermediate
ViewPoint/CameraPoint coordinates may exceed that bound after scaling and
need only remain finite. Inverse results must meet the WorldPoint bound. positive scale .001..1000,
shear -8..8; zoom .001..1000; half_height/focus_distance .001..1e9;
near .001..1e8; far≤1e9 and far-near≥.001. Viewport origin absolute≤1e6,
width/height .001..16384. Screen.depth0..1. Check every intermediate finite
and final inverse WorldPoint bound; return InvalidTransform/Overflow instead
of clamping to NaN or silently truncating. A round-trip test uses
`abs(a-b) <= 1e-7 * max(1,abs(a),abs(b))` for the tested well-conditioned
domain; extreme near-limit inputs require finite result or explicit error,
not a universal1e-7 claim.

## Format selection and render-facing representations

ViewId, CameraId and FormatId are distinct private-string newtypes with
`new(&str)->Result<Self,Error>` and `as_str()->&str`, dotted ASCII P2 ID rules
and128bytes maximum. Stable EntityRef/SceneRef are consumed as data only.
The native host does not issue authority handles; removed IDs can be reused,
so cached IDs identify the current presentation namespace, not freshness.

```rust
pub enum ViewFamily { Classic2D, TopDown, Side, Vertical }
pub struct RecordBinding {
    pub owner: PluginId, pub schema: SchemaId,
    pub entity_field: String, pub x_field: String, pub y_field: String,
    pub z_field: Option<String>, pub units_per_world: f64,
    pub representation: Representation,
}
pub enum Representation {
    Sprite { asset: String, size: [f64; 2] },
    Model { asset: String }, Background { asset: String },
    Billboard { asset: String, size: [f64; 2] },
}
pub struct PresentationItem {
    pub entity: EntityRef, pub world: WorldPoint, pub representation: Representation,
}
pub struct PresentationSelection { pub scene: SceneRef, pub items: Vec<PresentationItem> }
pub struct FormatDescriptor {
    pub id: FormatId, pub version: u32, pub family: ViewFamily,
    pub required_fields: BTreeMap<String, String>,
    pub representations: Vec<String>,
}
pub trait FormatPlugin {
    fn descriptor(&self) -> FormatDescriptor;
    fn extract(&self, read: &ReadFrame, binding: &RecordBinding, scene: &SceneRef)
        -> Result<PresentationSelection, Error>;
}
pub struct Classic2DFormat;
pub struct TopDownFormat;
pub struct SideFormat;
pub struct VerticalFormat;
```

These four builtin plugins are actual extractors, not just available-view
labels. They may share strict extraction code, while family-specific policy
comes from composed configuration below. `required_fields` describes semantic
slots `entity:ref(entity)`, `x:i64`, `y:i64`, optional `z:i64`; concrete field
names/schema/owner live in RecordBinding. Core has no position schema; ordinary
gameplay or compatibility source plugins may declare i64 records with any
names. Only matching owner/schema records are read. Missing/wrong-typed fields,
wrong/dead references, duplicate entity presentation or nonfinite conversion
are explicit InvalidSelection, no implicit defaults except absent configured
z means worldZ0. `units_per_world` positive .001..1e9. Required objects must
be included in ReadFrame, the scene must exist, entity references must resolve against the included objects. Records for valid
entities in other Scenes are validated and then filtered out; Core Selection
need not be manually edited to select one Scene. Unrelated schemas/owners are
ignored. Output items sort by EntityRef.

Assets are bounded UTF-8 identifiers≤1024bytes, not embedded pixels/mesh blobs.
Model/Background/Billboard descriptors preserve a3D world point and native
project handles true perspective. P3 exposes these render-facing descriptors;
GPU mesh rendering and mixed-scene renderer completeness remain UNVERIFIED.
Do not advertise finished3D rendering because these types exist.

## View policy and reusable behaviors

```rust
pub struct AxisMask { pub x: bool, pub y: bool, pub z: bool }
pub struct Bounds { pub min: [f64; 3], pub max: [f64; 3] }
pub struct ViewPolicy { pub family: ViewFamily, pub follow_axes: AxisMask }
pub struct ViewConfig {
    pub id: ViewId, pub scene: SceneRef, pub binding: RecordBinding, pub policy: ViewPolicy,
}
pub struct FollowBehavior {
    pub target: EntityRef, pub offset: [f64; 3],
    pub dead_zone: [f64; 3], pub look_ahead_ticks: u32, pub smoothing_ticks: u32,
}
pub struct ShakeBehavior { pub seed: u64, pub amplitude: [f64; 3], pub duration_ticks: u32 }
pub struct CameraBehaviors {
    pub follow: Option<FollowBehavior>, pub bounds: Option<Bounds>,
    pub shake: Option<ShakeBehavior>, pub zoom_range: [f64; 2],
}
pub enum TransitionMode { Instant, Smooth { duration_ticks: u32 } }
pub struct CameraConfig {
    pub id: CameraId, pub view: ViewId, pub viewport: Viewport,
    pub target: CameraTarget, pub behaviors: CameraBehaviors,
    pub default_transition: TransitionMode,
}
pub struct CameraChange {
    pub camera: CameraId, pub target: CameraTarget, pub transition: Option<TransitionMode>,
}
pub struct ViewInput { pub target_tick: u64, pub changes: Vec<CameraChange> }
```

`ViewPolicy::classic2d/top_down/side/vertical()->Self` return distinct
policies: Classic2D and TopDown follow X/Y; Side follows X with fixed baseY;
Vertical follows Y with fixed baseX; all initially ignoreZ. The family is
not a TopDown boolean switch. Explicit masks can combine tracking axes without
duplicating math or authoritative game logic. follow_axes masks only the final
View-space displacement; it never also masks World tracking coordinates. ViewPolicy family must match
installed Format descriptor. Static means no follow. Side/Vertical share
FollowBehavior but use separate policy axes; look-ahead is configurable.

Behavior order is exact:

1. Sample base CameraTarget transition at the current presentation tick.
2. Extract followed entity. Tracking anchor, offset, dead-zone and prior
   entity point use World space on all three axes. First observation initializes the
   anchor to the selected point; no velocity spike.
3. Dead-zone per axis: hold prior anchor while point-anchor lies within the
   inclusive±dead_zone; otherwise move desired anchor to the nearest dead-zone
   edge. Compute current entity delta from previous entity World point,
   multiplied by look_ahead_ticks and add it on all World axes. All values bounded.
4. Smooth anchor with `anchor += (desired-anchor)/smoothing_ticks` for
   smoothing_ticks>0; zero means immediate. This is a fixed-tick tracking
   filter, not a repeatedly restarted pose transition. Save prior entity
   World point and World anchor. Follow offset is added to the anchor. Map
   anchor through the current sampled World→View transform; disabled axes
   contribute zero to the resulting View-space displacement. Add this
   displacement to the base pose position. A changing View transform never
   masquerades as entity world velocity in look-ahead.
5. Add deterministic shake, then clamp final position to optional Bounds.
   Shake is pure keyed counter noise from seed, CameraId and saved active
   update count; never draws Tiny RNG. Key is seed XOR first8 SHA256(CameraId)
   bytes as little-endian u64 XOR count*0x9e3779b97f4a7c15 XOR
   axis*0xd1b54a32d192ed03. SplitMix64 adds0x9e3779b97f4a7c15,
   xor-shifts30/27/31 with multipliers0xbf58476d1ce4e5b9 and
   0x94d049bb133111eb, all wrapping u64; upper53bits divide by2^53-1
   then map to[-1,1]. Duration counts only active updates;
   after duration displacement zero. Exact SplitMix64 modular operations and
   integer-to[-1,1] conversion must be documented in implemented API evidence.
6. Validate final pose/projector; project selected representations to output.

Follow is a camera-local displacement; explicit pose.position targets the
base camera, not a command to move the followed entity. Bounds min≤max and
each coordinate≤1e9. Dead-zone/offset/amplitude finite bounded; lookahead≤120,
smoothing≤4096, shake duration≤4096; zoom_range valid positive subrange of
.001..1000 and every requested pose zoom must lie within it. A follow target
missing from the current validated selection returns InvalidSelection and
rolls back the entire presentation update; callers can remove/change camera
configuration to release that dependency. It does not create a Core dependency.

## Smooth2.5D transitions

`CameraConfig::new(id,view,viewport,target)->Self` defaults to static behavior
with no bounds/shake, zoom range .001..1000 and **Smooth12ticks**. Classic2D
callers can explicitly choose Instant. The default is a reusable transition
policy, not an effect wired into the Nuvema fixture or renderer.

On valid input at the next tick, process changes in sorted CameraId order.
At most one change per camera. Retarget begins at the exact already sampled
base target at the previous committed tick, then evaluates the new transition
for one tick of progress. Thus a request has no hidden instantaneous pose
jump; a visible sampled next-tick movement is expected. Instant samples the
new target immediately. `None` uses the saved camera default transition.
Smooth duration1..4096; elapsed0..duration, no wall clock/frame-rate input.

Interpolation parameter `u=elapsed/duration`, `t=u*u*(3-2*u)`. Linear
interpolation uses bounded convex endpoints for positions, zoom, view origin,
positive scales, shear, lens half_height/near/focus_distance/weight and
positive far-minus-near interval. Clamp rounded scalar samples to the convex
endpoint interval; derive far from sampled near+sampled width (cap1e9). If
rounded subtraction makes width<.001, outward-round (near+.001).next_up().
Exact endpoints retain requested near/far. This explicit precision repair
prevents valid large-near narrow intervals failing during smooth transition.
Orientation and View rotation use shortest-hemisphere normalized linear
quaternion interpolation (NLERP): negate destination iff dot<0; at dot0 use
canonical destination sign. This gives continuous rotation, not constant
angular speed or a claimed SLERP algorithm. Canonical quaternion component
sign can change at the180° representation boundary while the physical
rotation stays continuous. Interpolation of lens variants lowers to one
Blended parameterization; reaching endpoint uses exactly requested endpoint
variant. Near/far constraints survive convex mixing with the stated precision repair;
the lens denominator stays positive.
Use exact start/target when elapsed0/duration to avoid endpoint drift.

ViewTransform is part of each CameraTarget, so changing an actual viewing
mode's tilt/shear/orientation can transition smoothly per camera. Cameras in
one View need not share a transform. Retarget changes position/orientation/
zoom/lens/viewing mode together and saves all state. It guarantees C0 base
pose continuity, not velocity continuity (C1). Follow remains a separate
tracking filter. New targets during transition do not reset follow tracking
history, shake counter or unrelated cameras. All transitions are derived
from input/ticks; game state and Tiny RNG remain untouched.

## Host lifecycle and output

```rust
pub struct RenderItem {
    pub entity: EntityRef, pub world: WorldPoint, pub screen: ScreenPoint,
    pub representation: Representation,
}
pub struct CameraFrame {
    pub camera: CameraId, pub view: ViewId, pub scene: SceneRef,
    pub viewport: Viewport, pub target: CameraTarget, pub items: Vec<RenderItem>,
}
pub struct RenderFrame { pub tick: u64, pub source_tick: u64, pub cameras: Vec<CameraFrame> }
pub struct CameraStatus {
    pub id: CameraId, pub view: ViewId, pub active: bool,
    pub current: CameraTarget, pub transitioning: bool,
}
pub struct ViewStatus {
    pub config: ViewConfig, pub format: FormatDescriptor, pub cameras: Vec<CameraId>,
}
pub struct ViewDiscovery {
    pub contract_version: u32, pub tick: u64, pub available_formats: Vec<FormatDescriptor>,
    pub installed_views: Vec<ViewStatus>, pub cameras: Vec<CameraStatus>,
    pub projections: Vec<String>, pub behaviors: Vec<String>, pub limits: ViewLimits,
}
pub struct ViewHost { /* private staged state and linked plugins */ }
impl ViewHost {
    pub fn new(content_binding: &str) -> Result<Self, Error>;
    pub fn install(&mut self, plugin: Box<dyn FormatPlugin>, config: ViewConfig,
                   read: &ReadFrame) -> Result<(), Error>;
    pub fn replace(&mut self, id: &ViewId, plugin: Box<dyn FormatPlugin>,
                   config: ViewConfig, read: &ReadFrame) -> Result<(), Error>;
    pub fn remove(&mut self, id: &ViewId) -> Result<(), Error>;
    pub fn add_camera(&mut self, config: CameraConfig, read: &ReadFrame) -> Result<(), Error>;
    pub fn remove_camera(&mut self, id: &CameraId) -> Result<(), Error>;
    pub fn activate(&mut self, id: &CameraId) -> Result<(), Error>;
    pub fn deactivate(&mut self, id: &CameraId) -> Result<(), Error>;
    pub fn set_active(&mut self, view: &ViewId, cameras: BTreeSet<CameraId>) -> Result<(), Error>;
    pub fn update(&mut self, read: &ReadFrame, input: ViewInput) -> Result<RenderFrame, Error>;
    pub fn frame(&self, read: &ReadFrame) -> Result<RenderFrame, Error>;
    pub fn describe(&self) -> ViewDiscovery;
    pub fn save(&self) -> Result<Vec<u8>, Error>;
    pub fn hash(&self) -> Result<String, Error>;
    pub fn restore(&mut self, bytes: &[u8], read: &ReadFrame) -> Result<(), Error>;
}
```

Install includes initialize: descriptor/config/selection validate before
commit; no camera exists until add_camera. Newly added camera is inactive,
at its exact initial base target, tracking not yet initialized. activate does
not deactivate other cameras. `set_active` accepts any bounded set belonging
to that View, including empty; duplicates impossible in BTreeSet. Multiple
active cameras can observe the same Scene with independent transform/viewport,
including minimap/split-screen structure without complete client UI.

Remove View deletes only its own cameras/local state. replace requires
config.id==id; preserves existing cameras/base transitions and active flags,
while revalidating selection/follow and all current/endpoint transforms under
new policy before commit. This allows TopDown→Classic2D or Side→Vertical
without authoritative mutation; explicit remove/reinstall also works.
Any unrelated View/camera state stays byte-for-byte unchanged. Host callbacks
cannot write source data through the provided read-only signature.

CameraStatus.current is the sampled base target (describe has no source
frame); CameraFrame.target includes current follow/shake/bounds output.

Presentation time is independent from authoritative simulation time: host
starts tick0, ViewInput.target_tick must equal host.tick+1. source ReadFrame
tick is recorded in returned frame, not forced equal to presentation tick.
A frozen read frame can therefore prove exact whole-Core save/hash invariance
while camera transitions advance. Inactive cameras freeze all behavior and
transition elapsed state; reactivation resumes on their next active update.
Input requests to inactive cameras can retarget their base goal, but do not
increment elapsed until active; Instant applies immediately. Frame/read
extraction has no state mutation or elapsed advancement. update is whole-host
atomic: clone/stage bounded state, validate all changes/extractions/math/save
budget, then commit. Any invalid request/plugin error preserves ticks,
transitions, follow/shake state and all other cameras. IDs sort canonically;
no HashMap ordering or wall-clock dependence. Trusted callbacks must be pure;
hidden callback state/external side effects cannot be rolled back by this host.

## Bounds, errors, structured discovery and save

```rust
pub enum ErrorCode {
    InvalidIdentifier, InvalidDescriptor, DuplicateId, MissingView, MissingCamera,
    InvalidSelection, InvalidTransform, InvalidProjection, InvalidViewport,
    InvalidInput, InputOutOfSequence, OutsideDepth, BudgetExceeded, Overflow,
    PluginFailed, VersionMismatch, SaveMismatch, InvalidSave, Unsupported,
}
pub struct Error { /* bounded code/detail */ }
// new(code, impl Into<String>), code(), detail(), Display/Error.
pub struct ViewLimits {
    pub max_views: usize, pub max_cameras: usize, pub max_items_per_view: usize,
    pub max_changes: usize, pub max_save_bytes: usize, pub max_transition_ticks: u32,
}
```

Limits:8Views,32Cameras total,4096 selected items/View,32 changes/input,
1,048,576 canonical save bytes,4096 transition ticks; content binding128bytes of nonempty ASCII alphanumeric or _ . : -,
IDs128bytes, field names128bytes, error detail4096bytes. No silent truncation.
Validate/count before accepted-state clone where source can be unbounded;
canonical save uses capped writer before allocating more than max_save_bytes.
Malformed descriptor/query/extraction uses explicit scoped code. Failed
presentation mutations/restore preserve exact old save/hash. No new errors
are added to P1/P2 enums.

Discovery advertises builtin4families, endpoint and blended projections,
static/follow/axes/bounds/dead-zone/look-ahead/tracking-smoothing/zoom/shake/
smooth-transition, concrete installed source bindings and active camera sets.
It is structured serde data, not README parsing. Available Format descriptors
include compatibility constraints and exact configured schema requirements
via installed ViewConfig; names alone never imply general renderer support.

Presentation save **format_version1, contract_version1** is independent of
P2save2 and legacy save. Canonical compact JSON, strict recursive unknown-field
rejection, sorted ViewId/CameraId maps and sets, deterministic struct field order,
exact byte reencoding with float-roundtrip parsing. SHA256 lowercase hex.
Input normalization is separate from restore: already-saved quaternions are
validated for unit length within 1e-12 and canonical sign without renormalizing
their bits. Noncanonical encodings are rejected rather than rewritten. Save content binding, tick,
installed Format descriptors/configs, camera configs/active flags, sampled base
target, transition start/target/duration/elapsed, prior entity point/anchor,
   follow-initialized flag, shake active count. View host runtime selections,
output buffers and linked callbacks are not authoritative saved state.

Restore requires the already-installed exact Format descriptors/configs and
camera configurations (including defaults/behavior), matching content binding,
and a valid current read selection. It validates every numeric bound,
reference/type, camera ownership, active flags, transition counters and
sampled base pose recomputed from transition state; contradictions are
InvalidSave. Mathematically valid but different immutable installed config or
content is SaveMismatch; version mismatch VersionMismatch. No plugin discovery,
auto-install or legacy/Core restore occurs. Restore is fully staged and atomic.
Save at midpoint, restore and same future read/input must produce byte-identical
state/output to uninterrupted execution on the same target/toolchain.

## Legacy compatibility and fifth-demo acceptance

Scout found the existing fifth demo is `examples/flatland_nuvema`:5scenes,
320x240, existing1957-tick replay. The native fixture must reuse this content;
do not create a substitute demo. Its legacy menu also changes collision plane,
elevation and state. **Do not execute those commands to test presentation
invariance.** Native local targets reproduce only the presentation presets:
plan zoom100/tilt100/shear0; depth115/75/12; balcony125/70/18. Player follow
bounds are[0,0,192,336]. Use actual World snapshots/entities/state, actual
installed P2 scene/entity refs, and native outputs, never fake DTO integration.

Legacy camera classification:

| Legacy concept | Treatment |
|---|---|
| Tick-based camera blend, follow bounds, persistent presentation | Reusable concepts; native behavior owns implementation |
| Pixel-world Y down, integer tilt/shear, upright-foot projection | Compatibility-only map |
| Global scene camera, legacy gameplay-view menu state | Native multi-camera/local mode replacement |
| Whole-frame warping or moving plane/elevation to change camera | Obsolete approach; prohibited |

The existing renderer's public `render(Project,Snapshot,bool)` can be called
without modifying renderer source. Its private projection composes ground
and upright feet correctly; compatible native CameraFrame state is converted
to **cloned presentation-only** Project/Snapshot overrides before calling
render. Original Project/World/Snapshot and P2 game state remain unchanged.
Output must be driven by actual nativeCameraFrame, not old camera playback.

```rust
pub fn presentation_plugin(world: &World, owner: PluginId)
    -> Result<(Box<dyn CorePlugin>, RecordBinding), CompatibilityError>;
pub fn legacy_camera_target(snapshot: &Snapshot, viewport: &Viewport,
                            zoom: f64, tilt: f64, shear: f64)
    -> Result<CameraTarget, CompatibilityError>;
pub fn render_legacy(project: &Project, snapshot: &Snapshot, camera: &CameraFrame,
                     debug: bool) -> Result<ge4g_render2d::Frame, CompatibilityError>;
pub fn legacy_foot(camera: &CameraFrame, feet: [f64; 2], elevation: f64)
    -> Result<[f64; 2], CompatibilityError>;
```

CompatibilityError has codes InvalidSource/CoreValidation/Unsupported/
RenderFailed/BudgetExceeded, bounded detail4096 and ordinary accessors.
presentation_plugin is an explicit **fixture/source importer**, separate from
ViewHost. Actual P2 legacy.bridge provides no capability, so a second owner
cannot legally refer to its entities. The new importer therefore owns both
Scene/Entity/state snapshot identities and bounded generic position i64 +
Entity reference records in one ordinary initialization transaction. It directly validates bounded &World data using the documented P2 bridge
source labels/schema conventions, remaps refs to its actual Core allocations,
preserves state and excludes camera/view data. It does not call the private
P2 projection helper or clone the whole World/Project before bounds checks.
Its source descriptor declares no Input Actions, avoiding legacy.* action
collisions when legacy.bridge is also installed. Position
schema is `<owner>.position` with exact fields entity:Ref(Entity), x:I64,
y:I64,z:I64; x/y identify visual ground feet: legacy pixel-floor top-left
plus half sizeX/full sizeY, converted to subpixels; y is negated and z=0.
Legacy elevation is an upright sprite offset in cloned Snapshot, not silently
redefined as native worldZ; compatibility legacy_foot applies it after ground
projection. Metadata, plane and elevation remain unmodified in the source.
RecordBinding units_per_world=SUBPIXELS. Local position keys `p_<SHA256(scene
id+NUL+entity id)>` and source identity metadata retain Unicode names. State
schemas use new owner-prefixed IDs to avoid collisions with existing P2
adapter if both are installed. Total imported records must satisfy 2*entity_count+state_count+2<=256;
owner-prefixed schema IDs must also satisfy P2 ID bounds. All snapshot
counts/values validate before clone/import; factory returns a trusted immutable plugin whose tick is no-op.
Its authority is legacy snapshot import, not Camera. P2 APIs are not expanded
and cross-owner checks are never bypassed. Native typed position records are
allowed; their semantics belong to source plugins, not Tiny Core. No claim
that either immutable adapter continuously updates gameplay or preserves
identities across repeated imports.

The legacy rendering adapter supports exactly: viewport origin[0,0] and
window-matching size; orthographic lens (Orthographic or equivalent Blended weight0 during a smooth
transition) with half_height=height/2; identity
camera/View rotations; View scaleX=scaleZ=1, positive bounded scaleY, XY shear;
finite camera.positionX/Y and positive cameraZ depth; View originZ=0; no general
native rotation/perspective. Input source worldY=-legacy pixelY. With
z=zoom/100 and t=tilt/100, compatible target uses pose.zoom=z, scaleY=t,
shear_xy=-shear/(100*z*t), View.origin=[originX,-originY,0], cameraZ=focus.
This yields legacy ground screen dx=(pixelX-originX)*z+
(pixelY-originY)*shear/100, dy=(pixelY-originY)*z*t. Adapter derives effective
origin from View origin and any sampled follow displacement so native outputs
actually drive the renderer. Snapshot dimensions and pixel coordinate bounds
must validate. The renderer subset is stricter than native math: derived zoom
and tilt integer percentages must be 1..10000, shear absolute value<=10000;
effective origins and all current-scene projected ground geometry absolute
coordinates<=1e6, bounded upright offsets/elevations. The adapter preflights
renderer arithmetic with checked i128 before narrowing to i64 and rejects
unsafe source/parameter combinations before rendering. This includes ground
projection, inverse screen corners, entity/building feet and upright offsets. Each original i64 intermediate,
not only the final i128 sum, must fit; projected building polygon edge products
must fit too. Validate current scene map/patches and texture dimensions/RGBA/crop
indices, entity anchors/elevation/visual sizes and building geometry. Reject
unvalidated attack/battle rendering paths as Unsupported in this adapter.
Unsupported cases return Unsupported; no misleading fallback.
Native project supports full quaternion and perspective independently.
Map native sampled target to legacy zoom/tilt/shear and camera origin. Integer
legacy compositor rounds presentation parameters with explicit documented
tolerance (≤0.5 percentage point for rounded percentages,≤0.5pixel for rounded camera
offset before integer compositor truncation). Projected output error depends
on distance from origin, zoom/tilt/shear and integer truncation; no universal
1pixel output agreement is promised. Acceptance checks compute tolerances
from the actual fixture coordinates and rounded coefficients.
Transition state is native exact floats, not rounded renderer values.
legacy_foot first calls native project on ground WorldPoint[x,-y,0], then
subtracts elevation*pose.zoom from screenY; elevation finite bounded.
Ground uses projected coordinates; sprites/models/buildings remain upright
with feet anchored and zoom-scaled local offsets. Never tilt a finished frame.
Legacy actual integer actor_screen_foot can be checked against legacy_foot
within rounding tolerance; changed camera should change pixel/frame hash.

The acceptance example/test imports the actual fifth World into a stable
CoreHost once, installs the source records once, selects actual scenes/entities,
then freezes authoritative state while native local mode transitions advance.
For gameplay-motion follow tests, the source/gameplay plugin advances its own
typed positions separately; camera update still only consumes ReadFrame.
Tests compare authoritative Scene/Entity identities, record/RNG histories and
whole frozen Core save/hash around mode change/remove/reinstall, verify native
transition midpoint/continuation, actual render hashes/feet, and restore/replay.
This fixture is not a new client UI nor a full delegated legacy Pentomino loop.

## Required independent evidence and external gate

Before candidate: unchanged P1/P2 tests and API/source hashes; four native
View families actual extraction/policy outputs; A/B camera independence;
TopDown→Classic2D and Classic2D→Side substitution with frozen Core byte/hash
and identities; multi-camera transform/viewport isolation; follow axes,
bounds/dead-zone/lookahead/smoothing/shake; world↔screen with depth roundtrips;
real perspective depth-size behavior; malformed/extreme finite/NaN/overflow
rejections with atomic old state; default smooth/instant/midflight retarget,
view-transform and ortho↔perspective continuity; midpoint save/restore/replay,
malformed canonical presentation save rejection; discovery-only consumer;
Nuvema real assets/pixels/foot anchor and fifth existing1957 regression;
Basement/Pac-Man/Signal Yard/Harbor regressions; fmt/clippy/workspace tests and
dependency direction. Performance compare1Camera, many cameras, many selected
items, larger selection separately, recording observed commands/workloads/time,
without claiming unmeasured gains or confusing full frame selection costs with
authoritative simulation cost.

Package compiled native+compat public artifacts, exact compiler/target,
PUBLIC_API/contract/discovery, minimal actual-content example, runner,
MANIFEST/file SHA256/ZIP SHA256 and external tester instructions. Consumer
does not read repo/internal source/existing tests and must write new tests.
Parent packaging self-check is internal, never external certification.
Final status remains **EXTERNAL VALIDATION PENDING** until the user supplies
a separate GPT Work report. P3 does not implement Gameplay/Forge/renderer
rewrite or automatically resume suspended client/platform builds.

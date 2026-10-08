# P2 game-data Tiny Core contract

Baseline main `2d1ffd0`, GE4G0.3.0-alpha.1. **FROZEN FOR IMPLEMENTATION / runtime UNVERIFIED**. Architect released
ownership; auditor identified two MEDIUM gaps and parent integrated the precise
ordinal/bounded-writer fixes before technical acceptance. Runtime acceptance
requires actual evidence and the separate external GPT Work gate. User P2 covers retention isolation, Scene/Entity, typed records/
actions and a legacy compatibility bridge. P3 owns Camera & View/Format.

## Compatibility and dependency direction

Add `ge4g_pentomino::p2`, with public CoreHost/CorePlugin/CoreDescriptor. Do not
change scalar Host/Plugin/PluginDescriptor, save1/contract1 or its26 tests. In
particular scalar global256 history and Bhistory128vs192 behavior remain valid
scalar-compatibility behavior. New CoreHost has save2/contract2.0.0, explicit
different types and no implicit save1 migration. Root PluginId, CapabilityId,
Version and Bindings may be reused without changing their old semantics.
p2::Error/p2::ErrorCode are new scoped types; never add variants to root scalar
ErrorCode or break existing exhaustive matches. Normal Core dependencies remain std +
serde/serde_json/sha2/thiserror only.

New `ge4g-pentomino-legacy` depends on legacy core/project/runtime and generic
Pentomino contract. Dependency never points back from Tiny Core. No camera,
viewport, sprite, layer, coordinate, FlatLand or genre types/imports exist in p2.
Data-only imports, schema1/2, ABI1, legacy saves/resume and authoritative Rust
simulation remain. Linked host-installed trusted native plugins only; native
sandboxing/panic/process recovery remain UNVERIFIED.

## Exact public types (proposed)

All serialized structs/enums deny unknown fields. Enum serde representation is
adjacent tagged `type`/`value`, snake_case variants. Types are Clone/Debug/Eq;
IDs and stable object references also Ord. All structs below have public fields
except explicitly private runtime handles. New IDs provide `new(&str)->
Result<Self,Error>` and `as_str()->&str`, using P1 dotted ASCII namespace rules.
Local keys use P1 single-segment rules. Collections are BTreeMap/BTreeSet.

```rust
pub struct SceneRef { pub owner: PluginId, pub local: String, pub incarnation: u64 }
pub struct EntityRef { pub owner: PluginId, pub local: String, pub incarnation: u64 }
pub enum ObjectRef { Scene(SceneRef), Entity(EntityRef) }
pub enum RefKind { Scene, Entity }
pub enum FieldType {
    Bool, I64 { min: i64, max: i64 },
    String { max_bytes: usize }, Bytes { max_bytes: usize },
    Ref { kind: RefKind },
    List { item: Box<FieldType>, max_items: usize },
}
pub enum Value {
    Bool(bool), I64(i64), String(String), Bytes(Vec<u8>),
    Ref(ObjectRef), List(Vec<Value>),
}
pub struct RecordSchema { pub fields: BTreeMap<String, FieldType> }
pub struct Record { pub schema: SchemaId, pub fields: BTreeMap<String, Value> }
pub struct RecordKey { pub owner: PluginId, pub local: String }
pub enum ActionType { Bool, I64 { min: i64, max: i64 } }
pub enum ActionValue { Bool(bool), I64(i64) }
pub struct InputFrame { pub target_tick: u64, pub actions: Vec<(ActionId, ActionValue)> }
pub struct CoreDescriptor {
    pub id: PluginId, pub release: Version, pub contract: Version,
    pub provides: BTreeMap<CapabilityId, Version>,
    pub requires: BTreeMap<CapabilityId, Version>,
    pub schemas: BTreeMap<SchemaId, RecordSchema>,
    pub event_kinds: BTreeMap<String, SchemaId>,
    pub actions: BTreeMap<ActionId, ActionType>,
}
pub struct SceneData { pub reference: SceneRef }
pub struct EntityData { pub reference: EntityRef, pub scene: SceneRef }
pub struct CoreEvent {
    pub owner: PluginId, pub tick: u64, pub sequence: u64,
    pub ordinal: u64, pub kind: String, pub record: Record,
}
pub struct OwnerHistory {
    pub emitted: u64, pub dropped: u64,
    pub history: Vec<CoreEvent>, pub pending: Vec<CoreEvent>,
}
pub struct Selection {
    pub owners: BTreeSet<PluginId>, pub schemas: BTreeSet<SchemaId>,
    pub include_objects: bool, pub include_history: bool,
}
pub struct ReadFrame {
    pub tick: u64, pub records: BTreeMap<RecordKey, Record>,
    pub scenes: Vec<SceneData>, pub entities: Vec<EntityData>,
    pub histories: BTreeMap<PluginId, OwnerHistory>,
}
```

SchemaId and ActionId are separate newtypes, not PluginId aliases. Schema IDs
have exact definitions in owner's descriptor (no separately executable validator
or generic schema registry). A Record belongs to its owning plugin's schema;
all fields required, exact set, correct tagged values, no coercion/extra fields.
SchemaId uniqueness across installed plugins rejects ambiguity; ActionIds also
have one declaring owner across host. Declared required capability binding grants
read access to that provider's records/events/objects and permits live refs to
its objects, never write authority. Self reads/refs are implicit. Actions are
visible to their declaring plugin only; no Core gameplay interpretation.

## Exact proposed methods

```rust
pub trait CorePlugin {
    fn descriptor(&self) -> CoreDescriptor;
    fn initialize(&self, ctx: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error>;
    fn tick(&self, ctx: &CoreContext, tx: &mut CoreTransaction<'_>) -> Result<(), Error>;
}
impl CoreHost {
    pub fn new(seed: u64, content_binding: &str) -> Result<Self, Error>;
    pub fn install(&mut self, plugin: Box<dyn CorePlugin>, bindings: Bindings) -> Result<CoreOwnerToken, Error>;
    pub fn owner(&self, id: &PluginId) -> Result<CoreOwnerToken, Error>;
    pub fn remove(&mut self, owner: CoreOwnerToken) -> Result<(), Error>;
    pub fn step(&mut self, input: InputFrame) -> Result<(), Error>;
    pub fn select(&self, selection: &Selection) -> Result<ReadFrame, Error>;
    pub fn scene_handle(&self, reference: &SceneRef) -> Result<SceneHandle, Error>;
    pub fn entity_handle(&self, reference: &EntityRef) -> Result<EntityHandle, Error>;
    pub fn resolve_scene(&self, handle: SceneHandle) -> Result<SceneData, Error>;
    pub fn resolve_entity(&self, handle: EntityHandle) -> Result<EntityData, Error>;
    pub fn describe(&self) -> CoreDiscovery;
    pub fn save(&self) -> Result<Vec<u8>, Error>;
    pub fn hash(&self) -> Result<String, Error>;
    pub fn restore(&mut self, bytes: &[u8]) -> Result<(), Error>;
}
impl CoreContext {
    pub fn tick(&self) -> u64;
    pub fn own(&self, local: &str) -> Result<Option<&Record>, Error>;
    pub fn read(&self, owner: &PluginId, local: &str) -> Result<Option<&Record>, Error>;
    pub fn scene(&self, reference: &SceneRef) -> Result<&SceneData, Error>;
    pub fn entity(&self, reference: &EntityRef) -> Result<&EntityData, Error>;
    pub fn events(&self, owner: &PluginId) -> Result<&[CoreEvent], Error>;
    pub fn action(&self, action: &ActionId) -> Result<Option<&ActionValue>, Error>;
}
impl CoreTransaction<'_> {
    pub fn create_scene(&mut self, local: &str) -> Result<SceneRef, Error>;
    pub fn remove_scene(&mut self, reference: &SceneRef) -> Result<(), Error>;
    pub fn create_entity(&mut self, local: &str, scene: &SceneRef) -> Result<EntityRef, Error>;
    pub fn remove_entity(&mut self, reference: &EntityRef) -> Result<(), Error>;
    pub fn set(&mut self, local: &str, record: Record) -> Result<(), Error>;
    pub fn delete(&mut self, local: &str) -> Result<(), Error>;
    pub fn emit(&mut self, kind: &str, record: Record) -> Result<(), Error>;
    pub fn draw(&mut self) -> Result<u64, Error>;
}
```

CoreOwnerToken/SceneHandle/EntityHandle have private constructors/fields and
cannot deserialize. CoreOwnerToken is Copy/Eq/Debug; SceneHandle/EntityHandle
are Clone/Eq/Debug because stable refs contain Strings. They contain fresh host/generation identity and
stable wire ref internally. Host is not Clone. Host select is trusted host-facing
authority (not a plugin callback capability); unknown selection IDs reject.
Empty owners means no owners, empty schemas means all their schemas. A schema
filter affects records only; object/history flags are explicit. Context methods
enforce declared reads; no arbitrary host mutation reference escapes.
ReadFrame is a typed Rust selection DTO. Its structured RecordKey map has no
direct nonempty serde_json wire encoding; canonical portable persistence is
CoreHost::save() format_version2 bytes, not ReadFrame JSON. A consumer may define an array envelope.
Concrete discovery structs (public fields):

```rust
pub struct CoreInstalled {
    pub descriptor: CoreDescriptor, pub bindings: Bindings,
    pub removal_blockers: Vec<PluginId>,
}
pub struct CoreDiscovery {
    pub contract_version: Version, pub save_version: u32,
    pub scalar_compatibility_version: Version,
    pub limits: CoreLimits, pub tick: u64,
    pub plugins: Vec<CoreInstalled>, pub last_input: Option<InputFrame>,
}
```

No tokens in discovery. Tagged unit variants have `type` but no `value` field;
nonunit variants require `value`, with no alternate representation accepted.

## Numeric limits and canonical validation

| Limit | Value |
|---|---:|
| Plugins / provided and required capabilities per plugin | 16 / 32 each |
| Schemas / fields per schema / event kinds / declared actions per plugin | 32 / 32 / 32 / 32 |
| Live records / scenes / entities per plugin | 256 / 64 / 256 |
| String bytes / byte-array bytes / list items / type nesting depth | 1024 / 4096 / 64 / 4 |
| Canonical bytes per Record / actions per InputFrame | 65,536 / 128 |
| ID/local/content binding bytes / error detail bytes | 128 / 4096 |
| Emitted events per plugin commit / pending per plugin / retained per plugin | 128 / 128 / 256 |
| Transaction commands per plugin callback / complete save2 bytes | 4096 / 8,388,608 |

CoreLimits has public usize fields: max_plugins, max_provides, max_requires,
max_schemas, max_fields, max_event_kinds, max_actions, max_records, max_scenes,
max_entities, max_string_bytes, max_bytes, max_list_items, max_depth,
max_record_bytes, max_frame_actions, max_id_bytes, max_content_binding_bytes,
max_error_bytes, max_events_per_commit, max_pending_events, max_history_events,
max_commands, max_save_bytes, populated exactly from this table. Limits are fixed
in P2, not caller configurable. Descriptor field bounds cannot exceed limits; I64 min<=max,
string/bytes/list max at least1, root type depth1, nesting max4. Lists are
homogeneous, including nested lists; no Value containing undeclared JSON.
These bounds permit ordinary state and entity projections; they do not imply
asset storage. Oversized legacy source state is an adapter error, never truncated.
Before accepted-state insertion/clone in set/emit, recursively validate shape
and references, then count canonical bytes with a capped streaming writer that
stops at max_record_bytes. Do not allocate unrestricted to_vec output first.
Commit/save-size checks use a capped streaming writer stopping at max_save_bytes
before any successful install/tick/removal commit, so normal committed Core state
remains serializable. Trusted native caller allocations are outside this guarantee. All mutating tx
calls, including invalid calls, consume commands and poison on error; callback
cannot swallow a rejection and commit. Counter overflow rejects atomically;
SplitMix64 modular arithmetic follows P1 exactly. Descriptor/resources are data,
not a promise to bound malicious native code CPU/heap behavior.

## Identity, references and transactional validity

Host has one saved `next_identity:u64` starting1 for both scenes/entities.
Every staged create assigns that serial as incarnation and increments checked;
remove/unload/reinstall never resets it. Thus same owner/local recreated after
committed deletion gets fresh wire incarnation without unbounded tombstone maps.
Stable wire refs identify the current canonical restored timeline, not external
authority/freshness. Restoring a past save can reproduce an abandoned future
incarnation on deterministic replay; runtime handles from that future always
remain stale. Do not max(old,saved) the allocator: restore must preserve exact
bytes/hash. All live incarnations are globally unique and below next_identity. Failed
init/tick rolls back this authoritative counter. Staged refs are provisional
until successful commit; callbacks may not leak them through hidden state or
external effects. Such refs never denoted a committed object and have no external
lifecycle guarantee. Runtime handles are issued only for committed objects.

Only own objects can be created/removed. Entity.scene must be a live self or
bound-provider scene. All Ref values in live records AND retained/pending events
must target a live matching-kind exact incarnation and satisfy originating
owner's declared binding. Remove of an object with any surviving ref/entity-scene
link returns ReferenceInUse; delete referring record/remove child first. History
refs therefore retain their targets until history eviction or explicit plugin
unload; no silent stale reference publication. This conservative rule is explicit.
No scene deletion cascade. Plugin removal is rejected for live dependent binding;
otherwise removing its objects/records/events cannot leave any foreign refs.

Tx operates on a private own-namespace working copy plus immutable foreign reads;
set may refer to own objects created earlier in same transaction. Own deletion
checks own staged refs and immutable foreign refs conservatively. Cross-plugin
references target pre-tick committed objects only; if another owner removes a
target later in the same tick, final all-state validation rejects whole tick.
No same-tick foreign object creation visibility or re-entrant callbacks.

Initialize and each full tick stage all records, objects, allocators, RNG, emitted
events and actions. Validation/poison/callback failure leaves canonical state,
committed input/tick and existing handles unchanged. Successful remove invalidates
only removed object's/owner's handles; successful restore invalidates all handles
and owner tokens. Host/session nonce/fresh runtime generations stay out of save/
hash. Counter reservation may skip runtime generations on failure, never reuse.

## Input, history and replay isolation

Frame target == committed+1 only. Duplicate/past returns InputAlreadyCommitted;
future returns InputOutOfSequence; malformed/duplicate/unknown action or wrong
type/range returns InvalidInput. actions Vec must contain unique ActionIds;
normalize sorted by ID for callbacks/save. Reject duplicates before converting
to a map. Missing action is absent, not implicit held/default value. Plugins own
held-state/exact-once markers in saved records if needed. Failed tick leaves same
frame retryable. Saved last_input is None at tick0 and Some exact canonical frame
at tick>0. Input callbacks see target-frame actions and pre-tick committed data.
Removing a plugin removes only its declared actions from saved last_input;
last_input is explicitly the last committed frame's currently installed-owner
projection, not an immutable original trace. The caller's replay trace stays
outside Core. This keeps remaining saved actions restorable without dead action
declarations; target tick is retained even if projected action list becomes empty.

Each installed owner has its **own256 history** and128 pending budget, independent
of unrelated emitters. Successful event commit assigns global sequence and owner
ordinal, both starting0. Owner.emitted is next ordinal; dropped counts evicted
owned history only; emitted=history.len+dropped. History keeps newest256 and
pending is exactly owner's retained history projection at committed tick. Init
may append at current tick within pending128. New tick replaces pending.
Foreign events are available only through declared bound-provider reads.

Global next_sequence counts lifetime emitted events. Removed owners' emitted
counts accumulate into retired_emitted and their retained history count into
purged_retained; both are saved monotone checked counters. Thus next_sequence =
sum(active owner.emitted)+retired_emitted. Active histories' global sequences are
unique, strictly increasing within each owner, below next_sequence. Ordinals are
a complete newest suffix: checked history[i].ordinal == dropped+i and
history.len == min(emitted,256); ordinals below emitted and tick order nondecreasing. Pending is exact
current-tick projection. Removal does not change B history, dropped, pending,
RNG or objects. Compare B event content/ordinals excluding global sequence;
whole-host hash naturally differs when A exists or was removed.

## Save2 atomic restore and errors

Canonical compact JSON fixed structs, recursive deny_unknown_fields, sorted maps/
sets/plugin and object arrays, and exact byte-equal re-encoding as in save1.
Top-level field order: format_version(2), contract_version(2.0.0), content_binding,
seed, tick, next_identity, last_input, plugins, next_sequence, retired_emitted,
purged_retained. Plugin order: descriptor, bindings, records, scenes, entities,
rng_state, history. History order: emitted,
dropped, history, pending. Descriptor/public type order is declaration order above.
Bytes serialize as integer arrays; tagged Values preserve bool/i64 distinctions.
SHA256 canonical bytes returns lowercase64hex. Scene/entity arrays are sorted by
stable reference; records use local-key maps. Runtime handle generations omitted.

Restore stages complete validation: max bytes/canonical encoding, exact versions,
immutable binding/seed, exact installed descriptors AND bindings, all declared
schemas/actions/limits, valid unique IDs/keys, live object kind/incarnation and
every cross-owner binding/reference, unique live identity/counter constraints,
history/pending/input/global accounting. next_identity is at least1 and strictly
greater than every unique live incarnation. purged_retained <= retired_emitted;
all counter sums use checked arithmetic. Active history ordinal suffix/count
rules are validated even when other accounting equalities hold.
All retained owners merged by global sequence must also have nondecreasing ticks;
impossible cross-owner chronology is InvalidSave (post-implementation audit hardening).
Saved tick0 requires last_input None; tick>0 requires target==tick and valid frame.
Save1 is VersionMismatch, never silently upgraded. Version mismatch is distinct
from malformed InvalidSave; seed/content/descriptors/bindings mismatch SaveMismatch.
Canonical invalid references/record/action/accounting return InvalidSave.
Only then reserve all new runtime generations and replace state. Overflow or any
failure leaves old tokens/handles and canonical bytes unchanged. No callbacks or
automatic code installs on restore. Same save2 across fresh handles yields same
hash and next-frame continuation.

New p2::ErrorCode contains the old code names/meanings (InvalidIdentifier,
InvalidDescriptor, VersionMismatch, DuplicateId, MissingCapability,
InvalidBinding, DependencyCycle, PermissionDenied, UndeclaredKey,
BudgetExceeded, StaleHandle, DependencyInUse, InvalidTick, Overflow,
PluginFailed, SaveMismatch, InvalidSave) plus InputAlreadyCommitted, InputOutOfSequence,
InvalidInput, InvalidSchema, InvalidRecord, ReferenceInUse, InvalidReference.
p2::Error provides new(code, impl Into<String>)->Self, code()->p2::ErrorCode,
detail()->&str and Debug/Display/std::error::Error, UTF-8 truncation to4096bytes.
From<root::Error> maps each existing code/detail into p2 without loss, enabling
root PluginId/CapabilityId constructors with `?` in p2 callbacks. All Error names
in P2 signatures resolve to p2::Error; SchemaId/ActionId constructors return it.
Normal wrong/stale object lookup returns StaleHandle; wrong-owner write returns
PermissionDenied. Registration invalid schema returns InvalidSchema; set/emit
schema violation InvalidRecord, including Value::Ref(Entity) for a Scene-ref
field. A correctly tagged ref whose owner/local/incarnation does not identify
the required live object returns InvalidReference; no type coercion. Unbound
foreign refs return PermissionDenied. Well-formed payload above byte quotas is
BudgetExceeded; restore structural failures are InvalidSave.

## Legacy compatibility adapter: explicit projection boundary

Actual World exposes Project/entities/state and is Clone; step_replay already
clones World around choices/do/skip plus step_actions. Legacy Snapshot contains
presentation and FlatLand; legacy view/save semantics must remain unchanged.
Do **not** put mutable World inside CorePlugin or claim Core rollback covers it.

Proposed separate crate public surface:

```rust
pub struct LegacyBridge { /* private authoritative World */ }
pub struct LegacyProjection {
    pub owner: PluginId, pub tick: u64, pub descriptor: CoreDescriptor,
    pub scenes: Vec<SceneData>, pub entities: Vec<EntityData>,
    pub records: BTreeMap<RecordKey, Record>,
}
impl LegacyBridge {
    pub fn new(world: ge4g_runtime::World) -> Result<Self, BridgeError>;
    pub fn world(&self) -> &ge4g_runtime::World;
    pub fn projection(&self) -> Result<LegacyProjection, BridgeError>;
    pub fn input_frame(&self, replay: &ge4g_project::Replay) -> Result<InputFrame, BridgeError>;
    pub fn step_replay(&mut self, replay: &ge4g_project::Replay) -> Result<LegacyProjection, BridgeError>;
    pub fn legacy_snapshot(&self) -> ge4g_core::Snapshot;
}
```

LegacyProjection additionally has these concrete methods:

```rust
impl LegacyProjection {
    pub fn plugin(&self, input: &InputFrame) -> Result<Box<dyn CorePlugin>, BridgeError>;
    pub fn install_into(&self, host: &mut CoreHost, input: &InputFrame)
        -> Result<CoreOwnerToken, BridgeError>;
}
```

plugin returns a linked immutable import plugin owning cloned bounded snapshot
data. It declares exact schemas and the supplied mapped action declarations;
initialize uses create_scene/create_entity/set, remapping snapshot refs to the
actual newly allocated Core incarnations before writing records. Its tick is a
no-op: authority remains legacy World. install_into uses ordinary host.install
with empty Bindings, rejecting duplicate owner atomically. No special privileged
Core import API is introduced. Public CoreHost select must expose actual imported
scenes/entities/state records, not a DTO-only claimed integration. Snapshot tick
is a `legacy.snapshot` record, not a forced Core tick; fresh CoreHost starts0.
Tests can retarget mapped actions to Core tick1 explicitly; never claim that
this invokes legacy gameplay. A continuous delegated engine adapter is absent.

Projection is read-only generic data, not a continuous CoreHost execution adapter. Bridge
owns the authoritative legacy simulation; inherited CLI/client still use it.
step_replay clones World into candidate, invokes existing candidate.step_replay,
then validates/builds projection, and replaces original only on complete success.
Projection failure also rolls back legacy advancement. No external filesystem
save operation is inside this atomic call. legacy_snapshot returns unmodified
snapshot, including legacy presentation; it never enters Tiny canonical envelope.
Legacy opaque presentation/resume bytes remain outside Tiny Core. No claims of
full legacy save migration or rollback of runtime's native external side effects.

Projection descriptor uses owner `legacy.bridge`, contract2.0.0, release equal
to workspace alpha numeric base Version0.3.0, empty provides/requires/event_kinds,
and schemas `legacy.scene` (id:String), `legacy.entity` (id:String,
scene:Ref(Scene)), `legacy.state_bool` (key:String, value:Bool),
`legacy.state_i64` (key:String, value:I64 full range),
`legacy.state_string` (key:String, value:String), and `legacy.snapshot`
(tick:I64 min0 max1_000_000). String schema bounds are1024bytes. Existing state
Bool/Integer/String map without coercion. Entity record excludes position, size,
texture/color/layer/collision/FlatLand fields. Scene/entity refs only identify
data; no legacy body or view semantics imposed on Core. Local keys are lowercase
SHA256 hex of UTF-8 source identity (`scene_id` for scenes, scene_id + NUL +
entity_id for entities, state key for state records), prefixed respectively
`s_`, `e_`, `v_` to avoid record namespace overlap; snapshot record key is
`snapshot`. Source IDs remain String
fields. Projection covers current scene/current live World entities and state,
not imported texture data. Projection incarnation1 identifies a snapshot-scoped
legacy identity only; it is **not** a persistent Core object lifecycle identity.
Installing/reconciling projections into a long-lived CoreHost needs a future
explicit identity reconciliation contract; do not claim it implemented here.

Action mapping for input_frame: `legacy.left/right/up/down/interact` bool,
optional `legacy.direction_x/y` i64 full range from Input.direction, and sorted
named buttons mapped `legacy.button_<SHA256(button)>` bool true. Descriptor
declares common five bool actions and optional direction full-range i64 actions,
plus exactly supplied button IDs from trusted host callers. The bridge's
input_frame validates source-known replay labels before hashing; the plugin
factory validates supplied IDs by format, not Replay provenance. Unknown
non-button actions or the aggregate action bound fail. A supplied hashed button must match
legacy.button_ followed by64 lowercase hex characters and have Bool(true) value;
input_frame validates source replay labels before hashing. The generic plugin
factory cannot reverse a hash into a button name and does not claim that proof. Replay::input_at/actions_at use the current World.tick, matching World::step_replay;
only the mapped Core target is World.tick+1. This pure
mapping does not advance Core/legacy and does not replace replay commands.
Existing choices/do/skip still execute only through World.step_replay; opaque
commands are not converted into Core gameplay operations. Absence remains absent.
BridgeErrorCode enum has CoreValidation, LegacyRuntime, ProjectionLimit,
InvalidInput. BridgeError provides `new(code: BridgeErrorCode, detail: impl
Into<String>)->Self`, `code()->BridgeErrorCode`, `detail()->&str` and implements
Debug/Display/std::error::Error. Detail truncates safely at4096 UTF-8 bytes;
underlying Core error code is preserved in detail, without swallowed failures.
Sources exceeding field/object/action bounds fail explicitly.

## Acceptance and completion status

Required executable evidence: unchanged26 scalar tests; B whole retained history/
dropped/pending isolation past256 while A emits/removes; identity recreate,
foreign/stale/restored handles and ref cleanup; schema depth/type/size failures;
duplicate/future/bad action and failed retry replay; whole-init/tick RNG/record/
object/counter/input/event rollback including poisoned errors; malformed save2
refs/incarnations/counters/pending/inputs rejected with unchanged bytes/handles;
canonical restore continuation; adapter projection/input/failure assertions for
Pac-Man, v1 demo, Harbor Workshop and Signal Yard plus unchanged legacy replay/
frame/resume regressions. Four-game tests must create CoreHost, install the actual
immutable import plugin, assert scenes/entities/typed StateStore selections and
action mapping, then canonical save2 restore/re-select equality. Unicode scene
`항구` must survive as source-name data with reversible hashed-key lookup, never
ASCII-truncated. Command/artifact evidence belongs to parent rundown.

All P2 functionality/tests are **UNVERIFIED** here. Public docs/artifacts plus a
separate source-free GPT Work session and user's returned evidence are required
before P2 final acceptance. No in-repo agent can self-certify that gate. P2 is a
completion candidate; no P3 view implementation, gameplay plugins, Forge,
automatic platform expansion or final release is claimed.

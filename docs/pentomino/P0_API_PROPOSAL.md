# Pentomino P0 minimum public API proposal

Status: **PROPOSAL / UNVERIFIED**, 2026-10-08. PENTA-P0-A is documentation only.
Every new name, signature, lifecycle guarantee and test below is proposed, not an
implemented API. No engine tests were run by this worker. No package, schema,
save, ABI, version or supported-platform change is made here.

## Evidence and boundary

Baseline: main `293ba513`, Rust 2024 Cargo workspace of seven crates at 0.2.0;
Flutter client 0.2.0+8, as reported by P0-S. Read-only source inspection confirms:

- `crates/ge4g-core/src/lib.rs:94` exports directional `Input`; `:145` exports
  `StateStore`; `:202` exports `Event`. `EntitySnapshot` at `:212` contains color,
  texture, layer and FlatLand data; `Snapshot` at `:229` contains camera/background.
  Therefore current ge4g-core is **not already** the proposed Tiny Core.
- `crates/ge4g-runtime/src/lib.rs:77` exposes `World` containing `Project` and
  FlatLand state. Its snapshot/render_snapshot/step/save methods occur at
  `:258`, `:262`, `:475`, `:622`. Normal dependencies include core/project/mlua;
  render2d is a dev dependency. Existing runtime is authoritative simulation,
  not proof of an independently removable gameplay plugin.
- `crates/ge4g-project/src/lib.rs:138` has `Scene`, and `:317` has filesystem-root
  and decoded-texture-bearing `Project`. `crates/ge4g-render2d/src/lib.rs:77`
  requires `Project` and `Snapshot` to render. Those are legacy adapter inputs,
  never compulsory inputs to the new public contracts below.
- P0-S found client ABI1 with four C entrypoints, request JSON and a session
  registry; CLI discovery advertises current schema/package/ABI versions.
  P0-S found no implemented plugin loader/register/unload surface. Registry and
  discovery do not establish a plugin lifecycle.

Keep all existing APIs and schema1/schema2 saves, packages and ABI1 behavior.
Use an additive experimental Rust contract before deciding crate paths or a new
native ABI. Do not rename the existing crate or retrofit every existing feature
through a generic registry. No new cross-system stable identifier is activated
by this proposal; implementation must update F(x).md when introducing one.

## Smallest useful first slice

Implement later only an in-process host, one deterministic data-producing test
plugin, and a second independent test plugin. Then a read-only view adapter can
consume a selected resource snapshot. This proves boundary/lifecycle behavior
before extracting FlatLand. No generic ECS, dynamic library loader, scripting
registry, component query language or genre hierarchy is required in this slice.

Dependency direction (proposed): host and plugin implementations depend on the
contract; the contract depends on neither. View/Format and Gameplay depend on
the contract and explicitly declared provider capabilities. Core never imports
their camera, sprite, 3D, battle, world or movement implementations. Legacy
Project/World/Snapshot conversion belongs to an adapter outside Tiny Core.
Scene and Entity are opaque scoped identities and lifetime records; Core does
not assign position, size, collision, appearance or genre semantics to them.

## Proposed bounded data contract — UNVERIFIED

All identifiers below are distinct private-constructor Rust newtypes, not
interchangeable strings. Canonical external form is validated namespaced UTF-8
(`vendor.name`, maximum 128 bytes), except SceneId/EntityId/OwnerToken which are
host-issued session-scoped generation handles. Stale handles return StaleHandle;
an unload/reinstall never resurrects an old handle. Raw paths/pointers/World,
Project and Snapshot are absent from these contracts.

| Proposed type | Minimum fields / rule |
|---|---|
| PluginId, CapabilityId, SchemaId, ActionId | Distinct validated namespaces; version belongs in descriptor, not name |
| PluginDescriptor | PluginId, release version, exact contract major + supported minor interval, provided/required capabilities, registered schemas/actions, declared read/write/event permissions, resource budgets |
| CapabilityRequirement | CapabilityId + compatible version interval; host resolves to one explicit provider binding; ambiguous provider requires caller selection |
| SchemaDescriptor | SchemaId + integer version + finite declarative record fields; fields are bool, i64, bounded UTF-8, bounded bytes, typed stable identity references or bounded lists of these; no arbitrary JSON or executable validator |
| Record | SchemaId, schema version, typed field values validated against descriptor; canonical field order and encoding |
| ResourceKey | OwnerToken + SchemaId + optional SceneId/EntityId + local key; one owning writer; consumer reads require declared provider binding |
| InputFrame | target tick + ordered ActionId values (bool/i64) validated against registered action type/range; action IDs carry no built-in direction/genre meaning |
| EventEnvelope | tick, emitting OwnerToken, SchemaId/version, optional opaque scene/entity handles, ordered sequence number, validated Record |
| ReadFrame | committed tick + explicitly selected resource records and event envelopes; read-only owned values, no host mutation reference |
| TickResult | committed tick + canonical observation/event deltas; no camera/framebuffer fields |

HostLimits is immutable per session and public in discovery: maximum installed
plugins, descriptors/fields/records, record/string/byte/list sizes, input/events
per tick, retained events, save bytes and deterministic command budget. Exact
initial limits require P1 measurement and are **UNVERIFIED**. Registration cannot
omit a bound; no method accepts an unbounded collection. Over-limit requests
return BudgetExceeded with the bound name and leave committed state unchanged.
Events never silently disappear: retained-history truncation exposes the first
retained sequence and dropped count; tick production overflow fails the tick.
No Core coordinates are prescribed; formats own coordinate/layer conventions.

## Proposed Rust surface — UNVERIFIED

These are signature sketches, not compiling declarations or a new native ABI.
All named request/result types obey the bounded contracts above.

```rust
impl Host {
    fn new(limits: HostLimits, seed: u64) -> Result<Self, ContractError>;
    fn install(&mut self, plugin: Box<dyn Plugin>, bindings: ProviderBindings)
        -> Result<OwnerToken, ContractError>;
    fn step(&mut self, input: InputFrame) -> Result<TickResult, ContractError>;
    fn observe(&self, selection: ReadSelection) -> Result<ReadFrame, ContractError>;
    fn remove(&mut self, owner: OwnerToken) -> Result<(), ContractError>;
    fn describe(&self) -> DiscoveryDocument;
    fn save(&self) -> Result<SaveEnvelope, ContractError>;
    fn restore(&mut self, save: SaveEnvelope) -> Result<(), ContractError>;
}
trait Plugin {
    fn descriptor(&self) -> PluginDescriptor;
    fn initialize(&self, ctx: InitContext, tx: &mut Transaction)
        -> Result<(), PluginFault>;
    fn tick(&self, ctx: TickContext, tx: &mut Transaction)
        -> Result<(), PluginFault>;
}
```

Plugin's mutable simulation state lives exclusively in host-owned typed resources;
callbacks receive immutable committed reads and a scoped transaction. No hidden
mutable state, external side effects, wall clock, network/device access or
unseeded randomness is permitted in this first deterministic plugin contract.
Seeded RNG draws go through a transaction-scoped stream whose cursor commits or
rolls back with the tick. Rust's type system alone cannot enforce trusted native
code's behavior: this is a conformance/trust requirement, not a sandbox promise.
An implementation needing opaque mutable plugin state requires a separately
reviewed checkpoint/restore contract; it is not silently added here.

Transaction permits only bounded own-namespace resource create/update/delete,
scene/entity lifecycle operations on owned handles, and declared event emission.
It exposes no filesystem, renderer or arbitrary host dispatch. Providers publish
shared state; consumers cannot write another owner's resources. Scene/entity
references to another owner require a declared binding and block provider
removal while live. A read transaction never grants a view a simulation write.

Plugin callbacks cannot manufacture OwnerToken or use another owner's handle.
InitContext/TickContext expose resolved read permissions, host-issued identities,
tick/fixed step, registered inputs and committed declared events. Cross-plugin
events emitted during tick N become readable in tick N+1, avoiding order-dependent
re-entrant dispatch. Each plugin reads the same pre-tick state and can write only
its own namespace. Ordering is topological dependency order, then PluginId lexical
order for unrelated plugins; event sequence uses that order and local emission
order. No wall-time scheduling enters simulation results.

## Lifecycle, failure and cleanup — UNVERIFIED

1. **Trust/install:** only a host-shipped or explicitly host-installed native
   implementation may reach install. Imported portable games remain data-only:
   they may declare required capabilities, never load executable code or name a
   dynamic-library path. Missing requirements produce a typed error before play.
   Initial slice uses linked Rust implementations; portable ABI/dynamic loading,
   signatures, OS sandboxing, marketplace and network updater are out of scope.
2. **Validate then initialize:** validate descriptor, versions, namespace/budget,
   permissions, provider selection and acyclic dependencies before callbacks.
   Stage initialization records, identities, RNG and subscriptions. Publish the
   owner/capabilities only on success; on any failure discard all staged effects.
   A failed initialization cannot leave a discoverable plugin or reusable handle.
3. **Run:** no install/remove during a tick; return Busy. Commit all plugins'
   writes, events, RNG and next tick together only after every callback and record
   validation succeeds. A callback error aborts the whole tick, leaving tick,
   input cursor, resources, event history and RNG unchanged. Caller explicitly
   retries the same input or removes the failing independent plugin. Native
   panic/abort or malicious code is a host fault, not a guaranteed recoverable
   PluginFault; process crash recovery is **UNVERIFIED**.
4. **Remove:** accept only a live owner at a tick boundary. A provider with live
   dependent plugins/references returns DependencyInUse without mutation; no
   implicit cascading uninstall or silent rebinding. Caller removes dependents
   first. In one commit revoke registrations, subscriptions, handles, pending
   owned events, transient and persistent resource records and RNG streams.
   Historical owned events are purged while retained/drop sequence accounting
   stays explicit. Unrelated owners remain installed and keep state/tick.
   Host owns cleanup; no fallible arbitrary shutdown callback is necessary.
   Adapter-owned device objects are separately dropped outside simulation.
5. **Errors:** ContractError is a stable tagged code with PluginId/SchemaId or
   handle context where applicable and bounded human detail. Minimum codes:
   InvalidDescriptor, VersionMismatch, DuplicateId, MissingCapability,
   AmbiguousProvider, DependencyCycle, PermissionDenied, InvalidRecord,
   BudgetExceeded, StaleHandle, Busy, DependencyInUse, PluginFailed,
   SaveMismatch. Human strings are not machine contracts; errors never become
   success or silently discard invalid records.

## View/Format then Gameplay — UNVERIFIED

View substitution is deliberately outside Host/Plugin simulation callbacks:

```rust
trait ViewAdapter {
    fn descriptor(&self) -> ViewDescriptor;
    fn present(&mut self, frame: &ReadFrame, assets: &AssetRead,
               request: &ViewRequest) -> Result<CpuFrame, ViewError>;
}
```

ViewDescriptor declares format schema/version, required capabilities and bounded
asset kinds; ReadSelection names precisely the schemas it needs. AssetRead is a
read-only logical-ID/bytes lookup supplied by the host adapter, without mandatory
Project, filesystem-root or decoded-texture types. ViewRequest holds view-owned
camera/viewport settings; CpuFrame is bounded width/height + RGBA pixels for the
existing canonical CPU client path. 3D scene interpretation may later be inside
a view provider without imposing its types on Core; its implementation is
**UNVERIFIED**. Device presentation/audio remain client adapters.

Switch views by validating new descriptor/format compatibility and presenting
one frame before replacing the active adapter; failure keeps the old view.
Camera/settings are view-owned and never new Tiny simulation state. Replacement,
view errors and removal cannot change new Tiny canonical simulation/save/RNG
hashes; legacy view persistence and snapshot_hash remain unchanged (see P0-R below).
No provider exists yet for Side View, Vertical Scroll, Top-down/3D or improved
Classic 2D. Each must separately pass substitution tests; legacy render2d can
be adapted later, without claiming today's render(Project, Snapshot) satisfies it.

Gameplay follows working View/Format contracts: turn combat, realtime combat,
open world, then interaction/platformer. Plugins exchange declared schemas and
events rather than downcasting to World or inheriting a genre class. Top-down +
realtime + open-world composition is a future integration fixture, not a shipped
feature. Forge/story/assets generation remains deferred to v0.4.

## Serialization, discovery and compatibility — UNVERIFIED

SaveEnvelope has its own explicit format version, contract version, content
identity/digest binding, fixed tick, seed/RNG cursors, stable scene/entity IDs,
exact installed plugin/schema versions and selected provider bindings, plus
all authoritative resource records (including transient simulation state needed
for the next tick), next-tick event queue, retained history and sequence accounting.
Device/view
caches are excluded. Runtime OwnerTokens/generations are not serialized;
restore remaps IDs into fresh session handles after complete validation. Restore
requires the already installed compatible plugin set; it never installs code or
silently drops missing/unknown state. Validate all metadata, bounded records and
references into staging, then atomically replace state; mismatch leaves the old
session unchanged. P0 proposes exact-version acceptance only; migrations are
explicit separately versioned functions to be designed before broader support.
New envelope is never passed off as a legacy save. Preserve legacy content digest,
game_id settings ownership and exact-once result/resume rules in its adapter.

DiscoveryDocument is versioned and bounded, with installed plugin IDs/versions,
contract ranges, bound required/provided capabilities, input/schema/resource
permissions and limits, view format requirements, public documentation links,
and removal blockers. Unavailable capabilities are explicit; metadata does not
claim implementation or gate success. Public package docs list the same schema
and lifecycle contracts. Adding CLI discovery fields requires its own compatible
versioning review; existing schema/package/ABI numbers are unchanged in P0.
Source-free consumer validation requires public alpha artifacts and a separate
workspace/session, not an in-repo worker or discovery JSON alone.

## Parent-integrated P0-R clarifications — PROPOSAL / UNVERIFIED

These rules resolve the independent review's contract gaps before P1. They do
not change existing source, save schemas, ABI or implemented behavior.

### Stable identity and canonical projections

Session generation handles are access credentials, never wire identities or
canonical hash inputs. The stable owner identity is PluginId. A stable scene or
entity identity is (owning PluginId, bounded local key, monotonic incarnation).
Deleting and recreating the same local key increments its incarnation; never
reuse a retired identity. Save includes incarnation allocator counters. Resources
and every event/reference encode these stable identities, including resource
owner fields. References are typed schema fields with explicit target kind, not
opaque strings or bytes that the host cannot validate or remap. Host validates
all references before commit and restore, maps them to fresh session handles,
and revokes every pre-restore handle. Stable IDs cannot themselves authorize writes.

Canonical encoding orders plugins/bindings/schemas/resources by validated IDs
and events by sequence; includes authoritative records, input progression,
identity counters, pending events and seeded RNG cursors; excludes runtime handle
generations, pointers and presentation/device state. Exact encoding version must
be frozen before implementation claims interoperability. A comparison of B before
and after removing A means B's declared canonical state/event/RNG projection,
not a whole-host hash that includes A. Tests compare B to an equal-tick control.

### Legacy versus new view guarantees

View-invariant hash/save bytes refer only to the **new Tiny authoritative
SaveEnvelope and its canonical hash**. Existing `ge4g_runtime::snapshot_hash` includes
camera-bearing Snapshot serialization; existing schema2 saves persist gameplay
view settings. Preserve both behaviors and their existing resume tests. A legacy
adapter may expose a simulation-only projection while retaining presentation
state separately; it must not redefine legacy snapshot_hash or strip view state
from existing saves. The two new view test adapters compare only new authoritative
envelopes; user presentation preferences remain separately persistent.

### Tick acceptance, replay and exact-once state

With committed tick N, step accepts only target tick N+1, checked before callbacks.
Past/committed target ticks return InputAlreadyCommitted; future ticks return
InputOutOfSequence; both leave state, RNG, events and tick unchanged. Tick overflow
returns BudgetExceeded before callbacks. A failed tick commits no input cursor and
permits the same frame to be retried. ActionIds must be unique within a frame;
duplicates return InvalidInput before callbacks. Canonical frame ordering is
ActionId lexical order; no meaningful tie order or host-side queued future inputs
exists in the first slice. Registered action type/range validation precedes tick
execution. Add InputAlreadyCommitted, InputOutOfSequence and InvalidInput to the
stable error codes. Save carries the committed input/tick progression.

Gameplay reward/result/resume exact-once markers are plugin-owned authoritative
typed resources included in SaveEnvelope. Tick deduplication alone does not prove
gameplay exact-once behavior, which needs the later plugin's resume tests.

### Retained history and additional required tests

Save contains bounded retained event history, pending next-tick events, sequence
cursor, first retained sequence and explicit dropped/purged count. Restore
validates and restores all of them atomically; no silent history reset. Removal
purges owned history and records removed sequence gaps explicitly in bounded
accounting; reads expose those gaps. History limits cannot silently lose pending
events. Exact retained-history wire encoding and accounting bounds must be frozen
with the other P1 limits.

Additional acceptance specifications, all **NOT RUN / UNVERIFIED**:

- Restore equivalent state into different handle generations: equal stable hash,
  equal next-input events, and all old handles StaleHandle. Delete/recreate a local
  name: neither old handle nor old stable incarnation resolves to the new object.
- Swap new views: equal new authoritative envelope/hash; run unchanged legacy
  schema2 persistent-view resume regressions separately.
- Successful tick1 followed by the same frame: InputAlreadyCommitted and no
  effects/RNG/event/tick changes. Future target and duplicate ActionId reject
  atomically. Failure then retry matches clean control. Later gameplay save/resume
  cannot duplicate reward/result markers.
- Save/restore preserves observation history, gaps, first sequence and dropped
  accounting; removing A changes only A history and explicit purge accounting.

## Concrete future acceptance cases — all NOT RUN / UNVERIFIED

Commands/crate paths will be recorded after P1 creates real targets. These are
test specifications, not executable tests or passing evidence.

| Gate | Required fixture and assertions |
|---|---|
| G1 | Compile Tiny contract + host + dummy data plugin without project/runtime/render2d/client; inspect normal dependency graph and reject view/genre implementation imports. Confirm no mandatory legacy World/Project/Snapshot type leaks. |
| G2 | Install A and unrelated B, remove A, reject A's stale handle, continue B for two ticks with identical baseline hash; reinstallation allocates fresh generation. Provider P with dependent D rejects removal atomically; remove D then P without changing B. Missing/ambiguous/cyclic/version-incompatible dependency registration never runs initialize. |
| G2/G6 | Initialize writes a resource/event, draws RNG, then returns error: installed set, reads, events, handles and RNG match pre-install baseline. Tick A writes/draws/emits then B errors: same tick/state/input/RNG/event hash as pre-tick; explicit retry matches clean control. Reject unauthorized writes and oversized payloads before commit. |
| G3 | Render same recorded simulation with two small differently styled view adapters, replace at tick boundary and remove both; simulation hashes/save bytes remain identical. Incompatible format/failed first present retains old adapter. Actual named views each need separate installation metadata and render evidence. |
| G4 | Once all three exist, install top-down view + realtime combat + open-world plugins using declared schemas; fixed input replay reaches an asserted world/combat result without Core change. Remove an unrelated interaction plugin and repeat. Currently BLOCKED by absent implementations. |
| G6 | Save/restore seeded session yields identical next-input state/event hash, new valid handles and exact-once effects; wrong digest/version, absent plugin, bad reference/over-limit record leaves original session byte-identical. Removing A leaves no A records/subscriptions/events; repeated install/remove stays within measured host accounting. |
| G5 | Separate source-free consumer selects available capabilities from published discovery/docs, runs public alpha sample and reports failures with artifact/version. In-repo tests cannot certify this. |
| G7/G8 | Retain existing targeted and full TESTPLAN regression evidence and measured perf baseline or justified no-change; diff confirms no Forge/asset generation. Known Windows ZIP/timestamp failure remains separately investigated, never waived or fixed by weakening validation. |

P0 handoff: auditor should challenge transactional ownership, provider removal,
schema bounds, legacy leakage and view mutation. Any contract correction precedes
functionality. Implementation and runtime gate acceptance remain **UNVERIFIED**.

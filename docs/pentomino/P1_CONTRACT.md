# P1 scalar lifecycle contract freeze

2026-10-08; baseline `52e75c0`, branch `pentomino/p1-lifecycle`.
Status: **TECHNICALLY ACCEPTED FREEZE / runtime UNVERIFIED**. Architect
released ownership; independent auditor reviewed it; parent integrated two LOW
consistency corrections before implementation. Executed evidence is recorded
separately in P1_RUNDOWN.md. This document specifies the first implementation slice only.
It does not supersede P0's broader goals or claim all P0 APIs implemented.

## Boundary and public surface

New `crates/ge4g-pentomino` uses only std and workspace serde/serde_json/sha2/
thiserror. No dependency on existing core/project/runtime/render2d/client or
camera/genre code. Existing adapters, packages, saves, schemas, ABI1, version and
Android signing identity remain untouched. Only host-installed linked Rust
plugins are accepted; imported games remain data-only. No dynamic code loading.

Proposed Rust signatures (public names frozen here; implementation pending):

```rust
pub trait Plugin {
    fn descriptor(&self) -> PluginDescriptor;
    fn initialize(&self, ctx: &Context, tx: &mut Transaction<'_>) -> Result<(), Error>;
    fn tick(&self, ctx: &Context, tx: &mut Transaction<'_>) -> Result<(), Error>;
}
impl Host {
    pub fn new(seed: u64, content_binding: &str) -> Result<Self, Error>;
    pub fn install(&mut self, plugin: Box<dyn Plugin>, bindings: Bindings)
        -> Result<OwnerToken, Error>;
    pub fn remove(&mut self, owner: OwnerToken) -> Result<(), Error>;
    pub fn step(&mut self, target_tick: u64) -> Result<(), Error>;
    pub fn observe(&self, owner: OwnerToken) -> Result<Observation, Error>;
    pub fn owner(&self, id: &PluginId) -> Result<OwnerToken, Error>;
    pub fn describe(&self) -> Discovery;
    pub fn save(&self) -> Result<Vec<u8>, Error>;
    pub fn hash(&self) -> Result<String, Error>;
    pub fn restore(&mut self, bytes: &[u8]) -> Result<(), Error>;
}
impl Context {
    pub fn tick(&self) -> u64;
    pub fn own(&self, key: &str) -> Result<Option<i64>, Error>;
    pub fn read(&self, provider: &PluginId, key: &str) -> Result<Option<i64>, Error>;
    pub fn events(&self, provider: &PluginId) -> Result<&[Event], Error>;
}
impl Transaction<'_> {
    pub fn set(&mut self, key: &str, value: i64) -> Result<(), Error>;
    pub fn emit(&mut self, kind: &str, value: i64) -> Result<(), Error>;
    pub fn draw(&mut self) -> Result<u64, Error>;
}
```

PluginId/CapabilityId are separate validated newtypes with `new(&str) ->
Result<Self, Error>` and `as_str() -> &str`. OwnerToken fields/constructor are
private; it may be Copy but is not deserializable or constructible externally.
Public data structs use these exact field names/types. Version/descriptors have
Clone/Debug/PartialEq/Eq; IDs also Ord/Hash. Host validates public fields:

```rust
pub struct Version { pub major: u32, pub minor: u32, pub patch: u32 }
pub struct PluginDescriptor {
    pub id: PluginId, pub release: Version, pub contract: Version,
    pub provides: BTreeMap<CapabilityId, Version>,
    pub requires: BTreeMap<CapabilityId, Version>,
    pub resources: BTreeSet<String>, pub event_kinds: BTreeSet<String>,
}
pub type Bindings = BTreeMap<CapabilityId, PluginId>;
pub struct Event {
    pub tick: u64, pub owner: PluginId, pub kind: String,
    pub value: i64, pub sequence: u64,
}
pub struct Observation {
    pub owner: PluginId, pub tick: u64, pub resources: BTreeMap<String, i64>,
    pub rng_state: u64, pub events: Vec<Event>,
    pub next_sequence: u64, pub events_dropped: u64,
    pub first_retained_sequence: u64,
}
pub struct InstalledPlugin {
    pub descriptor: PluginDescriptor, pub bindings: Bindings,
    pub removal_blockers: Vec<PluginId>,
}
pub struct Discovery {
    pub contract_version: Version, pub save_version: u32,
    pub limits: Limits, pub tick: u64, pub plugins: Vec<InstalledPlugin>,
}
```

Limits has public usize fields max_plugins, max_provides, max_requires,
max_resources, max_event_kinds, max_id_bytes, max_events_per_commit,
max_pending_events, max_history_events, max_commands, max_error_bytes,
max_content_binding_bytes, max_save_bytes, with values from the table below.
Limits are fixed in this slice. Descriptor contract must equal Version1.0.0.
BTreeMaps/Sets canonicalize order; duplicate wire keys/elements reject. Error
provides `new(code: ErrorCode, detail: impl Into<String>) -> Self`,
`code() -> ErrorCode`, `detail() -> &str`; new truncates detail at a UTF-8
boundary to4096 bytes, preserving code. Plugin callbacks can use PluginFailed.
Capability entries contain id + exact Version; requirements use exact Version
in this slice. All scalar resource/event schemas are fixed signed i64. No
untyped JSON, caller callbacks or arbitrary validators enter descriptor data.

Bindings contains one entry per required capability.
This selection grants reads of that provider's declared resources/events only;
no foreign writes. Duplicate/extra/missing binding entries reject registration;
provider must already be installed and offer the exact capability version.
Multiple providers are permitted but caller explicitly selects one. Self binding
and cycles reject; no implicit rebinding. Descriptor is captured once on install,
validated and retained; callbacks cannot change it.

Observation contains stable PluginId, committed tick, own BTreeMap<String,i64>,
own RNG state and retained owned Event values, plus event history next_sequence
and dropped accounting. OwnerTokens/host nonce are excluded. Independent-plugin
comparison uses **equal-tick per-plugin resource/RNG/event projection**, not
whole-host discovery/save bytes or global event sequence counters. Compare B
resources/RNG and newly emitted/current-tick pending event (tick, owner, kind,
value) tuples, excluding Event.sequence and global next_sequence/dropped counters.
Whole retained-history equality is not promised: the global256 retention budget
means unrelated emitters can evict old B events. Comparisons filter observations
to the equal current tick; full-host save/hash still includes
global sequence. first_retained_sequence is the first sequence of the entire
host retained history (or next_sequence if empty), not owner-filtered events.
Discovery exposes contract/save versions, limits, installed exact descriptors,
resolved bindings, committed tick and removal blockers. It excludes tokens and
does not claim unimplemented capabilities.

## Fixed numeric bounds and validation

| Bound | Frozen value |
|---|---:|
| Installed plugins | 16 |
| Provided or required capabilities per descriptor | 32 each |
| Declared resources per plugin / actual records | 64 each |
| Declared event kinds per plugin | 32 |
| Plugin/capability ID or local key bytes | 128 |
| Emitted events per commit / pending events | 128 each |
| Retained history events across host | 256 |
| Transaction commands per plugin callback | 1024 |
| Error detail UTF-8 bytes | 4096 |
| Content binding UTF-8 bytes | 128 |
| Save input/output bytes | 1,048,576 |

IDs are ASCII dotted namespaces with at least two nonempty segments of
`[A-Za-z0-9_-]+`. Local resource/event keys use one nonempty segment with the
same alphabet. Content binding is caller-defined nonempty ASCII
`[A-Za-z0-9_.:-]+`, not a filename. Length is checked before allocation where
practical. Descriptor lists are sorted canonically; duplicates reject rather
than silently overwrite. Empty provides/requires/resource/event lists are valid.
Only declared keys/kinds may be written/emitted. Integer arithmetic used for
tick/sequence/generation/accounting is checked; overflow is a typed error and
cannot wrap. Plugin counter arithmetic must use checked operations itself.
RNG mixing explicitly uses modular u64 arithmetic as specified below.

Each set/emit/draw spends one command, including a call that fails validation.
Any denied/invalid/over-budget transaction operation **poisons** the transaction:
the callback cannot swallow its Result and commit partial work. The host rejects
it even if the callback returns Ok. No callbacks are run for invalid registration
or rejected target tick. A native callback's CPU time/hidden allocations cannot
be bounded by this command budget: plugins are trusted conforming code, not
sandboxed code. Panic/abort and malicious native code recovery are UNVERIFIED.

## Deterministic state and lifecycle

All mutable authoritative state is host-owned scalar records and RNG. Plugin
callbacks take `&self`; hidden interior-mutability state, external effects,
wall-clock/network/device access and unseeded randomness are forbidden by the
conformance contract. Type signatures do not prove native code obeys it.

Host starts at tick0. Install validates limits/descriptor/bindings, reserves a
fresh generation, then runs initialize at current tick against immutable
committed reads. Initialization stages own records/events/RNG; success publishes
everything together, failure publishes nothing and never reuses its generation.
Host/session identity and monotonically allocated generations are process-local,
excluded from saves/hashes. Host identity may use a checked process-wide nonce
counter for token rejection only; it is not simulation state or a global session
registry. Different host's tokens reject. Host must not be Clone.

Target tick must equal committed tick + 1. Past, duplicate or future ticks return
InvalidTick without callback/state mutation. Every plugin reads the same pre-tick
resources and preceding committed pending events. Callback order is dependency
topological order, with PluginId lexical tie-break. Each owns one write namespace.
Successful step commits all records/events/RNG and target tick atomically;
any callback/transaction error discards all staged changes and leaves the same
target retryable. No actions or button inputs exist yet: target tick is the
entire first-slice input. Tick0 initialization does not run tick callbacks.

Initial per-plugin RNG state is little-endian first eight bytes of
SHA256(seed.to_le_bytes() || PluginId ASCII bytes). draw uses SplitMix64:
state += 0x9e3779b97f4a7c15; z = state;
z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9;
z = (z ^ (z >> 27)) * 0x94d049bb133111eb;
return z ^ (z >> 31), with additions/multiplications modulo 2^64. Cursor/state
commits with the transaction. Installing/removing A never advances B's stream.

Event fields are tick:u64, owner:PluginId, kind:String, value:i64,
sequence:u64. Host assigns global sequence from next_sequence starting at0,
using callback order then emission order, only upon commit. Committed events
enter retained history and next-tick pending queue. A step replaces pending with
that step's events. Initialization appends events at current tick to pending;
total pending still cannot exceed128. Consumers read own events or declared
provider events only, always from committed pending at the start of callback.
Pending is a subset of retained history; newest history retention of256 and
pending bound128 preserve this. Overflow of produced/pending bounds fails commit.

History evicts oldest when exceeding256 and increments events_dropped per removed
history item. Removal purges owner's history/pending; purged history items also
increment events_dropped, pending removal does not count a second time. Therefore
next_sequence = retained_history.len() + events_dropped. Sequence gaps are valid
and exposed; first_retained_sequence is oldest history sequence or next_sequence
if empty. Removal never renumbers events or alters unrelated resource/RNG state.

remove accepts only a live own-host token between callbacks. A live dependent
binding returns DependencyInUse atomically; caller removes dependents first.
Success drops plugin object and own resources/RNG/capabilities/bindings/events,
revokes its token and retains unrelated owners. No arbitrary shutdown callback
or cascade exists. Install/remove are not available within Context/Transaction.
Reinstall same PluginId receives fresh generation; stale token never reactivates.

Stable ErrorCode minimum: InvalidIdentifier, InvalidDescriptor, VersionMismatch,
DuplicateId, MissingCapability, InvalidBinding, DependencyCycle,
PermissionDenied, UndeclaredKey, BudgetExceeded, StaleHandle, DependencyInUse,
InvalidTick, Overflow, PluginFailed, SaveMismatch, InvalidSave. Error includes
code + bounded detail; structured code is authoritative, not human wording.
Parent error mapping frozen for restore: malformed, noncanonical or structurally
invalid bytes return InvalidSave; content/seed/exact descriptor or binding mismatch
returns SaveMismatch. Registration contract/capability version incompatibility
returns VersionMismatch. Tests must not depend on human detail wording.

## Canonical save version1 and atomic restore

Save format version1 and contract1.0.0 are independent of old save/ABI versions.
Content binding is validated/stored immutably by Host::new; caller cannot relabel
a running session at save/hash time.
Canonical bytes use compact serde_json::to_vec on fixed-field-order structs with
deny_unknown_fields recursively, BTreeMaps for keyed state and canonical sorted
plugin arrays and descriptor/binding maps/sets. Every map key is unique; duplicate JSON keys
must be rejected (strict decoding or reject noncanonical input by exact byte
comparison after re-encoding). No whitespace/trailing data/floats/unknown fields.
Canonical input must exactly equal re-encoded bytes before mutation.

Fixed top-level field order: format_version, contract_version, content_binding,
seed, tick, plugins, history, pending, next_sequence, events_dropped. Each plugin
record order: descriptor, bindings, resources, rng_state. Plugin records sorted
by PluginId; descriptor field order follows the public definition above.
Event order in history/pending is ascending sequence. sha256 of canonical bytes
is returned as lowercase64-character hex. Tokens, session IDs/generations, device
and camera state never enter bytes/hash. save is read-only and bounded; invalid
binding/oversize returns Error. P1 hash invariance applies only to this new host,
never redefines ge4g_runtime::snapshot_hash or legacy view-persisted saves.

restore uses the **same already installed exact descriptor set and binding set**;
no auto install, no migrations, no absent/extra plugins or silent drops. It first
checks size/canonical encoding/version/immutable host content binding, same host seed, exact
descriptors/bindings, identifiers/bounds/declared resources/kinds, event owners,
RNG states, history/pending order and accounting. Pending must equal the entire retained-history projection with tick == saved
tick, not an arbitrary subset: initialization/step/removal preserve that rule.
Pending entries are distinct and exact history items. History event ticks are
nondecreasing and <= saved tick; sequences are strictly increasing and below
next_sequence. This rejects omission of upcoming consumer input during restore. Counts and accounting
must satisfy bounds and next_sequence = history.len + events_dropped. Every
record value is i64; resources can be absent if never set. PluginId wire owner
is stable; there are no scene/entity handles or typed references in this slice.

Only after full validation, reserve fresh generations for **all** installed
owners, then atomically replace tick/resources/RNG/events/counters. Generation
capacity is checked before any replacement. All previous tokens reject;
installations/descriptors/bindings remain. `Host::owner` is the explicit
host-authority method for obtaining restored live tokens. Tokens are authority
handles inside the trusted host, not confidentiality/security secrets.
Failed restore changes neither state nor existing token validity. Restoring the
same bytes yields equal canonical bytes/hash and next tick replay regardless of
fresh token generations. Restore never invokes plugin callbacks.

## Required evidence and deliberately omitted scope

P1 tests must prove independent unload/reinstall, dependency refusal/removal,
initialize and mid-tick rollback including RNG/events, poisoned transaction,
stale/cross-host/restored handles, wrong/missing/duplicate version/binding data,
budget limits, duplicate/past/future ticks and retry, canonical save roundtrip,
restored pending event consumption, malformed/noncanonical saves and atomic
failure, event history truncation/purge accounting, and equal-tick independent
resource/RNG and current-tick emitted-event projection, including a fixture that
crosses the global history eviction boundary. Historical retention depends on
the shared256 budget and is not an independence promise. Dependency graph must
show no engine crate imports.
Actual test commands/results belong to parent verification log, not this freeze.

All tests/features here are **UNVERIFIED until executed**. Broad P0 schemas,
Scene/Entity and typed references, actions/input mapping, format/view replacement,
gameplay extraction/composition, external source-free consumer proof, full legacy
platform regression and Windows ZIP root cause are **UNVERIFIED / out of this
slice**. Tiny Core → View/Format → Gameplay order remains. No Forge, marketplace,
dynamic native ABI or new platform-support claim is authorized by this contract.

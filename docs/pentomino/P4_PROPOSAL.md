# P4 native bridge proposal

Design history only. The implemented configuration/API names and native pixel
unit convention supersede the proposal's frozen-name sketch; P4_CONTRACT.md is
the authoritative implementation contract. Two-phase ownership/removal design
was accepted independently before production functionality was added.

Architect proposal, 2026-10-13. This document is not an implementation claim.
The parent must accept the bounded contract, have the auditor review it, and
update P4_CONTRACT.md to reflect the actual shipped signatures and evidence.
P0–P3 public contracts and ABI1 remain unchanged.

## Executable vertical slice

Add a Rust native bridge crate outside Tiny Core and a small linked turn-combat
plugin. Dependency direction is native bridge → P2 CoreHost, P3 ViewHost,
Gameplay, existing rendering adapter; Gameplay → P2 only. No World or legacy
runtime simulation is constructed to drive combat. The caller loads versioned
JSON game configuration, creates a session, maps named controls to P2 Actions,
advances one logical step through two fixed Core ticks, observes typed Records/Events, obtains native
CameraFrame and real CPU renderer pixels, saves/restores, and closes its handle.
Forge, a new renderer, Flutter UI changes and platform release work are excluded.

The minimum native surface may be linked Rust rather than a C ABI: the user asks
for a Rust-centred layer, and does not mandate C symbols. Version its discovery,
configuration, observation and save envelope independently of ABI1. If a C ABI
is added, all symbols must have a new versioned prefix such as `ge4g_p4_v1_`;
never reinterpret ABI1 handles, errors, saves or game packages. Do not claim C,
Android or Windows execution unless an actual consumer on that boundary ran.

Proposed Rust signatures (names are proposals; parent owns implementation):

```rust
pub struct NativeBridge { /* session table, no exposed Core handles */ }
pub struct SessionHandle { /* private registry nonce and generation */ }
impl NativeBridge {
    pub fn new() -> Self;
    pub fn create(&mut self, config_json: &[u8], seed: u64,
                  mode: ExecutionMode) -> Result<SessionHandle, BridgeError>;
    pub fn describe(&self, handle: SessionHandle) -> Result<SessionInfo, BridgeError>;
    pub fn step(&mut self, handle: SessionHandle,
                input: InputRequest) -> Result<Observation, BridgeError>;
    pub fn observe(&self, handle: SessionHandle) -> Result<Observation, BridgeError>;
    pub fn frame(&self, handle: SessionHandle) -> Result<Frame, BridgeError>;
    pub fn save(&self, handle: SessionHandle) -> Result<SaveBundle, BridgeError>;
    pub fn restore(&mut self, handle: SessionHandle,
                   save_json: &[u8]) -> Result<(), BridgeError>;
    pub fn detach_gameplay(&mut self, handle: SessionHandle) -> Result<(), BridgeError>;
    pub fn attach_gameplay(&mut self, handle: SessionHandle) -> Result<(), BridgeError>;
    pub fn set_camera(&mut self, handle: SessionHandle,
                      request: CameraRequest) -> Result<(), BridgeError>;
    pub fn close(&mut self, handle: SessionHandle) -> Result<(), BridgeError>;
}
```

A handle is opaque and never deserializable as a CoreOwnerToken. Its identity
includes bridge instance identity; a handle from another bridge, a closed
handle or a previously allocated generation is InvalidSession. Allocation must
never wrap or reuse an identity. Close succeeds once; all later requests,
including close, explicitly reject. Session slots and counts are bounded.

InputRequest is strict `{version:1,target_tick:u64,controls:[...]}`. Empty input
is allowed; attack is a discrete action, not a held-button sampler. Only named
controls explicitly configured in the game are accepted. Reject duplicate,
unknown, wrong-type or conflicting controls before creating the P2 InputFrame;
never use a map conversion that loses duplicate keys. The requested target is the current logical step + 1. No wall clock, variable
delta, asynchronous pending mutation, hidden mutable plugin RNG or hardware
device participates. A bridge step submits two P2 InputFrames at successive
Core ticks and one P3 presentation update at the next presentation tick. Expose
logical step, Core tick and presentation tick separately; ReadFrame source tick
is provenance and is not constrained to equal the P3 presentation tick.

## Small Gameplay plugin

Configuration version 1 declares exactly two distinct fighter identifiers,
positive bounded initial HP and attack damage, one public attack control, and
bounded world positions/colours used only by presentation. Validate before
allocation. Content binding is a deterministic digest of canonical validated
configuration, not a caller assertion and not a file path. Seed belongs to Core.
A sample must be a new P4 game JSON file, with documented public bridge runner.
It must not load FlatLand World and label the resulting runtime Pentomino.

The retained StateProvider owns a Scene, two Entities, one battle Record and two
position Records for P3 extraction. Its only update behavior copies its own
validated scalar Actions into its own Record. It contains no attack/turn rules.
The removable TurnCombat plugin requires the state capability through an
ordinary P2 read binding, declares the attack Action and computes a decision
from the committed battle Record. It emits one primitive decision Event; it
never writes the state provider's namespace. The bridge routes the decision's
scalar fields mechanically into state-provider Actions on the second tick.
There is no separate authoritative Rust combat struct and no mutation of Tiny
Core. The TurnCombat plugin is the only place that applies HP damage, chooses
next turn or determines the winner.

## Accepted two-phase fixed-step and literal removal

P2 `CoreHost::remove` deletes the removed owner's objects, records, events and
declared last-input actions. Required capability bindings grant only reads,
never foreign writes. Therefore battle authority must have a retained state
owner distinct from the genuinely removable rules owner. This design uses
P2's existing Actions and Events rather than changing permissions or substituting
an inert rules callback under an unchanged discovery descriptor.

One logical bridge step has exactly two fixed Core ticks:

1. Decision tick `2*n+1`: submit the configured attack Action to TurnCombat,
   or submit empty input. StateProvider receives no update Actions and does
   nothing. TurnCombat reads pre-tick committed authority through its capability
   and, for a valid attack, emits exactly one decision with primitive fields
   `hp0`, `hp1`, `turn`, `winner`, `attack_count`. After Core commit, immediately
   select the actual rules owner's pending Event at this exact tick. Validate
   its schema/kind/field types/ranges and expected count, then mechanically map
   fields to StateProvider Actions. No battle calculation belongs to the bridge.
2. Application tick `2*n+2`: submit those state Actions, or empty input if no
   attack. StateProvider applies the new Record. TurnCombat receives no attack
   Action and emits no decision. Only after this commit may the bridge update
   ViewHost from the new actual selection and return an observation/frame.

`pending` is replaced on the next Core tick, so the bridge must capture the
actual decision between the two commits, not query it afterwards or guess the
last retained Event. Require one decision for an accepted attack and zero for
empty input; wrong/missing/extra decisions reject the entire staged step. A
finished-battle attack explicitly fails in TurnCombat, leaving the old logical
step retryable. Observation histories retain actual decision Events. Decisions
never contain foreign object refs, avoiding unnecessary removal coupling.

Always use two Core ticks, including empty input and detached empty ticks, so
Core tick is exactly `2*logical_step` at the only exposed commit boundaries.
Use checked arithmetic. Presentation tick normally equals logical step, while
camera-only updates may have their own explicitly documented presentation
clock policy. P3 permits ReadFrame source tick to differ from presentation
update tick; do not invent a same-tick P3 requirement. Saves never expose an
intermediate phase or pending application transaction.

Detach literally obtains TurnCombat's current CoreOwnerToken and calls
`CoreHost::remove` on the staged candidate. The stable provider's whole selection,
SceneRef/EntityRef identities, records, RNG and owned history remain unchanged.
Its declared descriptor must not require TurnCombat, otherwise removal would
be correctly blocked. Core's overall save/hash changes because installed
plugins, purged history and counters change; compare stable owner authority,
not an incorrectly demanded identical whole-host hash. Empty logical steps
continue while detached; attack input explicitly returns GameplayDetached.
Reinstall literally installs a fresh TurnCombat plugin with the same immutable
config and capability binding to the retained provider. It initializes no battle
objects/records; ongoing HP, turn and attack count remain provider authority.

Use fixed configured damage and no draws in TurnCombat: reinstallation resets
that owner's RNG/history by P2 contract, so preserving a meaningful rule-owner
RNG stream would need a separate contract. The state provider's RNG remains
untouched. Gameplay attachment is inferred from actual Core discovery; the save
envelope records it only so fresh staging knows which descriptors to install.
No dynamic library machine-code unloading or process sandbox is claimed.

## Frozen Gameplay API for the implementer

Names below are the bounded proposal approved for the implementation packet.
The Gameplay owner may choose the exact crate name; parent owns Bridge/FFI and
shared workspace files. All constructors must validate and return ordinary
scoped errors instead of panicking on caller configuration.

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CombatConfig {
    pub version: u32,                // exactly 1
    pub fighters: [FighterConfig; 2],
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FighterConfig {
    pub id: String,                  // distinct bounded ASCII local IDs
    pub hp: i64,                     // 1..10000
    pub damage: i64,                 // fixed, 1..10000
    pub position: [i64; 2],          // generic native world units, each -512..512
    pub color: [u8; 4],              // presentation metadata, not authority
}
pub const UNITS_PER_WORLD: i64 = 1000;
impl CombatConfig { pub fn validate(&self) -> Result<(), GameplayError>; }
pub struct StateProvider { /* immutable validated config */ }
pub struct TurnCombat { /* immutable validated config */ }
impl StateProvider {
    pub fn new(config: CombatConfig) -> Result<Self, GameplayError>;
}
impl TurnCombat {
    pub fn new(config: CombatConfig) -> Result<Self, GameplayError>;
}
// Both implement existing p2::CorePlugin; no Core API additions.
pub fn state_owner() -> PluginId;        // p4.state
pub fn rules_owner() -> PluginId;        // p4.turn_combat
pub fn state_capability() -> CapabilityId; // p4.state.records
pub fn rules_bindings() -> Bindings;     // capability -> state_owner
pub fn selection(include_rules: bool) -> Selection;
pub fn attack_action() -> ActionId;      // p4.turn_combat.attack, Bool
pub fn position_schema() -> SchemaId;    // p4.state.position
pub fn extract_decision_actions(
    read: &ReadFrame, decision_tick: u64, expected_attack: bool,
) -> Result<Vec<(ActionId, ActionValue)>, GameplayError>;
pub fn battle(read: &ReadFrame) -> Result<BattleState, GameplayError>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BattleState {
    pub hp0: i64,
    pub hp1: i64,
    pub turn: i64,                      // 0 or 1
    pub winner: i64,                    // -1 unfinished, 0 or 1 finished
    pub attack_count: i64,              // 0..1_000_000
}
```

Provider installs capability version 1.0.0 and Battle schema `p4.state.battle`.
Its Battle record local key is `battle` and starts at config HP, turn 0,
winner -1, attack_count 0. Provider-owned Actions are `p4.state.hp0`,
`p4.state.hp1`, `p4.state.turn`, `p4.state.winner`, `p4.state.attack_count`,
all bounded I64. Missing all update Actions means no-op; a partial group is
InvalidInput. Copy all five validated values atomically into one Battle record.
The provider does not decide damage, next turn or winner. Its initialization
creates Scene local `arena`, Entities `fighter0` and `fighter1`, and position
Records `position0`/`position1`, each with entity ref and `x`, `y`, `z` I64.
Position coordinates multiply configured native values by UNITS_PER_WORLD
with checked arithmetic; z is zero. P3 units_per_world is 1000. Config IDs are
labels, not a source of undocumented wire keys. Selection includes provider
objects and histories and optionally rules history; positions and battle must
be discoverable without private plugin fields.

TurnCombat declares required state capability and attack Bool. Missing/false
attack means no-op; true attack uses turn's configured fixed damage, saturates
target HP at zero using checked bounded arithmetic, increments attack_count,
sets winner on zero HP, otherwise flips turn. It emits kind `decision` with
schema `p4.turn_combat.decision` and exactly the five Battle fields. It owns no
battle objects or authority records; initializer is no-op. Callback errors use
P2 Error with explicit InvalidInput/Overflow/validation detail. Helper/factory
errors may be a separate GameplayError. The helper checks actual event tick,
owner/kind/schema and five primitive fields, then returns the provider Action
array in deterministic order; it contains no combat rule recomputation.

The parent may add config renderer settings in its separate bridge envelope;
it must not change this Gameplay packet's owned config files concurrently.
The rendering adapter uses fighter position Records and colour from immutable
validated config, and battle fields from the actual Battle record. It converts
native Y-up to the existing renderer's pixel Y-down explicitly. No legacy World
owns, calculates or advances combat.

## Atomicity across Core, presentation, frame and restore

P2 and P3 each provide their own atomic host operations, but two successive
mutations of active hosts are not a bridge transaction. Saving then restoring
an already-mutated active Core as rollback invalidates runtime tokens even if
saved bytes recover. Never advertise exact atomicity using that strategy.

For every operation that can change multiple domains, build fresh staged hosts
with immutable validated configuration and the current actually installed rules mode, restore
current Core save2 and presentation save1, perform the requested changes on
those hosts, refresh native selection, and render/validate output in linked
mode before committing the replacement Session. Any error drops the candidate;
the previous session, frame, bytes, handles and clock remain untouched. Pure
callbacks contain only immutable configuration; no hidden state/external side
effects can escape failed staging. The bounded two-fighter workload makes this
simple correctness-first implementation appropriate; cost is a documented
limitation, not a reason to rewrite P2/P3.

SaveBundle version 1 separates exact `core_save2` bytes from exact
`presentation_save1` bytes, plus config binding and rules attachment metadata.
Byte arrays or validated UTF-8 strings may represent the two inner payloads;
the chosen encoding must be explicit. Restore accepts only the exact expected
bridge version/content binding/configuration and strict inner canonical saves.
Reject unknown fields, mismatched seed, descriptor/configuration, malformed
canonical bytes, stale references and tick contradictions. Linked/headless
execution mode is an execution choice, not combat authority. Define whether a
bundle may be restored across modes; recommend yes. Do not auto-migrate ABI1,
FlatLand, scalar save1, or game packages. Do not treat ReadFrame as a save file.

P3 needs a live actual read selection after Core restore. Construct the ViewHost
with exact installed configs/current real SceneRef, then restore its bytes
against that selection. Core and presentation tick must remain compatible and
cannot silently be normalized on an invalid save. Camera-only operations may
stage only presentation if no Core token can change, but the conservative whole
session staging implementation is acceptable.

## Observation DTO and real renderer boundary

ReadFrame contains `BTreeMap<RecordKey, Record>`; RecordKey is a structured
object and cannot be used directly as a nonempty JSON object key. Proposed
strict observation version 1 converts it into sorted rows:

```json
{
  "version": 1,
  "tick": 0,
  "rules_attached": true,
  "records": [
    {"owner": "p4.combat", "local": "turn", "record": {}}
  ],
  "scenes": [],
  "entities": [],
  "histories": [
    {"owner": "p4.combat", "history": {}}
  ],
  "camera_frames": []
}
```

The illustrative empty objects above stand for actual typed P2 Record and
OwnerHistory values. Records are sorted by (owner, local); no flattening into
concatenated strings, omitted state, lossy field conversion or runtime tokens.
CameraFrames are actual P3 outputs, sorted by camera ID. The renderer consumes
the actual selected fighter state and those CameraFrames; no FlatLand World
simulation drives the observed data.

The existing supported CPU renderer `ge4g_render2d::render` and the P3
`render_legacy(Project, Snapshot, CameraFrame, ...)` adapter can be reused with
a new small declared presentation Project and Snapshot synthesized from actual
P2 observations. This is a renderer DTO conversion only. It must not call
World::step or existing FlatLand combat/event paths. Omit legacy battle event
payloads so the renderer uses its supported world/entity path. Use fighter
colour/geometry and HP presentation derived from real Records; frame changes
must be measured across actual inputs/camera changes. No fabricated PNGs or
full-frame warping. Configure only the P3 adapter's proven orthographic,
identity-rotation, bounded zoom/tilt/shear subset and expose Unsupported for
other requests. CameraFrame must actually control projection, not merely be
returned alongside pixels from a separate legacy camera.

Frame is the actual ge4g_render2d Frame with RGBA8 dimensions and exact byte
length width*height*4; the bridge may wrap tick/camera identity and SHA256.
Headless excludes rasterization, never excludes Core simulation. Frame in
headless may explicitly return Unsupported; tests compare Core authority
between modes and separately validate linked pixels. No GPU/window is needed.

## Bounded requests, errors and tests

Suggested bridge bounds: 16 sessions; JSON config/input ≤64 KiB; two fighters;
viewport dimensions 1..512; pixels ≤512*512*4; 16 controls/frame (at most one
attack); HP/damage 1..10000; coordinates within adapter-proven safe bounds;
error details ≤4096 UTF-8 bytes; save bundle ≤16 MiB with inner P2/P3 limits
also enforced. Validate byte/count bounds before accepted-state clones.
Native source immutability and transaction assumptions are documented; trusted
callbacks are not a process sandbox. Do not introduce unbounded JSON or opaque
third-party engine dependencies.

Use a new scoped BridgeError enum: InvalidSession, InvalidConfig, InvalidInput,
InputOutOfSequence, GameplayDetached, InvalidSave, SaveMismatch, VersionMismatch,
Unsupported, BudgetExceeded, Overflow, CoreFailure, ViewFailure, RenderFailure.
Preserve source error code/detail in a bounded structured field. No swallowed
errors, panic-as-success, fallback camera, or scalar/P2/P3 enum changes.

Acceptance tests execute public NativeBridge calls and assert observations,
Core save/hash, event contents and actual pixel hashes, not only success codes:

- Same seed/inputs produce byte-identical results; headless/linked Core match.
- Mid-battle combined save/restore and subsequent replay exactly match control.
- Bad inputs and separately bad Core/presentation save payloads preserve the
  complete old bundle and cached frame, with the next valid action still usable.
- Closing a session rejects every future operation and foreign bridge handles.
- Literal rules remove/reinstall preserves exact state-provider selection/IDs,
  RNG/history, blocks attack while detached, allows empty logical steps, and
  resumes the retained battle without initialization. Whole Core hash naturally
  reflects plugin removal and is not the invariant.
- Camera/View replacement changes CameraFrame/projection/pixels while full Core
  save/hash and combat fields stay equal, including a failed camera request.
- Actual native input → tick → Records/Events → P3 frame → RGBA delivery is
  exercised through a small public runner and a consumer-owned out-of-tree
  example. A parent-controlled public-API check is internal evidence; only a
  separate source-free session can provide external consumer certification.

Required unchanged P0–P3 and four legacy game regressions, workspace fmt,
locked clippy with denied warnings, locked workspace test and evidence logs
remain mandatory. Preserve tests and their strength. Record actual compiler,
commit, commands, platform and CI links. Unrun platform and external gates are
UNVERIFIED, never inferred from a Linux Rust test.

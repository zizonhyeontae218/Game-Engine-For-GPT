# P4 Native Bridge & Gameplay contract v1

GE4G 0.3.0-alpha.1. Additive experimental contract, not FlatLand ABI1 or a
platform release. Baseline main: `8622034d1f0082a3178dc2e7262539a50a064c26`.
Implementation: `ge4g-pentomino-gameplay` and `ge4g-pentomino-native`.
No P0–P3 production/test source, existing game, ABI1 or save format is changed.

## Execution and ownership

NativeBridge → public P2 CoreHost + Gameplay + public P3 ViewHost + existing
P3 CPU-render adapter. Gameplay → P2 only; Tiny Core imports neither Gameplay
nor View. No FlatLand World, battle engine, replay or runtime step drives this
new game. The legacy-named rendering adapter is used solely for presentation.

StateProvider (`p4.state`) owns one Scene, two Entities, two position Records
and one status Record with HP, active turn, winner and attack count. Its
`p4.battle_state` capability grants read access. Its scalar Actions replace a
complete five-field status batch; partial batches reject. It performs generic
schema-bound storage, not damage/turn/victory rules. It emits `applied` Events.

TurnCombat (`p4.turn_combat`) requires/binds that capability. It reads status,
accepts its own Bool `p4.turn_combat.attack` Action and emits a `decision` Event
with five scalar fields. Fixed configured damage alternates fighters and ends
when one HP reaches zero. Its initializer creates no authority objects/state.
No hidden mutable callback state or randomness is used. Seed is still owned by
Core; equal seed/input reproduces full state. Different seeds are not promised
to change combat outcomes for these fixed rules.

One logical input executes **exactly two Core ticks**:

1. Decision tick: submit the mapped attack, or empty actions. State remains
   unchanged; TurnCombat emits the proposed result.
2. Application tick: select actual pending decision at the exact first tick,
   validate owner/schema/kind/field values/count against current battle state,
   then map it to five StateProvider Actions. Never send attack in this tick.
   StateProvider commits and emits `applied`. Empty input performs two empty ticks.

At public boundaries `core_tick = 2 * logical_step`, checked for overflow.
P3 updates once per logical step using the final real ReadFrame. Camera-only
updates can advance presentation time without advancing Core. All three clocks
are explicit; `tick_hz` config is the Core fixed-rate declaration, so logical
input frequency is half that rate if a scheduler chooses real-time pacing.
The bridge itself uses no wall clock or variable delta.

P2 clears pending Events at the next Core tick; public observations retain the
actual decision in rule history and applied result in carrier history. Removal
purges rule history as specified by P2; this is not permanent audit logging.

## Configuration and plugin extension

`samples/pentomino_p4/game.json` is independent new content. All DTOs deny
unknown fields. Config version1:

```json
{"version":1,"width":160,"height":96,"tick_hz":60,"attack_control":"attack",
 "fighters":[
   {"id":"blue","hp":12,"damage":4,"position":[40,72],"color":[48,144,255,255]},
   {"id":"red","hp":11,"damage":3,"position":[120,72],"color":[255,80,72,255]}]}
```

Exactly two distinct single-segment fighter IDs; HP/damage1..10000;
viewport32..512; tick_hz1..240; control ASCII alphanumeric/underscore1..64;
fighter ID1..128; RGBA four u8 values. Positions are pixel feet, bounded so
16×24 geometry and HP bar fit the viewport. Position Records use native
`x=pixel_x, y=-pixel_y, z=0`, units_per_world1.0. Semantics belong to the plugin.
Binding is `p4:` + SHA256 of canonical validated config. Seed and config must
match on restore. Config/path is not executable code or an ABI1 game package.

Public Gameplay APIs: `StateProvider::new`, `TurnCombat::new`, `state_owner`,
`rules_owner`, `capability`, `bindings`, `attack_action`, `position_schema`,
`selection`, `scene`, `fighter_entities`, `battle_state`, `decision_actions`.
Both plugin types implement public P2 CorePlugin. They can be linked/installed
by another public CoreHost consumer. No dynamic-library loader, sandbox or
arbitrary executable content import is provided. Changing combat rules or
immutable configuration requires a new contract/content binding and explicit
migration; v1 does not silently interpret saves under different rules.

## Native Rust and C interfaces

Public Rust `NativeBridge`: `new`, `create(config_json, seed, ExecutionMode)`,
`describe`, `observe`, `step(InputRequest)`, `frame`, `save`, `restore(save_json)`,
`detach_gameplay`, `attach_gameplay`, `set_camera`, `set_view`, `close`, `request`.
Methods use opaque Copy SessionHandle and return scoped BridgeError. Modes are
`Headless` and `Linked`; both run identical Core/View state. Headless validates
the supported camera subset and skips rasterization; frame returns Unsupported.
Linked frame returns actual RGBA8 and SHA256. No window/GPU/audio is required.

New symbols, documented in `include/ge4g_p4_v1.h`:

```c
uint32_t ge4g_p4_v1_abi_version(void); /* 1 */
char *ge4g_p4_v1_request(const uint8_t *data, size_t len);
void ge4g_p4_v1_string_free(char *value);
```

Requests borrow readable bytes for the call; responses are owned NUL-terminated
UTF-8 JSON, freed exactly once with this ABI's free function. Null/empty/oversized
requests yield explicit errors. Caller must supply valid pointers and lengths;
arbitrary invalid native pointers and allocator exhaustion are not sandboxed.
There is no callback or concurrent access to one session. **C sessions belong to
their creating thread**; use on another thread rejects InvalidSession. Thread
exit drops that thread's sessions. Thread affinity is necessary because P2/P3
trusted plugin traits do not promise Send; no unsafe Send implementation is added.

Handles are process-wide monotonically allocated u64 values, never recycled or
wrapped. Each Rust bridge/thread table accepts only its own live IDs. Close drops
the session, frame and hosts; all later operations including repeated close reject.
No Core owner/object token escapes or appears in serialized discovery. Numeric
wire handles are routing IDs for trusted callers, not a security boundary.

Request envelope: `{"version":1,"command":{"op":..., ...}}`.
Response: `{"version":1,"ok":true,"result":...,"error":null}` or
`{"version":1,"ok":false,"result":null,"error":{"code":...,"detail":...}}`.

| op | command fields / result |
|---|---|
| create | config, seed:u64, mode:`headless`/`linked`; result `{session:u64}` |
| describe | session; result SessionInfo with P2/P3 discovery |
| step | session, input:`{version:1,target_step:u64,controls:[string]}`; result Observation |
| observe | session; result Observation |
| frame | session; result FrameDto with clocks,width,height,rgba:[u8],sha256 |
| save | session; result SaveBundle |
| restore | session, save:SaveBundle; result null |
| gameplay | session, enabled:bool; result null |
| camera | session, request:CameraRequest; result null |
| view | session, family:`Classic2D`/`TopDown`/`Side`/`Vertical`; result null |
| close | session; result null |

Input is discrete, not held-button sampling. Target_step must be current+1;
controls is empty or exactly one configured attack. Unknown, duplicate, wrong
type, state-provider Action names, invalid version, attack while detached or
after victory reject without advancing any clock. Empty input after victory is
valid. Already attached/detached lifecycle requests explicitly reject InvalidInput.

Observation version1 has logical_step/core_tick/presentation_tick,
gameplay_attached, battle `{hp:[i64;2],turn,winner,attack_count}`, scenes, entities,
camera_frames and explicit ordered DTO rows:

```text
records:   [{owner:PluginId,local:string,record:p2::Record}, ...]
histories: [{owner:PluginId,history:p2::OwnerHistory}, ...]
```

Records sort by (owner,local), histories by owner. P2 ReadFrame's structured
RecordKey map is never serialized as JSON object keys or used as a save format.
Scene/Entity references are actual selected objects; CameraFrames are actual
P3 outputs. No flattened keys or runtime tokens are substituted.

## Removal, atomicity and separate saves

Detach literally calls public `CoreHost::remove` on TurnCombat. It deletes that
owner's descriptor/actions/events/RNG as P2 specifies. StateProvider has no
dependency on rules and stays installed with unchanged Records/Scene/Entity refs.
Whole Core save/hash changes on removal; carrier authority remains intact.
Detached empty ticks continue; attack rejects. Reattach literally installs a
new rule owner, bound to the existing carrier, and resumes the current turn.
Native machine code stays linked; this is plugin-instance unload, not dlclose.

Every multi-domain mutation constructs fresh candidate Core/View hosts, installs
the exact current descriptors/configs, restores current canonical saves, applies
the requested operation, validates and renders before replacing the live session.
Failed candidates are discarded; the live Core/View/frame and session ID survive.
Successful staging replaces private Core tokens; no public Core tokens can become
stale because none are exposed. Trusted plugins contain immutable config only;
arbitrary hidden callback side effects/panics are outside rollback guarantees.

SaveBundle version1 has content_binding, seed, logical_step, presentation_tick,
gameplay_attached, view_family and **separate exact UTF-8 strings** `core_save2`
and `presentation_save1`. Mode and pixels are not saved; restoring across modes
is supported with matching seed/config. No implicit FlatLand/scalar migration.

Inner P2/P3 canonical encodings, versions and installed configurations validate
first. Bridge additionally rejects odd/contradictory clocks, attack count above
logical steps, extra records, inconsistent applied-history count/final status,
wrong event phase, unapplied pending decisions and phase1 last-input actions.
`battle_state` verifies HP/turn/winner/count are reachable under v1 fixed rules
and that live identities/positions match config. Presentation restore verifies
metadata tick and the CPU subset for sampled base **and both transition endpoints**.
Malformed or mismatched Core/presentation saves never partially replace a session.
This validates consistency, not save authenticity or tamper-proof storage.

## Camera and actual frame delivery

CameraRequest is `{version:1,zoom:f64,tilt:f64,shear:f64,origin:[f64;2],transition_ticks:u32}`.
Zoom.25..4, tilt.25..2, shear±.5, finite origin offsets±1024, transition0..120.
Origin is relative to viewport-centre World origin. Zero transition is Instant;
positive counts use P3 Smooth. P3 static cameras use identity orientations and
orthographic projection; current P4 adapter does not accept general rotations,
perspective, models/GPU scenes or camera mutations outside its proven subset.
The P3 mathematical API retains its broader independent capabilities unchanged.
View replacement uses actual P3 Format plugins and policies. With static cameras
and no follow target, different policies can render identical pixels; no fake
projection difference is introduced merely to differentiate family labels.

The renderer receives a public Project/Snapshot presentation DTO assembled from
validated actual P2 Records and EntityRefs, and the real P3 CameraFrame.
`ge4g_pentomino_view_legacy::render_legacy` then calls the existing canonical CPU
`ge4g_render2d::render`. Only supported entity geometry/colour/HP bars are used;
no legacy battle/event simulation runs. It composes upright sprites at projected
feet, never warps a completed framebuffer. Snapshot.scene is `p4_arena`, and
the actual carrier Scene local is `s_` + SHA256(`p4_arena`) to satisfy the adapter's
explicit source-label contract. CameraFrame reference is never rewritten.

Limits:16 live sessions/bridge; config Rust bytes and accepted canonical config
≤65536; whole JSON request/save envelope≤24MiB; Core≤8MiB; presentation≤1MiB;
two fighters; frame≤512×512×4; at most one control; error detail≤4096UTF-8 bytes.
Wire RGBA is a JSON byte array for inspectability; binary frame transport and
staging performance improvements are future work. JSON parsing and trusted native
caller allocations are not a malicious-code resource sandbox.

Scoped error codes: invalid_session, invalid_config, invalid_input,
input_out_of_sequence, gameplay_detached, invalid_save, save_mismatch,
version_mismatch, unsupported, budget_exceeded, overflow, core_failure,
view_failure, render_failure, internal_failure. P2/P3 failures keep original
scoped code/detail in bounded detail text. No old enum or ABI1 error is changed.

## Reproduce and public consumer

```sh
cargo test --locked -p ge4g-pentomino-gameplay -p ge4g-pentomino-native
cargo run --locked -p ge4g-pentomino-native --example p4_public -- samples/pentomino_p4/game.json /tmp/p4-frames
cargo build --locked -p ge4g-pentomino-native
python3 samples/pentomino_p4/consumer.py target/debug/libge4g_pentomino_native.so
```

The ctypes consumer requires only the shared library, JSON config and its own
source. It exercises the real C ABI and never imports engine source. A packaged
out-of-tree run is internal reproducibility, **not separate-session external
consumer certification**. See P4_RUNDOWN/evidence and P4_HANDOFF. Android/Windows
client delivery, source-free external certification and release publication are
not implied by Linux-hosted native checks.

# P5 bounded Gameplay v2 proposal

Status: architecture proposal, not implementation or a PASS claim. 2026-10-30.
Parent approval and independent API audit precede functionality changes.

## Boundary and compatibility

Add `ge4g_pentomino_gameplay::v2` and
`ge4g_pentomino_native::p5`; retain P4 Rust APIs, v1 JSON, saves and
`ge4g_p4_v1_*` symbols unchanged. Gameplay imports P2 only. Native composes
P2 authority, removable rules, P3 ViewHost and the existing CPU renderer.
No FlatLand simulation, parallel authoritative engine, new dependency, Core
change or implicit P4/FlatLand save migration is needed.

P2 Actions are Bool/I64 only, with at most 32 declared Actions per plugin.
P5 command data therefore uses four scalar Actions. The provider's complete
state batch has at most 28 scalar Actions for four fighters. Records remain
structured P2 data; public observations use ordered record/history arrays.

## Public Gameplay API (proposed signatures)

Types live in `v2`; all wire DTOs derive Serialize/Deserialize with unknown
fields denied. Identifiers retain P2's bounded single-segment grammar.

```rust
pub struct FighterConfig {
    pub id: String, pub team: u32,
    pub hp: i64, pub resource: i64,
    pub attack: i64, pub defense: i64, pub speed: i64,
    pub position: [i64; 2], pub color: [u8; 4],
}
pub enum Target { SelfOnly, Ally, Enemy }
pub enum Effect { Poison, Regen }
pub enum ActionKind { Damage, Heal, Resource, Effect }
pub struct ActionConfig {
    pub id: String, pub kind: ActionKind, pub target: Target,
    pub cost: i64, pub power: i64,
    pub effect: Option<Effect>, pub duration: u32,
}
pub struct GameConfig {
    pub version: u32, // exactly 2
    pub width: u32, pub height: u32, pub tick_hz: u32,
    pub fighters: Vec<FighterConfig>, pub actions: Vec<ActionConfig>,
}
impl GameConfig { pub fn validate(&self) -> Result<(), p2::Error>; }

pub struct Status { pub power: i64, pub turns: u32 }
pub struct FighterState {
    pub hp: i64, pub resource: i64,
    pub poison: Status, pub regen: Status,
}
pub struct BattleState {
    pub fighters: Vec<FighterState>,
    pub turn: usize, // index in immutable configured roster
    pub winner: i64, // -1 running; -2 draw; otherwise team ID
    pub command_count: i64, pub last_command_id: i64,
}
pub struct Command {
    pub id: i64, pub actor: usize, pub action: usize, pub target: usize,
}
pub struct StateProvider { /* immutable config only */ }
pub struct TurnCombat { /* immutable config only */ }
impl StateProvider { pub fn new(config: GameConfig) -> Result<Self, p2::Error>; }
impl TurnCombat { pub fn new(config: GameConfig) -> Result<Self, p2::Error>; }
// Both implement public p2::CorePlugin.

pub fn state_owner() -> PluginId; // p5.state
pub fn rules_owner() -> PluginId; // p5.turn_combat
pub fn capability() -> CapabilityId; // p5.battle_state
pub fn bindings() -> Bindings;
pub fn selection(include_rules: bool) -> p2::Selection;
pub fn position_schema() -> p2::SchemaId;
pub fn scene(read: &p2::ReadFrame) -> Result<p2::SceneRef, p2::Error>;
pub fn fighter_entities(read: &p2::ReadFrame)
    -> Result<Vec<p2::EntityRef>, p2::Error>;
pub fn battle_state(read: &p2::ReadFrame, config: &GameConfig)
    -> Result<BattleState, p2::Error>;
pub fn command_actions(command: &Command, config: &GameConfig)
    -> Result<Vec<(p2::ActionId, p2::ActionValue)>, p2::Error>;
pub fn decide(state: &BattleState, command: &Command, config: &GameConfig)
    -> Result<BattleState, p2::Error>;
pub fn decision_actions(read: &p2::ReadFrame, tick: u64,
    expected: &Command, config: &GameConfig)
    -> Result<Vec<(p2::ActionId, p2::ActionValue)>, p2::Error>;
```

Wire enums use snake_case. Config permits 2..4 fighters, exactly two distinct
teams numbered 0/1, 1..16 distinct action IDs, HP/resource/attack/defense/speed
bounded to 0..10000 (HP at least 1), finite integer geometry matching P4 viewport
bounds. Initial resource is also its maximum. Action cost/power are 0..10000;
effects require nonzero power and duration 1..100, other kinds require no effect
and duration zero. Reject contradictory fields and all unknown versions.

Status has power=turns=0 when absent. Provider status fields are the roster's
six scalars per fighter (`hp`, `resource`, poison power/turns, regen power/turns)
plus turn, winner, command_count and last_command_id: maximum 28. Immutable
team/stats/positions come from the validated content configuration. StateProvider
owns Scene/Entities/position Records and generic complete-batch storage, with no
combat algorithm. Rule initializer creates no authoritative state.

## Rules and exact-once behavior

Turn order is descending speed, then original roster index; skip dead fighters.
Initial turn is the first live fighter. An action requires actor=current turn,
living actor and target, correct self/ally/enemy relation and resource >= cost.
`Ally` permits self; `Enemy` requires a different team. Subtract cost first.
Damage is `max(1, actor.attack + power - target.defense)`, clamped at zero HP.
Heal/resource add power and clamp at configured maxima; no resurrection.
Effect replaces that target's effect power/duration rather than stacking it.

After the chosen action, tick the acting fighter's poison (subtract power), then
regen only if still alive (add power, capped), decrement each positive duration
once and clear zero-duration powers. Clear effects of every dead fighter. Resolve
outcome after both the action and actor effects: no live team means draw; one live
team means winner; otherwise select next live fighter in fixed order. Terminal
turn remains the acting fighter index. `winner != -1` derives the finished phase;
all further action commands reject. There is no implicit AI turn or wall clock.

Command IDs are positive i64 and strictly increase; gaps are allowed. Accepted
ID, ordinal count and resulting state are committed together in the carrier.
ID <= last_command_id rejects atomically, including identical retries, conflicting
payload retries and stale commands. A rejected command does not consume its ID.
Exactly-once means no duplicate application; it does not promise successful retry
receipts or deduplication across an explicit restore to an older timeline.

A logical command is exactly two Core ticks: four complete rule-owned command
Actions produce one current-tick decision Event, then bridge validates event
owner/schema/tick/expected command and translates its complete result into the
provider's scalar Actions. Partial/extra command batches or partial result batches
reject. Only provider writes authority. Application emits one `applied` Event.
No saved or public session may stop between these ticks.

## Native boundary and saves

P5 exposes an independent `p5::NativeBridge` with `create`, `describe`, `observe`,
`act(handle, Command)`, `frame`, `save`, `restore`, `detach_gameplay`,
`attach_gameplay`, `set_camera`, `set_view`, `close`, and `request`. Reuse existing
ExecutionMode/CameraRequest/error DTOs where their meaning is identical. Add
`ge4g_p5_v1_abi_version/request/string_free` and `include/ge4g_p5_v1.h`.
Request envelope remains version1 for this distinct P5 ABI; gameplay config is
version2. C handles remain globally unique and creating-thread-affine. No unsafe
Send and no UI-side combat simulation. Allow P5 symbol prefixes explicitly in
`scripts/check_p4_boundaries.py`; retain its P4/Core dependency checks.

SaveBundle version2 binds canonical validated config (`p5:` + SHA256), seed,
logical step, gameplay attachment, view family, presentation clock, exact
`core_save2` and exact `presentation_save1`. Also retain a deterministic bounded
trace of accepted commands and actual detach/attach lifecycle operations. Reject
trace overflow (4096 operations) atomically. Journal is bridge-owned validation
metadata, never an alternative authority. Core history truncation at 256 must not
truncate the replay journal.

Restore stages fresh hosts and actually replays trace through these same P2
plugins, verifying final canonical Core save/attachment/clocks exactly against the
bundle. P3 restores its independent save and validates camera targets as in P4.
Trace entries preserve command payloads and order. Lifecycle operations do not
advance logical/Core ticks; no-ops attach/detach reject and are not journaled.
Commands advance logical step once; presentation updates once per command.
Camera-only changes may advance presentation independently and are saved by P3.
Cross-mode restore is valid with matching seed/content. Invalid/contradictory saves
or rendering failures leave active Core/View/frame/handle unchanged. Staging may
replay history; measure representative 100-command save/restore before optimizing.

P3 uses P5 Scene/position Records via its existing public RecordBinding. Renderer
maps the actual roster and HP to the existing canonical CPU composition; it must
not invoke FlatLand World or rewrite CameraFrame references. JSON RGBA transport
is acceptable for this milestone; binary transport is a separate optimization.

## Ownership, evidence and completion

Gameplay worker owns `gameplay/src/v2.rs` and its new contract tests; native
worker owns `native/src/p5*.rs`, new header and native tests. Parent alone edits
shared `lib.rs` exports, scanners, docs, samples, CI and manifest/hash ledger.
P4 files retain their original behavior. Existing Rust/C P4 consumers must pass.

Independent tests prioritize a hand-calculated resource/effect/turn oracle;
invalid and duplicate command state/frame preservation; detach/save/restore/
reattach with persistent ID; replay through new headless/linked sessions;
schema-valid tampering rejection; and existing compatibility acceptance. Use one
small sample to prove creation, action selection, statuses, victory, save,
restore and identical replay. Record measured execution separately from source
review and later physical user acceptance. No existing PR merge, release, signing
or deployment is authorized by this proposal.

# P2 public API guide

Import `ge4g_pentomino::p2::*`; reuse root `PluginId`, `CapabilityId`, `Version`
and `Bindings`. P1 root errors remain a separate unchanged API. P2 errors expose
`code()` and bounded `detail()`; old root errors convert into P2 errors.
See `CONTRACT.md` for every type, signature and exact numerical bound.

| Surface | Public behavior |
|---|---|
| `CorePlugin` | `descriptor`, `initialize(ctx, tx)`, `tick(ctx, tx)`; linked deterministic callbacks |
| `CoreHost::new(seed, content_binding)` | Isolated seeded host; no clocks/devices |
| `install(plugin, bindings)`, `owner(id)`, `remove(token)` | Exact version bindings; removal blocks installed dependents; atomic initialization/unload |
| `step(InputFrame)` | All plugins execute in dependency order against pre-tick snapshots; whole-tick commit or rollback |
| `select(&Selection)` | Explicit owner/schema selection; optional identities/history; empty owner set selects none |
| `describe()` | Contract/save versions, limits, installed descriptors/bindings/blockers, tick and last input |
| `scene_handle`, `entity_handle`, `resolve_scene`, `resolve_entity` | Opaque runtime handles are stale after remove/recreate/restore or in another host |
| `save()`, `restore(bytes)`, `hash()` | Capped canonical save2; strict atomic rejection; matching plugins must already be installed |
| `CoreContext` | `tick`, own/bound-provider `read`, `scene`, `entity`, prior committed `events`, declaring owner's `action` |
| `CoreTransaction` | `create_scene`, `create_entity`, removals, `set`, `delete`, `emit`, deterministic `draw` |

## Records, references and input

Schemas are exact required-field `BTreeMap`s of declarative `FieldType`:
`Bool`, bounded `I64`, UTF-8 `String`, `Bytes`, `Ref(Scene/Entity)` and bounded
recursive `List`. Records contain a `SchemaId` and matching typed `Value` fields.
Extra/missing fields, wrong types/ranges, forged refs or undeclared schemas fail.
No JSON escape hatch, executable validator or callback schema is available.

`SceneRef`/`EntityRef` carry owner, local ID and incarnation; entities also have a
scene scope. Saved refs identify objects on the restored timeline. They are data,
not owner authority or persistent freshness tokens. Use opaque handles for runtime
freshness; acquire replacements after restore. Incarnations resume from the saved
identity counter, permitting identical deterministic continuation after rewind.

References in records, entity links and retained events prevent unsafe removal.
Delete records/remove children first; retained event references last until eviction
or owner unload. Foreign refs require an explicit bound provider. Foreign mutation
is rejected. A callback reads only its own and bound committed state; own newly
created refs are valid in its transaction. Final validation rejects removal races.

`InputFrame { target_tick, actions: Vec<(ActionId, ActionValue)> }` targets exactly
the next tick. Actions support `Bool` and bounded `I64`; duplicate/unknown/wrong
type/range inputs fail. Core sorts accepted actions; missing actions are absent,
with no held/default interpretation. Only the declaring plugin can read an action.
Remove filters that owner's actions from the installed-owner last-input projection.

## Bounds and atomic failure

Discovery exposes the actual limits: plugins16; schemas/fields/actions32; owner
records256/scenes64/entities256; string1024/bytes4096/list64/depth4; canonical
record64KiB; frame actions128; callback commands4096; save8MiB. IDs are ASCII
bounded dotted namespaces/local keys; UTF-8 user labels belong in string fields.

Failed transaction commands poison the callback even when the plugin suppresses
their error. Failed initialize/tick/restore leaves Core records, objects, identity,
RNG, input, events and tick unchanged. The host cannot roll back external callback
effects. Aggregate save budget exhaustion rejects a commit atomically; per-owner
retention does not eliminate the shared bounded save budget.

Each `OwnerHistory` reports `emitted`, `dropped`, newest retained `history`, and
current commit `pending`. `emitted = dropped + retained.len()` and retained owner
ordinals are exactly `dropped + i`. Owner A cannot evict B's history. Global event
sequence is composition-dependent; isolation does not promise matching sequences
between different plugin compositions.

`ReadFrame` is a typed Rust selection. Its `BTreeMap<RecordKey, Record>` uses
structured keys; direct nonempty `serde_json::to_vec(ReadFrame)` is not a defined
wire API. Convert entries to your own declared array/envelope if needed. The
canonical portable persistence API is `CoreHost::save`, not ReadFrame JSON.

## Error codes for isolated public violations

These clarify implemented behavior; no Core behavior or signature changes.
For inputs with several violations, do not infer unspecified error precedence.

| Public operation / isolated violation | Error code |
|---|---|
| set/emit: missing/extra field, undeclared schema, wrong type/ref tag, declared string/bytes/list bound or i64 range | InvalidRecord |
| set/emit: schema-valid Record exceeds canonical64KiB | BudgetExceeded |
| callback: commands4096 or emitted events128 exceeded | BudgetExceeded |
| step: valid next tick but frame has129 actions, duplicate/unknown action, wrong type/range | InvalidInput |
| step: past/current target tick / future target tick | InputAlreadyCommitted / InputOutOfSequence |
| select: unknown owner / undeclared schema | StaleHandle / UndeclaredKey |
| typed correctly tagged reference targets a dead incarnation / unbound owner | InvalidReference / PermissionDenied |

For example, String max_bytes6 accepts `한글` (6 UTF-8 bytes), while `한글a`
(7 bytes) is InvalidRecord. A schema-valid larger payload whose whole canonical
Record exceeds64KiB is BudgetExceeded. These are distinct limits. Any rejected
transaction mutation still poisons its callback and rolls back the commit.

## Legacy public boundary

The separate `ge4g-pentomino-legacy` crate exposes `LegacyBridge`,
`LegacyProjection` and bounded bridge errors. `LegacyBridge::new(World)`,
`world()`, `projection()`, `input_frame(&Replay)`, `step_replay(&Replay)` and
`legacy_snapshot()` retain the existing runtime authority. `step_replay` validates
the resulting projection before replacing the World. `LegacyProjection::plugin`
or `install_into` imports an immutable snapshot and remaps provisional refs into
actual host identities. Core ticks do not advance the authoritative legacy World.
Unicode labels are metadata; hashed local IDs avoid namespace loss. Replay keyboard/
direction interpretation is entirely inside this compatibility crate.

The source-free Core SDK omits legacy runtime libraries/data; the legacy boundary
is documented and covered by repository regression tests. External source-free
legacy execution is `UNVERIFIED` in this package.

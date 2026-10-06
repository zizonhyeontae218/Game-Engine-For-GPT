# Basement 0.2 — FlatLand

Status: 0.2.0-rc.5 implemented candidate contract and evidence are in `FLATLAND_AUTHORING.md`
and `FLATLAND_RELEASE.md`. This original design retains proposed examples and optional
future extensions; it must not be mistaken for exact syntax or an unlimited capability claim.
Current clients are Android/Windows only. References below to iOS/Linux/Arch are
historical design targets, suspended until 0.3.0 development begins.

## Product outcome

AI-native means an agent can author, inspect, repair and prove gameplay using small,
local pieces of context. It does not mean an LLM runs during gameplay.
Target broad coverage of single-player top-view 2D adventures, action RPGs, puzzle
and stealth games, and turn-based RPGs through reusable systems plus Lua extensions.
“Most games” is a coverage objective, not a promise of arbitrary game fidelity.

Preserve Rust authority, deterministic headless execution, CPU reference rendering,
Flutter mobile import, embedded Windows/Arch distributions, digital brutalism,
and live per-game control profiles. Add gameplay rather than another launcher.
No full 3D engine, multiplayer, general rigid-body physics or authoring GUI in 0.2.

## 1. Map and Entity are distinct

A world scene references a map, entity placements and event definitions. Map data
contains static floor/wall/roof decoration, tile definitions, collision/navigation
cells, regions, exits and spawn points. Walls do not consume runtime Entity IDs.
Map regions may publish enter/exit events; behavior lives in the scene event table.

An Entity is a stable-ID actor or interactive object: player, NPC, enemy, crate,
chest, switch, projectile. It can own a body, sprite, interaction, stats, AI and
script hooks. A movable crate belongs here even if its art looks like a wall.
A door is an Entity or event-controlled map patch when it can open/close.

Map authoring supports ASCII tile rows with a legend, reusable chunks and optional
run-length rows. One named tile definition sets art, material, collision and plane;
no repeated collider JSON for each wall. PNG backgrounds may be paired with an
explicit collision grid. Art is never silently interpreted as collision.

Sparse map patches open doors/destroy walls and are saved separately from base maps.
A static spatial index queries nearby map cells and entity bodies, not all objects.
Scene loading validates references, placements and body penetration.

## 2. Body policy and facing

`body.mode` is one of:

| Mode | Player collision | Motion |
| --- | --- | --- |
| `pass` | No physical blocking | Static or controlled; overlap events remain available |
| `fixed` | Blocks like a wall | Cannot be pushed; explicit scripted relocation is validated |
| `push` | Blocks and may be pushed | Displacement only when the complete push succeeds |

The player/enemy controller drives its own body; `mode` specifies its response to
other movers. Optional collision groups/masks and `plane` filter physical contacts.
Interaction, damage and overlap sensors are separate from blocking body policy.

Resolve X then Y with swept AABB. For push chains, collect a deterministic dependency
graph, sweep every affected body against map solids/fixed bodies and commit together.
A blocked chain, cycle or configured maximum depth cancels that axis displacement;
never partially move a chain or push through a thin wall. Stable IDs break ties.
Mass/strength thresholds are optional presets, not a general physics simulation.

Each actor retains an eight-direction integer facing vector, initially `[0,1]`.
Successful movement updates facing; stopping preserves it. Explicit aim/facing
commands can update it even while blocked. A blocked movement alone preserves it.
Diagonal directions are integer pairs, not falsely described as unit vectors.

Proposed Lua API: `world:facing(id)` returns `dx,dy`; `world:face(id,dx,dy)` quantizes
to eight directions; `world:toward(a,b)` returns a direction; `world:in_front(a,b,range)`
uses integer dot/cross tests and a documented cone. Zero aim preserves facing.
Melee hitboxes, NPC interaction and directional animation consume this shared value.

## 3. Top-view 2.5D

Keep authoritative ground position `(x,y)`, a discrete collision `plane`, and
integer visual elevation `z`. Store sprite anchor separately from feet/body bounds.
Sort render packets by `(render_band, feet_y, z_bias, stable_id)` and render elevation
as a screen offset. Floor, actors and canopy/roof are explicit render bands.

This supports walking behind trees, tall NPCs, bridges and scripted jumps. Sorting
does not establish traversability: a bridge changes collision plane through authored
ramps/portals; visual `z` alone never bypasses a wall. Plane changes validate destination
clearance. Pathfinding uses the same plane/portal graph as collision.
Transparent roofs, follow camera and bounds are renderer/presentation policies.
Do not represent this as full 3D physics or arbitrary terrain height simulation.

## 4. Reusable gameplay components

- Stats: HP/max HP, team, speed, attack, defense and optional named integer stats.
- Combat: declarative attack presets, startup/active/recovery ticks, directional
  hitboxes, cooldown, damage rules, invulnerability, knockback, death and drops.
  Damage is an ordered command; hit events drive animation, quest and sound rules.
  Track each attack instance/target pair to avoid accidental damage each frame.
- Inventory: stackable item IDs, quantity, equipment slots, use effects and item gates.
- Enemy AI: idle, patrol, guard, chase, attack, return, flee presets; authored sight,
  range, team filtering and deterministic grid pathfinding. Bounded search work per
  tick and stable tie breaking. Lua handles unusual tactics without rewriting movement.
- Animation: atlas frames, named clips, integer frame ticks, loop/once, facing variants
  and frame-marker events. Gameplay hit windows belong to combat ticks, not GPU timing.
- Interaction: deterministic candidate selection by facing/range/priority/stable ID;
  conditional dialogue, choices, item use, switches and scene/event-scene transitions.
- Quests: inactive/active/completed/failed, named objectives and event-based progress;
  prerequisites, rewards and once-only transitions. Progress changes are inspectable.
- HUD: reusable dialogue/choice panels, HP bars, inventory, quest log and battle menus.
  Headless emits the same presentation model and accepts explicit choice commands.

Do not force Lua for ordinary enemies, dialogue branches, damage, pickups or quests.
Presets have discoverable defaults; an instance overrides only what differs.

## 5. Short authoring, without inventing another programming language

Use versioned JSON5 data and templates/prefabs as the primary compact format.
Keep field names meaningful; savings come from shared definitions and omitted defaults,
not cryptic single-letter keys. A prefab resolves once; multiple inheritance is excluded.
Maps, prefabs, dialogue, quests, events and scripts are separate addressable resources.

A representative proposed placement:

```json5
{ id: "box1", prefab: "crate", at: [96, 64], body: { mode: "push" } }
```

Common conditions/actions use a small validated operation vocabulary, rather than a
new general-purpose text DSL. Expressions are structured arrays. Every state reference
is declared and typed. No `eval` of strings, invisible globals or YAML implicit typing.
Proposed operations include `eq`, `gt`, `all`, `any`, `not`, `state`, `has_item`,
`quest_is`; actions include `set`, `add`, `give`, `take`, `say`, `damage`, `spawn`,
`despawn`, `animate`, `sound`, `quest`, `map_patch`, `goto`, `event_scene`.

```json5
{
  id: "open_gate", on: "interact", target: "gate",
  when: ["has_item", "player", "key", 1],
  once: true,
  do: [["take", "player", "key", 1],
       ["map_patch", "gate_open"], ["sound", "door_open"]]
}
```

Failed conditions have explicit optional fallback actions. Action batches validate
before commit, so a missing patch cannot consume a key. `once` is a persistent event-ID
latch. Counters use checked arithmetic; overflow is an error, never silent wrapping.
Dialogue branches have ordered predicates, named choices and a required fallback.
Named constants/aliases and editor-free reference checking make authoring errors local.

This format is an additional authoring route alongside Lua, not claimed universally
more token-efficient than Lua. Repetitive content benefits from prefabs/rules; complex
algorithms should use short Lua functions. Both compile/dispatch into the same commands.

## 6. Lua extension contract

Use embedded Lua 5.4 through a Rust binding (candidate `mlua`, vendored build). Pin
versions after Android/iOS/Windows/Linux build probes. LuaJIT is not required on mobile.
Package source only, not platform-specific bytecode or native modules.

Expose bounded world queries, integer math, declared persistent state, deterministic
seeded RNG, commands and event hooks. A minimal proposed hook:

```lua
return {
  interact = function(ctx, self, actor)
    local dx, dy = ctx.world:facing(actor)
    ctx.world:face(self, -dx, -dy)
    ctx:say(self, "Welcome.")
  end
}
```

Queries read a documented phase snapshot. Commands join an ordered transactional
buffer; scripts do not mutate Rust collections midway through iteration. Unknown IDs,
type errors or script faults abort the tick transaction, retaining the prior world
and reporting entity/hook/source location. Queue script faults outside gameplay state.

No filesystem/network/process APIs, native loading, host clock or unrestricted `debug`.
Provide a project-local module resolver. No gameplay reliance on unordered `pairs`;
ordered iteration APIs and stable entity query ordering are part of the contract.
Authoritative gameplay uses integers/fixed-point helpers; reject fractional/NaN/infinite
command values and exclude floating math helpers from the supported deterministic API.
Sandboxing alone cannot make arbitrary Lua algorithms cross-platform deterministic:
portable determinism applies to scripts obeying this API and verified by replay tests.

Limit instructions, memory, query results, commands and event recursion per tick/hook.
Use serialized event-machine program counters and typed locals for waits. Lua stacks,
closures, coroutines and arbitrary VM globals are not save data. Saveable game state
must go through declared state/components. Asset/script revision mismatches are explicit.

## 7. Event scenes for cutscenes and turn-based battle

Two scene kinds share one simulation: `world` and `event`. An event scene owns a
serializable sequence, actors/presentation references, program counter, typed locals
and explicit return route. A stack suspends the parent world with its exact state.
Nested event scenes have a bounded depth; transitions never reset the parent silently.

Instructions cover dialogue, choices, camera moves, actor movement/animation,
wait-by-ticks, conditions, calls, battle and return. Gameplay never waits on a video
frame, audio completion callback or wall clock. Unanswered choices enter a structured
waiting state; headless accepts the same choice IDs as Flutter.

Turn combat is a reusable event-scene controller with participant stats, speed/turn
order, action/item/target selection, cost, resolution, victory/defeat and rewards.
Stable IDs resolve equal speeds. A seeded RNG handles explicitly stochastic effects.
The parent world is frozen unless an event scene explicitly opts into background ticks.
Combat results commit once on return. Save/load resumes the same turn, selected phase,
pending choice and event stack. Animation/cutscene skipping skips presentation waits
while preserving gameplay commands exactly once; replay records skip/choice commands.

## 8. Sound is a real adapter

Runtime emits logical audio commands: play cue, stop voice, loop/music transition,
volume and optional positional data. Events carry tick, cue ID and playback instance ID.
Headless records/validates commands; device playback never influences simulation.
Use actual platform playback with a defined WAV/OGG decoding contract, bounded voice
pool, music streaming and focus handling. Evaluate a Flutter audio plugin and native
CLI backend with the same cues before selecting dependencies.

On pause/background, suspend audio; on resume/load reconcile active music and loops
from presentation state. Do not replay historical one-shots when loading a save or
requesting observation. Missing cues fail package validation. Device evidence must
include audible playback, not merely the presence of `audio` events.

## 9. Minimum input AND output context

Input: task-specific docs, small prefab catalogs, defaults, shared tiles, resource IDs,
partial schema lookup and atomic resource patches. Proposed `schema component body`
and `describe prefab guard` return one relevant contract/example, not the full schema.
Maintain a concise generated capability index stamped with engine/schema version.
Detailed docs are opt-in. Do not require agents to read the complete spec for each edit.

Output: default summaries contain tick, scene, waiting choice, changed IDs and failures.
Proposed `observe --since <revision> --fields ... --limit ...` returns filtered deltas
with a cursor, explicit truncation and reset-required when history is unavailable.
Do not imply a truncated trace is complete. Entity queries exclude map cells and
unchanged defaults unless requested. Full snapshots/traces/captures remain file artifacts.
CLI and FFI share semantics; observation polling does not advance simulation.

Patches specify expected revision, resource ID and changed fields; validation is atomic,
with localized file/field/reference diagnostics. Stable IDs avoid positional JSON patches.
Prototype/schema/effective-value expansion is available only on demand.

Measure authoring and debugging on the same game/tasks against verbose 0.1-style data:
source bytes, input tokens including docs/tool results, output tokens, calls, failures,
repair iterations and completion time. Pin tokenizer and count asset generation prompts
separately. No arbitrary token-saving percentage before measurements. Fewer tokens is
not success if the game needs more repairs or loses behavior.

## 10. Execution, saves and migration

Proposed tick phases: sample inputs -> event choices -> controllers/AI -> movement/push
-> overlap/hit collection -> combat -> event/quest/script command batches -> animation
and presentation -> publish state/events. Each phase has explicit read/write ownership.
Events generated while dispatching enter the next dispatch wave, with bounded waves;
recursive handlers cannot run forever. Stable ordering and integer seeded RNG are tested.
A scene transition ends the old scene's pending work; stale commands cannot target new
entities with reused local IDs. Entity references include world/scene instance identity.

Schema v2 covers map/entity/event/gameplay data and resumable saves. Save world scene,
actor positions/facing/HP, inventory, AI state, sparse map patches, quests, event latches,
RNG, timers, parent/event stack and presentation state. No audio handles or VM pointers.
Validate a complete temporary world before replacement. Saves bind to stable game ID,
content revision and explicit migration rules, rather than only a display title.

Keep v1 parsing/runtime behavior supported for existing games and goldens. An explicit
migration tool separates unambiguously static walls; colliders with interaction, tags,
metadata, player or behavior remain entities or produce a review diagnostic. Do not
infer authored intent from a sprite color. Existing v1 saves retain their flag-only
restart semantics unless explicitly migrated; never pretend missing positions are known.
Package capability requirements make old clients reject unsupported v2 games clearly.
Keep ABI v1 supported or negotiate ABI v2 explicitly; release held input on transitions,
choice mode, remapping and focus loss. Import packages remain data plus sandboxed Lua.

## 11. Release gates and coverage corpus

Ship a small FlatLand game proving: a map wall, pass/fixed/push NPC/object policies,
a blocked push chain, facing melee, a chasing enemy, HP/death, a key-gated door,
a branching dialogue/quest, Y-sorted canopy/bridge, sprite animation, audible music/SFX,
a cutscene, and a separate turn battle returning to the same world.

Behavioral tests prove exact positions/HP/quest rewards, once-only events, stable AI
paths, collision-plane separation, transaction rollback, Lua budget failures and RNG
repeatability. Save during a dialogue choice/turn and compare resumed execution.
Compare headless and Flutter states/CPU frames; keep v1 demo goldens unchanged.
Build/package and inspect Android/iOS/Windows/Linux artifacts. Physical-device/audio
checks are distinct from CI compilation; report actual evidence without inference.

A broader corpus covers action adventure, item-gated puzzle, stealth patrol and turn
RPG with declared supported rules and extensions. Completion means those behaviors
work from authored content, not simply that component names appear in schemas.

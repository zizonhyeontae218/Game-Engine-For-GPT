# FlatLand 0.2.0 — implemented authoring contract

Game schemas 1 and 2, native ABI 1. Start with `FLATLAND_QUICKSTART.md`, then query
`ge4g capabilities --json` and only the component schema needed. The historical design
in FLATLAND_SPEC is broader; this document defines the shipped syntax and limits.

## Maps, prefabs and resources

Scene schema 2 has `map: {cell, rows, tiles}`: equal ASCII rows, at most 128×128.
Tile fields are RGBA `color`, `solid`, optional spawn prefab and `planes`. Empty planes
means a solid tile blocks all planes; a list limits solidity to those planes.
`map.portals` entries contain id, at tile coordinates, from/to plane and destination.
Portals validate destination clearance, use an arrival latch and participate in AI's
bounded plane graph. Sparse `map_patch` commands are saved separately from base rows.

Entity placements inherit one prefab. Nested objects merge, arrays replace. Prefab
inheritance is rejected. Walls stay map cells, never runtime entity IDs.
`resources` includes up to 32 project-local JSON5 resources containing gameplay,
prefabs, sounds and rules; resources merge in order and the scene overrides them.
Referenced resources, Lua, textures and sounds enter the content revision for saves.

```json5
{ id:"crate1", prefab:"crate", position:[96,64], flatland:{body:"push"} }
```

`flatland.body` is pass/fixed/push; map walls still block pass actors. Push chains commit
as a whole or remain unchanged, max depth 32. Actor `plane` controls body/attack contacts;
`z` only offsets art. Body size stays separate from `visual_size` and pixel `anchor`.
`depth:true` sorts within a layer by ground feet Y then stable ID; `sprite.layer`
establishes floor/actor/canopy bands. Visual elevation is not height physics.
A follow camera is `gameplay.camera:{target:"player",bounds:[minX,minY,maxX,maxY]}`.

Grid actors use cell-sized aligned bodies, queue cardinal turns at intersections.
AI modes: idle, patrol, guard, chase, attack, return, flee. Declare speed, target/corner,
sight, patrol tile points and an optional attack preset. Guard/attack use map line of
sight; deterministic BFS uses stable up/left/down/right ties, at most 4096 nodes.
Dynamic blocking bodies stop movement; navigation does not solve moving-body congestion.

Facing is an eight-direction integer vector. `World::facing/face/toward/in_front` share
it across movement, melee and interactions. Blocked movement preserves facing;
`face` explicitly aims. `in_front` is a 90° cone with Chebyshev pixel distance.

## Combat, items and quests

`gameplay.attacks` maps IDs to damage/range/startup/active/recovery/immunity/knockback,
optional projectile_speed/lifetime and on_hit action arrays. Times are integer ticks,
speed authored pixels/second (one speed unit equals a subpixel per 60 Hz tick).
Each owner has recovery cooldown; an attack/target pair hits at most once. Projectiles
have a visible 4px CPU render packet and swept collision/LOS, filtered by plane/team.
Actors declare hp, team, defense, optional attack preset and death drops. `health_bar`
adds a reusable HP bar. Non-player deaths remove the actor and execute drops once.
Player death keeps the player for authored respawn/game-over rules. Damage respects
immunity; melee/projectile damage includes equipment attack/defense bonuses.

`gameplay.items` entries have name, stack limit, optional use_actions and equipment slot,
attack_bonus/defense_bonus. `give/take/use/equip` validate quantities/ownership. Removal
of the last equipped item clears its slot. Inventory and equipment persist across scenes.
Turn fighters declare their own combat stats; equipment bonuses apply to world combat.

`gameplay.quests` entries have name, prerequisite condition, objective count targets,
optional objective_labels and rewards. States are inactive/active/completed/failed.
Objectives cap at targets; completion requires all targets and rewards exactly once.
Flutter exposes item use/equipment and a quest log; headless observes identical state.

## Typed commands and conditions

Rules contain id, on, optional target_tag, when, once and actions. Events: start/tick,
contact/pickup/interact, hit/death, battle_win/battle_loss and action.NAME for named input.
Once latches are scene-qualified and saved. Actor references support $player/$target.

Conditions: always, state equality, timer, vulnerable, remaining tag count, hp threshold,
has_item, quest_is, objectives_complete, all, any, not. Query `schema condition` for fields.
Actions: set/add/timer/say/remove/damage/respawn/face/goto/stop/sound; heal, give/take/use/
equip, quest/objective, attack, event_scene, map_patch, plane/elevate/move/spawn,
animate/play_clip, roll and music. Spawn restores a removed authored entity definition.

```json5
{ id:"finish", on:"tick", once:true,
  when:{op:"objectives_complete",quest:"signal"},
  actions:[{op:"quest",quest:"signal",status:"completed"}] }
```

Checked integer arithmetic and invalid references abort the transaction. Batches max64,
recursive dispatch max16 and 4096 commands per tick. Scene transitions end old-scene
work. Normal ticks and explicit command/choice/skip/replay operations restore the prior
world on failure. State definitions and references validate before play.

## Animation and sound

Animation supports PNG frames, ticks per frame, once/loop, directional clips/rotation,
timer alternates and atlas:{file,cell:[w,h],indices:[...]}. Atlas bounds validate at import.
Frame markers map frame indices to action arrays. Base animation phase uses world ticks
excluding event pauses; named `gameplay.clips` have frames/ticks/once and `play_clip`
starts at its invocation tick. In event scenes, new temporary clips advance while parent
animations pause. Integer phases and logical audio state resume from saves.

Scene sounds map cue IDs to project-local WAV or OGG files. `sound` emits one-shots;
`music:{cue:"music",volume:25}` starts/replaces a loop, cue:null stops it. Four one-shot
voices and one looping music stream are implemented in Flutter (audioplayers) and
native CLI (rodio/cpal). Pause/focus suspends playback; resume reconciles music. Save
load does not replay historical one-shots. Decoding/device failures never change gameplay.
Named independent ambient-loop/positional voice controllers remain outside this release.

## Serializable event scenes and turn battle

`gameplay.events` maps an ID to instructions: do, wait, say, choice, branch, jump,
camera, call, battle and return. All waits are simulation ticks; max128 immediate
instructions, nested depth8. Choice options declare id/text/when/next/actions.
A waiting model exposes stable choice IDs to Flutter and headless/FFI; choice/skip
commands are recorded in replay schema2. CLI window uses 1..9 or Enter for a sole option.

```json5
{ op:"battle", fighters:[
  {id:"hero",name:"Hero",hp:20,attack:6,speed:8},
  {id:"bot",name:"Bot",hp:10,attack:3,speed:5,enemy:true}],
  victory:[{op:"give",item:"coin",count:2}], defeat:[] }
```

Battle menus support target attacks, guard, healing items; speed/ID fixes actor order,
seeded integer RNG modifies damage. Results fire victory/defeat rules exactly once.
The parent world controllers, attack timers and animations freeze. Actor/camera/plane/
elevation/animation presentation changes are temporary; durable items/quests/state persist.
Return restores the exact parent actor state. Save/load resumes choice, wait, stack and
turn; skip skips the current wait while preserving subsequent commands. No Lua VM stack
or audio device handle enters a save.

## Lua extension contract

Lua 5.4 is embedded/vendored through mlua. A script returns a function(ctx) → action
array. Context contains event/target/tick/state. Queries: facing(id), toward(a,b),
entity(id), entities(tag,limit) in stable ID order, item_count(id), random(min,max).
Convenience say(text), give(item,count), damage(id,amount), face(id,dx,dy) queue validated
commands before the returned action array. Commands read a phase snapshot, then commit.
Declared gameplay.modules map module IDs to local source; require resolves only those.
Globals are recreated per invocation; use typed declared state for persistence.

No IO/network/process/clock/debug/native loading, protected catches, pairs/next or math
API. RNG and commands require integers, reject fractional values. Scalar tostring only;
unbounded pattern helpers/dump are absent. Limits: 2MiB VM, 20k instructions/hook and
100k/tick, 64 returned/queued commands, query1..256 with explicit more. A fault aborts
with source/hook context. Determinism covers scripts obeying this API and replay tests.

## Context, saves and packages

`resource PROJECT FILE STABLE_PATH` returns a fragment plus SHA256 revision.
`patch ... --expected REVISION --patch JSON_FILE` validates a staged full project,
rechecks revision and atomically replaces bytes; failure leaves original bytes intact.
Use one schema/prefab/entity query rather than reading the entire project.
FFI observe entity/compact/after exposes cursor/more/reset_required; polling never ticks.

Schema2 saves bind to project content revision and carry actors, inventory/equipment,
quests, RNG, map patches, AI, timers, animations, event parents/PC/battle and music.
Candidate saves validate before replacement; edits to content invalidate old resumes.
Schema1 and its flag-only restart semantics remain supported with original goldens.
Required manifest features produce bundle schema3; older clients reject incompatible
packages. ABI remains1. Android ID/certificate stay fixed and versionCode increases.

`flatland_task_benchmark.py` measures a controlled author/repair task with real cl100k_base
tokens, tool calls/failures/time and equal resulting frames. It does not measure model
sampling time or claim universal savings. Pac-Man and Signal Yard are executable examples.

## rc3 villages and battle presentation

Use actor `step_walk:true` plus a16px map cell and speed48 for three tiles/s. Release
finishes the current cell then stops; maze `grid:true` retains its original behavior.
Touch profile joystick may set `cardinal:true`; tune dead_zone separately. Pace overrides
use `{op:"pace",entity:"$player",speed:72}` in native commands/Lua.

A battle instruction can set `stage:{background:"assets/field.png",hold_result:true}`.
Each fighter may supply `sprite`, `back_sprite` and up to four moves with stable ID,
name, power (percentage1..300) and PP. Native choices use `move:<move>:<target>`;
legacy fighters without moves still use `attack_<target>`. The dedicated client offers
moves, guard and inventory; `battle_continue` executes victory/defeat effects once.
Every attack resolves the round in native simulation; the displayed24-tick hit feedback
uses turn_tick/previous_hp while the parent world stays frozen. Save captures remaining PP.

Gameplay views are named presets: label, zoom100..160, tilt60..100, shear-25..25.
`view:{mode:"depth"}` projects the padded canonical world. Collision `plane` and visual
`elevate` remain independent. Default event return restores them; `{op:"return",retain_view:true}`
retains selected view/planes/elevation for a settings choice. There is no3D geometry.
Optional gameplay.lua_events filters hook names before creating a Lua VM; empty means
legacy all-hooks behavior. Packages using these additions declare step_walk, battle_stage
and view_projection capabilities so old clients reject them clearly.

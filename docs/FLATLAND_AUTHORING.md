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

## Movement, view and reusable presentation

Use actor `step_walk:true`, a16px map cell and speed68; pace112 is visibly faster.
Newest pressed direction wins; release finishes the current cell without an extra cell.
Cardinal touch profiles provide hysteresis and an optional native direction intent.
Legacy `grid:true` maze movement remains unchanged.

Views have label, zoom100..160, tilt60..100 and shear-25..25. `{op:"view",mode:"depth"}`
is persistent presentation-only; `view_reset` explicitly returns to unprojected default.
Scenes inherit the selected resolved view, even without a matching local preset. Save
stores its parameters. Event-scene `camera` is scoped and restored on return. Legacy
`retain_view` still parses; it no longer selects collision/height persistence.
Use independent `plane`, `portal`, `elevate` gameplay operations for collision/elevation.
`sprite.projection:"ground"` is appropriate for terrain/decal art; actors/props default to
upright projected feet. Never create a ground actor merely to configure normal sprites.

Actor `building:{footprint:[48,32],height:26,roof:"gable",material:"wood"}` receives
front/side/roof/trim/windows/door/contact-shadow defaults. Roofs: gable,flat,shed;
materials: wood,plaster,brick,stone,metal. Optional roof_surface/facade_surface/side_surface
PNG paths replace exactly that component. Missing surfaces fall back; custom transparent
pixels are preserved. Entity defaults provide feet anchors, depth sorting, contact
shadows, health feedback and a visible sprite-free projectile.

`gameplay.combatants:{seedling:{name:"연두",max_hp:38,attack:12,defense:4,speed:8,
moves:[{id:"scratch",name:"할퀴기",power:100,pp:25,fx:"slash"}]}}` defines a persistent
roster entry. Battle fighters reference `{id:"hero",combatant:"seedling"}`. HP/PP initialize
once, commit after authoritative resolution and persist across encounters/save. Zero stays
fainted; explicit `combatant_heal` (amount) or `combatant_reset` restores it. Heal/reset
roster commands are rejected during an active battle. Inline legacy fighters stay ephemeral.
Move power retains legacy percentage1..300 semantics, not Pokemon base-power semantics.

Built-in FX: strike(default),slash,projectile,burst,heal,guard. Typed BattleFxEvent includes
actor,target,preset,start_tick,before_hp,after_hp. Each result has a54-tick cosmetic sequence;
round results/PP/items are determined once before presentation. HP displays old→new at
impact. Unskipped FX lock choices. Skip clears cosmetic feedback without repeating math.
Saving retains exactly-once authoritative results and the deterministic presentation cursor.

Declare new features billboard_projection,persistent_combatants,battle_fx,
building_presentation,entity_defaults as used; old clients reject unsupported bundles.
Legacy packages without battle_fx keep their immediate-choice replay timing. ABI remains1.
Public source examples must include asset-licenses.json with exact provenance/license/hash.
See examples/flatland_harbor for deliberately incomplete but valid presentation input.


## Contact geometry and story bubbles

A building's footprint is its body size (overrides redundant entity.size). Omitted
body defaults fixed. Ordinary semantic buildings do not provide roof platforms.
Explicit body:pass authors a non-solid prop only; it does not create a walkable roof.
0.2 facing supports only south (default); north/east/west reject validation.
Roof forms gable/flat/shed and wood/plaster/brick/stone/metal materials remain supported.

With entity_defaults, upright sorting ignores sprite layer priority and uses plane,
gameplay elevation, body-foot Y and stable ID. Contact shadows live below every
upright entity. sprite.projection:ground remains the terrain/decal band. Preserve
legacy layer behavior when entity_defaults is absent.

Event scene: {op:"say_bubble",actor:"guide",text:"여기가 공방이야."}.
The actor must exist in the scene; tap/continue advances one instruction. Chain lines
as independent instructions; save retains the exact line/PC. Bubble UI is distinct
from ordinary say/choice. Camera ops are temporary, eased over12 deterministic
presentation ticks when cutscene_bubbles is declared, restored on return. Bubble
waiting freezes gameplay and permits only progression/save; animation cannot advance
story PC or apply gameplay effects. Nested camera overrides restore their parent.
Declare cutscene_bubbles,solid_buildings,contact_ordering for authored content so
older clients reject unsupported content clearly. Existing ABI1 remains unchanged.

## Cutscene speaker metadata

`{op:"say_bubble",actor:"guide",speaker:"안내인",text:"여기가 공방이야."}`
uses optional `speaker` (at most256 UTF-8 bytes, no NUL). Omitted speaker resolves
actor metadata `display_name`, then `name`, when a nonempty stable string. Without
a meaningful label the client omits the header; it never substitutes an entity ID
or generic story label. Speaker has no effect on gameplay, line progression or
RNG. Native waiting data carries the resolved label. Old speaker-free rc5 content
remains valid. Save/resume retains the authored current actor/text/line/name.

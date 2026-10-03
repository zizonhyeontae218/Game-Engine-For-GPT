# FlatLand alpha: author only what differs

Implemented in **0.2.0-alpha.1**, game schema 2. See `FLATLAND_SPEC.md` for the larger
0.2 design; it is not a list of completed features. Existing schema 1 games still run.

## Smallest useful context

Start with this file. Query `ge4g schema body|actor|rule|map|resume --json` only as needed.
Inspect one prefab with `ge4g inspect <project> prefab <id> --json`.
Inspect one entity with `ge4g inspect <project> entity <id> --json`; write complete
snapshots/traces to files when diagnosing a replay. The native `observe` operation
supports `entity`, or `compact: true, after: <event_cursor>`; cursors belong to a session.
Results report `cursor`, `more` and `reset_required`. Lost history is never silently complete.

## Map and actors

A schema 2 scene has a `map` with `cell`, equal ASCII `rows` (up to 128x128), and a
`tiles` legend. Tiles have RGBA `color`, `solid` and optional `spawn` prefab ID.
Walls are queried from nearby map cells and rendered without becoming runtime entities.
All map walls block every collision plane in this alpha; bridges/plane portals are later.

`prefabs` are shared entity objects. Instances use `prefab` plus overrides; object
fields merge recursively and arrays replace. Inheritance between prefabs is rejected.
A tile with `spawn` places one instance at the cell center with ID `tile_<x>_<y>`.

```json5
{ id: "box", prefab: "crate", position: [40,20], flatland: {body:"push"} }
```

Entity `flatland` fields: `body` (`pass`, `fixed`, `push`), `grid`, `plane`, optional
`hp`, `ai`, `pickup`, `animation`, `depth`. A body's policy describes whether other
movers can pass/push it. A controller may move its own body. Blocking push chains
commit together or remain unchanged; depth is limited to 32. Map walls remain solid
for pass actors. Entity collision filtering uses `plane`; map collision is shared.

Grid actors align to cells and have cell-sized bodies. Player input queues a cardinal
turn; movement continues in the previous direction until an available intersection.
AI has speed, target entity or corner, and optional flee timer. Bounded grid BFS uses
stable up/left/down/right tie order. Player/AI speed is at most 6000 pixels/second.

`World::facing`, `face`, `toward`, `in_front` share eight-direction integer vectors.
Stopping retains facing; blocked motion alone does not change it. Diagonals are not
unit vectors. `in_front` uses a 90-degree cone and Chebyshev distance in authored pixels.

Animation uses PNG `frames`, frame duration `ticks`, optional directional `rotate`,
and timer-selected `alternate` frames. CPU rendering remains canonical. `depth:true`
sorts sprites in a layer by ground feet Y, then ID. Sprite `layer` establishes bands.
There is no atlas crop, moving camera, elevation physics or frame-marker API yet.

## Rules and Lua

Rules have `id`, `on` (`start`, `tick`, `contact`, `pickup`, `interact`), optional
`target_tag`, `when`, `once` and `actions`. Once latches are scene-qualified and saved.
Conditions: `always`, `state` equality, active `timer`, `vulnerable`, tagged `remaining`
count, `hp` threshold, `all`, `not`. `$target` is available to vulnerability conditions.
Actions: `set`, checked integer `add`, `timer`, `say`, `remove`, `damage`, `respawn`,
`face`, `goto`, `stop`, `sound`. Actor references accept `$player` and `$target`.
Respawn can grant immunity ticks. Damage uses HP and immunity; this alpha does not
include melee/projectile attack controllers, defense stats or knockback.

```json5
{ id:"finish", on:"tick", once:true,
  when:{op:"remaining",tag:"food",count:0},
  actions:[{op:"set",key:"game.result",value:"won"},
           {op:"say",text:"Cleared!"},{op:"stop"}] }
```

A scene's `script` is a project-local Lua 5.4 source path. It returns a function taking
`ctx.event`, `ctx.target`, `ctx.tick`, `ctx.state` and returning an action array.
Global `facing(id)` returns dx,dy. Script globals are recreated per invocation and are
not persistence. Use declared state/actions for durable behavior.

```lua
return function(ctx)
  if ctx.event == "pickup" then
    return {{op="add", key="game.pickups", value=1}}
  end
  return {}
end
```

No IO, OS, package loader, native modules, print, dynamic load, unordered iteration,
clock or math/RNG API. Protected error catches, GC control, bytecode dump and
unbounded string-pattern helpers are excluded; `tostring` accepts scalar values only. Supported commands use integers; no hidden float coercion.
Limits: 2 MiB VM, 20,000 instructions per invocation, 100,000 per tick, 64 actions per
batch, 4096 actions per tick. Any rule/Lua fault rolls back that entire tick's state,
positions and events. Errors expose hook/source details. RNG, module imports, richer
query helpers and saveable event machines are not implemented yet.

`interaction.dialogue` supplies a fixed message; conditional `interact` rules can
select different `say` messages. Named choice/branch dialogue UI is a later slice.
Quest completion can be modeled with state and once rules; there is no dedicated
quest objective/inventory controller yet.

## Resume, presentation and packages

Schema 2 saves resume scene/tick, remaining entities/positions/facing/HP/immunity,
state, timers, once latches and the current text. They bind to a content digest and
reject changed content or invalid positions/types. V1 saves keep flag-only semantics.
Input holds and event history reset on resume; cursor sequences are session-local.

`say` emits an informational popup. Flutter suspends its ticking while the popup is
visible; closing it resumes presentation. It is not an authoritative choice/wait event
scene. Native replay can run past informational messages, as documented by the demo.

Scenes declare `sounds` mapping cues to packaged files. `sound` emits tick-tagged
commands; Flutter uses a bounded four-voice WAV-capable audio adapter. Loading a save
or polling status does not replay historical one-shots. Headless/minifb log cues.
Looping music, positional audio and audible physical-device acceptance are still pending.
Linux playback requires GStreamer base/good plugins; platform completion does not drive ticks.

Portable packages use bundle schema 2, `game_schema:2`, and native ABI 1; old clients
reject the new package version. Desktop bundles embed the game/client; mobile imports.
The rotation button selects portrait or landscape; resizing/sensor input never changes
the selected layout. Mobile requests exactly portrait-up or landscape-left. Text popups
are separate, scrollable surfaces, with a menu action to reopen the latest message.

Demo: `examples/flatland_pacman`. `ge4g test` proves score, repeatability, save/load and
canonical PNG; engine tests prove body policies, Lua rollback, AI, power/HP/win/loss,
reachable food, queued turns and corrupted-resume rejection.

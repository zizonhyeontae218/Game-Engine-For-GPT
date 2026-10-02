# Authoring a GE4G game

Copy `examples/basement_demo` as a starting project. Edit `ge4g.toml`, scene JSON5, and replay JSON; then run `ge4g validate <project>` and `ge4g test <project>`. Use `ge4g schema <kind> --json` for field structure. All formats use `schema_version = 1` / `schema_version: 1`; unknown fields are errors.

## Project

The TOML manifest declares `name`, `start_scene`, `window.width/height`, a `[scenes]` map from stable id to project-relative file, typed `[state."namespace.key"]` entries, and optional `[[tests]]` entries. Scene, texture and manifest-test replay paths must resolve inside the project.

```toml
[state."quest.greeted"]
kind = "bool" # bool, integer (signed i64), or string
default = false
persistent = true
```

Nonpersistent state uses `persistent = false` and resets to its default on restart. State keys contain at least two nonempty dot-separated segments using ASCII letters, digits, `_` or `-`. Every scene write must refer to a declared key with a matching value type.

## Scenes and components

A JSON5 scene declares its matching `id`, RGBA `background`, optional pixel `camera`, named `spawns`, optional `on_enter` state writes and `entities`. Entity ids use ASCII letters, digits, `_`, `-` or `.`, and are unique within a scene.

Each entity needs `id`, pixel `position: [x,y]` (top left) and positive `size: [width,height]`. Optional components:

| Field | Shape / behavior |
| --- | --- |
| `sprite` | `{color:[r,g,b,a], texture:'assets/tile.png', layer:0}`; fields optional, color defaults to white; without texture it draws a solid rectangle |
| `collider` | `{blocking:true}`; false remains inspectable but does not block movement |
| `player` | `{speed:120}` pixels/second; exactly one per scene, with a blocking collider |
| `trigger` | `{scene:'room_b', spawn:'entry'}`; transitions on entering/crossing its AABB; cannot also block |
| `interaction` | `{range:12, dialogue:'Hello', set_state:{'quest.greeted':true}, transition:{scene:'room_b',spawn:'entry'}}`; state writes and transition optional |
| `tags`, `metadata` | String array and metadata object exposed in inspect output |

Triggers and interactions have their own AABBs without requiring a collider. At most one transition per tick occurs. Scene/spawn references are checked before play, and player spawns cannot overlap blocking entities.

`on_enter: {'quest.room_b': true}` runs on every entry. Normal initial startup uses the player's authored position. A transition uses the target scene's named spawn. Saves restore state before the starting scene's `on_enter`; saves do not resume a scene or tick.

## Replay and assertions

```json
{"schema_version":1,"ticks":60,"inputs":[
  {"start":0,"end":30,"actions":["right"]},
  {"start":30,"end":31,"actions":["interact"]}
]}
```

Actions are `left`, `right`, `up`, `down`, `interact`. Spans are start-inclusive/end-exclusive; overlaps union actions. Index 0 produces snapshot tick 1. Inputs beyond replay duration are neutral. Opposing actions cancel; diagonal motion is full speed on both axes. Interaction occurs on a rising edge, so insert a neutral tick before pressing again.

Project tests declare a replay and nonempty checkpoint assertions:

```toml
[[tests]]
name = "greeting"
replay = "replays/greeting.json"
assertions = [
  {tick=30, entity={id="player", position=[84,40], max_x=84}},
  {tick=31, state={"quest.greeted"=true}, event={kind="interaction",entity="player",target="npc"}},
]
```

Assertions can check scene, exact entity position in pixels, entity max_x, a subset of state, and a previously emitted event matching kind plus optional entity/target. Event assertions check history retained through that checkpoint. A snapshot's raw positions use subpixels; assertion positions use pixels.

An optional `golden_rgba_sha256` pins the final framebuffer bytes, independent of PNG metadata. Record it after inspecting a deliberate reference frame using `capture --tick <duration> --replay ... --json`. Changing a golden should represent an intentional visual change, never hide a regression.

Every `test` run also compares a second replay's complete final snapshot/events, round-trips persistent state through an atomic save, and writes a real PNG. Replays whose history exceeds the event buffer are rejected by the project test runner.

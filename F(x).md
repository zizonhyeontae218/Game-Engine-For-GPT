# F(x) — implemented stable state / identifier registry

CLI/control/ABI transport stays v1. Game/scene/replay/snapshot/save data supports explicit v1 and FlatLand v2. The first table preserves the v1 baseline; FlatLand additions follow below. This registry records identifiers that are actually implemented and cross subsystem/serialization boundaries. Entity positions use 60 subpixels per pixel; scene authoring and test assertion coordinates use pixels.

| ID / public path | Kind | Default | Owner | Scope | Persistence | Writers | Readers | Reset | Range | Migration |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `schema_version` | u32 | 1 | core / serializers | project, scene, replay, snapshot, trace, save, CLI response | Artifact | format writers | parsers, agents, tests | Never | Exactly 1 | Other versions rejected; future changes require version/migration |
| `engine_version` | string | 0.1.0 | Cargo workspace / core | snapshot, trace, inspection | Artifact | snapshot/trace writers | CLI, tests, agents | Build | Package version | Semver; independent of data schema |
| `tick` / `events[].tick` | u64 | 0 | World | snapshot, events, assertions | Snapshot/trace only | step | all adapters, agents | Runtime restart | 0..1,000,000 | v1: input index N produces tick N+1 |
| `tick_hz` | u32 | 60 | core | snapshot / simulation | Snapshot | snapshot writer | platform, agents | Never | 60 | v1 fixed rate |
| `subpixels_per_pixel` | i64 | 60 | core | snapshot / entity transform | Snapshot | snapshot writer | renderer, agents | Never | 60 | v1 integer coordinate contract |
| `scene` / scene `id` | string | manifest start_scene | World / project | scene, snapshot, events | Snapshot/trace only | scene load / transition | renderer, inspect, tests | Runtime restart | Valid declared scene id | v1 matching manifest key |
| `entities[].id` | string | authored | project / World | scenes, snapshot, events, assertions | Snapshot/trace only | authoring / scene load | systems, agents | Scene load | Unique per scene; `(scene,id)` identity | v1 stable slug |
| `entities[].position` | `{x:i64,y:i64}` | authored pixel position × 60 | World | snapshot / rendering | Snapshot only | movement / spawn | collisions, renderer, inspect | Scene load | Integer subpixels | v1 sizes/camera remain pixels |
| `events[].kind` | enum-like string | emitted event | World | snapshot / trace | Trace | systems, save/load | CLI filters, tests, agents | Bounded history | Enumerated in CLI contract | v1 names and payloads |
| `events_dropped` | u64 | 0 | World | snapshot / trace | Snapshot/trace | bounded event queue | agents, project tests | Runtime restart | ≥0; latest 4096 retained | v1 explicit loss count |
| `demo.npc.spoken` | bool | false | StateStore / demo | runtime / save / inspect | Save | NPC interaction, save load | title, inspect, tests | Fresh runtime without save | false/true | v1 declared bool; no coercion |
| `demo.room_b.entered` | bool | false | StateStore / demo | runtime / save / inspect | Save | room_b on_enter, save load | title, inspect, tests | Fresh runtime without save | false/true | v1 declared bool; no coercion |
| save `project` | string | manifest name | project / save writer | save | Save | save writer | save loader | New project | Must match current name | v1 project identity validation |

Demo stable scene ids: `room_a`, `room_b`; named transition spawn: `entry`. Referenced entity ids: `player`, `wall`, `npc`, `door`, `return_door`. Transitions reload scene-local entities and preserve StateStore.

## Client ABI and package/control identifiers (v1)

| ID | Owner / scope | Persistence / reset | Contract |
| --- | --- | --- | --- |
| `abi_version`, bundle `engine_abi` | Rust native ABI / Flutter / `.ge4g` | Build/artifact | Exactly 1; incompatible versions fail |
| `session` | Rust ABI registry | Runtime only; close removes, monotonic ids | Opaque nonzero u64; at most eight open sessions |
| `game_id` | Game author / package / bindings/library | Stable across package updates | Lowercase slug max96; isolates controls and save paths |
| package `digest` | Library | Content-addressed install | SHA256 of complete `.ge4g`; immutable installed content |
| `active_profile`, profile `id` | Layout JSON / input router | Per-game settings | Matching mapping profile IDs; valid switching releases held input |
| touch button `id` | Layout and binding JSON | Per-game settings | Unique per profile; IDs must match in both files |
| `actions` / `action_pressed`, `action_released` | Router / World / event trace | Held during session; release/close clears | Max32 ASCII named actions ≤64 chars; sorted edges |
| `mode=embedded`, `allow_library=false` | Desktop `client_mode.json` | Shipped bundle | Game starts from relative included `.ge4g`; no runtime import prompt |

## FlatLand alpha schema 2

- `map.cell/rows/tiles`: static world collision/render/navigation; walls have no Entity IDs.
- `flatland.body`: pass/fixed/push; `plane` filters actor-body collision, not map walls.
- Actor `facing`, `direction`, `queued`, `hp`, `immune_until`: authoritative, v2 saved.
- World `timers`, `completed` (scene:rule), `stopped`, `popup`, `popup_serial`: v2 saved.
- Resume `content`: SHA256 of checked project inputs; mismatches reject instead of guessing a migration.
- Bundle `schema_version=2`, `game_schema=2`, `engine_abi=1`: explicit capability boundary.
- Native `event_cursor` / observe `after,cursor,more,reset_required`: session-local event sequence, including dropped entries.
- Native `popup` and tick-tagged `audio` carry presentation; historical audio is not replayed on resume/poll.
- Demo state: `game.score` integer0, `game.pickups` integer0, `game.result` string playing; all v2 saved.

Workspace engine version is now 0.2.0-alpha.1; existing v1 framebuffer goldens remain unchanged.

FlatLand rc.1: `flatland.animation.directions` selects fixed-tick up/down/left/right clips from actor facing. Layout schema 2 permits a joystick-only profile; game-local control files are packaged presets. Android identity is fixed by application ID and pinned release certificate, with increasing version code.

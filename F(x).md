# F(x) — implemented stable state / identifier registry

Public formats currently use schema version 1. This registry records identifiers that are actually implemented and cross subsystem/serialization boundaries. Entity positions use 60 subpixels per pixel; scene authoring and test assertion coordinates use pixels.

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

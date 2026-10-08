# F(x) — implemented stable state / identifier registry

## Experimental Pentomino scalar host (isolated Rust API)

`ge4g-pentomino` is an additive first lifecycle slice, not the shipped runtime or
a new native ABI. Its save format1/contract1.0.0 is separate from existing
schema1/schema2 and ABI1. No legacy identifier changes.

| Identifier | Owner / persistence | Implemented contract |
|---|---|---|
| PluginId / CapabilityId | linked plugin descriptor / canonical save-discovery | Distinct validated dotted ASCII IDs <=128 bytes; exact capability version/provider binding |
| OwnerToken | Host / runtime only | Private host nonce + monotonic generation; different-host/removed/reinstalled/pre-restore tokens reject; excluded from bytes/hash |
| resources local keys | owning plugin / authoritative canonical save | Declared one-segment ASCII <=128 bytes; signed i64 values; <=64 declared records; own writer only |
| Event owner/kind/value | owning plugin / pending and retained canonical history | Stable PluginId + declared local kind + i64; reads only own or explicitly bound provider |
| tick | Host / canonical save | Starts0; step accepts only committed+1; checked overflow; failed ticks retry with no committed effects |
| rng_state | plugin / canonical save | SHA256(seed LE || PluginId) prefix initializes per-owner SplitMix64; writes/draws roll back with callback/tick |
| format_version / contract_version | Host serializer / artifact | Exactly1 /1.0.0; compact canonical JSON, strict unknown/duplicate/noncanonical rejection; separate from legacy schemas |
| content_binding / seed | Host::new / canonical save | Immutable supplied binding <=128 validated bytes, exact saved binding/seed required |
| next_sequence / events_dropped | Host / canonical save-history | Sequence never renumbered; history<=256, pending<=128; checked next_sequence=history.len+dropped; eviction/purge explicit |
| first_retained_sequence | observation / derived runtime | Whole-host history first sequence or next_sequence if empty; owner-filtered history is affected by global retention |

Install/tick/restore use atomic staging. Restore requires the exact installed
descriptor/binding set and complete current-tick pending history, and revokes old
tokens after full validation. Numeric limits and public Rust surface:
`docs/pentomino/P1_CONTRACT.md`. This compatibility namespace is unchanged.

## Experimental Pentomino P2 typed Core (contract2 / save2)

Additive `ge4g_pentomino::p2`; no schema1/2 or ABI1 identifier migration.
P2 is an internal completion candidate; external GPT Work acceptance is UNVERIFIED.
Exact API/bounds: `docs/pentomino/P2_CONTRACT.md`.

| Identifier | Owner / persistence | Implemented contract |
|---|---|---|
| SchemaId / ActionId | descriptor / save2-discovery | Distinct dotted ASCII IDs <=128 bytes; globally unique declarations |
| RecordKey(owner,local) / Record(schema,fields) | plugin / save2 | Exact declarative fields; bounded bool/i64/UTF8/bytes/refs/lists; own writer |
| SceneRef / EntityRef | allocator / save2 | owner+local+incarnation; scene identity only; entity adds scene scope |
| next_identity | Core / save2 | Checked global serial starts1 for both kinds; restore resumes saved counter exactly |
| CoreOwnerToken / SceneHandle / EntityHandle | Core / runtime only | Opaque host+generation; stale on remove/recreate/restore; never saved |
| InputFrame.target_tick / actions | caller / last_input save2 | Exact next tick; sorted unique bool/bounded-i64; absent stays absent |
| history.emitted / dropped / ordinal | plugin / save2 | Independent newest256 suffix; ordinal=dropped+i; emitted=dropped+retained |
| CoreEvent.sequence / next_sequence | Core / save2 | Checked lifetime sequence=sum(active emitted)+retired; nondecreasing global ticks |
| retired_emitted / purged_retained | Core / save2 | Removed-owner lifetime/retained counts; purged<=retired; B history unchanged |
| contract_version / format_version | Core / artifact | Exactly2.0.0 /2; capped canonical8MiB JSON; save1 VersionMismatch |
| legacy hashed IDs | adapter / projection | SHA256 label keys; Unicode names as string metadata; refs remapped at install |

Serialized refs identify a restored timeline; old runtime handles remain stale
even when deterministic rewind reproduces future refs. Retained event refs block
unsafe removal. Legacy World/presentation/save/resume stay authoritative outside
Core. Continuous delegated legacy execution, Views/Gameplay and dynamic loading
are unimplemented; source-free consumer acceptance remains UNVERIFIED.

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

Workspace engine version is now 0.2.0; existing v1 framebuffer goldens remain unchanged.

FlatLand rc.1: `flatland.animation.directions` selects fixed-tick up/down/left/right clips from actor facing. Layout schema 2 permits a joystick-only profile; game-local control files are packaged presets. Android identity is fixed by application ID and pinned release certificate, with increasing version code.

## FlatLand gameplay systems (schema2 / 0.2.0)

| Identifier | Owner / persistence | Contract |
|---|---|---|
| `systems.rng` | World/save | Nonzero xorshift64 integer state; command/tick failures restore it |
| `inventory`, `equipment` | World/save | Stable item IDs, stack quantities, owned slotted items |
| `quests.<id>` | World/save | Status, bounded objectives, once-only rewarded latch |
| `attacks`, `cooldowns`, `pending_deaths` | Scene/save | Owner/preset/start/hit IDs, ordered combat/death effects |
| `planes`, `elevation`, `patches` | World/save | Actor plane/visual z, sparse scene:x:y map replacement |
| `animations`, `patrol`, `portal_latches` | Scene/save | Fixed clip phase, patrol cursor, portal arrival node |
| `events`, `event_serial`, `paused_ticks` | World/save | Bounded event stack and PC, waiting/battle, exact parent actors/camera/planes/animation; frozen parent clocks |
| `music` | World/save | Cue/file/volume; device handles excluded |
| resource `revision` | Authoring source | SHA256 checked before transactional merge/replace |
| bundle `features` | Package schema3 | Required capability IDs; ABI remains1, older importers reject |

Actor IDs remain scene-local. Scene transitions finish old-scene pending work; event scenes
return before world transitions. Save content revision prevents silent schema/content drift.

## Presentation and movement state

- Optional actor `step_walk`: held cardinal input queues a cell; release finishes that cell then stops; blocked inputs change facing. Legacy grid steering unchanged.
- Systems `view`, `pace`: saved preset ID and per-actor px/s overrides. Historical rc3 semantics (superseded by rc4): parent_view/retain_view coupled view and planes. rc4 view state is persistent and cannot change collision/elevation. parent_pace and scoped camera restore separately.
- Gameplay `views`, `default_view`, `lua_events`: bounded affine CPU camera presets and optional Lua event whitelist (empty preserves existing hooks).
- Battle fighters `sprite`, `back_sprite`, `moves{id,name,power,pp}`; save stores remaining PP. BattleState `turn_tick`, `previous_hp`, `result` drive deterministic hit feedback/held result; result continue executes reward once.
- Battle stage `background`, `hold_result`; native waiting exposes hp_max, option group/target/pp, result and animation_ticks. Client HUD consumes state; simulation remains native.
- Layout optional joystick.cardinal, defaultfalse: one axis plus12% hysteresis. No changes to existing layout defaults.
- Feature requirements `step_walk`, `battle_stage`, `view_projection` reject older clients explicitly.
- Isolated historical village fixture state town.running/starter/completed/viewed/won/crate; game ID demo.flatland.nuvema. It is not the public sample.

## Persistent gameplay and presentation state

- Systems gameplay_view: resolved view_id/zoom/tilt/shear; view_initialized preserves explicit reset across scenes. Systems.view remains a compatible preset ID.
- Systems combatants.<id>: current_hp and remaining_pp.<move>; explicit persistent fighters only.
- BattleState.fx: typed actor/target/preset/start_tick/before_hp/after_hp events; resolving_until locks cosmetic input, never combat math.
- ActorState.input: optional deterministic held-key priority for step_walk; core Input.direction is an optional most-recent cardinal intent. Maze steering remains separate.
- Actor.building: semantic footprint/height/material/roof/facing and optional roof/facade/side surfaces. Sprite.projection separates ground/upright.
- Public sample game ID demo.flatland.harbor, scene 항구, state harbor.running/completed/crate. Android application ID/certificate unchanged; versionCode8.

Presentation additions: systems.camera_blend {from,start_tick} is scoped cosmetic camera state; waiting.kind=bubble with actor/text/screen_anchor uses event serial/PC and continue. Building footprint normalizes body geometry; it is independent of camera.

Final0.2.0 waiting.kind=bubble optionally carries resolved speaker, with native
screen_anchor and screen_foot. Speaker is metadata; actor/text/line remain authored.
ABI1 errors may include error_code=save_content_revision_mismatch; no string parsing.
library.json games array persists stable IDs and order; digest owns immutable content.
saves/<game_id>/archive/<timestamp>.json preserves incompatible revisions. Settings
remain stable by ID across update/normal delete; full delete is separately confirmed.

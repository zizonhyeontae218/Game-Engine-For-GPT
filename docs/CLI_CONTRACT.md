# GE4G CLI contract — schema version 1

Binary: `ge4g`. Build/run package: `ge4g-cli`. `--json` is global and can appear after a command. JSON mode emits one object on stdout, including argument/data errors, without diagnostic prose on stderr. Non-JSON summaries are concise. `schema` emits a JSON Schema document directly in either mode.

Exit 0 means success. Validation/assertion/diagnostic failures in a completed report exit 1. Argument, I/O, parsing and runtime errors exit 2. An artifact is reported only after successful writing. No unsupported public versions or fields are silently accepted.

## Commands

| Command | Behavior / fields |
| --- | --- |
| `validate <project>` | Manifest, all scenes, PNG texture references and project replay assertions. `{schema_version,ok,errors,warnings,files_checked,summary}`. Fail-fast: the first causal error is returned; `files_checked` is empty when validation fails. |
| `inspect <project> [project\|scene\|entity\|state] [name]` | Project manifest info, scene snapshot, selected entity or state/definitions. `--scene <id>` selects a static scene; `--snapshot <file>` reads an exported runtime snapshot; `--load <save>` initializes from persistent state. Snapshot and save flags conflict. `entity` requires an id; absent entities are errors. Scene names must match the selected snapshot; use `--scene` to select. |
| `run <project>` | Opens the local window. Use `--headless` for hardware-independent execution. `--ticks N` sets exact steps (headless default: replay duration or 120); `--replay <file>` injects normalized actions. `--load`, `--save`, `--trace`, `--snapshot-out` use explicit paths. `--debug` displays outlines. Returns `{schema_version,ok,summary,snapshot,deterministic_sha256,trace,save,snapshot_out}`. |
| `capture <project> --out <file>` | Always CPU/headless, with optional `--tick N` (default 0), `--replay`, `--load`, `--debug`. Writes real RGBA PNG, returning `{schema_version,ok,summary,path,width,height,tick,scene,debug,rgba_sha256}`. |
| `test <project>` | Runs each project-defined replay, checkpoint assertions, a second identical replay, optional exact RGBA golden, save/restart and PNG write. Returns `{schema_version,ok,summary,results,errors}`. A project without tests is an error. |
| `diagnose <project>` | Explicit checks: project validation, ten idle ticks repeated, debug CPU framebuffer, typed state. Returns `{schema_version,ok,summary,checks,errors}`. It does not promise general performance analysis or repairs. |
| `schema <project\|scene\|replay\|trace\|save\|snapshot>` | JSON Schema draft 7 derived from the actual Rust types. Root schema_version is constrained to 1; semantic reference/limit/type-dependent checks remain the job of `validate`. |

Project arguments accept a directory or manifest filename. For example:

```sh
ge4g inspect examples/basement_demo entity player --json
ge4g inspect examples/basement_demo scene --scene room_b --json
ge4g inspect examples/basement_demo state --load artifacts/save.json --json
ge4g run examples/basement_demo --headless --ticks 1 --json
```

There is no separate `step` command: exact stepping uses `run --headless --ticks N` or `World::step`. There is no randomness or `--seed`. Unsupported flags fail with a structured error in JSON mode.

## Tick, input and snapshot semantics

Snapshot version 1 contains `engine_version`, `tick`, `tick_hz` (60), `subpixels_per_pixel` (60), `scene`, `camera`, `background`, `entities`, `state`, `events`, `events_dropped`.

Entity positions are integer subpixels (pixel coordinates multiplied by 60); sizes and camera offsets are integer pixels. Component names, tags, metadata, RGBA color, texture path, layer, blocking and trigger flags are explicit. Entity order is stable id order. Rendering sorts `(layer,id)`.

Tick 0 is initialized state. Replay input index 0 produces tick 1; event timestamps name the resulting snapshot tick. Movement speed is pixels/second with full per-axis diagonal motion. Interaction fires on a rising edge. Initial scene/entity events occur at tick 0.

A deterministic hash includes all snapshot state and ordered retained events, including `events_dropped`. If a save is requested, the subsequent `save_written` event includes its path and influences the hash. Repeatability tests compare gameplay before file side effects.

## Trace

The runtime retains the latest 4,096 events; discarded history is counted in `events_dropped`. A trace is `{schema_version,engine_version,events,events_dropped}`. `--trace-events interaction,scene_transition` filters event kinds in original order and requires `--trace`. Filtered-out events are not counted as buffer drops.

Supported kinds: `scene_loaded`, `entity_spawned`, `collision_started`, `collision_resolved`, `collision_ended`, `trigger_entered`, `trigger_exited`, `interaction`, `state_changed`, `scene_transition`, `save_loaded`, `save_written`, `audio`. Events contain `{tick,kind,scene,entity?,data}`. Collision and trigger data include `target`; interaction includes `target` and `dialogue`; state changes include `key`, `before`, `after`; transition includes `from`, `to`, `spawn`.

Collision contact events describe blocked attempted movement. Stopping input can emit `collision_ended` even while geometrically touching a wall. The current trigger behaviors always transition on entering/crossing; a transition clears previous contact/trigger bookkeeping.

## Save and artifacts

Save version 1 is `{schema_version,project,state}` with exactly the project's persistent keys and matching declared types. It saves declared state, not entities, scene or tick. Restart enters `start_scene`, applies loaded values, then normal scene `on_enter` behavior. Failed/corrupt/incompatible loads report context and never reset silently.

Output parent directories must exist. Save, snapshot, trace and PNG writes are atomic replacements in the same directory; Unix also syncs the directory. A capture with an unwritable/missing destination exits nonzero and does not return `ok:true`.

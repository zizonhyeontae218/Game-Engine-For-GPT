# Basement product specification

## Release identity

Version: `0.1`
Codename: `Basement`

Basement is an engineering proof, not a content-production release.

## Runtime capabilities

### Simulation
- fixed-step deterministic tick loop
- explicit simulation tick counter
- deterministic input sampling
- seeded RNG API if randomness is used
- pause/step support in headless mode

### World
- stable entity identifiers
- transforms
- sprite/render component
- AABB collider
- trigger volume
- simple player/controller component
- camera
- tags/metadata
- minimal persistent state store

### Scenes
- load one scene
- transition to another scene
- deterministic spawn point selection
- validation before runtime

### Interaction
- proximity/trigger interaction sufficient for NPC dialogue and doors
- interaction events visible in trace/inspect output

### Scripting
Basement may use embedded Lua for small game-specific behaviors.

Constraints:
- gameplay scripts execute through a narrow GE4G API
- filesystem, process, and network access are disabled by default
- script-visible time is simulation time, not wall clock
- script errors include entity/script/tick context
- engine correctness must not depend on undocumented Lua globals

If Lua meaningfully delays the first vertical slice, implement built-in declarative behaviors first and add Lua after the deterministic core is proven.

### Save
- explicit schema version
- atomic write
- minimal namespaced key/value state
- load validates version and types
- corrupt save reports a useful error; it must not silently reset

## Rendering

### Basement renderer
Use a deterministic CPU RGBA framebuffer for the canonical 2D render path.

The local window presents that framebuffer.
Headless capture writes that same framebuffer to an image file.

This deliberately avoids making GPU availability a requirement for Codex Cloud verification.

Required primitive support:
- clear/background
- textured or solid rectangle sprite
- camera transform
- deterministic draw ordering
- optional debug overlays for colliders/triggers

Text rendering is optional for the first vertical slice. If added, pin the font asset used in deterministic capture tests.

## Audio

Audio must not be required for simulation correctness.

Basement can expose an audio event API with:
- local playback backend when an audio device exists
- headless event logging/no-op sink

Do not block the release on rich mixing or effects.

## Authoring format

- project manifest: `ge4g.toml`
- scene files: JSON5 with an explicit schema version
- input replay: JSON
- machine output: JSON
- save: versioned JSON or another fully specified deterministic format

Why JSON5 for scenes:
- agent/human readable
- comments and trailing commas
- structured validation
- no new custom DSL in Basement

## Demo scenario

The reference demo must support this deterministic sequence:

1. Player starts in `room_a`.
2. Scripted input walks right.
3. Player collides with a wall; position cannot pass through it.
4. Player reaches NPC interaction range.
5. An interaction event is emitted.
6. Player reaches a door trigger.
7. Scene changes to `room_b`.
8. A persistent state value is modified.
9. Save is written.
10. Runtime is restarted headlessly.
11. Save is loaded and value is verified.
12. A known tick is captured to PNG.
13. Repeating the same replay produces equivalent state/event results.

## Definition of done

Basement is done only when the acceptance matrix in `TESTPLAN.md` passes from a clean checkout and the demo is locally playable through the interactive adapter.

## Required client extension (2026-10-02)

The user extended Basement with Flutter mobile/Windows/Arch runtime clients. Mobile uses import→play; desktop games embed the client at distribution time. This supersedes any earlier mobile-packaging exclusion. Game-authoring editor GUI stays out of scope; runtime touch-control editing is supported. See [CLIENT](CLIENT.md) and [LAUNCHER_PHILOSOPHY](LAUNCHER_PHILOSOPHY.md).

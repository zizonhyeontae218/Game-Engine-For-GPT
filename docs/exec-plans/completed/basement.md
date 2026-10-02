# GE4G 0.1 Basement implementation

## Outcome
A locally playable 2D demo and a real deterministic headless engine that an agent can validate, inspect, replay, test, diagnose, save and capture through a versioned JSON CLI.

## Context
The requested GitHub repository was empty on 2026-10-02. The supplied GE4C instruction bundle is the specification; the user renamed the product GE4G / GameEngineForGPT. Stable Rust edition 2024 is required. This executor initially had no Rust toolchain.

## Scope / non-scope
Implement fixed 60 Hz simulation, integer subpixel motion, swept AABB collision, NPC dialogue, scene transitions, typed persistent state, atomic saves, CPU RGBA rendering, PNG captures, and a window adapter. Declarative scene behaviors satisfy the scripting option. No editor, Lua dependency, 3D, networking, or general physics.

## Acceptance evidence
Format, strict Clippy, workspace tests, demo validate/test, repeat replay comparison, save/reload, exact framebuffer golden, real PNG, JSON CLI error tests, filetree lint, and a clean-checkout run. Interactive adapter will be exercised with a virtual display when possible. Actual human execution remains unclaimed until a human plays it.

## Milestones
1. Parse scenes → move a player through replay → inspect JSON → CPU capture, with executable tests.
2. Complete wall/NPC/door/state scenario, save/reload and project assertions.
3. Interactive adapter, CLI schemas, diagnostics, documentation and CI.
4. Clean verification and publish the initial commit to the requested repository.

## Decisions
- 2026-10-02: Brand `GE4G`, binary `ge4g`, manifest `ge4g.toml`, packages `ge4g-*`. No existing consumers or files require a migration. Public schemas start at version 1.
- 2026-10-02: Fixed 60 Hz and 60 integer subpixels per pixel avoid floating-point simulation drift. Input uses per-axis movement, documented diagonal behavior.
- 2026-10-02: Keep gameplay declarative. The specification makes Lua optional; current behaviors do not require a script VM.
- 2026-10-02: Headless dependencies are independent of the optional window adapter; both render the same CPU framebuffer.

## Progress
- Completed: all six populated crates, versioned CLI, playable two-room demo, swept collisions/triggers, typed atomic save/load, CPU renderer and PNG capture, optional window adapter, schema/inspection/trace/diagnostics, 22 tests, documentation and CI.
- Completed: strict default/headless-only checks, seven demo checkpoints, exact frame golden, repeated event/state comparison, real Xvfb window/headless equivalence.
- Completed: clean-checkout format/Clippy/22 tests/demo validate/demo test/filetree lint and real window equivalence. Extracted release bundle executes the demo test successfully.
- Publication: initial main commit is ready; remote CI will run after push.
- Blocked: no engineering blocker. Actual human play remains pending and is explicitly unclaimed.

## Verification log
- `git clone …`: empty requested repository confirmed.
- Managed environment network policy: unrestricted/enforced; GitHub connector has push access.
- `cargo fmt --all --check`, strict workspace Clippy and `cargo test --workspace`: passed, 22 meaningful tests.
- `ge4g test examples/basement_demo --json`: seven assertions, 160 ticks, deterministic snapshot/events, golden, save/reload and PNG passed.
- `cargo clippy/test -p ge4g-cli --no-default-features`: passed, including nine CLI integrations without the window adapter.
- `DISPLAY=:99 python3 scripts/interactive_smoke.py`: real minifb window/headless snapshots identical; final tick 160, room_b, SHA256 `a950e9a1c9d5028be69910976f3bfa85c929196d38a30261ea23eb61ef19bde9`.
- `ge4g validate/diagnose`: all referenced files and four explicit diagnostic checks passed.
- Captured and visually inspected the actual room_b PNG; RGBA hash `38cc4352e7160dcf9104cbe19acd709e36d8165989b3c99d8b6611f0f76e932b` is pinned in the project test.

- Clean checkout `/tmp/ge4g-verify`: full canonical command sequence passed with a fresh target directory; Xvfb adapter comparison also passed.
- `cargo build --locked --release -p ge4g-cli`, `python3 scripts/package.py`, extracted binary `ge4g test … --json`: passed with exact reference hashes.
- Local engineering milestone complete; human-inclusive final acceptance remains pending.

## Handoff
Work in `/workspace/Game-Engine-For-GPT`. Preserve versioned formats. Run `python scripts/filetree.py update` after final edits. Rebuild the default CLI before window checks after headless-only tests, which share the executable path. Actual human acceptance is the remaining external sign-off; follow TESTPLAN and record its result honestly.

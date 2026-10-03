# GE4G agent map

The shipped engine is **GameEngineForGPT 0.1 "Basement"**.
Current development build: **Basement 0.2 — FlatLand, 0.2.0-alpha.1**.
The larger 0.2 specification is partially implemented; consult `docs/FLATLAND_AUTHORING.md` for actual capabilities.

## Read by task, not by ritual

Use only the document relevant to the current work:
- Product intent/scope → `00_MASTER_CONCEPT.md`, `GOAL.md`, `docs/BASEMENT_SPEC.md`
- FlatLand authoring / current capabilities → `docs/FLATLAND_AUTHORING.md`
- FlatLand 0.2 design/implementation → `docs/exec-plans/active/flatland.md`, relevant section of `docs/FLATLAND_SPEC.md`
- Architecture/module boundaries → `docs/ARCHITECTURE.md`
- CLI behavior/output/exit codes → `docs/CLI_CONTRACT.md`
- Engine-visible durable state IDs → `F(x).md`
- Tests/acceptance → `docs/TESTPLAN.md`
- Known non-obvious failures → `docs/FAILURE_NOTES.md`
- Long or multi-stage work → `.agent/PLANS.md`
- Repository navigation → `FILETREE.md`
- Runtime clients / packaging / input profiles → `docs/LAUNCHER_PHILOSOPHY.md`, `docs/CLIENT.md`

Do not read every document before every edit.

## Non-negotiable Basement rules

1. **Headless is a first-class runtime**, not a mock.
2. The simulation must not depend on a window, GPU, audio device, wall-clock timing, or network.
3. Agent-facing observation must be structured and stable. Prefer machine-readable JSON plus concise human text.
4. Interactive rendering and audio are adapters around the same simulation used by headless tests.
5. Keep deterministic behavior deterministic: fixed-step simulation, seeded randomness when randomness is necessary, replayable input traces.
6. No game-authoring editor GUI in 0.1. Runtime clients and their touch-control editors are supported product surfaces.
7. No 3D, networking, plugin marketplace, visual scripting, or general-purpose physics engine in 0.1.
8. Do not add abstractions for hypothetical future features unless current Basement code needs them.
9. Prefer inspectable in-repo code over opaque framework behavior. Low-level libraries are allowed; importing another full game engine is not.
10. Never silently change a public CLI/scene/save schema. Version or migrate it.

## Launcher philosophy — all future runners

- Mobile: install the Flutter client, choose **Import**, load a portable Basement game package, then play. Imported games are data; the client owns the native runtime.
- Desktop (Windows and Arch Linux): distribute each game with its client and native runtime already embedded. A shipped game must start directly without asking the player to install an engine or locate a CLI.
- Every runtime adapter calls the authoritative Rust simulation and presents its canonical CPU framebuffer. Do not reimplement gameplay in Dart or spawn the development CLI as a mobile runtime.
- Use digital brutalism for the client: flat strong contrast, hard borders, explicit typography and direct controls.
- Mobile default controls are a joystick and Z, X, C, Space. Touch layouts are versioned JSON with live profile switching/editing. Game-specific mappings are persisted separately from layout JSON and game save state.
- A valid live profile change releases held inputs before applying it; an invalid edit retains the last valid profile and reports the error. Backgrounding, focus loss and touch cancellation release inputs.
- Prove imports, native play, live mappings and embedded desktop packaging with executable evidence. Report each platform's actual build/device verification honestly.
- Deliver requested demo builds through the connected Google Drive: `Demos/<engine version>/<build version>/` (for example `Demos/Basement 0.2 FlatLand/0.2.0-alpha.1/`). Reuse verified folders, upload the portable game and platform packages with instructions/checksums, verify the uploaded files, and return Drive links. Workspace file links are not downloadable for this user. Preserve existing Drive sharing permissions.

## Working style

- Inspect the narrow relevant surface before editing.
- Make the smallest coherent change that satisfies the task.
- Keep unrelated refactors out of the change.
- For multi-stage or architectural work, create/update an ExecPlan under `docs/exec-plans/active/`.
- Record important decisions in the plan; do not rely on chat history.
- If a new stable cross-system state identifier is introduced, update `F(x).md`.
- After repository structure changes, run the FILETREE update/lint helper.
- Tests are disposable/local. Run and repair tests affected by your change without asking for approval at each iteration.
- A task is not complete because code compiles; provide executable evidence appropriate to the change.

## Verification floor

Use the narrowest relevant checks first. Before declaring a Basement milestone complete, the repository must pass the acceptance commands in `docs/TESTPLAN.md`.

## Stop conditions

Do not paper over:
- nondeterministic tests,
- headless/interactive simulation divergence,
- schema drift,
- swallowed errors,
- fake captures,
- tests that only assert that a command returned success.

Fix the cause or document a real blocker in the active ExecPlan.

## FlatLand implementation direction

- Separate static map walls/cells from runtime actors and interactive Entity objects.
- Entity bodies declare `pass`, `fixed` or `push`; facing is shared by movement, interaction, combat and animation.
- Prefer compact JSON5 prefabs and validated conditions/actions for ordinary content; Lua 5.4 extends unusual behavior through the same authoritative commands.
- Minimize input context as well as output: query one component/prefab, patch one resource, observe selected deltas; full artifacts remain opt-in. Measure total task tokens and repair cost.
- Depth rendering, collision planes and visual elevation are separate concepts.
- Cutscenes and turn combat use serializable event scenes; headless must accept the same choices and reproduce their results.
- Sound events require actual playback adapters before claiming audio support.
- Version/migrate public data, saves, package requirements and ABI explicitly; preserve v1 behavior.
- Follow the plan's vertical slices. Proposed schemas/APIs must never be described as shipped features.

- Orientation is selected with a user button; do not enable sensor-driven landscape changes. Keep dialogue/text popups separate from controls.

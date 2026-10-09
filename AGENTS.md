# GE4G agent map

The main development line is **GE4G 0.3.0 — Pentomino**, currently
**0.3.0-alpha.1** (Flutter 0.3.0-alpha.1+9). Pentomino is the version name,
not a separate product. Read `docs/pentomino/STATUS.md` for implemented scope.
`release/0.2` preserves the FlatLand maintenance line and v0.2.0 remains released.
P0/P1 are merged; do not restart them or claim the legacy CLI runs Pentomino.
The previous claimed final is archived as rc2. Preserve rc1/rc2/rc3/rc4/rc5 binaries. Historical FlatLand final uses Flutter0.2.0+8 / Android versionCode8.
Schema 1 remains supported. Actual capabilities and limits: `docs/FLATLAND_AUTHORING.md`.
For small authoring edits read `docs/FLATLAND_QUICKSTART.md` and query one component.

## Read by task, not by ritual

Use only the document relevant to the current work:
- Product intent/scope → `00_MASTER_CONCEPT.md`, `GOAL.md`, `docs/BASEMENT_SPEC.md`
- FlatLand authoring / current capabilities → `docs/FLATLAND_AUTHORING.md`
- FlatLand 0.2 design/implementation → `docs/exec-plans/completed/flatland.md`, `docs/exec-plans/completed/flatland-polish.md`, relevant section of `docs/FLATLAND_SPEC.md`
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

## Supported platforms through FlatLand 0.2

- Active client support, build verification and distribution: **Android and Windows only**.
- iOS, macOS and Linux/Arch support work is suspended until **0.3.0 development begins**; do not build, test, package or publish clients for those platforms during FlatLand0.2.
- Preserve existing platform source and every historical artifact. Reconsider the support matrix explicitly at 0.3.0; do not automatically resume suspended jobs.
- Linux-hosted Rust/headless checks and Android cross-compilation are development infrastructure, not Linux client support.

## Launcher philosophy — all future runners

- Mobile: install the Flutter client, choose **Import**, load a portable Basement game package, then play. Imported games are data; the client owns the native runtime.
- Desktop (Windows; Linux/Arch deferred until 0.3.0): distribute each game with its client and native runtime already embedded. A shipped game must start directly without asking the player to install an engine or locate a CLI.
- Every runtime adapter calls the authoritative Rust simulation and presents its canonical CPU framebuffer. Do not reimplement gameplay in Dart or spawn the development CLI as a mobile runtime.
- Use digital brutalism for the client: flat strong contrast, hard borders, explicit typography and direct controls.
- Generic mobile controls are a joystick and Z, X, C, Space; packaged game presets may omit unused buttons using layout schema 2. Touch layouts are versioned JSON with live profile switching/editing. Game-specific mappings are persisted separately from layout JSON and game save state.
- A valid live profile change releases held inputs before applying it; an invalid edit retains the last valid profile and reports the error. Backgrounding, focus loss and touch cancellation release inputs.
- Prove imports, native play, live mappings and embedded desktop packaging with executable evidence. Report each platform's actual build/device verification honestly.
- Publish requested public demos and engine releases as GitHub Release assets: signed Android runner, embedded Windows example, portable game + editable source, C SDK/example, AgentKit, licenses and SHA256 checksums. Verify uploaded names, sizes and hashes. Actions artifacts are CI evidence, not permanent downloads. Private Drive remains a signing-key/history backup; never require Drive permissions for public releases or change existing sharing.

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

- Android releases must retain application ID `dev.ge4g.ge4g_client` and the certificate pinned in `client/android/signing-certificate.sha256`; increment the build version code. Restore the original key from the user's private `GE4G Private Signing` Drive folder. Never generate a replacement key or release-sign with ephemeral debug keys. Hosted APKs are unsigned; use `scripts/sign_android.py` before delivery. Private keys/passwords must never enter Git or the public Demos folders.

- Full 0.2 regression: test Harbor Workshop and Signal Yard as well as Pac-Man and the v1 demo. Keep choice/turn resume, RNG rollback, exact parent return and canonical client frames covered. Android final uses versionCode 8 and the rc.1 certificate.

- Village-style games should opt into `step_walk` and cardinal touch profiles; preserve maze steering for existing games. Battle presentation must use a dedicated stage rather than a dialogue popup. Viewpoint changes are presentation-only and persistent. Project ground and actor/building feet during composition, never tilt the completed framebuffer. Cutscene cameras restore the persistent gameplay view. Legacy retain_view parses but cannot couple view to plane/elevation. Keep battle simulation, roster, FX, camera, input and building fallback responsibilities separate.


## Final reliability invariants

Semantic footprints are solid by default and define body geometry. Roof pixels never
create gameplay walkability. South-only facing in0.2; other variants must reject.
Contact ordering/shadows and cutscene bubble presentation remain separate modules.
Preserve exact-once state and view invariants. User confirmed rc5 physical acceptance
and authorized direct final promotion after the focused finalization gates pass.
Never imply new library UI was physically tested from CI/rendered screenshots.
Run scripts/release_consistency.py; docs are part of the AI-facing API.
3D/pixelization is0.3+ exploration, not a0.2 blocker.

Stable game_id owns settings/save/order. Different content digest is an update, not a
duplicate. Validate before closing, atomically activate, archive only typed revision
mismatch, roll back failures, and retire old content only after successful open.
Normal delete keeps user data; full delete confirms and removes it. Embedded Windows
never exposes game management. Speaker metadata is presentation-only and optional.


## Pentomino 0.3 worker entrypoint

main targets Pentomino 0.3.0 alpha; the latest stable release remains FlatLand 0.2.0.
P1 preserves the scalar host subset documented in P1_CONTRACT.md. P2 adds the
experimental typed p2::CoreHost and explicit legacy snapshot adapter; read
P2_CONTRACT.md and P2_RUNDOWN.md. External public Core GPT Work23/23 accepted;
external legacy execution remains UNVERIFIED. P2 milestone is accepted.
Camera & View/Format belong to P3; gameplay remains unimplemented. Read STATUS.md.
When the user invokes `$pentomino-orchestrate`, read
`docs/pentomino/HANDOFF.md` and `docs/pentomino/BRIEF.md`, then dispatch
`pentomino_scout` read-only before architecture or implementation.
The current user request authorizes preparation for 0.3; Soul.md's previous
wait-for-new-instructions note does not cancel a subsequent explicit P0 start.

- Follow Tiny Core → View/Format → Gameplay, with Forge deferred to 0.4.
- Keep existing deterministic Rust simulation, data-only game imports, public
  schema/save/ABI compatibility, Android identity and pinned signing certificate.
- Define bounded packets and disjoint file ownership using
  `templates/TASK_PACKET.md`; parent integrates shared contracts sequentially.
- Review the minimum public API proposal with `pentomino_architect`, then
  `pentomino_auditor`, before functionality changes. Approval here is the parent
  orchestrator's technical review, not a new user confirmation for routine work.
- Use `docs/pentomino/GATES.md` for contract/release changes and
  `templates/RUNDOWN.md` for results. Unimplemented behavior or unrun checks are
  UNVERIFIED. Tooling checks do not prove the engine works.
- Preserve the historical tag Windows test failure in the handoff. The user reported
  all existing tests completed on 2026-10-09; record this separately from CI evidence
  and do not restart resolved work or weaken digest/save validation.
- Reconsider platform support explicitly in the 0.3 plan; do not automatically
  enable suspended client jobs.
- External consumer validation requires a separate source-free workspace/session
  and public artifacts; ordinary in-repo workers cannot certify it.

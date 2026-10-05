# FlatLand 0.2.0-rc.4 stabilization

## Outcome

Ship an Android/Windows-only stabilization candidate, never final 0.2.0. Engine
0.2.0-rc.4, Flutter 0.2.0-rc.4+6, Android versionCode 6 with unchanged application ID
and pinned signing certificate. Preserve every rc1/rc2/rc3 historical binary.

## Context

Physical rc3 Android feedback overrides rc3 design assumptions. Whole-frame affine
projection tilts actors/buildings; view selection wrongly changes collision plane and
height; scene transitions lose viewpoint; input arbitration favors horizontal input;
battles have thin feedback and ephemeral HP/PP. Save/resume worked well and must be
extended conservatively rather than rewritten.

## Scope / non-scope

- Project ground during composition, billboard actors/buildings at projected feet.
- Persistent gameplay view independent of simulation; scoped cutscene camera.
- Last-input-wins step movement, joystick hysteresis, faster village walk/run.
- Explicit persistent combatants, reusable deterministic cosmetic battle FX.
- Semantic building fallback and entity visual defaults, partial-surface fallback.
- Original public sample and deliberately under-authored acceptance scene with asset
  provenance manifests. No recognizable third-party map or branding in new releases.
- UTF-8 Korean authoring/validation/packaging/import/launch on Windows CI.
- Only Android and Windows client work until 0.3.0 development begins. Do not build,
  test, package or publish iOS/macOS/Linux clients. Retain their existing sources and
  historical artifacts. Linux-hosted Rust/headless and Android builds are infrastructure.
- Preserve ABI1, schema1 and existing schema2 content where possible; declare new
  capability requirements explicitly. Legacy inline fighters remain ephemeral.

## Acceptance evidence

Rust/Flutter/native regressions cover view simulation invariants, same camera target,
scene/cutscene/save view persistence, billboard ground anchors, newest input and
buffered turns, persistent HP/PP and explicit healing/reset, battle exactly-once
resume/rewards/RNG, legacy inline fighters, Pac-Man and Signal Yard.
Real Android/Windows rendered evidence must show top/depth at identical world
coordinates with upright actor/house over projected ground, and battle anticipation,
effect, red impact, HP interpolation and menu recovery. Record exact world snapshots.
Headless uses identical authoritative state independent of presentation. Physical
Android user acceptance remains separate; never infer it from CI/screenshots.

## Milestones

1. Platform policy/CI suspension and durable scope (implemented locally).
2. Extract view/projection responsibility; presentation-only persistent view and
   conservative save extension; scoped camera; invariants and rendered anchors.
3. Input arbitration/hysteresis and village speed, preserving maze grid movement.
4. Separate persistent combatants, battle simulation and deterministic FX state;
   exactly-once save/resume and reusable visual feedback with locked input.
5. Building presentation/fallback module and minimal entity defaults; original
   sample/asset manifests/under-authored scene.
6. Windows UTF-8 pipeline, full Android/Windows checks and rendered evidence.
7. Publish code, code6 signed APK and Windows bundle into a new rc4 Drive directory,
   verified names/parents/sizes/checksums; leave historical directories untouched.

## Decisions

- 2026-10-05: Latest user instruction suspends iOS/macOS/Linux support work until
  0.3.0 starts. Supersedes rc4's earlier all-platform build requirement. Client CI
  retains only Windows and Android jobs. Engine CI retains Linux as a host, but no
  longer publishes Linux release archives. Flutter analysis/tests move to Windows.
- Do not use a framebuffer tilt or view-driven plane/elevation changes. Camera/view,
  persistent gameplay state and temporary cosmetic state require separate boundaries.
- Preserve existing signing key; do not generate a replacement.

## Progress

Completed locally: support policy recorded in AGENTS/client/launcher docs; removed
Linux/iOS client jobs and Linux archive publication; Windows owns Flutter checks.
Next: verify/publish policy slice, then implement view and projection milestone.
No rc4 gameplay changes or rc4 candidate binaries exist yet.

## Verification log

2026-10-05: workflow YAML parsed successfully; jobs are exactly Windows/Android;
Windows runs Flutter analysis/tests; Android retains APK builds; engine workflow has
no Linux archive publication. `git diff --check` and FILETREE update/lint pass.
No suspended client builds were run. Hosted Windows/Android CI remains pending.

## Handoff

Repository /workspace/Game-Engine-For-GPT. rc3 is the latest delivered candidate.
Read narrow runtime/project/renderer surfaces, not every document. Active task is
rc4 stabilization with Android/Windows only. Old rc3 view/retain_view/plane coupling
in AGENTS and polish tests is superseded by the presentation-only invariant above.
Private signing material is outside Git; never upload it to public demo folders.
Delivery is through connected Drive Demos/Basement 0.2 FlatLand/0.2.0-rc.4.

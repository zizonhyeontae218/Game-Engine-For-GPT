> 역사 기록: 이 문서의 버전·배포 링크·검증 상태는 당시 기준이며 현재 안내는 [릴리즈 목차](../README.md)를 따른다.

> Reclassified as **0.2.0-rc.2** by the user. The original APK bytes/versionCode4 remain unchanged. This build is not a final 0.2.0 release.

# Basement 0.2 — FlatLand rc2 (archived)

User reported rc.1 testing complete on 2026-10-05 and authorized the full release plus
an additional demo. This release adds the gameplay absent from Maze Chase: combat
presets/projectiles, inventory/equipment, dedicated quests, layered bridge/camera/atlas,
serializable conditional choices/cutscenes and turn combat, seeded Lua modules/RNG,
looping music and revision-checked compact resource edits.

Signal Yard is a playable CC0 industrial adventure and feature laboratory. Its 714-tick
authored journey finishes the key gate, robot combat, bridge, arena, quest and cutscene.
Expected result: complete quest, 13 coins, player (80,240), exact return to the parent.
Pac-Man remains available; schema1 Basement retains its original frame golden.

## Executable evidence

- Rust behavioral suite: 44 tests; strict Clippy and rustfmt; headless-only CLI checks.
- Flutter: 15 tests and clean analysis, including native choice save/resume and large-text
  landscape choice/save usability.
- Authored project tests: Basement, Maze Chase and Signal Yard; behavioral checkpoints,
  repeatability, save/reload and decoded CPU frame goldens.
- Signal Yard embedded Linux Flutter/headless states and RGBA frame agree at tick714:
  `f50923d16622d3ee8232739c821d2e8cc6e98acaca9b0dc2c9c174604962a987`.
- Native rodio/cpal adapter decoded/mixed cues and looping music into nonzero virtual
  ALSA PCM. This is actual adapter playback evidence, not physical speaker acceptance.
- Android version 0.2.0+4 keeps application ID `dev.ge4g.ge4g_client` and the pinned
  rc.1 certificate. Signing verifies the prior APK; installed rc.1 can update in place.
- Hosted workflows build Linux/Windows embedded games, three Android ABIs and unsigned
  iOS. Final run IDs/artifact inspection are recorded in the delivery manifest.

## Reproducible context measurement

`python3 scripts/flatland_task_benchmark.py` (tiktoken0.14.0/cl100k_base) performs the
same attack edit through whole-resource rewrites and scoped revision-checked patches.
Each route deliberately fails once and repairs once; resulting source and frames match.
Counts include supplied docs and tool results. Timing measures tool completion, not LLM
sampling. See the delivered JSON for measured input/output tokens, calls and repair time.
Assets use deterministic Python generation; no asset-generation prompts are counted.
Canonical vocabulary SHA256:
`223921b76ee99bde995b7ff738513eef100fb51d18c93597a113bcffe865b2a7`.

## Practical limits and acceptance scope

The shipped contract is FLATLAND_AUTHORING. This is a deterministic top-view engine,
not a promise to reproduce every game. Dynamic congestion avoidance, generalized
rigid-body physics, network play and 3D are outside scope. Turn fighters use explicit
stats; independent positional/ambient-loop voices are future audio extensions.
Content-bound saves reject changed game resources rather than silently migrating them.

rc.1 user testing is recorded as reported, without inferring specific audio/device tests.
This changed release has automated Linux execution/virtual audio evidence and platform
build inspection; physical Android/Windows/Arch interaction and audible speaker checks
remain device acceptance. Drive delivery preserves old versions and existing sharing.

## Final publication

Build source: `14b6942d0c2ed15be6e499ff57ad59c1a9c8f1c7`. Engine acceptance
[37303276002](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37303276002)
and all four client jobs in
[37303275913](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37303275913)
succeeded. Windows package game entries match portable packages byte-for-byte. Android
natives cover three ABIs; apksigner verifies v2/v3 and prior-certificate equality; aapt
confirms app ID and versionCode4. iOS native framework/version0.2.0 build4 inspected.

[Drive rc2 archive](https://drive.google.com/drive/folders/1YeIJO1SoQ-ncx-mlU6jxm1QLM4vI5XKX)
contains installable signed APK, both .ge4g games, embedded Windows/Linux builds, source,
preview, Korean instructions, context/audio evidence, checksums and unsigned iOS.
Uploaded names, sizes, parent folders and download availability were verified.

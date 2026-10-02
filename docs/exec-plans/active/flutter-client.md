# Flutter Basement client and embedded launchers

## Outcome
A digital-brutalist Flutter player for Android/iOS, Windows and Arch Linux. Mobile imports versioned game packages. Desktop distributions embed the client, runtime and one game. Joystick + Z/X/C/Space use separate per-game JSON layouts/mappings, switch and edit live, and persist safely.

## Context
GE4G 0.1 Basement is implemented and verified. Its native Rust world and CPU renderer must remain authoritative. The user explicitly extends scope to mobile packaging and makes the mobile-import / desktop-embedded philosophy permanent. This executor initially has Rust but no Flutter/native desktop build toolchain.

## Scope / non-scope
Implement portable game archives, a versioned C ABI over the existing runtime, Flutter native bindings, import/library/play/control editing, per-game save/settings storage, platform projects and reproducible build/bundle workflows. No second game simulation, cloud service, downloaded game executable, or general game editor.

## Acceptance evidence
Rust regressions and ABI/frame equivalence, package import/identity/path validation, Dart profile/mapping/storage tests, widget multi-touch/live-switch tests, Flutter analysis, a built Linux client with real native play, embedded-game launch, and hosted Windows/Android builds where available. iOS requires Xcode and signing/device evidence is reported separately.

## Milestones
1. Record launcher direction; expose the authoritative Rust runtime through native ABI and portable package.
2. Implement Flutter import/player, digital-brutalist shell and live controls/settings.
3. Integrate mobile/desktop native builds and embedded distributions.
4. Validate, publish to the requested repository, and record precise platform evidence.

## Decisions
- 2026-10-02: Mobile targets Android and iOS; desktop targets Windows x64 and Arch Linux x64.
- 2026-10-02: Use a native Rust library through Dart FFI, not a gameplay rewrite or CLI subprocess.
- 2026-10-02: Portable `.ge4g` ZIP contains game metadata, Basement project/assets and control defaults. Stable game id isolates settings and saves across games.
- 2026-10-02: Layout profiles and action bindings are separate JSON files. The client validates a combined candidate before activating it and watches external edits.
- 2026-10-02: Keep original scene/save/replay v1 behavior intact. New client/package/control/ABI contracts have explicit version 1; named actions are additive host observations, while standard directional/interact input remains identical.

## Progress
- Completed: authoritative Rust ABI; versioned validated `.ge4g` import; Flutter player/library; digital brutalism; joystick/Z/X/C/Space; live profiles and JSON editing; separate game controls/saves; desktop embedded bundles; platform build workflow; permanent launcher docs.
- Completed locally: 24 Rust tests, strict Clippy/fmt, authored demo acceptance, 8 Flutter behavioral tests including actual native FFI, Linux release build.
- Completed: packaged Flutter replay/frame smoke; complete snapshot equals CLI, exact golden RGBA; real running window visually inspected.
- Next: publish and inspect native-host platform CI.
- Limit: local executor is Linux; iOS signing/device and actual Arch installation require their platforms and owner credentials.

## Verification log
- Starting commit `ff5d7fc`; Flutter 3.47.6/Dart 3.13.5 installed.
- `cargo fmt --all --check`, strict workspace Clippy, all 24 tests and demo test pass; original deterministic hash and frame golden unchanged.
- `flutter test`: 8 pass, including FFI imported-demo replay at tick160 room_b, canonical RGBA golden and real save/reload.
- `flutter build linux --release`: complete Flutter/native runtime bundle produced.
- Temporary GCC/GTK/Clang tools extracted into workspace because no administrative package installation is available. These are executor dependencies, never checked in or shipped as engine code.

## Handoff
Repository `/workspace/Game-Engine-For-GPT`. Keep saves, touch layouts and game mappings separate. Never claim device tests that were not performed. Native code-assets hooks build Rust automatically. Platform CI is `.github/workflows/client.yml`. Before final publication synchronize this plan, release notes and filetree.

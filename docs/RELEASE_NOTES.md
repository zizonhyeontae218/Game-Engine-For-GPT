# GE4G 0.1.0 — Basement

Date: 2026-10-02. The requested repository started empty. The supplied GE4C 0.1 instruction set was implemented as **GE4G / GameEngineForGPT**, with binary `ge4g`, project `ge4g.toml`, and six Rust edition 2024 workspace packages. This initial release has no earlier published schemas to migrate.

## Delivered

A real deterministic headless 2D runtime, an optional local game window using the same simulation and CPU framebuffer, versioned TOML/JSON5/JSON authoring and observation formats, swept AABB collision and triggers, NPC interaction, named scene transitions, typed persistent state, atomic save/load and PNG output, structured inspect/trace/diagnostics/schema commands, authored replay assertions, a reference demo and CI.

The reference journey has seven checkpoints across 160 ticks. It moves the player, proves a wall stops motion, talks to the NPC, enters room_b at the named spawn, and changes both persistent demo flags. Tests restart from an actual save and write/decode real PNGs. Repeated final snapshot/event results match.

## Evidence

- `cargo fmt --all --check`: passed.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: passed.
- `cargo test --locked --workspace`: 22 tests passed.
- Demo `validate`, `test` and `diagnose`: passed; exact golden checked.
- `cargo clippy/test --locked -p ge4g-cli --no-default-features`: passed, including all nine CLI integration tests without a window adapter.
- Real minifb window run under Xvfb matches headless snapshot/events at tick 160, scene room_b.
- Gameplay snapshot SHA256: `a950e9a1c9d5028be69910976f3bfa85c929196d38a30261ea23eb61ef19bde9`.
- Canonical final RGBA SHA256: `38cc4352e7160dcf9104cbe19acd709e36d8165989b3c99d8b6611f0f76e932b`.
- Build toolchain used: stable Rust 1.99.0 on Linux x86_64. Other platforms and older compiler versions have not been exercised here.
- Fresh local clone with a fresh build directory: format, strict Clippy, all 22 tests, demo validate/test, filetree lint and real window comparison passed.
- Optimized release build and extracted executable/demo package test passed.
- Detailed command evidence is recorded in `docs/exec-plans/completed/basement.md`.

## Publication

Implementation commit: `ad339f1d9152c2d68035ff387e251af7212c4a5a`. [Hosted acceptance CI](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/36986830491) passed every step, including the real window comparison, headless-only checks and release bundle build/upload. The published Git tree was checked against the locally verified tree.

[Download the Linux x86_64 executable and demo bundle](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/36986830491/artifacts/11218325229). Extract the nested tar.gz, enter `ge4g-basement`, and run `./ge4g run examples/basement_demo`. Linux X11/XWayland runtime libraries are required for a window; `--headless` needs no display.

## Scope and remaining acceptance

The engine implements declarative behaviors; Lua was optional in the instructions and is omitted. Audio cues are logged with a silent adapter; sound playback is omitted. There is no game-authoring GUI editor, 3D, network gameplay or general physics. Saved data contains declared persistent keys, not a whole-world resume point.

Automated engineering acceptance is implemented and tested. **Actual human execution is pending:** no person has yet played the demo in this session, tested the keyboard controls and F5/F9 workflow, or signed off on the experience. The Xvfb run is automated adapter evidence only. Therefore the instruction set's final human-inclusive definition of done and ILCX™ human contribution evaluation remain pending.

Human acceptance record: tester **pending**, date **pending**, result **pending**. Follow `docs/TESTPLAN.md` when conducting it.

## Flutter Basement client extension — 2026-10-02

GE4G / GameEngineForGPT now includes Flutter Android/iOS, Windows and Linux projects, the bundled authoritative Rust client ABI, validated portable `.ge4g` imports, digital-brutalist library/player screens, joystick and Z/X/C/Space controls, live profile switching/editing, separate per-game layout/mapping JSON and game save files. Windows/Arch game distributions embed the full client and game and open directly. AGENTS and product/architecture docs record this as the standing launcher philosophy.

Locally verified: 24 Rust tests and strict Clippy/fmt; authored deterministic demo; 8 Flutter tests including actual FFI replay/pixels/save/load, invalid imports, game isolation and touch chords/cancellation; Linux release build; a real packaged Flutter window at tick160 room_b with its full snapshot equal to headless and canonical RGBA hash unchanged (`38cc4352e7160dcf9104cbe19acd709e36d8165989b3c99d8b6611f0f76e932b`). The actual running client window was captured and visually inspected. Evidence files are local `artifacts/client-smoke/` and uploaded by hosted Linux client CI.

Platform build workflow produces Windows embedded ZIP, Linux embedded tarball, `.ge4g` demo, Android APK (development-signed) and unsigned iOS app. Native-host [client run 37020424545](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37020424545) passed all four platform jobs; Windows/Linux bundles, the APK and unsigned iOS app were uploaded. Downloaded Android artifacts contain the actual Rust runtime for arm64-v8a, armeabi-v7a and x86_64; the downloaded iOS app contains ge4g_client.framework. The Windows bundle contains all four ABI exports, Flutter assets and embedded game configuration. No physical mobile touch-device, Windows interactive, Arch `makepkg`/Wayland or Apple signing acceptance is claimed by these automated checks.

A final client repair serializes restart against an in-flight frame decode, shows loading while replacing a session, frees codec resources on failures and keeps paused restarts visible. All eight Flutter tests and the rebuilt packaged Linux replay/frame smoke pass after this repair. The native-host platform workflow also repackages unsigned iOS with its Runner.app directory preserved.

### Final verified source and downloadable clients

Final native-host [run 37022204555](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37022204555) at source `e1497ce` passed **Linux, Windows, Android and iOS**. [Basement acceptance 37022204025](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37022204025) also passed the full Rust/CLI/window/no-window acceptance floor. The loading-clock regression is covered and the hosted packaged Flutter replay again matches exactly tick160, the complete headless snapshot and canonical pixels. All eight Flutter tests and analysis pass.

Artifacts: `ge4g-android-client` (APK, three native ABIs), `ge4g-basement-windows-embedded` (ZIP), `ge4g-basement-linux-embedded` (tarball, portable demo and replay evidence), `ge4g-ios-unsigned` (ZIP preserving Runner.app). Latest copies were downloaded into local `dist/GE4G-*` files. iOS app/framework paths were inspected after repackaging. APK is development-signed; Apple owner signing is still required for iOS device/store distribution. Physical touch devices, Windows human interaction and Arch installation/Wayland acceptance remain unclaimed.

## User acceptance — 2026-10-03

The user reports the Flutter 0.1 runners tested normally on PC, Linux and Android.
Record these platforms as user-confirmed; specific devices, Arch/Wayland installation
and Apple signing were not supplied. This closes the general runner play-check pending
status for those platforms. FlatLand changes require fresh regression evidence.

## Basement 0.2 — FlatLand alpha.1, 2026-10-03

First engine implementation, not completion of the full 0.2 design: map-wall/entity
separation, compact prefabs, pass/fixed/push bodies, direction helpers, cardinal
queued movement and deterministic chase/flee AI, typed conditions/actions and Lua 5.4,
HP/immunity/pickups/timers, sprite clips/Y sorting, exact versioned resume, local
component/prefab schemas and selective native observation. The Pac-Man-style demo
uses pinned CC0 pacman-canvas SVG/PNG sprites and in-repository synthesized WAV cues.

Flutter adds a manual portrait/landscape button, separate scrollable text popups,
four-voice device audio playback, and explicit v2 imports/embedded packages. ABI
functions remain v1; package schema/game schema v2 keeps older clients from accepting
unsupported data. Android's build hook now configures the C compiler for vendored Lua.

Evidence: 32 Rust tests, old/new authored replays, strict Clippy/fmt; 10 Flutter tests;
Linux release and both real embedded Flutter windows equal headless snapshots/frames.
Pac-Man canonical tick30 RGBA: `435f09d564ece74d46699508521ab86839c2f1f0c9ebbe84bc9353768be12f35`.
The v1 tick160 framebuffer golden remains unchanged. User acceptance of 0.1 PC/Linux/
Android runners is recorded above; alpha device rotation and audible playback are
separate pending checks. Source `b1dbf97` passed hosted
[engine acceptance](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37095556205)
and [all four client builds](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37095556206).
Downloaded packages contain native runtimes for all three Android ABIs, Windows DLL
exports and the iOS framework; both desktop games embed their client/runtime. Follow-up
packaging fixes pin LF source bytes across hosts to preserve content-bound saves and
the original CC0 SVG hashes. iOS remains unsigned and Android development-signed.

Remaining: dedicated quest/inventory systems, advanced melee/projectile combat,
elevation/bridge/camera/atlas features, RNG, serialized event scenes/choices/turn battle,
loop/music/positional audio, broader coverage and end-to-end token/repair benchmarks.
Read FLATLAND_AUTHORING for actual APIs rather than assuming every proposal is shipped.

### Final alpha build evidence

Native-host [client run 37096137150](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37096137150)
at `6fcf909` passed Linux, Windows, Android and iOS.
[Engine acceptance 37096318044](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37096318044)
at `2996369` passed every step, including v1/v2 replays, strict Rust checks, filetree,
real window comparison and no-window CLI tests. The latter commit synchronizes the
generated hash registry; gameplay and client code are identical.

Downloaded Windows/Linux games have identical game/control file bytes and metadata
values (JSON file-index key ordering may differ). All seven original SVG hashes match
the pinned sources. Hosted Linux evidence includes both FlatLand and v1 screenshots,
snapshots and results. A further real 1200-tick idle replay records ghost hits at ticks
257/437/617 with HP 2/1/0 and GAME OVER. A running Linux Flutter window was visually
inspected with the popup, portrait layout and button-selected landscape layout.

Downloads are in that client run: `ge4g-flatland-windows-embedded`,
`ge4g-basement-linux-embedded` (contains both games and both smoke evidence sets),
`ge4g-android-client` and `ge4g-ios-unsigned`. Local copies use `dist/GE4G-FlatLand-*`;
mobile imports `dist/flatland-pacman.ge4g`. Desktop bundles open the maze directly.

Static context sample: authored maze 7,979 bytes / five explicit actors plus shared
prefabs and 21 map rows; body schema 120 bytes versus scene schema 9,395; one-entity
observation 463 versus full snapshot 192,381. This measures command/source bytes,
not end-to-end task tokens. Token counts are unmeasured because the tokenizer vocabulary
could not be retrieved; conversations, repairs and asset prompts remain outside this sample.

## FlatLand 0.2.0-rc.1 playable candidate — 2026-10-05

Requested polish: automatic game-authored mobile presets inside .ge4g packages,
layout schema 2 with joystick-only profiles, conservative migration of untouched
generic controls and explicit preset reset, a single 44px HUD and separate settings
panel, maximum frame fitting independent of manual orientation and control geometry,
no permanent debug/footer/action rows, capped touch sizes and nonwrapping labels.

The maze now uses unchanged MIT bward2/pacman-js PNG strips (pinned commit
93f1bb08b8e8d4c511e7194b967b3d4fdf765681): four mouth frames, two ghost frames
per direction and frightened frames. Originals, frame coordinates, hashes and license
are packaged. Animation is selected by authoritative ticks/facing/timers and resumes
at its exact phase. New tick30 RGBA: a0be5c7fc85ca45c9dd8a54e0fa1bfc6c355a1e7e1d78a7ce5598b447fa43546.

33 Rust tests and strict checks, 13 Flutter tests/analyze, local Linux release passed.
Actual packaged Flutter/headless snapshots and RGBA agree. Frame geometry is tested
at narrow/wide viewports and twice the normal text scale. A real 1280x527 running window was captured at the reported screenshot dimensions: the game is 437x483 pixels, with no reserved touch region or persistent footer. Hosted builds passed on all four platforms.

Android's old CI debug keys were not retained and the older certificates differed.
The user explicitly authorized one final reinstall for rc.1, then stable updates.
The application ID stays dev.ge4g.ge4g_client; version code is 3. A permanent key is
backed up in an owner-only private Drive folder and its SHA256 is pinned in source.
Hosted builds are unsigned; delivery signs/verifies with scripts/sign_android.py.
Never regenerate the key or release-sign using temporary debug certificates.

This candidate completes the requested presentation/preset/animation polish, not
the entire broader proposed 0.2 contract. Dedicated quests/inventory, advanced combat
and serialized event scenes/cutscenes/turn combat remain in the full implementation plan.
Final [client run 37290624550](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37290624550) and [engine acceptance 37290624631](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37290624631) passed at build source `f22c7fd9f90a2810fa1598861ccdf5c98b7450ce`. Downloaded Linux/Windows bundles contain the exact same portable game/control bytes. Hosted v1 tick160 and FlatLand tick30 native/headless smoke passed. An actual old-alpha native save at tick30 resumes identically with the new runtime when its game source is retained; changed demo content requires New Game, without deleting the old save.

The delivered APK has application ID `dev.ge4g.ge4g_client`, versionCode 3 and all three native ABIs. Android v2/v3 signatures were cryptographically verified with pinned certificate SHA256 `d6d5ca948e5c1ed644478d7c4c3243efcfcbc30a196f03c39ef7af7118fc00d4`. The signing helper lets the PKCS12 key reuse the store password; reading the same password file twice incorrectly exhausted its stream. Physical installation of this candidate is not claimed. iOS remains unsigned and requires owner signing.

[Drive delivery: Demos / Basement 0.2 FlatLand / 0.2.0-rc.1](https://drive.google.com/drive/folders/13a3oRUOBWZ-RCNHaBaXEnDLbtfS13TVR) contains the signed Android APK, embedded Windows/Linux games, portable game, unsigned iOS app, actual landscape preview, Korean instructions and checksum/provenance manifest. All eight uploaded filenames, byte sizes and download availability were verified. Signing secrets are excluded; their separate backup remains owner-only.

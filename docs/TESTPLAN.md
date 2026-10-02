# Basement acceptance plan

Tests prove behavior and artifacts, not just command success. Headless verification requires no GPU, display, audio device or interactive input. The canonical Linux checks from a clean checkout are:

```sh
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --locked -p ge4g-cli -- validate examples/basement_demo
cargo run --locked -p ge4g-cli -- test examples/basement_demo
python3 scripts/filetree.py lint
```

## Automated coverage

The workspace currently has 24 behavioral tests: 3 core, 2 project, 6 runtime, 2 renderer, 2 native-client ABI and 9 CLI integration tests.

| Requirement | Executable evidence |
| --- | --- |
| Player movement | Journey checkpoint tick 20: player (64,40), changed from (24,40) |
| Wall blocks player / observable collision | Tick 50: x=84, max_x=84; `collision_started` target wall. Runtime high-speed test proves no tunneling |
| NPC interaction | Tick 96: `interaction` target npc, `demo.npc.spoken=true`; held key test proves edge behavior |
| Door changes scene | Tick 147: room_b at named entry (24,40), trigger event; high-speed test proves crossed triggers fire |
| Persistent state changes | Journey checks both declared bool keys |
| Atomic save and restart | Project tests save to a temp directory, restart/load and compare; CLI test verifies one file and no temporary-file leak |
| Invalid save handling | Corrupt JSON, wrong types, future version and missing persistent keys fail; failed type load does not partially modify store |
| Real PNG capture | CLI integration decodes RGBA PNG and checks player/wall/door pixels and frame hash; texture/camera and debug captures also checked |
| Deterministic repeat | Project test and CLI integration compare full final snapshots and ordered event history from two clean runtimes |
| Exact frame golden | Final RGBA SHA256 `38cc4352e7160dcf9104cbe19acd709e36d8165989b3c99d8b6611f0f76e932b`; reference `examples/basement_demo/golden/room_b.png` |
| Versioned CLI | Schema fields, JSON argument errors, useful entity/file context, failed assertion exit 1, I/O/data error exit 2 |
| Bounded/queryable trace | Runtime buffer overflow count; CLI kind filter preserves event order |
| Validation / diagnosis | Future scene version and absent scene/texture references rejected; diagnosis returns four explicit checks |

The demo `test` command checks seven authored checkpoints across 160 ticks, complete replay repeatability, final frame golden, save/reload and actual PNG output. A deliberately changed expectation proves failed assertions return a failure, not a success report.

## Window adapter verification

Build the default CLI, then with a local display:

```sh
cargo build --locked -p ge4g-cli
python3 scripts/interactive_smoke.py
```

On Linux with Xvfb:

```sh
cargo build --locked -p ge4g-cli
xvfb-run -a python3 scripts/interactive_smoke.py
```

This opens a real minifb window, presents CPU frames, plays the 160-tick replay and compares its snapshot/events with a headless run. It does not constitute human testing. CI runs this plus a build/test with the interactive feature excluded:

```sh
cargo clippy --locked -p ge4g-cli --no-default-features --all-targets -- -D warnings
cargo test --locked -p ge4g-cli --no-default-features
```

## Actual human acceptance — pending

A human must launch `cargo run --locked -p ge4g-cli -- run examples/basement_demo`, move with WASD/arrows, collide with the wall, talk to the NPC using E, enter the door, save with F5 and reload with F9. The title should retain `NPC:true` and `RoomB:true`. Close and relaunch with `--load examples/basement_demo/save.json` to check persistent values again.

Record tester/date/result and any issues in RELEASE_NOTES when performed. Automated Xvfb evidence does not earn the human execution contribution or complete ILCX™ evaluation. See RELEASE_NOTES for the current verified milestone and remaining human sign-off.

## Flutter client acceptance

Use Flutter 3.47.6 and stable Rust. Build the portable demo before Flutter tests:

```sh
cargo build --locked -p ge4g-cli
python3 scripts/pack_game.py examples/basement_demo --game-id demo.basement --out dist/basement-demo.ge4g
cd client
flutter pub get
flutter analyze
flutter test
flutter build linux --release
cd ..
python3 scripts/bundle_desktop.py --client client/build/linux/x64/release/bundle --game dist/basement-demo.ge4g --platform linux --out dist/ge4g-basement-linux
xvfb-run -a python3 scripts/client_smoke.py dist/ge4g-basement-linux
```

Flutter tests execute the actual bundled Rust FFI replay, compare canonical frame SHA256, save/reload and live profile/remapping input release through the native player, reject invalid/traversal/hash-corrupt packages, prove settings/save game isolation, reload external JSON while retaining invalid edits, and exercise real button chords, joystick ownership and cancellation widgets. The embedded desktop smoke opens a real Flutter window and compares its full Rust snapshot and decoded frame to CLI/headless.

Hosted client CI additionally builds Windows embedded ZIP, Android APK and unsigned iOS app. Build results do not constitute human touch-device tests, Windows interaction tests or `makepkg`/Wayland acceptance on Arch. Those remain explicit human/platform checks.

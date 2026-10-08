# GE4G 0.3.0 — Pentomino alpha development

Current main version: 0.3.0-alpha.1; Flutter 0.3.0-alpha.1+9.
See [Pentomino status](../docs/pentomino/STATUS.md) for implemented scope and next work.
The FlatLand material below is the inherited implementation/released 0.2 baseline.
Its historical versions, delivery instructions and evidence are not a new alpha release.
Existing acceptance checks remain regressions; also run
`cargo test --locked -p ge4g-pentomino` for the experimental scalar host.
Android/Windows remain the client verification matrix. No alpha binary is published here.

## GE4G / GameEngineForGPT client

Flutter presentation, authoritative native Rust Basement simulation. Android clients import `.ge4g` games; Windows game distributions embed this client, the native runtime and a game package and start directly.

Active platforms through FlatLand 0.2: **Android and Windows**. iOS, macOS and Linux/Arch support work is suspended until 0.3.0 development begins; historical builds and platform source remain preserved.

Build from the repository checkout with Flutter 3.47.6 (Dart 3.13), stable Rust and your platform's Flutter prerequisites. The `ge4g_native` code-assets hook builds Cargo and bundles its library automatically. No separate CLI is required on a player's device.

See [client usage, controls and distribution](../docs/CLIENT.md), [launcher philosophy](../docs/LAUNCHER_PHILOSOPHY.md) and [platform CI](../.github/workflows/client.yml).

```sh
cargo build --locked -p ge4g-cli  # developer validator
python3 scripts/pack_game.py examples/basement_demo --game-id demo.basement --out dist/basement-demo.ge4g
cd client
flutter pub get
flutter analyze
flutter test
flutter build windows --release  # Windows host
# Or: flutter build apk --release
```

The mobile library has an Import action; desktop release packaging must use `scripts/bundle_desktop.py`. Running a development build without `client_mode.json` opens the library.

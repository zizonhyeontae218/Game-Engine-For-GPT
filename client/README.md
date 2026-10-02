# GE4G / GameEngineForGPT client

Flutter presentation, authoritative native Rust Basement simulation. Android/iOS clients import `.ge4g` games; Windows/Arch game distributions embed this client, the native runtime and a game package and start directly.

Build from the repository checkout with Flutter 3.47.6 (Dart 3.13), stable Rust and your platform's Flutter prerequisites. The `ge4g_native` code-assets hook builds Cargo and bundles its library automatically. No separate CLI is required on a player's device.

See [client usage, controls and distribution](../docs/CLIENT.md), [launcher philosophy](../docs/LAUNCHER_PHILOSOPHY.md) and [platform CI](../.github/workflows/client.yml).

```sh
cargo build --locked -p ge4g-cli  # developer validator
python3 scripts/pack_game.py examples/basement_demo --game-id demo.basement --out dist/basement-demo.ge4g
cd client
flutter pub get
flutter analyze
flutter test
flutter build linux --release  # or windows / apk / ios --no-codesign
```

The mobile library has an Import action; desktop release packaging must use `scripts/bundle_desktop.py`. Running a development build without `client_mode.json` opens the library.

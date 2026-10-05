# GE4G Basement client

The product name is **GE4G / GameEngineForGPT**. Flutter presents the authoritative Rust Basement world and its CPU framebuffer. Design uses paper, black and acid lime, monospaced typography, square controls, heavy borders and solid offset shadows.

## Player flows

**Mobile (Android/iOS):** install the GE4G client → **게임 불러오기 / Import** → select a `.ge4g` file → validate and load → play. Imported games appear in the library and reopen with their own saved controls and save file. Games are data packages; they never supply native libraries. Android file selection uses the system document picker; iOS uses Files.

**Desktop (Windows/Arch Linux):** open the executable in the game's complete bundle. Its adjacent `client_mode.json` identifies the included game and disables the library flow. Every shipped game includes Flutter assets, plugins, the Rust runtime and its game data. The developer CLI remains a development/verification tool.

Joystick/WASD/arrows move. Default **Z/E = interact**, **X = x**, **C = c**, **Space = space**. X/C/Space are versioned named actions producing `action_pressed`/`action_released` runtime events. The original Basement demo implements movement and NPC interaction; these default buttons do not invent jump or combat behavior. A game can remap a button to an existing built-in action (for example Space → interact) or consume named actions when its runtime behavior is extended.

The compact in-game HUD contains score/lives/power, pause, manual rotation and a **게임 메뉴** button. Its separate settings sheet contains save, load, fresh restart, debug details, profile selection, packaged-preset reset and **조작 편집 / Control Lab**. Inputs release on profile changes, valid edits, cancellation, focus loss and backgrounding. The game pauses while backgrounded; resuming preserves the prior pause state and never fast-forwards through background time.

FlatLand adds a **rotation button in the black title bar**: it selects portrait
or landscape explicitly. Mobile locks to portrait-up or landscape-left; sensors and
desktop window resizing do not select a layout. Text/dialogue appears in a separate
scrollable popup which pauses client ticking until **확인 / CONTINUE**. The title-bar
menu reopens the latest message or toggles sound. Cue playback uses four voices;
headless simulation remains independent of device audio.

## Separate control files

Each game uses a stable `game_id`, independently of its package hash or display name. Updating a package retains the game's controls and save. Installed content is immutable per content hash, so an imported update cannot replace the files of a running game.

Under the platform application support directory:

```text
library.json
games/<game_id>/<sha256>/bundle.json, game/, controls/
settings/<game_id>/controls/layouts.json
settings/<game_id>/controls/bindings.json
saves/<game_id>/save.json
```

`layouts.json` owns active profile, profile names, joystick and touch button geometry. `bindings.json` owns **game-specific** profile button/joystick/keyboard mappings. The save owns gameplay state. Neither input file is part of a game save. Default/reference files are in `client/assets/` and `.ge4g` packages ship game-specific initial copies. Existing user files take priority on update.

Coordinates `x`,`y` are normalized centers in the available touch viewport. Joystick `size`, button `width`,`height` are fractions of its shorter dimension. Hit targets clamp to the viewport and at least 48 logical pixels; joystick is normally at least 96. The game fits the full viewport below a single 44-pixel HUD. Controls overlay without reserving game space, independently of the selected manual orientation. Tall portrait displays put the fitted game at the top, leaving spare space for controls; wide displays center it. Button labels stay on one line. Joystick is single-pointer owned and supports simultaneous independent button touches.

```json
{
  "schema_version": 1,
  "active_profile": "default",
  "profiles": [{
    "id": "default", "name": "DEFAULT",
    "joystick": {"x": 0.18, "y": 0.76, "size": 0.38, "dead_zone": 0.18},
    "buttons": [{"id": "z", "label": "Z", "x": 0.77, "y": 0.73, "width": 0.19, "height": 0.17}]
  }]
}
```

```json
{
  "schema_version": 1,
  "game_id": "my.game",
  "profiles": {
    "default": {
      "joystick": {"left": ["left"], "right": ["right"], "up": ["up"], "down": ["down"]},
      "buttons": {"z": ["interact"]},
      "keys": {"KeyZ": ["interact"], "Space": ["space"]}
    }
  }
}
```

Layouts accept schema 1 (1..16 buttons) or schema 2 (0..16 buttons, including joystick-only presets); bindings remain schema 1. Both require matching profile/button IDs and correct game identity. Empty action arrays disable a binding; multiple actions form a chord. Built-in actions are left/right/up/down/interact. Named action identifiers are ASCII letters/digits/underscore/dot/hyphen, max 64 characters; the runtime accepts up to 32 held named actions. Up to 16 profiles and 16 touch buttons per profile are supported.

Choose DEFAULT or LEFT HAND during play. Control Lab provides joystick/button position and size sliders, button mapping input, profile cloning and separate JSON editors. Slider release or **JSON 적용** commits valid changes immediately while the game continues ticking. Game keys are released while typing in the editor. Invalid JSON keeps the last valid running configuration and displays the error. Desktop external edits are watched/debounced, with a one-second polling fallback on platforms where watchers are unavailable. File paths are shown in Control Lab. Each JSON file is atomically replaced; a two-file edit is validated together before publication to the running game.

## Make a portable game

Requires Python 3.11+ and a built developer validator:

```sh
cargo build --locked -p ge4g-cli
python3 scripts/pack_game.py examples/basement_demo \
  --game-id demo.basement --version 0.1.0 --out dist/basement-demo.ge4g
```

`.ge4g` is a deterministic ZIP containing `bundle.json`, `game/` and `controls/`. The versioned manifest includes engine ABI 1, stable ID, name/version and SHA256 for every data file. Imports check paths, duplicate/case collisions, symlinks, file sizes, hashes, control schemas and actual Rust project validation before library activation. Limits: 64 MiB compressed, 256 MiB extracted, 16 MiB per entry, 4096 entries. These hashes detect corruption; packages are not publisher-signed.

Schema 2 projects produce bundle schema 2 with `game_schema:2`; native ABI remains 1
and controls remain schema 1. The new client accepts both game versions; older clients
reject the unsupported package version. Example:

```sh
python3 scripts/pack_game.py examples/flatland_pacman \
  --game-id demo.flatland.pacman --version 0.2.0-rc.1 --out dist/flatland-pacman.ge4g
```

To supply authored initial controls, use `--layouts path/layouts.json --bindings path/bindings.json`. Bindings are stamped with the specified game ID; existing player overrides remain preserved. Game-local `controls/layouts.json` and `controls/bindings.json` are automatically packaged ahead of generic app defaults, without extra CLI flags. Untouched legacy generic controls migrate to a new schema-2 game preset; edited mappings/layouts remain and can explicitly reset from the game menu.

## Build clients and embed desktop games

Pinned Flutter **3.47.6**, Dart **3.13**, stable Rust. Build from this repository; the native hook compiles `ge4g-client` and Flutter bundles its code asset. Install the platform's Flutter prerequisites; `flutter doctor` checks them.

```sh
cd client
flutter pub get
flutter analyze
flutter test
flutter build linux --release
# Windows host with Visual Studio Desktop development with C++:
flutter build windows --release
```

Linux/Arch build prerequisites: `flutter`, `rust`, `clang`, `cmake`, `ninja`, `pkgconf`,
`gtk3`, GStreamer and base-plugin development headers. Runtime playback needs
`gstreamer`, `gst-plugins-base` and `gst-plugins-good` (Ubuntu: `gstreamer1.0-plugins-base`
and `gstreamer1.0-plugins-good`); GTK3 and the system graphics stack present the window.
Wayland/X11 follow the standard Flutter GTK runner. Windows uses the Flutter release
folder including its plugins and runtime DLLs; install the Microsoft Visual C++ runtime
if the target system lacks it.

From repository root:

```sh
python3 scripts/bundle_desktop.py \
  --client client/build/linux/x64/release/bundle \
  --game dist/basement-demo.ge4g --platform linux --out dist/ge4g-basement-linux
# Windows:
python scripts/bundle_desktop.py \
  --client client/build/windows/x64/runner/Release \
  --game dist/basement-demo.ge4g --platform windows --out dist/ge4g-basement-windows
```

Outputs a complete directory and `.tar.gz`/`.zip`. Unpack and run `ge4g_client`/`ge4g_client.exe`; no engine installation or import prompt. Existing output directories are rejected. For Arch packaging copy `packaging/arch/PKGBUILD` beside `dist/ge4g-basement-linux.tar.gz` and run `makepkg -si`; it installs the complete bundle to `/opt` and a desktop launcher. For another game, adjust package name/description and the archive name together.

Android requires Java 21, Android SDK/NDK (Flutter chooses its pinned NDK) and Rust targets:

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cd client && flutter build apk --release
```

CI now builds an unsigned APK. The user authorized one final reinstall for rc.1 because older runner-generated debug keys were lost. Every delivered release from rc.1 onward uses the same preserved key, package ID `dev.ge4g.ge4g_client` and increasing version code (rc.1: 3). The public certificate SHA256 is pinned in `client/android/signing-certificate.sha256`. Restore the encrypted PKCS12 and password from the owner's private `GE4G Private Signing` Drive backup; never generate another key. Do not share that folder or put key/password files in Git. `scripts/sign_android.py` signs and verifies the pin; `--previous-apk` additionally rejects mismatched prior signatures. An environment-configured Gradle release key is also checked against the same fingerprint. Never deliver the unsigned CI APK as an installable build.

iOS requires macOS/Xcode and `rustup target add aarch64-apple-ios`. `flutter build ios --release --no-codesign` verifies the app build. Device installation/App Store release requires the owner's Apple team/provisioning/signature. Simulator builds additionally require the matching `aarch64-apple-ios-sim` or `x86_64-apple-ios` Rust target.

## Verification

`cargo test --workspace` compares the native ABI's full replay snapshot/events and exact RGBA pixels to the headless World. Flutter tests cover actual FFI replay/frame/save/load, validated import rejection, per-game isolation, persistence and real touch widget chords/cancellation. Build a demo `.ge4g` before running them.

`xvfb-run -a python3 scripts/client_smoke.py dist/ge4g-basement-linux` opens the **packaged Flutter executable**, autoloads its game, completes all 160 replay ticks, compares its complete snapshot to the CLI and compares its decoded real frame to the canonical hash. Evidence is written to `artifacts/client-smoke/`. No substituted/synthesized captures are used.

`.github/workflows/client.yml` builds Linux, Windows, Android APK and unsigned iOS on native hosts and uploads artifacts. Automated builds are separate from human mobile/device and Arch installation acceptance; record those honestly in release notes.

## Demo delivery

Deliver demo files to the user's connected Google Drive under
`Demos/<engine version>/<build version>/`, starting with
`Demos/Basement 0.2 FlatLand/0.2.0-alpha.1/`. Reuse existing verified folders and retain
older versions. Include the portable `.ge4g`, available platform packages, installation
instructions and checksum manifest. Read back file names, parents and byte counts
before giving the user the Drive folder/file links; internal workspace paths do not
provide usable downloads for this user. Preserve existing sharing permissions.

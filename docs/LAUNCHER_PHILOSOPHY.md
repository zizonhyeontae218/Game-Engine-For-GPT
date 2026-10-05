# GE4G launcher philosophy

This is the user's standing design direction for every future launcher, established 2026-10-02. It supersedes the original Basement exclusion of mobile packaging; the simulation constraints still apply.

Through FlatLand 0.2, active client support and distribution cover **Android and Windows only**. iOS, macOS and Linux/Arch client work is suspended until 0.3.0 development begins. Existing source and historical downloads are retained. Linux-hosted engine/headless checks and Android builds remain development infrastructure.

Mobile is a reusable **client → Import → load Basement game → play** flow. Games are portable versioned data packages, not downloaded executables. The installed client supplies the authoritative native engine. Imported versions and game-local saves/controls live in application storage.

Desktop games for **Windows** ship with the Flutter client, native runtime and game package embedded. They open their embedded game directly. Players do not install Rust, Flutter, an engine or a development CLI. Library mode is useful for development, but is not the default distribution experience for a desktop game.

Flutter owns presentation, import, lifecycle, keyboard/touch normalization and control editing. Rust owns simulation, collision, scene lifecycle, saves, events and canonical CPU RGBA frames. Both surfaces consume the same engine. Flutter must not substitute a second implementation or a subprocess CLI bridge.

The client uses digital brutalism: flat paper/ink colors, acid accents, hard borders, square surfaces, heavy type and explicit information hierarchy. Keep product flows readable; ABI/build details belong in developer documentation.

Mobile ships with a joystick plus Z, X, C and Space. Layouts and profiles are versioned JSON. Game mappings are a separate JSON file keyed by stable game identity, and saves are a separate concern. Users can switch profiles while the game runs, edit JSON in the client, or edit the files externally and see valid changes applied live. Clear held input before applying changes; retain the last valid profile and show diagnostics on invalid edits.

Input is simultaneous and multi-touch. Touch cancellation, focus loss, profile replacement and app backgrounding must not leave an action held. Running in the background pauses the host clock; resume must not fast-forward gameplay through the elapsed absence.

Builds and game bundles must include the actual native runtime and all game assets. Packaging must fail clearly when required files are missing. Preserve explicit versions and game identity across updates; do not mix settings or saves between games.

## FlatLand orientation and text

All future clients expose an explicit portrait/landscape toggle. Sensor-driven
orientation changes must not select the gameplay layout. Release held inputs before
changing layout. Mobile requests one fixed device orientation per selected mode.
Dialogue/text uses a separate modal surface with scrolling for short screens.

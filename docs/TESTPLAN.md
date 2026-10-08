# GE4G 0.3.0 — Pentomino alpha development

Current main version: 0.3.0-alpha.1; Flutter 0.3.0-alpha.1+9.
See [Pentomino status](pentomino/STATUS.md) for implemented scope and next work.
The FlatLand material below is the inherited implementation/released 0.2 baseline.
Its historical versions, delivery instructions and evidence are not a new alpha release.
Existing acceptance checks remain regressions; also run
`cargo test --locked -p ge4g-pentomino` for unchanged P1 plus typed P2 contracts,
`cargo test --locked -p ge4g-pentomino-legacy` for the compatibility boundary,
and `python3 scripts/check_p2_boundaries.py` for P2 dependency/preservation checks.
Android/Windows remain the client verification matrix. No alpha binary is published here.

## FlatLand 0.2.0 acceptance

Supported clients: Android and Windows only. Linux-hosted Rust/headless and Android
cross-compilation are infrastructure. Do not build/test suspended clients until0.3.
Historical evidence/counts live in docs/releases and completed ExecPlans.

## Engine floor

```sh
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --locked -p ge4g-cli -- test examples/basement_demo --json
cargo run --locked -p ge4g-cli -- test examples/flatland_pacman --json
cargo run --locked -p ge4g-cli -- test examples/flatland_signal_yard --json
cargo run --locked -p ge4g-cli -- test examples/flatland_harbor --json
python3 scripts/release_consistency.py --self-test
python3 scripts/filetree.py lint
cargo clippy --locked -p ge4g-cli --no-default-features --all-targets -- -D warnings
cargo test --locked -p ge4g-cli --no-default-features
```

Preserve view/simulation separation, billboard anchors, feet/contact ordering,
solid footprints/no roof access, South-only validation, movement priority/reversal,
Pac-Man continuous-grid behavior, persistent HP/PP, RNG rollback, exact-once battle
damage/items/PP/rewards, event return, scoped camera and conservative save validation.
The rc4/rc5 integration suites remain executable regressions. Final tests cover
explicit speaker, metadata fallback, absent header, line/actor/name resume and typed
content-revision failure distinct from corruption. Replay/save golden commands also
capture canonical PNGs; command success alone is insufficient evidence.

## Windows Flutter and embedded acceptance

Use Flutter3.47.6. Prepare all four portable demos with scripts/pack_game.py using
version0.2.0. On Windows run flutter analyze, flutter test, flutter build windows
--release, bundle_desktop.py for each example, and windows_utf8_regression.py
--prepare then --launch. The client workflow is the exact reproducible matrix.

Keep actual FFI/canonical pixels, live controls/profile release, portrait/landscape,
ordinary dialogue, bubble taps/expiration, battle FX/red flash/HP interpolation,
feedback lock and save UI covered. Final additions exercise same-game two-revision
update, exact old-save archival/fresh start, compatible duplicate resume, native
open failure/corruption rollback, unchanged controls, deletion/reinstall/full deletion,
persisted C-A-B reorder/update order, missing-index repair and confirmation UI.

Windows UTF-8 checks cover Korean TOML/JSON/JSON5/resources and non-ASCII package,
import, application-data and embedded executable paths. Real portrait/landscape
captures assert three UI bubble taps, same top/depth simulation, three blocked
building attempts, battle phases and restored persistent state. Embedded mode must
retain allow_library:false and never show import/delete/reorder/manager controls.
CI graphics capture is explicitly muted; this is not physical audio acceptance.

## Android final packaging

Build release APK with arm64-v8a, armeabi-v7a and x86_64 native runtimes. Confirm
Flutter0.2.0+8, versionCode8, applicationId dev.ge4g.ge4g_client. Sign with the
preserved private key via scripts/sign_android.py and --previous-apk pointing to
rc5. Verify certificate against pinned SHA256 and actual rc5 APK. Never regenerate
keys or upload private material. Preserve all historical binaries.

## Human / publication evidence

User confirmed rc5 engine/cutscene physical acceptance and authorized this focused
finalization. Keep engine behavior frozen. Record actual final automated results and
rendered review separately from human testing; new management UI is not physically
verified by CI. Publish final only after every finalization test/build/signing gate
passes. Deliver new0.2.0 Drive artifacts, instructions, provenance and checksums.
3D/pixelization, other building directions, external-storage relocation and online
updates are future scope.

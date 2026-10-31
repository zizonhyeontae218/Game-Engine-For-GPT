# Commands, timing, exit status and logs

All commands ran with cwd `C:/Users/gokon/Documents/personal_Project/GameEngine4GPT`. No prior endurance/full suite was rerun.

| Operation | Exact command shape | Duration / exit | Raw evidence |
|---|---|---|---|
| Rust ARM64 target setup | `C:/Users/gokon/.cargo/bin/rustup.exe target add aarch64-linux-android` | 5.323s / 0 | `rust-target-add.log`, `rust-target-add.meta.txt` |
| P4 ARM64 cdylib | NDK 28.2 env (`CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER`, `CC_aarch64-linux-android`, `AR_aarch64-linux-android`) then `cargo.exe build --locked --release -p ge4g-pentomino-native --target aarch64-linux-android --target-dir client/.dart_tool/p4_harness/cargo` | 99.447s / 0 | `cargo-android-arm64.log`, `cargo-android-arm64.meta.txt` |
| Original client analysis | `C:/Users/gokon/development/flutter/bin/flutter.bat analyze lib/p4_device_harness.dart` from `client` | 4.585s / 1 | `flutter-analyze.log`; blocked before analysis because Flutter 3.44.4/Dart 3.12.2 is below the client SDK constraint `>=3.13.0` |
| Isolated wrapper setup | `flutter.bat create --platforms android --org dev.ge4g.ge4g_client --project-name p4harness client/.dart_tool/p4_device_app` | 5.564s / 0 | `flutter-create-isolated.log`, `.meta.txt` |
| Harness analysis | `flutter.bat analyze lib/main.dart` in isolated wrapper | final 4.153s / 0 | `isolated-flutter-analyze-final.log`, `.meta.txt`; earlier lint/compiler repair logs are retained |
| Flutter APK first build | `flutter.bat build apk --debug --target-platform android-arm64 --target lib/main.dart` | 95.155s / 0 | `flutter-build-apk.log`, `.meta.txt` |
| APK signature inspection | `apksigner.bat verify --print-certs ge4g-p4-harness-debug.apk` | exit 0 | `apksigner-verify.log` |
| Safe install | `adb.exe -s R3CW208CWZH install --user 0 ge4g-p4-harness-debug.apk` | 9.298s / 0 | `adb-install.log`, `.meta.txt` |
| Cold launch | `adb.exe -s R3CW208CWZH shell am start -W -n dev.ge4g.ge4g_client.p4harness/.MainActivity` | host 1.695s / 0; Android TotalTime 1373ms | `adb-launch.log`, `.meta.txt` |
| App-only runtime log | `adb.exe -s R3CW208CWZH logcat --pid=6726 -d -v threadtime` | exit 0 | `device-logcat-recovered.raw.log` |
| Lifecycle input test | `adb shell input touchscreen motionevent DOWN 540 2180`; `adb shell input keyevent HOME`; `adb shell am start -W ...`; `adb shell input touchscreen motionevent UP 540 2180` | 0.604s; all exits 0 | `adb-lifecycle-resume.log`, `device-lifecycle-logcat.raw.log`, `device-lifecycle.meta.txt` |
| Screenshot | `adb.exe -s R3CW208CWZH exec-out screencap -p` with binary stdout redirected by `Start-Process` | exit 0 | `harness-pass.png`; SHA-256 `af0b335645775f2b991a831ed432008c72108b4ba62610163ad68b5908164747` |

## Evidence-collection corrections

- The first marker waiter used PowerShell's reserved `$PID` variable, captured host PID 7048 and timed out after 30.190s. The app itself had already passed. The same live app process was recovered as device PID 6726 and its app-scoped log contained the result marker at 23:27:49.458. No reinstall or main-scenario rerun occurred.
- The first screenshot command had invalid `cmd.exe` quoting. `Start-Process -RedirectStandardOutput` then captured the PNG successfully. Neither issue changes the device result.

First native + Flutter build time was 194.602s. Rust-target setup and isolated-wrapper generation were another 10.887s. Deliberate device scenario waiting was approximately 1.665s, excluding installation and the failed log collector timeout.

# Commands and provenance

2026-10-18 Asia/Seoul start; long runs continue on2026-10-19. PowerShell7 host.
Repository was not found in Documents/Downloads Git metadata; a fresh clone and
`git worktree add ../p4-validation -b qa/p4-remaining-20261018 origin/qa/p4-independent-final-20261016`
preserve other worktrees. Both supplied remote tips are unchanged. GitHub CLI's
existing keyring authentication was used; credentials are not retained here.

## Windows A and endurance G

Rust installed with official rustup-init `-y --no-modify-path --profile minimal`.
Existing VS2026/MSVC14.50.35717 used. Cargo.lock unchanged.

```powershell
$env:PATH="$env:USERPROFILE\.cargo\bin;"+$env:PATH
cargo build --locked -p ge4g-pentomino-native -j 2
$env:GE4G_LIB="$PWD\target\debug\ge4g_pentomino_native.dll"
python evidence/p4-remaining-20261018/native_verify.py smoke --out evidence/p4-remaining-20261018/windows-native.json
# Separate processes, started about30 seconds apart:
python evidence/p4-remaining-20261018/native_verify.py endurance --mode headless --out evidence/p4-remaining-20261018/endurance-headless.json
python evidence/p4-remaining-20261018/native_verify.py endurance --mode linked --out evidence/p4-remaining-20261018/endurance-linked.json
pwsh -File evidence/p4-remaining-20261018/monitor.ps1
```

Each mode requires7200 monotonic seconds, performs real attacks against a separate
fixed-damage oracle every step (HP10000/damage1), 2Core ticks/logical step,
save/restore every64, Camera/View every127 with failure atomicity, detach/reinstall
every509, close/create/restore every2003, and Linked RGBA SHA256 validation every16.
Every11th input is empty; no-idle-only workload. New battle starts after victory.
Histories are exercised beyond256 entries. Every30seconds logs RSS, trailing4096
operation latency p50/p95/p99, counts, errors and canonical saved-state SHA256.
Each process has768MiB RSS cap and maximum50 iterations/sec; host supervisor stops
these QA trees if available memory remains below512MiB for two samples. No global
security setting is changed. Debug timings are not game performance claims.

## Android B

Existing NDK28.2.13676358, Clang19, Android SDK and existing
Medium_Phone_API_36.1 AVD (x86_64, API36.1). No physical device was attached.
WHPX acceleration reported installed/usable. Emulator invoked via hidden
Start-Process, with `-no-window -no-audio -no-snapshot -read-only -memory 1024
-cores 2 -gpu swiftshader_indirect -port 5580`. Its actual emulator process was
terminated using `adb -s emulator-5580 emu kill` after tests. No wipe, APK install,
uninstall, app-data operation, signing-key change, or AVD configuration edit.

```powershell
rustup target add x86_64-linux-android
$ndk="$env:LOCALAPPDATA\Android\Sdk\ndk\28.2.13676358\toolchains\llvm\prebuilt\windows-x86_64\bin"
$env:CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$ndk\x86_64-linux-android26-clang.cmd"
$env:CC_x86_64_linux_android="$ndk\x86_64-linux-android26-clang.cmd"
$env:AR_x86_64_linux_android="$ndk\llvm-ar.exe"
cargo build --locked -p ge4g-pentomino-native --target x86_64-linux-android -j 2
& "$ndk\x86_64-linux-android26-clang.cmd" -std=c11 -Wall -Wextra -Werror -g evidence/p4-remaining-20261018/android_consumer.c -Icrates/ge4g-pentomino-native/include -Ltarget/x86_64-linux-android/debug -lge4g_pentomino_native '-Wl,-rpath,$ORIGIN' -o ../android-consumer
# adb mkdir/push/chmod confined to /data/local/tmp/ge4g-p4-qa-20261018
$env:GE4G_ADB="$env:LOCALAPPDATA\Android\Sdk\platform-tools\adb.exe"
$env:GE4G_ANDROID_LIB="$PWD\target\x86_64-linux-android\debug\libge4g_pentomino_native.so"
python evidence/p4-remaining-20261018/android_verify.py
```

Host Python sends JSON to adb shell -T; the Android native executable calls and
frees actual C ABI responses. Host assertions check full frames and combat/save
results. Its single Android native thread does not test Android thread affinity.
No P4 Flutter integration exists in the current client; app/UI is NOT TESTED.

## Linux C/D/E and F limitation

WSL binary reports WSL not installed; no Linux distro is available locally.
Standard GitHub-hosted ubuntu-22.04 runners on this public repository execute
`.github/workflows/p4-remaining-qa.yml`, with2 Cargo jobs and35-minute safety cap.
See `linux_gates.sh` for exact commands, raw set-x logs, versions and hashes.
Rust official sanitizer guidance:
https://doc.rust-lang.org/unstable-book/compiler-flags/sanitizer.html
Nightly Rust sanitizer + rebuilt std instrument Rust dependencies/tests including
the actual exported FFI functions. Vendored C Lua and system libc are not claimed
to be fully sanitizer instrumented. TSan tests use8 distinct thread-local session
tables and128 unique global handles; no shared session is used as valid input.
Valgrind installs matching libc6-dbg and uses the original CI ELF, existing
ownership_stress.c/requests, plus128 sessions reclaimed on pthread exit. No custom
suppression is supplied; lost blocks are distinguished from still reachable.

F remains NOT TESTED: tool interfaces do not allow restricting a newly spawned
agent's source/tool read access. No ordinary out-of-tree test is relabeled blind.
Existing Drive review was reused instead of repeating historical searches.
Existing broad acceptance workflows automatically triggered by branch pushes were
cancelled; unchanged169/169 workspace and other completed checks are prior evidence,
not added to new execution counts.

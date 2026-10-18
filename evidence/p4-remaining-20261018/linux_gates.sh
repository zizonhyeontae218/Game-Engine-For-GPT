#!/usr/bin/env bash
set -euo pipefail
qa=evidence/p4-remaining-20261018
out="$RUNNER_TEMP/p4-linux-$1"
mkdir -p "$out"
exec > >(tee "$out/driver.log") 2>&1
set -x
date --iso-8601=seconds
uname -a
cat /etc/os-release
rustc -vV
clang --version
valgrind --version
git rev-parse HEAD
sha256sum Cargo.lock
cp "$qa/threaded_ffi.rs" crates/ge4g-pentomino-native/tests/qa_threaded_ffi.rs
if [[ "$1" == address || "$1" == thread ]]; then
  rustup toolchain install nightly --profile minimal --component rust-src
  rustc +nightly -vV
  export CARGO_TARGET_DIR="$RUNNER_TEMP/target-$1" CARGO_BUILD_JOBS=2
  export RUSTFLAGS="-Zsanitizer=$1 -Cforce-frame-pointers=yes" RUSTDOCFLAGS="-Zsanitizer=$1"
  export ASAN_OPTIONS=detect_leaks=1:detect_stack_use_after_return=1:halt_on_error=1
  export TSAN_OPTIONS=halt_on_error=1
  cargo +nightly test --locked -Zbuild-std --target x86_64-unknown-linux-gnu -p ge4g-pentomino-gameplay -p ge4g-pentomino-native --tests -- --nocapture
  find "$CARGO_TARGET_DIR/x86_64-unknown-linux-gnu/debug/deps" -maxdepth 1 -type f -executable -exec sha256sum '{}' \; > "$out/binaries.sha256"
else
  # Original CI ELF, not a new replacement build. Authenticated gh avoids token output.
  gh api repos/zizonhyeontae218/Game-Engine-For-GPT/actions/artifacts/11823886424/zip > "$out/original-artifact.zip"
  echo 'af98895c3baf92fb502b13951b056d85ad558f5eb4c915b6d5177c41cbf8600c  '"$out/original-artifact.zip" | sha256sum -c -
  unzip -q "$out/original-artifact.zip" -d "$out/artifact"
  mkdir "$out/sdk"
  unzip -q "$out/artifact/p4-public-sdk.zip" -d "$out/sdk"
  echo 'c5f81138a559154e287c0710daf497b04484f6cc37906a51ddc5713b75e8b822  '"$out/sdk/libge4g_pentomino_native.so" | sha256sum -c -
  cc -g -std=c11 -Wall -Wextra -Werror evidence/p4-independent-20261016/ownership_stress.c -I"$out/sdk" -L"$out/sdk" -lge4g_pentomino_native -Wl,-rpath,"$out/sdk" -o "$out/ownership"
  valgrind --leak-check=full --show-leak-kinds=all --errors-for-leak-kinds=definite,indirect --error-exitcode=99 --log-file="$out/valgrind-ownership.log" "$out/ownership" evidence/p4-independent-20261016/ownership-requests.jsonl
  # Exercise actual pthread exit and FFI response ownership in the original ELF too.
  cc -g -std=c11 -Wall -Wextra -Werror "$qa/thread_ownership.c" -I"$out/sdk" -L"$out/sdk" -lge4g_pentomino_native -pthread -Wl,-rpath,"$out/sdk" -o "$out/thread-ownership"
  valgrind --leak-check=full --show-leak-kinds=all --errors-for-leak-kinds=definite,indirect --error-exitcode=99 --log-file="$out/valgrind-thread.log" "$out/thread-ownership"
  sha256sum "$out/ownership" "$out/thread-ownership" > "$out/binaries.sha256"
fi
date --iso-8601=seconds
echo 'GATE EXECUTION PASS'

# P4 native turn-combat probe

This new sample runs P2/P3 and the removable P4 TurnCombat plugin. It does not
run a FlatLand World or its combat engine. It is a development integration
probe, not Android/Windows Playable Alpha delivery or an official release.

The two fighters alternate fixed-damage attacks. JSON config controls their HP,
damage, identity, colour, pixel feet, viewport and control name. `attack` is a
discrete control; each logical input executes two fixed Core ticks. Camera/View
state and save1 remain separate from Core authority/save2.

From the repository:

```sh
cargo run --locked -p ge4g-pentomino-native --example p4_public -- samples/pentomino_p4/game.json /tmp/p4-frames
cargo build --locked -p ge4g-pentomino-native
python3 samples/pentomino_p4/consumer.py target/debug/libge4g_pentomino_native.so
```

The Rust example captures actual CPU-rendered before/after PNGs. The Python
consumer calls the exported C ABI, asserts gameplay/event/frame/save/error/
removal/restore/close behavior, and writes exact delivered `frame.rgba` plus
`result.json`. It needs only Python's standard library and the shared library.
It does not open engine source. FFI sessions must stay on their creating thread.

Package the committed public native sample outside the repository:

```sh
python3 scripts/package_p4_native.py --out /tmp/p4-public-sdk
cd /tmp/p4-public-sdk
python3 consumer.py ./libge4g_pentomino_native.so
cc -std=c11 -Wall -Wextra -Werror smoke.c -I. -L. -lge4g_pentomino_native -Wl,-rpath,'$ORIGIN' -o c-smoke
./c-smoke
```

MANIFEST.json identifies the exact commit, compiler, library and public-file
SHA256s. Verify them before testing. The script creates a ZIP and ZIP checksum.
Only Linux-hosted native development is verified by this packaging path.

Parent-controlled packaging/consumer execution demonstrates reproducibility;
independent external certification requires a new source-free workspace/session
and consumer-authored tests. Existing acceptance, releases, client identity,
packages and FlatLand saves remain separate. See P4_CONTRACT/P4_RUNDOWN.

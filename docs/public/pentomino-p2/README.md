# Pentomino P2 public Core SDK

GE4G 0.3.0-alpha.1 adds the experimental typed API `ge4g_pentomino::p2`.
The user's separate public Core GPT Work report accepts23/23 tests.
P2 milestone is complete; external legacy execution remains UNVERIFIED. This is
an SDK test package, not a new game client or final release.

Start with [QUICKSTART.ko.md](QUICKSTART.ko.md), then
[PUBLIC_API.md](PUBLIC_API.md) and [PLUGIN_DISCOVERY.md](PLUGIN_DISCOVERY.md).
Give the separate tester [EXTERNAL_TESTER_TASK.md](EXTERNAL_TESTER_TASK.md).
The packaged `CONTRACT.md` is the normative public contract.

The source-free ZIP contains compiled Rust libraries, a public example, these
documents, checksums and a test runner. It omits engine implementation and
in-repository tests. The package pins its exact Linux x86_64 Rust compiler in
`MANIFEST.json`; Rust rlibs are compiler-specific. No GPU, window or ALSA SDK is
needed to consume the Tiny Core SDK. A different compiler/platform needs an
equivalent SDK build, not a claim of a failed runtime test.

Plugins are linked trusted callbacks. Determinism requires all durable plugin
state and randomness to use Core transactions. Core does not sandbox native code,
roll back hidden callback state, catch process termination, or dynamically load
native libraries. The package does not itself isolate a session: give the external
tester a fresh workspace with no source checkout, shared source mounts or private
connectors. Internal packaging checks do not satisfy this external gate.

## Compatibility and scope

- Existing scalar P1 `Host` and save1 remain unchanged, including their global
  event budget. P2 `CoreHost` uses contract2/save2 and independent owner histories.
  Explicitly choose P2 for the corrected retention behavior; no silent save1 import.
- Each owner keeps its newest 256 events; at most 128 new events per commit.
  Global sequence and lifetime counters remain checked and deterministic.
- Scene/Entity express identity and lifetime only. Typed records/actions carry
  declared data without Core interpretation. Input is an explicit next-tick frame.
- Legacy compatibility is a separate snapshot projection crate. Existing `World`
  stays authoritative; imported plugins do not continuously execute legacy games.
- Camera & View/Format are P3 work. Gameplay/Forge and native dynamic loading are
  not implemented by P2.

From a full repository checkout, internal validation commands are:

```sh
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo test --locked -p ge4g-pentomino
cargo test --locked -p ge4g-pentomino-legacy
cargo run --locked -p ge4g-pentomino --example p2_public
python3 scripts/check_p2_boundaries.py
```

Full legacy workspace builds on Linux additionally need the ALSA development SDK.
The source-free instructions below use `rustc` and require no repository checkout.

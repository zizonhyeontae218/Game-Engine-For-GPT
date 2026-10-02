# GE4G 0.1.0 — Basement

Date: 2026-10-02. The requested repository started empty. The supplied GE4C 0.1 instruction set was implemented as **GE4G / GameEngineForGPT**, with binary `ge4g`, project `ge4g.toml`, and six Rust edition 2024 workspace packages. This initial release has no earlier published schemas to migrate.

## Delivered

A real deterministic headless 2D runtime, an optional local game window using the same simulation and CPU framebuffer, versioned TOML/JSON5/JSON authoring and observation formats, swept AABB collision and triggers, NPC interaction, named scene transitions, typed persistent state, atomic save/load and PNG output, structured inspect/trace/diagnostics/schema commands, authored replay assertions, a reference demo and CI.

The reference journey has seven checkpoints across 160 ticks. It moves the player, proves a wall stops motion, talks to the NPC, enters room_b at the named spawn, and changes both persistent demo flags. Tests restart from an actual save and write/decode real PNGs. Repeated final snapshot/event results match.

## Evidence

- `cargo fmt --all --check`: passed.
- `cargo clippy --locked --workspace --all-targets -- -D warnings`: passed.
- `cargo test --locked --workspace`: 22 tests passed.
- Demo `validate`, `test` and `diagnose`: passed; exact golden checked.
- `cargo clippy/test --locked -p ge4g-cli --no-default-features`: passed, including all nine CLI integration tests without a window adapter.
- Real minifb window run under Xvfb matches headless snapshot/events at tick 160, scene room_b.
- Gameplay snapshot SHA256: `a950e9a1c9d5028be69910976f3bfa85c929196d38a30261ea23eb61ef19bde9`.
- Canonical final RGBA SHA256: `38cc4352e7160dcf9104cbe19acd709e36d8165989b3c99d8b6611f0f76e932b`.
- Build toolchain used: stable Rust 1.99.0 on Linux x86_64. Other platforms and older compiler versions have not been exercised here.
- Fresh local clone with a fresh build directory: format, strict Clippy, all 22 tests, demo validate/test, filetree lint and real window comparison passed.
- Optimized release build and extracted executable/demo package test passed.
- Detailed command evidence is recorded in `docs/exec-plans/completed/basement.md`.

## Publication

Implementation commit: `ad339f1d9152c2d68035ff387e251af7212c4a5a`. [Hosted acceptance CI](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/36986830491) passed every step, including the real window comparison, headless-only checks and release bundle build/upload. The published Git tree was checked against the locally verified tree.

[Download the Linux x86_64 executable and demo bundle](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/36986830491/artifacts/11218325229). Extract the nested tar.gz, enter `ge4g-basement`, and run `./ge4g run examples/basement_demo`. Linux X11/XWayland runtime libraries are required for a window; `--headless` needs no display.

## Scope and remaining acceptance

The engine implements declarative behaviors; Lua was optional in the instructions and is omitted. Audio cues are logged with a silent adapter; sound playback is omitted. There is no GUI editor, 3D, network gameplay or general physics. Saved data contains declared persistent keys, not a whole-world resume point.

Automated engineering acceptance is implemented and tested. **Actual human execution is pending:** no person has yet played the demo in this session, tested the keyboard controls and F5/F9 workflow, or signed off on the experience. The Xvfb run is automated adapter evidence only. Therefore the instruction set's final human-inclusive definition of done and ILCX™ human contribution evaluation remain pending.

Human acceptance record: tester **pending**, date **pending**, result **pending**. Follow `docs/TESTPLAN.md` when conducting it.

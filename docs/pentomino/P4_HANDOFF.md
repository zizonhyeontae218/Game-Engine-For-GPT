# P4 → P5 Turn Combat handoff

P4 creates the runnable Rust/C session boundary and deliberately small, removable
two-fighter fixed-damage rule plugin. See P4_CONTRACT/P4_RUNDOWN and evidence/p4.
P0–P3 are unchanged. Platform delivery, Forge and a general full battle engine
are outside this slice. Do not route future gameplay through Tiny Core.

## Continue from the real boundary

- StateProvider owns authority; TurnCombat reads its capability and emits
  decisions. Only the bridge maps verified decisions to carrier Actions.
- One native logical input is two fixed Core ticks. Save only complete boundaries;
  camera time is independent. Preserve exact decision/applied pairing and reject
  contradictory canonical saves, not just malformed JSON.
- Actual rule detach uses CoreHost::remove, preserves carrier authority and
  deletes rule history/RNG. Reinstall resumes current status; it is not a battle
  reset. New mechanics must state which owner survives removal.
- The sample fixed-damage reachability validator is v1-specific. Expanded P5
  actions, parties, costs, status effects and outcome semantics require a new
  versioned contract/content binding and explicit save migration policy.
- Public NativeBridge, JSON array observation DTO, C header and ctypes consumer
  are the first downstream consumers. Keep existing ABI1/schema/save unchanged.

## Concrete P5 work

1. Review a bounded P5 rule/status/action schema with architect/auditor before
   implementing richer turns. Include command identity/exact-once outcome
   semantics; do not invent a new engine or merge FlatLand battle code into Core.
2. Keep action legality, rule decisions, authority storage and presentation
   separate. Add future mechanics as inspectable plugins with JSON configuration.
3. Measure candidate reconstruction/history validation cost before optimizing.
   Current P4 favors atomic correctness. Avoid live rollback that invalidates
   Core handles; do not add unsafe Send to trusted non-Send plugin traits.
4. Consider bounded binary frame delivery and a runtime-owned scheduling/thread
   API before mobile use. C sessions currently have creating-thread affinity;
   JSON RGBA arrays prioritize inspectability over bandwidth.
5. Design richer battle presentation outside Core. Current rendering is an
   existing CPU world/entity compositor with HP bars, actual P3 camera output and
   a documented orthographic subset; it is not a finished battle-stage UI or
   Full3D renderer. General perspective/GPU output remains separate work.
6. Run independent new-session/source-free consumer tests from the packaged SDK.
   Parent's out-of-tree ctypes/C checks establish reproducibility only. Record
   new test source, raw logs and library/ZIP hashes before external acceptance.
7. Android/embedded Windows Playable Alpha remains later work: wire the existing
   client to this stable boundary, validate actual platform execution/input/
   save/shutdown, preserve Android app identity/certificate and do not claim
   physical device acceptance from this Linux-hosted session.

## Reproduce

```sh
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
python3 scripts/check_p2_boundaries.py
python3 scripts/check_p3_boundaries.py
python3 scripts/check_p4_boundaries.py
cargo run --locked -p ge4g-pentomino-native --example p4_public -- samples/pentomino_p4/game.json /tmp/p4-frames
python3 scripts/package_p4_native.py --out /tmp/p4-public-sdk
cd /tmp/p4-public-sdk
python3 consumer.py ./libge4g_pentomino_native.so
```

Keep five existing game replay/golden checks and docs/TESTPLAN floors. Previous
Windows fixture failure is historical in HANDOFF; current baseline main CI
success and 2026-10-09 user's P2/P3 acceptance are separate evidence. No release
or supported-platform expansion is authorized by this handoff.

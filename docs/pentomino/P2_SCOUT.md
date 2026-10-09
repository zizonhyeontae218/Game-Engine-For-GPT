# P2 read-only source inventory

Baseline main2d1ffd060389109f341fb5854ce07890f445f3f9, alpha0.3.0-alpha.1. Initial clean working tree. Maintenance release/0.2 object293ba513f5727d4a7a0a59476c9a98eb193aaa25 fetched; no maintenance branch writes. Scout read only, no tests/install/edits.

## P1 compatibility facts
- lib.rs:115 public PluginDescriptor strict shape; :206 scalar Plugin; :213 scoped Context; :255 poisoned Transaction set/emit/draw1024commands.
- :343/352 SavedPlugin/SavedHost deny unknown fields; :654 next-tick step; :723/:741 discovery/save1; :765/:774/:778 canonical bytes/hash/restore. History global256/pending128 at :868/:955.
- Save1 top-level fixed order: format_version/contract_version/content_binding/seed/tick/plugins/history/pending/next_sequence/events_dropped. Plugin descriptor/bindings/resources/rng_state. Re-encode byte equality and exact installed metadata required.
- Existing tests/lifecycle_contract.rs has26 tests. History test:689 requires combined256; descriptor limits:830 checks256; independence:1101 expects B128vs192 then192vs256. Changing default behavior or descriptor/save shape breaks these tests. Preserve exact P1 surface and save1; new typed/history contract needs explicit version separation, not weakened tests.

## Legacy seams and authority
- Project::load(project/src/lib.rs:363), Project:317, Scene:138, Entity:108. These contain filesystem/decoded content/presentation/gameplay; no mandatory import to Core.
- core StateType:119 Bool/Integer/String, StateDefinition:126, StateStore:145/set:176/persistent_values:191. Adapter can losslessly map to declared generic record fields, with explicit rejection of oversized data.
- project Replay input_at:251/actions_at:202/ReplayCommand:181. runtime World::step_actions:410 validates<=32 names<=64 bytes; step_replay:448 clones/restores World on failure; release_inputs:467 no tick increment. Action direction/choice/skip mapping must remain adapter-owned, not Core semantics.
- runtime World:77 exposes project/tick/scene/entities/state. Mutable World hidden inside a Plugin Mutex cannot participate in Core rollback/save. Prefer explicit checked snapshot/projection bridge; keep legacy simulation authoritative and distinguish projection from delegated execution.
- Legacy save1 runtime/lib.rs:643/648 persistent StateStore save. Save2 flatland.rs:48 Resume includes content/tick/scene/state/actors/runtime; save_flatland:1045 uses filesystem content hash:1032. Camera/view remains opaque adapter-owned. ge4g_runtime::snapshot_hash:686 serializes camera-containing Snapshot and must not change meaning.

## Migration risks
P1 strict bytes, identity incarnation/handle stale remap, external reference removal blocking, typed recursive bounds and shared transaction rollback all need a reviewed contract. Legacy maxima4096entities/128scenes/16MiB text exceed P1 scalar caps; adapter must disclose/prove fit or typed rejection. Two simultaneously mutating authorities are unsafe unless transaction/save ownership is explicitly designed.

## Version/source and test path
Diff maintenance293ba513..currentmain showed no changes in legacy core/project/runtime/render2d/client Rust source or client/lib/test; main version transition changed manifests/docs/consistency checks. Rust1.99.0 installed outside repository; source /home/agent/.cargo/env. Flutter/Dart not found. Parent executes workspace91/P1 tests and legacy four-replay tests separately; source inventory alone is not runtime PASS.

Relevant commands: cargo test --locked -p ge4g-pentomino; workspace fmt/clippy/test; ge4g-cli test examples/{basement_demo,flatland_pacman,flatland_signal_yard,flatland_harbor} --json; headless no-default-features CLI. All P2 behaviors are UNVERIFIED pending implementation and tests. Known Windows ZIP timestamp cause remains UNVERIFIED.

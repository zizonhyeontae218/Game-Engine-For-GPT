# P2 bounded task packets

All packets follow templates/TASK_PACKET.md scope/contract/acceptance rules. Escalate overlap, invented APIs, private dependency, ABI break or rollback/reference/bounds violation. No worker edits root manifests/lock/registry/parent documents. No role self-certifies external tests.

## PENTA-P2-S — Scout
**Goal / Gate:** inventory/G0. **Owner:** pentomino_scout read-only. **Prerequisites:** clean main2d1ffd0 alpha1. **Tools:** Rust2024/Cargo1.99/Python.
- OWNED:none. READ-ONLY:P0/P1 source/contracts/tests, legacy core/project/runtime/saves, STATUS/HANDOFF, maintenance objects.
- FORBIDDEN:edits/tests/install/commits. Output verified APIs/compatibility risks/unknowns.

## PENTA-P2-A — Architect
**Goal / Gate:** freeze typed contract/G1,G2,G6. **Owner:** pentomino_architect docs-only. **Prerequisites:** Scout findings.
- OWNED:docs/pentomino/P2_CONTRACT.md only until release. READ-ONLY:actual baseline/user requirements/P0/P1.
- FORBIDDEN:source/tests/root edits. Output exact types/signatures/limits/encoding, retention/accounting, stable IDs/ephemeral handles, schema/ref ownership, actions/replay/rollback, save compatibility and bridge responsibility.
- Done:independent auditor review before parent acceptance and implementation.

## PENTA-P2-R — Auditor
**Goal / Gate:** refute contract/source/G1,G2,G6. **Owner:** pentomino_auditor read-only. OWNED:none.
- READ-ONLY:proposal/source/tests/diffs/evidence. FORBIDDEN:edits/tests/install.
- Output path:line/severity/reproduction/minimal repair; challenge contamination, retention/ref cleanup/ownership, failed tick/RNG/event/identity rollback, bounds/remap and adapter authority. No invented PASS.

## PENTA-P2-C — Core Implementer
**Goal / Gate:** approved Tiny Core. **Owner:** pentomino_core. **Prerequisites:** reviewed contract.
- OWNED:crates/ge4g-pentomino/src/** only. READ-ONLY:P2 contract/P1 tests/adapter/tests.
- FORBIDDEN:legacy source/dependencies, tests/doc/root changes, View/Format/genre/device/network/dynamic loading.
- Done:exact API, lifecycle/history/scenes/entities/records/actions/discovery/save with atomic failure/ref validity; P1 suite intact.

## PENTA-P2-L — Legacy Adapter Implementer
**Goal / Gate:** explicit preserving bridge. **Owner:** pentomino_legacy_adapter. **Prerequisites:** frozen APIs/scout seams.
- OWNED:crates/ge4g-pentomino-legacy/Cargo.toml and src/** only. READ-ONLY:Core API/legacy Project/World/State/examples/tests.
- FORBIDDEN:Core/legacy source edits, tests/root/lock.
- Output:bridge legacy data/state/actions to generic scenes/entities/records, preserve presentation/resume outside Core, explicit authority/atomicity.
- Done:independent mapping/failure tests plus existing regressions.

## PENTA-P2-T — Independent Tester
**Goal / Gate:** public boundary/failure/replay tests. **Owner:** pentomino_test. **Prerequisites:** frozen API.
- OWNED:crates/ge4g-pentomino/tests/p2_*.rs and crates/ge4g-pentomino-legacy/tests/** only.
- READ-ONLY:source/contracts/P1 lifecycle_contract.rs/legacy regression.
- FORBIDDEN:production/root edits or deleting/weakening P1 suite.
- Done:required P2 executable cases; exact error/unchanged state-token tests, history isolation, malformed refs/records/saves, four-game adapter mapping; report bugs to source owner.

Parent exclusive:Cargo.toml/Cargo.lock, F(x).md, indexes, plans/packets, P2_SCOUT/AUDIT/RUNDOWN/HANDOFF/STATUS, consumer artifacts/packaging and crates/ge4g-pentomino/examples/**; P2_CONTRACT only after architect releases. Shared integration sequential. All unavailable/unrun behavior UNVERIFIED.

Integration ownership: all role-owned files released to parent after source audit and independent tests. Parent chronology repair sequential; no concurrent source edits. External user gate remains UNVERIFIED.

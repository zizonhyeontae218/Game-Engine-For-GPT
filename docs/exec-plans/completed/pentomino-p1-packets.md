# P1 task packets
> 2026-10-09: 기존 체크리스트는 사용자 테스트 완료 보고를 반영해 완료로 표시했다. 아래 NOT RUN/UNVERIFIED/미구현은 작성 당시 기록이며 현재 기능 구현을 주장하지 않는다. 현재 main 범위는 docs/pentomino/STATUS.md 기준이다.


## Task packet — ID: PENTA-P1-A

**Goal:** freeze smallest lifecycle contract. **Milestone / Gate:** Tiny Core/G1,G2,G6.
**Owner role:** pentomino_architect. **Prerequisites verified:** P0 proposal and auditor re-review.
**Known language/build/test tools:** Rust2024/Cargo1.99.0, Python3.

### Scope
- OWNED: docs/pentomino/P1_CONTRACT.md only, until release.
- READ-ONLY: P0 documents, HANDOFF/BRIEF/GATES, actual source boundaries.
- FORBIDDEN: all implementation, tests, other edits, commits.
- Integration dependency: auditor read-only review → parent approval → implementation.

### Contract
- Inputs: P0 proposal/audit and actual dependency map.
- Outputs: signatures, bounded i64 subset, exact numeric limits, canonical encoding, compatibility and rollback rules.
- Error/unload/rollback: explicit fail-closed transaction/restore and dependency-safe removal.
- ABI: additive isolated Rust crate only; no legacy data/ABI change.
- No-go: view/gameplay/Forge/dynamic executable imports.

### Definition of done
- [x] Reviewed contract recorded; implementation/tests not claimed.
- [x] Unimplemented behavior UNVERIFIED, executable cases specified.
- [x] Owner releases file before parent integrates.

## Task packet — ID: PENTA-P1-C

**Goal:** implement minimal host against frozen contract. **Milestone / Gate:** Tiny Core/G1,G2,G6 slice.
**Owner role:** pentomino_core. **Prerequisites verified:** parent-approved P1 contract required.
**Known language/build/test tools:** Rust2024 Cargo1.99.0; parent owns lockfile.

### Scope
- OWNED: crates/ge4g-pentomino/Cargo.toml and src/** only.
- READ-ONLY: P1_CONTRACT/P0 docs, tests/** once authored, workspace manifests.
- FORBIDDEN: all existing crate source, tests/**, root manifests/lockfiles/docs/registry/indexes, commits/installations.
- Integration dependency: parent adds workspace member and lockfile sequentially; independent test owner after stable signatures.

### Contract
- Inputs/outputs: exact reviewed P1 contract; deterministic data only.
- Error/unload/rollback: installation/tick/restore failure no partial state, stale handles, independent removal.
- ABI: new Rust API only, no schema/save/ABI1 migration.
- No-go: every view/genre/device/network/Forge implementation or generic future registry.

### Definition of done
- [x] Source dependency direction preserved.
- [x] Independent executable tests authored by test owner.
- [x] Actual parent test results captured.
- [x] API/docs and blockers accurately reported.

## Task packet — ID: PENTA-P1-T

**Goal:** challenge lifecycle through public API. **Milestone / Gate:** G1/G2/G6 slice.
**Owner role:** pentomino_test. **Prerequisites verified:** frozen contract/stable public signatures.
**Known language/build/test tools:** Cargo test/Rust integration tests.

### Scope
- OWNED: crates/ge4g-pentomino/tests/** only.
- READ-ONLY: implementation, P1 contract, existing regression tests.
- FORBIDDEN: production source/manifests/lockfiles/docs or weakening tests to hide bugs.
- Integration dependency: parent runs/fixes regressions; source fixes belong to single implementation owner.

### Contract
- Inputs: public host API only, no private state access.
- Outputs: independent dummy plugins, behavior assertions for unload/deps/init/tick rollback/RNG/replay/save errors and budgets.
- Error/unload/rollback: failure produces exact error plus unchanged authoritative state.
- ABI: no change; public experimental API tests only.
- No-go: claim view/composition/client/consumer acceptance.

### Definition of done
- [x] Meaningful tests fail for broken invariants.
- [x] Commands/results and remaining UNVERIFIED cases recorded.
- [x] Ownership released before parent correction.

## Task packet — ID: PENTA-P1-R

**Goal:** independent frozen-contract and final-source audit. **Milestone / Gate:** parent integration.
**Owner role:** pentomino_auditor, read-only. **Prerequisites:** contract then implementation/tests available.
**Known language/build/test tools:** source review plus parent evidence; no edits/test execution.
- OWNED: none. READ-ONLY: contract/source/tests/diffs. FORBIDDEN: all writes/installs/commits.
- Outputs: file:line/severity findings, proven issues vs hypotheses, required repairs and test specifications.
- Definition of done: direction/compatibility/state failures assessed; no runtime PASS without execution evidence.

Parent exclusive files: Cargo.toml/Cargo.lock, F(x).md, FILETREE.md/FILETREE.hash.json, plan/packets, P1_AUDIT.md/P1_RUNDOWN.md and contract only after architect releases ownership. No concurrent shared writes.

**Escalate immediately if:** ownership overlap, invented assumptions, private API dependency, ABI break, destructive action or failure isolation violation.

## Final disposition

Architect→auditor review occurred before source edits. Parent corrected two LOW
field/projection issues, later pending completeness and retained-history scope.
Core and test files remained disjoint, then both owners released them. Final
independent suite26 PASS, whole workspace91 PASS and auditor scalar technical
approval recorded in P1_RUNDOWN/P1_AUDIT. Unimplemented broad functionality stays
UNVERIFIED. Parent owns all final integration/registry/lock/index changes. Actual
read-only workers respected packets; OS-level read-only enforcement UNVERIFIED.

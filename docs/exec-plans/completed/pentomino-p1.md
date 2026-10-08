# Pentomino P1 — isolated deterministic lifecycle slice

## Outcome

Implement the smallest independently testable host and two independent dummy plugins. Prove install/failure rollback, dependency-safe removal and authoritative state restoration. This is an experimental Rust addition, not a migrated/released Pentomino engine.

## Context

Start 2026-10-08 at clean P0 commit52e75c004cc7626091fa6b51e3baecdc1a89dd6d, branch pentomino/p1-lifecycle. Origin main remains293ba513; P0 draft PR1 remains separate. Read the orchestration skill, HANDOFF/BRIEF/GATES, P0 proposal/audit and role definitions. Rust tools were absent at P0; this session installed stable rustc/cargo1.99.0, rustfmt/clippy locally. Existing workspace remains0.2.0 and ABI1.

Known Windows digest failure from HANDOFF remains UNVERIFIED; client/source/signing changes are outside this slice. System apt installation failed due to unprivileged host permissions; local ALSA development files may be used for Linux Rust infrastructure only.

## Scope / non-scope

New `crates/ge4g-pentomino`: bounded plugin-owned i64 records/events, declared provider reads, transactional deterministic RNG, tick sequencing, versioned canonical state, discovery and stale owner handles. No existing core/runtime/project/view dependency, adapter integration, native ABI change, dynamic loading or executable game imports. Generic schemas, scene/entity handles, input actions, views and gameplay remain UNVERIFIED/deferred.

## Acceptance evidence

Freeze `P1_CONTRACT.md` through architect→auditor before implementation. Implementer and independent test owner have disjoint files. Run fmt/clippy/new crate tests, dependency graph review and docs/TESTPLAN.md regression commands where available; record exact failures and unrun checks. Full client/platform/consumer gates remain UNVERIFIED.

## Milestones

1. Contract: concrete numeric budgets/encoding/tick/owner/removal/save semantics; independent technical review.
2. Tiny Core: minimal host implementation and independent integration fixtures/tests, G1/G2/G6 slice evidence.
3. Parent sequential integration, auditor source review, regression checks and Rundown.
4. Next slices: View/Format before Gameplay; Forge0.4. No automatic suspended client jobs.

## Decisions

- First implementation is a new isolated crate, not extracting legacy ge4g-core with presentation fields.
- Narrow P0 into i64 data records/events; broader P0 proposals remain UNVERIFIED. Contract version and canonical envelope are experimental and separate from legacy saves/ABI.
- Parent owns root Cargo.toml/Cargo.lock, identifier registry and generated indexes. Implementation and tests never concurrently edit those files.

## Progress

Architect freeze completed and ownership released; independent auditor recommended acceptance after two LOW field/projection clarifications. Parent integrated those and approved before implementation. Core owned only new manifest/src; independent test role owned only tests. Both released ownership before parent final checks. No shared simultaneous writes. Parent prepared Rust1.99.0 and a local ALSA SDK. Baseline workspace65 tests and workspace clippy passed before new functionality.

P1 scalar implementation and26 independent tests completed. Parent repaired
pending completeness/history monotonicity before atomic restore. Auditor's
global-retention independence scope finding corrected and tested, then re-review
technically approved the slice. Full workspace91 tests, clippy/fmt, headless and
all four legacy replays passed. Evidence commit7c9dd6d; complete Rundown in
`docs/pentomino/P1_RUNDOWN.md`. Entire Tiny Core or0.3 release is not claimed.

## Verification log

- git initial status clean; HEAD52e75c0; origin/main293ba513.
- cargo/rustc1.99.0 installed; rustfmt/clippy installed.
- apt-get system install failed permission denied; no repository effect.
- Pre-change `cargo test --locked --workspace`: PASS65 tests across21 test/doc targets.
- Pre-change `cargo clippy --locked --workspace --all-targets -- -D warnings`: PASS.
- Final `cargo test --locked -p ge4g-pentomino`:26 PASS.
- Final workspace fmt/clippy/test: PASS,91 tests.
- Final four TESTPLAN example replays: PASS deterministic/save_reload/golden/PNG.
- Final no-default-features CLI clippy/test: PASS,9 tests.
- Python setting/tooling tests and release-consistency: PASS.
- Index/lint and remaining platform/consumer evidence recorded in Rundown.
- Linux Rust checks use PKG_CONFIG_PATH=/workspace/scratch/pentomino-build-deps/alsa/usr/lib/x86_64-linux-gnu/pkgconfig with a locally extracted Debian ALSA1.2.14 SDK; no client-platform support claim.

## Handoff

Completed ownership packets: `pentomino-p1-packets.md`. Next work must freeze the minimal format-facing data contract and legacy adapter before named-view integration; current scalar host lacks Scene/Entity/actions. P1 is a verified first slice, not full Tiny Core. Existing data-only imports, schema/save/ABI1, Android identity/certificate, historical binaries and supported clients preserved. Windows ZIP causality remains UNVERIFIED. P0 technical review does not substitute for P1 tests.

# Pentomino P2 — game-data Tiny Core completion candidate

## Outcome
Per-plugin retained events, presentation-free Scene/Entity identities, bounded declarative typed records/actions and an explicit legacy compatibility adapter. Prepare public external GPT Work test artifacts after internal validation. P2 stays a completion candidate until the user supplies external verification.

## Context
Start from clean main2d1ffd0, GE4G0.3.0-alpha.1, Flutter0.3.0-alpha.1+9. Branch pentomino/p2-tiny-core. P0/P1 merged. release/0.2 points to293ba513; the maintenance branch is read-only. Rust/Cargo1.99.0 and local ALSA SDK available. Scout investigates current source/tests. P1's26 tests explicitly assert save1/scalar descriptors/global256 history; do not delete or weaken them. Architect must version the new retention/typed surface explicitly and preserve scalar save/API behavior. Existing schema1/2 and ABI1 legacy runtime remain unchanged.

## Scope / non-scope
P2: typed core/public API, lifecycle/references/actions/save/discovery, per-owner history and legacy bridge. No camera, viewport, sprite, layer, coordinate, FlatLand or gameplay semantics in Tiny Core. Adapter may retain legacy data; Core receives generic records only. No P3 views/renderers/3D/combat/world/interaction/platformer/Forge or dynamic loading. No release/0.2 edits, client/signing/platform changes or external-test self-certification.

## Acceptance evidence
Scout baseline → architect contract → independent auditor → parent freeze. Disjoint core/adapter/test ownership. Required automatic tests: per-owner history isolation, identity/stale/foreign/ref cleanup, schema/bounds validation, deterministic actions/replay, whole-tick/RNG/event rollback, canonical roundtrip/continuation, malformed-save atomic rejection, four legacy regressions, workspace fmt/clippy/test and dependency direction. Every PASS names command/artifact; others UNVERIFIED/BLOCKED.

## Milestones
1. Discovery/design freeze with save compatibility and numeric limits.
2. Core implementation, separate legacy bridge, independent tests.
3. Parent integration/source audit, regression and reproducible evidence.
4. Public package: README/Quickstart/API/plugin discovery/example/build-test commands/external instruction sheet; verify staging/checksums internally.
5. Request user's separate GPT Work test. Await external evidence; P2 remains candidate. P3 handoff explicitly Camera & View/Format next; no P3 code.

## Decisions
- User's P2 scope supersedes old P2-view bootstrap; P3 owns views.
- Parent owns workspace/lock/shared contracts/registry/docs/indexes and integration.
- Preserve P1 scalar/save1 exactly; additive typed contract2/save2 has independent
  event history. No silent save1 migration; retained references block unsafe removal.
- Legacy adapter is snapshot projection, not continuous Core-delegated gameplay.
- Audit hardening: reject impossible cross-owner global sequence/tick chronology,
  independently reproduced failing before repair and passing after repair.
- Source-free SDK pins exact Rust compiler/target; package selfcheck is internal.
- Trusted native plugins are not an OS sandbox; hidden state/panic/process recovery UNVERIFIED.

## Progress
Discovery, contract review/freeze, implementation, source audit and internal acceptance
complete. Core/adapter/test source ownership returned and sequentially integrated.
Production commit48d7f4a. Public source-free SDK built and unpack/compile/run selfcheck
passed. Drive ZIP upload/download SHA256 readback and Rundown complete. User's separate GPT Work gate
UNVERIFIED; plan stays active.

## Verification log
git fetch/ls-remote main/release0.2: baseline main2d1ffd0, maintenance293ba513.
Rust/Cargo1.99.0. Baseline workspace91 PASS; final workspace116 PASS (P1 26,
P2 21, bridge4), fmt/clippy/headless9/Python tooling PASS. Four actual legacy
CLI tests assert deterministic/save_reload/golden/PNG true. Public example301
and source-free SDK internal selfcheck PASS. Exact commands/observations and
source commit in docs/pentomino/evidence/p2/verification.json. External gate
UNVERIFIED; Windows/Android physical/client builds not run.

## Handoff
Ownership in pentomino-p2-packets.md. Keep plan active through external verification. Preserve known intermittent Windows ZIP digest failure; cause/fix UNVERIFIED. No automatic final release/merge or P3 implementation.

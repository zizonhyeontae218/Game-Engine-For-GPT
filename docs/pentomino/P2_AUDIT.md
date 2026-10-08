# P2 independent audit — technical integration accepted

Scope: frozen P2_CONTRACT.md and production Core/adapter source; user external
GPT Work gate is separate: user public Core23/23 accepted; external legacy UNVERIFIED. Source artifact:
`48d7f4aa6932c9031afee722e0c58f7756b864c8` (0.3.0-alpha.1).
Scout/architect/auditor had no production-source write ownership. Test author
owned only new P2 and bridge tests; implementers owned distinct Core/adapter paths.
Parent integrated manifests, locks, public example, docs and chronology repair.

## Contract review before implementation

Auditor initially requested two MEDIUM repairs, both incorporated before freeze:

- Owner history must be the complete newest contiguous suffix, not merely obey
  dropped+retained=emitted: ordinal=dropped+i and retained=min(emitted,256).
- Record/save byte bounds require capped streaming writers/counting before
  accepted insertion or unbounded serialization; cloning then measuring is unsafe.

LOW clarifications: stable saved references identify the restored timeline while
runtime handles provide freshness; restore must not max old/saved identity counters.
Legacy replay reads current World.tick; only mapped Core target is tick+1.

## Source audit and reproduced repair

Auditor found no blocking departure from frozen Core contracts. Staging covers
install/tick/remove/restore; mutation errors poison callbacks; references include
records, nested lists, entity scenes and retained/pending events. Final validation
catches same-tick foreign provider removal; Core remains engine independent.

The cross-owner chronology hardening suggestion became a concrete failing test:
A emits tick1/seq0, B tick2/seq1. Swap sequence values consistently in histories and
pending; individual owner ordering, suffix/accounting and pending remain valid.
Before repair restore accepted this impossible global chronology (exit101,
expected InvalidSave assertion). Parent added retained events merged by sequence
with nondecreasing ticks. The identical targeted test then passed; independent
tester confirmed. Auditor separately accepted this read-only source repair.
Command: `cargo test --locked -p ge4g-pentomino --test p2_contract
restore_rejects_global_event_sequence_tick_regression_across_owners -- --exact`.
Observed fixed result: PASS1,20 filtered; [log](evidence/p2/chronology-fixed.log).

## Public limitations made explicit

- ReadFrame is a Rust typed DTO; its structured RecordKey map has no direct
  nonempty serde_json encoding. Save2 persistence is `CoreHost::save()` bytes.
- LegacyProjection::plugin accepts trusted caller-supplied, well-formed button
  hashes with Bool(true). It cannot recover hash provenance; bridge input_frame
  validates actual Replay labels. Imported callbacks do not execute gameplay.
- Legacy World and camera/presentation/save/resume remain outside Core.
  Snapshots are immutable; continuous synchronization is unimplemented.
- Trusted callbacks must avoid hidden mutable durable state and external effects;
  native sandbox/panic/process recovery and adversarial native code are UNVERIFIED.

## Executed evidence

Parent executed workspace fmt/clippy/test: PASS116 tests, including unchanged
P1 26 + new P2 21 + bridge4. Four legacy CLI regressions report deterministic,
save_reload, golden_checked and png_capture true. Dependency scanner confirms
Core has no transitive engine dependency, root P1 only adds pub mod p2, P1 test
checksum unchanged and legacy baseline source/client/examples unchanged.
[Exact commands/results](evidence/p2/verification.json) and [Rundown](P2_RUNDOWN.md).
The auditor did not run these tests and does not self-certify external acceptance.

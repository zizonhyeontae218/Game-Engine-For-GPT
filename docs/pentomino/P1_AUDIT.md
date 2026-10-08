# P1 independent review and parent integration

## Contract review before implementation

Architect authored `P1_CONTRACT.md` and released ownership. Auditor read it
independently without editing or running tests, and recommended technical
acceptance after two LOW corrections:

- Observation must expose `first_retained_sequence` for the whole host history,
  distinct from owner-filtered event lists.
- Independent B projections compare resources/RNG and event tick/owner/kind/value;
  exclude host-global Event.sequence and sequence/drop counters. Whole-host
  canonical save/hash retains those counters.

Parent integrated both and accepted the frozen scalar lifecycle contract before
dispatching core implementation. Reviewer confirmed no mandatory legacy engine
dependencies, ABI change, executable game import or genre/view assumption.
Numerical limits, scoped handles, poisoned transaction behavior, exact dependency
bindings, canonical version1 bytes and atomic restore are documented.

Technical acceptance is not runtime PASS. Baseline workspace tests65 passed and
baseline clippy passed on Rust1.99.0, but those precede the new implementation.

## Implementation review

Parent source review found that a canonical save could omit current-tick
pending events while retaining their history, silently losing next-tick input.
Parent clarified the contract; core added exact pending/current-tick history
matching and nondecreasing history tick validation before any mutation. The
independent tester added rejection/state-and-token preservation regression.

Auditor identified a MEDIUM scope mismatch: global history256 retention means
unrelated A events can evict B history. Parent narrowed independence to
resource/RNG/current-tick outputs and disclosed shared retention. Independent
boundary fixture proves B history128 vs192, and after removal192 vs256, while
current-tick projections match. No history redesign or weakened assertion.

Auditor re-read corrected source/contract/tests and parent26-PASS log, resolving
its conditional finding and technically approving the scalar host slice. No
additional blocking source defect found in scoped review. Reviewer did not
execute tests. Source/tests owners released ownership before parent final checks.

Implemented evidence is commit7c9dd6d7e2c89984738fd7553e14b8cb6aa25107;
P1_RUNDOWN.md records the parent26-test public suite, full91-test workspace,
fmt/clippy, headless and four legacy replay results. G1/G2/G6 evidence applies
only to this additive scalar slice; broad Tiny Core/view/gameplay/client/consumer
remain UNVERIFIED. No existing ABI/save/source changes or native sandbox claim.

# P5 Gameplay and pre1P6 playable candidate

## Outcome
Finish versioned modular Gameplay through the authoritative P2 Core, then deliver
an actual pre1P6 Android or Windows build for the user's PASS/FIX/CHANGE decision.
No PR merge, release, signing or deployment is authorized. Stop before pre2P6.

## Context
2026-10-30: main 8622034; P4 chain #8 -> #7 -> #5 remains open.
Development branches from #8 head 1d964020, preserving all P4 evidence.
P4 engine native evidence is conditionally sufficient for P5. Blind source-free
certification, Flutter, ARM/physical acceptance remain incomplete.
Drive project folder currently contains P2/P3 candidates; authoritative P4
contracts/evidence are checked into Git. Historical Windows failure and user's
2026-10-09 P2/P3 acceptance remain separate evidence, not rerun claims.

## Scope / non-scope
Add bounded Gameplay v2, additive P5 native API and one minimal Flutter sample.
Preserve P0-P4, FlatLand, ABI1, save formats and Android identity/certificate.
No editor, Full3D, Forge, content expansion or new general engine.
ILCX™ nH+ grade is a design target, not an independently measured certification.

## Acceptance evidence
New command -> rule decision -> persistent carrier application; invalid input
atomic, exactly-once receipts, save/restore + deterministic replay, removal/reinstall.
Run focused new tests, then existing TESTPLAN floor once, plus independent review.
Build actual client; distinguish automated platform evidence from user acceptance.

## Milestones
1. P4 dependency/evidence/CI investigation and compact API review.
2. P5 gameplay/native implementation and deterministic sample.
3. Focused tests, independent audit, regression floor, PR and handoff.
4. pre1P6 existing Flutter client/native integration and actual build delivery.
5. User acceptance boundary; no pre2 without PASS.

## Decisions
- New v2 module/contracts preserve proven P4 v1 rather than widening its save semantics.
- Linux infrastructure is permitted; supported user clients remain Android/Windows.
- P6 roadmap: pre1 playable; pre2 platforms; pre3 management; pre4 presentation;
  pre5 persistence/recovery; pre6 integration; optional pre7 stabilization.

## Progress
- P4 chain and contracts confirmed. Latest #8 HEAD filetree lint passes locally.
- CI 39138891574 failed on older 33b8193 only at filetree hash lint, after
  workspace and five game checks passed. Later 1d96402 repaired committed
  Windows evidence bytes with [skip ci]; current clean checkout lint passes.
- Architecture + independent preimplementation API review completed. Gameplay v2,
  additive native P5 and existing Flutter adapter implemented in disjoint files.
- P5 engine technically accepted: workspace181/181, strict Clippy/fmt,
  five legacy goldens/save-replay,12 new P5 tests, public C/ctypes and independent
  source-aware audit. Raw evidence and scoped limits are in P5_RUNDOWN.md.
- PR #9 created against #8. Local Git push lacks credentials; connected GitHub
  Git-tree/commit/ref APIs published an exactly matching source tree.
- Initial Windows Flutter analysis found one redundant import; fixed. Latest
  build source8c7e9e72 is in Actions. Actual pre1P6 delivery remains pending.
- Local Flutter bootstrap was auto-review rejected for metadata endpoint access;
  no bypass/retry. Existing GitHub client workflow is the safer build path.

## Verification log
- git clone, branch from origin/qa/p4-remaining-20261018: PASS.
- python3 scripts/filetree.py lint at pristine 1d96402: PASS.
- No runtime feature or client acceptance is claimed yet.

## Handoff
Use this plan plus P5 contract/proposal and P4_HANDOFF. Parent owns shared
contracts/manifests/client integration; coding packets have disjoint source paths.

# P4 Independent QA — 2026-10-16

## Outcome
Reproduce pinned CI SDK behavior, extend boundary/endurance evidence, repair only reproduced P4 defects, and retain final audit before removing temporary WORK_HANDOFF.md.

## Context
PR #5 source 72d49a63f730e2a3ee2032e83948894e4fd480ba; SDK artifact 11823886424. Original audit ZIP is immutable evidence. Public P4_CONTRACT.md defines scope. Drive P3_RUNDOWN/Soul.md are historical; current source/approved contracts take precedence.

## Scope / non-scope
Linux C ABI/Rust bridge and regression tests. Preserve P0–P3, FlatLand, ABI1, saves, Android identity/signing. No P5, merge, release or platform feature implementation.

## Acceptance evidence
Keep independent scripts, raw outputs, hashes, environment, platform status and exact commands. Distinguish direct execution, prior logs and source review. This agent can access source, so out-of-tree SDK tests are not a filesystem-enforced blind consumer certification.

## Milestones
1. Verify artifact and rerun baseline independent 14 suites.
2. Extend soak/concurrency/JSON/save/resource boundary tests.
3. Reproduce/fix any defect on this branch and run available regression checks.
4. Preserve final evidence, post Issue #6 and remove temporary handoff only.

## Decisions
- 2026-10-16: Windows/Android require actual platform execution; unavailable = NOT TESTED.
- Use repository scripts/filetree.py to preserve its established manifest format.

## Progress
SDK 24/24 groups PASS; local build13/13 groups, workspace169/169 and headless CLI9/9 PASS. C ownership302 responses freed. No product defect reproduced. ASan/Valgrind execution blocked before tests;7 residual gates documented. Rust/tooling and local build dependencies provisioned. Evidence publication/Issue cleanup pending.

## Verification log
Baseline: python3 independent_p4_test.py (in independent-run): 14 PASS, 0 FAIL.

## Handoff
Final results will be in evidence/p4-independent-20261016/ and Issue #6. Official docs/pentomino/P4_HANDOFF.md is protected.

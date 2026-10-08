# P3 bounded task packets — Camera/View/Format

Common: inherit task/skill constraints. Escalate overlap/private Core needed,
unsupported assumption/ABI break/failed isolation. No role self-certifies external
consumer tests. Parent owns manifests/lock/shared contracts/docs/indexes/package.
Native Camera source and compatibility adapter remain separate crates/modules.

## P3-S Camera Scout
OWNED:none (read-only). Read actual P2 public API, legacy core/project/runtime/
render2d/client camera/views/save and fifthdemo. No writes/tests/install/commits.
Deliver file:line seams, verified fixture and legacy classification, Core blockers.

## P3-A Camera Architect
OWNED:docs/pentomino/P3_CONTRACT.md only once assigned. No implementation/tests.
Read Scout/P2/public assumptions. Specify exact minimal lifecycle/types/signatures,
math/precision/bounds, multi cameras, view policies, behavior/transitions, read-only
input/output, local saves/discovery, render compatibility and error taxonomy.

## P3-M Math/Transform Reviewer
OWNED:none, read-only. Review spaces/units/order/handedness/inverses/projections,
finite/overflow/clipping/precision, canonical state and smooth retarget continuity.
No tests/writes. Output concrete issues for parent/architect before freeze.

## P3-B Core Boundary Auditor
OWNED:none, read-only. Review contract/source/deps, authoritative ownership,
P2 surface preservation, saved state/tick rollback, independent lifecycle, legacy
compatibility. Any genuine Core change requires reproduced failure/minimal review.

## P3-V View Implementer
OWNED:NEW Camera/View crate production source+crate manifest (final path after
contract freeze). No parent workspace/lock/Core/legacy/tests/docs edits.
Four view families and independent multi cameras/math/behaviors/smooth local save.

## P3-C Compatibility Implementer
OWNED:NEW compatibility crate production source+crate manifest after freeze.
No existing Core/legacy runtime/renderer or native View source/tests/root edits.
Reuse public readonly legacy boundaries; actual existing fifth2.5D fixture display
via Pentomino pose output; preserve authoritative Core/World state and old formats.

## P3-T Independent Tester
OWNED:NEW view/compat crates tests/** only after freeze. No production source,
Core/P1/P2 tests/root/docs changes. Boundary/failure/roundtrip/save/substitution/
viewfamilies/discovery/fifthfixture+four regression assertions; defects to owner.

Parent source-free public examples/performance harness/packaging/scripts/shared
integration remain exclusive. All source/test owners released after implementation. Math/Boundary source
audits complete; parent exclusively integrates/docs/packages. No shared-file
simultaneous source writes occurred.

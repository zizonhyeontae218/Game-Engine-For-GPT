# GE4G 0.3.0 — Pentomino alpha

main is the next-generation development line. Pentomino is the name of GE4G
0.3.0, not a separately versioned product. Current development identity is
**0.3.0-alpha.1**, with Flutter **0.3.0-alpha.1+9**. No final 0.3.0 release,
signed alpha delivery or physical acceptance is claimed.

## Branch policy

- main: Pentomino alpha development, including P0/P1. New architecture work lands
  here in bounded reviewed slices; completing the full engine is not a prerequisite
  for main integration.
- release/0.2: FlatLand maintenance baseline, initially
  293ba513f5727d4a7a0a59476c9a98eb193aaa25. The released v0.2.0 tag and original binary bytes
  are unchanged; public delivery moves to GitHub Releases. Maintenance fixes can be selectively ported to main;
  never bulk-merge alpha version/architecture changes back into release/0.2.
- Merged P0/P1 branch refs were removed on 2026-10-09; commits and closed PRs remain.
  P2 PR #3 and P3 PR #4 are merged; completed milestone refs are historical.
  Save/archive/schema migration decisions are explicit, not inferred from version.

## Implemented and next

P0–P3 are merged into main. P2 PR #3 merge: bc475561; P3 PR #4 merge: fa10ddfe. P2 adds identity,
typed records/actions, independent owner histories and save2/discovery. P3 adds
independent View/Format, multiple Cameras, coordinate transforms and saved smooth
transitions. P1 scalar/save1 and legacy runtime/schema/save/ABI1 remain preserved.

All P2/P3 tests are PASS by user report on 2026-10-09. This acceptance is separate
from stored CI logs and the earlier P2 external Core23/23 report. Full3D GPU rendering,
continuous legacy gameplay delegation remain unimplemented.

P4 branch candidate adds a removable two-fighter Gameplay plugin, staged native
sessions and additive `ge4g_p4_v1` C ABI. Actual P2 input/events/authority, P3
CameraFrame, existing CPU pixels, separate save2/save1, removal and shutdown run
through public APIs. Internal Rust/FFI/regression evidence is PASS; independent
new-session source-free consumer certification remains UNVERIFIED. No claim that
P4 is already merged or that Android/Windows Playable Alpha is delivered.
Read [P4_CONTRACT.md](P4_CONTRACT.md), [P4_RUNDOWN.md](P4_RUNDOWN.md) and
[P4_HANDOFF.md](P4_HANDOFF.md).

Next: P5 richer Turn Combat, then Android and embedded Windows deliverables once
the boundary is stable. Read [NEXT.md](NEXT.md), P2/P3/P4 contracts before choosing
work. Do not repeat P0–P3. Forge remains0.4.

## Verification and release policy

- Existing workspace fmt/clippy/test, headless CLI and four legacy replay/frame
  checks remain required regressions; new host tests are part of the workspace.
- cargo test --locked -p ge4g-pentomino verifies the current scalar host only.
- Android/Windows are the current client CI matrix; no automatic platform expansion.
- scripts/release_consistency.py checks alpha manifests/lockfile and branch-facing
  documentation while retaining historical FlatLand documentation sanity checks.
  The release/0.2 copy retains its original strict 0.2 identity checks.
- Existing ABI1, game schema1/2 and save semantics are not automatically changed
  by alpha package versions. The experimental host's save version is separate.
- Android application ID and pinned signing certificate are preserved. Build
  number 9 exceeds the historical 0.2 build number 8. Existing binaries/keys are
  untouched. No signing or deployment is performed during this branch transition.
- Existing test checklists are complete by the user's 2026-10-09 report. Historical
  Windows ZIP fixture failure evidence remains in HANDOFF.md; this repository
  cleanup does not claim a new investigation or feature implementation.

## P5 / pre1P6 development candidate (2026-10-30)

PR #9 stacks on unmerged P4 #8 -> #7 -> #5. Gameplay v2/native P5 is implemented
and technically accepted in the reviewed scope: workspace181/181, new rules/native
contracts, actual C/ctypes consumers, five legacy replay/golden checks and an
independent source-aware audit. See P5_RUNDOWN.md for counts, failures and limits.
This is not a merge, blind certification, signed release or physical acceptance.
The existing Flutter client has the pre1P6 sample adapter; actual platform build
evidence and user acceptance remain separate. No pre2P6 before user PASS.
P5_CONTRACT.md and P6_HANDOFF.md supersede the old 'next P5' planning paragraph
for this candidate only; main remains unchanged until explicitly approved merge.

## Final submission checkpoint

P5 technical work and pre1P6 executable submission are complete. Sourcec18adf76;
engine run41068956609 PASS; client run41068956566 Windows/Android PASS,
Flutter38/38 and actual native Windows startup/capture PASS. See P6_HANDOFF.md
submitted checkpoint for exact artifact/hash/run receipts and resolved failures.
User acceptance PENDING; no pre2, merge, signing or release without authorization.

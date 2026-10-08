# Make main the Pentomino alpha line

## Outcome
main develops GE4G 0.3.0 Pentomino alpha; preserve release/0.2, integrate P0/P1
and align identity/docs/checks without implementing new engine functionality.

## Context
User selected option3 on 2026-10-08 (Asia/Seoul): Pentomino is GE4G0.3.0's name
and still alpha; main need not remain the 0.2 stable product. P1 was merged into
P0 at dc485fe. Existing main was293ba51. P0+P1 CI was green before metadata changes.

## Scope / non-scope
Create release/0.2 from verified current main. Advance workspace and Flutter to
0.3.0-alpha.1 (+9 client build), update Cargo.lock package versions and entry docs,
check strict alpha identity, merge the reviewed branch into main. Preserve stable
tag/artifacts/certificate/schema/ABI and existing gameplay. No new view/gameplay,
new platforms, signing, release publishing or consumer acceptance.

## Acceptance evidence
Config/tooling tests, strict alpha version/lockfile checks, historical docs checks,
FILETREE lint and whitespace checks. GitHub main/maintenance readback and PR merge.
CI on the changed alpha tree is distinct from pre-change P0/P1 CI.

## Milestones
1. Preserve0.2 branch: COMPLETE, release/0.2 at293ba51.
2. Prepare alpha manifests/docs/gates: COMPLETE.
3. Verify and integrate main: pending publish/readback.

## Decisions
- User's option3 replaces earlier recommendation to keep main a0.2 product.
- Single workspace alpha identity; Pentomino is a GE4G version name.
- Keep legacy implementation until replacement slices work; it is not a second
  main product or proof that Pentomino runs games.
- Regression sample package versions can remain0.2.0: game content identity is
  independent of the runtime/client version. Historical release docs remain factual.
- Android build9 preserves update ordering and signing identity.

## Progress
Prepared identity and next-worker entrypoints; maintenance branch created.

## Verification log
To be finalized before commit. No local Cargo/Flutter toolchain in this environment;
use GitHub CI for product/build checks without claiming unrun local results.

## Handoff
STATUS.md → P1_CONTRACT.md → next reviewed Tiny Core slice. Do not restart P0/P1.
Do not publish the development identity as a completed0.3 release.

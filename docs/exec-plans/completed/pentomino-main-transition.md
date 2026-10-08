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
3. Verify and integrate main: COMPLETE, PR#1 merge8f0abe7; engine/client CI PASS.

## Decisions
- User's option3 replaces earlier recommendation to keep main a0.2 product.
- Single workspace alpha identity; Pentomino is a GE4G version name.
- Keep legacy implementation until replacement slices work; it is not a second
  main product or proof that Pentomino runs games.
- Regression sample package versions can remain0.2.0: game content identity is
  independent of the runtime/client version. Historical release docs remain factual.
- Android build9 preserves update ordering and signing identity.

## Progress
Identity/entrypoints published; release/0.2 preserved; PR#1 merged into main.
Engine and Android/Windows client CI on8f0abe7 PASS.
Archived P0 plan/packets to completed; next development slice is STATUS.md.

## Verification log
- Strict alpha identity/docs/lockfile: PASS. Tooling5/5, settings and FILETREE/diff checks PASS.
- Injected stable0.3 identity, stale host lock version and client build8 were each rejected;
  original files were restored before publication.
- Engine CI https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37732408794
  on8f0abe74449b9dbd8cafcf48dcea2b577dcfa461: PASS. Rust workspace91 tests,
  including26 host cases; fmt/clippy, four replay/save/frame regressions and
  no-default-features9 CLI tests all PASS.
- Client CI https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37732408873
  on8f0abe7: Android and Windows both PASS. Windows analyze/test/build, embedded
  package/UTF-8 regression launch and artifacts succeeded; Android APK build succeeded.
  These are automatic CI checks, not new physical-device/audio acceptance.
- main remote8f0abe7 and release/0.2 remote293ba51 read back.
- No local Cargo/Flutter toolchain; product checks above ran in GitHub CI.
- No signed binary release, physical device/audio test or consumer test performed.

## Handoff
STATUS.md → P1_CONTRACT.md → next reviewed Tiny Core slice. Do not restart P0/P1.
Do not publish the development identity as a completed0.3 release.

Completion: 2026-10-08 (Asia/Seoul). Final documentation/index commit does not
change the manifests, lockfile, engine/client sources or workflows tested at8f0abe7.

> Archived after P0/P1 main integration on 2026-10-08. The baseline/tool results below are historical. Current alpha scope and next work are in docs/pentomino/STATUS.md.

# Pentomino P0 — discovery and minimum contract

## Outcome

Establish a source-backed migration baseline and a reviewed minimum public API proposal. P0 produces documents, not implemented 0.3 functionality. Follow Tiny Core → View/Format → Gameplay; Forge remains 0.4.

## Context

2026-10-08: cloned origin main at `293ba513f5727d4a7a0a59476c9a98eb193aaa25`; initial `git status --short` empty. Origin is https://github.com/zizonhyeontae218/Game-Engine-For-GPT.git. Read AGENTS.md, HANDOFF.md, BRIEF.md, orchestration skill, routing, bootstrap packets and gates. Rust workspace remains version 0.2.0, edition 2024. Scout inventory is recorded in docs/pentomino/P0_SCOUT.md. PATH has python3/java, but not cargo/rustc/rustup/flutter/dart/cmake/ninja/adb; absolute-path installations remain UNVERIFIED.

Known handoff failure: Windows run 37486665662/job 112348341230 failed `client/test/final_library_test.dart` digest comparison; build/embedded launch skipped. ZIP timestamps are an UNVERIFIED hypothesis. Preserve digest/save validation. This session has not rerun Windows tests.

## Scope / non-scope

P0-S read-only inventory, P0-A docs-only proposal, P0-R read-only review, parent integration and evidence. No source, dependency, ABI, schema, version, signing, client CI or release changes. External consumer validation requires a separate source-free session; it is not performed here.

## Acceptance evidence

Source-backed inventory, minimum contract and independent review, bounded packets with exclusive ownership, an updated filetree, and a Rundown. Run relevant repository tooling checks and distinguish them from engine tests. Unrun tests and unimplemented functionality are UNVERIFIED.

## Milestones

1. P0: scout → architect → auditor → parent technical integration. No functionality edits before review.
2. Tiny Core / P1: after P0 acceptance, one minimal plugin lifecycle slice, a toy plugin and unload/isolation tests; G1/G2/G6 evidence required. No genre/camera dependency.
3. View/Format / P2: one removable Classic 2D or Top-down adapter selected from actual inventory, then Side View, Vertical Scroll, Top-down with 2D/3D and improved Classic 2D. Each available adapter needs independent G3 evidence; all unimplemented behavior UNVERIFIED.
4. Gameplay: turn combat → realtime combat → open world → interaction/platformer. Declared Core contracts and view capabilities only; prove removability and top-down + realtime + open-world composition (G4).
5. Each release: measured optimization or justified no-change (G7), published public discovery evidence from a separate consumer session (G5). Forge/story/asset creation deferred to 0.4 (G8).

Platform decision: retain Android/Windows client scope pending an explicit 0.3 feasibility decision. No automatic iOS/macOS/Linux/Arch jobs; Linux Rust/headless checks remain development infrastructure. Platform builds/device acceptance UNVERIFIED.

## Decisions

- 2026-10-08: P0 only; HANDOFF and BOOTSTRAP_PACKETS explicitly place first implementation in P1 after reviewed contracts.
- 2026-10-08: shared contracts and integration files remain parent-owned. Scout/auditor have no write ownership. Architect receives one proposal document only.
- 2026-10-08: re-reviewed proposal technically accepted for P0, not runtime acceptance. P1 must freeze stable canonical encoding and numeric budgets before implementation.
- Native executable plugins inside imported games are not authorized; game imports stay data-only. Loader/trust details require the proposal and review.

## Progress

- Repository located and clean main cloned; required skill and role instructions found.
- Actual repository scout dispatched read-only before architecture.
- P0-S inventory complete; architect proposal complete with ownership released.
- Auditor initially required three MEDIUM corrections and one LOW history clarification. Parent integrated them sequentially; auditor re-review approved the P0 proposal technically. Free-function naming corrected.
- Parent accepts P0 documentation deliverables. Next implementation branch is P1, after numeric HostLimits, wire encoding/version and bounded accounting are frozen. All runtime gates remain UNVERIFIED.
- Missing QUICKSTART.ko.md caused initial tooling failures. User supplied the original attachment; parent restored it byte-for-byte. No unrelated engine changes.

## Verification log

- `git rev-parse HEAD`: 293ba513f5727d4a7a0a59476c9a98eb193aaa25.
- Initial `git status --short`: empty.
- `git remote -v`: expected origin, main cloned.
- `python3 scripts/validate_pentaworks.py`: initial FAIL missing QUICKSTART; after original restoration PASS.
- `python3 -m unittest discover -s tests -v`: initial 4/5 PASS, test_roles failed the same missing file; after restoration 5/5 PASS.
- `python3 scripts/release_consistency.py --self-test`: PASS, 19 current documents and release identity.
- Engine/client/runtime tests: NOT RUN — UNVERIFIED. Required Cargo/Flutter tools absent from PATH; this docs-only P0 does not certify engine milestones.
- Final filetree/diff checks are recorded in docs/pentomino/P0_RUNDOWN.md.

## Handoff

Task ownership and negative boundaries are in `pentomino-p0-packets.md`. Do not equate installed PentaWorks configuration with runtime support. Preserve 0.2 product and historical artifacts. Proposals must be labeled UNVERIFIED until implemented and tested.

# FlatLand 0.2.0 finalization

## Outcome
Publish final 0.2.0 / Flutter0.2.0+8 / Androidcode8, identical application ID and
certificate, after documentation, bubble speakers and safe library management pass.

## Context
User explicitly confirmed rc5 physical bubble/engine acceptance and authorized final
promotion after this focused pass. Supersedes rc5's pending physical gate. Current
renderer/gameplay/save/input architecture stays frozen. No rc6 unless a new blocker.

## Scope / non-scope
Current-document consistency gate; optional native-resolved bubble speaker; stable
game_id transactional import/update with typed save mismatch/archive/rollback;
library delete/reorder/self-healing. Android/Windows only. No new rendering systems.

## Acceptance evidence
Rust format/clippy/workspace and existing replay/save goldens; native speaker/resume
and typed error coverage. Windows Flutter update/duplicate/rollback/delete/reorder,
bubble UI tests, embedded/UTF-8/render gates. Android three ABI build, stable signing.

## Milestones
1. Speaker metadata and typed revision failure without save validation weakening.
2. Library transaction, lifecycle-safe cleanup, archive, rollback and management UI.
3. Current docs/version/sample cleanup with focused consistency script.
4. Android/Windows CI, real Windows frames, signed final Drive delivery and main.

## Decisions
Keep immutable content by digest; stable identity/order/settings by game_id. A failed
activation retains old digest and user data. Incompatible save alone allows archival
and fresh start. Embedded mode remains locked. Authored speaker wins, metadata
display_name/name follows, otherwise omit header. Engine behavior stays rc5.

## Progress
Complete. Optional speaker/fallback and typed revision errors are implemented;
transactional stable-ID updates/archive/rollback, leases, management/delete/reorder
and self-healing are verified. Current docs/version/sample are coherent. Hosted
Rust/Flutter/Android/Windows checks pass. Signed final artifacts and14 Drive files
were uploaded and read back; existing rc1–rc5 artifacts are preserved.

## Verification log
Local65Rust tests, strictclippy, format and Flutteranalysis pass. All four replay/save
goldens pass unchanged.17-document consistency gate/self-test passes. Hosted final
Flutter/update/management tests and Android/Windows builds pending.

## Handoff
Repo /workspace/Game-Engine-For-GPT. Private stable key /workspace/signing/android;
never copy to Git/public Drive. Publish new0.2.0 folder under verified FlatLand Demos
parent1ygYLP34cQx2fwzQAK974fZbGy_JZOYsR; preserve rc1–rc5. Physical acceptance is
user-reported for rc5; final modified library UI is CI-verified unless later tested.

Review refinements: blank speaker falls through to valid metadata; NUL metadata is
ignored. Native compatible update (new bundle digest, same game content) must retain
the save. Management widget IO uses real async execution before assertions. Initial
Windows Flutter pass still running; restart coherent verification on revised source.

Hosted regressions caught delayed loading publication while awaiting frame/audio
shutdown. Restore synchronous loading/session-clear contract before awaiting work;
keep the existing lifecycle assertions. Update only the Korean sample title assertion
to its intentional final name, preserving import/launch/render coverage.

Latest source9974501 passes hosted engine acceptance and Android release; Android
ID/name/code8/three ELF ABIs verified. Windows test run was stopped to investigate
an IO/FakeAsync stall. Keep all assertions, run each filesystem-backed widget flow
inside one real-async scope, and finish menu transitions before waiting on storage.
No simulation/render changes. Re-run full Windows/Android verification on that patch.

Hosted Windows14e4d71:35 tests pass, including all new library/speaker/manager tests
and lifecycle/frame-failure rollback. Remaining old popup/layout test inspected
after a fixed150ms before first-frame readiness. Replace that wait with bounded
UI-ready polling, keeping every original popup/orientation/layout assertion.

## Final evidence —2026-10-06
Verified build cdb7eb1d336dee95ea2b1e4d82ddc93d478e4098:
engine37483880616 succeeds(65 workspace +9 headless CLI tests/four goldens);
clients37483880756 succeeds(36 Flutter tests, Android three ABIs, Windows release,
Korean UTF-8 pipeline and181 real capture evidence files). Signed code8 APK v2/v3
certificate equals pinned and actual rc5 certificate. Source/portable/embedded game
entries match; camera/world and battle/bubble/save proofs pass. Real frames reviewed.
Delivery:0.2.0 folder1iJPNZLzYvo3CRjxqfKQAOmqC0vNIjM_f,14 verified downloadable
files. See docs/FLATLAND_RELEASE.md for final metadata and measured hashes.
No new physical-device/audio acceptance is claimed. Promote directly to v0.2.0.

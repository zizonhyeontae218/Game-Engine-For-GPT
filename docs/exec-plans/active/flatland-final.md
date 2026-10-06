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
Implemented optional speaker/fallback, typed save revision errors, library leases and
serialized transactional update/rollback with archive, management/delete/reorder and
self-healing. Current docs/version/sample promoted for final build verification,
focused consistency gate passes. No final artifacts published until CI passes.

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

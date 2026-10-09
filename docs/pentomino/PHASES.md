# Pentomino milestones — current user scope

The user's P2 request supersedes the bootstrap phase numbering. Preserve
Tiny Core → View/Format → Gameplay; a handoff does not implement P3.

**P0 Map 0.2 → 0.3:** verified source/build/test inventory and initial architecture.
Merged; historical evidence retained.

**P1 Scalar lifecycle host:** isolated deterministic lifecycle, scoped i64 resources,
RNG/events, rollback, discovery and canonical save1. Merged.

**P2 Complete game-data Tiny Core:** owner history isolation, Scene/Entity identity,
bounded declarative typed records, tick-bound input actions, save2/discovery and
explicit legacy snapshot adapter. Completed: internal acceptance and separate user-run
public Core GPT Work23/23 accepted; all P2 tests additionally accepted by user report on 2026-10-09. No Camera/View/Format/Gameplay in P2.

**P3 Camera & View/Format:** completed; user reported all tests successful on
2026-10-09. Independent views/formats, multi-camera transforms and saved smooth
transitions are implemented. Full3D GPU renderer remains outside this scope.

**Later Gameplay:** turn combat, realtime combat, open world, interaction/platformer
as independent plugins after view contracts exist. Forge/generation remain0.4.
Alpha hardening and separate consumer checks apply at every milestone.

Every phase returns a Rundown with executed commands/artifacts/observations.

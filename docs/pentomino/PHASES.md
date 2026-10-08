# Pentomino milestones — current user scope

The user's P2 request supersedes the bootstrap phase numbering. Preserve
Tiny Core → View/Format → Gameplay; a handoff does not implement P3.

**P0 Map 0.2 → 0.3:** verified source/build/test inventory and initial architecture.
Merged; historical evidence retained.

**P1 Scalar lifecycle host:** isolated deterministic lifecycle, scoped i64 resources,
RNG/events, rollback, discovery and canonical save1. Merged.

**P2 Complete game-data Tiny Core:** owner history isolation, Scene/Entity identity,
bounded declarative typed records, tick-bound input actions, save2/discovery and
explicit legacy snapshot adapter. Internal completion candidate; separate user-run
external GPT Work verification required. No Camera/View/Format/Gameplay in P2.

**P3 Camera & View/Format:** next focused design/development after the P2 external
gate. Design camera lifetime/serialization, coordinate transforms, replaceable
views/formats and legacy compatibility outside Tiny Core. Then develop tested
Classic2D/Side/Vertical Scroll/Top-down contracts. P3 is UNVERIFIED / unimplemented.

**Later Gameplay:** turn combat, realtime combat, open world, interaction/platformer
as independent plugins after view contracts exist. Forge/generation remain0.4.
Alpha hardening and separate consumer checks apply at every milestone.

Every phase returns a Rundown with executed commands/artifacts/observations.

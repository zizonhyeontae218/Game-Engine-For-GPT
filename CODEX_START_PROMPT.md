# Continue work on GE4G

GE4G / GameEngineForGPT 0.1 "Basement" is implemented in this repository. Read `AGENTS.md`, inspect relevant source and consult the execution plan and `docs/RELEASE_NOTES.md` for established evidence.

Preserve the real deterministic headless runtime and shared CPU renderer. Maintain explicit public schema versions. Use the implemented CLI to observe state and events; do not recreate the scaffold or assume human acceptance has been performed.

For a new multi-stage change, create an ExecPlan using `.agent/PLANS.md`. Complete requested implementation and relevant verification, update `F(x).md` when stable cross-system identifiers change, and run the filetree update/lint helper.

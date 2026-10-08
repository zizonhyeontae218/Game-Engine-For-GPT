---
name: pentomino-orchestrate
description: Coordinate GE4G v0.3 Pentomino milestone work across Codex workers with file ownership, plugin contracts, and release gates.
---

# Pentomino dispatch workflow

Use for multi-file milestones, ABI changes, module extraction or release preparation. Do not activate for a one-line fix.

1. Read `docs/pentomino/HANDOFF.md` and `docs/pentomino/BRIEF.md`; read `docs/pentomino/GATES.md` only if contracts, integration or release are involved. For the first P0 start, always call `pentomino_scout` read-only, even though the handoff provides a baseline. Create an active ExecPlan under `docs/exec-plans/active/` using `.agent/PLANS.md`. Record observed HEAD, dirty files, tool availability and the known Windows fixture failure.
2. Identify **one vertical goal** and choose the phase: Core, View/Format, Gameplay or Release. Track unknown assumptions explicitly.
3. Write bounded task packets using `templates/TASK_PACKET.md`: owned files, read-only dependencies, deliverables, negative boundaries, deterministic tests and stop conditions.
4. Route by `docs/pentomino/ROUTING.md`. Use 1–3 subagents and parallelize only independent work. Architecture decisions and shared files belong to the parent until a single owner is assigned.
5. Integrate sequentially at ABI boundaries. Check contract reviewer independently from implementer. For alpha, package released public artifacts and invoke `$pentomino-consumer-eval` in a **separate Codex session / workspace**; never claim this is achieved by ordinary in-repo spawning.
6. Require gate evidence, with `UNVERIFIED` for tests not run. End every workstream with the Rundown format in `templates/RUNDOWN.md`.

### Dispatch rules

- The parent orchestrator is responsible for final integration and conflict resolution; child messages are proposals/evidence, not automatic truth.
- A worker may read dependencies but edits only explicitly owned files. No concurrent writes to `AGENTS.md`, `.codex/config.toml`, dependency lockfiles or shared Core APIs.
- Core ABI breaking change: stop, write compatibility/migration note, seek parent approval before implementation.
- Repo state differs from plan: revise packet instead of fabricating files.
- Never automatically claim a release passed; `scripts/check_gate_report.py` only verifies evidence completeness, not correctness of engine behavior.

# PentaWorks 0.3 agent setup

## Outcome
Install the supplied 10 workers and 2 skills into the actual GE4G repository so a
fresh Codex session can receive the unchanged QUICKSTART P0 prompt.
Preparation is complete; no Pentomino engine functionality is implemented here.

## Context
2026-10-08 request: read Drive Soul.md and GitHub GE4G, correctly set up agent files
before the next worker starts 0.3. Baseline ed3301c7af12ac36dd0b5074d79693d203aeeb28.
Soul.md, Drive final folder/README, repository rules/manifests and GitHub tag jobs
were read. HANDOFF.md records verified facts and the unresolved Windows failure.

## Scope / non-scope
Merged existing AGENTS.md, installed .codex/agents and .agents/skills, supplied
brief/contracts/gates/templates/tooling, added HANDOFF and updated FILETREE.
Preserved root README, runtime/client/game code, versions, workflows, signing,
release tags and existing Drive files. No engine implementation or consumer test.

## Acceptance evidence
- python3 scripts/validate_pentaworks.py: PASS; 10 role files, 2 skills, config/templates.
- python3 -m unittest discover -s tests -v: 5/5 PASS (bundle tooling only).
- python3 scripts/release_consistency.py --self-test: PASS (19 docs/release identity).
- python3 scripts/filetree.py update followed by lint: PASS.
- Codex CLI 0.159.0-alpha.3 app-server --strict-config: config/read returned project
  [agents] enabled=true/max_concurrent_threads_per_session=3; skills/list returned
  both Pentomino skills enabled, with no relevant discovery errors. Used a
  process-local trusted-project override; did not modify user/global trust.
- Actual custom-role spawning, sandbox enforcement, plugin behavior, runtime tests,
  Windows failure repair and external consumer isolation: UNVERIFIED.

## Milestones
1. Ground repository/Soul/package: complete.
2. Merge setup and baseline handoff: complete.
3. Validate config, discovery, packaging and documentation: complete.
4. Publish setup for next session: recorded in the setup commit; verify remote SHA.

## Decisions
- 2026-10-08: retain supplied current standalone custom-agent TOML format, verified
  against https://developers.openai.com/codex/multi-agent. No model pin.
- Preserve existing AGENTS and README; package README stored as SETUP.md.
- HANDOFF summarizes source facts without copying the entire private history.
- First P0 always uses read-only Scout; architect/auditor precede implementation.
- Contract approval is parent technical integration; routine work does not require
  repeated user permission. Consumer remains a separate workspace/session.
- UTF-8 explicitly used by new helper scripts for Korean Windows paths/content.

## Progress
Complete: setup and checks. Next: fresh session receives QUICKSTART prompt and
creates an active Pentomino implementation ExecPlan after read-only discovery.

## Verification log
Only setup/doc/tooling checks ran. GitHub readback confirmed tag clients run
37486665662 Windows failure / Android success; this task did not rerun product CI.
CI on future commits must be interpreted separately from these local setup checks.

## Handoff
Use QUICKSTART.ko.md → docs/pentomino/HANDOFF.md → BRIEF.md. Preserve current
0.2 ABI/schema/save/certificate invariants. Do not promote proposal signatures to
shipped contracts. Known Windows fixture digest issue remains unresolved.

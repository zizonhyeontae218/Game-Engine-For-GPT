---
name: pentomino-consumer-eval
description: Evaluate published GE4G Pentomino API discoverability and composability from source-free public artifacts in a separate workspace.
---

# Blind consumer evaluation

Use after a distributable alpha exists; do not treat in-repository code review as a blind evaluation.

1. Publisher creates an explicit allowlist of public README, public API docs, usage skills and installable package/builds. Use `scripts/stage_consumer_bundle.py` with `--root` and `--allowlist`, and a destination **outside** the source tree; inspect its manifest.
2. Start a *new*, separately scoped Codex session with working directory set to that destination. The consumer must not have filesystem/tool/connector access to internal GE4G checkout. Read-only TOML guidance alone does not enforce that.
3. Give only `consumer/CONSUMER_TASKS.md`, public materials and generated manifest. Attempt: find relevant package, install it, build a minimal composition, switch view, detach a gameplay plugin, inspect errors/rollback.
4. Record first-contact friction, absent docs, API surprises, broken instructions, time to first playable, and evidence in `templates/CONSUMER_REPORT.md`. Unknown or unavailable package => BLOCKED, not PASS.
5. Parent takes anonymized findings only, fixes public contract/docs, and repeats after a newly packaged build.

Never copy secrets, hidden source, private notes or test oracle code. Never infer success from README examples alone.

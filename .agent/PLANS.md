# ExecPlan policy

Use an ExecPlan for work that is multi-stage, architectural, likely to span multiple agent runs, or risky to resume from chat context alone.

Small/local edits do not need a checked-in plan.

## Location

Active:
`docs/exec-plans/active/<slug>.md`

Completed:
`docs/exec-plans/completed/<slug>.md`

## Required sections

1. **Outcome** — observable end state
2. **Context** — only facts needed for this task
3. **Scope / non-scope**
4. **Acceptance evidence**
5. **Milestones** — independently verifiable slices
6. **Decisions** — decision + reason + date/status
7. **Progress** — completed / next / blocked
8. **Verification log** — commands and meaningful results
9. **Handoff** — what a fresh agent must know to continue

## Plan behavior

- Keep the plan synchronized with reality.
- Prefer vertical slices over creating an entire architecture skeleton at once.
- When a decision changes, preserve the old decision as superseded rather than rewriting history.
- Do not paste huge command output into the plan; summarize it and keep the exact artifact elsewhere only when useful.
- Move the plan to `completed/` when acceptance evidence exists.

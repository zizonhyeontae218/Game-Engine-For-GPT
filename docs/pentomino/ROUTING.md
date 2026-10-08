# Dispatch routing, ownership & topology

| Task type | Primary owner | Independent checker | Allowed parallel peer |
|---|---|---|---|
| Existing code structure | `pentomino_scout` | parent | none needed |
| Contract proposal | `pentomino_architect` | `pentomino_auditor` | `pentomino_scout` (read only) |
| Shared runtime/plugin loader | `pentomino_core` | `pentomino_test` + auditor | docs author only with disjoint files |
| Camera/view format | `pentomino_view` | `pentomino_test` | plugin worker **only after** frozen shared interfaces |
| Gameplay feature | `pentomino_plugin` | test worker + auditor | another plugin worker only on truly separate directories |
| Engine integration regression | `pentomino_test` | auditor | read-only research |
| Performance, ongoing optimization | `pentomino_optimizer` | `pentomino_test` | doc worker |
| Alpha release report | `pentomino_release` | auditor | consumer in separate clean session |
| Public-first experience | `pentomino_consumer` | release curator | **separate workspace/session only** |

## Five work cells

- **Discovery:** Scout + Architect decide verified state and interface shape.
- **Foundation:** Core + View build the shell and spatial presentation.
- **Extension:** Plugin + Test implement composable mechanics with independent evidence.
- **Challenge:** Auditor + Consumer try to break internal/public assumptions.
- **Delivery:** Optimizer + Release preserve performance and communicate proof.

## Dependency schedule

- Core contract v0 proposal precedes View/Format runtime integration.
- View public interfaces precede mixed-mode gameplay composition testing.
- Gameplay plugins depend **on Core contracts** and declared view capabilities, not on private View internals.
- Consumer acceptance follows published alpha artifact; don't dispatch to inspect raw developer working tree.
- No generic unconditional 10-agent swarm. Default 1–3 children; choose low-parallel mode for ABI-heavy edits.

## File ownership protocol

Each task packet contains explicit `OWNED:` path patterns, `READ-ONLY:` dependencies and `FORBIDDEN:` others. Ask parent when overlap discovered. Serialize lockfile, central API, and shared test-fixture edits. If using Git worktrees, merge one role at a time and rerun affected tests.

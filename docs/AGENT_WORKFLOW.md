# Agent workflow notes

This file contains workflow guidance that is too specific to keep permanently in `AGENTS.md`.

## Context discipline

- Start from the task and the nearest source of truth.
- Do not preload every architecture/product/test document.
- Search narrowly before reading whole large files.
- Prefer structured command output and repository-local documentation over reconstructing state from chat history.

## Vertical slices

For Basement, a working thin path is more valuable than ten empty crates.

A preferred early sequence is:
`scene file -> parser -> runtime world -> replay input -> framebuffer -> CLI JSON -> test`

Only split crates/modules when a real boundary has emerged.

## Parallel/subagent work

If the harness supports subagents, use them for independent bounded work such as:
- researching a library/API,
- reviewing a proposed schema,
- independently reproducing a bug,
- checking tests/docs against implementation.

Avoid concurrent edits to the same core files unless the harness isolates worktrees and integration is explicit.

The primary implementation agent owns final integration and verification.

## Observation before mutation

For debugging:
1. reproduce
2. inspect structured state/events
3. identify the first divergence
4. change the smallest responsible layer
5. rerun the narrow reproduction
6. then run broader affected tests

Do not debug graphical symptoms solely by staring at source if a capture/trace can expose the runtime state.

## Tool output

Keep tool output bounded.
Prefer:
- targeted test names,
- JSON queries,
- filtered traces,
- concise compiler diagnostics.

Do not dump entire repositories or giant logs into model context when a focused command can answer the question.

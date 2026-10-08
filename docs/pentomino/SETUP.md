> Installed in the actual GE4G repository. Use the root QUICKSTART.ko.md and HANDOFF.md. The repository README.md and AGENTS.md were preserved/merged. This file retains the supplied package overview; generic copy instructions below apply only to manual installation elsewhere.

# PentaWorks v1.0 — GE4G Pentomino Codex Worker Set

An agent-first, contract-gated worker system for **GE4G v0.3 Pentomino**. This repository overlay configures Codex; it does **not** itself implement the game engine.

## Install

Copy the **contents** of this folder into the root of the existing GE4G repository (do not overwrite project-specific `AGENTS.md` or `.codex/config.toml` without merging them). Restart Codex if custom roles do not appear.

- `.codex/agents/*.toml`: 10 project-scoped Codex subagents.
- `.codex/config.toml`: conservative concurrency cap (three children).
- `.agents/skills/pentomino-orchestrate/SKILL.md`: work-order orchestration.
- `.agents/skills/pentomino-consumer-eval/SKILL.md`: isolated consumer test procedure.
- `AGENTS.md`: concise project-wide rules; detailed workflows live in skills/docs.
- `docs/pentomino/`: source-aligned scope, architecture gates, responsibility matrix, example dispatches.
- `templates/`: work packets, handoff and release evidence.
- `scripts/`: self-validation, consumer-safe packaging, release gate evidence check.

## Start

From a new Codex session in the GE4G repository:

```text
Use $pentomino-orchestrate. Inspect this repository as GE4G v0.3 Pentomino. First dispatch pentomino_scout for a read-only architecture map; create a bounded milestone plan for Tiny Core → View/Format → Gameplay. Do not assume a programming language or create speculative engine APIs. Produce a Rundown and first verified task packet.
```

To validate only these configuration files:

```bash
python3 scripts/validate_pentaworks.py
python3 -m unittest discover -s tests -v
```

> **Important:** Passing bundle validation does not mean GE4G engine or plugin tests passed. Gate evidence remains BLOCKED/UNVERIFIED until real engine tests run.

## Routine

1. **Survey:** Scout identifies actual folders, language, build tools, tested behavior.
2. **Specify:** Architect records boundary proposals and acceptance tests; parent approves any core ABI breaking change.
3. **Build:** Route small, file-disjoint tasks to Core / View / Plugin / Test workers. Never assume parallel edits of shared files are safe.
4. **Challenge:** Auditor reviews module boundaries; Consumer tests released public artifacts from a *different workspace*.
5. **Ship:** Perf checks regressions; Release builds evidence and Rundown.

For small tasks use one worker or the main agent, not ten. Default maximum open subagents is 3; change it only when file ownership is separated and benefits are clear.

## Source & implementation policy

The attached document is named **GE4G v0.3 Pentomino**; this bundle retains its `GE4G` spelling. If the actual repository calls itself `GE4C`, normalize only after inspecting repo identity, not by guessing. The system is implementation-language agnostic. API names and manifests in these documents are **contract proposals**, not existing engine functions.

**Out of scope for v0.3:** story/world setting generation, 2D/3D asset synthesis/processing, Forge features (v0.4 Mr. Smith).

## Limits

Project-scoped agent support and available models depend on the current Codex client. The TOML intentionally does not pin models. Parent runtime sandbox/approval overrides may supersede individual agent defaults. A true black-box consumer test requires launching Codex **separately from the GE4G source checkout**, inside an isolated workspace containing only allowlisted public docs and distributable binaries/packages.

Documentation: https://learn.chatgpt.com/docs/agent-configuration/subagents and https://learn.chatgpt.com/docs/build-skills

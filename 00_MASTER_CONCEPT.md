ILCX™ 4H target — actual human execution is pending.

# GameEngineForGPT — master concept

## One sentence

**Do not connect Codex to a human-first game engine; build an engine whose native operating surface is already friendly to Codex.**

## Problem

Traditional engines assume a human developer can continuously:
- inspect a scene visually,
- click an editor,
- understand hidden editor state,
- watch runtime behavior,
- navigate proprietary project metadata,
- and manually correlate a graphical symptom with code.

A cloud coding agent often has none of those affordances. Even when an engine can be automated, the agent spends context and tool calls reconstructing state that the engine already knows.

GE4G makes that state explicit.

## Core idea

Every meaningful engine operation should have an agent-legible path:
- create/build
- validate
- run
- replay
- inspect
- capture
- diagnose
- test

The engine should expose **what happened**, not merely whether a process exited.

Examples:
- Which entity is colliding with what?
- What scene is active?
- What persistent state changed?
- Which trigger fired?
- What did the camera render?
- Which frame/tick first diverged from an expected replay?

## Dual surface

GE4G has two equal surfaces:

### Human runtime surface
A normal playable game window.

### Agent runtime surface
Headless execution plus deterministic state, event, collision, and framebuffer inspection.

Neither is allowed to have separate gameplay semantics.

## Basement philosophy

0.1 is called **Basement** because it builds the layer every later GE4G feature stands on.

Basement proves the architecture with one tiny 2D demo. It does not attempt:
- a polished editor,
- 3D,
- a complete asset pipeline,
- sophisticated physics,
- networking,
- mod/plugin ecosystems,
- or a large custom language.

## Design priorities

In order:
1. observability
2. determinism
3. correctness
4. simplicity
5. agent editability
6. local human playability
7. performance

Performance matters, but not at the cost of hiding behavior behind complex machinery in 0.1.

## Agent-native does NOT mean

- an LLM inside every game,
- cloud-only runtime,
- OpenAI account requirement,
- prompts as a gameplay scripting language,
- an autonomous agent shipping unreviewed releases.

GE4G is a conventional deterministic engine with an unusually inspectable developer interface.

## Permanent launcher direction

Human runtime delivery uses the Flutter GE4G client. Mobile imports portable Basement games; Windows/Arch games ship with their client and native Rust runtime already embedded. Digital brutalism and live, separately stored layout/mapping JSON are the common design language. This applies to every future runner; see [LAUNCHER_PHILOSOPHY](docs/LAUNCHER_PHILOSOPHY.md).

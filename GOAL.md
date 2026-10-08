# Goal — GE4G 0.3.0 "Pentomino" (alpha)

main develops the small independent Core, replaceable View/Format and composable
Gameplay described in `docs/pentomino/BRIEF.md`. Current version is
0.3.0-alpha.1. P1 is a scalar lifecycle subset; remaining Tiny Core/view/gameplay
work follows `docs/pentomino/STATUS.md`. Forge is deferred to 0.4.

## Preserved Basement foundation goal

The original goal below describes the inherited implementation and regression baseline.

Build the smallest real engine foundation that demonstrates this claim:

> A coding agent can implement and verify a small 2D game through repository files and CLI tools without needing a graphical editor.

## Success artifact

An `examples/basement_demo/` game containing:
- a player,
- at least one blocking wall,
- one NPC interaction,
- one trigger/door changing scenes,
- one persistent value that survives save/reload,
- one scripted input replay,
- one deterministic headless framebuffer capture.

## Required developer experience

An agent should be able to answer these from CLI output without scraping source code:
- What scene is loaded?
- What entities exist?
- Where is a named entity?
- What components does it have?
- What collision/trigger events occurred during a replay?
- What state values changed?
- Did a replay finish deterministically?
- Where was a frame capture written?

## Explicitly outside 0.1

3D, networking, game-authoring editor GUI, visual scripting, hot asset authoring UI, general rigid-body physics, navigation mesh, ECS optimization work, consoles, marketplace/plugins.

## Runtime client extension

Flutter mobile import/play clients and Windows clients (Linux/Arch deferred until0.3) embedded in each game distribution are required. Joystick + Z/X/C/Space, digital brutalism and live per-game JSON profiles follow [the permanent launcher philosophy](docs/LAUNCHER_PHILOSOPHY.md).

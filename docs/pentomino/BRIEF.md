# Pentomino 0.3 design brief (from the supplied PentaWorks package)

**Project title in the attachment: GE4G v0.3 — Pentomino.**

This is the target architecture, not a claim about shipped 0.2 functionality.
The standalone original 3-page concept was not supplied or independently verified.
Read HANDOFF.md for the verified repository baseline and compatibility constraints.

## Goal

Convert GE4G 0.2 from a monolithic engine into a **small, open core + separately replaceable View/Format libraries + independent Gameplay plugins + agent-facing discovery**. New features should normally extend plugins rather than modify Core.

## Design laws

1. **Tiny Core**: Scene/Entity, Event, Resource, Plugin Loader, Serialization, Input Abstraction, Agent-facing API — only shared primitives. These are conceptual responsibilities, not confirmed implementation modules.
2. **View/Format First**: camera, coordinates, layers, sprites and 3D assets/background interpretation before combat/world systems.
3. **Gameplay as plugins**: turn combat, real-time combat, open world, interaction, platformer — independently removable where contracts allow.
4. **Composable**: Top-down + real-time combat + open world must be structurally supportable; avoid genre inheritance trees.
5. **Agent-first**: public docs/API/skill metadata should be sufficient to discover, combine and execute functionality.

## Build order

A. Minimal Core and plugin contracts → B. Side View / Vertical Scroll / Top-down (2D sprites + 3D scene) and improved Classic 2D → C. Turn combat → realtime combat → open world → interaction/platformer → D. small optimization in every release.

## External validation

Engine implementation agent != consumer agent. Alpha → external consumer using README/public API/Skills/Plugin docs → collect blockers → next release. Make actual source isolation part of the test setup, not merely a prompt.

## Completion conditions

- No genre/camera/combat assumption in Core.
- Major view libraries installable/replaceable separately.
- Removing one gameplay plugin leaves Core and unrelated plugins operational.
- Agent finds needed capabilities without reading entire engine internals.
- Feature additions normally use new plugins rather than core rewrites.

**Not in 0.3:** story/setting generation, 2D/3D asset creation/processing and Forge functionality, deferred to v0.4 Mr. Smith.

The original brief defines principles, **not** actual API signatures, path conventions, language/runtime, test frameworks, plugin manifests or performance targets. Those require repository discovery and design decisions.

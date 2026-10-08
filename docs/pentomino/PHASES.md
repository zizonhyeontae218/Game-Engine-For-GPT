# Milestones without assumptions about programming language

**P0 Map 0.2 → 0.3:** Scout records source tree, entrypoint, build/test, game examples, inferred engine interfaces, dependency graph. Architect drafts contracts with explicit uncertainty; preserve compatibility if feasible.

**P1 Tiny Core:** Establish lifecycle boundaries for scene/entity, event, resource, input, serialization, plugin loader and public discovery. Freeze the smallest needed v0 API; test plugin load/unload and clear failure cases. Do not bake in camera/combat rules.

**P2 View/Format:** In separate libraries, expose Side View, Vertical Scroll, Top-down (2D sprite with 3D scene support) and improved Classic 2D. Work out coordinate transforms, camera behavior, layer mapping, required backend capabilities and compatibility tests. Don't start gameplay plugins until at least one tested view contract is usable.

**P3 Gameplay:** Turn-based combat first, then realtime combat, open world, interaction/platformer. Each plugin has a manifest/capability description and can be removed without disabling unrelated systems. Test mixed mode compositions after required pieces exist.

**P4 Alpha & hardening:** publish docs/SDK examples; physically separated consumer tests; contract audits and regression/benchmark measurements. Include minor Core/main-plugin optimization in each subsequent release when justified by evidence.

Every phase returns a Rundown. Stage completion is based on tests/gates, not number of files written.

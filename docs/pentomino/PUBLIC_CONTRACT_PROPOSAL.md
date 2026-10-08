# Candidate plugin public contract (DESIGN PROPOSAL ONLY)

**Not an implemented interface.** Adapt to the real language/framework after P0 discovery.

Minimal manifest information (not a mandatory concrete JSON schema):

- `id` stable unique namespace, `version` semantic version, `engine_api` supported range.
- `entry` public install/discovery target; `provides` capabilities (`view.topdown`, `gameplay.turn`, etc.).
- `requires` required capabilities and version ranges; `optional` optional capabilities.
- `lifecycle` contract: install/register → enable/start → stop/disable → unload/dispose; errors must not poison unrelated modules.
- `state` ownership and serialization/import/export boundaries.
- `agent` short synopsis, prerequisites, public callable surface and runnable example.

## Interface testing questions

- How does the Core resolve cycles, missing/incompatible versions and duplicate providers?
- Can two implementations of the same view capability be exchanged without assuming a global singleton renderer?
- If a gameplay plugin unloads, are its event listeners/resources cleaned up?
- Can an external agent discover and assemble capabilities using the public index, never private type names?
- Are constraints for hybrid 2D sprites + 3D background explicit and testable?

Do not freeze entrypoint names, data serialization format, engine runtime or exact compatibility rules without a working prototype and migration review.

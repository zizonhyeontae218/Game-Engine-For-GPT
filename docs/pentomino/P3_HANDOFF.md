# P3 handoff — Camera & View/Format focused development

**Next stage is Camera & View/Format. P2 does not implement P3.**
The user's separate public Core GPT Work23/23 report is accepted; P2 ExecPlan
is completed. External legacy execution remains UNVERIFIED. P2 source is
`48d7f4aa6932c9031afee722e0c58f7756b864c8`; package/commands in P2_RUNDOWN.md.
No final release, merge, platform expansion or signed client is implied.

## Verified boundary to retain

- P2 CoreHost contract2/save2 contains identity/lifetime, bounded declarative typed
  data, RNG/events, actions, lifecycle and discovery. No position, size, orientation,
  camera/viewport/sprite/layer/coordinate or gameplay role is intrinsic to objects.
- P1 root Host/save1 remains a separate unchanged scalar compatibility surface.
- LegacyBridge owns authoritative World outside callbacks. Typed imports are
  immutable snapshots; existing legacy runtime remains the real game execution.
  Presentation and legacy save/resume are preserved outside Core.
- ABI1, project/save schema1/2, release/0.2 and historical binaries/Android identity
  remain preserved. Windows ZIP fixture timestamp/digest failure cause is UNVERIFIED.

## P3 design before implementation

Use read-only scout for actual legacy camera/render/view/save seams; architect
proposes a minimal public camera/view/format contract, auditor checks isolation,
ownership, deterministic save/replay, bounds and explicit compatibility. Parent
freezes the contract and assigns disjoint files before implementation.

Design camera identity/lifetime and persistence, coordinate transforms, view
selection/switching, layer/asset/backend capabilities and removable view behavior
in separate plugins/adapters. Keep presentation changes from altering authoritative
simulation/identity/RNG/collision or existing legacy saves. Decide compatibility
and migration explicitly; do not temporarily add camera structs to Tiny Core.
Test one vertical public view slice before extending Classic2D, Side, Vertical
Scroll and Top-down. 3D interpretation requires its own supported contract/evidence.

All these features are currently UNVERIFIED / unimplemented. Gameplay plugins
(combat/world/interaction/platformer), Forge, generation and dynamic native loading
are later work. No preliminary P3 renderer/camera implementation is in the P2 diff.

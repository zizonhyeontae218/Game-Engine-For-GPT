# GE4G 0.3.0 — Pentomino alpha

main is the next-generation development line. Pentomino is the name of GE4G
0.3.0, not a separately versioned product. Current development identity is
**0.3.0-alpha.1**, with Flutter **0.3.0-alpha.1+9**. No final 0.3.0 release,
signed alpha delivery or physical acceptance is claimed.

## Branch policy

- main: Pentomino alpha development, including P0/P1. New architecture work lands
  here in bounded reviewed slices; completing the full engine is not a prerequisite
  for main integration.
- release/0.2: FlatLand maintenance baseline, initially
  293ba513f5727d4a7a0a59476c9a98eb193aaa25. The released v0.2.0 tag and Drive
  artifacts are unchanged. Maintenance fixes can be selectively ported to main;
  never bulk-merge alpha version/architecture changes back into release/0.2.
- Short milestone branches/PRs target main. Old P0/P1 branch names are historical.
  Save/archive/schema migration decisions are explicit, not inferred from version.
  P3 is stacked on accepted P2 branch2159304 while P2 PR3 remains open; main
  still contains P0/P1 until that integration occurs.

## Implemented and missing

P0 discovery/review and P1 scalar lifecycle are integrated. The ge4g-pentomino
crate has no dependency on legacy core/project/runtime/render2d/client crates.
P1 includes registration, exact capability bindings, unload, scoped resources,
deterministic RNG, transactional failure rollback, discovery and canonical save/
atomic restore. P1_CONTRACT.md specifies its implemented scalar public surface.
P0_API_PROPOSAL.md is the broader proposal, not shipped API.

Completed P2 adds `ge4g_pentomino::p2::CoreHost`: per-owner event
history256/pending128, Scene/Entity identity/lifetime, bounded declarative typed
records, next-tick bool/bounded-i64 actions, transactional rollback, save2 and
public discovery. P1/root scalar API and save1 remain unchanged (including global
retention); opt into P2 for the corrected independent budgets.

`ge4g-pentomino-legacy` is a separate compatibility adapter: authoritative legacy
World stays outside Core callbacks; immutable projections initialize actual Core
objects/records via public transactions. Imported plugin ticks are no-op snapshots,
not continuously delegated legacy gameplay. Existing game/CLI/client execution
uses inherited FlatLand code. Preserve schema/save/ABI1 and release/0.2.

External source-free public Core GPT Work23/23 is ACCEPTED from the user's
separate report; the tested Core rlib hash matches the delivered SDK. External
legacy runtime/game execution remains UNVERIFIED. See P2_RUNDOWN.md for concrete internal evidence and
../public/pentomino-p2/ for public test package instructions. P2 is not a final release or client support expansion.

P3 internal candidate adds separate ge4g-pentomino-view and -view-legacy crates:
four replaceable Format/View policies, independent multiCamera lifecycle,
explicit World/View/Camera/Screen-depth transforms, orthographic/perspective,
composable follow/bounds/zoom/shake and default tick-based smooth2.5D transitions,
canonical presentation save1 and structured discovery. Native production consumes
only ReadFrame; Core and legacy source remain unchanged. Existing fifth Nuvema
content drives real legacy-compatible ground/upright raster composition.
P3_RUNDOWN.md records actual evidence and limitations. **EXTERNAL VALIDATION
PENDING**: internal150 Rust tests (34 new), five legacy regressions and source
audits do not substitute for a separate GPT Work consumer report.
Gameplay, dynamic loading and client execution through Core remain UNVERIFIED.

## Next slice

P3 external consumer validation is the active gate. Public SDK instructions are
../public/pentomino-p3/. Gameplay plugins are the next development stage AFTER
this Camera/View gate; combat/world/interaction/platformer and Forge are not
implemented here. Follow Core → View/Format → Gameplay; Forge belongs to0.4.

## Verification and release policy

- Existing workspace fmt/clippy/test, headless CLI and four legacy replay/frame
  checks remain required regressions; new host tests are part of the workspace.
- cargo test --locked -p ge4g-pentomino verifies unchanged P1 and typed P2.
- cargo test --locked -p ge4g-pentomino-legacy verifies projection/replay boundary.
- python3 scripts/check_p2_boundaries.py verifies dependency/preservation direction.
- python3 scripts/check_p3_boundaries.py verifies readonly native direction and
  unchanged P2/legacy production/tests. P3_CONTRACT.md defines presentation save1.
- Android/Windows are the current client CI matrix; no automatic platform expansion.
- scripts/release_consistency.py checks alpha manifests/lockfile and branch-facing
  documentation while retaining historical FlatLand documentation sanity checks.
  The release/0.2 copy retains its original strict 0.2 identity checks.
- Existing ABI1, game schema1/2 and save semantics are not automatically changed
  by alpha package versions. The experimental host's save version is separate.
- Android application ID and pinned signing certificate are preserved. Build
  number 9 exceeds the historical 0.2 build number 8. Existing binaries/keys are
  untouched. No signing or deployment is performed during this branch transition.
- The past Windows ZIP fixture digest failure remains an unresolved intermittent
  issue; later successful CI runs do not establish its timestamp cause or repair.

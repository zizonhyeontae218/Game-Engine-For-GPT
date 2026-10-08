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

## Implemented and missing

P0 discovery/review and P1 scalar lifecycle are integrated. The ge4g-pentomino
crate has no dependency on legacy core/project/runtime/render2d/client crates.
P1 includes registration, exact capability bindings, unload, scoped resources,
deterministic RNG, transactional failure rollback, discovery and canonical save/
atomic restore. P1_CONTRACT.md specifies its implemented scalar public surface.
P0_API_PROPOSAL.md is the broader proposal, not shipped API.

P2 completion candidate adds `ge4g_pentomino::p2::CoreHost`: per-owner event
history256/pending128, Scene/Entity identity/lifetime, bounded declarative typed
records, next-tick bool/bounded-i64 actions, transactional rollback, save2 and
public discovery. P1/root scalar API and save1 remain unchanged (including global
retention); opt into P2 for the corrected independent budgets.

`ge4g-pentomino-legacy` is a separate compatibility adapter: authoritative legacy
World stays outside Core callbacks; immutable projections initialize actual Core
objects/records via public transactions. Imported plugin ticks are no-op snapshots,
not continuously delegated legacy gameplay. Existing game/CLI/client execution
uses inherited FlatLand code. Preserve schema/save/ABI1 and release/0.2.

External source-free GPT Work acceptance is UNVERIFIED until the user's separate
report arrives. See P2_RUNDOWN.md for concrete internal evidence and
../public/pentomino-p2/ for public test package instructions. Camera/View/
Format, Gameplay, dynamic loading and client execution through Core are
unimplemented / UNVERIFIED. P2 is not a final release or client support expansion.

## Next slice

Receive and address the user's external P2 GPT Work report first. P3 is focused
Camera & View/Format design and implementation outside Tiny Core; P3_HANDOFF.md
defines the boundary. Follow Core → View/Format → Gameplay; Forge belongs to0.4.
No P3 code is implemented in P2. Preserve deterministic Rust authority/data-only
imports and existing legacy behavior through verified adapter slices.

## Verification and release policy

- Existing workspace fmt/clippy/test, headless CLI and four legacy replay/frame
  checks remain required regressions; new host tests are part of the workspace.
- cargo test --locked -p ge4g-pentomino verifies unchanged P1 and typed P2.
- cargo test --locked -p ge4g-pentomino-legacy verifies projection/replay boundary.
- python3 scripts/check_p2_boundaries.py verifies dependency/preservation direction.
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

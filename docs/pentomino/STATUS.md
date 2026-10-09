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
  293ba513f5727d4a7a0a59476c9a98eb193aaa25. The released v0.2.0 tag and original binary bytes
  are unchanged; public delivery moves to GitHub Releases. Maintenance fixes can be selectively ported to main;
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

Scene/Entity/input actions, format-facing typed records, view adapters, gameplay
plugins, CLI/client execution integration and external source-free consumer
acceptance are still UNVERIFIED / unimplemented. Existing games and CLI/client
execution use inherited FlatLand code. Do not label a legacy sample a Pentomino
host demo. The inherited paths are retained for reuse and regression, not as an
indefinite second product on main; replace them only through verified slices.

## Next slice

Design minimum Scene/Entity/input and format-facing records/selection against
P1, with architect proposal and auditor review before implementation. Define the
legacy compatibility boundary; then integrate one removable Classic2D/Top-down
view. Follow Core → View/Format → Gameplay; Forge belongs to 0.4. Avoid a big-bang
rewrite, preserving deterministic Rust authority and data-only imported games.

## Verification and release policy

- Existing workspace fmt/clippy/test, headless CLI and four legacy replay/frame
  checks remain required regressions; new host tests are part of the workspace.
- cargo test --locked -p ge4g-pentomino verifies the current scalar host only.
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

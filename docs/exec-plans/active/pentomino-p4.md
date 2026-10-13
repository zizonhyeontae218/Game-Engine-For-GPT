# P4 Native Bridge & Gameplay Integration

## Outcome
Run a new native Pentomino session through input, fixed Core tick, removable
minimal turn combat, P3 camera, real CPU frame, independent saves and shutdown.
Preserve P0–P3, ABI1, FlatLand saves/packages and release/0.2.

## Context
2026-10-13 clean origin/main: 8622034d1f0082a3178dc2e7262539a50a064c26.
Branch: feature/pentomino-p4-native-bridge. Main acceptance CI PASS:
https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37926779602.
Read AGENTS.md, orchestrate skill, STATUS/NEXT/P2/P3/HANDOFF/BRIEF/GATES,
PLANS.md, routing and task packet. Drive Demos/GE4G P3 candidate README was
read; its historical pending external test note is superseded by the repository's
2026-10-09 user acceptance. No Drive sharing or artifacts are modified.
Historical Windows ZIP fixture failure remains in HANDOFF, distinct from current CI.
Rust tools initially missing; installation underway. gh and Python available.

## Scope / non-scope
New Rust native bridge and linked JSON-configured two-combatant Gameplay sample.
No Core rewrite, FlatLand gameplay delegation, ABI1 change, new full renderer,
Flutter redesign, platform delivery, release publication or Forge.

## Acceptance evidence
Targeted native boundary tests, real renderer pixels, deterministic headless/linked
state, split save continuation, malformed request atomicity, stale session handle,
detach/reinstall and camera invariance. Required fmt/clippy/workspace tests, P2/P3
boundaries, five existing replay/frame regressions and repository checks.
Public-only independently compiled consumer example is internal reproducibility;
separate-session external consumer certification stays UNVERIFIED until supplied.

## Milestones
1. Scout actual APIs; architect proposal; independent auditor review.
2. Freeze owned interfaces; implement Gameplay and bridge with disjoint ownership.
3. Independent integration tests and public consumer; required regression floor.
4. Contract, rundown, evidence, P5 handoff and PR; inspect remote CI.

## Decisions
- Parent owns Cargo manifests/lockfile, shared APIs and integration documentation.
- Use 1–3 specialists; proposal reviewer is independent from implementer.
- P2 removal deletes owner state. Preserve that contract; resolve durable battle
  state outside Core before implementation, document actual detach semantics.
- Cross-host failures must stage candidates before replacing a live session.

## Progress
Implementation and internal acceptance complete. Scout mapped public APIs;
architect proposal + auditor design accepted; plugin worker owned only new
Gameplay source/tests; independent test worker owned only new native tests;
parent owned bridge/Cargo/lock/docs/CI integration. Shared file edits serialized.

Specialist packets were provided at dispatch with OWNED/READ-ONLY/FORBIDDEN
boundaries and public API-only requirements. No worker edited shared manifests,
Core/View source or existing game fixtures. Active tasks are now complete.

Literal rule owner removal was chosen over the superseded inert-carrier callback
proposal: two Core ticks per logical input, decision Events then carrier Actions.
Authority provider remains independent and survives actual CoreHost::remove.
Auditor found four real defects; all are fixed with independent regression tests.
Workspace fmt/clippy/test (169), P2/P3/P4 checks, five legacy golden replays,
public examples, C linkage and ctypes calls PASS. Source-free packaging, PR and
remote CI finalization remain next. External new-session Consumer certification
cannot be supplied by ordinary in-repo workers and remains UNVERIFIED.

## Verification log
- git fetch origin main --tags: PASS; branch created from current main.
- gh run list: current main acceptance success (link above).
- Initial rustc missing was resolved by rustup stable1.99.0 with fmt/clippy.
- System package manager writes unavailable; extracted Debian libasound2-dev
  outside repo and set task-local PKG_CONFIG_PATH to existing ALSA runtime.
- cargo fmt --all --check: PASS.
- cargo clippy --locked --workspace --all-targets -- -D warnings: PASS.
- cargo test --locked --workspace:169 PASS; native14/gameplay5 included.
- Five legacy replay/golden/save checks and P2/P3 preservation: PASS.
- Public Rust example, actual CPU before/after PNG, C header/linkage, real ctypes
  ABI consumer: PASS. Detailed commands/logs: docs/pentomino/evidence/p4.
- Auditor final narrow rerun14/14 PASS, no HIGH/MEDIUM in reviewed scope.

## Handoff
Specialist role instructions live in .codex/agents. Parent integrates sequentially.
Do not imply Android/Windows executable delivery or external certification.

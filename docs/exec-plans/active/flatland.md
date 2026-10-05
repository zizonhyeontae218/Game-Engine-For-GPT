# Basement 0.2 — FlatLand implementation plan

Status: full 0.2 implementation authorized on 2026-10-05; rc.1 user play-test passed.

## Outcome
A reproducible AI-authored top-view game with maps separated from interactive entities,
body policies/facing, combat/AI, depth rendering, conditions/quests, real sound, Lua,
animations, cutscenes and turn combat. Compact authoring and selective observation
must reduce total task context without hiding diagnostics.

## Context
Basement 0.1 uses walls as entities, one moving player, fixed interactions, flag-only
saves, static PNG sprites and silent audio events. Flutter calls authoritative Rust
through ABI v1. The user authorized implementation, a Pac-Man-style demo using open-source sprites,
manual landscape switching and separate dialogue/text popups on 2026-10-03.
`docs/FLATLAND_SPEC.md` is the proposed contract, not implemented functionality.

## Scope / non-scope
The initial playable slice implemented: maps/bodies/facing, reusable grid AI, pickups/HP,
conditions/actions and Lua, resumable saves, sprites and Flutter manual orientation/popups.
The first-slice restriction was superseded by the user on 2026-10-05. Complete remaining
gameplay with Signal Yard, reusable event/quest/combat systems and measured authoring edits. Implementation
is divided into vertical slices below. No 3D, networking or authoring GUI. Preserve the
permanent launcher philosophy. Do not rename the currently shipped version to 0.2
until the release gates pass.

## Acceptance evidence
Each slice supplies content + executable behavioral proof + narrow CLI/FFI observation.
Final evidence includes the cross-feature demo, resumed saves, headless/client equivalence,
actual audible sound, platform packaging and measured total input/output context.
Documentation validation alone does not prove engine capabilities.

## Milestones
1. Map/entity split, schema v2 with v1 compatibility, three body modes, facing, static
   spatial queries; demo pushing chains and blocked paths. Confirm format/API shape here.
2. Shared condition/action executor, typed arithmetic, prefabs, localized schema queries,
   compact authoring and delta observations. Key door + branch + exactly-once quest.
3. Lua build probe on four targets; sandbox, budgets, transaction errors, seeded RNG;
   Lua and declarative rules invoke the same authoritative commands.
4. Dynamic controllers, AI/pathfinding, HP/combat/knockback, items; saveable actor state.
   Facing attack kills enemy deterministically and records one reward.
5. Atlas animation, depth/plane rendering, follow camera, real audio adapters and HUD;
   gameplay state remains independent of presentation and audio scheduling.
6. Serializable event scenes, cutscenes/choices and reusable turn battle. Resume a saved
   choice/turn and return to exact parent world without duplicated effects.
7. Integrate resumed world saves, migration/package capability negotiation and controls;
   keep v1 tests/goldens, compare clients and package all target platforms.
8. Coverage corpus and token/time/error benchmark, publish honest 0.2 acceptance evidence.

## Decisions
- 2026-10-03: Lua 5.4 is the general scripting route; validate `mlua` vendored builds
  before dependency commitment. Ordinary content uses JSON5 prefabs and rule operations.
- 2026-10-03: No new general-purpose DSL or token-savings percentage without measurement.
- 2026-10-03: 2.5D separates Y sorting/visual height from discrete collision planes.
- 2026-10-03: Event scenes suspend serializable worlds; no Lua coroutine saves.
- 2026-10-03: Product name is Basement 0.2 — FlatLand; shipped 0.1 remains explicit.
- 2026-10-03: Pin checkout text to LF. The first Windows artifact converted authored
  game files and original SVGs to CRLF, changing content-bound resume IDs and source
  hashes despite equivalent gameplay. Preserve identical game bytes on every host.

## Progress
Completed: source inspection, requested scope, draft contract, execution order and agent
routing. Completed first alpha: schema v2 maps/prefabs, body policies/facing, grid AI, conditions/actions, Lua, HP/pickups, exact resume and selective queries.
Completed client additions: manual orientation, separate text popup, four-voice sound adapter, v2 import/embedded packaging.
2026-10-05: User completed rc.1 testing and requested full v0.2 plus a second demo proving features absent from Pac-Man. Implementing the remaining full-0.2 slices: inventory/quest controllers, richer combat/AI, elevation/bridge/camera/atlas features, serializable cutscenes/choices and turn battle, RNG, complete context/coverage benchmarks.
User confirmed existing 0.1 Flutter runners worked on PC, Linux and Android.
This is user-reported acceptance, not proof of the new 0.2 build.

## Verification log
2026-10-03: inspected current project, runtime/render architecture, AGENTS and plan policy.
2026-10-03: `python3 scripts/filetree.py update/lint` and `git diff --check` passed.
Both initial design documents have balanced code fences and the required plan sections.
2026-10-03: 32 Rust behavioral tests passed; strict Clippy/fmt passed. Both authored
replays passed, retaining the original v1 RGBA golden. Flutter 10 tests passed;
Linux release built. Real embedded Flutter/headless full snapshots and CPU frames
match at tick30 maze and tick160 room_b. New alpha device/audio checks remain pending.
2026-10-03: Source `b1dbf97` passed hosted engine acceptance run 37095556205 and all
four client build jobs in run 37095556206. Downloaded artifacts contain the actual
Rust runtime (three Android ABIs, Windows DLL exports, iOS framework), embedded v2
desktop games and portable demo. A follow-up packaging check fixes Windows CRLF
conversion and uploads the new FlatLand smoke evidence alongside v1 evidence.
2026-10-03: Final client run 37096137150 (`6fcf909`) passed all four platform builds;
engine run 37096318044 (`2996369`, hash registry synchronization) passed the complete
acceptance floor. Downloaded Windows/Linux game/control source bytes match, metadata
values match and all seven pinned SVG hashes survive packaging. Real Linux popup,
portrait and manual landscape views were inspected. Actual idle chase loses three
lives at ticks 257/437/617 without test-position mutation. Static context byte counts
are recorded in RELEASE_NOTES; tokenizer/end-to-end token measurement remains pending.

## Handoff
Read this plan and only the relevant numbered section of `docs/FLATLAND_SPEC.md`.
The first playable slice is implemented; start from `docs/FLATLAND_AUTHORING.md`
and current behavioral tests. Continue the remaining milestones without treating the
full design as already shipped. Update this plan and
`F(x).md` when cross-system identifiers actually become implemented. Generated filetree
must describe actual files only. The authoring guide defines actual alpha syntax; unimplemented full-spec APIs remain
proposed until their tests and documentation land.

2026-10-05 implementation direction: optional scene gameplay resources and serialized system state retain old v2 defaults. Rust owns inventory/quests, combat instances, RNG, sparse patches, event program counters and battle choices. Flutter presents the same authoritative choice model; no Dart gameplay. Deliver a cross-feature adventure through Demos/0.2.0, using the fixed Android certificate and a higher versionCode.

## 2026-10-05 full release implementation

Completed gameplay, inventory/equipment, quests, combat/AI, bridge planes, atlas/named
clips/camera/elevation, seeded Lua modules, serializable choices/cutscenes/turn battle,
looping device music and compact revision-checked resource edits. Signal Yard authored
journey completes at714 with13 coins and exact parent return. Systems tests resume
every waiting choice and compare authoritative state. 44 Rust/15 Flutter tests; strict
checks, original goldens and Linux Flutter/headless exact frame equivalence passed.
Actual native audio adapter produced nonzero virtual PCM; physical audibility is not
inferred. Full design's optional independent positional audio/dynamic congestion are
explicit future extensions in shipped authoring contract.

Next: inspect hosted platform artifacts, verify fixed-key Android update identity,
complete Drive0.2.0 delivery and move this plan to completed.

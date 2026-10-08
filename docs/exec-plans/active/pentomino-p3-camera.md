# Pentomino P3 Camera — replaceable Camera/View/Format completion candidate

## Outcome
Read-only presentation consumers on P2, independent multi-camera/view lifecycle,
explicit transforms/projections, four view families, deterministic composable
behaviors and default smooth2.5D transitions. Existing fifth legacy demo is the
2.5D acceptance fixture. Prepare external source-free SDK; remain EXTERNAL
VALIDATION PENDING until user's separate GPT Work evidence returns.

## Context
Start clean accepted P2 HEAD2159304, Core artifact48d7f4a alpha1. main2d1ffd0
still contains P0/P1 only; P2 PR3 OPEN, latest acceptance/Windows/Android SUCCESS.
P2 public Core23/23 external ACCEPTED, no blocking defect; external legacy remains
UNVERIFIED. User confirmed tested ZIP recompression; Core rlib hash matches.
P3 branch pentomino/p3-camera is stacked on P2; do not silently merge prior PR.
release/0.2 object293ba513 remains read-only. Rust/Cargo1.99, local ALSA SDK;
Flutter/Dart absent. Known Windows intermittent ZIP fixture cause/fix UNVERIFIED.

## Scope / non-scope
Camera/View contracts, multi-camera lifecycle, world/view/camera/screen transforms,
orthographic and perspective-compatible projections, static/follow/bounds/smooth/
dead-zone/look-ahead/zoom/shake behavior, Classic2D/Top-down/Side/Vertical views,
render-facing output, legacy2.5D fixture/render boundary, local canonical saves,
structured discovery, isolation/substitution, performance measurements and external
SDK. No gameplay/physics/Forge/generation/full renderer rewrite or native loader.
Tiny Core source/API/save behavior unchanged unless reproduced genuine blocker,
minimum proposal and independent auditor approval recorded before Core changes.

## Acceptance evidence
Scout read-only first; architect public contract; Math Reviewer and Core Boundary
Auditor review before parent freeze. Disjoint implementation/test ownership and
sequential integration. Actual automatic tests cover Camera A/B independence,
view replacement preserving exact gameplay Core bytes/IDs/RNG/unrelated plugins,
multiple views/cameras/viewport, explicit transforms/roundtrip/projection bounds,
follow/extreme values, deterministic behavior and saved transition retarget/replay,
legacyfour regression + fifth2.5D display/content/camera/save/remove fixture,
public discovery-only consumer, workspace fmt/clippy/test/dependency direction.
Performance1/many cameras/entity/selection loads recorded, no unmeasured uplift.

## Milestones
1. Actual source/P2/legacy/fifth-fixture discovery; minimum contract.
2. Math/transform and Core boundary review; freeze exact APIs/bounds/save rules.
3. Separate View/Camera and compatibility implementation; independent tests.
4. Parent source audit/regression/performance/public example and SDK integration.
5. External test package and Rundown, explicit P3 external test request; no self
   certification or premature final completion. Gameplay is a later phase.

## Decisions
- Stack on accepted unmerged P2; P3 PR initially targets P2 branch, clearly state
  dependency. main merge/release/0.2 changes are not part of this work.
- Camera consumers only receive owned public ReadFrame (or immutable host selection);
  no mutable CoreHost or gameplay callback authority.
-2.5D transitions default smooth, explicit instant; reusable tick behavior, saved
  start/target/progress and specified interruption/retarget; Classic2D may opt out.
- Fifth demo confirmed flatland_nuvema, no new demo substitution; camera-only
acceptance does not execute legacy menus that also change gameplay state.

## Progress
Scout completed read-only; existing fifth fixture identified as flatland_nuvema.
Existing1957tick baseline PASS. Architect draft corrected and frozen v1 after
independent Math/Transform and Core Boundary design reviews. Native/compat implementation and independent34tests complete; source audits
PASS after actual null-health/lens-interval/replace-endpoint fixes. Workspace
150tests plus fivelegacy golden/replay/save regression PASS. Performance measured.
Source-free SDKb43a251 produced:194declaredfiles/196ZIPentries,29,765,917bytes.
Unpacked native/fifth examples and compiler/hash selfchecks PASS internally.
Drive upload/download SHAe462a726… matches. Draft PR4 stacks on P2. New CI
shallow-history defect fixed with fetch-depth0;0c8bbdb acceptance/AndroidSUCCESS,
WindowsIN_PROGRESS atcapture. External Gate is PENDING; plan remains active until
separate GPT Work evidence, no milestone completion claimed.

## Verification log
2026-10-08 git status clean initially, HEAD2159304; ls-remote main2d1ffd0,
maintenance293ba513; PR3 head2159304 OPEN and all latest checks SUCCESS.
Rust/Cargo1.99.0; local ALSA SDK exists. P3 detailed executed checks are evidence/p3/internal-checks.json and logs;
workspace150PASS, new34PASS; all fivelegacy replay/frame/save checks PASS.

## Handoff
Parent owns manifests/locks/contracts/docs/indexes/public artifacts/integration.
Roles/files in pentomino-p3-packets.md. Keep active through external Gate.

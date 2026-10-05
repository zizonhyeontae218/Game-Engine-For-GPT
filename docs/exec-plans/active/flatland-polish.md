# FlatLand rc.2 correction and 0.2.0 polish

## Outcome
Reclassify the previous delivery as rc.2 and deliver a patched 0.2.0-rc.3 with a Nuvema
Town (Pokémon Black opening town) reconstruction/feature demo, a real battle stage,
clear 2.5D viewpoint feedback, calmer responsive movement and measured optimizations.

## Context
User reports major problems with the previous claimed final release. The original
0.2.0 Drive folder is now renamed 0.2.0-rc.2; historical artifact bytes/versionCode4
remain unchanged. New Android must keep certificate/app ID and increase code to5.
Rust owns simulation and CPU rendering; Flutter presents native state and choices.

## Scope / non-scope
Recreate the opening town arrangement with original reusable art: houses/lab, north
route, coastal edge, walking NPCs, interiors/doors and experimental stations. Dedicated
battle arena with actor sprites, HP bars, move/item menus and hit/turn feedback. Camera
projection/zoom/depth cues must make plane changes perceptible. Add authored walking
behavior instead of maze-style automatic continuation, reduce speed and control jitter.
Keep existing games and saves compatible where content is unchanged. No copied game ROM,
network gameplay or general 3D engine. Preserve fixed signing and mobile/desktop philosophy.

## Acceptance evidence
Behavioral movement/planes/battle/save tests, native/headless frame equivalence, real
Flutter battle/town capture, responsive portrait/landscape widgets, performance counters,
all platform builds and prior-certificate update verification. Deliver through Drive.

## Milestones
1. rc.2 classification, inspect reported issues/reference and define concrete fixes.
2. Engine movement and view/presentation model; dedicated battle stage.
3. Town/interior content and faithful landmark layout with original art.
4. Real input playthrough/save/replay, rendering and UI/performance checks.
5. Build/sign/package, publish GitHub, verify Drive0.2.0 delivery.

## Decisions
- Previous published source14b6942/build4 is rc.2 by user instruction; do not rewrite old
  signed binary bytes to fake a different version. New 0.2.0 uses versionCode5.
- Town/demo uses original assets and reference layout, with named feature stations.
- Battle rules remain in Rust; animation/selection UI consumes authoritative state.

## Progress
Drive archive renamed. Inspecting native tick cost, controls and town references.

## Verification log
Previous tests did not establish satisfactory game feel or battle presentation; record
new visual/input acceptance rather than treating compile success as product completeness.

## Handoff
Private key outside repo at /workspace/signing/android; certificate pin stays unchanged.

- User correction: deliver this work as v0.2 rc3 (0.2.0-rc.3), not a final release. Preserve rc2 archive.

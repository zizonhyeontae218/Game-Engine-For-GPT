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
5. Build/sign/package, publish GitHub, verify Drive0.2.0-rc.3 delivery.

## Decisions
- Previous published source14b6942/build4 is rc.2 by user instruction; do not rewrite old
  signed binary bytes to fake a different version. New rc3 uses versionCode5.
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

## Current verification and delivery progress
- Rust48 + Flutter19 pass; strict checks/headless-only checks pass.
- Input playthrough1957ticks completes battle/view/crate/combat quest; frame golden9a57d2...
- Local Flutter Xvfb actual move button changes PP25→24, enemyHP36→29; saved round verified.
- Real portrait/landscape frames captured; portrait battle eliminates unused stage gap.
- CPU pixel path optimized; local medians top0.58ms,depth2.05ms,battle0.52ms.
- Source61eca954; UTF-8 Windows packager fix53c29ada. Hosted acceptance37326997024 passes.
- Clients37326997058: Linux and iOS success; Android/Windows running.
- New Drive folder1MxzOZKg-C59aFR3gn3Lqt0IO1RgvGUqH; rc2 foldername and README/manifest corrected.
- APK signing and remaining platform deliveries pending; retain key and incrementcode5.

## Completion
All milestones complete as rc3 candidate. Hosted acceptance37326997024 and all four
client jobs37326997058 pass. Android v2/v3 signatures and code5/ID/certificate match
previous rc2. Windows UTF-8 packing fixed; unpacked game entries match Linux/mobile.
Drive rc3 folder contains platform/game/source/evidence artifacts, verified by size,
parent and download availability. rc2 archive renamed with corrected README/checksum
classification; signed build4 bytes preserved. Physical new APK install remains for
user acceptance. This work does not promote a final0.2.0 release.

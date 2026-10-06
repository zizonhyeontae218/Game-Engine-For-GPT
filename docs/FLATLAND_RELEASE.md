# Basement 0.2 — FlatLand 0.2.0-rc.4

Stabilization candidate; never final 0.2.0. Flutter 0.2.0-rc.4+6, Android versionCode6.
Supported client releases: Android and Windows only until 0.3.0 development begins.
All rc1/rc2/rc3 binaries are preserved. [Historical rc3 evidence](releases/flatland-rc3.md).

## Changes

Ground is projected during composition. Actors/props project their feet and stay upright;
semantic buildings compose an upright facade/roof/side over projected contact geometry.
No finished-framebuffer tilt. View state persists across scenes/save; cutscene camera is
scoped. View commands never run a gameplay/death/event phase or change positions/planes.

Step movement uses most-recent pressed direction with buffered legal tile-boundary turns.
Cardinal joystick hysteresis and native intent preserve intentional direction. The original
maze grid path remains separate. Harbor walks/runs at68/112px/s (235/143ms per16px cell).

Explicit gameplay.combatants preserve HP and PP. Inline legacy fighters remain ephemeral.
Six built-in cosmetic presets: strike, slash, projectile, burst, heal, guard. Native turn
results are committed once; typed FX drive lunge/effect/red flash/recoil/interpolated HP.
Unskipped feedback locks battle input. Saving/skipping never reapplies combat or rewards.

[바람항 공방 / Harbor Workshop](../examples/flatland_harbor/README.md) is independently
created content with original CC0 art/audio/map and an asset-license manifest. Four
under-authored buildings, minimal NPC/opponent and sprite-free projectile demonstrate
fallbacks. Interiors, push/fixed/pass bodies, quests, inventory, battle and save remain.
The previous village source is a historical regression fixture, not the public rc4 demo.

## Verification status

Local Rust workspace tests, strict clippy, formatting, replay/save goldens and Flutter
static analysis are checked before source publication. Windows CI additionally validates
Korean TOML/JSON/JSON5/resource names/scene IDs/storage paths, packages and imports the
content, launches the real Flutter Windows client and captures portrait/landscape UI,
top/depth state equality and battle phases. Android CI builds three native ABIs.
Hosted results and signed delivery are pending until recorded below.
Physical Android user acceptance is separate and has not been claimed for rc4.

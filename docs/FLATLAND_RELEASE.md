# Basement 0.2 — FlatLand v0.2 rc3

Candidate `0.2.0-rc.3`, Android build5. Previous delivery is archived as
[rc2](releases/flatland-rc2.md); there is no final 0.2.0 release in this work.

## Playable demo

[마름꽃마을 / Nuvema Field Study](../examples/flatland_nuvema/README.md) rebuilds
Pokémon Black's opening town layout with original CC0 graphics: northwest lab, central
player house, two southern friend houses, northern path, railed coastal overlook.
Buildings have playable interiors and correct named exits. The village has a walking NPC.
The lab supplies a starter, pushable crate, fixed combat dummy and pass-through marker.
Battle, camera, crate and combat stations complete a persisted quest and award a badge.

## Changes

- Opt-in `step_walk` completes the current cell on release, then stops. Defaults retain
  Pac-Man's maze steering. Town walk/run speeds are 48/72px/s (previous Yard was120).
- Game-local JSON presets use four cardinal directions, a22% dead zone and12% axis
  hysteresis; Z/Space interacts, X chooses viewpoint and C toggles pace.
- Turn battle has its own responsive screen: canonical front/back creature sprites,
  HP/maxHP, targeted moves/PP, guard, bag, save and visible victory/defeat before return.
  Fixed-tick hit flash/shake continues while the parent world remains frozen.
- Camera presets actually zoom/tilt/shear the padded CPU world frame. Upper mode
  changes collision plane and actor elevation; ground-only fence visibly demonstrates it.
  This is a2D affine projection, not a3D renderer. Failed occupied plane changes roll back.
- Rendering clips tile iteration, fast-paths fully opaque/transparent pixels, and avoids
  serializing logs/event-parent save copies for every native frame. Music synchronization
  deduplicates unchanged desired states; authored Lua can filter irrelevant event hooks.

## Evidence

1957-tick authored input/choice replay completes all four quest objectives, starter=ember,
battle win and badge reward. One-tick input release completes exactly one16px cell and
remains still. Mid-turn HP/PP and completed battle resume from disk. Full observations and
minimal rendering snapshots produce identical frames for normal/depth/battle scenes.
Old Basement, Maze Chase and Signal Yard regressions remain required.

Build, UI and delivery evidence will be recorded after final verification. Hardware
Android/Windows/Arch playtesting is not inferred from CI builds or Linux Xvfb execution.

# FlatLand playable candidate improvements

## Outcome
Deliver the requested game-specific mobile presets, maximum usable game viewport and
visible sprite animation as 0.2.0-rc.1 through the connected Drive Demos folder.

## Context
The user's landscape screenshot keeps the manually selected portrait layout and
reserves half the remaining height for generic controls. A second toolbar, debug HUD
and action text further reduce the maze. Demo normal clips contain only one frame.
The user additionally requires stable Android certificate/package identity and updates
without uninstalling. Previous CI builds used runner-generated debug signing keys.

## Scope / non-scope
Fix these three playable-candidate issues, retain manual rotation and JSON editing,
preserve v1 games, verify actual layout sizes/animation/frame equivalence and builds.
The broader unimplemented full-0.2 design remains tracked by flatland.md.

## Acceptance evidence
Game-authored package presets and joystick-only profiles; update/reset behavior;
small portrait/landscape/high-text-scale viewports without overflow; exact largest
fitted game area independent of touch controls; deterministic multi-frame rendering
and resume; hosted platform builds and verified Drive files. An installable Android
update requires the original private key; never silently replace it with a new key.

## Milestones
1. Versioned joystick-only layouts, automatic authored preset packaging and reset.
2. Full game canvas with compact overlay HUD and a separate settings sheet.
3. Open-source multi-frame sprites, deterministic frame/resume assertions.
4. Android stable signing configuration, version bump, build and artifact delivery.

## Decisions
- 2026-10-05: Geometry fits the game to all available space, independently of the
  manually selected orientation; controls overlay and never reserve game space.
- 2026-10-05: Add layout schema 2 for zero-button joystick profiles; bindings remain
  schema 1. A game's controls/layouts.json and bindings.json override generic defaults.
- 2026-10-05: Keep Android package ID; remove ephemeral release debug signing. Search
  for the original key before claiming same-certificate in-place updates.

## Progress
Game-specific schema-2 joystick-only presets, untouched-default migration/explicit reset, full fitted viewport/compact HUD/settings sheet, MIT multi-frame directional sprites and fixed signing configuration implemented. Private key backup is owner-only on Drive. Hosted builds and final delivery pending.

## Verification log
2026-10-05: 33 Rust tests and strict Clippy/fmt; 13 Flutter tests and analyze passed. Tests cover full fitted frame size before manual rotation, twofold text scaling, zero-button preset parsing, old-default migration/custom override preservation/reset, distinct mouth pixels, direction/timer clips and exact animation-phase resume. Local Linux release built. Animated tick30 RGBA: a0be5c7fc85ca45c9dd8a54e0fa1bfc6c355a1e7e1d78a7ce5598b447fa43546. Original v1 golden unchanged. User authorized a final reinstall for rc.1 and stable signatures thereafter.

## Handoff
Keep this plan and flatland.md honest. Do not describe the entire full-0.2 proposal as
implemented, or an APK with a different/new certificate as an update for existing installs.

Actual packaged Linux replay completed; signing backup is verified owner-only, 5,145 bytes, and preserved outside Git. Real 1280x527 capture confirmed a full 437x483 game and joystick-only controls. Complete native/headless snapshots and actual RGBA match; hosted platform builds remain next.

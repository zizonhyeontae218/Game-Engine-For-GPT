> 역사 기록: 이 문서의 버전·배포 링크·검증 상태는 당시 기준이며 현재 안내는 [릴리즈 목차](../README.md)를 따른다.

# Basement 0.2 — FlatLand v0.2 rc3

Candidate `0.2.0-rc.3`, Android build5. Previous delivery is archived as
[rc2](flatland-rc2.md); there is no final 0.2.0 release in this work.

## Playable demo

[마름꽃마을 / Nuvema Field Study](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/tree/ed3301c7af12ac36dd0b5074d79693d203aeeb28) rebuilds
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

Local strict Rust checks and48 behavioral tests pass; Flutter analysis and19 tests pass,
including dedicated battle controls at390×844,800×360 and640×320 with2× text.
A real Xvfb Flutter button executes a native round (hero PP25→24, enemy HP36→29,
hero HP32→26), then saves the turn. Scene/frame equality at tick1957 is checked by
both the local embedded release and hosted Linux smoke. Final RGBA:
`9a57d2fad6c3a96ecf1e4f1378b751729325b1907f5c89319ada082f4df361c3`.

The local release CPU benchmark runs1000 renders per mode/path. Median snapshot+render
fell from1.73 to0.58ms in top view and6.40 to2.05ms in depth after moving crop/rotation
parsing out of each pixel and fast-pathing opaque untinted pixels. Battle input serialization
shrinks from97065 to33929bytes; minimal battle snapshot+render median0.52ms.
These are local CPU measurements, not mobile FPS or whole-game frame budgets.
Reproduce with `cargo run --release -p ge4g-runtime --example presentation_bench`.

The Windows packager previously used the system encoding for Korean TOML names;
manifest/bindings and validator output now explicitly useUTF-8. ASCII-locale packing
produces the identical `.ge4g` bytes as UTF-8 Linux packing. Hardware
Android/Windows/Arch playtesting is not inferred from CI builds or Linux Xvfb execution.

## Verified build and delivery

Build source `53c29ada60e41443bb271fbcbcd55e9ce2c6315f` (gameplay commit61eca9548a).
[Acceptance37326997024](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37326997024)
and [clients37326997058](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37326997058)
completed successfully. Client jobs cover Linux/Windows embedded games, Android3ABIs,
and unsigned iOS. Windows ZIP includes the native DLL/client/game; its unpacked game
entries and manifest match the portable package (container compression bytes may differ).

Android ID `dev.ge4g.ge4g_client`, versionName `0.2.0-rc.3`, versionCode5.
APK v2/v3 verification and prior rc2 certificate comparison pass; pinned SHA256 remains
`d6d5ca948e5c1ed644478d7c4c3243efcfcbc30a196f03c39ef7af7118fc00d4`.
The release supports updating the installed rc1/rc2 app in place. No signing key was rotated.
Actual device install/physical audio and Windows/Arch user gameplay are not claimed.

[rc3 Drive delivery](https://drive.google.com/drive/folders/1MxzOZKg-C59aFR3gn3Lqt0IO1RgvGUqH)
contains APK, portable game, embedded desktop builds, source, real UI captures, CPU
benchmarks and verification/checksum instructions. [rc2 archive](https://drive.google.com/drive/folders/1YeIJO1SoQ-ncx-mlU6jxm1QLM4vI5XKX)
is renamed and its README/manifest identify it as rc2; original signed build4 bytes remain intact.

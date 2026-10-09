> 역사 기록: 이 문서의 버전·배포 링크·검증 상태는 당시 기준이며 현재 안내는 [릴리즈 목차](../README.md)를 따른다.

# Basement 0.2 — FlatLand 0.2.0-rc.4

Stabilization candidate; never final 0.2.0. Flutter 0.2.0-rc.4+6, Android versionCode6.
Supported client releases: Android and Windows only until 0.3.0 development begins.
All rc1/rc2/rc3 binaries are preserved. [Historical rc3 evidence](flatland-rc3.md).

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

[바람항 공방 / Harbor Workshop](../../../examples/flatland_harbor/README.md) is independently
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
Engine CI passed (58 Rust tests, strict clippy, schema1/Pac-Man/Signal Yard regressions,
Harbor deterministic replay/save goldens). Windows Flutter tests (23), release build and rendered portrait/landscape UI
acceptance passed, including the full Korean UTF-8 pipeline. Android three-ABI build passed;
code6 APK is signed with the unchanged certificate.
Physical Android user acceptance is separate and has not been claimed for rc4.


## Delivery / reproducible evidence

- [rc4 Drive files](https://drive.google.com/drive/folders/14FhxMy6tjUJr1vlbwFasHb2S3DIclMe_)
- [Engine acceptance](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37401561428)
- [Windows/Android clients and rendered evidence](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37401395040)
- Android application ID: `dev.ge4g.ge4g_client`, versionCode `6`.
- Signing certificate SHA-256: `d6d5ca948e5c1ed644478d7c4c3243efcfcbc30a196f03c39ef7af7118fc00d4` (matches rc3).
- APK SHA-256: `d0fa15f7b575c3e48d3ac43c0acaae35ee8d5453dfc1bf1784f4529f12adfc69`.
- Portable game SHA-256: `4d223221aa57faf0a4cb99ab2e744d5316af0f7db340cee205b5929d2981795f`.

Source for the APK is `1f75e300be96313a8b5c5fff6d01d474b5e532a0`.
Windows source is `2554fdf6b2444a50bad28a482272d7b668e07458`;
that follow-up changes only the screenshot assertion and failure-artifact upload policy.
The integrity registry follow-up does not change runtime binaries.

The rendered evidence archive contains actual Windows Flutter portrait/landscape UI,
canonical frame PNGs, exact world snapshots, and a battle save. Top/depth snapshots
have identical entities. Native actor pixels may return to their pre-attack positions
at recovery; HP bars/menu live in Flutter and are verified through UI captures/tests.
These captures are Windows evidence, not physical Android acceptance.


Rendered review: upright player and facade/roof in depth view, visibly projected ground,
identical player `[256,176]` pixels / plane0 before and after view switch.
Battle frames at ticks0/12/24/36/108 show slash/lunge, red impact, interpolated
opponent HP34→28→22 and restored menu. Actual HP commits to22 once at resolution;
cosmetic ticks do not apply damage again. Windows and mobile game package entries
are byte-identical. Capture UI is paused for deterministic inspection; `*-frame.png`
shows the canonical actor unobscured by the pause badge.

Drive delivery readback verified file names, sizes and rc4 parents: signed APK,
embedded Windows ZIP, portable game, original sample source, rendered evidence,
world-state proof, verification metadata, Korean instructions and SHA256SUMS.

# P3 Camera/View handoff — accepted

P2/P3 tests are PASS by user report on 2026-10-09. P3 internal CI was already
successful; external acceptance is user-reported, not performed by repository workers.
Read P3_RUNDOWN.md, P3_CONTRACT.md and NEXT.md. The next slice is a removable
turn-combat Gameplay plugin, preserving Core/View boundaries.

## Keep these boundaries

Native ge4g-pentomino-view only reads selected P2 ReadFrame. P1/root save1 and
P2/save2 source/contracts/tests, existing schema/save/ABI1 and release/0.2 remain
unchanged. Presentation save1 is separate. Camera-local targets/transition state,
follow/shake/active-camera settings never enter Tiny Core.

ge4g-pentomino-view-legacy imports current legacy World as a bounded immutable
ordinary source plugin, with real Core Scene/Entity allocations and source data.
It does not continuously delegate legacy gameplay. Real existing fifth Nuvema
assets render from native CameraFrame via the bounded orthographic subset,
including intermediate Blended weight0. General perspective/rotation projection
works natively but the unchanged legacy raster adapter explicitly rejects it.
Never execute legacy view menus to prove camera invariance: they also alter
plane/elevation/state. Project ground/building geometry and upright feet before
composition; never warp a completed framebuffer.

## Precision and validation lessons

Near/far independent interpolation failed at valid minimum-width large-near
endpoints; interpolate positive width, outward-round inward samples, clamp convex
roundoff and retain exact endpoints. View replacement validates saved transition
endpoints under the new policy. Float-roundtrip restore validates quaternion bits
without renormalization. Follow tracks all World axes, masks only final View axes.
Null legacy health fields are valid. All unsafe source/renderer paths reject before
cloning or entering the old renderer. Trusted linked callbacks are not sandboxed.

## Next

P2 and P3 are accepted for main integration. Proceed to NEXT.md after confirming
merged HEAD/CI. Do not repeat their implementation or external acceptance. No final
0.3 release, platform expansion, full3D renderer or Forge implementation is implied.

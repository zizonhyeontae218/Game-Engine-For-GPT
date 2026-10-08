# P3 Camera/View handoff — external validation pending

P2 public Core GPT Work23/23 is accepted. P3 internal implementation and
source-independent tests are complete as a candidate; **EXTERNAL VALIDATION
PENDING**. Do not declare external validation performed by repository workers.
See P3_RUNDOWN.md, P3_CONTRACT.md and ../public/pentomino-p3/.

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

Finish separate GPT Work public-consumer validation with SDK hashes/raw test
sources/logs and fix any blocking reproducers before P3 acceptance. Main remains
P0/P1; accepted P2 PR3 is open, P3 PR stacks on P2 until integration. No automatic
main merge, final release, client deployment/signing or platform expansion occurs.
After acceptance, Gameplay plugins can begin as separately designed consumers;
combat/world/interaction/platformer/Forge/generation are unimplemented. Windows
historical ZIP timestamp/digest intermittent cause/fix remains UNVERIFIED.

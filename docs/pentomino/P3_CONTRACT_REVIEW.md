# P3 contract review / frozen v1

Baseline: accepted P2 2159304; production unchanged. Camera Architect drafted
P3_CONTRACT; resumed parent integrated independent reviewers before functionality.

**Math/Transform: PASS (design review only).** p3_math_review accepted bounded
negative-Z unified lens, factorized TRS+XY shear, shortest NLERP, smoothstep
fixed-tick transitions and current-sampled-pose C0 retarget. Corrected inconsistent
follow double masking: full World tracking, only final View-space displacement
mask. Required float_roundtrip/no restore renormalization and coordinate-dependent
legacy rounding tolerances. Each legacy i64 intermediate must be checked.

**Core Boundary: PASS (design review only).** p3_boundary_audit confirmed
readonly ReadFrame consumption and ordinary owned source initialization. Corrected
import record count (2*N+state+2<=256), schema-ID bounds, legacy action collisions,
valid other-Scene filtering, pre-clone bounded source extraction, and native-vs-
legacy arithmetic restrictions. No Tiny Core API/source changes justified.

Implementation/source audit and all new behavior tests are **UNVERIFIED** at
freeze. Final audit must verify actual bounded canonical saves, whole-host
rollback, callback limitations, renderer arithmetic/source geometry preflight,
all four policies and existing fifth-demo real output. Review statements do not
certify external consumer validation.

## Final source audits / candidate

p3_math_source_audit: PASS after positive-depth interval and replacement endpoint
fixes. Reviewed actual inverse/projection/NLERP/follow/shake/tick/save code; no
remaining blocking math defect. World-only stored bounds clarified.

p3_boundary_source_audit: PASS. Scanner found31 baseline files unchanged and
zero Core changes; native engine dependency is Core only. Actual independent
cargo test for both new crates:27native+7compat PASS. Compatibility validates
bounded source before cloning and derives renderer parameters from CameraFrame.
The old unsafe renderer is never a fallback for unsupported native transforms.

Discovery and independent behavior results are internal evidence; public GPT Work
consumer gate is now PASS by user report on2026-10-09. Larger custom selection lookup complexity and
cross-platform float identity are limitations, not verified optimizations.

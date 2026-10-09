# Failure notes

Record only non-obvious failures that are costly to rediscover. Do not log transient compiler errors.

## Feature builds share an executable path

**Symptom:** Window smoke fails with “interactive adapter was excluded” after a previous successful window run.

**Root cause:** Default and `--no-default-features` builds both write `target/debug/ge4g`; the latest headless-only CLI test replaces the default binary.

**Reliable detection:** The structured runtime error identifies the excluded adapter.

**Fix:** Run `cargo build --locked -p ge4g-cli` before window verification, or use separate `CARGO_TARGET_DIR` locations for each feature configuration. CI verifies the window before switching to the headless-only build.

**Applies to:** GE4G 0.1 feature-combination tests. No gameplay divergence is involved.

## Managed Git transport authentication

**Symptom:** `git push` returns HTTP 401 even while connected GitHub API reads/writes succeed.

**Fix used in this session:** Publish through the authorized GitHub connector. Initialize the empty repository using the contents API, create the binary blob and complete Git tree, compare its SHA with the local verified tree, create a commit and fast-forward main. Fetch and align the local checkout with the published commit. Do not inspect or replace injected credentials.

**Applies to:** This managed executor transport on 2026-10-02; an ordinary developer checkout may support normal Git push.

## Flutter loading clock

A real hosted packaged-client smoke exposed tick161 after a nominal 160-step replay when the display was slower than local tests. Session open reset pause before awaiting an in-flight image codec; the host ticker could advance a tick while the loading spinner was visible. `Player.tick` now treats loading as suspended, tracks the elapsed baseline without stepping and does not accumulate hidden time. A native-player regression attempts ticks from the loading frame callback. The embedded smoke must continue asserting exactly tick160 and complete snapshot equality; do not weaken it or subtract a tick.

## Windows Korean project names (rc3)

Python locale-default read_text/subprocess text decoding rejected the Nuvema TOML name
under Windows cp1252. The packager must decode authored UTF-8 and Rust JSON explicitly
as UTF-8; never rely on the machine locale. An ASCII-locale package is byte-identical.

## P3 minimum-width projection transition precision

Valid near/far endpoint intervals (.001 minimum, near≈26–59million) failed during
smooth transition when near/far were interpolated independently: subtraction
rounded the interval inward. Public probe observed tick8 InvalidProjection with
exact rollback, independent4096tick test first failed tick2. P3 interpolation now
lerps positive depth width, clamps convex scalar roundoff and outward-rounds a
minimum interval with next_up; endpoints retain exact requested variants/values.
Regression: ge4g-pentomino-view/tests/independent_contract.rs. Do not weaken .001
validation or swallow a failed camera tick. P3_RUNDOWN.md contains evidence.

Legacy valid noncombat actors serialize hp/hp_max:null; P3 preflight accepts null
while bounding numeric health values. Smooth orthographic transitions emit Blended
weight0 intermediate lens and require equivalent orthographic compatibility handling.

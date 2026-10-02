# Failure notes

Record only non-obvious failures that are costly to rediscover. Do not log transient compiler errors.

## Feature builds share an executable path

**Symptom:** Window smoke fails with “interactive adapter was excluded” after a previous successful window run.

**Root cause:** Default and `--no-default-features` builds both write `target/debug/ge4g`; the latest headless-only CLI test replaces the default binary.

**Reliable detection:** The structured runtime error identifies the excluded adapter.

**Fix:** Run `cargo build --locked -p ge4g-cli` before window verification, or use separate `CARGO_TARGET_DIR` locations for each feature configuration. CI verifies the window before switching to the headless-only build.

**Applies to:** GE4G 0.1 feature-combination tests. No gameplay divergence is involved.

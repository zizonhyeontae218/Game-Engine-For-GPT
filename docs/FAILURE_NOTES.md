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

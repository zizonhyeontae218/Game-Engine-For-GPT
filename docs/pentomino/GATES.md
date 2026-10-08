# Pentomino contract gates

These are acceptance requirements. **They are not claims that the current engine satisfies them.** Evidence must name executable commands and exact commits/artifacts, with failures retained.

| Gate | Requirement | Minimum honest evidence |
|---|---|---|
| G0 inventory | Known repo language/tooling/module boundaries | file paths + observed commands; unavailable = UNVERIFIED |
| G1 tiny-core | Core does not import genre or view implementations | dependency direction scanner or code review on actual source |
| G2 plugin lifecycle | load, fail cleanly, unload one plugin, keep unrelated plugins alive | tests for unload/isolation and declared deps |
| G3 view substitution | side/vertical/top-down/classic 2D independently install/replace | tests and package metadata for each available view |
| G4 composition | top-down + realtime combat + open world combination works without Core genre fork | reproducible integration fixture; missing plugins = BLOCKED |
| G5 public agent discovery | public API + skills/docs enable capability selection without internals | source-free consumer report and versioned alpha |
| G6 state/resource stability | serialization and cleanup do not leak or corrupt between modules | targeted tests; safety checks if applicable |
| G7 regression/perf | release includes small measured Core/main-plugin improvement or justified no-change | command, baseline, after, platform and noise notes |
| G8 scope | Forge/story/asset generation remains v0.4-only | source/diff review and release notes |

## PASS rules

Use statuses `PASS`, `FAIL`, `BLOCKED`, `UNVERIFIED`, `NOT_APPLICABLE`. PASS requires a reference to **real executed** evidence with artifact/commit ID, command, observed result and reproducibility note. A test with missing baseline can confirm function but not performance uplift. If no engine checkout exists, all runtime gates are UNVERIFIED/BLOCKED. No placeholder evidence is acceptable.

A breaking contract change requires version bump or explicit migration, release note, affected consumers inventory and rollback plan. Parent integrates only after auditor and test evidence.

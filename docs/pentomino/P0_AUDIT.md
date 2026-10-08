# P0-R independent contract review

Baseline: main `293ba513f5727d4a7a0a59476c9a98eb193aaa25`. Reviewer:
pentomino_auditor, read-only, no edits/installations/tests. Initial 254-line
P0_API_PROPOSAL.md draft received **CHANGES_REQUIRED**. All runtime conclusions
and proposed acceptance tests remain **UNVERIFIED / NOT RUN**.

## Initial findings and parent integration

| Severity | Confirmed contract gap (original draft lines) | Parent correction |
|---|---|---|
| MEDIUM | Session handles in keys/events (:55,69,71) versus unspecified stable save IDs/remap (:211–217) and equal restored hash (:248) | Stable wire owner=PluginId, scene/entity local key+incarnation, saved allocation counters, typed validated references, fresh-handle remap; generations excluded from canonical hash; B compared using its equal-tick canonical projection |
| MEDIUM | Generic view/save/hash invariance (:197–198,246) could imply changing legacy behavior | Restrict invariance to new Tiny authoritative envelope/hash, preserve legacy snapshot_hash and schema2 view-persisted saves and regression tests |
| MEDIUM | Target tick (:70) and retry (:150–154) lacked committed/past/future/duplicate input rules | Accept committed+1 only, typed no-mutation rejection, unique ActionIds sorted canonically, failed ticks retryable; plugin exact-once markers saved separately |
| LOW | Retained history restore versus queue/cursor (:215) unspecified | Save/restore bounded retained history, pending queue, sequence/gap/drop accounting; purge ownership explicitly |

These are proven gaps in a proposed contract, not executed implementation defects.
Hash mismatch, stale reference resurrection and double application remain
UNVERIFIED hypotheses until their specified tests run.

Source evidence for the legacy finding: runtime `src/lib.rs:686` snapshot_hash
serializes the whole Snapshot, core `src/lib.rs:236` includes camera; runtime
`src/flatland.rs:1054` serializes FlatLand state; runtime `tests/rc4.rs:98` expects
saved gameplay-view restoration. `rc4.rs:21,25` remove camera/view fields in a
simulation-only test projection. New Tiny invariance cannot redefine these APIs.

Auditor accepted proposed direction and negative boundaries: no Core imports of
View/Gameplay or mandatory legacy types; host-owned typed state and transaction
RNG; declared dependencies and non-cascading unload; data-only imported games;
linked Rust first slice; existing schema/save/ABI1 preserved. Actual enforcement
and rollback remain UNVERIFIED.

Parent integrated corrections only after architect released ownership and initial
auditor review completed. Corrections are in the proposal's parent-integrated
P0-R section. Auditor read-only re-review approved the P0 proposal technically: all three MEDIUM gaps and the LOW history gap are resolved. Its final LOW naming correction (`World::snapshot_hash` → `ge4g_runtime::snapshot_hash`, a free function) was applied by the parent. Parent accepts the reviewed proposal for P0 only; P1 encoding/limits freeze and runtime evidence remain prerequisites.

## P1 prerequisites

Freeze numeric host limits, canonical encoding/version, identity remapping,
input acceptance and bounded history accounting before functionality. First slice
is host plus two independent dummy plugins, not extraction of every game system.
Required future tests: restore across generations/stale handles/recreated identity;
new-view invariant hashes plus legacy view resume; duplicate/future/bad input and
failed-tick retry; save/history/purge accounting; initialization rollback and
independent-plugin unload. G1/G2/G6 require executed evidence; no P0 runtime PASS.

External consumer/source isolation, named views, gameplay composition, Windows
ZIP timestamp root cause and platform regression remain UNVERIFIED/BLOCKED.

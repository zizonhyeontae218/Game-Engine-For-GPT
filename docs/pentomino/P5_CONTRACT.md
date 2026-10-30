# P5 Gameplay v2 and native v1 contract

Additive GE4G 0.3.0-alpha.1 contract, implemented in
`ge4g_pentomino_gameplay::v2` and `ge4g_pentomino_native::p5`.
Actual acceptance evidence lives in P5_RUNDOWN.md. P4 v1, FlatLand ABI1 and all
previous save contracts remain unchanged. API rationale: P5_PROPOSAL.md.

## Authority and rules

StateProvider (`p5.state`) owns Scene/Entities, immutable positions and a status
Record. Removable TurnCombat (`p5.turn_combat`) reads its capability and emits a
decision from four scalar command Actions. The bridge verifies the real decision,
then maps its complete scalar batch to StateProvider Actions on a second Core
tick. Only the provider writes authority. Each accepted command advances logical
step once/Core twice; rejected input changes no clocks, state, pixels or journal.
No FlatLand World or UI-side battle simulation is used.

JSON config version2: 2..4 fighters, teams 0/1 both present, 1..16 named actions,
viewport32..512, tick_hz1..240. Fighters define HP1..10000, max/initial resource,
attack/defense/speed0..10000, pixel feet and RGBA. Unknown fields reject.
Actions define id/kind/target/cost/power/effect/duration. Kinds:damage,heal,
resource,effect; targets:self_only,ally,enemy (ally allows self).
Poison/regen require power1..10000/duration1..100. Other kinds require null effect
and duration0. No resurrection, automatic opponent or wall-clock steps.

Turn order:descending speed, roster index tie-break. Living actor must own turn;
validate living target/relation/resource first. Spend cost, then act.
Damage=max(1,attack+power-defense); healing/resource cap at configured maxima.
Effects replace the same effect, never stack. After action, acting fighter takes
poison then, if alive, regen; decrement durations and clear absent effect powers.
Dead fighters lose effects. Resolve victory/draw after effects, then skip dead
fighters in fixed order. winner=-1 running,-2 draw,otherwise winning team.
Terminal commands reject.

## Identity, lifecycle and atomicity

Command:`{id:i64,actor:usize,action:usize,target:usize}`, positive strictly
increasing ID, gaps allowed. Duplicate/stale IDs reject, including identical
retries. ID/count persist in provider through actual CoreHost rule remove/install.
Exactly-once means duplicate application prevention; retries are not cached success.
Restoring an older save intentionally restores an older timeline.
Partial command/provider batches and no-op lifecycle calls reject.
Staged mutations retain live Core/View/frame/clocks/journal on failure. Trusted
plugins are not a malicious-code sandbox; callback side effects are not rollbackable.

## Native and saves

Header:`include/ge4g_p5_v1.h`. Additive C symbols:
`ge4g_p5_v1_abi_version`(1),`ge4g_p5_v1_request(data,len)`,
`ge4g_p5_v1_string_free`. Request borrows valid readable bytes; response is owned
UTF-8 JSON, freed once. Sessions are creating-thread-affine, handles globally
unique and disjoint from P4. No unsafe Send.

Envelope:`{version:1,command:{op:..., ...}}`; response:
`{version:1,ok:bool,result:...,error:...}`.

| op | fields/result |
|---|---|
| create | config,seed:u64,mode:headless/linked -> {session} |
| describe/observe | session -> discovery/ordered P2/P3 observation |
| act | session,command:{id,actor,action,target} -> observation |
| frame | session -> width,height,RGBA byte array,SHA256 (linked only) |
| save | session -> version2 SaveBundle |
| restore | session,save -> null |
| gameplay | session,enabled -> null |
| camera | session,request -> null (P4 supported camera subset) |
| view | session,family -> null (existing P3 ViewFamily) |
| close | session -> null; subsequent handle use rejects |

Save version2 binds validated canonical config (`p5:`+SHA256),seed,clocks,
attachment, exact Core save2, independent presentation save1 and accepted
command/lifecycle/camera/view trace. Restore stages fresh hosts, replays actual
plugins and compares exact canonical Core/P3 saves and clocks before replacement.
Limits:4096 trace operations,16 sessions/bridge,24MiB request/save envelope,
config64KiB,Core8MiB,presentation1MiB. No implicit P4/FlatLand migration.
P3 camera/view and existing canonical CPU compositor provide actual pixels;
JSON RGBA remains the transport. No general 3D support is added.

Public consumers:samples/pentomino_p5/consumer.py and smoke.c; Flutter pre1P6.
Out-of-tree execution proves internal reproducibility only; isolated Blind Consumer
certification remains UNVERIFIED. Native verification, platform builds, signing,
release and physical user acceptance are separate.

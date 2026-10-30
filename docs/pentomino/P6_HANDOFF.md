# P5 -> P6 acceptance handoff

P5 API: P5_CONTRACT.md. Evidence/status: P5_RUNDOWN.md.
P4 #8 -> #7 -> #5 is unmerged; P5/pre1P6 stacks on #8 in PR #9.
No PR merge, signing, release or deployment has been performed or authorized.

## Sequential user gates

| Stage | Added executable scope | Acceptance focus |
|---|---|---|
| pre1P6 | Existing Flutter client + linked P5 native battle, choose action/target, play to outcome | Actual launch, controls and visible results |
| pre2P6 | Android/Windows platform parity and native input/output | Separate platform failures and same sample state |
| pre3P6 | Config loading, game/plugin management, explicit errors | Data compatibility and module switching |
| pre4P6 | Input/camera/2.5D/state/result presentation | Usable play flow and readable results |
| pre5P6 | Durable Save/Load, process restart, recovery/replay | Continue after closing, consistent restoration |
| pre6P6 | Combined platform/package/performance/resource regression | Candidate quality and outstanding defects |
| pre7P6 (optional) | Fix remaining severe defects and SDK/docs | Final user acceptance candidate |

Combine stages only when their evidence exists. Seven stages is a target, not a
reason to hide defects. Each delivery stops for the user's PASS/FIX/CHANGE.
No pre2P6 work before pre1P6 PASS. Automated facts must not be reassigned to user
manual checks. Signed Android delivery requires explicit signing authorization and
the existing pinned certificate; never create a replacement key. Because signing
is not authorized now, pre1P6 first targets an executable Windows bundle. An
unsigned Android CI artifact cannot be labeled installable/user-accepted.

## pre1P6 implementation

`--pentomino` opens one bundled version2 JSON sample in the existing client.
The library also has an entry. P5 is a separate native code asset; old native ABI
stays bundled. All commands execute on Rust P2; Dart only displays actual native
observations and canonical pixels. Both teams are manually controlled.
The screen provides action/target/execute, turn/HP/resource/effects/outcome,
restart, and temporary in-memory save/restore. The latter intentionally does not
promise persistence across process exit; durable saves belong to pre5P6.
Windows bundle includes PLAY-PENTOMINO.cmd and the entire Flutter runtime.
Do not distribute only the EXE. `--pentomino --pentomino-capture` writes actual
startup UI/state under GE4G_SMOKE_OUTPUT; this automation is not physical acceptance.

## Known boundaries

2..4 fighters/two teams, <=16 static JSON actions, poison+regen, <=4096 accepted
journal operations/session. Plugin registration/removal uses trusted compiled
Core plugin instances, not executable downloads or dlclose. Dynamic content
mutation requires a new validated config/session; old save binding cannot silently
change. Renderer keeps the existing orthographic CPU subset and JSON RGBA transport.
Isolated Blind Consumer certification, physical ARM/device execution and signing
remain separate unmet gates. Debug trace replay cost is measured in P5_RUNDOWN;
no performance improvement or nH+ certification is asserted from these checks.

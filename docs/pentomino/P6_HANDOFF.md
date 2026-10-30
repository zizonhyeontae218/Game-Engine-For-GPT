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

## Submitted pre1P6 checkpoint — 2026-10-30

**P5 technical implementation complete; pre1P6 executable submitted, user acceptance PENDING.**
Build source: c18adf76c336ef5a4718283bbc747268724883da.
Engine run41068956609 and client run41068956566 completed PASS.
Windows job124108443877 and Android job124108443316 both PASS.

1. Implemented: existing Flutter client + actual linked P5 CPU pixels/state,
   action/target selection, turns/resources/poison/regen/outcome, restart and
   temporary in-memory save/restore. Both sample teams are manually controlled.
2. Actual Windows artifact12984837494, ge4g-pre1p6-windows. Inner ZIP15624093bytes,
   SHA256 `1411beaeff074e809f39b6317c06d47c0f3c3bc4845dc7fd49bcebead9bd03bd`.
   Downloaded wrapper and all23 payload hashes verified. Delivered same ZIP as
   GE4G-pre1P6-windows.zip. Full Flutter runtime/native DLLs included.
3. Extract everything, run PLAY-PENTOMINO.cmd. No separate engine/CLI install.
4. Rust181/181, five legacy golden/replay/save checks, public P4/P5 C consumers
   and independent audit PASS. Flutter analyze and38/38 tests PASS. Actual Windows
   EXE/native create/frame/startup capture PASS; legacy embedded/UTF-8 job PASS.
   Native startup PNG+JSON: artifact12984462867, same client run.
5. User check: launch, select actions/targets, finish one battle; report any
   screen/input/play issue. Do not repeat automated tamper/duplicate/replay tests.
6. Simple rectangle/HP-bar sample, no automatic opponent/editor/Full3D.
   Temporary saves do not persist after process exit. Android APK artifact12984562755
   built successfully but is unsigned; no install/physical/ARM execution claim.
   P4 isolated Blind Consumer remains UNVERIFIED. No signing/release/deploy/merge.
7. Stop for PASS/FIX/CHANGE. No pre2 implementation before user decision.
   Roadmap remains pre2 platforms,pre3 management,pre4 input/camera/presentation,
   pre5 durable recovery,pre6 integration,optional pre7 stabilization.

Initial Windows analysis import error and new UI decode/library-position defects
were corrected. Next run37/38 passed; an old ZIP duplicate fixture encoded fresh
archives while expecting equal full-byte digests. Reused the identical byte array
for duplicate/reinstall tests, keeping production hashing and all assertions.
Final38/38 passed. The auditor approved the eighth exact client fixture exception;
all other client/P0-P3 sources/tests remain preserved. Android initially hit a
corrupt NDK download before compilation; final fresh build succeeded.

Actual startup screenshot was visually inspected after narrow/wide layout fixture
captures; no severe overlap/clipping in reviewed sample. CI graphics and user
physical acceptance remain separate. Private file-delivery identifiers are omitted.

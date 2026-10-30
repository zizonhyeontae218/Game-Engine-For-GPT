# P5 implementation and verification rundown

2026-10-30. PR #9 stacks on #8 -> #7 -> #5; nothing merged.
P5 engine scope is technically accepted after implementation, execution and an
independent source-aware audit. pre1P6 client delivery is still pending actual
platform build evidence; no user/physical acceptance or signed release is claimed.

## Implemented

- Config v2:2..4 fighters/two teams, HP/attack/defense/speed/resource and <=16
  named data-configured actions with explicit target/cost/conditions.
- Damage/healing/resource recovery, replaceable poison/regen, deterministic
  speed order and dead skipping, victory/draw/terminal action rejection.
- Persistent generic StateProvider and independently removable Core rule instance.
  Public four-scalar rule input -> real decision -> complete provider application.
- Positive monotonic command identity, duplicate/stale/conflicting atomic rejection.
  Invalid commands do not consume IDs or advance clocks/state/pixels.
- New native Rust/C P5 interface and save2 bundle with full command/lifecycle/
  camera/view replay, canonical Core/P3 comparison and atomic restore.
- Reused P3 camera/View and canonical CPU renderer; headless/linked same authority.
- Minimal sample and existing Flutter client adapter; no Dart combat simulation.

P0-P3/legacy engine source/tests are byte-preserved. P4 implementation and v1
Rust/JSON/C/save semantics remain. Eight exact client integration/fixture files are
intentionally exempted from historical whole-client byte-preservation scans;
all other old client files and Android app ID/certificate checks are preserved.
No dependencies added and Cargo.lock remains unchanged.

## Executed evidence

Host:Linux x86_64 infrastructure, Rust1.99.1. Raw logs in `evidence/p5/`.

| Check | Result/evidence |
|---|---|
| cargo test --locked --workspace | 181/181 PASS, workspace-tests.log |
| workspace fmt + strict all-target Clippy | PASS, boundaries*.json/workspace-clippy.log |
| Gameplay v2 | 5/5 PASS (included in181) |
| P5 native contracts + actual FFI | 5/5 +2/2 PASS (included in181), native-tests.log |
| Independent worker rerun | 5 Gameplay +4 native +2 FFI PASS, independent-audit.md; not added to181 |
| Actual public ctypes consumer | 6 verification groups PASS,11 accepted commands to team0 victory, public-consumer.json |
| Out-of-tree packaged SDK consumer | Same6 groups PASS; internal reproducibility only |
| Real C header/linkage | P4+P5 symbols coexist and reject stale handle PASS |
| P2/P3/P4 boundaries | PASS, boundaries-final.json |
| Five legacy games | deterministic replay, canonical PNG golden and save/reload PASS, game-*.log |
| CLI no-default-features Clippy/tests | PASS, headless-*.log |
| release consistency | PASS, boundaries.json |
| filetree update/lint | Run after final evidence integration; see final commit/CI |

Native270-command test explicitly covers event history rollover beyond256,
resource shortage and reuse of a rejected command ID, trace budget rejection.
Independent additional actual C request changed canonical typed HP within valid
range:restore rejected invalid_save and exact prior save remained unchanged.
No separate isolated Blind Consumer certification is claimed.

## Defects, failures and correction

- P4 historical CI39138891574 at33b8193 failed only at stale filetree hash ledger;
  latest P4 head1d96402 passes clean-checkout lint. These are QA bytes, not engine
  behavior. Original P4 evidence and conditional acceptance remain intact.
- First local full workspace attempt failed linking host ALSA development library.
  Downloaded/extracted official build dependencies into a temporary prefix and
  fixed its library link; same command then passed181 tests. Failed log is
  workspace-env-failure.log. No product patch was made for this environment issue.
- Initial P2/P3 scanners correctly flagged the deliberate P5 client additions.
  Auditor approved the exact client allowlist; engine preservation checks
  remain. Initial/final boundary results are both retained.
- LOW UI default target could select a dead enemy with four actors; corrected to
  choose a living enemy. Two-fighter sample behavior unchanged.
- Local Flutter bootstrap was auto-review blocked for unexpected cloud metadata
  access. No bypass/retry; actual client checks use existing GitHub Actions.
- The independent four-fighter C probe completed301 commands, retained256
  history entries, checked5 tamper variants and lifecycle restore:PASS. It took
  283.234s in debug including repeated full replays; this is a performance limit,
  not a production latency benchmark. The native270 test is separate evidence.

## Measured cost and limits

One debug .so/reference run:100 headless resource commands7.823080s total,
204186-byte JSON save0.032144s, full100-command replay restore2.895827s.
This is a diagnostic sample with no controlled performance baseline, no speedup
claim, and no production latency guarantee. Replaying a large trace and serializing
full observations can be expensive. Retain bounded4096 trace; optimize only with
measured need and preserve staging correctness. ILCX™ nH+ grade remains a design
principle, not a benchmark/certification result from these tests.

No engine HIGH/MEDIUM defect remains in independently reviewed scope. Dynamic
plugins are trusted compiled instances, not a DLL loader. Physical ARM, platform
input/audio, signed Android installation and final public consumer acceptance
remain outside P5 technical scope. P6_HANDOFF defines <=7 sequential user gates.

## Client regression follow-up

Windows source6e758d5:37 UI tests passed, including both new P5 tests and old
FlatLand UI. One unchanged duplicate-import fixture failed because it encoded a
new ZIP twice but expected identical full-archive digest. Production correctly
hashes full bytes. Reuse exactly one byte array in duplicate import and reinstall
checks, preserving all assertions and product behavior. The auditor approved
this eighth exact preservation exception. NDK download ZipFile failure happened
before Android compilation and remains a separate environment failure.

## Final build acceptance

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

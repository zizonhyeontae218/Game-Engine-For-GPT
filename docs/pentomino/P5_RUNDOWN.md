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
Rust/JSON/C/save semantics remain. Seven exact client integration files are
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
  Auditor approved the exact seven-file allowlist; engine preservation checks
  remain. Initial/final boundary results are both retained.
- LOW UI default target could select a dead enemy with four actors; corrected to
  choose a living enemy. Two-fighter sample behavior unchanged.
- Local Flutter bootstrap was auto-review blocked for unexpected cloud metadata
  access. No bypass/retry; actual client checks use existing GitHub Actions.
- An exploratory independent C300+multiple replay probe has no final result and is
  NOT COMPLETED, never counted as PASS. The separate native270 test completed.

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

# External GPT Work tester instructions

Test this P2 candidate in a **separate GPT Work session/workspace** using only this
ZIP and public documents. Do not fetch the engine repository or read internal Core/
adapter source or existing implementation tests. A compiled artifact is allowed.
Do not certify behavior from documentation, compiler success or the supplied example
alone. Write new consumer tests and report actual observations.

## Environment and initial commands

Record `MANIFEST.json`, archive SHA256, exact Rust version/target and OS. Run:

```sh
python3 run_public.py --check
python3 run_public.py
```

Then write independent tests using `PUBLIC_API.md`, `CONTRACT.md` and discovery:

```sh
python3 run_public.py --source external_tests.rs --test
```

The runner invokes `rustc --test --edition=2024`, the supplied public rlib and
dependency libraries. For a normal custom executable omit `--test`. Compiler
metadata version mismatch or unavailable target is `BLOCKED`; obtain the exact
listed toolchain before evaluating API behavior.

## Required independent challenges

1. Install A and B; make A emit heavily beyond retention. Check B's retained
   events/dropped/RNG/records across A activity and removal. Verify global ordering
   and owner suffix accounting, then save/restore/continue.
2. Create/remove/recreate Scene/Entity; reject stale handles, foreign writes,
   invalid refs and cross-host handles. Test child/ref removal blockers, explicit
   bindings, consumer-first unload cleanup and refreshed handles after restore.
3. Exercise every typed field, nested bounded lists, UTF-8 byte limits, missing/
   extra fields, wrong ref tag, dead refs, 64KiB record limit and command/event
   quotas. Failed commands suppressed by a callback must still abort its tick.
4. Declare bool/bounded-i64 actions; replay explicit next-tick inputs, test duplicate,
   out-of-order/unknown/wrong-range frames and action privacy. Verify failed tick
   rolls back input, events, RNG, objects/identity and all plugins' records.
5. Require canonical save→restore→save byte equality and deterministic continuation.
   Corrupt/truncate/oversize a save and change content/descriptor metadata; reject
   atomically. Do not rely on undocumented internal save layout to write plugins.
6. Inspect public descriptors for hidden Camera/View/Gameplay assumptions. Describe
   any integration friction, unclear bounds, API contradictions or required
   undocumented knowledge. Legacy execution outside this SDK is `UNVERIFIED`.

Do not implement P3 to make P2 pass. Camera & View/Format are the next milestone.

## Return report

Return TL;DR, SDK commit/artifact checksums, environment, the new test files,
exact commands and relevant output, PASS/FAIL/BLOCKED/UNVERIFIED per challenge,
bugs with smallest public reproduction and expected/observed result, public API
clarity feedback, and whether the external P2 gate is accepted. Include unexecuted
areas explicitly. The user returns this report to the implementation session.

The original SDK was a candidate with external_gate UNVERIFIED at packaging time.
The user has now returned an independent23/23 PASS public Core report; that gate
is ACCEPTED. Codex did not perform the external test. Legacy execution remains
UNVERIFIED. Retain this instruction sheet for future independently evaluated artifacts.

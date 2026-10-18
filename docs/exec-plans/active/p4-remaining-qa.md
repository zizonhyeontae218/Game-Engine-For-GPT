# Endurance completion pending

## Outcome
Finish gates A-G with raw execution evidence and separate code/platform/release decisions.
## Context
Remote implementation72d49a63 and QA7258d48 unchanged. Existing completed regressions reused.
## Scope / non-scope
No product/client/API/dependency/signing changes. New evidence is additive.
## Acceptance evidence
Windows9/9 groups; Android emulator7/7 groups294 calls; ASan20/20; TSan20/20; Valgrind2/2,0exit bytes/errors/suppressions. Raw evidence under evidence/p4-remaining-20261018.
## Milestones
[x] Environment and original SDK integrity
[x] Windows native and Android emulator execution
[x] Linux instrumented Rust/FFI and Valgrind
[ ] Collect7200seconds per mode endurance
[ ] Final ledger, FILETREE, commit/push, PR8 and Issue6
## Decisions
2026-10-19: F remains NOT TESTED; agent tool permissions cannot enforce source-read isolation. No agent/out-of-tree run falsely certified.
Linux public standard free runners replace unavailable WSL. ASan prior zero-length rlib failure did not recur; exact historical cause unproven. Matching libc6-dbg and normal loader permit Valgrind; no broad suppression/security relaxation.
## Progress
Draft PR8 targets PR7 QA branch. Linux run39138455495 actually passed20+20 tests but jobs failed afterwards because QA assumed old Cargo output layout. QA hash collection fixed in33b8193; run39138832654 both succeeded with hashes. Valgrind first run retained; not repeated. Endurance running with active30s host resource supervisor.
## Verification log
Commands: evidence/p4-remaining-20261018/COMMANDS.md. First and corrected Linux raw logs retained separately. Windows/Android native execution does not imply Flutter app/UI, ARM or physical-device acceptance.
## Handoff
No temporary handoff file created. Official P4_HANDOFF and old evidence untouched. Endurance sessions89270/26866; supervisor52861. Wait to collect final reports before claiming G PASS. Finalize with finalize_evidence.py after monitor stops; then update/lint FILETREE and publish final GitHub evidence.

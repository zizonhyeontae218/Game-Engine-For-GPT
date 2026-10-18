# P4 remaining verification — completed2026-10-19

## Outcome
All executable remaining gates completed and evidence retained. A/B/C/D/E/G PASS; F NOT TESTED because enforced agent source/tool isolation is unavailable. Full P4 acceptance/release withheld; code merge conditionally supportable and native platform approval bounded.
## Context
Implementation72d49a63 and previous QA7258d48 unchanged at initial remote comparison. Existing completed functional/regression evidence reused, not recounted.
## Scope / non-scope
Evidence/QA only. No product/client/ABI/dependency/game/save/signing change, P5, merge, release or user-data deletion.
## Acceptance evidence
Windows9/9 groups; Android16/API36 x86_64 emulator7/7 groups294calls; ASan20/20 final tests; TSan20/20 final tests; Valgrind2/2 processes, zero exit bytes/errors/suppressions. Both sanitizer first executions also passed20tests each; jobs then failed in QA hash collection. Headless7200.109s/68972steps, Linked7200.032s/67608steps; exit0 both. Raw logs and hashes: evidence/p4-remaining-20261018.
## Milestones
[x] Original SDK integrity and unchanged source/previous evidence verified
[x] Actual Windows/Android native calls
[x] Linux ASan/TSan and original SDK Valgrind
[x] Monitored >=2hours per mode mixed endurance, final process exits collected
[x] Approval boundaries and F limitation documented
[x] GitHub PR8 follow-up created and attached; final result committed/pushed and Issue6 updated in publication step
## Decisions
Use normal public-repository standard free Linux runners instead of unavailable local WSL. No security relaxation or broad suppression. Preserve prior Drive review instead of repeating search. User cancelled further local-repository discovery; fixed worktree used thereafter, no further setup or test restart. Manual progress polling reduced to five minutes; resource-only guard sampling remains30s.
F is a P4 independent-consumer acceptance limitation, not an engine FAIL. Flutter/platform app delivery is later P4_HANDOFF item7 scope and not a reason to classify all P4 code as failed.
## Progress
Completed all possible execution. No engine defect reproduced, no product fix. QA hash-path bug fixed33b8193. Resource supervisor stopped after both exit0; no resource cap triggered. No temporary handoff created; official handoff and historical evidence preserved.
## Verification log
Commands and versions: evidence/p4-remaining-20261018/COMMANDS.md. Linux first run39138455495 and corrected run39138832654; only the two sanitizer jobs repeated once after changed hash collection. Valgrind not repeated. Existing169/169workspace and other old passing checks remain prior evidence. Finalization validates duration/continuity/exit/metrics; FILETREE update/lint and clean committed-checkout ledger checked before final report.
## Handoff
PR8 targets qa/p4-independent-final-20261016 (PR7), related PR5/Issue6. Future work: provide genuinely enforced blind consumer session; perform subsequent app/ARM/device/package acceptance before platform release. No PR merge or release authorized/executed. This permanent plan is not a temporary WORK_HANDOFF.

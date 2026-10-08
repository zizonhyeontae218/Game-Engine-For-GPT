> Archived after P0/P1 main integration on 2026-10-08. The baseline/tool results below are historical. Current alpha scope and next work are in docs/pentomino/STATUS.md.

# Pentomino P0 task packets

## Task packet — ID: PENTA-P0-S

**Goal:** discover actual repository baseline. **Milestone / Gate:** P0/G0.
**Owner role:** pentomino_scout, read-only.
**Prerequisites verified:** main checkout, AGENTS/HANDOFF/BRIEF/role instructions available.
**Known language/build/test tools:** Rust Cargo workspace observed; detailed tools pending scout.

### Scope
- OWNED files/paths: none.
- READ-ONLY reference files: actual manifests, source boundaries, docs, tests, workflows and role instructions relevant to inventory.
- FORBIDDEN files/actions: all edits, installs, test runs, publishing.
- Expected integration dependency: scout report feeds P0-A.

### Contract
- Inputs/capabilities: verified checkout and P0-S bootstrap instructions.
- Outputs/observable behavior: paths/symbols, tool observations, public APIs and three strongest boundary risks.
- Error/unload/rollback behavior: report missing evidence; do not invent internals.
- Backward compatibility/ABI: preserve all current contracts.
- No-go / v0.4 boundary: no functionality or Forge work.

### Definition of done
- [ ] Scope and dependency direction preserved.
- [ ] Executable tests added / updated: NOT_APPLICABLE (read-only).
- [ ] Actual inventory commands/results captured; engine tests NOT RUN/UNVERIFIED.
- [ ] API/docs updated: parent integration only.
- [ ] Handoff includes blockers/unknowns.

## Task packet — ID: PENTA-P0-A

**Goal:** minimum public API proposal grounded in P0-S. **Milestone / Gate:** P0, future G1/G2/G6.
**Owner role:** pentomino_architect, docs-only.
**Prerequisites verified:** P0-S report required before dispatch.
**Known language/build/test tools:** use scout evidence; no test execution.

### Scope
- OWNED files/paths: `docs/pentomino/P0_API_PROPOSAL.md` exclusively during architect turn.
- READ-ONLY reference files: scout report, AGENTS, HANDOFF, BRIEF, GATES, actual API/source/test boundaries.
- FORBIDDEN files/actions: all other edits, source/lockfile/schema/ABI/version changes.
- Expected integration dependency: auditor reads completed proposal; parent integrates only after architect releases ownership.

### Contract
- Inputs/capabilities: current engine evidence; minimum typed contracts only.
- Outputs/observable behavior: proposed API, lifecycle, errors, resource/event/input ownership, view-independent boundaries, public discovery, compatibility and executable acceptance test specifications.
- Error/unload/rollback behavior: explicit registration failure and cleanup/dependency behavior; avoid unsafe generic registries and global mutable state.
- Backward compatibility/ABI: additive proposal; document current versus proposed surfaces.
- No-go / v0.4 boundary: no Forge, marketplace, network updater or executable imported plugins.

### Definition of done
- [ ] Scope and dependency direction preserved.
- [ ] Executable tests added / updated: NOT_APPLICABLE for P0; specify future tests.
- [ ] Actual tests NOT RUN/UNVERIFIED recorded.
- [ ] API proposal labels unimplemented behavior UNVERIFIED.
- [ ] Handoff includes blockers/unknowns.

## Task packet — ID: PENTA-P0-R

**Goal:** independent attempt to refute engine independence. **Milestone / Gate:** P0 contract review.
**Owner role:** pentomino_auditor, read-only.
**Prerequisites verified:** completed P0-A draft; source evidence.
**Known language/build/test tools:** inherit scout observations; review does not establish runtime success.

### Scope
- OWNED files/paths: none.
- READ-ONLY reference files: proposal, baseline source and relevant docs/tests.
- FORBIDDEN files/actions: all edits, tests, publication.
- Expected integration dependency: parent owns sequential corrections and review report.

### Contract
- Inputs/capabilities: proposed public contract and source baseline.
- Outputs/observable behavior: exact path:line findings, severity, proven defects versus hypotheses, smallest test resolving each uncertainty, technical recommendation.
- Error/unload/rollback behavior: challenge cascading unload, rollback, undeclared dependencies, renderer imports and singleton state.
- Backward compatibility/ABI: reject undocumented breakage.
- No-go / v0.4 boundary: no consumer certification or implementation.

### Definition of done
- [ ] Scope and dependency direction preserved.
- [ ] Executable tests added / updated: NOT_APPLICABLE; identify required tests.
- [ ] Actual runtime tests NOT RUN/UNVERIFIED.
- [ ] Public contract review delivered to parent.
- [ ] Handoff includes blockers/unknowns.

**Escalate immediately if:** ownership overlap, unsupported assumptions, private API requirement, ABI break, failed isolation or destructive action.

Parent exclusive ownership: this packet file, `pentomino-p0.md`, `docs/pentomino/P0_SCOUT.md`, `docs/pentomino/P0_AUDIT.md`, `docs/pentomino/P0_RUNDOWN.md`, the user-provided `QUICKSTART.ko.md`, and generated `FILETREE.md`/`FILETREE.hash.json`. Parent never writes architect's proposal concurrently. All other shared files and runtime sources remain read-only.

## Final P0 packet disposition

P0-S/P0-A/P0-R scope checks, source evidence, proposal and handoffs complete. Runtime tests NOT RUN/UNVERIFIED; no executable tests were added for this documentation-only work. Architect released ownership before parent corrections; auditor performed independent read-only re-review. Shared ownership never overlapped. The missing quickstart was restored from the user attachment, not fabricated. Actual role spawning ran with inherited platform permissions and explicit packet restrictions; repository read-only declarations were not independently proven to be OS-enforced. Runtime sandbox enforcement remains UNVERIFIED.

# Repository cleanup — 2026-10-09

## Outcome

main clearly identifies Pentomino alpha, stable FlatLand remains on release/0.2,
public releases contain usable packages, and documentation separates current work from history.

## Context and scope

The user authorized repository cleanup, reports all existing tests completed, and
requires Ponytail for new code. Do not refactor engine code or merge active P2/P3 work.
Preserve stable tag, original binary bytes, signing identity, and private Drive sharing.

## Acceptance evidence

Existing checklist completion is user-reported. New C example and recovery guards
have targeted executable tests. Links, FILETREE, agent settings and release consistency
are checked locally. Windows recovery CI must compile the C example against the
original DLL before public upload; uploaded originals must match the 14-file manifest.

## Milestones and decisions

README and document index become concise entrypoints; old RC narratives move to history.
Merged P0/P1 refs may be removed after ancestry verification; P2/P3 stay open.
Public distribution uses GitHub Release assets. Private Drive remains archival.
Because local uploads.github.com authentication returns 401, use the existing
authorized Actions runner and its scoped GITHUB_TOKEN to upload verified artifacts.
Temporary download URLs are workflow inputs, never checked-in files or printed logs.
Stable AgentKit uses v0.2.0 source; 0.3 roles are already installed on main.

## Progress

Complete: documentation and local checks, merged P0/P1 ref removal, Windows C
validate/open and release recovery CI, public release body and 17 attached assets.
P2/P3 remain open; stable tag and original binaries are unchanged.

## Verification log

2026-10-09: settings validation PASS (10 roles, 2 skills), Python tests 7/7 PASS,
release consistency PASS (21 current documents), no broken relative Markdown links,
original release files 14/14 match recorded sizes and SHA256.
Windows recovery run 37923840175 PASS. GitHub assets 17/17 present; originals
14/14 match both API sizes and published SHA256 digests. New C SDK and AgentKit
include provenance and versioned source respectively. No new test checklist added.

## Handoff

Read docs/releases/README.md for package roles and actual publication status.
Do not request or expose private signing keys; this task reuses the original signed APK.
Follow-up test reports do not mean unmerged Pentomino features exist on main.

# FlatLand 0.2.0-rc.5 final stabilization

## Outcome
Ship rc5 Android/Windows candidate with collision/layering/occlusion corrections,
scoped actor-linked story bubbles, truthful building semantics and consistent docs.
Do not promote final until user physical mobile acceptance explicitly passes.

## Context
rc4 physical feedback: roof-looking traversal, player layer priority swallowing NPCs,
shadows painted amongst entities, ignored building facing, no story bubble flow.
Save, persistent roster and view/simulation split must remain stable.

## Scope / non-scope
Correct reported issues only. Keep semantic fallback, six battle FX, input priority,
UTF-8 pipeline and original Harbor. No 3D experiments or suspended platform clients.

## Acceptance evidence
Rust/native tests plus Windows Flutter tests and real rendered shadow/occlusion,
building traversal, three-line bubble, battle and save/view snapshots. Physical
mobile checklist is a final promotion requirement that CI cannot satisfy.

## Milestones
1. Correct contact ordering/shadows and normalized solid building footprint.
2. Scoped speech bubbles and light camera/bubble transition without simulation coupling.
3. Battle FX polish, regression coverage, independent Harbor acceptance cases.
4. Documentation/version cleanup and Android/Windows builds, signed Drive delivery.

## Decisions
rc5 uses Flutter0.2.0-rc.5+7 / Androidcode7 to update code6 with identical signing.
Final, if accepted, may use +8/code8; preserve every previous binary.
South-facing billboard buildings only in0.2: reject unsupported facing variants.
Do not pretend north/east/west rendering exists. 3D/pixelization is0.3 roadmap only.

## Progress
Implemented solid normalized semantic footprints, contact sorting/shadow band,
South-only validation, actor-linked tap bubbles and scoped12-tick camera easing.
Heal/guard motion corrected. Original Harbor three-line story and rendered acceptance
script covers roof/contact/battle/save cases; docs archived/current truth aligned.
Local63Rust tests, strictclippy, Flutteranalysis and three replay/save goldens pass.
Next: publish coherent source for Windows tests/render and Android builds; signcode7,
inspect actual captures, deliver new rc5 Drive folder. Final promotion awaits human.

## Verification log
Local tests63passed; strictclippy passed; Flutterstaticanalysis passed. Harbor420-tick
journey golden568334e7d0aaf36ebd4ed958f431dbfb1da0713e2f1ba4dc3a29f5bc084448a5,
Pac-Man and SignalYard goldens/save pass. Hosted clients pending.

## Handoff
Repo /workspace/Game-Engine-For-GPT. Signing keys /workspace/signing/android remain
private. Deliver new Demos/Basement0.2FlatLand/0.2.0-rc.5 directory, preserve rc1-rc4.

# GE4G 0.3.0 — Pentomino alpha development

Current main version: 0.3.0-alpha.1; Flutter 0.3.0-alpha.1+9.
See [Pentomino status](pentomino/STATUS.md) for implemented scope and next work.
The FlatLand material below is the inherited implementation/released 0.2 baseline.
Its historical versions, delivery instructions and evidence are not a new alpha release.
Existing acceptance checks remain regressions; also run
`cargo test --locked -p ge4g-pentomino` for the experimental scalar host.
Android/Windows remain the client verification matrix. No alpha binary is published here.

## Basement 0.2 — FlatLand 0.2.0 FINAL

Completed FlatLand0.2 feature set. Public sample: Harbor Workshop / 바람항 공방,
original CC0 content/provenance. Presentation-only camera and upright composition,
contact shadows/feet ordering, solid South-only semantic buildings with fallback,
step movement, combat/inventory/quests, persistent HP/PP, event scenes, turn battles,
six built-in battle FX, actor-linked cutscene bubbles with optional semantic speakers,
Lua5.4 with deterministic RNG, sound/music adapters and schema2 exact resume.

Android .ge4g updates use stable identity, transactional activation/rollback, typed
incompatible-save archival, duplicate prevention and persisted order. Game Manager
confirms normal/full deletion; settings survive normal updates/deletion. Embedded
Windows stays locked to its bundled game.

Rust0.2.0 / Flutter0.2.0+8 / Android versionCode8. Existing app ID and signing
certificate unchanged. Android/Windows are supported clients; iOS/macOS/Linux work
is deferred until0.3. 3D/pixelized exploration is0.3+ scope. No rc6 introduced.
User confirmed rc5 physical acceptance and authorized this finalization. New library
management and speaker semantics require the recorded final automated checks; do
not claim an additional physical test. [Release evidence](FLATLAND_RELEASE.md).
Historical rc1–rc5 artifacts remain unchanged under [releases](releases/).

Final verification/publication:2026-10-06.65 Rust workspace tests,9 headless CLI
tests,36 Flutter tests and Android/Windows release+UTF-8/render checks pass.
[Final delivery and measured evidence](FLATLAND_RELEASE.md).

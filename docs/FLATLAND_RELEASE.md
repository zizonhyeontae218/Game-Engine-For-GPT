# Basement 0.2 — FlatLand 0.2.0

Released:2026-10-06. Rust **0.2.0**, Flutter **0.2.0+8**, Android **versionCode8**.
Supported release clients: **Android and Windows**. iOS/macOS/Linux client work is
suspended until0.3.0 development begins;3D/pixelized exploration is0.3+ scope.
Historical rc1–rc5 binaries and provenance remain preserved. No rc6 was created.

## Included systems

Rust-authoritative deterministic2D/2.5D composition, presentation-only persistent
views, upright actors, contact shadows/feet ordering, solid South-only semantic
building fallbacks, step movement, combat/inventory/quests, persistent combatant
HP/PP, event scenes, turn battles/battle FX, actor-linked speech bubbles,
embedded Lua5.4/deterministic RNG, actual cue/music playback adapters and
schema2 exact-once save/resume. The public sample is original CC0 Harbor Workshop
(**FlatLand0.2 / 바람항 공방**).

The finalization fixes current documentation, optional native-resolved bubble
speakers and stable-game_id library management. Speaker precedence is authored
label, actor display_name/name, then no header. Normal updates resume compatible
saves; typed revision mismatches archive the old save and start fresh. Failed
activation restores the old revision/data; duplicate imports preserve identity/order.
Updates retain controls. Management supports persisted order and two named,
confirmed deletion levels. Ordinary deletion retains saves/settings. Full deletion
also removes saves/archives/settings. Embedded Windows exposes no library manager.

## Verification

Build source: `cdb7eb1d336dee95ea2b1e4d82ddc93d478e4098`. Later release-documentation changes do
not alter runtime/client/game code or the verified binaries.

- [Engine acceptance](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37483880616): format, strict workspace/headless clippy,
  **65 workspace tests**, **9 additional headless CLI tests**, four replay/save goldens.
- [Client acceptance](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37483880756): Flutter analysis, **36 tests**, Windows
  release, Android release with arm64-v8a/armeabi-v7a/x86_64 native engines.
- Windows Korean TOML/JSON/JSON5/resources/paths passed authoring→validation→package→
  import→launch→render. Real portrait/landscape captures contain181 evidence files.
- Actual frames show authored **안내인** headers, upright composition, contact ordering,
  blocked normal roofs, attack/red flash/HP interpolation and save/view/combatant restore.
- Exact world snapshots match across view changes. Portable and embedded game entries
  match. Android package ID remains`dev.ge4g.ge4g_client`; signed APK is0.2.0/code8.
- APK v2/v3 signature verifies. Its certificate matches both the pinned fingerprint
  and the actual rc5 APK: `d6d5ca948e5c1ed644478d7c4c3243efcfcbc30a196f03c39ef7af7118fc00d4`.

The user accepted rc5 engine/render/control/cutscene behavior physically and authorized
this final promotion. New speaker/library changes were verified automatically;
no new physical Android play-check is claimed. CI graphic captures are muted and
are not physical audio-device acceptance.

## Download

[GitHub v0.2.0 release](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/releases/tag/v0.2.0)
is the public distribution entrypoint. The original 14 files are recovered byte-for-byte;
C SDK and AgentKit are separate additions. Current upload status and artifact roles
are documented in [release distribution](releases/README.md). Original Drive delivery
metadata is retained as historical provenance, not a public download requirement.

| Artifact | SHA256 |
|---|---|
| [GE4G-FlatLand-0.2.0-Android.apk](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/releases/download/v0.2.0/GE4G-FlatLand-0.2.0-Android.apk) | `772d9dbae32ec32840e525b2df27233a9dc3412bc524b9488d9cf555bfa9775c` |
| [GE4G-Harbor-0.2.0-Windows.zip](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/releases/download/v0.2.0/GE4G-Harbor-0.2.0-Windows.zip) | `bc080ab5d623c632ca1c1fc40a739bb13d34166c6056eeb01cbfaf890a72d61b` |
| [FlatLand-Harbor-0.2.0.ge4g](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/releases/download/v0.2.0/FlatLand-Harbor-0.2.0.ge4g) | `1ec1630ec9e6eed3f4a6ade1609955184b285f416738ce58a77a6d31e97ac23d` |

[Full verification metadata](releases/flatland-final-verification.json) · [Landscape world/battle/bubble proof](releases/flatland-final-landscape-proof.json) · [Original private-Drive delivery provenance](releases/flatland-final-drive.json).

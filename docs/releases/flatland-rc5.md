# Historical rc5 release evidence

This record describes rc5, not the current final product.

# Basement 0.2 — FlatLand 0.2.0-rc.5

Final stabilization candidate, not final0.2.0. Flutter0.2.0-rc.5+7, Androidcode7.
Android/Windows only. iOS/macOS/Linux support remains suspended until0.3 begins.
Preserve all rc1/rc2/rc3/rc4 binaries and their historical release records.

## Corrections

Contact shadows have a dedicated ground band: above floor, below upright entities.
Top shadows are subtle and feet-attached. Upright ordering uses plane, gameplay
height, body-foot Y and stable ID, not sprite top or player's former layer priority.
Legacy packages without entity_defaults retain authored layer behavior.

Semantic building footprint defines actual body size; omitted body defaults fixed.
Roof/facade pixels remain presentation only. View switching never changes collision,
position, plane, elevation, RNG or gameplay rules. Ordinary roofs grant no traversal.
Only south-facing semantic billboard buildings are supported in0.2; other facing
values are rejected. Component-level custom art/fallback and intentional transparency
remain supported. 3D/pixelized buildings are0.3+ exploration, not0.2 scope.

New event-scene say_bubble attaches to a native-projected actor anchor. Three-line
Harbor story advances by tap, softly transitions, scopes camera movement over12ticks,
locks gameplay input and resumes exact current line from a save. Ordinary dialogue
remains separate. Camera transition is cosmetic; persistent gameplay view survives.

Six battle FX and persistent HP/PP remain authoritative/exactly-once. Heal/guard have
no offensive lunge/recoil; damage retains red feedback.54ticks/effect (~0.9s) remains
skippable; two-fighter round is ~1.8s. Save/load remains conservatively extended.
Harbor Workshop stays independently authored with original CC0 assets/provenance.

## Verification / delivery

Source: `9ead55bd67500e55250974343ff22077266827d2` (2026-10-06).
[Engine CI](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37461167770)
passed63 workspace Rust tests, formatting/strictclippy, headless and replay/save goldens.
[Client CI](https://github.com/zizonhyeontae218/Game-Engine-For-GPT/actions/runs/37461167767)
passed Android release build,25 Windows Flutter tests, Windows release/embedded build,
and Korean UTF-8 authoring-validation-package-import-launch-render pipeline.

Actual Windows portrait/landscape captures were downloaded and visually reviewed:
feet-attached top shadows, proximity ordering, upright buildings, three-view blocked
footprint attempts, attack impact/red flash/HP interpolation, guard/heal, three actual
UI bubble taps and restored view/roster. Completed bubbles cannot reopen as dialogue
history. Native projected feet keep a lower story bubble clear of its actor.
CI graphics capture is explicitly muted; audio_device_validation:false. This does
not verify physical audio or physical Android touch/movement acceptance.

[Drive candidate folder](https://drive.google.com/drive/folders/1gIi8VYH32CkShkqsiXkIFylRh79rw_Du)
contains signed APK, embedded Harbor Windows ZIP, portable game, game source ZIP,
rendered screenshots/status/world snapshots, world-state proof, Korean instructions,
physical acceptance checklist, verification JSON and SHA256 checksums. Uploaded file
names, sizes and parents were read back. Existing sharing and rc1–rc4 files retained.

Android app ID `dev.ge4g.ge4g_client`, versionCode7; three native ABIs included.
Certificate SHA256 `d6d5ca948e5c1ed644478d7c4c3243efcfcbc30a196f03c39ef7af7118fc00d4`
was compared with the existing rc4 APK and is unchanged. APK SHA256
`dddbc6dbc48bd3b028f7fb532c46d583ef2a8fe6cd72a107261a4e00200fb4b6`.
Harbor game SHA256 `9d3aad1758b62453b87e35990cb4e9e7db229a08ab657a6d8726e3e8dbd7d80d`;
Windows ZIP SHA256 `b0213be63033b335864d23118ff2bcc70930dff935729044adace95c02339698`.

## Final promotion gate

Physical mobile acceptance is PENDING. CI does not grant final promotion.
The user must confirm shadows, occlusion, blocked roofs, movement/run feel, view
invariance, bubble/tap flow, readable combat and solid save/load on a real device.
If any fail, fix within rc5. Only after every gate passes may this exact codebase be
promoted with version/release-note/artifact-only changes. Keep appID/certificate;
final may use Flutter0.2.0+8 / Androidcode8 for an installable update.

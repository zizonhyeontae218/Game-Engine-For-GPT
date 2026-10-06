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

Pending recorded Android/Windows CI and actual rendered review. Rust tests cover new
collision/shadow/contact/bubble/scoped-camera cases alongside rc4 save/input/battle,
Pac-Man and Signal Yard regressions. Windows captures top/depth, occlusion, attempted
roof traversal in three views, damaging/guard/heal feedback, three actual UI taps,
and restored view/roster. Korean UTF-8 pipeline remains required.

## Final promotion gate

Physical mobile acceptance is PENDING. CI does not grant final promotion.
The user must confirm shadows, occlusion, blocked roofs, movement/run feel, view
invariance, bubble/tap flow, readable combat and solid save/load on a real device.
If any fail, fix within rc5. Only after every gate passes may this exact codebase be
promoted with version/release-note/artifact-only changes. Keep appID/certificate;
final may use Flutter0.2.0+8 / Androidcode8 for an installable update.

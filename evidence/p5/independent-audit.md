# Independent P5 review — 2026-10-30

Worker: pentomino_auditor, separate read-only worker with source access.
This is independent source review/execution, not isolated Blind Consumer certification.

Result: no remaining HIGH/MEDIUM defect in reviewed Gameplay v2/native/FFI scope.
Independent execution: Gameplay5/5, Native4/4, actual C ABI2/2 PASS.
Additional actual C request altered an existing canonical I64 HP by -1 within valid
range; restore rejected invalid_save and preserved prior full save exactly.

Reviewed provider authority, full scalar batches, immutable rule config, monotonic
persistent IDs, Core remove/install, replay comparison, staged Core/View/frame,
P4/P5 globally disjoint non-recycled handles and FFI thread/string ownership.
LOW: multi-fighter default target could select a dead enemy; parent corrected to
select a living enemy. Initial two-fighter sample was unaffected.

Approved exact client integration allowlist in P2/P3 preservation scanners; all
other client and P0-P3 engine/test byte checks remain. P4 C export scanner permits
only existing P4 and new P5 prefixes. Android certificate/ID remain pinned.

The pending exploratory probe later completed:301 public C commands, retained
history256,5 tamper variants,lifecycle restore PASS in283.234s debug. This is
a performance limitation, not a production benchmark. Its HP variant used a
malformed Value; the earlier separate typed HP test proves schema-valid rejection.
Keep it separate from the native worker's270-command test. No new probe was run.

Follow-up source review approved an eighth exact allowlisted client test file:
final_library_test.dart now reuses identical ZIP bytes for duplicate/reinstall
identity checks. Fresh re-encoding can change ZIP timestamps; production hashes
full bytes correctly. All assertions and game_library.dart behavior stay intact.
Flutter builds, physical devices, performance gains and blind certification are
outside this audit's certification scope.

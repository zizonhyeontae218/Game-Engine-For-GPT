# Independent GPT Work instructions

P3 외부 테스트는 2026-10-09 사용자 보고로 성공 처리됐다. 아래는 향후 배포물 재검증 절차다. Use only this package and your own source files in
an independent workspace/session. Do not read the engine repository, internal
implementation or implementation tests. Documents describe expected behavior;
only executable observations may establish PASS.

1. Verify ZIP SHA256, MANIFEST file hashes and exact rustc; record package identity.
2. Write new consumer tests using only PUBLIC_API/CONTRACT/discovery. Discover and
   install all four View families; exercise multiple active cameras and viewports.
3. Observe hot replacement/removal/reinstall preserving Core canonical bytes,
   Scene/Entity identities, records, RNG continuation and unrelated plugins.
4. Test world/view/camera/screen-depth roundtrips and real perspective; invalid
   finite bounds/NaN/overflow must reject and preserve presentation save bytes.
5. Test follow/bounds/dead-zone/lookahead/smoothing/shake; verify source immutability.
6. Test default smooth 2.5D pose AND View-transform change, explicit Instant,
   midflight retarget, inactive-camera timing, canonical midpoint save/restore
   and identical future output/state. Malformed saves must reject atomically.
7. Use the included EXISTING fifth fixture via public legacy adapter, render real
   assets and verify changed pose changes output while authoritative state stays
   identical. Check midpoint continuation and View removal/reinstall. Do not invoke
   legacy view menus that also change gameplay plane/elevation/state.
8. Report command, new test source, observed assertions/logs, artifact hashes,
   failure reproducers and PASS/FAIL/BLOCKED/UNVERIFIED per goal. Do not label this
   external test complete merely because examples pass. Preserve raw logs/source.

Full renderer/client/device acceptance, arbitrary native sandboxing and arbitrary
cross-platform float equality are outside this SDK gate; mark untested scope
UNVERIFIED. The orchestrator cannot certify this external session itself.

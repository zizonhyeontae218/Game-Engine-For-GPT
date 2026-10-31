# Concise decision record

- 2026-10-19 — The production Flutter client contains no P4 ABI wiring, and P4 handoff section 7 explicitly defers Android client integration. Product P4 Flutter integration is therefore `NOT DELIVERED`; running the inherited FlatLand app would not answer this task.
- 2026-10-19 — Use a dedicated Dart entry point copied into an ignored, isolated Flutter wrapper whose package is `dev.ge4g.ge4g_client.p4harness`. The production Gradle file, production ID `dev.ge4g.ge4g_client`, release signing configuration and pinned certificate remain unchanged.
- 2026-10-19 — Package the existing P4 `cdylib` from an ignored local `.dart_tool` path and call only the published `ge4g_p4_v1_*` ABI. The harness renders the returned canonical RGBA bytes and does not reimplement combat in Dart.
- 2026-10-19 — Reuse the prior Windows/emulator/sanitizer/endurance record as background only. This run reports only new Flutter ARM64 physical-device observations.

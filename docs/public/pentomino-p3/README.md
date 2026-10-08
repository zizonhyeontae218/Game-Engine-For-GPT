# Pentomino P3 public Camera/View SDK

This compiler-pinned SDK exposes independent presentation consumers on accepted
P2 Tiny Core. Four linked Format plugins, multi-camera state, explicit 3D
coordinate transforms, orthographic/perspective math, tick-based composable
behavior and default smooth transitions are public surfaces.

**EXTERNAL VALIDATION PENDING.** Packaging checks and in-repository tests are
internal evidence. A separate tester must write new consumer tests. No finished
3D GPU renderer, dynamic native loading, gameplay plugin or client release is
claimed. The legacy raster adapter supports a documented subset of native math.

Start with QUICKSTART.ko.md and PUBLIC_API.md. CONTRACT.md defines the exact
Camera/View contract; CORE_CONTRACT.md supplies consumed P2 types. Use structured
ViewHost::describe(), not implementation inspection, to choose capabilities.

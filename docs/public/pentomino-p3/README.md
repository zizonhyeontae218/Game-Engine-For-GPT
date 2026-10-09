# Pentomino P3 public Camera/View SDK

This compiler-pinned SDK exposes independent presentation consumers on accepted
P2 Tiny Core. Four linked Format plugins, multi-camera state, explicit 3D
coordinate transforms, orthographic/perspective math, tick-based composable
behavior and default smooth transitions are public surfaces.

**EXTERNAL VALIDATION PASS (user report, 2026-10-09).** Packaging checks and in-repository tests remain internal evidence; acceptance is separately user-reported. No finished
3D GPU renderer, dynamic native loading, gameplay plugin or client release is
claimed. The legacy raster adapter supports a documented subset of native math.

Start with QUICKSTART.ko.md and PUBLIC_API.md. CONTRACT.md defines the exact
Camera/View contract; CORE_CONTRACT.md supplies consumed P2 types. Use structured
ViewHost::describe(), not implementation inspection, to choose capabilities.

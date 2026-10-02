# ge4g_native

Dart FFI bindings to the version 1 GE4G native client ABI in `crates/ge4g-client`. The Flutter/Dart code-assets build hook compiles that crate using Cargo for the requested target and bundles the resulting library. This local package must be built from the GE4G repository checkout.

`BasementNative.request` sends versioned JSON operations; `frame` copies the canonical Rust RGBA pixels. Returned C strings and temporary buffers are freed deterministically. No mobile subprocess, downloaded executable or Dart gameplay implementation is involved.

Supported targets: Android arm64/armv7/x64, iOS arm64 device/simulator and x64 simulator, Windows x64/arm64, Linux x64/arm64. Install the corresponding Rust target and Flutter platform tools before building. Actual release CI covers Android, iOS device, Windows x64 and Linux x64.

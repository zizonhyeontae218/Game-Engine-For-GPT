# ge4g_native

Dart FFI bindings to the version 1 GE4G native client ABI in `crates/ge4g-client`. The Flutter/Dart code-assets build hook compiles that crate using Cargo for the requested target and bundles the resulting library. This local package must be built from the GE4G repository checkout.

`BasementNative.request` sends versioned JSON operations; `frame` copies the canonical Rust RGBA pixels. Returned C strings and temporary buffers are freed deterministically. No mobile subprocess, downloaded executable or Dart gameplay implementation is involved.

FlatLand 0.2 release targets and CI: Android arm64/armv7/x64 and Windows x64. Install the corresponding Rust target and Flutter platform tools before building. Existing iOS, macOS and Linux hooks are preserved, but client builds, tests and distribution for those platforms are deferred until 0.3.0 development begins.

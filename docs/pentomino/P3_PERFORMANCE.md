# P3 observed performance / x86_64 release

Command: `cargo run --locked --release -p ge4g-pentomino-view --example p3_perf`.
Exact workload and five100-call batches: evidence/p3/performance.json and log.
Rust1.99.0, Linux-hosted development container; scheduling/load is uncontrolled.
No cross-release optimization or arbitrary hardware latency claim is made.

| Cameras | Visible source entities | Median update | Median Core step |
|---|---:|---:|---:|
| 1 | 1 | 0.010ms | 0.007ms |
| 1 | 256 | 0.361ms | 0.638ms |
| 16 | 256 | 1.399ms | 0.674ms |
| 32 | 256 | 2.746ms | 0.658ms |

Selection of4096 real Core entities across16 owners measured
4.848ms/call.
Builtin RecordBinding selects one owner: many-visible workload is256 entities,
not a fabricated4096-entity single-owner Core. Static orthographic camera updates
project each selected entity and serialize bounded camera-local state; no renderer
or asset I/O occurs in this timing. Core save bytes stayed identical during all
camera-only updates. Core step is measured separately with native host installed;
there is no camera callback inside Core. Simulation+selection+presentation pipeline
costs are explicit; more cameras increase the presentation work.

UNVERIFIED: comparison against a P2 timing baseline, real-device/client latency,
maximal eight-View custom4096-item workloads, cross-platform float equality or
complete mixed3D renderer. Current selection validation does repeated linear
entity searches; larger custom selections can cost quadratically. No performance
uplift is claimed from these measurements.

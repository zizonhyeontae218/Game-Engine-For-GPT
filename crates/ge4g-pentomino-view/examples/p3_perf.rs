//! Reproducible public-surface timing harness, not a cross-release speed claim.
#[allow(dead_code)]
mod support;
use ge4g_pentomino::p2::{CoreHost, InputFrame};
use ge4g_pentomino::{Bindings, PluginId};
use ge4g_pentomino_view::*;
use std::time::Instant;
fn target() -> CameraTarget {
    CameraTarget {
        pose: CameraPose {
            position: [80.0, 80.0, 100.0],
            orientation: Quaternion::identity(),
            zoom: 1.0,
        },
        view_transform: ViewTransform::default(),
        projection: Projection::Orthographic {
            half_height: 120.0,
            near: 0.1,
            far: 1000.0,
            focus_distance: 100.0,
        },
    }
}
fn timings(mut run: impl FnMut()) -> Vec<u128> {
    (0..5)
        .map(|_| {
            let start = Instant::now();
            for _ in 0..100 {
                run();
            }
            start.elapsed().as_nanos() / 100
        })
        .collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut measurements = vec![];
    for (entities, cameras) in [(1, 1), (256, 1), (256, 16), (256, 32)] {
        let owner = PluginId::new("perf.source")?;
        let source = support::Source {
            owner: owner.clone(),
            count: entities,
        };
        let schema = source.schema();
        let mut core = CoreHost::new(17, "p3.perf")?;
        core.install(Box::new(source), Bindings::new())?;
        let selection = support::selection([owner.clone()]);
        let read = core.select(&selection)?;
        let mut host = ViewHost::new("p3.perf")?;
        let view = ViewId::new("perf.view")?;
        host.install(
            Box::new(TopDownFormat),
            ViewConfig {
                id: view.clone(),
                scene: read.scenes[0].reference.clone(),
                binding: RecordBinding {
                    owner,
                    schema,
                    entity_field: "entity".into(),
                    x_field: "x".into(),
                    y_field: "y".into(),
                    z_field: Some("z".into()),
                    units_per_world: 1.0,
                    representation: Representation::Sprite {
                        asset: "perf.sprite".into(),
                        size: [16.0; 2],
                    },
                },
                policy: ViewPolicy::top_down(),
            },
            &read,
        )?;
        for n in 0..cameras {
            let id = CameraId::new(&format!("perf.camera_{n}"))?;
            host.add_camera(
                CameraConfig::new(
                    id.clone(),
                    view.clone(),
                    Viewport {
                        x: 0.0,
                        y: 0.0,
                        width: 320.0,
                        height: 240.0,
                    },
                    target(),
                ),
                &read,
            )?;
            host.activate(&id)?;
        }
        let before = core.save()?;
        let mut tick = 0;
        let update_ns = timings(|| {
            tick += 1;
            std::hint::black_box(
                host.update(
                    &read,
                    ViewInput {
                        target_tick: tick,
                        changes: vec![],
                    },
                )
                .unwrap(),
            );
        });
        assert_eq!(core.save()?, before);
        let selection_ns = timings(|| {
            std::hint::black_box(core.select(&selection).unwrap());
        });
        let core_step_ns = timings(|| {
            let target_tick = core.describe().tick + 1;
            core.step(InputFrame {
                target_tick,
                actions: vec![],
            })
            .unwrap();
        });
        measurements.push(serde_json::json!({"entities":entities,"cameras":cameras,"camera_update_ns_per_call_5_batches":update_ns,"core_selection_ns_per_call_5_batches":selection_ns,"authoritative_step_ns_per_call_5_batches":core_step_ns,"core_unchanged_by_camera_updates":true}));
    }
    let mut core = CoreHost::new(17, "p3.perf.large_selection")?;
    let mut owners = vec![];
    for n in 0..16 {
        let owner = PluginId::new(&format!("perf.source_{n}"))?;
        core.install(
            Box::new(support::Source {
                owner: owner.clone(),
                count: 256,
            }),
            Bindings::new(),
        )?;
        owners.push(owner);
    }
    let selection = support::selection(owners);
    let read = core.select(&selection)?;
    assert_eq!(read.entities.len(), 4096);
    let selection_ns = timings(|| {
        std::hint::black_box(core.select(&selection).unwrap());
    });
    println!(
        "{}",
        serde_json::json!({"profile":"release","target":std::env::consts::ARCH,"iterations_per_batch":100,"batches":5,"measurements":measurements,"large_selection":{"entities":4096,"owners":16,"ns_per_call_5_batches":selection_ns},"claim":"observed workload costs only; no cross-release speedup claim","external_validation":"PENDING"})
    );
    Ok(())
}

//! Camera consumer example: public surfaces only.
#[allow(dead_code)]
mod support;
use ge4g_pentomino::p2::CoreHost;
use ge4g_pentomino::{Bindings, PluginId};
use ge4g_pentomino_view::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let owner = PluginId::new("example.source")?;
    let source = support::Source {
        owner: owner.clone(),
        count: 3,
    };
    let schema = source.schema();
    let mut core = CoreHost::new(17, "p3.public.example")?;
    core.install(Box::new(source), Bindings::new())?;
    let read = core.select(&support::selection([owner.clone()]))?;
    let preserved = core.save()?;
    let view = ViewId::new("example.view")?;
    let camera = CameraId::new("example.camera")?;
    let mut host = ViewHost::new("p3.public.example")?;
    assert_eq!(host.describe().available_formats.len(), 4);
    let config = ViewConfig {
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
                asset: "consumer.sprite".into(),
                size: [16.0, 16.0],
            },
        },
        policy: ViewPolicy::top_down(),
    };
    host.install(Box::new(TopDownFormat), config.clone(), &read)?;
    let identity = Quaternion {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };
    let target = CameraTarget {
        pose: CameraPose {
            position: [0.0, 0.0, 100.0],
            orientation: identity,
            zoom: 1.0,
        },
        view_transform: ViewTransform {
            origin: [0.0; 3],
            rotation: identity,
            scale: [1.0; 3],
            shear_xy: 0.0,
        },
        projection: Projection::Orthographic {
            half_height: 120.0,
            near: 0.1,
            far: 1000.0,
            focus_distance: 100.0,
        },
    };
    host.add_camera(
        CameraConfig::new(
            camera.clone(),
            view.clone(),
            Viewport {
                x: 0.0,
                y: 0.0,
                width: 320.0,
                height: 240.0,
            },
            target.clone(),
        ),
        &read,
    )?;
    host.activate(&camera)?;
    let mut changed = target;
    changed.pose.position[0] = 40.0;
    changed.pose.zoom = 1.25;
    changed.view_transform.scale[1] = 0.7;
    changed.view_transform.shear_xy = 0.1;
    for tick in 1..=6 {
        host.update(
            &read,
            ViewInput {
                target_tick: tick,
                changes: if tick == 1 {
                    vec![CameraChange {
                        camera: camera.clone(),
                        target: changed.clone(),
                        transition: None,
                    }]
                } else {
                    vec![]
                },
            },
        )?;
    }
    let saved = host.save()?;
    let middle = host.frame(&read)?;
    host.update(
        &read,
        ViewInput {
            target_tick: 7,
            changes: vec![],
        },
    )?;
    let continued = host.frame(&read)?;
    let continuation_save = host.save()?;
    host.restore(&saved, &read)?;
    assert_eq!(host.save()?, saved);
    host.update(
        &read,
        ViewInput {
            target_tick: 7,
            changes: vec![],
        },
    )?;
    assert_eq!(host.frame(&read)?, continued);
    assert_eq!(host.save()?, continuation_save);
    let mut replacement = config;
    replacement.policy = ViewPolicy::classic2d();
    host.replace(&view, Box::new(Classic2DFormat), replacement, &read)?;
    host.remove(&view)?;
    assert_eq!(core.save()?, preserved);
    println!(
        "{}",
        serde_json::json!({"example":"p3.public", "midpoint":middle, "core_bytes_preserved":true, "canonical_continuation":true, "external_validation":"PENDING"})
    );
    Ok(())
}

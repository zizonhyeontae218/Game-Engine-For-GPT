//! Existing fifth-demo acceptance consumer; camera transitions never run game menus.
use ge4g_pentomino::p2::{CoreHost, Selection};
use ge4g_pentomino::{Bindings, PluginId};
use ge4g_pentomino_view::*;
use ge4g_pentomino_view_legacy::*;
use ge4g_project::Project;
use ge4g_runtime::World;
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::path::PathBuf;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let fixture = PathBuf::from(
        args.get(1)
            .map(String::as_str)
            .unwrap_or("examples/flatland_nuvema"),
    );
    let output = PathBuf::from(
        args.get(2)
            .map(String::as_str)
            .unwrap_or("/workspace/scratch/p3-fifth-frames"),
    );
    std::fs::create_dir_all(&output)?;
    let world = World::new(Project::load(&fixture)?)?;
    let snapshot = world.snapshot();
    let original = serde_json::to_vec(&snapshot)?;
    let owner = PluginId::new("acceptance.nuvema")?;
    let (plugin, binding) = presentation_plugin(&world, owner.clone())?;
    let mut core = CoreHost::new(17, "p3.fifth.existing")?;
    core.install(plugin, Bindings::new())?;
    let read = core.select(&Selection {
        owners: BTreeSet::from([owner]),
        schemas: BTreeSet::new(),
        include_objects: true,
        include_history: true,
    })?;
    let authoritative = core.save()?;
    assert_eq!(read.entities.len(), snapshot.entities.len());
    let view = ViewId::new("acceptance.view")?;
    let camera = CameraId::new("acceptance.camera")?;
    let config = ViewConfig {
        id: view.clone(),
        scene: read.scenes[0].reference.clone(),
        binding,
        policy: ViewPolicy::top_down(),
    };
    let viewport = Viewport {
        x: 0.0,
        y: 0.0,
        width: f64::from(world.project.manifest.window.width),
        height: f64::from(world.project.manifest.window.height),
    };
    let plan = legacy_camera_target(&snapshot, &viewport, 100.0, 100.0, 0.0)?;
    let depth = legacy_camera_target(&snapshot, &viewport, 115.0, 75.0, 12.0)?;
    let mut balcony = legacy_camera_target(&snapshot, &viewport, 125.0, 70.0, 18.0)?;
    // Pan the native Camera pose as well as changing the existing viewing mode.
    balcony.pose.position[0] = 12.0;
    balcony.pose.position[1] = -8.0;
    let camera_config = CameraConfig::new(camera.clone(), view.clone(), viewport, plan);
    let mut host = ViewHost::new("p3.fifth.existing")?;
    host.install(Box::new(TopDownFormat), config.clone(), &read)?;
    host.add_camera(camera_config.clone(), &read)?;
    host.activate(&camera)?;
    let initial = render_legacy(
        &world.project,
        &snapshot,
        &host.frame(&read)?.cameras[0],
        false,
    )?;
    initial.write_png(&output.join("plan.png"))?;
    for tick in 1..=6 {
        host.update(
            &read,
            ViewInput {
                target_tick: tick,
                changes: if tick == 1 {
                    vec![CameraChange {
                        camera: camera.clone(),
                        target: depth.clone(),
                        transition: None,
                    }]
                } else {
                    vec![]
                },
            },
        )?;
    }
    let midpoint = host.save()?;
    let midpoint_frame = render_legacy(
        &world.project,
        &snapshot,
        &host.frame(&read)?.cameras[0],
        false,
    )?;
    midpoint_frame.write_png(&output.join("smooth-midpoint.png"))?;
    assert_ne!(initial.rgba, midpoint_frame.rgba);
    host.update(
        &read,
        ViewInput {
            target_tick: 7,
            changes: vec![],
        },
    )?;
    let expected = host.frame(&read)?;
    let expected_save = host.save()?;
    let expected_pixels = render_legacy(&world.project, &snapshot, &expected.cameras[0], false)?;
    host.restore(&midpoint, &read)?;
    assert_eq!(host.save()?, midpoint);
    host.update(
        &read,
        ViewInput {
            target_tick: 7,
            changes: vec![],
        },
    )?;
    assert_eq!(host.frame(&read)?, expected);
    assert_eq!(host.save()?, expected_save);
    assert_eq!(
        render_legacy(
            &world.project,
            &snapshot,
            &host.frame(&read)?.cameras[0],
            false
        )?,
        expected_pixels
    );
    for tick in 8..=12 {
        host.update(
            &read,
            ViewInput {
                target_tick: tick,
                changes: vec![],
            },
        )?;
    }
    let end = render_legacy(
        &world.project,
        &snapshot,
        &host.frame(&read)?.cameras[0],
        false,
    )?;
    end.write_png(&output.join("depth.png"))?;
    for tick in 13..=18 {
        host.update(
            &read,
            ViewInput {
                target_tick: tick,
                changes: if tick == 13 {
                    vec![CameraChange {
                        camera: camera.clone(),
                        target: balcony.clone(),
                        transition: None,
                    }]
                } else {
                    vec![]
                },
            },
        )?;
    }
    let retarget = render_legacy(
        &world.project,
        &snapshot,
        &host.frame(&read)?.cameras[0],
        false,
    )?;
    retarget.write_png(&output.join("balcony-transition.png"))?;
    assert_ne!(end.rgba, retarget.rgba);
    let sampled_pose = &host.frame(&read)?.cameras[0].target.pose;
    assert!(sampled_pose.position[0] > 0.0 && sampled_pose.position[0] < 12.0);
    host.remove(&view)?;
    let mut classic = config;
    classic.policy = ViewPolicy::classic2d();
    host.install(Box::new(Classic2DFormat), classic, &read)?;
    host.add_camera(camera_config, &read)?;
    host.activate(&camera)?;
    assert_eq!(
        render_legacy(
            &world.project,
            &snapshot,
            &host.frame(&read)?.cameras[0],
            false
        )?,
        initial
    );
    assert_eq!(core.save()?, authoritative);
    assert_eq!(serde_json::to_vec(&world.snapshot())?, original);
    println!(
        "{}",
        serde_json::json!({"fixture":"existing fifth flatland_nuvema","core_hash":core.hash()?,"scene_identity":read.scenes[0].reference,"entities":read.entities.len(),"whole_core_bytes_preserved":true,"legacy_snapshot_bytes_preserved":true,"default_smooth":true,"midpoint_save_restore_continuation":true,"remove_reinstall":true,"rgba_sha256":{"plan":format!("{:x}",Sha256::digest(&initial.rgba)),"midpoint":format!("{:x}",Sha256::digest(&midpoint_frame.rgba)),"depth":format!("{:x}",Sha256::digest(&end.rgba)),"balcony_transition":format!("{:x}",Sha256::digest(&retarget.rgba))},"output":output,"external_validation":"PENDING"})
    );
    Ok(())
}

//! Independent public-API consumer, also compilable outside this workspace.
use ge4g_pentomino_native::{CameraRequest, ExecutionMode, InputRequest, NativeBridge, ViewFamily};
use std::{error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().collect();
    let config = std::fs::read(
        args.get(1)
            .map(String::as_str)
            .unwrap_or("samples/pentomino_p4/game.json"),
    )?;
    let output = args
        .get(2)
        .map(String::as_str)
        .unwrap_or("/tmp/ge4g-p4-frames");
    std::fs::create_dir_all(output)?;
    let mut bridge = NativeBridge::new();
    let handle = bridge.create(&config, 42, ExecutionMode::Linked)?;
    let start = bridge.frame(handle)?;
    ge4g_render2d::Frame {
        width: start.width,
        height: start.height,
        rgba: start.rgba,
    }
    .write_png(&Path::new(output).join("before.png"))?;
    let attack = |step| InputRequest {
        version: 1,
        target_step: step,
        controls: vec!["attack".into()],
    };
    bridge.step(handle, attack(1))?;
    let save = bridge.save(handle)?;
    let restore = serde_json::to_vec(&save)?;
    bridge.step(handle, attack(2))?;
    let expected = bridge.save(handle)?;
    bridge.restore(handle, &restore)?;
    bridge.step(handle, attack(2))?;
    assert_eq!(bridge.save(handle)?, expected);
    bridge.detach_gameplay(handle)?;
    assert!(bridge.step(handle, attack(3)).is_err());
    bridge.attach_gameplay(handle)?;
    let core = bridge.save(handle)?.core_save2;
    bridge.set_camera(
        handle,
        CameraRequest {
            zoom: 1.25,
            ..Default::default()
        },
    )?;
    bridge.set_view(handle, ViewFamily::TopDown)?;
    assert_eq!(bridge.save(handle)?.core_save2, core);
    for step in 3..=5 {
        bridge.step(handle, attack(step))?;
    }
    let observation = bridge.observe(handle)?;
    assert_eq!(observation.battle.winner, 0);
    let frame = bridge.frame(handle)?;
    assert_ne!(frame.sha256, start.sha256);
    ge4g_render2d::Frame {
        width: frame.width,
        height: frame.height,
        rgba: frame.rgba,
    }
    .write_png(&Path::new(output).join("after.png"))?;
    println!(
        "{}",
        serde_json::json!({"ok":true,"logical_step":observation.logical_step,
        "core_tick":observation.core_tick,"battle":observation.battle,"frame_sha256":frame.sha256,
        "records":observation.records.len(),"renderer":"ge4g-render2d via P3 render_legacy"})
    );
    bridge.close(handle)?;
    assert!(bridge.observe(handle).is_err());
    Ok(())
}

use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn demo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/basement_demo")
}
fn command(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ge4g"))
        .args(args)
        .arg("--json")
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .output()
        .unwrap()
}
fn decoded(output: &Output) -> Value {
    assert!(
        output.stderr.is_empty(),
        "JSON mode wrote diagnostics outside its response: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "invalid JSON output: {e}: {}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}
fn path(path: &Path) -> &str {
    path.to_str().unwrap()
}
fn copy_demo() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    for name in [
        "ge4g.toml",
        "scenes/room_a.json5",
        "scenes/room_b.json5",
        "replays/journey.json",
    ] {
        let dest = dir.path().join(name);
        fs::create_dir_all(dest.parent().unwrap()).unwrap();
        fs::copy(demo().join(name), dest).unwrap();
    }
    dir
}

#[test]
fn headless_journey_is_repeatable_and_exports_real_state_and_events() {
    let dir = tempfile::tempdir().unwrap();
    let trace = dir.path().join("trace.json");
    let snapshot = dir.path().join("snapshot.json");
    let demo = demo();
    let replay = demo.join("replays/journey.json");
    let args = [
        "run",
        path(&demo),
        "--headless",
        "--replay",
        path(&replay),
        "--trace",
        path(&trace),
        "--snapshot-out",
        path(&snapshot),
    ];
    let first = command(&args);
    let second = command(&args);
    assert!(first.status.success());
    assert!(second.status.success());
    let first = decoded(&first);
    let second = decoded(&second);
    assert_eq!(first["snapshot"], second["snapshot"]);
    assert_eq!(
        first["deterministic_sha256"],
        second["deterministic_sha256"]
    );
    assert_eq!(first["snapshot"]["tick"], 160);
    assert_eq!(first["snapshot"]["scene"], "room_b");
    assert_eq!(
        first["snapshot"]["state"],
        json!({"demo.npc.spoken": true, "demo.room_b.entered": true})
    );
    let events = first["snapshot"]["events"].as_array().unwrap();
    for kind in [
        "collision_started",
        "interaction",
        "trigger_entered",
        "scene_transition",
        "state_changed",
    ] {
        assert!(events.iter().any(|e| e["kind"] == kind), "missing {kind}");
    }
    let trace: Value = serde_json::from_slice(&fs::read(trace).unwrap()).unwrap();
    assert_eq!(trace["events"], first["snapshot"]["events"]);
    let inspect = command(&[
        "inspect",
        path(&demo),
        "entity",
        "player",
        "--snapshot",
        path(&snapshot),
    ]);
    assert!(inspect.status.success());
    assert_eq!(
        decoded(&inspect)["entity"]["position"],
        json!({"x": 1440, "y": 2400})
    );
}

#[test]
fn png_pixels_match_the_canonical_frame_and_debug_is_distinct() {
    let dir = tempfile::tempdir().unwrap();
    let png = dir.path().join("frame.png");
    let overlay = dir.path().join("debug.png");
    let capture = command(&["capture", path(&demo()), "--out", path(&png)]);
    assert!(capture.status.success());
    let result = decoded(&capture);
    assert_eq!(result["width"], 320);
    assert_eq!(result["height"], 200);
    assert_eq!(result["tick"], 0);
    let mut reader = png::Decoder::new(fs::File::open(&png).unwrap())
        .read_info()
        .unwrap();
    let mut pixels = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut pixels).unwrap();
    assert_eq!(info.color_type, png::ColorType::Rgba);
    let pixel = |x: usize, y: usize| &pixels[(y * 320 + x) * 4..(y * 320 + x + 1) * 4];
    assert_eq!(pixel(24, 40), [99, 191, 255, 255]);
    assert_eq!(pixel(100, 40), [143, 157, 176, 255]);
    assert_eq!(pixel(200, 120), [70, 200, 151, 255]);
    assert_eq!(result["rgba_sha256"], ge4g_runtime::hash_bytes(&pixels));
    let debug = command(&["capture", path(&demo()), "--debug", "--out", path(&overlay)]);
    assert!(debug.status.success());
    assert_ne!(result["rgba_sha256"], decoded(&debug)["rgba_sha256"]);
    assert_ne!(fs::read(png).unwrap(), fs::read(overlay).unwrap());
}

#[test]
fn save_restart_restores_values_and_corruption_is_explicit() {
    let dir = tempfile::tempdir().unwrap();
    let save = dir.path().join("save.json");
    let replay = demo().join("replays/journey.json");
    let run = command(&[
        "run",
        path(&demo()),
        "--headless",
        "--replay",
        path(&replay),
        "--save",
        path(&save),
    ]);
    assert!(run.status.success());
    let load = command(&[
        "run",
        path(&demo()),
        "--headless",
        "--ticks",
        "0",
        "--load",
        path(&save),
    ]);
    assert!(load.status.success());
    assert_eq!(
        decoded(&load)["snapshot"]["state"],
        decoded(&run)["snapshot"]["state"]
    );
    assert_eq!(decoded(&load)["snapshot"]["scene"], "room_a"); // Saves are declared persistent state, not the whole world.
    assert_eq!(
        fs::read_dir(dir.path()).unwrap().count(),
        1,
        "atomic writer leaked temp files"
    );
    fs::write(&save, "{broken").unwrap();
    let load = command(&["run", path(&demo()), "--headless", "--load", path(&save)]);
    assert!(!load.status.success());
    let error = decoded(&load);
    assert_eq!(error["ok"], false);
    assert!(error["errors"][0].as_str().unwrap().contains("save.json"));
}

#[test]
fn project_test_checks_golden_and_reports_failed_assertions() {
    let output = command(&["test", path(&demo())]);
    assert!(output.status.success());
    let result = decoded(&output);
    assert_eq!(result["results"][0]["assertions_checked"], 7);
    assert_eq!(result["results"][0]["golden_checked"], true);
    let copied = copy_demo();
    let manifest = copied.path().join("ge4g.toml");
    fs::write(
        &manifest,
        fs::read_to_string(&manifest)
            .unwrap()
            .replace("position = [64, 40]", "position = [65, 40]"),
    )
    .unwrap();
    let failure = command(&["test", path(copied.path())]);
    assert_eq!(failure.status.code(), Some(1));
    assert!(
        decoded(&failure)["errors"][0]
            .as_str()
            .unwrap()
            .contains("tick 20")
    );
}

#[test]
fn invalid_scene_version_and_missing_references_fail_with_context() {
    let copied = copy_demo();
    let scene = copied.path().join("scenes/room_a.json5");
    let original = fs::read_to_string(&scene).unwrap();
    fs::write(
        &scene,
        original.replace("schema_version: 1", "schema_version: 99"),
    )
    .unwrap();
    let validation = command(&["validate", path(copied.path())]);
    assert!(!validation.status.success());
    let result = decoded(&validation);
    let message = result["errors"][0].as_str().unwrap();
    assert!(message.contains("room_a.json5"));
    assert!(message.contains("99"));
    fs::write(
        &scene,
        original.replace("scene: 'room_b'", "scene: 'missing_scene'"),
    )
    .unwrap();
    let validation = command(&["validate", path(copied.path())]);
    assert!(!validation.status.success());
    assert!(
        decoded(&validation)["errors"][0]
            .as_str()
            .unwrap()
            .contains("entity door")
    );
}

#[test]
fn errors_never_claim_unwritten_capture_and_json_argument_errors_are_structured() {
    let dir = tempfile::tempdir().unwrap();
    let absent = dir.path().join("absent/frame.png");
    let capture = command(&["capture", path(&demo()), "--out", path(&absent)]);
    assert!(!capture.status.success());
    assert_eq!(decoded(&capture)["ok"], false);
    assert!(!absent.exists());
    let invalid = command(&["run", path(&demo()), "--ticks", "not-a-number"]);
    assert_eq!(invalid.status.code(), Some(2));
    assert_eq!(decoded(&invalid)["schema_version"], 1);
}

#[test]
fn trace_filters_preserve_order_and_schema_commands_are_machine_readable() {
    let dir = tempfile::tempdir().unwrap();
    let trace = dir.path().join("trace.json");
    let replay = demo().join("replays/journey.json");
    let run = command(&[
        "run",
        path(&demo()),
        "--headless",
        "--replay",
        path(&replay),
        "--trace",
        path(&trace),
        "--trace-events",
        "interaction,scene_transition",
    ]);
    assert!(run.status.success());
    let trace: Value = serde_json::from_slice(&fs::read(trace).unwrap()).unwrap();
    let events = trace["events"].as_array().unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["kind"], "interaction");
    assert_eq!(events[1]["kind"], "scene_transition");
    for kind in ["project", "scene", "replay", "trace", "save", "snapshot"] {
        let output = command(&["schema", kind]);
        assert!(output.status.success());
        let schema = decoded(&output);
        assert!(schema["$schema"].as_str().unwrap().contains("json-schema"));
        assert!(schema["properties"]["schema_version"].is_object());
        if ["save", "trace"].contains(&kind) {
            assert_eq!(schema["properties"]["schema_version"]["const"], 1);
        } else {
            assert_eq!(
                schema["properties"]["schema_version"]["enum"],
                json!([1, 2])
            );
        }
        assert_eq!(schema["additionalProperties"], false);
    }
    let inspect = command(&["inspect", path(&demo()), "entity", "does_not_exist"]);
    assert!(!inspect.status.success());
    assert!(
        decoded(&inspect)["errors"][0]
            .as_str()
            .unwrap()
            .contains("does_not_exist")
    );
    let diagnosis = command(&["diagnose", path(&demo())]);
    assert!(diagnosis.status.success());
    assert_eq!(decoded(&diagnosis)["checks"].as_array().unwrap().len(), 4);
}

#[test]
fn textured_sprites_and_camera_use_the_same_cpu_capture_path() {
    let copied = copy_demo();
    let assets = copied.path().join("assets");
    fs::create_dir(&assets).unwrap();
    let texture_path = assets.join("tile.png");
    {
        let mut encoder = png::Encoder::new(fs::File::create(&texture_path).unwrap(), 2, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&[255, 0, 0, 255, 0, 255, 0, 255])
            .unwrap();
    }
    let scene = copied.path().join("scenes/room_a.json5");
    let original = fs::read_to_string(&scene).unwrap();
    fs::write(
        &scene,
        original
            .replace("camera: [0, 0]", "camera: [8, 8]")
            .replace(
                "sprite: { color: [99, 191, 255, 255], layer: 2 }",
                "sprite: { texture: 'assets/tile.png', layer: 2 }",
            ),
    )
    .unwrap();
    let out = copied.path().join("frame.png");
    let capture = command(&["capture", path(copied.path()), "--out", path(&out)]);
    assert!(capture.status.success());
    let mut reader = png::Decoder::new(fs::File::open(&out).unwrap())
        .read_info()
        .unwrap();
    let mut rgba = vec![0; reader.output_buffer_size()];
    reader.next_frame(&mut rgba).unwrap();
    let red = (32 * 320 + 16) * 4;
    let green = (32 * 320 + 31) * 4;
    assert_eq!(&rgba[red..red + 4], [255, 0, 0, 255]);
    assert_eq!(&rgba[green..green + 4], [0, 255, 0, 255]);
    fs::remove_file(texture_path).unwrap();
    let validation = command(&["validate", path(copied.path())]);
    assert!(!validation.status.success());
    assert!(
        decoded(&validation)["errors"][0]
            .as_str()
            .unwrap()
            .contains("tile.png")
    );
}

#[test]
fn a_manifest_filename_can_be_used_from_the_project_directory() {
    let output = Command::new(env!("CARGO_BIN_EXE_ge4g"))
        .current_dir(demo())
        .args(["validate", "ge4g.toml", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(decoded(&output)["ok"], true);
}

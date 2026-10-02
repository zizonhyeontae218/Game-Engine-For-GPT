use clap::{Parser, Subcommand, ValueEnum};
use ge4g_core::{ENGINE_VERSION, Error, Input, Result, SCHEMA_VERSION, Snapshot};
use ge4g_platform::PlayOptions;
use ge4g_project::{Manifest, Project, Replay, Scene, atomic_json, version};
use ge4g_render2d::render;
use ge4g_runtime::{Save, World, assert_snapshot, hash_bytes, snapshot_hash};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser)]
#[command(
    name = "ge4g",
    version,
    about = "GE4G / GameEngineForGPT — deterministic, inspectable 2D engine"
)]
struct Cli {
    #[arg(long, global = true, help = "Emit one versioned JSON object on stdout")]
    json: bool,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Validate all project files and references without opening a window.
    Validate { project: PathBuf },
    /// Inspect a project, scene, entity or state; optionally read a runtime snapshot.
    Inspect {
        project: PathBuf,
        #[arg(value_enum, default_value = "project")]
        target: InspectTarget,
        name: Option<String>,
        #[arg(long)]
        scene: Option<String>,
        #[arg(long)]
        snapshot: Option<PathBuf>,
        #[arg(long, conflicts_with = "snapshot")]
        load: Option<PathBuf>,
    },
    /// Play in a local window, or execute exact simulation ticks headlessly.
    Run {
        project: PathBuf,
        #[arg(long)]
        headless: bool,
        #[arg(long)]
        ticks: Option<u64>,
        #[arg(long)]
        replay: Option<PathBuf>,
        #[arg(long)]
        load: Option<PathBuf>,
        #[arg(long)]
        save: Option<PathBuf>,
        #[arg(long)]
        trace: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', requires = "trace")]
        trace_events: Vec<String>,
        #[arg(long)]
        snapshot_out: Option<PathBuf>,
        #[arg(long)]
        debug: bool,
    },
    /// Write the canonical CPU framebuffer as a real RGBA PNG.
    Capture {
        project: PathBuf,
        #[arg(long, default_value_t = 0)]
        tick: u64,
        #[arg(long)]
        replay: Option<PathBuf>,
        #[arg(long)]
        load: Option<PathBuf>,
        #[arg(long)]
        debug: bool,
        #[arg(long)]
        out: PathBuf,
    },
    /// Run project replay assertions, repeatability, golden and save/capture checks.
    Test { project: PathBuf },
    /// Explain project, simulation, framebuffer and persistent state health.
    Diagnose { project: PathBuf },
    /// Print a JSON Schema for an authoring or observation format.
    Schema {
        #[arg(value_enum)]
        kind: SchemaKind,
    },
}
#[derive(Clone, Copy, ValueEnum)]
enum InspectTarget {
    Project,
    Scene,
    Entity,
    State,
}
#[derive(Clone, Copy, ValueEnum)]
enum SchemaKind {
    Project,
    Scene,
    Replay,
    Trace,
    Save,
    Snapshot,
}
#[derive(Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct Trace {
    schema_version: u32,
    engine_version: String,
    events: Vec<ge4g_core::Event>,
    events_dropped: u64,
}
const EVENT_KINDS: &[&str] = &[
    "scene_loaded",
    "entity_spawned",
    "collision_started",
    "collision_resolved",
    "collision_ended",
    "trigger_entered",
    "trigger_exited",
    "interaction",
    "state_changed",
    "scene_transition",
    "save_loaded",
    "save_written",
    "audio",
];

fn main() -> ExitCode {
    let json_requested = std::env::args().any(|s| s == "--json");
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            if matches!(
                error.kind(),
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
            ) {
                print!("{error}");
                return ExitCode::SUCCESS;
            }
            if json_requested {
                println!(
                    "{}",
                    json!({"schema_version": 1, "ok": false, "errors": [error.to_string()]})
                );
            } else {
                eprint!("{error}");
            }
            return ExitCode::from(2);
        }
    };
    match dispatch(cli.command) {
        Ok(value) => {
            let ok = value.get("ok").and_then(Value::as_bool).unwrap_or(true);
            if cli.json {
                println!(
                    "{}",
                    serde_json::to_string(&value).expect("JSON value is serializable")
                );
            } else if let Some(summary) = value.get("summary").and_then(Value::as_str) {
                println!("{summary}");
                if !ok {
                    for e in value["errors"].as_array().into_iter().flatten() {
                        eprintln!("{}", e.as_str().unwrap_or("unknown failure"));
                    }
                }
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&value).expect("JSON value is serializable")
                );
            }
            if ok {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(error) => {
            if cli.json {
                println!(
                    "{}",
                    json!({"schema_version": SCHEMA_VERSION, "ok": false, "errors": [error.to_string()]})
                );
            } else {
                eprintln!("GE4G: {error}");
            }
            ExitCode::from(2)
        }
    }
}
fn load_replay(path: Option<&Path>) -> Result<Option<Replay>> {
    path.map(Replay::load).transpose()
}
fn execute(world: &mut World, replay: Option<&Replay>, ticks: u64) -> Result<()> {
    if ticks > 1_000_000 {
        return Err(Error("--ticks/--tick exceeds 1,000,000".into()));
    }
    for _ in 0..ticks {
        let input = replay.map_or_else(Input::default, |r| r.input_at(world.tick));
        world.step(&input)?;
    }
    Ok(())
}
fn dispatch(command: Command) -> Result<Value> {
    match command {
        Command::Validate { project } => {
            let report = Project::validate(&project);
            let summary = format!(
                "GE4G validate: {} ({} files)",
                if report.ok { "passed" } else { "failed" },
                report.files_checked.len()
            );
            let mut value = serde_json::to_value(report).map_err(|e| Error(e.to_string()))?;
            value["summary"] = json!(summary);
            Ok(value)
        }
        Command::Inspect {
            project,
            target,
            name,
            scene,
            snapshot,
            load,
        } => inspect(
            &project,
            target,
            name.as_deref(),
            scene.as_deref(),
            snapshot.as_deref(),
            load.as_deref(),
        ),
        Command::Run {
            project,
            headless,
            ticks,
            replay,
            load,
            save,
            trace,
            trace_events,
            snapshot_out,
            debug,
        } => {
            for kind in &trace_events {
                if !EVENT_KINDS.contains(&kind.as_str()) {
                    return Err(Error(format!(
                        "unknown --trace-events kind {kind}; supported: {}",
                        EVENT_KINDS.join(",")
                    )));
                }
            }
            if ticks.is_some_and(|n| n > 1_000_000) {
                return Err(Error("--ticks exceeds 1,000,000".into()));
            }
            let replay = load_replay(replay.as_deref())?;
            let mut world = World::with_save(Project::load(&project)?, load.as_deref())?;
            if headless {
                execute(
                    &mut world,
                    replay.as_ref(),
                    ticks.unwrap_or_else(|| replay.as_ref().map_or(120, |r| r.ticks)),
                )?;
            } else {
                let default_save = world.project.root.join("save.json");
                ge4g_platform::play(
                    &mut world,
                    PlayOptions {
                        ticks,
                        replay: replay.as_ref(),
                        save_path: save.as_deref().unwrap_or(&default_save),
                        debug,
                    },
                )?;
            }
            if let Some(path) = &save {
                world.save(path)?;
            }
            let snapshot = world.snapshot();
            let hash = snapshot_hash(&snapshot)?;
            if let Some(path) = &snapshot_out {
                atomic_json(path, &snapshot)?;
            }
            if let Some(path) = &trace {
                let events = snapshot
                    .events
                    .iter()
                    .filter(|e| trace_events.is_empty() || trace_events.contains(&e.kind))
                    .cloned()
                    .collect();
                atomic_json(
                    path,
                    &Trace {
                        schema_version: 1,
                        engine_version: ENGINE_VERSION.into(),
                        events,
                        events_dropped: snapshot.events_dropped,
                    },
                )?;
            }
            Ok(
                json!({"schema_version": 1, "ok": true, "summary": format!("GE4G run: tick {} · {} · {hash}", snapshot.tick, snapshot.scene), "snapshot": snapshot, "deterministic_sha256": hash, "trace": trace, "save": save, "snapshot_out": snapshot_out}),
            )
        }
        Command::Capture {
            project,
            tick,
            replay,
            load,
            debug,
            out,
        } => {
            let replay = load_replay(replay.as_deref())?;
            let mut world = World::with_save(Project::load(&project)?, load.as_deref())?;
            execute(&mut world, replay.as_ref(), tick)?;
            let frame = render(&world.project, &world.snapshot(), debug)?;
            frame.write_png(&out)?;
            Ok(
                json!({"schema_version": 1, "ok": true, "summary": format!("GE4G capture: {} ({}×{}, tick {}, {})", out.display(), frame.width, frame.height, tick, world.scene), "path": out, "width": frame.width, "height": frame.height, "tick": tick, "scene": world.scene, "debug": debug, "rgba_sha256": hash_bytes(&frame.rgba)}),
            )
        }
        Command::Test { project } => project_tests(Project::load(&project)?),
        Command::Diagnose { project } => diagnose(&project),
        Command::Schema { kind } => {
            let schema = match kind {
                SchemaKind::Project => schemars::schema_for!(Manifest),
                SchemaKind::Scene => schemars::schema_for!(Scene),
                SchemaKind::Replay => schemars::schema_for!(Replay),
                SchemaKind::Save => schemars::schema_for!(Save),
                SchemaKind::Snapshot => schemars::schema_for!(Snapshot),
                SchemaKind::Trace => schemars::schema_for!(Trace),
            };
            let mut schema =
                serde_json::to_value(schema).map_err(|e| Error(format!("schema: {e}")))?;
            schema["properties"]["schema_version"]["const"] = json!(1);
            if matches!(kind, SchemaKind::Snapshot) {
                schema["properties"]["tick_hz"]["const"] = json!(60);
                schema["properties"]["subpixels_per_pixel"]["const"] = json!(60);
            }
            Ok(schema)
        }
    }
}
fn inspect(
    path: &Path,
    target: InspectTarget,
    name: Option<&str>,
    selected_scene: Option<&str>,
    snapshot_path: Option<&Path>,
    load: Option<&Path>,
) -> Result<Value> {
    let mut project = Project::load(path)?;
    if let Some(scene) = selected_scene {
        if !project.scenes.contains_key(scene) {
            return Err(Error(format!("inspect: unknown scene {scene}")));
        }
        project.manifest.start_scene = scene.into();
    }
    if matches!(target, InspectTarget::Project) {
        return Ok(
            json!({"schema_version": 1, "ok": true, "name": project.manifest.name, "engine_version": ENGINE_VERSION, "start_scene": project.manifest.start_scene, "window": project.manifest.window, "scenes": project.manifest.scenes, "state_definitions": project.manifest.state, "tests": project.manifest.tests, "files_checked": project.files_checked}),
        );
    }
    let snapshot = if let Some(path) = snapshot_path {
        let snapshot: Snapshot = serde_json::from_str(&ge4g_project::read_text(path)?)
            .map_err(|e| Error(format!("snapshot {}: {e}", path.display())))?;
        version(snapshot.schema_version, "snapshot")?;
        if !project.scenes.contains_key(&snapshot.scene)
            || snapshot.tick_hz != 60
            || snapshot.subpixels_per_pixel != 60
        {
            return Err(Error(
                "snapshot scene or time/coordinate units do not match this engine/project".into(),
            ));
        }
        let mut state = ge4g_core::StateStore::new(project.manifest.state.clone())?;
        if state.values.keys().collect::<Vec<_>>() != snapshot.state.keys().collect::<Vec<_>>() {
            return Err(Error("snapshot state key set differs from project".into()));
        }
        for (k, v) in &snapshot.state {
            state.set(k, v.clone())?;
        }
        snapshot
    } else {
        World::with_save(project.clone(), load)?.snapshot()
    };
    match target {
        InspectTarget::Entity => {
            let id = name.ok_or_else(|| Error("inspect entity requires an entity id".into()))?;
            let entity = snapshot
                .entities
                .iter()
                .find(|e| e.id == id)
                .ok_or_else(|| Error(format!("scene {}: entity {id} not found", snapshot.scene)))?;
            Ok(
                json!({"schema_version": 1, "ok": true, "scene": snapshot.scene, "tick": snapshot.tick, "subpixels_per_pixel": snapshot.subpixels_per_pixel, "entity": entity}),
            )
        }
        InspectTarget::State => Ok(
            json!({"schema_version": 1, "ok": true, "scene": snapshot.scene, "tick": snapshot.tick, "definitions": project.manifest.state, "state": snapshot.state}),
        ),
        InspectTarget::Scene => {
            if name.is_some_and(|n| n != snapshot.scene) {
                return Err(Error("select another scene with --scene <id>".into()));
            }
            Ok(json!({"schema_version": 1, "ok": true, "snapshot": snapshot}))
        }
        InspectTarget::Project => unreachable!(),
    }
}
fn project_tests(project: Project) -> Result<Value> {
    if project.manifest.tests.is_empty() {
        return Err(Error(
            "project defines no replay tests; add [[tests]] with behavioral assertions".into(),
        ));
    }
    let mut results = Vec::new();
    let mut errors = Vec::new();
    for test in &project.manifest.tests {
        let result = (|| -> Result<Value> {
            let replay = Replay::load(&project.path(&test.replay)?)?;
            let mut first = World::new(project.clone())?;
            let mut checked = 0;
            for tick in 0..=replay.ticks {
                let snapshot = first.snapshot();
                for assertion in test.assertions.iter().filter(|a| a.tick == tick) {
                    assert_snapshot(&snapshot, assertion)?;
                    checked += 1;
                }
                if tick < replay.ticks {
                    first.step(&replay.input_at(tick))?;
                }
            }
            if first.events_dropped > 0 {
                return Err(Error(
                    "test trace overflowed; shorten replay or checkpoint events earlier".into(),
                ));
            }
            let snapshot = first.snapshot();
            let mut second = World::new(project.clone())?;
            execute(&mut second, Some(&replay), replay.ticks)?;
            if snapshot != second.snapshot() {
                return Err(Error(
                    "repeating replay diverged in state/entities/ordered events".into(),
                ));
            }
            let frame = render(&project, &snapshot, false)?;
            let frame_hash = hash_bytes(&frame.rgba);
            if let Some(expected) = &test.golden_rgba_sha256
                && expected != &frame_hash
            {
                return Err(Error(format!(
                    "frame golden mismatch: expected {expected}, got {frame_hash}"
                )));
            }
            let dir = tempfile::tempdir().map_err(|e| Error(format!("test tempdir: {e}")))?;
            let save = dir.path().join("save.json");
            first.save(&save)?;
            let restored = World::with_save(project.clone(), Some(&save))?;
            // Reload before start-scene on_enter may have intentional effects; the demo
            // starts in a scene without reset writes, so persistence remains observable.
            if first.state.persistent_values() != restored.state.persistent_values() {
                return Err(Error("persistent values differ after restart/load".into()));
            }
            let png = dir.path().join("frame.png");
            frame.write_png(&png)?;
            if std::fs::metadata(&png)
                .map_err(|e| Error(e.to_string()))?
                .len()
                <= 8
            {
                return Err(Error("PNG capture is empty".into()));
            }
            Ok(
                json!({"name": test.name, "ok": true, "assertions_checked": checked, "ticks": replay.ticks, "deterministic": true, "deterministic_sha256": snapshot_hash(&snapshot)?, "frame_rgba_sha256": frame_hash, "golden_checked": test.golden_rgba_sha256.is_some(), "save_reload": true, "png_capture": true}),
            )
        })();
        match result {
            Ok(result) => results.push(result),
            Err(e) => {
                let message = format!("test {}: {e}", test.name);
                errors.push(message.clone());
                results.push(json!({"name": test.name, "ok": false, "error": message}));
            }
        }
    }
    Ok(
        json!({"schema_version": 1, "ok": errors.is_empty(), "summary": format!("GE4G test: {} of {} passed", results.iter().filter(|r| r["ok"] == true).count(), results.len()), "results": results, "errors": errors}),
    )
}
fn diagnose(path: &Path) -> Result<Value> {
    let validation = Project::validate(path);
    if !validation.ok {
        return Ok(
            json!({"schema_version": 1, "ok": false, "summary": "GE4G diagnose: project validation failed", "checks": [{"id": "project", "ok": false, "details": validation.errors}], "errors": validation.errors}),
        );
    }
    let project = Project::load(path)?;
    let mut first = World::new(project.clone())?;
    let mut second = World::new(project.clone())?;
    execute(&mut first, None, 10)?;
    execute(&mut second, None, 10)?;
    let deterministic = first.snapshot() == second.snapshot();
    let frame = render(&project, &first.snapshot(), true)?;
    Ok(
        json!({"schema_version": 1, "ok": deterministic, "summary": "GE4G diagnose: project, fixed-step runtime, CPU framebuffer and typed state checked", "checks": [
        {"id": "project", "ok": true, "files_checked": validation.files_checked},
        {"id": "fixed_step_repeatability", "ok": deterministic, "ticks": 10},
        {"id": "cpu_framebuffer", "ok": true, "width": frame.width, "height": frame.height, "rgba_sha256": hash_bytes(&frame.rgba)},
        {"id": "state_schema", "ok": true, "keys": first.state.values.len()},
    ], "errors": if deterministic { vec![] } else { vec!["idle simulation diverged"] }}),
    )
}

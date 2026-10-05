//! Local input/window adapter. Gameplay always calls World::step.
#[cfg(feature = "window")]
mod audio;
use ge4g_core::{Error, Result};
use ge4g_project::Replay;
use ge4g_runtime::World;
use std::path::Path;

pub struct PlayOptions<'a> {
    pub ticks: Option<u64>,
    pub replay: Option<&'a Replay>,
    pub save_path: &'a Path,
    pub debug: bool,
}

#[cfg(not(feature = "window"))]
pub fn play(_world: &mut World, _options: PlayOptions<'_>) -> Result<()> {
    Err(Error(
        "interactive adapter was excluded; rebuild with default features or use --headless".into(),
    ))
}

#[cfg(feature = "window")]
pub fn play(world: &mut World, options: PlayOptions<'_>) -> Result<()> {
    use ge4g_core::{Input, TICK_HZ};
    use ge4g_render2d::render;
    use minifb::{Key, KeyRepeat, Scale, Window, WindowOptions};
    use std::time::{Duration, Instant};
    let width = world.project.manifest.window.width as usize;
    let height = world.project.manifest.window.height as usize;
    let mut window = Window::new(
        "GE4G / GameEngineForGPT — Basement",
        width,
        height,
        WindowOptions {
            scale: Scale::X2,
            resize: true,
            ..WindowOptions::default()
        },
    )
    .map_err(|e| {
        Error(format!(
            "create game window: {e}; use --headless on a machine without a display"
        ))
    })?;
    window.set_target_fps(120);
    let step = Duration::from_nanos(1_000_000_000 / u64::from(TICK_HZ));
    let mut previous = Instant::now();
    let mut accumulated = Duration::ZERO;
    let mut paused = false;
    let mut debug = options.debug;
    let start_tick = world.tick;
    let mut audio = audio::Audio::new();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        if options
            .ticks
            .is_some_and(|limit| world.tick - start_tick >= limit)
        {
            break;
        }
        let now = Instant::now();
        accumulated += now.duration_since(previous).min(Duration::from_millis(250));
        previous = now;
        if window.is_key_pressed(Key::P, KeyRepeat::No) {
            paused = !paused;
            accumulated = Duration::ZERO;
        }
        if window.is_key_pressed(Key::F3, KeyRepeat::No) {
            debug = !debug;
        }
        if window.is_key_pressed(Key::F5, KeyRepeat::No) {
            world.save(options.save_path)?;
        }
        if window.is_key_pressed(Key::F9, KeyRepeat::No) {
            if options.ticks.is_some() {
                return Err(Error(
                    "F9 reload is unavailable during a finite verification run".into(),
                ));
            }
            *world = World::with_save(world.project.clone(), Some(options.save_path))?;
            accumulated = Duration::ZERO;
        }
        let live = Input {
            left: window.is_key_down(Key::A) || window.is_key_down(Key::Left),
            right: window.is_key_down(Key::D) || window.is_key_down(Key::Right),
            up: window.is_key_down(Key::W) || window.is_key_down(Key::Up),
            down: window.is_key_down(Key::S) || window.is_key_down(Key::Down),
            interact: window.is_key_down(Key::E) || window.is_key_down(Key::Space),
        };
        let active = options.replay.is_some() || window.is_active();
        if !active {
            world.release_inputs();
            accumulated = Duration::ZERO;
        }
        let buttons: std::collections::BTreeSet<String> =
            [(Key::Z, "melee"), (Key::X, "shoot"), (Key::C, "heal")]
                .into_iter()
                .filter(|(key, _)| active && window.is_key_down(*key))
                .map(|(_, id)| id.to_owned())
                .collect();
        if options.replay.is_none()
            && let Some(waiting) = world.waiting()
        {
            let choices = waiting["options"].as_array().cloned().unwrap_or_default();
            for (i, key) in [
                Key::Key1,
                Key::Key2,
                Key::Key3,
                Key::Key4,
                Key::Key5,
                Key::Key6,
                Key::Key7,
                Key::Key8,
                Key::Key9,
            ]
            .into_iter()
            .enumerate()
            {
                if window.is_key_pressed(key, KeyRepeat::No)
                    && let Some(id) = choices.get(i).and_then(|c| c["id"].as_str())
                {
                    world.choose(id)?;
                    break;
                }
            }
            if choices.len() == 1 && window.is_key_pressed(Key::Enter, KeyRepeat::No) {
                world.choose(choices[0]["id"].as_str().unwrap())?;
            }
        }
        if paused || !active {
            accumulated = Duration::ZERO;
            if window.is_key_pressed(Key::N, KeyRepeat::No) {
                if let Some(replay) = options.replay {
                    world.step_replay(replay)?;
                } else {
                    world.step_actions(&live, &buttons)?;
                }
            }
        } else {
            while accumulated >= step
                && !options
                    .ticks
                    .is_some_and(|limit| world.tick - start_tick >= limit)
            {
                if let Some(replay) = options.replay {
                    world.step_replay(replay)?;
                } else {
                    world.step_actions(&live, &buttons)?;
                }
                accumulated -= step;
            }
        }
        audio.sync(world, paused || !active);
        let snapshot = world.snapshot();
        let waiting = world.waiting().map(|w| {
            format!(
                "{} | {}",
                w["text"].as_str().unwrap_or(""),
                w["options"]
                    .as_array()
                    .unwrap_or(&vec![])
                    .iter()
                    .enumerate()
                    .map(|(i, c)| format!("{}:{}", i + 1, c["text"].as_str().unwrap_or("")))
                    .collect::<Vec<_>>()
                    .join(" | ")
            )
        });
        let dialogue = snapshot
            .events
            .iter()
            .rev()
            .find(|e| e.kind == "interaction")
            .and_then(|e| e.data["dialogue"].as_str())
            .unwrap_or("WASD move | E talk | F3 debug | F5 save | F9 load | P pause | N step");
        let dialogue = waiting.as_deref().unwrap_or(dialogue);
        window.set_title(&format!(
            "GE4G | {} | NPC:{} RoomB:{} {} | {dialogue}",
            world.scene,
            world
                .state
                .values
                .get("demo.npc.spoken")
                .map_or("-".into(), ToString::to_string),
            world
                .state
                .values
                .get("demo.room_b.entered")
                .map_or("-".into(), ToString::to_string),
            if paused { "PAUSED" } else { "" }
        ));
        let frame = render(&world.project, &snapshot, debug)?;
        window
            .update_with_buffer(&frame.window_pixels(), width, height)
            .map_err(|e| Error(format!("present framebuffer at tick {}: {e}", world.tick)))?;
    }
    Ok(())
}

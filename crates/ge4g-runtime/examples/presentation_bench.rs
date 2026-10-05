//! Local CPU presentation measurement; not an FPS/device acceptance claim.
use ge4g_project::{Project, flatland::Action};
use ge4g_render2d::render;
use ge4g_runtime::World;
use serde_json::json;
use std::{path::Path, time::Instant};
fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/flatland_nuvema");
    let mut w = World::new(Project::load(&root).unwrap()).unwrap();
    w.choose("continue").unwrap();
    let mut result = Vec::new();
    for mode in ["top", "depth", "battle"] {
        if mode == "depth" {
            w.command(&[Action::View {
                mode: Some("depth".into()),
            }])
            .unwrap();
        }
        if mode == "battle" {
            w.command(&[Action::EventScene {
                event: "battle_ember".into(),
            }])
            .unwrap();
        }
        let full = w.snapshot();
        let minimal = w.render_snapshot();
        assert_eq!(
            render(&w.project, &full, false).unwrap(),
            render(&w.project, &minimal, false).unwrap()
        );
        let mut row = json!({"mode":mode,"full_input_bytes":serde_json::to_vec(&full).unwrap().len(),"render_input_bytes":serde_json::to_vec(&minimal).unwrap().len()});
        for (key, trimmed) in [
            ("full_snapshot_render_ms", false),
            ("minimal_snapshot_render_ms", true),
        ] {
            let mut samples = Vec::new();
            for _ in 0..20 {
                let start = Instant::now();
                for _ in 0..50 {
                    let s = if trimmed {
                        w.render_snapshot()
                    } else {
                        w.snapshot()
                    };
                    std::hint::black_box(render(&w.project, &s, false).unwrap());
                }
                samples.push(start.elapsed().as_secs_f64() * 1000.0 / 50.0);
            }
            samples.sort_by(f64::total_cmp);
            row[key] = json!({"median":samples[10],"p95":samples[19]});
        }
        result.push(row);
    }
    println!(
        "{}",
        json!({"iterations_per_mode_path":1000,"measurement":"local release CPU snapshot+render; no simulation/device timing","results":result})
    );
}

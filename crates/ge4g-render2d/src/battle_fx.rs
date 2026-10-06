//! Reusable cosmetic vocabulary, driven solely by typed resolved battle events.
use super::Frame;
use serde_json::Value;
pub(super) fn offset(fx: &Value, id: &str, tick: u64) -> [i64; 2] {
    let start = fx["start_tick"].as_u64().unwrap_or(tick);
    if tick < start {
        return [0, 0];
    }
    let age = tick - start;
    if matches!(fx["preset"].as_str(), Some("heal" | "guard")) {
        return [0, 0];
    }
    if fx["actor"] == id && age < 22 {
        let travel = if age < 8 {
            -(age as i64) / 2
        } else {
            (22 - age) as i64
        };
        return [travel, 0];
    }
    if fx["target"] == id && (22..34).contains(&age) {
        return [if age % 4 < 2 { -4 } else { 4 }, 0];
    }
    [0, 0]
}
pub(super) fn flash(fx: &Value, id: &str, tick: u64) -> bool {
    let start = fx["start_tick"].as_u64().unwrap_or(tick);
    tick >= start
        && (22..34).contains(&(tick - start))
        && fx["target"] == id
        && fx["before_hp"].as_i64() > fx["after_hp"].as_i64()
        && (tick - start) % 6 < 3
}
pub(super) fn draw(frame: &mut Frame, fx: &Value, tick: u64, actor: [i64; 2], target: [i64; 2]) {
    let start = fx["start_tick"].as_u64().unwrap_or(tick);
    if tick < start {
        return;
    }
    let age = tick - start;
    let preset = fx["preset"].as_str().unwrap_or("strike");
    if !(8..42).contains(&age) {
        return;
    }
    let [x, y] = target;
    match preset {
        "projectile" => {
            let phase = (age.saturating_sub(8)).min(16) as i64;
            let at = [
                actor[0] + (x - actor[0]) * phase / 16,
                actor[1] + (y - actor[1]) * phase / 16,
            ];
            for dy in -5..=5 {
                for dx in -7..=7 {
                    if dx * dx + dy * dy < 40 {
                        frame.blend(at[0] + dx, at[1] + dy, [255, 208, 64, 255]);
                    }
                }
            }
        }
        "heal" | "guard" => {
            let color = if preset == "heal" {
                [91, 255, 146, 230]
            } else {
                [107, 195, 255, 220]
            };
            let size = 12 + (age % 12) as i64;
            for i in -size..=size {
                frame.blend(x + i, y - size, color);
                frame.blend(x + i, y + size, color);
                frame.blend(x - size, y + i, color);
                frame.blend(x + size, y + i, color);
            }
            if preset == "heal" {
                for i in -10..=10 {
                    for t in -2..=2 {
                        frame.blend(x + i, y + t, color);
                        frame.blend(x + t, y + i, color);
                    }
                }
            }
        }
        "slash" => {
            for line in -1..=1 {
                for i in -22..=22 {
                    for t in -2..=2 {
                        frame.blend(x + i + line * 9, y - i + t, [255, 240, 216, 255]);
                    }
                }
            }
        }
        "burst" => {
            let radius = 8 + (age - 8) as i64;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let d = dx * dx + dy * dy;
                    if d < radius * radius && d > (radius - 4).pow(2) {
                        frame.blend(x + dx, y + dy, [255, 141, 65, 240]);
                    }
                }
            }
        }
        _ => {
            if age >= 18 {
                for i in -18i64..=18 {
                    frame.blend(x + i, y, [255, 222, 75, 255]);
                    frame.blend(x, y + i, [255, 222, 75, 255]);
                    frame.blend(x + i, y + i, [255, 222, 75, 255]);
                    frame.blend(x + i, y - i, [255, 222, 75, 255]);
                }
            }
        }
    }
}

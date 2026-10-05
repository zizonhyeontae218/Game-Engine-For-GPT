//! Canonical deterministic CPU renderer. Window and PNG share these bytes.
use ge4g_core::{Error, Result, SUBPIXELS, Snapshot};
use ge4g_project::{Project, atomic_bytes};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
impl Frame {
    pub fn write_png(&self, path: &Path) -> Result<()> {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, self.width, self.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder
                .write_header()
                .map_err(|e| Error(format!("PNG {}: {e}", path.display())))?;
            writer
                .write_image_data(&self.rgba)
                .map_err(|e| Error(format!("PNG {}: {e}", path.display())))?;
            writer
                .finish()
                .map_err(|e| Error(format!("PNG {}: {e}", path.display())))?;
        }
        atomic_bytes(path, &bytes)
    }
    fn blend(&mut self, x: i64, y: i64, color: [u8; 4]) {
        if x < 0 || y < 0 || x >= i64::from(self.width) || y >= i64::from(self.height) {
            return;
        }
        let i = ((y as usize) * self.width as usize + x as usize) * 4;
        if color[3] == 0 {
            return;
        }
        if color[3] == 255 {
            self.rgba[i..i + 4].copy_from_slice(&color);
            return;
        }
        let src_a = u32::from(color[3]);
        let dst_a = u32::from(self.rgba[i + 3]);
        let out_a_scaled = src_a * 255 + dst_a * (255 - src_a);
        for (channel, &src) in color.iter().take(3).enumerate() {
            let numerator = u32::from(src) * src_a * 255
                + u32::from(self.rgba[i + channel]) * dst_a * (255 - src_a)
                + out_a_scaled / 2;
            self.rgba[i + channel] = numerator.checked_div(out_a_scaled).unwrap_or(0) as u8;
        }
        self.rgba[i + 3] = ((out_a_scaled + 127) / 255) as u8;
    }
    fn outline(&mut self, x: i64, y: i64, w: u32, h: u32, color: [u8; 4]) {
        for px in x.max(0)..(x + i64::from(w)).min(i64::from(self.width)) {
            self.blend(px, y, color);
            self.blend(px, y + i64::from(h) - 1, color);
        }
        for py in y.max(0)..(y + i64::from(h)).min(i64::from(self.height)) {
            self.blend(x, py, color);
            self.blend(x + i64::from(w) - 1, py, color);
        }
    }
    pub fn window_pixels(&self) -> Vec<u32> {
        self.rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| (u32::from(p[0]) << 16) | (u32::from(p[1]) << 8) | u32::from(p[2]))
            .collect()
    }
}
pub fn render(project: &Project, snapshot: &Snapshot, debug: bool) -> Result<Frame> {
    if let Some(event) = snapshot
        .flatland
        .as_ref()
        .and_then(|f| f.get("systems"))
        .and_then(|s| s.get("events"))
        .and_then(|v| v.as_array())
        .and_then(|v| v.last())
        && event.get("battle").is_some_and(|b| !b.is_null())
    {
        return render_battle(project, snapshot, event);
    }
    let width = project.manifest.window.width;
    let height = project.manifest.window.height;
    let view = snapshot
        .flatland
        .as_ref()
        .and_then(|f| f.get("systems"))
        .and_then(|s| s.get("view"))
        .and_then(|v| v.as_str())
        .and_then(|v| project.scenes[&snapshot.scene].gameplay.views.get(v));
    if let Some(view) = view
        && (view.zoom != 100 || view.tilt != 100 || view.shear != 0)
    {
        // Render extra world area before projection; never stretch a clipped viewport.
        let padding = 128;
        let mut padded = snapshot.clone();
        padded.camera[0] -= i64::from(padding);
        padded.camera[1] -= i64::from(padding);
        let source = render_world(
            project,
            &padded,
            debug,
            width + padding * 2,
            height + padding * 2,
        )?;
        let mut frame = Frame {
            width,
            height,
            rgba: snapshot.background.repeat((width * height) as usize),
        };
        for y in 0..height {
            for x in 0..width {
                let dy = i64::from(y) - i64::from(height / 2);
                let wy = dy * 10000 / (view.zoom * view.tilt);
                let wx =
                    ((i64::from(x) - i64::from(width / 2)) * 100 - wy * view.shear) / view.zoom;
                let sx = wx + i64::from(source.width / 2);
                let sy = wy + i64::from(source.height / 2);
                if sx >= 0
                    && sy >= 0
                    && sx < i64::from(source.width)
                    && sy < i64::from(source.height)
                {
                    let a = ((sy as u32 * source.width + sx as u32) * 4) as usize;
                    let b = ((y * width + x) * 4) as usize;
                    frame.rgba[b..b + 4].copy_from_slice(&source.rgba[a..a + 4]);
                }
            }
        }
        return Ok(frame);
    }
    render_world(project, snapshot, debug, width, height)
}
fn render_world(
    project: &Project,
    snapshot: &Snapshot,
    debug: bool,
    width: u32,
    height: u32,
) -> Result<Frame> {
    let mut frame = Frame {
        width,
        height,
        rgba: snapshot.background.repeat((width * height) as usize),
    };
    if let Some(mut map) = project
        .scenes
        .get(&snapshot.scene)
        .and_then(|s| s.map.clone())
    {
        if let Some(patches) = snapshot
            .flatland
            .as_ref()
            .and_then(|f| f.get("systems"))
            .and_then(|s| s.get("patches"))
            .and_then(|p| p.as_object())
        {
            for (key, tile) in patches {
                if let Some(at) = key.strip_prefix(&format!("{}:", snapshot.scene)) {
                    let a: Vec<_> = at
                        .split(':')
                        .filter_map(|v| v.parse::<usize>().ok())
                        .collect();
                    if a.len() == 2
                        && a[1] < map.rows.len()
                        && a[0] < map.rows[a[1]].len()
                        && let Some(tile) = tile.as_str()
                    {
                        map.rows[a[1]].replace_range(a[0]..a[0] + 1, tile);
                    }
                }
            }
        }
        let cell = i64::from(map.cell);
        let first_y = snapshot.camera[1].div_euclid(cell).max(0) as usize;
        let last_y =
            ((snapshot.camera[1] + i64::from(height)).div_euclid(cell) + 1).max(0) as usize;
        let first_x = snapshot.camera[0].div_euclid(cell).max(0) as usize;
        let last_x = ((snapshot.camera[0] + i64::from(width)).div_euclid(cell) + 1).max(0) as usize;
        for (y, row) in map.rows.iter().enumerate().take(last_y).skip(first_y) {
            for (x, ch) in row.chars().enumerate().take(last_x).skip(first_x) {
                let tile = &map.tiles[&ch.to_string()];
                if tile.color[3] == 0 {
                    continue;
                }
                let left = x as i64 * i64::from(map.cell) - snapshot.camera[0];
                let top = y as i64 * i64::from(map.cell) - snapshot.camera[1];
                for py in top.max(0)..(top + i64::from(map.cell)).min(i64::from(height)) {
                    for px in left.max(0)..(left + i64::from(map.cell)).min(i64::from(width)) {
                        frame.blend(px, py, tile.color);
                    }
                }
            }
        }
    }
    let mut entities: Vec<_> = snapshot.entities.iter().collect();
    entities.sort_by_key(|e| {
        (
            e.layer,
            if e.flatland
                .as_ref()
                .and_then(|a| a.get("depth"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
            {
                e.position.y + i64::from(e.size[1]) * SUBPIXELS
            } else {
                0
            },
            &e.id,
        )
    });
    for entity in &entities {
        let anchor = entity
            .flatland
            .as_ref()
            .and_then(|a| a.get("anchor"))
            .and_then(|v| v.as_array());
        let ax = anchor
            .and_then(|a| a.first())
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let ay = anchor
            .and_then(|a| a.get(1))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let x = entity.position.x.div_euclid(SUBPIXELS) - snapshot.camera[0] + ax;
        let z = entity
            .flatland
            .as_ref()
            .and_then(|a| a.get("z"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let y = entity.position.y.div_euclid(SUBPIXELS) - snapshot.camera[1] - z + ay;
        let visual = entity
            .flatland
            .as_ref()
            .and_then(|a| a.get("visual_size"))
            .and_then(|v| v.as_array());
        let w = visual
            .and_then(|a| a.first())
            .and_then(|v| v.as_u64())
            .unwrap_or(u64::from(entity.size[0])) as u32;
        let h = visual
            .and_then(|a| a.get(1))
            .and_then(|v| v.as_u64())
            .unwrap_or(u64::from(entity.size[1])) as u32;
        if let Some(color) = entity.color {
            let texture = entity
                .texture
                .as_ref()
                .map(|t| {
                    project.textures.get(t).ok_or_else(|| {
                        Error(format!("render entity {}: missing texture {t}", entity.id))
                    })
                })
                .transpose()?;
            let crop = entity
                .flatland
                .as_ref()
                .and_then(|a| a.get("atlas"))
                .and_then(|v| v.as_array());
            let get = |i: usize, default: u32| {
                crop.and_then(|v| v.get(i))
                    .and_then(|v| v.as_u64())
                    .map_or(default, |n| n as u32)
            };
            let bounds = texture.map(|t| (get(0, 0), get(1, 0), get(2, t.width), get(3, t.height)));
            let facing = entity
                .flatland
                .as_ref()
                .and_then(|a| a.get("facing"))
                .and_then(|v| v.as_array());
            let dx = facing
                .and_then(|f| f.first())
                .and_then(|v| v.as_i64())
                .unwrap_or(1);
            let dy = facing
                .and_then(|f| f.get(1))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let rotate = entity
                .flatland
                .as_ref()
                .and_then(|a| a.get("rotate"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let tint = color != [255; 4];
            for py in y.max(0)..(y + i64::from(h)).min(i64::from(height)) {
                for px in x.max(0)..(x + i64::from(w)).min(i64::from(width)) {
                    let pixel = if let Some(texture) = texture {
                        let (ox, oy, tw, th) = bounds.unwrap();
                        let mut tx = (px - x) as u32 * tw / w;
                        let mut ty = (py - y) as u32 * th / h;
                        if tw == th && rotate {
                            let max = tw - 1;
                            (tx, ty) = if dx < 0 {
                                (max - tx, max - ty)
                            } else if dy > 0 {
                                (ty, max - tx)
                            } else if dy < 0 {
                                (max - ty, tx)
                            } else {
                                (tx, ty)
                            };
                        }
                        let i = (((ty + oy) * texture.width + tx + ox) * 4) as usize;
                        if tint {
                            std::array::from_fn(|c| {
                                ((u16::from(texture.rgba[i + c]) * u16::from(color[c]) + 127) / 255)
                                    as u8
                            })
                        } else {
                            texture.rgba[i..i + 4].try_into().unwrap()
                        }
                    } else {
                        color
                    };
                    frame.blend(px, py, pixel);
                }
            }
        }
        if let Some(a) = entity.flatland.as_ref()
            && let (Some(max), Some(hp)) = (
                a.get("hp_max").and_then(|v| v.as_i64()),
                a.get("hp").and_then(|v| v.as_i64()),
            )
            && max > 0
        {
            let length = w.min(32);
            for dx in 0..length {
                let filled = i64::from(dx) * max < i64::from(length) * hp;
                frame.blend(
                    x + i64::from(dx),
                    y - 3,
                    if filled {
                        [210, 255, 56, 255]
                    } else {
                        [80, 40, 40, 255]
                    },
                );
            }
        }
    }
    if debug {
        for entity in entities {
            let x = entity.position.x.div_euclid(SUBPIXELS) - snapshot.camera[0];
            let y = entity.position.y.div_euclid(SUBPIXELS) - snapshot.camera[1];
            if entity.blocking || entity.trigger {
                frame.outline(
                    x,
                    y,
                    entity.size[0],
                    entity.size[1],
                    if entity.trigger {
                        [255, 64, 220, 255]
                    } else {
                        [255, 240, 60, 255]
                    },
                );
            }
        }
    }
    Ok(frame)
}

fn blit(
    frame: &mut Frame,
    project: &Project,
    file: &str,
    at: [i64; 2],
    size: [u32; 2],
    flash: bool,
) -> Result<()> {
    let [x, y] = at;
    let [w, h] = size;
    let t = project
        .textures
        .get(file)
        .ok_or_else(|| Error(format!("battle: missing texture {file}")))?;
    for py in y.max(0)..(y + i64::from(h)).min(i64::from(frame.height)) {
        for px in x.max(0)..(x + i64::from(w)).min(i64::from(frame.width)) {
            let tx = (px - x) as u32 * t.width / w;
            let ty = (py - y) as u32 * t.height / h;
            let i = ((ty * t.width + tx) * 4) as usize;
            let mut c: [u8; 4] = t.rgba[i..i + 4].try_into().unwrap();
            if flash && c[3] > 0 {
                c = [255, 255, 255, c[3]];
            }
            frame.blend(px, py, c);
        }
    }
    Ok(())
}
fn render_battle(
    project: &Project,
    snapshot: &Snapshot,
    event: &serde_json::Value,
) -> Result<Frame> {
    let width = project.manifest.window.width;
    let height = project.manifest.window.height;
    let mut frame = Frame {
        width,
        height,
        rgba: [169, 207, 187, 255].repeat((width * height) as usize),
    };
    let battle = &event["battle"];
    let pc = event["pc"].as_u64().unwrap_or(0) as usize;
    let event_id = event["event"].as_str().unwrap_or("");
    if let Some(ge4g_project::gameplay::Instruction::Battle {
        stage: Some(stage), ..
    }) = project.scenes[&snapshot.scene]
        .gameplay
        .events
        .get(event_id)
        .and_then(|s| s.get(pc))
        && let Some(file) = &stage.background
    {
        blit(&mut frame, project, file, [0, 0], [width, height], false)?;
    }
    let age = snapshot
        .tick
        .saturating_sub(battle["turn_tick"].as_u64().unwrap_or(snapshot.tick));
    let fighters = battle["fighters"]
        .as_array()
        .ok_or_else(|| Error("battle fighters missing".into()))?;
    for f in fighters {
        let enemy = f["enemy"].as_bool().unwrap_or(false);
        let hp = f["hp"].as_i64().unwrap_or(0);
        let previous = battle["previous_hp"]
            .get(f["id"].as_str().unwrap_or(""))
            .and_then(|v| v.as_i64())
            .unwrap_or(hp);
        let hurt = previous > hp;
        let shake = if hurt && age < 24 {
            if age % 6 < 3 { -3 } else { 3 }
        } else {
            0
        };
        let x = if enemy {
            i64::from(width) * 65 / 100
        } else {
            i64::from(width) * 13 / 100
        } + shake;
        let y = if enemy {
            i64::from(height) * 17 / 100
        } else {
            i64::from(height) * 51 / 100
        };
        let w = width * 27 / 100;
        let h = height * 38 / 100;
        for dy in 0..12i64 {
            for dx in 0..i64::from(w + 16) {
                let a = (dx - i64::from(w + 16) / 2) * 2;
                let b = (dy - 6) * i64::from(w + 16) / 6;
                if a * a + b * b < i64::from(w + 16).pow(2) {
                    frame.blend(x - 8 + dx, y + i64::from(h) - 8 + dy, [63, 90, 73, 160]);
                }
            }
        }
        if hp > 0 || age < 36 {
            let file = if enemy {
                f["sprite"].as_str()
            } else {
                f["back_sprite"].as_str().or_else(|| f["sprite"].as_str())
            };
            if let Some(file) = file {
                blit(
                    &mut frame,
                    project,
                    file,
                    [x, y],
                    [w, h],
                    hurt && age < 24 && age % 8 < 4,
                )?;
            } else {
                frame.outline(x, y, w, h, [24, 34, 47, 255]);
            }
        }
    }
    Ok(frame)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integer_alpha_blends_and_clips() {
        let mut frame = Frame {
            width: 1,
            height: 1,
            rgba: vec![0, 0, 0, 255],
        };
        frame.blend(0, 0, [255, 0, 0, 128]);
        assert_eq!(frame.rgba, [128, 0, 0, 255]);
        frame.blend(-1, 0, [255; 4]);
        assert_eq!(frame.rgba, [128, 0, 0, 255]);
    }
    #[test]
    fn transparent_destination_preserves_straight_rgba() {
        let mut frame = Frame {
            width: 1,
            height: 1,
            rgba: vec![0; 4],
        };
        frame.blend(0, 0, [10, 20, 30, 128]);
        assert_eq!(frame.rgba, [10, 20, 30, 128]);
    }
}

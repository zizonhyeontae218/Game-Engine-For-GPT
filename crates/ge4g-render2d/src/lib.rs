//! Canonical deterministic CPU renderer. Window and PNG share these bytes.
use ge4g_core::{Error, Result, SUBPIXELS, Snapshot};
use ge4g_project::{Project, atomic_bytes};
use std::path::Path;
mod battle_fx;
mod building;
mod projection;

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
    let projection = projection::Projection::new(project, snapshot);
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
        let colors: Vec<Vec<[u8; 4]>> = map
            .rows
            .iter()
            .map(|row| {
                row.chars()
                    .map(|ch| map.tiles[&ch.to_string()].color)
                    .collect()
            })
            .collect();
        for py in 0..i64::from(height) {
            for px in 0..i64::from(width) {
                let [wx, wy] = projection.inverse(px, py);
                let tx = wx.div_euclid(cell);
                let ty = wy.div_euclid(cell);
                if tx >= 0
                    && ty >= 0
                    && let Some(color) =
                        colors.get(ty as usize).and_then(|row| row.get(tx as usize))
                {
                    frame.blend(px, py, *color);
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
        if let Some(b) = entity.flatland.as_ref().and_then(|a| a.get("building")) {
            let b =
                serde_json::from_value(b.clone()).map_err(|e| Error(format!("building: {e}")))?;
            building::render(
                &mut frame,
                project,
                projection,
                &b,
                [
                    entity.position.x.div_euclid(SUBPIXELS),
                    entity.position.y.div_euclid(SUBPIXELS),
                ],
            )?;
            continue;
        }
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
        let world_x = entity.position.x.div_euclid(SUBPIXELS);
        let z = entity
            .flatland
            .as_ref()
            .and_then(|a| a.get("z"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let world_y = entity.position.y.div_euclid(SUBPIXELS);
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
        let ground = entity
            .flatland
            .as_ref()
            .and_then(|a| a.get("projection"))
            .and_then(|v| v.as_str())
            == Some("ground");
        let feet = [
            world_x + i64::from(entity.size[0]) / 2,
            world_y + i64::from(entity.size[1]),
        ];
        let [x, y] = if ground {
            projection.ground(world_x + ax, world_y + ay - z)
        } else {
            projection.upright(
                feet,
                [
                    ax - i64::from(entity.size[0]) / 2,
                    ay - i64::from(entity.size[1]) - z,
                ],
            )
        };
        let source_w = w;
        let source_h = h;
        let w = (i64::from(w) * projection.zoom / 100) as u32;
        let h = (i64::from(h) * projection.zoom / 100) as u32;
        let corners = [
            projection.ground(world_x + ax, world_y + ay - z),
            projection.ground(world_x + ax + i64::from(source_w), world_y + ay - z),
            projection.ground(world_x + ax, world_y + ay - z + i64::from(source_h)),
            projection.ground(
                world_x + ax + i64::from(source_w),
                world_y + ay - z + i64::from(source_h),
            ),
        ];
        let left = if ground {
            corners.iter().map(|c| c[0]).min().unwrap()
        } else {
            x
        };
        let right = if ground {
            corners.iter().map(|c| c[0]).max().unwrap()
        } else {
            x + i64::from(w)
        };
        let top = if ground {
            corners.iter().map(|c| c[1]).min().unwrap()
        } else {
            y
        };
        let bottom = if ground {
            corners.iter().map(|c| c[1]).max().unwrap()
        } else {
            y + i64::from(h)
        };
        let defaults = project
            .manifest
            .features
            .iter()
            .any(|f| f == "entity_defaults");
        if defaults && !ground {
            let [sx, sy] = projection.ground(feet[0], feet[1]);
            for dy in -2i64..=2 {
                for dx in -7i64..=7 {
                    if dx * dx + dy * dy * 12 < 50 {
                        frame.blend(sx + dx, sy + dy, [25, 39, 32, 75]);
                    }
                }
            }
        }
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
            for py in top.max(0)..bottom.min(i64::from(height)) {
                for px in left.max(0)..right.min(i64::from(width)) {
                    let [local_x, local_y] = if ground {
                        let [wx, wy] = projection.inverse(px, py);
                        [wx - world_x - ax, wy - world_y - ay + z]
                    } else {
                        [
                            (px - x) * 100 / projection.zoom,
                            (py - y) * 100 / projection.zoom,
                        ]
                    };
                    if local_x < 0
                        || local_y < 0
                        || local_x >= i64::from(source_w)
                        || local_y >= i64::from(source_h)
                    {
                        continue;
                    }
                    let pixel = if let Some(texture) = texture {
                        let (ox, oy, tw, th) = bounds.unwrap();
                        let mut tx = local_x as u32 * tw / source_w;
                        let mut ty = local_y as u32 * th / source_h;
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
                    } else if defaults
                        && entity.texture.is_none()
                        && !entity.components.iter().any(|c| c == "sprite")
                    {
                        let mid = i64::from(source_w) / 2;
                        if local_y < i64::from(source_h) / 3 {
                            if (local_x - mid).pow(2) + (local_y - i64::from(source_h) / 6).pow(2)
                                < (i64::from(source_w) / 3).max(2).pow(2)
                            {
                                color
                            } else {
                                [0; 4]
                            }
                        } else if local_x >= i64::from(source_w) / 4
                            && local_x < i64::from(source_w) * 3 / 4
                        {
                            color
                        } else {
                            [0; 4]
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
    if project
        .manifest
        .features
        .iter()
        .any(|f| f == "entity_defaults")
        && let Some(attacks) = snapshot
            .flatland
            .as_ref()
            .and_then(|f| f.get("systems"))
            .and_then(|s| s.get("attacks"))
            .and_then(|v| v.as_array())
    {
        for shot in attacks {
            if let Some(a) = shot["preset"]
                .as_str()
                .and_then(|id| project.scenes[&snapshot.scene].gameplay.attacks.get(id))
                && a.projectile_speed > 0
            {
                let age = snapshot
                    .tick
                    .saturating_sub(shot["start"].as_u64().unwrap_or(snapshot.tick));
                if age < u64::from(a.startup) {
                    continue;
                }
                let travel = (age - u64::from(a.startup)) as i64 * i64::from(a.projectile_speed);
                let dx = shot["direction"][0].as_i64().unwrap_or(1);
                let dy = shot["direction"][1].as_i64().unwrap_or(0);
                let x = (shot["origin"]["x"].as_i64().unwrap_or(0) + travel * dx) / SUBPIXELS + 8;
                let y = (shot["origin"]["y"].as_i64().unwrap_or(0) + travel * dy) / SUBPIXELS + 8;
                let [px, py] = projection.ground(x, y);
                for t in 0..8 {
                    frame.blend(
                        px - dx * t,
                        py - dy * t,
                        [255, 211, 74, (220 - t * 20) as u8],
                    );
                }
                for oy in -3i64..=3 {
                    for ox in -3i64..=3 {
                        if ox * ox + oy * oy <= 9 {
                            frame.blend(px + ox, py + oy, [255, 225, 93, 255]);
                        }
                    }
                }
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
                c = [255, 72, 75, c[3]];
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
    let fx = battle["fx"].as_array().cloned().unwrap_or_default();
    let mut centers = std::collections::BTreeMap::new();
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
        let offsets: Vec<_> = fx
            .iter()
            .map(|fx| battle_fx::offset(fx, f["id"].as_str().unwrap_or(""), snapshot.tick))
            .collect();
        let motion: i64 = offsets.iter().map(|d| d[0]).sum();
        let x = if enemy {
            i64::from(width) * 65 / 100
        } else {
            i64::from(width) * 13 / 100
        } + shake
            + if enemy { -motion } else { motion };
        let y = if enemy {
            i64::from(height) * 17 / 100
        } else {
            i64::from(height) * 51 / 100
        };
        let w = width * 27 / 100;
        let h = height * 38 / 100;
        centers.insert(
            f["id"].as_str().unwrap_or(""),
            [x + i64::from(w) / 2, y + i64::from(h) / 2],
        );
        for dy in 0..12i64 {
            for dx in 0..i64::from(w + 16) {
                let a = (dx - i64::from(w + 16) / 2) * 2;
                let b = (dy - 6) * i64::from(w + 16) / 6;
                if a * a + b * b < i64::from(w + 16).pow(2) {
                    frame.blend(x - 8 + dx, y + i64::from(h) - 8 + dy, [63, 90, 73, 160]);
                }
            }
        }
        let flash = fx
            .iter()
            .any(|fx| battle_fx::flash(fx, f["id"].as_str().unwrap_or(""), snapshot.tick));
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
                    fx.iter().any(|fx| {
                        battle_fx::flash(fx, f["id"].as_str().unwrap_or(""), snapshot.tick)
                    }) || (fx.is_empty() && hurt && age < 24 && age % 8 < 4),
                )?;
            } else {
                for dy in 0..i64::from(h) {
                    for dx in 0..i64::from(w) {
                        let a = dx - i64::from(w) / 2;
                        let b = dy - i64::from(h) / 2;
                        if a * a * 4 + b * b * 2 < i64::from(w).pow(2) {
                            frame.blend(
                                x + dx,
                                y + dy,
                                if flash {
                                    [255, 72, 75, 255]
                                } else if enemy {
                                    [147, 104, 197, 255]
                                } else {
                                    [85, 159, 99, 255]
                                },
                            );
                        }
                    }
                }
                frame.outline(
                    x + i64::from(w) / 3,
                    y + i64::from(h) / 3,
                    5,
                    5,
                    [25, 35, 37, 255],
                );
            }
        }
    }
    for fx in &fx {
        if let (Some(actor), Some(target)) = (
            centers.get(fx["actor"].as_str().unwrap_or("")),
            centers.get(fx["target"].as_str().unwrap_or("")),
        ) {
            battle_fx::draw(&mut frame, fx, snapshot.tick, *actor, *target);
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

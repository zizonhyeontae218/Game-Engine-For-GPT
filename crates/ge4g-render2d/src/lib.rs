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
    let (width, height) = (
        project.manifest.window.width,
        project.manifest.window.height,
    );
    let mut frame = Frame {
        width,
        height,
        rgba: snapshot.background.repeat((width * height) as usize),
    };
    if let Some(map) = project
        .scenes
        .get(&snapshot.scene)
        .and_then(|s| s.map.as_ref())
    {
        for (y, row) in map.rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let tile = &map.tiles[&ch.to_string()];
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
        let x = entity.position.x.div_euclid(SUBPIXELS) - snapshot.camera[0];
        let y = entity.position.y.div_euclid(SUBPIXELS) - snapshot.camera[1];
        let [w, h] = entity.size;
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
            for py in y.max(0)..(y + i64::from(h)).min(i64::from(height)) {
                for px in x.max(0)..(x + i64::from(w)).min(i64::from(width)) {
                    let pixel = if let Some(texture) = texture {
                        let mut tx = (px - x) as u32 * texture.width / w;
                        let mut ty = (py - y) as u32 * texture.height / h;
                        if texture.width == texture.height
                            && entity
                                .flatland
                                .as_ref()
                                .and_then(|a| a.get("rotate"))
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false)
                        {
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
                            let max = texture.width - 1;
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
                        let i = ((ty * texture.width + tx) * 4) as usize;
                        std::array::from_fn(|c| {
                            ((u16::from(texture.rgba[i + c]) * u16::from(color[c]) + 127) / 255)
                                as u8
                        })
                    } else {
                        color
                    };
                    frame.blend(px, py, pixel);
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

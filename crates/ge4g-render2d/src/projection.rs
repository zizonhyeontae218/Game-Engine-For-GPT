//! Ground projection; upright sprites only project their feet.
use ge4g_core::{SUBPIXELS, Snapshot};
use ge4g_project::Project;
#[derive(Clone, Copy)]
pub struct Projection {
    pub zoom: i64,
    pub tilt: i64,
    pub shear: i64,
    origin: [i64; 2],
    anchor: [i64; 2],
}
impl Projection {
    pub fn new(project: &Project, snapshot: &Snapshot) -> Self {
        let scene = &project.scenes[&snapshot.scene];
        let systems = snapshot.flatland.as_ref().and_then(|f| f.get("systems"));
        let persistent = systems
            .and_then(|s| s.get("gameplay_view"))
            .filter(|v| v.is_object());
        let legacy = systems
            .and_then(|s| s.get("view"))
            .and_then(|v| v.as_str())
            .and_then(|id| scene.gameplay.views.get(id));
        let mut p = Self {
            zoom: persistent
                .and_then(|v| v["zoom"].as_i64())
                .or(legacy.map(|v| v.zoom))
                .unwrap_or(100),
            tilt: persistent
                .and_then(|v| v["tilt"].as_i64())
                .or(legacy.map(|v| v.tilt))
                .unwrap_or(100),
            shear: persistent
                .and_then(|v| v["shear"].as_i64())
                .or(legacy.map(|v| v.shear))
                .unwrap_or(0),
            origin: [
                snapshot.camera[0] + i64::from(project.manifest.window.width) / 2,
                snapshot.camera[1] + i64::from(project.manifest.window.height) / 2,
            ],
            anchor: [
                i64::from(project.manifest.window.width) / 2,
                i64::from(project.manifest.window.height) / 2,
            ],
        };
        // Preserve the followed actor's old screen anchor even at camera bounds.
        if systems
            .and_then(|s| s.get("camera"))
            .is_none_or(|v| v.is_null())
            && let Some(actor) = scene
                .gameplay
                .camera
                .as_ref()
                .and_then(|c| snapshot.entities.iter().find(|e| e.id == c.target))
        {
            p.origin = [
                actor.position.x.div_euclid(SUBPIXELS) + i64::from(actor.size[0]) / 2,
                actor.position.y.div_euclid(SUBPIXELS) + i64::from(actor.size[1]),
            ];
            p.anchor = [
                p.origin[0] - snapshot.camera[0],
                p.origin[1] - snapshot.camera[1],
            ];
        }
        p
    }
    pub fn ground(self, x: i64, y: i64) -> [i64; 2] {
        let dx = x - self.origin[0];
        let dy = y - self.origin[1];
        [
            self.anchor[0] + (dx * self.zoom + dy * self.shear) / 100,
            self.anchor[1] + dy * self.zoom * self.tilt / 10000,
        ]
    }
    pub fn inverse(self, x: i64, y: i64) -> [i64; 2] {
        let dy = (y - self.anchor[1]) * 10000 / (self.zoom * self.tilt);
        [
            self.origin[0] + ((x - self.anchor[0]) * 100 - dy * self.shear) / self.zoom,
            self.origin[1] + dy,
        ]
    }
    pub fn upright(self, feet: [i64; 2], offset: [i64; 2]) -> [i64; 2] {
        let a = self.ground(feet[0], feet[1]);
        [
            a[0] + offset[0] * self.zoom / 100,
            a[1] + offset[1] * self.zoom / 100,
        ]
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn feet_stay_at_target_anchor_and_vertical_sprites_never_shear() {
        for (zoom, tilt, shear) in [(100, 100, 0), (115, 68, 12), (125, 72, -12)] {
            let p = Projection {
                zoom,
                tilt,
                shear,
                origin: [160, 120],
                anchor: [168, 136],
            };
            assert_eq!(p.ground(160, 120), [168, 136]);
            let feet = p.upright([160, 120], [0, 0]);
            let head = p.upright([160, 120], [0, -32]);
            assert_eq!(feet[0], head[0]);
            assert_eq!(feet[1] - head[1], 32 * zoom / 100);
            assert_eq!(
                p.ground(200, 160)[1] - p.ground(200, 120)[1],
                40 * zoom * tilt / 10000
            );
        }
    }
}

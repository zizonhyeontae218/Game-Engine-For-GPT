//! Deterministic semantic surfaces, composited upright over a projected footprint.
use super::{Frame, Project, Result, projection::Projection};
use ge4g_project::building::{Building, Material, Roof};
fn polygon(
    frame: &mut Frame,
    project: &Project,
    points: &[[i64; 2]],
    color: [u8; 4],
    surface: Option<&str>,
) -> Result<()> {
    let left = points.iter().map(|p| p[0]).min().unwrap();
    let right = points.iter().map(|p| p[0]).max().unwrap();
    let top = points.iter().map(|p| p[1]).min().unwrap();
    let bottom = points.iter().map(|p| p[1]).max().unwrap();
    let texture = surface
        .map(|f| {
            project
                .textures
                .get(f)
                .ok_or_else(|| ge4g_core::Error(format!("building missing surface {f}")))
        })
        .transpose()?;
    for y in top.max(0)..bottom.min(i64::from(frame.height)) {
        for x in left.max(0)..right.min(i64::from(frame.width)) {
            let mut inside = false;
            let mut j = points.len() - 1;
            for (i, p) in points.iter().enumerate() {
                let q = points[j];
                if (p[1] > y) != (q[1] > y) && x < (q[0] - p[0]) * (y - p[1]) / (q[1] - p[1]) + p[0]
                {
                    inside = !inside;
                }
                j = i;
            }
            if inside {
                let c = if let Some(t) = texture {
                    let tx = ((x - left) * i64::from(t.width) / (right - left).max(1)) as u32;
                    let ty = ((y - top) * i64::from(t.height) / (bottom - top).max(1)) as u32;
                    let at = ((ty * t.width + tx) * 4) as usize;
                    t.rgba[at..at + 4].try_into().unwrap()
                } else {
                    color
                };
                frame.blend(x, y, c);
            }
        }
    }
    Ok(())
}
pub(super) fn render(
    frame: &mut Frame,
    project: &Project,
    p: Projection,
    b: &Building,
    origin: [i64; 2],
) -> Result<()> {
    let w = i64::from(b.footprint[0]);
    let d = i64::from(b.footprint[1]);
    let h = i64::from(b.height) * p.zoom / 100;
    let front = p.ground(origin[0], origin[1] + d);
    let back = p.ground(origin[0], origin[1]);
    let right = p.ground(origin[0] + w, origin[1] + d);
    let rear = p.ground(origin[0] + w, origin[1]);
    polygon(
        frame,
        project,
        &[
            [front[0] - 3, front[1] + 3],
            [right[0] + 5, right[1] + 3],
            [rear[0] + 5, rear[1] + 3],
            [back[0] - 3, back[1] + 3],
        ],
        [28, 35, 29, 65],
        None,
    )?;
    let base = match b.material {
        Material::Wood => [176, 126, 76, 255],
        Material::Plaster => [220, 211, 183, 255],
        Material::Brick => [177, 92, 78, 255],
        Material::Stone => [140, 148, 158, 255],
        Material::Metal => [145, 173, 184, 255],
    };
    let side = std::array::from_fn(|i| {
        if i == 3 {
            255
        } else {
            (u16::from(base[i]) * 3 / 4) as u8
        }
    });
    polygon(
        frame,
        project,
        &[
            right,
            rear,
            [rear[0], rear[1] - h],
            [right[0], right[1] - h],
        ],
        side,
        b.side_surface.as_deref(),
    )?;
    polygon(
        frame,
        project,
        &[
            front,
            right,
            [right[0], right[1] - h],
            [front[0], front[1] - h],
        ],
        base,
        b.facade_surface.as_deref(),
    )?;
    let rise = match b.roof {
        Roof::Flat => 3,
        Roof::Shed => 10,
        Roof::Gable => 15,
    } * p.zoom
        / 100;
    polygon(
        frame,
        project,
        &[
            [front[0] - 3, front[1] - h],
            [right[0] + 3, right[1] - h],
            [rear[0] + 3, rear[1] - h - rise],
            [back[0] - 3, back[1] - h - rise],
        ],
        [103, 123, 157, 255],
        b.roof_surface.as_deref(),
    )?;
    // Authored transparent facade remains transparent; details only belong to fallback.
    if b.facade_surface.is_none() {
        let span = right[0] - front[0];
        let door_w = (span / 5).max(4);
        let door_h = h * 2 / 3;
        let x = front[0] + span / 2 - door_w / 2;
        let y = front[1] - door_h;
        polygon(
            frame,
            project,
            &[
                [x, y],
                [x + door_w, y],
                [x + door_w, front[1]],
                [x, front[1]],
            ],
            [62, 55, 50, 255],
            None,
        )?;
        for at in [front[0] + span / 8, front[0] + span * 3 / 4] {
            frame.outline(
                at,
                front[1] - h * 3 / 4,
                (span / 7).max(3) as u32,
                (h / 3).max(3) as u32,
                [57, 68, 81, 255],
            );
        }
        frame.outline(
            front[0],
            front[1] - h,
            span.max(1) as u32,
            h as u32,
            [76, 63, 54, 255],
        );
    }
    Ok(())
}

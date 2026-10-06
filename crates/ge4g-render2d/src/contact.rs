//! Ground contact band and feet ordering; sprite top/layer cannot override depth.
use super::{Frame, projection::Projection};
use ge4g_core::{EntitySnapshot, SUBPIXELS};
pub(super) fn ground(e: &EntitySnapshot) -> bool {
    e.flatland
        .as_ref()
        .is_some_and(|a| a["projection"] == "ground")
}

pub(super) fn sort_key(e: &EntitySnapshot, defaults: bool) -> (i64, i64, i64, i64, &str) {
    let a = e.flatland.as_ref();
    if !defaults {
        return (
            i64::from(e.layer),
            0,
            0,
            if a.is_some_and(|a| a["depth"] == true) {
                e.position.y + i64::from(e.size[1]) * SUBPIXELS
            } else {
                0
            },
            &e.id,
        );
    }
    (
        if ground(e) { 0 } else { 1 },
        a.and_then(|a| a["plane"].as_i64()).unwrap_or(0),
        a.and_then(|a| a["z"].as_i64()).unwrap_or(0),
        e.position.y + i64::from(e.size[1]) * SUBPIXELS,
        &e.id,
    )
}
pub(super) fn shadow(frame: &mut Frame, e: &EntitySnapshot, p: Projection) {
    if ground(e) {
        return;
    }
    let x = e.position.x.div_euclid(SUBPIXELS);
    let y = e.position.y.div_euclid(SUBPIXELS) + i64::from(e.size[1]);
    let building = e
        .flatland
        .as_ref()
        .is_some_and(|a| a.get("building").is_some());
    let [sx, sy] = p.ground(x + i64::from(e.size[0]) / 2, y);
    let rx = if building {
        i64::from(e.size[0]) / 2
    } else {
        7
    } * p.zoom
        / 100;
    let ry = if p.tilt == 100 { 1 } else { 2 };
    for dy in -ry..=ry {
        for dx in -rx..=rx {
            if dx * dx * ry * ry + dy * dy * rx * rx <= rx * rx * ry * ry {
                frame.blend(
                    sx + dx,
                    sy + dy,
                    [25, 39, 32, if p.tilt == 100 { 38 } else { 55 }],
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entity(id: &str, y: i64, layer: i32) -> EntitySnapshot {
        EntitySnapshot {
            id: id.into(),
            position: ge4g_core::Vec2::pixels(40, y),
            size: [16, 16],
            components: vec![],
            tags: vec![],
            metadata: Default::default(),
            color: Some([40, 80, 200, 255]),
            texture: None,
            layer,
            blocking: false,
            trigger: false,
            flatland: Some(serde_json::json!({"depth":true,"anchor":[0,-40],"plane":0,"z":0})),
        }
    }
    #[test]
    fn feet_contact_overrides_player_layer_and_visual_top_with_stable_ties() {
        let player = entity("player", 100, 10);
        let npc = entity("npc", 108, 0);
        assert!(sort_key(&player, true) < sort_key(&npc, true));
        assert!(sort_key(&player, false) > sort_key(&npc, false)); // legacy explicit layers retained
        let tied = entity("z_npc", 100, 0);
        assert!(sort_key(&player, true) < sort_key(&tied, true));
        let mut raised = player.clone();
        raised.flatland.as_mut().unwrap()["z"] = serde_json::json!(1);
        assert!(sort_key(&raised, true) > sort_key(&npc, true));
    }
}

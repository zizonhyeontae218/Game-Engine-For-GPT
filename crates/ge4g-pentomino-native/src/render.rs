//! Renderer DTO conversion only: authoritative battle data comes from P2.
use crate::{BridgeError, ErrorCode, GameConfig, Result};
use ge4g_pentomino::p2::ReadFrame;
use ge4g_pentomino_gameplay as gameplay;
use ge4g_pentomino_view::{CameraFrame, CameraTarget, Projection, Quaternion, WorldPoint, project};
use std::collections::BTreeMap;

pub(crate) fn validate_camera(
    config: &GameConfig,
    read: &ReadFrame,
    camera: &CameraFrame,
) -> Result<()> {
    validate_target(config, &camera.target)?;
    if camera.scene != gameplay::scene(read)?
        || camera.viewport.x != 0.
        || camera.viewport.y != 0.
        || camera.viewport.width != f64::from(config.width)
        || camera.viewport.height != f64::from(config.height)
    {
        return Err(BridgeError::new(
            ErrorCode::RenderFailure,
            "camera/source scene or viewport differs",
        ));
    }
    let entities = gameplay::fighter_entities(read)?;
    for item in &camera.items {
        let index = entities
            .iter()
            .position(|r| r == &item.entity)
            .ok_or_else(|| {
                BridgeError::new(ErrorCode::RenderFailure, "unexpected rendered entity")
            })?;
        let [x, y] = config.fighters[index].position;
        let world = WorldPoint([x as f64, -y as f64, 0.]);
        if item.world != world || item.screen != project(world, &camera.target, &camera.viewport)? {
            return Err(BridgeError::new(
                ErrorCode::RenderFailure,
                "CameraFrame differs from actual authority positions",
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_target(config: &GameConfig, target: &CameraTarget) -> Result<()> {
    let lens = match target.projection {
        Projection::Orthographic {
            half_height,
            near,
            far,
            focus_distance,
        }
        | Projection::Blended {
            half_height,
            near,
            far,
            focus_distance,
            perspective_weight: 0.,
        } => [half_height, near, far, focus_distance],
        _ => {
            return Err(BridgeError::new(
                ErrorCode::Unsupported,
                "P4 CPU adapter requires orthographic lens",
            ));
        }
    };
    let transform = &target.view_transform;
    if lens != [f64::from(config.height) / 2., 0.1, 1000., 100.]
        || target.pose.position != [0., 0., 100.]
        || target.pose.orientation != Quaternion::identity()
        || transform.rotation != Quaternion::identity()
        || transform.scale[0] != 1.
        || transform.scale[2] != 1.
        || !(0.25..=2.).contains(&transform.scale[1])
        || !(0.25..=4.).contains(&target.pose.zoom)
        || !transform.shear_xy.is_finite()
        || transform.shear_xy.abs() > 8.
        || transform.origin[2] != 0.
        || !transform.origin[0].is_finite()
        || (transform.origin[0] - f64::from(config.width) / 2.).abs() > 1024.
        || !transform.origin[1].is_finite()
        || (transform.origin[1] + f64::from(config.height) / 2.).abs() > 1024.
    {
        return Err(BridgeError::new(
            ErrorCode::Unsupported,
            "target outside P4 supported CPU camera subset",
        ));
    }
    Ok(())
}

/// P3's public canonical save1 retains base and transition endpoints. Validate
/// every target against the P4 adapter subset, including a not-yet-visible goal.
pub(crate) fn validate_saved_targets(config: &GameConfig, bytes: &[u8]) -> Result<()> {
    let saved: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|e| BridgeError::new(ErrorCode::InvalidSave, e.to_string()))?;
    let cameras = saved
        .get("cameras")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| BridgeError::new(ErrorCode::InvalidSave, "missing saved cameras"))?;
    for camera in cameras.values() {
        let validate = |value: &serde_json::Value| -> Result<()> {
            let target: CameraTarget = serde_json::from_value(value.clone())
                .map_err(|e| BridgeError::new(ErrorCode::InvalidSave, e.to_string()))?;
            validate_target(config, &target)
        };
        validate(
            camera
                .get("base")
                .ok_or_else(|| BridgeError::new(ErrorCode::InvalidSave, "missing saved base"))?,
        )?;
        if let Some(transition) = camera.get("transition").filter(|v| !v.is_null()) {
            for key in ["start", "target"] {
                validate(transition.get(key).ok_or_else(|| {
                    BridgeError::new(ErrorCode::InvalidSave, "missing saved transition endpoint")
                })?)?;
            }
        }
    }
    Ok(())
}

pub(crate) fn render(
    config: &GameConfig,
    read: &ReadFrame,
    camera: &CameraFrame,
) -> Result<ge4g_render2d::Frame> {
    validate_camera(config, read, camera)?;
    let battle = gameplay::battle_state(read, config)?;
    let references = gameplay::fighter_entities(read)?;
    let background = [24, 28, 40, 255];
    let scene_id = gameplay::SCENE_ID.to_owned();
    let scene = ge4g_project::Scene {
        schema_version: 1,
        id: scene_id.clone(),
        background,
        camera: [0, 0],
        spawns: BTreeMap::new(),
        on_enter: BTreeMap::new(),
        entities: Vec::new(),
        map: None,
        prefabs: BTreeMap::new(),
        rules: Vec::new(),
        script: None,
        sounds: BTreeMap::new(),
        resources: Vec::new(),
        gameplay: Default::default(),
    };
    let project = ge4g_project::Project {
        root: Default::default(),
        manifest: ge4g_project::Manifest {
            schema_version: 1,
            name: "P4 native turn probe".into(),
            start_scene: scene_id.clone(),
            window: ge4g_project::WindowSpec {
                width: config.width,
                height: config.height,
            },
            scenes: BTreeMap::new(),
            state: BTreeMap::new(),
            tests: Vec::new(),
            features: Vec::new(),
        },
        scenes: BTreeMap::from([(scene_id.clone(), scene)]),
        textures: BTreeMap::new(),
        files_checked: Vec::new(),
        scripts: BTreeMap::new(),
    };
    let entities = config
        .fighters
        .iter()
        .enumerate()
        .map(|(i, f)| ge4g_core::EntitySnapshot {
            id: references[i].local.clone(),
            position: ge4g_core::Vec2::pixels(f.position[0] - 8, f.position[1] - 24),
            size: [16, 24],
            components: vec!["sprite".into()],
            tags: Vec::new(),
            metadata: BTreeMap::new(),
            color: Some(f.color),
            texture: None,
            layer: 0,
            blocking: false,
            trigger: false,
            flatland: Some(serde_json::json!({"hp":battle.hp[i],"hp_max":f.hp})),
        })
        .collect();
    let snapshot = ge4g_core::Snapshot {
        schema_version: 1,
        engine_version: "0.3.0-alpha.1".into(),
        tick: read.tick,
        tick_hz: config.tick_hz,
        subpixels_per_pixel: ge4g_core::SUBPIXELS,
        scene: scene_id,
        camera: [0, 0],
        background,
        entities,
        state: BTreeMap::new(),
        events: Vec::new(),
        events_dropped: 0,
        flatland: None,
    };
    ge4g_pentomino_view_legacy::render_legacy(&project, &snapshot, camera, false)
        .map_err(|e| BridgeError::new(ErrorCode::RenderFailure, e.to_string()))
}

use crate::{format::*, math::*, types::*};
use ge4g_pentomino::p2::ReadFrame;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedView {
    config: ViewConfig,
    descriptor: FormatDescriptor,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Transition {
    start: CameraTarget,
    target: CameraTarget,
    duration: u32,
    elapsed: u32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CameraState {
    config: CameraConfig,
    active: bool,
    base: CameraTarget,
    transition: Option<Transition>,
    follow_initialized: bool,
    previous_entity: [f64; 3],
    anchor: [f64; 3],
    shake_count: u32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    format_version: u32,
    contract_version: u32,
    content_binding: String,
    tick: u64,
    views: BTreeMap<ViewId, SavedView>,
    cameras: BTreeMap<CameraId, CameraState>,
}
struct CappedWriter {
    bytes: Vec<u8>,
    limit: usize,
}
impl Write for CappedWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() > self.limit.saturating_sub(self.bytes.len()) {
            return Err(io::Error::other("presentation save budget exceeded"));
        }
        self.bytes.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn encode(state: &State) -> Result<Vec<u8>, Error> {
    let mut writer = CappedWriter {
        bytes: Vec::new(),
        limit: ViewLimits::default().max_save_bytes,
    };
    serde_json::to_writer(&mut writer, state)
        .map_err(|_| err(ErrorCode::BudgetExceeded, "canonical save exceeds budget"))?;
    Ok(writer.bytes)
}
/// Presentation lifecycle host, consuming only immutable public read selections.
pub struct ViewHost {
    state: State,
    plugins: BTreeMap<ViewId, Box<dyn FormatPlugin>>,
}
impl ViewHost {
    pub fn new(content_binding: &str) -> Result<Self, Error> {
        if content_binding.is_empty()
            || content_binding.len() > 128
            || !content_binding
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.:-".contains(&b))
        {
            return Err(err(ErrorCode::InvalidIdentifier, "invalid content binding"));
        }
        Ok(Self {
            state: State {
                format_version: 1,
                contract_version: 1,
                content_binding: content_binding.into(),
                tick: 0,
                views: BTreeMap::new(),
                cameras: BTreeMap::new(),
            },
            plugins: BTreeMap::new(),
        })
    }
    pub fn install(
        &mut self,
        plugin: Box<dyn FormatPlugin>,
        config: ViewConfig,
        read: &ReadFrame,
    ) -> Result<(), Error> {
        if self.state.views.contains_key(&config.id) {
            return Err(err(ErrorCode::DuplicateId, "view already installed"));
        }
        if self.state.views.len() >= 8 {
            return Err(err(ErrorCode::BudgetExceeded, "view count budget"));
        }
        preflight_read(read)?;
        let descriptor = plugin.descriptor();
        validate_view_config(&config, &descriptor)?;
        let mut selection = plugin.extract(read, &config.binding, &config.scene)?;
        normalize_selection(&mut selection)?;
        validate_selection(read, &selection, &config.scene)?;
        let mut state = self.state.clone();
        let id = config.id.clone();
        state
            .views
            .insert(id.clone(), SavedView { config, descriptor });
        encode(&state)?;
        self.state = state;
        self.plugins.insert(id, plugin);
        Ok(())
    }
    pub fn replace(
        &mut self,
        id: &ViewId,
        plugin: Box<dyn FormatPlugin>,
        config: ViewConfig,
        read: &ReadFrame,
    ) -> Result<(), Error> {
        if !self.state.views.contains_key(id) {
            return Err(err(ErrorCode::MissingView, "replace view absent"));
        }
        if config.id != *id {
            return Err(err(ErrorCode::InvalidInput, "replacement id mismatch"));
        }
        preflight_read(read)?;
        let descriptor = plugin.descriptor();
        validate_view_config(&config, &descriptor)?;
        let mut selection = plugin.extract(read, &config.binding, &config.scene)?;
        normalize_selection(&mut selection)?;
        validate_selection(read, &selection, &config.scene)?;
        let mut state = self.state.clone();
        state
            .views
            .insert(id.clone(), SavedView { config, descriptor });
        for camera in state.cameras.values().filter(|c| c.config.view == *id) {
            validate_camera_state(camera)?;
            validate_follow(camera, &selection)?;
            let target = effective_target(camera, &state.views[id].config.policy, &selection)?;
            render_camera(camera, &state.views[id], &selection, target)?;
            if let Some(transition) = &camera.transition {
                for endpoint in [&transition.start, &transition.target] {
                    let mut endpoint_camera = camera.clone();
                    endpoint_camera.base = endpoint.clone();
                    let target = effective_target(
                        &endpoint_camera,
                        &state.views[id].config.policy,
                        &selection,
                    )?;
                    render_camera(&endpoint_camera, &state.views[id], &selection, target)?;
                }
            }
        }
        encode(&state)?;
        self.state = state;
        self.plugins.insert(id.clone(), plugin);
        Ok(())
    }
    pub fn remove(&mut self, id: &ViewId) -> Result<(), Error> {
        if !self.state.views.contains_key(id) {
            return Err(err(ErrorCode::MissingView, "view absent"));
        }
        let mut state = self.state.clone();
        state.views.remove(id);
        state.cameras.retain(|_, c| c.config.view != *id);
        encode(&state)?;
        self.state = state;
        self.plugins.remove(id);
        Ok(())
    }
    pub fn add_camera(&mut self, mut config: CameraConfig, read: &ReadFrame) -> Result<(), Error> {
        if self.state.cameras.contains_key(&config.id) {
            return Err(err(ErrorCode::DuplicateId, "camera already exists"));
        }
        if self.state.cameras.len() >= 32 {
            return Err(err(ErrorCode::BudgetExceeded, "camera count budget"));
        }
        if !self.state.views.contains_key(&config.view) {
            return Err(err(ErrorCode::MissingView, "camera view absent"));
        }
        normalize_config(&mut config)?;
        validate_camera_config(&config)?;
        let selections = self.selections(read)?;
        let selected = &selections[&config.view];
        let camera = CameraState {
            base: config.target.clone(),
            config,
            active: false,
            transition: None,
            follow_initialized: false,
            previous_entity: [0.; 3],
            anchor: [0.; 3],
            shake_count: 0,
        };
        validate_follow(&camera, selected)?;
        let view = &self.state.views[&camera.config.view];
        render_camera(
            &camera,
            view,
            selected,
            effective_target(&camera, &view.config.policy, selected)?,
        )?;
        let mut state = self.state.clone();
        state.cameras.insert(camera.config.id.clone(), camera);
        encode(&state)?;
        self.state = state;
        Ok(())
    }
    pub fn remove_camera(&mut self, id: &CameraId) -> Result<(), Error> {
        if !self.state.cameras.contains_key(id) {
            return Err(err(ErrorCode::MissingCamera, "camera absent"));
        }
        let mut state = self.state.clone();
        state.cameras.remove(id);
        encode(&state)?;
        self.state = state;
        Ok(())
    }
    pub fn activate(&mut self, id: &CameraId) -> Result<(), Error> {
        self.change_active(id, true)
    }
    pub fn deactivate(&mut self, id: &CameraId) -> Result<(), Error> {
        self.change_active(id, false)
    }
    fn change_active(&mut self, id: &CameraId, active: bool) -> Result<(), Error> {
        if !self.state.cameras.contains_key(id) {
            return Err(err(ErrorCode::MissingCamera, "camera absent"));
        }
        let mut state = self.state.clone();
        state.cameras.get_mut(id).expect("checked").active = active;
        encode(&state)?;
        self.state = state;
        Ok(())
    }
    pub fn set_active(&mut self, view: &ViewId, cameras: BTreeSet<CameraId>) -> Result<(), Error> {
        if !self.state.views.contains_key(view) {
            return Err(err(ErrorCode::MissingView, "view absent"));
        }
        if cameras.len() > 32 {
            return Err(err(ErrorCode::BudgetExceeded, "active camera set budget"));
        }
        for id in &cameras {
            let c = self
                .state
                .cameras
                .get(id)
                .ok_or_else(|| err(ErrorCode::MissingCamera, "active camera absent"))?;
            if c.config.view != *view {
                return Err(err(
                    ErrorCode::InvalidInput,
                    "active camera belongs to different view",
                ));
            }
        }
        let mut state = self.state.clone();
        for (id, c) in &mut state.cameras {
            if c.config.view == *view {
                c.active = cameras.contains(id);
            }
        }
        encode(&state)?;
        self.state = state;
        Ok(())
    }
    fn selections(
        &self,
        read: &ReadFrame,
    ) -> Result<BTreeMap<ViewId, PresentationSelection>, Error> {
        preflight_read(read)?;
        let mut selections = BTreeMap::new();
        for (id, view) in &self.state.views {
            let plugin = self
                .plugins
                .get(id)
                .ok_or_else(|| err(ErrorCode::PluginFailed, "installed extractor absent"))?;
            // Mutable hidden descriptors violate the pure linked-plugin contract.
            if plugin.descriptor() != view.descriptor {
                return Err(err(ErrorCode::PluginFailed, "format descriptor changed"));
            }
            let mut selected = plugin.extract(read, &view.config.binding, &view.config.scene)?;
            normalize_selection(&mut selected)?;
            validate_selection(read, &selected, &view.config.scene)?;
            selections.insert(id.clone(), selected);
        }
        Ok(selections)
    }
    pub fn update(&mut self, read: &ReadFrame, input: ViewInput) -> Result<RenderFrame, Error> {
        let next = self
            .state
            .tick
            .checked_add(1)
            .ok_or_else(|| err(ErrorCode::Overflow, "presentation tick overflow"))?;
        if input.target_tick != next {
            return Err(err(
                ErrorCode::InputOutOfSequence,
                "target tick must be next presentation tick",
            ));
        }
        if input.changes.len() > 32 {
            return Err(err(ErrorCode::BudgetExceeded, "camera change budget"));
        }
        let mut changes = BTreeMap::new();
        for change in input.changes {
            if !self.state.cameras.contains_key(&change.camera) {
                return Err(err(ErrorCode::MissingCamera, "changed camera absent"));
            }
            if changes.contains_key(&change.camera) {
                return Err(err(ErrorCode::InvalidInput, "duplicate camera change"));
            }
            let target = normalize_target(change.target)?;
            let camera = &self.state.cameras[&change.camera];
            validate_zoom(&target, &camera.config.behaviors)?;
            let mode = change
                .transition
                .unwrap_or(camera.config.default_transition);
            validate_transition_mode(mode)?;
            changes.insert(change.camera, (target, mode));
        }
        let selections = self.selections(read)?;
        let mut staged = self.state.clone();
        for (id, camera) in &mut staged.cameras {
            if let Some((target, mode)) = changes.remove(id) {
                match mode {
                    TransitionMode::Instant => {
                        camera.base = target;
                        camera.transition = None;
                    }
                    TransitionMode::Smooth { duration_ticks } => {
                        camera.transition = Some(Transition {
                            start: camera.base.clone(),
                            target,
                            duration: duration_ticks,
                            elapsed: 0,
                        });
                    }
                }
            }
            let selection = &selections[&camera.config.view];
            validate_follow(camera, selection)?;
            if camera.active {
                if let Some(transition) = &mut camera.transition {
                    transition.elapsed = transition
                        .elapsed
                        .checked_add(1)
                        .ok_or_else(|| err(ErrorCode::Overflow, "transition elapsed overflow"))?;
                    camera.base = interpolate(
                        &transition.start,
                        &transition.target,
                        transition.elapsed,
                        transition.duration,
                    )?;
                    if transition.elapsed == transition.duration {
                        camera.transition = None;
                    }
                }
                advance_follow(camera, selection)?;
                if let Some(shake) = &camera.config.behaviors.shake {
                    camera.shake_count = camera
                        .shake_count
                        .saturating_add(1)
                        .min(shake.duration_ticks + 1);
                }
            }
        }
        staged.tick = next;
        let frame = frame_state(&staged, &selections, read.tick)?;
        encode(&staged)?;
        self.state = staged;
        Ok(frame)
    }
    pub fn frame(&self, read: &ReadFrame) -> Result<RenderFrame, Error> {
        let selected = self.selections(read)?;
        frame_state(&self.state, &selected, read.tick)
    }
    pub fn describe(&self) -> ViewDiscovery {
        let installed_views = self
            .state
            .views
            .iter()
            .map(|(id, v)| ViewStatus {
                config: v.config.clone(),
                format: v.descriptor.clone(),
                cameras: self
                    .state
                    .cameras
                    .iter()
                    .filter(|(_, c)| c.config.view == *id)
                    .map(|(id, _)| id.clone())
                    .collect(),
            })
            .collect();
        let cameras = self
            .state
            .cameras
            .iter()
            .map(|(id, c)| CameraStatus {
                id: id.clone(),
                view: c.config.view.clone(),
                active: c.active,
                current: c.base.clone(),
                transitioning: c.transition.is_some(),
            })
            .collect();
        ViewDiscovery {
            contract_version: 1,
            tick: self.state.tick,
            available_formats: builtins(),
            installed_views,
            cameras,
            projections: ["orthographic", "perspective", "blended"]
                .map(String::from)
                .into(),
            behaviors: [
                "static",
                "follow",
                "axes",
                "bounds",
                "dead-zone",
                "look-ahead",
                "tracking-smoothing",
                "zoom",
                "shake",
                "smooth-transition",
            ]
            .map(String::from)
            .into(),
            limits: ViewLimits::default(),
        }
    }
    pub fn save(&self) -> Result<Vec<u8>, Error> {
        encode(&self.state)
    }
    pub fn hash(&self) -> Result<String, Error> {
        Ok(format!("{:x}", Sha256::digest(self.save()?)))
    }
    pub fn restore(&mut self, bytes: &[u8], read: &ReadFrame) -> Result<(), Error> {
        if bytes.len() > ViewLimits::default().max_save_bytes {
            return Err(err(ErrorCode::BudgetExceeded, "save byte budget"));
        }
        let staged: State = serde_json::from_slice(bytes)
            .map_err(|_| err(ErrorCode::InvalidSave, "malformed presentation save"))?;
        if staged.format_version != 1 || staged.contract_version != 1 {
            return Err(err(
                ErrorCode::VersionMismatch,
                "presentation save version mismatch",
            ));
        }
        if encode(&staged)?.as_slice() != bytes {
            return Err(err(
                ErrorCode::InvalidSave,
                "noncanonical presentation save",
            ));
        }
        if staged.views.len() > 8 || staged.cameras.len() > 32 {
            return Err(err(ErrorCode::BudgetExceeded, "save object count budget"));
        }
        for (id, view) in &staged.views {
            if id != &view.config.id {
                return Err(err(ErrorCode::InvalidSave, "saved view map key mismatch"));
            }
            validate_view_config(&view.config, &view.descriptor)
                .map_err(|e| Error::new(ErrorCode::InvalidSave, e.detail()))?;
        }
        for (id, camera) in &staged.cameras {
            if id != &camera.config.id {
                return Err(err(ErrorCode::InvalidSave, "saved camera map key mismatch"));
            }
            validate_camera_state(camera)
                .map_err(|e| Error::new(ErrorCode::InvalidSave, e.detail()))?;
            if u64::from(camera.shake_count) > staged.tick
                || (camera.follow_initialized && staged.tick == 0)
                || camera
                    .transition
                    .as_ref()
                    .is_some_and(|t| u64::from(t.elapsed) > staged.tick)
            {
                return Err(err(
                    ErrorCode::InvalidSave,
                    "behavior progress exceeds presentation tick",
                ));
            }
            if !staged.views.contains_key(&camera.config.view) {
                return Err(err(
                    ErrorCode::InvalidSave,
                    "saved camera references missing view",
                ));
            }
        }
        if staged.content_binding != self.state.content_binding
            || staged.views != self.state.views
            || staged.cameras.len() != self.state.cameras.len()
        {
            return Err(err(
                ErrorCode::SaveMismatch,
                "installed content/views/cameras differ",
            ));
        }
        for (id, camera) in &staged.cameras {
            if id != &camera.config.id
                || self
                    .state
                    .cameras
                    .get(id)
                    .is_none_or(|old| old.config != camera.config)
            {
                return Err(err(
                    ErrorCode::SaveMismatch,
                    "installed camera config differs",
                ));
            }
        }
        let selections = self.selections(read)?;
        for camera in staged.cameras.values() {
            validate_follow(camera, &selections[&camera.config.view])?;
        }
        frame_state(&staged, &selections, read.tick)?;
        self.state = staged;
        Ok(())
    }
}
fn validate_view_config(config: &ViewConfig, descriptor: &FormatDescriptor) -> Result<(), Error> {
    validate_descriptor(descriptor)?;
    validate_binding(&config.binding)?;
    validate_scene_ref(&config.scene)?;
    if config.policy.family != descriptor.family {
        return Err(err(
            ErrorCode::InvalidDescriptor,
            "view policy and format family mismatch",
        ));
    }
    let kind = match config.binding.representation {
        Representation::Sprite { .. } => "sprite",
        Representation::Model { .. } => "model",
        Representation::Background { .. } => "background",
        Representation::Billboard { .. } => "billboard",
    };
    if !descriptor.representations.iter().any(|s| s == kind) {
        return Err(err(
            ErrorCode::InvalidDescriptor,
            "format does not support requested representation",
        ));
    }
    Ok(())
}
fn normalize_selection(selection: &mut PresentationSelection) -> Result<(), Error> {
    if selection.items.len() > ViewLimits::default().max_items_per_view {
        return Err(err(ErrorCode::BudgetExceeded, "selected item budget"));
    }
    for item in &mut selection.items {
        clean_point(&mut item.world.0);
    }
    Ok(())
}
fn normalize_config(config: &mut CameraConfig) -> Result<(), Error> {
    config.target = normalize_target(config.target.clone())?;
    config.viewport.x = zero(config.viewport.x);
    config.viewport.y = zero(config.viewport.y);
    if let Some(f) = &mut config.behaviors.follow {
        clean_point(&mut f.offset);
        clean_point(&mut f.dead_zone);
    }
    if let Some(b) = &mut config.behaviors.bounds {
        clean_point(&mut b.min);
        clean_point(&mut b.max);
    }
    if let Some(s) = &mut config.behaviors.shake {
        clean_point(&mut s.amplitude);
    }
    Ok(())
}
fn validate_transition_mode(mode: TransitionMode) -> Result<(), Error> {
    if let TransitionMode::Smooth { duration_ticks } = mode
        && !(1..=4096).contains(&duration_ticks)
    {
        return Err(err(
            ErrorCode::InvalidInput,
            "transition duration outside1..4096",
        ));
    }
    Ok(())
}
fn validate_zoom(target: &CameraTarget, behaviors: &CameraBehaviors) -> Result<(), Error> {
    range(
        target.pose.zoom,
        behaviors.zoom_range[0],
        behaviors.zoom_range[1],
        ErrorCode::InvalidTransform,
    )
}
fn validate_camera_config(config: &CameraConfig) -> Result<(), Error> {
    validate_target(&config.target)?;
    validate_viewport(&config.viewport)?;
    validate_transition_mode(config.default_transition)?;
    let b = &config.behaviors;
    range(b.zoom_range[0], 0.001, 1000., ErrorCode::InvalidTransform)?;
    range(
        b.zoom_range[1],
        b.zoom_range[0],
        1000.,
        ErrorCode::InvalidTransform,
    )?;
    validate_zoom(&config.target, b)?;
    if let Some(f) = &b.follow {
        validate_entity_ref(&f.target)?;
        point(f.offset)?;
        for v in f.dead_zone {
            range(v, 0., 1e9, ErrorCode::InvalidTransform)?;
        }
        if f.look_ahead_ticks > 120 || f.smoothing_ticks > 4096 {
            return Err(err(ErrorCode::InvalidInput, "follow duration bounds"));
        }
    }
    if let Some(bounds) = &b.bounds {
        point(bounds.min)?;
        point(bounds.max)?;
        for i in 0..3 {
            if bounds.min[i] > bounds.max[i] {
                return Err(err(ErrorCode::InvalidTransform, "inverted camera bounds"));
            }
        }
    }
    if let Some(shake) = &b.shake {
        for v in shake.amplitude {
            range(v, 0., 1e9, ErrorCode::InvalidTransform)?;
        }
        if shake.duration_ticks > 4096 {
            return Err(err(ErrorCode::InvalidInput, "shake duration bound"));
        }
    }
    Ok(())
}
fn validate_camera_state(c: &CameraState) -> Result<(), Error> {
    validate_camera_config(&c.config)?;
    validate_target(&c.base)?;
    validate_zoom(&c.base, &c.config.behaviors)?;
    point(c.previous_entity)?;
    point(c.anchor)?;
    if c.config.behaviors.follow.is_none() && c.follow_initialized {
        return Err(err(ErrorCode::InvalidSave, "unexpected follow tracking"));
    }
    if !c.follow_initialized && (c.previous_entity != [0.; 3] || c.anchor != [0.; 3]) {
        return Err(err(
            ErrorCode::InvalidSave,
            "uninitialized tracking contains state",
        ));
    }
    match &c.config.behaviors.shake {
        Some(shake) if c.shake_count <= shake.duration_ticks + 1 => {}
        None if c.shake_count == 0 => {}
        _ => return Err(err(ErrorCode::InvalidSave, "shake count outside duration")),
    }
    if let Some(t) = &c.transition {
        if t.duration == 0 || t.duration > 4096 || t.elapsed >= t.duration {
            return Err(err(ErrorCode::InvalidSave, "transition progress invalid"));
        }
        validate_target(&t.start)?;
        validate_target(&t.target)?;
        validate_zoom(&t.start, &c.config.behaviors)?;
        validate_zoom(&t.target, &c.config.behaviors)?;
        let sample = interpolate(&t.start, &t.target, t.elapsed, t.duration)?;
        if sample != c.base {
            return Err(err(
                ErrorCode::InvalidSave,
                "saved sampled pose contradicts transition",
            ));
        }
    }
    // Saved input/state boundaries always use positive zero. Reject rather than rewrite.
    fn has_negative_zero(v: f64) -> bool {
        v == 0. && v.is_sign_negative()
    }
    let encoded = serde_json::to_value(c)
        .map_err(|_| err(ErrorCode::InvalidSave, "invalid camera numbers"))?;
    fn check(v: &serde_json::Value) -> bool {
        match v {
            serde_json::Value::Number(n) => n.as_f64().is_some_and(has_negative_zero),
            serde_json::Value::Array(a) => a.iter().any(check),
            serde_json::Value::Object(m) => m.values().any(check),
            _ => false,
        }
    }
    if check(&encoded) {
        return Err(err(ErrorCode::InvalidSave, "negative zero in saved state"));
    }
    Ok(())
}
fn validate_follow(c: &CameraState, s: &PresentationSelection) -> Result<(), Error> {
    if let Some(f) = &c.config.behaviors.follow
        && !s.items.iter().any(|i| i.entity == f.target)
    {
        return Err(err(
            ErrorCode::InvalidSelection,
            "follow target absent from scene selection",
        ));
    }
    Ok(())
}
fn advance_follow(c: &mut CameraState, s: &PresentationSelection) -> Result<(), Error> {
    let Some(f) = &c.config.behaviors.follow else {
        return Ok(());
    };
    let entity = s
        .items
        .iter()
        .find(|i| i.entity == f.target)
        .ok_or_else(|| err(ErrorCode::InvalidSelection, "follow target missing"))?
        .world
        .0;
    if !c.follow_initialized {
        c.anchor = entity;
        c.previous_entity = entity;
        c.follow_initialized = true;
    }
    let mut desired = c.anchor;
    for i in 0..3 {
        let delta = finite(entity[i] - c.anchor[i])?;
        if delta > f.dead_zone[i] {
            desired[i] = finite(entity[i] - f.dead_zone[i])?;
        } else if delta < -f.dead_zone[i] {
            desired[i] = finite(entity[i] + f.dead_zone[i])?;
        }
        let velocity = finite((entity[i] - c.previous_entity[i]) * f64::from(f.look_ahead_ticks))?;
        desired[i] = finite(desired[i] + velocity)?;
    }
    point(desired)?;
    if f.smoothing_ticks == 0 {
        c.anchor = desired;
    } else {
        for (i, value) in desired.into_iter().enumerate() {
            c.anchor[i] =
                finite(c.anchor[i] + (value - c.anchor[i]) / f64::from(f.smoothing_ticks))?;
        }
    }
    point(c.anchor)?;
    clean_point(&mut c.anchor);
    c.previous_entity = entity;
    clean_point(&mut c.previous_entity);
    Ok(())
}
/// SplitMix64 keyed noise: seed XOR first8bytes SHA256(CameraId) interpreted LE,
/// XOR active_count*0x9e3779b97f4a7c15 XOR axis*0xd1b54a32d192ed03.
/// SplitMix adds0x9e3779b97f4a7c15, xor>>30 *0xbf58476d1ce4e5b9,
/// xor>>27 *0x94d049bb133111eb, xor>>31; all arithmetic wraps u64.
/// The upper53bits map exactly to [0,1] via/(2^53-1), then to[-1,1].
fn shake_noise(seed: u64, id: &CameraId, count: u32, axis: u64) -> f64 {
    let digest = Sha256::digest(id.as_str().as_bytes());
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    let mut z = seed
        ^ u64::from_le_bytes(bytes)
        ^ u64::from(count).wrapping_mul(0x9e3779b97f4a7c15)
        ^ axis.wrapping_mul(0xd1b54a32d192ed03);
    z = z.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^= z >> 31;
    (z >> 11) as f64 / ((1u64 << 53) - 1) as f64 * 2. - 1.
}
fn effective_target(
    c: &CameraState,
    policy: &ViewPolicy,
    s: &PresentationSelection,
) -> Result<CameraTarget, Error> {
    let mut target = c.base.clone();
    if let Some(f) = &c.config.behaviors.follow {
        let mut anchor = if c.follow_initialized {
            c.anchor
        } else {
            s.items
                .iter()
                .find(|i| i.entity == f.target)
                .ok_or_else(|| err(ErrorCode::InvalidSelection, "follow target missing"))?
                .world
                .0
        };
        for (i, value) in anchor.iter_mut().enumerate() {
            *value = finite(*value + f.offset[i])?;
        }
        point(anchor)?;
        let displacement = world_to_view(WorldPoint(anchor), &target.view_transform)?.0;
        let axes = [
            policy.follow_axes.x,
            policy.follow_axes.y,
            policy.follow_axes.z,
        ];
        for i in 0..3 {
            if axes[i] {
                target.pose.position[i] = finite(target.pose.position[i] + displacement[i])?;
            }
        }
    }
    if let Some(shake) = &c.config.behaviors.shake
        && c.shake_count > 0
        && c.shake_count <= shake.duration_ticks
    {
        for i in 0..3 {
            target.pose.position[i] = finite(
                target.pose.position[i]
                    + shake_noise(shake.seed, &c.config.id, c.shake_count, i as u64)
                        * shake.amplitude[i],
            )?;
        }
    }
    if let Some(bounds) = &c.config.behaviors.bounds {
        for i in 0..3 {
            target.pose.position[i] = target.pose.position[i].clamp(bounds.min[i], bounds.max[i]);
        }
    }
    clean_point(&mut target.pose.position);
    validate_target(&target)?;
    Ok(target)
}
fn render_camera(
    c: &CameraState,
    v: &SavedView,
    s: &PresentationSelection,
    target: CameraTarget,
) -> Result<CameraFrame, Error> {
    let mut items = Vec::new();
    for item in &s.items {
        let screen = match project(item.world, &target, &c.config.viewport) {
            Ok(p) => p,
            Err(e) if e.code() == ErrorCode::OutsideDepth => continue,
            Err(e) => return Err(e),
        };
        let viewport = &c.config.viewport;
        if screen.x < viewport.x
            || screen.x > viewport.x + viewport.width
            || screen.y < viewport.y
            || screen.y > viewport.y + viewport.height
        {
            continue;
        }
        items.push(RenderItem {
            entity: item.entity.clone(),
            world: item.world,
            screen,
            representation: item.representation.clone(),
        });
    }
    Ok(CameraFrame {
        camera: c.config.id.clone(),
        view: c.config.view.clone(),
        scene: v.config.scene.clone(),
        viewport: c.config.viewport.clone(),
        target,
        items,
    })
}
fn frame_state(
    state: &State,
    selections: &BTreeMap<ViewId, PresentationSelection>,
    source_tick: u64,
) -> Result<RenderFrame, Error> {
    for camera in state.cameras.values() {
        validate_follow(camera, &selections[&camera.config.view])?;
    }
    let mut cameras = Vec::new();
    for c in state.cameras.values().filter(|c| c.active) {
        let view = &state.views[&c.config.view];
        let selected = &selections[&c.config.view];
        validate_follow(c, selected)?;
        cameras.push(render_camera(
            c,
            view,
            selected,
            effective_target(c, &view.config.policy, selected)?,
        )?);
    }
    Ok(RenderFrame {
        tick: state.tick,
        source_tick,
        cameras,
    })
}

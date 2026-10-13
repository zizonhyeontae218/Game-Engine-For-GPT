//! P4 native session boundary over public P2/P3 APIs. No FlatLand simulation runs here.
mod ffi;
mod render;
mod types;
pub use ffi::*;
pub use ge4g_pentomino_gameplay::{BattleState, FighterConfig, GameConfig};
pub use ge4g_pentomino_view::ViewFamily;
pub use types::*;

use ge4g_pentomino::{Bindings, p2};
use ge4g_pentomino_gameplay as gameplay;
use ge4g_pentomino_view as presentation;
use presentation::{CameraId, ViewHost, ViewId};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);
fn err(code: ErrorCode, detail: &str) -> BridgeError {
    BridgeError::new(code, detail)
}
fn version(value: u32) -> Result<()> {
    if value != CONTRACT_VERSION {
        return Err(err(
            ErrorCode::VersionMismatch,
            "P4 contract version must be 1",
        ));
    }
    Ok(())
}
fn next(value: u64) -> Result<u64> {
    value
        .checked_add(1)
        .ok_or_else(|| err(ErrorCode::Overflow, "tick overflow"))
}
fn core_tick(step: u64) -> Result<u64> {
    step.checked_mul(2)
        .ok_or_else(|| err(ErrorCode::Overflow, "Core tick overflow"))
}
fn view_id() -> ViewId {
    ViewId::new("p4.arena").expect("static ID")
}
fn camera_id() -> CameraId {
    CameraId::new("p4.camera").expect("static ID")
}
fn format(family: ViewFamily) -> Box<dyn presentation::FormatPlugin> {
    match family {
        ViewFamily::Classic2D => Box::new(presentation::Classic2DFormat),
        ViewFamily::TopDown => Box::new(presentation::TopDownFormat),
        ViewFamily::Side => Box::new(presentation::SideFormat),
        ViewFamily::Vertical => Box::new(presentation::VerticalFormat),
    }
}
fn view_config(read: &p2::ReadFrame, family: ViewFamily) -> Result<presentation::ViewConfig> {
    let policy = match family {
        ViewFamily::Classic2D => presentation::ViewPolicy::classic2d(),
        ViewFamily::TopDown => presentation::ViewPolicy::top_down(),
        ViewFamily::Side => presentation::ViewPolicy::side(),
        ViewFamily::Vertical => presentation::ViewPolicy::vertical(),
    };
    Ok(presentation::ViewConfig {
        id: view_id(),
        scene: gameplay::scene(read)?,
        policy,
        binding: presentation::RecordBinding {
            owner: gameplay::state_owner(),
            schema: gameplay::position_schema(),
            entity_field: "entity".into(),
            x_field: "x".into(),
            y_field: "y".into(),
            z_field: Some("z".into()),
            units_per_world: 1.,
            representation: presentation::Representation::Sprite {
                asset: "p4.fighter".into(),
                size: [16., 24.],
            },
        },
    })
}
fn camera_target(
    config: &GameConfig,
    request: &CameraRequest,
) -> Result<presentation::CameraTarget> {
    version(request.version)?;
    if !request.zoom.is_finite()
        || !(0.25..=4.).contains(&request.zoom)
        || !request.tilt.is_finite()
        || !(0.25..=2.).contains(&request.tilt)
        || !request.shear.is_finite()
        || !(-0.5..=0.5).contains(&request.shear)
        || request
            .origin
            .iter()
            .any(|v| !v.is_finite() || v.abs() > 1024.)
        || request.transition_ticks > 120
    {
        return Err(err(
            ErrorCode::InvalidInput,
            "camera outside supported CPU adapter bounds",
        ));
    }
    Ok(presentation::CameraTarget {
        pose: presentation::CameraPose {
            position: [0., 0., 100.],
            orientation: presentation::Quaternion::identity(),
            zoom: request.zoom,
        },
        view_transform: presentation::ViewTransform {
            origin: [
                f64::from(config.width) / 2. + request.origin[0],
                -f64::from(config.height) / 2. - request.origin[1],
                0.,
            ],
            rotation: presentation::Quaternion::identity(),
            scale: [1., request.tilt, 1.],
            shear_xy: -request.shear / (request.zoom * request.tilt),
        },
        projection: presentation::Projection::Orthographic {
            half_height: f64::from(config.height) / 2.,
            near: 0.1,
            far: 1000.,
            focus_distance: 100.,
        },
    })
}
fn make_view(
    config: &GameConfig,
    binding: &str,
    read: &p2::ReadFrame,
    family: ViewFamily,
) -> Result<ViewHost> {
    let mut view = ViewHost::new(binding)?;
    view.install(format(family), view_config(read, family)?, read)?;
    let camera = presentation::CameraConfig::new(
        camera_id(),
        view_id(),
        presentation::Viewport {
            x: 0.,
            y: 0.,
            width: f64::from(config.width),
            height: f64::from(config.height),
        },
        camera_target(config, &CameraRequest::default())?,
    );
    view.add_camera(camera, read)?;
    view.activate(&camera_id())?;
    Ok(view)
}

fn validate_boundary(
    core: &p2::CoreHost,
    read: &p2::ReadFrame,
    config: &GameConfig,
    step: u64,
) -> Result<()> {
    let battle = gameplay::battle_state(read, config)?;
    let count = u64::try_from(battle.attack_count)
        .map_err(|_| err(ErrorCode::InvalidSave, "negative attack count"))?;
    if read.tick != core_tick(step)? || count > step || read.records.len() != 3 {
        return Err(err(
            ErrorCode::InvalidSave,
            "authority differs from complete bridge step boundary",
        ));
    }
    let state_owner = gameplay::state_owner();
    let history = read
        .histories
        .get(&state_owner)
        .ok_or_else(|| err(ErrorCode::InvalidSave, "missing state history"))?;
    let status = read
        .records
        .get(&p2::RecordKey {
            owner: state_owner.clone(),
            local: "status".into(),
        })
        .ok_or_else(|| err(ErrorCode::InvalidSave, "missing state status"))?;
    if history.emitted != count
        || history
            .history
            .iter()
            .any(|e| e.kind != "applied" || e.tick % 2 != 0)
        || history.history.last().is_some_and(|e| &e.record != status)
    {
        return Err(err(
            ErrorCode::InvalidSave,
            "state history differs from committed attack count/phase",
        ));
    }
    let mut event_read = read.clone();
    let status_key = p2::RecordKey {
        owner: state_owner,
        local: "status".into(),
    };
    for (index, event) in history.history.iter().enumerate() {
        let ordinal = history
            .dropped
            .checked_add(index as u64)
            .and_then(|v| v.checked_add(1))
            .ok_or_else(|| err(ErrorCode::InvalidSave, "history count overflow"))?;
        if event.tick == 0
            || event.record.fields.get("attack_count") != Some(&p2::Value::I64(ordinal as i64))
        {
            return Err(err(
                ErrorCode::InvalidSave,
                "applied event attack order differs",
            ));
        }
        event_read
            .records
            .insert(status_key.clone(), event.record.clone());
        gameplay::battle_state(&event_read, config)?;
    }
    if let Some(rules) = read.histories.get(&gameplay::rules_owner())
        && (!rules.pending.is_empty()
            || rules.emitted > count
            || rules.history.iter().any(|decision| {
                decision.kind != "decision"
                    || decision.tick % 2 != 1
                    || !history.history.iter().any(|applied| {
                        decision.tick.checked_add(1) == Some(applied.tick)
                            && decision.record.fields == applied.record.fields
                    })
            }))
    {
        return Err(err(
            ErrorCode::InvalidSave,
            "unapplied or wrong-phase rules decision",
        ));
    }
    let applied_now = history.history.last().is_some_and(|e| e.tick == read.tick);
    if history.pending.len() != usize::from(applied_now) {
        return Err(err(
            ErrorCode::InvalidSave,
            "pending application differs from current phase",
        ));
    }
    if let Some(input) = core.describe().last_input
        && (input.target_tick != read.tick
            || input.actions.is_empty() == applied_now
            || (!input.actions.is_empty() && input.actions.len() != 5)
            || input.actions.iter().any(|(id, value)| {
                let Some(field) = id.as_str().strip_prefix("p4.state.") else {
                    return true;
                };
                match (status.fields.get(field), value) {
                    (Some(p2::Value::I64(expected)), p2::ActionValue::I64(actual)) => {
                        expected != actual
                    }
                    _ => true,
                }
            }))
    {
        return Err(err(
            ErrorCode::InvalidSave,
            "last input is not an applied phase-two batch",
        ));
    }
    Ok(())
}

struct Session {
    config: GameConfig,
    content_binding: String,
    seed: u64,
    mode: ExecutionMode,
    logical_step: u64,
    attached: bool,
    family: ViewFamily,
    core: p2::CoreHost,
    view: ViewHost,
    frame: Option<ge4g_render2d::Frame>,
}
impl Session {
    fn core(config: &GameConfig, binding: &str, seed: u64, attached: bool) -> Result<p2::CoreHost> {
        let mut core = p2::CoreHost::new(seed, binding)?;
        core.install(
            Box::new(gameplay::StateProvider::new(config.clone())?),
            Bindings::new(),
        )?;
        if attached {
            core.install(
                Box::new(gameplay::TurnCombat::new(config.clone())?),
                gameplay::bindings(),
            )?;
        }
        Ok(core)
    }
    fn read(&self) -> Result<p2::ReadFrame> {
        Ok(self.core.select(&gameplay::selection(self.attached))?)
    }
    fn new(config: GameConfig, seed: u64, mode: ExecutionMode) -> Result<Self> {
        config
            .validate()
            .map_err(|e| BridgeError::new(ErrorCode::InvalidConfig, e.to_string()))?;
        let bytes = serde_json::to_vec(&config)
            .map_err(|e| BridgeError::new(ErrorCode::InvalidConfig, e.to_string()))?;
        if bytes.len() > MAX_CONFIG_BYTES {
            return Err(err(ErrorCode::BudgetExceeded, "config too large"));
        }
        let binding = format!("p4:{:x}", Sha256::digest(bytes));
        let core = Self::core(&config, &binding, seed, true)?;
        let read = core.select(&gameplay::selection(true))?;
        let view = make_view(&config, &binding, &read, ViewFamily::Classic2D)?;
        let mut result = Self {
            config,
            content_binding: binding,
            seed,
            mode,
            logical_step: 0,
            attached: true,
            family: ViewFamily::Classic2D,
            core,
            view,
            frame: None,
        };
        result.refresh()?;
        Ok(result)
    }
    fn bundle(&self) -> Result<SaveBundle> {
        Ok(SaveBundle {
            version: 1,
            content_binding: self.content_binding.clone(),
            seed: self.seed,
            logical_step: self.logical_step,
            presentation_tick: self.view.describe().tick,
            gameplay_attached: self.attached,
            view_family: self.family,
            core_save2: String::from_utf8(self.core.save()?)
                .map_err(|_| err(ErrorCode::InternalFailure, "Core save not UTF-8"))?,
            presentation_save1: String::from_utf8(self.view.save()?)
                .map_err(|_| err(ErrorCode::InternalFailure, "View save not UTF-8"))?,
        })
    }
    fn restore_candidate(&self, save: &SaveBundle) -> Result<Self> {
        version(save.version)?;
        if save.content_binding != self.content_binding || save.seed != self.seed {
            return Err(err(ErrorCode::SaveMismatch, "seed/content binding differs"));
        }
        if save.core_save2.len() > 8_388_608 || save.presentation_save1.len() > 1_048_576 {
            return Err(err(
                ErrorCode::BudgetExceeded,
                "inner save size exceeds P2/P3 bounds",
            ));
        }
        let expected = core_tick(save.logical_step)?;
        let mut core = Self::core(
            &self.config,
            &self.content_binding,
            self.seed,
            save.gameplay_attached,
        )?;
        core.restore(save.core_save2.as_bytes())?;
        if core.describe().tick != expected || save.presentation_tick < save.logical_step {
            return Err(err(
                ErrorCode::InvalidSave,
                "saved clocks contradict complete two-phase boundary",
            ));
        }
        let read = core.select(&gameplay::selection(save.gameplay_attached))?;
        gameplay::battle_state(&read, &self.config)?;
        let mut view = make_view(&self.config, &self.content_binding, &read, save.view_family)?;
        view.restore(save.presentation_save1.as_bytes(), &read)?;
        render::validate_saved_targets(&self.config, save.presentation_save1.as_bytes())?;
        if view.describe().tick != save.presentation_tick {
            return Err(err(
                ErrorCode::InvalidSave,
                "presentation clock metadata differs",
            ));
        }
        let mut result = Self {
            config: self.config.clone(),
            content_binding: self.content_binding.clone(),
            seed: self.seed,
            mode: self.mode,
            logical_step: save.logical_step,
            attached: save.gameplay_attached,
            family: save.view_family,
            core,
            view,
            frame: None,
        };
        result.refresh()?;
        Ok(result)
    }
    fn candidate(&self) -> Result<Self> {
        self.restore_candidate(&self.bundle()?)
    }
    fn refresh(&mut self) -> Result<()> {
        let read = self.read()?;
        validate_boundary(&self.core, &read, &self.config, self.logical_step)?;
        let camera = self.view.frame(&read)?;
        if camera.cameras.len() != 1 {
            return Err(err(
                ErrorCode::ViewFailure,
                "exactly one active camera required",
            ));
        }
        // Validate renderer support in both modes; only linked mode retains pixels.
        render::validate_camera(&self.config, &read, &camera.cameras[0])?;
        self.frame = if self.mode == ExecutionMode::Linked {
            Some(render::render(&self.config, &read, &camera.cameras[0])?)
        } else {
            None
        };
        Ok(())
    }
    fn observation(&self) -> Result<Observation> {
        let read = self.read()?;
        let battle = gameplay::battle_state(&read, &self.config)?;
        let frame = self.view.frame(&read)?;
        Ok(Observation {
            version: 1,
            logical_step: self.logical_step,
            core_tick: read.tick,
            presentation_tick: frame.tick,
            gameplay_attached: self.attached,
            battle,
            records: read
                .records
                .into_iter()
                .map(|(key, record)| RecordRow {
                    owner: key.owner,
                    local: key.local,
                    record,
                })
                .collect(),
            scenes: read.scenes,
            entities: read.entities,
            histories: read
                .histories
                .into_iter()
                .map(|(owner, history)| HistoryRow { owner, history })
                .collect(),
            camera_frames: frame.cameras,
        })
    }
}

/// Rust native API. Each mutating multi-domain call stages a complete candidate.
#[derive(Default)]
pub struct NativeBridge {
    sessions: BTreeMap<u64, Session>,
}
impl NativeBridge {
    pub fn new() -> Self {
        Self::default()
    }
    fn session(&self, handle: SessionHandle) -> Result<&Session> {
        self.sessions.get(&handle.0).ok_or_else(|| {
            err(
                ErrorCode::InvalidSession,
                "closed or foreign session handle",
            )
        })
    }
    pub fn create(
        &mut self,
        config_json: &[u8],
        seed: u64,
        mode: ExecutionMode,
    ) -> Result<SessionHandle> {
        if config_json.len() > MAX_CONFIG_BYTES {
            return Err(err(ErrorCode::BudgetExceeded, "config too large"));
        }
        let config = serde_json::from_slice(config_json)
            .map_err(|e| BridgeError::new(ErrorCode::InvalidConfig, e.to_string()))?;
        self.create_config(config, seed, mode)
    }
    fn create_config(
        &mut self,
        config: GameConfig,
        seed: u64,
        mode: ExecutionMode,
    ) -> Result<SessionHandle> {
        if self.sessions.len() >= MAX_SESSIONS {
            return Err(err(ErrorCode::BudgetExceeded, "session limit 16"));
        }
        let session = Session::new(config, seed, mode)?;
        let handle = NEXT_HANDLE
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
            .map_err(|_| err(ErrorCode::Overflow, "session handle exhaustion"))?;
        self.sessions.insert(handle, session);
        Ok(SessionHandle(handle))
    }
    pub fn describe(&self, handle: SessionHandle) -> Result<SessionInfo> {
        let s = self.session(handle)?;
        Ok(SessionInfo {
            version: 1,
            mode: s.mode,
            content_binding: s.content_binding.clone(),
            logical_step: s.logical_step,
            gameplay_attached: s.attached,
            core: s.core.describe(),
            view: s.view.describe(),
        })
    }
    pub fn observe(&self, handle: SessionHandle) -> Result<Observation> {
        self.session(handle)?.observation()
    }
    pub fn frame(&self, handle: SessionHandle) -> Result<FrameDto> {
        let s = self.session(handle)?;
        let frame = s.frame.as_ref().ok_or_else(|| {
            err(
                ErrorCode::Unsupported,
                "headless session has no raster frame",
            )
        })?;
        Ok(FrameDto {
            version: 1,
            core_tick: s.core.describe().tick,
            presentation_tick: s.view.describe().tick,
            width: frame.width,
            height: frame.height,
            rgba: frame.rgba.clone(),
            sha256: format!("{:x}", Sha256::digest(&frame.rgba)),
        })
    }
    pub fn save(&self, handle: SessionHandle) -> Result<SaveBundle> {
        self.session(handle)?.bundle()
    }
    pub fn restore(&mut self, handle: SessionHandle, save_json: &[u8]) -> Result<()> {
        if save_json.len() > MAX_REQUEST_BYTES {
            return Err(err(ErrorCode::BudgetExceeded, "save envelope too large"));
        }
        let save = serde_json::from_slice(save_json)
            .map_err(|e| BridgeError::new(ErrorCode::InvalidSave, e.to_string()))?;
        let candidate = self.session(handle)?.restore_candidate(&save)?;
        self.sessions.insert(handle.0, candidate);
        Ok(())
    }
    pub fn step(&mut self, handle: SessionHandle, input: InputRequest) -> Result<Observation> {
        let s = self.session(handle)?;
        version(input.version)?;
        if input.target_step != next(s.logical_step)? {
            return Err(err(
                ErrorCode::InputOutOfSequence,
                "target_step must be logical_step+1",
            ));
        }
        if input.controls.len() > 1 || input.controls.iter().any(|v| v != &s.config.attack_control)
        {
            return Err(err(
                ErrorCode::InvalidInput,
                "unknown or duplicate control; at most one attack",
            ));
        }
        let attack = !input.controls.is_empty();
        if attack && !s.attached {
            return Err(err(
                ErrorCode::GameplayDetached,
                "turn combat rules are removed",
            ));
        }
        if attack && gameplay::battle_state(&s.read()?, &s.config)?.winner != -1 {
            return Err(err(ErrorCode::InvalidInput, "battle has already finished"));
        }
        let mut candidate = s.candidate()?;
        let decision_tick = next(candidate.core.describe().tick)?;
        candidate.core.step(p2::InputFrame {
            target_tick: decision_tick,
            actions: if attack {
                vec![(gameplay::attack_action(), p2::ActionValue::Bool(true))]
            } else {
                Vec::new()
            },
        })?;
        let read = candidate.read()?;
        let actions = if candidate.attached {
            gameplay::decision_actions(&read, decision_tick, attack, &candidate.config)?
        } else {
            Vec::new()
        };
        candidate.core.step(p2::InputFrame {
            target_tick: next(decision_tick)?,
            actions,
        })?;
        candidate.logical_step = input.target_step;
        if candidate.core.describe().tick != core_tick(input.target_step)? {
            return Err(err(
                ErrorCode::InternalFailure,
                "fixed-tick protocol mismatch",
            ));
        }
        let read = candidate.read()?;
        candidate.view.update(
            &read,
            presentation::ViewInput {
                target_tick: next(candidate.view.describe().tick)?,
                changes: Vec::new(),
            },
        )?;
        candidate.refresh()?;
        let observation = candidate.observation()?;
        self.sessions.insert(handle.0, candidate);
        Ok(observation)
    }
    pub fn detach_gameplay(&mut self, handle: SessionHandle) -> Result<()> {
        self.gameplay(handle, false)
    }
    pub fn attach_gameplay(&mut self, handle: SessionHandle) -> Result<()> {
        self.gameplay(handle, true)
    }
    fn gameplay(&mut self, handle: SessionHandle, enabled: bool) -> Result<()> {
        let s = self.session(handle)?;
        if s.attached == enabled {
            return Err(err(
                ErrorCode::InvalidInput,
                "Gameplay already in requested lifecycle state",
            ));
        }
        let mut candidate = s.candidate()?;
        if enabled {
            candidate.core.install(
                Box::new(gameplay::TurnCombat::new(candidate.config.clone())?),
                gameplay::bindings(),
            )?;
        } else {
            let owner = candidate.core.owner(&gameplay::rules_owner())?;
            candidate.core.remove(owner)?;
        }
        candidate.attached = enabled;
        candidate.refresh()?;
        self.sessions.insert(handle.0, candidate);
        Ok(())
    }
    pub fn set_camera(&mut self, handle: SessionHandle, request: CameraRequest) -> Result<()> {
        let s = self.session(handle)?;
        let target = camera_target(&s.config, &request)?;
        let mut candidate = s.candidate()?;
        let read = candidate.read()?;
        candidate.view.update(
            &read,
            presentation::ViewInput {
                target_tick: next(candidate.view.describe().tick)?,
                changes: vec![presentation::CameraChange {
                    camera: camera_id(),
                    target,
                    transition: Some(if request.transition_ticks == 0 {
                        presentation::TransitionMode::Instant
                    } else {
                        presentation::TransitionMode::Smooth {
                            duration_ticks: request.transition_ticks,
                        }
                    }),
                }],
            },
        )?;
        candidate.refresh()?;
        self.sessions.insert(handle.0, candidate);
        Ok(())
    }
    pub fn set_view(&mut self, handle: SessionHandle, family: ViewFamily) -> Result<()> {
        let mut candidate = self.session(handle)?.candidate()?;
        let read = candidate.read()?;
        candidate.view.replace(
            &view_id(),
            format(family),
            view_config(&read, family)?,
            &read,
        )?;
        candidate.family = family;
        candidate.refresh()?;
        self.sessions.insert(handle.0, candidate);
        Ok(())
    }
    pub fn close(&mut self, handle: SessionHandle) -> Result<()> {
        self.session(handle)?;
        self.sessions.remove(&handle.0);
        Ok(())
    }
    pub fn request(&mut self, bytes: &[u8]) -> Response {
        let result = (|| -> Result<serde_json::Value> {
            if bytes.len() > MAX_REQUEST_BYTES {
                return Err(err(ErrorCode::BudgetExceeded, "request too large"));
            }
            let request: Request = serde_json::from_slice(bytes)
                .map_err(|e| BridgeError::new(ErrorCode::InvalidInput, e.to_string()))?;
            version(request.version)?;
            self.dispatch(request.command)
        })();
        match result {
            Ok(value) => Response {
                version: 1,
                ok: true,
                result: Some(value),
                error: None,
            },
            Err(error) => Response {
                version: 1,
                ok: false,
                result: None,
                error: Some(error),
            },
        }
    }
    fn dispatch(&mut self, command: Command) -> Result<serde_json::Value> {
        fn json(value: impl serde::Serialize) -> Result<serde_json::Value> {
            serde_json::to_value(value)
                .map_err(|e| BridgeError::new(ErrorCode::InternalFailure, e.to_string()))
        }
        match command {
            Command::Create { config, seed, mode } => {
                Ok(serde_json::json!({"session": self.create_config(config, seed, mode)?.id()}))
            }
            Command::Describe { session } => json(self.describe(SessionHandle(session))?),
            Command::Step { session, input } => json(self.step(SessionHandle(session), input)?),
            Command::Observe { session } => json(self.observe(SessionHandle(session))?),
            Command::Frame { session } => json(self.frame(SessionHandle(session))?),
            Command::Save { session } => json(self.save(SessionHandle(session))?),
            Command::Restore { session, save } => {
                let bytes = serde_json::to_vec(&save)
                    .map_err(|e| BridgeError::new(ErrorCode::InvalidSave, e.to_string()))?;
                self.restore(SessionHandle(session), &bytes)?;
                Ok(serde_json::Value::Null)
            }
            Command::Gameplay { session, enabled } => {
                self.gameplay(SessionHandle(session), enabled)?;
                Ok(serde_json::Value::Null)
            }
            Command::Camera { session, request } => {
                self.set_camera(SessionHandle(session), request)?;
                Ok(serde_json::Value::Null)
            }
            Command::View { session, family } => {
                self.set_view(SessionHandle(session), family)?;
                Ok(serde_json::Value::Null)
            }
            Command::Close { session } => {
                self.close(SessionHandle(session))?;
                Ok(serde_json::Value::Null)
            }
        }
    }
}

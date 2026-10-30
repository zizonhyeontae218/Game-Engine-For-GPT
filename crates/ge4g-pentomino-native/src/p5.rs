//! Additive P5 native sessions: public P2 authority and P3 presentation.
use crate::p5_render as render;
use crate::{
    BridgeError, CameraRequest, ErrorCode, ExecutionMode, FrameDto, HistoryRow, MAX_CONFIG_BYTES,
    MAX_REQUEST_BYTES, MAX_SESSIONS, RecordRow, Response, Result, SessionHandle, ViewFamily,
};
pub use gameplay::{BattleState, Command as ActionCommand, GameConfig};
use ge4g_pentomino::{Bindings, p2};
use ge4g_pentomino_gameplay::v2 as gameplay;
use ge4g_pentomino_view as presentation;
use presentation::{CameraId, ViewHost, ViewId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, sync::atomic::Ordering};

/// Includes lifecycle and camera/view operations, not only combat commands.
pub const MAX_TRACE: usize = 4096;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum TraceEntry {
    Act { command: ActionCommand },
    Gameplay { enabled: bool },
    Camera { request: CameraRequest },
    View { family: ViewFamily },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SaveBundle {
    pub version: u32,
    pub content_binding: String,
    pub seed: u64,
    pub logical_step: u64,
    pub presentation_tick: u64,
    pub gameplay_attached: bool,
    pub view_family: ViewFamily,
    pub core_save2: String,
    pub presentation_save1: String,
    pub trace: Vec<TraceEntry>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub version: u32,
    pub logical_step: u64,
    pub core_tick: u64,
    pub presentation_tick: u64,
    pub gameplay_attached: bool,
    pub battle: BattleState,
    pub records: Vec<RecordRow>,
    pub scenes: Vec<p2::SceneData>,
    pub entities: Vec<p2::EntityData>,
    pub histories: Vec<HistoryRow>,
    pub camera_frames: Vec<presentation::CameraFrame>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionInfo {
    pub version: u32,
    pub mode: ExecutionMode,
    pub content_binding: String,
    pub logical_step: u64,
    pub gameplay_attached: bool,
    pub core: p2::CoreDiscovery,
    pub view: presentation::ViewDiscovery,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Command {
    Create {
        config: GameConfig,
        seed: u64,
        mode: ExecutionMode,
    },
    Describe {
        session: u64,
    },
    Act {
        session: u64,
        command: ActionCommand,
    },
    Observe {
        session: u64,
    },
    Frame {
        session: u64,
    },
    Save {
        session: u64,
    },
    Restore {
        session: u64,
        save: SaveBundle,
    },
    Gameplay {
        session: u64,
        enabled: bool,
    },
    Camera {
        session: u64,
        request: CameraRequest,
    },
    View {
        session: u64,
        family: ViewFamily,
    },
    Close {
        session: u64,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub version: u32,
    pub command: Command,
}

fn err(code: ErrorCode, detail: &str) -> BridgeError {
    BridgeError::new(code, detail)
}
fn view_id() -> ViewId {
    ViewId::new("p5.arena").expect("static ID")
}
fn camera_id() -> CameraId {
    CameraId::new("p5.camera").expect("static ID")
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
                asset: "p5.fighter".into(),
                size: [16., 24.],
            },
        },
    })
}
fn camera_target(
    config: &GameConfig,
    request: &CameraRequest,
) -> Result<presentation::CameraTarget> {
    crate::version(request.version)?;
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
            "camera outside supported CPU bounds",
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
    view.install(crate::format(family), view_config(read, family)?, read)?;
    view.add_camera(
        presentation::CameraConfig::new(
            camera_id(),
            view_id(),
            presentation::Viewport {
                x: 0.,
                y: 0.,
                width: f64::from(config.width),
                height: f64::from(config.height),
            },
            camera_target(config, &CameraRequest::default())?,
        ),
        read,
    )?;
    view.activate(&camera_id())?;
    Ok(view)
}

struct Session {
    config: GameConfig,
    binding: String,
    seed: u64,
    mode: ExecutionMode,
    logical_step: u64,
    attached: bool,
    family: ViewFamily,
    core: p2::CoreHost,
    view: ViewHost,
    frame: Option<ge4g_render2d::Frame>,
    trace: Vec<TraceEntry>,
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
            return Err(err(ErrorCode::BudgetExceeded, "config exceeds 65536 bytes"));
        }
        let binding = format!("p5:{:x}", Sha256::digest(&bytes));
        let core = Self::core(&config, &binding, seed, true)?;
        let read = core.select(&gameplay::selection(true))?;
        let view = make_view(&config, &binding, &read, ViewFamily::Classic2D)?;
        let mut s = Self {
            config,
            binding,
            seed,
            mode,
            logical_step: 0,
            attached: true,
            family: ViewFamily::Classic2D,
            core,
            view,
            frame: None,
            trace: Vec::new(),
        };
        s.refresh()?;
        Ok(s)
    }
    fn bundle(&self) -> Result<SaveBundle> {
        Ok(SaveBundle {
            version: 2,
            content_binding: self.binding.clone(),
            seed: self.seed,
            logical_step: self.logical_step,
            presentation_tick: self.view.describe().tick,
            gameplay_attached: self.attached,
            view_family: self.family,
            core_save2: String::from_utf8(self.core.save()?)
                .map_err(|_| err(ErrorCode::InternalFailure, "Core UTF8"))?,
            presentation_save1: String::from_utf8(self.view.save()?)
                .map_err(|_| err(ErrorCode::InternalFailure, "View UTF8"))?,
            trace: self.trace.clone(),
        })
    }
    /// Clone only a known-live boundary; public restore additionally replays its trace.
    fn candidate(&self) -> Result<Self> {
        let mut core = Self::core(&self.config, &self.binding, self.seed, self.attached)?;
        core.restore(&self.core.save()?)?;
        let read = core.select(&gameplay::selection(self.attached))?;
        let mut view = make_view(&self.config, &self.binding, &read, self.family)?;
        view.restore(&self.view.save()?, &read)?;
        Ok(Self {
            config: self.config.clone(),
            binding: self.binding.clone(),
            seed: self.seed,
            mode: self.mode,
            logical_step: self.logical_step,
            attached: self.attached,
            family: self.family,
            core,
            view,
            frame: None,
            trace: self.trace.clone(),
        })
    }
    fn refresh(&mut self) -> Result<()> {
        let read = self.read()?;
        if read.tick != crate::core_tick(self.logical_step)? {
            return Err(err(ErrorCode::InvalidSave, "incomplete command phase"));
        }
        gameplay::battle_state(&read, &self.config)?;
        let camera = self.view.frame(&read)?;
        if camera.cameras.len() != 1 {
            return Err(err(ErrorCode::ViewFailure, "expected one active camera"));
        }
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
        let camera = self.view.frame(&read)?;
        Ok(Observation {
            version: 1,
            logical_step: self.logical_step,
            core_tick: read.tick,
            presentation_tick: camera.tick,
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
            camera_frames: camera.cameras,
        })
    }
    fn apply(&mut self, entry: &TraceEntry) -> Result<()> {
        if self.trace.len() >= MAX_TRACE {
            return Err(err(ErrorCode::BudgetExceeded, "trace limit 4096"));
        }
        match entry {
            TraceEntry::Act { command } => {
                if !self.attached {
                    return Err(err(ErrorCode::GameplayDetached, "turn rules removed"));
                }
                // Legality before tick one gives native callers a stable input error.
                let state = gameplay::battle_state(&self.read()?, &self.config)?;
                gameplay::decide(&state, command, &self.config)
                    .map_err(|e| BridgeError::new(ErrorCode::InvalidInput, e.to_string()))?;
                let tick = crate::next(self.core.describe().tick)?;
                self.core.step(p2::InputFrame {
                    target_tick: tick,
                    actions: gameplay::command_actions(command, &self.config)?,
                })?;
                let actions =
                    gameplay::decision_actions(&self.read()?, tick, command, &self.config)?;
                self.core.step(p2::InputFrame {
                    target_tick: crate::next(tick)?,
                    actions,
                })?;
                self.logical_step = crate::next(self.logical_step)?;
                let read = self.read()?;
                self.view.update(
                    &read,
                    presentation::ViewInput {
                        target_tick: crate::next(self.view.describe().tick)?,
                        changes: Vec::new(),
                    },
                )?;
            }
            TraceEntry::Gameplay { enabled } => {
                if self.attached == *enabled {
                    return Err(err(
                        ErrorCode::InvalidInput,
                        "already in requested lifecycle state",
                    ));
                }
                if *enabled {
                    self.core.install(
                        Box::new(gameplay::TurnCombat::new(self.config.clone())?),
                        gameplay::bindings(),
                    )?;
                } else {
                    self.core
                        .remove(self.core.owner(&gameplay::rules_owner())?)?;
                }
                self.attached = *enabled;
            }
            TraceEntry::Camera { request } => {
                let target = camera_target(&self.config, request)?;
                let read = self.read()?;
                self.view.update(
                    &read,
                    presentation::ViewInput {
                        target_tick: crate::next(self.view.describe().tick)?,
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
            }
            TraceEntry::View { family } => {
                let read = self.read()?;
                self.view.replace(
                    &view_id(),
                    crate::format(*family),
                    view_config(&read, *family)?,
                    &read,
                )?;
                self.family = *family;
            }
        }
        self.trace.push(entry.clone());
        Ok(())
    }
    fn restore_candidate(&self, save: &SaveBundle) -> Result<Self> {
        if save.version != 2 {
            return Err(err(ErrorCode::VersionMismatch, "P5 save version must be 2"));
        }
        if save.content_binding != self.binding || save.seed != self.seed {
            return Err(err(ErrorCode::SaveMismatch, "seed/content differs"));
        }
        if save.trace.len() > MAX_TRACE
            || save.core_save2.len() > 8 * 1024 * 1024
            || save.presentation_save1.len() > 1024 * 1024
        {
            return Err(err(
                ErrorCode::BudgetExceeded,
                "save or trace limit exceeded",
            ));
        }
        // Replay uses the actual plugins, not a duplicated battle implementation.
        // Rasterize only once; mode does not affect saved Core or presentation.
        let mut candidate = Self::new(self.config.clone(), self.seed, ExecutionMode::Headless)?;
        for entry in &save.trace {
            candidate
                .apply(entry)
                .map_err(|e| BridgeError::new(ErrorCode::InvalidSave, format!("trace: {e}")))?;
        }
        render::validate_saved_targets(&self.config, save.presentation_save1.as_bytes())?;
        if candidate.bundle()? != *save {
            return Err(err(
                ErrorCode::InvalidSave,
                "saved state/clocks/trace contradict canonical replay",
            ));
        }
        candidate.mode = self.mode;
        candidate.refresh()?;
        Ok(candidate)
    }
}

/// Independent P5 registry; P4 and P5 handles share a non-recycling allocator.
#[derive(Default)]
pub struct NativeBridge {
    sessions: BTreeMap<u64, Session>,
}
impl NativeBridge {
    pub fn new() -> Self {
        Self::default()
    }
    fn session(&self, handle: SessionHandle) -> Result<&Session> {
        self.sessions
            .get(&handle.id())
            .ok_or_else(|| err(ErrorCode::InvalidSession, "closed or foreign P5 handle"))
    }
    pub fn create(
        &mut self,
        bytes: &[u8],
        seed: u64,
        mode: ExecutionMode,
    ) -> Result<SessionHandle> {
        if bytes.len() > MAX_CONFIG_BYTES {
            return Err(err(ErrorCode::BudgetExceeded, "config exceeds 65536 bytes"));
        }
        let config = serde_json::from_slice(bytes)
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
        let id = crate::NEXT_HANDLE
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |v| v.checked_add(1))
            .map_err(|_| err(ErrorCode::Overflow, "session IDs exhausted"))?;
        self.sessions.insert(id, session);
        Ok(SessionHandle(id))
    }
    pub fn describe(&self, handle: SessionHandle) -> Result<SessionInfo> {
        let s = self.session(handle)?;
        Ok(SessionInfo {
            version: 1,
            mode: s.mode,
            content_binding: s.binding.clone(),
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
        let f = s
            .frame
            .as_ref()
            .ok_or_else(|| err(ErrorCode::Unsupported, "headless has no raster frame"))?;
        Ok(FrameDto {
            version: 1,
            core_tick: s.core.describe().tick,
            presentation_tick: s.view.describe().tick,
            width: f.width,
            height: f.height,
            rgba: f.rgba.clone(),
            sha256: format!("{:x}", Sha256::digest(&f.rgba)),
        })
    }
    pub fn save(&self, handle: SessionHandle) -> Result<SaveBundle> {
        self.session(handle)?.bundle()
    }
    pub fn restore(&mut self, handle: SessionHandle, bytes: &[u8]) -> Result<()> {
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err(err(
                ErrorCode::BudgetExceeded,
                "save exceeds request bounds",
            ));
        }
        let save = serde_json::from_slice(bytes)
            .map_err(|e| BridgeError::new(ErrorCode::InvalidSave, e.to_string()))?;
        let candidate = self.session(handle)?.restore_candidate(&save)?;
        self.sessions.insert(handle.id(), candidate);
        Ok(())
    }
    fn mutate(&mut self, handle: SessionHandle, entry: TraceEntry) -> Result<Observation> {
        let mut candidate = self.session(handle)?.candidate()?;
        candidate.apply(&entry)?;
        candidate.refresh()?;
        let result = candidate.observation()?;
        self.sessions.insert(handle.id(), candidate);
        Ok(result)
    }
    pub fn act(&mut self, handle: SessionHandle, command: ActionCommand) -> Result<Observation> {
        self.mutate(handle, TraceEntry::Act { command })
    }
    pub fn detach_gameplay(&mut self, handle: SessionHandle) -> Result<()> {
        self.mutate(handle, TraceEntry::Gameplay { enabled: false })
            .map(|_| ())
    }
    pub fn attach_gameplay(&mut self, handle: SessionHandle) -> Result<()> {
        self.mutate(handle, TraceEntry::Gameplay { enabled: true })
            .map(|_| ())
    }
    pub fn set_camera(&mut self, handle: SessionHandle, request: CameraRequest) -> Result<()> {
        self.mutate(handle, TraceEntry::Camera { request })
            .map(|_| ())
    }
    pub fn set_view(&mut self, handle: SessionHandle, family: ViewFamily) -> Result<()> {
        self.mutate(handle, TraceEntry::View { family }).map(|_| ())
    }
    pub fn close(&mut self, handle: SessionHandle) -> Result<()> {
        self.session(handle)?;
        self.sessions.remove(&handle.id());
        Ok(())
    }
    pub fn request(&mut self, bytes: &[u8]) -> Response {
        let result = (|| -> Result<serde_json::Value> {
            if bytes.len() > MAX_REQUEST_BYTES {
                return Err(err(ErrorCode::BudgetExceeded, "request too large"));
            }
            let request: Request = serde_json::from_slice(bytes)
                .map_err(|e| BridgeError::new(ErrorCode::InvalidInput, e.to_string()))?;
            crate::version(request.version)?;
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
        fn json(value: impl Serialize) -> Result<serde_json::Value> {
            serde_json::to_value(value)
                .map_err(|e| BridgeError::new(ErrorCode::InternalFailure, e.to_string()))
        }
        match command {
            Command::Create { config, seed, mode } => {
                Ok(serde_json::json!({"session":self.create_config(config,seed,mode)?.id()}))
            }
            Command::Describe { session } => json(self.describe(SessionHandle(session))?),
            Command::Act { session, command } => json(self.act(SessionHandle(session), command)?),
            Command::Observe { session } => json(self.observe(SessionHandle(session))?),
            Command::Frame { session } => json(self.frame(SessionHandle(session))?),
            Command::Save { session } => json(self.save(SessionHandle(session))?),
            Command::Restore { session, save } => {
                self.restore(
                    SessionHandle(session),
                    &serde_json::to_vec(&save)
                        .map_err(|e| BridgeError::new(ErrorCode::InvalidSave, e.to_string()))?,
                )?;
                Ok(serde_json::Value::Null)
            }
            Command::Gameplay { session, enabled } => {
                self.mutate(SessionHandle(session), TraceEntry::Gameplay { enabled })?;
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

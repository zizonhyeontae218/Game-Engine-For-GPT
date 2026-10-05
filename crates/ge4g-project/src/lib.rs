//! File authoring, version checks and validation before simulation.
pub mod flatland;
pub mod gameplay;
pub mod patch;
pub const CAPABILITIES: &[&str] = &[
    "combat",
    "inventory",
    "quests",
    "event_scenes",
    "turn_battle",
    "planes",
    "atlas",
    "music",
    "lua_rng",
    "step_walk",
    "battle_stage",
    "view_projection",
];
use ge4g_core::{Error, Input, Result, SCHEMA_VERSION, StateDefinition, StateStore};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Write,
    path::{Component, Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct WindowSpec {
    pub width: u32,
    pub height: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub name: String,
    pub start_scene: String,
    pub window: WindowSpec,
    pub scenes: BTreeMap<String, String>,
    #[serde(default)]
    pub state: BTreeMap<String, StateDefinition>,
    #[serde(default)]
    pub tests: Vec<ProjectTest>,
    #[serde(default)]
    pub features: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Sprite {
    #[serde(default = "white")]
    pub color: [u8; 4],
    #[serde(default)]
    pub texture: Option<String>,
    #[serde(default)]
    pub layer: i32,
}
fn white() -> [u8; 4] {
    [255; 4]
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Collider {
    pub blocking: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Player {
    pub speed: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub scene: String,
    pub spawn: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Interaction {
    pub range: u32,
    pub dialogue: String,
    #[serde(default)]
    pub set_state: BTreeMap<String, Value>,
    #[serde(default)]
    pub transition: Option<Transition>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    pub id: String,
    #[serde(default)]
    pub prefab: Option<String>,
    #[serde(default)]
    pub position: [i64; 2],
    #[serde(default = "default_size")]
    pub size: [u32; 2],
    #[serde(default)]
    pub flatland: Option<flatland::Actor>,
    #[serde(default)]
    pub sprite: Option<Sprite>,
    #[serde(default)]
    pub collider: Option<Collider>,
    #[serde(default)]
    pub player: Option<Player>,
    #[serde(default)]
    pub trigger: Option<Transition>,
    #[serde(default)]
    pub interaction: Option<Interaction>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, Value>,
}
fn default_size() -> [u32; 2] {
    [16, 16]
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub schema_version: u32,
    pub id: String,
    pub background: [u8; 4],
    #[serde(default)]
    pub camera: [i64; 2],
    pub spawns: BTreeMap<String, [i64; 2]>,
    #[serde(default)]
    pub on_enter: BTreeMap<String, Value>,
    pub entities: Vec<Entity>,
    #[serde(default)]
    pub map: Option<flatland::Map>,
    #[serde(default)]
    pub prefabs: BTreeMap<String, Value>,
    #[serde(default)]
    pub rules: Vec<flatland::Rule>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default)]
    pub sounds: BTreeMap<String, String>,
    #[serde(default)]
    pub resources: Vec<String>,
    #[serde(default)]
    pub gameplay: gameplay::Gameplay,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct InputSpan {
    pub start: u64,
    pub end: u64,
    pub actions: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Replay {
    pub schema_version: u32,
    pub ticks: u64,
    pub inputs: Vec<InputSpan>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<ReplayCommand>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum ReplayCommand {
    Choice {
        tick: u64,
        id: String,
    },
    Do {
        tick: u64,
        actions: Vec<flatland::Action>,
    },
    Skip {
        tick: u64,
    },
}
impl ReplayCommand {
    pub fn tick(&self) -> u64 {
        match self {
            Self::Choice { tick, .. } | Self::Do { tick, .. } | Self::Skip { tick } => *tick,
        }
    }
}
impl Replay {
    pub fn actions_at(&self, tick: u64) -> BTreeSet<String> {
        self.inputs
            .iter()
            .filter(|s| tick >= s.start && tick < s.end)
            .flat_map(|s| s.actions.iter())
            .filter(|a| !["left", "right", "up", "down", "interact"].contains(&a.as_str()))
            .cloned()
            .collect()
    }

    pub fn load(path: &Path) -> Result<Self> {
        let replay: Self = serde_json::from_str(&read_text(path)?)
            .map_err(|e| Error(format!("replay {}: {e}", path.display())))?;
        replay
            .validate()
            .map_err(|e| Error(format!("replay {}: {e}", path.display())))?;
        Ok(replay)
    }
    pub fn validate(&self) -> Result<()> {
        version(self.schema_version, "replay")?;
        if self.ticks > 1_000_000 || self.inputs.len() > 100_000 {
            return Err(Error(
                "replay exceeds 1,000,000 ticks or 100,000 input spans".into(),
            ));
        }
        if self.commands.len() > 10000
            || (self.schema_version == 1 && !self.commands.is_empty())
            || self.commands.iter().any(|c| c.tick() >= self.ticks)
        {
            return Err(Error("invalid replay commands/tick/schema".into()));
        }
        for span in &self.inputs {
            if span.start >= span.end || span.end > self.ticks {
                return Err(Error(format!(
                    "input span {}..{} must lie within 0..{}",
                    span.start, span.end, self.ticks
                )));
            }
            for action in &span.actions {
                if !["left", "right", "up", "down", "interact"].contains(&action.as_str())
                    && (self.schema_version == 1 || action.len() > 64 || !valid_id(action))
                {
                    return Err(Error(format!("unknown replay action {action}")));
                }
            }
        }
        Ok(())
    }
    /// Input intervals are [start,end); overlapping intervals union their actions.
    pub fn input_at(&self, tick: u64) -> Input {
        let mut input = Input::default();
        for span in &self.inputs {
            if tick >= span.start && tick < span.end {
                for action in &span.actions {
                    match action.as_str() {
                        "left" => input.left = true,
                        "right" => input.right = true,
                        "up" => input.up = true,
                        "down" => input.down = true,
                        "interact" => input.interact = true,
                        _ => {}
                    }
                }
            }
        }
        input
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EventMatch {
    pub kind: String,
    #[serde(default)]
    pub entity: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EntityAssertion {
    pub id: String,
    #[serde(default)]
    pub position: Option<[i64; 2]>,
    #[serde(default)]
    pub max_x: Option<i64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Assertion {
    pub tick: u64,
    #[serde(default)]
    pub scene: Option<String>,
    #[serde(default)]
    pub entity: Option<EntityAssertion>,
    #[serde(default)]
    pub state: BTreeMap<String, Value>,
    #[serde(default)]
    pub event: Option<EventMatch>,
}
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectTest {
    pub name: String,
    pub replay: String,
    pub assertions: Vec<Assertion>,
    #[serde(default)]
    pub golden_rgba_sha256: Option<String>,
}
#[derive(Debug, Clone)]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}
#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub manifest: Manifest,
    pub scenes: BTreeMap<String, Scene>,
    pub textures: BTreeMap<String, Texture>,
    pub files_checked: Vec<String>,
    pub scripts: BTreeMap<String, String>,
}
#[derive(Debug, Serialize)]
pub struct ValidationReport {
    pub schema_version: u32,
    pub ok: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub files_checked: Vec<String>,
}
pub fn read_text(path: &Path) -> Result<String> {
    let meta = fs::metadata(path).map_err(|e| Error(format!("read {}: {e}", path.display())))?;
    if meta.len() > 16 * 1024 * 1024 {
        return Err(Error(format!(
            "{} exceeds the 16 MiB file limit",
            path.display()
        )));
    }
    fs::read_to_string(path).map_err(|e| Error(format!("read {}: {e}", path.display())))
}
pub fn version(actual: u32, kind: &str) -> Result<()> {
    if !(SCHEMA_VERSION..=2).contains(&actual) {
        return Err(Error(format!(
            "{kind}: unsupported schema_version {actual}; expected 1 or 2"
        )));
    }
    Ok(())
}
fn coordinate_ok(point: [i64; 2]) -> bool {
    point.iter().all(|v| (-1_000_000..=1_000_000).contains(v))
}
fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
}
impl Project {
    pub fn path(&self, relative: &str) -> Result<PathBuf> {
        safe_path(&self.root, relative)
    }
    pub fn load(path: &Path) -> Result<Self> {
        let manifest_path = if path.is_dir() {
            path.join("ge4g.toml")
        } else {
            path.to_path_buf()
        };
        let root = manifest_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."))
            .canonicalize()
            .map_err(|e| Error(format!("project {}: {e}", path.display())))?;
        let manifest: Manifest = toml::from_str(&read_text(&manifest_path)?)
            .map_err(|e| Error(format!("manifest {}: {e}", manifest_path.display())))?;
        version(manifest.schema_version, "manifest")?;
        if manifest.name.is_empty() || manifest.scenes.is_empty() || manifest.scenes.len() > 128 {
            return Err(Error("manifest needs a name and 1..128 scenes".into()));
        }
        if manifest.window.width == 0
            || manifest.window.height == 0
            || manifest.window.width > 2048
            || manifest.window.height > 2048
        {
            return Err(Error("window dimensions must be 1..2048".into()));
        }
        let mut project = Self {
            root,
            manifest,
            scenes: BTreeMap::new(),
            textures: BTreeMap::new(),
            files_checked: vec!["ge4g.toml".into()],
            scripts: BTreeMap::new(),
        };
        for (id, filename) in &project.manifest.scenes {
            let path = project.path(filename)?;
            let raw: Value = json5::from_str(&read_text(&path)?)
                .map_err(|e| Error(format!("scene {filename}: {e}")))?;
            let mut expanded = serde_json::json!({});
            if let Some(resources) = raw.get("resources").and_then(Value::as_array) {
                if resources.len() > 32 {
                    return Err(Error("scene resource limit 32".into()));
                }
                for resource in resources {
                    let file = resource
                        .as_str()
                        .ok_or_else(|| Error("resource needs a path".into()))?;
                    let value: Value = json5::from_str(&read_text(&project.path(file)?)?)
                        .map_err(|e| Error(format!("resource {file}: {e}")))?;
                    if value.as_object().is_none_or(|o| {
                        o.keys().any(|k| {
                            !["gameplay", "prefabs", "sounds", "rules"].contains(&k.as_str())
                        })
                    }) {
                        return Err(Error(format!(
                            "resource {file}: expected gameplay/prefabs/sounds/rules"
                        )));
                    }
                    flatland::merge(&mut expanded, value);
                    project.files_checked.push(file.into());
                }
            }
            flatland::merge(&mut expanded, raw);
            let scene: Scene = serde_json::from_value(
                flatland::expand_scene(expanded)
                    .map_err(|e| Error(format!("scene {filename}: {e}")))?,
            )
            .map_err(|e| Error(format!("scene {filename}: {e}")))?;
            if scene.schema_version != project.manifest.schema_version {
                return Err(Error(format!(
                    "scene {filename}: schema_version {} must match manifest {}",
                    scene.schema_version, project.manifest.schema_version
                )));
            }
            if !valid_id(id) || &scene.id != id {
                return Err(Error(format!(
                    "scene {filename}: id must equal manifest scene key {id}"
                )));
            }
            version(scene.schema_version, filename)?;
            project.files_checked.push(filename.clone());
            project.scenes.insert(id.clone(), scene);
        }
        let mut texture_paths: BTreeSet<String> = project
            .scenes
            .values()
            .flat_map(|s| &s.entities)
            .flat_map(|e| {
                let mut paths = Vec::new();
                if let Some(t) = e.sprite.as_ref().and_then(|s| s.texture.clone()) {
                    paths.push(t);
                }
                if let Some(a) = e.flatland.as_ref().and_then(|a| a.animation.as_ref()) {
                    paths.extend(a.frames.clone());
                    if let Some(atlas) = &a.atlas {
                        paths.push(atlas.file.clone());
                    }
                    paths.extend(a.alternate.clone());
                    paths.extend(a.directions.values().flatten().cloned());
                }
                paths
            })
            .collect();
        fn clips(value: &Value, paths: &mut BTreeSet<String>) {
            match value {
                Value::Object(o) => {
                    for (k, v) in o {
                        if k == "frames"
                            && let Some(a) = v.as_array()
                        {
                            for f in a {
                                if let Some(f) = f.as_str() {
                                    paths.insert(f.into());
                                }
                            }
                        }
                        if matches!(
                            k.as_str(),
                            "sprite" | "back_sprite" | "background" | "texture"
                        ) && let Some(file) = v.as_str()
                            && file.ends_with(".png")
                        {
                            paths.insert(file.to_owned());
                        }
                        clips(v, paths);
                    }
                }
                Value::Array(a) => {
                    for v in a {
                        clips(v, paths);
                    }
                }
                _ => {}
            }
        }
        for scene in project.scenes.values() {
            clips(
                &serde_json::to_value(scene).expect("scene JSON"),
                &mut texture_paths,
            );
        }
        for texture in texture_paths {
            let path = project.path(&texture)?;
            project
                .textures
                .insert(texture.clone(), load_texture(&path)?);
            project.files_checked.push(texture);
        }
        for scene in project.scenes.values() {
            for (name, file) in &scene.gameplay.modules {
                project.scripts.insert(
                    format!("module:{}:{name}", scene.id),
                    read_text(&project.path(file)?)?,
                );
                project.files_checked.push(file.clone());
            }
            if let Some(file) = &scene.script {
                project
                    .scripts
                    .insert(scene.id.clone(), read_text(&project.path(file)?)?);
                project.files_checked.push(file.clone());
            }
            for file in scene.sounds.values() {
                project.path(file)?;
                project.files_checked.push(file.clone());
            }
        }
        project.validate_world()?;
        for test in &project.manifest.tests {
            if !valid_id(&test.name) {
                return Err(Error(format!("invalid test name {}", test.name)));
            }
            let replay = Replay::load(&project.path(&test.replay)?)?;
            project.files_checked.push(test.replay.clone());
            if test.assertions.is_empty() {
                return Err(Error(format!(
                    "test {} needs behavioral assertions",
                    test.name
                )));
            }
            for assertion in &test.assertions {
                if assertion.entity.as_ref().is_some_and(|e| {
                    !valid_id(&e.id)
                        || e.position.is_some_and(|p| !coordinate_ok(p))
                        || e.max_x
                            .is_some_and(|x| !(-1_000_000..=1_000_000).contains(&x))
                }) {
                    return Err(Error(format!(
                        "test {}: invalid asserted entity id/coordinate",
                        test.name
                    )));
                }
                if assertion.tick > replay.ticks
                    || (assertion.scene.is_none()
                        && assertion.entity.is_none()
                        && assertion.state.is_empty()
                        && assertion.event.is_none())
                {
                    return Err(Error(format!(
                        "test {}: empty assertion or tick beyond replay",
                        test.name
                    )));
                }
                if assertion
                    .scene
                    .as_ref()
                    .is_some_and(|s| !project.scenes.contains_key(s))
                {
                    return Err(Error(format!("test {}: unknown asserted scene", test.name)));
                }
                project.validate_state_writes(&assertion.state, &format!("test {}", test.name))?;
            }
            if test
                .golden_rgba_sha256
                .as_ref()
                .is_some_and(|s| s.len() != 64 || !s.bytes().all(|c| c.is_ascii_hexdigit()))
            {
                return Err(Error(format!(
                    "test {}: golden hash must be SHA256 hex",
                    test.name
                )));
            }
        }
        project.files_checked.sort();
        project.files_checked.dedup();
        Ok(project)
    }
    pub fn validate(path: &Path) -> ValidationReport {
        match Self::load(path) {
            Ok(p) => ValidationReport {
                schema_version: 1,
                ok: true,
                errors: vec![],
                warnings: if p.manifest.tests.is_empty() {
                    vec!["project defines no replay tests".into()]
                } else {
                    vec![]
                },
                files_checked: p.files_checked,
            },
            Err(e) => ValidationReport {
                schema_version: 1,
                ok: false,
                errors: vec![e.to_string()],
                warnings: vec![],
                files_checked: vec![],
            },
        }
    }
    fn validate_state_writes(&self, writes: &BTreeMap<String, Value>, context: &str) -> Result<()> {
        let mut state = StateStore::new(self.manifest.state.clone())?;
        for (key, value) in writes {
            state
                .set(key, value.clone())
                .map_err(|e| Error(format!("{context}: {e}")))?;
        }
        Ok(())
    }
    fn validate_transition(&self, t: &Transition, context: &str) -> Result<()> {
        let target = self
            .scenes
            .get(&t.scene)
            .ok_or_else(|| Error(format!("{context}: unknown target scene {}", t.scene)))?;
        if !target.spawns.contains_key(&t.spawn) {
            return Err(Error(format!(
                "{context}: unknown spawn {} in {}",
                t.spawn, t.scene
            )));
        }
        Ok(())
    }
    fn validate_world(&self) -> Result<()> {
        if self.manifest.features.len() > 32
            || self
                .manifest
                .features
                .iter()
                .any(|f| !CAPABILITIES.contains(&f.as_str()))
            || (self.manifest.schema_version == 1 && !self.manifest.features.is_empty())
        {
            return Err(Error("unsupported required engine feature".into()));
        }
        StateStore::new(self.manifest.state.clone())?;
        if !self.scenes.contains_key(&self.manifest.start_scene) {
            return Err(Error(format!(
                "unknown start_scene {}",
                self.manifest.start_scene
            )));
        }
        for scene in self.scenes.values() {
            let context = format!("scene {}", scene.id);
            self.validate_flatland(scene)?;
            self.validate_state_writes(&scene.on_enter, &context)?;
            if !coordinate_ok(scene.camera)
                || scene.spawns.is_empty()
                || scene
                    .spawns
                    .iter()
                    .any(|(k, v)| !valid_id(k) || !coordinate_ok(*v))
            {
                return Err(Error(format!("{context}: invalid camera or spawns")));
            }
            if scene.entities.len() > 4096
                || scene.entities.iter().filter(|e| e.player.is_some()).count() != 1
            {
                return Err(Error(format!(
                    "{context}: expected exactly one player and at most 4096 entities"
                )));
            }
            let mut ids = BTreeSet::new();
            for entity in &scene.entities {
                let ctx = format!("{context}, entity {}", entity.id);
                if !valid_id(&entity.id) || !ids.insert(&entity.id) {
                    return Err(Error(format!("{ctx}: invalid or duplicate entity id")));
                }
                if !coordinate_ok(entity.position)
                    || entity.size.contains(&0)
                    || entity.size.iter().any(|&s| s > 4096)
                {
                    return Err(Error(format!("{ctx}: invalid position or size")));
                }
                if let Some(player) = &entity.player
                    && (!(1..=60_000).contains(&player.speed)
                        || (entity.collider.as_ref().is_none_or(|c| !c.blocking)
                            && entity.flatland.is_none()))
                {
                    return Err(Error(format!(
                        "{ctx}: player needs speed 1..60000 and a blocking collider"
                    )));
                }
                if let Some(t) = &entity.trigger {
                    if entity.collider.as_ref().is_some_and(|c| c.blocking) {
                        return Err(Error(format!(
                            "{ctx}: trigger cannot be a blocking collider"
                        )));
                    }
                    self.validate_transition(t, &ctx)?;
                }
                if let Some(i) = &entity.interaction {
                    if i.range > 4096 {
                        return Err(Error(format!("{ctx}: interaction range exceeds 4096")));
                    }
                    self.validate_state_writes(&i.set_state, &ctx)?;
                    if let Some(t) = &i.transition {
                        self.validate_transition(t, &ctx)?;
                    }
                }
            }
            let player = scene.entities.iter().find(|e| e.player.is_some()).unwrap();
            // Reject penetration at authoring time, including every transition spawn.
            for point in std::iter::once(player.position).chain(scene.spawns.values().copied()) {
                let a = ge4g_core::Aabb {
                    position: ge4g_core::Vec2::pixels(point[0], point[1]),
                    size: ge4g_core::Vec2::pixels(player.size[0].into(), player.size[1].into()),
                };
                for wall in scene.entities.iter().filter(|e| {
                    e.player.is_none() && e.collider.as_ref().is_some_and(|c| c.blocking)
                }) {
                    let b = ge4g_core::Aabb {
                        position: ge4g_core::Vec2::pixels(wall.position[0], wall.position[1]),
                        size: ge4g_core::Vec2::pixels(wall.size[0].into(), wall.size[1].into()),
                    };
                    if a.overlaps(b) {
                        return Err(Error(format!(
                            "{context}: player/spawn at {point:?} overlaps blocking entity {}",
                            wall.id
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}
fn safe_path(root: &Path, relative: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    if path.is_absolute()
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return Err(Error(format!(
            "project reference {relative} must be a relative path inside the project"
        )));
    }
    let resolved = root
        .join(path)
        .canonicalize()
        .map_err(|e| Error(format!("project reference {relative}: {e}")))?;
    if !resolved.starts_with(root) {
        return Err(Error(format!(
            "project reference {relative} resolves outside the project"
        )));
    }
    Ok(resolved)
}
fn load_texture(path: &Path) -> Result<Texture> {
    let file =
        fs::File::open(path).map_err(|e| Error(format!("texture {}: {e}", path.display())))?;
    let mut decoder = png::Decoder::new(file);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|e| Error(format!("texture {}: {e}", path.display())))?;
    if reader.info().width == 0
        || reader.info().height == 0
        || reader.info().width > 2048
        || reader.info().height > 2048
    {
        return Err(Error(format!(
            "texture {}: dimensions must be 1..2048",
            path.display()
        )));
    }
    let mut buffer = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|e| Error(format!("texture {}: {e}", path.display())))?;
    let bytes = &buffer[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgba => bytes.to_vec(),
        png::ColorType::Rgb => bytes
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::Grayscale => bytes.iter().flat_map(|&p| [p, p, p, 255]).collect(),
        png::ColorType::GrayscaleAlpha => bytes
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Indexed => {
            return Err(Error(format!(
                "texture {}: unexpanded indexed PNG",
                path.display()
            )));
        }
    };
    Ok(Texture {
        width: info.width,
        height: info.height,
        rgba,
    })
}

pub fn atomic_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut temp = tempfile::NamedTempFile::new_in(parent)
        .map_err(|e| Error(format!("write {}: {e}", path.display())))?;
    temp.write_all(bytes)
        .and_then(|()| temp.as_file().sync_all())
        .map_err(|e| Error(format!("write {}: {e}", path.display())))?;
    temp.persist(path)
        .map_err(|e| Error(format!("replace {}: {e}", path.display())))?;
    #[cfg(unix)]
    fs::File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(|e| Error(format!("sync directory {}: {e}", parent.display())))?;
    Ok(())
}
pub fn atomic_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|e| Error(format!("serialize {}: {e}", path.display())))?;
    bytes.push(b'\n');
    atomic_bytes(path, &bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replay_intervals_are_half_open_and_unknown_actions_fail() {
        let mut r = Replay {
            schema_version: 1,
            ticks: 3,
            commands: vec![],
            inputs: vec![InputSpan {
                start: 0,
                end: 2,
                actions: vec!["right".into()],
            }],
        };
        r.validate().unwrap();
        assert!(r.input_at(1).right);
        assert!(!r.input_at(2).right);
        r.inputs[0].actions.push("teleport".into());
        assert!(r.validate().is_err());
    }
    #[test]
    fn future_versions_are_rejected() {
        assert!(
            version(3, "scene")
                .unwrap_err()
                .to_string()
                .contains("schema_version 3")
        );
    }
}

//! Native CLI audio adapter. Device failures never alter authoritative simulation.
use ge4g_core::{Error, Result};
use ge4g_runtime::World;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::fs::File;
pub struct Audio {
    stream: Option<OutputStream>,
    voices: Vec<Sink>,
    music: Option<Sink>,
    music_key: Option<String>,
    cursor: u64,
    warned: bool,
}
impl Audio {
    pub fn new() -> Self {
        Self {
            stream: None,
            voices: Vec::new(),
            music: None,
            music_key: None,
            cursor: 0,
            warned: false,
        }
    }
    fn sink(&mut self) -> Result<Sink> {
        if self.stream.is_none() {
            let mut stream = OutputStreamBuilder::open_default_stream()
                .map_err(|e| Error(format!("audio device: {e}")))?;
            stream.log_on_drop(false);
            self.stream = Some(stream);
        }
        Ok(Sink::connect_new(self.stream.as_ref().unwrap().mixer()))
    }
    pub fn sync(&mut self, world: &World, paused: bool) {
        if let Err(e) = self.update(world, paused)
            && !self.warned
        {
            eprintln!("GE4G audio: {e}");
            self.warned = true;
        }
    }
    fn update(&mut self, world: &World, paused: bool) -> Result<()> {
        let snapshot = world.snapshot();
        let end = snapshot.events_dropped + snapshot.events.len() as u64;
        if end < self.cursor {
            self.cursor = 0;
            self.music_key = None;
        }
        if paused {
            for sink in &self.voices {
                sink.pause();
            }
            if let Some(s) = &self.music {
                s.pause();
            }
            self.cursor = end;
            return Ok(());
        }
        for sink in &self.voices {
            sink.play();
        }
        let music = world
            .flatland
            .systems
            .as_ref()
            .and_then(|s| s.music.as_ref());
        let key = music.map(|m| format!("{}:{}", m.file, m.volume));
        if key != self.music_key {
            if let Some(s) = self.music.take() {
                s.stop();
            }
            if let Some(m) = music {
                let file = File::open(world.project.path(&m.file)?)
                    .map_err(|e| Error(format!("music file: {e}")))?;
                let decoder =
                    Decoder::try_from(file).map_err(|e| Error(format!("music decode: {e}")))?;
                let sink = self.sink()?;
                sink.set_volume(m.volume as f32 / 100.);
                sink.append(decoder.repeat_infinite());
                self.music = Some(sink);
            }
            self.music_key = key;
        }
        if let Some(s) = &self.music {
            s.play();
        }
        for (i, e) in snapshot.events.iter().enumerate() {
            let id = snapshot.events_dropped + i as u64 + 1;
            if id <= self.cursor || e.kind != "audio" {
                continue;
            }
            if let Some(file) = e.data.get("file").and_then(|v| v.as_str()) {
                let data = File::open(world.project.path(file)?)
                    .map_err(|e| Error(format!("cue file: {e}")))?;
                let decoder =
                    Decoder::try_from(data).map_err(|e| Error(format!("cue decode: {e}")))?;
                self.voices.retain(|s| !s.empty());
                if self.voices.len() >= 4 {
                    self.voices.remove(0).stop();
                }
                let sink = self.sink()?;
                sink.set_volume(
                    e.data.get("volume").and_then(|v| v.as_f64()).unwrap_or(40.) as f32 / 100.,
                );
                sink.append(decoder);
                self.voices.push(sink);
            }
        }
        self.cursor = end;
        Ok(())
    }
}

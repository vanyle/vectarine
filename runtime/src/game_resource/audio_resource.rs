use std::{cell::RefCell, path::Path, rc::Rc};

use crate::game_resource::{Resource, ResourceId, Status};

pub static AUDIO_SAMPLE_FREQUENCY: i32 = 48000; // in Hz
pub static AUDIO_CHANNELS: i32 = 2; // Stereo
pub static BYTES_PER_SAMPLE: u32 = 2; // 16-bit audio

pub struct AudioResource {
    pub chunk: RefCell<Option<sdl2::mixer::Chunk>>,
    pub duration: RefCell<f32>,
    pub currently_used_channel: RefCell<Option<sdl2::mixer::Channel>>,
}

impl Resource for AudioResource {
    fn load_from_data(
        self: std::rc::Rc<Self>,
        _assigned_id: ResourceId,
        _dependency_reporter: &super::DependencyReporter,
        _lua: &Rc<mlua::Lua>,
        _gl: std::sync::Arc<glow::Context>,
        _path: &Path,
        data: Box<[u8]>,
    ) -> Status {
        let data_length = data.len();
        let result = sdl2::mixer::Chunk::from_raw_buffer(data);
        let chunk = match result {
            Ok(chunk) => chunk,
            Err(e) => return Status::Error(format!("Unable to load audio: {}", e)),
        };
        self.chunk.replace(Some(chunk));

        let duration_secs = data_length as f32
            / (AUDIO_SAMPLE_FREQUENCY as f32 * AUDIO_CHANNELS as f32 * BYTES_PER_SAMPLE as f32);
        self.duration.replace(duration_secs);

        Status::Loaded
    }

    fn draw_debug_gui(&self, ui: &mut egui::Ui) {
        ui.label("[TODO] Audio Resource");
        let c = self.currently_used_channel.borrow();
        let c = c.as_ref();
        let Some(c) = c else {
            ui.label("No channel allocated");
            return;
        };
        ui.label(format!("Using channel {:?}/8", c.0));
    }

    fn get_type_name(&self) -> &'static str {
        "Audio"
    }

    fn default() -> Self
    where
        Self: Sized,
    {
        Self {
            chunk: RefCell::new(None),
            currently_used_channel: RefCell::new(None),
            duration: RefCell::new(0.0),
        }
    }
}

pub fn get_group() -> sdl2::mixer::Group {
    sdl2::mixer::Group::default()
}

// Our audio implementation is backed by sdl2_mixer.
// Maybe I should switch to using SDL directly?
//
// This means that we can play at most 8 audio resources simultaneously (by default).
// We'll hide that fact in the API to keep things simple for the user.
// This means we need a global object which stores what channels are available / in use and which allocates them as needed.

impl AudioResource {
    /// Start playing the audio from the beginning.
    /// If `looped` is true, the audio will loop until paused.
    /// If `fade_in_ms` is provided, the audio will fade in over that duration in milliseconds.
    pub fn play(&self, looped: bool, fade_in_ms: Option<i32>) {
        let channel = self.get_channel();
        let Some(channel) = channel else {
            println!("No available audio channels to play sound.");
            return;
        };
        let loops = if looped { -1 } else { 0 };
        let chunk = self.chunk.borrow();
        let Some(chunk) = chunk.as_ref() else {
            println!("No audio chunk loaded to play.");
            return;
        };
        let result = if let Some(ms) = fade_in_ms {
            channel.fade_in(chunk, loops, ms)
        } else {
            channel.play(chunk, loops)
        };
        if let Err(e) = result {
            println!("Failed to play audio: {}", e);
        }
    }
    pub fn pause(&self) {
        let channel = self.currently_used_channel.borrow();
        let Some(channel) = channel.as_ref() else {
            return;
        };
        channel.pause();
    }
    pub fn resume(&self) {
        let channel = self.currently_used_channel.borrow();
        let Some(channel) = channel.as_ref() else {
            return;
        };
        channel.resume();
    }

    pub fn is_playing(&self) -> bool {
        let channel = self.currently_used_channel.borrow();
        let Some(channel) = channel.as_ref() else {
            return false;
        };
        channel.is_playing()
    }

    // Set the volume of the audio resource. Volume is a float between 0.0 and 1.0.
    pub fn set_volume(&self, volume: f32) -> Option<()> {
        let channel = self.currently_used_channel.borrow();
        let channel = channel.as_ref()?;
        let sdl_volume = (volume.clamp(0.0, 1.0) * sdl2::mixer::MAX_VOLUME as f32) as i32;
        channel.set_volume(sdl_volume);
        Some(())
    }

    // Get the volume of the audio resource. Volume is a float between 0.0 and 1.0.
    // If no audio is loaded, returns 0.0.
    pub fn get_volume(&self) -> f32 {
        let channel = self.currently_used_channel.borrow();
        let Some(channel) = channel.as_ref() else {
            return 0.0;
        };
        channel.get_volume() as f32 / sdl2::mixer::MAX_VOLUME as f32
    }

    pub fn current_position(&self) -> f32 {
        todo!(
            "AudioResource.current_position() is not implemented yet. When using SDL2_mixer, this is non-trivial to implement. Track the time yourself!"
        );
    }
    /// Get the duration of the audio in seconds.
    /// Returns 0.0 if no audio is loaded or if the audio failed to load.
    pub fn duration(&self) -> f32 {
        *self.duration.borrow()
    }

    fn get_channel(&self) -> Option<sdl2::mixer::Channel> {
        let channel = *self.currently_used_channel.borrow();
        if let Some(channel) = channel {
            return Some(channel);
        };

        let group = get_group();
        let channel = group.find_available();
        if let Some(channel) = &channel {
            self.currently_used_channel.borrow_mut().replace(*channel);
            return Some(*channel);
        }
        None
    }
}

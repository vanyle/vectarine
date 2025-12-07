use std::{cell::RefCell, path::Path, rc::Rc};

use symphonia::core::audio::SampleBuffer;
use symphonia::core::io::MediaSourceStream;

use crate::{
    game_resource::{Resource, ResourceId, Status},
    sound::{self, ChannelId},
};

pub static AUDIO_SAMPLE_FREQUENCY: i32 = 48000; // in Hz
pub static AUDIO_CHANNELS: i32 = 2; // Stereo
pub static BYTES_PER_SAMPLE: u32 = 2; // 16-bit audio

pub struct AudioResource {
    pub chunk: RefCell<Option<Box<[f32]>>>,
    pub duration: RefCell<f32>,
    pub currently_used_channel: RefCell<Option<ChannelId>>,
}

pub struct ReadableBytes {
    pub data: Box<[u8]>,
}

impl ReadableBytes {
    pub fn new(data: Box<[u8]>) -> Self {
        Self { data }
    }
}

impl std::io::Read for ReadableBytes {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = std::cmp::min(self.data.len(), buf.len());
        buf[..len].copy_from_slice(&self.data[..len]);
        self.data = self.data[len..].into();
        Ok(len)
    }
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

        // Decode audio
        let readable_data = ReadableBytes::new(data);
        let read_only_source = Box::new(symphonia::core::io::ReadOnlySource::new(readable_data));
        let mss = MediaSourceStream::new(read_only_source, Default::default());

        let hint = symphonia::core::probe::Hint::new();
        let format_opts: symphonia::core::formats::FormatOptions = Default::default();
        let metadata_opts: symphonia::core::meta::MetadataOptions = Default::default();
        let decoder_opts: symphonia::core::codecs::DecoderOptions = Default::default();
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .expect("Probe to work");
        let mut format = probed.format;
        let track = format.default_track().expect("No default track");
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .expect("Failed to create decoder");

        let mut result = Vec::new();
        loop {
            let maybe_packet = format.next_packet();
            let Ok(packet) = maybe_packet else {
                break;
            };

            let decoded = decoder.decode(&packet).expect("Failed to decode packet");

            let spec = *decoded.spec();
            let duration = decoded.capacity() as u64;
            let mut sample_buf = SampleBuffer::<f32>::new(duration, spec);
            sample_buf.copy_interleaved_ref(decoded);
            result.extend_from_slice(sample_buf.samples());
        }

        self.chunk.replace(Some(result.into_boxed_slice()));

        let duration_secs = data_length as f32
            / (AUDIO_SAMPLE_FREQUENCY as f32 * AUDIO_CHANNELS as f32 * BYTES_PER_SAMPLE as f32);
        self.duration.replace(duration_secs);

        if self.currently_used_channel.borrow().is_none() {
            self.currently_used_channel
                .borrow_mut()
                .replace(sound::get_available_channel());
        }

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
        ui.label(format!("Using channel {:?}", c));
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

impl AudioResource {
    /// Start playing the audio from the beginning.
    /// TODO: If `looped` is true, the audio will loop until paused.
    /// TODO: If `fade_in_ms` is provided, the audio will fade in over that duration in milliseconds.
    pub fn play(&self, looped: bool, fade_in_ms: Option<i32>) {
        let channel = self.get_channel();
        let Some(channel) = channel else {
            println!("No available audio channels to play sound.");
            return;
        };
        let chunk = self.chunk.borrow();
        let Some(chunk) = chunk.as_ref() else {
            println!("No audio chunk loaded to play.");
            return;
        };
        sound::resume_audio(channel);
        sound::add_sound_data_to_channel(
            channel,
            chunk,
            fade_in_ms.unwrap_or(100) as f32,
            100.0,
            looped,
        );
    }
    pub fn pause(&self) {
        let channel = self.currently_used_channel.borrow();
        let Some(channel) = channel.as_ref() else {
            return;
        };
        sound::pause_audio(*channel);
    }
    pub fn resume(&self) {
        let channel = self.currently_used_channel.borrow();
        let Some(channel) = channel.as_ref() else {
            return;
        };
        sound::resume_audio(*channel);
    }

    pub fn is_playing(&self) -> bool {
        let channel = self.currently_used_channel.borrow();
        let Some(channel) = channel.as_ref() else {
            return false;
        };
        sound::is_playing(*channel)
    }

    // Set the volume of the audio resource. Volume is a float between 0.0 and 1.0.
    pub fn set_volume(&self, volume: f32) -> Option<()> {
        let channel = self.currently_used_channel.borrow();
        let channel = channel.as_ref()?;
        sound::set_volume(*channel, volume);
        Some(())
    }

    // Get the volume of the audio resource. Volume is a float between 0.0 and 1.0.
    // If no audio is loaded, returns 0.0.
    pub fn get_volume(&self) -> f32 {
        let channel = self.currently_used_channel.borrow();
        let Some(channel) = channel.as_ref() else {
            return 0.0;
        };
        sound::get_volume(*channel)
    }

    pub fn current_position(&self) -> f32 {
        todo!("AudioResource.current_position() is not implemented yet");
    }
    /// Get the duration of the audio in seconds.
    /// Returns 0.0 if no audio is loaded or if the audio failed to load.
    pub fn duration(&self) -> f32 {
        *self.duration.borrow()
    }

    fn get_channel(&self) -> Option<ChannelId> {
        *self.currently_used_channel.borrow()
    }
}

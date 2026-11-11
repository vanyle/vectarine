use std::{cell::RefCell, path::Path, rc::Rc};

use rubato::{Fft, FixedSync, Resampler, audioadapter_buffers::direct::InterleavedSlice};
use symphonia::core::io::MediaSourceStream;

use crate::graphics::gfx::Gfx;
use crate::{
    game_resource::{Resource, ResourceId, Status},
    lua_env::LuaHandle,
    sound::{self, ChannelId},
};

/// Sounds are converted to this rate and to stereo when they load, so that they can be mixed together.
pub static AUDIO_SAMPLE_FREQUENCY: i32 = 48000; // in Hz

pub static AUDIO_CHANNELS: i32 = 2; // Stereo

pub struct AudioResource {
    // A chunk stored the full raw audio uncompressed which can cause memory issues.
    // An idea: Store a compressed lossless format instead (flac?)
    // and decompress the chunks of the flac blob on the fly as needed to
    // reduce memory use without degrading sound quality.
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
        _lua: &Rc<LuaHandle>,
        _gfx: Gfx,
        path: &Path,
        data: Box<[u8]>,
    ) -> Status {
        // Decode audio
        let readable_data = ReadableBytes::new(data);
        let read_only_source = Box::new(symphonia::core::io::ReadOnlySource::new(readable_data));
        let mss = MediaSourceStream::new(read_only_source, Default::default());

        let mut hint = symphonia::core::formats::probe::Hint::new();
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| hint.with_extension(ext));
        let format_opts: symphonia::core::formats::FormatOptions = Default::default();
        // we don't decode metadata because we don't care about them, and symphonia sometimes has issues decoding them.
        let zero_bytes = symphonia::core::common::Limit::Maximum(0);
        let metadata_opts: symphonia::core::meta::MetadataOptions =
            symphonia::core::meta::MetadataOptions::default()
                .limit_tag_bytes(zero_bytes)
                .limit_visual_bytes(zero_bytes);

        let mut probed =
            match symphonia::default::get_probe().probe(&hint, mss, format_opts, metadata_opts) {
                Ok(probed) => probed,
                Err(cause) => {
                    return Status::Error(format!("Unable to detect audio format: {}", cause));
                }
            };

        let decoder_opts: symphonia::core::codecs::audio::AudioDecoderOptions = Default::default();

        let track = match probed.default_track(symphonia::core::formats::TrackType::Audio) {
            Some(track) => track,
            None => return Status::Error("No default audio track found".to_string()),
        };

        let Some(audio_codec_params) = track.codec_params.as_ref().and_then(|o| o.audio()) else {
            return Status::Error("No audio codec parameters found".to_string());
        };

        let mut decoder = match symphonia::default::get_codecs()
            .make_audio_decoder(audio_codec_params, &decoder_opts)
        {
            Ok(decoder) => decoder,
            Err(cause) => {
                return Status::Error(format!("Unable to create audio decoder: {}", cause));
            }
        };

        let mut result = Vec::new();
        let mut sample_buffer = Vec::new();
        let mut channels = 0;

        let Some(sample_rate) = audio_codec_params.sample_rate else {
            return Status::Error("Unable to determine the sample rate".to_string());
        };

        loop {
            let maybe_packet = probed.next_packet();
            let Ok(Some(packet)) = maybe_packet else {
                break; // end-of-stream (either because of error, or because of EOF)
            };

            let Ok(audio_buf) = decoder.decode(&packet) else {
                continue; // skip this packet if it can't be decoded
            };

            channels = audio_buf.num_planes();
            sample_buffer.resize(audio_buf.samples_interleaved(), 0.0);
            audio_buf.copy_to_slice_interleaved(&mut sample_buffer);
            result.extend_from_slice(&sample_buffer);
        }

        let result = match to_output_format(&result, channels, sample_rate) {
            Ok(result) => result,
            Err(cause) => {
                return Status::Error(format!("Unable to resample the audio: {}", cause));
            }
        };
        let sample_count = result.len();
        self.chunk.replace(Some(result.into_boxed_slice()));

        let duration_secs =
            sample_count as f32 / (AUDIO_SAMPLE_FREQUENCY as f32 * AUDIO_CHANNELS as f32);
        self.duration.replace(duration_secs);

        if self.currently_used_channel.borrow().is_none() {
            self.currently_used_channel
                .borrow_mut()
                .replace(sound::get_available_channel());
        }

        Status::Loaded
    }

    fn draw_debug_gui(
        &self,
        _painter: &mut vectarine_plugin_sdk::egui_wgpu::Renderer,
        ui: &mut vectarine_plugin_sdk::egui::Ui,
    ) {
        let c = self.currently_used_channel.borrow();
        let c = c.as_ref();
        let Some(c) = c else {
            ui.label("No channel allocated");
            return;
        };
        ui.label(format!("Using channel {:?}", c));
        ui.label(format!("Is currently playing: {}", self.is_playing()));
        ui.label(format!(
            "Playback: {} / {} sec",
            self.current_position(),
            self.duration()
        ));
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

fn to_output_format(
    samples: &[f32],
    channels: usize,
    sample_rate: u32,
) -> Result<Vec<f32>, String> {
    let stereo: Vec<f32> = match channels {
        0 => Vec::new(),
        1 => samples
            .iter()
            .flat_map(|&sample| [sample, sample])
            .collect(),
        2 => samples.to_vec(),
        _ => samples
            .chunks_exact(channels)
            .flat_map(|frame| [frame[0], frame[1]])
            .collect(),
    };
    let output_rate = AUDIO_SAMPLE_FREQUENCY as u32;
    if sample_rate == output_rate || stereo.is_empty() {
        return Ok(stereo);
    }

    let output_channels = AUDIO_CHANNELS as usize;
    let frames = stereo.len() / output_channels;
    let mut resampler = Fft::<f32>::new(
        sample_rate as usize,
        output_rate as usize,
        1024,
        output_channels,
        FixedSync::Both,
    )
    .map_err(|e| e.to_string())?;
    let input =
        InterleavedSlice::new(&stereo, output_channels, frames).map_err(|e| e.to_string())?;
    let output = resampler
        .process_all(&input, frames, None)
        .map_err(|e| e.to_string())?;
    Ok(output.take_data())
}

/// The fade at the end of a sound that is not looped, in milliseconds
const END_FADE_MS: f32 = 10.0;

impl AudioResource {
    /// Start playing the audio from the beginning.
    pub fn play(&self, looped: bool, fade_in_ms: Option<i32>) {
        let channel = self.get_channel();
        let Some(channel) = channel else {
            println!("No available audio channels to play sound.");
            return;
        };
        if sound::is_playing(channel) {
            return;
        }
        let chunk = self.chunk.borrow();
        let Some(chunk) = chunk.as_ref() else {
            println!("No audio chunk loaded to play.");
            return;
        };
        // A short fade at the end of a sound that is not looped, in case it doesn't end in silence (no click)
        sound::set_sound_data_to_channel(
            channel,
            chunk,
            fade_in_ms.unwrap_or(100) as f32,
            END_FADE_MS,
            looped,
        );
    }

    pub fn pause(&self, fade_out_ms: Option<f32>) {
        let channel = self.currently_used_channel.borrow();
        let Some(channel) = channel.as_ref() else {
            return;
        };
        sound::pause_audio(*channel, fade_out_ms.unwrap_or(0.0));
    }

    pub fn resume(&self, fade_in_ms: Option<f32>) {
        let channel = self.currently_used_channel.borrow();
        let Some(channel) = channel.as_ref() else {
            return;
        };
        sound::resume_audio(*channel, fade_in_ms.unwrap_or(0.0));
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
        let Some(channel) = self.get_channel() else {
            return 0.0;
        };
        let progress_ratio = sound::get_audio_buffer(channel, |buffer| {
            (buffer.progress as f32) / buffer.buffer.len() as f32
        });
        progress_ratio * self.duration()
    }

    pub fn is_finished(&self) -> bool {
        let Some(channel) = self.get_channel() else {
            return true;
        };
        sound::get_audio_buffer(channel, |buffer| {
            (buffer.progress as f32) >= buffer.buffer.len() as f32
        })
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

#[cfg(test)]
mod tests {
    use super::{AUDIO_SAMPLE_FREQUENCY, to_output_format};

    fn sine(frequency: f32, sample_rate: u32, seconds: f32) -> Vec<f32> {
        let count = (sample_rate as f32 * seconds) as usize;
        (0..count)
            .map(|i| (i as f32 * frequency * std::f32::consts::TAU / sample_rate as f32).sin())
            .collect()
    }

    #[test]
    fn mono_is_duplicated_into_stereo() {
        let mono = sine(440.0, AUDIO_SAMPLE_FREQUENCY as u32, 0.1);
        let stereo = to_output_format(&mono, 1, AUDIO_SAMPLE_FREQUENCY as u32).expect("conversion");
        assert_eq!(stereo.len(), mono.len() * 2);
        let left = stereo.iter().step_by(2);
        let right = stereo.iter().skip(1).step_by(2);
        assert!(
            left.zip(right)
                .zip(&mono)
                .all(|((l, r), m)| l == m && r == m)
        );
    }

    #[test]
    fn other_rates_keep_their_duration_and_pitch() {
        let mono = sine(440.0, 44100, 1.0);
        let stereo = to_output_format(&mono, 1, 44100).expect("conversion");
        assert!(stereo.len().abs_diff(AUDIO_SAMPLE_FREQUENCY as usize * 2) <= 2);

        let left: Vec<f32> = stereo.iter().step_by(2).copied().collect();
        let middle = &left[left.len() / 4..3 * left.len() / 4];
        let crossings: Vec<usize> = middle
            .windows(2)
            .enumerate()
            .filter(|(_, w)| w[0] < 0.0 && w[1] >= 0.0)
            .map(|(i, _)| i)
            .collect();
        let periods = (crossings.len() - 1) as f32;
        let seconds =
            (crossings[crossings.len() - 1] - crossings[0]) as f32 / AUDIO_SAMPLE_FREQUENCY as f32;
        let frequency = periods / seconds;
        assert!(
            (frequency - 440.0).abs() < 1.0,
            "{frequency} Hz instead of 440 Hz"
        );
    }
}

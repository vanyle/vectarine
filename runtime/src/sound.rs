// We use a global audio queue to manage mixing.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use sdl2::Sdl;

static DURATION_OF_BUFFER_IN_MS: f32 = 150.0;

pub struct AudioResourceBuffer {
    pub buffer: VecDeque<f32>,
    pub is_playing: bool,
    pub volume: f32,
    pub is_looped: bool,
}

impl Default for AudioResourceBuffer {
    fn default() -> Self {
        Self {
            buffer: VecDeque::new(),
            is_playing: true,
            is_looped: false,
            volume: 1.0,
        }
    }
}

// Invariant: ChannelId refers to an index in the audio_buffers vector of the AudioQueue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelId(usize);

pub struct AudioQueue {
    pub audio_queue: sdl2::audio::AudioQueue<f32>,
    pub audio_buffers: HashMap<ChannelId, AudioResourceBuffer>,
}

impl AudioQueue {
    pub fn new(audio_queue: sdl2::audio::AudioQueue<f32>) -> Self {
        Self {
            audio_queue,
            audio_buffers: HashMap::new(),
        }
    }
    pub fn mix_audio(&mut self, bytes_to_advance: usize) -> Vec<f32> {
        let mut output = vec![0.0; bytes_to_advance * size_of::<f32>()];

        for buffer in self.audio_buffers.values_mut() {
            for output_sample in output.iter_mut() {
                let sample = buffer.buffer.pop_front().unwrap_or(0.0);
                if buffer.is_looped {
                    buffer.buffer.push_back(sample);
                }
                *output_sample += sample * buffer.volume;
            }
        }

        // Pad with zeros if needed.
        if output.len() < bytes_to_advance {
            let res = bytes_to_advance.saturating_sub(output.len());
            output.extend_from_slice(&vec![0.0; res]);
        }
        output
    }
}

thread_local! {
    static AUDIO_QUEUE: RefCell<Option<AudioQueue>> = const { RefCell::new(None) };
}

pub fn init_sound_system(sdl: &Sdl) {
    let audio = sdl.audio();
    let audio = match audio {
        Ok(audio) => audio,
        Err(audio_err) => {
            println!(
                "Failed to initialize audio subsystem: {:?}. Audio will be disabled.",
                audio_err
            );
            return;
        }
    };

    let desired_spec = sdl2::audio::AudioSpecDesired {
        freq: Some(crate::AUDIO_SAMPLE_FREQUENCY),
        channels: Some(crate::AUDIO_CHANNELS as u8), // stereo
        samples: None,                               // default sample size
    };

    let audio_queue = audio
        .open_queue::<f32, Option<&str>>(None, &desired_spec)
        .expect("Queue to be available");

    AUDIO_QUEUE.with_borrow_mut(|global_audio_queue| {
        *global_audio_queue = Some(AudioQueue::new(audio_queue));
    });
}

pub fn get_available_channel() -> ChannelId {
    let mut channel_id = ChannelId(0);
    AUDIO_QUEUE.with_borrow_mut(|global_audio_queue| {
        let buffers = &mut global_audio_queue
            .as_mut()
            .expect("Audio system should be initialized")
            .audio_buffers;
        channel_id = ChannelId(buffers.len());
        buffers.insert(channel_id, AudioResourceBuffer::default());
    });
    channel_id
}

pub fn get_audio_buffer<F>(channel_id: ChannelId, f: F)
where
    F: FnOnce(&mut AudioResourceBuffer),
{
    AUDIO_QUEUE.with_borrow_mut(|global_audio_queue| {
        let audio_queue = global_audio_queue
            .as_mut()
            .expect("Audio system should be initialized");
        let audio_buffer = audio_queue
            .audio_buffers
            .get_mut(&channel_id)
            .expect("Channel id refers to a channel in the buffers");
        f(audio_buffer);
    });
}

pub fn add_sound_data_to_channel(
    channel_id: ChannelId,
    data_to_play: &[f32],
    fade_in_ms: f32,
    fade_out_ms: f32,
    looped: bool,
) {
    let byte_count_needed_for_a_ms =
        (crate::AUDIO_CHANNELS as f32 * crate::AUDIO_SAMPLE_FREQUENCY as f32) / 1000.0;

    let samples_to_fade_in = (fade_in_ms * byte_count_needed_for_a_ms) as usize;
    let samples_to_fade_out = (fade_out_ms * byte_count_needed_for_a_ms) as usize;

    // For loop is clearer in this context
    #[allow(clippy::needless_range_loop)]
    get_audio_buffer(channel_id, |audio_buffer| {
        let mut sample_copy = data_to_play.to_vec();
        // Linear fade
        for i in 0..std::cmp::min(sample_copy.len(), samples_to_fade_in) {
            sample_copy[i] *= i as f32 / samples_to_fade_in as f32;
        }
        for i in 0..std::cmp::min(sample_copy.len(), samples_to_fade_out) {
            sample_copy[i] =
                sample_copy[sample_copy.len() - i - 1] * (i as f32 / samples_to_fade_out as f32);
        }

        audio_buffer.buffer.extend(sample_copy);
        audio_buffer.is_looped = looped;
    });
}

pub fn resume_audio(channel_id: ChannelId) {
    get_audio_buffer(channel_id, |audio_buffer| {
        audio_buffer.is_playing = true;
    });
}

pub fn pause_audio(channel_id: ChannelId) {
    get_audio_buffer(channel_id, |audio_buffer| {
        audio_buffer.is_playing = false;
    });
}

pub fn set_volume(channel_id: ChannelId, volume: f32) {
    get_audio_buffer(channel_id, |audio_buffer| {
        audio_buffer.volume = volume;
    });
}

pub fn get_volume(channel_id: ChannelId) -> f32 {
    let mut volume = 0.0;
    get_audio_buffer(channel_id, |audio_buffer| {
        volume = audio_buffer.volume;
    });
    volume
}

pub fn is_playing(channel_id: ChannelId) -> bool {
    let mut is_playing = false;
    get_audio_buffer(channel_id, |audio_buffer| {
        is_playing = audio_buffer.is_playing;
    });
    is_playing
}

pub fn flush_all_samples() {
    AUDIO_QUEUE.with_borrow_mut(|global_audio_queue| {
        let Some(global_audio_queue) = global_audio_queue else {
            return;
        };
        global_audio_queue.audio_queue.clear();
    });
}

/// You need to call this regularly for the sound system to work.
/// At least once every 150ms
pub fn update_sound_system() {
    // How long does a byte take to play (in ms)
    let byte_count_needed_for_a_ms =
        (crate::AUDIO_CHANNELS as f32 * crate::AUDIO_SAMPLE_FREQUENCY as f32) / 1000.0;
    let desired_size = (byte_count_needed_for_a_ms * DURATION_OF_BUFFER_IN_MS) as usize;

    AUDIO_QUEUE.with_borrow_mut(|global_audio_queue| {
        let Some(global_audio_queue) = global_audio_queue else {
            return;
        };

        let size = global_audio_queue.audio_queue.size() as usize;
        // We append to the queue enough bytes to be able to play for at least 150ms
        let number_of_bytes_to_append = desired_size.saturating_sub(size);
        if number_of_bytes_to_append > 0 {
            let bytes_to_queue = global_audio_queue.mix_audio(number_of_bytes_to_append);
            let result = global_audio_queue.audio_queue.queue_audio(&bytes_to_queue);
            if let Err(result) = result {
                println!("Failed to queue audio: {:?}", result);
            }
            // Resume all the time, we manage the bytes ourselves.
            global_audio_queue.audio_queue.resume();
        }
    });
}

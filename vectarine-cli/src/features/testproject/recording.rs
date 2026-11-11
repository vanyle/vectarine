use runtime::image::{
    Delay, Frame,
    codecs::gif::{GifEncoder, Repeat},
};
use runtime::{
    anyhow::{Result, anyhow},
    image::RgbaImage,
};
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    process::{Child, Command, Stdio},
};

use crate::headless::GameHeadlessRunner;

pub(crate) enum Encoder {
    Gif(GifEncoder<BufWriter<File>>),
    Mp4(Child),
}

pub(crate) struct Recording {
    pub(crate) output: PathBuf,
    pub(crate) frame_count: usize,
    pub(crate) frames_recorded: usize,
    pub(crate) encoder: Option<Encoder>,
}

impl Recording {
    pub(crate) fn new(output: PathBuf, frame_count: usize) -> Result<Self> {
        if !matches!(
            output.extension().and_then(|e| e.to_str()),
            Some("gif" | "mp4")
        ) {
            return Err(anyhow!(
                "Unsupported recording format for {}: use .gif or .mp4",
                output.display()
            ));
        }
        Ok(Self {
            output,
            frame_count,
            frames_recorded: 0,
            encoder: None,
        })
    }

    pub(crate) fn start_encoder(&self, width: u32, height: u32) -> Result<Encoder> {
        if self.output.extension().and_then(|e| e.to_str()) == Some("gif") {
            let file = BufWriter::new(File::create(&self.output)?);
            let mut gif = GifEncoder::new_with_speed(file, 10);
            gif.set_repeat(Repeat::Infinite)?;
            return Ok(Encoder::Gif(gif));
        }
        let ffmpeg = Command::new("ffmpeg")
            .args(["-v", "error", "-y", "-f", "rawvideo", "-pix_fmt", "rgba", "-s"])
            .arg(format!("{width}x{height}"))
            .args(["-r", "60", "-i", "-", "-vf", "pad=ceil(iw/2)*2:ceil(ih/2)*2"])
            .args(["-c:v", "libx264", "-crf", "12", "-pix_fmt", "yuv420p"])
            .args(["-movflags", "+faststart"])
            .arg(&self.output)
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|e| {
                anyhow!("Recording an mp4 requires ffmpeg in your PATH ({e}). Record a .gif instead, or install ffmpeg.")
            })?;
        Ok(Encoder::Mp4(ffmpeg))
    }

    pub(crate) fn add_frame(&mut self, frame: &RgbaImage) -> Result<()> {
        if self.encoder.is_none() {
            self.encoder = Some(self.start_encoder(frame.width(), frame.height())?);
        }
        match &mut self.encoder {
            // GIF delays are in centiseconds and browsers slow down delays under 2, so we keep 30 fps.
            Some(Encoder::Gif(gif)) if self.frames_recorded.is_multiple_of(2) => {
                let centiseconds_at = |frame_index: usize| (frame_index as u32 * 100 + 30) / 60;
                let delay = centiseconds_at(self.frames_recorded + 2)
                    - centiseconds_at(self.frames_recorded);
                gif.encode_frame(Frame::from_parts(
                    frame.clone(),
                    0,
                    0,
                    Delay::from_numer_denom_ms(delay * 10, 1),
                ))?;
            }
            Some(Encoder::Mp4(ffmpeg)) => {
                ffmpeg
                    .stdin
                    .as_mut()
                    .ok_or_else(|| anyhow!("Failed to send the frames to ffmpeg"))?
                    .write_all(frame.as_raw())?;
            }
            _ => {}
        }
        self.frames_recorded += 1;
        Ok(())
    }

    pub(crate) fn finish(self) -> Result<()> {
        match self.encoder {
            None => {
                println!(
                    "Warning: no frames were recorded for {}",
                    self.output.display()
                );
                return Ok(());
            }
            Some(Encoder::Gif(gif)) => drop(gif),
            Some(Encoder::Mp4(mut ffmpeg)) => {
                drop(ffmpeg.stdin.take());
                if !ffmpeg.wait()?.success() {
                    return Err(anyhow!("ffmpeg failed to write {}", self.output.display()));
                }
            }
        }
        println!(
            "Recorded {} frames to {}",
            self.frames_recorded,
            self.output.display()
        );
        Ok(())
    }
}

pub(crate) fn capture_screenshot(game_runner: &GameHeadlessRunner) -> Result<RgbaImage> {
    let (data, width, height) = game_runner.screenshot()?;
    RgbaImage::from_raw(width, height, data)
        .ok_or_else(|| anyhow!("Failed to create image buffer from screenshot data"))
}

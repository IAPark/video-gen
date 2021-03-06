extern crate ffmpeg_next as ffmpeg;

use ffmpeg_next::codec::video::FormatIter;
use std::ops::Drop;
use ffmpeg_next::Rational;
use ffmpeg::Packet;
use ffmpeg::format::{output, Pixel};
use ffmpeg::util::frame::video::Video as FfmpegFrame;
use ffmpeg::format::context::output::Output;

use std::path::Path;
use std::fmt;

pub struct VideoGenerator {
  output: Output,
  time_base: Rational,
  stream_id: i32,
  pixel_format: Pixel,
  frame_num: i64,
  encoder: ffmpeg::encoder::Video,

  finalized: bool,
}
impl VideoGenerator {
  pub fn new<P: AsRef<Path>, R: Into<Rational>>(
    path: P,
    time_base: R,
    codec: ffmpeg::Codec,
    width: u32,
    height: u32,
  ) -> Result<VideoGenerator, Error> {
    let time_base: Rational = time_base.into();

    let mut output = output(&path)?;
    let mut stream = output.add_stream(codec)?;
    stream.set_time_base(time_base);
    let stream_id = stream.id();

    let mut encoder = stream.codec().encoder().video()?;
    encoder.set_time_base(time_base);
    let pixel_format = valid_format(codec.video()?.
      formats().
      ok_or(Error::MissingFormat)?).
      ok_or(Error::MissingFormat)?;
    encoder.set_format(pixel_format);
    encoder.set_width(width);
    encoder.set_height(height);

    let encoder = encoder.open()?;

    output.write_header()?;

    return Ok(
      VideoGenerator {
        output,
        time_base,
        stream_id: stream_id,
        pixel_format,
        frame_num: 0,
        encoder,
        finalized: false,
      }
    )
  }

  pub fn add_frame<T: Into<RgbFrame>>(&mut self, data: T) -> Result<(), Error> {
    let frame: RgbFrame = data.into();
    let mut codec_frame = frame.into_fmpeg(self.pixel_format)?;
    codec_frame.set_pts(Some(self.frame_num));
    self.frame_num += 1;

    self.encoder.send_frame(&codec_frame)?;

    self.drain_encoder()?;

    Ok(())
  }

  pub fn finalize(&mut self) -> Result<(), Error> {
    if self.finalized {
      return Ok(());
    }
    self.finalized = true;

    self.encoder.send_eof()?;
    self.drain_encoder()?;
    self.output.write_trailer()?;
    Ok(())
  }

  pub fn un_finalize(&mut self) {
    self.finalized = false;

  }

  pub fn drain_encoder(&mut self) -> Result<(), Error> {
    let mut encoded = Packet::empty();
    while self.encoder.receive_packet(&mut encoded).is_ok() {
      encoded.set_stream(self.stream_id as usize);
      encoded.rescale_ts(self.time_base, self.stream()?.time_base());
      encoded.write_interleaved(&mut self.output)?;
    }

    Ok(())
  }

  pub fn width(&self) -> u32 {
    self.encoder.width()
  }
  pub fn height(&self) -> u32 {
    self.encoder.height()
  }

  pub fn stream(&self) -> Result<ffmpeg::Stream, Error> {
    for stream in self.output.streams() {
      if stream.id() == self.stream_id {
        return Ok(stream)
      }
    }

    Err(Error::MissingStream)
  }

  pub fn init() -> Result<(), Error> {
    Ok(ffmpeg::init()?)
  }

}

impl Drop for VideoGenerator {
  fn drop(&mut self) {
    self.finalize().unwrap()
  }
}

fn valid_format(mut formats: FormatIter) -> Option<Pixel> {
  formats.find(|&format| format != Pixel::VIDEOTOOLBOX)
}


#[derive(Debug)]
pub enum Error {
  Ffmpeg(ffmpeg::Error),
  MissingFormat,
  InvalidCodec,
  MissingStream,
}

impl std::convert::From<ffmpeg::Error> for Error {
  fn from(error: ffmpeg::Error) -> Self {
    Error::Ffmpeg(error)
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::Ffmpeg(error) => error.fmt(f),
      Error::MissingFormat => write!(f, "Codec is missing a pixel format"),
      Error::InvalidCodec => write!(f, "Can't find that codec"),
      Error::MissingStream => write!(f, "Can't find the stream"),
    }
  }
}
impl std::error::Error for Error {}

pub struct RgbFrame {
  frame: FfmpegFrame,
}

impl RgbFrame {
  pub fn into_fmpeg(&self, pixel_format: Pixel) -> Result<FfmpegFrame, Error> {
    let mut converter = self.frame.converter(pixel_format)?;

    let mut output = FfmpegFrame::new(pixel_format, self.width(), self.height());
    converter.run(&self.frame, &mut output)?;

    Ok(output)
  }

  pub unsafe fn from_ptr(buffer: *const u8, width: u32, height: u32) -> RgbFrame {
    let mut frame = FfmpegFrame::empty();
    frame.set_format(Pixel::RGB32);
    frame.set_width(width);
    frame.set_height(height);
    (*frame.as_mut_ptr()).data[0] = buffer as *mut u8;

    RgbFrame {
      frame,
    }
  }

  pub fn width(&self) -> u32 {
    self.frame.width()
  }

  pub fn height(&self) -> u32 {
    self.frame.height()
  }
}

impl std::convert::From<FfmpegFrame> for RgbFrame {
  fn from(frame: FfmpegFrame) -> Self {
    RgbFrame {
      frame,
    }
  }
}

impl std::convert::From<RgbFrame> for FfmpegFrame {
  fn from(rgb_frame: RgbFrame) -> Self {
    rgb_frame.frame
  }
}

# video-gen
A simple crate to generate a video from raw frame buffers in rust

# Example

```rust
extern crate ffmpeg_next as ffmpeg;

use ffmpeg_next::Rational;
use ffmpeg::format::Pixel;
use ffmpeg::util::frame::video::Video as Frame;
use video_gen::VideoGenerator;
use ffmpeg::{codec, encoder};

fn main() {
    VideoGenerator::init().unwrap();

    let mut generator = VideoGenerator::new(
        &"test.mp4",
        Rational(1, 30),
        encoder::find(codec::Id::H264).unwrap(),
        1000, 1000
    ).unwrap();

    let max = 30*10;
    for i in 0..max {
        let mut frame = Frame::new(Pixel::RGB24, 1000, 1000);
        for x in 0..frame.planes() {
            for d in frame.data_mut(x) {
                let color: i32 = (i * 255)/max;
                *d = color as u8;
            }
        }

        generator.add_frame(frame).unwrap();
    }

    generator.finalize().unwrap();
}
```

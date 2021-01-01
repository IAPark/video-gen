extern crate ffmpeg_next as ffmpeg;

use ffmpeg_next::Rational;
use ffmpeg::format::Pixel;
use ffmpeg::util::frame::video::Video as Frame;
use video_gen::VideoGenerator;
use ffmpeg::codec;

fn main() {
    VideoGenerator::init().unwrap();

    let mut generator = VideoGenerator::new(
        &"test.mp4",
        Rational(1, 30),
        codec::Id::H264,
        1000, 1000
    ).unwrap();

    let max = 30*60;
    for i in 0..max {
        let mut frame = Frame::new(Pixel::RGB24, 1000, 1000);
        for x in 0..frame.planes() {
            for d in frame.data_mut(x) {
                if i % 2 == 0 {
                    *d = 0;
                } else {
                    *d = 255;
                }
            }
        }

        generator.add_frame(frame).unwrap();
    }

    generator.finalize().unwrap();
}

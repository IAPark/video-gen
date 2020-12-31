extern crate ffmpeg_next as ffmpeg;

use ffmpeg_next::Rational;
use ffmpeg::Packet;
use ffmpeg::format::{output, Pixel};
use ffmpeg::util::frame::video::Video as Frame;
use ffmpeg::{log, encoder, codec};


fn main() {
    ffmpeg::init().unwrap();
    log::set_level(log::Level::Error);

    if let Ok(mut out) = output(&"test.mp4") {
        let mut stream = out.add_stream(encoder::find(codec::Id::H264)).unwrap();
        println!("opened");
        let mut encoder = stream.codec().encoder().video().unwrap();
        let mut time_base = Rational(1, 3000);

        stream.set_time_base(time_base);
        let stream_id = stream.id();
        drop(stream);

        encoder.set_time_base(time_base);
        encoder.set_format(Pixel::YUV420P);
        encoder.set_width(100);
        encoder.set_height(100);
        let mut encoder = encoder.open().unwrap();
        out.write_header().unwrap();
        for stream in out.streams() {
            time_base = stream.time_base();
        }
        let max = 30*60;
        for i in 0..max {
            let mut frame = Frame::new(Pixel::YUV420P, 100, 100);
            for x in 0..frame.planes() {
                for d in frame.data_mut(x) {
                    if i % 2 == 0 {
                        *d = 0;
                    } else {
                        *d = 255;
                    }
                }
            }
            frame.set_pts(Some(i));
            encoder.send_frame(&frame).unwrap();
            let mut encoded = Packet::empty();
            while encoder.receive_packet(&mut encoded).is_ok() {
                encoded.set_stream(stream_id as usize);
                encoded.rescale_ts(Rational(1, 30), time_base);
                encoded.write_interleaved(&mut out).unwrap();
            }
        }
        encoder.send_eof().unwrap();
        let mut encoded = Packet::empty();
        while encoder.receive_packet(&mut encoded).is_ok() {
            encoded.set_stream(stream_id as usize);
            encoded.rescale_ts(Rational(1, 30), time_base);
            encoded.write_interleaved(&mut out).unwrap();
        }
        out.write_trailer().unwrap();

    }
    println!("Hello, world!");
}

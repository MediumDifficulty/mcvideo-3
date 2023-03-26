use ffmpeg::{format::{context::Input, Pixel}, media::Type, software::scaling, frame::Video, decoder};
use valence::util::{vec3, Vec3};

pub struct FrameExtractor {
    input: Input,
    decoder: decoder::Video,
    scaler: scaling::Context,
    video_stream_index: usize,
}

impl FrameExtractor {
    pub fn new(input: Input, width: u32, height: u32) -> Self {
        let video = input.streams().best(Type::Video).unwrap();
        let video_stream_index = video.index();
    
        let context_decoder = ffmpeg::codec::context::Context::from_parameters(video.parameters()).unwrap();
        let decoder = context_decoder.decoder().video().unwrap();
    
        let scaler = scaling::Context::get(
            decoder.format(),
            decoder.width(),
            decoder.height(),
            Pixel::RGB24,
            width,
            height,
            scaling::Flags::BILINEAR
        ).unwrap();


        Self { input, decoder, scaler, video_stream_index }
    }
}

impl Iterator for FrameExtractor {
    type Item = Vec<Vec3>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut decoded = Video::empty();

        if self.decoder.receive_frame(&mut decoded).is_ok() {
            let mut rgb_frame = Video::empty();
            self.scaler.run(&decoded, &mut rgb_frame).unwrap();

            return Some(bytes_f32(rgb_frame.data(0)));
        }

        loop {
            let next = self.input.packets().next();
            next.as_ref()?;

            let next = next.unwrap();

            if next.0.index() == self.video_stream_index {
                self.decoder.send_packet(&next.1).unwrap();

                if self.decoder.receive_frame(&mut decoded).is_ok() {
                    let mut rgb_frame = Video::empty();
                    self.scaler.run(&decoded, &mut rgb_frame).unwrap();

                    return Some(bytes_f32(rgb_frame.data(0)));
                }
            }
        }
    }
}

fn bytes_f32(bytes: &[u8]) -> Vec<Vec3> {
    bytes.chunks(3)
        .map(|b| vec3(b[0] as f32, b[1] as f32, b[2] as f32) / 255.)
        .collect::<Vec<Vec3>>()
}
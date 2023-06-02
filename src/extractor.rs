use std::{time::Duration, process::Command, fs};

use ffmpeg::{format::{context::Input, Pixel}, media::Type, software::scaling, frame::Video, decoder};
use valence::{prelude::Vec3, glam::vec3};

pub struct FrameExtractor {
    input: Input,
    decoder: decoder::Video,
    scaler: scaling::Context,
    video_stream_index: usize,
    framerate: f32,
    index: usize,
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
        let framerate = video.avg_frame_rate();
        let framerate = framerate.0 as f32 / framerate.1 as f32;
        
        Self { input, decoder, scaler, video_stream_index, framerate, index: 0 }
    }

    pub fn frametime(&self) -> Duration {
        Duration::from_secs_f32(self.index as f32 * (1. / self.framerate))
    }
}

impl Iterator for FrameExtractor {
    type Item = (Vec<Vec3>, Duration);

    fn next(&mut self) -> Option<Self::Item> {
        let mut decoded = Video::empty();

        let frame_time = self.frametime();
        self.index += 1;

        if self.decoder.receive_frame(&mut decoded).is_ok() {
            let mut rgb_frame = Video::empty();
            self.scaler.run(&decoded, &mut rgb_frame).unwrap();

            return Some((bytes_f32(rgb_frame.data(0)), frame_time));
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

                    return Some((bytes_f32(rgb_frame.data(0)), frame_time));
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

pub fn extract_audio(filename: &str, clip_length: usize) -> Vec<Vec<u8>> {
    // Extract audio to ogg
    fs::create_dir_all("temp").unwrap();

    let mut command = Command::new("ffmpeg");

    command
        .args([
            "-i", filename,
            "-vn",
            "-acodec","libvorbis",
            "-y",
            "-f", "segment",
            "-reset_timestamps", "1",
            "-segment_time", clip_length.to_string().as_str(),
            "temp/%d.ogg"
        ]);

    command.output().unwrap();
    
    let mut audio = Vec::new();
    fs::read_dir("temp").unwrap()
        .map(|file| {
            let path = file.unwrap().path();
            (path.file_stem().unwrap().to_string_lossy().to_string().parse::<usize>().unwrap(), path)
        }).for_each(|e| audio.push(e));

    audio.sort_by(|a, b| a.0.cmp(&b.0));
    
    let files = audio.iter()
        .map(|(_, path)| fs::read(path).unwrap().to_vec())
        .collect::<Vec<Vec<u8>>>();

    fs::remove_dir_all("temp").unwrap();

    files
}
// use std::time::Instant;

use std::time::{Instant, Duration};

use ffmpeg::format::context::Input;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use valence::glam::uvec2;

use crate::{extractor::FrameExtractor, util::Colour};

const BLACK_ID: u8 = 119;

static BAKED_MAP_COLOURS: &[u8] = include_bytes!("assets/closest_colours.dat");

pub struct FrameProcessor {
    extractor: FrameExtractor,
    pub width: u32,
    pub height: u32,
    start_time: Instant,
    started: bool,
    pub paused: bool,
    last_played_audio: Option<usize>,
    audio_clip_length: usize,
}

impl FrameProcessor {
    pub fn new(input: Input, width: u32, height: u32, audio_clip_length: usize) -> Self {
        let extractor = FrameExtractor::new(input, width, height);

        Self {
            extractor,
            width,
            height,
            start_time: Instant::now(),
            paused: true,
            started: false,
            last_played_audio: None,
            audio_clip_length,
        }
    }

    pub fn start(&mut self) -> bool {
        let started = !self.started;
        if !self.started {
            self.start_time = Instant::now();
            self.started = true;
        }
        self.paused = false;

        started
    }

    pub fn should_play_audio(&mut self) -> (bool, usize) {
        let Some(last_played_audio) = self.last_played_audio.as_mut() else {
            self.last_played_audio = Some(0);
            return (true, 0);
        };

        let playing_audio = self.start_time.elapsed().as_secs() as usize / self.audio_clip_length;

        if playing_audio > *last_played_audio {
            *last_played_audio = playing_audio;
            return (true, playing_audio);
        }

        (false, 0)
    }
}

impl Iterator for FrameProcessor {
    type Item = Option<Vec<Vec<u8>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.extractor.frametime() > self.start_time.elapsed() + Duration::from_millis(50) {
            return Some(None);
        }

        // Find the closest map colour
        // let start_a = Instant::now();
        let map_colours = get_next_frame_for_time(&mut self.extractor, self.start_time.elapsed())
            .map(|raw_frame| raw_frame.par_iter()
                .map(|pixel| BAKED_MAP_COLOURS[pixel.as_usize()])
                // .map(|pixel| MAP_COLOURS.iter().enumerate()
                //     .filter(|e| e.0 > 3)
                //     .min_by(|&a, &b| a.1.distance_squared(*pixel).total_cmp(&b.1.distance_squared(*pixel)))
                //         .unwrap()
                //         .0 as u8
                // )
                .collect::<Vec<u8>>()
            )?;
        // info!("finding closest colour took {}ms", start_a.elapsed().as_millis());


        // let start_b = Instant::now();
        let width_maps = num_integer::div_ceil(self.width, 128);
        let height_maps = num_integer::div_ceil(self.height, 128);

        let mut maps = vec![vec![BLACK_ID; 16384]; (width_maps * height_maps) as usize];
        
        let offset = uvec2((width_maps * 128 - self.width) / 2, (height_maps * 128 - self.height) / 2);

        for (i, &colour) in map_colours.iter().enumerate() {
            let coords = uvec2(i as u32 % self.width, i as u32 / self.width) + offset;
            let block_coords = coords / 128;
            let map_coords = coords % 128;

            maps[(block_coords.y * width_maps + block_coords.x) as usize][(map_coords.y * 128 + map_coords.x) as usize] = colour;
        }

        // info!("Plotting onto maps took {}ms", start_b.elapsed().as_millis());

        Some(Some(maps))
    }
}

fn get_next_frame_for_time(extractor: &mut FrameExtractor, elapsed_time: Duration) -> Option<Vec<Colour>> {
    let (mut frame, mut frame_time) = extractor.next()?;

    while frame_time < elapsed_time {
        (frame, frame_time) = extractor.next()?;
    }

    Some(frame)
}
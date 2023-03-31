// use std::time::Instant;

use std::time::{Instant, Duration};

use ffmpeg::format::context::Input;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use valence::util::{uvec2, Vec3};

use crate::{extractor::FrameExtractor, map_colours::MAP_COLOURS};

const BLACK_ID: u8 = 119;

pub struct FrameProcessor {
    extractor: FrameExtractor,
    pub width: u32,
    pub height: u32,
    start_time: Instant,
}

impl FrameProcessor {
    pub fn new(input: Input, width: u32, height: u32) -> Self {
        let extractor = FrameExtractor::new(input, width, height);

        Self { extractor, width, height, start_time: Instant::now() }
    }

    pub fn start(&mut self) {
        self.start_time = Instant::now();
    }
}

impl Iterator for FrameProcessor {
    type Item = Vec<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Find the closest map colour
        // let start_a = Instant::now();
        let map_colours = get_next_frame_for_time(&mut self.extractor, self.start_time.elapsed())
            .map(|raw_frame| raw_frame.par_iter()
                .map(|pixel| MAP_COLOURS.iter().enumerate()
                    .filter(|e| e.0 > 3)
                    .min_by(|&a, &b| a.1.distance_squared(*pixel).total_cmp(&b.1.distance_squared(*pixel)))
                        .unwrap()
                        .0 as u8
                )
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

        Some(maps)
    }
}

fn get_next_frame_for_time(extractor: &mut FrameExtractor, elapsed_time: Duration) -> Option<Vec<Vec3>> {
    let (mut frame, mut frame_time) = extractor.next()?;

    while frame_time < elapsed_time {
        (frame, frame_time) = extractor.next()?;
    }

    Some(frame)
}
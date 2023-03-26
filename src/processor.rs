// use std::time::Instant;

use ffmpeg::format::context::Input;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use valence::util::uvec2;

use crate::{extractor::FrameExtractor, map_colours::MAP_COLOURS};

const BLACK_ID: u8 = 119;

pub struct FrameProcessor {
    extractor: FrameExtractor,
    pub width: u32,
    pub height: u32,
}

impl FrameProcessor {
    pub fn new(input: Input, width: u32, height: u32) -> Self {
        let extractor = FrameExtractor::new(input, width, height);

        Self { extractor, width, height }
    }
}

impl Iterator for FrameProcessor {
    type Item = Vec<Vec<u8>>;

    fn next(&mut self) -> Option<Self::Item> {
        // Find the closest map colour
        // let start_a = Instant::now();
        let map_colours = self.extractor.next()
            .map(|raw_frame| raw_frame.par_iter()
                .map(|pixel| MAP_COLOURS.iter().enumerate()
                    .filter(|e| e.0 > 3)
                    .min_by(|&a, &b| a.1.distance_squared(*pixel).total_cmp(&b.1.distance_squared(*pixel)))
                        .unwrap()
                        .0 as u8
                )
                .collect::<Vec<u8>>()
            )?;
        // println!("finding closest colour took {}ms", start_a.elapsed().as_millis());


        // let start_b = Instant::now();
        let width_maps = num_integer::div_ceil(self.width, 128);
        let height_maps = num_integer::div_ceil(self.height, 128);

        let mut maps = vec![vec![BLACK_ID; 16384]; (width_maps * height_maps) as usize];
        
        let offset = uvec2((width_maps * 128 - self.width) / 2, (height_maps * 128 - self.height) / 2);

        for (i, &colour) in map_colours.iter().enumerate() {
            let coords = uvec2(i as u32 % self.width, i as u32 / self.width) + offset;
            let block_coords = coords / 128;
            let map_coords = coords % 128;

            // println!("Block: {} Map: {}", block_coords.y * width_maps + block_coords.x, map_coords.y * self.width + map_coords.x);

            maps[(block_coords.y * width_maps + block_coords.x) as usize][(map_coords.y * 128 + map_coords.x) as usize] = colour;
        }

        // println!("Plotting onto maps took {}ms", start_b.elapsed().as_millis());

        Some(maps)
    }
}
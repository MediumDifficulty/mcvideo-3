pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Colour {
    pub fn from_slice(slice: &[u8]) -> Colour {
        Colour { r: slice[0], g: slice[1], b: slice[2] }
    }

    pub fn as_usize(&self) -> usize {
        (self.r as usize) << 16 | (self.g as usize) << 8 | self.b as usize
    }
}
//! PIG texture/bitmap file format parser
//!
//! Corresponds to: `include/piggy.h`, `2d/piggy.cpp`, `2d/bitmap.cpp`

use crate::error::Result;

pub struct PigFile {
    // TODO: Implement
}

pub struct BitmapEntry {
    pub name: String,
    pub width: u16,
    pub height: u16,
}

pub enum BitmapData {
    Indexed(Vec<u8>),
    Rgb(Vec<u8>),
    Rgba(Vec<u8>),
}

impl PigFile {
    pub fn parse(_data: &[u8]) -> Result<Self> {
        todo!("Implement PIG parsing")
    }

    pub fn bitmaps(&self) -> &[BitmapEntry] {
        todo!()
    }
}

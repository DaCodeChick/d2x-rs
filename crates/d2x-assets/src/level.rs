//! Level geometry file format parser (RDL/RL2)
//!
//! Corresponds to: `include/segment.h`, `main/loadgeometry.cpp`

use crate::error::Result;
use glam::Vec3;

pub struct Level {
    pub metadata: LevelMetadata,
    pub segments: Vec<Segment>,
}

pub struct LevelMetadata {
    pub name: String,
    pub version: u32,
}

pub struct Segment {
    pub vertices: [u16; 8],
    pub sides: [Side; 6],
}

pub struct Side {
    pub tmap_num: u16,
}

impl Level {
    pub fn parse(_data: &[u8]) -> Result<Self> {
        todo!("Implement level parsing")
    }
}

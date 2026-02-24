//! Sound and music file format parsers

use crate::error::Result;

pub struct SoundFile {
    // TODO: Implement
}

pub struct SoundEffect {
    pub name: String,
}

impl SoundFile {
    pub fn parse(_data: &[u8]) -> Result<Self> {
        todo!("Implement sound parsing")
    }
}

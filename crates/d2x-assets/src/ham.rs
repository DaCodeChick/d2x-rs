//! HAM game data file format parser
//!
//! Corresponds to: `include/loadgamedata.h`, `main/loadgamedata.cpp`

use crate::error::Result;

pub struct HamFile {
    // TODO: Implement
}

pub struct RobotInfo {
    pub model_num: u8,
}

pub struct WeaponInfo {
    pub damage: f32,
}

impl HamFile {
    pub fn parse(_data: &[u8]) -> Result<Self> {
        todo!("Implement HAM parsing")
    }
}

//! Video conversion setup for cutscenes.
//!
//! This module handles the one-time setup conversion of MVE video files to MP4 format
//! during the client's first-time setup process. The actual conversion logic lives in
//! `descent-core::video` - this module provides the setup/UI integration.

#[cfg(feature = "cutscenes")]
#[allow(unused_imports)] // Re-exported for convenience, may not be used directly
pub use descent_core::VideoConverter;

#[cfg(not(feature = "cutscenes"))]
/// Stub when cutscenes feature is disabled
pub struct VideoConverter;

#[cfg(not(feature = "cutscenes"))]
impl VideoConverter {
    pub const fn new() -> Self {
        Self
    }
}

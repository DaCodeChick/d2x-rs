//! # D2X Assets
//!
//! Asset extraction and parsing library for Descent 1 and 2 game files.
//!
//! This crate provides parsers for all Descent data file formats:
//! - **HOG**: Archive files containing game assets
//! - **PIG**: Texture and bitmap data
//! - **HAM**: Game data definitions (robots, weapons, physics)
//! - **Palette**: Color palettes for indexed bitmaps
//! - **RDL/RL2**: Level geometry and metadata
//! - **POF/OOF/ASE**: 3D model formats
//! - **Sound**: Audio samples and music
//!
//! ## Example
//!
//! ```ignore
//! use d2x_assets::{HogArchive, PigFile, Palette};
//!
//! // Open HOG archive
//! let hog = HogArchive::open("descent2.hog")?;
//!
//! // Load palette
//! let palette_data = std::fs::read("groupa.256")?;
//! let palette = Palette::parse(&palette_data)?;
//!
//! // Extract and parse PIG file
//! let pig_data = hog.read_file("groupa.pig")?;
//! let pig = PigFile::parse(&pig_data)?;
//!
//! // Convert indexed texture to RGBA
//! let bitmap = pig.get_bitmap_by_name("WALL01")?;
//! let rgba = palette.indexed_to_rgba(
//!     bitmap.data(),
//!     bitmap.header.width as usize,
//!     bitmap.header.height as usize
//! )?;
//! ```

pub mod error;
pub mod ham;
pub mod hog;
pub mod level;
pub mod models;
pub mod palette;
pub mod pig;
pub mod sound;

pub use error::{AssetError, Result};
pub use ham::{HamFile, RobotInfo, WeaponInfo};
pub use hog::{HogArchive, HogEntry};
pub use level::{Level, Segment, Side};
pub use palette::Palette;
pub use pig::{BitmapData, BitmapFlags, BitmapHeader, PigFile};

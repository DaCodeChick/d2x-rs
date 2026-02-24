//! # D2X Assets
//!
//! Asset extraction and parsing library for Descent 1 and 2 game files.
//!
//! This crate provides parsers for all Descent data file formats:
//! - **HOG**: Archive files containing game assets
//! - **PIG**: Texture and bitmap data
//! - **HAM**: Game data definitions (robots, weapons, physics)
//! - **RDL/RL2**: Level geometry and metadata
//! - **POF/OOF/ASE**: 3D model formats
//! - **Sound**: Audio samples and music
//!
//! ## Corresponds to D2X-XL Code
//!
//! - `include/hogfile.h`, `io/hogfile.cpp`
//! - `include/piggy.h`, `2d/piggy.cpp`, `2d/bitmap.cpp`
//! - `include/loadgamedata.h`, `main/loadgamedata.cpp`
//! - `include/segment.h`, `main/loadgeometry.cpp`
//!
//! ## Example
//!
//! ```ignore
//! use d2x_assets::{HogArchive, PigFile};
//!
//! // Open HOG archive
//! let hog = HogArchive::open("descent2.hog")?;
//!
//! // Extract and parse PIG file
//! let pig_data = hog.read_file("groupa.pig")?;
//! let pig = PigFile::parse(&pig_data)?;
//!
//! // Access textures
//! for bitmap in pig.bitmaps() {
//!     println!("Texture: {}", bitmap.name);
//! }
//! ```

pub mod error;
pub mod hog;
pub mod pig;
pub mod ham;
pub mod level;
pub mod models;
pub mod sound;

pub use error::{AssetError, Result};
pub use hog::{HogArchive, HogEntry};
pub use pig::{PigFile, BitmapEntry, BitmapData};
pub use ham::{HamFile, RobotInfo, WeaponInfo};
pub use level::{Level, Segment, Side};

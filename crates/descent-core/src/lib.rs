//! # Descent Core
//!
//! Core data parsing and asset extraction library for Descent 1, 2, and 3.
//!
//! This crate provides parsers for all Descent game data file formats across all three games:
//!
//! ## Descent 1 & 2 (1995-1996) - Parallax Engine
//! - **DHF**: Archive files containing game assets (DHF format)
//! - **MVL**: Movie library archives containing MVE video files
//! - **MVE**: Interplay movie format for cutscenes
//! - **PIG**: Texture and bitmap data (RLE compressed, 8-bit indexed)
//! - **HAM**: Game data definitions (robots, weapons, physics)
//! - **Palette**: Color palettes for indexed bitmaps (6-bit RGB)
//! - **RDL/RL2**: Level geometry and metadata (segment-based)
//! - **POF**: 3D polygon models
//! - **Sound**: Audio samples (SNDs) and music (HMP)
//! - **Mission**: Mission files (.MSN/.MN2)
//! - **Player**: Player profiles (.PLR binary, .PLX text)
//!
//! ## Descent 3 (1999) - Outrage Engine
//! - **HOG2**: Archive format (enhanced version)
//! - **D3L**: Level files (room-based with portals)
//! - **OGF**: Outrage Graphics Format (textures with modern features)
//! - **OOF**: Outrage Object Format (3D models with animations)
//! - **OSF**: Outrage Sound Format
//! - **GAM**: Game data tables (replaces HAM)
//! - **OSIRIS**: Scripting system (DLL-based)
//! - **MN3**: Mission definition files
//!
//! ## Example - Descent 1/2
//!
//! ```ignore
//! use descent_core::{DhfArchive, PigFile, Palette};
//!
//! // Open DHF archive (Descent 1/2)
//! let mut hog = DhfArchive::open("descent2.hog")?;
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
//!
//! ## Example - Descent 3
//!
//! ```ignore
//! use descent_core::Hog2Archive;
//!
//! // Open HOG2 archive (Descent 3)
//! let mut hog = Hog2Archive::open("d3.hog")?;
//!
//! // List all files
//! for entry in hog.entries() {
//!     println!("{} - {} bytes", entry.name, entry.size);
//! }
//!
//! // Extract a level file
//! let level_data = hog.read_file("level1.d3l")?;
//! ```
//!
//! ## Example - Converting Textures
//!
//! ```ignore
//! use descent_core::{PigFile, Palette};
//! use descent_core::converters::texture::{TextureConverter, ImageFormat};
//!
//! // Load PIG file and palette
//! let pig_data = std::fs::read("descent2.pig")?;
//! let pig = PigFile::parse(pig_data, false)?;
//!
//! let palette_data = std::fs::read("groupa.256")?;
//! let palette = Palette::parse(&palette_data)?;
//!
//! // Convert texture to PNG
//! let converter = TextureConverter::new(&palette);
//! let png = converter.pig_to_png(&pig, "wall01-0")?;
//! std::fs::write("wall01-0.png", png)?;
//! ```

pub mod converters;
pub mod dhf;
pub mod error;
pub mod fixed_point;
pub mod geometry;
pub mod ham;
pub mod hog2;
pub mod io;
pub mod level;
pub mod mission;
pub mod models;
pub mod mve;
pub mod mvl;
pub mod ogf;
pub mod palette;
pub mod pig;
pub mod player;
pub mod pof;
pub mod sound;
pub mod validation;

pub use dhf::{DhfArchive, DhfEntry};
pub use error::{AssetError, Result};
pub use fixed_point::Fix;
pub use geometry::{FixVector, Uvl};
pub use ham::{HamFile, RobotInfo, WeaponInfo};
pub use hog2::{Hog2Archive, Hog2Entry};
pub use io::ReadExt;
pub use level::{Level, Segment, Side};
pub use mission::{MissionFile, SecretLevel};
pub use mve::{MveChunk, MveFile, MveSegment, MveSegmentType};
pub use mvl::{MvlArchive, MvlEntry};
pub use ogf::{OgfHeader, OgfTexture, PixelFormat, TextureFlags};
pub use palette::Palette;
pub use pig::{BitmapData, BitmapFlags, BitmapHeader, PigFile};
pub use player::{
    CALLSIGN_LEN, COMPATIBLE_PLAYER_FILE_VERSION, PlayerProfile, PlrProfile, PlxProfile,
};
pub use pof::{
    FlatPolygon, GlowPoint, Opcode, PofModel, PofParser, Polygon, RodBitmap, SubmodelCall,
    TexturedPolygon,
};
pub use sound::{HmpFile, HmpTrack, SoundData, SoundHeader};

//! Format converters for converting Descent legacy formats to modern formats.
//!
//! This module provides converters for:
//! - **Textures**: PIG (8-bit indexed) and OGF (D3 textures) → PNG, WebP
//! - **Models**: POF → glTF 2.0 / GLB
//! - **Audio**: SNDs → WAV, HMP → MIDI (future)
//! - **Levels**: RDL/RL2 → glTF scenes (future)
//!
//! # Examples
//!
//! ## Converting PIG Textures to PNG
//!
//! ```no_run
//! use descent_core::pig::PigFile;
//! use descent_core::palette::Palette;
//! use descent_core::converters::texture::TextureConverter;
//! use std::fs;
//!
//! // Load PIG file and palette
//! let pig_data = fs::read("descent2.pig").unwrap();
//! let pig = PigFile::parse(pig_data, false).unwrap();
//!
//! let palette_data = fs::read("groupa.256").unwrap();
//! let palette = Palette::parse(&palette_data).unwrap();
//!
//! // Convert a texture to PNG
//! let converter = TextureConverter::new(&palette);
//! let png_data = converter.pig_to_png(&pig, "wall01-0").unwrap();
//! fs::write("wall01-0.png", png_data).unwrap();
//! ```
//!
//! ## Converting POF Models to GLB
//!
//! ```no_run
//! use descent_core::pof::PofParser;
//! use descent_core::converters::model::ModelConverter;
//! use std::fs;
//!
//! // Load POF file
//! let pof_data = fs::read("pyrogl.pof").unwrap();
//! let model = PofParser::parse(&pof_data).unwrap();
//!
//! // Convert to GLB (geometry only, no textures)
//! let converter = ModelConverter::new();
//! let glb_data = converter.pof_to_glb(&model, "Pyro-GL Ship", None).unwrap();
//! fs::write("pyrogl.glb", glb_data).unwrap();
//! ```

pub mod model;
pub mod texture;

// Re-export main types
pub use model::ModelConverter;
pub use texture::{ImageFormat, TextureConvertError, TextureConverter};

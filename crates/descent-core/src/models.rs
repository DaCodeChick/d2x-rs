//! 3D model file format parsers (POF/OOF/ASE)
//!
//! This module provides a unified interface for parsing 3D model files across
//! different Descent game versions:
//!
//! - **POF**: Descent 1 & 2 polygon object format
//! - **ASE**: ASCII Scene Export format (D2X-XL high-res models, requires `hires-assets` feature)
//! - **OOF**: Descent 3 Outrage object format (future)
//!
//! # Examples
//!
//! ## Parsing a POF model
//!
//! ```no_run
//! use descent_core::models::Model;
//! use std::fs;
//!
//! let data = fs::read("pyro-gl.pof").unwrap();
//! let model = Model::from_pof(&data).unwrap();
//!
//! println!("Model has {} subobjects", model.subobject_count());
//! ```
//!
//! ## Parsing an ASE model
//!
//! ```no_run
//! # #[cfg(feature = "hires-assets")]
//! # {
//! use descent_core::models::Model;
//! use std::fs;
//!
//! let data = fs::read_to_string("pyro-hires.ase").unwrap();
//! let model = Model::from_ase(&data).unwrap();
//!
//! println!("Model has {} objects", model.subobject_count());
//! # }
//! ```
//!
//! ## Converting to glTF
//!
//! ```no_run
//! use descent_core::models::Model;
//! use descent_core::converters::model::ModelConverter;
//! use std::fs;
//!
//! let data = fs::read("pyro-gl.pof").unwrap();
//! let model = Model::from_pof(&data).unwrap();
//!
//! // Convert to glTF
//! let converter = ModelConverter::new();
//! let glb = converter.pof_to_glb(model.as_pof().unwrap(), "Pyro-GL", None).unwrap();
//! fs::write("pyro-gl.glb", glb).unwrap();
//! ```

#[cfg(feature = "hires-assets")]
use crate::ase::AseFile;
use crate::error::Result;
use crate::pof::PofModel;

/// A 3D model from any supported format.
///
/// This is a unified wrapper around different model formats (POF, ASE, OOF, etc.).
/// Use the format-specific methods to access the underlying data.
///
/// # Feature Requirements
///
/// - `Ase` variant requires the `hires-assets` feature
#[derive(Debug)]
pub enum Model {
    /// Descent 1/2 POF (Polygon Object Format) model
    Pof(PofModel),
    /// D2X-XL ASE (ASCII Scene Export) high-resolution model
    #[cfg(feature = "hires-assets")]
    Ase(AseFile),
    /// Descent 3 OOF (Outrage Object Format) model - not yet implemented
    #[allow(dead_code)]
    Oof,
}

impl Model {
    /// Parse a POF (Descent 1/2) model from binary data.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw POF file data
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use descent_core::models::Model;
    /// # let data = vec![];
    /// let model = Model::from_pof(&data).unwrap();
    /// ```
    pub fn from_pof(data: &[u8]) -> Result<Self> {
        let pof = crate::pof::PofParser::parse(data)?;
        Ok(Self::Pof(pof))
    }

    /// Parse an ASE (D2X-XL) model from text data.
    ///
    /// Requires the `hires-assets` feature to be enabled.
    ///
    /// # Arguments
    ///
    /// * `data` - ASE file content as string
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "hires-assets")]
    /// # {
    /// # use descent_core::models::Model;
    /// # let data = String::new();
    /// let model = Model::from_ase(&data).unwrap();
    /// # }
    /// ```
    #[cfg(feature = "hires-assets")]
    pub fn from_ase(data: &str) -> Result<Self> {
        let ase = AseFile::parse(data)?;
        Ok(Self::Ase(ase))
    }

    /// Detect model format and parse accordingly.
    ///
    /// Automatically detects the format based on file signature/content and parses.
    /// For text files (if `hires-assets` feature enabled), attempts to parse as ASE.
    /// Otherwise, attempts to parse as binary POF.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw model file data
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use descent_core::models::Model;
    /// # let data = vec![];
    /// let model = Model::parse(data).unwrap();
    /// ```
    pub fn parse(data: Vec<u8>) -> Result<Self> {
        // Try ASE first (text file starting with *3DSMAX_ASCIIEXPORT)
        #[cfg(feature = "hires-assets")]
        if let Ok(text) = std::str::from_utf8(&data) {
            if text.trim_start().starts_with("*3DSMAX_ASCIIEXPORT") {
                return Self::from_ase(text);
            }
        }

        // Try POF (binary opcode format)
        // POF files don't have a signature, just raw opcodes
        Self::from_pof(&data)
    }

    /// Get the underlying POF model, if this is a POF.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use descent_core::models::Model;
    /// # let data = vec![b'P', b'S', b'P', b'O', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    /// let model = Model::from_pof(&data).unwrap();
    /// if let Some(pof) = model.as_pof() {
    ///     println!("POF has {} vertices", pof.vertices.len());
    /// }
    /// ```
    pub fn as_pof(&self) -> Option<&PofModel> {
        match self {
            Self::Pof(pof) => Some(pof),
            _ => None,
        }
    }

    /// Get the underlying ASE model, if this is an ASE.
    ///
    /// Requires the `hires-assets` feature to be enabled.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # #[cfg(feature = "hires-assets")]
    /// # {
    /// # use descent_core::models::Model;
    /// # let data = "*3DSMAX_ASCIIEXPORT 200\n";
    /// let model = Model::from_ase(data).unwrap();
    /// if let Some(ase) = model.as_ase() {
    ///     println!("ASE has {} objects", ase.geom_objects.len());
    /// }
    /// # }
    /// ```
    #[cfg(feature = "hires-assets")]
    pub fn as_ase(&self) -> Option<&AseFile> {
        match self {
            Self::Ase(ase) => Some(ase),
            _ => None,
        }
    }

    /// Get the number of subobjects in the model.
    ///
    /// Subobjects represent individual parts (ship hull, wings, turrets, etc.).
    pub fn subobject_count(&self) -> usize {
        match self {
            Self::Pof(pof) => pof.submodel_calls.len(),
            #[cfg(feature = "hires-assets")]
            Self::Ase(ase) => ase.geom_objects.len(),
            Self::Oof => 0,
        }
    }

    /// Get the model format type as a string.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use descent_core::models::Model;
    /// # let data = vec![b'P', b'S', b'P', b'O', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    /// let model = Model::from_pof(&data).unwrap();
    /// assert_eq!(model.format_type(), "POF");
    /// ```
    pub fn format_type(&self) -> &'static str {
        match self {
            Self::Pof(_) => "POF",
            #[cfg(feature = "hires-assets")]
            Self::Ase(_) => "ASE",
            Self::Oof => "OOF",
        }
    }
}

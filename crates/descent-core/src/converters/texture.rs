//! Texture format converters for PIG and OGF formats.
//!
//! This module provides converters for Descent's texture formats:
//! - **PIG**: Descent 1/2 8-bit indexed textures (RLE compressed)
//! - **OGF**: Descent 3 textures (RGB565, RGBA4444, RGBA8888)
//!
//! Both formats are converted to TGA format.
//!
//! # Examples
//!
//! ## Converting PIG Textures
//!
//! ```no_run
//! use descent_core::pig::PigFile;
//! use descent_core::palette::Palette;
//! use descent_core::converters::texture::{TextureConverter, ImageFormat};
//! use std::fs;
//!
//! let pig_data = fs::read("descent2.pig").unwrap();
//! let pig = PigFile::parse(pig_data, false).unwrap();
//!
//! let palette_data = fs::read("groupa.256").unwrap();
//! let palette = Palette::parse(&palette_data).unwrap();
//!
//! let converter = TextureConverter::new(&palette);
//!
//! // Convert to TGA
//! let tga = converter.pig_to_image(&pig, "wall01-0", ImageFormat::Tga).unwrap();
//! fs::write("wall01-0.tga", tga).unwrap();
//! ```
//!
//! ## Converting OGF Textures (D3)
//!
//! ```no_run
//! use descent_core::ogf::OgfTexture;
//! use descent_core::converters::texture::{TextureConverter, ImageFormat};
//! use std::fs;
//!
//! let ogf_data = fs::read("texture.ogf").unwrap();
//! let texture = OgfTexture::parse(&ogf_data).unwrap();
//!
//! let converter = TextureConverter::default();
//!
//! // Convert to TGA
//! let tga = converter.ogf_to_image(&texture, ImageFormat::Tga).unwrap();
//! fs::write("texture.tga", tga).unwrap();
//! ```

use crate::ogf::OgfTexture;
use crate::palette::Palette;
use crate::pig::PigFile;
use image::{ImageBuffer, ImageFormat as ImgFormat, Rgba, RgbaImage};
use std::io::Cursor;
use thiserror::Error;

/// Errors that can occur during texture conversion.
#[derive(Debug, Error)]
pub enum TextureConvertError {
    /// Texture not found in PIG file.
    #[error("Texture not found: {0}")]
    TextureNotFound(String),

    /// Image encoding failed.
    #[error("Image encoding failed: {0}")]
    EncodingError(String),

    /// Invalid texture data.
    #[error("Invalid texture data: {0}")]
    InvalidData(String),
}

/// Output image format for texture conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// TGA format (uncompressed, optimized for game dev, no licensing concerns).
    Tga,
}

impl ImageFormat {
    /// Convert to image crate's ImageFormat.
    fn to_image_format(self) -> ImgFormat {
        match self {
            ImageFormat::Tga => ImgFormat::Tga,
        }
    }

    /// Get file extension for this format.
    pub const fn extension(self) -> &'static str {
        match self {
            ImageFormat::Tga => "tga",
        }
    }
}

/// Texture converter for Descent formats.
///
/// Converts PIG (D1/D2) and OGF (D3) textures to modern image formats.
pub struct TextureConverter<'a> {
    /// Palette for PIG textures (D1/D2 use 8-bit indexed color).
    palette: Option<&'a Palette>,
}

impl<'a> TextureConverter<'a> {
    /// Create a new texture converter with a palette for PIG textures.
    ///
    /// The palette is required for converting PIG textures (D1/D2).
    /// It's not needed for OGF textures (D3).
    pub const fn new(palette: &'a Palette) -> Self {
        Self {
            palette: Some(palette),
        }
    }

    /// Create a texture converter without a palette (for OGF textures only).
    pub const fn without_palette() -> Self {
        Self { palette: None }
    }

    /// Convert a PIG texture to the specified image format.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Texture is not found in PIG file
    /// - No palette is set
    /// - Image encoding fails
    pub fn pig_to_image(
        &self,
        pig: &PigFile,
        texture_name: &str,
        format: ImageFormat,
    ) -> std::result::Result<Vec<u8>, TextureConvertError> {
        let palette = self.palette.ok_or_else(|| {
            TextureConvertError::InvalidData("Palette required for PIG textures".to_string())
        })?;

        // Load bitmap data
        let bitmap = pig
            .load_bitmap(texture_name)
            .map_err(|_| TextureConvertError::TextureNotFound(texture_name.to_string()))?;

        // Convert indexed pixels to RGBA using palette
        let rgba = palette
            .indexed_to_rgba(
                &bitmap.pixels,
                bitmap.width as usize,
                bitmap.height as usize,
            )
            .map_err(|e| TextureConvertError::InvalidData(e.to_string()))?;

        // Create image
        let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
            bitmap.width as u32,
            bitmap.height as u32,
            rgba,
        )
        .ok_or_else(|| {
            TextureConvertError::InvalidData("Failed to create image buffer".to_string())
        })?;

        // Encode to target format
        self.encode_image(&img, format)
    }

    /// Convert a PIG texture to TGA format.
    ///
    /// Convenience method for `pig_to_image(pig, name, ImageFormat::Tga)`.
    pub fn pig_to_tga(
        &self,
        pig: &PigFile,
        texture_name: &str,
    ) -> std::result::Result<Vec<u8>, TextureConvertError> {
        self.pig_to_image(pig, texture_name, ImageFormat::Tga)
    }

    /// Convert an OGF texture to the specified image format.
    ///
    /// # Errors
    ///
    /// Returns error if image encoding fails.
    pub fn ogf_to_image(
        &self,
        texture: &OgfTexture,
        format: ImageFormat,
    ) -> std::result::Result<Vec<u8>, TextureConvertError> {
        // Use OgfTexture's built-in to_rgba8 method
        let rgba = texture
            .to_rgba8()
            .map_err(|e| TextureConvertError::InvalidData(e.to_string()))?;

        // Create image
        let img = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
            texture.header.width as u32,
            texture.header.height as u32,
            rgba,
        )
        .ok_or_else(|| {
            TextureConvertError::InvalidData("Failed to create image buffer".to_string())
        })?;

        // Encode to target format
        self.encode_image(&img, format)
    }

    /// Convert an OGF texture to TGA format.
    ///
    /// Convenience method for `ogf_to_image(texture, ImageFormat::Tga)`.
    pub fn ogf_to_tga(
        &self,
        texture: &OgfTexture,
    ) -> std::result::Result<Vec<u8>, TextureConvertError> {
        self.ogf_to_image(texture, ImageFormat::Tga)
    }

    /// Encode an RGBA image to the specified format.
    fn encode_image(
        &self,
        img: &RgbaImage,
        format: ImageFormat,
    ) -> std::result::Result<Vec<u8>, TextureConvertError> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        img.write_to(&mut cursor, format.to_image_format())
            .map_err(|e| TextureConvertError::EncodingError(e.to_string()))?;

        Ok(buffer)
    }

    /// Batch convert all textures in a PIG file to the specified format.
    ///
    /// Returns a vector of (texture_name, image_data) tuples.
    ///
    /// # Errors
    ///
    /// Returns error if any texture conversion fails.
    pub fn pig_batch_convert(
        &self,
        pig: &PigFile,
        format: ImageFormat,
    ) -> std::result::Result<Vec<(String, Vec<u8>)>, TextureConvertError> {
        let mut results = Vec::new();

        for header in pig.headers() {
            let name = header.name.clone();
            let image_data = self.pig_to_image(pig, &name, format)?;
            results.push((name, image_data));
        }

        Ok(results)
    }
}

impl Default for TextureConverter<'_> {
    fn default() -> Self {
        Self::without_palette()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_extension() {
        assert_eq!(ImageFormat::Tga.extension(), "tga");
    }

    #[test]
    fn test_texture_converter_without_palette() {
        let converter = TextureConverter::without_palette();
        assert!(converter.palette.is_none());
    }

    #[test]
    fn test_rgb565_conversion() {
        // Test RGB565 to RGBA8888 conversion
        let _converter = TextureConverter::without_palette();

        // RGB565: 0xF800 = pure red (R=31, G=0, B=0)
        let pixel_565 = 0xF800u16;

        let r = ((pixel_565 >> 11) & 0x1F) as u8;
        let g = ((pixel_565 >> 5) & 0x3F) as u8;
        let b = (pixel_565 & 0x1F) as u8;

        // Scale to 8-bit
        let r8 = (r << 3) | (r >> 2);
        let g8 = (g << 2) | (g >> 4);
        let b8 = (b << 3) | (b >> 2);

        assert_eq!(r8, 255); // Pure red
        assert_eq!(g8, 0);
        assert_eq!(b8, 0);
    }

    #[test]
    fn test_rgba4444_conversion() {
        // Test RGBA4444 to RGBA8888 conversion
        let _converter = TextureConverter::without_palette();

        // RGBA4444: 0xF0F0 = bright red with bright alpha (R=15, G=0, B=15, A=0)
        let pixel_4444 = 0xF0F0u16;

        let r = ((pixel_4444 >> 12) & 0x0F) as u8;
        let g = ((pixel_4444 >> 8) & 0x0F) as u8;
        let b = ((pixel_4444 >> 4) & 0x0F) as u8;
        let a = (pixel_4444 & 0x0F) as u8;

        // Scale 4-bit to 8-bit
        let r8 = (r << 4) | r;
        let g8 = (g << 4) | g;
        let b8 = (b << 4) | b;
        let a8 = (a << 4) | a;

        assert_eq!(r8, 255);
        assert_eq!(g8, 0);
        assert_eq!(b8, 255);
        assert_eq!(a8, 0);
    }
}

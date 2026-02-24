//! Palette file format parser
//!
//! Palette files (`.256` extension) contain color palettes for Descent 1 and 2. They define the
//! RGB color mapping for 8-bit indexed color bitmaps stored in PIG files.
//!
//! **File Format**:
//! - 256 RGB triplets (768 bytes)
//! - Fade table (256 * MAX_FADE_LEVELS bytes, optional)
//! - Color values are 6-bit (0-63), not 8-bit!
//!
//! **Important**: Index 255 is typically used as the transparency color.
//!
//! # Format Documentation
//!
//! See `docs/formats/HAM_FORMAT.md` (Palette Files section) for complete specification.
//!
//! # Reference Implementation
//!
//! D2X-XL v1.18.77:
//! - `include/palette.h` - CPalette class
//! - `2d/palette.cpp` - Palette loading and conversion
//!
//! # Examples
//!
//! ```no_run
//! # use d2x_assets::palette::Palette;
//! # use d2x_assets::error::Result;
//! # fn example() -> Result<()> {
//! // Load Descent 2 palette
//! let data = std::fs::read("groupa.256")?;
//! let palette = Palette::parse(&data)?;
//!
//! // Convert 6-bit color to 8-bit RGB
//! let rgb = palette.get_rgb8(42);
//! println!("Color 42: R={}, G={}, B={}", rgb[0], rgb[1], rgb[2]);
//!
//! // Check transparency
//! println!("Transparency index: {}", Palette::transparent_index());
//! # Ok(())
//! # }
//! ```

use crate::error::{AssetError, Result};

/// Palette size (number of colors)
pub const PALETTE_SIZE: usize = 256;

/// Transparent color index (usually 255)
pub const TRANSPARENCY_INDEX: u8 = 255;

/// Super-transparency index (usually 254)
pub const SUPER_TRANSP_INDEX: u8 = 254;

// ============================================================================
// Core Structures
// ============================================================================

/// Color palette containing 256 RGB triplets.
///
/// Colors are stored in 6-bit format (0-63). Use `get_rgb8()` to convert to
/// standard 8-bit RGB (0-255).
#[derive(Debug, Clone)]
pub struct Palette {
    /// 256 RGB colors (6-bit values: 0-63)
    colors: [[u8; 3]; PALETTE_SIZE],
}

impl Palette {
    /// Parse a palette from raw bytes.
    ///
    /// Expects at least 768 bytes (256 * 3). Extra bytes (fade table) are ignored.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Data is less than 768 bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        const PALETTE_BYTES: usize = PALETTE_SIZE * 3;

        if data.len() < PALETTE_BYTES {
            return Err(AssetError::InvalidPigFormat(format!(
                "Palette data too short: expected at least {} bytes, got {}",
                PALETTE_BYTES,
                data.len()
            )));
        }

        let mut colors = [[0u8; 3]; PALETTE_SIZE];

        for (i, color) in colors.iter_mut().enumerate() {
            let offset = i * 3;
            color[0] = data[offset]; // Red
            color[1] = data[offset + 1]; // Green
            color[2] = data[offset + 2]; // Blue

            // Validate 6-bit color values
            if color[0] > 63 || color[1] > 63 || color[2] > 63 {
                return Err(AssetError::InvalidPigFormat(format!(
                    "Invalid color at index {}: RGB({}, {}, {}) exceeds 6-bit range (0-63)",
                    i, color[0], color[1], color[2]
                )));
            }
        }

        Ok(Self { colors })
    }

    /// Get a color as 6-bit RGB (0-63).
    ///
    /// # Panics
    ///
    /// Panics if index >= 256.
    pub const fn get_rgb6(&self, index: u8) -> [u8; 3] {
        self.colors[index as usize]
    }

    /// Get a color as 8-bit RGB (0-255).
    ///
    /// Converts 6-bit color values (0-63) to 8-bit (0-255) using the formula:
    /// `rgb8 = (rgb6 * 255) / 63`
    ///
    /// # Panics
    ///
    /// Panics if index >= 256.
    pub fn get_rgb8(&self, index: u8) -> [u8; 3] {
        let rgb6 = self.get_rgb6(index);
        [
            scale_6bit_to_8bit(rgb6[0]),
            scale_6bit_to_8bit(rgb6[1]),
            scale_6bit_to_8bit(rgb6[2]),
        ]
    }

    /// Get a color as 32-bit RGBA (0-255 per channel).
    ///
    /// The alpha channel is set to 255 (opaque) unless the index is the
    /// transparency index (255) or super-transparency index (254), in which
    /// case alpha is set to 0.
    ///
    /// # Panics
    ///
    /// Panics if index >= 256.
    pub fn get_rgba8(&self, index: u8) -> [u8; 4] {
        let rgb8 = self.get_rgb8(index);
        let alpha = if index == TRANSPARENCY_INDEX || index == SUPER_TRANSP_INDEX {
            0
        } else {
            255
        };
        [rgb8[0], rgb8[1], rgb8[2], alpha]
    }

    /// Get transparency color index (usually 255).
    pub const fn transparent_index() -> u8 {
        TRANSPARENCY_INDEX
    }

    /// Get super-transparency color index (usually 254).
    pub const fn super_transparent_index() -> u8 {
        SUPER_TRANSP_INDEX
    }

    /// Returns all 256 colors as 6-bit RGB triplets.
    pub const fn colors(&self) -> &[[u8; 3]; PALETTE_SIZE] {
        &self.colors
    }

    /// Convert an entire indexed bitmap to RGBA.
    ///
    /// # Arguments
    ///
    /// * `indexed` - Input indexed color bitmap (8-bit per pixel)
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    ///
    /// # Returns
    ///
    /// RGBA bitmap (32-bit per pixel). Size = width * height * 4 bytes.
    ///
    /// # Errors
    ///
    /// Returns error if indexed data length doesn't match width * height.
    pub fn indexed_to_rgba(&self, indexed: &[u8], width: usize, height: usize) -> Result<Vec<u8>> {
        let expected_len = width * height;
        if indexed.len() != expected_len {
            return Err(AssetError::InvalidPigFormat(format!(
                "Indexed data length {} doesn't match dimensions {}x{} (expected {} bytes)",
                indexed.len(),
                width,
                height,
                expected_len
            )));
        }

        let mut rgba = Vec::with_capacity(expected_len * 4);

        for &pixel_index in indexed {
            let color = self.get_rgba8(pixel_index);
            rgba.extend_from_slice(&color);
        }

        Ok(rgba)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Scale 6-bit color value (0-63) to 8-bit (0-255).
///
/// Uses precise formula: `(value * 255) / 63`
const fn scale_6bit_to_8bit(value: u8) -> u8 {
    // (value * 255) / 63
    // We use 16-bit intermediate to avoid overflow
    ((value as u16 * 255) / 63) as u8
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_6bit_to_8bit() {
        assert_eq!(scale_6bit_to_8bit(0), 0);
        assert_eq!(scale_6bit_to_8bit(63), 255);
        assert_eq!(scale_6bit_to_8bit(31), 125); // Approximately half
        assert_eq!(scale_6bit_to_8bit(32), 129); // Slightly above half
    }

    #[test]
    fn test_palette_parse() {
        // Create a minimal valid palette (768 bytes)
        let mut data = vec![0u8; 768];

        // Set color 0 to black (0, 0, 0)
        data[0] = 0;
        data[1] = 0;
        data[2] = 0;

        // Set color 1 to white (63, 63, 63)
        data[3] = 63;
        data[4] = 63;
        data[5] = 63;

        // Set color 255 to red (63, 0, 0)
        data[255 * 3] = 63;
        data[255 * 3 + 1] = 0;
        data[255 * 3 + 2] = 0;

        let palette = Palette::parse(&data).unwrap();

        assert_eq!(palette.get_rgb6(0), [0, 0, 0]);
        assert_eq!(palette.get_rgb6(1), [63, 63, 63]);
        assert_eq!(palette.get_rgb6(255), [63, 0, 0]);
    }

    #[test]
    fn test_palette_rgb8_conversion() {
        let mut data = vec![0u8; 768];

        // Set color 42 to max RGB (63, 63, 63)
        data[42 * 3] = 63;
        data[42 * 3 + 1] = 63;
        data[42 * 3 + 2] = 63;

        let palette = Palette::parse(&data).unwrap();
        let rgb8 = palette.get_rgb8(42);

        assert_eq!(rgb8, [255, 255, 255]);
    }

    #[test]
    fn test_palette_rgba8_transparency() {
        let data = vec![0u8; 768];
        let palette = Palette::parse(&data).unwrap();

        // Normal color should have alpha = 255
        let rgba = palette.get_rgba8(42);
        assert_eq!(rgba[3], 255);

        // Transparency index should have alpha = 0
        let rgba_transp = palette.get_rgba8(TRANSPARENCY_INDEX);
        assert_eq!(rgba_transp[3], 0);

        // Super-transparency index should have alpha = 0
        let rgba_super = palette.get_rgba8(SUPER_TRANSP_INDEX);
        assert_eq!(rgba_super[3], 0);
    }

    #[test]
    fn test_palette_too_short() {
        let data = vec![0u8; 767]; // One byte too short
        let result = Palette::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_palette_invalid_color() {
        let mut data = vec![0u8; 768];
        data[0] = 64; // Invalid: exceeds 6-bit range

        let result = Palette::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_indexed_to_rgba() {
        let mut palette_data = vec![0u8; 768];

        // Set color 0 to black (0, 0, 0)
        palette_data[0] = 0;
        palette_data[1] = 0;
        palette_data[2] = 0;

        // Set color 1 to white (63, 63, 63)
        palette_data[3] = 63;
        palette_data[4] = 63;
        palette_data[5] = 63;

        let palette = Palette::parse(&palette_data).unwrap();

        // Create a 2x2 indexed bitmap: [0, 1, 1, 0]
        let indexed = vec![0, 1, 1, 0];
        let rgba = palette.indexed_to_rgba(&indexed, 2, 2).unwrap();

        // Check converted RGBA
        assert_eq!(rgba.len(), 16); // 2*2 pixels * 4 bytes

        // Pixel 0: black with alpha
        assert_eq!(&rgba[0..4], &[0, 0, 0, 255]);

        // Pixel 1: white with alpha
        assert_eq!(&rgba[4..8], &[255, 255, 255, 255]);

        // Pixel 2: white with alpha
        assert_eq!(&rgba[8..12], &[255, 255, 255, 255]);

        // Pixel 3: black with alpha
        assert_eq!(&rgba[12..16], &[0, 0, 0, 255]);
    }

    #[test]
    fn test_indexed_to_rgba_wrong_size() {
        let data = vec![0u8; 768];
        let palette = Palette::parse(&data).unwrap();

        let indexed = vec![0u8; 10];
        let result = palette.indexed_to_rgba(&indexed, 4, 4); // 4*4 = 16, but we have 10
        assert!(result.is_err());
    }
}

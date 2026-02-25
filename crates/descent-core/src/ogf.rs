//! OGF texture format parser (Descent 3)
//!
//! OGF (Outrage Graphics Format) is the proprietary texture format used by Descent 3.
//!
//! ## Format Features
//! - True color (16/32-bit) vs D1/D2's 8-bit indexed
//! - Multiple sizes: 32×32, 64×64, 128×128, 256×256
//! - Color formats: RGB565, RGBA4444, RGBA8888
//! - Mipmaps for LOD
//! - Animation support
//! - Procedural textures (fire, water, plasma)
//! - 31 texture property flags

use crate::error::{AssetError, Result};
use bitflags::bitflags;

bitflags! {
    /// Texture property flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct TextureFlags: u32 {
        /// Water surface effect
        const WATER         = 1 << 0;
        /// Lava/damage surface
        const LAVA          = 1 << 1;
        /// Metallic surface properties
        const METAL         = 1 << 2;
        /// Alpha transparency
        const ALPHA         = 1 << 3;
        /// Animated texture (frame sequence)
        const ANIMATED      = 1 << 4;
        /// Procedurally generated at runtime
        const PROCEDURAL    = 1 << 5;
        /// Light-emitting surface
        const LIGHT         = 1 << 6;
        /// Can be destroyed/broken
        const BREAKABLE     = 1 << 7;
        /// Secondary texture layer (multitexturing)
        const TMAP2         = 1 << 8;
        /// Force field effect
        const FORCEFIELD    = 1 << 9;
        /// Saturated colors
        const SATURATE      = 1 << 10;
        /// Smooth rendering
        const SMOOTH        = 1 << 11;
        /// Brightness multiplier applied
        const BRIGHTNESS    = 1 << 12;
        /// Bumpmap/normal map data
        const BUMPMAP       = 1 << 13;
        /// Fog effect applied
        const FOGGED        = 1 << 14;
        /// Texture scrolls (animated UV)
        const SCROLLING     = 1 << 15;
        /// Transparent texture
        const TRANSPARENT   = 1 << 16;
        /// Ping-pong animation (forward then reverse)
        const PINGPONG      = 1 << 17;
        /// Coronas/glow effect
        const CORONA        = 1 << 18;
        /// Mipmaps included
        const MIPMAPS       = 1 << 19;
        /// 4444 color format
        const FORMAT_4444   = 1 << 20;
        /// Texture is compressed
        const COMPRESSED    = 1 << 21;
    }
}

/// Pixel format for OGF textures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// RGB 5-6-5 (16-bit)
    Rgb565,
    /// RGBA 4-4-4-4 (16-bit)
    Rgba4444,
    /// RGBA 8-8-8-8 (32-bit)
    Rgba8888,
    /// 8-bit indexed (rarely used in D3)
    Indexed8,
}

impl PixelFormat {
    /// Get bytes per pixel for this format
    pub const fn bytes_per_pixel(&self) -> usize {
        match self {
            Self::Rgb565 | Self::Rgba4444 => 2,
            Self::Rgba8888 => 4,
            Self::Indexed8 => 1,
        }
    }

    /// Check if format has alpha channel
    pub const fn has_alpha(&self) -> bool {
        matches!(self, Self::Rgba4444 | Self::Rgba8888)
    }
}

impl TryFrom<u8> for PixelFormat {
    type Error = AssetError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Rgb565),
            1 => Ok(Self::Rgba4444),
            2 => Ok(Self::Rgba8888),
            3 => Ok(Self::Indexed8),
            _ => Err(AssetError::InvalidFormat(format!(
                "Unknown pixel format: {}",
                value
            ))),
        }
    }
}

/// OGF texture header
#[derive(Debug, Clone)]
pub struct OgfHeader {
    /// Format version (typically 1 or 2)
    pub version: u32,
    /// Texture width in pixels
    pub width: u16,
    /// Texture height in pixels
    pub height: u16,
    /// Pixel format
    pub format: PixelFormat,
    /// Texture property flags
    pub flags: TextureFlags,
    /// Number of mipmap levels (0 = no mipmaps, just base image)
    pub num_mipmaps: u8,
    /// Number of animation frames (1 = static texture)
    pub num_frames: u16,
    /// Animation speed in FPS (0 = not animated)
    pub fps: f32,
}

impl OgfHeader {
    /// Calculate the size of pixel data for the base texture (mip level 0)
    pub const fn base_data_size(&self) -> usize {
        self.width as usize * self.height as usize * self.format.bytes_per_pixel()
    }

    /// Calculate size of a specific mipmap level
    pub fn mipmap_data_size(&self, level: u8) -> usize {
        let divisor = 1 << level; // 2^level
        let mip_width = (self.width as usize).max(1) / divisor;
        let mip_height = (self.height as usize).max(1) / divisor;
        mip_width * mip_height * self.format.bytes_per_pixel()
    }

    /// Check if texture is animated
    pub const fn is_animated(&self) -> bool {
        self.num_frames > 1
    }

    /// Check if texture has mipmaps
    pub const fn has_mipmaps(&self) -> bool {
        self.num_mipmaps > 0
    }
}

/// OGF texture with pixel data
#[derive(Debug, Clone)]
pub struct OgfTexture {
    /// Texture header
    pub header: OgfHeader,
    /// Raw pixel data (base texture + mipmaps)
    pub data: Vec<u8>,
}

impl OgfTexture {
    /// Parse an OGF texture from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 32 {
            return Err(AssetError::InvalidFormat(
                "OGF file too short for header".to_string(),
            ));
        }

        let mut offset = 0;

        // Read version (4 bytes)
        let version = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        offset += 4;

        // Read dimensions (4 bytes)
        let width = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;
        let height = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        // Read format (1 byte)
        let format = PixelFormat::try_from(data[offset])?;
        offset += 1;

        // Read flags (4 bytes)
        let flags_bits = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        let flags = TextureFlags::from_bits_truncate(flags_bits);
        offset += 4;

        // Read mipmap count (1 byte)
        let num_mipmaps = data[offset];
        offset += 1;

        // Read animation info (6 bytes)
        let num_frames = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;
        let fps = f32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);
        // offset += 4; // Not needed, we start at byte 32 for pixel data

        // Padding to align to 32 bytes (skip remaining header bytes)
        offset = 32;

        let header = OgfHeader {
            version,
            width,
            height,
            format,
            flags,
            num_mipmaps,
            num_frames,
            fps,
        };

        // Calculate expected data size
        let base_size = header.base_data_size();
        let mipmap_sizes: usize = (1..=num_mipmaps)
            .map(|level| header.mipmap_data_size(level))
            .sum();
        let total_size = (base_size + mipmap_sizes) * num_frames as usize;

        if data.len() < offset + total_size {
            return Err(AssetError::InvalidFormat(format!(
                "OGF data truncated: expected {} bytes, got {}",
                offset + total_size,
                data.len()
            )));
        }

        // Extract pixel data
        let pixel_data = data[offset..offset + total_size].to_vec();

        Ok(Self {
            header,
            data: pixel_data,
        })
    }

    /// Get the base texture data (mipmap level 0, frame 0)
    pub fn base_texture(&self) -> &[u8] {
        let size = self.header.base_data_size();
        &self.data[..size]
    }

    /// Get a specific mipmap level (0 = base texture)
    pub fn get_mipmap(&self, level: u8) -> Result<&[u8]> {
        if level > self.header.num_mipmaps {
            return Err(AssetError::NotFound(format!(
                "Mipmap level {} not available (max: {})",
                level, self.header.num_mipmaps
            )));
        }

        // Skip previous mipmap levels
        let offset: usize = (0..level).map(|l| self.header.mipmap_data_size(l)).sum();
        let size = self.header.mipmap_data_size(level);
        Ok(&self.data[offset..offset + size])
    }

    /// Get a specific animation frame (0 = first frame)
    pub fn get_frame(&self, frame: u16) -> Result<&[u8]> {
        if frame >= self.header.num_frames {
            return Err(AssetError::NotFound(format!(
                "Frame {} not available (max: {})",
                frame,
                self.header.num_frames - 1
            )));
        }

        // Calculate size of one complete frame (base + all mipmaps)
        let frame_size = self.header.base_data_size()
            + (1..=self.header.num_mipmaps)
                .map(|level| self.header.mipmap_data_size(level))
                .sum::<usize>();

        let offset = frame as usize * frame_size;
        Ok(&self.data[offset..offset + frame_size])
    }

    /// Convert texture to RGBA8 format (base texture, frame 0)
    pub fn to_rgba8(&self) -> Result<Vec<u8>> {
        let width = self.header.width as usize;
        let height = self.header.height as usize;
        let base_data = self.base_texture();

        match self.header.format {
            PixelFormat::Rgb565 => Self::rgb565_to_rgba8(base_data, width, height),
            PixelFormat::Rgba4444 => Self::rgba4444_to_rgba8(base_data, width, height),
            PixelFormat::Rgba8888 => Ok(base_data.to_vec()),
            PixelFormat::Indexed8 => Err(AssetError::InvalidFormat(
                "Indexed format not supported (requires palette)".to_string(),
            )),
        }
    }

    /// Convert RGB565 to RGBA8888
    fn rgb565_to_rgba8(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>> {
        let pixel_count = width * height;
        if data.len() < pixel_count * 2 {
            return Err(AssetError::InvalidFormat(
                "RGB565 data truncated".to_string(),
            ));
        }

        let rgba = (0..pixel_count)
            .flat_map(|i| {
                let offset = i * 2;
                let pixel = u16::from_le_bytes([data[offset], data[offset + 1]]);

                // Extract RGB565 components
                let r5 = ((pixel >> 11) & 0x1F) as u8;
                let g6 = ((pixel >> 5) & 0x3F) as u8;
                let b5 = (pixel & 0x1F) as u8;

                // Scale to 8-bit
                let r = (r5 << 3) | (r5 >> 2); // 5-bit to 8-bit
                let g = (g6 << 2) | (g6 >> 4); // 6-bit to 8-bit
                let b = (b5 << 3) | (b5 >> 2); // 5-bit to 8-bit

                [r, g, b, 255]
            })
            .collect();

        Ok(rgba)
    }

    /// Convert RGBA4444 to RGBA8888
    fn rgba4444_to_rgba8(data: &[u8], width: usize, height: usize) -> Result<Vec<u8>> {
        let pixel_count = width * height;
        if data.len() < pixel_count * 2 {
            return Err(AssetError::InvalidFormat(
                "RGBA4444 data truncated".to_string(),
            ));
        }

        let rgba = (0..pixel_count)
            .flat_map(|i| {
                let offset = i * 2;
                let pixel = u16::from_le_bytes([data[offset], data[offset + 1]]);

                // Extract RGBA4444 components
                let r4 = ((pixel >> 12) & 0x0F) as u8;
                let g4 = ((pixel >> 8) & 0x0F) as u8;
                let b4 = ((pixel >> 4) & 0x0F) as u8;
                let a4 = (pixel & 0x0F) as u8;

                // Scale to 8-bit (duplicate nibble)
                let r = (r4 << 4) | r4;
                let g = (g4 << 4) | g4;
                let b = (b4 << 4) | b4;
                let a = (a4 << 4) | a4;

                [r, g, b, a]
            })
            .collect();

        Ok(rgba)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test OGF file with RGB565 format
    fn create_test_ogf_rgb565() -> Vec<u8> {
        let mut data = Vec::new();

        // Header (32 bytes)
        data.extend_from_slice(&1u32.to_le_bytes()); // version
        data.extend_from_slice(&4u16.to_le_bytes()); // width
        data.extend_from_slice(&4u16.to_le_bytes()); // height
        data.push(0); // format = RGB565
        data.extend_from_slice(&0u32.to_le_bytes()); // flags
        data.push(0); // num_mipmaps
        data.extend_from_slice(&1u16.to_le_bytes()); // num_frames
        data.extend_from_slice(&0f32.to_le_bytes()); // fps

        // Padding to 32 bytes
        while data.len() < 32 {
            data.push(0);
        }

        // Pixel data: 4x4 RGB565 pixels (32 bytes)
        // Red pixel (RGB565: 0xF800)
        (0..4).for_each(|_| data.extend_from_slice(&0xF800u16.to_le_bytes()));
        // Green pixel (RGB565: 0x07E0)
        (0..4).for_each(|_| data.extend_from_slice(&0x07E0u16.to_le_bytes()));
        // Blue pixel (RGB565: 0x001F)
        (0..4).for_each(|_| data.extend_from_slice(&0x001Fu16.to_le_bytes()));
        // White pixel (RGB565: 0xFFFF)
        (0..4).for_each(|_| data.extend_from_slice(&0xFFFFu16.to_le_bytes()));

        data
    }

    #[test]
    fn test_ogf_header_parsing() {
        let data = create_test_ogf_rgb565();
        let ogf = OgfTexture::parse(&data).unwrap();

        assert_eq!(ogf.header.version, 1);
        assert_eq!(ogf.header.width, 4);
        assert_eq!(ogf.header.height, 4);
        assert_eq!(ogf.header.format, PixelFormat::Rgb565);
        assert_eq!(ogf.header.num_frames, 1);
        assert!(!ogf.header.is_animated());
        assert!(!ogf.header.has_mipmaps());
    }

    #[test]
    fn test_ogf_base_texture() {
        let data = create_test_ogf_rgb565();
        let ogf = OgfTexture::parse(&data).unwrap();

        let base = ogf.base_texture();
        assert_eq!(base.len(), 4 * 4 * 2); // 4x4 pixels, 2 bytes each
    }

    #[test]
    fn test_rgb565_to_rgba8_conversion() {
        let data = create_test_ogf_rgb565();
        let ogf = OgfTexture::parse(&data).unwrap();

        let rgba = ogf.to_rgba8().unwrap();
        assert_eq!(rgba.len(), 4 * 4 * 4); // 4x4 pixels, 4 bytes each

        // Check first pixel (should be red: 0xF800 = R:31, G:0, B:0)
        // 5-bit 31 -> 8-bit: (31 << 3) | (31 >> 2) = 248 | 7 = 255
        assert_eq!(rgba[0], 255); // R (scaled from 5-bit 31)
        assert_eq!(rgba[1], 0); // G
        assert_eq!(rgba[2], 0); // B
        assert_eq!(rgba[3], 255); // A
    }

    #[test]
    fn test_texture_flags() {
        let water = TextureFlags::WATER;
        let metal = TextureFlags::METAL;

        assert!(water.contains(TextureFlags::WATER));
        assert!(!water.contains(TextureFlags::METAL));

        let combined = water | metal;
        assert!(combined.contains(TextureFlags::WATER));
        assert!(combined.contains(TextureFlags::METAL));
    }

    #[test]
    fn test_pixel_format_properties() {
        assert_eq!(PixelFormat::Rgb565.bytes_per_pixel(), 2);
        assert_eq!(PixelFormat::Rgba4444.bytes_per_pixel(), 2);
        assert_eq!(PixelFormat::Rgba8888.bytes_per_pixel(), 4);

        assert!(!PixelFormat::Rgb565.has_alpha());
        assert!(PixelFormat::Rgba4444.has_alpha());
        assert!(PixelFormat::Rgba8888.has_alpha());
    }

    #[test]
    fn test_invalid_format() {
        let result = PixelFormat::try_from(99);
        assert!(matches!(result, Err(AssetError::InvalidFormat(_))));
    }

    #[test]
    fn test_truncated_data() {
        let data = vec![0u8; 16]; // Too short for header
        let result = OgfTexture::parse(&data);
        assert!(matches!(result, Err(AssetError::InvalidFormat(_))));
    }
}

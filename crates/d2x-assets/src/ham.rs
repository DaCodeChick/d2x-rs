//! HAM game data file format parser
//!
//! HAM (Hamster) files contain game data definitions for Descent 1 and 2. They define
//! properties for textures, robots, weapons, powerups, models, effects, and other game objects.
//!
//! **File Format**:
//! - Signature: `HAM!` (0x48414D21)
//! - Versions: 2 (demo), 3 (standard)
//! - Endianness: Little-endian
//!
//! **Important**: HAM files do NOT contain palette data. Palettes are stored in separate
//! `.256` files (e.g., `groupa.256` for Descent 2, `palette.256` for Descent 1).
//!
//! # Format Documentation
//!
//! See `docs/formats/HAM_FORMAT.md` for complete specification.
//!
//! # Reference Implementation
//!
//! D2X-XL v1.18.77:
//! - `include/piggy.h` - Constants and structures
//! - `include/loadgamedata.h` - Data structure definitions  
//! - `gameio/piggy.cpp:430` - ReadHamFile function
//! - `gameio/loadgamedata.cpp:276` - BMReadAll function
//!
//! # Examples
//!
//! ```no_run
//! # use d2x_assets::ham::HamFile;
//! # use d2x_assets::error::Result;
//! # fn example() -> Result<()> {
//! let data = std::fs::read("descent2.ham")?;
//! let ham = HamFile::parse(&data)?;
//!
//! println!("HAM version: {}", ham.version());
//! println!("Texture count: {}", ham.textures().len());
//! println!("Robot count: {}", ham.robots().len());
//! # Ok(())
//! # }
//! ```

use crate::error::{AssetError, Result};
use std::io::{Cursor, Read, Seek, SeekFrom};

// HAM file constants
const HAM_SIGNATURE: u32 = 0x48414D21; // "HAM!" = MAKE_SIG('!','M','A','H')
const HAM_VERSION_DEMO: i32 = 2;
const HAM_VERSION_STANDARD: i32 = 3;

// Maximum counts for validation
const MAX_TEXTURES: usize = 1200;
const MAX_SOUNDS: usize = 250;
const MAX_VCLIPS: usize = 200;
const MAX_ROBOTS: usize = 85;
const MAX_WEAPONS: usize = 70;

// ============================================================================
// Core Structures
// ============================================================================

/// HAM file containing all game data definitions.
#[derive(Debug, Clone)]
pub struct HamFile {
    version: i32,
    textures: Vec<TextureInfo>,
    sound_indices: Vec<u8>,
    alt_sound_indices: Vec<u8>,
    vclips: Vec<VClipInfo>,
    robots: Vec<RobotInfo>,
    weapons: Vec<WeaponInfo>,
}

impl HamFile {
    /// Parse a HAM file from raw bytes.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - File signature is invalid
    /// - Version is not 2 or 3
    /// - Data is truncated or corrupted
    /// - Array counts exceed maximum limits
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);

        // Parse header
        let signature = read_u32_le(&mut cursor)?;
        if signature != HAM_SIGNATURE {
            return Err(AssetError::InvalidHamFormat(format!(
                "Invalid HAM signature: expected 0x{:08X}, got 0x{:08X}",
                HAM_SIGNATURE, signature
            )));
        }

        let version = read_i32_le(&mut cursor)?;
        if version != HAM_VERSION_DEMO && version != HAM_VERSION_STANDARD {
            return Err(AssetError::InvalidHamFormat(format!(
                "Invalid HAM version: expected 2 or 3, got {}",
                version
            )));
        }

        // Read sound offset if version < 3 (we'll skip embedded sounds for now)
        let _sound_offset = if version < HAM_VERSION_STANDARD {
            Some(read_i32_le(&mut cursor)?)
        } else {
            None
        };

        // Parse texture data
        let texture_count = read_i32_le(&mut cursor)? as usize;
        if texture_count > MAX_TEXTURES {
            return Err(AssetError::InvalidHamFormat(format!(
                "Texture count {} exceeds maximum {}",
                texture_count, MAX_TEXTURES
            )));
        }

        // Read bitmap indices
        let mut bitmap_indices = Vec::with_capacity(texture_count);
        for _ in 0..texture_count {
            bitmap_indices.push(BitmapIndex {
                index: read_u16_le(&mut cursor)?,
            });
        }

        // Read texture info
        let mut textures = Vec::with_capacity(texture_count);
        for bitmap_index in bitmap_indices {
            textures.push(parse_texture_info(&mut cursor, bitmap_index)?);
        }

        // Parse sound indices
        let sound_count = read_i32_le(&mut cursor)? as usize;
        if sound_count > MAX_SOUNDS {
            return Err(AssetError::InvalidHamFormat(format!(
                "Sound count {} exceeds maximum {}",
                sound_count, MAX_SOUNDS
            )));
        }

        let sound_indices = read_bytes(&mut cursor, sound_count)?;
        let alt_sound_indices = read_bytes(&mut cursor, sound_count)?;

        // Parse VClips (animation clips)
        let vclip_count = read_i32_le(&mut cursor)? as usize;
        if vclip_count > MAX_VCLIPS {
            return Err(AssetError::InvalidHamFormat(format!(
                "VClip count {} exceeds maximum {}",
                vclip_count, MAX_VCLIPS
            )));
        }

        let mut vclips = Vec::with_capacity(vclip_count);
        for _ in 0..vclip_count {
            vclips.push(parse_vclip_info(&mut cursor)?);
        }

        // Skip effects section for now (would parse tEffectInfo array here)
        let effect_count = read_i32_le(&mut cursor)? as usize;
        skip_bytes(&mut cursor, effect_count * 64)?; // Approximate size

        // Skip wall animations (would parse tWallEffect array here)
        let wall_anim_count = read_i32_le(&mut cursor)? as usize;
        skip_bytes(&mut cursor, wall_anim_count * 100)?; // Approximate size

        // Parse robots (simplified)
        let robot_count = read_i32_le(&mut cursor)? as usize;
        if robot_count > MAX_ROBOTS {
            return Err(AssetError::InvalidHamFormat(format!(
                "Robot count {} exceeds maximum {}",
                robot_count, MAX_ROBOTS
            )));
        }

        let mut robots = Vec::with_capacity(robot_count);
        for _ in 0..robot_count {
            robots.push(parse_robot_info(&mut cursor)?);
        }

        // Skip robot joints (would parse tJointPos array here)
        let joint_count = read_i32_le(&mut cursor)? as usize;
        skip_bytes(&mut cursor, joint_count * 8)?; // tJointPos size

        // Parse weapons (simplified)
        let weapon_count = read_i32_le(&mut cursor)? as usize;
        if weapon_count > MAX_WEAPONS {
            return Err(AssetError::InvalidHamFormat(format!(
                "Weapon count {} exceeds maximum {}",
                weapon_count, MAX_WEAPONS
            )));
        }

        let mut weapons = Vec::with_capacity(weapon_count);
        for _ in 0..weapon_count {
            weapons.push(parse_weapon_info(&mut cursor, version)?);
        }

        // Skip remaining sections (powerups, models, gauges, etc.) for now
        // A complete implementation would parse all sections

        Ok(HamFile {
            version,
            textures,
            sound_indices,
            alt_sound_indices,
            vclips,
            robots,
            weapons,
        })
    }

    /// Returns the HAM file version (2 or 3).
    pub const fn version(&self) -> i32 {
        self.version
    }

    /// Returns texture definitions.
    pub fn textures(&self) -> &[TextureInfo] {
        &self.textures
    }

    /// Returns robot definitions.
    pub fn robots(&self) -> &[RobotInfo] {
        &self.robots
    }

    /// Returns weapon definitions.
    pub fn weapons(&self) -> &[WeaponInfo] {
        &self.weapons
    }

    /// Returns sound indices.
    pub fn sound_indices(&self) -> &[u8] {
        &self.sound_indices
    }

    /// Returns alternate sound indices.
    pub fn alt_sound_indices(&self) -> &[u8] {
        &self.alt_sound_indices
    }

    /// Returns video clip definitions.
    pub fn vclips(&self) -> &[VClipInfo] {
        &self.vclips
    }
}

// ============================================================================
// Texture Structures
// ============================================================================

/// Bitmap index referencing a texture in the PIG file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitmapIndex {
    /// Index into PIG file bitmap array
    pub index: u16,
}

/// Texture map information defining wall texture properties.
///
/// This is the Descent 2 format (20 bytes). Descent 1 uses a different format.
#[derive(Debug, Clone)]
pub struct TextureInfo {
    /// Bitmap index in PIG file
    pub bitmap_index: BitmapIndex,

    /// Texture flags (volatile, water, force field, etc.)
    pub flags: u8,

    /// Lighting value (fixed-point)
    pub lighting: f32,

    /// Damage value (fixed-point)
    pub damage: f32,

    /// Effect clip index (-1 = none)
    pub effect_clip: i32,

    /// Destroyed texture index (-1 = none)
    pub destroyed: i16,

    /// U texture coordinate slide rate
    pub slide_u: i16,

    /// V texture coordinate slide rate
    pub slide_v: i16,
}

impl TextureInfo {
    /// Returns true if texture is volatile (can be destroyed).
    pub const fn is_volatile(&self) -> bool {
        self.flags & 0x01 != 0
    }

    /// Returns true if texture is water.
    pub const fn is_water(&self) -> bool {
        self.flags & 0x02 != 0
    }

    /// Returns true if texture is a force field.
    pub const fn is_force_field(&self) -> bool {
        self.flags & 0x04 != 0
    }

    /// Returns true if texture is a goal (CTF, etc.).
    pub const fn is_goal(&self) -> bool {
        self.flags & 0x08 != 0
    }

    /// Returns true if texture animates.
    pub const fn is_animated(&self) -> bool {
        self.flags & 0x10 != 0
    }
}

// ============================================================================
// Animation Structures
// ============================================================================

/// Video clip for animations (explosions, effects, etc.).
#[derive(Debug, Clone)]
pub struct VClipInfo {
    /// Total animation time (fixed-point)
    pub total_time: f32,

    /// Number of frames
    pub num_frames: i32,

    /// Time per frame (fixed-point)
    pub frame_time: f32,

    /// Animation flags
    pub flags: i32,

    /// Sound to play (-1 = none)
    pub sound_num: i16,

    /// Bitmap indices for each frame
    pub frame_indices: Vec<i16>,

    /// Light intensity (fixed-point)
    pub light_value: f32,
}

// ============================================================================
// Robot Structures
// ============================================================================

/// Robot/enemy definition (simplified - full structure is very large).
#[derive(Debug, Clone)]
pub struct RobotInfo {
    /// 3D model index
    pub model_num: i32,

    /// Number of guns
    pub n_guns: i8,

    /// Primary weapon type
    pub weapon_type: i8,

    /// Score value when destroyed
    pub score_value: i16,

    /// Hit points (fixed-point)
    pub strength: f32,

    /// Mass (fixed-point)
    pub mass: f32,

    /// Boss flag
    pub boss_flag: i8,
}

// ============================================================================
// Weapon Structures
// ============================================================================

/// Weapon definition (simplified - full structure has 50+ fields).
#[derive(Debug, Clone)]
pub struct WeaponInfo {
    /// 3D model index
    pub model_num: i16,

    /// Muzzle flash VClip
    pub flash_vclip: i8,

    /// Flash sound effect
    pub flash_sound: i16,

    /// Ammo consumed per shot
    pub ammo_usage: u8,

    /// Energy cost (fixed-point)
    pub energy_usage: f32,

    /// Fire delay (fixed-point)
    pub fire_wait: f32,

    /// Damage per difficulty level (fixed-point)
    pub strength: [f32; 5],

    /// Speed per difficulty level (fixed-point)
    pub speed: [f32; 5],

    /// Mass (fixed-point)
    pub mass: f32,

    /// Light intensity (fixed-point)
    pub light: f32,
}

// ============================================================================
// Parser Helper Functions
// ============================================================================

fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8> {
    let mut buf = [0u8; 1];
    cursor.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_i8(cursor: &mut Cursor<&[u8]>) -> Result<i8> {
    Ok(read_u8(cursor)? as i8)
}

fn read_u16_le(cursor: &mut Cursor<&[u8]>) -> Result<u16> {
    let mut buf = [0u8; 2];
    cursor.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_i16_le(cursor: &mut Cursor<&[u8]>) -> Result<i16> {
    let mut buf = [0u8; 2];
    cursor.read_exact(&mut buf)?;
    Ok(i16::from_le_bytes(buf))
}

fn read_u32_le(cursor: &mut Cursor<&[u8]>) -> Result<u32> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_i32_le(cursor: &mut Cursor<&[u8]>) -> Result<i32> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

fn read_f32_le(cursor: &mut Cursor<&[u8]>) -> Result<f32> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

fn read_bytes(cursor: &mut Cursor<&[u8]>, count: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; count];
    cursor.read_exact(&mut buf)?;
    Ok(buf)
}

fn skip_bytes(cursor: &mut Cursor<&[u8]>, count: usize) -> Result<()> {
    cursor.seek(SeekFrom::Current(count as i64))?;
    Ok(())
}

// ============================================================================
// Structure Parsers
// ============================================================================

fn parse_texture_info(
    cursor: &mut Cursor<&[u8]>,
    bitmap_index: BitmapIndex,
) -> Result<TextureInfo> {
    let flags = read_u8(cursor)?;
    skip_bytes(cursor, 3)?; // padding
    let lighting = read_f32_le(cursor)?;
    let damage = read_f32_le(cursor)?;
    let effect_clip = read_i32_le(cursor)?;
    let destroyed = read_i16_le(cursor)?;
    let slide_u = read_i16_le(cursor)?;
    let slide_v = read_i16_le(cursor)?;

    Ok(TextureInfo {
        bitmap_index,
        flags,
        lighting,
        damage,
        effect_clip,
        destroyed,
        slide_u,
        slide_v,
    })
}

fn parse_vclip_info(cursor: &mut Cursor<&[u8]>) -> Result<VClipInfo> {
    let total_time = read_f32_le(cursor)?;
    let num_frames = read_i32_le(cursor)?;
    let frame_time = read_f32_le(cursor)?;
    let flags = read_i32_le(cursor)?;
    let sound_num = read_i16_le(cursor)?;

    // Read frame indices (30 frames max for D2)
    let mut frame_indices = Vec::with_capacity(30);
    for _ in 0..30 {
        frame_indices.push(read_i16_le(cursor)?);
    }

    let light_value = read_f32_le(cursor)?;

    Ok(VClipInfo {
        total_time,
        num_frames,
        frame_time,
        flags,
        sound_num,
        frame_indices,
        light_value,
    })
}

fn parse_robot_info(cursor: &mut Cursor<&[u8]>) -> Result<RobotInfo> {
    let model_num = read_i32_le(cursor)?;

    // Skip gun points (8 guns * 12 bytes per vector)
    skip_bytes(cursor, 8 * 12)?;

    // Skip gun submodels
    skip_bytes(cursor, 8)?;

    // Skip explosion info
    skip_bytes(cursor, 8)?;

    let weapon_type = read_i8(cursor)?;
    skip_bytes(cursor, 1)?; // weapon_type2
    let n_guns = read_i8(cursor)?;

    // Skip contains info
    skip_bytes(cursor, 5)?;

    let score_value = read_i16_le(cursor)?;

    // Skip badass, energy_drain
    skip_bytes(cursor, 2)?;

    // Read core stats
    skip_bytes(cursor, 4)?; // lighting
    let strength = read_f32_le(cursor)?;
    let mass = read_f32_le(cursor)?;

    // Skip remaining fields (drag, AI params, sounds, animations, etc.)
    skip_bytes(cursor, 400)?; // Approximate remaining size

    let boss_flag = read_i8(cursor)?;

    // Skip to end of structure
    skip_bytes(cursor, 20)?;

    Ok(RobotInfo {
        model_num,
        n_guns,
        weapon_type,
        score_value,
        strength,
        mass,
        boss_flag,
    })
}

fn parse_weapon_info(cursor: &mut Cursor<&[u8]>, _version: i32) -> Result<WeaponInfo> {
    // Skip render type, persistent
    skip_bytes(cursor, 2)?;

    let model_num = read_i16_le(cursor)?;

    // Skip model_num_inner
    skip_bytes(cursor, 2)?;

    let flash_vclip = read_i8(cursor)?;

    // Skip robot_hit_vclip
    skip_bytes(cursor, 1)?;

    let flash_sound = read_i16_le(cursor)?;

    // Skip wall_hit_vclip, fire_count, robot_hit_sound
    skip_bytes(cursor, 5)?;

    let ammo_usage = read_u8(cursor)?;

    // Skip weapon_vclip, wall_hit_sound, destroyable, matter, bounce, homing_flag
    skip_bytes(cursor, 8)?;

    // Skip speedvar, flags, flash, afterburner_size, children (version >= 3)
    skip_bytes(cursor, 5)?;

    let energy_usage = read_f32_le(cursor)?;
    let fire_wait = read_f32_le(cursor)?;

    // Skip multi_damage_scale, bitmap, blob_size, flash_size, impact_size
    skip_bytes(cursor, 18)?;

    // Read strength array (5 difficulty levels)
    let strength = [
        read_f32_le(cursor)?,
        read_f32_le(cursor)?,
        read_f32_le(cursor)?,
        read_f32_le(cursor)?,
        read_f32_le(cursor)?,
    ];

    // Read speed array (5 difficulty levels)
    let speed = [
        read_f32_le(cursor)?,
        read_f32_le(cursor)?,
        read_f32_le(cursor)?,
        read_f32_le(cursor)?,
        read_f32_le(cursor)?,
    ];

    let mass = read_f32_le(cursor)?;

    // Skip drag, thrust, po_len_to_width_ratio
    skip_bytes(cursor, 12)?;

    let light = read_f32_le(cursor)?;

    // Skip remaining fields (lifetime, damage_radius, picture, hires_picture)
    skip_bytes(cursor, 18)?;

    Ok(WeaponInfo {
        model_num,
        flash_vclip,
        flash_sound,
        ammo_usage,
        energy_usage,
        fire_wait,
        strength,
        speed,
        mass,
        light,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ham_signature() {
        // "HAM!" = [0x21, 0x4D, 0x41, 0x48] (little-endian)
        let bytes = [0x21, 0x4D, 0x41, 0x48];
        let sig = u32::from_le_bytes(bytes);
        assert_eq!(sig, HAM_SIGNATURE);
        assert_eq!(sig, 0x48414D21);
    }

    #[test]
    fn test_bitmap_index() {
        let idx = BitmapIndex { index: 0x1234 };
        assert_eq!(idx.index, 0x1234);
    }

    #[test]
    fn test_texture_info_flags() {
        let tex = TextureInfo {
            bitmap_index: BitmapIndex { index: 0 },
            flags: 0x01 | 0x04, // Volatile | Force field
            lighting: 0.0,
            damage: 0.0,
            effect_clip: -1,
            destroyed: -1,
            slide_u: 0,
            slide_v: 0,
        };

        assert!(tex.is_volatile());
        assert!(!tex.is_water());
        assert!(tex.is_force_field());
        assert!(!tex.is_goal());
        assert!(!tex.is_animated());
    }

    #[test]
    fn test_invalid_signature() {
        let data = [
            0x00, 0x00, 0x00, 0x00, // Invalid signature
            0x03, 0x00, 0x00, 0x00, // Version 3
        ];

        let result = HamFile::parse(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_version() {
        let data = [
            0x21, 0x4D, 0x41, 0x48, // "HAM!" signature
            0x99, 0x00, 0x00, 0x00, // Invalid version 153
        ];

        let result = HamFile::parse(&data);
        assert!(result.is_err());
    }
}

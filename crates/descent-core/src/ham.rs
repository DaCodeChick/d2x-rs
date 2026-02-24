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
//! # Examples
//!
//! ```no_run
//! # use descent_core::ham::HamFile;
//! # use descent_core::error::Result;
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

use crate::error::Result;
use crate::io::ReadExt;
use crate::validation::{validate_max, validate_signature, validate_version};
use std::io::Cursor;

// HAM file constants
const HAM_SIGNATURE: u32 = 0x48414D21; // "HAM!"
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
        let signature = cursor.read_u32_le()?;
        validate_signature(signature, HAM_SIGNATURE, "HAM")?;

        let version = cursor.read_i32_le()?;
        validate_version(version, &[HAM_VERSION_DEMO, HAM_VERSION_STANDARD], "HAM")?;

        // Read sound offset if version < 3 (we'll skip embedded sounds for now)
        let _sound_offset = if version < HAM_VERSION_STANDARD {
            Some(cursor.read_i32_le()?)
        } else {
            None
        };

        // Parse texture data
        let texture_count = cursor.read_i32_le()? as usize;
        validate_max(texture_count, MAX_TEXTURES, "texture count", "MAX_TEXTURES")?;

        // Read bitmap indices
        let mut bitmap_indices = Vec::with_capacity(texture_count);
        for _ in 0..texture_count {
            bitmap_indices.push(BitmapIndex {
                index: cursor.read_u16_le()?,
            });
        }

        // Read texture info
        let mut textures = Vec::with_capacity(texture_count);
        for bitmap_index in bitmap_indices {
            textures.push(parse_texture_info(&mut cursor, bitmap_index)?);
        }

        // Parse sound indices
        let sound_count = cursor.read_i32_le()? as usize;
        validate_max(sound_count, MAX_SOUNDS, "sound count", "MAX_SOUNDS")?;

        let sound_indices = cursor.read_bytes(sound_count)?;
        let alt_sound_indices = cursor.read_bytes(sound_count)?;

        // Parse VClips (animation clips)
        let vclip_count = cursor.read_i32_le()? as usize;
        validate_max(vclip_count, MAX_VCLIPS, "VClip count", "MAX_VCLIPS")?;

        let mut vclips = Vec::with_capacity(vclip_count);
        for _ in 0..vclip_count {
            vclips.push(parse_vclip_info(&mut cursor)?);
        }

        // Skip effects section for now (would parse tEffectInfo array here)
        let effect_count = cursor.read_i32_le()? as usize;
        cursor.skip_bytes(effect_count * 64)?; // Approximate size

        // Skip wall animations (would parse tWallEffect array here)
        let wall_anim_count = cursor.read_i32_le()? as usize;
        cursor.skip_bytes(wall_anim_count * 100)?; // Approximate size

        // Parse robots (simplified)
        let robot_count = cursor.read_i32_le()? as usize;
        validate_max(robot_count, MAX_ROBOTS, "robot count", "MAX_ROBOTS")?;

        let mut robots = Vec::with_capacity(robot_count);
        for _ in 0..robot_count {
            robots.push(parse_robot_info(&mut cursor)?);
        }

        // Skip robot joints (would parse tJointPos array here)
        let joint_count = cursor.read_i32_le()? as usize;
        cursor.skip_bytes(joint_count * 8)?; // tJointPos size

        // Parse weapons (simplified)
        let weapon_count = cursor.read_i32_le()? as usize;
        validate_max(weapon_count, MAX_WEAPONS, "weapon count", "MAX_WEAPONS")?;

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
// Structure Parsers
// ============================================================================

fn parse_texture_info(
    cursor: &mut Cursor<&[u8]>,
    bitmap_index: BitmapIndex,
) -> Result<TextureInfo> {
    let flags = cursor.read_u8()?;
    cursor.skip_bytes(3)?; // padding
    let lighting = cursor.read_f32_le()?;
    let damage = cursor.read_f32_le()?;
    let effect_clip = cursor.read_i32_le()?;
    let destroyed = cursor.read_i16_le()?;
    let slide_u = cursor.read_i16_le()?;
    let slide_v = cursor.read_i16_le()?;

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
    let total_time = cursor.read_f32_le()?;
    let num_frames = cursor.read_i32_le()?;
    let frame_time = cursor.read_f32_le()?;
    let flags = cursor.read_i32_le()?;
    let sound_num = cursor.read_i16_le()?;

    // Read frame indices (30 frames max for D2)
    let mut frame_indices = Vec::with_capacity(30);
    for _ in 0..30 {
        frame_indices.push(cursor.read_i16_le()?);
    }

    let light_value = cursor.read_f32_le()?;

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
    let model_num = cursor.read_i32_le()?;

    // Skip gun points (8 guns * 12 bytes per vector)
    cursor.skip_bytes(8 * 12)?;

    // Skip gun submodels
    cursor.skip_bytes(8)?;

    // Skip explosion info
    cursor.skip_bytes(8)?;

    let weapon_type = cursor.read_i8()?;
    cursor.skip_bytes(1)?; // weapon_type2
    let n_guns = cursor.read_i8()?;

    // Skip contains info
    cursor.skip_bytes(5)?;

    let score_value = cursor.read_i16_le()?;

    // Skip badass, energy_drain
    cursor.skip_bytes(2)?;

    // Read core stats
    cursor.skip_bytes(4)?; // lighting
    let strength = cursor.read_f32_le()?;
    let mass = cursor.read_f32_le()?;

    // Skip remaining fields (drag, AI params, sounds, animations, etc.)
    cursor.skip_bytes(400)?; // Approximate remaining size

    let boss_flag = cursor.read_i8()?;

    // Skip to end of structure
    cursor.skip_bytes(20)?;

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
    cursor.skip_bytes(2)?;

    let model_num = cursor.read_i16_le()?;

    // Skip model_num_inner
    cursor.skip_bytes(2)?;

    let flash_vclip = cursor.read_i8()?;

    // Skip robot_hit_vclip
    cursor.skip_bytes(1)?;

    let flash_sound = cursor.read_i16_le()?;

    // Skip wall_hit_vclip, fire_count, robot_hit_sound
    cursor.skip_bytes(5)?;

    let ammo_usage = cursor.read_u8()?;

    // Skip weapon_vclip, wall_hit_sound, destroyable, matter, bounce, homing_flag
    cursor.skip_bytes(8)?;

    // Skip speedvar, flags, flash, afterburner_size, children (version >= 3)
    cursor.skip_bytes(5)?;

    let energy_usage = cursor.read_f32_le()?;
    let fire_wait = cursor.read_f32_le()?;

    // Skip multi_damage_scale, bitmap, blob_size, flash_size, impact_size
    cursor.skip_bytes(18)?;

    // Read strength array (5 difficulty levels)
    let strength = [
        cursor.read_f32_le()?,
        cursor.read_f32_le()?,
        cursor.read_f32_le()?,
        cursor.read_f32_le()?,
        cursor.read_f32_le()?,
    ];

    // Read speed array (5 difficulty levels)
    let speed = [
        cursor.read_f32_le()?,
        cursor.read_f32_le()?,
        cursor.read_f32_le()?,
        cursor.read_f32_le()?,
        cursor.read_f32_le()?,
    ];

    let mass = cursor.read_f32_le()?;

    // Skip drag, thrust, po_len_to_width_ratio
    cursor.skip_bytes(12)?;

    let light = cursor.read_f32_le()?;

    // Skip remaining fields (lifetime, damage_radius, picture, hires_picture)
    cursor.skip_bytes(18)?;

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

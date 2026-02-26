//! Level geometry file format parser (RDL/RL2)
//!
//! Parses Descent 1 (RDL) and Descent 2 (RL2) level files containing mine geometry,
//! segments (cube rooms), walls, triggers, and objects.
//!
//! # Format Overview
//!
//! Level files store the 3D geometry of game levels as a collection of segments.
//! Each segment is a cube-shaped room with:
//! - 8 vertices (corners)
//! - 6 sides (faces with textures)
//! - 6 children (connections to adjacent segments)
//! - Optional function (reactor, fuel center, etc.)
//!
//! See `docs/formats/LEVEL_FORMAT.md` for complete format specification.

use crate::error::{AssetError, Result};
use crate::fixed_point::{Fix, I2X_MULTIPLIER};
use crate::geometry::{FixVector, Uvl};
use crate::io::ReadExt;
use bitflags::bitflags;
use std::io::Cursor;

// ================================================================================================
// CONSTANTS
// ================================================================================================

/// Descent 1 level version
pub const LEVEL_VERSION_D1: u8 = 1;

/// Descent 2 Shareware level version
pub const LEVEL_VERSION_D2_SHAREWARE: u8 = 5;

/// Current mine version expected
pub const MINE_VERSION: u8 = 20;

/// Oldest compatible version that can be safely loaded
pub const COMPATIBLE_VERSION: u8 = 16;

/// Compiled mine version (always 0)
pub const COMPILED_MINE_VERSION: u8 = 0;

/// Number of vertices per segment
pub const SEGMENT_VERTEX_COUNT: usize = 8;

/// Number of sides per segment
pub const SEGMENT_SIDE_COUNT: usize = 6;

/// Number of corners per side
pub const SIDE_CORNER_COUNT: usize = 4;

/// Maximum wall textures
pub const MAX_WALL_TEXTURES: usize = 910;

/// Texture ID mask (14 bits)
pub const TEXTURE_ID_MASK: u16 = 0x3FFF;

// ================================================================================================
// ENUMS
// ================================================================================================

/// Segment side index
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum SideIndex {
    Left = 0,
    Top = 1,
    Right = 2,
    Bottom = 3,
    Back = 4,
    Front = 5,
}

impl From<SideIndex> for usize {
    fn from(side: SideIndex) -> Self {
        side as usize
    }
}

impl TryFrom<usize> for SideIndex {
    type Error = AssetError;

    fn try_from(value: usize) -> Result<Self> {
        match value {
            0 => Ok(Self::Left),
            1 => Ok(Self::Top),
            2 => Ok(Self::Right),
            3 => Ok(Self::Bottom),
            4 => Ok(Self::Back),
            5 => Ok(Self::Front),
            _ => Err(AssetError::InvalidLevelFormat(format!(
                "Invalid side index: {}",
                value
            ))),
        }
    }
}

/// Segment function type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum SegmentFunc {
    #[default]
    None = 0,
    FuelCenter = 1,
    RepairCenter = 2,
    Reactor = 3,
    RobotMaker = 4,
    GoalBlue = 5,
    GoalRed = 6,
    TeamBlue = 9,
    TeamRed = 10,
    SpeedBoost = 11,
    SkyBox = 14,
    EquipMaker = 15,
}

impl From<u8> for SegmentFunc {
    fn from(value: u8) -> Self {
        match value {
            1 => Self::FuelCenter,
            2 => Self::RepairCenter,
            3 => Self::Reactor,
            4 => Self::RobotMaker,
            5 => Self::GoalBlue,
            6 => Self::GoalRed,
            9 => Self::TeamBlue,
            10 => Self::TeamRed,
            11 => Self::SpeedBoost,
            14 => Self::SkyBox,
            15 => Self::EquipMaker,
            _ => Self::None,
        }
    }
}

impl From<SegmentFunc> for u8 {
    fn from(func: SegmentFunc) -> Self {
        func as u8
    }
}

bitflags! {
    /// Segment property flags (can be combined)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SegmentProps: u8 {
        const WATER = 0x01;
        const LAVA = 0x02;
        const BLOCKED = 0x04;
        const NO_DAMAGE = 0x08;
        const OUTDOORS = 0x10;
        const LIGHT_FOG = 0x20;
        const DENSE_FOG = 0x40;
    }
}

// ================================================================================================
// DATA STRUCTURES
// ================================================================================================

/// Side type (quad or triangle)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SideType {
    Quad = 0,
    Tri02 = 1,
    Tri13 = 2,
}

/// Side of a segment (one face of the cube)
#[derive(Debug, Clone)]
pub struct Side {
    /// Wall number (0xFFFF if no wall)
    pub wall_num: u16,

    /// Corner indices (which of segment's 8 vertices, 0xFF = unused)
    pub corners: [u8; SIDE_CORNER_COUNT],

    /// Base texture index
    pub base_texture: u16,

    /// Overlay texture index (0 = none)
    pub overlay_texture: u16,

    /// Overlay orientation (0-3 = 0°, 90°, 180°, 270°)
    pub overlay_orient: u8,

    /// UV coordinates and lighting for 4 corners
    pub uvls: [Uvl; SIDE_CORNER_COUNT],

    /// Side shape type
    pub side_type: SideType,
}

impl Default for Side {
    fn default() -> Self {
        Self {
            wall_num: 0xFFFF,
            corners: [0, 1, 2, 3],
            base_texture: 0,
            overlay_texture: 0,
            overlay_orient: 0,
            uvls: [Uvl::default(); SIDE_CORNER_COUNT],
            side_type: SideType::Quad,
        }
    }
}

/// Segment (cube room)
#[derive(Debug, Clone)]
pub struct Segment {
    /// Multiplayer team owner (-1 = none, D2X-XL only)
    pub owner: i8,

    /// Editor grouping (-1 = none, D2X-XL only)
    pub group: i8,

    /// Vertex indices (8 corners of the cube)
    pub vertices: [u16; SEGMENT_VERTEX_COUNT],

    /// Child segment indices (-1 = solid wall, -2 = outside)
    pub children: [i16; SEGMENT_SIDE_COUNT],

    /// Six sides (faces)
    pub sides: [Side; SEGMENT_SIDE_COUNT],

    /// Segment function (reactor, fuel center, etc.)
    pub function: SegmentFunc,

    /// Object producer index (-1 = none)
    pub obj_producer: i16,

    /// Function-specific value
    pub value: i16,

    /// Segment property flags (can be combined)
    pub props: SegmentProps,

    /// Damage amounts [entry, continuous]
    pub damage: [Fix; 2],

    /// Average segment light
    pub avg_seg_light: Fix,
}

impl Default for Segment {
    fn default() -> Self {
        Self {
            owner: -1,
            group: -1,
            vertices: [0; SEGMENT_VERTEX_COUNT],
            children: [-1; SEGMENT_SIDE_COUNT],
            sides: [
                Side::default(),
                Side::default(),
                Side::default(),
                Side::default(),
                Side::default(),
                Side::default(),
            ],
            function: SegmentFunc::None,
            obj_producer: -1,
            value: 0,
            props: SegmentProps::empty(),
            damage: [Fix::ZERO; 2],
            avg_seg_light: Fix::ZERO,
        }
    }
}

/// Complete level data
#[derive(Debug, Clone)]
pub struct Level {
    /// Level version
    pub version: u8,

    /// Is new file format (not D1 shareware .sdl)
    pub new_file_format: bool,

    /// Vertices (3D points)
    pub vertices: Vec<FixVector>,

    /// Segments (cube rooms)
    pub segments: Vec<Segment>,
}

// ================================================================================================
// HELPER FUNCTIONS
// ================================================================================================

/// Read fixed-point number
fn read_fix(cursor: &mut Cursor<&[u8]>) -> Result<Fix> {
    Ok(Fix::from(cursor.read_i32_le()?))
}

/// Read 3D vector (fixed-point)
fn read_fix_vector(cursor: &mut Cursor<&[u8]>) -> Result<FixVector> {
    Ok(FixVector {
        x: read_fix(cursor)?,
        y: read_fix(cursor)?,
        z: read_fix(cursor)?,
    })
}

/// Read UVL (texture coordinates + lighting)
fn read_uvl(cursor: &mut Cursor<&[u8]>) -> Result<Uvl> {
    // Stored as i16, scaled by << 5 for U/V, << 1 for L
    let u_raw = cursor.read_i16_le()?;
    let v_raw = cursor.read_i16_le()?;
    let l_raw = cursor.read_u16_le()?;

    Ok(Uvl {
        u: Fix::from((u_raw as i32) << 5),
        v: Fix::from((v_raw as i32) << 5),
        l: Fix::from((l_raw as i32) << 1),
    })
}

/// Upgrade old segment type to function + props
fn upgrade_segment_type(old_type: u8) -> (SegmentFunc, SegmentProps) {
    let function = match old_type {
        1 => SegmentFunc::FuelCenter,
        2 => SegmentFunc::RepairCenter,
        3 => SegmentFunc::Reactor,
        4 => SegmentFunc::RobotMaker,
        5 => SegmentFunc::GoalBlue,
        6 => SegmentFunc::GoalRed,
        7 | 8 => SegmentFunc::None, // Water/lava
        9 => SegmentFunc::TeamBlue,
        10 => SegmentFunc::TeamRed,
        11 => SegmentFunc::SpeedBoost,
        14 => SegmentFunc::SkyBox,
        15 => SegmentFunc::EquipMaker,
        _ => SegmentFunc::None,
    };

    let props = match old_type {
        7 => SegmentProps::WATER,
        8 => SegmentProps::LAVA,
        12 => SegmentProps::BLOCKED,
        13 => SegmentProps::NO_DAMAGE,
        14 => SegmentProps::BLOCKED, // Skybox
        16 => SegmentProps::OUTDOORS,
        _ => SegmentProps::empty(),
    };

    (function, props)
}

// ================================================================================================
// PARSER IMPLEMENTATION
// ================================================================================================

impl Level {
    /// Parse a level file from bytes
    ///
    /// # Arguments
    ///
    /// * `data` - Raw level file data (RDL/RL2)
    /// * `filename` - Optional filename for format detection (e.g., "level01.rdl")
    ///
    /// # Example
    ///
    /// ```no_run
    /// use descent_core::level::Level;
    ///
    /// let data = std::fs::read("level01.rdl").unwrap();
    /// let level = Level::parse(&data, Some("level01.rdl")).unwrap();
    /// println!("Loaded {} segments", level.segments.len());
    /// ```
    pub fn parse(data: &[u8], filename: Option<&str>) -> Result<Self> {
        let mut cursor = Cursor::new(data);

        // Detect file format from extension
        let new_file_format = if let Some(name) = filename {
            let lower = name.to_lowercase();
            !lower.ends_with(".sdl") // D1 shareware = old format
        } else {
            true // Assume new format if unknown
        };

        // Read header
        let compiled_version = cursor.read_u8()?;
        if compiled_version != COMPILED_MINE_VERSION {
            // Not an error, just informational
        }

        let vertex_count = if new_file_format {
            cursor.read_u16_le()? as usize
        } else {
            cursor.read_i32_le()? as usize
        };

        let segment_count = if new_file_format {
            cursor.read_u16_le()? as usize
        } else {
            cursor.read_i32_le()? as usize
        };

        // Infer version from file format
        // Note: Cannot distinguish D2 Shareware (v5) from D2+ (v2-20) without heuristics
        // Default to current version for new format, D1 for old format
        let version = if new_file_format {
            MINE_VERSION
        } else {
            LEVEL_VERSION_D1
        };

        // Read vertices
        let vertices = (0..vertex_count)
            .map(|_| read_fix_vector(&mut cursor))
            .collect::<Result<Vec<_>>>()?;

        // Read segments
        let mut segments = (0..segment_count)
            .map(|_| Self::read_segment(&mut cursor, new_file_format, vertex_count, version))
            .collect::<Result<Vec<_>>>()?;

        // Read segment extras (only for D2+, versions 2-20)
        if version >= 2 {
            segments
                .iter_mut()
                .try_for_each(|segment| Self::read_segment_extras(&mut cursor, segment, version))?;
        }

        Ok(Self {
            version,
            new_file_format,
            vertices,
            segments,
        })
    }

    /// Read a single segment
    fn read_segment(
        cursor: &mut Cursor<&[u8]>,
        new_file_format: bool,
        vertex_count: usize,
        version: u8,
    ) -> Result<Segment> {
        // Note: owner and group default to -1 (D2X-XL only, not implemented yet)
        let mut segment = Segment::default();

        // Read flags byte
        let flags = if new_file_format {
            cursor.read_u8()?
        } else {
            0x7F // All flags set
        };

        // Determine reading order based on version
        // - D1 (v1): children, verts, function
        // - D2 Shareware (v5): function, verts, children
        // - D2+ (v2-20): children, verts

        match version {
            LEVEL_VERSION_D2_SHAREWARE => {
                // D2 Shareware (v5): function, verts, children
                Self::read_segment_function_v5(cursor, &mut segment, flags)?;
                Self::read_segment_vertices(cursor, &mut segment, vertex_count)?;
                Self::read_segment_children(cursor, &mut segment, flags)?;
            }
            LEVEL_VERSION_D1 => {
                // D1 (v1): children, verts, function
                Self::read_segment_children(cursor, &mut segment, flags)?;
                Self::read_segment_vertices(cursor, &mut segment, vertex_count)?;
                Self::read_segment_function_v1(cursor, &mut segment)?;
            }
            _ => {
                // D2+ (v2-20): children, verts (no inline function)
                Self::read_segment_children(cursor, &mut segment, flags)?;
                Self::read_segment_vertices(cursor, &mut segment, vertex_count)?;
            }
        }

        // Read wall flags
        let wall_flags = if new_file_format {
            cursor.read_u8()?
        } else {
            0x3F // All sides have walls
        };

        // Read wall numbers for each side
        let wall_nums: Vec<u16> = (0..SEGMENT_SIDE_COUNT)
            .map(|i| {
                if (wall_flags & (1 << i)) != 0 {
                    // Wall numbers are always u16 in all versions
                    cursor.read_u16_le()
                } else {
                    Ok(0xFFFF)
                }
            })
            .collect::<Result<Vec<_>>>()?;

        for (i, wall_num) in wall_nums.into_iter().enumerate() {
            segment.sides[i].wall_num = wall_num;
        }

        // Read sides
        (0..SEGMENT_SIDE_COUNT).try_for_each(|i| {
            Self::read_side(
                cursor,
                &mut segment.sides[i],
                &segment.vertices,
                segment.children[i],
                new_file_format,
                version,
            )
        })?;

        Ok(segment)
    }

    /// Read segment children array
    fn read_segment_children(
        cursor: &mut Cursor<&[u8]>,
        segment: &mut Segment,
        flags: u8,
    ) -> Result<()> {
        segment.children = (0..SEGMENT_SIDE_COUNT)
            .map(|i| {
                if (flags & (1 << i)) != 0 {
                    cursor.read_i16_le()
                } else {
                    Ok(-1)
                }
            })
            .collect::<Result<Vec<_>>>()?
            .try_into()
            .map_err(|_| {
                AssetError::InvalidLevelFormat("Invalid children array length".to_string())
            })?;
        Ok(())
    }

    /// Read segment vertices array
    fn read_segment_vertices(
        cursor: &mut Cursor<&[u8]>,
        segment: &mut Segment,
        vertex_count: usize,
    ) -> Result<()> {
        segment.vertices = (0..SEGMENT_VERTEX_COUNT)
            .map(|_| {
                let vertex_idx = cursor.read_u16_le()?;
                if vertex_idx as usize >= vertex_count {
                    return Err(AssetError::InvalidLevelFormat(format!(
                        "Vertex index {} out of range (max {})",
                        vertex_idx, vertex_count
                    )));
                }
                Ok(vertex_idx)
            })
            .collect::<Result<Vec<_>>>()?
            .try_into()
            .map_err(|_| {
                AssetError::InvalidLevelFormat("Invalid vertices array length".to_string())
            })?;
        Ok(())
    }

    /// Read segment function data (D1 v1 format)
    fn read_segment_function_v1(cursor: &mut Cursor<&[u8]>, segment: &mut Segment) -> Result<()> {
        // D1 format: i16 static_light, followed by segment type
        let static_light = cursor.read_i16_le()?;
        segment.avg_seg_light = Fix::from((static_light as i32) << 4); // Convert to fix by shifting

        // Read segment type/function
        let func_byte = cursor.read_u8()?;
        segment.function = SegmentFunc::from(func_byte);

        Ok(())
    }

    /// Read segment function data (D2 Shareware v5 format)  
    fn read_segment_function_v5(
        cursor: &mut Cursor<&[u8]>,
        segment: &mut Segment,
        flags: u8,
    ) -> Result<()> {
        // V5 format: function data is present if bit 6 of flags is set
        if (flags & (1 << 6)) != 0 {
            let func_byte = cursor.read_u8()?;
            segment.function = SegmentFunc::from(func_byte);
        }
        Ok(())
    }

    /// Read a single side
    fn read_side(
        cursor: &mut Cursor<&[u8]>,
        side: &mut Side,
        _seg_vertices: &[u16; SEGMENT_VERTEX_COUNT],
        child: i16,
        new_file_format: bool,
        version: u8,
    ) -> Result<()> {
        // Determine if this side has textures
        let is_solid = child == -1;
        let has_wall = side.wall_num != 0xFFFF;
        let has_texture = is_solid || has_wall;

        // Read corner indices (v25+ only)
        if version >= 25 {
            side.corners = [
                cursor.read_u8()?,
                cursor.read_u8()?,
                cursor.read_u8()?,
                cursor.read_u8()?,
            ];
        }
        // For v < 25, corners remain at default [0, 1, 2, 3] from Side::default()

        if !has_texture {
            // No texture data
            side.base_texture = 0;
            side.overlay_texture = 0;
            return Ok(());
        }

        // Read base texture
        let base_tex_raw = if new_file_format {
            cursor.read_u16_le()?
        } else {
            cursor.read_i16_le()? as u16
        };

        side.base_texture = base_tex_raw & 0x7FFF;

        // Check if overlay is present
        let has_overlay = if new_file_format {
            (base_tex_raw & 0x8000) != 0
        } else {
            true // Old format always reads overlay
        };

        if has_overlay {
            // Read overlay texture
            let ovl_tex_raw = cursor.read_i16_le()?;
            side.overlay_texture = (ovl_tex_raw as u16) & TEXTURE_ID_MASK;
            side.overlay_orient = ((ovl_tex_raw >> 14) & 3) as u8;
        } else {
            side.overlay_texture = 0;
            side.overlay_orient = 0;
        }

        // Clamp texture indices (safety for bad data)
        side.base_texture %= MAX_WALL_TEXTURES as u16;
        side.overlay_texture %= MAX_WALL_TEXTURES as u16;

        // Read UVLs (4 corners)
        side.uvls = (0..SIDE_CORNER_COUNT)
            .map(|_| read_uvl(cursor))
            .collect::<Result<Vec<_>>>()?
            .try_into()
            .map_err(|_| AssetError::InvalidLevelFormat("Invalid UVL array length".to_string()))?;

        Ok(())
    }

    /// Read segment extras
    fn read_segment_extras(
        cursor: &mut Cursor<&[u8]>,
        segment: &mut Segment,
        version: u8,
    ) -> Result<()> {
        // Read function as raw u8
        let function_raw = cursor.read_u8()?;

        // Read obj_producer and value
        if version < 24 {
            segment.obj_producer = cursor.read_u8()? as i16;
            segment.value = cursor.read_i8()? as i16;
        } else {
            segment.obj_producer = cursor.read_i16_le()?;
            segment.value = cursor.read_i16_le()?;
        }

        // Read flags (unused but must be read)
        let _flags = cursor.read_u8()?;

        // Read props and damage
        if version <= 20 {
            // Old format: upgrade from function type
            let (new_function, props) = upgrade_segment_type(function_raw);
            segment.function = new_function;
            segment.props = props;
            segment.damage = [Fix::ZERO, Fix::ZERO];
        } else {
            // New format: explicit props and damage
            segment.function = function_raw.into();
            segment.props = SegmentProps::from_bits_truncate(cursor.read_u8()?);
            let damage0 = cursor.read_i16_le()?;
            let damage1 = cursor.read_i16_le()?;
            segment.damage[0] = Fix::from(damage0 as i32 * I2X_MULTIPLIER);
            segment.damage[1] = Fix::from(damage1 as i32 * I2X_MULTIPLIER);
        }

        // Read average segment light
        segment.avg_seg_light = read_fix(cursor)?;

        Ok(())
    }
}

// ================================================================================================
// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_conversion() {
        let fix = Fix::from(1.0);
        let raw: i32 = fix.into();
        assert_eq!(raw, I2X_MULTIPLIER);
        let f: f32 = fix.into();
        assert_eq!(f, 1.0);

        let fix = Fix::from(2.5);
        let f: f32 = fix.into();
        assert!((f - 2.5).abs() < 0.0001);

        // Test roundtrip conversion
        let original = 2.5_f32;
        let fix = Fix::from(original);
        let result: f32 = fix.into();
        assert!((result - original).abs() < 0.0001);
    }

    #[test]
    fn test_upgrade_segment_type() {
        let (func, props) = upgrade_segment_type(1);
        assert_eq!(func, SegmentFunc::FuelCenter);
        assert_eq!(props, SegmentProps::empty());

        let (func, props) = upgrade_segment_type(7);
        assert_eq!(func, SegmentFunc::None);
        assert_eq!(props, SegmentProps::WATER);

        let (func, props) = upgrade_segment_type(8);
        assert_eq!(func, SegmentFunc::None);
        assert_eq!(props, SegmentProps::LAVA);
    }

    #[test]
    fn test_segment_default() {
        let seg = Segment::default();
        assert_eq!(seg.owner, -1);
        assert_eq!(seg.group, -1);
        assert_eq!(seg.function, SegmentFunc::None);
        assert_eq!(seg.props, SegmentProps::empty());
        assert_eq!(seg.children, [-1; SEGMENT_SIDE_COUNT]);
    }

    #[test]
    fn test_side_default() {
        let side = Side::default();
        assert_eq!(side.wall_num, 0xFFFF);
        assert_eq!(side.base_texture, 0);
        assert_eq!(side.overlay_texture, 0);
        assert_eq!(side.side_type, SideType::Quad);
    }

    #[test]
    fn test_segment_func_conversion() {
        assert_eq!(SegmentFunc::from(0), SegmentFunc::None);
        assert_eq!(SegmentFunc::from(1), SegmentFunc::FuelCenter);
        assert_eq!(SegmentFunc::from(3), SegmentFunc::Reactor);
        assert_eq!(SegmentFunc::from(99), SegmentFunc::None);

        assert_eq!(u8::from(SegmentFunc::FuelCenter), 1);
        assert_eq!(u8::from(SegmentFunc::Reactor), 3);
    }

    #[test]
    fn test_segment_prop_flags() {
        assert!(SegmentProps::WATER.contains(SegmentProps::WATER));
        assert!(!SegmentProps::WATER.contains(SegmentProps::LAVA));
        assert!(SegmentProps::LAVA.contains(SegmentProps::LAVA));

        let combined = SegmentProps::WATER | SegmentProps::LAVA;
        assert_eq!(combined.bits(), 0x03);
        assert!(combined.contains(SegmentProps::WATER));
        assert!(combined.contains(SegmentProps::LAVA));
    }
}

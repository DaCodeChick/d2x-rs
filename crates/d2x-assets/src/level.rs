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
use bitflags::bitflags;
use std::io::{Cursor, Read};

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

/// Fixed-point multiplier (16.16 format)
pub const I2X_MULTIPLIER: i32 = 65536;

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

/// Fixed-point number (16.16 format: 16-bit integer, 16-bit fraction)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fix(pub i32);

impl Fix {
    // Removed to_f32() method - use Into<f32> trait instead
}

impl From<f32> for Fix {
    fn from(val: f32) -> Self {
        Self((val * I2X_MULTIPLIER as f32) as i32)
    }
}

impl From<Fix> for f32 {
    fn from(fix: Fix) -> Self {
        fix.0 as f32 / I2X_MULTIPLIER as f32
    }
}

/// 3D vector in fixed-point format
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FixVector {
    pub x: Fix,
    pub y: Fix,
    pub z: Fix,
}

impl FixVector {
    /// Convert to glam Vec3
    pub fn to_vec3(&self) -> [f32; 3] {
        [self.x.into(), self.y.into(), self.z.into()]
    }
}

/// UV texture coordinates and lighting value
#[derive(Debug, Clone, Copy)]
pub struct UVL {
    pub u: Fix,
    pub v: Fix,
    pub l: Fix,
}

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
    pub uvls: [UVL; SIDE_CORNER_COUNT],

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
            uvls: [UVL {
                u: Fix(0),
                v: Fix(0),
                l: Fix(0),
            }; SIDE_CORNER_COUNT],
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
            damage: [Fix(0); 2],
            avg_seg_light: Fix(0),
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

/// Read u8
fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8> {
    let mut buf = [0u8; 1];
    cursor.read_exact(&mut buf)?;
    Ok(buf[0])
}

/// Read i8
fn read_i8(cursor: &mut Cursor<&[u8]>) -> Result<i8> {
    Ok(read_u8(cursor)? as i8)
}

/// Read u16 (little-endian)
fn read_u16(cursor: &mut Cursor<&[u8]>) -> Result<u16> {
    let mut buf = [0u8; 2];
    cursor.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

/// Read i16 (little-endian)
fn read_i16(cursor: &mut Cursor<&[u8]>) -> Result<i16> {
    let mut buf = [0u8; 2];
    cursor.read_exact(&mut buf)?;
    Ok(i16::from_le_bytes(buf))
}

/// Read i32 (little-endian)
fn read_i32(cursor: &mut Cursor<&[u8]>) -> Result<i32> {
    let mut buf = [0u8; 4];
    cursor.read_exact(&mut buf)?;
    Ok(i32::from_le_bytes(buf))
}

/// Read fixed-point number
fn read_fix(cursor: &mut Cursor<&[u8]>) -> Result<Fix> {
    Ok(Fix(read_i32(cursor)?))
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
fn read_uvl(cursor: &mut Cursor<&[u8]>) -> Result<UVL> {
    // Stored as i16, scaled by << 5 for U/V, << 1 for L
    let u_raw = read_i16(cursor)?;
    let v_raw = read_i16(cursor)?;
    let l_raw = read_u16(cursor)?;

    Ok(UVL {
        u: Fix((u_raw as i32) << 5),
        v: Fix((v_raw as i32) << 5),
        l: Fix((l_raw as i32) << 1),
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
    /// use d2x_assets::level::Level;
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
        let compiled_version = read_u8(&mut cursor)?;
        if compiled_version != COMPILED_MINE_VERSION {
            // Not an error, just informational
        }

        let vertex_count = if new_file_format {
            read_u16(&mut cursor)? as usize
        } else {
            read_i32(&mut cursor)? as usize
        };

        let segment_count = if new_file_format {
            read_u16(&mut cursor)? as usize
        } else {
            read_i32(&mut cursor)? as usize
        };

        // Read vertices
        let mut vertices = Vec::with_capacity(vertex_count);
        for _ in 0..vertex_count {
            vertices.push(read_fix_vector(&mut cursor)?);
        }

        // Read segments
        let mut segments = Vec::with_capacity(segment_count);
        for _ in 0..segment_count {
            let segment = Self::read_segment(&mut cursor, new_file_format, vertex_count)?;
            segments.push(segment);
        }

        // Read segment extras
        for segment in &mut segments {
            Self::read_segment_extras(&mut cursor, segment, MINE_VERSION)?;
        }

        Ok(Self {
            version: MINE_VERSION,
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
    ) -> Result<Segment> {
        // Note: owner and group default to -1 (D2X-XL only, not implemented yet)
        let mut segment = Segment::default();

        // Read flags byte
        let flags = if new_file_format {
            read_u8(cursor)?
        } else {
            0x7F // All flags set
        };

        // Determine reading order based on version
        // For now, assume D2 format (v2-20): children, verts
        // TODO: Support D1 (v1) and D2 Shareware (v5) layouts

        // Read children
        for i in 0..SEGMENT_SIDE_COUNT {
            segment.children[i] = if (flags & (1 << i)) != 0 {
                read_i16(cursor)?
            } else {
                -1
            };
        }

        // Read vertices
        for i in 0..SEGMENT_VERTEX_COUNT {
            let vertex_idx = read_u16(cursor)?;
            if vertex_idx as usize >= vertex_count {
                return Err(AssetError::InvalidLevelFormat(format!(
                    "Vertex index {} out of range (max {})",
                    vertex_idx, vertex_count
                )));
            }
            segment.vertices[i] = vertex_idx;
        }

        // Read wall flags
        let wall_flags = if new_file_format {
            read_u8(cursor)?
        } else {
            0x3F // All sides have walls
        };

        // Read wall numbers for each side
        for i in 0..SEGMENT_SIDE_COUNT {
            segment.sides[i].wall_num = if (wall_flags & (1 << i)) != 0 {
                // TODO: Check version for u8 vs u16
                read_u16(cursor)?
            } else {
                0xFFFF
            };
        }

        // Read sides
        for i in 0..SEGMENT_SIDE_COUNT {
            Self::read_side(
                cursor,
                &mut segment.sides[i],
                &segment.vertices,
                segment.children[i],
                new_file_format,
            )?;
        }

        Ok(segment)
    }

    /// Read a single side
    fn read_side(
        cursor: &mut Cursor<&[u8]>,
        side: &mut Side,
        _seg_vertices: &[u16; SEGMENT_VERTEX_COUNT],
        child: i16,
        new_file_format: bool,
    ) -> Result<()> {
        // Determine if this side has textures
        let is_solid = child == -1;
        let has_wall = side.wall_num != 0xFFFF;
        let has_texture = is_solid || has_wall;

        // TODO: Support v25+ corner indices

        if !has_texture {
            // No texture data
            side.base_texture = 0;
            side.overlay_texture = 0;
            return Ok(());
        }

        // Read base texture
        let base_tex_raw = if new_file_format {
            read_u16(cursor)?
        } else {
            read_i16(cursor)? as u16
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
            let ovl_tex_raw = read_i16(cursor)?;
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
        for i in 0..SIDE_CORNER_COUNT {
            side.uvls[i] = read_uvl(cursor)?;
        }

        Ok(())
    }

    /// Read segment extras
    fn read_segment_extras(
        cursor: &mut Cursor<&[u8]>,
        segment: &mut Segment,
        version: u8,
    ) -> Result<()> {
        // Read function as raw u8
        let function_raw = read_u8(cursor)?;

        // Read obj_producer and value
        if version < 24 {
            segment.obj_producer = read_u8(cursor)? as i16;
            segment.value = read_i8(cursor)? as i16;
        } else {
            segment.obj_producer = read_i16(cursor)?;
            segment.value = read_i16(cursor)?;
        }

        // Read flags (unused but must be read)
        let _flags = read_u8(cursor)?;

        // Read props and damage
        if version <= 20 {
            // Old format: upgrade from function type
            let (new_function, props) = upgrade_segment_type(function_raw);
            segment.function = new_function;
            segment.props = props;
            segment.damage = [Fix(0), Fix(0)];
        } else {
            // New format: explicit props and damage
            segment.function = function_raw.into();
            segment.props = SegmentProps::from_bits_truncate(read_u8(cursor)?);
            let damage0 = read_i16(cursor)?;
            let damage1 = read_i16(cursor)?;
            segment.damage[0] = Fix(damage0 as i32 * I2X_MULTIPLIER);
            segment.damage[1] = Fix(damage1 as i32 * I2X_MULTIPLIER);
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
        assert_eq!(fix.0, I2X_MULTIPLIER);
        let f: f32 = fix.into();
        assert_eq!(f, 1.0);

        let fix = Fix::from(2.5);
        let f: f32 = fix.into();
        assert!((f - 2.5).abs() < 0.0001);

        // Test From trait both ways
        let f: f32 = fix.into();
        assert!((f - 2.5).abs() < 0.0001);
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

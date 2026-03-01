//! OOF (Outrage Object Format) model parser for Descent 3.
//!
//! OOF files are polygon-based 3D models used for ships, robots, powerups, and other
//! game objects in Descent 3. The format uses a chunk-based IFF-like structure similar
//! to PNG or RIFF formats.
//!
//! # Format Overview
//!
//! OOF files consist of a sequence of chunks, each with:
//! - 4-byte chunk ID (little-endian integer, typically ASCII in reverse)
//! - Variable-length chunk data
//!
//! Key chunks include:
//! - **OHDR**: File header with version info
//! - **SOBJ**: Subobject/mesh data (BSP structure)
//! - **TXTR**: Texture filename list  
//! - **GPNT**: Gun/attach points
//! - **ANIM/ROT_ANIM/POS_ANIM**: Animation data (timed and untimed)
//! - **WBS**: Weapon battery/turret info
//! - **ATTACH**: Object attachment points
//!
//! # Coordinate System
//!
//! OOF uses floating-point coordinates (f32).
//!
//! # Example
//!
//! ```no_run
//! use descent_core::oof::OofParser;
//! use std::fs;
//!
//! let data = fs::read("pyro.oof").unwrap();
//! let model = OofParser::parse(&data).unwrap();
//!
//! println!("Model '{}' has {} subobjects", model.name, model.subobjects.len());
//! ```

use crate::error::{AssetError, Result};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read, Seek, SeekFrom};

/// OOF file format version constants from Descent 3 source.
const PM_COMPATIBLE_VERSION: u32 = 1807;
const PM_OBJFILE_VERSION: u32 = 2300;

/// Chunk IDs as u32 (little-endian representation of 4-char codes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ChunkId {
    /// POF file header ('RDHO' = 0x4F524448)
    Ohdr = 0x4F524448,
    /// Subobject header ('JBOS' = 0x534F424A)
    Sobj = 0x534F424A,
    /// Interpreter data ('ATDI' = 0x49544441)
    Idta = 0x49544441,
    /// Texture filename list ('RTXT' = 0x54585452)
    Txtr = 0x54585452,
    /// POF file information ('FNIP' = 0x504E4946)
    Info = 0x504E4946,
    /// Grid information ('DIRG' = 0x47524944)
    Grid = 0x47524944,
    /// Gun points ('TNPG' = 0x474E5054)
    Gpnt = 0x474E5054,
    /// Rotational animation data ('INAR' = 0x52414E49)
    RotAnim = 0x52414E49,
    /// Positional animation data ('INAP' = 0x5041_4E49)
    PosAnim = 0x5041_4E49,
    /// Angular information ('MINA' = 0x414E494D)
    Anim = 0x414E494D,
    /// Weapon Battery Info ('TABW' = 0x57425441)
    Wbs = 0x57425441,
    /// Ground Plane info ('DNRG' = 0x47524E44)
    Ground = 0x47524E44,
    /// Attach points ('HCTA' = 0x41544348)
    Attach = 0x41544348,
    /// Attach uvecs ('HTAN' = 0x4E415448)
    AttachNormals = 0x4E415448,
}

impl ChunkId {
    /// Parse chunk ID from u32 value.
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0x4F524448 => Some(ChunkId::Ohdr),
            0x534F424A => Some(ChunkId::Sobj),
            0x49544441 => Some(ChunkId::Idta),
            0x54585452 => Some(ChunkId::Txtr),
            0x504E4946 => Some(ChunkId::Info),
            0x47524944 => Some(ChunkId::Grid),
            0x474E5054 => Some(ChunkId::Gpnt),
            0x52414E49 => Some(ChunkId::RotAnim),
            0x5041_4E49 => Some(ChunkId::PosAnim),
            0x414E494D => Some(ChunkId::Anim),
            0x57425441 => Some(ChunkId::Wbs),
            0x47524E44 => Some(ChunkId::Ground),
            0x41544348 => Some(ChunkId::Attach),
            0x4E415448 => Some(ChunkId::AttachNormals),
            _ => None,
        }
    }

    /// Get ASCII representation of chunk ID.
    pub fn as_str(&self) -> &'static str {
        match self {
            ChunkId::Ohdr => "OHDR",
            ChunkId::Sobj => "SOBJ",
            ChunkId::Idta => "IDTA",
            ChunkId::Txtr => "TXTR",
            ChunkId::Info => "INFO",
            ChunkId::Grid => "GRID",
            ChunkId::Gpnt => "GPNT",
            ChunkId::RotAnim => "ROT_ANIM",
            ChunkId::PosAnim => "POS_ANIM",
            ChunkId::Anim => "ANIM",
            ChunkId::Wbs => "WBS",
            ChunkId::Ground => "GROUND",
            ChunkId::Attach => "ATTACH",
            ChunkId::AttachNormals => "ATTACH_NORMALS",
        }
    }
}

/// 3D vector (f32 coordinates).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Face/polygon in a subobject.
#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    /// Face normal vector.
    pub normal: Vector3,
    /// Vertex indices into subobject's vertex list.
    pub vertex_indices: Vec<u16>,
    /// UV coordinates for each vertex (if textured).
    pub uvs: Vec<(f32, f32)>,
    /// Texture ID (index into model's texture list).
    pub texture_id: u16,
}

/// Subobject (mesh) in the model hierarchy.
#[derive(Debug, Clone, PartialEq)]
pub struct Subobject {
    /// Subobject name (e.g., "$mainbody", "$wing-left").
    pub name: String,
    /// Parent subobject index (-1 if root).
    pub parent: i32,
    /// Offset from parent.
    pub offset: Vector3,
    /// Geometric center.
    pub center: Vector3,
    /// Bounding radius for collision detection.
    pub radius: f32,
    /// Vertices in this subobject.
    pub vertices: Vec<Vector3>,
    /// Vertex normals.
    pub normals: Vec<Vector3>,
    /// Faces/polygons in this subobject.
    pub faces: Vec<Face>,
    /// Child subobject indices.
    pub children: Vec<usize>,
}

/// Gun point (weapon hardpoint).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GunPoint {
    /// Parent subobject index.
    pub parent: u16,
    /// Position relative to parent.
    pub position: Vector3,
    /// Normal/direction vector.
    pub normal: Vector3,
}

/// Attach point (for object attachment).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttachPoint {
    /// Parent subobject index.
    pub parent: u16,
    /// Position relative to parent.
    pub position: Vector3,
    /// Normal/up vector.
    pub normal: Vector3,
}

/// Weapon battery info (turret group).
#[derive(Debug, Clone, PartialEq)]
pub struct WeaponBattery {
    /// Gun point indices in this battery.
    pub gun_points: Vec<u16>,
    /// Turret subobject indices.
    pub turrets: Vec<u16>,
}

/// Keyframe for animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Keyframe {
    /// Keyframe time/tick.
    pub time: i32,
    /// Rotation axis (normalized).
    pub axis: Vector3,
    /// Rotation angle (fixed-point, units unclear from source).
    pub angle: i32,
}

/// Positional animation keyframe.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PosKeyframe {
    /// Keyframe time/tick.
    pub time: i32,
    /// Position offset.
    pub position: Vector3,
}

/// Animation data for a subobject.
#[derive(Debug, Clone, PartialEq)]
pub struct SubobjectAnimation {
    /// Rotational keyframes (if any).
    pub rotation_keyframes: Vec<Keyframe>,
    /// Positional keyframes (if any).
    pub position_keyframes: Vec<PosKeyframe>,
    /// Rotation track min time.
    pub rot_track_min: i32,
    /// Rotation track max time.
    pub rot_track_max: i32,
    /// Position track min time.
    pub pos_track_min: i32,
    /// Position track max time.
    pub pos_track_max: i32,
}

/// Complete OOF model.
#[derive(Debug, Clone, PartialEq)]
pub struct OofModel {
    /// Model version.
    pub version: u32,
    /// Model name/filename.
    pub name: String,
    /// Subobjects (meshes) in hierarchy.
    pub subobjects: Vec<Subobject>,
    /// Texture filenames (e.g., "metal01").
    pub textures: Vec<String>,
    /// Gun points (weapon hardpoints).
    pub gun_points: Vec<GunPoint>,
    /// Attach points (for object attachment).
    pub attach_points: Vec<AttachPoint>,
    /// Weapon batteries (turret groups).
    pub weapon_batteries: Vec<WeaponBattery>,
    /// Per-subobject animation data.
    pub animations: Vec<SubobjectAnimation>,
    /// Animation frame range.
    pub frame_min: i32,
    /// Animation frame range.
    pub frame_max: i32,
}

impl Default for OofModel {
    fn default() -> Self {
        Self {
            version: PM_OBJFILE_VERSION,
            name: String::new(),
            subobjects: Vec::new(),
            textures: Vec::new(),
            gun_points: Vec::new(),
            attach_points: Vec::new(),
            weapon_batteries: Vec::new(),
            animations: Vec::new(),
            frame_min: 0,
            frame_max: 0,
        }
    }
}

/// OOF file parser.
pub struct OofParser;

impl OofParser {
    /// Parse an OOF file from binary data.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw OOF file data
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use descent_core::oof::OofParser;
    /// # let data = vec![];
    /// let model = OofParser::parse(&data).unwrap();
    /// println!("Loaded model: {}", model.name);
    /// ```
    pub fn parse(data: &[u8]) -> Result<OofModel> {
        let mut cursor = Cursor::new(data);
        let mut model = OofModel::default();

        // Parse chunks until EOF
        while cursor.position() < data.len() as u64 {
            let chunk_id = cursor.read_u32::<LittleEndian>()?;
            let chunk_size = cursor.read_u32::<LittleEndian>()?;
            let chunk_start = cursor.position();

            // Try to parse known chunks
            if let Some(id) = ChunkId::from_u32(chunk_id) {
                match id {
                    ChunkId::Ohdr => Self::parse_ohdr(&mut cursor, &mut model)?,
                    ChunkId::Sobj => Self::parse_sobj(&mut cursor, &mut model)?,
                    ChunkId::Txtr => Self::parse_txtr(&mut cursor, &mut model)?,
                    ChunkId::Gpnt => Self::parse_gpnt(&mut cursor, &mut model)?,
                    ChunkId::Attach => Self::parse_attach(&mut cursor, &mut model)?,
                    ChunkId::Wbs => Self::parse_wbs(&mut cursor, &mut model)?,
                    ChunkId::RotAnim => Self::parse_rot_anim(&mut cursor, &mut model)?,
                    ChunkId::PosAnim => Self::parse_pos_anim(&mut cursor, &mut model)?,
                    // Ignore other chunks for now
                    _ => {}
                }
            }

            // Seek to next chunk (skip any unread data in this chunk)
            cursor.seek(SeekFrom::Start(chunk_start + chunk_size as u64))?;
        }

        Ok(model)
    }

    /// Parse OHDR (header) chunk.
    fn parse_ohdr(cursor: &mut Cursor<&[u8]>, model: &mut OofModel) -> Result<()> {
        model.version = cursor.read_u32::<LittleEndian>()?;

        // Validate version
        if model.version < PM_COMPATIBLE_VERSION || model.version > PM_OBJFILE_VERSION {
            return Err(AssetError::InvalidFormat(format!(
                "Unsupported OOF version: {} (expected {}-{})",
                model.version, PM_COMPATIBLE_VERSION, PM_OBJFILE_VERSION
            )));
        }

        // Read model name (null-terminated string, max 35 chars based on D3 source)
        let mut name_bytes = vec![0u8; 36];
        cursor.read_exact(&mut name_bytes)?;
        if let Some(null_pos) = name_bytes.iter().position(|&b| b == 0) {
            name_bytes.truncate(null_pos);
        }
        model.name = String::from_utf8_lossy(&name_bytes).to_string();

        // Read additional header fields
        let n_models = cursor.read_u32::<LittleEndian>()?;
        model.subobjects = Vec::with_capacity(n_models as usize);

        // Initialize animations vector
        model.animations = vec![
            SubobjectAnimation {
                rotation_keyframes: Vec::new(),
                position_keyframes: Vec::new(),
                rot_track_min: 0,
                rot_track_max: 0,
                pos_track_min: 0,
                pos_track_max: 0,
            };
            n_models as usize
        ];

        Ok(())
    }

    /// Parse SOBJ (subobject) chunk.
    fn parse_sobj(cursor: &mut Cursor<&[u8]>, model: &mut OofModel) -> Result<()> {
        let mut subobj = Subobject {
            name: String::new(),
            parent: 0,
            offset: Vector3::default(),
            center: Vector3::default(),
            radius: 0.0,
            vertices: Vec::new(),
            normals: Vec::new(),
            faces: Vec::new(),
            children: Vec::new(),
        };

        // Read subobject index
        let _subobj_num = cursor.read_u32::<LittleEndian>()?;

        // Read subobject name
        let name_len = cursor.read_u8()? as usize;
        let mut name_bytes = vec![0u8; name_len];
        cursor.read_exact(&mut name_bytes)?;
        subobj.name = String::from_utf8_lossy(&name_bytes).to_string();

        // Read parent index
        subobj.parent = cursor.read_i32::<LittleEndian>()?;

        // Read geometric properties
        subobj.offset = Self::read_vector3(cursor)?;
        subobj.radius = cursor.read_f32::<LittleEndian>()?;
        subobj.center = Self::read_vector3(cursor)?;

        // Read vertices
        let num_verts = cursor.read_u32::<LittleEndian>()?;
        subobj.vertices.reserve(num_verts as usize);
        for _ in 0..num_verts {
            subobj.vertices.push(Self::read_vector3(cursor)?);
        }

        // Read normals
        let num_normals = cursor.read_u32::<LittleEndian>()?;
        subobj.normals.reserve(num_normals as usize);
        for _ in 0..num_normals {
            subobj.normals.push(Self::read_vector3(cursor)?);
        }

        // Read faces
        let num_faces = cursor.read_u32::<LittleEndian>()?;
        subobj.faces.reserve(num_faces as usize);
        for _ in 0..num_faces {
            let face = Self::parse_face(cursor)?;
            subobj.faces.push(face);
        }

        model.subobjects.push(subobj);
        Ok(())
    }

    /// Parse a face/polygon.
    fn parse_face(cursor: &mut Cursor<&[u8]>) -> Result<Face> {
        let mut face = Face {
            normal: Vector3::default(),
            vertex_indices: Vec::new(),
            uvs: Vec::new(),
            texture_id: 0,
        };

        // Read face normal
        face.normal = Self::read_vector3(cursor)?;

        // Read number of vertices
        let num_verts = cursor.read_u8()? as usize;

        // Read texture ID
        face.texture_id = cursor.read_u16::<LittleEndian>()?;

        // Read vertex indices
        face.vertex_indices.reserve(num_verts);
        for _ in 0..num_verts {
            face.vertex_indices.push(cursor.read_u16::<LittleEndian>()?);
        }

        // Read UV coordinates
        face.uvs.reserve(num_verts);
        for _ in 0..num_verts {
            let u = cursor.read_f32::<LittleEndian>()?;
            let v = cursor.read_f32::<LittleEndian>()?;
            face.uvs.push((u, v));
        }

        Ok(face)
    }

    /// Parse TXTR (texture list) chunk.
    fn parse_txtr(cursor: &mut Cursor<&[u8]>, model: &mut OofModel) -> Result<()> {
        let num_textures = cursor.read_u32::<LittleEndian>()?;
        model.textures.reserve(num_textures as usize);

        for _ in 0..num_textures {
            // Read texture name length
            let name_len = cursor.read_u8()? as usize;

            // Read texture name
            let mut name_bytes = vec![0u8; name_len];
            cursor.read_exact(&mut name_bytes)?;
            let name = String::from_utf8_lossy(&name_bytes).to_string();

            model.textures.push(name);
        }

        Ok(())
    }

    /// Parse GPNT (gun points) chunk.
    fn parse_gpnt(cursor: &mut Cursor<&[u8]>, model: &mut OofModel) -> Result<()> {
        let num_gun_points = cursor.read_u32::<LittleEndian>()?;
        model.gun_points.reserve(num_gun_points as usize);

        for _ in 0..num_gun_points {
            let parent = cursor.read_u16::<LittleEndian>()?;
            let position = Self::read_vector3(cursor)?;
            let normal = Self::read_vector3(cursor)?;

            model.gun_points.push(GunPoint {
                parent,
                position,
                normal,
            });
        }

        Ok(())
    }

    /// Parse ATTACH (attach points) chunk.
    fn parse_attach(cursor: &mut Cursor<&[u8]>, model: &mut OofModel) -> Result<()> {
        let num_attach = cursor.read_u32::<LittleEndian>()?;
        model.attach_points.reserve(num_attach as usize);

        for _ in 0..num_attach {
            let parent = cursor.read_u16::<LittleEndian>()?;
            let position = Self::read_vector3(cursor)?;
            let normal = Self::read_vector3(cursor)?;

            model.attach_points.push(AttachPoint {
                parent,
                position,
                normal,
            });
        }

        Ok(())
    }

    /// Parse WBS (weapon batteries) chunk.
    fn parse_wbs(cursor: &mut Cursor<&[u8]>, model: &mut OofModel) -> Result<()> {
        let num_batteries = cursor.read_u32::<LittleEndian>()?;
        model.weapon_batteries.reserve(num_batteries as usize);

        for _ in 0..num_batteries {
            let num_gps = cursor.read_u32::<LittleEndian>()?;
            let mut gun_points = Vec::with_capacity(num_gps as usize);
            for _ in 0..num_gps {
                gun_points.push(cursor.read_u16::<LittleEndian>()?);
            }

            let num_turrets = cursor.read_u32::<LittleEndian>()?;
            let mut turrets = Vec::with_capacity(num_turrets as usize);
            for _ in 0..num_turrets {
                turrets.push(cursor.read_u16::<LittleEndian>()?);
            }

            model.weapon_batteries.push(WeaponBattery {
                gun_points,
                turrets,
            });
        }

        Ok(())
    }

    /// Parse ROT_ANIM (rotational animation) chunk.
    fn parse_rot_anim(cursor: &mut Cursor<&[u8]>, model: &mut OofModel) -> Result<()> {
        // This is a timed animation format
        for i in 0..model.subobjects.len() {
            let num_keyframes = cursor.read_u32::<LittleEndian>()?;
            let rot_track_min = cursor.read_i32::<LittleEndian>()?;
            let rot_track_max = cursor.read_i32::<LittleEndian>()?;

            if i < model.animations.len() {
                model.animations[i].rot_track_min = rot_track_min;
                model.animations[i].rot_track_max = rot_track_max;

                // Update model frame range
                if rot_track_min < model.frame_min {
                    model.frame_min = rot_track_min;
                }
                if rot_track_max > model.frame_max {
                    model.frame_max = rot_track_max;
                }

                model.animations[i]
                    .rotation_keyframes
                    .reserve(num_keyframes as usize);

                for _ in 0..num_keyframes {
                    let time = cursor.read_i32::<LittleEndian>()?;
                    let axis = Self::read_vector3(cursor)?;
                    let angle = cursor.read_i32::<LittleEndian>()?;

                    model.animations[i]
                        .rotation_keyframes
                        .push(Keyframe { time, axis, angle });
                }
            }
        }

        Ok(())
    }

    /// Parse POS_ANIM (positional animation) chunk.
    fn parse_pos_anim(cursor: &mut Cursor<&[u8]>, model: &mut OofModel) -> Result<()> {
        // This is a timed animation format
        for i in 0..model.subobjects.len() {
            let num_keyframes = cursor.read_u32::<LittleEndian>()?;
            let pos_track_min = cursor.read_i32::<LittleEndian>()?;
            let pos_track_max = cursor.read_i32::<LittleEndian>()?;

            if i < model.animations.len() {
                model.animations[i].pos_track_min = pos_track_min;
                model.animations[i].pos_track_max = pos_track_max;

                // Update model frame range
                if pos_track_min < model.frame_min {
                    model.frame_min = pos_track_min;
                }
                if pos_track_max > model.frame_max {
                    model.frame_max = pos_track_max;
                }

                model.animations[i]
                    .position_keyframes
                    .reserve(num_keyframes as usize);

                for _ in 0..num_keyframes {
                    let time = cursor.read_i32::<LittleEndian>()?;
                    let position = Self::read_vector3(cursor)?;

                    model.animations[i]
                        .position_keyframes
                        .push(PosKeyframe { time, position });
                }
            }
        }

        Ok(())
    }

    /// Read a Vector3 from cursor.
    fn read_vector3(cursor: &mut Cursor<&[u8]>) -> Result<Vector3> {
        Ok(Vector3 {
            x: cursor.read_f32::<LittleEndian>()?,
            y: cursor.read_f32::<LittleEndian>()?,
            z: cursor.read_f32::<LittleEndian>()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_id_parsing() {
        assert_eq!(ChunkId::from_u32(0x4F524448), Some(ChunkId::Ohdr));
        assert_eq!(ChunkId::from_u32(0x534F424A), Some(ChunkId::Sobj));
        assert_eq!(ChunkId::from_u32(0x54585452), Some(ChunkId::Txtr));
        assert_eq!(ChunkId::from_u32(0x12345678), None);
    }

    #[test]
    fn test_chunk_id_strings() {
        assert_eq!(ChunkId::Ohdr.as_str(), "OHDR");
        assert_eq!(ChunkId::Sobj.as_str(), "SOBJ");
        assert_eq!(ChunkId::Txtr.as_str(), "TXTR");
    }

    #[test]
    fn test_default_model() {
        let model = OofModel::default();
        assert_eq!(model.version, PM_OBJFILE_VERSION);
        assert_eq!(model.name, "");
        assert_eq!(model.subobjects.len(), 0);
        assert_eq!(model.textures.len(), 0);
    }
}

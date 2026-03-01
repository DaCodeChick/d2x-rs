//! POF (Polygon Object File) model parser for Descent 1 and Descent 2.
//!
//! POF files are polygon-based 3D models used for ships, robots, powerups, and other
//! game objects. The format uses a bytecode-like opcode interpreter system for encoding
//! model geometry, materials, and hierarchy.
//!
//! # Format Overview
//!
//! POF files consist of a sequence of opcodes (16-bit little-endian values) that define:
//! - Vertex positions (fixed-point coordinates)
//! - Flat-shaded polygons (solid colors)
//! - Texture-mapped polygons (with UV coordinates)
//! - BSP sorting nodes (for proper rendering order)
//! - Rod bitmaps (cylindrical sprites)
//! - Submodel hierarchy (recursive geometry)
//! - Glow points (engine effects, weapon muzzles)
//!
//! # Coordinate System
//!
//! POF uses a fixed-point coordinate system where coordinates are stored as `i32` values.
//! To convert to floating-point: `value as f32 / 65536.0`
//!
//! # Example
//!
//! ```no_run
//! use descent_core::pof::PofParser;
//! use std::fs;
//!
//! let data = fs::read("pyrogl.pof").unwrap();
//! let model = PofParser::parse(&data).unwrap();
//!
//! println!("Model has {} vertices", model.vertices.len());
//! println!("Model has {} polygons", model.polygons.len());
//! ```

use crate::error::{AssetError, Result};
use crate::fixed_point::Fix;
use crate::geometry::{FixVector, Uvl};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Seek, SeekFrom};

/// POF opcode enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Opcode {
    /// End of model data.
    Eof = 0,
    /// Define vertex positions.
    DefPoints = 1,
    /// Flat-shaded polygon.
    FlatPoly = 2,
    /// Texture-mapped polygon.
    TmapPoly = 3,
    /// BSP sorting node.
    SortNorm = 4,
    /// Rod bitmap (cylindrical billboard).
    RodBm = 5,
    /// Submodel call (hierarchical geometry).
    SubCall = 6,
    /// Define points with starting index.
    DefPStart = 7,
    /// Glow point definition.
    Glow = 8,
}

impl Opcode {
    /// Parse opcode from u16 value.
    pub fn from_u16(value: u16) -> Result<Self> {
        match value {
            0 => Ok(Opcode::Eof),
            1 => Ok(Opcode::DefPoints),
            2 => Ok(Opcode::FlatPoly),
            3 => Ok(Opcode::TmapPoly),
            4 => Ok(Opcode::SortNorm),
            5 => Ok(Opcode::RodBm),
            6 => Ok(Opcode::SubCall),
            7 => Ok(Opcode::DefPStart),
            8 => Ok(Opcode::Glow),
            _ => Err(AssetError::InvalidFormat(format!(
                "Unknown POF opcode: {}",
                value
            ))),
        }
    }
}

/// Flat-shaded polygon with solid color.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlatPolygon {
    pub center: FixVector,
    pub normal: FixVector,
    pub color: u16,
    pub vertices: Vec<u16>,
}

/// Texture-mapped polygon with UV coordinates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TexturedPolygon {
    pub center: FixVector,
    pub normal: FixVector,
    pub texture_id: u16,
    pub vertices: Vec<u16>,
    pub uvls: Vec<Uvl>,
}

/// Rod bitmap (cylindrical billboard sprite).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RodBitmap {
    pub texture_id: u16,
    pub bot_point: FixVector,
    pub bot_width: Fix,
    pub top_point: FixVector,
    pub top_width: Fix,
}

/// Submodel reference (hierarchical geometry).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubmodelCall {
    pub submodel_num: u16,
    pub offset: FixVector,
    pub data_offset: u16,
}

/// Glow point (engine glow, weapon muzzle flash, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlowPoint {
    pub glow_num: u16,
}

/// Polygon type enumeration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Polygon {
    Flat(FlatPolygon),
    Textured(TexturedPolygon),
}

/// A POF model containing geometry, materials, and hierarchy.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PofModel {
    /// All vertices defined in the model.
    pub vertices: Vec<FixVector>,
    /// All polygons (flat and textured).
    pub polygons: Vec<Polygon>,
    /// Rod bitmaps (cylindrical billboards).
    pub rod_bitmaps: Vec<RodBitmap>,
    /// Submodel calls (hierarchical geometry).
    pub submodel_calls: Vec<SubmodelCall>,
    /// Glow points (engine effects, weapon muzzles).
    pub glow_points: Vec<GlowPoint>,
    /// Number of textures used by this model.
    pub n_textures: u8,
    /// Starting index in the texture table (used with HAM file).
    pub first_texture: u16,
}

/// POF parser with opcode dispatcher.
pub struct PofParser<'a> {
    cursor: Cursor<&'a [u8]>,
    model: PofModel,
}

impl<'a> PofParser<'a> {
    /// Parse a POF model from byte data (without header).
    pub fn parse(data: &'a [u8]) -> Result<PofModel> {
        let mut parser = Self {
            cursor: Cursor::new(data),
            model: PofModel::default(),
        };
        parser.parse_opcodes()?;
        Ok(parser.model)
    }

    /// Parse a POF model with full header (as stored in HAM files).
    ///
    /// The header contains submodel information, bounding boxes, and texture metadata.
    /// This format is used when POF data is embedded in HAM files.
    pub fn parse_with_header(data: &'a [u8]) -> Result<PofModel> {
        let mut parser = Self {
            cursor: Cursor::new(data),
            model: PofModel::default(),
        };
        parser.parse_header()?;
        parser.parse_opcodes()?;
        Ok(parser.model)
    }

    /// Parse POF header structure.
    ///
    /// Header structure (all values little-endian):
    /// - n_models: i32
    /// - data_size: i32
    /// - unused: i32
    /// - submodel data: 10 submodels × various fields (not needed for texture extraction)
    /// - model_mins: FixVector (12 bytes)
    /// - model_maxs: FixVector (12 bytes)
    /// - model_rad: Fix (4 bytes)
    /// - n_textures: u8
    /// - first_texture: u16
    /// - simpler_model: u8
    fn parse_header(&mut self) -> Result<()> {
        const MAX_SUBMODELS: usize = 10;

        // Read basic header
        let _n_models = self.cursor.read_i32::<LittleEndian>()?;
        let _data_size = self.cursor.read_i32::<LittleEndian>()?;
        let _unused = self.cursor.read_i32::<LittleEndian>()?;

        // Skip submodel data (we don't need it for texture extraction)
        self.skip_submodel_arrays(MAX_SUBMODELS)?;

        // Model-level bounds and radius
        let _model_mins = self.read_fixvector()?;
        let _model_maxs = self.read_fixvector()?;
        let _model_rad = self.cursor.read_i32::<LittleEndian>()?;

        // Texture metadata (what we actually need!)
        self.model.n_textures = self.cursor.read_u8()?;
        self.model.first_texture = self.cursor.read_u16::<LittleEndian>()?;
        let _simpler_model = self.cursor.read_u8()?;

        Ok(())
    }

    /// Skip submodel arrays in header (we don't use them for conversion).
    fn skip_submodel_arrays(&mut self, max_submodels: usize) -> Result<()> {
        // submodel_ptrs[10]: i32 × 10
        (0..max_submodels).try_for_each(|_| self.cursor.read_i32::<LittleEndian>().map(|_| ()))?;
        // submodel_offsets[10]: FixVector × 10 (12 bytes each)
        (0..max_submodels).try_for_each(|_| self.read_fixvector().map(|_| ()))?;
        // submodel_norms[10]: FixVector × 10
        (0..max_submodels).try_for_each(|_| self.read_fixvector().map(|_| ()))?;
        // submodel_pnts[10]: FixVector × 10
        (0..max_submodels).try_for_each(|_| self.read_fixvector().map(|_| ()))?;
        // submodel_rads[10]: Fix × 10 (i32)
        (0..max_submodels).try_for_each(|_| self.cursor.read_i32::<LittleEndian>().map(|_| ()))?;
        // submodel_parents[10]: u8 × 10
        (0..max_submodels).try_for_each(|_| self.cursor.read_u8().map(|_| ()))?;
        // submodel_mins[10]: FixVector × 10
        (0..max_submodels).try_for_each(|_| self.read_fixvector().map(|_| ()))?;
        // submodel_maxs[10]: FixVector × 10
        (0..max_submodels).try_for_each(|_| self.read_fixvector().map(|_| ()))?;
        Ok(())
    }

    /// Main opcode parsing loop.
    fn parse_opcodes(&mut self) -> Result<()> {
        loop {
            let opcode_value = self.cursor.read_u16::<LittleEndian>()?;
            let opcode = Opcode::from_u16(opcode_value)?;

            match opcode {
                Opcode::Eof => break,
                Opcode::DefPoints => self.parse_defpoints()?,
                Opcode::FlatPoly => self.parse_flatpoly()?,
                Opcode::TmapPoly => self.parse_tmappoly()?,
                Opcode::SortNorm => self.parse_sortnorm()?,
                Opcode::RodBm => self.parse_rodbm()?,
                Opcode::SubCall => self.parse_subcall()?,
                Opcode::DefPStart => self.parse_defpstart()?,
                Opcode::Glow => self.parse_glow()?,
            }
        }
        Ok(())
    }

    /// Parse OP_DEFPOINTS: Define vertex positions.
    /// Structure: [opcode:u16][count:u16][points:FixVector×n]
    fn parse_defpoints(&mut self) -> Result<()> {
        let count = self.cursor.read_u16::<LittleEndian>()? as usize;
        let vertices = (0..count)
            .map(|_| self.read_fixvector())
            .collect::<Result<Vec<_>>>()?;
        self.model.vertices.extend(vertices);
        Ok(())
    }

    /// Parse OP_FLATPOLY: Flat-shaded polygon.
    /// Structure: [opcode:u16][nverts:u16][center:vec][normal:vec][color:u16][verts:u16×n]
    /// Size: 30 + ((n|1))×2 bytes
    fn parse_flatpoly(&mut self) -> Result<()> {
        let nverts = self.cursor.read_u16::<LittleEndian>()? as usize;
        let center = self.read_fixvector()?;
        let normal = self.read_fixvector()?;
        let color = self.cursor.read_u16::<LittleEndian>()?;

        let vertices = (0..nverts)
            .map(|_| self.cursor.read_u16::<LittleEndian>().map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;

        // Padding: vertex count is padded to odd alignment
        if nverts.is_multiple_of(2) {
            let _padding = self.cursor.read_u16::<LittleEndian>()?;
        }

        self.model.polygons.push(Polygon::Flat(FlatPolygon {
            center,
            normal,
            color,
            vertices,
        }));

        Ok(())
    }

    /// Parse OP_TMAPPOLY: Texture-mapped polygon.
    /// Structure: [opcode:u16][nverts:u16][center:vec][normal:vec][texture:u16][verts:u16×n][uvls:Uvl×n]
    /// Size: 30 + ((n|1))×2 + n×12 bytes
    fn parse_tmappoly(&mut self) -> Result<()> {
        let nverts = self.cursor.read_u16::<LittleEndian>()? as usize;
        let center = self.read_fixvector()?;
        let normal = self.read_fixvector()?;
        let texture_id = self.cursor.read_u16::<LittleEndian>()?;

        let vertices = (0..nverts)
            .map(|_| self.cursor.read_u16::<LittleEndian>().map_err(Into::into))
            .collect::<Result<Vec<_>>>()?;

        // Padding: vertex count is padded to odd alignment
        if nverts.is_multiple_of(2) {
            let _padding = self.cursor.read_u16::<LittleEndian>()?;
        }

        let uvls = (0..nverts)
            .map(|_| self.read_uvl())
            .collect::<Result<Vec<_>>>()?;

        self.model.polygons.push(Polygon::Textured(TexturedPolygon {
            center,
            normal,
            texture_id,
            vertices,
            uvls,
        }));

        Ok(())
    }

    /// Parse OP_SORTNORM: BSP sorting node.
    /// Structure: [opcode:u16][unused:u16][point:vec][normal:vec][front_offset:u16][back_offset:u16]
    /// Size: 32 bytes
    ///
    /// SORTNORM creates a BSP tree for proper front-to-back polygon sorting.
    /// The front_offset and back_offset point to child nodes in the opcode stream.
    fn parse_sortnorm(&mut self) -> Result<()> {
        let _unused = self.cursor.read_u16::<LittleEndian>()?;
        let _point = self.read_fixvector()?;
        let _normal = self.read_fixvector()?;
        let front_offset = self.cursor.read_u16::<LittleEndian>()?;
        let back_offset = self.cursor.read_u16::<LittleEndian>()?;

        // Save current position
        let current_pos = self.cursor.position();

        // Parse front side (recursively)
        if front_offset > 0 {
            self.cursor.seek(SeekFrom::Start(front_offset as u64))?;
            self.parse_opcodes()?;
        }

        // Parse back side (recursively)
        if back_offset > 0 {
            self.cursor.seek(SeekFrom::Start(back_offset as u64))?;
            self.parse_opcodes()?;
        }

        // Restore position to continue after SORTNORM
        self.cursor.seek(SeekFrom::Start(current_pos))?;

        Ok(())
    }

    /// Parse OP_RODBM: Rod bitmap (cylindrical billboard).
    /// Structure: [opcode:u16][texture:u16][bot_point:vec][bot_width:fix][top_point:vec][top_width:fix]
    /// Size: 36 bytes
    fn parse_rodbm(&mut self) -> Result<()> {
        let texture_id = self.cursor.read_u16::<LittleEndian>()?;
        let bot_point = self.read_fixvector()?;
        let bot_width = self.read_fix()?;
        let top_point = self.read_fixvector()?;
        let top_width = self.read_fix()?;

        self.model.rod_bitmaps.push(RodBitmap {
            texture_id,
            bot_point,
            bot_width,
            top_point,
            top_width,
        });

        Ok(())
    }

    /// Parse OP_SUBCALL: Submodel call (hierarchical geometry).
    /// Structure: [opcode:u16][submodel:u16][offset:vec][data_offset:u16]
    /// Size: 20 bytes
    fn parse_subcall(&mut self) -> Result<()> {
        let submodel_num = self.cursor.read_u16::<LittleEndian>()?;
        let offset = self.read_fixvector()?;
        let data_offset = self.cursor.read_u16::<LittleEndian>()?;

        self.model.submodel_calls.push(SubmodelCall {
            submodel_num,
            offset,
            data_offset,
        });

        // Save current position
        let current_pos = self.cursor.position();

        // Parse submodel data recursively
        if data_offset > 0 {
            self.cursor.seek(SeekFrom::Start(data_offset as u64))?;
            self.parse_opcodes()?;
        }

        // Restore position to continue after SUBCALL
        self.cursor.seek(SeekFrom::Start(current_pos))?;

        Ok(())
    }

    /// Parse OP_DEFP_START: Define points with starting index.
    /// Structure: [opcode:u16][count:u16][start:u16][unused:u16][points:FixVector×n]
    /// Size: 8 + n×12 bytes
    fn parse_defpstart(&mut self) -> Result<()> {
        let count = self.cursor.read_u16::<LittleEndian>()? as usize;
        let start = self.cursor.read_u16::<LittleEndian>()? as usize;
        let _unused = self.cursor.read_u16::<LittleEndian>()?;

        // Ensure vertices vector is large enough
        if self.model.vertices.len() < start + count {
            self.model.vertices.resize(start + count, FixVector::ZERO);
        }

        for i in 0..count {
            self.model.vertices[start + i] = self.read_fixvector()?;
        }

        Ok(())
    }

    /// Parse OP_GLOW: Glow point definition.
    /// Structure: [opcode:u16][glow_num:u16]
    /// Size: 4 bytes
    fn parse_glow(&mut self) -> Result<()> {
        let glow_num = self.cursor.read_u16::<LittleEndian>()?;
        self.model.glow_points.push(GlowPoint { glow_num });
        Ok(())
    }

    // ===== Binary reading helpers =====

    fn read_fix(&mut self) -> Result<Fix> {
        Ok(Fix::from(self.cursor.read_i32::<LittleEndian>()?))
    }

    fn read_fixvector(&mut self) -> Result<FixVector> {
        Ok(FixVector {
            x: self.read_fix()?,
            y: self.read_fix()?,
            z: self.read_fix()?,
        })
    }

    fn read_uvl(&mut self) -> Result<Uvl> {
        Ok(Uvl {
            u: self.read_fix()?,
            v: self.read_fix()?,
            l: self.read_fix()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create test data with opcodes
    fn create_test_data(opcodes: &[u8]) -> Vec<u8> {
        opcodes.to_vec()
    }

    #[test]
    fn test_opcode_from_u16() {
        assert_eq!(Opcode::from_u16(0).unwrap(), Opcode::Eof);
        assert_eq!(Opcode::from_u16(1).unwrap(), Opcode::DefPoints);
        assert_eq!(Opcode::from_u16(2).unwrap(), Opcode::FlatPoly);
        assert_eq!(Opcode::from_u16(3).unwrap(), Opcode::TmapPoly);
        assert_eq!(Opcode::from_u16(4).unwrap(), Opcode::SortNorm);
        assert_eq!(Opcode::from_u16(5).unwrap(), Opcode::RodBm);
        assert_eq!(Opcode::from_u16(6).unwrap(), Opcode::SubCall);
        assert_eq!(Opcode::from_u16(7).unwrap(), Opcode::DefPStart);
        assert_eq!(Opcode::from_u16(8).unwrap(), Opcode::Glow);
        assert!(Opcode::from_u16(999).is_err());
    }

    #[test]
    fn test_fixvector_to_f32() {
        let v = FixVector {
            x: Fix::from(65536_i32),  // 1.0
            y: Fix::from(32768_i32),  // 0.5
            z: Fix::from(-65536_i32), // -1.0
        };
        let f = v.to_f32();
        assert!((f[0] - 1.0).abs() < 0.0001);
        assert!((f[1] - 0.5).abs() < 0.0001);
        assert!((f[2] - -1.0).abs() < 0.0001);
    }

    #[test]
    fn test_uvl_to_f32() {
        let uvl = Uvl {
            u: Fix::from(65536_i32),  // 1.0
            v: Fix::from(32768_i32),  // 0.5
            l: Fix::from(131072_i32), // 2.0
        };
        let f = uvl.to_f32();
        assert!((f[0] - 1.0).abs() < 0.0001);
        assert!((f[1] - 0.5).abs() < 0.0001);
        assert!((f[2] - 2.0).abs() < 0.0001);
    }

    #[test]
    fn test_parse_empty_model() {
        // Just EOF opcode
        let data = create_test_data(&[0x00, 0x00]); // OP_EOF
        let model = PofParser::parse(&data).unwrap();
        assert_eq!(model.vertices.len(), 0);
        assert_eq!(model.polygons.len(), 0);
    }

    #[test]
    fn test_parse_defpoints() {
        // OP_DEFPOINTS with 2 vertices + OP_EOF
        let mut data = vec![];
        data.extend_from_slice(&[0x01, 0x00]); // OP_DEFPOINTS
        data.extend_from_slice(&[0x02, 0x00]); // count = 2
                                               // Vertex 1: (1.0, 2.0, 3.0) in fixed-point
        data.extend_from_slice(&65536i32.to_le_bytes()); // x = 1.0
        data.extend_from_slice(&131072i32.to_le_bytes()); // y = 2.0
        data.extend_from_slice(&196608i32.to_le_bytes()); // z = 3.0
                                                          // Vertex 2: (4.0, 5.0, 6.0) in fixed-point
        data.extend_from_slice(&262144i32.to_le_bytes()); // x = 4.0
        data.extend_from_slice(&327680i32.to_le_bytes()); // y = 5.0
        data.extend_from_slice(&393216i32.to_le_bytes()); // z = 6.0
        data.extend_from_slice(&[0x00, 0x00]); // OP_EOF

        let model = PofParser::parse(&data).unwrap();
        assert_eq!(model.vertices.len(), 2);

        let v0 = model.vertices[0].to_f32();
        assert!((v0[0] - 1.0).abs() < 0.0001);
        assert!((v0[1] - 2.0).abs() < 0.0001);
        assert!((v0[2] - 3.0).abs() < 0.0001);

        let v1 = model.vertices[1].to_f32();
        assert!((v1[0] - 4.0).abs() < 0.0001);
        assert!((v1[1] - 5.0).abs() < 0.0001);
        assert!((v1[2] - 6.0).abs() < 0.0001);
    }

    #[test]
    fn test_parse_flatpoly() {
        // OP_FLATPOLY with 3 vertices (triangle) + OP_EOF
        let mut data = vec![];
        data.extend_from_slice(&[0x02, 0x00]); // OP_FLATPOLY
        data.extend_from_slice(&[0x03, 0x00]); // nverts = 3
                                               // Center: (0, 0, 0)
        data.extend_from_slice(&[0; 12]);
        // Normal: (0, 0, 1.0)
        data.extend_from_slice(&[0; 8]);
        data.extend_from_slice(&65536i32.to_le_bytes());
        data.extend_from_slice(&[0x0F, 0x00]); // color = 15
                                               // Vertex indices: 0, 1, 2
        data.extend_from_slice(&[0x00, 0x00]);
        data.extend_from_slice(&[0x01, 0x00]);
        data.extend_from_slice(&[0x02, 0x00]);
        // Padding (nverts=3 is odd, so (3|1)=3, no padding needed actually)
        // But implementation expects padding for even counts
        data.extend_from_slice(&[0x00, 0x00]); // OP_EOF

        let model = PofParser::parse(&data).unwrap();
        assert_eq!(model.polygons.len(), 1);

        match &model.polygons[0] {
            Polygon::Flat(poly) => {
                assert_eq!(poly.vertices.len(), 3);
                assert_eq!(poly.color, 15);
                assert_eq!(poly.vertices, vec![0, 1, 2]);
            }
            _ => panic!("Expected flat polygon"),
        }
    }

    #[test]
    fn test_parse_tmappoly() {
        // OP_TMAPPOLY with 3 vertices (triangle) + OP_EOF
        let mut data = vec![];
        data.extend_from_slice(&[0x03, 0x00]); // OP_TMAPPOLY
        data.extend_from_slice(&[0x03, 0x00]); // nverts = 3
                                               // Center: (0, 0, 0)
        data.extend_from_slice(&[0; 12]);
        // Normal: (0, 0, 1.0)
        data.extend_from_slice(&[0; 8]);
        data.extend_from_slice(&65536i32.to_le_bytes());
        data.extend_from_slice(&[0x05, 0x00]); // texture_id = 5
                                               // Vertex indices: 0, 1, 2
        data.extend_from_slice(&[0x00, 0x00]);
        data.extend_from_slice(&[0x01, 0x00]);
        data.extend_from_slice(&[0x02, 0x00]);
        // UVL data for 3 vertices
        // Vertex 0: u=0.0, v=0.0, l=1.0
        data.extend_from_slice(&[0; 8]);
        data.extend_from_slice(&65536i32.to_le_bytes());
        // Vertex 1: u=1.0, v=0.0, l=1.0
        data.extend_from_slice(&65536i32.to_le_bytes());
        data.extend_from_slice(&[0; 4]);
        data.extend_from_slice(&65536i32.to_le_bytes());
        // Vertex 2: u=0.5, v=1.0, l=1.0
        data.extend_from_slice(&32768i32.to_le_bytes());
        data.extend_from_slice(&65536i32.to_le_bytes());
        data.extend_from_slice(&65536i32.to_le_bytes());
        data.extend_from_slice(&[0x00, 0x00]); // OP_EOF

        let model = PofParser::parse(&data).unwrap();
        assert_eq!(model.polygons.len(), 1);

        match &model.polygons[0] {
            Polygon::Textured(poly) => {
                assert_eq!(poly.vertices.len(), 3);
                assert_eq!(poly.texture_id, 5);
                assert_eq!(poly.vertices, vec![0, 1, 2]);
                assert_eq!(poly.uvls.len(), 3);

                let uv0 = poly.uvls[0].to_f32();
                assert!((uv0[0] - 0.0).abs() < 0.0001);
                assert!((uv0[1] - 0.0).abs() < 0.0001);
            }
            _ => panic!("Expected textured polygon"),
        }
    }

    #[test]
    fn test_parse_glow() {
        // OP_GLOW + OP_EOF
        let mut data = vec![];
        data.extend_from_slice(&[0x08, 0x00]); // OP_GLOW
        data.extend_from_slice(&[0x03, 0x00]); // glow_num = 3
        data.extend_from_slice(&[0x00, 0x00]); // OP_EOF

        let model = PofParser::parse(&data).unwrap();
        assert_eq!(model.glow_points.len(), 1);
        assert_eq!(model.glow_points[0].glow_num, 3);
    }

    #[test]
    fn test_parse_rodbm() {
        // OP_RODBM + OP_EOF
        let mut data = vec![];
        data.extend_from_slice(&[0x05, 0x00]); // OP_RODBM
        data.extend_from_slice(&[0x0A, 0x00]); // texture_id = 10
                                               // bot_point: (1.0, 0, 0)
        data.extend_from_slice(&65536i32.to_le_bytes());
        data.extend_from_slice(&[0; 8]);
        // bot_width: 0.5
        data.extend_from_slice(&32768i32.to_le_bytes());
        // top_point: (1.0, 1.0, 0)
        data.extend_from_slice(&65536i32.to_le_bytes());
        data.extend_from_slice(&65536i32.to_le_bytes());
        data.extend_from_slice(&[0; 4]);
        // top_width: 0.25
        data.extend_from_slice(&16384i32.to_le_bytes());
        data.extend_from_slice(&[0x00, 0x00]); // OP_EOF

        let model = PofParser::parse(&data).unwrap();
        assert_eq!(model.rod_bitmaps.len(), 1);
        assert_eq!(model.rod_bitmaps[0].texture_id, 10);

        let bot = model.rod_bitmaps[0].bot_point.to_f32();
        assert!((bot[0] - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_parse_defpstart() {
        // OP_DEFP_START with start=5, count=2 + OP_EOF
        let mut data = vec![];
        data.extend_from_slice(&[0x07, 0x00]); // OP_DEFP_START
        data.extend_from_slice(&[0x02, 0x00]); // count = 2
        data.extend_from_slice(&[0x05, 0x00]); // start = 5
        data.extend_from_slice(&[0x00, 0x00]); // unused
                                               // Vertex at index 5: (1.0, 2.0, 3.0)
        data.extend_from_slice(&65536i32.to_le_bytes());
        data.extend_from_slice(&131072i32.to_le_bytes());
        data.extend_from_slice(&196608i32.to_le_bytes());
        // Vertex at index 6: (4.0, 5.0, 6.0)
        data.extend_from_slice(&262144i32.to_le_bytes());
        data.extend_from_slice(&327680i32.to_le_bytes());
        data.extend_from_slice(&393216i32.to_le_bytes());
        data.extend_from_slice(&[0x00, 0x00]); // OP_EOF

        let model = PofParser::parse(&data).unwrap();
        assert_eq!(model.vertices.len(), 7); // 0-6 (start=5, count=2)

        let v5 = model.vertices[5].to_f32();
        assert!((v5[0] - 1.0).abs() < 0.0001);
        assert!((v5[1] - 2.0).abs() < 0.0001);
        assert!((v5[2] - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_parse_invalid_opcode() {
        let data = create_test_data(&[0xFF, 0xFF]); // Invalid opcode
        assert!(PofParser::parse(&data).is_err());
    }
}

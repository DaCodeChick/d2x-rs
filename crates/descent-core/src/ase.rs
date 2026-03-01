//! ASE (ASCII Scene Export) model format parser.
//!
//! ASE is a text-based 3D model format exported from 3D Studio Max, used by D2X-XL
//! for high-resolution models. The format is human-readable and contains geometry,
//! materials, and animation data.
//!
//! # Format Overview
//!
//! ASE files are structured as a hierarchy of blocks:
//! ```text
//! *3DSMAX_ASCIIEXPORT 200
//! *SCENE { ... }
//! *MATERIAL_LIST { ... }
//! *GEOMOBJECT {
//!     *NODE_NAME "Object01"
//!     *MESH {
//!         *MESH_NUMVERTEX 8
//!         *MESH_NUMFACES 12
//!         *MESH_VERTEX_LIST { ... }
//!         *MESH_FACE_LIST { ... }
//!         *MESH_NORMALS { ... }
//!         *MESH_TVERTLIST { ... }  // Texture coordinates
//!     }
//! }
//! ```
//!
//! # Example
//!
//! ```no_run
//! use descent_core::ase::AseFile;
//!
//! let data = std::fs::read_to_string("pyro-gl.ase").unwrap();
//! let ase = AseFile::parse(&data).unwrap();
//!
//! println!("Model: {}", ase.scene.filename);
//! println!("Objects: {}", ase.geom_objects.len());
//! ```

use crate::error::{AssetError, Result};
use std::str::FromStr;

/// Type alias for vertex/normal vectors
type Vec3 = [f32; 3];

/// Type alias for parsing normals result
type NormalsResult = (Vec<Vec3>, Vec<Vec3>);

/// Root structure of an ASE file.
#[derive(Debug, Clone)]
pub struct AseFile {
    /// Scene metadata
    pub scene: AseScene,
    /// Material definitions
    pub materials: Vec<AseMaterial>,
    /// Geometry objects (meshes)
    pub geom_objects: Vec<AseGeomObject>,
}

/// Scene metadata
#[derive(Debug, Clone, Default)]
pub struct AseScene {
    /// Original filename
    pub filename: String,
    /// First frame
    pub first_frame: i32,
    /// Last frame
    pub last_frame: i32,
    /// Frame speed (FPS)
    pub frame_speed: i32,
    /// Ticks per frame
    pub ticks_per_frame: i32,
    /// Background color (static)
    pub background_static: [f32; 3],
    /// Ambient light color (static)
    pub ambient_static: [f32; 3],
}

/// Material definition
#[derive(Debug, Clone, Default)]
pub struct AseMaterial {
    /// Material name
    pub name: String,
    /// Ambient color
    pub ambient: [f32; 3],
    /// Diffuse color
    pub diffuse: [f32; 3],
    /// Specular color
    pub specular: [f32; 3],
    /// Shininess value
    pub shine: f32,
    /// Transparency (0.0 = opaque, 1.0 = transparent)
    pub transparency: f32,
    /// Diffuse texture map
    pub map_diffuse: Option<AseMap>,
}

/// Texture map
#[derive(Debug, Clone, Default)]
pub struct AseMap {
    /// Map name/type
    pub name: String,
    /// Texture class
    pub class: String,
    /// Bitmap filename
    pub bitmap: String,
    /// UV tiling
    pub u_tiling: f32,
    /// UV tiling
    pub v_tiling: f32,
}

/// Geometry object (mesh)
#[derive(Debug, Clone)]
pub struct AseGeomObject {
    /// Node name
    pub node_name: String,
    /// Material reference (index into materials list)
    pub material_ref: Option<usize>,
    /// Mesh data
    pub mesh: AseMesh,
    /// Transform matrix (row-major, 4x3)
    pub tm: AseTransform,
}

/// Transform matrix
#[derive(Debug, Clone)]
pub struct AseTransform {
    /// Translation
    pub pos: [f32; 3],
    /// Rotation (as 3x3 matrix rows)
    pub rot_row0: [f32; 3],
    pub rot_row1: [f32; 3],
    pub rot_row2: [f32; 3],
    /// Scale
    pub scale: [f32; 3],
}

impl Default for AseTransform {
    fn default() -> Self {
        Self {
            pos: [0.0, 0.0, 0.0],
            rot_row0: [1.0, 0.0, 0.0],
            rot_row1: [0.0, 1.0, 0.0],
            rot_row2: [0.0, 0.0, 1.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

/// Mesh geometry data
#[derive(Debug, Clone, Default)]
pub struct AseMesh {
    /// Vertex positions
    pub vertices: Vec<[f32; 3]>,
    /// Face indices (triangles)
    pub faces: Vec<AseFace>,
    /// Texture vertices (UV coordinates)
    pub tvertices: Vec<[f32; 2]>,
    /// Vertex normals
    pub normals: Vec<[f32; 3]>,
    /// Face normals
    pub face_normals: Vec<[f32; 3]>,
}

/// Face definition
#[derive(Debug, Clone)]
pub struct AseFace {
    /// Vertex indices (A, B, C)
    pub vertices: [usize; 3],
    /// Texture coordinate indices (A, B, C)
    pub tvertices: [usize; 3],
    /// Material ID
    pub material_id: usize,
    /// Smoothing group
    pub smoothing: u32,
}

impl AseFile {
    /// Parse an ASE file from string data.
    ///
    /// # Arguments
    ///
    /// * `data` - ASE file content as text
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File format is invalid
    /// - Required blocks are missing
    /// - Numeric values cannot be parsed
    pub fn parse(data: &str) -> Result<Self> {
        let mut parser = AseParser::new(data);
        parser.parse()
    }
}

/// ASE parser state
struct AseParser<'a> {
    lines: std::iter::Peekable<std::str::Lines<'a>>,
    current_line: usize,
}

impl<'a> AseParser<'a> {
    fn new(data: &'a str) -> Self {
        Self {
            lines: data.lines().peekable(),
            current_line: 0,
        }
    }

    fn parse(&mut self) -> Result<AseFile> {
        let mut scene = AseScene::default();
        let mut materials = Vec::new();
        let mut geom_objects = Vec::new();

        // Parse top-level blocks
        while let Some(line) = self.next_line() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if line.starts_with("*3DSMAX_ASCIIEXPORT") {
                // Version header
                continue;
            } else if line.starts_with("*SCENE") {
                scene = self.parse_scene()?;
            } else if line.starts_with("*MATERIAL_LIST") {
                materials = self.parse_material_list()?;
            } else if line.starts_with("*GEOMOBJECT") {
                geom_objects.push(self.parse_geom_object()?);
            }
        }

        Ok(AseFile {
            scene,
            materials,
            geom_objects,
        })
    }

    fn parse_scene(&mut self) -> Result<AseScene> {
        // Opening brace might be on same line or next line
        self.skip_to_opening_brace()?;
        let mut scene = AseScene::default();

        while let Some(line) = self.next_line() {
            let line = line.trim();
            if line == "}" {
                break;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "*SCENE_FILENAME" => {
                    scene.filename = self.parse_string_value(line)?;
                }
                "*SCENE_FIRSTFRAME" => {
                    scene.first_frame = self.parse_int_value(line)?;
                }
                "*SCENE_LASTFRAME" => {
                    scene.last_frame = self.parse_int_value(line)?;
                }
                "*SCENE_FRAMESPEED" => {
                    scene.frame_speed = self.parse_int_value(line)?;
                }
                "*SCENE_TICKSPERFRAME" => {
                    scene.ticks_per_frame = self.parse_int_value(line)?;
                }
                "*SCENE_BACKGROUND_STATIC" => {
                    scene.background_static = self.parse_color3(line)?;
                }
                "*SCENE_AMBIENT_STATIC" => {
                    scene.ambient_static = self.parse_color3(line)?;
                }
                _ => {}
            }
        }

        Ok(scene)
    }

    fn parse_material_list(&mut self) -> Result<Vec<AseMaterial>> {
        self.skip_to_opening_brace()?;
        let mut materials = Vec::new();

        while let Some(line) = self.next_line() {
            let line = line.trim();
            if line == "}" {
                break;
            }

            if line.starts_with("*MATERIAL_COUNT") {
                // Optional: could pre-allocate
                continue;
            } else if line.starts_with("*MATERIAL") {
                materials.push(self.parse_material()?);
            }
        }

        Ok(materials)
    }

    fn parse_material(&mut self) -> Result<AseMaterial> {
        self.skip_to_opening_brace()?;
        let mut material = AseMaterial::default();

        while let Some(line) = self.next_line() {
            let line = line.trim();
            if line == "}" {
                break;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "*MATERIAL_NAME" => {
                    material.name = self.parse_string_value(line)?;
                }
                "*MATERIAL_AMBIENT" => {
                    material.ambient = self.parse_color3(line)?;
                }
                "*MATERIAL_DIFFUSE" => {
                    material.diffuse = self.parse_color3(line)?;
                }
                "*MATERIAL_SPECULAR" => {
                    material.specular = self.parse_color3(line)?;
                }
                "*MATERIAL_SHINE" => {
                    material.shine = self.parse_float_value(line)?;
                }
                "*MATERIAL_TRANSPARENCY" => {
                    material.transparency = self.parse_float_value(line)?;
                }
                "*MAP_DIFFUSE" => {
                    material.map_diffuse = Some(self.parse_map()?);
                }
                _ => {}
            }
        }

        Ok(material)
    }

    fn parse_map(&mut self) -> Result<AseMap> {
        self.skip_to_opening_brace()?;
        let mut map = AseMap {
            u_tiling: 1.0,
            v_tiling: 1.0,
            ..Default::default()
        };

        while let Some(line) = self.next_line() {
            let line = line.trim();
            if line == "}" {
                break;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "*MAP_NAME" => {
                    map.name = self.parse_string_value(line)?;
                }
                "*MAP_CLASS" => {
                    map.class = self.parse_string_value(line)?;
                }
                "*BITMAP" => {
                    map.bitmap = self.parse_string_value(line)?;
                }
                "*UVW_U_TILING" => {
                    map.u_tiling = self.parse_float_value(line)?;
                }
                "*UVW_V_TILING" => {
                    map.v_tiling = self.parse_float_value(line)?;
                }
                _ => {}
            }
        }

        Ok(map)
    }

    fn parse_geom_object(&mut self) -> Result<AseGeomObject> {
        self.skip_to_opening_brace()?;
        let mut node_name = String::new();
        let mut material_ref = None;
        let mut mesh = AseMesh::default();
        let mut tm = AseTransform::default();

        while let Some(line) = self.next_line() {
            let line = line.trim();
            if line == "}" {
                break;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "*NODE_NAME" => {
                    node_name = self.parse_string_value(line)?;
                }
                "*MATERIAL_REF" => {
                    material_ref = Some(self.parse_int_value::<usize>(line)?);
                }
                "*MESH" => {
                    mesh = self.parse_mesh()?;
                }
                "*TM_POS" => {
                    tm.pos = self.parse_vector3(line)?;
                }
                "*TM_ROW0" => {
                    tm.rot_row0 = self.parse_vector3(line)?;
                }
                "*TM_ROW1" => {
                    tm.rot_row1 = self.parse_vector3(line)?;
                }
                "*TM_ROW2" => {
                    tm.rot_row2 = self.parse_vector3(line)?;
                }
                "*TM_SCALE" => {
                    tm.scale = self.parse_vector3(line)?;
                }
                _ => {}
            }
        }

        Ok(AseGeomObject {
            node_name,
            material_ref,
            mesh,
            tm,
        })
    }

    fn parse_mesh(&mut self) -> Result<AseMesh> {
        self.skip_to_opening_brace()?;
        let mut mesh = AseMesh::default();

        while let Some(line) = self.next_line() {
            let line = line.trim();
            if line == "}" {
                break;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            match parts[0] {
                "*MESH_NUMVERTEX" => {
                    let count = self.parse_int_value::<usize>(line)?;
                    mesh.vertices.reserve(count);
                }
                "*MESH_NUMFACES" => {
                    let count = self.parse_int_value::<usize>(line)?;
                    mesh.faces.reserve(count);
                }
                "*MESH_VERTEX_LIST" => {
                    mesh.vertices = self.parse_vertex_list()?;
                }
                "*MESH_FACE_LIST" => {
                    mesh.faces = self.parse_face_list()?;
                }
                "*MESH_NUMTVERTEX" => {
                    let count = self.parse_int_value::<usize>(line)?;
                    mesh.tvertices.reserve(count);
                }
                "*MESH_TVERTLIST" => {
                    mesh.tvertices = self.parse_tvert_list()?;
                }
                "*MESH_NORMALS" => {
                    let (normals, face_normals) = self.parse_normals()?;
                    mesh.normals = normals;
                    mesh.face_normals = face_normals;
                }
                _ => {}
            }
        }

        Ok(mesh)
    }

    fn parse_vertex_list(&mut self) -> Result<Vec<[f32; 3]>> {
        self.skip_to_opening_brace()?;
        let mut vertices = Vec::new();

        while let Some(line) = self.next_line() {
            let line = line.trim();
            if line == "}" {
                break;
            }

            if line.starts_with("*MESH_VERTEX") {
                vertices.push(self.parse_vertex(line)?);
            }
        }

        Ok(vertices)
    }

    fn parse_face_list(&mut self) -> Result<Vec<AseFace>> {
        self.skip_to_opening_brace()?;
        let mut faces = Vec::new();

        while let Some(line) = self.next_line() {
            let line = line.trim();
            if line == "}" {
                break;
            }

            if line.starts_with("*MESH_FACE") {
                faces.push(self.parse_face(line)?);
            }
        }

        Ok(faces)
    }

    fn parse_tvert_list(&mut self) -> Result<Vec<[f32; 2]>> {
        self.skip_to_opening_brace()?;
        let mut tvertices = Vec::new();

        while let Some(line) = self.next_line() {
            let line = line.trim();
            if line == "}" {
                break;
            }

            if line.starts_with("*MESH_TVERT") {
                tvertices.push(self.parse_tvert(line)?);
            }
        }

        Ok(tvertices)
    }

    fn parse_normals(&mut self) -> Result<NormalsResult> {
        self.skip_to_opening_brace()?;
        let mut normals = Vec::new();
        let mut face_normals = Vec::new();

        while let Some(line) = self.next_line() {
            let line = line.trim();
            if line == "}" {
                break;
            }

            if line.starts_with("*MESH_FACENORMAL") {
                face_normals.push(self.parse_face_normal(line)?);
            } else if line.starts_with("*MESH_VERTEXNORMAL") {
                normals.push(self.parse_vertex_normal(line)?);
            }
        }

        Ok((normals, face_normals))
    }

    // Parsing helper methods

    fn parse_vertex(&self, line: &str) -> Result<[f32; 3]> {
        // Format: *MESH_VERTEX <index> <x> <y> <z>
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return Err(AssetError::ParseError(format!(
                "Invalid vertex line: {}",
                line
            )));
        }

        Ok([
            Self::parse_f32(parts[2])?,
            Self::parse_f32(parts[3])?,
            Self::parse_f32(parts[4])?,
        ])
    }

    fn parse_face(&self, line: &str) -> Result<AseFace> {
        // Format: *MESH_FACE <index>: A: <a> B: <b> C: <c> ... *MESH_MTLID <id>
        let parts: Vec<&str> = line.split_whitespace().collect();

        let mut a = 0;
        let mut b = 0;
        let mut c = 0;
        let mut material_id = 0;
        let mut smoothing = 0;

        for i in 0..parts.len() {
            match parts[i] {
                "A:" if i + 1 < parts.len() => {
                    a = Self::parse_usize(parts[i + 1])?;
                }
                "B:" if i + 1 < parts.len() => {
                    b = Self::parse_usize(parts[i + 1])?;
                }
                "C:" if i + 1 < parts.len() => {
                    c = Self::parse_usize(parts[i + 1])?;
                }
                "*MESH_MTLID" if i + 1 < parts.len() => {
                    material_id = Self::parse_usize(parts[i + 1])?;
                }
                "*MESH_SMOOTHING" if i + 1 < parts.len() => {
                    smoothing = Self::parse_u32(parts[i + 1])?;
                }
                _ => {}
            }
        }

        Ok(AseFace {
            vertices: [a, b, c],
            tvertices: [0, 0, 0], // Will be filled from TFACELIST
            material_id,
            smoothing,
        })
    }

    fn parse_tvert(&self, line: &str) -> Result<[f32; 2]> {
        // Format: *MESH_TVERT <index> <u> <v> <w>
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return Err(AssetError::ParseError(format!(
                "Invalid tvert line: {}",
                line
            )));
        }

        Ok([Self::parse_f32(parts[2])?, Self::parse_f32(parts[3])?])
    }

    fn parse_face_normal(&self, line: &str) -> Result<[f32; 3]> {
        // Format: *MESH_FACENORMAL <index> <x> <y> <z>
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return Err(AssetError::ParseError(format!(
                "Invalid face normal line: {}",
                line
            )));
        }

        Ok([
            Self::parse_f32(parts[2])?,
            Self::parse_f32(parts[3])?,
            Self::parse_f32(parts[4])?,
        ])
    }

    fn parse_vertex_normal(&self, line: &str) -> Result<[f32; 3]> {
        // Format: *MESH_VERTEXNORMAL <index> <x> <y> <z>
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 5 {
            return Err(AssetError::ParseError(format!(
                "Invalid vertex normal line: {}",
                line
            )));
        }

        Ok([
            Self::parse_f32(parts[2])?,
            Self::parse_f32(parts[3])?,
            Self::parse_f32(parts[4])?,
        ])
    }

    fn parse_string_value(&self, line: &str) -> Result<String> {
        // Extract quoted string
        let start = line
            .find('"')
            .ok_or_else(|| AssetError::ParseError(format!("No opening quote in: {}", line)))?;
        let end = line
            .rfind('"')
            .ok_or_else(|| AssetError::ParseError(format!("No closing quote in: {}", line)))?;

        if start >= end {
            return Err(AssetError::ParseError(format!(
                "Invalid quoted string: {}",
                line
            )));
        }

        Ok(line[start + 1..end].to_string())
    }

    fn parse_int_value<T: FromStr>(&self, line: &str) -> Result<T> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(AssetError::ParseError(format!(
                "No value in line: {}",
                line
            )));
        }

        parts[1]
            .parse()
            .map_err(|_| AssetError::ParseError(format!("Invalid integer in: {}", line)))
    }

    fn parse_float_value(&self, line: &str) -> Result<f32> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(AssetError::ParseError(format!(
                "No value in line: {}",
                line
            )));
        }

        Self::parse_f32(parts[1])
    }

    fn parse_color3(&self, line: &str) -> Result<[f32; 3]> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return Err(AssetError::ParseError(format!(
                "Invalid color3 line: {}",
                line
            )));
        }

        Ok([
            Self::parse_f32(parts[1])?,
            Self::parse_f32(parts[2])?,
            Self::parse_f32(parts[3])?,
        ])
    }

    fn parse_vector3(&self, line: &str) -> Result<[f32; 3]> {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return Err(AssetError::ParseError(format!(
                "Invalid vector3 line: {}",
                line
            )));
        }

        Ok([
            Self::parse_f32(parts[1])?,
            Self::parse_f32(parts[2])?,
            Self::parse_f32(parts[3])?,
        ])
    }

    // Low-level parsers

    fn parse_f32(s: &str) -> Result<f32> {
        s.parse()
            .map_err(|_| AssetError::ParseError(format!("Invalid float: {}", s)))
    }

    fn parse_usize(s: &str) -> Result<usize> {
        s.parse()
            .map_err(|_| AssetError::ParseError(format!("Invalid usize: {}", s)))
    }

    fn parse_u32(s: &str) -> Result<u32> {
        s.parse()
            .map_err(|_| AssetError::ParseError(format!("Invalid u32: {}", s)))
    }

    fn next_line(&mut self) -> Option<&'a str> {
        self.current_line += 1;
        self.lines.next()
    }

    #[allow(dead_code)]
    fn expect_line(&mut self, expected: &str) -> Result<()> {
        // Check if the previous line already contained the expected token (inline)
        // This is handled by checking if we should skip this call
        // Note: Currently unused but kept for potential future validation needs

        let line = self.next_line().ok_or_else(|| {
            AssetError::ParseError(format!(
                "Expected '{}' but reached EOF at line {}",
                expected, self.current_line
            ))
        })?;

        let line = line.trim();

        // Check if this line starts with the expected string and optionally contains it
        if line.contains(expected) || line == expected {
            Ok(())
        } else {
            Err(AssetError::ParseError(format!(
                "Expected '{}' but got '{}' at line {}",
                expected, line, self.current_line
            )))
        }
    }

    fn skip_to_opening_brace(&mut self) -> Result<()> {
        // Look for opening brace, might be on current or next line
        while let Some(line) = self.lines.peek() {
            let line = line.trim();
            if line.contains('{') {
                // Found it, consume the line if it's only a brace
                if line == "{" {
                    self.next_line();
                }
                return Ok(());
            } else if !line.is_empty() && !line.starts_with("//") {
                // Non-brace content, brace might be on same line as tag
                return Ok(());
            }
            self.next_line();
        }
        Err(AssetError::ParseError(format!(
            "Expected opening brace but reached EOF at line {}",
            self.current_line
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_ase() {
        let ase_data = r#"
*3DSMAX_ASCIIEXPORT 200
*SCENE {
    *SCENE_FILENAME "test.max"
    *SCENE_FIRSTFRAME 0
    *SCENE_LASTFRAME 100
}
*MATERIAL_LIST {
    *MATERIAL_COUNT 1
    *MATERIAL 0 {
        *MATERIAL_NAME "Material01"
        *MATERIAL_DIFFUSE 1.0 0.5 0.25
    }
}
*GEOMOBJECT {
    *NODE_NAME "Box01"
    *MESH {
        *MESH_NUMVERTEX 3
        *MESH_NUMFACES 1
        *MESH_VERTEX_LIST {
            *MESH_VERTEX 0 0.0 0.0 0.0
            *MESH_VERTEX 1 1.0 0.0 0.0
            *MESH_VERTEX 2 0.0 1.0 0.0
        }
        *MESH_FACE_LIST {
            *MESH_FACE 0: A: 0 B: 1 C: 2 *MESH_MTLID 0
        }
    }
}
"#;

        let result = AseFile::parse(ase_data);
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());

        let ase = result.unwrap();
        assert_eq!(ase.scene.filename, "test.max");
        assert_eq!(ase.materials.len(), 1);
        assert_eq!(ase.geom_objects.len(), 1);
        assert_eq!(ase.geom_objects[0].mesh.vertices.len(), 3);
        assert_eq!(ase.geom_objects[0].mesh.faces.len(), 1);
    }
}

//! Model format converters for POF (Polygon Object File) format.
//!
//! This module provides converters for Descent's 3D model format:
//! - **POF**: Descent 1/2 polygon-based models (ships, robots, powerups)
//!
//! POF models can be converted to glTF 2.0 / GLB format for use in modern engines.
//!
//! # Examples
//!
//! ## Converting POF Model to GLB
//!
//! ```no_run
//! use descent_core::pof::PofParser;
//! use descent_core::converters::model::ModelConverter;
//! use std::fs;
//!
//! let pof_data = fs::read("pyrogl.pof").unwrap();
//! let model = PofParser::parse(&pof_data).unwrap();
//!
//! let converter = ModelConverter::new();
//! let glb = converter.pof_to_glb(&model, "Pyro-GL Ship").unwrap();
//! fs::write("pyrogl.glb", glb).unwrap();
//! ```

use crate::error::{AssetError, Result};
use crate::pof::{PofModel, Polygon};
use gltf_json as json;
use gltf_json::validation::Checked::Valid;
use gltf_json::validation::USize64;
use std::io::Write;

/// Converter for POF models to glTF/GLB format.
pub struct ModelConverter {
    /// Generator name to embed in glTF metadata
    generator: String,
}

impl ModelConverter {
    /// Create a new model converter with default settings.
    pub fn new() -> Self {
        Self {
            generator: format!("descent-core v{}", env!("CARGO_PKG_VERSION")),
        }
    }

    /// Convert a POF model to GLB (binary glTF 2.0) format.
    ///
    /// # Arguments
    ///
    /// * `model` - The parsed POF model
    /// * `name` - Name for the model (e.g., "Pyro-GL Ship")
    ///
    /// # Returns
    ///
    /// A Vec<u8> containing the complete GLB file
    pub fn pof_to_glb(&self, model: &PofModel, name: &str) -> Result<Vec<u8>> {
        // Convert POF geometry to glTF data
        let (positions, indices, normals) = self.extract_geometry(model)?;

        // Build binary buffer (vertex + index data)
        let bin_buffer = self.build_binary_buffer(&positions, &indices, &normals)?;

        // Build glTF JSON structure
        let root = self.build_gltf_json(name, &positions, &indices, &normals, bin_buffer.len())?;

        // Serialize to JSON
        let json_string = serde_json::to_string(&root).map_err(|e| {
            crate::error::AssetError::InvalidFormat(format!("Failed to serialize glTF JSON: {}", e))
        })?;
        let mut json_bytes = json_string.into_bytes();

        // Pad JSON to 4-byte alignment with spaces (0x20)
        let json_padding = (4 - (json_bytes.len() % 4)) % 4;
        json_bytes.resize(json_bytes.len() + json_padding, 0x20);

        // Pad binary buffer to 4-byte alignment with zeros (0x00)
        let mut bin_buffer = bin_buffer;
        let bin_padding = (4 - (bin_buffer.len() % 4)) % 4;
        bin_buffer.resize(bin_buffer.len() + bin_padding, 0x00);

        // Build GLB file
        let total_length = 12 + 8 + json_bytes.len() + 8 + bin_buffer.len();
        let mut glb = Vec::with_capacity(total_length);

        // Write GLB header (12 bytes)
        glb.write_all(&0x46546C67u32.to_le_bytes())?; // magic: "glTF"
        glb.write_all(&2u32.to_le_bytes())?; // version: 2
        glb.write_all(&(total_length as u32).to_le_bytes())?; // length

        // Write JSON chunk (8 + json_bytes.len())
        glb.write_all(&(json_bytes.len() as u32).to_le_bytes())?; // chunkLength
        glb.write_all(&0x4E4F534Au32.to_le_bytes())?; // chunkType: "JSON"
        glb.write_all(&json_bytes)?; // chunkData

        // Write BIN chunk (8 + bin_buffer.len())
        glb.write_all(&(bin_buffer.len() as u32).to_le_bytes())?; // chunkLength
        glb.write_all(&0x004E4942u32.to_le_bytes())?; // chunkType: "BIN\0"
        glb.write_all(&bin_buffer)?; // chunkData

        Ok(glb)
    }

    /// Extract geometry data from POF model.
    ///
    /// Returns (positions, indices, normals) as flat f32 arrays.
    fn extract_geometry(&self, model: &PofModel) -> Result<(Vec<f32>, Vec<u32>, Vec<f32>)> {
        if model.vertices.is_empty() || model.polygons.is_empty() {
            return Err(AssetError::InvalidFormat(
                "Model has no geometry".to_string(),
            ));
        }

        let mut positions = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();

        // Convert vertices from fixed-point to f32
        for vertex in &model.vertices {
            let pos = vertex.to_f32();
            positions.push(pos[0]);
            positions.push(pos[1]);
            positions.push(pos[2]);
        }

        // Extract polygon data and build indices
        for polygon in &model.polygons {
            match polygon {
                Polygon::Flat(poly) => {
                    let normal = poly.normal.to_f32();

                    // Triangulate polygon (simple fan triangulation)
                    for i in 1..poly.vertices.len() - 1 {
                        let idx0 = poly.vertices[0] as u32;
                        let idx1 = poly.vertices[i] as u32;
                        let idx2 = poly.vertices[i + 1] as u32;

                        indices.push(idx0);
                        indices.push(idx1);
                        indices.push(idx2);

                        // Add normals for each vertex in the triangle
                        for _ in 0..3 {
                            normals.push(normal[0]);
                            normals.push(normal[1]);
                            normals.push(normal[2]);
                        }
                    }
                }
                Polygon::Textured(poly) => {
                    let normal = poly.normal.to_f32();

                    // Triangulate polygon (simple fan triangulation)
                    for i in 1..poly.vertices.len() - 1 {
                        let idx0 = poly.vertices[0] as u32;
                        let idx1 = poly.vertices[i] as u32;
                        let idx2 = poly.vertices[i + 1] as u32;

                        indices.push(idx0);
                        indices.push(idx1);
                        indices.push(idx2);

                        // Add normals for each vertex in the triangle
                        for _ in 0..3 {
                            normals.push(normal[0]);
                            normals.push(normal[1]);
                            normals.push(normal[2]);
                        }
                    }
                }
            }
        }

        Ok((positions, indices, normals))
    }

    /// Build binary buffer containing vertex and index data.
    fn build_binary_buffer(
        &self,
        positions: &[f32],
        indices: &[u32],
        normals: &[f32],
    ) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Write positions (Vec3<f32>)
        for &val in positions {
            buffer.write_all(&val.to_le_bytes())?;
        }

        // Write normals (Vec3<f32>)
        for &val in normals {
            buffer.write_all(&val.to_le_bytes())?;
        }

        // Write indices (u32)
        for &idx in indices {
            buffer.write_all(&idx.to_le_bytes())?;
        }

        if buffer.len() > u32::MAX as usize {
            return Err(AssetError::InvalidFormat(
                "Binary buffer too large (max 4GB)".to_string(),
            ));
        }

        Ok(buffer)
    }

    /// Build the glTF JSON structure.
    fn build_gltf_json(
        &self,
        _name: &str,
        positions: &[f32],
        indices: &[u32],
        normals: &[f32],
        buffer_length: usize,
    ) -> Result<json::Root> {
        let positions_byte_length = std::mem::size_of_val(positions);
        let normals_byte_length = std::mem::size_of_val(normals);
        let indices_byte_offset = positions_byte_length + normals_byte_length;
        let indices_byte_length = std::mem::size_of_val(indices);

        // Compute bounding box for positions
        let (min_pos, max_pos) = self.compute_bounds(positions);

        // Build glTF structure
        let buffer = json::Buffer {
            byte_length: USize64::from(buffer_length),
            uri: None,
            extensions: None,
            extras: Default::default(),
        };

        let buffer_views = vec![
            // BufferView 0: Positions
            json::buffer::View {
                buffer: json::Index::new(0),
                byte_length: USize64::from(positions_byte_length),
                byte_offset: Some(USize64::from(0usize)),
                byte_stride: None,
                target: Some(Valid(json::buffer::Target::ArrayBuffer)),
                extensions: None,
                extras: Default::default(),
            },
            // BufferView 1: Normals
            json::buffer::View {
                buffer: json::Index::new(0),
                byte_length: USize64::from(normals_byte_length),
                byte_offset: Some(USize64::from(positions_byte_length)),
                byte_stride: None,
                target: Some(Valid(json::buffer::Target::ArrayBuffer)),
                extensions: None,
                extras: Default::default(),
            },
            // BufferView 2: Indices
            json::buffer::View {
                buffer: json::Index::new(0),
                byte_length: USize64::from(indices_byte_length),
                byte_offset: Some(USize64::from(indices_byte_offset)),
                byte_stride: None,
                target: Some(Valid(json::buffer::Target::ElementArrayBuffer)),
                extensions: None,
                extras: Default::default(),
            },
        ];

        let accessors = vec![
            // Accessor 0: Positions
            json::Accessor {
                buffer_view: Some(json::Index::new(0)),
                byte_offset: Some(USize64::from(0usize)),
                count: USize64::from(positions.len() / 3),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::F32,
                )),
                type_: Valid(json::accessor::Type::Vec3),
                min: Some(json::Value::from(min_pos)),
                max: Some(json::Value::from(max_pos)),
                normalized: false,
                sparse: None,
                extensions: None,
                extras: Default::default(),
            },
            // Accessor 1: Normals
            json::Accessor {
                buffer_view: Some(json::Index::new(1)),
                byte_offset: Some(USize64::from(0usize)),
                count: USize64::from(normals.len() / 3),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::F32,
                )),
                type_: Valid(json::accessor::Type::Vec3),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
                extensions: None,
                extras: Default::default(),
            },
            // Accessor 2: Indices
            json::Accessor {
                buffer_view: Some(json::Index::new(2)),
                byte_offset: Some(USize64::from(0usize)),
                count: USize64::from(indices.len()),
                component_type: Valid(json::accessor::GenericComponentType(
                    json::accessor::ComponentType::U32,
                )),
                type_: Valid(json::accessor::Type::Scalar),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
                extensions: None,
                extras: Default::default(),
            },
        ];

        let primitives = vec![json::mesh::Primitive {
            attributes: {
                let mut map = std::collections::BTreeMap::new();
                map.insert(Valid(json::mesh::Semantic::Positions), json::Index::new(0));
                map.insert(Valid(json::mesh::Semantic::Normals), json::Index::new(1));
                map
            },
            indices: Some(json::Index::new(2)),
            material: None,
            mode: Valid(json::mesh::Mode::Triangles),
            targets: None,
            extensions: None,
            extras: Default::default(),
        }];

        let meshes = vec![json::Mesh {
            primitives,
            weights: None,
            extensions: None,
            extras: Default::default(),
        }];

        let nodes = vec![json::scene::Node {
            mesh: Some(json::Index::new(0)),
            camera: None,
            children: None,
            extensions: None,
            extras: Default::default(),
            matrix: None,
            rotation: None,
            scale: None,
            translation: None,
            skin: None,
            weights: None,
        }];

        let scenes = vec![json::Scene {
            nodes: vec![json::Index::new(0)],
            extensions: None,
            extras: Default::default(),
        }];

        let root = json::Root {
            accessors,
            buffers: vec![buffer],
            buffer_views,
            meshes,
            nodes,
            scenes,
            scene: Some(json::Index::new(0)),
            asset: json::Asset {
                version: "2.0".to_string(),
                generator: Some(self.generator.clone()),
                copyright: None,
                min_version: None,
                extensions: None,
                extras: Default::default(),
            },
            animations: vec![],
            cameras: vec![],
            images: vec![],
            materials: vec![],
            samplers: vec![],
            skins: vec![],
            textures: vec![],
            extensions: None,
            extensions_used: vec![],
            extensions_required: vec![],
            extras: Default::default(),
        };

        Ok(root)
    }

    /// Compute min/max bounds for position data.
    fn compute_bounds(&self, positions: &[f32]) -> (Vec<f32>, Vec<f32>) {
        let mut min = vec![f32::MAX, f32::MAX, f32::MAX];
        let mut max = vec![f32::MIN, f32::MIN, f32::MIN];

        for chunk in positions.chunks(3) {
            for i in 0..3 {
                if chunk[i] < min[i] {
                    min[i] = chunk[i];
                }
                if chunk[i] > max[i] {
                    max[i] = chunk[i];
                }
            }
        }

        (min, max)
    }
}

impl Default for ModelConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_converter_new() {
        let converter = ModelConverter::new();
        assert!(converter.generator.contains("descent-core"));
    }

    #[test]
    fn test_compute_bounds() {
        let converter = ModelConverter::new();
        let positions = vec![
            0.0, 0.0, 0.0, // vertex 0
            1.0, 2.0, 3.0, // vertex 1
            -1.0, -2.0, -3.0, // vertex 2
        ];

        let (min, max) = converter.compute_bounds(&positions);
        assert_eq!(min, vec![-1.0, -2.0, -3.0]);
        assert_eq!(max, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_glb_header_magic() {
        // Verify the magic number is correct
        assert_eq!(0x46546C67u32.to_le_bytes(), [0x67, 0x6C, 0x54, 0x46]); // "glTF" in ASCII
    }
}

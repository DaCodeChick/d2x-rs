//! Model format converters for POF (Polygon Object File) format.
//!
//! This module provides converters for Descent's 3D model format:
//! - **POF**: Descent 1/2 polygon-based models (ships, robots, powerups)
//!
//! POF models can be converted to glTF 2.0 / GLB format for use in modern engines.
//!
//! # Examples
//!
//! ## Converting POF Model to GLB (Geometry Only)
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
//! let glb = converter.pof_to_glb(&model, "Pyro-GL Ship", None).unwrap();
//! fs::write("pyrogl.glb", glb).unwrap();
//! ```
//!
//! ## Converting POF Model to GLB with Textures
//!
//! ```no_run
//! use descent_core::pof::PofParser;
//! use descent_core::pig::PigFile;
//! use descent_core::palette::Palette;
//! use descent_core::converters::model::{ModelConverter, TextureProvider};
//! use std::fs;
//!
//! let pof_data = fs::read("pyrogl.pof").unwrap();
//! let model = PofParser::parse(&pof_data).unwrap();
//!
//! let pig_data = fs::read("descent2.pig").unwrap();
//! let pig = PigFile::parse(pig_data, false).unwrap();
//!
//! let palette_data = fs::read("groupa.256").unwrap();
//! let palette = Palette::parse(&palette_data).unwrap();
//!
//! let provider = TextureProvider::new(pig, palette);
//! let converter = ModelConverter::new();
//! let glb = converter.pof_to_glb(&model, "Pyro-GL Ship", Some(&provider)).unwrap();
//! fs::write("pyrogl.glb", glb).unwrap();
//! ```

use crate::error::{AssetError, Result};
use crate::palette::Palette;
use crate::pig::PigFile;
use crate::pof::{PofModel, Polygon};
use gltf_json as json;
use gltf_json::validation::Checked::Valid;
use gltf_json::validation::USize64;
use std::collections::HashMap;
use std::io::Write;

/// Provides texture data for model conversion.
///
/// This struct holds a PIG file and palette for converting
/// indexed textures to modern formats (PNG) during GLB export.
pub struct TextureProvider {
    pig: PigFile,
    palette: Palette,
}

impl TextureProvider {
    /// Create a new texture provider.
    pub fn new(pig: PigFile, palette: Palette) -> Self {
        Self { pig, palette }
    }

    /// Get the PIG file reference.
    pub fn pig(&self) -> &PigFile {
        &self.pig
    }

    /// Get the palette reference.
    pub fn palette(&self) -> &Palette {
        &self.palette
    }
}

/// Material type for grouping polygons.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum MaterialKey {
    /// Flat-shaded material with palette color index.
    Flat(u16),
    /// Textured material with texture ID.
    Textured(u16),
}

/// Geometry data for a single mesh primitive (one material).
#[derive(Debug, Clone)]
struct MeshPrimitive {
    /// Material key for this primitive (currently unused, for future material support).
    #[allow(dead_code)]
    material_key: MaterialKey,
    /// Triangle indices (3 per triangle).
    indices: Vec<u32>,
    /// Normals per triangle vertex (3 components per normal).
    normals: Vec<f32>,
    /// UV coordinates per triangle vertex (2 components per UV), if textured.
    uvs: Option<Vec<f32>>,
}

/// Complete geometry data for the model.
#[derive(Debug, Clone)]
struct GeometryData {
    /// All vertex positions in the model (3 components per vertex).
    positions: Vec<f32>,
    /// Mesh primitives grouped by material.
    primitives: Vec<MeshPrimitive>,
}

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
    /// * `texture_provider` - Optional texture provider for embedding textures
    ///
    /// # Returns
    ///
    /// A Vec<u8> containing the complete GLB file
    ///
    /// # Examples
    ///
    /// Geometry only (no textures):
    /// ```no_run
    /// # use descent_core::pof::PofParser;
    /// # use descent_core::converters::model::ModelConverter;
    /// # let pof_data = vec![];
    /// # let model = PofParser::parse(&pof_data).unwrap();
    /// let converter = ModelConverter::new();
    /// let glb = converter.pof_to_glb(&model, "Ship", None).unwrap();
    /// ```
    ///
    /// With textures:
    /// ```no_run
    /// # use descent_core::pof::PofParser;
    /// # use descent_core::pig::PigFile;
    /// # use descent_core::palette::Palette;
    /// # use descent_core::converters::model::{ModelConverter, TextureProvider};
    /// # let pof_data = vec![];
    /// # let model = PofParser::parse(&pof_data).unwrap();
    /// # let pig = PigFile::parse(vec![], false).unwrap();
    /// # let palette = Palette::parse(&[]).unwrap();
    /// let provider = TextureProvider::new(pig, palette);
    /// let converter = ModelConverter::new();
    /// let glb = converter.pof_to_glb(&model, "Ship", Some(&provider)).unwrap();
    /// ```
    pub fn pof_to_glb(
        &self,
        model: &PofModel,
        name: &str,
        texture_provider: Option<&TextureProvider>,
    ) -> Result<Vec<u8>> {
        // TODO: Extract texture images if provider is available
        // Currently disabled until we understand POF texture ID mapping
        let _texture_images: HashMap<u16, String> = HashMap::new();

        // Convert POF geometry to glTF data (grouped by material)
        let geometry = self.extract_geometry(model, texture_provider.is_some())?;

        // Build binary buffer (vertex + index data)
        let bin_buffer = self.build_binary_buffer(&geometry)?;

        // Build glTF JSON structure
        let root = self.build_gltf_json(name, &geometry, bin_buffer.len())?;

        // Serialize to JSON
        let json_string = serde_json::to_string(&root).map_err(|e| {
            AssetError::InvalidFormat(format!("Failed to serialize glTF JSON: {}", e))
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

    /// Extract geometry data from POF model, grouped by material.
    ///
    /// Returns complete geometry with positions and material-grouped primitives.
    fn extract_geometry(&self, model: &PofModel, with_textures: bool) -> Result<GeometryData> {
        if model.vertices.is_empty() || model.polygons.is_empty() {
            return Err(AssetError::InvalidFormat(
                "Model has no geometry".to_string(),
            ));
        }

        // Convert all vertices from fixed-point to f32
        let mut positions = Vec::with_capacity(model.vertices.len() * 3);
        for vertex in &model.vertices {
            let pos = vertex.to_f32();
            positions.push(pos[0]);
            positions.push(pos[1]);
            positions.push(pos[2]);
        }

        // Group polygons by material
        let mut material_groups: HashMap<MaterialKey, Vec<&Polygon>> = HashMap::new();
        for polygon in &model.polygons {
            let key = match polygon {
                Polygon::Flat(poly) => MaterialKey::Flat(poly.color),
                Polygon::Textured(poly) => MaterialKey::Textured(poly.texture_id),
            };
            material_groups.entry(key).or_default().push(polygon);
        }

        // Convert each material group to a mesh primitive
        let mut primitives = Vec::new();
        for (material_key, polygons) in material_groups {
            let mut indices = Vec::new();
            let mut normals = Vec::new();
            let mut uvs = if with_textures && matches!(material_key, MaterialKey::Textured(_)) {
                Some(Vec::new())
            } else {
                None
            };

            for polygon in polygons {
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

                            // Add UVs for each vertex in the triangle if textures enabled
                            if let Some(ref mut uvs_vec) = uvs {
                                let uv0 = poly.uvls[0].to_f32();
                                let uv1 = poly.uvls[i].to_f32();
                                let uv2 = poly.uvls[i + 1].to_f32();

                                // Only use U and V components, ignore L (lighting)
                                uvs_vec.push(uv0[0]);
                                uvs_vec.push(uv0[1]);
                                uvs_vec.push(uv1[0]);
                                uvs_vec.push(uv1[1]);
                                uvs_vec.push(uv2[0]);
                                uvs_vec.push(uv2[1]);
                            }
                        }
                    }
                }
            }

            primitives.push(MeshPrimitive {
                material_key,
                indices,
                normals,
                uvs,
            });
        }

        Ok(GeometryData {
            positions,
            primitives,
        })
    }

    /// Extract and convert textures from PIG file to PNG format.
    ///
    /// Returns a map of texture_id -> PNG image data (base64 encoded data URI).
    ///
    /// NOTE: Currently disabled - needs HAM file for texture ID mapping
    #[allow(dead_code)]
    fn extract_textures(
        &self,
        _model: &PofModel,
        _provider: &TextureProvider,
    ) -> Result<HashMap<u16, String>> {
        // TODO: Implement once we have HAM file support for texture mapping
        // The challenge: POF uses texture IDs, but PIG uses texture names
        // HAM file contains the mapping: pObjBmIndex[first_texture + texture_id] -> objBmIndex -> bitmap name
        Ok(HashMap::new())
    }

    /// Build binary buffer containing vertex and index data.
    ///
    /// Buffer layout:
    /// 1. Positions (all vertices, Vec3<f32>)
    /// 2. For each primitive:
    ///    - Normals (Vec3<f32> per triangle vertex)
    ///    - UVs (Vec2<f32> per triangle vertex, if textured)
    ///    - Indices (u32)
    fn build_binary_buffer(&self, geometry: &GeometryData) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Write positions (shared by all primitives)
        for &val in &geometry.positions {
            buffer.write_all(&val.to_le_bytes())?;
        }

        // Write per-primitive data
        for primitive in &geometry.primitives {
            // Write normals
            for &val in &primitive.normals {
                buffer.write_all(&val.to_le_bytes())?;
            }

            // Write UVs if present
            if let Some(ref uvs) = primitive.uvs {
                for &val in uvs {
                    buffer.write_all(&val.to_le_bytes())?;
                }
            }

            // Write indices
            for &idx in &primitive.indices {
                buffer.write_all(&idx.to_le_bytes())?;
            }
        }

        if buffer.len() > u32::MAX as usize {
            return Err(AssetError::InvalidFormat(
                "Binary buffer too large (max 4GB)".to_string(),
            ));
        }

        Ok(buffer)
    }

    /// Build the glTF JSON structure.
    ///
    /// Creates glTF 2.0 JSON with multiple mesh primitives grouped by material.
    fn build_gltf_json(
        &self,
        _name: &str,
        geometry: &GeometryData,
        buffer_length: usize,
    ) -> Result<json::Root> {
        // Compute bounding box for positions
        let (min_pos, max_pos) = self.compute_bounds(&geometry.positions);
        let positions_byte_length = geometry.positions.len() * std::mem::size_of::<f32>();

        // Track buffer offsets as we lay out data
        let mut current_offset = positions_byte_length;

        // Build buffer views and accessors for all primitives
        let mut buffer_views = Vec::new();
        let mut accessors = Vec::new();
        let mut gltf_primitives = Vec::new();

        // BufferView 0: Positions (shared by all primitives)
        buffer_views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: USize64::from(positions_byte_length),
            byte_offset: Some(USize64::from(0usize)),
            byte_stride: None,
            target: Some(Valid(json::buffer::Target::ArrayBuffer)),
            extensions: None,
            extras: Default::default(),
        });

        // Accessor 0: Positions (shared)
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(0)),
            byte_offset: Some(USize64::from(0usize)),
            count: USize64::from(geometry.positions.len() / 3),
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
        });

        // Create buffer views and accessors for each primitive
        for primitive in &geometry.primitives {
            let normals_byte_length = primitive.normals.len() * std::mem::size_of::<f32>();
            let normals_offset = current_offset;
            current_offset += normals_byte_length;

            let normals_buffer_view_index = buffer_views.len();
            buffer_views.push(json::buffer::View {
                buffer: json::Index::new(0),
                byte_length: USize64::from(normals_byte_length),
                byte_offset: Some(USize64::from(normals_offset)),
                byte_stride: None,
                target: Some(Valid(json::buffer::Target::ArrayBuffer)),
                extensions: None,
                extras: Default::default(),
            });

            let normals_accessor_index = accessors.len();
            accessors.push(json::Accessor {
                buffer_view: Some(json::Index::new(normals_buffer_view_index as u32)),
                byte_offset: Some(USize64::from(0usize)),
                count: USize64::from(primitive.normals.len() / 3),
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
            });

            // UVs if present
            let uvs_accessor_index = if let Some(ref uvs) = primitive.uvs {
                let uvs_byte_length = uvs.len() * std::mem::size_of::<f32>();
                let uvs_offset = current_offset;
                current_offset += uvs_byte_length;

                let uvs_buffer_view_index = buffer_views.len();
                buffer_views.push(json::buffer::View {
                    buffer: json::Index::new(0),
                    byte_length: USize64::from(uvs_byte_length),
                    byte_offset: Some(USize64::from(uvs_offset)),
                    byte_stride: None,
                    target: Some(Valid(json::buffer::Target::ArrayBuffer)),
                    extensions: None,
                    extras: Default::default(),
                });

                let uvs_accessor_idx = accessors.len();
                accessors.push(json::Accessor {
                    buffer_view: Some(json::Index::new(uvs_buffer_view_index as u32)),
                    byte_offset: Some(USize64::from(0usize)),
                    count: USize64::from(uvs.len() / 2),
                    component_type: Valid(json::accessor::GenericComponentType(
                        json::accessor::ComponentType::F32,
                    )),
                    type_: Valid(json::accessor::Type::Vec2),
                    min: None,
                    max: None,
                    normalized: false,
                    sparse: None,
                    extensions: None,
                    extras: Default::default(),
                });
                Some(uvs_accessor_idx)
            } else {
                None
            };

            // Indices
            let indices_byte_length = primitive.indices.len() * std::mem::size_of::<u32>();
            let indices_offset = current_offset;
            current_offset += indices_byte_length;

            let indices_buffer_view_index = buffer_views.len();
            buffer_views.push(json::buffer::View {
                buffer: json::Index::new(0),
                byte_length: USize64::from(indices_byte_length),
                byte_offset: Some(USize64::from(indices_offset)),
                byte_stride: None,
                target: Some(Valid(json::buffer::Target::ElementArrayBuffer)),
                extensions: None,
                extras: Default::default(),
            });

            let indices_accessor_index = accessors.len();
            accessors.push(json::Accessor {
                buffer_view: Some(json::Index::new(indices_buffer_view_index as u32)),
                byte_offset: Some(USize64::from(0usize)),
                count: USize64::from(primitive.indices.len()),
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
            });

            // Build primitive attributes
            let mut attributes = std::collections::BTreeMap::new();
            attributes.insert(Valid(json::mesh::Semantic::Positions), json::Index::new(0)); // Shared positions
            attributes.insert(
                Valid(json::mesh::Semantic::Normals),
                json::Index::new(normals_accessor_index as u32),
            );
            if let Some(uvs_idx) = uvs_accessor_index {
                attributes.insert(
                    Valid(json::mesh::Semantic::TexCoords(0)),
                    json::Index::new(uvs_idx as u32),
                );
            }

            gltf_primitives.push(json::mesh::Primitive {
                attributes,
                indices: Some(json::Index::new(indices_accessor_index as u32)),
                material: None, // TODO: Add materials
                mode: Valid(json::mesh::Mode::Triangles),
                targets: None,
                extensions: None,
                extras: Default::default(),
            });
        }

        // Build glTF root structure
        let buffer = json::Buffer {
            byte_length: USize64::from(buffer_length),
            uri: None,
            extensions: None,
            extras: Default::default(),
        };

        let meshes = vec![json::Mesh {
            primitives: gltf_primitives,
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

//! Level geometry converter (RDL/RL2 → glTF/GLB)
//!
//! Converts Descent mine geometry to modern glTF 2.0 format with:
//! - Segment geometry (cube rooms) as meshes
//! - Wall textures mapped to materials
//! - UV coordinates preserved from level data
//! - Optional texture extraction from PIG files
//!
//! # Examples
//!
//! ## Basic Level Conversion (Geometry Only)
//!
//! ```no_run
//! use descent_core::level::Level;
//! use descent_core::converters::level::LevelConverter;
//! use std::fs;
//!
//! let level_data = fs::read("level01.rdl").unwrap();
//! let level = Level::parse(&level_data, None).unwrap();
//!
//! let converter = LevelConverter::new();
//! let glb = converter.level_to_glb(&level, "Level 1", None).unwrap();
//! fs::write("level01.glb", glb).unwrap();
//! ```
//!
//! ## Level Conversion with Textures
//!
//! ```no_run
//! use descent_core::level::Level;
//! use descent_core::pig::PigFile;
//! use descent_core::palette::Palette;
//! use descent_core::ham::HamFile;
//! use descent_core::converters::level::{LevelConverter, LevelTextureProvider};
//! use std::fs;
//!
//! let level_data = fs::read("level01.rdl").unwrap();
//! let level = Level::parse(&level_data, None).unwrap();
//!
//! let pig_data = fs::read("groupa.pig").unwrap();
//! let pig = PigFile::parse(pig_data, false).unwrap();
//!
//! let palette_data = fs::read("groupa.256").unwrap();
//! let palette = Palette::parse(&palette_data).unwrap();
//!
//! let ham_data = fs::read("descent2.ham").unwrap();
//! let ham = HamFile::parse(&ham_data).unwrap();
//!
//! let provider = LevelTextureProvider::new(pig, palette, ham);
//! let converter = LevelConverter::new();
//! let glb = converter.level_to_glb(&level, "Level 1", Some(&provider)).unwrap();
//! fs::write("level01.glb", glb).unwrap();
//! ```

use crate::converters::texture::TextureConverter;
use crate::error::{AssetError, Result};
use crate::geometry::{FixVector, Uvl};
use crate::ham::HamFile;
use crate::level::{Level, SIDE_CORNER_COUNT, Segment, Side, SideType};
use crate::palette::Palette;
use crate::pig::PigFile;
use base64::{Engine as _, engine::general_purpose};
use gltf_json as json;
use gltf_json::validation::USize64;
use std::collections::HashMap;
use std::io::Write;

/// Provides texture data for level conversion.
///
/// This struct holds a PIG file, palette, and HAM file for converting
/// indexed textures to modern formats (PNG) and mapping texture IDs during GLB export.
pub struct LevelTextureProvider {
    pig: PigFile,
    palette: Palette,
    ham: HamFile,
}

impl LevelTextureProvider {
    /// Create a new level texture provider.
    pub fn new(pig: PigFile, palette: Palette, ham: HamFile) -> Self {
        Self { pig, palette, ham }
    }

    /// Get the PIG file reference.
    pub fn pig(&self) -> &PigFile {
        &self.pig
    }

    /// Get the palette reference.
    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    /// Get the HAM file reference.
    pub fn ham(&self) -> &HamFile {
        &self.ham
    }
}

/// Converts Descent level geometry to glTF/GLB format.
pub struct LevelConverter;

impl LevelConverter {
    /// Create a new level converter.
    pub fn new() -> Self {
        Self
    }

    /// Convert a Descent level to GLB format.
    ///
    /// # Arguments
    ///
    /// - `level` - The parsed level data
    /// - `name` - Name for the glTF scene
    /// - `texture_provider` - Optional texture provider for extracting wall textures
    ///
    /// # Returns
    ///
    /// GLB file data as a byte vector.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use descent_core::level::Level;
    /// # use descent_core::pig::PigFile;
    /// # use descent_core::palette::Palette;
    /// # use descent_core::ham::HamFile;
    /// # use descent_core::converters::level::{LevelConverter, LevelTextureProvider};
    /// # let level_data = vec![];
    /// # let level = Level::parse(&level_data, None).unwrap();
    /// # let pig = PigFile::parse(vec![], false).unwrap();
    /// # let palette = Palette::parse(&[]).unwrap();
    /// # let ham_data = vec![];
    /// # let ham = HamFile::parse(&ham_data).unwrap();
    /// let provider = LevelTextureProvider::new(pig, palette, ham);
    /// let converter = LevelConverter::new();
    /// let glb = converter.level_to_glb(&level, "Level 1", Some(&provider)).unwrap();
    /// ```
    pub fn level_to_glb(
        &self,
        level: &Level,
        name: &str,
        texture_provider: Option<&LevelTextureProvider>,
    ) -> Result<Vec<u8>> {
        // Extract texture images if provider is available
        let texture_images: HashMap<u16, String> = if let Some(provider) = texture_provider {
            self.extract_textures(level, provider)?
        } else {
            HashMap::new()
        };

        // Get palette reference for color conversion (if no textures available)
        let palette = texture_provider.map(|p| p.palette());

        // Build geometry data
        let (positions, normals, uvs, indices, material_indices) =
            self.build_geometry(level, palette)?;

        // Build binary buffer
        let buffer_data = self.build_buffer(&positions, &normals, &uvs, &indices)?;

        // Build glTF JSON
        let gltf_json = self.build_gltf_json(
            name,
            &buffer_data,
            &positions,
            &normals,
            &uvs,
            &indices,
            &material_indices,
            &texture_images,
            palette,
        )?;

        // Serialize JSON
        let json_string = json::serialize::to_string(&gltf_json)
            .map_err(|e| AssetError::InvalidFormat(format!("JSON serialization: {}", e)))?;
        let json_bytes = json_string.as_bytes();

        // Build GLB file
        self.build_glb_file(json_bytes, &buffer_data)
    }

    /// Extract textures from level geometry.
    fn extract_textures(
        &self,
        level: &Level,
        provider: &LevelTextureProvider,
    ) -> Result<HashMap<u16, String>> {
        let mut texture_images: HashMap<u16, String> = HashMap::new();
        let texture_converter = TextureConverter::new(provider.palette());

        // Collect unique texture IDs from all segments
        let mut texture_ids: Vec<u16> = Vec::new();
        for segment in &level.segments {
            for side in &segment.sides {
                // Check if side has a child (solid wall)
                let side_idx = segment
                    .sides
                    .iter()
                    .position(|s| std::ptr::eq(s, side))
                    .unwrap();
                if segment.children[side_idx] != -1 {
                    continue; // Skip non-solid sides
                }

                // Add base texture
                if side.base_texture != 0 && !texture_ids.contains(&side.base_texture) {
                    texture_ids.push(side.base_texture);
                }

                // Add overlay texture
                if side.overlay_texture != 0 && !texture_ids.contains(&side.overlay_texture) {
                    texture_ids.push(side.overlay_texture);
                }
            }
        }

        // Convert each unique texture to PNG
        for texture_id in texture_ids {
            // Look up texture in HAM
            if let Some(bitmap_index) = provider.ham().lookup_texture(texture_id as usize) {
                // Get bitmap from PIG by index
                if let Some(bitmap_header) = provider.pig().get_by_index(bitmap_index as usize) {
                    let bitmap_name = &bitmap_header.name;

                    // Convert to PNG
                    if let Ok(png_data) = texture_converter.pig_to_png(provider.pig(), bitmap_name)
                    {
                        // Encode as base64 data URI
                        let base64_data = general_purpose::STANDARD.encode(&png_data);
                        let data_uri = format!("data:image/png;base64,{}", base64_data);
                        texture_images.insert(texture_id, data_uri);
                    }
                }
            }
        }

        Ok(texture_images)
    }

    /// Build geometry data from level segments.
    #[allow(clippy::type_complexity)]
    fn build_geometry(
        &self,
        level: &Level,
        _palette: Option<&Palette>,
    ) -> Result<(Vec<f32>, Vec<f32>, Vec<f32>, Vec<u32>, Vec<u32>)> {
        let mut positions: Vec<f32> = Vec::new();
        let mut normals: Vec<f32> = Vec::new();
        let mut uvs: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut material_indices: Vec<u32> = Vec::new();

        let mut vertex_offset: u32 = 0;

        for segment in &level.segments {
            for (side_idx, side) in segment.sides.iter().enumerate() {
                // Check if side has a child (if so, it's not a solid wall)
                if segment.children[side_idx] != -1 {
                    continue; // Skip non-solid sides
                }

                // Get vertex positions for this side
                let side_verts = self.get_side_vertices(segment, side, &level.vertices)?;

                // Build geometry for this side
                let (side_pos, side_norms, side_uvs, side_inds) =
                    self.build_side_geometry(&side_verts, side)?;

                // Add to buffers
                positions.extend(side_pos);
                normals.extend(side_norms);
                uvs.extend(side_uvs);

                // Adjust indices for current vertex offset
                for idx in side_inds {
                    indices.push(idx + vertex_offset);
                }

                // Material index (base texture ID)
                let material_id = if side.base_texture != 0 {
                    side.base_texture as u32
                } else {
                    0 // Default material
                };

                // Add material index for each triangle
                let triangle_count = if side.side_type == SideType::Quad {
                    2
                } else {
                    1
                };
                for _ in 0..triangle_count {
                    material_indices.push(material_id);
                }

                vertex_offset += side_verts.len() as u32;
            }
        }

        Ok((positions, normals, uvs, indices, material_indices))
    }

    /// Get vertex positions for a side.
    fn get_side_vertices(
        &self,
        segment: &Segment,
        side: &Side,
        level_vertices: &[FixVector],
    ) -> Result<Vec<FixVector>> {
        let mut verts = Vec::new();
        for &corner_idx in &side.corners {
            if corner_idx == 0xFF {
                break; // Unused corner
            }
            let vert_idx = segment.vertices[corner_idx as usize] as usize;
            if vert_idx >= level_vertices.len() {
                return Err(AssetError::InvalidLevelFormat(format!(
                    "Vertex index {} out of bounds",
                    vert_idx
                )));
            }
            verts.push(level_vertices[vert_idx]);
        }
        Ok(verts)
    }

    /// Build geometry for a single side (quad or triangle).
    #[allow(clippy::type_complexity)]
    fn build_side_geometry(
        &self,
        vertices: &[FixVector],
        side: &Side,
    ) -> Result<(Vec<f32>, Vec<f32>, Vec<f32>, Vec<u32>)> {
        let mut positions: Vec<f32> = Vec::new();
        let mut normals: Vec<f32> = Vec::new();
        let mut uvs: Vec<f32> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        // Convert vertices to f32
        for vert in vertices {
            let [x, y, z] = vert.to_f32();
            positions.push(x);
            positions.push(y);
            positions.push(z);
        }

        // Calculate normal (cross product of two edges)
        if vertices.len() >= 3 {
            let v0 = vertices[0].to_f32();
            let v1 = vertices[1].to_f32();
            let v2 = vertices[2].to_f32();

            // Edge vectors
            let e1 = (v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]);
            let e2 = (v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]);

            // Cross product
            let nx = e1.1 * e2.2 - e1.2 * e2.1;
            let ny = e1.2 * e2.0 - e1.0 * e2.2;
            let nz = e1.0 * e2.1 - e1.1 * e2.0;

            // Normalize
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            let (nx, ny, nz) = if len > 0.0 {
                (nx / len, ny / len, nz / len)
            } else {
                (0.0, 0.0, 1.0)
            };

            // Same normal for all vertices
            for _ in 0..vertices.len() {
                normals.push(nx);
                normals.push(ny);
                normals.push(nz);
            }
        }

        // UV coordinates from side.uvls
        for i in 0..vertices.len() {
            let uvl = if i < SIDE_CORNER_COUNT {
                side.uvls[i]
            } else {
                Uvl::default()
            };
            let [u, v, _l] = uvl.to_f32();
            uvs.push(u);
            uvs.push(v);
        }

        // Build indices based on side type
        match side.side_type {
            SideType::Quad => {
                // Two triangles: 0-1-2, 0-2-3
                indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
            }
            SideType::Tri02 => {
                // One triangle: 0-1-2
                indices.extend_from_slice(&[0, 1, 2]);
            }
            SideType::Tri13 => {
                // One triangle: 1-2-3
                indices.extend_from_slice(&[1, 2, 3]);
            }
        }

        Ok((positions, normals, uvs, indices))
    }

    /// Build binary buffer containing all geometry data.
    fn build_buffer(
        &self,
        positions: &[f32],
        normals: &[f32],
        uvs: &[f32],
        indices: &[u32],
    ) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Write positions (VEC3 float)
        for &pos in positions {
            buffer
                .write_all(&pos.to_le_bytes())
                .map_err(|e| AssetError::Other(format!("Failed to write positions: {}", e)))?;
        }

        // Pad to 4-byte alignment
        while buffer.len() % 4 != 0 {
            buffer.push(0);
        }

        // Write normals (VEC3 float)
        for &norm in normals {
            buffer
                .write_all(&norm.to_le_bytes())
                .map_err(|e| AssetError::Other(format!("Failed to write normals: {}", e)))?;
        }

        // Pad to 4-byte alignment
        while buffer.len() % 4 != 0 {
            buffer.push(0);
        }

        // Write UVs (VEC2 float)
        for &uv in uvs {
            buffer
                .write_all(&uv.to_le_bytes())
                .map_err(|e| AssetError::Other(format!("Failed to write UVs: {}", e)))?;
        }

        // Pad to 4-byte alignment
        while buffer.len() % 4 != 0 {
            buffer.push(0);
        }

        // Write indices (SCALAR u32)
        for &idx in indices {
            buffer
                .write_all(&idx.to_le_bytes())
                .map_err(|e| AssetError::Other(format!("Failed to write indices: {}", e)))?;
        }

        Ok(buffer)
    }

    /// Build glTF JSON structure.
    #[allow(clippy::too_many_arguments)]
    fn build_gltf_json(
        &self,
        name: &str,
        buffer_data: &[u8],
        positions: &[f32],
        normals: &[f32],
        uvs: &[f32],
        indices: &[u32],
        material_indices: &[u32],
        texture_images: &HashMap<u16, String>,
        palette: Option<&Palette>,
    ) -> Result<json::Root> {
        let mut root = json::Root {
            asset: json::Asset {
                version: "2.0".to_string(),
                generator: Some("d2x-rs level converter".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        // Calculate buffer view offsets
        let positions_byte_length = positions.len() * 4;
        let normals_byte_length = normals.len() * 4;
        let uvs_byte_length = uvs.len() * 4;
        let indices_byte_length = indices.len() * 4;

        let positions_offset = 0;
        let normals_offset = (positions_offset + positions_byte_length).div_ceil(4) * 4;
        let uvs_offset = (normals_offset + normals_byte_length).div_ceil(4) * 4;
        let indices_offset = (uvs_offset + uvs_byte_length).div_ceil(4) * 4;

        // Buffer
        root.buffers.push(json::Buffer {
            name: Some(format!("{} - Buffer", name)),
            byte_length: USize64::from(buffer_data.len()),
            uri: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        // Buffer views
        root.buffer_views.push(json::buffer::View {
            name: Some(format!("{} - Positions BufferView", name)),
            buffer: json::Index::new(0),
            byte_offset: Some(USize64::from(positions_offset)),
            byte_length: USize64::from(positions_byte_length),
            byte_stride: None,
            target: Some(json::validation::Checked::Valid(
                json::buffer::Target::ArrayBuffer,
            )),
            extensions: Default::default(),
            extras: Default::default(),
        });

        root.buffer_views.push(json::buffer::View {
            name: Some(format!("{} - Normals BufferView", name)),
            buffer: json::Index::new(0),
            byte_offset: Some(USize64::from(normals_offset)),
            byte_length: USize64::from(normals_byte_length),
            byte_stride: None,
            target: Some(json::validation::Checked::Valid(
                json::buffer::Target::ArrayBuffer,
            )),
            extensions: Default::default(),
            extras: Default::default(),
        });

        root.buffer_views.push(json::buffer::View {
            name: Some(format!("{} - UVs BufferView", name)),
            buffer: json::Index::new(0),
            byte_offset: Some(USize64::from(uvs_offset)),
            byte_length: USize64::from(uvs_byte_length),
            byte_stride: None,
            target: Some(json::validation::Checked::Valid(
                json::buffer::Target::ArrayBuffer,
            )),
            extensions: Default::default(),
            extras: Default::default(),
        });

        root.buffer_views.push(json::buffer::View {
            name: Some(format!("{} - Indices BufferView", name)),
            buffer: json::Index::new(0),
            byte_offset: Some(USize64::from(indices_offset)),
            byte_length: USize64::from(indices_byte_length),
            byte_stride: None,
            target: Some(json::validation::Checked::Valid(
                json::buffer::Target::ElementArrayBuffer,
            )),
            extensions: Default::default(),
            extras: Default::default(),
        });

        // Compute bounds for positions
        let (min_pos, max_pos) = self.compute_bounds(positions);

        // Accessors
        root.accessors.push(json::Accessor {
            name: Some(format!("{} - Positions", name)),
            buffer_view: Some(json::Index::new(0)),
            byte_offset: Some(USize64::from(0usize)),
            count: USize64::from(positions.len() / 3),
            component_type: json::validation::Checked::Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            type_: json::validation::Checked::Valid(json::accessor::Type::Vec3),
            min: Some(json::Value::from(min_pos)),
            max: Some(json::Value::from(max_pos)),
            normalized: false,
            sparse: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        root.accessors.push(json::Accessor {
            name: Some(format!("{} - Normals", name)),
            buffer_view: Some(json::Index::new(1)),
            byte_offset: Some(USize64::from(0usize)),
            count: USize64::from(normals.len() / 3),
            component_type: json::validation::Checked::Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            type_: json::validation::Checked::Valid(json::accessor::Type::Vec3),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        root.accessors.push(json::Accessor {
            name: Some(format!("{} - UVs", name)),
            buffer_view: Some(json::Index::new(2)),
            byte_offset: Some(USize64::from(0usize)),
            count: USize64::from(uvs.len() / 2),
            component_type: json::validation::Checked::Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            type_: json::validation::Checked::Valid(json::accessor::Type::Vec2),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        root.accessors.push(json::Accessor {
            name: Some(format!("{} - Indices", name)),
            buffer_view: Some(json::Index::new(3)),
            byte_offset: Some(USize64::from(0usize)),
            count: USize64::from(indices.len()),
            component_type: json::validation::Checked::Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::U32,
            )),
            type_: json::validation::Checked::Valid(json::accessor::Type::Scalar),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        // Create images and textures
        let mut texture_index_map: HashMap<u16, json::Index<json::Texture>> = HashMap::new();
        for (&texture_id, data_uri) in texture_images {
            let image_idx = root.images.len() as u32;
            root.images.push(json::Image {
                name: Some(format!("{} - Texture {}", name, texture_id)),
                uri: Some(data_uri.clone()),
                mime_type: None,
                buffer_view: None,
                extensions: Default::default(),
                extras: Default::default(),
            });

            let texture_idx = root.textures.len() as u32;
            root.textures.push(json::Texture {
                name: Some(format!("{} - Texture {}", name, texture_id)),
                sampler: None,
                source: json::Index::new(image_idx),
                extensions: Default::default(),
                extras: Default::default(),
            });

            texture_index_map.insert(texture_id, json::Index::new(texture_idx));
        }

        // Create materials for each unique texture ID
        let unique_texture_ids: Vec<u16> = {
            let mut ids: Vec<u16> = material_indices.iter().map(|&id| id as u16).collect();
            ids.sort_unstable();
            ids.dedup();
            ids
        };

        let mut material_id_to_index: HashMap<u16, u32> = HashMap::new();
        for texture_id in unique_texture_ids {
            let material_idx = root.materials.len() as u32;
            material_id_to_index.insert(texture_id, material_idx);

            let pbr = if let Some(&texture_idx) = texture_index_map.get(&texture_id) {
                // Textured material
                json::material::PbrMetallicRoughness {
                    base_color_factor: json::material::PbrBaseColorFactor([1.0, 1.0, 1.0, 1.0]),
                    base_color_texture: Some(json::texture::Info {
                        index: texture_idx,
                        tex_coord: 0,
                        extensions: Default::default(),
                        extras: Default::default(),
                    }),
                    metallic_factor: json::material::StrengthFactor(0.0),
                    roughness_factor: json::material::StrengthFactor(1.0),
                    metallic_roughness_texture: None,
                    extensions: Default::default(),
                    extras: Default::default(),
                }
            } else {
                // Flat material (no texture) - use gray or palette color
                let color = if texture_id == 0 {
                    [0.5, 0.5, 0.5, 1.0] // Default gray
                } else if let Some(pal) = palette {
                    // Use palette color (texture_id as palette index)
                    let color_index = (texture_id % 256) as u8;
                    let [r, g, b] = pal.get_rgb8(color_index);
                    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0]
                } else {
                    [0.5, 0.5, 0.5, 1.0] // Fallback gray
                };

                json::material::PbrMetallicRoughness {
                    base_color_factor: json::material::PbrBaseColorFactor(color),
                    base_color_texture: None,
                    metallic_factor: json::material::StrengthFactor(0.0),
                    roughness_factor: json::material::StrengthFactor(1.0),
                    metallic_roughness_texture: None,
                    extensions: Default::default(),
                    extras: Default::default(),
                }
            };

            root.materials.push(json::Material {
                name: Some(format!("{} - Material {}", name, texture_id)),
                pbr_metallic_roughness: pbr,
                normal_texture: None,
                occlusion_texture: None,
                emissive_texture: None,
                emissive_factor: json::material::EmissiveFactor([0.0, 0.0, 0.0]),
                alpha_mode: json::validation::Checked::Valid(json::material::AlphaMode::Opaque),
                alpha_cutoff: None,
                double_sided: false,
                extensions: Default::default(),
                extras: Default::default(),
            });
        }

        // Create mesh primitives grouped by material
        let mut primitives: Vec<json::mesh::Primitive> = Vec::new();
        // TODO: Group triangles by material and create separate primitives
        // For now, create one primitive with default material
        primitives.push(json::mesh::Primitive {
            attributes: {
                let mut map = std::collections::BTreeMap::new();
                map.insert(
                    json::validation::Checked::Valid(json::mesh::Semantic::Positions),
                    json::Index::new(0),
                );
                map.insert(
                    json::validation::Checked::Valid(json::mesh::Semantic::Normals),
                    json::Index::new(1),
                );
                map.insert(
                    json::validation::Checked::Valid(json::mesh::Semantic::TexCoords(0)),
                    json::Index::new(2),
                );
                map
            },
            indices: Some(json::Index::new(3)),
            material: Some(json::Index::new(0)),
            mode: json::validation::Checked::Valid(json::mesh::Mode::Triangles),
            targets: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        // Mesh
        root.meshes.push(json::Mesh {
            name: Some(format!("{} - Mesh", name)),
            primitives,
            weights: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        // Node
        root.nodes.push(json::Node {
            name: Some(format!("{} - Node", name)),
            mesh: Some(json::Index::new(0)),
            camera: None,
            children: None,
            skin: None,
            matrix: None,
            rotation: None,
            scale: None,
            translation: None,
            weights: None,
            extensions: Default::default(),
            extras: Default::default(),
        });

        // Scene
        root.scenes.push(json::Scene {
            name: Some(format!("{} - Scene", name)),
            nodes: vec![json::Index::new(0)],
            extensions: Default::default(),
            extras: Default::default(),
        });

        root.scene = Some(json::Index::new(0));

        Ok(root)
    }

    /// Compute bounding box for positions.
    fn compute_bounds(&self, positions: &[f32]) -> (Vec<f32>, Vec<f32>) {
        if positions.is_empty() {
            return (vec![0.0, 0.0, 0.0], vec![0.0, 0.0, 0.0]);
        }

        let mut min = [f32::MAX, f32::MAX, f32::MAX];
        let mut max = [f32::MIN, f32::MIN, f32::MIN];

        for chunk in positions.chunks(3) {
            if chunk.len() == 3 {
                for i in 0..3 {
                    min[i] = min[i].min(chunk[i]);
                    max[i] = max[i].max(chunk[i]);
                }
            }
        }

        (min.to_vec(), max.to_vec())
    }

    /// Build GLB file from JSON and binary buffer.
    fn build_glb_file(&self, json_bytes: &[u8], buffer_data: &[u8]) -> Result<Vec<u8>> {
        let mut glb = Vec::new();

        // GLB header
        glb.write_all(b"glTF")
            .map_err(|e| AssetError::Other(format!("Failed to write GLB magic: {}", e)))?;
        glb.write_all(&2u32.to_le_bytes())
            .map_err(|e| AssetError::Other(format!("Failed to write GLB version: {}", e)))?;

        // Calculate chunk sizes (with padding)
        let json_padded_len = json_bytes.len().div_ceil(4) * 4;
        let buffer_padded_len = buffer_data.len().div_ceil(4) * 4;

        // Total length = header(12) + json_chunk_header(8) + json_data + buffer_chunk_header(8) + buffer_data
        let total_length = 12 + 8 + json_padded_len + 8 + buffer_padded_len;
        glb.write_all(&(total_length as u32).to_le_bytes())
            .map_err(|e| AssetError::Other(format!("Failed to write GLB length: {}", e)))?;

        // JSON chunk
        glb.write_all(&(json_padded_len as u32).to_le_bytes())
            .map_err(|e| AssetError::Other(format!("Failed to write JSON chunk length: {}", e)))?;
        glb.write_all(b"JSON")
            .map_err(|e| AssetError::Other(format!("Failed to write JSON chunk type: {}", e)))?;
        glb.write_all(json_bytes)
            .map_err(|e| AssetError::Other(format!("Failed to write JSON data: {}", e)))?;

        // Pad JSON to 4-byte alignment with spaces
        while glb.len() % 4 != 0 {
            glb.push(b' ');
        }

        // Binary chunk
        glb.write_all(&(buffer_padded_len as u32).to_le_bytes())
            .map_err(|e| {
                AssetError::Other(format!("Failed to write buffer chunk length: {}", e))
            })?;
        glb.write_all(b"BIN\0")
            .map_err(|e| AssetError::Other(format!("Failed to write buffer chunk type: {}", e)))?;
        glb.write_all(buffer_data)
            .map_err(|e| AssetError::Other(format!("Failed to write buffer data: {}", e)))?;

        // Pad buffer to 4-byte alignment with zeros
        while glb.len() % 4 != 0 {
            glb.push(0);
        }

        Ok(glb)
    }
}

impl Default for LevelConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_converter_new() {
        let converter = LevelConverter::new();
        assert!(std::ptr::eq(&converter, &converter));
    }

    #[test]
    fn test_compute_bounds() {
        let converter = LevelConverter::new();
        let positions = vec![
            -1.0, -2.0, -3.0, // min point
            4.0, 5.0, 6.0, // max point
            0.0, 0.0, 0.0, // origin
        ];
        let (min, max) = converter.compute_bounds(&positions);
        assert_eq!(min, vec![-1.0, -2.0, -3.0]);
        assert_eq!(max, vec![4.0, 5.0, 6.0]);
    }

    #[test]
    fn test_compute_bounds_empty() {
        let converter = LevelConverter::new();
        let (min, max) = converter.compute_bounds(&[]);
        assert_eq!(min, vec![0.0, 0.0, 0.0]);
        assert_eq!(max, vec![0.0, 0.0, 0.0]);
    }
}

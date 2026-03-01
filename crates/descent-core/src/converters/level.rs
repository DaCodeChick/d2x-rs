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
use crate::level::{Level, Segment, Side, SideType, SIDE_CORNER_COUNT};
use crate::palette::Palette;
use crate::pig::PigFile;
use base64::{engine::general_purpose, Engine as _};
use gltf_json as json;
use gltf_json::validation::USize64;
use std::collections::{HashMap, HashSet};
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

/// Accumulated geometry data for level conversion.
struct GeometryData {
    positions: Vec<f32>,
    normals: Vec<f32>,
    uvs: Vec<f32>,
    indices: Vec<u32>,
    material_indices: Vec<u32>,
}

impl GeometryData {
    /// Create a new empty geometry data accumulator.
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            material_indices: Vec::new(),
        }
    }
}

/// Material-grouped index data for creating separate primitives.
#[derive(Debug, Clone)]
struct MaterialGroup {
    material_id: u32,
    indices: Vec<u32>,
}

/// Buffer layout information for glTF structure.
struct BufferLayout {
    positions_offset: usize,
    positions_length: usize,
    normals_offset: usize,
    normals_length: usize,
    uvs_offset: usize,
    uvs_length: usize,
    /// Offset and length for each material group's indices
    material_index_ranges: Vec<(usize, usize)>,
}

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
        let geometry = self.build_geometry(level, palette)?;

        // Group indices by material for separate primitives
        let material_groups =
            Self::group_indices_by_material(&geometry.indices, &geometry.material_indices);

        // Build binary buffer with material-grouped indices
        let buffer_data = self.build_buffer_with_materials(
            &geometry.positions,
            &geometry.normals,
            &geometry.uvs,
            &material_groups,
        )?;

        // Build glTF JSON
        let gltf_json = self.build_gltf_json(
            name,
            &buffer_data,
            &geometry.positions,
            &geometry.normals,
            &geometry.uvs,
            &material_groups,
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
        let texture_ids = self.collect_texture_ids(level);
        self.convert_textures_to_data_uris(&texture_ids, provider)
    }

    /// Collect all unique texture IDs from solid walls.
    fn collect_texture_ids(&self, level: &Level) -> HashSet<u16> {
        level
            .segments
            .iter()
            .flat_map(|segment| {
                segment
                    .sides
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| Self::is_solid_wall(segment, *idx))
                    .flat_map(|(_, side)| [side.base_texture, side.overlay_texture])
                    .filter(|&tex_id| tex_id != 0)
            })
            .collect()
    }

    /// Check if a side is a solid wall (not an open passage).
    const fn is_solid_wall(segment: &Segment, side_idx: usize) -> bool {
        segment.children[side_idx] == -1
    }

    /// Convert texture IDs to PNG data URIs.
    fn convert_textures_to_data_uris(
        &self,
        texture_ids: &HashSet<u16>,
        provider: &LevelTextureProvider,
    ) -> Result<HashMap<u16, String>> {
        let texture_converter = TextureConverter::new(provider.palette());
        let mut texture_images = HashMap::new();

        for &texture_id in texture_ids {
            if let Some(data_uri) =
                self.convert_single_texture(texture_id, provider, &texture_converter)
            {
                texture_images.insert(texture_id, data_uri);
            }
        }

        Ok(texture_images)
    }

    /// Convert a single texture ID to a PNG data URI.
    fn convert_single_texture(
        &self,
        texture_id: u16,
        provider: &LevelTextureProvider,
        converter: &TextureConverter,
    ) -> Option<String> {
        let bitmap_index = provider.ham().lookup_texture(texture_id as usize)?;
        let bitmap_header = provider.pig().get_by_index(bitmap_index as usize)?;
        let png_data = converter
            .pig_to_png(provider.pig(), &bitmap_header.name)
            .ok()?;

        let base64_data = general_purpose::STANDARD.encode(&png_data);
        Some(format!("data:image/png;base64,{}", base64_data))
    }

    /// Build geometry data from level segments.
    fn build_geometry(&self, level: &Level, _palette: Option<&Palette>) -> Result<GeometryData> {
        let mut geometry = GeometryData::new();
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
                geometry.positions.extend(side_pos);
                geometry.normals.extend(side_norms);
                geometry.uvs.extend(side_uvs);

                // Adjust indices for current vertex offset
                geometry
                    .indices
                    .extend(side_inds.iter().map(|idx| idx + vertex_offset));

                // Material index (base texture ID)
                let material_id = if side.base_texture != 0 {
                    side.base_texture as u32
                } else {
                    0 // Default material
                };

                // Add material index for each triangle
                let triangle_count = Self::triangle_count_for_side_type(side.side_type);
                geometry
                    .material_indices
                    .extend(std::iter::repeat_n(material_id, triangle_count));

                vertex_offset += side_verts.len() as u32;
            }
        }

        Ok(geometry)
    }

    /// Group triangle indices by material ID.
    ///
    /// Takes the flat index array and material_indices array (one entry per triangle),
    /// and returns a vector of MaterialGroup structs, each containing indices for
    /// triangles of that material. Groups are sorted by material_id for deterministic output.
    fn group_indices_by_material(indices: &[u32], material_indices: &[u32]) -> Vec<MaterialGroup> {
        // material_indices contains one entry per triangle
        // indices contains 3 entries per triangle (one triangle = 3 indices)

        // Build a map of material_id -> Vec<triangle_indices>
        let mut material_map: HashMap<u32, Vec<u32>> = HashMap::new();

        for (tri_idx, &mat_id) in material_indices.iter().enumerate() {
            let start_idx = tri_idx * 3;
            let tri_indices = &indices[start_idx..start_idx + 3];
            material_map
                .entry(mat_id)
                .or_default()
                .extend_from_slice(tri_indices);
        }

        // Convert to sorted vector (sort by material_id for deterministic output)
        let mut groups: Vec<MaterialGroup> = material_map
            .into_iter()
            .map(|(material_id, indices)| MaterialGroup {
                material_id,
                indices,
            })
            .collect();
        groups.sort_by_key(|g| g.material_id);

        groups
    }

    /// Get the number of triangles for a side type.
    const fn triangle_count_for_side_type(side_type: SideType) -> usize {
        match side_type {
            SideType::Quad => 2,
            SideType::Tri02 | SideType::Tri13 => 1,
        }
    }

    /// Get vertex positions for a side.
    fn get_side_vertices(
        &self,
        segment: &Segment,
        side: &Side,
        level_vertices: &[FixVector],
    ) -> Result<Vec<FixVector>> {
        side.corners
            .iter()
            .take_while(|&&idx| idx != 0xFF)
            .map(|&corner_idx| {
                let vert_idx = segment.vertices[corner_idx as usize] as usize;
                level_vertices.get(vert_idx).copied().ok_or_else(|| {
                    AssetError::InvalidLevelFormat(format!(
                        "Vertex index {} out of bounds",
                        vert_idx
                    ))
                })
            })
            .collect()
    }

    /// Build geometry for a single side (quad or triangle).
    #[allow(clippy::type_complexity)]
    fn build_side_geometry(
        &self,
        vertices: &[FixVector],
        side: &Side,
    ) -> Result<(Vec<f32>, Vec<f32>, Vec<f32>, Vec<u32>)> {
        let positions = Self::build_positions(vertices);
        let normals = Self::build_normals(vertices);
        let uvs = Self::build_uvs(vertices, side);
        let indices = Self::build_indices(side.side_type);

        Ok((positions, normals, uvs, indices))
    }

    /// Convert vertex positions from FixVector to f32.
    fn build_positions(vertices: &[FixVector]) -> Vec<f32> {
        vertices
            .iter()
            .flat_map(|vert| {
                let [x, y, z] = vert.to_f32();
                [x, y, z]
            })
            .collect()
    }

    /// Calculate face normal and replicate for all vertices (flat shading).
    fn build_normals(vertices: &[FixVector]) -> Vec<f32> {
        if vertices.len() < 3 {
            return Vec::new();
        }

        let normal = Self::calculate_face_normal(vertices);
        std::iter::repeat_n(normal, vertices.len())
            .flatten()
            .collect()
    }

    /// Calculate face normal using cross product of first two edges.
    fn calculate_face_normal(vertices: &[FixVector]) -> [f32; 3] {
        debug_assert!(vertices.len() >= 3);

        let v0 = vertices[0].to_f32();
        let v1 = vertices[1].to_f32();
        let v2 = vertices[2].to_f32();

        // Edge vectors
        let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
        let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

        // Cross product
        let nx = e1[1] * e2[2] - e1[2] * e2[1];
        let ny = e1[2] * e2[0] - e1[0] * e2[2];
        let nz = e1[0] * e2[1] - e1[1] * e2[0];

        // Normalize
        let len = (nx * nx + ny * ny + nz * nz).sqrt();
        if len > 0.0 {
            [nx / len, ny / len, nz / len]
        } else {
            [0.0, 0.0, 1.0] // Fallback for degenerate triangles
        }
    }

    /// Extract UV coordinates from side data.
    fn build_uvs(vertices: &[FixVector], side: &Side) -> Vec<f32> {
        (0..vertices.len())
            .flat_map(|i| {
                let uvl = if i < SIDE_CORNER_COUNT {
                    side.uvls[i]
                } else {
                    Uvl::default()
                };
                let [u, v, _l] = uvl.to_f32();
                [u, v]
            })
            .collect()
    }

    /// Generate triangle indices based on side type.
    fn build_indices(side_type: SideType) -> Vec<u32> {
        match side_type {
            SideType::Quad => vec![0, 1, 2, 0, 2, 3],
            SideType::Tri02 => vec![0, 1, 2],
            SideType::Tri13 => vec![1, 2, 3],
        }
    }

    /// Build binary buffer containing all geometry data.
    /// Build binary buffer with material-grouped indices.
    ///
    /// Writes vertex attributes (positions, normals, UVs) once, then writes
    /// index data for each material group sequentially.
    fn build_buffer_with_materials(
        &self,
        positions: &[f32],
        normals: &[f32],
        uvs: &[f32],
        material_groups: &[MaterialGroup],
    ) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Write positions (VEC3 float)
        for &pos in positions {
            buffer
                .write_all(&pos.to_le_bytes())
                .map_err(|e| AssetError::Other(format!("Failed to write positions: {}", e)))?;
        }
        Self::align_buffer_to_4_bytes(&mut buffer);

        // Write normals (VEC3 float)
        for &norm in normals {
            buffer
                .write_all(&norm.to_le_bytes())
                .map_err(|e| AssetError::Other(format!("Failed to write normals: {}", e)))?;
        }
        Self::align_buffer_to_4_bytes(&mut buffer);

        // Write UVs (VEC2 float)
        for &uv in uvs {
            buffer
                .write_all(&uv.to_le_bytes())
                .map_err(|e| AssetError::Other(format!("Failed to write UVs: {}", e)))?;
        }
        Self::align_buffer_to_4_bytes(&mut buffer);

        // Write indices for each material group (SCALAR u32)
        for group in material_groups {
            for &idx in &group.indices {
                buffer
                    .write_all(&idx.to_le_bytes())
                    .map_err(|e| AssetError::Other(format!("Failed to write indices: {}", e)))?;
            }
            Self::align_buffer_to_4_bytes(&mut buffer);
        }

        Ok(buffer)
    }

    /// Align buffer to 4-byte boundary with zero padding.
    fn align_buffer_to_4_bytes(buffer: &mut Vec<u8>) {
        while !buffer.len().is_multiple_of(4) {
            buffer.push(0);
        }
    }

    /// Calculate buffer view offsets and lengths for material-grouped layout.
    fn calculate_buffer_layout(
        positions: &[f32],
        normals: &[f32],
        uvs: &[f32],
        material_groups: &[MaterialGroup],
    ) -> BufferLayout {
        let positions_byte_length = positions.len() * 4;
        let normals_byte_length = normals.len() * 4;
        let uvs_byte_length = uvs.len() * 4;

        let positions_offset = 0;
        let normals_offset = (positions_offset + positions_byte_length).div_ceil(4) * 4;
        let uvs_offset = (normals_offset + normals_byte_length).div_ceil(4) * 4;

        // Calculate offset and length for each material group's indices
        let mut current_offset = (uvs_offset + uvs_byte_length).div_ceil(4) * 4;
        let mut material_index_ranges = Vec::new();

        for group in material_groups {
            let byte_length = group.indices.len() * 4;
            material_index_ranges.push((current_offset, byte_length));
            current_offset = (current_offset + byte_length).div_ceil(4) * 4;
        }

        BufferLayout {
            positions_offset,
            positions_length: positions_byte_length,
            normals_offset,
            normals_length: normals_byte_length,
            uvs_offset,
            uvs_length: uvs_byte_length,
            material_index_ranges,
        }
    }

    /// Create buffer and buffer views for glTF.
    fn create_buffer_and_views(
        name: &str,
        buffer_data: &[u8],
        layout: &BufferLayout,
    ) -> (json::Buffer, Vec<json::buffer::View>) {
        let buffer = json::Buffer {
            name: Some(format!("{} - Buffer", name)),
            byte_length: USize64::from(buffer_data.len()),
            uri: None,
            extensions: Default::default(),
            extras: Default::default(),
        };

        let mut views = vec![
            json::buffer::View {
                name: Some(format!("{} - Positions BufferView", name)),
                buffer: json::Index::new(0),
                byte_offset: Some(USize64::from(layout.positions_offset)),
                byte_length: USize64::from(layout.positions_length),
                byte_stride: None,
                target: Some(json::validation::Checked::Valid(
                    json::buffer::Target::ArrayBuffer,
                )),
                extensions: Default::default(),
                extras: Default::default(),
            },
            json::buffer::View {
                name: Some(format!("{} - Normals BufferView", name)),
                buffer: json::Index::new(0),
                byte_offset: Some(USize64::from(layout.normals_offset)),
                byte_length: USize64::from(layout.normals_length),
                byte_stride: None,
                target: Some(json::validation::Checked::Valid(
                    json::buffer::Target::ArrayBuffer,
                )),
                extensions: Default::default(),
                extras: Default::default(),
            },
            json::buffer::View {
                name: Some(format!("{} - UVs BufferView", name)),
                buffer: json::Index::new(0),
                byte_offset: Some(USize64::from(layout.uvs_offset)),
                byte_length: USize64::from(layout.uvs_length),
                byte_stride: None,
                target: Some(json::validation::Checked::Valid(
                    json::buffer::Target::ArrayBuffer,
                )),
                extensions: Default::default(),
                extras: Default::default(),
            },
        ];

        // Add buffer view for each material group's indices
        for (mat_idx, &(offset, length)) in layout.material_index_ranges.iter().enumerate() {
            views.push(json::buffer::View {
                name: Some(format!(
                    "{} - Material {} Indices BufferView",
                    name, mat_idx
                )),
                buffer: json::Index::new(0),
                byte_offset: Some(USize64::from(offset)),
                byte_length: USize64::from(length),
                byte_stride: None,
                target: Some(json::validation::Checked::Valid(
                    json::buffer::Target::ElementArrayBuffer,
                )),
                extensions: Default::default(),
                extras: Default::default(),
            });
        }

        (buffer, views)
    }

    /// Create accessors for glTF geometry data with material groups.
    fn create_accessors(
        name: &str,
        positions: &[f32],
        normals: &[f32],
        uvs: &[f32],
        material_groups: &[MaterialGroup],
        min_pos: Vec<f32>,
        max_pos: Vec<f32>,
    ) -> Vec<json::Accessor> {
        let mut accessors = vec![
            json::Accessor {
                name: Some(format!("{} - Positions", name)),
                buffer_view: Some(json::Index::new(0)),
                byte_offset: Some(USize64::from(0usize)),
                count: USize64::from(positions.len() / 3),
                component_type: json::validation::Checked::Valid(
                    json::accessor::GenericComponentType(json::accessor::ComponentType::F32),
                ),
                type_: json::validation::Checked::Valid(json::accessor::Type::Vec3),
                min: Some(json::Value::from(min_pos)),
                max: Some(json::Value::from(max_pos)),
                normalized: false,
                sparse: None,
                extensions: Default::default(),
                extras: Default::default(),
            },
            json::Accessor {
                name: Some(format!("{} - Normals", name)),
                buffer_view: Some(json::Index::new(1)),
                byte_offset: Some(USize64::from(0usize)),
                count: USize64::from(normals.len() / 3),
                component_type: json::validation::Checked::Valid(
                    json::accessor::GenericComponentType(json::accessor::ComponentType::F32),
                ),
                type_: json::validation::Checked::Valid(json::accessor::Type::Vec3),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
                extensions: Default::default(),
                extras: Default::default(),
            },
            json::Accessor {
                name: Some(format!("{} - UVs", name)),
                buffer_view: Some(json::Index::new(2)),
                byte_offset: Some(USize64::from(0usize)),
                count: USize64::from(uvs.len() / 2),
                component_type: json::validation::Checked::Valid(
                    json::accessor::GenericComponentType(json::accessor::ComponentType::F32),
                ),
                type_: json::validation::Checked::Valid(json::accessor::Type::Vec2),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
                extensions: Default::default(),
                extras: Default::default(),
            },
        ];

        // Add accessor for each material group's indices
        // Buffer views start at index 3 (after positions, normals, UVs)
        for (mat_idx, group) in material_groups.iter().enumerate() {
            accessors.push(json::Accessor {
                name: Some(format!("{} - Material {} Indices", name, mat_idx)),
                buffer_view: Some(json::Index::new((3 + mat_idx) as u32)),
                byte_offset: Some(USize64::from(0usize)),
                count: USize64::from(group.indices.len()),
                component_type: json::validation::Checked::Valid(
                    json::accessor::GenericComponentType(json::accessor::ComponentType::U32),
                ),
                type_: json::validation::Checked::Valid(json::accessor::Type::Scalar),
                min: None,
                max: None,
                normalized: false,
                sparse: None,
                extensions: Default::default(),
                extras: Default::default(),
            });
        }

        accessors
    }

    /// Create images and textures from texture data URIs.
    fn create_images_and_textures(
        name: &str,
        texture_images: &HashMap<u16, String>,
    ) -> (
        Vec<json::Image>,
        Vec<json::Texture>,
        HashMap<u16, json::Index<json::Texture>>,
    ) {
        let mut images = Vec::new();
        let mut textures = Vec::new();
        let mut texture_index_map = HashMap::new();

        for (&texture_id, data_uri) in texture_images {
            let image_idx = images.len() as u32;
            images.push(json::Image {
                name: Some(format!("{} - Texture {}", name, texture_id)),
                uri: Some(data_uri.clone()),
                mime_type: None,
                buffer_view: None,
                extensions: Default::default(),
                extras: Default::default(),
            });

            let texture_idx = textures.len() as u32;
            textures.push(json::Texture {
                name: Some(format!("{} - Texture {}", name, texture_id)),
                sampler: None,
                source: json::Index::new(image_idx),
                extensions: Default::default(),
                extras: Default::default(),
            });

            texture_index_map.insert(texture_id, json::Index::new(texture_idx));
        }

        (images, textures, texture_index_map)
    }

    /// Create materials for each unique texture ID.
    /// Create materials from unique material IDs.
    fn create_materials(
        name: &str,
        material_ids: &[u32],
        texture_index_map: &HashMap<u16, json::Index<json::Texture>>,
        palette: Option<&Palette>,
    ) -> (Vec<json::Material>, HashMap<u32, usize>) {
        let mut materials = Vec::new();
        let mut material_id_to_index = HashMap::new();

        for &material_id in material_ids {
            let material_idx = materials.len();
            material_id_to_index.insert(material_id, material_idx);

            let texture_id = material_id as u16;

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

            materials.push(json::Material {
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

        (materials, material_id_to_index)
    }

    /// Create mesh with primitives.
    /// Create mesh with primitives grouped by material.
    ///
    /// Creates one primitive per material group, with each primitive referencing
    /// shared vertex attributes (positions, normals, UVs) but different index accessors.
    fn create_mesh(
        name: &str,
        material_groups: &[MaterialGroup],
        material_id_to_index: &HashMap<u32, usize>,
    ) -> (json::Mesh, Vec<json::mesh::Primitive>) {
        let mut primitives = Vec::new();

        // Create one primitive per material group
        // Accessor indices: 0=positions, 1=normals, 2=UVs, 3+=indices (one per material)
        for (mat_idx, group) in material_groups.iter().enumerate() {
            // Map material_id to glTF material index
            let material_index = material_id_to_index
                .get(&group.material_id)
                .copied()
                .unwrap_or(0);

            primitives.push(json::mesh::Primitive {
                attributes: {
                    let mut map = std::collections::BTreeMap::new();
                    map.insert(
                        json::validation::Checked::Valid(json::mesh::Semantic::Positions),
                        json::Index::new(0), // Shared positions
                    );
                    map.insert(
                        json::validation::Checked::Valid(json::mesh::Semantic::Normals),
                        json::Index::new(1), // Shared normals
                    );
                    map.insert(
                        json::validation::Checked::Valid(json::mesh::Semantic::TexCoords(0)),
                        json::Index::new(2), // Shared UVs
                    );
                    map
                },
                indices: Some(json::Index::new((3 + mat_idx) as u32)), // Per-material indices
                material: Some(json::Index::new(material_index as u32)),
                mode: json::validation::Checked::Valid(json::mesh::Mode::Triangles),
                targets: None,
                extensions: Default::default(),
                extras: Default::default(),
            });
        }

        let mesh = json::Mesh {
            name: Some(format!("{} - Mesh", name)),
            primitives: primitives.clone(),
            weights: None,
            extensions: Default::default(),
            extras: Default::default(),
        };

        (mesh, primitives)
    }

    /// Create scene hierarchy (node and scene).
    fn create_scene_hierarchy(name: &str) -> (json::Node, json::Scene) {
        let node = json::Node {
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
        };

        let scene = json::Scene {
            name: Some(format!("{} - Scene", name)),
            nodes: vec![json::Index::new(0)],
            extensions: Default::default(),
            extras: Default::default(),
        };

        (node, scene)
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
        material_groups: &[MaterialGroup],
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

        // Calculate buffer layout with material groups
        let layout = Self::calculate_buffer_layout(positions, normals, uvs, material_groups);

        // Create buffer and buffer views
        let (buffer, views) = Self::create_buffer_and_views(name, buffer_data, &layout);
        root.buffers.push(buffer);
        root.buffer_views.extend(views);

        // Compute bounds and create accessors
        let (min_pos, max_pos) = self.compute_bounds(positions);
        let accessors = Self::create_accessors(
            name,
            positions,
            normals,
            uvs,
            material_groups,
            min_pos,
            max_pos,
        );
        root.accessors.extend(accessors);

        // Create images and textures
        let (images, textures, texture_index_map) =
            Self::create_images_and_textures(name, texture_images);
        root.images.extend(images);
        root.textures.extend(textures);

        // Extract unique material IDs from groups
        let material_ids: Vec<u32> = material_groups.iter().map(|g| g.material_id).collect();

        // Create materials
        let (materials, material_id_to_index) =
            Self::create_materials(name, &material_ids, &texture_index_map, palette);
        root.materials.extend(materials);

        // Create mesh with material-grouped primitives
        let (mesh, _primitives) = Self::create_mesh(name, material_groups, &material_id_to_index);
        root.meshes.push(mesh);

        let (node, scene) = Self::create_scene_hierarchy(name);
        root.nodes.push(node);
        root.scenes.push(scene);

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

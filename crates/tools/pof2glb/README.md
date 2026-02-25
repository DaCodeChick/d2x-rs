# pof2glb

A command-line tool to convert Descent POF (Parallax Object Format) models to glTF/GLB format with full texture support.

## Features

- Converts POF models to industry-standard GLB format
- Embeds textures as PNG images with base64 data URIs
- Supports both Descent 1 and Descent 2 formats
- Preserves UV mapping and materials (flat + textured)
- Proper texture lookup through HAM → PIG pipeline
- All glTF structures include meaningful names

## Usage

```bash
pof2glb --pof <POF_FILE> --ham <HAM_FILE> --pig <PIG_FILE> --palette <PALETTE_FILE> --output <OUTPUT.glb>
```

### Required Arguments

- `--pof <POF_FILE>` - Path to the POF model file to convert
- `--ham <HAM_FILE>` - Path to the HAM file (contains texture metadata and object definitions)
- `--pig <PIG_FILE>` - Path to the PIG file (contains texture bitmap data)
- `--palette <PALETTE_FILE>` - Path to the palette file (.256 or .pal format)
- `--output <OUTPUT.glb>` - Path where the GLB file will be written

### Optional Arguments

- `--name <NAME>` - Model name to use in glTF metadata (defaults to POF filename)
- `--with-header` - Parse POF with embedded 846-byte header (for POFs extracted from HAM files)
- `--d1` - Treat PIG file as Descent 1 format (default: Descent 2)

## Example

Convert a Descent 2 player ship model:

```bash
pof2glb \
  --pof pyro-gl.pof \
  --ham descent2.ham \
  --pig groupa.pig \
  --palette groupa.256 \
  --output pyro-gl.glb \
  --name "Pyro-GL"
```

## Output Format

The tool generates GLB files with:

- **Geometry**: Triangulated meshes with positions and normals
- **Materials**: PBR materials with:
  - Flat materials: Palette colors as base color
  - Textured materials: PNG textures as base color maps
  - Metallic factor: 0.0, Roughness factor: 1.0 (matte finish)
- **Textures**: Embedded as PNG with base64 data URIs
- **Naming**: All glTF structures (buffers, meshes, materials, etc.) have descriptive names

## File Requirements

### POF Files
Descent model files, typically found in HOG archives or standalone.

### HAM Files
- Descent 1: `descent.ham`
- Descent 2: `descent2.ham`

Contains texture metadata including `obj_bitmap_indices` and `obj_bitmap_pointers` arrays.

### PIG Files
- Descent 1: `descent.pig`
- Descent 2: `groupa.pig`, `groupb.pig`, etc.

Contains indexed bitmap data in RLE-compressed format.

### Palette Files
- Extension: `.256` or `.pal`
- Format: 768 bytes (256 colors × 3 channels)
- Color depth: 6-bit RGB (0-63), scaled to 8-bit (0-255) during conversion

## Viewing the Output

GLB files can be viewed in:

- **glTF Viewer**: https://gltf-viewer.donmccurdy.com/
- **Three.js Editor**: https://threejs.org/editor/
- **Blender**: File → Import → glTF 2.0
- **Bevy**: Use `bevy::prelude::Gltf` asset loader

## Building from Source

```bash
cd d2x-rs
cargo build --package pof2glb --release
```

Binary location: `target/release/pof2glb`

## License

Licensed under either:
- MIT License
- Apache License 2.0

at your option.

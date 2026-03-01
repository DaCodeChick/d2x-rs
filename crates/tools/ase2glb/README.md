# ase2glb

A command-line tool to convert D2X-XL ASE (ASCII Scene Export) models to glTF/GLB format.

## Features

- Converts 3D Studio Max ASE files to industry-standard GLB format
- Preserves geometry, normals, and UV coordinates
- Supports materials and texture references
- All glTF structures include meaningful names

## Usage

```bash
ase2glb --ase <ASE_FILE> --output <OUTPUT.glb> [--name <NAME>]
```

### Required Arguments

- `--ase <ASE_FILE>` - Path to the ASE model file to convert
- `--output <OUTPUT.glb>` - Path where the GLB file will be written

### Optional Arguments

- `--name <NAME>` - Model name to use in glTF metadata (defaults to ASE filename)

## Example

Convert a high-resolution ship model:

```bash
ase2glb \
  --ase pyro-gl-hires.ase \
  --output pyro-gl-hires.glb \
  --name "Pyro-GL High-Resolution"
```

## Output Format

The tool generates GLB files with:

- **Geometry**: Triangulated meshes with positions and normals
- **Materials**: PBR materials with:
  - Diffuse colors from ASE materials
  - Texture maps (referenced by filename)
  - Metallic factor: 0.0, Roughness factor: 1.0 (matte finish)
- **Naming**: All glTF structures (buffers, meshes, materials, etc.) have descriptive names

## File Requirements

### ASE Files

ASE (ASCII Scene Export) files from 3D Studio Max, typically used by D2X-XL for high-resolution models. The format must include:

- `*GEOMOBJECT` blocks with mesh data
- `*MESH` blocks with vertices, faces, and normals
- `*MESH_TVERTLIST` for texture coordinates (optional)
- `*MATERIAL_LIST` for materials and textures (optional)

## Viewing the Output

GLB files can be viewed in:

- **glTF Viewer**: https://gltf-viewer.donmccurdy.com/
- **Three.js Editor**: https://threejs.org/editor/
- **Blender**: File → Import → glTF 2.0
- **Bevy**: Use `bevy::prelude::Gltf` asset loader

## Building from Source

```bash
cd d2x-rs
cargo build --package ase2glb --release
```

Binary location: `target/release/ase2glb`

## License

Licensed under either:
- MIT License
- Apache License 2.0

at your option.

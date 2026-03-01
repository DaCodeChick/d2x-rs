# Descent 3 File Formats Overview

## Introduction

Descent 3 uses the **Outrage Engine**, which introduced significantly different file formats from Descent 1 and 2. This document provides an overview of all D3 formats supported or planned for the `descent-core` library.

## Format Summary

| Format | Extension | Purpose | Status |
|--------|-----------|---------|--------|
| **HOG2** | `.hog` | Archive container | ✅ Implemented |
| **OGF** | `.ogf` | Outrage Graphics (textures) | ✅ Implemented |
| **D3L** | `.d3l` | Level geometry | 📋 Planned |
| **OOF** | `.oof` | Outrage Object (models) | 📋 Planned |
| **OSF** | `.osf` | Outrage Sound | 📋 Planned |
| **GAM** | `.gam` | Game data tables | 📋 Planned |
| **MN3** | `.mn3` | Mission definitions | 📋 Planned |
| **SAV/DSV** | `.sav`, `.dsv` | Savegames | 📋 Planned |

## Archive Format: HOG2

**See**: [HOG_FORMAT.md](HOG_FORMAT.md#hog2-format-descent-3)

HOG2 is the enhanced archive format for D3, featuring:
- Structured 12-byte header with file count and data offset
- 48-byte entries (vs 17 bytes in D1/D2)
- 36-character filenames (vs 13 characters in D1/D2)
- File flags and timestamps
- Separate directory and data sections for faster lookup

**Key Files**:
- `d3.hog` - Main game data (~100+ MB)
- `extra.hog` - Additional content
- `extra1.hog` through `extra13.hog` - Mission packs

## Texture Format: OGF

**Status**: ✅ Implemented

**See**: [OGF_FORMAT.md](OGF_FORMAT.md)

**Outrage Graphics Format** - D3's proprietary texture format with true color support.

### Key Features
- Multiple pixel formats: RGB565, RGBA4444, RGBA8888, Indexed8
- Texture sizes: 32×32, 64×64, 128×128, 256×256
- Mipmap support for level-of-detail rendering
- Animation support with frame sequences and FPS control
- 21 texture property flags (water, lava, metal, alpha, etc.)
- Conversion to RGBA8 for modern rendering APIs

### Usage Example
```rust
use descent_core::ogf::{OgfTexture, TextureFlags};

// Parse OGF texture
let data = std::fs::read("texture.ogf")?;
let texture = OgfTexture::parse(&data)?;

// Check properties
if texture.header.is_animated() {
    println!("Animated: {} frames at {} fps", 
        texture.header.num_frames, texture.header.fps);
}

// Convert to RGBA8 for rendering
let rgba = texture.to_rgba8()?;

// Access mipmaps
let mipmap1 = texture.get_mipmap(1)?;
```

## Level Format: D3L

**Status**: 📋 Planned

**Descent 3 Level** - Room-based level geometry (not segment cubes like D1/D2).

### Key Features
- **Room-based architecture** (not cubic segments)
- Portals connect rooms
- Complex geometry (arbitrary polygons, not just cubes)
- Integrated lighting, fog, wind, damage properties
- Vertex-based with face definitions
- Path nodes for AI navigation
- Trigger and event systems

### Conceptual Differences from D1/D2
| Aspect | D1/D2 (RDL/RL2) | D3 (D3L) |
|--------|------------------|----------|
| **Base Unit** | Cubic segment (8 vertices) | Room (arbitrary vertices) |
| **Walls** | Sides with texture IDs | Faces with texture references |
| **Connections** | Side connections | Portal system |
| **Geometry** | Fixed cube topology | Free-form polygons |
| **Lighting** | Per-vertex static/dynamic | Per-vertex + lightmaps |

### File Structure (Conceptual)
```
+-------------------------+
| Header                  |
|  - Version              |
|  - Room count           |
|  - Portal count         |
|  - Object count         |
+-------------------------+
| Room Definitions        |
|  - Vertices             |
|  - Faces                |
|  - Portals              |
|  - Properties (fog, etc)|
+-------------------------+
| Portal Definitions      |
|  - Connect room A to B  |
|  - Visibility info      |
+-------------------------+
| Objects                 |
|  - Players, robots, etc |
|  - Triggers, events     |
+-------------------------+
| Path Nodes (AI)         |
+-------------------------+
```

## Model Format: OOF

**Status**: 📋 Planned

**Outrage Object Format** - D3's 3D model format.

### Key Features
- Version 2300 (compatible with 1807)
- Hierarchical subobjects (for animations, turrets)
- Multiple LOD (Level of Detail) levels
- Weapon battery points
- Attach points for items
- Collision spheres and bounding boxes
- Lightmap support
- Glow effects for engines, lights
- Monitor subobjects for in-game displays

### File Structure (Conceptual)
```
+-------------------------+
| Header                  |
|  - Version (2300)       |
|  - Subobject count      |
|  - LOD count            |
+-------------------------+
| Subobject Hierarchy     |
|  - Parent/child links   |
|  - Transform matrices   |
+-------------------------+
| Geometry Data           |
|  - Vertices             |
|  - Faces                |
|  - UV coordinates       |
|  - Normals              |
+-------------------------+
| LOD Levels              |
|  - Distance thresholds  |
|  - Simplified meshes    |
+-------------------------+
| Weapon Batteries        |
|  - Position, direction  |
+-------------------------+
| Collision Data          |
|  - Bounding spheres     |
|  - Bounding boxes       |
+-------------------------+
```

## Sound Format: OSF

**Status**: 📋 Planned

**Outrage Sound Format** - D3's proprietary sound format.

### Key Features
- Compressed audio data
- 3D positional audio metadata
- Streaming support for music
- Loop points for ambient sounds
- Priority and attenuation settings

**Alternative**: Standard WAV files are also supported by D3.

## Game Data: GAM

**Status**: 📋 Planned

**Game Tables** - Replaces HAM files from D1/D2.

### Contents
- Object definitions (robots, powerups, weapons)
- Weapon properties (damage, speed, sound effects)
- Ship definitions (player ship stats)
- AI parameters
- Physics constants
- Mix of binary and text-based sections

### Structure
Unlike D1/D2's binary HAM format, GAM files use a hybrid approach:
- Text-based sections for readability/modding
- Binary sections for performance-critical data

## Scripting: OSIRIS

**Status**: 📋 Planned (Script execution not in scope; only metadata parsing)

**OSIRIS System** - DLL-based scripting for D3.

### Key Features
- C++ compiled scripts (not interpreted)
- Event-driven: collision, damage, AI notify, timer, use, etc.
- Script types:
  - **Object scripts** - Attached to game objects
  - **Trigger scripts** - Level triggers
  - **Level scripts** - Level-wide logic
  - **Mission scripts** - Mission progression

**Note**: The `descent-core` library will only parse script metadata (names, parameters, references), not execute scripts.

## Mission Format: MN3

**Status**: 📋 Planned

**Mission Definition** - D3 mission metadata and level sequences.

### Contents
- Mission name and description
- Level sequence
- Briefing text
- Custom HOG references
- Mission-specific settings

### File Structure (Text-based)
```
[MISSION]
name = "Mission Name"
author = "Author Name"
description = "Mission description"

[LEVELS]
level1 = "level1.d3l"
level2 = "level2.d3l"

[BRIEFINGS]
level1 = "briefing1.txt"
level2 = "briefing2.txt"

[RESOURCES]
hog = "missiondata.hog"
```

## Savegame Format: SAV/DSV

**Status**: 📋 Planned (Low priority)

### Key Features
- Binary format
- Version 2
- Maximum 8 save slots
- Includes: player stats, inventory, level state, mission progress

**Note**: Savegame parsing is lower priority as it's primarily used for game saves, not asset extraction.

## Cinematics: MVE

**Status**: 📋 Already supported (inherited from D1/D2)

D3 reuses the MVE format from D1/D2 for cinematics.

## Implementation Roadmap

### Phase 2A: HOG2 Support (✅ Completed)
- [x] Parse HOG2 headers
- [x] Read 48-byte entries
- [x] Extract files by name
- [x] Game version detection

### Phase 2B: OGF Texture Support (Planned)
- [ ] Parse OGF headers
- [ ] Decode pixel formats (RGB, RGBA, 4444)
- [ ] Extract mipmap levels
- [ ] Handle animated textures
- [ ] Parse texture flags
- [ ] Convert to standard image formats

### Phase 2C: D3L Level Support (Planned)
- [ ] Parse D3L headers
- [ ] Read room definitions
- [ ] Parse portal connections
- [ ] Extract geometry data
- [ ] Parse object placements
- [ ] Handle AI path nodes

### Phase 2D: OOF Model Support (Planned)
- [ ] Parse OOF headers (version 2300)
- [ ] Read subobject hierarchy
- [ ] Extract geometry (vertices, faces)
- [ ] Parse LOD levels
- [ ] Handle weapon batteries
- [ ] Extract collision data

### Phase 2E: GAM/MN3 Support (Planned)
- [ ] Parse GAM tables (hybrid binary/text)
- [ ] Read MN3 mission definitions
- [ ] Extract mission metadata

### Phase 2F: OSF Sound Support (Planned)
- [ ] Parse OSF headers
- [ ] Decode compressed audio
- [ ] Extract 3D audio metadata
- [ ] Convert to standard formats (WAV)

## Testing Strategy

### Test Data Sources
- Official Descent 3 demo (freely available)
- Community-created test files
- Synthetic test files for edge cases

### Test Coverage
- Format detection (HOG2 vs DHF)
- File extraction from HOG2
- Texture format conversion (OGF → TGA)
- Level geometry parsing (D3L)
- Model loading (OOF)

## Compatibility Notes

### D3 vs D1/D2 Architecture
- **D3 uses 3D acceleration** - No paletted graphics
- **True color textures** - 16/32-bit color (not 8-bit indexed)
- **Room-based levels** - Not cubic segments
- **DLL scripts** - Not interpreted (OSIRIS vs D2X-XL scripting)
- **Different physics** - Separate physics engine

### Modding Support
All format parsers should support:
- Reading custom content
- Validating file integrity
- Converting to standard formats for editing tools
- Documenting format details for modders

## References

### Official Sources
- Descent 3 SDK (if available)
- Outrage Engine documentation

### Community Resources
- Descent 3 community forums
- Reverse-engineered format specs
- Community modding tools

### Code References
- This implementation: `crates/descent-core/src/`
- D1/D2 reference: D2X-XL source code
- D3 reference: Community tools and documentation

---

**Document Version**: 1.0  
**Created**: 2026-02-24  
**Status**: Living document - will be updated as formats are implemented

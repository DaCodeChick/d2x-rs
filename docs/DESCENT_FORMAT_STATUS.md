# Descent Format Implementation Status

This document tracks which Descent file formats are implemented and which are needed for the level editor.

## Implemented Formats ✅

### Archives
- **DHF/HOG** - Descent 1/2 archive format (read only) - `dhf.rs`
- **HOG2** - Descent 3 archive format - `hog2.rs`
- **MVL** - Movie library archives - `mvl.rs`

### Textures
- **PIG** - Texture/bitmap data (8-bit indexed, RLE compressed) - `pig.rs`
- **OGF** - Descent 3 Outrage Graphics Format - `ogf.rs`
- **PCX** - PC Paintbrush briefing screens (8-bit indexed, 24-bit RGB) - `pcx.rs`
- **TGA** - Targa image format - `tga.rs`
- **Palette** - Color palettes (.256 files, 6-bit RGB) - `palette.rs`

### Models
- **POF** - Descent 1/2 Polygon Object Format - `pof.rs`
- **OOF** - Descent 3 Outrage Object Format - `oof.rs`
- **ASE** - 3D Studio Max ASCII Scene Export (D2X-XL high-res) - `ase.rs`

### Level Data
- **RDL/RL2** - Level geometry (segment-based) - `level.rs`
- **HAM** - Game data definitions (robots, weapons, physics) - `ham.rs`

### Other
- **MSN/MN2/MN3** - Mission files - `mission.rs`
- **PLR/PLX** - Player profiles - `player.rs`
- **HMP** - Music files - `sound.rs`
- **MVE** - Interplay movie format - `mve.rs`

## Missing Formats ❌

### Critical for Level Editor

#### HXM Archive Format (HIGH PRIORITY)
**Status**: Not implemented
**Description**: HXM (HaMster eXtended/Mod) is an archive format for custom robot/model data
**Used by**: Descent 2 Vertigo expansion, D2X-XL custom content
**Format Structure**:
```
Offset  Size  Description
------  ----  -----------
0x00    4     Signature: "HMX!" (0x21584D48)
0x04    4     Version (0x00000001)
0x08    4     Number of custom robots (n)
0x0C    ...   Robot data blocks:
              - Robot index (4 bytes)
              - Robot info struct (variable size)
0x??    ...   Extra data (custom models, textures, etc.)
```
**Purpose**: Stores custom robot definitions and model data that override HAM file entries
**Implementation needs**:
- Parser for HXM header
- Robot info struct reader (similar to HAM)
- Custom model data extractor
- Integration with HAM for override logic

**Reference**: `/home/admin/Downloads/dle-src/RobotManager.cpp` lines with `ReadHXM`/`WriteHXM`

#### BBM - Briefing Bitmap Format (MEDIUM PRIORITY)
**Status**: Not implemented
**Description**: IFF-based bitmap format used for Descent briefing screens
**Used by**: Descent 1/2 briefings (alternative to PCX)
**Format**: IFF ILBM (Interleaved Bitmap)
**Purpose**: Displays images during mission briefings
**Implementation needs**:
- IFF chunk parser
- ILBM bitmap decoder
- Palette extraction
- Convert to modern format (PNG/TGA)

**Note**: PCX is already implemented and covers most briefing images, so BBM is lower priority

### Optional/Enhancement Formats

#### DTX - D2X-XL Texture Format (LOW PRIORITY)
**Status**: Not implemented
**Description**: Custom texture format used by D2X-XL
**Purpose**: Enhanced textures for D2X-XL engine
**Implementation needs**: Research format structure

#### POG - Polygon Graphics (LOW PRIORITY)
**Status**: Not implemented  
**Description**: Custom polygon/graphics data
**Purpose**: Unknown - needs research
**Implementation needs**: Research format structure

## Converters Implemented ✅

### Texture Converters
- PIG → TGA - `converters/texture.rs`
- OGF → TGA - `converters/texture.rs`
- PCX → TGA - `converters/texture.rs` (via `pcx.rs`)
- Indexed (8-bit) → RGBA - `palette.rs`

### Model Converters
- POF → GLB/glTF - `converters/model.rs`
- OOF → GLB/glTF - `converters/model.rs`
- ASE → GLB/glTF - `converters/model.rs`

### Other Converters
- PCM → WAV - `converters/audio.rs`
- HMP → MIDI - `converters/audio.rs`
- DHF/HOG2 extraction - `converters/archive.rs`

## Qt Integration (for DLE)

### Implemented ✅
- **PCX Image Plugin** - Qt 6 image I/O plugin for loading PCX files
  - Location: `dle/src/plugins/imageformats/`
  - Allows `QImage::load("file.pcx")` in Qt applications

### Needed ❌
- **PIG/Palette → QImage loader** - Convert indexed PIG textures to Qt images
- **BBM Image Plugin** - Qt plugin for IFF/BBM format (if needed)

## Recommendations for Level Editor

### Phase 1: Core Functionality (REQUIRED)
1. ✅ DHF/HOG archive reading
2. ✅ PIG texture reading
3. ✅ Palette loading
4. ✅ POF model reading
5. ✅ HAM data reading
6. ✅ Level (RDL/RL2) reading
7. ❌ **HXM format support** - NEEDED for Vertigo levels and custom content

### Phase 2: Asset Display (IMPORTANT)
1. ✅ Convert PIG textures to displayable format (TGA/PNG)
2. ✅ Convert POF models to modern format (glTF/GLB)
3. ✅ PCX image display (via Qt plugin)
4. ⚠️ BBM image display (only if levels use it)

### Phase 3: Enhancement (OPTIONAL)
1. Descent 3 support (already has OGF, OOF, HOG2)
2. D2X-XL high-res assets (ASE models - already implemented)
3. DTX/POG custom formats (research needed)

## Action Items

### Immediate (for level editor to work):
- [ ] Implement HXM parser in `crates/descent-core/src/hxm.rs`
- [ ] Add HXM reading to DLE's `DhfArchive` or similar
- [ ] Test with Descent 2 Vertigo levels
- [ ] Create Qt integration for PIG textures (or pre-convert to PCX/TGA)

### Near-term (for better compatibility):
- [ ] Research BBM format necessity
- [ ] Implement BBM parser if needed
- [ ] Add Qt image plugin for BBM (if needed)

### Long-term (enhancements):
- [ ] HXM writing support (for custom content creation)
- [ ] Research DTX and POG formats
- [ ] Full Descent 3 level editor support

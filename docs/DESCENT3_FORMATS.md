# Descent 3 File Format Analysis

**Date**: 2026-03-01  
**Source**: Descent3-1.5.0-Source (`/home/admin/Downloads/Descent3-1.5.0-Source/`)  
**Purpose**: Document Descent 3 file formats for potential inclusion in descent-core library

---

## Summary

This document analyzes the Descent 3 source code to identify new file formats and changes from Descent 1/2 that could be added to the d2x-rs project's descent-core library.

---

## 1. OOF Format (Outrage Object Format) - Descent 3 Model Format

**Status**: ✅ Fully documented  
**Source Files**: 
- `model/polymodel.cpp` (lines 630-643, 1400+)
- `model/polymodel.h` (lines 305-307, 414)

### File Structure

**File Header**:
- Magic: `OHDR` ('RDHO' in little-endian 4-byte int)
- Version numbers:
  - `PM_COMPATIBLE_VERSION = 1807`
  - `PM_OBJFILE_VERSION = 2300`

### Chunk Types

OOF files use a chunk-based format similar to IFF. Each chunk has:
- 4-byte chunk ID (little-endian integer)
- Variable-length data

**Chunk IDs** (defined in `polymodel.cpp:630-643`):

| Chunk ID | Name | Description |
|----------|------|-------------|
| `RDHO` (0x4F524448) | ID_OHDR | POF file header (version, model info) |
| `JBOS` (0x534F424A) | ID_SOBJ | Subobject header (bsp_info structure) |
| `ATDI` (0x49544441) | ID_IDTA | Interpreter data (unused in D3) |
| `RTXT` (0x54585452) | ID_TXTR | Texture filename list |
| `FNIP` (0x504E4946) | ID_INFO | Model information (command line, etc.) |
| `DIRG` (0x47524944) | ID_GRID | Grid information |
| `TNPG` (0x474E5054) | ID_GPNT | Gun points |
| `INAR` (0x52414E49) | ID_ROT_ANIM | Angular/rotational animation data (timed) |
| `INAP` (0x5041 4E49) | ID_POS_ANIM | Positional animation data (timed) |
| `MINA` (0x414E494D) | ID_ANIM | Angular information (untimed) |
| `TABW` (0x57425441) | ID_WBS | Weapon Battery Info |
| `DNRG` (0x47524E44) | ID_GROUND | Ground plane info |
| `HCTA` (0x41544348) | ID_ATTACH | Attach points |
| `HTAN` (0x4E415448) | ID_ATTACH_NORMALS | Attach normals (uvecs) |

### Key Structures

```c
// From polymodel.h
struct poly_model {
    char name[PAGENAME_LEN];
    int n_models;                   // number of subobjects
    int model_data_size;            // size of subobject data
    uint8_t *model_data;            // pointer to subobject data
    bsp_info *submodel;             // array of submodel structures
    int n_textures;                 // number of textures
    int16_t textures[MAX_MODEL_TEXTURES];
    // ... animation, weapon battery, attachment point data
};

struct bsp_info {
    int parent;                     // parent submodel number
    vector pnt;                     // offset from parent
    vector geometric_center;        // geometric center of this subobject
    float radius;                   // radius for collision detection
    int num_faces;                  // number of faces
    int num_verts;                  // number of vertices
    // ... face, vertex, normal data
};
```

### Differences from POF (Descent 1/2)

1. **Timed animations**: OOF supports both timed (`ID_ROT_ANIM`, `ID_POS_ANIM`) and untimed (`ID_ANIM`) animations
2. **Weapon batteries**: New `ID_WBS` chunk for turret/weapon mounting points
3. **Attach points**: `ID_ATTACH` and `ID_ATTACH_NORMALS` for object attachment system
4. **Ground planes**: `ID_GROUND` chunk for landing gear/ground contact points
5. **Texture references**: Uses .OGF texture format instead of .TGA

---

## 2. D3L Format - Descent 3 Level Format

**Status**: ✅ Fully documented  
**Source Files**:
- `Descent3/LoadLevel.cpp` (lines 1319, 2658, 3452+, chunk processing)
- `Descent3/LoadLevel.h` (lines 450-484, chunk definitions)

### File Structure

**File Header**:
- Magic: `"D3LV"` (4 bytes ASCII)
- Version: uint32 (varies by level, check in code around line 3750+)

### Chunk-Based Format

Each chunk has:
- 4-byte chunk name (ASCII, e.g., "ROOM", "TERR")
- 4-byte chunk size (uint32)
- Variable-length data

**Chunk Definitions** (from `LoadLevel.h:450-484`):

| Chunk | ID | Description |
|-------|-----|-------------|
| CHUNK_TEXTURE_NAMES | "TXNM" | Texture name list for xlate table |
| CHUNK_GENERIC_NAMES | "GNNM" | Generic object names |
| CHUNK_ROBOT_NAMES | "RBNM" | Robot/enemy names (legacy, unused) |
| CHUNK_POWERUP_NAMES | "PWNM" | Powerup names (legacy, unused) |
| CHUNK_DOOR_NAMES | "DRNM" | Door names |
| CHUNK_ROOMS | "ROOM" | **Primary room data** (geometry, faces, portals) |
| CHUNK_ROOM_WIND | "RWND" | Per-room wind vectors |
| CHUNK_OBJECTS | "OBJS" | **Object instances** in level |
| CHUNK_TERRAIN | "TERR" | Terrain container chunk |
| CHUNK_TERRAIN_HEIGHT | "TERH" | Terrain heightmap data |
| CHUNK_TERRAIN_TMAPS_FLAGS | "TETM" | Terrain texture mapping + flags |
| CHUNK_TERRAIN_SKY | "TSKY" | Sky properties (fog, satellites, lighting) |
| CHUNK_TERRAIN_END | "TEND" | End of terrain data marker |
| CHUNK_TERRAIN_SOUND | "TSND" | Terrain sound bands |
| CHUNK_LEVEL_INFO | "INFO" | Level metadata (name, designer, notes, gravity) |
| CHUNK_PLAYER_STARTS | "PSTR" | Player starting positions |
| CHUNK_OBJECT_HANDLES | "OHND" | Persistent object handle mapping |
| CHUNK_TRIGGERS | "TRIG" | Trigger definitions |
| CHUNK_GAME_PATHS | "PATH" | AI pathfinding nodes |
| CHUNK_BOA | "CBOA" | Big Object Array (large object optimization) |
| CHUNK_BNODES | "NODE" | Pathfinding graph nodes |
| CHUNK_NEW_BSP | "CNBS" | Binary space partition tree |
| CHUNK_ROOM_AABB | "AABB" | Axis-aligned bounding boxes for rooms |
| CHUNK_MATCEN_DATA | "MTCN" | Matcen (robot generator) data |
| CHUNK_LEVEL_GOALS | "LVLG" | Mission objective data |
| CHUNK_LIGHTMAPS | "LMAP" | Legacy lightmap data |
| CHUNK_NEW_LIGHTMAPS | "NLMP" | **New lightmap format** (compressed) |
| CHUNK_ALIFE_DATA | "LIFE" | Ambient life system data |
| CHUNK_OVERRIDE_SOUNDS | "OSND" | Level-specific sound overrides |
| CHUNK_FFT_MOD | "FFTM" | Fast-fourier transform modifiers |
| CHUNK_EDITOR_INFO | "EDIT" | Editor-specific metadata |
| CHUNK_SCRIPT | "SCPT" | D3 Script bytecode (unused - scripts disabled) |
| CHUNK_SCRIPT_CODE | "CODE" | Script source code |

### Key Differences from RDL (Descent 1/2)

1. **Terrain support**: Dedicated terrain chunks (`TERH`, `TETM`, `TSKY`, etc.) for outdoor environments
2. **Compressed lightmaps**: `CHUNK_NEW_LIGHTMAPS` uses RLE compression for lightmap data
3. **Advanced AI**: `BOA` (Big Object Array), `BNODES` for complex pathfinding
4. **Room wind**: Per-room wind vectors for physics
5. **Level goals**: Built-in mission objective system
6. **Larger scale**: Support for much larger levels with terrain

---

## 3. MN3 Format - Descent 3 Mission File

**Status**: ✅ Partially documented  
**Source Files**:
- `Descent3/Mission.cpp` (lines 698-1867)

### Format Overview

**File Extension**: `.mn3`  
**Companion Format**: `.msn` (text-based mission script)

### How MN3 Works

MN3 files are **HOG2 archives** (see HOG2 format below) that contain:
1. Mission script file (`.msn` - text format)
2. Level files (`.d3l`)
3. Briefing files
4. Custom textures, sounds, models (optional)
5. Mission-specific assets

### Key Functions

- `mn3_Open(const char *mn3file)` - Opens MN3 archive as a virtual filesystem
- `IS_MN3_FILE(fname)` - Checks if file has `.mn3` extension
- `MN3_TO_MSN_NAME(mn3name, msnname)` - Converts `foo.mn3` → `foo.msn`

### MSN Text Format

The `.msn` file inside the MN3 archive uses keywords:

| Keyword | Description |
|---------|-------------|
| `NAME` | Mission name |
| `TYPE` | Mission type (Single, Training, Multi) |
| `LEVEL` | Level filename (`.d3l`) |
| `BRIEFING` | Briefing filename |
| `END` | Movie to play at end |
| `SHIP` | Override default ship |
| `HOARD` | Hoard mode settings |
| `KEYWORD` | Multiplayer mod identifiers |

### Example

```
NAME "Descent 3"
TYPE 0
LEVEL level1.d3l
BRIEFING Brief01.msg
END End01.mve
LEVEL level2.d3l
...
```

---

## 4. HOG2 Format - Descent 3 Archive

**Status**: ✅ VERIFIED - Already supported in descent-core  
**Source Files**:
- `cfile/hogfile.h` (lines with tHogHeader, tHogFileEntry)
- `cfile/hogfile.cpp`

### File Structure

**Header** (68 bytes):
```c
struct tHogHeader {
    char magic[4];              // "HOG2"
    uint32_t nfiles;            // number of files
    uint32_t file_data_offset;  // offset to start of file data
    char padding[56];           // reserved/padding
};
```

**File Entries** (48 bytes each):
```c
struct tHogFileEntry {
    char name[36];              // null-terminated filename
    uint32_t flags;             // extra info flags
    uint32_t len;               // file length in bytes
    uint32_t timestamp;         // Unix timestamp
};
```

### Verification

✅ **Our implementation in `crates/descent-core/src/hog2.rs` matches the D3 source exactly**

The HOG2 format is **unchanged** from what we already support.

---

## 5. OGF Format - Outrage Graphics Format

**Status**: ⚠️ Proprietary texture format - likely not needed  
**Source Files**:
- `lib/bitmap.h` (lines 34-40)
- References in `Descent3/newui.cpp`, `model/polymodel.cpp:1760`

### Format Overview

OGF is Outrage's proprietary texture format with several variants:

| Format ID | Constant | Description |
|-----------|----------|-------------|
| 121 | `OUTRAGE_4444_COMPRESSED_MIPPED` | 16-bit RGBA 4:4:4:4 with mipmaps |
| 122 | `OUTRAGE_1555_COMPRESSED_MIPPED` | 16-bit RGBA 1:5:5:5 with mipmaps |
| 123 | `OUTRAGE_NEW_COMPRESSED_MIPPED` | New compression format with mipmaps |
| 124 | `OUTRAGE_COMPRESSED_MIPPED` | Legacy compressed format with mipmaps |
| 125 | `OUTRAGE_COMPRESSED_OGF_8BIT` | 8-bit paletted format |
| 126 | `OUTRAGE_TGA_TYPE` | TGA format (Descent 3 can load standard TGAs) |
| 127 | `OUTRAGE_COMPRESSED_OGF` | Standard OGF compression |

### Analysis

**OGF files contain**:
- 16-bit RGB565 or RGBA4444/RGBA5551 pixel data
- Optional mipmaps (up to 5 levels: `NUM_MIP_LEVELS = 5`)
- Proprietary compression (implementation not fully documented)

**Decision**: ❌ **Not recommended for inclusion**

**Reasoning**:
1. OGF is superseded by standard formats (TGA, PNG)
2. Descent 3 already supports TGA natively (see `OUTRAGE_TGA_TYPE`)
3. Modern projects use DDS, KTX2, or Basis Universal
4. Minimal benefit for d2x-rs (we already support TGA)

---

## 6. Audio Formats

**Status**: ✅ Documented - No changes needed  
**Source Files**:
- `manage/soundpage.h` (lines 27-30)
- `lib/soundload.h` (not examined in detail)

### Format Overview

Descent 3 sound system uses:
- **WAV format**: Standard PCM audio (same as D1/D2)
- **Sound pages**: Managed through table files (`.gam` files)

```c
struct mngs_sound_page {
    sound_info sound_struct;
    char raw_name[PAGENAME_LEN];  // Base filename (e.g., "explosion")
};
```

### Analysis

**No new audio formats detected**. Descent 3 continues to use standard WAV files.

---

## Recommendations for descent-core

Based on this analysis, here are the recommendations for adding Descent 3 support:

### ✅ High Priority - Recommended

1. **OOF Model Format** (`crates/descent-core/src/oof.rs`)
   - **Reason**: Essential for loading Descent 3 ships, robots, and objects
   - **Effort**: Medium (similar complexity to POF)
   - **Benefit**: Enables D3 model viewing/conversion
   - **Implementation**: Chunk-based parser similar to POF

2. **D3L Level Format** (`crates/descent-core/src/d3l.rs`)
   - **Reason**: Core format for Descent 3 levels
   - **Effort**: High (complex with 30+ chunk types)
   - **Benefit**: Full D3 level support
   - **Implementation**: Chunk-based parser, terrain support optional initially

3. **MN3 Mission Format** (`crates/descent-core/src/mn3.rs`)
   - **Reason**: Simple wrapper around HOG2 + MSN text parser
   - **Effort**: Low (HOG2 already implemented, MSN is text-based)
   - **Benefit**: D3 mission management
   - **Implementation**: Use existing HOG2 parser + simple MSN keyword parser

### ⚠️ Medium Priority - Consider

4. **Terrain System** (part of D3L)
   - **Reason**: D3 levels can have outdoor terrain
   - **Effort**: Medium
   - **Benefit**: Complete D3 level rendering
   - **Implementation**: Can be stubbed initially, added later

5. **Lightmap Compression** (part of D3L)
   - **Reason**: D3 uses RLE-compressed lightmaps for efficiency
   - **Effort**: Low
   - **Benefit**: Proper lighting in D3 levels
   - **Implementation**: Simple RLE decompressor

### ❌ Not Recommended

6. **OGF Texture Format**
   - **Reason**: Proprietary, superseded by TGA/PNG
   - **Decision**: Skip - Descent 3 already supports TGA natively

7. **D3 Script Format**
   - **Reason**: Scripts were disabled in final D3 release
   - **Decision**: Skip - not used in shipping game

---

## Implementation Plan

### Phase 1: OOF Model Support (Estimated: 2-3 days)

```rust
// crates/descent-core/src/oof.rs
pub struct OofFile {
    pub version: u32,
    pub n_models: usize,
    pub submodels: Vec<BspInfo>,
    pub textures: Vec<String>,
    pub gun_points: Vec<GunPoint>,
    pub animations: AnimationData,
    // ...
}

impl OofFile {
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError>;
    pub fn write(&self, writer: &mut impl Write) -> Result<(), ParseError>;
}
```

### Phase 2: MN3 Mission Support (Estimated: 1 day)

```rust
// crates/descent-core/src/mn3.rs
pub struct Mn3Mission {
    pub name: String,
    pub mission_type: MissionType,
    pub levels: Vec<MissionLevel>,
    pub hog: Hog2File,  // Reuse existing HOG2 implementation
}

impl Mn3Mission {
    pub fn open(path: &Path) -> Result<Self, ParseError>;
    pub fn get_level(&self, index: usize) -> Option<&[u8]>;
}
```

### Phase 3: D3L Level Support (Estimated: 5-7 days)

```rust
// crates/descent-core/src/d3l.rs
pub struct D3lLevel {
    pub version: u32,
    pub level_info: LevelInfo,
    pub rooms: Vec<Room>,
    pub objects: Vec<Object>,
    pub terrain: Option<TerrainData>,
    pub lightmaps: Vec<Lightmap>,
    pub triggers: Vec<Trigger>,
    // ... 30+ chunk types
}

impl D3lLevel {
    pub fn from_bytes(data: &[u8]) -> Result<Self, ParseError>;
    // Chunk-by-chunk parsing
}
```

---

## Testing Strategy

### Test Assets Needed

1. Sample OOF files (ships, robots, powerups)
2. Sample D3L levels (with/without terrain)
3. Sample MN3 missions (main campaign, custom missions)

### Validation

- Compare parsed data against D3 source code structures
- Verify round-trip: parse → serialize → parse again
- Cross-reference with original D3 game behavior

---

## Conclusion

The Descent 3 source code reveals three main formats worth adding to descent-core:

1. **OOF** - Essential for D3 model support
2. **D3L** - Core level format with advanced features
3. **MN3** - Simple mission packaging (wraps HOG2 + text files)

The existing HOG2 implementation is verified correct. OGF texture format can be skipped since D3 supports standard TGA files.

**Total estimated effort**: 8-11 days for full D3 support.

---

## References

- Descent 3 source: `/home/admin/Downloads/Descent3-1.5.0-Source/`
- Key files:
  - `model/polymodel.cpp` - OOF format implementation
  - `model/polymodel.h` - OOF structures
  - `Descent3/LoadLevel.cpp` - D3L format implementation  
  - `Descent3/LoadLevel.h` - D3L chunk definitions
  - `Descent3/Mission.cpp` - MN3/MSN handling
  - `cfile/hogfile.h/cpp` - HOG2 format (verified)
  - `lib/bitmap.h` - OGF texture format definitions

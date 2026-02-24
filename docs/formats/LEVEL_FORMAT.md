# Descent Level File Format (RDL/RL2)

This document describes the binary format for Descent 1 (`.rdl`, `.sdl`) and Descent 2 (`.rl2`, `.sl2`) level files. These files contain the mine geometry, segments, walls, triggers, and objects that make up a game level.

## File Extensions

- **RDL**: Descent 1 Registered Level
- **SDL**: Descent 1 Shareware Level (older format)
- **RL2**: Descent 2 Level
- **SL2**: Descent 2 Shareware Level (older format)

## Overall Structure

```
┌─────────────────────────┐
│ Compiled Version (1)    │  u8: Always 0 for compiled mines
├─────────────────────────┤
│ Vertex Count            │  u16 (new) / i32 (old): Number of vertices
│ Segment Count           │  u16 (new) / i32 (old): Number of segments
├─────────────────────────┤
│ Vertices                │  Array of 3D points (fixed-point vectors)
├─────────────────────────┤
│ Segments                │  Array of segment structures
│  └─ Sides (6 per seg)   │    Each side: textures, UVs, lighting
├─────────────────────────┤
│ Segment Extras          │  Extended segment data (function, props, light)
├─────────────────────────┤
│ Vertex Lights (D2X)     │  Per-vertex colored lighting (D2X-XL only)
│ Side Lights (D2X)       │  Per-side colored lighting (D2X-XL only)
│ Texture Colors (D2X)    │  Texture color data (D2X-XL only)
└─────────────────────────┘
```

## Version Detection

```rust
// File format version (bNewFileFormat in C++)
// "New file format" = anything newer than D1 shareware
// Detected by checking level filename extension:
//   - .sdl or .SDL → old format (D1 shareware)
//   - anything else → new format
```

**Version Constants:**
- `MINE_VERSION`: 20 (current expected version)
- `COMPATIBLE_VERSION`: 16 (oldest safely loadable version)
- `COMPILED_MINE_VERSION`: 0 (for compiled mine data)

## Data Types

All multi-byte integers are **little-endian**.

| Type | Size | Description |
|------|------|-------------|
| `u8` | 1 | Unsigned 8-bit integer |
| `i8` | 1 | Signed 8-bit integer |
| `u16` | 2 | Unsigned 16-bit integer |
| `i16` | 2 | Signed 16-bit integer |
| `i32` | 4 | Signed 32-bit integer |
| `fix` | 4 | Fixed-point (16.16 format: 16-bit integer, 16-bit fraction) |
| `FixVector` | 12 | Three `fix` values (x, y, z) |

### Fixed-Point Math

Fixed-point values use 16.16 format:
```rust
// Convert to float: value / 65536.0
// Convert from float: (value * 65536.0) as i32
const I2X_MULTIPLIER: i32 = 65536; // I2X(1)
```

## Header

```rust
struct LevelHeader {
    compiled_version: u8,     // Always 0
    vertex_count: u16,        // New format: u16, old format: i32
    segment_count: u16,       // New format: u16, old format: i32
}
```

**Notes:**
- Old format (D1 shareware `.sdl`) uses `i32` for counts
- New format uses `u16` for compactness
- `compiled_version` is always 0 for all compiled mine formats

## Vertices

Array of 3D vertex positions in fixed-point format.

```rust
struct Vertex {
    x: fix,  // i32 fixed-point
    y: fix,  // i32 fixed-point
    z: fix,  // i32 fixed-point
}
```

**Size:** 12 bytes per vertex  
**Count:** Specified in header `vertex_count`

**Limits:**
- **D1:** 800 segments max → ~3200 vertices max
- **D2:** 900 segments max → ~3600 vertices max
- **D2X-XL:** 20,000 segments max

## Segments

A segment is the basic building block of Descent levels - a cube-shaped room element. Each segment has:
- 8 vertex indices (corners of the cube)
- 6 sides (faces of the cube)
- 6 child indices (connections to adjacent segments)
- Function and properties (e.g., fuel center, reactor, water, lava)

### Segment Structure

```rust
struct Segment {
    // D2X-XL only (not in original D1/D2)
    owner: u8,         // Multiplayer team owner (-1 = none)
    group: u8,         // Editor grouping (-1 = none)
    
    // Flags byte (new file format only, 0x7f for old format)
    flags: u8,         // Bit flags for children and function
    
    // Layout depends on version:
    // - D2 Shareware (v5): function, verts, children
    // - D1 (v1): children, verts, function
    // - D2+ (v2-20): children, verts
    
    children: [i16; 6],    // Indices to connected segments (-1 = none, -2 = outside)
    vertices: [u16; 8],    // Indices into vertex array
    
    // Old levels (v1-5) only
    avg_segment_light: fix,  // Average static light (old format: i16 << 4)
    
    // Wall flags (new format only, 0x3f for old)
    wall_flags: u8,    // Which sides have walls (6 bits)
    
    // 6 sides
    sides: [Side; 6],
}
```

### Segment Flags Byte

The `flags` byte encodes which optional data is present:

```
Bits 0-5: Child present flags (1 = has child, 0 = -1)
Bit 6:    Function data present (v5 only)
Bit 7:    Unused
```

**Default values:**
- New file format: Read from file
- Old file format (D1 shareware): Always `0x7f` (all flags set)

### Segment Children

Each segment has 6 children, one per side:

```rust
const SIDE_LEFT: usize = 0;
const SIDE_TOP: usize = 1;
const SIDE_RIGHT: usize = 2;
const SIDE_BOTTOM: usize = 3;
const SIDE_BACK: usize = 4;
const SIDE_FRONT: usize = 5;
```

**Child values:**
- `>= 0`: Segment index of connected neighbor
- `-1`: No connection (solid wall)
- `-2`: Connection to outside/void

### Segment Vertices

Each segment has 8 vertices forming a cube (indices into the global vertex array).

```
     Vertices:           Sides (looking from outside):
       4-------5            ┌─────┐
      /|      /|            │  1  │  TOP
     / |     / |            ├─────┤
    7-------6  |            │  5  │  FRONT (face us)
    |  0----|--1            ├─────┤
    | /     | /             │  3  │  BOTTOM
    |/      |/              └─────┘
    3-------2                 0 2 4  = LEFT, RIGHT, BACK
```

**Note:** Vertex order defines segment geometry and must be consistent with side definitions.

### Average Segment Light

For old level versions (v1-5), each segment stores average static lighting:
- **D2 Shareware & earlier:** `i16` value, shifted left by 4 bits (`<< 4`) to convert to `fix`
- **Later versions:** Stored in segment extras

### Wall Flags

The `wall_flags` byte indicates which sides have wall numbers:

```
Bit 0: Side 0 has wall
Bit 1: Side 1 has wall
Bit 2: Side 2 has wall
Bit 3: Side 3 has wall
Bit 4: Side 4 has wall
Bit 5: Side 5 has wall
```

Sides without walls have wall number set to `-1` (0xFFFF).

## Sides

Each segment has 6 sides (faces). A side contains textures, UV coordinates, and lighting.

```rust
struct Side {
    // Wall number (only if wall_flags bit is set)
    wall_num: u16,     // Index into walls array, or 0xFFFF if no wall
                       // Old levels (v < 13): u8 instead of u16
    
    // Corner indices (v25+): which of segment's 8 vertices form this side
    corners: [u8; 4],  // 0xFF = unused (for non-quad sides)
    
    // Textures (only if side has wall OR child == -1)
    base_texture: u16,    // Bits 0-14: texture index, Bit 15: has overlay (new format)
    overlay_texture: u16, // Bits 0-13: texture index, Bits 14-15: orientation
    
    // UV and lighting for 4 corners
    uvls: [UVL; 4],
}
```

### Side Type

Sides can be different shapes:

```rust
enum SideType {
    Quad = 0,    // 4 vertices (default)
    Tri02 = 1,   // Triangle using vertices 0, 2 (+ implicit)
    Tri13 = 2,   // Triangle using vertices 1, 3 (+ implicit)
}
```

**Note:** Triangle sides have one corner marked as `0xFF` (unused).

### Texture Encoding

**Base Texture** (new file format):
```
Bits 0-14: Texture index (0-32767)
Bit 15:    Overlay present flag (1 = has overlay, 0 = no overlay)
```

**Overlay Texture** (if present):
```
Bits 0-13: Texture index (0-16383)
Bits 14-15: Orientation (0-3, 90° rotations)
```

**D1 Texture Conversion:**
- D1 levels (version ≤ 1) use different texture indices
- Must convert using `ConvertD1Texture()` lookup table
- Applies to both base and overlay textures

**Texture ID Masking:**
```rust
const TEXTURE_ID_MASK: u16 = 0x3FFF;  // Mask for texture number (14 bits)
```

### UVL (Texture Coordinates and Lighting)

Each side corner has UV coordinates and a lighting value:

```rust
struct UVL {
    u: fix,  // Texture U coordinate (horizontal)
    v: fix,  // Texture V coordinate (vertical)
    l: fix,  // Lighting/brightness value
}
```

**Storage format in file:**
```rust
// Each field stored as i16, then scaled:
u_file: i16  →  u = (u_file as i32) << 5   // Multiply by 32
v_file: i16  →  v = (v_file as i32) << 5   // Multiply by 32
l_file: u16  →  l = (l_file as i32) << 1   // Multiply by 2
```

**Size:** 6 bytes per UVL (2 + 2 + 2)  
**Count:** 4 per side (even if triangulated, 4 UVLs are stored)

**Purpose:**
- **UV:** Map texture pixels to 3D coordinates
- **L:** Static lightmap intensity (0.0 = black, higher = brighter)

## Segment Extras

After all segments are parsed, extended data is stored for each segment:

```rust
struct SegmentExtras {
    function: u8,              // Segment function type
    obj_producer: u8 | u16,    // Producer/generator index (v24+: u16, else: u8)
    value: i8 | i16,           // Function-specific value (v24+: i16, else: i8)
    flags: u8,                 // Bitfield for various properties
    
    // v21+ only (else: derived from function)
    props: u8,                 // Property flags (water, lava, etc.)
    damage: [i16; 2],          // Damage amounts (convert to fix with I2X())
    
    avg_seg_light: fix,        // Average segment lighting
}
```

### Segment Functions

Segment function types define special purposes:

```rust
const SEGMENT_FUNC_NONE: u8 = 0;
const SEGMENT_FUNC_FUELCENTER: u8 = 1;     // Energy/shield recharge
const SEGMENT_FUNC_REPAIRCENTER: u8 = 2;   // Robot repair center
const SEGMENT_FUNC_REACTOR: u8 = 3;        // Reactor (boss)
const SEGMENT_FUNC_ROBOTMAKER: u8 = 4;     // Robot spawn point
const SEGMENT_FUNC_GOAL_BLUE: u8 = 5;      // CTF blue goal
const SEGMENT_FUNC_GOAL_RED: u8 = 6;       // CTF red goal
const SEGMENT_FUNC_TEAM_BLUE: u8 = 9;      // Blue team base
const SEGMENT_FUNC_TEAM_RED: u8 = 10;      // Red team base
const SEGMENT_FUNC_SPEEDBOOST: u8 = 11;    // Speed boost zone
const SEGMENT_FUNC_SKYBOX: u8 = 14;        // Skybox segment
const SEGMENT_FUNC_EQUIPMAKER: u8 = 15;    // Equipment maker
```

**Old Segment Types (v1-20):**

Legacy levels store a "type" field that maps to function + properties:

```rust
const SEGMENT_TYPE_NONE: u8 = 0;
const SEGMENT_TYPE_PRODUCER: u8 = 1;       // → FUNC_FUELCENTER
const SEGMENT_TYPE_REPAIRCEN: u8 = 2;      // → FUNC_REPAIRCENTER
const SEGMENT_TYPE_CONTROLCEN: u8 = 3;     // → FUNC_REACTOR
const SEGMENT_TYPE_ROBOTMAKER: u8 = 4;     // → FUNC_ROBOTMAKER
const SEGMENT_TYPE_GOAL_BLUE: u8 = 5;      // → FUNC_GOAL_BLUE
const SEGMENT_TYPE_GOAL_RED: u8 = 6;       // → FUNC_GOAL_RED
const SEGMENT_TYPE_WATER: u8 = 7;          // → PROP_WATER
const SEGMENT_TYPE_LAVA: u8 = 8;           // → PROP_LAVA
```

### Segment Properties

Property flags (v21+):

```rust
const SEGMENT_PROP_NONE: u8 = 0x00;
const SEGMENT_PROP_WATER: u8 = 0x01;       // Water physics
const SEGMENT_PROP_LAVA: u8 = 0x02;        // Lava (damage)
const SEGMENT_PROP_BLOCKED: u8 = 0x04;     // Cannot enter
const SEGMENT_PROP_NODAMAGE: u8 = 0x08;    // Invulnerable zone
const SEGMENT_PROP_OUTDOORS: u8 = 0x10;    // Outdoor area
const SEGMENT_PROP_LIGHT_FOG: u8 = 0x20;   // Light fog effect
const SEGMENT_PROP_DENSE_FOG: u8 = 0x40;   // Dense fog effect
```

**Note:** For old levels (v ≤ 20), properties are derived from the function/type field.

### Upgrading Old Segments

For levels v20 and earlier, the parser must "upgrade" segment data:

```rust
fn upgrade_segment(function: u8) -> (u8, u8) {
    let new_function = match function {
        1 => SEGMENT_FUNC_FUELCENTER,
        2 => SEGMENT_FUNC_REPAIRCENTER,
        3 => SEGMENT_FUNC_REACTOR,
        4 => SEGMENT_FUNC_ROBOTMAKER,
        5 => SEGMENT_FUNC_GOAL_BLUE,
        6 => SEGMENT_FUNC_GOAL_RED,
        7 | 8 => SEGMENT_FUNC_NONE,  // Water/lava → NONE (props set instead)
        _ => SEGMENT_FUNC_NONE,
    };
    
    let props = match function {
        7 => SEGMENT_PROP_WATER,
        8 => SEGMENT_PROP_LAVA,
        12 => SEGMENT_PROP_BLOCKED,
        13 => SEGMENT_PROP_NODAMAGE,
        14 => SEGMENT_PROP_BLOCKED,  // Skybox
        16 => SEGMENT_PROP_OUTDOORS,
        _ => SEGMENT_PROP_NONE,
    };
    
    (new_function, props)
}
```

## D2X-XL Extended Data

D2X-XL levels (detected by presence in file) have additional data:

### Vertex Lights

Per-vertex colored ambient lighting (if `gameStates.app.bD2XLevel`):

```rust
struct VertexLight {
    index: u8,         // Vertex index (may be 0 for default)
    color: RGBColor,   // RGB color values
}

// Color format depends on level version:
// v14 and earlier: double precision (3 x f64 = 24 bytes)
// v15+: fixed-point (3 x i32 = 12 bytes)
```

**Color encoding (v15+):**
```rust
// Read as i32, convert to float:
let red = (color_int as f32) / (i32::MAX as f32);
let green = (color_int as f32) / (i32::MAX as f32);
let blue = (color_int as f32) / (i32::MAX as f32);
```

**Count:** One per vertex (`vertex_count`)

### Side Lights

Per-side colored lighting (if `gameStates.app.bD2XLevel`):

```rust
struct SideLight {
    index: u8,
    color: RGBColor,
}
```

**Count:** `segment_count × 6` (one per side)  
**Format:** Same as vertex lights (depends on level version)

### Texture Colors

Default texture colors (if `gameStates.app.bD2XLevel`):

```rust
struct TextureColor {
    index: u8,
    color: RGBColor,
}
```

**Count:** `MAX_WALL_TEXTURES` (typically 800-900)  
**Format:** Same as vertex lights

## Version History

| Version | Description |
|---------|-------------|
| 1 | Descent 1 levels (.rdl) |
| 5 | Descent 2 Shareware |
| 13 | Wall numbers changed from u8 to u16 |
| 14 | Vertex light colors: f64 → i32 |
| 15 | Side light colors: f64 → i32 |
| 16 | Oldest compatible version |
| 20 | Last version using old segment type/function system |
| 21+ | New function/props separation |
| 24 | obj_producer and value: u8 → u16/i16 |
| 25+ | Side corners explicitly stored |

## Parsing Algorithm

```rust
// 1. Read header
let compiled_version = reader.read_u8()?;  // Always 0
let vertex_count = if new_file_format {
    reader.read_u16()?
} else {
    reader.read_i32()? as u16
};
let segment_count = if new_file_format {
    reader.read_u16()?
} else {
    reader.read_i32()? as u16
};

// 2. Read vertices
let mut vertices = Vec::with_capacity(vertex_count);
for _ in 0..vertex_count {
    vertices.push(read_vertex(&mut reader)?);
}

// 3. Read segments
let mut segments = Vec::with_capacity(segment_count);
for _ in 0..segment_count {
    segments.push(read_segment(&mut reader, level_version)?);
}

// 4. Read segment extras
for segment in &mut segments {
    read_segment_extras(&mut reader, segment, level_version)?;
}

// 5. Read D2X-XL extended data (if present)
if is_d2xl_level {
    read_vertex_lights(&mut reader, vertex_count, level_version)?;
    read_side_lights(&mut reader, segment_count * 6, level_version)?;
    read_texture_colors(&mut reader, MAX_WALL_TEXTURES, level_version)?;
}
```

## Important Notes

1. **Endianness:** All multi-byte values are **little-endian**
2. **Fixed-point:** All spatial coordinates and lighting use 16.16 fixed-point
3. **Version detection:** File extension determines old vs. new format
4. **D1 compatibility:** Must convert D1 texture indices to D2 indices
5. **Validation:** Check that vertex indices are within bounds
6. **Special values:**
   - Wall number `-1` (0xFFFF): No wall
   - Child index `-1`: Solid wall (no connection)
   - Child index `-2`: Outside/void
   - Corner index `0xFF`: Unused vertex (triangulated side)

## File Size Estimation

Approximate sizes for a level with N segments and V vertices:

```
Base size: 5 bytes (header)
Vertices:  V × 12 bytes
Segments:  N × ~200 bytes (varies by version)
  - Segment data: ~50 bytes
  - 6 sides × ~25 bytes each
  - Extras: ~10 bytes

D2X-XL extra: V × 13 bytes (vertex lights)
            + N × 6 × 13 bytes (side lights)
            + 800 × 13 bytes (texture colors)
```

**Example:** 400 segments, 1600 vertices
- Base: ~90 KB
- With D2X-XL: ~110 KB

## References

- **D2X-XL Source:** `loadgeometry.cpp`, `segment.cpp`, `side.cpp`
- **Headers:** `loadgeometry.h`, `segment.h`
- **Structures:** `mfi` (mine file info), `CSegment`, `CSide`, `tUVL`

## See Also

- [HOG Format](HOG_FORMAT.md) - Archive container for level files
- [HAM Format](HAM_FORMAT.md) - Game data (textures, robots, weapons)
- [PIG Format](PIG_FORMAT.md) - Texture bitmap data

# POF Format Specification

**POF (Polygon Object File)** is the 3D model format used in Descent 1 and Descent 2 for ships, robots, powerups, weapons, and other game objects. The format uses an opcode-based interpreter system similar to bytecode, where sequential opcodes define geometry, materials, and hierarchical structure.

## Status

**✅ IMPLEMENTED** - Parser complete in `src/pof.rs` with comprehensive tests.

## Format Overview

### Key Characteristics

- **Opcode-based**: Sequential 16-bit opcodes define model data
- **Little-endian**: All multi-byte values are little-endian
- **Fixed-point coordinates**: Uses 16.16 fixed-point (i32) for precision
- **Hierarchical**: Supports nested submodels for animation
- **BSP-sorted**: Built-in BSP tree for proper rendering order
- **No header**: Begins immediately with opcodes

### Coordinate System

POF uses fixed-point arithmetic for coordinates:
- **Format**: 16.16 fixed-point (32-bit signed integer)
- **Conversion to float**: `value as f32 / 65536.0`
- **Example**: `65536` = 1.0, `32768` = 0.5, `-65536` = -1.0

## File Structure

```
POF File:
├─ Opcode stream (variable length)
│  ├─ OP_DEFPOINTS (define vertices)
│  ├─ OP_FLATPOLY (solid color polygon)
│  ├─ OP_TMAPPOLY (textured polygon)
│  ├─ OP_SORTNORM (BSP sorting node)
│  ├─ OP_RODBM (cylindrical billboard)
│  ├─ OP_SUBCALL (submodel reference)
│  ├─ OP_DEFP_START (define vertices at index)
│  ├─ OP_GLOW (glow point)
│  └─ OP_EOF (end of stream)
```

## Opcodes

### Opcode 0: OP_EOF

**Purpose**: End of model data / opcode stream

**Size**: 2 bytes

**Structure**:
```
Offset  Size  Type   Description
------  ----  -----  -----------
0x00    2     u16    Opcode (0)
```

**Notes**:
- Terminates opcode parsing
- Multiple EOF opcodes can exist in BSP branches
- Parser stops when EOF is encountered

---

### Opcode 1: OP_DEFPOINTS

**Purpose**: Define vertex positions

**Size**: 4 + n×12 bytes (where n = vertex count)

**Structure**:
```
Offset  Size  Type        Description
------  ----  -----       -----------
0x00    2     u16         Opcode (1)
0x02    2     u16         Vertex count (n)
0x04    12    FixVector   Vertex 0 (x, y, z as i32 each)
0x10    12    FixVector   Vertex 1
...     ...   ...         ...
```

**FixVector Structure** (12 bytes):
```
Offset  Size  Type  Description
------  ----  ----  -----------
0x00    4     i32   X coordinate (16.16 fixed-point)
0x04    4     i32   Y coordinate (16.16 fixed-point)
0x08    4     i32   Z coordinate (16.16 fixed-point)
```

**Example**:
```rust
// Define 2 vertices: (1.0, 2.0, 3.0) and (4.0, 5.0, 6.0)
let data = vec![
    0x01, 0x00,                    // OP_DEFPOINTS
    0x02, 0x00,                    // count = 2
    // Vertex 0: (1.0, 2.0, 3.0) in fixed-point
    0x00, 0x00, 0x01, 0x00,        // x = 65536 = 1.0
    0x00, 0x00, 0x02, 0x00,        // y = 131072 = 2.0
    0x00, 0x00, 0x03, 0x00,        // z = 196608 = 3.0
    // Vertex 1: (4.0, 5.0, 6.0)
    0x00, 0x00, 0x04, 0x00,        // x = 262144 = 4.0
    0x00, 0x00, 0x05, 0x00,        // y = 327680 = 5.0
    0x00, 0x00, 0x06, 0x00,        // z = 393216 = 6.0
];
```

---

### Opcode 2: OP_FLATPOLY

**Purpose**: Flat-shaded polygon (solid color)

**Size**: 30 + ((n|1))×2 bytes (where n = vertex count)

**Structure**:
```
Offset  Size  Type        Description
------  ----  -----       -----------
0x00    2     u16         Opcode (2)
0x02    2     u16         Vertex count (n)
0x04    12    FixVector   Polygon center point
0x10    12    FixVector   Normal vector
0x1C    2     u16         Color index (palette index)
0x1E    n×2   u16[]       Vertex indices
(pad)   0-2   u16         Padding if n is even
```

**Vertex Padding**:
- If vertex count is even, add 2 bytes of padding
- Formula: `(n | 1)` ensures odd alignment
- Example: 3 vertices = no padding, 4 vertices = 2 bytes padding

**Example**:
```rust
// Triangle: vertices 0, 1, 2 with color 15
let data = vec![
    0x02, 0x00,                    // OP_FLATPOLY
    0x03, 0x00,                    // nverts = 3
    // Center: (0, 0, 0)
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    // Normal: (0, 0, 1.0)
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x01, 0x00,        // z = 65536 = 1.0
    0x0F, 0x00,                    // color = 15
    0x00, 0x00,                    // vertex 0
    0x01, 0x00,                    // vertex 1
    0x02, 0x00,                    // vertex 2
    // No padding (3 is odd)
];
```

---

### Opcode 3: OP_TMAPPOLY

**Purpose**: Texture-mapped polygon with UV coordinates

**Size**: 30 + ((n|1))×2 + n×12 bytes

**Structure**:
```
Offset  Size  Type        Description
------  ----  -----       -----------
0x00    2     u16         Opcode (3)
0x02    2     u16         Vertex count (n)
0x04    12    FixVector   Polygon center point
0x10    12    FixVector   Normal vector
0x1C    2     u16         Texture ID
0x1E    n×2   u16[]       Vertex indices
(pad)   0-2   u16         Padding if n is even
(uv)    n×12  UVL[]       UV coords + light for each vertex
```

**UVL Structure** (12 bytes per vertex):
```
Offset  Size  Type  Description
------  ----  ----  -----------
0x00    4     i32   U coordinate (16.16 fixed-point, 0.0-1.0)
0x04    4     i32   V coordinate (16.16 fixed-point, 0.0-1.0)
0x08    4     i32   L light value (16.16 fixed-point, brightness)
```

**Notes**:
- UV coordinates are normalized (0.0 to 1.0)
- L value controls vertex lighting/brightness
- Texture ID references texture in PIG file (D1/D2)

**Example**:
```rust
// Textured triangle with vertices 0, 1, 2 and texture 5
// UVs: (0,0), (1,0), (0.5,1)
let data = vec![
    0x03, 0x00,                    // OP_TMAPPOLY
    0x03, 0x00,                    // nverts = 3
    // Center and normal (simplified)
    // ... 24 bytes ...
    0x05, 0x00,                    // texture_id = 5
    0x00, 0x00,                    // vertex 0
    0x01, 0x00,                    // vertex 1
    0x02, 0x00,                    // vertex 2
    // UVL for vertex 0: u=0, v=0, l=1.0
    0x00, 0x00, 0x00, 0x00,        // u = 0
    0x00, 0x00, 0x00, 0x00,        // v = 0
    0x00, 0x00, 0x01, 0x00,        // l = 1.0
    // UVL for vertex 1: u=1.0, v=0, l=1.0
    0x00, 0x00, 0x01, 0x00,        // u = 1.0
    0x00, 0x00, 0x00, 0x00,        // v = 0
    0x00, 0x00, 0x01, 0x00,        // l = 1.0
    // UVL for vertex 2: u=0.5, v=1.0, l=1.0
    0x00, 0x80, 0x00, 0x00,        // u = 0.5 (32768)
    0x00, 0x00, 0x01, 0x00,        // v = 1.0
    0x00, 0x00, 0x01, 0x00,        // l = 1.0
];
```

---

### Opcode 4: OP_SORTNORM

**Purpose**: BSP sorting node for proper rendering order

**Size**: 32 bytes

**Structure**:
```
Offset  Size  Type        Description
------  ----  -----       -----------
0x00    2     u16         Opcode (4)
0x02    2     u16         Unused
0x04    12    FixVector   BSP split point
0x10    12    FixVector   BSP split plane normal
0x1C    2     u16         Front side offset (file offset to opcodes)
0x1E    2     u16         Back side offset (file offset to opcodes)
```

**BSP Tree Structure**:
- Creates binary space partition for front-to-back sorting
- Front/back offsets point to child opcode streams
- Recursively parse front and back branches
- Offsets of 0 indicate no child node

**Rendering Algorithm**:
```
1. Calculate dot product of camera position with BSP plane
2. If positive:
   - Render back side first
   - Render this node
   - Render front side
3. If negative:
   - Render front side first
   - Render this node
   - Render back side
```

**Parser Behavior**:
```rust
// Pseudo-code for parsing SORTNORM
fn parse_sortnorm() {
    let front_offset = read_u16();
    let back_offset = read_u16();
    
    let saved_pos = current_position();
    
    if front_offset > 0 {
        seek(front_offset);
        parse_opcodes(); // Recursively parse front side
    }
    
    if back_offset > 0 {
        seek(back_offset);
        parse_opcodes(); // Recursively parse back side
    }
    
    seek(saved_pos); // Restore position
}
```

---

### Opcode 5: OP_RODBM

**Purpose**: Rod bitmap (cylindrical billboard sprite)

**Size**: 36 bytes

**Structure**:
```
Offset  Size  Type        Description
------  ----  -----       -----------
0x00    2     u16         Opcode (5)
0x02    2     u16         Texture ID
0x04    12    FixVector   Bottom point (x, y, z)
0x10    4     i32         Bottom width (16.16 fixed-point)
0x14    12    FixVector   Top point (x, y, z)
0x20    4     i32         Top width (16.16 fixed-point)
```

**Usage**:
- Cylindrical billboards for exhaust trails, laser beams, etc.
- Always faces camera
- Tapers from bottom width to top width
- Texture wraps around cylinder

**Example Use Cases**:
- Engine exhaust plumes
- Weapon beam effects
- Antenna or thin structural elements

---

### Opcode 6: OP_SUBCALL

**Purpose**: Submodel reference (hierarchical geometry)

**Size**: 20 bytes

**Structure**:
```
Offset  Size  Type        Description
------  ----  -----       -----------
0x00    2     u16         Opcode (6)
0x02    2     u16         Submodel number
0x04    12    FixVector   Offset/pivot point
0x10    2     u16         Data offset (file offset to submodel opcodes)
```

**Hierarchy**:
- Submodels can contain other submodels (recursive)
- Offset is the pivot point for animation/rotation
- Data offset points to opcode stream for submodel geometry
- Enables articulated models (robot arms, turrets, landing gear)

**Animation**:
- Submodels can rotate around their pivot point
- Engine tracks submodel transformations
- Enables complex multi-part models

**Example**:
```
Ship Model:
├─ Main hull (root opcodes)
├─ Left wing (submodel 0, offset = (-5, 0, 0))
├─ Right wing (submodel 1, offset = (5, 0, 0))
├─ Turret (submodel 2, offset = (0, 2, 0))
│  └─ Gun barrel (submodel 3, nested inside turret)
└─ Landing gear (submodel 4, offset = (0, -3, 0))
```

---

### Opcode 7: OP_DEFP_START

**Purpose**: Define vertices starting at a specific index

**Size**: 8 + n×12 bytes

**Structure**:
```
Offset  Size  Type        Description
------  ----  -----       -----------
0x00    2     u16         Opcode (7)
0x02    2     u16         Vertex count (n)
0x04    2     u16         Starting index
0x06    2     u16         Unused
0x08    12    FixVector   Vertex 0
0x14    12    FixVector   Vertex 1
...     ...   ...         ...
```

**Purpose**:
- Allows sparse vertex arrays
- Useful for submodels that reference parent vertices
- Can overwrite existing vertices at specific indices

**Difference from OP_DEFPOINTS**:
- `OP_DEFPOINTS`: Appends vertices to array
- `OP_DEFP_START`: Places vertices at specific indices

**Example**:
```rust
// Define 2 vertices starting at index 5
// Vertices array will have indices [0, 1, 2, 3, 4, 5, 6, ...]
let data = vec![
    0x07, 0x00,                    // OP_DEFP_START
    0x02, 0x00,                    // count = 2
    0x05, 0x00,                    // start = 5
    0x00, 0x00,                    // unused
    // Vertex at index 5: (1.0, 2.0, 3.0)
    0x00, 0x00, 0x01, 0x00,        // x = 1.0
    0x00, 0x00, 0x02, 0x00,        // y = 2.0
    0x00, 0x00, 0x03, 0x00,        // z = 3.0
    // Vertex at index 6: (4.0, 5.0, 6.0)
    0x00, 0x00, 0x04, 0x00,        // x = 4.0
    0x00, 0x00, 0x05, 0x00,        // y = 5.0
    0x00, 0x00, 0x06, 0x00,        // z = 6.0
];
```

---

### Opcode 8: OP_GLOW

**Purpose**: Glow point definition (engine glow, weapon muzzle flash)

**Size**: 4 bytes

**Structure**:
```
Offset  Size  Type   Description
------  ----  -----  -----------
0x00    2     u16    Opcode (8)
0x02    2     u16    Glow number/ID
```

**Usage**:
- Marks points for special effects
- Engine glows (ship thrusters)
- Weapon muzzle flashes
- Running lights
- Glow position determined by previous geometry context

**Glow Numbers**:
- Multiple glow points can have same ID
- Engine uses glow number to determine effect type
- Animation and intensity controlled by game code

---

## Opcode Summary Table

| Opcode | Name          | Size (bytes)                | Description                          |
|--------|---------------|-----------------------------|--------------------------------------|
| 0      | OP_EOF        | 2                           | End of opcode stream                 |
| 1      | OP_DEFPOINTS  | 4 + n×12                    | Define n vertices                    |
| 2      | OP_FLATPOLY   | 30 + ((n\|1))×2             | Flat-shaded polygon (n vertices)     |
| 3      | OP_TMAPPOLY   | 30 + ((n\|1))×2 + n×12      | Textured polygon with UVs            |
| 4      | OP_SORTNORM   | 32                          | BSP sorting node                     |
| 5      | OP_RODBM      | 36                          | Rod bitmap (cylindrical billboard)   |
| 6      | OP_SUBCALL    | 20                          | Submodel reference                   |
| 7      | OP_DEFP_START | 8 + n×12                    | Define vertices at specific index    |
| 8      | OP_GLOW       | 4                           | Glow point effect                    |

## Type Definitions

### FixVector (12 bytes)

```
struct FixVector {
    x: i32,  // 16.16 fixed-point
    y: i32,  // 16.16 fixed-point
    z: i32,  // 16.16 fixed-point
}
```

**Conversion**:
```rust
fn to_float(fix: i32) -> f32 {
    fix as f32 / 65536.0
}
```

### UVL (12 bytes)

```
struct UVL {
    u: i32,  // 16.16 fixed-point (0.0-1.0 texture coord)
    v: i32,  // 16.16 fixed-point (0.0-1.0 texture coord)
    l: i32,  // 16.16 fixed-point (light/brightness)
}
```

## Parsing Algorithm

### Main Loop

```rust
loop {
    let opcode = read_u16();
    match opcode {
        0 => break,                    // OP_EOF
        1 => parse_defpoints(),
        2 => parse_flatpoly(),
        3 => parse_tmappoly(),
        4 => parse_sortnorm(),         // Recursive
        5 => parse_rodbm(),
        6 => parse_subcall(),          // Recursive
        7 => parse_defpstart(),
        8 => parse_glow(),
        _ => return error,
    }
}
```

### Recursive Opcodes

Two opcodes require recursive parsing:
- **OP_SORTNORM**: Recursively parse front and back branches
- **OP_SUBCALL**: Recursively parse submodel data

**Important**: Save and restore file position when handling recursion.

## Example POF File Structure

```
Typical Ship Model:

1. OP_DEFPOINTS (100 vertices for main hull)
2. OP_SORTNORM (BSP root node)
   ├─ Front branch:
   │  ├─ OP_TMAPPOLY (cockpit window)
   │  ├─ OP_TMAPPOLY (nose panel)
   │  └─ OP_EOF
   └─ Back branch:
      ├─ OP_FLATPOLY (engine housing)
      ├─ OP_RODBM (exhaust trail)
      ├─ OP_GLOW (engine glow)
      └─ OP_EOF
3. OP_SUBCALL (left wing submodel)
   └─ OP_DEFPOINTS (wing vertices)
   └─ OP_TMAPPOLY (wing surface)
   └─ OP_EOF
4. OP_SUBCALL (right wing submodel)
   └─ ... similar to left wing
5. OP_EOF (main stream end)
```

## Implementation Notes

### Fixed-Point Arithmetic

- All coordinates use 16.16 fixed-point (i32)
- Upper 16 bits = integer part
- Lower 16 bits = fractional part
- **Convert to float**: `value as f32 / 65536.0`
- **Convert from float**: `(value * 65536.0) as i32`

### Vertex Padding

Flat and textured polygons pad vertex indices to odd alignment:
- If vertex count is even, add 2 bytes of padding after vertex indices
- Formula: `(count | 1)` ensures odd count
- This is a historical quirk for memory alignment

### BSP Tree Rendering

The BSP tree (OP_SORTNORM) must be traversed at render time:
1. Calculate camera's position relative to split plane
2. Render far side first (back-to-front)
3. Then render near side
4. Ensures proper alpha blending and depth sorting

### Submodel Animation

Submodels (OP_SUBCALL) enable articulated models:
- Each submodel has a pivot point (offset)
- Engine applies rotation/translation to submodel
- Nested submodels create kinematic chains
- Example: turret → barrel → muzzle flash

### Memory Layout

POF files have no fixed header - they start immediately with opcodes. The interpreter pattern means:
- No random access to model parts
- Must parse sequentially
- Recursive structures require stack or recursion
- File size is determined by OP_EOF positions

## Related Formats

- **PIG**: Texture bitmaps referenced by OP_TMAPPOLY texture IDs
- **HAM**: Game data defining which POF to use for each object
- **RDL/RL2**: Level files that place POF models in 3D space

## Tools & References

- **D2X-XL Source**: `/tmp/d2x-xl/3d/interp.cpp` - Reference interpreter
- **Implementation**: `crates/d2x-assets/src/pof.rs` - Rust parser
- **Tests**: 11 comprehensive unit tests covering all opcodes

## Limitations & Future Work

### Current Implementation

- ✅ All 9 opcodes parsed
- ✅ Fixed-point conversion
- ✅ Recursive SORTNORM and SUBCALL
- ✅ Comprehensive unit tests
- ✅ Zero-copy parsing where possible

### Not Yet Implemented

- ⏳ Full POF model headers (if they exist in actual files)
- ⏳ High-level model API (bounding boxes, mass, etc.)
- ⏳ POF renderer (this is a parser library only)
- ⏳ Model validation (duplicate vertices, degenerate polygons)

### Known Issues

- Opcode stream must be well-formed (no validation of offsets)
- No bounds checking on vertex indices in polygons
- BSP tree correctness not verified (just parsed)

## Version History

- **v0.1.0 (2026-02-24)**: Initial POF parser implementation
  - All 9 opcodes supported
  - 11 unit tests passing
  - Zero clippy warnings

---

**Format Source**: D2X-XL source code (`/tmp/d2x-xl/3d/interp.cpp`, `/tmp/d2x-xl/include/interp.h`)  
**Reverse Engineered By**: Analysis of D2X-XL interpreter implementation  
**Implemented By**: d2x-rs project (Rust)  
**Documentation Date**: February 24, 2026

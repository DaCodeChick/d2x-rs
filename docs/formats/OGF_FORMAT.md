# OGF Format - Outrage Graphics Format

The OGF (Outrage Graphics Format) is the texture format used in Descent 3. Unlike Descent 1/2's indexed 8-bit textures (PIG format), OGF supports true color textures with various pixel formats, mipmaps, and animation.

## Format Overview

- **Used in:** Descent 3
- **Purpose:** Texture storage with true color support
- **Features:**
  - Multiple pixel formats (RGB565, RGBA4444, RGBA8888, Indexed8)
  - Mipmap support for level-of-detail rendering
  - Animation support with frame sequences
  - Rich texture properties (water, metal, lava, etc.)
  - Texture dimensions: 32×32, 64×64, 128×128, 256×256

## File Structure

```
+-------------------------+
| Header (32 bytes)       |
|  - Version (4)          |
|  - Width, Height (4)    |
|  - Format (1)           |
|  - Flags (4)            |
|  - Num Mipmaps (1)      |
|  - Num Frames (2)       |
|  - FPS (4)              |
|  - Padding (12)         |
+-------------------------+
| Pixel Data              |
|  - Frame 0:             |
|    - Mipmap 0 (base)    |
|    - Mipmap 1...N       |
|  - Frame 1...N          |
+-------------------------+
```

## Header Format (32 bytes)

| Offset | Size | Type   | Name        | Description                           |
|--------|------|--------|-------------|---------------------------------------|
| 0      | 4    | u32    | version     | Format version (typically 1)          |
| 4      | 2    | u16    | width       | Texture width in pixels               |
| 6      | 2    | u16    | height      | Texture height in pixels              |
| 8      | 1    | u8     | format      | Pixel format (see below)              |
| 9      | 4    | u32    | flags       | Texture property flags (see below)    |
| 13     | 1    | u8     | num_mipmaps | Number of mipmap levels (0 = none)    |
| 14     | 2    | u16    | num_frames  | Number of animation frames (1 = none) |
| 16     | 4    | f32    | fps         | Animation frames per second           |
| 20     | 12   | -      | padding     | Padding to align to 32 bytes          |

All multi-byte values are stored in **little-endian** format.

## Pixel Formats

The `format` field specifies how pixel data is encoded:

| Value | Name       | Bytes/Pixel | Description                                      |
|-------|------------|-------------|--------------------------------------------------|
| 0     | RGB565     | 2           | 5-bit red, 6-bit green, 5-bit blue, no alpha    |
| 1     | RGBA4444   | 2           | 4-bit each for red, green, blue, and alpha      |
| 2     | RGBA8888   | 4           | 8-bit each for red, green, blue, and alpha      |
| 3     | Indexed8   | 1           | 8-bit palette index (requires external palette) |

### RGB565 Encoding

16-bit pixel format with bit layout:
```
Bits: RRRR RGGG GGGB BBBB
```

To convert to 8-bit per channel:
```rust
let r5 = (pixel >> 11) & 0x1F;
let g6 = (pixel >> 5) & 0x3F;
let b5 = pixel & 0x1F;

// Scale to 8-bit by replicating high bits
let r8 = (r5 << 3) | (r5 >> 2);  // 5-bit to 8-bit
let g8 = (g6 << 2) | (g6 >> 4);  // 6-bit to 8-bit
let b8 = (b5 << 3) | (b5 >> 2);  // 5-bit to 8-bit
```

### RGBA4444 Encoding

16-bit pixel format with bit layout:
```
Bits: RRRR GGGG BBBB AAAA
```

To convert to 8-bit per channel:
```rust
let r4 = (pixel >> 12) & 0x0F;
let g4 = (pixel >> 8) & 0x0F;
let b4 = (pixel >> 4) & 0x0F;
let a4 = pixel & 0x0F;

// Scale to 8-bit by replicating the 4-bit value
let r8 = (r4 << 4) | r4;
let g8 = (g4 << 4) | g4;
let b8 = (b4 << 4) | b4;
let a8 = (a4 << 4) | a4;
```

### RGBA8888 Encoding

32-bit pixel format with direct 8-bit channels:
```
Bytes: [R, G, B, A]
```

No conversion needed - data is already in RGBA8 format.

### Indexed8 Format

8-bit palette index. Requires an external palette (typically from a .pal file or embedded in the level). Not commonly used in Descent 3 compared to true color formats.

## Texture Flags

The `flags` field is a 32-bit bitfield describing texture properties:

| Bit | Flag Name           | Description                                  |
|-----|---------------------|----------------------------------------------|
| 0   | WATER               | Water surface texture                        |
| 1   | LAVA                | Lava surface texture                         |
| 2   | METAL               | Metallic surface                             |
| 3   | SMOOTH              | Smooth surface (high specular)               |
| 4   | ALPHA               | Has alpha transparency                       |
| 5   | SATURATE            | Saturate colors when lit                     |
| 6   | FORCE_LIGHTMAP      | Force lightmap rendering                     |
| 7   | PROCEDURAL          | Procedurally generated texture               |
| 8   | TMAP2               | Secondary texture map                        |
| 9   | ANIMATED            | Animated texture (multiple frames)           |
| 10  | DESTROYABLE         | Can be destroyed                             |
| 11  | LIGHT               | Emits light                                  |
| 12  | BREAKABLE           | Glass-like breakable surface                 |
| 13  | SATURATE_LIGHTMAP   | Saturate lightmap colors                     |
| 14  | TEXTURE_64          | 64×64 texture size hint                      |
| 15  | TMAP2_VERTEX        | Secondary vertex mapping                     |
| 16  | VOLATILE            | Texture can change at runtime                |
| 17  | WATER_PROCEDURAL    | Procedural water animation                   |
| 18  | FORCE_LIGHTMAP_BLEND| Force blended lightmap                       |
| 19  | SATURATE_VERTEX     | Saturate vertex colors                       |
| 20  | NO_COMPRESS         | Do not compress in video memory              |

Flags 21-31 are reserved for future use.

## Mipmaps

Mipmaps are progressively smaller versions of the base texture used for level-of-detail (LOD) rendering. Each mipmap level is half the size of the previous level:

- Level 0 (base): width × height
- Level 1: (width/2) × (height/2)
- Level 2: (width/4) × (height/4)
- Level N: (width/2^N) × (height/2^N)

Minimum dimension is 1 pixel. For example, a 256×256 texture with 8 mipmaps:
- Level 0: 256×256
- Level 1: 128×128
- Level 2: 64×64
- Level 3: 32×32
- Level 4: 16×16
- Level 5: 8×8
- Level 6: 4×4
- Level 7: 2×2
- Level 8: 1×1

Mipmaps are stored sequentially after the base texture. The total size for one frame is:
```
size = base_size + mipmap1_size + mipmap2_size + ... + mipmapN_size
```

## Animation

Animated textures have multiple frames stored sequentially. The `num_frames` field indicates the number of frames, and `fps` specifies the playback speed.

For an animated texture with mipmaps, the data layout is:
```
Frame 0:
  - Mipmap 0 (base)
  - Mipmap 1
  - ...
  - Mipmap N
Frame 1:
  - Mipmap 0 (base)
  - Mipmap 1
  - ...
  - Mipmap N
...
```

Each frame contains a complete set of mipmaps.

## Data Size Calculations

### Base Texture Size
```rust
base_size = width * height * bytes_per_pixel
```

### Mipmap Size
```rust
mipmap_size(level) = (width >> level) * (height >> level) * bytes_per_pixel
// Note: minimum dimension is 1
```

### Total Data Size
```rust
// Size of one frame with all mipmaps
let mut frame_size = base_size;
for level in 1..=num_mipmaps {
    frame_size += mipmap_size(level);
}

// Total size for all frames
total_size = frame_size * num_frames;
```

## Usage Examples

### Loading an OGF Texture

```rust
use descent_core::ogf::OgfTexture;

// Read file data
let data = std::fs::read("texture.ogf")?;

// Parse OGF texture
let texture = OgfTexture::parse(&data)?;

// Access header info
println!("Size: {}×{}", texture.header.width, texture.header.height);
println!("Format: {:?}", texture.header.format);
println!("Animated: {}", texture.header.is_animated());
println!("Mipmaps: {}", texture.header.num_mipmaps);
```

### Converting to RGBA8

```rust
// Convert to RGBA8888 format
let rgba = texture.to_rgba8()?;

// RGBA data is now suitable for OpenGL/Vulkan/Direct3D
// Each pixel is 4 bytes: [R, G, B, A]
```

### Accessing Mipmaps

```rust
// Get base texture (mipmap level 0)
let base = texture.base_texture();

// Get a specific mipmap level
let mipmap1 = texture.get_mipmap(1)?;
let mipmap2 = texture.get_mipmap(2)?;
```

### Accessing Animation Frames

```rust
// Get first frame
let frame0 = texture.get_frame(0)?;

// Get second frame
let frame1 = texture.get_frame(1)?;

// Check animation properties
if texture.header.is_animated() {
    println!("Animation: {} frames at {} fps",
        texture.header.num_frames,
        texture.header.fps
    );
}
```

### Checking Texture Flags

```rust
use descent_core::ogf::TextureFlags;

let flags = TextureFlags::from_bits_truncate(texture.header.flags);

if flags.contains(TextureFlags::WATER) {
    println!("This is a water texture");
}

if flags.contains(TextureFlags::ALPHA) {
    println!("This texture has transparency");
}

if flags.contains(TextureFlags::LIGHT) {
    println!("This texture emits light");
}
```

## Implementation Notes

### Conversion Quality

The RGB565 and RGBA4444 to RGBA8 conversions use bit replication to preserve full range:
- 5-bit values (0-31) map to (0-255)
- 6-bit values (0-63) map to (0-255)
- 4-bit values (0-15) map to (0-255)

This ensures that maximum values map to 255 (not 248 or 240).

### Indexed Format Support

The Indexed8 format is not fully supported in the current implementation because it requires an external palette. To use indexed textures:
1. Load the palette from a .pal file or level data
2. Manually convert indices to RGB/RGBA using the palette

### Memory Layout

Pixel data is tightly packed with no padding between pixels, rows, or mipmaps. Data is stored in row-major order (left-to-right, top-to-bottom).

## See Also

- [Descent 3 Formats Overview](D3_FORMATS.md) - Overview of all D3 formats
- [HOG Format](HOG_FORMAT.md) - Archive format containing OGF files
- **Implementation:** `crates/descent-core/src/ogf.rs`

## References

- Descent 3 Open Source Release
- Outrage Entertainment texture system documentation
- OpenD3 project format analysis

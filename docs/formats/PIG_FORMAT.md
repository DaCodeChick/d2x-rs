# PIG File Format Specification

## Overview

PIG (Parallax Image Group) files contain texture and bitmap data for Descent 1 and Descent 2. These files store game graphics including wall textures, robot sprites, weapon effects, HUD elements, and more. Bitmaps are stored with RLE (Run-Length Encoding) compression for efficient storage.

## File Structure

```
+------------------+
| Header (8 bytes) |
+------------------+
| Bitmap Headers   |
| (variable size)  |
+------------------+
| Bitmap Data      |
| (RLE compressed) |
+------------------+
```

## Header Format (8 bytes)

| Offset | Size | Type   | Description                                    |
|--------|------|--------|------------------------------------------------|
| 0x00   | 4    | uint32 | Signature: `PPIG` (0x47495050 little-endian)  |
| 0x04   | 4    | int32  | Version: 2                                     |

**Note**: The signature is stored as the ASCII string "PPIG" in the file, which reads as `MAKE_SIG('G','I','P','P')` in D2X-XL code due to little-endian byte order.

## Bitmap Count (4 bytes)

Immediately after the header:

| Offset | Size | Type  | Description                        |
|--------|------|-------|------------------------------------|
| 0x08   | 4    | int32 | Number of bitmap entries in file   |

## Bitmap Header Array

Following the bitmap count, there is an array of bitmap headers. The format differs between Descent 1 and Descent 2:

### Descent 2 Bitmap Header (17 bytes)

| Offset | Size | Type      | Description                                    |
|--------|------|-----------|------------------------------------------------|
| 0x00   | 8    | char[8]   | Bitmap name (null-padded, not null-terminated) |
| 0x08   | 1    | uint8     | dflags: Animation frame info (bits 0-5: frame num, bit 6: ABM flag) |
| 0x09   | 1    | uint8     | width: Low 8 bits of width                     |
| 0x0A   | 1    | uint8     | height: Low 8 bits of height                   |
| 0x0B   | 1    | uint8     | wh_extra: High 4 bits of width (bits 0-3) and height (bits 4-7) |
| 0x0C   | 1    | uint8     | flags: Bitmap flags (transparent, RLE, etc.)   |
| 0x0D   | 1    | uint8     | avgColor: Average color index (for minimap)    |
| 0x0E   | 4    | int32     | offset: Offset from start of bitmap data       |

**Width/Height Calculation**:
```rust
actual_width = width | ((wh_extra & 0x0F) << 8)  // 12 bits total (max 4095)
actual_height = height | ((wh_extra & 0xF0) << 4) // 12 bits total (max 4095)
```

### Descent 1 Bitmap Header (17 bytes)

| Offset | Size | Type      | Description                                    |
|--------|------|-----------|------------------------------------------------|
| 0x00   | 8    | char[8]   | Bitmap name (null-padded, not null-terminated) |
| 0x08   | 1    | uint8     | dflags: Animation frame info                   |
| 0x09   | 1    | uint8     | width: Full width (8 bits, max 255)            |
| 0x0A   | 1    | uint8     | height: Full height (8 bits, max 255)          |
| 0x0B   | 1    | uint8     | flags: Bitmap flags                            |
| 0x0C   | 1    | uint8     | avgColor: Average color index                  |
| 0x0D   | 4    | int32     | offset: Offset from start of bitmap data       |

**Note**: D1 format has no `wh_extra` field, so the on-disk structure is only 17 bytes. Width and height are limited to 255 pixels each.

## Bitmap Flags

Common bitmap flags found in the `flags` field:

| Flag              | Value | Description                                    |
|-------------------|-------|------------------------------------------------|
| BM_FLAG_TRANSPARENT        | 0x01  | Bitmap has transparent pixels (color index 255) |
| BM_FLAG_SUPER_TRANSPARENT  | 0x02  | Super transparency (color index 254)           |
| BM_FLAG_NO_LIGHTING        | 0x04  | Bitmap ignores lighting                        |
| BM_FLAG_RLE                | 0x08  | Bitmap data is RLE compressed                  |
| BM_FLAG_RLE_BIG            | 0x10  | Large RLE compressed bitmap                    |

## Animation Flags (dflags)

The `dflags` field encodes animation information:

| Bits  | Description                                              |
|-------|----------------------------------------------------------|
| 0-5   | Animation frame number (0-63)                            |
| 6     | ABM flag: Part of an animated bitmap sequence           |
| 7     | Reserved                                                 |

```rust
frame_number = dflags & 0x3F;  // bits 0-5
is_animated = (dflags & 0x40) != 0;  // bit 6
```

## Bitmap Data Section

After all bitmap headers, the actual bitmap data begins. The `offset` field in each header is relative to the start of this data section (i.e., after all headers).

**Data Section Start**:
```
data_start = 12 + (num_bitmaps * header_size)
// header_size = 17 bytes for both D1 and D2
```

### Pixel Format

Bitmaps use **8-bit indexed color** (paletted):
- Each pixel is a single byte indexing into a 256-color palette
- Color index **255** = transparent pixel
- Color index **254** = super-transparent pixel (for special effects)
- Color index **0** = sometimes swapped with 255 for special handling

**Palette**: The palette is stored separately in the HAM file or loaded from external palette files (e.g., `palette.256`).

## RLE Compression

Most bitmaps in PIG files are RLE (Run-Length Encoded) compressed to save space.

### RLE Algorithm

The RLE format uses a simple scheme:

```
RLE_CODE = 0xE0        // Top 3 bits set (binary: 11100000)
NOT_RLE_CODE = 0x1F    // Bottom 5 bits (binary: 00011111)
```

**Encoding Rules**:

1. **Literal byte** (non-RLE):
   - If byte < 0xE0: Store byte as-is
   - Output: Single pixel with this color index

2. **RLE sequence** (byte >= 0xE0):
   - First byte: 0xE0 | count (count = 0-31 in bottom 5 bits)
   - Second byte: The color index to repeat
   - Output: `count` pixels of the specified color

3. **End marker**:
   - Byte 0xE0 (count = 0) signals end of compressed data

### RLE Decompression Pseudocode

```rust
fn rle_decompress(src: &[u8]) -> Vec<u8> {
    let mut dest = Vec::new();
    let mut i = 0;
    
    loop {
        let data = src[i];
        i += 1;
        
        if (data & 0xE0) != 0xE0 {
            // Literal byte
            dest.push(data);
        } else {
            // RLE sequence
            let count = data & 0x1F;
            if count == 0 {
                // End marker
                break;
            }
            let color = src[i];
            i += 1;
            // Repeat color 'count' times
            dest.extend(std::iter::repeat(color).take(count as usize));
        }
    }
    
    dest
}
```

### RLE Compression Efficiency

- **Best case**: Large areas of solid color → Very high compression
- **Worst case**: Highly detailed textures with no repeated pixels → Slight expansion (~3% larger than uncompressed)
- **Average**: 30-50% size reduction for typical game textures

## File Format Versions

| Version | Games                     | Notes                                |
|---------|---------------------------|--------------------------------------|
| 1       | Early Descent 1 (v1.0)    | Uncompressed bitmaps                 |
| 2       | Descent 1 & 2, D2X-XL     | RLE compression, standard format     |

## Known PIG File Sizes

### Descent 1

| File Type            | Size (bytes) | Description                       |
|----------------------|--------------|-----------------------------------|
| D1 Shareware v1.0-1.4 (uncompressed) | 5,092,871 | Pre-RLE compression |
| D1 Shareware v1.0-1.2 | 2,529,454    | With RLE                          |
| D1 Shareware v1.4    | 2,509,799    | With RLE                          |
| D1 Registered v1.0 (uncompressed) | 7,640,220 | Pre-RLE compression  |
| D1 Registered v1.0   | 4,520,145    | With RLE                          |
| D1 Registered v1.4-1.5 | 4,920,305  | Standard registered version       |
| D1 OEM v1.0          | 5,039,735    | OEM release                       |
| D1 Mac Registered    | 3,975,533    | Mac version                       |
| D1 Mac Shareware     | 2,714,487    | Mac shareware                     |

### Descent 2

| File Type            | Size (bytes) | Description                       |
|----------------------|--------------|-----------------------------------|
| D2 Standard          | ~6-8 MB      | groupa.pig (main texture set)     |
| D2 Demo              | ~3-5 MB      | d2demo.pig                        |
| D2 Mac Alien1        | 5,013,035    | Mac mission pack                  |
| D2 Mac Alien2        | 4,909,916    | Mac mission pack                  |
| D2 Mac Fire          | 4,969,035    | Mac mission pack                  |
| D2 Mac GroupA        | 4,929,684    | Mac standard (also shareware)     |
| D2 Mac Ice           | 4,923,425    | Mac mission pack                  |
| D2 Mac Water         | 4,832,403    | Mac mission pack                  |

## Maximum Limits

| Limit                | Descent 1   | Descent 2   | D2X-XL (extended) |
|----------------------|-------------|-------------|-------------------|
| Max bitmaps per file | 1,555       | 2,620       | No hard limit     |
| Max textures         | 800         | 910         | Configurable      |
| Max bitmap dimensions | 255x255    | 4095x4095   | 4095x4095         |

## Integration with Other Files

### HOG Archives
PIG files are typically stored inside HOG archives:
```
DESCENT2.HOG
├── GROUPA.PIG    (main texture file)
├── DESCENT2.HAM  (game data including palette)
├── DESCENT2.S11  (11kHz sounds)
└── ... (level files, etc.)
```

### Palette Files
The color palette is stored separately:
- **D1**: Loaded from `DESCENT.PAL` or embedded in HAM file
- **D2**: Stored in `DESCENT2.HAM` file
- **External**: Can use `PALETTE.256` or `PALETTE.BBM` files

### Texture Mapping
The HAM file contains texture information that references bitmaps by index:
```
HAM file: "wall01" → bitmap index 42
PIG file: bitmap index 42 → bitmap header → actual pixel data
```

## Reading Algorithm

High-level algorithm for reading a PIG file:

```rust
1. Open file and read 4-byte signature
2. Verify signature == "PPIG" (0x47495050)
3. Read 4-byte version number
4. Verify version == 2
5. Read 4-byte bitmap count (N)
6. Calculate data_start = 12 + (N * 17)
7. For each of N bitmaps:
   a. Read bitmap header (17 bytes)
   b. Parse name, dimensions, flags, offset
   c. Store header in lookup table (by name)
8. To load a specific bitmap:
   a. Look up bitmap header by name
   b. Seek to data_start + header.offset
   c. Read bitmap data
   d. If BM_FLAG_RLE set: decompress with RLE algorithm
   e. Convert indexed pixels to RGBA using palette
   f. Return bitmap
```

## Common Issues & Gotchas

1. **Name Padding**: Bitmap names are 8 bytes, null-padded but NOT null-terminated. A 3-character name like "FOO" is stored as `"FOO\0\0\0\0\0"`.

2. **Case Sensitivity**: Bitmap name lookups should be case-insensitive. "wall01.bbm" == "WALL01.BBM".

3. **Endianness**: All multi-byte integers are little-endian. When reading on big-endian systems, byte-swap is required.

4. **D1 vs D2 Detection**: 
   - Check dimensions: D1 images rarely exceed 255x255
   - Check file size against known sizes
   - Parse based on game context (loading D1 vs D2 data)

5. **Transparency**: Color index 255 is transparent, but some bitmaps swap color 0 and 255 for rendering purposes.

6. **RLE Bounds**: RLE decompression must check that output size matches expected `width * height`. Malformed RLE can cause buffer overflows.

7. **Palette Dependency**: PIG files are useless without the corresponding palette from the HAM file or palette files.

## Modern Rendering Considerations

For modern engines using GPU rendering (like Bevy):

1. **Convert to RGBA8**: Transform 8-bit indexed pixels to RGBA8 format using the palette
   - Indexed 0-254 → RGB from palette, A=255
   - Indexed 255 → RGBA(0, 0, 0, 0) for transparency

2. **Texture Atlasing**: Combine multiple small bitmaps into texture atlases for better GPU performance

3. **Mipmaps**: Generate mipmaps for textures to improve rendering quality and performance

4. **sRGB Handling**: Original palette is linear color; convert to sRGB for modern rendering pipelines

5. **Animation Support**: Group animated bitmaps (identified by `dflags` ABM bit) into texture arrays or sprite sheets

## References

- D2X-XL source code: `include/piggy.h`, `gameio/piggy.cpp`, `2d/rle.cpp`
- Original Descent source code documentation
- Descent hacker's guide (community resources)

## Example: Reading a Bitmap

```rust
// Open PIG file
let mut file = File::open("groupa.pig")?;

// Read and verify header
let signature = file.read_u32::<LittleEndian>()?;
assert_eq!(signature, 0x47495050); // "PPIG"
let version = file.read_i32::<LittleEndian>()?;
assert_eq!(version, 2);

// Read bitmap count
let num_bitmaps = file.read_i32::<LittleEndian>()?;

// Calculate data section start
let data_start = 12 + (num_bitmaps * 17);

// Read all bitmap headers
let mut headers = Vec::new();
for _ in 0..num_bitmaps {
    let mut name = [0u8; 8];
    file.read_exact(&mut name)?;
    let dflags = file.read_u8()?;
    let width_lo = file.read_u8()?;
    let height_lo = file.read_u8()?;
    let wh_extra = file.read_u8()?;
    let flags = file.read_u8()?;
    let avg_color = file.read_u8()?;
    let offset = file.read_i32::<LittleEndian>()?;
    
    let width = width_lo as u16 | (((wh_extra & 0x0F) as u16) << 8);
    let height = height_lo as u16 | (((wh_extra & 0xF0) as u16) << 4);
    
    headers.push(BitmapHeader {
        name,
        width,
        height,
        flags,
        offset,
        // ... other fields
    });
}

// Load a specific bitmap by name
let header = headers.iter().find(|h| h.name == b"wall01\0\0")?;
file.seek(SeekFrom::Start((data_start + header.offset) as u64))?;

// Read and decompress bitmap data
let compressed_data = read_until_rle_end(&mut file)?;
let pixels = rle_decompress(&compressed_data);
assert_eq!(pixels.len(), (header.width * header.height) as usize);

// Convert to RGBA using palette
let rgba_pixels = convert_indexed_to_rgba(&pixels, &palette);
```

## See Also

- [HOG_FORMAT.md](HOG_FORMAT.md) - HOG archive format (contains PIG files)
- [HAM_FORMAT.md](HAM_FORMAT.md) - HAM game data format (contains palettes)
- [LEVEL_FORMAT.md](LEVEL_FORMAT.md) - RDL/RL2 level format (references textures)

# HOG File Format Specification

## Overview

HOG files are archive formats used by Descent games to package game assets. They provide no compression, only concatenation of files with a directory structure.

There are two versions, implemented in separate modules:
- **DHF format** (`dhf.rs`) - Used by Descent 1 and 2
- **HOG2 format** (`hog2.rs`) - Used by Descent 3

## DHF Format (Descent 1 & 2)

**Module**: `d2x_assets::dhf`  
**Types**: `DhfArchive`, `DhfEntry`

### File Structure

```
+-------------------+
| Header (3 bytes)  |
+-------------------+
| Entry 1           |
|  - Filename (13)  |
|  - Size (4)       |
|  - Data (var)     |
+-------------------+
| Entry 2           |
|  ...              |
+-------------------+
| Entry N           |
+-------------------+
```

### Header

```rust
// No formal header in most HOG files
// Some have "DHF" signature
const HOG_SIGNATURE: &[u8; 3] = b"DHF";
```

### Entry Format

Each entry consists of:

#### Filename (13 bytes)
- Null-terminated string
- Maximum 12 characters + null terminator
- If name is shorter, remaining bytes are 0x00

```rust
struct HogFilename {
    name: [u8; 13],  // null-padded
}
```

#### Size (4 bytes, little-endian)
```rust
size: u32  // little-endian, file size in bytes
```

#### Data (variable length)
Raw file data, `size` bytes

### Example Entry

```
Filename: "LEVEL01.RDL\0"  (13 bytes)
Size:     0x00012A40       (4 bytes) = 76,352 bytes
Data:     [76,352 bytes of level data]
```

## HOG2 Format (Descent 3)

**Module**: `d2x_assets::hog2`  
**Types**: `Hog2Archive`, `Hog2Entry`

Descent 3 uses an enhanced HOG format called HOG2 with structured headers and larger filename support.

### File Structure

```
+---------------------------+
| Header (12 bytes)         |
|  - Signature "HOG2" (4)   |
|  - File Count (4)         |
|  - Data Offset (4)        |
+---------------------------+
| Entry 1 (48 bytes)        |
|  - Filename (36)          |
|  - Flags (4)              |
|  - Size (4)               |
|  - Timestamp (4)          |
+---------------------------+
| Entry 2 (48 bytes)        |
|  ...                      |
+---------------------------+
| Entry N (48 bytes)        |
+---------------------------+
| [Padding to data_offset]  |
+---------------------------+
| File 1 Data               |
+---------------------------+
| File 2 Data               |
+---------------------------+
| File N Data               |
+---------------------------+
```

### Header (12 bytes)

```rust
struct Hog2Header {
    signature: [u8; 4],  // "HOG2"
    file_count: u32,     // Number of files (little-endian)
    data_offset: u32,    // Offset to start of file data (little-endian)
}
```

- **Signature**: Must be exactly `b"HOG2"` (0x48 0x4F 0x47 0x32)
- **File Count**: Total number of files in the archive
- **Data Offset**: Byte offset from start of file to where file data begins
  - Typically `12 + (file_count * 48)` but may have padding

### Entry Format (48 bytes)

```rust
struct Hog2Entry {
    filename: [u8; 36],  // Null-terminated filename
    flags: u32,          // File flags (little-endian)
    size: u32,           // File size in bytes (little-endian)
    timestamp: u32,      // Unix timestamp (little-endian)
}
```

#### Filename (36 bytes)
- Null-terminated string
- Maximum 35 characters + null terminator
- Significantly longer than DHF's 13-byte limit
- Allows for more descriptive filenames
- If name is shorter, remaining bytes are 0x00

#### Flags (4 bytes)
- Purpose not fully documented
- Typically 0x00000000 in standard files
- May be used for compression or other features in some implementations

#### Size (4 bytes)
- File size in bytes (little-endian)
- Size of the file data that follows in the data section

#### Timestamp (4 bytes)
- Unix timestamp (seconds since 1970-01-01 00:00:00 UTC)
- File modification time
- Used for version tracking and freshness checks

### Example Entry

```
Filename: "levels/level1.d3l\0" + padding  (36 bytes)
Flags:    0x00000000                       (4 bytes)
Size:     0x00015840                       (4 bytes) = 88,128 bytes
Timestamp: 0x3B9ACA00                      (4 bytes) = 1,000,000,000 (Sep 2001)
```

### Parsing Algorithm for HOG2

```rust
pub fn parse_hog2<R: Read + Seek>(mut reader: R) -> Result<HogArchive, HogError> {
    // Read signature
    let mut sig = [0u8; 4];
    reader.read_exact(&mut sig)?;
    if &sig != b"HOG2" {
        return Err(HogError::InvalidSignature);
    }
    
    // Read header
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    let file_count = u32::from_le_bytes(buf);
    
    reader.read_exact(&mut buf)?;
    let data_offset = u32::from_le_bytes(buf);
    
    // Read entries
    let mut entries = BTreeMap::new();
    let mut current_data_offset = data_offset as u64;
    
    for _ in 0..file_count {
        // Read filename (36 bytes)
        let mut name_bytes = [0u8; 36];
        reader.read_exact(&mut name_bytes)?;
        let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(36);
        let name = String::from_utf8_lossy(&name_bytes[..name_len]).to_uppercase();
        
        // Read flags, size, timestamp
        reader.read_exact(&mut buf)?;
        let flags = u32::from_le_bytes(buf);
        
        reader.read_exact(&mut buf)?;
        let size = u32::from_le_bytes(buf);
        
        reader.read_exact(&mut buf)?;
        let timestamp = u32::from_le_bytes(buf);
        
        entries.insert(name.clone(), HogEntry {
            name,
            offset: current_data_offset,
            size,
            flags,
            timestamp,
        });
        
        current_data_offset += size as u64;
    }
    
    Ok(HogArchive { entries })
}
```

## Known HOG Files

### Descent 1 (DHF)
- `descent.hog` (~5 MB) - Retail version
- `descent.sow` - Shareware version
- Size detection for versioning

### Descent 2 (DHF)
- `descent2.hog` (~6 MB) - Base game
- `descent2.ham` - Game data definitions
- `alien1.pig`, `alien2.pig`, `fire.pig`, `ice.pig`, `water.pig` - Texture sets
- `groupa.pig` - Common textures

### Descent 3 (HOG2)
- `d3.hog` - Main game data
- `extra.hog` - Additional content
- `extra1.hog` through `extra13.hog` - Mission packs
- Custom mission HOGs

### Custom Missions
- `*.mn2` - D2 mission HOG files
- `*.mn3` - D3 mission HOG files
- May contain custom levels, briefings, textures

## HOG Types in D2X-XL

D2X-XL organizes DHF HOG files into categories:

### D1HogFiles
- `descent.hog` - Descent 1 data
- Contains D1 levels, textures, sounds

### D2HogFiles  
- `descent2.hog` - Descent 2 base data
- Contains D2 levels, textures, sounds

### D2XHogFiles
- `descent2.ham` - Enhanced game data
- Additional content for D2X modifications

### XLHogFiles
- D2X-XL specific enhancements
- Custom models, textures, effects

### ExtraHogFiles
- User-provided addon content
- Texture packs, custom sounds

### MsnHogFiles
- Mission-specific HOG files
- Custom levels and assets

## Parsing Algorithm (DHF)

```rust
use std::io::{Read, Seek, SeekFrom};

pub struct DhfArchive {
    entries: BTreeMap<String, DhfEntry>,
}

pub struct DhfEntry {
    name: String,
    offset: u64,
    size: u32,
}

impl DhfArchive {
    pub fn parse<R: Read + Seek>(mut reader: R) -> Result<Self, HogError> {
        let mut entries = BTreeMap::new();
        let mut current_offset = 0u64;
        
        // Optional: Check for DHF signature
        let mut sig = [0u8; 3];
        if reader.read_exact(&mut sig).is_ok() && &sig == b"DHF" {
            current_offset = 3;
        } else {
            reader.seek(SeekFrom::Start(0))?;
        }
        
        loop {
            // Read filename
            let mut name_bytes = [0u8; 13];
            match reader.read_exact(&mut name_bytes) {
                Ok(_) => {},
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }
            
            // Parse null-terminated filename
            let name_len = name_bytes.iter()
                .position(|&b| b == 0)
                .unwrap_or(13);
            let name = String::from_utf8_lossy(&name_bytes[..name_len])
                .to_uppercase();
            
            // Read size
            let mut size_bytes = [0u8; 4];
            reader.read_exact(&mut size_bytes)?;
            let size = u32::from_le_bytes(size_bytes);
            
            current_offset += 17; // 13 + 4
            
            // Store entry
            entries.insert(name.clone(), DhfEntry {
                name,
                offset: current_offset,
                size,
            });
            
            // Skip file data
            reader.seek(SeekFrom::Current(size as i64))?;
            current_offset += size as u64;
        }
        
        Ok(DhfArchive { entries })
    }
}
```

## Parsing Algorithm (HOG2)

See the HOG2 section above for the parsing algorithm specific to Descent 3 format.

## Performance Considerations

### Binary Search
D2X-XL sorts entries and uses binary search (`CHogFile::BinSearch`). For large HOG files (300+ entries), this is significantly faster than linear search.

```rust
// Store entries in BTreeMap for O(log n) lookup
entries: BTreeMap<String, DhfEntry>  // or Hog2Entry
```

### Memory Mapping
For frequently accessed HOG files, consider memory-mapping:

```rust
use memmap2::Mmap;

pub struct MmapDhfArchive {
    mmap: Mmap,
    entries: BTreeMap<String, DhfEntry>,
}
```

### Lazy Loading
Don't extract all files upfront. Load on-demand:

```rust
pub fn get_file(&self, name: &str) -> Result<&[u8], HogError> {
    let entry = self.entries.get(name)?;
    let start = entry.offset as usize;
    let end = start + entry.size as usize;
    Ok(&self.mmap[start..end])
}
```

## Known HOG Files

### Descent 1
- `descent.hog` (~5 MB) - Retail version
- `descent.sow` - Shareware version
- Size detection for versioning (see `piggy.h` constants)

### Descent 2
- `descent2.hog` (~6 MB) - Base game
- `descent2.ham` - Game data definitions
- `alien1.pig`, `alien2.pig`, `fire.pig`, `ice.pig`, `water.pig` - Texture sets
- `groupa.pig` - Common textures

### Custom Missions
- `*.mn2` - Mission HOG files
- May contain custom levels, briefings, textures

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum HogError {
    #[error("Invalid HOG signature")]
    InvalidSignature,
    
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Corrupt entry at offset {offset}")]
    CorruptEntry { offset: u64 },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_dhf() {
        // Test with actual descent2.hog
        let dhf = DhfArchive::open("test_data/descent2.hog").unwrap();
        
        // Verify known files exist
        assert!(dhf.contains_file("LEVEL01.RL2"));
        assert!(dhf.contains_file("ROBOT001.POF"));
        
        // Test case insensitivity
        assert!(dhf.contains_file("level01.rl2"));
    }
    
    #[test]
    fn test_read_file() {
        let mut dhf = DhfArchive::open("test_data/descent2.hog").unwrap();
        let data = dhf.read_file("LEVEL01.RL2").unwrap();
        
        // Verify non-zero size
        assert!(data.len() > 0);
    }
    
    #[test]
    fn test_parse_hog2() {
        // Test with Descent 3 archive
        let hog = Hog2Archive::open("test_data/d3.hog").unwrap();
        
        // Check file count is available
        assert!(hog.file_count() > 0);
        
        // Test case insensitivity
        let first_entry = hog.entries().next().unwrap();
        assert!(hog.contains_file(&first_entry.name.to_lowercase()));
    }
}
```

## Format Comparison

| Feature | DHF (D1/D2) | HOG2 (D3) |
|---------|-------------|-----------|
| **Signature** | Optional "DHF" | Required "HOG2" |
| **Header Size** | 0-3 bytes | 12 bytes |
| **Entry Size** | 17 bytes + data | 48 bytes (header), data separate |
| **Filename Length** | 13 bytes (12 chars + null) | 36 bytes (35 chars + null) |
| **File Count** | None (scan until EOF) | In header |
| **Data Organization** | Inline with entries | Separate data section |
| **Flags** | None | 4-byte flags field |
| **Timestamp** | None | Unix timestamp |
| **Data Offset** | None | In header |
| **Binary Search** | External (D2X-XL) | Header supports |

## References

- D2X-XL HOG (D1/D2): `/tmp/d2x-xl-src/include/hogfile.h`, `/tmp/d2x-xl-src/io/hogfile.cpp`
- Descent 3 SDK: Outrage Engine documentation
- Descent Technical Specs: https://www.descent-community.org/

---

**Document Version**: 2.0  
**Last Updated**: 2026-02-24
**Changes**: Added HOG2 (Descent 3) format specification

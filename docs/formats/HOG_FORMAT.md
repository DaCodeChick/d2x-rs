# HOG File Format Specification

## Overview

HOG files are simple archive formats used by Descent to package game assets. They provide no compression, only concatenation of files with a directory structure.

**Corresponds to**: `include/hogfile.h`, `io/hogfile.cpp`

## File Structure

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

## Header

```rust
// No formal header in most HOG files
// Some have "DHF" signature
const HOG_SIGNATURE: &[u8; 3] = b"DHF";
```

## Entry Format

Each entry consists of:

### Filename (13 bytes)
- Null-terminated string
- Maximum 12 characters + null terminator
- If name is shorter, remaining bytes are 0x00

```rust
struct HogFilename {
    name: [u8; 13],  // null-padded
}
```

### Size (4 bytes, little-endian)
```rust
size: u32  // little-endian, file size in bytes
```

### Data (variable length)
Raw file data, `size` bytes

## Example Entry

```
Filename: "LEVEL01.RDL\0"  (13 bytes)
Size:     0x00012A40       (4 bytes) = 76,352 bytes
Data:     [76,352 bytes of level data]
```

## HOG Types in D2X-XL

D2X-XL organizes HOG files into categories:

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

## Parsing Algorithm

```rust
use std::io::{Read, Seek, SeekFrom};

pub struct HogArchive {
    entries: BTreeMap<String, HogEntry>,
}

pub struct HogEntry {
    name: String,
    offset: u64,
    size: u32,
}

impl HogArchive {
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
            entries.insert(name, HogEntry {
                name: name.clone(),
                offset: current_offset,
                size,
            });
            
            // Skip file data
            reader.seek(SeekFrom::Current(size as i64))?;
            current_offset += size as u64;
        }
        
        Ok(HogArchive { entries })
    }
    
    pub fn read_file(&self, name: &str) -> Result<Vec<u8>, HogError> {
        let entry = self.entries.get(&name.to_uppercase())
            .ok_or(HogError::FileNotFound)?;
        
        // Seek and read file data
        // ...
    }
}
```

## Performance Considerations

### Binary Search
D2X-XL sorts entries and uses binary search (`CHogFile::BinSearch`). For large HOG files (300+ entries), this is significantly faster than linear search.

```rust
// Store entries in BTreeMap for O(log n) lookup
entries: BTreeMap<String, HogEntry>
```

### Memory Mapping
For frequently accessed HOG files, consider memory-mapping:

```rust
use memmap2::Mmap;

pub struct MmapHogArchive {
    mmap: Mmap,
    entries: BTreeMap<String, HogEntry>,
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
    fn test_parse_hog() {
        // Test with actual descent2.hog
        let hog = HogArchive::open("test_data/descent2.hog").unwrap();
        
        // Verify known files exist
        assert!(hog.contains_file("LEVEL01.RL2"));
        assert!(hog.contains_file("ROBOT001.POF"));
        
        // Test case insensitivity
        assert!(hog.contains_file("level01.rl2"));
    }
    
    #[test]
    fn test_read_file() {
        let hog = HogArchive::open("test_data/descent2.hog").unwrap();
        let data = hog.read_file("LEVEL01.RL2").unwrap();
        
        // Verify non-zero size
        assert!(data.len() > 0);
    }
}
```

## References

- D2X-XL: `/tmp/d2x-xl-src/include/hogfile.h`
- D2X-XL: `/tmp/d2x-xl-src/io/hogfile.cpp`
- Descent Technical Specs: https://www.descent-community.org/

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-23

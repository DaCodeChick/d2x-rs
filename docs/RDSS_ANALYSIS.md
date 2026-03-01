# RDSS Policy Analysis - D2X-RS Codebase

This document summarizes the comprehensive codebase analysis performed to establish the RDSS (Refactoring, Documentation, Style, and Standardization) policy for the d2x-rs project.

**Date**: 2026-03-01  
**Analysis Scope**: `crates/descent-core/` (primary library)  
**Total Lines**: 13,367 lines of code  
**Test Coverage**: 179 passing tests  

---

## Executive Summary

The d2x-rs codebase is in **excellent shape** with strong foundations:

### ✅ Strengths
- **Zero unsafe code**: No unsafe blocks, mem::transmute, or raw pointers
- **Custom I/O trait**: Well-designed `ReadExt` trait for safe binary parsing
- **Consistent error handling**: Uses thiserror for structured errors (with one exception)
- **Good test coverage**: 179 tests, all passing
- **Safe type conversions**: Uses `from_le_bytes()` exclusively (no transmute)

### ⚠️ Minor Inconsistencies Found
1. Error handling: `video.rs` uses `anyhow::Result` (inconsistent)
2. Collections: Mixed use of HashMap vs BTreeMap (pattern unclear)
3. String conversions: Multiple methods used (`.to_string()` preferred)
4. Documentation: 2.5% doc comment ratio (should be 5%+)

---

## Detailed Findings

### 1. Memory Safety ✅ EXCELLENT

**Status**: Zero tolerance policy fully met

- **Unsafe blocks**: 0 (verified with `rg "unsafe"`)
- **mem::transmute**: 0
- **bytemuck**: Not used
- **Raw pointers**: 0

**Conclusion**: Maintain zero tolerance. All binary parsing uses safe abstractions.

---

### 2. Binary I/O Operations ✅ GOOD

**Pattern**: Custom `ReadExt` trait in `crates/descent-core/src/io.rs`

**Usage Statistics**:
```
from_le_bytes: 44 uses (mostly in ReadExt implementation)
ReadExt trait: Used in 8 core parsers
```

**Files using ReadExt**:
- `level.rs` - Level file parser
- `sound.rs` - Audio file parser  
- `pof.rs` - 3D model parser
- `ham.rs` - Game data parser
- `pig.rs` - Texture file parser
- `player.rs` - Player save parser

**Standard established**: Use `ReadExt` trait methods for all binary parsing.

**Available methods**:
```rust
read_u8(), read_u16_le(), read_u32_le()
read_i16_le(), read_i32_le()
read_f32_le()
read_bytes(count), skip_bytes(count)
```

**Fallback pattern** (when ReadExt not available):
```rust
let mut buf = [0u8; 4];
file.read_exact(&mut buf)?;
let value = u32::from_le_bytes(buf);
```

---

### 3. Error Handling ⚠️ MOSTLY CONSISTENT

**Primary Pattern**: `crate::error::Result<T>` with `AssetError` (via thiserror)

**Usage**:
- ✅ **23 core files** use `AssetError` + `Result<T>`
- ❌ **1 file** (`video.rs`) uses `anyhow::Result` (inconsistent)

**Error Type Hierarchy**:
```
AssetError (main, 23 uses)
├── InvalidFormat
├── FileNotFound
├── UnsupportedVersion
├── CorruptData
└── Io(#[from] io::Error)

ArchiveError (converters/archive.rs)
├── Io, Parse, FileNotFound, InvalidPath

AudioConvertError (converters/audio.rs)
├── Io, InvalidSampleRate, EmptyData, DataTooLarge

TextureConvertError (converters/texture.rs)
├── Io, UnsupportedFormat, InvalidDimensions
```

**Recommendation**: 
1. ✅ Accepted: Converters may define custom error types
2. ⚠️ Fix: Change `video.rs` to use `AssetError` for consistency

**Standard established**: Use `crate::error::Result<T>` everywhere except converters with specialized error needs.

---

### 4. String Conversions ⚠️ MIXED (Standardized)

**Usage Statistics**:
```
.to_string()   : 81 occurrences  ✅ Preferred
.into()        : 37 occurrences  (acceptable in generic code)
String::from() : 0  occurrences  (not used)
.to_owned()    : 0  occurrences  (not used)
```

**Analysis**:
- `.to_string()` is most prevalent (81 vs 37)
- `.into()` used primarily in math/geometry code (trait-heavy)
- `.to_owned()` and `String::from()` not used at all

**Standard established**: Use `.to_string()` as default, `.into()` acceptable in generic contexts.

**Rationale**:
1. Most prevalent pattern in codebase
2. Clear and explicit intent
3. Works with `Display` trait (more flexible)

---

### 5. Collections: HashMap vs BTreeMap ⚠️ PATTERN UNCLEAR (Clarified)

**BTreeMap usage** (3 files):
- `hog2.rs` - HOG2 archive entries
- `dhf.rs` - DHF archive entries
- `mvl.rs` - MVL archive entries

**HashMap usage** (2 files):
- `pig.rs` - PIG texture/bitmap data
- `player.rs` - Player save data

**Pattern identified**:
- **BTreeMap**: Used for archive file entries (sorted keys beneficial)
- **HashMap**: Used for in-memory asset lookup (performance priority)

**Standard established**:
- Use `BTreeMap` for archives where sorted iteration is useful
- Use `HashMap` for asset lookup where only key-based access is needed

---

### 6. Validation Patterns ✅ EXCELLENT

**Pattern**: Centralized validation functions in `crate::validation` module

**Available validators**:
```rust
validate_signature(actual, expected, format_name)
validate_string_signature(actual, expected, format_name)
validate_version(actual, expected_list, format_name)
validate_range(value, min, max, field_name)
validate_max(value, max, field_name, max_name)
validate_min(value, min, field_name)
validate_non_zero(value, field_name)
validate_index(index, max, field_name)
validate_buffer_size(buffer, required, format_name)
```

**Usage**: Consistently used across all parsers (ham.rs, pig.rs, hog2.rs, etc.)

**Standard established**: Always use validation module functions for consistency.

---

### 7. Import Organization ✅ CONSISTENT

**Pattern observed** (97 use statements analyzed):

```rust
// 1. Internal crate imports
use crate::error::{AssetError, Result};
use crate::io::ReadExt;

// 2. External crate imports  
use thiserror::Error;
use glam::Vec3;

// 3. Standard library imports
use std::collections::HashMap;
use std::io::{Cursor, Read};
```

**Standard established**: Three-group import order with blank lines between groups.

---

### 8. Documentation Standards ⚠️ NEEDS IMPROVEMENT

**Statistics**:
- **Doc comment lines**: 331
- **Total lines**: 13,367
- **Doc ratio**: 2.5% (industry standard: 5-10%)

**Quality**: High quality where present
- Module-level docs include format specifications
- Function-level docs include examples
- Error conditions documented

**Recommendation**: Increase documentation coverage, especially for public APIs.

**Standard established**: All public APIs must have:
1. Module-level docs with format specs
2. Function-level docs with examples
3. Error condition documentation

---

### 9. Naming Conventions ✅ EXCELLENT

**Observed patterns** (40 function signatures analyzed):

```rust
Types:     PascalCase  - PigFile, BitmapHeader, AssetError
Functions: snake_case  - read_file(), parse_entries(), validate_signature()
Constants: SCREAMING   - PIG_SIGNATURE, MAX_TEXTURES, BITMAP_HEADER_SIZE
Modules:   snake_case  - fixed_point, io, converters
```

**Format-specific naming**:
```rust
Archives:   {Format}Archive   - Hog2Archive, DhfArchive
Parsers:    {Format}File      - PigFile, HamFile, HmpFile
Converters: {Type}Converter   - TextureConverter, ModelConverter
```

**Standard established**: Follow existing naming patterns strictly.

---

### 10. Parsing Patterns ✅ CONSISTENT

**Common pattern across all parsers**:

```rust
pub fn parse(data: &[u8]) -> Result<Self> {
    let mut cursor = Cursor::new(data);
    
    // 1. Validate signature
    let signature = cursor.read_u32_le()?;
    validate_signature(signature, EXPECTED_SIG, "FORMAT")?;
    
    // 2. Validate version
    let version = cursor.read_i32_le()?;
    validate_version(version, &[2, 3], "FORMAT")?;
    
    // 3. Read counts with validation
    let count = cursor.read_i32_le()? as usize;
    validate_max(count, MAX_COUNT, "field", "MAX")?;
    
    // 4. Parse entries
    let entries = (0..count)
        .map(|_| parse_entry(&mut cursor))
        .collect::<Result<Vec<_>>>()?;
    
    Ok(Self { entries })
}
```

**Null-terminated string handling**:
```rust
// Option 1: Helper function
use crate::io::read_null_padded_string;
let name = read_null_padded_string(&bytes[0..8]);

// Option 2: Inline (small cases)
let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(name_bytes.len());
let name = String::from_utf8_lossy(&name_bytes[..name_len]).to_uppercase();
```

**Standard established**: Follow consistent parsing structure across all format parsers.

---

### 11. Constants and Magic Numbers ✅ GOOD

**Pattern**: Module-level constants with documentation

```rust
/// PIG file signature "PPIG" (0x47495050 little-endian)
const PIG_SIGNATURE: u32 = 0x47495050;

/// Bitmap header size in bytes (same for D1 and D2)
const BITMAP_HEADER_SIZE: usize = 17;
```

**Standard established**: All magic numbers must be named constants with doc comments.

---

### 12. Feature Gates ✅ EXCELLENT

**Current features**:
```toml
[features]
base-d1 = []
base-d2 = ["base-d1"]
base-d3 = []
extended-limits = []
hires-assets = []
d2x-xl = ["extended-limits", "hires-assets"]
video = ["dep:ffmpeg-next"]
```

**Usage**: Properly feature-gated optional functionality (video, hires-assets)

**Standard established**: Continue using feature gates for optional dependencies.

---

## Dependencies Analysis

**Cargo.toml** (`descent-core`):

**Core dependencies** (always included):
```toml
serde, bincode          # Serialization
thiserror, anyhow       # Error handling (anyhow only for CLI tools)
winnow                  # Binary parsing
glam                    # Math
image                   # Image processing (TGA only)
gltf-json, serde_json   # 3D model formats
once_cell, bitflags     # Utilities
tracing                 # Logging
base64                  # Encoding
```

**Optional dependencies**:
```toml
ffmpeg-next = { optional = true, features = ["video"] }
```

**No duplicate dependencies found** (verified with `cargo tree --duplicates`)

**Recommendation**: Clean and minimal dependency tree. No changes needed.

---

## Code Quality Metrics

| Metric | Value | Status |
|--------|-------|--------|
| Total lines | 13,367 | - |
| Doc comment lines | 331 | ⚠️ 2.5% (target: 5%+) |
| Test count | 179 | ✅ Good |
| Unsafe blocks | 0 | ✅ Excellent |
| Public functions | 184 | - |
| from_le_bytes uses | 44 | ✅ Safe pattern |
| ReadExt usage | 8 parsers | ✅ Consistent |

---

## Recommendations

### High Priority
1. ✅ **DONE**: Document standards in AGENTS.md
2. ⚠️ **TODO**: Fix `video.rs` error handling (use AssetError instead of anyhow)
3. ⚠️ **TODO**: Increase documentation coverage (aim for 5%+)

### Medium Priority
4. ✅ **DONE**: Clarify HashMap vs BTreeMap usage patterns
5. ✅ **DONE**: Standardize string conversion methods (prefer `.to_string()`)
6. ⚠️ **TODO**: Add more inline examples in doc comments

### Low Priority
7. ✅ **DONE**: Document ReadExt trait usage
8. ✅ **DONE**: Document validation function usage
9. ⚠️ **TODO**: Consider adding helper for null-padded strings (currently has read_null_padded_string)

---

## Conclusion

The d2x-rs codebase demonstrates **excellent engineering practices** with a strong foundation:

- **Safety-first approach**: Zero unsafe code
- **Well-designed abstractions**: ReadExt trait, validation module
- **Consistent patterns**: Error handling, parsing, naming
- **Clean dependencies**: Minimal, well-organized

The minor inconsistencies identified have been documented and standardized in the AGENTS.md policy document. All future code should follow these established patterns.

**Next Steps**:
1. Apply standards to new code
2. Gradually improve documentation coverage
3. Fix `video.rs` error handling inconsistency
4. Continue maintaining zero unsafe code policy

---

**Analysis performed by**: OpenCode AI Agent  
**Methodology**: Static analysis, pattern matching, usage statistics  
**Tools used**: ripgrep, cargo tree, file analysis  

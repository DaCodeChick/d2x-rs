# Agent Policies

This document defines coding policies and guidelines for AI agents working on this project.

## RDSS Policy

**RDSS** = **Refactor, Despaghettify, Simplify, Split**

When working on code in this project, always apply the RDSS policy:

### R - Refactor
- Improve code structure and organization
- Extract reusable functions and modules
- Apply language-specific best practices and idioms
- Eliminate code duplication
- Improve naming for clarity and consistency

### D - Despaghettify
- Remove complex nested logic and callback chains
- Flatten deeply nested code structures
- Break up circular dependencies
- Eliminate global state where possible
- Replace complex control flow with clearer alternatives

### S - Simplify
- Reduce cognitive complexity
- Remove unnecessary abstractions
- Use clearer, more direct approaches
- Eliminate dead code and unused features
- Prefer explicit over implicit behavior

### S - Split
- Break up monolithic files into focused modules
- Separate concerns into distinct components
- Keep functions and methods small and focused
- Split large data structures into logical pieces
- Organize code by feature or domain, not by type

## Language-Specific Guidelines

### Rust (Primary Language)

- **Edition**: Use Rust 2024 edition
- **Idioms**: Leverage modern Rust patterns (async/await, const generics, trait bounds)
- **Error Handling**: Use `Result<T, E>` and `?` operator, never `unwrap()` in production code
- **Memory Safety**: Prefer owned types, use references judiciously, avoid `unsafe` unless absolutely necessary
- **Zero-cost Abstractions**: Use iterators, closures, and trait objects appropriately
- **Compile-Time Evaluation**: Always check if functions can be `const fn`
  - Maximize compile-time computation
  - Use `const fn` for functions that can be evaluated at compile time
  - Use `const` for constant values and static data
- **Documentation**: Write comprehensive doc comments with examples
- **Testing**: Include unit tests for all public APIs

#### D2X-RS Specific Standards

This section defines coding standards specific to the d2x-rs project based on analysis of the existing codebase.

##### Error Handling

**Standard**: Use `crate::error::Result<T>` with `AssetError` (via `thiserror`)

```rust
use crate::error::{AssetError, Result};

pub fn parse_file(data: &[u8]) -> Result<FileData> {
    // Implementation
}
```

**Anti-patterns to avoid**:
- ❌ Do NOT use `anyhow::Result` in library code (only acceptable in CLI tools)
- ❌ Do NOT create custom error types unless for converters with specific error variants

**Converter exceptions**: Converters may define their own error types when they need additional error variants:
- `ArchiveError` in `converters/archive.rs`
- `AudioConvertError` in `converters/audio.rs`
- `TextureConvertError` in `converters/texture.rs`

##### Binary I/O Operations

**Standard**: Use the `ReadExt` trait from `crate::io` for all binary parsing

```rust
use crate::io::ReadExt;
use std::io::Cursor;

let mut cursor = Cursor::new(data);
let value = cursor.read_u32_le()?;  // ✅ Correct
let signed = cursor.read_i16_le()?; // ✅ Correct
let float = cursor.read_f32_le()?;  // ✅ Correct
```

**Available methods**:
- `read_u8()`, `read_u16_le()`, `read_u32_le()`
- `read_i16_le()`, `read_i32_le()`
- `read_f32_le()`
- `read_bytes(count)` - reads exactly N bytes
- `skip_bytes(count)` - skips N bytes

**Anti-patterns to avoid**:
- ❌ Do NOT use manual buffer reads with `from_le_bytes()` (except in ReadExt implementation)
- ❌ Do NOT use `unsafe` or `mem::transmute` for type conversions
- ❌ Do NOT use `bytemuck` crate (not used in this project)

**For manual parsing** (when ReadExt isn't available):
```rust
let mut buf = [0u8; 4];
file.read_exact(&mut buf)?;
let value = u32::from_le_bytes(buf); // ✅ Acceptable fallback
```

##### String Conversions

**Standard**: Use `.to_string()` as the default string conversion method

```rust
let owned: String = string_slice.to_string();  // ✅ Preferred (81 uses)
```

**When to use alternatives**:
- Use `.into()` for generic conversions in trait-heavy code (37 uses in math/geometry)
- Use `String::from()` only when explicitly converting from non-string types (0 uses - avoid)
- Use `.to_owned()` only when semantically cloning data (0 uses - avoid)

**Rationale**: `.to_string()` is:
1. Most prevalent in the codebase (81 vs 37 uses)
2. Clear and explicit about intent
3. Works with `Display` trait (more flexible than `to_owned()`)

##### Collections: HashMap vs BTreeMap

**Standard**: Choose based on use case

**Use `BTreeMap`** for:
- Archive file entries (HOG2, DHF, MVL) where sorted keys are useful
- Cases where iteration order matters
- Cases where you need range queries

```rust
use std::collections::BTreeMap;

pub struct Hog2Archive {
    entries: BTreeMap<String, Hog2Entry>, // ✅ Sorted archive entries
}
```

**Use `HashMap`** for:
- In-memory asset lookup (PIG, Player files)
- Cases where only key-based lookup is needed
- Performance-critical lookups

```rust
use std::collections::HashMap;

pub struct PigFile {
    bitmaps: HashMap<String, BitmapData>, // ✅ Fast asset lookup
}
```

##### Validation Patterns

**Standard**: Use validation functions from `crate::validation` module

```rust
use crate::validation::{validate_signature, validate_version, validate_max};

// Validate file signature
validate_signature(signature, EXPECTED_SIG, "PIG")?;

// Validate version
validate_version(version, &[2, 3], "PIG")?;

// Validate array bounds
validate_max(count, MAX_TEXTURES, "texture count", "MAX_TEXTURES")?;
```

**Available validators**:
- `validate_signature(actual, expected, format_name)` - for u32 magic numbers
- `validate_string_signature(actual, expected, format_name)` - for string signatures
- `validate_version(actual, expected_list, format_name)` - for version checks
- `validate_range(value, min, max, field_name)` - for range validation
- `validate_max(value, max, field_name, max_name)` - for upper bounds
- `validate_min(value, min, field_name)` - for lower bounds
- `validate_non_zero(value, field_name)` - for non-zero checks
- `validate_index(index, max, field_name)` - for array index validation

##### Import Organization

**Standard**: Group imports in the following order (with blank lines between groups):

```rust
// 1. Internal crate imports
use crate::error::{AssetError, Result};
use crate::io::ReadExt;
use crate::validation::validate_signature;

// 2. External crate imports
use thiserror::Error;
use glam::Vec3;

// 3. Standard library imports
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::Path;
```

**Rationale**: This matches the existing codebase pattern and improves readability.

##### Naming Conventions

**Standard**: Follow Rust naming conventions strictly

- **Types**: `PascalCase` - `PigFile`, `BitmapHeader`, `AssetError`
- **Functions**: `snake_case` - `read_file()`, `parse_entries()`, `validate_signature()`
- **Constants**: `SCREAMING_SNAKE_CASE` - `PIG_SIGNATURE`, `MAX_TEXTURES`, `BITMAP_HEADER_SIZE`
- **Modules**: `snake_case` - `fixed_point`, `io`, `converters`

**Format-specific naming**:
- Archive structs: `{Format}Archive` - `Hog2Archive`, `DhfArchive`, `MvlArchive`
- File parsers: `{Format}File` - `PigFile`, `HamFile`, `HmpFile`
- Converters: `{Type}Converter` - `TextureConverter`, `ModelConverter`, `AudioConverter`

##### Null-Terminated Strings

**Standard**: Use helper function for null-padded strings

```rust
use crate::io::read_null_padded_string;

let name = read_null_padded_string(&bytes[0..8]); // ✅ Correct
```

**For inline parsing** (small cases):
```rust
let name_len = name_bytes.iter().position(|&b| b == 0).unwrap_or(name_bytes.len());
let name = String::from_utf8_lossy(&name_bytes[..name_len]).to_uppercase();
```

##### Documentation Standards

**Standard**: All public APIs must have doc comments with:

1. **Module-level docs** with format specifications:
```rust
//! PIG texture/bitmap file format parser
//!
//! ## File Format
//! ```text
//! Header:
//!   - signature: 4 bytes ("PPIG")
//!   - version: 4 bytes (little-endian i32, value 2)
//!   - bitmap_count: 4 bytes (little-endian i32)
//! ```
```

2. **Function-level docs** with examples:
```rust
/// Parse a PIG file from raw bytes.
///
/// # Example
/// ```no_run
/// let data = std::fs::read("descent.pig")?;
/// let pig = PigFile::parse(&data)?;
/// ```
///
/// # Errors
/// Returns error if:
/// - File signature is invalid
/// - Version is not 2
/// - Data is truncated
pub fn parse(data: &[u8]) -> Result<Self> {
```

3. **Error documentation**: Always document error conditions in doc comments

**Statistics**: Codebase has 331 doc comment lines for 13,367 total lines (~2.5% documentation ratio - aim for 5%+)

##### File References in Errors

**Standard**: Use `.to_string()` for string literals in error messages

```rust
return Err(AssetError::InvalidFormat(
    "Expected HOG2 signature".to_string(), // ✅ Correct
));

return Err(AssetError::FileNotFound(name.to_string())); // ✅ Correct
```

##### Constants and Magic Numbers

**Standard**: Define constants at module level with documentation

```rust
/// PIG file signature "PPIG" (0x47495050 little-endian)
const PIG_SIGNATURE: u32 = 0x47495050;

/// PIG file version
const PIG_VERSION: i32 = 2;

/// Bitmap header size in bytes (same for D1 and D2)
const BITMAP_HEADER_SIZE: usize = 17;
```

**Anti-patterns to avoid**:
- ❌ Magic numbers in code without explanation
- ❌ Hardcoded array sizes without named constants

##### Feature Gates

**Standard**: Use feature gates for optional functionality

```rust
// In Cargo.toml
[features]
default = []
base-d1 = []
base-d2 = ["base-d1"]
hires-assets = []
video = ["dep:ffmpeg-next"]

// In code
#[cfg(feature = "hires-assets")]
pub fn parse_ase_model(data: &[u8]) -> Result<AseFile> {
    // Implementation
}
```

**Current features**:
- `base-d1`, `base-d2`, `base-d3` - Game version support
- `extended-limits` - D2X-XL extended limits
- `hires-assets` - D2X-XL ASE models and high-res textures
- `d2x-xl` - Combines `extended-limits` and `hires-assets`
- `video` - MVE to MP4 conversion (requires FFmpeg)

##### Unsafe Code Policy

**Standard**: Zero tolerance for unsafe code

**Status**: ✅ Codebase currently has ZERO `unsafe` blocks (verified via `rg "unsafe"`)

**Policy**:
- ❌ Do NOT use `unsafe` blocks
- ❌ Do NOT use `mem::transmute`
- ❌ Do NOT use raw pointers
- ✅ Use safe abstractions (ReadExt trait, `from_le_bytes()`, standard collections)

**Exception**: If `unsafe` is absolutely necessary (extremely rare):
1. Document WHY it's needed with detailed comments
2. Document safety invariants being upheld
3. Minimize scope of unsafe block
4. Get explicit approval before committing

### C++ (Level Editor Only - Phase 2)

- **Standard**: Use C++23 exclusively
- **Modern Idioms**: 
  - Use `<format>` and `<print>` for formatted output (no `printf`, no `iostream` operators)
  - Use `std::expected` for error handling (**NEVER** use exceptions)
  - Use concepts for template constraints
  - Use ranges and views from `<ranges>`
  - Use `std::span` for array views

- **Error Handling**: *ALWAYS* use `std::expected<T, ErrorType>`, *NEVER* use exceptions
  - Define custom error types as simple structs with `std::string message` field
  - Use `std::unexpected(error)` to return errors
  - Return `{}` or `std::expected<void, E>{}` for success with no value
  - Propagate errors explicitly (no `throw`, no `try`/`catch`)
  - Example:
    ```cpp
    struct ParseError {
        std::string message;
    };
    
    std::expected<Data, ParseError> parseFile(const std::string& path) {
        if (!fileExists(path)) {
            return std::unexpected(ParseError{"File not found"});
        }
        // Parse and return data
        return data;
    }
    
    // Caller propagates errors explicitly
    auto result = parseFile("level.rdl");
    if (!result) {
        std::println(stderr, "Error: {}", result.error().message);
        return std::unexpected(result.error());
    }
    Data data = *result;  // or result.value()
    ```
  - **NEVER** use exceptions: no `throw`, no `try`, no `catch`, no `noexcept`
  - **NEVER** use `-fno-exceptions` compiler flag (std::expected requires exception infrastructure)
  - **ALWAYS** handle errors at call sites (check return values)

- **STL Containers**: *Always* use modern STL containers
  - `std::vector` for dynamic arrays
  - `std::array` for fixed-size arrays
  - `std::string` and `std::string_view` for strings
  - `std::unordered_map` / `std::map` for associative containers
  - `std::optional` for nullable values
  - `std::variant` for sum types

- **Smart Pointers**: *Always* use smart pointers, *NEVER* raw pointers
  - `std::unique_ptr` for exclusive ownership
  - `std::shared_ptr` for shared ownership (use sparingly)
  - `std::weak_ptr` to break cycles
  - Raw pointers are **ONLY** allowed for non-owning references (prefer references instead)

- **C Library**: *NEVER* use C library functions unless ABSOLUTELY NECESSARY
  - No `malloc/free` (use smart pointers and containers)
  - No `printf/scanf` (use `<print>` and `<format>`)
  - No `strcpy/strcat` (use `std::string`)
  - No `memcpy` (use `std::copy` or container methods)
  - Exception: Low-level system calls or third-party C APIs only

- **Compile-Time Evaluation**: Always check if functions can be `constexpr` or `consteval`
  - Maximize compile-time computation
  - Use `constexpr` variables and functions where possible
  - Use `consteval` for functions that must be compile-time only

### Reference Code Guidelines

When working with D2X-XL reference code (`/tmp/d2x-xl-src/`):

- **DO NOT** document corresponding C/C++ source files in our codebase
- **DO** extract algorithms, data structures, and format specifications
- **DO** document file formats, protocols, and data layouts
- **DO** reference original file paths in comments for traceability (e.g., `// Corresponds to: include/piggy.h`)
- **DO NOT** port code line-by-line; rewrite in idiomatic Rust/C++23
- **DO NOT** preserve original code style or structure

## Code Quality Standards

### General Principles

1. **Clarity over cleverness**: Readable code is more important than clever code
2. **Explicit over implicit**: Make intentions clear, avoid magic
3. **Local over global**: Prefer local variables and parameters over global state
4. **Composition over inheritance**: Favor composition and traits/interfaces
5. **Fail fast**: Validate input early, return errors immediately
6. **Test coverage**: Every public API should have tests
7. **Documentation**: All public APIs must have doc comments with examples

### Performance Considerations

- Profile before optimizing
- Optimize hot paths identified by profiling data
- Document performance characteristics in comments
- Use appropriate data structures for access patterns
- Consider cache locality and memory layout

### Code Review Checklist

Before committing code, verify:

- [ ] Follows RDSS policy (Refactor, Despaghettify, Simplify, Split)
- [ ] Uses language-appropriate idioms (Rust 2024 / C++23)
- [ ] No raw pointers or C library usage in C++ (unless justified)
- [ ] Functions checked for `const` / `constexpr` where applicable
- [ ] Comprehensive error handling with proper error types
- [ ] All public APIs have documentation
- [ ] Unit tests cover functionality
- [ ] No compiler warnings
- [ ] Code is formatted (rustfmt / clang-format)
- [ ] Commit message is clear and descriptive

## Architecture Guidelines

### Bevy ECS Integration (Rust)

- Implement game systems as Bevy plugins
- Use ECS components for game state
- Leverage Bevy's scheduling and parallelism
- Keep systems small and focused
- Use events for cross-system communication

### Asset Loading Pipeline

- Parse assets in `d2x-assets` crate (pure data transformation)
- Load assets into Bevy in `d2x-engine` crate (game integration)
- Cache and stream assets efficiently
- Support hot-reloading for development

### Networking Architecture

- Client-server model with authoritative server
- Client-side prediction with server reconciliation
- Deterministic simulation for replay/demo support
- Modern protocols (TCP/IP, WebRTC) - no legacy IPX/serial

## Commit Guidelines

### Commit Message Format

```
Short summary (50 chars or less)

- Bullet point list of changes
- Each point describes one logical change
- Use present tense ("Add feature" not "Added feature")
- Include relevant file paths and function names

Closes #issue-number (if applicable)
```

### Commit Scope

- Each commit should be atomic and self-contained
- Related changes should be in the same commit
- Unrelated changes should be separate commits
- All commits should pass tests and build successfully

## Documentation Standards

### Code Documentation

- **Rust**: Use `///` for doc comments, include examples in doc tests
- **C++**: Use Doxygen-style comments with `@brief`, `@param`, `@return`
- Document **why**, not just **what**
- Include complexity analysis for algorithms (Big-O notation)
- Document thread-safety and const-correctness

### Format Documentation

When documenting file formats (HOG, PIG, HAM, RDL, etc.):

- Include byte-level layout tables
- Show example data and parsing code
- Document all known versions and variations
- Reference original source code locations
- Include known file sizes and checksums
- Explain endianness and alignment
- Document error conditions and edge cases

## Testing Strategy

### Test Pyramid

1. **Unit Tests** (most): Test individual functions and modules
2. **Integration Tests** (moderate): Test crate interfaces and subsystems
3. **End-to-End Tests** (few): Test complete features with real data

### Test Organization

- Unit tests in same file as implementation (`#[cfg(test)]` module)
- Integration tests in `tests/` directory
- Test fixtures in `test_data/` (gitignored)
- Use `#[should_panic]` for expected failures
- Use property-based testing for parsers (consider `proptest` crate)

## Development Workflow

1. **Plan**: Understand requirements and design approach
2. **Document**: Write or update format/architecture docs first
3. **Implement**: Write code following RDSS policy
4. **Test**: Write tests and verify functionality
5. **Review**: Self-review against checklist
6. **Commit**: Create atomic commit with clear message
7. **Verify**: Ensure `cargo test` and `cargo build` pass

## Questions or Clarifications

When uncertain about how to proceed:

1. Check this document first
2. Review existing code for patterns
3. Consult project documentation (README, ARCHITECTURE, FEATURES)
4. Ask the user for clarification
5. Document the decision for future reference

---

**Remember**: The goal is clean, maintainable, idiomatic code that will last for years. Take time to do it right the first time.

# D2X-RS: Descent Engine Rewrite in Rust

[![License](https://img.shields.io/badge/license-GPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)
[![Bevy](https://img.shields.io/badge/bevy-0.18-purple.svg)](https://bevyengine.org/)

A complete rewrite of the Descent 1, 2, and 3 game engines in Rust 2024 edition using the Bevy 0.18 game engine. This project reimplements the original low-level C code from D2X-XL and the Outrage Engine into Bevy's modern, high-level ECS architecture.

## Project Status

🚧 **Phase 1: Asset Parsers** - Active Development (Multi-Game Support)

Currently implementing asset parsing foundation with comprehensive documentation and idiomatic Rust code. Now adding Descent 3 format support alongside D1/D2.

### Current Progress

**✅ Completed:**
- [x] Complete architecture documentation
- [x] Cargo workspace structure (Rust 2024 edition, GPL-3.0)
- [x] Asset parsing crate foundation with split DHF/HOG2 modules
- [x] Engine crate structure
- [x] Client application skeleton
- [x] DHF archive format parser (D1/D2 with 13-char filenames)
- [x] HOG2 archive format parser (D3 with 36-char filenames, flags, timestamps)
- [x] PIG texture format parser (with RLE decompression)
- [x] HAM game data parser (textures, robots, weapons)
- [x] Palette handling (6-bit to 8-bit RGB conversion)
- [x] Level geometry parser (RDL/RL2 format for D1/D2)
- [x] OGF texture format parser (D3 with RGB565/RGBA4444/RGBA8888 support)
- [x] Comprehensive format documentation (DHF/HOG2, PIG, HAM, LEVEL, OGF, D3 overview)
- [x] POF model format parser (D1/D2 polygon models with 9 opcodes)
- [x] Sound format parsers (D1/D2: SNDs 8-bit PCM, HMP/MIDI music)
- [x] Mission file parser (D1/D2: .MSN and .MN2 text-based format)
- [x] Player profile parser (D1/D2: .PLR binary, D2X-XL: .PLX text)
- [x] Unit tests (92 tests passing: 6 DHF + 7 HOG2 + 7 OGF + 11 POF + 10 Sound + 13 Mission + 16 Player + 22 others)
- [x] Idiomatic Rust refactoring (traits, bitflags, enums)

**🚧 In Progress:**
- [ ] Additional level format features (D2X-XL extensions)

**📋 Next Up (D1/D2 First):**
- [ ] Integration tests with real game files
- [ ] Level rendering (Phase 2 - D1/D2 segment-based)
- [ ] Physics system (Phase 3 - D1/D2)
- [ ] D3 format support (later - D3L, OOF, OSF, GAM, MN3)

## Why Rewrite?

The existing Descent derivatives (D2X-XL, DXX-Rebirth) have several issues:

- **Outdated dependencies**: SDL1, old OpenGL patterns
- **Memory safety**: C/C++ prone to crashes and undefined behavior
- **Maintainability**: Large legacy codebase difficult to modify
- **Modern features**: Hard to add modern graphics and networking

D2X-RS addresses these by:

- ✅ **Modern Rust**: Memory safety, fearless concurrency
- ✅ **Bevy ECS**: Clean, modular game systems
- ✅ **Modern rendering**: PBR, HDR, advanced lighting
- ✅ **Better networking**: Client-server architecture, modern protocols
- ✅ **Extensibility**: Feature flags for derivative content

## Features

### Core Engine

**Primary Focus: Descent 1 & 2** (D3 support planned for later phases)

- **Descent 1 & 2 Support**: Full compatibility with original game data files
- **6DOF Flight**: Complete six degrees of freedom movement
- **Segment-Based Rendering**: Portal-based visibility culling with segment graph
- **Advanced Physics**: Collision detection, realistic movement
- **AI System**: Robot behaviors, pathfinding through segment connections
- **Weapons**: All primary and secondary weapons from D1/D2
- **Multiplayer**: Modern client-server networking
- **Descent 3**: Planned for future phases (room-based rendering, different AI)

### Enhancements (Feature-Gated)

- **D2X-XL Features**: Extended level limits (20k segments), custom game modes
- **Modern Graphics**: PBR materials, HDR rendering, per-pixel lighting
- **High-Res Assets**: Support for high-resolution textures and models
- **Advanced Networking**: NAT traversal, dedicated servers, master server

## Project Structure

```
d2x-rs/
├── crates/
│   ├── descent-core/       # Asset extraction (HOG/HOG2, PIG, HAM, POF, RDL/RL2, D3L, OGF, OOF)
│   ├── d2x-engine/       # Core engine (Bevy ECS systems)
│   └── d2x-client/       # Game client application
├── editor/               # Level editor (C++23/Qt6) - Phase 2
├── assets/              
│   └── shaders/         # Custom Bevy shaders
├── docs/                # Comprehensive documentation
│   ├── ARCHITECTURE.md  # Detailed architecture design
│   ├── FEATURES.md      # Feature flag documentation
│   ├── formats/         # File format specifications
│   │   ├── HOG_FORMAT.md    # D1/D2 (DHF) and D3 (HOG2)
│   │   ├── D3_FORMATS.md    # D3 format overview
│   │   └── ...
│   └── networking/      # Networking architecture
└── README.md
```

## Building

### Prerequisites

- **Rust 2024 edition** (rustc 1.82+)
- Cargo
- Descent 1 or 2 game files (for testing)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/yourusername/d2x-rs
cd d2x-rs

# Build all crates
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy -- -D warnings

# Run the client (currently minimal, asset parsing only)
cargo run --bin d2x-client
```

### Development Build

The project uses optimized development builds for faster iteration:

```bash
# Development build with deps optimized
cargo build

# Run tests with output
cargo test -- --nocapture

# Run specific crate tests
cargo test --package descent-core
```

### Feature Flags

Build with different feature sets:

```bash
# Note: Feature flag system is designed but not yet fully utilized
# Current build includes all asset parsers

# Feature examples:
cargo build --release --features base-d1          # D1 support only
cargo build --release --features base-d2          # D1+D2 support
cargo build --release --features base-d3          # D3 support only
cargo build --release --features base-d2,base-d3  # All games

# Future enhanced features:
# cargo build --release --features enhanced-graphics,hdr-rendering
# cargo build --release --features d2x-xl
```

See [docs/FEATURES.md](docs/FEATURES.md) for planned feature documentation.

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)**: Complete system architecture
  - Crate structure and organization
  - Core systems (physics, AI, rendering, networking)
  - ECS component design
  - Bevy integration patterns

- **[FEATURES.md](docs/FEATURES.md)**: Feature flag system (planned)
  - Feature combinations and presets
  - Graphics quality tiers
  - Gameplay modifications
  - Platform-specific features

- **File Format Specifications** (✅ D1/D2 Complete, 🚧 D3 In Progress):
  - **[formats/HOG_FORMAT.md](docs/formats/HOG_FORMAT.md)**: HOG/HOG2 archive format (D1/D2/D3)
  - **[formats/D3_FORMATS.md](docs/formats/D3_FORMATS.md)**: Descent 3 formats overview
  - **[formats/PIG_FORMAT.md](docs/formats/PIG_FORMAT.md)**: PIG texture format (D1/D2)
  - **[formats/HAM_FORMAT.md](docs/formats/HAM_FORMAT.md)**: HAM game data format (D1/D2)
  - **[formats/LEVEL_FORMAT.md](docs/formats/LEVEL_FORMAT.md)**: RDL/RL2 level format (D1/D2)
  - **[formats/POF_FORMAT.md](docs/formats/POF_FORMAT.md)**: POF polygon model format (D1/D2)
  - **[formats/SOUND_FORMAT.md](docs/formats/SOUND_FORMAT.md)**: SNDs and HMP audio formats (D1/D2)
  - **[formats/MISSION_FORMAT.md](docs/formats/MISSION_FORMAT.md)**: MSN/MN2 mission files (D1/D2)
  - **[formats/OGF_FORMAT.md](docs/formats/OGF_FORMAT.md)**: OGF texture format (D3)

- **[networking/ARCHITECTURE.md](docs/networking/ARCHITECTURE.md)**: Networking design (planned)
  - Client-server architecture
  - Prediction and reconciliation
  - LAN discovery
  - Master server integration

## Development Roadmap

### Phase 1: Asset Foundation (Months 1-2) - 🚧 IN PROGRESS

**✅ D1/D2 Core Formats:**
- [x] HOG archive parser (DHF format) with comprehensive tests
- [x] PIG texture extraction with RLE decompression
- [x] HAM game data parser (textures, robots, weapons, sounds)
- [x] Level geometry loader (RDL/RL2 format)
- [x] POF model parser (ships, robots, powerups) with 9 opcodes
- [x] Sound format parsers (SNDs 8-bit PCM, HMP/MIDI music)
- [x] Mission file parser (.MSN/.MN2 text-based format)
- [x] Palette handling (6-bit to 8-bit conversion)
- [x] Fixed-point math support (16.16 format)
- [x] 76 unit tests passing
- [x] Format documentation (3200+ lines across 9 format files)

**🚧 D1/D2 Remaining (Priority):**
- [ ] Savegame format parser
- [ ] Savegame format parser
- [ ] Integration tests with real game files

**✅ D3 Initial Support (Lower Priority):**
- [x] HOG2 archive parser (separate module: hog2.rs)
- [x] 36-char filenames, flags, and timestamps
- [x] Split architecture: dhf.rs (D1/D2) and hog2.rs (D3)
- [x] D3 formats overview documentation
- [x] 7 comprehensive HOG2 unit tests
- [x] OGF texture format parser (RGB565/RGBA4444/RGBA8888 support)
- [x] 7 comprehensive OGF unit tests with bit-accurate color conversion

**📋 D3 Remaining (Future):**
- [ ] D3L level format (room-based geometry)
- [ ] OOF model format (Outrage Object)
- [ ] OSF sound format (Outrage Sound)
- [ ] GAM game data tables
- [ ] MN3 mission files

### Phase 2: Level Rendering (Month 3) - 📋 PLANNED
**Focus: Descent 1/2 segment-based rendering first**
- [ ] Segment mesh generation from D1/D2 level geometry
- [ ] Basic camera and free-flight controls
- [ ] Texture rendering (PIG textures with palette)
- [ ] POF model rendering (ships, robots, powerups)
- [ ] Portal culling system (segment-based)
- [ ] Basic lighting
- [ ] (Later) Room mesh generation from D3 level geometry

### Phase 3: Physics (Month 4) - 📋 PLANNED
**Focus: Descent 1/2 physics first**
- [ ] 6DOF physics system (six degrees of freedom)
- [ ] Collision detection with D1/D2 level geometry
- [ ] Wall collision and sliding
- [ ] Player ship controls and movement

### Phase 4: Objects & Gameplay (Months 5-6)
**Focus: Descent 1/2 gameplay first**
- [ ] Player ship (D1/D2)
- [ ] Robot objects with POF models
- [ ] Powerups
- [ ] Basic weapons (primary and secondary)
- [ ] HUD display
- [ ] (Later) D3 player and enemies

### Phase 5: AI & Combat (Months 7-8)
**Focus: Descent 1/2 AI first**
- [ ] AI behavior system (D1/D2 robot AI)
- [ ] Pathfinding through segment graph
- [ ] Combat mechanics
- [ ] All weapon types (D1/D2)
- [ ] Particle effects
- [ ] (Later) D3 AI and combat

### Phase 6: Game Client (Month 9)
- [ ] Menu system
- [ ] Mission selection
- [ ] Save/load
- [ ] Single-player progression

### Phase 7: Multiplayer (Months 10-12)
- [ ] Networking foundation
- [ ] Client-server architecture
- [ ] LAN discovery
- [ ] Multiplayer game modes
- [ ] Server browser

### Phase 8: Polish (Months 13-14)
- [ ] Audio system complete
- [ ] Visual effects polish
- [ ] Performance optimization
- [ ] Bug fixes

### Phase 9: Level Editor (Months 15-17)
- [ ] Qt6 application
- [ ] Segment editor
- [ ] Texture browser
- [ ] Object placement
- [ ] Export to RDL

### Phase 10: D2X-XL Features (Months 18+)
- [ ] Feature-gated enhancements
- [ ] Entropy mode
- [ ] Monsterball mode
- [ ] Advanced graphics options

## Contributing

Contributions are welcome! This project follows the **RDSS** development policy:

### RDSS Policy: Refactor, Despaghettify, Simplify, Split

When porting code from D2X-XL:

1. **Refactor**: Use idiomatic Rust patterns
   - Prefer `const fn` where possible
   - Use `Result<T, E>` with `?` operator
   - Never `unwrap()` in production code
   - Use traits (`From`, `Into`) instead of manual conversion methods

2. **Despaghettify**: Break down complex logic
   - Replace deep nesting with early returns
   - Extract helper functions
   - Use enums and pattern matching

3. **Simplify**: Focus on clarity
   - Document formats, not C code translations
   - Use descriptive variable names
   - Remove unnecessary complexity

4. **Split**: Modular organization
   - Separate concerns into modules
   - Keep files under ~1000 lines
   - Use `bitflags` crate for flag types

### Code Style

- Follow Rust 2024 edition conventions
- Run `cargo fmt` and `cargo clippy -- -D warnings`
- Document all public APIs with rustdoc
- Include references to original D2X-XL code locations
- Write comprehensive tests for parsing code
- Document file formats with examples and diagrams

## Original Source Reference

This rewrite is based on D2X-XL version 1.18.77, with references to:

- **D2X-XL**: Enhanced Descent engine by Diedel
- **DXX-Rebirth**: Actively maintained D1X/D2X ports
- **Original Descent**: Released under Parallax license

### License Note

This project is licensed under GPL-3.0 to comply with the D2X-XL source code license terms. The original Descent source code is covered by the Parallax Software license.

## Requirements

### To Build
- Rust toolchain 1.82+ (Rust 2024 edition)
- Cargo
- ~2GB disk space for dependencies

### To Run Tests
- Original Descent 1 or 2 game files (optional, for integration tests)
- Available legally from:
  - [GOG.com](https://www.gog.com/game/descent)
  - [Steam](https://store.steampowered.com/app/273570/Descent/)

### System Requirements (Target)
- **CPU**: Dual-core 2GHz+
- **RAM**: 4GB+
- **GPU**: OpenGL 4.5 or Vulkan support (planned)
- **OS**: Windows, Linux, macOS

## Acknowledgments

- **Parallax Software**: Original Descent creators
- **Diedel**: D2X-XL enhancements
- **DXX-Rebirth Team**: Modern port maintenance
- **Bevy Community**: Excellent game engine
- **Rust Community**: Amazing tooling and libraries

## Related Projects

- [DXX-Rebirth](https://github.com/dxx-rebirth/dxx-rebirth) - Active D1X/D2X ports
- [D2X-XL](http://www.descent2.de/) - Enhanced Descent engine
- [Bevy Engine](https://bevyengine.org/) - Modern Rust game engine

## Contact

- **Issues**: GitHub issue tracker
- **Discussions**: GitHub discussions
- **Discord**: (Coming soon)

## License

This project is licensed under **GPL-3.0** ([LICENSE](LICENSE) or https://www.gnu.org/licenses/gpl-3.0.html)

The original Descent source code is covered by the Parallax Software license.
This rewrite is based on D2X-XL version 1.18.77, which is GPL-licensed.

---

**Status**: Phase 1 - Asset Parsers (Active Development) | **Version**: 0.1.0 | **Last Updated**: 2026-02-23

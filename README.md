# D2X-RS: Descent Engine Rewrite in Rust

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Bevy](https://img.shields.io/badge/bevy-0.14-purple.svg)](https://bevyengine.org/)

A complete rewrite of the Descent 1 and 2 game engine in Rust 2024 edition using the Bevy 0.18 game engine. This project reimplements the original low-level C code from D2X-XL into Bevy's modern, high-level ECS architecture.

## Project Status

🚧 **Early Development** - Foundation and documentation phase

This project is currently in its initial planning and architecture phase. The comprehensive documentation is complete, and the basic project structure has been established.

### Current Progress

- [x] Complete architecture documentation
- [x] Cargo workspace structure
- [x] Asset parsing crate foundation (HOG file format)
- [x] Engine crate structure
- [x] Client application skeleton
- [x] Feature flag design
- [x] Network architecture design
- [ ] Asset parsers (PIG, HAM, RDL)
- [ ] Level rendering
- [ ] Physics system
- [ ] Gameplay systems

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

- **Descent 1 & 2 Support**: Full compatibility with original game data files
- **6DOF Flight**: Complete six degrees of freedom movement
- **Portal-Based Rendering**: Optimized visibility culling
- **Advanced Physics**: Collision detection, realistic movement
- **AI System**: Robot behaviors, pathfinding through segment graphs
- **Weapons**: All primary and secondary weapons from D1/D2
- **Multiplayer**: Modern client-server networking

### Enhancements (Feature-Gated)

- **D2X-XL Features**: Extended level limits (20k segments), custom game modes
- **Modern Graphics**: PBR materials, HDR rendering, per-pixel lighting
- **High-Res Assets**: Support for high-resolution textures and models
- **Advanced Networking**: NAT traversal, dedicated servers, master server

## Project Structure

```
d2x-rs/
├── crates/
│   ├── d2x-assets/       # Asset extraction (HOG, PIG, HAM, RDL)
│   ├── d2x-engine/       # Core engine (Bevy ECS systems)
│   └── d2x-client/       # Game client application
├── editor/               # Level editor (C++23/Qt6) - Phase 2
├── assets/              
│   └── shaders/         # Custom Bevy shaders
├── docs/                # Comprehensive documentation
│   ├── ARCHITECTURE.md  # Detailed architecture design
│   ├── FEATURES.md      # Feature flag documentation
│   ├── formats/         # File format specifications
│   └── networking/      # Networking architecture
└── README.md
```

## Building

### Prerequisites

- Rust 1.75 or later
- Cargo
- Descent 1 or 2 game files (for testing)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/yourusername/d2x-rs
cd d2x-rs

# Build all crates
cargo build --release

# Run the client (currently minimal)
cargo run --bin d2x-client
```

### Feature Flags

Build with different feature sets:

```bash
# Minimal build (D1/D2 only)
cargo build --release --no-default-features --features base-d2

# Enhanced graphics
cargo build --release --features enhanced-graphics,hdr-rendering

# D2X-XL full experience
cargo build --release --features d2x-xl

# Custom combination
cargo build --release --features "base-d2,pbr-materials,lan-discovery"
```

See [docs/FEATURES.md](docs/FEATURES.md) for complete feature documentation.

## Documentation

Comprehensive documentation is available in the `docs/` directory:

- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)**: Complete system architecture
  - Crate structure and organization
  - Core systems (physics, AI, rendering, networking)
  - ECS component design
  - Bevy integration patterns

- **[FEATURES.md](docs/FEATURES.md)**: Feature flag system
  - Feature combinations and presets
  - Graphics quality tiers
  - Gameplay modifications
  - Platform-specific features

- **[formats/HOG_FORMAT.md](docs/formats/HOG_FORMAT.md)**: HOG archive format
  - File structure specification
  - Parsing algorithms
  - Performance considerations

- **[networking/ARCHITECTURE.md](docs/networking/ARCHITECTURE.md)**: Networking design
  - Client-server architecture
  - Prediction and reconciliation
  - LAN discovery
  - Master server integration

## Development Roadmap

### Phase 1: Asset Foundation (Months 1-2)
- [ ] Complete HOG archive parser
- [ ] Implement PIG texture extraction
- [ ] HAM game data parser
- [ ] Level geometry loader (RDL/RL2)
- [ ] Unit tests with real D1/D2 files

### Phase 2: Level Rendering (Month 3)
- [ ] Segment mesh generation
- [ ] Basic camera and free-flight
- [ ] Texture rendering
- [ ] Portal culling

### Phase 3: Physics (Month 4)
- [ ] 6DOF physics system
- [ ] Collision detection
- [ ] Wall collision
- [ ] Player ship controls

### Phase 4: Objects & Gameplay (Months 5-6)
- [ ] Player object
- [ ] Robot objects
- [ ] Powerups
- [ ] Basic weapons
- [ ] HUD display

### Phase 5: AI & Combat (Months 7-8)
- [ ] AI behavior system
- [ ] Pathfinding
- [ ] Combat mechanics
- [ ] All weapon types
- [ ] Particle effects

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

Contributions are welcome! Please read our contributing guidelines (coming soon) before submitting PRs.

### Code Organization

- **d2x-assets**: Pure Rust, no game logic dependencies
- **d2x-engine**: Bevy plugins and systems
- **d2x-client**: Application-level code

### Code Style

- Follow Rust standard conventions
- Document all public APIs
- Include references to original D2X-XL code locations
- Write tests for parsing code

## Original Source Reference

This rewrite is based on D2X-XL version 1.18.77, with references to:

- **D2X-XL**: Enhanced Descent engine by Diedel
- **DXX-Rebirth**: Actively maintained D1X/D2X ports
- **Original Descent**: Released under Parallax license

### License Note

This project respects the Parallax Software license terms from the original Descent source code release. The Rust rewrite is licensed under MIT/Apache-2.0.

## Requirements

### To Build
- Rust toolchain 1.75+
- ~4GB disk space for dependencies

### To Play
- Original Descent 1 or 2 game files (descent.hog, descent2.hog, etc.)
- Available legally from:
  - [GOG.com](https://www.gog.com/game/descent)
  - [Steam](https://store.steampowered.com/app/273570/Descent/)

### System Requirements (Target)
- **CPU**: Dual-core 2GHz+
- **RAM**: 4GB+
- **GPU**: OpenGL 4.5 or Vulkan support
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

This project is dual-licensed under:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

The original Descent source code is covered by the Parallax Software license.

---

**Status**: Early Development | **Version**: 0.1.0 | **Last Updated**: 2026-02-23

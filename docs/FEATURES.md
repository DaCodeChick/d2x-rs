# Feature Flags Design

## Overview

D2X-RS uses Cargo feature flags to provide flexibility in which game features are compiled into the binary. This allows:

1. **Faithful Recreation**: Build just base Descent 1/2 functionality
2. **Enhanced Experience**: Add D2X-XL enhancements
3. **Performance Tuning**: Choose graphics quality levels
4. **Compatibility**: Support different derivative ports
5. **Binary Size**: Exclude unused features

---

## Default Feature Set

```toml
[features]
default = ["base-d1", "base-d2", "lan-discovery"]
```

Minimum viable game with both D1 and D2 support, plus LAN multiplayer.

---

## Base Game Features

### `base-d1`
Descent 1 content support

**Includes**:
- D1 level format (RDL) parsing
- D1 HAM file support
- D1 robot definitions
- D1 weapons (laser, vulcan, spreadfire, plasma, fusion)
- D1 textures and sounds
- 7 primary weapons, 5 secondary weapons
- Original 800 segments per level limit

### `base-d2`
Descent 2 content support (implies `base-d1`)

```toml
base-d2 = ["base-d1"]
```

**Adds**:
- D2 level format (RL2) parsing
- Enhanced HAM with additional robots
- D2 weapons (gauss, helix, phoenix, omega)
- Afterburner system
- Guide-bot (buddy)
- Thief robot
- 900 segments per level limit
- Super laser (quad/quad-quad)

---

## Derivative Port Features

### `d2x-xl`
D2X-XL enhancements

```toml
d2x-xl = [
    "enhanced-graphics",
    "extended-limits",
    "custom-weapons",
    "advanced-game-modes",
    "per-pixel-lighting"
]
```

**Adds**:
- 20,000 segment limit
- High-resolution model support (OOF format)
- Entropy game mode
- Monsterball game mode
- Lightning cannon
- Slow motion effects
- Advanced particle systems
- Custom sound effects (43 additional sounds)
- Radar system
- Headlight enhancements

### `dxx-rebirth`
DXX-Rebirth specific features

```toml
dxx-rebirth = ["rebirth-improvements"]
```

**Adds**:
- Rebirth-specific bug fixes
- Enhanced multiplayer code
- Improved OpenGL rendering
- Modern control improvements

---

## Graphics Features

### Quality Tiers

#### `classic-graphics` (default if no graphics features enabled)
Original Descent look

**Includes**:
- Per-vertex lighting
- Original texture quality
- Classic particle effects
- Software-style rendering

#### `enhanced-graphics`
Improved visual quality

```toml
enhanced-graphics = [
    "per-pixel-lighting",
    "advanced-particles",
    "dynamic-shadows"
]
```

**Includes**:
- Per-pixel lighting calculations
- Enhanced particle systems (smoke, sparks, explosions)
- Dynamic shadow mapping
- Improved transparency rendering

#### `pbr-materials`
Modern PBR rendering

**Includes**:
- Physically-based materials
- Normal mapping support
- Metallic/roughness workflows
- Environment mapping

#### `hdr-rendering`
High dynamic range

**Includes**:
- HDR framebuffer
- Bloom effects
- Tonemapping
- Exposure control

---

### Texture Features

#### `hires-textures`
High-resolution texture support

**Includes**:
- 2x/4x texture scaling
- Replacement texture loading
- Texture filtering options
- Mipmap generation

#### `texture-packs`
Support for community texture packs

**Includes**:
- Custom texture directory scanning
- Texture override system
- PNG/TGA/DDS texture loading

---

### Lighting Features

#### `per-pixel-lighting`
Fragment shader lighting (vs per-vertex)

**Enables**:
- Smooth lighting gradients
- Detailed light/shadow boundaries
- Normal map support

#### `dynamic-shadows`
Real-time shadow mapping

**Enables**:
- Shadow map generation
- PCF shadow filtering
- Multiple light shadows

#### `lightmaps`
Precomputed lightmap support

**Enables**:
- Baked lighting
- Faster rendering
- Consistent lighting

---

## Gameplay Features

### `extended-limits`
Increased game limits beyond original D2

```rust
#[cfg(feature = "extended-limits")]
const MAX_SEGMENTS: usize = 20000;

#[cfg(not(feature = "extended-limits"))]
const MAX_SEGMENTS: usize = 900;
```

**Increases**:
- Segments: 900 → 20,000
- Objects: 350 → 2,000
- Triggers: 100 → 1,000
- Walls: 175 → 2,000

### `custom-weapons`
Additional weapon types

**Adds**:
- Lightning cannon (D2X-XL)
- Custom projectile types
- Weapon modding support

### `advanced-game-modes`
Extended multiplayer modes

```toml
advanced-game-modes = ["entropy-mode", "monsterball-mode"]
```

#### `entropy-mode`
Team-based virus infection mode

#### `monsterball-mode`
Soccer-style multiplayer with ball physics

---

## Networking Features

### `lan-discovery` (default)
Zero-config LAN multiplayer

**Enables**:
- UDP broadcast server discovery
- Automatic server listing
- Quick join

### `master-server`
Internet game browser

**Enables**:
- Master server client
- Server registration
- Global server list

### `dedicated-server`
Headless server support

**Enables**:
- No graphics initialization
- Console-only interface
- Reduced memory footprint

---

## Audio Features

### `audio-3d`
Positional audio

**Enables**:
- 3D sound positioning
- Doppler effect
- Reverb/environment effects

### `hires-sounds`
High-quality audio

**Enables**:
- 44.1kHz+ sample rates
- 16-bit audio
- OGG/FLAC support

### `music-streaming`
Streamed music playback

**Enables**:
- MP3/OGG music
- CD audio replacement
- Custom soundtracks

---

## Performance Features

### `multithreading`
Multi-threaded rendering and physics

**Enables**:
- Parallel AI processing
- Threaded physics
- Render job distribution

### `simd`
SIMD optimizations

**Enables**:
- Vectorized math operations
- Faster collision detection
- Optimized rendering

---

## Development Features

### `debug-rendering`
Visual debugging aids

**Enables**:
- Wireframe mode
- Portal visualization
- Collision bounds display
- AI pathfinding display
- Performance overlays

### `editor-support`
Level editor integration

**Enables**:
- Runtime level reloading
- Developer console
- Entity inspector

### `profiling`
Performance profiling

**Enables**:
- Frame time tracking
- System profiling
- Memory allocation tracking

---

## Platform Features

### `native`
Native platform features

**Enables**:
- OS-specific file dialogs
- Native notifications
- Platform integrations

### `wasm`
WebAssembly build support

**Enables**:
- Browser compatibility
- Web networking
- IndexedDB saves

---

## Feature Combinations

### Presets

#### Minimal Build
```toml
[features]
minimal = ["base-d2"]
```

Smallest possible binary, base D1/D2 only.

#### Standard Build
```toml
[features]
default = ["base-d2", "lan-discovery", "hires-textures"]
```

Good balance of features and compatibility.

#### Enhanced Build
```toml
[features]
enhanced = [
    "base-d2",
    "enhanced-graphics",
    "hires-textures",
    "lan-discovery",
    "master-server",
    "audio-3d"
]
```

Modern experience with quality improvements.

#### D2X-XL Build
```toml
[features]
d2x-xl-full = [
    "d2x-xl",
    "pbr-materials",
    "hdr-rendering",
    "hires-textures",
    "advanced-game-modes",
    "master-server"
]
```

Complete D2X-XL experience with all enhancements.

---

## Usage Examples

### Building with Features

```bash
# Default build
cargo build --release

# Minimal build
cargo build --release --no-default-features --features minimal

# D2X-XL full experience
cargo build --release --features d2x-xl-full

# Custom combination
cargo build --release --features "base-d2,pbr-materials,lan-discovery"
```

### Runtime Feature Detection

```rust
pub fn get_enabled_features() -> Vec<&'static str> {
    let mut features = vec![];
    
    #[cfg(feature = "base-d1")]
    features.push("Descent 1 Support");
    
    #[cfg(feature = "base-d2")]
    features.push("Descent 2 Support");
    
    #[cfg(feature = "d2x-xl")]
    features.push("D2X-XL Enhancements");
    
    #[cfg(feature = "pbr-materials")]
    features.push("PBR Rendering");
    
    #[cfg(feature = "hdr-rendering")]
    features.push("HDR");
    
    features
}
```

### Conditional Compilation

```rust
// Different segment limits
#[cfg(feature = "extended-limits")]
pub const MAX_SEGMENTS: usize = 20000;

#[cfg(all(feature = "base-d2", not(feature = "extended-limits")))]
pub const MAX_SEGMENTS: usize = 900;

#[cfg(all(not(feature = "base-d2"), feature = "base-d1"))]
pub const MAX_SEGMENTS: usize = 800;

// Feature-specific systems
#[cfg(feature = "entropy-mode")]
mod entropy {
    pub fn setup_entropy_mode(app: &mut App) {
        app.add_systems(Update, entropy_system);
    }
}

// Graphics quality
pub fn setup_materials(
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    #[cfg(feature = "pbr-materials")]
    {
        // PBR material setup with normal maps
    }
    
    #[cfg(not(feature = "pbr-materials"))]
    {
        // Classic unlit materials
    }
}
```

---

## Feature Compatibility Matrix

| Feature | base-d1 | base-d2 | d2x-xl | dxx-rebirth |
|---------|---------|---------|--------|-------------|
| RDL levels | ✓ | ✓ | ✓ | ✓ |
| RL2 levels | - | ✓ | ✓ | ✓ |
| 800 segments | ✓ | - | - | - |
| 900 segments | - | ✓ | - | ✓ |
| 20k segments | - | - | ✓ | - |
| Afterburner | - | ✓ | ✓ | ✓ |
| Guide-bot | - | ✓ | ✓ | ✓ |
| Entropy mode | - | - | ✓ | - |
| Monsterball | - | - | ✓ | - |

---

## Configuration File

Users can override features at runtime via config:

```toml
# config.toml

[graphics]
# Override feature defaults
rendering_mode = "pbr"  # "classic" | "pbr"
lighting_mode = "per-pixel"  # "per-vertex" | "per-pixel"
texture_quality = "high"  # "original" | "high"
hdr = true
shadows = true

[gameplay]
segment_limit = 20000  # If extended-limits feature enabled
extended_content = true  # Load D2X-XL content if available

[network]
lan_discovery = true
master_server = true
```

---

## Binary Size Impact

Estimated binary size increases:

| Feature Set | Binary Size | Description |
|-------------|-------------|-------------|
| Minimal | ~15 MB | Base game only |
| Default | ~25 MB | Standard features |
| Enhanced | ~35 MB | All graphics features |
| D2X-XL Full | ~45 MB | Complete feature set |
| With Debug | +10 MB | Debug rendering |

---

## Testing All Features

```bash
# Test all feature combinations
cargo test --all-features

# Test specific combination
cargo test --features "base-d2,pbr-materials"

# Test without default features
cargo test --no-default-features --features minimal
```

---

## Documentation

Generate docs for specific features:

```bash
# Document all features
cargo doc --all-features --open

# Document specific build
cargo doc --features d2x-xl-full --open
```

---

## Future Features

Planned for future releases:

### `vr-support`
Virtual reality rendering

### `raytracing`
Hardware raytracing (RTX)

### `mod-api`
Lua/WASM scripting for mods

### `level-streaming`
Dynamic level loading for huge levels

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-23

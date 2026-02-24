# HAM File Format Specification

## Overview

HAM (Hamster) files contain game data definitions for Descent 1 and Descent 2. They define properties for textures, robots, weapons, powerups, models, effects, and other game objects. HAM files work in conjunction with PIG files (which contain actual bitmap/texture data) and separate palette files (`.256` extension).

**Reference Implementation**: D2X-XL v1.18.77
- `include/piggy.h` - Constants and structures
- `include/loadgamedata.h` - Data structure definitions
- `gameio/piggy.cpp:430` - ReadHamFile function
- `gameio/loadgamedata.cpp:276` - BMReadAll function (main parser)

## File Metadata

- **Magic Signature**: `HAM!` (0x48414D21 = `MAKE_SIG('!','M','A','H')`)
- **Versions**:
  - Version 1: Original (obsolete)
  - Version 2: Demo version (saves marker_model_num)
  - Version 3: Standard version (removed sound files from HAM)
- **Endianness**: Little-endian
- **Extension**: `.ham`
- **Common Files**: 
  - `descent2.ham` (Descent 2)
  - `descent.ham` (Descent 1)

## Version Differences

| Feature | Version 2 (Demo) | Version 3 (Standard) |
|---------|------------------|----------------------|
| Sound Data | Embedded in HAM | External `.s11`/`.s22` files |
| Exit Models | 2 exit model numbers stored | Uses last poly model index |
| Marker Model | Stored | Stored |

## HAM File Structure

All integers are stored as **little-endian** unless otherwise noted.

```
┌─────────────────────────────────────────────────────────────┐
│ HAM FILE                                                    │
├─────────────────────────────────────────────────────────────┤
│ Header (8 bytes)                                            │
│   ├─ Signature: "HAM!" (4 bytes)                            │
│   └─ Version: int32 (4 bytes)                               │
├─────────────────────────────────────────────────────────────┤
│ [Version < 3 only] Sound Offset: int32 (4 bytes)           │
├─────────────────────────────────────────────────────────────┤
│ TEXTURE DATA                                                │
│   ├─ Texture Count: int32                                   │
│   ├─ Bitmap Indices: tBitmapIndex[count]                    │
│   └─ Texture Map Info: tTexMapInfo[count]                   │
├─────────────────────────────────────────────────────────────┤
│ SOUND INDICES                                               │
│   ├─ Sound Count: int32                                     │
│   ├─ Sound Indices: uint8[count]                            │
│   └─ Alt Sound Indices: uint8[count]                        │
├─────────────────────────────────────────────────────────────┤
│ VIDEO CLIPS (VCLIPS)                                        │
│   ├─ VClip Count: int32                                     │
│   └─ VClip Definitions: tAnimationInfo[count]               │
├─────────────────────────────────────────────────────────────┤
│ EFFECTS                                                     │
│   ├─ Effect Count: int32                                    │
│   └─ Effect Info: tEffectInfo[count]                        │
├─────────────────────────────────────────────────────────────┤
│ WALL ANIMATIONS                                             │
│   ├─ Wall Anim Count: int32                                 │
│   └─ Wall Effect Info: tWallEffect[count]                   │
├─────────────────────────────────────────────────────────────┤
│ ROBOT DEFINITIONS                                           │
│   ├─ Robot Count: int32                                     │
│   ├─ Robot Info: tRobotInfo[count]                          │
│   ├─ Joint Count: int32                                     │
│   └─ Joint Positions: tJointPos[joint_count]                │
├─────────────────────────────────────────────────────────────┤
│ WEAPON DEFINITIONS                                          │
│   ├─ Weapon Count: int32                                    │
│   └─ Weapon Info: CWeaponInfo[count]                        │
├─────────────────────────────────────────────────────────────┤
│ POWERUP DEFINITIONS                                         │
│   ├─ Powerup Count: int32                                   │
│   └─ Powerup Type Info: tPowerupTypeInfo[count]             │
├─────────────────────────────────────────────────────────────┤
│ POLYGON MODELS                                              │
│   ├─ Model Count: int32                                     │
│   ├─ Poly Model Info: CPolyModel[count]                     │
│   ├─ Model Data (vertex/polygon data for each model)        │
│   ├─ Dying Models: int32[count]                             │
│   └─ Dead Models: int32[count]                              │
├─────────────────────────────────────────────────────────────┤
│ COCKPIT GAUGES                                              │
│   ├─ Gauge Count: int32                                     │
│   ├─ Gauge Bitmap Indices (Hires): tBitmapIndex[count]      │
│   └─ Gauge Bitmap Indices (Lores): tBitmapIndex[count]      │
├─────────────────────────────────────────────────────────────┤
│ OBJECT BITMAPS                                              │
│   ├─ Object Bitmap Count: int32                             │
│   ├─ Object Bitmap Indices: tBitmapIndex[count]             │
│   └─ Object Bitmap Pointers: int16[count]                   │
├─────────────────────────────────────────────────────────────┤
│ PLAYER SHIP                                                 │
│   └─ Player Ship Data: tPlayerShip (132 bytes)              │
├─────────────────────────────────────────────────────────────┤
│ COCKPIT BITMAPS                                             │
│   ├─ Cockpit Count: int32                                   │
│   ├─ Cockpit Bitmap Indices: tBitmapIndex[count]            │
│   └─ First Multi-Bitmap Index: int32                        │
├─────────────────────────────────────────────────────────────┤
│ REACTORS                                                    │
│   ├─ Reactor Count: int32                                   │
│   └─ Reactor Info: tReactorInfo[count]                      │
├─────────────────────────────────────────────────────────────┤
│ MISC DATA                                                   │
│   ├─ Marker Model Number: int32                             │
│   └─ [Version < 3 only]                                     │
│       ├─ Exit Model Number: int32                           │
│       └─ Destroyed Exit Model Number: int32                 │
├─────────────────────────────────────────────────────────────┤
│ BITMAP INDEX TRANSLATION TABLE                              │
│   └─ Bitmap Xlat: uint8[MAX_BITMAP_FILES]                   │
├─────────────────────────────────────────────────────────────┤
│ [Version < 3 only] EMBEDDED SOUND DATA                      │
│   └─ (See sound offset from header)                         │
└─────────────────────────────────────────────────────────────┘
```

## Data Structures

### Header

```rust
struct HamHeader {
    signature: [u8; 4],  // "HAM!" = [0x21, 0x4D, 0x41, 0x48]
    version: i32,        // 2 or 3
}
```

If version < 3, immediately after header:
```rust
sound_offset: i32  // File offset to sound data section
```

### tBitmapIndex

```rust
struct BitmapIndex {
    index: u16,  // Index into PIG file bitmap array
}
```

Size: 2 bytes

### tTexMapInfo (Descent 2)

Defines properties for wall textures.

```rust
struct TexMapInfo {
    flags: u8,           // Texture flags (see below)
    pad: [u8; 3],        // Padding for alignment
    lighting: f32,       // Fixed-point lighting value
    damage: f32,         // Fixed-point damage value
    effect_clip: i32,    // Effect clip index (-1 = none)
    destroyed: i16,      // Destroyed texture index (-1 = none)
    slide_u: i16,        // U texture coordinate slide rate
    slide_v: i16,        // V texture coordinate slide rate
}
```

Size: 20 bytes

**Texture Flags** (`flags` field):
- `0x01` - Volatile (can be destroyed)
- `0x02` - Water texture
- `0x04` - Force field
- `0x08` - Goal (CTF goals, etc.)
- `0x10` - Animates (linked to effect clip)

### tTexMapInfoD1 (Descent 1)

```rust
struct TexMapInfoD1 {
    filename: [u8; 13],  // Texture filename (not used at runtime)
    flags: u8,
    lighting: f32,       // Fixed-point
    damage: f32,         // Fixed-point
    effect_clip: i32,
}
```

Size: 26 bytes

### tAnimationInfo (VClip)

Video clips for animations (explosions, effects, etc.).

```rust
struct AnimationInfo {
    total_time: f32,         // Fixed-point total animation time
    num_frames: i32,         // Number of frames
    frame_time: f32,         // Fixed-point time per frame
    flags: i32,              // Animation flags
    sound_num: i16,          // Sound to play (-1 = none)
    frame_indices: [i16; MAX_VCLIP_FRAMES],  // Bitmap indices
    light_value: f32,        // Fixed-point light intensity
}
```

MAX_VCLIP_FRAMES = 30 for Descent 2, 20 for Descent 1

### tEffectInfo

```rust
struct EffectInfo {
    vclip_num: i32,              // VClip index for effect
    time_left: f32,              // Fixed-point time remaining
    frame_count: i32,            // Current frame number
    changing_wall_texture: i16,  // Wall texture being changed
    changing_object_texture: i16, // Object texture being changed
    flags: i32,                  // Effect flags
    crit_clip: i32,              // Critical effect clip
    dest_bm_num: i32,            // Destroyed bitmap number
    dest_vclip: i32,             // Destroyed VClip
    dest_eclip: i32,             // Destroyed effect clip
    dest_size: f32,              // Fixed-point destroyed size
    sound_num: i16,              // Sound effect
    segment_num: i16,            // Segment where effect occurs
    side_num: i16,               // Side of segment
    dest_orient: [[f32; 3]; 3],  // Destroyed orientation matrix
}
```

### tWallEffect

Wall animations (opening/closing doors, etc.).

```rust
struct WallEffect {
    total_time: f32,                             // Fixed-point
    num_frames: i16,
    frames: [i16; MAX_WALL_EFFECT_FRAMES],      // Texture indices
    open_sound: i16,
    close_sound: i16,
    flags: i16,
    filename: [u8; 13],
    pad: u8,
}
```

MAX_WALL_EFFECT_FRAMES = 20 (D2), 12 (D1)

### tRobotInfo

Robot/enemy definitions.

```rust
struct RobotInfo {
    model_num: i32,                           // 3D model index
    gun_points: [Vector3; MAX_GUNS],          // Gun positions (8 max)
    gun_submodels: [u8; MAX_GUNS],            // Submodel indices
    exp1_vclip_num: i16,                      // Explosion 1 VClip
    exp1_sound_num: i16,                      // Explosion 1 sound
    exp2_vclip_num: i16,                      // Explosion 2 VClip
    exp2_sound_num: i16,                      // Explosion 2 sound
    weapon_type: i8,                          // Primary weapon
    weapon_type2: i8,                         // Secondary weapon
    n_guns: i8,                               // Number of guns
    contains_id: i8,                          // Powerup contained
    contains_count: i8,                       // Powerup count
    contains_prob: i8,                        // Probability of drop
    contains_type: i8,                        // Powerup type
    kamikaze: i8,                             // Kamikaze behavior
    score_value: i16,                         // Points awarded
    badass: i8,                               // Explosion damage multiplier
    energy_drain: i8,                         // Energy drain amount
    lighting: f32,                            // Fixed-point light value
    strength: f32,                            // Fixed-point hit points
    mass: f32,                                // Fixed-point mass
    drag: f32,                                // Fixed-point drag coefficient
    
    // AI parameters per difficulty level (5 levels)
    field_of_view: [f32; 5],                  // Fixed-point FOV
    firing_wait: [f32; 5],                    // Fixed-point fire delay
    firing_wait2: [f32; 5],                   // Fixed-point secondary fire
    turn_time: [f32; 5],                      // Fixed-point turn speed
    max_speed: [f32; 5],                      // Fixed-point max velocity
    circle_distance: [f32; 5],                // Fixed-point circle dist
    rapidfire_count: [i8; 5],                 // Rapid fire shots
    evade_speed: [i8; 5],                     // Evasion speed
    
    cloak_type: i8,                           // Cloaking behavior
    attack_type: i8,                          // Attack pattern
    see_sound: u8,                            // Sound when player seen
    attack_sound: u8,                         // Sound when attacking
    claw_sound: u8,                           // Melee sound
    taunt_sound: u8,                          // Taunt sound
    boss_flag: i8,                            // Boss indicator
    companion: i8,                            // Companion bot flag
    smart_blobs_on_death: i8,                 // Smart mine count on death
    smart_blobs_on_hit: i8,                   // Smart mine count on hit
    thief: i8,                                // Thief bot flag
    pursuit: i8,                              // Pursuit behavior
    lightcast: i8,                            // Can cast light
    death_roll: i8,                           // Death animation
    flags: u8,                                // Misc flags
    pad: [u8; 3],
    deathroll_sound: u8,                      // Death roll sound
    glow: u8,                                 // Glow amount
    behavior: i8,                             // AI behavior type
    aim: i8,                                  // Aiming ability
    
    // Animation joint lists for each state
    anim_states: [[JointList; MAX_GUNS+1]; N_ANIM_STATES],
    
    always_0xabcd: i32,                       // Validation marker
}
```

N_ANIM_STATES = 5 (idle, alert, firing, recoil, flinch)

### tJointPos

Joint positions for robot model animations.

```rust
struct JointPos {
    joint_num: i16,    // Joint index
    angles: Vector3,   // Fixed-point rotation angles (pitch, bank, heading)
}
```

Size: 8 bytes

### CWeaponInfo

Weapon definitions (extensive structure).

```rust
struct WeaponInfo {
    render_type: u8,              // Rendering type
    persistent: u8,               // Persistent across levels
    model_num: i16,               // 3D model index
    model_num_inner: i16,         // Inner model index
    flash_vclip: i8,              // Muzzle flash VClip
    robot_hit_vclip: i8,          // Robot hit VClip
    flash_sound: i16,             // Flash sound effect
    wall_hit_vclip: i8,           // Wall hit VClip
    fire_count: i8,               // Shots per trigger pull
    robot_hit_sound: i16,         // Robot hit sound
    ammo_usage: u8,               // Ammo consumed per shot
    weapon_vclip: i8,             // Weapon projectile VClip
    wall_hit_sound: i16,          // Wall hit sound
    destroyable: u8,              // Can be destroyed
    matter: u8,                   // Affected by matter
    bounce: i8,                   // Bounce behavior
    homing_flag: u8,              // Homing capability
    
    // Version-specific fields
    speedvar: u8,                 // Speed variance (v >= 3)
    flags: u8,                    // Weapon flags (v >= 3)
    flash: u8,                    // Flash intensity
    afterburner_size: u8,         // Afterburner trail size
    children: i8,                 // Child weapon type
    energy_usage: f32,            // Fixed-point energy cost
    fire_wait: f32,               // Fixed-point fire delay
    
    // Multiple bitmaps for animation
    multi_damage_scale: f32,      // Fixed-point multiplayer damage
    bitmap: BitmapIndex,          // Sprite bitmap
    blob_size: f32,               // Fixed-point projectile size
    flash_size: f32,              // Fixed-point flash size
    impact_size: f32,             // Fixed-point impact size
    strength: [f32; 5],           // Fixed-point damage per difficulty
    speed: [f32; 5],              // Fixed-point speed per difficulty
    mass: f32,                    // Fixed-point mass
    drag: f32,                    // Fixed-point drag
    thrust: f32,                  // Fixed-point thrust
    po_len_to_width_ratio: f32,   // Fixed-point aspect ratio
    light: f32,                   // Fixed-point light intensity
    lifetime: f32,                // Fixed-point lifetime
    damage_radius: f32,           // Fixed-point blast radius
    picture: BitmapIndex,         // HUD icon
    hires_picture: BitmapIndex,   // High-res HUD icon (v >= 3)
}
```

Size varies by HAM version (115-138 bytes)

### tPowerupTypeInfo

Powerup definitions (energy, shields, weapons, keys, etc.).

```rust
struct PowerupTypeInfo {
    vclip_num: i32,    // VClip for powerup animation
    hit_sound: i32,    // Sound when collected
    size: f32,         // Fixed-point collision size
    light: f32,        // Fixed-point light intensity
}
```

Size: 16 bytes

### CPolyModel

3D polygon model metadata.

```rust
struct PolyModel {
    n_models: i32,                    // Number of sub-models
    model_data_size: i32,             // Size of model data in bytes
    model_data: *u8,                  // Pointer to vertex/polygon data (file offset)
    submodel_ptrs: [i32; MAX_SUBMODELS], // Offsets to each sub-model
    submodel_offsets: [Vector3; MAX_SUBMODELS], // Fixed-point offsets
    submodel_norms: [Vector3; MAX_SUBMODELS],   // Fixed-point normals
    submodel_pnts: [Vector3; MAX_SUBMODELS],    // Fixed-point points
    submodel_rads: [f32; MAX_SUBMODELS],        // Fixed-point radii
    submodel_parents: [u8; MAX_SUBMODELS],      // Parent indices
    submodel_mins: [Vector3; MAX_SUBMODELS],    // Fixed-point bbox mins
    submodel_maxs: [Vector3; MAX_SUBMODELS],    // Fixed-point bbox maxs
    mins: Vector3,                    // Fixed-point model bbox min
    maxs: Vector3,                    // Fixed-point model bbox max
    rad: f32,                         // Fixed-point bounding radius
    n_textures: u8,                   // Texture count
    first_texture: u16,               // First texture index
    simpler_model: u8,                // LOD model index
}
```

MAX_SUBMODELS = 10

**Model Data Format** (after reading all CPolyModel structs):
- For each model, read `model_data_size` bytes of vertex and polygon data
- Format is complex binary format with submodel structure
- Not covered in detail here (requires separate POF format documentation)

### tPlayerShip

Player ship statistics.

```rust
struct PlayerShip {
    model_num: i32,             // 3D model index
    expl_vclip_num: i32,        // Explosion VClip
    mass: f32,                  // Fixed-point
    drag: f32,                  // Fixed-point
    max_thrust: f32,            // Fixed-point
    reverse_thrust: f32,        // Fixed-point
    brakes: f32,                // Fixed-point
    wiggle: f32,                // Fixed-point
    max_rotthrust: f32,         // Fixed-point
    gun_points: [Vector3; 8],   // Fixed-point gun positions
    // ... additional fields (132 bytes total)
}
```

Size: 132 bytes (PLAYER_SHIP_SIZE)

### tReactorInfo

Reactor (core) definitions.

```rust
struct ReactorInfo {
    model_num: i32,               // 3D model index
    n_guns: i32,                  // Number of gun positions
    gun_points: [Vector3; MAX_REACTOR_GUNS],   // Fixed-point
    gun_dirs: [Vector3; MAX_REACTOR_GUNS],     // Fixed-point
}
```

MAX_REACTOR_GUNS = 8

## Palette Files

**Important**: HAM files do NOT contain palette data. Palettes are stored in separate `.256` files.

### Palette File Format

```rust
struct PaletteFile {
    colors: [[u8; 3]; 256],  // 256 RGB triplets (768 bytes)
    fade_table: [u8; 256 * MAX_FADE_LEVELS],  // Fade/transparency table
}
```

**Palette Structure**:
- 256 color entries
- Each entry: 3 bytes (R, G, B)
- Color values range: 0-63 (6-bit color, not 8-bit!)
- Index 255 is typically the transparency color
- Total palette size: 768 bytes

**Common Palette Files**:
- `groupa.256` - Main Descent 2 palette (default)
- `palette.256` - Descent 1 palette
- `<levelname>.256` - Level-specific palettes (optional)

### Using Palettes with PIG Textures

1. Load palette from `.256` file (768 bytes of RGB triplets)
2. Read texture bitmap from PIG file (8-bit indexed color)
3. For each pixel index in bitmap:
   - Look up RGB triplet in palette: `color = palette[pixel_index]`
   - Scale 6-bit color to 8-bit: `rgb8 = (rgb6 * 255) / 63`
4. Handle transparency: pixel index 255 typically represents transparent pixels

## Constants

From `include/piggy.h`:

```rust
const HAMFILE_ID: u32 = 0x48414D21;  // "HAM!" signature
const HAMFILE_VERSION: i32 = 3;      // Current version

// Descent 2 limits
const MAX_TEXTURES: usize = 1200;
const MAX_SOUNDS: usize = 250;
const MAX_VCLIPS: usize = 200;
const MAX_EFFECTS: usize = 110;
const MAX_WALL_ANIMS: usize = 60;
const MAX_ROBOT_TYPES: usize = 85;
const MAX_WEAPONS: usize = 70;
const MAX_POWERUP_TYPES: usize = 50;
const MAX_POLYGON_MODELS: usize = 200;
const MAX_REACTORS: usize = 10;
const MAX_BITMAP_FILES: usize = 3000;

// Descent 1 limits (smaller)
const D1_MAX_TEXTURES: usize = 800;
const D1_MAX_SOUNDS: usize = 250;
const D1_MAX_VCLIPS: usize = 70;
const D1_MAX_EFFECTS: usize = 60;
const D1_MAX_WALL_ANIMS: usize = 30;
const D1_MAX_ROBOT_TYPES: usize = 30;
const D1_MAX_WEAPONS: usize = 30;
const D1_MAX_POWERUP_TYPES: usize = 29;
const D1_MAX_POLYGON_MODELS: usize = 85;
const D1_MAX_BITMAP_FILES: usize = 1800;
```

## Implementation Notes

### Fixed-Point Values

Many fields use fixed-point representation:
- 16.16 fixed-point format (C type: `fix`)
- Convert to float: `value_float = fixed_i32 as f32 / 65536.0`
- Used for: lighting, damage, speeds, masses, angles, times, etc.

### Reading Strategy

1. Read header and verify signature
2. Read sound offset if version < 3
3. Read each section sequentially using count-prefixed arrays
4. After all fixed-size sections, read bitmap translation table
5. If version < 3, seek to sound_offset and read embedded sounds

### Validation

- Verify signature matches `HAMFILE_ID`
- Check version is 2 or 3
- Ensure array counts don't exceed maximum constants
- Validate model indices reference valid model numbers
- Check bitmap indices don't exceed `MAX_BITMAP_FILES`

### Descent 1 Compatibility

For Descent 1 HAM files:
- Use D1-specific structure sizes (tTexMapInfoD1, smaller arrays)
- Use D1 constants (D1_MAX_*)
- Handle differences in animation frame counts
- Use `palette.256` instead of `groupa.256`

## Usage Example

```rust
// Pseudo-code for loading HAM + Palette + PIG
let ham = HamFile::load("descent2.ham")?;
let palette = Palette::load("groupa.256")?;
let pig = PigFile::load("descent2.pig")?;

// Get texture definition
let tex_info = ham.textures[texture_id];

// Get bitmap from PIG
let bitmap = pig.get_bitmap(tex_info.bitmap_index)?;

// Convert indexed texture to RGBA using palette
let rgba = bitmap.to_rgba(&palette);
```

## References

- D2X-XL Source Code v1.18.77
- Original Descent source code
- Descent Developer Network documentation
- [PIG File Format](PIG_FORMAT.md)
- [HOG Archive Format](HOG_FORMAT.md)

## License

This specification is derived from GPL-3.0 licensed D2X-XL source code.
Format documentation: GPL-3.0 compatible (facts not copyrightable).

# D2X-RS Architecture

## Project Overview

D2X-RS is a complete rewrite of the Descent 1 and 2 game engine in Rust 2024 edition using the Bevy 0.18 game engine. This project reimplements the original low-level C code from D2X-XL into Bevy's high-level ECS architecture, providing a modern, maintainable, and extensible codebase.

### Design Philosophy

1. **Modularity**: Clean separation between asset extraction, engine logic, and client application
2. **Extensibility**: Feature flags for derivative ports (D2X-XL, DXX-Rebirth additions)
3. **Modernization**: Replace outdated libraries (SDL1) with modern Rust equivalents
4. **Performance**: Leverage Bevy's ECS for efficient game logic
5. **Safety**: Rust's memory safety guarantees eliminate entire classes of bugs from original C code
6. **Compatibility**: Support original D1/D2 data files while enabling modern enhancements

---

## Crate Structure

```
d2x-rs/
├── crates/
│   ├── d2x-assets/       # Asset extraction and parsing
│   ├── d2x-engine/       # Core game engine (ECS systems)
│   └── d2x-client/       # Game client application
├── editor/               # C++23/Qt6 level editor (Phase 2)
├── assets/
│   └── shaders/         # Custom Bevy shaders
├── docs/                # Comprehensive documentation
└── tests/               # Integration tests
```

### Workspace Dependencies

The project uses a Cargo workspace to manage dependencies and ensure consistency across crates.

**Shared Dependencies**:
- Bevy 0.18 (engine, client)
- Serde (all crates - serialization)
- Thiserror (all crates - error handling)
- Anyhow (all crates - error propagation)
- Tracing (all crates - logging)

---

## Crate 1: d2x-assets

**Purpose**: Parse and extract all Descent game data files without game logic dependencies.

### Modules

#### `hog.rs` - HOG Archive System
Maps from: `include/hogfile.h`, `io/hogfile.cpp`

HOG files are simple archive formats containing game assets.

**Structure**:
```rust
pub struct HogArchive {
    files: BTreeMap<String, HogEntry>,
}

pub struct HogEntry {
    name: String,
    offset: u64,
    length: u32,
}
```

**Features**:
- Supports multiple HOG types (D1, D2, D2X, XL, Extra, Mission)
- Binary search for fast file lookup
- Lazy loading (don't extract until requested)
- Memory-mapped file access for performance

**File Format**:
```
[Header: "DHF" + 0x00]
For each file:
  - filename: 13 bytes (null-terminated)
  - size: u32 (little-endian)
  - data: [size] bytes
```

---

#### `pig.rs` - PIG Texture/Bitmap System
Maps from: `include/piggy.h`, `2d/piggy.cpp`, `2d/bitmap.cpp`

PIG files contain all game textures and bitmaps.

**Structure**:
```rust
pub struct PigFile {
    bitmaps: Vec<BitmapEntry>,
    palette: Option<Palette>,
}

pub struct BitmapEntry {
    name: String,
    width: u16,
    height: u16,
    flags: BitmapFlags,
    avg_color: u8,
    data: BitmapData,
}

pub enum BitmapData {
    Indexed(Vec<u8>),      // Paletted 8-bit
    Rgb(Vec<u8>),          // 24-bit RGB
    Rgba(Vec<u8>),         // 32-bit RGBA
}
```

**Features**:
- RLE decompression (custom Descent format)
- D1 vs D2 format detection and parsing
- Texture atlas generation for modern GPU rendering
- High-resolution texture replacement support
- Automatic super-transparency mask generation

**File Format**:
```
D2 PIG:
  - Signature: "PPIG" (PIGFILE_ID)
  - Version: u32 (2)
  - Bitmap count: u32
  - Bitmap headers: [count] * tPIGBitmapHeader
  - Bitmap data: raw or RLE compressed

D1 PIG: Similar but no wh_extra field (17 bytes vs 18)
```

---

#### `ham.rs` - HAM Game Data
Maps from: `include/loadgamedata.h`, `main/loadgamedata.cpp`

HAM files contain game definitions (robots, weapons, physics constants).

**Structure**:
```rust
pub struct HamFile {
    pub robot_info: Vec<RobotInfo>,
    pub weapon_info: Vec<WeaponInfo>,
    pub powerup_info: Vec<PowerupInfo>,
    pub polygon_models: Vec<PolygonModel>,
    pub textures: Vec<TextureInfo>,
    pub tmaps: Vec<TMapInfo>,
    pub vclips: Vec<VClip>,
    pub effects: Vec<Effect>,
}

pub struct RobotInfo {
    pub model_num: u8,
    pub behavior: AiBehavior,
    pub strength: f32,
    pub mass: f32,
    pub drag: f32,
    // ... weapon slots, sounds, etc.
}
```

**Features**:
- Version detection (D1 demo, D1 full, D2)
- Endian-aware parsing (support Mac data files)
- Validation and error reporting
- Merge support for custom HAM files

---

#### `level.rs` - Level Geometry
Maps from: `include/segment.h`, `main/loadgeometry.cpp`

Descent levels use a segment-based 6DOF portal system.

**Key Concepts**:
- **Segment**: Cubic unit of space with 8 vertices, 6 sides
- **Portal**: Connection between two segments through a side
- **Side**: Face of a segment (can be wall, door, or open portal)
- **Cube Model**: Each segment can be deformed from a cube shape

**Structure**:
```rust
pub struct Level {
    pub metadata: LevelMetadata,
    pub segments: Vec<Segment>,
    pub vertices: Vec<Vec3>,
    pub triggers: Vec<Trigger>,
    pub objects: Vec<LevelObject>,
    pub walls: Vec<Wall>,
}

pub struct Segment {
    pub vertices: [u16; 8],           // Indices into level.vertices
    pub children: [i16; 6],           // Connected segment IDs (-1 = wall)
    pub sides: [Side; 6],
    pub static_light: f32,
    pub segment_type: SegmentType,    // Normal, Fuel, Repair, Reactor, etc.
}

pub struct Side {
    pub wall_num: Option<u16>,
    pub tmap_num: u16,                // Primary texture
    pub tmap_num2: u16,               // Secondary texture (overlay)
    pub uvls: [[UVL; 4]; 2],          // UV coords + lighting (triangle pair)
}

pub struct UVL {
    pub u: f32,
    pub v: f32,
    pub light: f32,                   // Per-vertex lighting
}
```

**File Formats**:
- RDL (D1) - Original level format
- RL2 (D2) - Extended format with more features
- D2X-XL extensions - Up to 20,000 segments (vs 900 in D2)

**Features**:
- Portal connectivity graph generation
- Segment deformation (non-cubic shapes)
- Trigger system parsing
- Secret level connections
- Exit markers

---

#### `sound.rs` - Audio Assets
Maps from: `include/audio.h`, `audio/digi.cpp`, `audio/songs.cpp`

**Structure**:
```rust
pub struct SoundFile {
    pub sounds: Vec<SoundEffect>,
}

pub struct SoundEffect {
    pub name: String,
    pub sample_rate: u32,
    pub data: Vec<u8>,
    pub format: AudioFormat,
}

pub enum AudioFormat {
    Raw8Bit,
    Wav,
    Ogg,
}
```

**Features**:
- D1/D2 sound format support
- HMP (MIDI) music file parsing
- Addon sound support (D2X-XL's 43 additional sounds)
- Sound replacement system

---

#### `models.rs` - 3D Model Formats
Maps from: `include/pof.h`, `include/oof.h`, `include/ase.h`

**POF (Parallax Object Format)**: Original D1/D2 models
**OOF (Outrage Object Format)**: D2X-XL high-res models
**ASE (ASCII Scene Export)**: 3D Studio Max export format

**Structure**:
```rust
pub struct PolygonModel {
    pub submodels: Vec<SubModel>,
    pub textures: Vec<String>,
    pub guns: Vec<GunPoint>,
    pub dying_models: Vec<u8>,
}

pub struct SubModel {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub faces: Vec<Face>,
    pub offset: Vec3,
    pub animation: Option<Animation>,
}
```

---

### Public API

```rust
// Example usage
use d2x_assets::{HogArchive, PigFile, HamFile, Level};

// Load assets
let hog = HogArchive::open("descent2.hog")?;
let pig_data = hog.read_file("groupa.pig")?;
let pig = PigFile::parse(&pig_data)?;

// Extract level
let level_data = hog.read_file("level01.rl2")?;
let level = Level::parse(&level_data)?;

// Load game data
let ham_data = hog.read_file("descent2.ham")?;
let ham = HamFile::parse(&ham_data)?;
```

### Error Handling

All parsing uses `Result<T, AssetError>` with detailed error messages:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    #[error("Invalid HOG signature")]
    InvalidHogSignature,
    
    #[error("Failed to parse PIG header: {0}")]
    PigParseError(String),
    
    #[error("Unsupported format version: {version}")]
    UnsupportedVersion { version: u32 },
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

---

## Crate 2: d2x-engine

**Purpose**: Core game systems implemented as Bevy ECS plugins and systems.

### Bevy ECS Architecture

The engine leverages Bevy's Entity Component System for game logic:

- **Entities**: Game objects (player, robots, powerups, projectiles)
- **Components**: Data (position, health, weapon state)
- **Systems**: Logic (physics, AI, collision, rendering)
- **Resources**: Global state (level data, game settings)

### Plugin Organization

```rust
pub struct D2xEnginePlugin;

impl Plugin for D2xEnginePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(LevelPlugin)
            .add_plugins(PhysicsPlugin)
            .add_plugins(ObjectPlugin)
            .add_plugins(WeaponPlugin)
            .add_plugins(AiPlugin)
            .add_plugins(CollisionPlugin)
            .add_plugins(AudioPlugin);
    }
}
```

---

### Core Systems

#### Level System (`level/`)
Maps from: `include/segment.h`, `render/rendermine.cpp`

**Components**:
```rust
#[derive(Component)]
pub struct Segment {
    pub vertices: [Vec3; 8],
    pub segment_type: SegmentType,
    pub static_light: f32,
}

#[derive(Component)]
pub struct SegmentSide {
    pub segment: Entity,
    pub side_index: u8,
    pub wall: Option<Entity>,
    pub texture_primary: Handle<Image>,
    pub texture_secondary: Option<Handle<Image>>,
    pub uvls: [[UVL; 4]; 2],
}

#[derive(Component)]
pub struct Portal {
    pub from_segment: Entity,
    pub to_segment: Entity,
    pub side_index: u8,
}
```

**Systems**:
- `load_level_system`: Convert Level asset to ECS entities
- `portal_culling_system`: Determine visible segments from camera
- `segment_lighting_system`: Update dynamic lighting
- `segment_effects_system`: Animated textures, flickering lights

**Resources**:
```rust
#[derive(Resource)]
pub struct CurrentLevel {
    pub metadata: LevelMetadata,
    pub segment_graph: SegmentGraph,
    pub exit_segment: Entity,
}

pub struct SegmentGraph {
    adjacency: HashMap<Entity, Vec<(Entity, u8)>>,  // segment -> [(neighbor, side)]
}
```

---

#### Physics System (`physics/`)
Maps from: `physics/*.cpp`, `include/physics.h`

**6DOF Flight Model**:
```rust
#[derive(Component)]
pub struct Physics6Dof {
    pub velocity: Vec3,
    pub rotational_velocity: Vec3,  // Pitch, heading, bank rates
    pub mass: f32,
    pub drag: f32,
    pub thrust: Vec3,
    pub torque: Vec3,
}

#[derive(Component)]
pub struct Collider {
    pub radius: f32,
    pub hitbox: Option<OrientedBoundingBox>,
}
```

**Systems**:
- `apply_thrust_system`: Player/robot thrust input
- `apply_drag_system`: Air resistance simulation
- `integrate_physics_system`: Update positions and rotations
- `segment_transition_system`: Move objects between segments
- `gravity_system`: Special segment types (lava/water/ice)

**Physics Integration**:
```rust
fn integrate_physics_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Physics6Dof)>,
) {
    let dt = time.delta_seconds();
    
    for (mut transform, mut physics) in query.iter_mut() {
        // Apply forces
        let acceleration = physics.thrust / physics.mass;
        physics.velocity += acceleration * dt;
        
        // Apply drag
        physics.velocity *= (1.0 - physics.drag * dt);
        
        // Integrate position
        transform.translation += physics.velocity * dt;
        
        // Integrate rotation
        let rotation_delta = Quat::from_euler(
            EulerRot::XYZ,
            physics.rotational_velocity.x * dt,
            physics.rotational_velocity.y * dt,
            physics.rotational_velocity.z * dt,
        );
        transform.rotation = rotation_delta * transform.rotation;
    }
}
```

---

#### Collision System (`collision/`)
Maps from: `physics/collide.cpp`, `physics/sphere_collision.cpp`, `physics/hitbox_collision.cpp`

**Collision Types**:
1. **Sphere-Sphere**: Fast, used for most collision detection
2. **Sphere-Wall**: Ray-sphere intersection with segment sides
3. **Hitbox**: Oriented bounding boxes for precise robot collision
4. **Ray-Segment**: Weapon fire collision

**Components**:
```rust
#[derive(Component)]
pub struct SphereCollider {
    pub radius: f32,
}

#[derive(Component)]
pub struct HitboxCollider {
    pub boxes: Vec<OrientedBoundingBox>,
}

#[derive(Component)]
pub struct CurrentSegment {
    pub segment: Entity,
}
```

**Systems**:
- `broad_phase_collision_system`: Spatial partitioning by segment
- `narrow_phase_collision_system`: Precise collision tests
- `wall_collision_system`: Segment boundary collision
- `collision_response_system`: Apply collision effects

**Collision Events**:
```rust
#[derive(Event)]
pub enum CollisionEvent {
    ObjectObject { a: Entity, b: Entity, normal: Vec3 },
    ObjectWall { object: Entity, wall: Entity, normal: Vec3 },
    Weapon { weapon: Entity, target: Entity, damage: f32 },
}
```

---

#### Object System (`objects/`)
Maps from: `objects/*.cpp`, `include/object.h`

**Base Component**:
```rust
#[derive(Component)]
pub struct GameObject {
    pub object_type: ObjectType,
    pub shields: f32,
    pub current_segment: Entity,
}

#[derive(Component, Clone, Copy)]
pub enum ObjectType {
    Player,
    Robot,
    Weapon,
    Powerup,
    Hostage,
    Reactor,
    Debris,
    Effect,
}
```

**Player**:
```rust
#[derive(Component)]
pub struct Player {
    pub player_id: u8,
    pub energy: f32,
    pub shields: f32,
    pub primary_weapon: PrimaryWeapon,
    pub secondary_weapon: SecondaryWeapon,
    pub laser_level: u8,
    pub flags: PlayerFlags,
}

#[derive(Component)]
pub struct Inventory {
    pub keys: KeyFlags,
    pub missiles: [u16; 5],  // Concussion, Homing, Proximity, Smart, Mega
    pub primary_ammo: [u16; 5],  // Vulcan, Gauss ammo
}
```

**Robot**:
```rust
#[derive(Component)]
pub struct Robot {
    pub robot_id: u8,
    pub ai_state: AiState,
    pub cloaked: bool,
    pub fire_cooldown: Timer,
}
```

**Powerup**:
```rust
#[derive(Component)]
pub struct Powerup {
    pub powerup_type: PowerupType,
    pub spawn_time: f32,
}

pub enum PowerupType {
    Energy,
    Shield,
    Laser,
    QuadLaser,
    VulcanAmmo,
    Missile(MissileType),
    // ... 50+ types
}
```

---

#### Weapon System (`weapons/`)
Maps from: `weapons/*.cpp`, `include/weapon.h`, `include/laser.h`

**Weapon Categories**:

**Primary Weapons** (energy-based):
```rust
#[derive(Clone, Copy)]
pub enum PrimaryWeapon {
    Laser,
    Vulcan,
    Spreadfire,
    Plasma,
    Fusion,
    Gauss,      // D2
    Helix,      // D2
    Phoenix,    // D2
    Omega,      // D2
}
```

**Secondary Weapons** (ammo-based):
```rust
#[derive(Clone, Copy)]
pub enum SecondaryWeapon {
    Concussion,
    Homing,
    Proximity,
    Smart,
    Mega,
}
```

**Projectile Component**:
```rust
#[derive(Component)]
pub struct Projectile {
    pub weapon_type: WeaponType,
    pub owner: Entity,
    pub damage: f32,
    pub lifetime: Timer,
    pub homing_target: Option<Entity>,
}

#[derive(Component)]
pub struct HomingMissile {
    pub target: Entity,
    pub turn_rate: f32,
    pub lock_strength: f32,
}
```

**Systems**:
- `weapon_fire_system`: Create projectiles from player/robot fire
- `projectile_physics_system`: Update projectile positions
- `homing_guidance_system`: Smart missile targeting
- `weapon_hit_system`: Apply damage on collision
- `weapon_effects_system`: Muzzle flash, tracer rendering

**Smart Weapon Logic**:
```rust
fn smart_missile_targeting_system(
    mut missiles: Query<(&Transform, &mut HomingMissile)>,
    targets: Query<(&Transform, &GameObject), With<Robot>>,
) {
    for (missile_transform, mut homing) in missiles.iter_mut() {
        // Find best target in cone
        let mut best_target = None;
        let mut best_score = f32::MIN;
        
        for (target_transform, _) in targets.iter() {
            let to_target = target_transform.translation - missile_transform.translation;
            let distance = to_target.length();
            
            // Smart missiles can see through walls in original game
            let forward = missile_transform.forward();
            let dot = forward.dot(to_target.normalize());
            
            if dot > 0.7 {  // ~45 degree cone
                let score = dot / distance;
                if score > best_score {
                    best_score = score;
                    best_target = Some(target_transform);
                }
            }
        }
        
        // Update target
        // ...
    }
}
```

---

#### AI System (`ai/`)
Maps from: `ai/*.cpp`, `include/ai.h`, `include/aistruct.h`

**AI State Machine**:
```rust
#[derive(Component)]
pub struct AiState {
    pub mode: AiMode,
    pub target: Option<Entity>,
    pub goal_segment: Option<Entity>,
    pub path: Vec<Entity>,
    pub awareness: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum AiMode {
    Still,
    Normal,
    Hide,
    RunFrom,
    FollowPath,
    Chase,
    Attack,
    Flinch,
}
```

**Robot Behavior**:
```rust
#[derive(Component)]
pub struct RobotBehavior {
    pub attack_type: AttackType,
    pub circle_distance: f32,
    pub rapidfire_count: u8,
    pub companion: bool,  // Buddy bot
    pub thief: bool,      // Steal items
    pub smart_blobs: bool, // Release smart mine blobs on death
}
```

**Systems**:
- `ai_awareness_system`: Detect player (line of sight, sound)
- `ai_behavior_system`: State machine transitions
- `ai_pathfinding_system`: A* through segment graph
- `ai_movement_system`: Execute movement along path
- `ai_attack_system`: Weapon fire logic
- `ai_boss_system`: Special boss behaviors

**Pathfinding**:
```rust
pub struct SegmentPathfinder {
    graph: SegmentGraph,
}

impl SegmentPathfinder {
    pub fn find_path(&self, start: Entity, goal: Entity) -> Option<Vec<Entity>> {
        // A* pathfinding through segment graph
        // Uses portal connections as edges
        // Heuristic: Euclidean distance between segment centers
    }
}
```

---

#### Rendering System (`render/`)
Maps from: `render/*.cpp`, `ogl/*.cpp`

**Modern Rendering Approach**:
Uses Bevy's PBR renderer instead of reimplementing portal rendering.

**Optimization Strategy**:
- Portal culling for segment visibility
- Frustum culling (Bevy built-in)
- Occlusion culling using portal graph
- Instanced rendering for repeated geometry

**Custom Materials**:
```rust
#[derive(AsBindGroup, TypePath, Asset)]
pub struct DescentMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub base_texture: Handle<Image>,
    
    #[texture(2)]
    #[sampler(3)]
    pub overlay_texture: Option<Handle<Image>>,
    
    #[uniform(4)]
    pub static_light: f32,
    
    #[uniform(5)]
    pub glow: f32,
}
```

**Lighting**:
```rust
#[derive(Component)]
pub struct DynamicLight {
    pub color: Color,
    pub intensity: f32,
    pub flicker: Option<FlickerPattern>,
}

pub enum FlickerPattern {
    Steady,
    Flicker,
    Strobe,
    Random,
}
```

**Special Effects**:
```rust
#[derive(Component)]
pub struct ParticleEmitter {
    pub particle_type: ParticleType,
    pub spawn_rate: f32,
    pub lifetime: f32,
}

pub enum ParticleType {
    Smoke,
    Spark,
    Debris,
    ThrusterFlame,
    Explosion,
}
```

---

### Graphics Feature Flags

```rust
#[cfg(feature = "pbr-materials")]
fn setup_pbr_materials() {
    // Modern PBR with normal maps, metallic, roughness
}

#[cfg(not(feature = "pbr-materials"))]
fn setup_classic_materials() {
    // Original texture-only appearance
}

#[cfg(feature = "hdr-rendering")]
fn setup_hdr_pipeline() {
    // HDR bloom, tonemapping
}

#[cfg(feature = "high-quality-lighting")]
fn setup_per_pixel_lighting() {
    // Enhanced lighting shader
}
```

---

## Crate 3: d2x-client

**Purpose**: User-facing game application, menus, HUD, game modes.

### Client Plugin Architecture

```rust
pub struct D2xClientPlugin;

impl Plugin for D2xClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(MenuPlugin)
            .add_plugins(HudPlugin)
            .add_plugins(InputPlugin)
            .add_plugins(GameModePlugin)
            .add_plugins(SaveLoadPlugin);
    }
}
```

---

### Menu System (`menu/`)
Maps from: `menus/*.cpp`

**UI Framework**: `bevy_egui` for immediate mode UI

**Menu Structure**:
```rust
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum MenuState {
    #[default]
    MainMenu,
    SinglePlayer,
    Multiplayer,
    Options,
    Controls,
    Graphics,
    Audio,
    LoadGame,
    InGame,
}
```

**Settings**:
```rust
#[derive(Resource, Serialize, Deserialize)]
pub struct GameSettings {
    pub graphics: GraphicsSettings,
    pub audio: AudioSettings,
    pub controls: ControlSettings,
    pub gameplay: GameplaySettings,
}

#[derive(Serialize, Deserialize)]
pub struct GraphicsSettings {
    pub resolution: (u32, u32),
    pub fullscreen: bool,
    pub vsync: bool,
    pub render_mode: RenderMode,  // Classic, PBR, High Quality
    pub texture_quality: TextureQuality,  // Original, High-res
    pub lighting_quality: LightingQuality,  // Per-vertex, Per-pixel
    pub hdr: bool,
}

pub enum RenderMode {
    Classic,
    PbrStandard,
    PbrEnhanced,
}
```

---

### HUD System (`hud/`)
Maps from: `cockpit/*.cpp`

**Cockpit Types**:
```rust
#[derive(Component)]
pub enum CockpitType {
    Full,       // Full cockpit with instrument panels
    Status,     // Minimal status bar
    Letterbox,  // Widescreen bars
    None,       // Full screen
}
```

**HUD Elements**:
```rust
#[derive(Component)]
pub struct HudElement {
    pub element_type: HudElementType,
    pub position: Vec2,
    pub visible: bool,
}

pub enum HudElementType {
    Reticle,
    ShieldBar,
    EnergyBar,
    WeaponDisplay,
    AmmoCount,
    KeyDisplay,
    Lives,
    Score,
    MissileView,
    RadarMap,       // D2X-XL feature
    AfterburnerBar, // D2
}
```

**Reticle System**:
```rust
#[derive(Component)]
pub struct Reticle {
    pub reticle_type: ReticleType,
    pub lock_state: LockState,
}

pub enum LockState {
    None,
    Acquiring { progress: f32 },
    Locked { target: Entity },
}
```

---

### Input System (`input/`)
Maps from: `input/*.cpp`

**6DOF Control Scheme**:
```rust
#[derive(Resource)]
pub struct FlightControls {
    pub pitch: f32,      // Up/down
    pub heading: f32,    // Left/right
    pub bank: f32,       // Roll
    pub forward: f32,    // Thrust
    pub slide_h: f32,    // Horizontal slide
    pub slide_v: f32,    // Vertical slide
    
    pub fire_primary: bool,
    pub fire_secondary: bool,
    pub fire_flare: bool,
    
    pub cycle_primary: bool,
    pub cycle_secondary: bool,
    
    pub afterburner: bool,  // D2
    pub headlight: bool,
    pub automap: bool,
    pub rear_view: bool,
}
```

**Input Mapping**:
Uses `leafwing-input-manager` for configurable controls.

```rust
#[derive(Actionlike, Clone, Copy)]
pub enum FlightAction {
    PitchUp,
    PitchDown,
    HeadingLeft,
    HeadingRight,
    BankLeft,
    BankRight,
    Forward,
    Backward,
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
    // ... weapon and utility actions
}
```

---

### Game Modes (`gamemode/`)
Maps from: `gamemodes/*.cpp`, `main/game.cpp`

**Mode Definitions**:
```rust
#[derive(Clone, Copy, PartialEq)]
pub enum GameMode {
    SinglePlayer,
    Cooperative,
    Anarchy,           // Free-for-all deathmatch
    TeamAnarchy,
    RoboAnarchy,       // Player vs robots
    CaptureTheFlag,
    Hoard,             // Collect orbs
    Monsterball,       // D2X-XL
    Entropy,           // D2X-XL team mode
}
```

**Mission System**:
```rust
#[derive(Resource)]
pub struct Mission {
    pub name: String,
    pub levels: Vec<LevelMetadata>,
    pub secret_levels: Vec<u8>,
    pub briefing: Option<String>,
}

#[derive(Resource)]
pub struct GameProgress {
    pub current_level: usize,
    pub difficulty: Difficulty,
    pub score: u32,
    pub lives: u8,
}
```

---

### Save System (`save/`)
Maps from: `main/savegame.cpp`, `main/loadgame.cpp`

**Save Data**:
```rust
#[derive(Serialize, Deserialize)]
pub struct SaveGame {
    pub version: u32,
    pub player: PlayerState,
    pub level: LevelState,
    pub mission: MissionProgress,
}

#[derive(Serialize, Deserialize)]
pub struct PilotProfile {
    pub name: String,
    pub difficulty: Difficulty,
    pub missions_completed: Vec<String>,
    pub high_scores: HashMap<String, u32>,
    pub control_settings: ControlSettings,
}
```

---

## Networking Architecture

See: `docs/networking/ARCHITECTURE.md` for detailed networking design.

**High-Level Overview**:
- Client-server architecture using `bevy_renet`
- Authoritative server prevents cheating
- Client-side prediction for responsiveness
- LAN discovery via UDP broadcast
- Internet play via master server
- Dedicated server support

---

## Feature Flag System

See: `docs/FEATURES.md` for complete feature flag documentation.

**Core Flags**:
```toml
[features]
default = ["base-d1", "base-d2"]

# Base games
base-d1 = []
base-d2 = ["base-d1"]

# Derivative features
d2x-xl = [
    "enhanced-graphics",
    "custom-weapons", 
    "advanced-modes",
    "entropy-mode",
    "monsterball-mode"
]

dxx-rebirth = ["rebirth-improvements"]

# Graphics tiers
enhanced-graphics = ["per-pixel-lighting", "advanced-particles"]
pbr-materials = []
hdr-rendering = []
high-quality-lighting = []

# Texture quality
hires-textures = []

# Gameplay
custom-weapons = []
advanced-ai = []
```

**Usage in Code**:
```rust
#[cfg(feature = "d2x-xl")]
const MAX_SEGMENTS: usize = 20000;

#[cfg(not(feature = "d2x-xl"))]
const MAX_SEGMENTS: usize = 900;
```

---

## Development Phases

### Phase 1: Asset Foundation (Months 1-2)
- [ ] `d2x-assets` crate complete
- [ ] HOG, PIG, HAM, Level parsers
- [ ] Unit tests with real D1/D2 files
- [ ] Documentation

### Phase 2: Level Rendering (Month 3)
- [ ] Basic `d2x-engine` structure
- [ ] Level loader system
- [ ] Segment mesh generation
- [ ] Camera and free-flight
- [ ] Texture rendering

### Phase 3: Physics (Month 4)
- [ ] 6DOF physics system
- [ ] Collision detection
- [ ] Wall collision
- [ ] Player ship controls

### Phase 4: Objects & Gameplay (Months 5-6)
- [ ] Player object
- [ ] Robot objects
- [ ] Powerups
- [ ] Basic weapons (laser, missiles)
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
- [ ] Mod support

---

## Testing Strategy

### Unit Tests
Each crate has comprehensive unit tests:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hog_parsing() {
        let hog_data = include_bytes!("../../test_data/test.hog");
        let hog = HogArchive::parse(hog_data).unwrap();
        assert_eq!(hog.file_count(), 3);
    }
}
```

### Integration Tests
```rust
#[test]
fn test_load_level() {
    let mut app = App::new();
    app.add_plugins(D2xEnginePlugin);
    
    // Load test level
    let level = Level::load("test_level.rl2").unwrap();
    app.insert_resource(level);
    
    // Verify segment entities created
    app.update();
    // assertions...
}
```

### Performance Tests
- Benchmark segment rendering
- Profile physics system
- Measure network latency
- Memory usage tracking

---

## Build Configuration

### Rust Edition
```toml
[package]
edition = "2021"  # 2024 when stable
rust-version = "1.75"
```

### Release Optimizations
```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1

[profile.release-fast]
inherits = "release"
lto = "fat"
codegen-units = 1
```

### Platform Support
- Windows (primary)
- Linux
- macOS
- Future: WebAssembly (browser play)

---

## Documentation Standards

All public APIs must have:
- Doc comments with examples
- References to original D2X-XL code locations
- Usage examples
- Performance considerations

```rust
/// Parse a HOG archive file.
///
/// HOG files are simple archive formats used by Descent to package game assets.
/// 
/// # Format
/// - Header: "DHF\0" (4 bytes)
/// - Entries: filename (13 bytes) + size (4 bytes) + data
///
/// # Corresponds to
/// - `include/hogfile.h`: CHogFile class
/// - `io/hogfile.cpp`: CHogFile::Init, CHogFile::Use
///
/// # Example
/// ```
/// use d2x_assets::HogArchive;
/// 
/// let hog = HogArchive::open("descent2.hog")?;
/// let file_data = hog.read_file("level01.rl2")?;
/// ```
pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, AssetError> {
    // ...
}
```

---

## Contributing Guidelines

(To be expanded with contribution workflow)

---

## License

This rewrite respects the Parallax Software license terms from the original Descent source code release. See LICENSE file for details.

---

## References

- Original Descent source: Available under Parallax license
- D2X-XL source: `/tmp/d2x-xl-src`
- DXX-Rebirth: https://github.com/dxx-rebirth/dxx-rebirth
- Bevy Engine: https://bevyengine.org/
- Descent Level Specs: https://www.descent-community.org/

---

**Document Version**: 1.0  
**Last Updated**: 2026-02-23  
**Status**: Initial Architecture Design

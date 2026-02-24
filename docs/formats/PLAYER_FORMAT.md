# Player Profile File Formats (.PLR / .PLX)

Player profiles store player identity, game progress, control configuration, and settings. This document describes both the original Descent 1/2 binary format and the D2X-XL text format.

## Format Versions

- **Descent 1** - Binary .PLR format
- **Descent 2** - Binary .PLR format (version 17 compatible)
- **D2 Windows 95** - Version 24
- **D2X-W32** - Version 45
- **D2X-XL** - Text-based .PLX format (version 161)

## Binary Format (.PLR) - Descent 1/2

The original Descent 1 and Descent 2 games use a binary format to store player profiles. These files contain:

- Player callsign (name, 8 characters max)
- Mission progress (highest levels reached per mission)
- Control configuration (keyboard/mouse/joystick bindings)
- Game statistics and preferences

### File Structure (Preliminary)

**Note**: The exact binary structure requires reverse engineering or access to original source code. This is a preliminary specification based on D2X-XL source analysis.

```
Offset  Size  Type        Description
------  ----  ----        -----------
0x00    4     char[4]     File signature "PLYR"
0x04    4     uint32      File version number
0x08    ?     -           Player data (version-dependent)
```

### Version History

- **Version 17** (COMPATIBLE_PLAYER_FILE_VERSION)
  - Original Descent 2 compatible version
  - Player structure with callsign, network address, stats
  - Control configuration data
  - Mission progress tracking

- **Version 24** (D2W95_PLAYER_FILE_VERSION)
  - Windows 95 version enhancements

- **Version 45** (D2XW32_PLAYER_FILE_VERSION)
  - First flawless D2X-W32 player file version

### Player Data Structure

Based on D2X-XL source (player.h), the player info contains:

```c
struct CPlayerInfo {
    char     callsign[9];              // Player name (8 chars + null)
    uint8_t  netAddress[6];            // Network address
    int8_t   connected;                // Connection status
    int32_t  nObject;                  // Object number
    int32_t  nPacketsGot;              // Packets received
    int32_t  nPacketsSent;             // Packets sent
    
    uint32_t flags;                    // Powerup flags
    fix      energy;                   // Energy remaining (fixed-point)
    fix      shield;                   // Shield remaining (fixed-point)
    uint8_t  lives;                    // Lives remaining
    int8_t   level;                    // Current level
    uint8_t  laserLevel;               // Laser level
    int8_t   startingLevel;            // Starting level
    int16_t  nKillerObj;               // Who killed player
    uint16_t primaryWeaponFlags;       // Primary weapon flags
    uint16_t secondaryWeaponFlags;     // Secondary weapon flags
    uint16_t primaryAmmo[10];          // Primary ammo counts
    uint16_t secondaryAmmo[10];        // Secondary ammo counts
    uint8_t  nInvuls;                  // Invulnerability count
    uint8_t  nCloaks;                  // Cloak count
    
    // Statistics
    int32_t  lastScore;                // Score at level start
    int32_t  score;                    // Current score
    fix      timeLevel;                // Time on level
    fix      timeTotal;                // Total game time
    fix      cloakTime;                // Cloak time remaining
    fix      invulnerableTime;         // Invulnerability time
    int16_t  nScoreGoalCount;          // Score goal count
    int16_t  netKilledTotal;           // Times killed
    int16_t  netKillsTotal;            // Total kills
    int16_t  numKillsLevel;            // Kills this level
    int16_t  numKillsTotal;            // Total kills
    int16_t  numRobotsLevel;           // Robots this level
    int16_t  numRobotsTotal;           // Total robots
    // ... hostages, etc.
};
```

### Constants

```c
#define CALLSIGN_LEN           8       // Max callsign length
#define N_SAVE_SLOTS          10       // Number of save game slots
#define GAME_NAME_LEN         25       // Mission name length

#define INITIAL_ENERGY  I2X(100)       // 100% energy to start
#define INITIAL_SHIELD  I2X(100)       // 100% shield to start
#define MAX_ENERGY      I2X(200)       // Maximum energy
#define MAX_SHIELD      I2X(200)       // Maximum shield
```

### Player Flags

```c
#define PLAYER_FLAGS_INVULNERABLE   0x0001
#define PLAYER_FLAGS_BLUE_KEY       0x0002
#define PLAYER_FLAGS_RED_KEY        0x0004
#define PLAYER_FLAGS_GOLD_KEY       0x0008
#define PLAYER_FLAGS_FLAG           0x0010  // Has team flag
#define PLAYER_FLAGS_FULLMAP        0x0040
#define PLAYER_FLAGS_AMMO_RACK      0x0080
#define PLAYER_FLAGS_CONVERTER      0x0100
#define PLAYER_FLAGS_QUAD_LASERS    0x0400
#define PLAYER_FLAGS_CLOAKED        0x0800
#define PLAYER_FLAGS_AFTERBURNER    0x1000
#define PLAYER_FLAGS_HEADLIGHT      0x2000
#define PLAYER_FLAGS_HEADLIGHT_ON   0x4000
```

### Implementation Status

**Current Status**: Binary .PLR parsing is not yet fully implemented in descent-core.

The binary format requires detailed reverse engineering or access to original source code to fully implement. The structure is version-dependent and contains complex nested data.

## Text Format (.PLX) - D2X-XL

D2X-XL replaced the binary .PLR format with a human-readable text format (.PLX files). This format is much simpler and easier to parse.

### File Format

PLX files are plain text files with one parameter per line:

```
parameter.name=value
```

### Format Specification

- Each line contains a single key=value pair
- Keys are hierarchical, using dot notation: `category.subcategory.parameter`
- Array indices use bracket notation: `array[0].field`
- Values are stored as strings (integers, booleans, or text)
- Boolean values: 0 = false, non-zero = true
- Empty lines are ignored
- No comments supported

### Example PLX File

```
gameData.renderData.screen.m_w=1920
gameData.renderData.screen.m_h=1080
gameStates.render.bShowFrameRate=0
gameStates.render.bShowTime=1
gameStates.render.cockpit.nType=3
gameData.escortData.szName=GUIDE-BOT
gameOptions[0].render.nQuality=2
gameOptions[0].render.bCartoonize=0
gameOptions[0].input.mouse.sensitivity[0]=8
gameOptions[0].input.mouse.sensitivity[1]=8
gameOptions[0].input.mouse.sensitivity[2]=8
keyboard.Fire primary[0].value=-1
keyboard.Fire secondary[0].value=57
keyboard.Accelerate[0].value=72
```

### Parameter Categories

#### Display Settings
```
gameData.renderData.screen.m_w=1920           # Screen width
gameData.renderData.screen.m_h=1080           # Screen height
gameStates.video.nDefaultDisplayMode=3        # Display mode
customDisplayMode.w=0                         # Custom width
customDisplayMode.h=0                         # Custom height
```

#### Game State
```
gameStates.render.bShowFrameRate=0            # Show FPS
gameStates.render.bShowTime=1                 # Show time
gameStates.render.cockpit.nType=3             # Cockpit type
gameStates.app.nDifficultyLevel=2             # Difficulty
```

#### Network Statistics
```
networkData.nNetLifeKills=0                   # Lifetime kills
networkData.nNetLifeKilled=0                  # Times killed
gameData.appData.nLifetimeChecksum=0          # Checksum
```

#### Escort Bot
```
gameData.escortData.szName=GUIDE-BOT          # Guide-bot name
```

#### Multiplayer Macros
```
gameData.multigame.msg.szMacro[0]=Why can't we all just get along?
gameData.multigame.msg.szMacro[1]=Hey, I got a present for ya
gameData.multigame.msg.szMacro[2]=I got a hankerin' for a spankerin'
gameData.multigame.msg.szMacro[3]=This one's headed for Uranus
```

#### Multiplayer Parameters
```
mpParams.nLevel=1                             # Starting level
mpParams.nGameType=3                          # Game type
mpParams.nGameMode=3                          # Game mode
mpParams.nDifficulty=2                        # Difficulty
mpParams.udpPorts[0]=28342                    # UDP port 1
mpParams.udpPorts[1]=28342                    # UDP port 2
mpParams.szServerIpAddr=127.0.0.1             # Server IP
```

#### Game Options (Per Mode)

D2X-XL stores two sets of game options:
- `gameOptions[0]` - Enhanced mode with all features
- `gameOptions[1]` - Pure Descent 2 mode (nostalgia mode)

```
gameOptions[0].render.nQuality=2              # Render quality
gameOptions[0].render.bCartoonize=0           # Cartoon effect
gameOptions[0].render.nMaxFPS=60              # Max FPS
gameOptions[0].input.mouse.sensitivity[0]=8   # Mouse X sens
gameOptions[0].input.mouse.sensitivity[1]=8   # Mouse Y sens
gameOptions[0].input.mouse.sensitivity[2]=8   # Mouse Z sens
gameOptions[0].sound.bFadeMusic=1             # Fade music
gameOptions[0].gameplay.nAutoSelectWeapon=1   # Auto-select weapon
```

#### Extra Game Info

D2X-XL specific features:

```
extraGameInfo[0].bAutoBalanceTeams=0          # Auto-balance teams
extraGameInfo[0].bDamageExplosions=0          # Explosive damage
extraGameInfo[0].bShadows=1                   # Enable shadows
extraGameInfo[0].bTracers=1                   # Bullet tracers
extraGameInfo[0].bUseCameras=1                # Camera monitors
extraGameInfo[0].bUseParticles=1              # Particle effects
extraGameInfo[0].bUseLightning=1              # Lightning effects
extraGameInfo[0].nRadar=1                     # Radar type
extraGameInfo[0].nWeaponIcons=3               # Weapon icons
extraGameInfo[0].bMouseLook=0                 # Mouse look
```

#### Control Configuration

Keyboard bindings:

```
keyboard.Pitch forward[0].value=-56           # Key code (negative = special)
keyboard.Pitch backward[0].value=-48
keyboard.Turn left[0].value=-53
keyboard.Turn right[0].value=-51
keyboard.Slide left[0].value=75
keyboard.Slide right[0].value=77
keyboard.Slide up[0].value=71
keyboard.Slide down[0].value=73
keyboard.Fire primary[0].value=-1             # -1 = unbound
keyboard.Fire secondary[0].value=57
keyboard.Accelerate[0].value=72
keyboard.reverse[0].value=76
keyboard.Afterburner[0].value=-100
```

Most controls have two slots: `[0]` and `[1]` for primary and alternate bindings.

Mouse and joystick configuration:

```
mouse.Fire primary.value=0
mouse.Fire secondary.value=1
joystick.Pitch forward.value=0
joystick.Turn left.value=0
```

### Parsing Rules

1. **Whitespace**: Leading and trailing whitespace is trimmed
2. **Empty Lines**: Skipped
3. **Missing Keys**: Lines without '=' are ignored
4. **Duplicate Keys**: Later values override earlier ones
5. **Type Conversion**: Values are strings; parse as needed
6. **Arrays**: Bracket notation for indices: `array[0]`
7. **Nested Objects**: Dot notation: `object.field.subfield`

### Value Types

- **Integers**: `42`, `-1`, `1920`
- **Booleans**: `0` (false), `1` or any non-zero (true)
- **Strings**: `GUIDE-BOT`, `127.0.0.1`
- **Key Codes**: Negative numbers for special keys

### File Naming

PLX files are named after the player's callsign:

```
PLAYER_NAME.plx
```

Where `PLAYER_NAME` is the player's callsign (8 characters max).

## Implementation

### Rust API

```rust
use descent_core::player::{PlxProfile, PlayerProfile};

// Parse text profile
let data = std::fs::read_to_string("player.plx")?;
let profile = PlxProfile::parse(&data)?;

// Access parameters
let width = profile.get_int("gameData.renderData.screen.m_w");
let show_fps = profile.get_bool("gameStates.render.bShowFrameRate");

// Modify and save
let mut profile = PlxProfile::new();
profile.set("key".to_string(), "value".to_string());
let output = profile.serialize();
std::fs::write("player.plx", output)?;
```

### Binary Format Status

The binary .PLR parser returns an error indicating the format is not yet supported:

```rust
use descent_core::player::PlrProfile;

let data = std::fs::read("player.plr")?;
let result = PlrProfile::parse(&data);
// Returns Err(AssetError::UnsupportedFormat(...))
```

To fully implement binary .PLR parsing, we would need:
1. Detailed binary structure specification
2. Handling of version differences
3. Fixed-point math conversion (I2X macro)
4. Control configuration binary format
5. Mission progress data structures

## References

- D2X-XL Source: `playerprofile.h`, `playerprofile.cpp`
- D2X-XL Source: `player.h` (CPlayerInfo structure)
- D2X-XL Source: `kconfig.h` (control configuration)

## Notes

- Binary .PLR format is version-dependent and complex
- D2X-XL moved to text .PLX for better flexibility
- Text format is human-editable and easier to debug
- Binary format requires access to original source for full implementation
- Player profiles are separate from in-game save states (.SGC files)
- Callsign is limited to 8 characters in both formats

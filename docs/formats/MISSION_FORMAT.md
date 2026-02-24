# Descent 1/2 Mission File Format

This document describes the mission file format used in Descent 1 (.msn) and Descent 2 (.mn2), based on the D2X-XL source code and reverse engineering.

## Overview

Mission files are text-based configuration files that define:
- Mission metadata (name, type, enhancement level)
- Level list (regular levels and secret levels)
- Associated HOG archive file
- Briefing and ending text files
- Music/song lists (optional)

The format is **identical** between `.msn` (Descent 1) and `.mn2` (Descent 2) - only the file extension differs to indicate which game version the mission is for.

---

## File Format

### Basic Structure

Mission files use a simple line-based text format similar to INI files:

```
; Comments start with semicolon
key = value

; Some keys are followed by additional data on subsequent lines
num_levels = 3
level01.rdl
level02.rdl
level03.rdl
```

### File Encoding

- **Format**: Plain text (ASCII)
- **Line endings**: DOS (CRLF) or Unix (LF) - both supported
- **Comments**: Everything after `;` on a line is ignored
- **Case sensitivity**: Keys are case-insensitive
- **Whitespace**: Trimmed from keys and values

---

## Keywords and Structure

### Required Fields

#### `name` - Mission Name

The display name of the mission (required).

```
name = Descent: First Strike
```

- **Type**: String
- **Length**: Up to 25 characters (MISSION_NAME_LEN)
- **Required**: Yes

**Alternative name keywords (with enhancement indicators):**
- `xname` - Enhanced mission (sets enhancement_level = 1)
- `zname` - Another enhancement type (sets enhancement_level = 2)
- `d2x-name` - D2X-XL enhanced mission (sets enhancement_level = 3)

These alternative keywords set both the mission name and indicate special features/enhancements.

#### `num_levels` - Regular Level Count

Number of regular levels, followed by level filenames (one per line).

```
num_levels = 3
level01.rdl
level02.rdl
level03.rdl
```

- **Type**: Integer
- **Range**: 1-100 (MAX_LEVELS_PER_MISSION)
- **Required**: Yes (at least 1 level)
- **Followed by**: N lines of level filenames

**Level filename format:**
- Descent 1: `.rdl` or `.sdl` (shareware demo levels)
- Descent 2: `.rl2` or `.sl2` (shareware demo levels)
- Max length: 255 characters (for enhanced missions) or 12 characters (standard)

### Optional Fields

#### `type` - Mission Type

Mission gameplay type (optional).

```
type = normal
```

- **Type**: String
- **Common values**: `normal`, `anarchy`
- **Required**: No

#### `hog` - Associated HOG File

HOG archive file containing mission assets (textures, levels, sounds, etc.).

```
hog = mymission.hog
```

- **Type**: Filename string
- **Required**: No (missions can use loose files or game's main HOG)

#### `briefing` - Briefing Text File

Text file containing mission briefing (shown before starting mission).

```
briefing = briefing.txb
```

- **Type**: Filename string
- **Max length**: 12 characters (DOS 8.3 naming + extension)
- **Required**: No
- **File format**: .TXB (Descent briefing text format)

#### `ending` - Ending Text File

Text file containing mission ending/credits (shown after completing mission).

```
ending = ending.txb
```

- **Type**: Filename string
- **Max length**: 12 characters
- **Required**: No
- **File format**: .TXB

#### `num_secrets` - Secret Level Count

Number of secret levels, followed by secret level data (one per line).

```
num_secrets = 2
levels1.rdl,10
levels2.rdl,21,24
```

- **Type**: Integer
- **Range**: 0-20 (MAX_SECRET_LEVELS_PER_MISSION / 5)
- **Required**: No
- **Followed by**: N lines of secret level data

**Secret level format:**
```
filename.rdl,level1,level2,...
```

- **Filename**: Secret level file
- **level1, level2, ...**: Base level numbers (1-indexed) from which this secret level can be accessed
- At least one base level link is required

**Example:**
```
levels1.rdl,10
```
This secret level can be accessed from level 10.

```
levels2.rdl,21,24
```
This secret level can be accessed from level 21 or level 24.

---

## Complete Example

### Descent 2 Mission File (d2.mn2)

```ini
; Descent 2: Counterstrike
; Official Descent 2 campaign

name = Descent 2: Counterstrike
type = normal
hog = d2.hog
briefing = d2.txb
ending = endreg.txb

num_levels = 24
d2leva-1.rl2
d2leva-2.rl2
d2leva-3.rl2
d2leva-4.rl2
d2levb-1.rl2
d2levb-2.rl2
d2levb-3.rl2
d2levb-4.rl2
d2levc-1.rl2
d2levc-2.rl2
d2levc-3.rl2
d2levc-4.rl2
d2levd-1.rl2
d2levd-2.rl2
d2levd-3.rl2
d2levd-4.rl2
d2leve-1.rl2
d2leve-2.rl2
d2leve-3.rl2
d2leve-4.rl2
d2levf-1.rl2
d2levf-2.rl2
d2levf-3.rl2
d2levf-4.rl2

num_secrets = 6
d2leva-s.rl2,1
d2levb-s.rl2,5
d2levc-s.rl2,9
d2levd-s.rl2,13
d2leve-s.rl2,17
d2levf-s.rl2,21
```

### Descent 1 Mission File (descent.msn)

```ini
; Descent: First Strike
; Original Descent 1 campaign

name = Descent: First Strike
type = normal
briefing = briefing.tex
ending = endreg.tex

num_levels = 27
level01.rdl
level02.rdl
level03.rdl
level04.rdl
level05.rdl
level06.rdl
level07.rdl
level08.rdl
level09.rdl
level10.rdl
level11.rdl
level12.rdl
level13.rdl
level14.rdl
level15.rdl
level16.rdl
level17.rdl
level18.rdl
level19.rdl
level20.rdl
level21.rdl
level22.rdl
level23.rdl
level24.rdl
level25.rdl
level26.rdl
level27.rdl

num_secrets = 3
levels1.rdl,10
levels2.rdl,21
levels3.rdl,24
```

### Enhanced D2X-XL Mission

```ini
; D2X-XL enhanced mission with extended features

d2x-name = [XL] Mega Mission
type = normal
hog = mega.hog
briefing = mega.txb

; D2X-XL supports much longer filenames
num_levels = 5
levels/intro_station.rl2
levels/mining_complex_alpha.rl2
levels/core_reactor_facility.rl2
levels/weapons_research_lab.rl2
levels/final_showdown.rl2

num_secrets = 1
levels/hidden_cache.rl2,3,4
```

---

## Parsing Algorithm

### Step-by-Step

1. **Read file line-by-line**
2. **For each line:**
   - Remove comments (everything after `;`)
   - Trim whitespace
   - Skip empty lines
3. **Parse key-value pairs:**
   - Split on `=` character
   - Trim key and value
   - Match key (case-insensitive)
4. **For `num_levels` and `num_secrets`:**
   - Parse count value
   - Read N subsequent lines as data
   - Trim and validate each line
5. **Validate required fields:**
   - Mission name must be present
   - At least one level must be defined

### Parsing Helper Functions (from D2X-XL)

**MsnGetS** - Read line and strip newline
```c
char *MsnGetS(char *s, int n, FILE *fp) {
    char *r = fgets(s, n, fp);
    if (r) {
        int l = strlen(s) - 1;
        if (s[l] == '\n')
            s[l] = 0;
    }
    return r;
}
```

**MsnTrimComment** - Remove comments
```c
char *MsnTrimComment(char *buf) {
    char *ps = strchr(buf, ';');
    if (ps) {
        while (ps > buf && isspace(*(ps-1)))
            --ps;
        *ps = '\0';
    }
    return buf;
}
```

**MsnIsTok** - Check if line starts with token
```c
int MsnIsTok(char *buf, const char *tok) {
    return !strnicmp(buf, tok, strlen(tok));
}
```

**MsnGetValue** - Extract value after '='
```c
char *MsnGetValue(char *buf) {
    char *t = strchr(buf, '=');
    if (t) {
        t++;
        while (*t && isspace(*t))
            t++;
        if (*t)
            return t;
    }
    return NULL;
}
```

---

## Implementation Notes

### Case Sensitivity

Keys are case-insensitive. These are equivalent:
```
name = My Mission
Name = My Mission
NAME = My Mission
```

### Comment Handling

Comments can appear:
- On their own line
- At the end of a line (inline)

```
; Full line comment
name = Test Mission  ; inline comment
```

### Whitespace Handling

Leading and trailing whitespace is ignored:
```
  name  =  Test Mission  
```
Parses as: `name = "Test Mission"`

### Enhancement Levels

```rust
pub struct MissionFile {
    pub enhancement_level: u8,
    // 0 = standard (used 'name')
    // 1 = enhanced (used 'xname')
    // 2 = z-enhanced (used 'zname')
    // 3 = D2X-XL (used 'd2x-name')
}
```

Enhancement levels indicate mission features:
- **0 (standard)**: Compatible with original Descent 1/2
- **1-2 (enhanced)**: Uses extended features from various mods
- **3 (D2X-XL)**: Requires D2X-XL engine, uses advanced features

### Secret Level Links

Secret levels must specify at least one base level:
```
levels1.rdl,10        ; Can access from level 10
levels2.rdl,5,15,20   ; Can access from levels 5, 15, or 20
```

Level numbers are 1-indexed (level01.rdl = 1, level02.rdl = 2, etc.).

### Filename Constraints

**Standard missions (enhancement_level 0):**
- Level filenames: Max 12 characters (DOS 8.3 format)
- Briefing/ending: Max 12 characters

**Enhanced missions (enhancement_level > 0):**
- Level filenames: Max 255 characters
- Can use subdirectories (e.g., `levels/intro.rl2`)

### Error Handling

**Invalid mission files:**
- Missing `name` field → Error
- `num_levels` with insufficient level lines → Error
- Secret level without base level links → Error
- Briefing filename > 12 chars → Error
- Level count > 100 → Error
- Secret level count > 20 → Error

---

## Mission Discovery

### Mission Locations

The game searches for mission files in:
1. `missions/` directory
2. `missions/single/` directory (single-player)
3. Current directory
4. CD-ROM directory (if applicable)

### Mission List Building

When the game starts:
1. Scans mission directories for `.msn` and `.mn2` files
2. Parses each mission file to extract name and metadata
3. Builds mission list for menu
4. Sorts missions alphabetically

**Mission list entry structure:**
```c
typedef struct {
    char filename[FILENAME_LEN];     // Without extension
    char mission_name[MISSION_NAME_LEN + 5 + 1];
    uint8_t anarchy_only;            // If true, anarchy mode only
    uint8_t location;                // Where mission was found
    uint8_t descent_version;         // 1 or 2
} tMsnListEntry;
```

---

## Reference Implementation

The Rust implementation in `descent-core` provides:

```rust
/// Mission file parser for Descent 1 (.msn) and Descent 2 (.mn2)
pub struct MissionFile {
    pub name: String,
    pub mission_type: Option<String>,
    pub hog_file: Option<String>,
    pub briefing_file: Option<String>,
    pub ending_file: Option<String>,
    pub levels: Vec<String>,
    pub secret_levels: Vec<SecretLevel>,
    pub enhancement_level: u8,
}

/// Secret level with links to base levels
pub struct SecretLevel {
    pub filename: String,
    pub linked_from: Vec<u32>,  // 1-indexed level numbers
}

impl MissionFile {
    /// Parse mission file from text content
    pub fn parse(content: &str) -> Result<Self>;
}
```

### Usage Example

```rust
use descent_core::MissionFile;

// Read mission file
let content = std::fs::read_to_string("mymission.mn2")?;
let mission = MissionFile::parse(&content)?;

println!("Mission: {}", mission.name);
println!("Enhancement level: {}", mission.enhancement_level);

// List levels
for (i, level) in mission.levels.iter().enumerate() {
    println!("  Level {}: {}", i + 1, level);
}

// List secret levels
for secret in &mission.secret_levels {
    println!("  Secret: {} (from levels: {:?})", 
             secret.filename, secret.linked_from);
}
```

---

## Further Reading

- **D2X-XL Source**: `/tmp/d2x-xl/include/mission.h` - Mission structures
- **D2X-XL Source**: `/tmp/d2x-xl/gameio/mission.cpp` - Mission parsing code
- **Briefing Format**: `.TXB` files (text-based mission briefings)
- **Level Format**: `RDL_FORMAT.md` and `RL2_FORMAT.md` for level files
- **HOG Format**: `HOG_FORMAT.md` for HOG archive structure

---

## Version History

- **2026-02-24**: Initial documentation based on D2X-XL source analysis
- **Format Era**: 1995-1996 (Descent 1 & 2 release period)
- **Reference**: D2X-XL 1.18.77 source code (`mission.h`, `mission.cpp`)

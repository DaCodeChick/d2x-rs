# Descent 1/2 Sound Formats

This document describes the sound and music formats used in Descent 1 and Descent 2, based on the D2X-XL source code and reverse engineering.

## Overview

Descent 1 and 2 use two distinct audio formats:

1. **SNDs (Digitized Sound Effects)** - 8-bit PCM audio stored within PIG files
2. **HMP (HMI MIDI Music)** - Multi-track MIDI files using HMI format

Both formats were standard in DOS games of the mid-1990s era.

---

## SNDs Format - Digitized Sound Effects

### File Location

Sound effects in Descent 1/2 are **not** separate files. They are embedded inside PIG files alongside textures and bitmaps. Each sound has:
- A header entry in the PIG file's sound table
- Raw PCM audio data stored after the bitmap data

### Sound Header Structure

Each sound has a 20-byte header (defined in D2X-XL's `piggy.h` as `tPIGSoundHeader`):

```
Offset  Size  Type    Description
------  ----  ----    -----------
0x00    8     char    Sound name (null-padded, not null-terminated)
0x08    4     u32     Total length (header + data, little-endian)
0x0C    4     u32     Data length (PCM data only, little-endian)
0x10    4     u32     Offset within PIG file (little-endian)
```

**Example Header (hex dump):**
```
00000000: 42 4F 4F 4D 30 31 00 00  08 20 00 00 00 20 00 00  |BOOM01... ... ..|
00000010: 40 10 00 00                                       |@...|
```

This represents:
- Name: "BOOM01" (padded to 8 bytes)
- Total length: 0x2008 (8,200 bytes)
- Data length: 0x2000 (8,192 bytes)
- Offset: 0x1040 (4,160 bytes into PIG file)

### Audio Data Format

After the header, the sound data is stored as:

- **Format**: Raw 8-bit unsigned PCM
- **Channels**: Mono (1 channel)
- **Sample Rate**: Not stored in header! Specified in HAM file metadata
  - Common rates: 11025 Hz (11 KHz) or 22050 Hz (22 KHz)
  - Some rare sounds at 8000 Hz
- **Bit Depth**: 8-bit unsigned (values 0-255)
- **Endianness**: N/A (single bytes)

**PCM Sample Range:**
- `0x00` (0) = Maximum negative amplitude
- `0x80` (128) = Zero/silence
- `0xFF` (255) = Maximum positive amplitude

### Conversion to Float

For modern audio APIs (Bevy, rodio, cpal), convert u8 to f32:

```rust
/// Convert 8-bit unsigned PCM to f32 samples (-1.0 to +1.0)
pub fn to_f32_samples(&self) -> Vec<f32> {
    self.data
        .iter()
        .map(|&sample| (sample as f32 - 128.0) / 128.0)
        .collect()
}
```

This maps:
- `0` → `-1.0` (full negative)
- `128` → `0.0` (silence)
- `255` → `~0.992` (full positive)

### Usage Example

```rust
use d2x_assets::{SoundHeader, SoundData, PigFile};

// Parse PIG file
let pig_data = std::fs::read("groupa.pig")?;
let pig = PigFile::parse(&pig_data)?;

// Read sound header from PIG sound table
let header = SoundHeader::parse(&pig_data[sound_offset..sound_offset+20])?;
println!("Sound: {}, Length: {} bytes", header.name, header.data_length);

// Extract PCM data using offset from header
let sound = SoundData::parse(&pig_data, &header)?;

// Convert to f32 for playback
let samples = sound.to_f32_samples();

// Play using audio engine (pseudo-code)
audio_engine.play(samples, sample_rate_hz);
```

### Sound Types

Descent uses digitized sounds for:
- **Weapons**: Laser fire, missile launch, bomb explosions
- **Robot**: Robot alerts, death sounds, claw attacks
- **Player**: Shield/hull damage, powerup pickup, death
- **Ambient**: Door opening/closing, reactor countdown, mine destruction
- **Voice**: Pilot taunts (Descent 2 only)

### Sound Naming Convention

Sound names in PIG files follow patterns:
- `BOOMxx` - Explosion sounds
- `LASRxx` - Laser fire
- `ROBTxx` - Robot sounds
- `DORRxx` - Door sounds
- `PLYRxx` - Player sounds
- `POWRxx` - Powerup sounds

Names are limited to 8 characters (DOS 8.3 convention).

---

## HMP Format - HMI MIDI Music

### File Location

Music files are **separate** files with `.hmp` extension, stored in HOG archives or as loose files. Common examples:
- `game01.hmp` - Level 1 music
- `game02.hmp` - Level 2 music
- `descent.hmp` - Main menu theme
- `briefing.hmp` - Briefing screen music

### File Structure

HMP is a proprietary MIDI format created by Human Machine Interfaces (HMI). It differs from standard MIDI files.

**File Layout:**
```
Offset    Size   Description
--------  -----  -----------
0x000     8      Signature: "HMIMIDIP"
0x008     296    Header data (mostly unused/reserved)
0x030     4      Number of tracks (u32, little-endian, range: 1-32)
0x038     4      MIDI division (u32, little-endian, ticks per quarter note)
0x308     var    Track data (n tracks)
```

### Header Structure

```rust
/// HMP file signature (8 bytes)
const HMP_SIGNATURE: &[u8; 8] = b"HMIMIDIP";

/// Maximum tracks allowed in HMP file
const HMP_MAX_TRACKS: u32 = 32;

/// Offset to track count field
const TRACK_COUNT_OFFSET: usize = 0x30;

/// Offset to MIDI division field
const DIVISION_OFFSET: usize = 0x38;

/// Offset where track data begins
const TRACK_DATA_OFFSET: usize = 0x308;
```

### Track Structure

Each track has a 12-byte header followed by MIDI data:

```
Offset  Size  Type   Description
------  ----  ----   -----------
+0      4     u32    Skip/padding (unused, typically 0)
+4      4     u32    Track length (includes 12-byte overhead, little-endian)
+8      4     u32    Skip/padding (unused, typically 0)
+12     var   u8[]   MIDI event data (length = track_length - 12)
```

**Track Length Calculation:**
```rust
let midi_data_length = track_length - 12;
```

### MIDI Data Format

HMP uses HMI variable-length encoding, which **differs** from standard MIDI:

**Standard MIDI VLQ (Variable Length Quantity):**
- Uses 7 bits per byte, MSB=1 for continuation
- Example: `0x81 0x00` = 128

**HMI VLQ (used in HMP):**
- Different encoding scheme
- Must be decoded with HMI-specific decoder

**Common MIDI Events in HMP:**
- `0x90-0x9F` - Note On (channel 0-15)
- `0x80-0x8F` - Note Off (channel 0-15)
- `0xB0-0xBF` - Control Change
- `0xC0-0xCF` - Program Change (instrument selection)
- `0xE0-0xEF` - Pitch Bend
- `0xFF` - Meta events (tempo, end of track)

### Timing and Tempo

- **Division**: Ticks per quarter note (typically 60, 96, or 120)
- **Default Tempo**: 120 BPM (if not specified by tempo event)
- **Tempo Event**: `0xFF 0x51 0x03 [3 bytes]` - Microseconds per quarter note

**Calculate BPM from tempo event:**
```rust
let microseconds_per_quarter = u32::from_be_bytes([0, tt, tt, tt]); // 3 bytes
let bpm = 60_000_000 / microseconds_per_quarter;
```

### Parsing Example

```rust
use d2x_assets::HmpFile;

// Read HMP file
let data = std::fs::read("game01.hmp")?;
let hmp = HmpFile::parse(&data)?;

println!("Tracks: {}", hmp.track_count);
println!("Division: {} ticks/quarter", hmp.division);

// Iterate through tracks
for (i, track) in hmp.tracks.iter().enumerate() {
    println!("Track {}: {} bytes", i, track.data.len());
}

// To play: Use a MIDI library that supports HMI format
// or convert to standard MIDI first
```

### HMP vs Standard MIDI

| Feature              | HMP Format            | Standard MIDI (.mid)     |
|----------------------|-----------------------|--------------------------|
| Signature            | "HMIMIDIP"            | "MThd"                   |
| Track count offset   | 0x30                  | Header chunk             |
| Division offset      | 0x38                  | Header chunk             |
| Track data offset    | 0x308 (fixed)         | After header             |
| Variable-length nums | HMI encoding          | MIDI VLQ (7-bit)         |
| Max tracks           | 32                    | 65,535                   |
| Track overhead       | 12 bytes              | 8 bytes (chunk header)   |

### Converting HMP to MIDI

To convert HMP to standard MIDI:

1. Parse HMP header to get track count and division
2. For each track, decode HMI variable-length numbers to MIDI VLQ
3. Write standard MIDI file header (`MThd` chunk)
4. Write each track as `MTrk` chunk with converted data
5. Ensure all tracks end with `0xFF 0x2F 0x00` (End of Track)

This conversion is complex and requires understanding both formats. Libraries like `libADLMIDI` or D2X-XL's HMP player can handle HMP directly.

### Music Types

Descent uses HMP music for:
- **Level Music**: Different tracks for each level
- **Menu Music**: Main menu, briefings, credits
- **Boss Music**: Special tracks for boss encounters
- **Ending Music**: Victory/defeat sequences

---

## Implementation Notes

### Memory Considerations

**Sound Effects:**
- Small files (typically 2-16 KB per sound)
- Loaded on-demand from PIG file
- Can be cached in memory for frequently used sounds
- Total sound data in Descent 2: ~5-10 MB

**Music Files:**
- Small MIDI data (typically 10-50 KB per track)
- Much smaller than equivalent MP3/OGG
- Can stream or preload entire file
- Total music data: ~500 KB - 1 MB

### Sample Rate Detection

Since sound headers don't store sample rate, you must:

1. Parse HAM file to get sound metadata
2. Look up sound ID in HAM sound table
3. Read sample rate from HAM entry
4. Default to 11025 Hz if HAM unavailable

**HAM Sound Entry (pseudo-structure):**
```rust
struct HamSound {
    sound_id: u16,
    sample_rate: u32,  // 8000, 11025, or 22050 Hz
    flags: u32,
}
```

### Audio Playback Integration

**For Bevy Engine:**
```rust
use bevy::prelude::*;
use bevy::audio::*;

fn load_sound(sound_data: &SoundData, sample_rate: u32) -> AudioSource {
    let samples = sound_data.to_f32_samples();
    
    // Create audio source
    AudioSource {
        samples,
        sample_rate,
        channels: 1, // Mono
    }
}
```

**For rodio:**
```rust
use rodio::{Decoder, OutputStream, Sink, Source};

fn play_sound(sound_data: &SoundData, sample_rate: u32) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    
    let samples = sound_data.to_f32_samples();
    let source = rodio::buffer::SamplesBuffer::new(1, sample_rate, samples);
    
    sink.append(source);
    sink.play();
}
```

### HMP Playback

For HMP music playback, consider:

1. **Use existing MIDI library**: `midly`, `rustysynth`, or `libADLMIDI`
2. **Convert to standard MIDI**: Write converter to .mid format
3. **Use D2X-XL approach**: Port HMP player from D2X-XL source
4. **Modern alternative**: Replace with MP3/OGG versions of Descent music

---

## Reference Implementation

The Rust implementation in `d2x-assets` provides:

### Sound Types

```rust
/// Sound header from PIG file (20 bytes)
pub struct SoundHeader {
    pub name: String,          // Up to 8 characters
    pub total_length: u32,     // Header + data
    pub data_length: u32,      // PCM data only
    pub offset: u32,           // Offset in PIG file
}

/// Sound data with PCM samples
pub struct SoundData {
    pub header: SoundHeader,
    pub data: Vec<u8>,         // 8-bit unsigned PCM
}

impl SoundData {
    /// Parse sound from PIG file
    pub fn parse(pig_data: &[u8], header: &SoundHeader) -> Result<Self>;
    
    /// Convert to f32 samples for playback
    pub fn to_f32_samples(&self) -> Vec<f32>;
}
```

### HMP Types

```rust
/// Single HMP track
pub struct HmpTrack {
    pub data: Vec<u8>,         // MIDI event data
}

/// Complete HMP file
pub struct HmpFile {
    pub track_count: u32,      // 1-32 tracks
    pub division: u32,         // Ticks per quarter note
    pub tracks: Vec<HmpTrack>, // Track data
}

impl HmpFile {
    /// Parse HMP file
    pub fn parse(data: &[u8]) -> Result<Self>;
}
```

---

## Further Reading

- **D2X-XL Source**: `/tmp/d2x-xl/include/piggy.h` - Sound header definition
- **D2X-XL Source**: `/tmp/d2x-xl/include/hmpfile.h` - HMP structure
- **D2X-XL Source**: `/tmp/d2x-xl/audio/linux/hmpfile.cpp` - HMP parser implementation
- **Descent Data Format**: Community wiki on game formats
- **HMI MIDI Format**: Technical documentation from HMI/AdLib era

---

## Version History

- **2026-02-24**: Initial documentation based on D2X-XL source analysis
- **Format Era**: 1995-1996 (Descent 1 & 2 release period)
- **Reference**: D2X-XL 1.18.77 source code

# MVE/MVL Movie File Formats (Descent 1 & 2)

## Overview

MVE (Interplay Movie) and MVL (Movie Library) files are the video cutscene formats used in Descent 1 and 2. MVE files contain full-motion video with audio, while MVL files are archive files that contain multiple MVE movies.

**Key Facts:**
- **Format**: Interplay MVE (Motion Video Entertainment)
- **Used In**: Descent 1, Descent 2, and many other Interplay games
- **Video**: Custom codec with 8-bit indexed color
- **Audio**: PCM audio data
- **Compression**: Custom frame compression
- **Resolution**: Typically 320x200 or 640x480

## File Types

### MVL Files (Movie Library)

MVL files are simple archive formats similar to HOG/DHF but specifically for movies.

**Common MVL Files:**
- `intro-l.mvl` / `intro-h.mvl` - Intro movies (low/high resolution)
- `other-l.mvl` / `other-h.mvl` - Other cutscene movies
- `robots-l.mvl` / `robots-h.mvl` - Robot briefing movies
- `d2x-l.mvl` / `d2x-h.mvl` - Additional D2X movies

### MVE Files (Movie)

MVE files are individual movie files that can exist inside MVL archives or as standalone files.

**Common MVE Files:**
- `intro.mve` - Game intro
- `psh.mve` - Pyro-GX ship intro
- `credits.mve` - End credits
- Various robot briefing movies

## MVL Archive Format

MVL archives use a simple header + directory + data structure:

```
[Header - 8 bytes]
  Offset | Size | Type  | Description
  -------|------|-------|----------------------------------
  0x00   | 4    | char  | Signature: "DMVL"
  0x04   | 4    | i32   | Number of files (little-endian)

[File Entries] (repeated num_files times)
  Each entry is 17 bytes:
  
  Offset | Size | Type  | Description
  -------|------|-------|----------------------------------
  0x00   | 13   | char  | Filename (null-terminated)
  0x0D   | 4    | i32   | File size (little-endian)

[File Data]
  Concatenated MVE file data in the order listed in the directory.
  First file starts at offset: 4 + 4 + (num_files * 17)
```

### MVL Example

```
Offset(h) | 00 01 02 03 04 05 06 07 | ASCII
----------|-------------------------|--------
00000000  | 44 4D 56 4C 02 00 00 00 | DMVL....   ; Signature + 2 files
00000008  | 49 4E 54 52 4F 2E 4D 56 | INTRO.MV   ; Filename 1: "INTRO.MVE"
00000010  | 45 00 00 00 00 00 20 00 | E..... .   ; Size 1: 0x2000 = 8192 bytes
00000018  | 43 52 45 44 49 54 53 2E | CREDITS.   ; Filename 2: "CREDITS.MVE"
00000020  | 4D 56 45 00 00 40 00 00 | MVE..@..   ; Size 2: 0x4000 = 16384 bytes
00000028  | [MVE data for INTRO.MVE]              ; First movie data
00002028  | [MVE data for CREDITS.MVE]            ; Second movie data
```

## MVE Movie Format

MVE files use a chunk-based structure similar to RIFF/AVI containers but with Interplay's proprietary format.

```
[Header - 26 bytes]
  Offset | Size | Type   | Description
  -------|------|--------|----------------------------------
  0x00   | 20   | char   | Signature: "Interplay MVE File\x1A\x00"
  0x14   | 2    | u16    | Header constant 1: 0x001A
  0x16   | 2    | u16    | Header constant 2: 0x0100
  0x18   | 2    | u16    | Header constant 3: 0x1133

[Chunks] (repeated until end of file)
  Each chunk:
  
  Offset | Size | Type   | Description
  -------|------|--------|----------------------------------
  0x00   | 2    | u16    | Chunk length (little-endian)
  0x02   | N    | byte[] | Chunk data (contains segments)
  N+2    | 2    | u16    | Chunk type (usually 0x0000)

[Segments within chunks]
  Each segment:
  
  Offset | Size | Type   | Description
  -------|------|--------|----------------------------------
  0x00   | 2    | u16    | Segment length (little-endian)
  0x02   | 1    | u8     | Major type (segment category)
  0x03   | 1    | u8     | Minor type (segment version)
  0x04   | N    | byte[] | Segment data
```

### MVE Segment Types

The major type field identifies the purpose of each segment:

| Major | Name                   | Description                                    |
|-------|------------------------|------------------------------------------------|
| 0x00  | End of Stream          | Marks the end of the movie                     |
| 0x01  | End of Chunk           | Marks the end of segments in current chunk     |
| 0x02  | Create Timer           | Sets up timing information                     |
| 0x03  | Initialize Audio       | Initializes audio buffers                      |
| 0x04  | Start/Stop Audio       | Controls audio playback                        |
| 0x05  | Initialize Video       | Initializes video buffers                      |
| 0x07  | Display Video Frame    | Signals to display the current frame           |
| 0x08  | Audio Frame Data       | Contains compressed audio data                 |
| 0x09  | Audio Frame Silence    | Indicates a silent audio frame                 |
| 0x0A  | Initialize Video Mode  | Sets video resolution and color depth          |
| 0x0B  | Create Gradient        | Creates a color gradient (rarely used)         |
| 0x0C  | Set Palette            | Sets the 256-color palette                     |
| 0x0D  | Set Palette Compressed | Sets palette with run-length encoding          |
| 0x0E  | Unknown                | Purpose unknown                                |
| 0x0F  | Set Decoding Map       | Sets up video decompression lookup table       |
| 0x11  | Video Data             | Contains compressed video frame data           |

### Video Decompression

MVE uses a proprietary frame-based compression algorithm:

1. **Indexed Color**: Frames use 8-bit indexed color (256 colors)
2. **Palette Updates**: Palette can change between frames
3. **Block-Based Encoding**: Video is divided into blocks (typically 8x8 or 4x4)
4. **Delta Encoding**: Frames can reference previous frames
5. **Multiple Codecs**: Different minor types indicate different compression methods

The decompression map (segment 0x0F) provides lookup tables for decoding video data blocks.

### Audio Format

Audio data in MVE files is stored as:
- **Format**: PCM (Pulse Code Modulation)
- **Bit Depth**: 8-bit or 16-bit
- **Channels**: Mono or stereo
- **Sample Rate**: Varies (typically 11025 Hz or 22050 Hz)

Audio is synchronized with video using timing segments.

## Implementation Details

### MVL Archive Reading

**Algorithm:**
1. Read and verify "DMVL" signature
2. Read file count (i32, little-endian)
3. Calculate data offset: 8 + (file_count * 17)
4. For each file entry:
   - Read 13-byte filename
   - Read 4-byte file size
   - Store entry with calculated offset
   - Advance offset by file size

**Edge Cases:**
- Empty archives (file count = 0)
- Very large file counts (sanity check: max 1000 files)
- Filenames with null bytes (stop at first null)
- Negative file sizes (validation error)

### MVE Header Validation

**Algorithm:**
1. Verify file is at least 26 bytes
2. Check signature matches "Interplay MVE File\x1A\x00"
3. Verify constant1 = 0x001A
4. Verify constant2 = 0x0100
5. Verify constant3 = 0x1133

All constants must match exactly or the file is considered invalid.

### Chunk Iteration

**Algorithm:**
1. Start at offset 26 (after header)
2. While not at end of file:
   - Read 2-byte chunk length
   - Read chunk data (length bytes)
   - Skip 2-byte chunk type field
   - Move to next chunk

**End Detection:**
- End of file
- Segment type 0x00 (End of Stream)
- Chunk length exceeds remaining file size

### Segment Iteration

**Algorithm:**
1. Start at beginning of chunk data
2. While not at end of chunk:
   - Read 2-byte segment length
   - Read 1-byte major type
   - Read 1-byte minor type
   - Process segment data (length bytes)
   - Stop if major type = 0x01 (End of Chunk)
   - Move to next segment

## Rust Implementation

### MVL Archive Example

```rust
use descent_core::{MvlArchive, Result};

fn extract_movies() -> Result<()> {
    // Open MVL archive
    let mut mvl = MvlArchive::open("intro-l.mvl")?;
    
    // List all movies
    println!("Movies in archive:");
    for entry in mvl.entries() {
        println!("  {} - {} bytes", entry.name, entry.size);
    }
    
    // Extract a specific movie
    let intro_data = mvl.read_file("intro.mve")?;
    std::fs::write("intro.mve", intro_data)?;
    
    Ok(())
}
```

### MVE File Parsing Example

```rust
use descent_core::{MveFile, Result};

fn analyze_movie() -> Result<()> {
    // Load MVE file
    let mve_data = std::fs::read("intro.mve")?;
    let mve = MveFile::parse(&mve_data)?;
    
    // Count chunks
    let chunk_count = mve.chunk_count();
    println!("Movie has {} chunks", chunk_count);
    
    // Iterate through chunks
    for (i, chunk) in mve.chunks().enumerate() {
        println!("Chunk {}: {} bytes", i, chunk.length);
        
        // Iterate through segments in chunk
        for segment in mve.chunk_segments(&chunk) {
            println!("  Segment type 0x{:02X} (minor 0x{:02X}): {} bytes",
                segment.major_type, segment.minor_type, segment.length);
        }
    }
    
    Ok(())
}
```

### Extracting Video Metadata

```rust
use descent_core::{MveFile, MveSegmentType, Result};
use std::convert::TryFrom;

fn get_video_info() -> Result<()> {
    let mve_data = std::fs::read("intro.mve")?;
    let mve = MveFile::parse(&mve_data)?;
    
    for chunk in mve.chunks() {
        for segment in mve.chunk_segments(&chunk) {
            if let Ok(seg_type) = MveSegmentType::try_from(segment.major_type) {
                match seg_type {
                    MveSegmentType::InitVideoMode => {
                        let data = mve.get_segment_data(&segment);
                        // Parse video mode data (width, height, etc.)
                        println!("Video mode segment: {} bytes", data.len());
                    }
                    MveSegmentType::SetPalette => {
                        let data = mve.get_segment_data(&segment);
                        // Parse palette (256 RGB entries)
                        println!("Palette segment: {} bytes", data.len());
                    }
                    _ => {}
                }
            }
        }
    }
    
    Ok(())
}
```

## Video Playback Considerations

### Decoding Requirements

To fully decode and play MVE movies:

1. **Palette Handling**: Track palette changes throughout playback
2. **Frame Buffer**: Maintain double-buffered video frames
3. **Audio Buffer**: Queue audio data for continuous playback
4. **Timing**: Use timer segments to synchronize audio/video
5. **Decompression**: Implement video decompression algorithms

### Performance Notes

- MVE decompression is CPU-intensive
- Original code used assembly optimizations
- Modern implementations can use SIMD instructions
- Frame-by-frame decoding allows seeking

### Alternative: Conversion

Instead of implementing full MVE playback, consider:

1. **Pre-conversion**: Convert MVE to modern formats (MP4, WebM)
2. **External Tools**: Use existing MVE decoders
3. **Streaming**: Stream decoded frames to video player
4. **Skip Movies**: Provide option to skip cutscenes

## File Relationships

```
intro-l.mvl (Archive)
  ├── intro.mve     (Intro cutscene)
  ├── psh.mve       (Pyro-GX ship)
  └── logo.mve      (Company logos)

other-l.mvl (Archive)
  ├── briefing.mve  (Mission briefing)
  ├── endgame.mve   (Ending cutscene)
  └── credits.mve   (Credits roll)

robots-l.mvl (Archive)
  ├── robot01.mve   (Robot 1 briefing)
  ├── robot02.mve   (Robot 2 briefing)
  └── ...           (More robots)
```

## Size Information

**Typical File Sizes:**

MVL Archives:
- `intro-l.mvl`: ~2-5 MB (low resolution)
- `intro-h.mvl`: ~10-20 MB (high resolution)
- `robots-l.mvl`: ~5-10 MB (multiple short clips)

Individual MVE Files:
- Short clips (logos): 100-500 KB
- Medium cutscenes: 1-3 MB
- Long cutscenes: 5-10 MB

**Resolution Impact:**
- Low resolution (320x200): Smaller files, faster playback
- High resolution (640x480): 4x larger, better quality

## Historical Context

### Interplay MVE Format

The MVE format was developed by Interplay Productions and used in many of their games:

- **Descent 1 & 2** (1995-1996)
- **Fallout 1 & 2** (1997-1998)
- **Baldur's Gate** (1998)
- **Icewind Dale** (2000)
- Many other Interplay titles

### Why MVE?

In the mid-1990s:
- CD-ROM storage was limited
- CPUs were slow for video decoding
- MPEG-1 was too demanding
- MVE provided good compression with reasonable CPU usage

### Modern Alternatives

Today, MVE is obsolete:
- **MP4/H.264**: Better compression, hardware acceleration
- **WebM/VP9**: Open format, excellent quality
- **AV1**: Next-generation codec

However, for preservation and compatibility, MVE support remains important.

## Technical Limitations

### Original Implementation Constraints

- **Resolution**: Limited to 640x480 maximum
- **Color Depth**: 8-bit indexed color only
- **Frame Rate**: Typically 15 FPS
- **Audio Quality**: Low bitrate, often mono
- **Seeking**: Limited or no seeking support

### Modern Playback Issues

- **Color Accuracy**: 256-color palette limitations
- **Scaling**: Pixelation when upscaling
- **Synchronization**: Manual audio/video sync needed
- **Platform Differences**: Endianness issues on big-endian systems

## Testing

The descent-core crate includes unit tests for MVE/MVL parsing:

```bash
# Run MVE/MVL tests
cargo test --package descent-core -- mve
cargo test --package descent-core -- mvl

# Test with verbose output
cargo test --package descent-core -- mve --nocapture
```

**Test Coverage:**
- MVL signature validation
- MVL file entry parsing
- MVE header validation
- MVE segment type conversion
- Iterator functionality
- Edge cases (empty files, invalid headers)

## References

### Source Code

- **D2X-XL**: `main/movie.cpp`, `libmve/mvelib.cpp`
- **descent-core**: `src/mvl.rs`, `src/mve.rs`

### Documentation

- Interplay MVE format (unofficial specifications)
- D2X-XL source code comments
- DXX-Rebirth movie handling

### Tools

- **FFmpeg**: Native "Interplay MVE" demuxer support (`ipmovie` format)
- **VLC**: Basic MVE playback
- **D2X-RS Setup**: Automated MVE→MP4 conversion during first-time setup

## Conversion Strategy

### Why Convert to MP4/H.264?

Rather than implementing a full MVE decoder, D2X-RS converts MVE files to modern MP4/H.264 format during the **first-time setup process**. This approach offers significant advantages:

**Performance Benefits:**
- ✅ **Hardware acceleration**: GPUs can decode H.264 natively
- ✅ **Lower CPU usage**: ~10-20x less CPU than software MVE decoding
- ✅ **Faster loading**: Modern container formats optimize for streaming

**Technical Benefits:**
- ✅ **Better compression**: H.264 achieves 10-50x smaller files than 1990s MVE codec
- ✅ **Proven integration**: Works seamlessly with Bevy video plugins (`bevy_video`)
- ✅ **Maintenance**: No complex codec code to debug and maintain
- ✅ **Quality**: Can upscale/enhance during conversion

**User Experience:**
- ✅ **One-time conversion**: Happens automatically during game setup
- ✅ **No manual work**: Players just select Descent folder and click "Start"
- ✅ **Progress tracking**: Visual feedback during conversion
- ✅ **Familiar formats**: MP4 can be previewed in any media player

### Conversion Process

The conversion happens during the **First-Time Setup** stage:

1. **Setup Detection**: Game checks if `assets/videos/` exists on first launch
2. **User Prompt**: GUI asks player to select Descent installation folder
3. **Extraction**: MVL archives extracted using `descent-core::mvl` parser
4. **Conversion**: FFmpeg (via `ffmpeg-next` Rust crate) converts each MVE to MP4/H.264
5. **Organization**: Converted videos saved to `assets/videos/intro.mp4`, etc.
6. **Progress**: UI shows "Converting Videos..." with progress bar

**FFmpeg Integration:**
- Uses `ffmpeg-next` crate with `BUILD` feature enabled
- **Automatically compiles FFmpeg** from source during `cargo build`
- **Statically links** FFmpeg libraries into the game executable
- **No separate installation required** - users just download and run the game
- FFmpeg's "Interplay MVE" demuxer (`ipmovie`) natively handles MVE decoding

**Conversion Parameters:**
- Video codec: H.264 (libx264)
- Quality: CRF 18 (visually lossless)
- Audio codec: AAC
- Audio bitrate: 128 kbps (sufficient for 22050 Hz PCM source)

### Integration Points

**Setup System** (`d2x-client/src/setup.rs`):
- `ConversionStage::ConvertingVideos` stage added
- `assets/videos/` directory checked on startup
- Progress tracking: `ConversionProgress` tracks current video file

**Asset Loading** (future):
- Bevy asset system loads MP4 files
- `bevy_video` plugin handles playback
- Cutscene system triggers videos at appropriate times

**File Mapping:**
```
Source (Descent Install)    →    Converted (D2X-RS Assets)
intro-l.mvl/intro.mve       →    assets/videos/intro.mp4
intro-l.mvl/psh.mve         →    assets/videos/psh.mp4
robots-l.mvl/*.mve          →    assets/videos/robots/*.mp4
other-l.mvl/*.mve           →    assets/videos/other/*.mp4
```

### Alternative Approaches Considered

**Runtime MVE Decoding:**
- ❌ High CPU cost (software-only decode)
- ❌ Weeks of development (complex codec reverse-engineering)
- ❌ Maintenance burden (custom codec code)
- ❌ No hardware acceleration

**Pre-converted Distribution:**
- ❌ Legal issues (can't ship copyrighted videos)
- ❌ Manual conversion (user friction)

**Hybrid Approach:**
- ✅ **Chosen**: Automated conversion during setup
- ✅ Legal compliance (users provide own files)
- ✅ Optimal performance (modern format)
- ✅ No user friction (GUI-driven process)

## Future Enhancements

### Potential Improvements

1. **Audio Extraction**: Extract audio streams to WAV/OGG (if needed separately)
2. **Frame Export**: Export individual frames as PNG (for debugging/analysis)
3. **Metadata Extraction**: Parse all segment types fully (for advanced features)
4. **Seeking Support**: Implement frame-accurate seeking (if needed for gameplay)
5. **Quality Options**: Configurable CRF/bitrate settings in setup UI
6. **Upscaling**: AI upscaling (e.g., Topaz Video AI) during conversion

### Integration Possibilities

- ✅ **Bevy video playback**: Via `bevy_video` plugin with MP4 files
- ✅ **In-game cutscene system**: Trigger videos at mission start/end
- ✅ **Setup progress UI**: Visual feedback during conversion
- 🚧 **Level editor preview**: Play cutscenes in mission editor
- 🚧 **Asset extraction tool**: Standalone converter for modders

## Summary

The MVE/MVL formats are the video cutscene system for Descent 1 & 2:

- **MVL**: Simple archive containing multiple MVE movies
- **MVE**: Chunk-based video format with proprietary compression
- **Reading**: Header validation + chunk/segment iteration
- **Playback**: Converted to MP4/H.264 during first-time setup
- **Integration**: Seamless playback via Bevy video plugins

The descent-core implementation provides:
- ✅ MVL archive reading and file extraction
- ✅ MVE header validation and structure parsing
- ✅ Chunk and segment iteration
- ✅ Conversion strategy (MVE→MP4 via FFmpeg during setup)
- ❌ Direct MVE video decompression (not needed - using conversion)
- ❌ Direct audio extraction (not needed - FFmpeg handles it)

**Conversion Strategy**: Rather than implementing a complex MVE decoder, D2X-RS converts videos to modern MP4/H.264 format during the **automated first-time setup**. This provides optimal performance (hardware acceleration), smaller file sizes, and seamless integration with Bevy's ecosystem - all while maintaining legal compliance by having users provide their own game files.

---

**Format Version**: Interplay MVE (1990s era)  
**Descent Usage**: D1 (1995), D2 (1996)  
**Implementation Status**: Parsing complete, decoding not implemented  
**Last Updated**: 2026-03-01

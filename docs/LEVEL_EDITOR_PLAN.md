# Descent Level Editor (DLE) - Rust/Qt Implementation Plan

## Overview

Modern reimplementation of the Descent Level Editor (DLE-XP) using Rust and Qt, based on analysis of the original C++ source code and UI screenshots.

## UI Layout Structure

### Main Window
```
┌─────────────────────────────────────────────────────────────────┐
│ Menu Bar: File Edit View Select Insert Delete Tools Help        │
├─────────────────────────────────────────────────────────────────┤
│ Toolbar: [Icon buttons for common operations]                   │
├──────────┬──────────────────────────────────────────┬───────────┤
│          │                                          │           │
│  Left    │         Central 3D Viewport              │   Right   │
│ Sidebar  │         (OpenGL/wgpu)                    │  Sidebar  │
│          │                                          │           │
│ - Texture│         [Level geometry view]            │ Tool-     │
│   Browser│                                          │ specific  │
│ - MiniMap│         [Selected segment highlighted]   │ Properties│
│ - View   │                                          │           │
│   Control│         [Objects, lighting, etc.]        │           │
│ - Tool   │                                          │           │
│   Panels │                                          │           │
│          │                                          │           │
├──────────┴──────────────────────────────────────────┴───────────┤
│ Bottom Texture Palette: [Scrollable texture grid]               │
├─────────────────────────────────────────────────────────────────┤
│ Status Bar: [Segment info] [Coordinates] [Mode]                 │
└─────────────────────────────────────────────────────────────────┘
```

## Architecture

### Technology Stack
- **UI Framework**: Qt 6 (via `qmetaobject` or `cxx-qt` crate)
- **3D Rendering**: `wgpu` (modern Vulkan/Metal/DX12 abstraction)
- **Level Data**: `descent-core` crate (existing parsers)
- **Language**: Rust 2021 edition

### Crate Structure
```
crates/
├── d2x-editor/          # Main level editor application
│   ├── src/
│   │   ├── main.rs      # Application entry point
│   │   ├── ui/          # Qt UI components
│   │   │   ├── main_window.rs
│   │   │   ├── texture_browser.rs
│   │   │   ├── minimap.rs
│   │   │   ├── viewport.rs
│   │   │   └── tool_panels/
│   │   │       ├── segment_tool.rs
│   │   │       ├── texture_tool.rs
│   │   │       ├── object_tool.rs
│   │   │       ├── trigger_tool.rs
│   │   │       ├── wall_tool.rs
│   │   │       └── light_tool.rs
│   │   ├── renderer/    # 3D rendering (wgpu)
│   │   │   ├── mod.rs
│   │   │   ├── level_renderer.rs
│   │   │   ├── selection_renderer.rs
│   │   │   ├── grid_renderer.rs
│   │   │   └── shaders/
│   │   ├── editor/      # Core editor logic
│   │   │   ├── mod.rs
│   │   │   ├── state.rs
│   │   │   ├── selection.rs
│   │   │   ├── undo.rs
│   │   │   └── operations/
│   │   │       ├── add_segment.rs
│   │   │       ├── delete_segment.rs
│   │   │       ├── split_segment.rs
│   │   │       ├── join_segments.rs
│   │   │       └── texture_apply.rs
│   │   └── camera.rs    # 3D camera controller
│   ├── Cargo.toml
│   └── resources/       # UI resources (icons, etc.)
```

## Core Features (Priority Order)

### Phase 1: Basic Structure (Week 1-2)
- [x] OOF parser complete (already done)
- [ ] Create `d2x-editor` crate
- [ ] Qt main window with menu bar
- [ ] Basic wgpu viewport integration
- [ ] Load and display level geometry
- [ ] Camera controls (pan, rotate, zoom)
- [ ] Simple wireframe rendering

### Phase 2: Selection & Basic Editing (Week 3-4)
- [ ] Segment selection (click to select)
- [ ] Selection visualization (yellow wireframe box)
- [ ] Segment info panel
- [ ] Add segment operation
- [ ] Delete segment operation
- [ ] Undo/redo system

### Phase 3: Texture System (Week 5-6)
- [ ] Texture browser widget
- [ ] Load textures from PIG files
- [ ] Bottom texture palette
- [ ] Texture application to faces
- [ ] Texture alignment tools
- [ ] UV coordinate display

### Phase 4: Object Placement (Week 7-8)
- [ ] Object tool panel
- [ ] Object placement (robots, powerups, etc.)
- [ ] Object property editor
- [ ] Object rendering in viewport
- [ ] Object selection and manipulation
- [ ] Player start placement

### Phase 5: Advanced Features (Week 9-12)
- [ ] Trigger editor
- [ ] Wall properties
- [ ] Lighting system
- [ ] Mini-map widget
- [ ] Split/join segments
- [ ] Tunnel maker
- [ ] Reactor editor
- [ ] Mission properties

### Phase 6: Polish & Testing (Week 13-14)
- [ ] Save level functionality
- [ ] Validation & diagnostics
- [ ] Keyboard shortcuts
- [ ] Settings/preferences
- [ ] Documentation
- [ ] User testing

## Technical Details

### Rendering Pipeline (wgpu)
1. **Geometry Pass**: Render level segments with textures
2. **Selection Pass**: Render selected objects with highlight
3. **Grid Pass**: Render alignment grid
4. **Object Pass**: Render robots, powerups as models/sprites
5. **UI Overlay**: Render coordinate axes, gizmos

### Data Structures

```rust
// Core editor state
pub struct EditorState {
    level: Level,                    // Current level data
    selection: Selection,            // Selected segments/faces/objects
    undo_stack: Vec<Operation>,      // Undo history
    redo_stack: Vec<Operation>,      // Redo history
    current_tool: Tool,              // Active tool
    camera: Camera,                  // 3D camera
    texture_manager: TextureManager, // Loaded textures
    object_manager: ObjectManager,   // Available objects
}

pub enum Selection {
    None,
    Segment(u16),
    Face { segment: u16, side: u8 },
    Object(usize),
    Multiple(Vec<SelectionItem>),
}

pub enum Tool {
    Segment,
    Texture,
    Object,
    Trigger,
    Wall,
    Light,
}

pub trait Operation {
    fn execute(&self, state: &mut EditorState) -> Result<()>;
    fn undo(&self, state: &mut EditorState) -> Result<()>;
}
```

### Qt Integration Options

#### Option 1: qmetaobject-rs
```rust
use qmetaobject::*;

#[derive(QObject)]
struct MainWindow {
    base: qt_base_class!(trait QMainWindow),
    // Qt properties
}
```

#### Option 2: cxx-qt
```rust
#[cxx_qt::bridge]
mod editor {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }
    
    #[cxx_qt::qobject]
    struct MainWindow {
        // Rust fields
    }
}
```

### Performance Considerations
- Use `wgpu` for GPU-accelerated rendering
- Implement level-of-detail (LOD) for large levels
- Frustum culling for off-screen geometry
- Texture atlasing for reduced draw calls
- Batch similar geometry for efficient rendering

## DLE-XP Feature Comparison

### Must-Have Features (from DLE-XP)
- ✓ 3D viewport with textured/wireframe modes
- ✓ Segment-based editing
- ✓ Texture browser and application
- ✓ Object placement (robots, powerups, etc.)
- ✓ Trigger editor
- ✓ Wall properties
- ✓ Undo/redo
- ✓ Mini-map
- ✓ Split/join segments

### Nice-to-Have Features
- Tunnel maker
- Lighting preview
- ASE model support (D2X-XL high-res)
- Script editor
- Multi-level mission editor
- Diagnostic tools

### Features to Skip Initially
- Built-in scripting (focus on standard D1/D2)
- Advanced D2X-XL features (effects, particles)
- Network multiplayer testing

## Dependencies

```toml
[dependencies]
descent-core = { path = "../descent-core" }
wgpu = "0.19"
winit = "0.29"           # Window creation
bytemuck = "1.14"        # Shader data
glam = "0.25"            # Math library
image = "0.25"           # Texture loading
anyhow = "1.0"           # Error handling
tracing = "0.1"          # Logging

# Qt bindings (choose one)
qmetaobject = "0.2"      # OR
cxx-qt = "0.6"

[dev-dependencies]
criterion = "0.5"        # Benchmarking
```

## File Organization

### Project Files
```
crates/d2x-editor/
├── Cargo.toml
├── build.rs             # Qt resource compilation
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── ui/              # Qt UI code
│   ├── renderer/        # wgpu rendering
│   ├── editor/          # Editor logic
│   └── shaders/         # WGSL shaders
├── resources/
│   ├── icons/           # Toolbar icons
│   ├── shaders/         # Shader source files
│   └── ui/              # Qt UI files (.ui)
└── tests/
    └── integration/
```

## Next Steps

1. **Decide on Qt binding**: qmetaobject vs cxx-qt
   - Research pros/cons of each
   - Create simple "Hello World" with both
   - Choose based on ergonomics and maintenance

2. **Set up basic project structure**
   - Create `d2x-editor` crate
   - Add basic dependencies
   - Create main window skeleton

3. **Implement minimal viewport**
   - Integrate wgpu for 3D rendering
   - Load a test level from descent-core
   - Render basic wireframe geometry
   - Add camera controls

4. **Iterate on features**
   - Add one feature at a time
   - Test with real D1/D2 levels
   - Refine based on usability

## References

- Original DLE-XP source: `/home/admin/Downloads/dle-src/`
- UI screenshots: `dle_1.jpg`, `dle_3.jpg`, `dle_5.jpg`, `dle_9.jpg`, `dle_12.jpg`
- Descent 1/2 data: `/run/media/admin/New Volume/d2x-xl-win/data/`
- descent-core parsers: `/home/admin/Documents/GitHub/d2x-rs/crates/descent-core/`

## Timeline Estimate

- **Minimal viable editor**: 4-6 weeks (basic viewing + segment editing)
- **Feature-complete**: 12-14 weeks (all major tools)
- **Polished release**: 16-18 weeks (with testing and docs)

*Assumes 20-30 hours/week development time*

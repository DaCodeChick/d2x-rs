# DLE - Descent Level Editor Architecture

## Overview

A modern Qt 6 + C++ rewrite of the Descent Level Editor (DLE-XP), providing a cross-platform tool for creating and editing Descent levels.

## Technology Stack

- **UI Framework**: Qt 6.10+ (Widgets + QML)
- **Language**: C++23
- **Graphics**: Qt 3D / OpenGL 4.5+
- **Build System**: CMake
- **Version Control**: Git

## Core Architecture

### 1. Model-View Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Presentation Layer                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ MainWindow   │  │  ToolPalette │  │ TextureView  │  │
│  │  (Qt)        │  │    (Qt)      │  │    (Qt)      │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ EditorCore   │  │  ToolManager │  │ ViewManager  │  │
│  │              │  │              │  │              │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                      Domain Layer                        │
│  ┌─────────┬─────────┬─────────┬─────────┬──────────┐  │
│  │Segment  │Texture  │ Object  │ Light   │ Trigger  │  │
│  │Manager  │Manager  │Manager  │Manager  │ Manager  │  │
│  └─────────┴─────────┴─────────┴─────────┴──────────┘  │
│  ┌─────────┬─────────┬─────────┬─────────┬──────────┐  │
│  │  Wall   │ Vertex  │ Robot   │  Undo   │   HOG    │  │
│  │Manager  │Manager  │Manager  │Manager  │ Manager  │  │
│  └─────────┴─────────┴─────────┴─────────┴──────────┘  │
└─────────────────────────────────────────────────────────┘
                           │
┌─────────────────────────────────────────────────────────┐
│                      Data Layer                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ Mine Model   │  │  File I/O    │  │  Resources   │  │
│  │  (Core)      │  │  (RDL/RL2)   │  │  (HOG/PIG)   │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└─────────────────────────────────────────────────────────┘
```

## Module Breakdown

### Core Modules

#### 1. **Mine Model** (`core/mine/`)
- `Mine` - Main level data container
- `Segment` - Cube/room geometry (vertices, sides, children)
- `Side` - Wall surface with textures, UVs
- `Vertex` - 3D point in space
- `Edge` - Connection between vertices

**Key Features:**
- Level version management (D1, D2, D2X-XL)
- Geometry validation
- Coordinate system transformations

#### 2. **Managers** (`core/managers/`)

**SegmentManager**
- Add/delete segments
- Join segments (remove walls)
- Split segments (add walls)
- Mark/select segments
- Validate connectivity

**VertexManager**
- Vertex pooling (shared vertices)
- Position updates
- Welding/merging
- Snapping to grid

**TextureManager**
- Load textures from PIG/HOG
- Texture atlas management
- UV mapping
- Animated textures
- Custom texture support

**ObjectManager**
- Place objects (robots, powerups, weapons)
- Object properties
- Spawn points (player, coop, matcen)
- AI settings

**LightManager**
- Static lighting
- Dynamic lights
- Light delta calculation
- Illumination preview

**TriggerManager**
- Door triggers
- Wall triggers
- Object triggers
- Teleporters
- Secret exits

**WallManager**
- Wall types (door, illusory, forcefield)
- Wall keys (blue, gold, red)
- Destructible walls
- Wall flags

**RobotManager**
- Robot type definitions
- AI behavior settings
- Combat properties
- Spawn counts

**UndoManager**
- Command pattern implementation
- Undo/redo stack
- Memento snapshots
- Smart grouping

**HogManager**
- Read HOG/HOG2 archives
- Extract assets
- Create new HOGs

**PaletteManager**
- Load D1/D2 palettes
- Color conversion
- Lighting tables

#### 3. **File I/O** (`core/io/`)

**Formats:**
- RDL (Descent 1)
- RL2 (Descent 2)
- D2X-XL extensions
- POG (texture list)
- HXM (extra data)

**Classes:**
- `LevelReader` - Load levels
- `LevelWriter` - Save levels
- `HogReader` - Read HOG archives
- `PigReader` - Read PIG textures
- `HamReader` - Read HAM game data

#### 4. **Rendering** (`render/`)

**OpenGL Renderer**
- Segment rendering (wireframe, textured, lit)
- Object rendering (models, sprites)
- Grid overlay
- Selection highlighting
- Camera system (perspective, orthographic)
- View modes (3D, front, top, side)

**Qt 3D Integration**
- Scene graph
- Materials
- Shaders
- Picking

### UI Modules

#### 1. **Main Window** (`ui/mainwindow/`)
- Menu bar
- Tool bars
- Status bar
- Dock widgets
- MDI/SDI views

#### 2. **Tool Palette** (`ui/tools/`)
- Segment tools (add, delete, join, split)
- Texture tools (apply, align, rotate)
- Object tools (place, move, properties)
- Light tools (add, adjust, preview)
- Trigger tools (create, edit, test)
- Wall tools (add, remove, configure)

#### 3. **Property Editors** (`ui/properties/`)
- Segment properties
- Side properties (textures, UVs, lighting)
- Object properties (type, position, orientation)
- Trigger properties (type, targets, flags)
- Wall properties (type, keys, flags)
- Level properties (name, version, settings)

#### 4. **Views** (`ui/views/`)

**3D View** (Qt 3D Widget)
- Free camera navigation
- Selection (click, box, lasso)
- Gizmos (move, rotate, scale)
- Realtime rendering

**Texture View**
- Texture browser
- Thumbnail grid
- Search/filter
- Drag-and-drop

**Mine View** (Orthographic)
- Top/Front/Side projections
- Grid snapping
- Measurement tools

**Object Browser**
- Tree view of all objects
- Filter by type
- Quick navigation

#### 5. **Dialogs** (`ui/dialogs/`)
- New level wizard
- Level settings
- Import/export
- Preferences
- About

## Data Structures

### Core Types (from original DLE)

```cpp
// Fixed-point math (original Descent format)
typedef int32_t fix;  // 16.16 fixed point

inline double X2D(fix x) { return double(x) / 65536.0; }
inline fix D2X(double d) { return fix(d * 65536.0); }

// 3D Vector
struct CVector {
    fix x, y, z;
};

struct CDoubleVector {
    double x, y, z;
};

// Matrix (orientation)
struct CMatrix {
    CVector r, u, f;  // right, up, forward
};

// UV Coordinates
struct CUVCoords {
    fix u, v;
};

// Segment (cube room)
struct CSegment {
    int16_t children[6];     // Adjacent segments (-1 = wall)
    int16_t vertices[8];     // Vertex indices
    uint8_t special;         // Special type (matcen, etc)
    uint8_t matcen;          // Matcen number
    int16_t value;           // Damage/speed
    fix     staticLight;     // Base light level
    CSide   sides[6];        // Wall data
};

// Side (wall face)
struct CSide {
    uint8_t  type;           // Wall type
    uint8_t  num;            // Side number
    int16_t  wallNum;        // Wall index
    uint16_t textures[2];    // Texture IDs (base, overlay)
    CUVCoords uvls[4];       // UV + light per vertex
    uint32_t flags;          // Rendering flags
};

// Object
struct CObject {
    uint8_t  type;           // Robot, powerup, player, etc
    uint8_t  id;             // Subtype
    CVector  position;       // Location
    CMatrix  orientation;    // Rotation
    fix      size;           // Radius
    int16_t  segment;        // Current segment
};

// Wall
struct CWall {
    int16_t  segmentNum;     // Segment index
    int16_t  sideNum;        // Side index
    uint8_t  type;           // Door, illusory, etc
    uint8_t  flags;          // Keys required
    fix      hps;            // Hit points
    uint8_t  linkedWall;     // Opposite wall
    int16_t  triggerNum;     // Trigger index
};

// Trigger
struct CTrigger {
    uint8_t  type;           // Door, matcen, exit, etc
    uint8_t  flags;          // Control flags
    fix      value;          // Time delay
    int16_t  numLinks;       // Number of targets
    int16_t  segments[10];   // Target segments
    int16_t  sides[10];      // Target sides
};
```

## Qt Integration

### Signal/Slot Architecture

```cpp
// Example: SegmentManager emits signals on changes
class SegmentManager : public QObject {
    Q_OBJECT
public:
    void addSegment(const CSegment& seg);
    void deleteSegment(int index);
    void joinSegments(int seg1, int seg2);

signals:
    void segmentAdded(int index);
    void segmentDeleted(int index);
    void segmentModified(int index);
    void geometryChanged();
};

// MainWindow connects to update UI
connect(segmentMgr, &SegmentManager::geometryChanged,
        view3D, &View3D::refresh);
```

### Model/View Pattern

```cpp
// Qt model for object list
class ObjectListModel : public QAbstractListModel {
    Q_OBJECT
public:
    int rowCount(const QModelIndex& parent) const override;
    QVariant data(const QModelIndex& index, int role) const override;
    
private:
    ObjectManager* m_objectManager;
};

// Use in QListView, QTreeView, etc.
ObjectListModel* model = new ObjectListModel(objectManager);
listView->setModel(model);
```

## File Format Support

### Level Files
- **RDL** (Descent 1) - Read/Write
- **RL2** (Descent 2) - Read/Write
- **D2X-XL** (Extended) - Read/Write
- **POG** (Texture assignments) - Read/Write

### Asset Files
- **HOG/HOG2** (Archives) - Read
- **PIG** (Textures) - Read
- **HAM** (Game data) - Read
- **HXM** (Robot data) - Read
- **POF** (Models) - Read

### Export Formats
- **OBJ** (3D model export)
- **glTF** (Modern 3D format)
- **JSON** (Level metadata)

## Build System

### CMakeLists.txt Structure

```cmake
cmake_minimum_required(VERSION 3.20)
project(dle VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 REQUIRED COMPONENTS
    Core Widgets Gui OpenGL 3DCore 3DRender 3DInput 3DExtras
)

add_subdirectory(src/core)
add_subdirectory(src/render)
add_subdirectory(src/ui)

add_executable(dle
    src/main.cpp
)

target_link_libraries(dle PRIVATE
    Qt6::Core Qt6::Widgets Qt6::Gui Qt6::OpenGL
    Qt6::3DCore Qt6::3DRender Qt6::3DInput Qt6::3DExtras
    dle_core dle_render dle_ui
)
```

## Development Phases

### Phase 1: Foundation (Weeks 1-2)
- [ ] Project setup (CMake, Qt)
- [ ] Core data structures (Mine, Segment, Vertex)
- [ ] Basic file I/O (read RDL/RL2)
- [ ] Main window skeleton

### Phase 2: Geometry (Weeks 3-4)
- [ ] SegmentManager implementation
- [ ] VertexManager implementation
- [ ] 3D OpenGL renderer
- [ ] Basic camera controls
- [ ] Selection system

### Phase 3: Textures (Weeks 5-6)
- [ ] TextureManager (PIG/HOG loading)
- [ ] Texture view UI
- [ ] Apply textures to sides
- [ ] UV editor
- [ ] Texture alignment tools

### Phase 4: Objects (Weeks 7-8)
- [ ] ObjectManager
- [ ] Object placement
- [ ] Robot/powerup browser
- [ ] Object property editor
- [ ] Model rendering (POF)

### Phase 5: Advanced Features (Weeks 9-12)
- [ ] LightManager (static lighting)
- [ ] TriggerManager
- [ ] WallManager
- [ ] UndoManager
- [ ] Level validation
- [ ] Testing tools

### Phase 6: Polish (Weeks 13-14)
- [ ] Preferences system
- [ ] Keyboard shortcuts
- [ ] Tooltips/help
- [ ] User manual
- [ ] Bug fixes
- [ ] Performance optimization

## References

- Original DLE-XP source: `/tmp/dle-src/`
- Descent file formats: `docs/file-formats/`
- Qt 6 documentation: https://doc.qt.io/qt-6/
- OpenGL 4.5: https://www.khronos.org/opengl/

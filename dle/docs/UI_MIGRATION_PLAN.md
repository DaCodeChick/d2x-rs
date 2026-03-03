# DLE UI Migration Plan: MFC to Qt6

## Overview
This document outlines the plan to replicate the original MFC-based DLE user interface in Qt6, following modern C++23 practices while maintaining the familiar layout and workflow.

## Original DLE Layout Structure

### MFC Implementation
The original DLE uses:
- **CSplitterWnd** for fixed 3-pane layout
- **CPropertySheet** with **CPropertyPage** for tool tabs
- **3 Layout Modes**:
  - Mode 0: Texture left | (Mine top / Tools bottom)
  - Mode 1: (Tools top / Texture bottom) left | Mine right
  - Mode 2: Texture left | Mine right (no tool pane)

### Qt6 Approach
Current Qt6 DLE uses:
- **QDockWidget** for flexible panel docking
- **Central widget** for 3D OpenGL viewport
- More flexible, modern approach (can dock/float/resize)

## Migration Strategy

### Phase 1: Core Layout ✅ COMPLETE
- [x] MainWindow with QMainWindow
- [x] Central widget with LevelViewport (OpenGL)
- [x] QDockWidget for Texture Palette (left)
- [x] QDockWidget for Texture Bar (bottom)
- [x] QDockWidget for Properties (right, hidden by default)
- [x] QDockWidget for Segment Info (right, hidden by default)
- [x] Menu bar structure
- [x] Toolbar with common actions
- [x] Status bar

### Phase 2: Tool Panel Structure (CURRENT)
Create tabbed tool panel with essential editing tools:

#### 2.1 Tool Panel Container
- **QDockWidget** named "Tools" (left side by default)
- **QTabWidget** for tool categories
- Tabs: Segment, Wall, Trigger, Object, Texture, Diagnostics

#### 2.2 Segment Tool Tab
Original: `CSegmentTool` (toolview.h:1085-1201)
- Segment selector (ID, function)
- Side/point selector (6 sides, 4 points each)
- Properties (water, lava, volatile, etc.)
- Actions: Add, Delete, Split, Other Segment
- Damage settings
- Light level control
- Vertex coordinates

#### 2.3 Wall Tool Tab
Original: `CWallTool` (toolview.h:1205-1304)
- Wall type selector
- Clip number
- Keys (None, Blue, Gold, Red)
- Flags (Blasted, Door Open, Door Locked, etc.)
- Strength/Cloak values
- Texture preview
- Actions: Add, Delete

#### 2.4 Trigger Tool Tab
Original: `CTriggerTool` (toolview.h:1308-1430)
- Trigger type selector
- Target list
- D1/D2 flags
- Time/strength settings
- Texture change selectors
- Actions: Add, Delete, Add Target

#### 2.5 Object Tool Tab
Original: `CObjectTool` (toolview.h:775-843)
- Object type/ID selectors
- AI settings
- Spawn settings
- Object preview
- Actions: Add, Delete, Move, Reset

#### 2.6 Texture Tool Tab
Original: Multiple texture tools
- Align tool (rotation, offset, stretch)
- Light settings
- Animation settings

#### 2.7 Diagnostics Tab
Original: `CDiagTool` (toolview.h:411-482)
- Level statistics
- Bug list
- Check mine function
- Object count
- Texture count

### Phase 3: Texture Browser (LEFT DOCK)
Replicate `CTextureView` (TextureView.h:21-76):
- Scrollable texture grid
- Texture selection
- Filter by type/category
- Current/ovl texture display
- Mouse selection

### Phase 4: Additional Panels
- Properties panel (context-sensitive)
- Segment info panel
- Advanced object properties
- Effect tools (particles, lightning, sound)

### Phase 5: Edit Tool Bar
Original: `CEditTool` (MainFrame.h:27-66)
- 10 geometry edit buttons:
  - Forward, Up, Back, Left, Grow, Right, Rotate Left, Down, Rotate Right, Shrink
- Modal editing operations
- Timer-based continuous editing

### Phase 6: Status Bar Enhancement
Original: 5 panes (MainFrame.cpp status bar)
- Status messages
- Info messages
- Insert mode indicator
- Selection mode indicator
- Progress bar

## MFC → Qt Widget Mappings

| MFC Class | Qt6 Class | Notes |
|-----------|-----------|-------|
| `CPropertySheet` | `QTabWidget` | Tool panel container |
| `CPropertyPage` | `QWidget` | Individual tool tabs |
| `CToolDlg` | `QWidget` | Base class for tool panels |
| `CComboBox` | `QComboBox` | Dropdown selections |
| `CListBox` | `QListWidget` | Lists (targets, objects) |
| `CSliderCtrl` | `QSlider` | Sliders (transparency, etc.) |
| `CSpinButtonCtrl` | `QSpinBox` | Numeric spinners |
| `CEdit` | `QLineEdit` | Text entry |
| `CButton` | `QPushButton` / `QCheckBox` | Buttons/checkboxes |
| `CBitmapButton` | `QPushButton` with `QIcon` | Icon buttons |
| `CExtSliderCtrl` | Custom Qt widget | Slider + spinner + label |
| `CSplitterWnd` | `QSplitter` | Pane dividers |

## Implementation Guidelines

### C++23 Standards (AGENTS.md)
- Use `#pragma once` for all headers ✅
- Use `std::expected<T, ErrorType>` for error handling
- Use `<format>` and `<print>` for output
- NO exceptions, NO raw pointers
- Modern STL containers, smart pointers, ranges

### Qt6 Patterns
- Use `.ui` files for complex layouts
- Use Qt Designer where practical
- Connect signals/slots in code
- Use Qt's MOC system
- Use Qt property system for bindings

### Code Organization
```
dle/src/ui/
├── mainwindow/
│   ├── MainWindow.h/cpp/ui     ✅
├── toolpanel/
│   ├── ToolPanel.h/cpp/ui      [TODO]
│   ├── SegmentTool.h/cpp/ui    [TODO]
│   ├── WallTool.h/cpp/ui       [TODO]
│   ├── TriggerTool.h/cpp/ui    [TODO]
│   ├── ObjectTool.h/cpp/ui     [TODO]
│   ├── TextureTool.h/cpp/ui    [TODO]
│   └── DiagnosticsTool.h/cpp/ui [TODO]
├── texturebrowser/
│   ├── TextureBrowser.h/cpp/ui [TODO]
│   └── TextureGrid.h/cpp       [TODO]
├── edittoolbar/
│   └── EditToolBar.h/cpp/ui    [TODO]
└── widgets/
    ├── SliderSpinner.h/cpp     [TODO]
    └── ColorControl.h/cpp      [TODO]
```

## Priority Order

1. **Phase 2.1-2.2**: Tool Panel + Segment Tool (NEXT)
2. **Phase 2.3**: Wall Tool
3. **Phase 3**: Texture Browser
4. **Phase 2.4-2.7**: Remaining tool tabs
5. **Phase 5**: Edit Toolbar
6. **Phase 4**: Advanced panels
7. **Phase 6**: Status bar enhancement

## Testing Strategy

- Build after each major component
- Visual comparison with original DLE screenshots
- Test widget interactions
- Test data binding to Mine structure
- Test layout persistence

## Current Status

- **Completed**: Phase 1 (Core Layout) ✅
- **In Progress**: Phase 2.1 (Tool Panel Structure)
- **Next**: Phase 2.2 (Segment Tool Tab)

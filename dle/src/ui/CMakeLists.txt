cmake_minimum_required(VERSION 3.20)

# UI library (Qt widgets and UI components)
add_library(dle_ui STATIC
    mainwindow/MainWindow.h
    mainwindow/MainWindow.cpp
    mainwindow/MainWindow.ui
    toolpanel/ToolPanel.h
    toolpanel/ToolPanel.cpp
    toolpanel/SegmentTool.h
    toolpanel/SegmentTool.cpp
    toolpanel/SegmentTool.ui
    toolpanel/WallTool.h
    toolpanel/WallTool.cpp
    toolpanel/WallTool.ui
    toolpanel/TriggerTool.h
    toolpanel/TriggerTool.cpp
    toolpanel/TriggerTool.ui
    toolpanel/ObjectTool.h
    toolpanel/ObjectTool.cpp
    toolpanel/ObjectTool.ui
)

target_include_directories(dle_ui PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/..
)

target_link_libraries(dle_ui PUBLIC
    Qt6::Core
    Qt6::Widgets
    Qt6::Gui
    dle_core
    dle_render
)

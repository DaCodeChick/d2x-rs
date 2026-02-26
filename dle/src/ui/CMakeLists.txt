cmake_minimum_required(VERSION 3.20)

# UI library (Qt widgets and UI components)
add_library(dle_ui STATIC
    # Placeholder - will add UI code later
    # mainwindow/MainWindow.h
    # mainwindow/MainWindow.cpp
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

# Add a placeholder source file to avoid empty library error
file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/ui_placeholder.cpp "// Placeholder\n")
target_sources(dle_ui PRIVATE ${CMAKE_CURRENT_BINARY_DIR}/ui_placeholder.cpp)

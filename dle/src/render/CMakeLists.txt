cmake_minimum_required(VERSION 3.20)

# Render library (Qt RHI rendering)
add_library(dle_render STATIC
    LevelViewport.h
    LevelViewport.cpp
    MineRenderer.h
    MineRenderer.cpp
)

target_include_directories(dle_render PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/..
)

target_link_libraries(dle_render PUBLIC
    Qt6::Core
    Qt6::Widgets
    Qt6::Gui
    Qt6::GuiPrivate
    dle_core
)

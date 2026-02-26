cmake_minimum_required(VERSION 3.20)

# Render library (OpenGL rendering)
add_library(dle_render STATIC
    # Placeholder - will add rendering code later
    # Renderer.h
    # Renderer.cpp
)

target_include_directories(dle_render PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/..
)

target_link_libraries(dle_render PUBLIC
    Qt6::Core
    Qt6::OpenGL
    Qt6::OpenGLWidgets
    dle_core
)

# Add a placeholder source file to avoid empty library error
file(WRITE ${CMAKE_CURRENT_BINARY_DIR}/render_placeholder.cpp "// Placeholder\n")
target_sources(dle_render PRIVATE ${CMAKE_CURRENT_BINARY_DIR}/render_placeholder.cpp)

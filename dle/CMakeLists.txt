cmake_minimum_required(VERSION 3.20)
project(dle VERSION 1.0.0 LANGUAGES CXX)

# C++ Standard
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Qt Configuration
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

# Find Qt6
find_package(Qt6 REQUIRED COMPONENTS
    Core
    Widgets
    Gui
    OpenGL
    OpenGLWidgets
)

# Compiler warnings
if(MSVC)
    add_compile_options(/W4)
else()
    add_compile_options(-Wall -Wextra -Wpedantic)
endif()

# Include directories
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/src)

# Subdirectories
add_subdirectory(src/core)
add_subdirectory(src/render)
add_subdirectory(src/ui)

# Main executable
add_executable(dle
    src/main.cpp
)

target_link_libraries(dle PRIVATE
    Qt6::Core
    Qt6::Widgets
    Qt6::Gui
    Qt6::OpenGL
    Qt6::OpenGLWidgets
    dle_core
    dle_render
    dle_ui
)

# Install
install(TARGETS dle
    RUNTIME DESTINATION bin
)

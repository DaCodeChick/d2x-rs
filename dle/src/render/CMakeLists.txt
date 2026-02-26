cmake_minimum_required(VERSION 3.20)

# Find qsb tool for shader compilation
find_program(QSB_TOOL qsb HINTS ${Qt6_DIR}/../../../bin /usr/lib/qt6/bin)
if(NOT QSB_TOOL)
    message(WARNING "qsb tool not found - shaders will not be automatically recompiled when changed")
endif()

# Shader compilation function
function(compile_shader SHADER_SOURCE)
    get_filename_component(SHADER_NAME ${SHADER_SOURCE} NAME)
    set(SHADER_OUTPUT ${CMAKE_CURRENT_SOURCE_DIR}/shaders/${SHADER_NAME}.qsb)
    set(SHADER_INPUT ${CMAKE_CURRENT_SOURCE_DIR}/shaders/${SHADER_SOURCE})
    
    if(QSB_TOOL)
        add_custom_command(
            OUTPUT ${SHADER_OUTPUT}
            COMMAND ${QSB_TOOL} --glsl 100es,120,150 --hlsl 50 --msl 12 -o ${SHADER_OUTPUT} ${SHADER_INPUT}
            DEPENDS ${SHADER_INPUT}
            COMMENT "Compiling shader: ${SHADER_NAME}"
            VERBATIM
        )
    endif()
endfunction()

# Compile shaders
compile_shader(basic.vert)
compile_shader(basic.frag)

# Render library (Qt RHI rendering)
add_library(dle_render STATIC
    LevelViewport.h
    LevelViewport.cpp
    MineRenderer.h
    MineRenderer.cpp
    shaders.qrc
    # Shader dependencies - triggers recompilation when GLSL sources change
    ${CMAKE_CURRENT_SOURCE_DIR}/shaders/basic.vert.qsb
    ${CMAKE_CURRENT_SOURCE_DIR}/shaders/basic.frag.qsb
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

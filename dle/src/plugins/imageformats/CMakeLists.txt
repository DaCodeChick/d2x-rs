cmake_minimum_required(VERSION 3.20)

# PCX Image Format Plugin
add_library(dle_pcx_plugin MODULE
    PcxHandler.h
    PcxHandler.cpp
    PcxPlugin.h
    PcxPlugin.cpp
    pcx.json
)

target_link_libraries(dle_pcx_plugin PRIVATE
    Qt6::Core
    Qt6::Gui
)

# Set plugin properties
set_target_properties(dle_pcx_plugin PROPERTIES
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/plugins/imageformats"
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/plugins/imageformats"
)

# Copy the JSON metadata file to the build directory
add_custom_command(TARGET dle_pcx_plugin POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CMAKE_CURRENT_SOURCE_DIR}/pcx.json
        $<TARGET_FILE_DIR:dle_pcx_plugin>/pcx.json
)

# Install plugin
install(TARGETS dle_pcx_plugin
    LIBRARY DESTINATION lib/dle/plugins/imageformats
    RUNTIME DESTINATION lib/dle/plugins/imageformats
)

install(FILES pcx.json
    DESTINATION lib/dle/plugins/imageformats
)

# IFF/ILBM Image Format Plugin
add_library(dle_ilbm_plugin MODULE
    IlbmHandler.h
    IlbmHandler.cpp
    IlbmPlugin.h
    IlbmPlugin.cpp
    ilbm.json
)

target_link_libraries(dle_ilbm_plugin PRIVATE
    Qt6::Core
    Qt6::Gui
)

# Set plugin properties
set_target_properties(dle_ilbm_plugin PROPERTIES
    LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/plugins/imageformats"
    RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/plugins/imageformats"
    CXX_STANDARD 23
    CXX_STANDARD_REQUIRED ON
)

# Copy the JSON metadata file to the build directory
add_custom_command(TARGET dle_ilbm_plugin POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CMAKE_CURRENT_SOURCE_DIR}/ilbm.json
        $<TARGET_FILE_DIR:dle_ilbm_plugin>/ilbm.json
)

# Install plugin
install(TARGETS dle_ilbm_plugin
    LIBRARY DESTINATION lib/dle/plugins/imageformats
    RUNTIME DESTINATION lib/dle/plugins/imageformats
)

install(FILES ilbm.json
    DESTINATION lib/dle/plugins/imageformats
)

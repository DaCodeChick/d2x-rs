cmake_minimum_required(VERSION 3.20)

# Core library
add_library(dle_core STATIC
    # Types
    types/Types.h
    
    # Mine data structures
    mine/Mine.h
    mine/Mine.cpp
    mine/Segment.h
    mine/Segment.cpp
    mine/Side.h
    mine/Side.cpp
    mine/Wall.h
    mine/Wall.cpp
    mine/Trigger.h
    mine/Trigger.cpp
    mine/Object.h
    mine/Object.cpp
    
    # File I/O
    io/FileReader.h
    io/FileReader.cpp
    io/FileWriter.h
    io/FileWriter.cpp
)

target_include_directories(dle_core PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/..
)

target_link_libraries(dle_core PUBLIC
    Qt6::Core
)

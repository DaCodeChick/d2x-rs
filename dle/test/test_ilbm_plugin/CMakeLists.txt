cmake_minimum_required(VERSION 3.20)
project(test_ilbm_plugin)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui)

add_executable(test_ilbm_plugin test_ilbm_plugin.cpp)
target_link_libraries(test_ilbm_plugin PRIVATE Qt6::Core Qt6::Gui)

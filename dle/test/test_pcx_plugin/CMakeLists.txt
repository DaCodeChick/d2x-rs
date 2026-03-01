cmake_minimum_required(VERSION 3.20)
project(test_pcx_plugin)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui)

add_executable(test_pcx_plugin test_pcx_plugin.cpp)
target_link_libraries(test_pcx_plugin PRIVATE Qt6::Core Qt6::Gui)

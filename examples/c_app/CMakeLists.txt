cmake_minimum_required(VERSION 3.16)
project(oxide_c_app C)

set(CMAKE_C_STANDARD 99)
set(CMAKE_C_STANDARD_REQUIRED ON)

set(OXIDE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../..")
set(OXIDE_INCLUDE_DIR "${OXIDE_ROOT}/include")

include_directories(${OXIDE_INCLUDE_DIR})

add_executable(c_app main.c)

# Linking against the Oxide dynamic library produced by `cargo build --release`
if(MSVC)
    target_link_libraries(c_app PRIVATE "${OXIDE_ROOT}/target/release/oxide.dll.lib")
elseif(WIN32)
    target_link_libraries(c_app PRIVATE "${OXIDE_ROOT}/target/release/oxide.dll")
elseif(APPLE)
    target_link_libraries(c_app PRIVATE "${OXIDE_ROOT}/target/release/liboxide.dylib")
else()
    target_link_libraries(c_app PRIVATE "${OXIDE_ROOT}/target/release/liboxide.so" -lm -ldl -lpthread)
endif()

if(WIN32)
    add_custom_command(TARGET c_app POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "${OXIDE_ROOT}/target/release/oxide.dll"
        $<TARGET_FILE_DIR:c_app>
    )
endif()

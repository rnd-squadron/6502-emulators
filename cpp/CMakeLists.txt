cmake_minimum_required(VERSION 3.14)

project(NesEmulator CXX)

# ==== CONFIGURATION ==== #

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

option(ENABLE_TESTS "Enables tests to be built and show up in the IDE" ON)


# ==== BUILDING TARGET ==== #

set(HEADERS
)

set(CLASS_SOURCES
)

list(TRANSFORM HEADERS PREPEND ${CMAKE_CURRENT_LIST_DIR}/src/)
list(TRANSFORM CLASS_SOURCES PREPEND ${CMAKE_CURRENT_LIST_DIR}/src/)

add_executable(${PROJECT_NAME} src/main.cpp ${HEADERS} ${CLASS_SOURCES})


# ==== TEST SUBMODULE ==== #

if(ENABLE_TESTS)
    enable_testing()
    add_subdirectory(tests)
endif()


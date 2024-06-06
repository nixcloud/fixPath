# orig

# mkdir build; cd build
#  cmake -G "Ninja" -D_CMAKE_TOOLCHAIN_PREFIX=llvm- ..
#   or
#  cmake -G "Ninja" -D_CMAKE_TOOLCHAIN_PREFIX=llvm- .. && ninja --verbose && ./test_mylib.exe && objdump.exe -h .\test_mylib.exe | grep fix

cmake_minimum_required(VERSION 3.10)
project(MyLibrary)

# Specify the version of the library
set(MYLIB_VERSION_MAJOR 1)
set(MYLIB_VERSION_MINOR 0)
set(DELAYEDLIB_VERSION_MAJOR 1)
set(DELAYEDLIB_VERSION_MINOR 0)

set(VERBOSE 1)

set (CMAKE_CPP_FLAGS "-Xlinker /useFixPath -Xlinker /fixPathSize:333")
set (CMAKE_C_FLAGS "-Xlinker /useFixPath -Xlinker /fixPathSize:333")

# Add include directory
include_directories(${PROJECT_SOURCE_DIR}/include)

# Add the shared libraries
add_library(mylib SHARED src/mylib.c)
add_library(delayedlib SHARED src/delayedlib.c)

# Set the version information for the libraries
set_target_properties(mylib PROPERTIES VERSION ${MYLIB_VERSION_MAJOR}.${MYLIB_VERSION_MINOR})
set_target_properties(delayedlib PROPERTIES VERSION ${DELAYEDLIB_VERSION_MAJOR}.${DELAYEDLIB_VERSION_MINOR})

# Define MYLIB_EXPORTS and DELAYEDLIB_EXPORTS symbols to export the functions
target_compile_definitions(mylib PRIVATE MYLIB_EXPORTS)
target_compile_definitions(delayedlib PRIVATE DELAYEDLIB_EXPORTS)

# Add an executable to test the libraries
add_executable(test_mylib src/main.c)

# Link the shared library to the executable
target_link_libraries(test_mylib mylib)

# Enable delay loading for delayedlib on Windows
if (WIN32)
    target_link_libraries(test_mylib delayedlib delayimp)
    set_property(TARGET test_mylib PROPERTY LINK_FLAGS "-Xlinker /DELAYLOAD:delayedlib.dll")
endif()
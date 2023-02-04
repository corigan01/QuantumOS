

if(CMAKE_CXX_COMPILER_LOADED)
    message(STATUS "Compiler path: ${CMAKE_CXX_COMPILER}")
    message(STATUS "Compiler ID: ${CMAKE_CXX_COMPILER_ID}")
    message(STATUS "Compiler version: ${CMAKE_CXX_COMPILER_VERSION}")
    message(STATUS "Compiler is part of GCC: ${CMAKE_COMPILER_IS_GNUCXX}")
endif()


if (CMAKE_CXX_COMPILER_ID STREQUAL "Clang")
    add_subdirectory(stage-1)

else()
    message(SEND_ERROR "Quantum does not support this compiler ${CMAKE_C_COMPILER}\nPlease use clang!
        &> export CC=clang
        &> export CXX=clang
    ")

endif()





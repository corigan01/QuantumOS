cmake_minimum_required(VERSION 3.10)

set(CMAKE_C_COMPILER clang)
set(CMAKE_CXX_COMPILER clang++)

set(CMAKE_SYSTEM_NAME Generic)
set(CMAKE_SYSTEM_PROCESSOR i386)

# C support
set(triple i386)
set(CMAKE_C_COMPILER_TARGET elf32-i386)
set(CMAKE_CXX_COMPILER_TARGET ${triple})

# NASM support
set(CMAKE_NASM_LINK_EXECUTABLE "ld   -o  ")
set(CMAKE_ASM_NASM_FLAGS "")
set(CMAKE_ASM_NASM_OBJECT_FORMAT "")
set(CMAKE_ASM_NASM_COMPILE_OBJECT "<CMAKE_ASM_NASM_COMPILER> <INCLUDES> <FLAGS> ${CMAKE_ASM_NASM_OBJECT_FORMAT} -o <OBJECT> <SOURCE>")

project(QuantumBiosBoot LANGUAGES CXX C ASM_NASM ASM)


set( BOOT_NASM
        src/asm/mbr.asm
        )

set( NASM
        src/asm/runc.asm
        )

set( C
        src/c/start.c
        )


set_source_files_properties(${NASM} PROPERTIES LANGUAGE ASM_NASM)
set_source_files_properties(${NASM} PROPERTIES COMPILE_FLAGS "-f elf")
set_source_files_properties(${BOOT_NASM} PROPERTIES LANGUAGE ASM_NASM)
set_source_files_properties(${BOOT_NASM} PROPERTIES COMPILE_FLAGS "-f bin")

set(CMAKE_C_COMPILE_FEATURES
        -g -m32 -nostdinc -nostdlib -c
        -fPIE  -Wall -Werror -Wextra -O2
        -fno-builtin)

add_link_options(
        -T ../linker.ld -m32
        -g -Wall -Werror -Wextra -fPIE -nodefaultlibs
        -fno-stack-protector -fno-exceptions -nostartfiles
        -Wno-undef -nostdlib -ffreestanding -fno-rtti )


add_executable(quantumbiosboot.bin ${NASM} ${C})
add_library(mbr OBJECT ${BOOT_NASM})

target_include_directories(quantumbiosboot.bin PUBLIC src/c)
target_include_directories(mbr PUBLIC src/asm)

target_compile_options(quantumbiosboot.bin PRIVATE $<$<COMPILE_LANGUAGE:C>:${CMAKE_C_COMPILE_FEATURES}>)
target_compile_options(quantumbiosboot.bin PRIVATE $<$<COMPILE_LANGUAGE:CXX>:${CMAKE_CXX_COMPILE_FEATURES}>)

add_library(libstage-1 STATIC IMPORTED GLOBAL)
add_dependencies(libstage-1 rust)
set_target_properties(libstage-1  PROPERTIES IMPORTED_LOCATION libstage_1.o)
target_link_libraries(quantumbiosboot.bin PUBLIC libstage-1)

add_custom_target(rust USES_TERMINAL
        COMMAND sh build_rust.sh ${CMAKE_CURRENT_BINARY_DIR} ${CMAKE_CURRENT_LIST_DIR}
        WORKING_DIRECTORY ${CMAKE_CURRENT_LIST_DIR}
        COMMENT "Building Rust"
        BYPRODUCTS libstage_1.o
        )

add_custom_target(offsetloader
        COMMAND dd if=quantumbiosboot.bin skip=512 bs=1 iflag=skip_bytes,count_bytes of=loader.bin && echo -n -e \\\\xef\\\\xbe >> loader.bin
        COMMENT "Packaging Loader"
        DEPENDS quantumbiosboot.bin
        BYPRODUCTS loader.bin
)

add_custom_target(${PROJECT_NAME} ALL
        COMMAND touch QuantumBiosBoot.sys && rm QuantumBiosBoot.sys && cat $<TARGET_OBJECTS:mbr> loader.bin > QuantumBiosBoot.sys
        COMMENT "Creating QuantumBiosBoot.sys"
        DEPENDS mbr offsetloader
        BYPRODUCTS QuantumBiosBoot.sys
)

add_custom_target(run USES_TERMINAL
        COMMAND qemu-system-i386 -d cpu_reset --no-shutdown -fda QuantumBiosBoot.sys
        DEPENDS ${PROJECT_NAME}
)

add_custom_target(debug USES_TERMINAL
        COMMAND qemu-system-i386 -S -s -d cpu_reset --no-reboot -fda QuantumBiosBoot.sys
        DEPENDS ${PROJECT_NAME}
        )
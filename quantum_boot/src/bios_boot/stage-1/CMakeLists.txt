cmake_minimum_required(VERSION 3.25)
project(STAGE-1)


add_custom_target(stage-1
        COMMAND cargo build --release --target ${CMAKE_CURRENT_LIST_DIR}/i386-quantum_loader.json
        COMMENT "building rust"
        BYPRODUCTS ${CMAKE_CURRENT_LIST_DIR}/target/i386-quantum_loader/release/stage-1
)

add_custom_target(flat-stage-1
        COMMAND objcopy -I elf32-i386 -O binary ${CMAKE_CURRENT_LIST_DIR}/target/i386-quantum_loader/release/stage-1
        COMMENT "Flatting binaries"
        BYPRODUCTS ${CMAKE_CURRENT_LIST_DIR}/target/i386-quantum_loader/release/stage-1
        DEPENDS stage-1
)

add_custom_target(run USES_TERMINAL
        COMMAND qemu-system-i386 -d cpu_reset -d int --no-shutdown -drive format=raw,file=${CMAKE_CURRENT_LIST_DIR}/target/i386-quantum_loader/release/stage-1
        COMMENT Running QEMU
        DEPENDS flat-stage-1
        )
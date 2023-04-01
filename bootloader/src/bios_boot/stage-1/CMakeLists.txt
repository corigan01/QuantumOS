cmake_minimum_required(VERSION 3.25)
project(STAGE-1)

add_custom_target(stage-1-clippy
        COMMAND cargo clippy --release --target ${CMAKE_CURRENT_LIST_DIR}/i386-quantum_loader.json
        COMMENT "Checking Rust"
)

add_custom_target(stage-1
        COMMAND cargo build --release --target ${CMAKE_CURRENT_LIST_DIR}/i386-quantum_loader.json
        COMMENT "building rust"
        BYPRODUCTS ${CMAKE_CURRENT_LIST_DIR}/target/i386-quantum_loader/release/stage-1
        DEPENDS stage-1-clippy
)

add_custom_target(img
        COMMAND cp ${CMAKE_CURRENT_LIST_DIR}/tools/* ${CMAKE_BINARY_DIR}/ && sh mkimg.sh
        COMMENT "Generating Disk img"
        BYPRODUCTS disk.img
)

add_custom_target(get_ready_stage_1
        COMMAND
                cp ${CMAKE_CURRENT_LIST_DIR}/target/i386-quantum_loader/release/stage-1 ${CMAKE_BINARY_DIR}/ &&
                objcopy -I elf32-i386 -O binary ${CMAKE_BINARY_DIR}/stage-1 &&
                stat --printf="%s" ${CMAKE_BINARY_DIR}/stage-1 &&
                head -c 440 stage-1 | dd conv=notrunc bs=512 count=1 of=disk.img &&
                head -c 512 disk.img &> tmp.bin &&
                tail -c+513 stage-1 &>> tmp.bin &&
                cat tmp.bin | dd conv=notrunc of=disk.img &&
                rm tmp.bin stage-1
        COMMENT "Pasting stage into diskimg"
        BYPRODUCTS ${CMAKE_CURRENT_LIST_DIR}/target/i386-quantum_loader/release/stage-1
        DEPENDS stage-1
)

add_custom_target(run USES_TERMINAL
        COMMAND qemu-system-i386 -d cpu_reset --no-shutdown -drive format=raw,file=${CMAKE_BINARY_DIR}/disk.img
        COMMENT Running QEMU
        DEPENDS get_ready_stage_1
        )
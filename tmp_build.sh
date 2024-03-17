#!/bin/bash

cd bootloader/stage-bootsector/
cargo clean && cargo build --release --target ../../linkerscripts/i386-quantum_loader.json
cd ../../target/i386-quantum_loader/release/
ls
objcopy -I elf32-i386 -O binary ./stage-bootsector && hexdump stage-bootsector && qemu-system-x86_64 -drive format=raw,file=./stage-bootsector -enable-kvm

#!/bin/bash

cd bootloader/
cargo build
qemu-system-x86_64 -drive format=raw,file=./target/i386-quantum_loader/stage-bootsector/stage-bootsector.bin  -enable-kvm

#!/usr/bin/env bash

# $1 should be the build dir
# $2 should be this files dir

# Run Cargo
cd "$2"
cargo rustc --release --target-dir "$1" --target "$2"/x86_64-quantum_os.json -Zbuild-std=core -Zbuild-std-features=compiler-builtins-mem -- --emit=obj
cp "$1"/x86_64-quantum_os/release/deps/stage_1* "$1"/

mv "$(find "$1"/*.o)" "$1"/libstage_1.o
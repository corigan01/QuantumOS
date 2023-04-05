#!/bin/bash

echo "Setting nightly"
rustup override set nightly

echo "Adding rust components"
rustup component add rust-src
rustup component add llvm-tools-preview

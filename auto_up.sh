#!/bin/bash

rustup override set nightly

rustup component add rust-src
rustup component add llvm-tools-preview

cargo install bootimage
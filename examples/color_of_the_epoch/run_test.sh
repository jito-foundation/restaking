#!/bin/bash

# Build programs
cargo build-sbf --manifest-path programs/color_of_the_epoch/Cargo.toml;

# Run all tests except the specified one
SBF_OUT_DIR=$(pwd)/target/deploy cargo test


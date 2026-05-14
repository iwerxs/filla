#!/usr/bin/env bash
# Minimal, robust build script for filler
# Works locally or inside Docker

set -euo pipefail

echo "Cleaning previous build artifacts..."
cargo clean

echo "Building Filler bot in release mode..."
cargo build --release

echo "Build complete. Binary is at:"
echo "  $(pwd)/target/release/filler"

echo "Example run command:"
echo "  ./game_engine -f maps/map01 -p1 target/release/filler -p2 robots/bender"



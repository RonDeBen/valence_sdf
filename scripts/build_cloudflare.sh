#!/bin/bash
set -e

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Install tools
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.108

# Install wasm-opt
apt-get update && apt-get install -y binaryen

# Build
cargo build --target wasm32-unknown-unknown --profile wasm-release

# Generate bindings
wasm-bindgen --out-dir ./wasm --target web \
  ./target/wasm32-unknown-unknown/wasm-release/valence_sdf.wasm

# Optimize WASM
wasm-opt -Oz \
  --enable-bulk-memory \
  --enable-nontrapping-float-to-int \
  --enable-sign-ext \
  --enable-mutable-globals \
  --strip-debug \
  --strip-producers \
  ./wasm/valence_sdf_bg.wasm -o ./wasm/valence_sdf_bg.wasm

# Prepare dist
mkdir -p dist/assets/fonts/fredoka dist/assets/shaders
cp wasm/* dist/
cp index.html dist/
cp assets/fonts/fredoka/fredoka-bold-digits.{json,png} dist/assets/fonts/fredoka/
cp assets/shaders/{sdf_scene,seven_segment}.wgsl dist/assets/shaders/

echo "✅ Build complete! WASM size:"
ls -lh dist/valence_sdf_bg.wasm

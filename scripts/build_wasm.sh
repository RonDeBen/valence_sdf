#!/bin/bash
set -e

echo "🔨 Building WASM..."
cargo build --target wasm32-unknown-unknown --profile wasm-release

echo "🔗 Generating bindings..."
wasm-bindgen --out-dir ./wasm --target web \
  ./target/wasm32-unknown-unknown/wasm-release/valence_sdf.wasm

echo "📦 Preparing dist..."
rm -rf dist
mkdir -p dist/assets/fonts/fredoka dist/assets/shaders

cp wasm/* dist/
cp index.html dist/
cp assets/fonts/fredoka/fredoka-bold-digits.{json,png} dist/assets/fonts/fredoka/
cp assets/shaders/{sdf_scene,seven_segment}.wgsl dist/assets/shaders/

echo "✅ Done! Starting server at http://localhost:8000"
echo "   Press Ctrl+C to stop"
miniserve dist --port 8000

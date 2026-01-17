#!/bin/bash
set -e

echo "🔨 Building WASM..."
cargo build --target wasm32-unknown-unknown --profile wasm-release

echo "🔗 Generating bindings..."
wasm-bindgen --out-dir ./wasm --target web \
  ./target/wasm32-unknown-unknown/wasm-release/valence_sdf.wasm

echo "⚡ Optimizing WASM..."
if command -v wasm-opt &>/dev/null; then
  wasm-opt -Oz \
    --enable-bulk-memory \
    --enable-nontrapping-float-to-int \
    --enable-sign-ext \
    --enable-mutable-globals \
    --strip-debug \
    --strip-producers \
    ./wasm/valence_sdf_bg.wasm -o ./wasm/valence_sdf_bg.wasm
  echo "✅ Optimized with wasm-opt"
else
  echo "⚠️  wasm-opt not found. Install with: brew install binaryen"
fi

echo "📦 Preparing dist..."
rm -rf dist
mkdir -p dist/assets/fonts/fredoka dist/assets/shaders

cp wasm/* dist/
cp index.html dist/
cp assets/fonts/fredoka/fredoka-bold-digits.{json,png} dist/assets/fonts/fredoka/
cp assets/shaders/{sdf_scene,seven_segment}.wgsl dist/assets/shaders/

echo "📊 WASM size:"
ls -lh dist/valence_sdf_bg.wasm

echo "✅ Done! Starting server at http://localhost:8000"
echo "   Press Ctrl+C to stop"
miniserve dist --port 8000 --index index.html

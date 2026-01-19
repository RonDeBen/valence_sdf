# Valence SDF

A graph-based puzzle game rendered entirely with signed distance fields (SDFs) in custom WGSL shaders.

**[🎮 Play Now](https://valence-sdf.fly.dev/)**

## About

Connect nodes on a grid by manipulating their valences. Each node displays its current valence as a digit rendered through SDF techniques, with smooth animations and effects.

## Local Development
```bash
cargo run
```

## Building for Web
```bash
./scripts/build_wasm.sh
```

Requires:
- `wasm-bindgen-cli`: `cargo install wasm-bindgen-cli`
- `miniserve`: `cargo install miniserve`

---

*Requires a WebGPU-compatible browser (Chrome 113+, Firefox 131+, Safari 18+)*

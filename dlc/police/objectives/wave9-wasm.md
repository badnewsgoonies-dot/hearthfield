# Wave 9: WASM Build — Precinct DLC

## Deliverables
1. build_wasm.sh script for Precinct DLC
2. Bevy WASM config: webgl2, no externref, Tonemapping::None (already set)
3. tonemapping_luts + ktx2 + zstd features in Cargo.toml
4. index.html with loading overlay and error capture
5. Audio: .ogg only (WASM can't decode wav)
6. Asset path resolution for WASM context
7. Test in browser: game loads, no purple/gray screen

## Known WASM pitfalls (from Hearthfield):
- wasm-bindgen version must match Bevy's pinned version
- externref causes segfaults on iOS Safari
- Missing tonemapping LUT = gray/purple screen
- Only vorbis (.ogg) audio works in WASM

#!/usr/bin/env bash
set -euo pipefail

echo "=== Building Hearthfield for WASM ==="

# Ensure tools are installed
rustup target add wasm32-unknown-unknown 2>/dev/null || true

# Check if wasm-bindgen-cli is installed
if ! command -v wasm-bindgen &>/dev/null; then
    echo "Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
fi

# Build release for WASM (wasm-release profile: opt-level=z, LTO)
echo "Building wasm-release..."
cargo build --profile wasm-release --target wasm32-unknown-unknown

# wasm-bindgen to generate JS glue
# RAYON_NUM_THREADS=1 prevents rayon worker thread stack overflows
# when walrus parses large WASM binaries (73K+ functions)
echo "Running wasm-bindgen..."
mkdir -p web
WASM_BIN="target/wasm32-unknown-unknown/wasm-release/hearthfield.wasm"
if [ ! -f "$WASM_BIN" ]; then
    WASM_BIN="target/wasm32-unknown-unknown/release/hearthfield.wasm"
fi
RAYON_NUM_THREADS=1 wasm-bindgen \
    --out-dir web \
    --target web \
    --no-typescript \
    "$WASM_BIN"

# Optional: optimize with wasm-opt if available (saves ~40% size)
if command -v wasm-opt &>/dev/null; then
    echo "Optimizing with wasm-opt..."
    wasm-opt -Oz \
        --enable-bulk-memory \
        --enable-sign-ext \
        --enable-nontrapping-float-to-int \
        --enable-mutable-globals \
        --strip-debug \
        --strip-producers \
        web/hearthfield_bg.wasm \
        -o web/hearthfield_bg.wasm
fi

# Copy assets into web folder
echo "Copying assets..."
rm -rf web/assets
cp -r assets web/assets

# Copy index.html
cp web_template/index.html web/index.html

# Report sizes
echo ""
echo "=== Build complete ==="
echo "WASM size: $(du -h web/hearthfield_bg.wasm | cut -f1)"
echo "JS glue:   $(du -h web/hearthfield.js | cut -f1)"
echo "Assets:    $(du -sh web/assets/ | cut -f1)"
echo "Total:     $(du -sh web/ | cut -f1)"
echo ""
echo "Upload web/ folder to itch.io as HTML game."
echo "Or test locally: cd web && python3 -m http.server 8080"

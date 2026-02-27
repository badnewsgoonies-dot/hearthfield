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

# Build release for WASM
echo "Building release..."
cargo build --release --target wasm32-unknown-unknown --profile wasm-release 2>/dev/null \
  || cargo build --release --target wasm32-unknown-unknown

# wasm-bindgen to generate JS glue
echo "Running wasm-bindgen..."
mkdir -p web
wasm-bindgen \
    --out-dir web \
    --target web \
    --no-typescript \
    target/wasm32-unknown-unknown/release/hearthfield.wasm

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
echo "Assets:    $(du -sh web/assets/ | cut -f1)"
echo "Total:     $(du -sh web/ | cut -f1)"
echo ""
echo "Upload web/ folder to itch.io as HTML game."
echo "Or test locally: cd web && python3 -m http.server 8080"

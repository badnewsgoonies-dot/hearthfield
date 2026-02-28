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

WASM_BIN="target/wasm32-unknown-unknown/wasm-release/hearthfield.wasm"
if [ ! -f "$WASM_BIN" ]; then
    WASM_BIN="target/wasm32-unknown-unknown/release/hearthfield.wasm"
fi

# Strip target_features section to prevent wasm-bindgen from using externref.
# Mobile Safari on older iOS doesn't reliably support externref; without
# target_features, wasm-bindgen falls back to the slab-based approach.
echo "Stripping target_features (disable externref)..."
STRIPPED_BIN="${WASM_BIN%.wasm}-stripped.wasm"
python3 tools/strip_target_features.py "$WASM_BIN" "$STRIPPED_BIN"

# wasm-bindgen to generate JS glue
# RAYON_NUM_THREADS=1 prevents rayon worker thread stack overflows
# when walrus parses large WASM binaries (73K+ functions)
echo "Running wasm-bindgen..."
mkdir -p web
RAYON_NUM_THREADS=1 wasm-bindgen \
    --out-dir web \
    --target web \
    --no-typescript \
    --remove-name-section \
    --remove-producers-section \
    "$STRIPPED_BIN"

# Rename outputs from stripped name to hearthfield
if [ -f "web/hearthfield-stripped.js" ]; then
    mv web/hearthfield-stripped.js web/hearthfield.js
    mv web/hearthfield-stripped_bg.wasm web/hearthfield_bg.wasm
fi

# Optional: optimize with wasm-opt if available (saves ~40% size)
if command -v wasm-opt &>/dev/null; then
    echo "Optimizing with wasm-opt..."
    wasm-opt -Oz \
        --enable-bulk-memory \
        --enable-sign-ext \
        --enable-nontrapping-float-to-int \
        --enable-mutable-globals \
        --disable-reference-types \
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

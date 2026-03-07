#!/bin/bash
# Build Hearthfield for WASM deployment
set -e

echo "=== Hearthfield WASM Build ==="

# Check prerequisites
if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

if ! command -v wasm-bindgen &>/dev/null; then
    echo "Installing wasm-bindgen-cli..."
    cargo install wasm-bindgen-cli
fi

# Build with WASM release profile
echo "Building for WASM (this may take a few minutes)..."
cargo build --profile wasm-release --target wasm32-unknown-unknown

# Generate JS bindings
echo "Generating JS bindings..."
mkdir -p web/pkg
wasm-bindgen --out-dir web/pkg --target web \
    target/wasm32-unknown-unknown/wasm-release/hearthfield.wasm

# Optional size optimization
if command -v wasm-opt &>/dev/null; then
    echo "Optimizing WASM binary size..."
    wasm-opt -Oz web/pkg/hearthfield_bg.wasm -o web/pkg/hearthfield_bg.wasm
    echo "Size after optimization: $(du -h web/pkg/hearthfield_bg.wasm | cut -f1)"
else
    echo "wasm-opt not found, skipping size optimization"
    echo "Install with: cargo install wasm-opt (or via binaryen package)"
fi

echo ""
echo "Build complete!"
echo "WASM size: $(du -h web/pkg/hearthfield_bg.wasm | cut -f1)"
echo ""
echo "To test locally:"
echo "  python3 -m http.server 8080 --directory web"
echo "  Open http://localhost:8080"
echo ""
echo "To deploy to itch.io:"
echo "  1. Zip the web/ directory"
echo "  2. Upload to your itch.io project"
echo "  3. Set 'Kind of project' to 'HTML'"
echo "  4. Check 'This file will be played in the browser'"

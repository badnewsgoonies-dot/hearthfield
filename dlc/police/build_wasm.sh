#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
WEB_DIR="$SCRIPT_DIR/web"
PKG_DIR="$WEB_DIR/pkg"
LOCKFILE="$WORKSPACE_DIR/Cargo.lock"

echo "=== Precinct WASM Build ==="

if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "Installing wasm32-unknown-unknown target..."
    rustup target add wasm32-unknown-unknown
fi

WASM_BINDGEN_VERSION="$(
    awk '
        $0 == "name = \"wasm-bindgen\"" {
            getline;
            gsub(/version = "|"/, "", $0);
            print $0;
            exit;
        }
    ' "$LOCKFILE"
)"

if [ -z "$WASM_BINDGEN_VERSION" ]; then
    echo "Failed to determine wasm-bindgen version from $LOCKFILE" >&2
    exit 1
fi

INSTALLED_WASM_BINDGEN_VERSION=""
if command -v wasm-bindgen >/dev/null 2>&1; then
    INSTALLED_WASM_BINDGEN_VERSION="$(wasm-bindgen --version | awk '{print $2}')"
fi

if [ "$INSTALLED_WASM_BINDGEN_VERSION" != "$WASM_BINDGEN_VERSION" ]; then
    echo "Installing wasm-bindgen-cli $WASM_BINDGEN_VERSION..."
    cargo install --locked --force "wasm-bindgen-cli" --version "$WASM_BINDGEN_VERSION"
fi

echo "Building precinct for wasm32-unknown-unknown..."
cargo build \
    -p precinct \
    --profile wasm-release \
    --target wasm32-unknown-unknown \
    --manifest-path "$SCRIPT_DIR/Cargo.toml"

WASM_BIN="$WORKSPACE_DIR/target/wasm32-unknown-unknown/wasm-release/precinct.wasm"
if [ ! -f "$WASM_BIN" ]; then
    WASM_BIN="$WORKSPACE_DIR/target/wasm32-unknown-unknown/release/precinct.wasm"
fi

if [ ! -f "$WASM_BIN" ]; then
    echo "Precinct wasm binary was not produced." >&2
    exit 1
fi

STRIPPED_BIN="${WASM_BIN%.wasm}-stripped.wasm"
echo "Stripping target_features (disable externref)..."
python3 "$WORKSPACE_DIR/tools/strip_target_features.py" "$WASM_BIN" "$STRIPPED_BIN"

mkdir -p "$PKG_DIR"

echo "Running wasm-bindgen..."
RAYON_NUM_THREADS=1 wasm-bindgen \
    --out-dir "$PKG_DIR" \
    --target web \
    --no-typescript \
    --remove-name-section \
    --remove-producers-section \
    "$STRIPPED_BIN"

if [ -f "$PKG_DIR/precinct-stripped.js" ]; then
    sed -i 's/precinct-stripped_bg\.wasm/precinct_bg.wasm/g' "$PKG_DIR/precinct-stripped.js"
    mv "$PKG_DIR/precinct-stripped.js" "$PKG_DIR/precinct.js"
    mv "$PKG_DIR/precinct-stripped_bg.wasm" "$PKG_DIR/precinct_bg.wasm"
fi

if command -v wasm-opt >/dev/null 2>&1; then
    echo "Optimizing WASM binary size..."
    wasm-opt -Oz \
        --enable-bulk-memory \
        --enable-sign-ext \
        --enable-nontrapping-float-to-int \
        --enable-mutable-globals \
        --disable-reference-types \
        --strip-debug \
        --strip-producers \
        "$PKG_DIR/precinct_bg.wasm" \
        -o "$PKG_DIR/precinct_bg.wasm"
fi

echo "Copying precinct assets..."
rm -rf "$WEB_DIR/assets"
cp -r "$SCRIPT_DIR/assets" "$WEB_DIR/assets"

if [ ! -f "$WEB_DIR/index.html" ]; then
    echo "Missing $WEB_DIR/index.html" >&2
    exit 1
fi

echo
echo "=== Build complete ==="
echo "WASM size: $(du -h "$PKG_DIR/precinct_bg.wasm" | cut -f1)"
echo "JS glue:   $(du -h "$PKG_DIR/precinct.js" | cut -f1)"
echo "Assets:    $(du -sh "$WEB_DIR/assets" | cut -f1)"
echo "Total:     $(du -sh "$WEB_DIR" | cut -f1)"
echo
echo "Test locally: cd \"$SCRIPT_DIR\" && python3 -m http.server 8080"
echo "Open:       http://localhost:8080/web/"

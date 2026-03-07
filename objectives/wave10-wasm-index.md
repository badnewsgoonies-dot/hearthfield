# Worker: WASM Deployment Bundle

## Scope (mechanically enforced)
You may only modify/create files in the project root (index.html, web/) and Cargo.toml.
Do NOT modify any src/ files.

## Required reading
1. Cargo.toml — WASM profile and dependencies already configured
2. .cargo/config.toml — WASM rustflags already set

## Task
Create the web deployment bundle files needed for WASM:

1. **Create `web/index.html`** — a loading page for the WASM game:
   - Dark background, centered canvas
   - Loading spinner/progress bar
   - Import the WASM module via wasm-bindgen
   - Auto-resize canvas to window
   - Touch event prevention (for mobile)
   - Basic error handling if WASM fails to load

2. **Create `web/build.sh`** — build script:
   ```bash
   #!/bin/bash
   # Build Hearthfield for WASM
   set -e
   cargo build --profile wasm-release --target wasm32-unknown-unknown
   wasm-bindgen --out-dir web/pkg --target web \
     target/wasm32-unknown-unknown/wasm-release/hearthfield.wasm
   # Optional: wasm-opt for size reduction
   if command -v wasm-opt &>/dev/null; then
     wasm-opt -Oz web/pkg/hearthfield_bg.wasm -o web/pkg/hearthfield_bg.wasm
   fi
   echo "Build complete! Serve web/ with any HTTP server."
   ```

3. **Create `web/README.md`** with build/deploy instructions for itch.io

## Validation
- Files created correctly
- build.sh is executable
- HTML is valid and references correct WASM paths

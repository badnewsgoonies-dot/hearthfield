Original prompt: can yiu launch build its wasmn its a rust harvest moons game but recently had a harness being built up could giu play on the dex screen

- Pulled latest `origin/master` to pick up the top-level WASM harness (`build_wasm.sh`, `web/`, `web_template/`).
- Confirmed the prior local checkout was stale; the fresh checkout does include a browser path.
- Started the top-level WASM build and hit a real toolchain mismatch: project lockfile uses `wasm-bindgen` 0.2.114 while the installed CLI was 0.2.115.
- Patched `build_wasm.sh` to auto-detect the lockfile version and install the matching `wasm-bindgen-cli` before running glue generation.
- Reran `build_wasm.sh` successfully and served `web/` locally over HTTP.
- Opened the build on Samsung DeX and caught a real runtime failure from the in-page error overlay: Bevy requested a `4320x2430` surface while the browser max texture dimension is `4096`.
- Patched the web harness to clamp effective `devicePixelRatio` on high-density displays before Bevy initializes so DeX/high-DPI browsers stay under the surface cap.
- The DPR override alone did not clear the DeX failure, so I added a second guardrail: the harness now shrinks `#game-container` on high-density displays so `fit_canvas_to_parent` cannot exceed the browser texture ceiling even if DPR override is ignored.
- The panic persisted after a cache-busted reload, which means I still need visibility into what Chrome on DeX thinks the DPR and viewport are.
- Exposed harness debug values in the in-page error overlay and confirmed the clamp logic is executing, but Chrome on DeX reports tiny `visualViewport` metrics (`642x280`) while Bevy still requests a much larger `960x540` surface (`4320x2430` at DPR `4.5`).
- Next step: switch the clamp to the largest available viewport/screen metrics (`screen.*`, `outer*`, `client*`) instead of trusting `visualViewport`, then reload again.

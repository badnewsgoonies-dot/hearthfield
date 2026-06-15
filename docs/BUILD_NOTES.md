# Build & dev-loop notes (optimization findings)

Measured on a constrained box (1 core, 3.7 GB disk). Numbers will be much better on a
multi-core dev machine, but the *relative* findings hold.

## The real bottleneck: full recompile of one monolithic crate

`hearthfield` is essentially one large crate (all of `src/` compiled as a unit). Touching **any**
source file recompiles the whole crate. Measured cost of a one-file change with incremental
compilation **off**: **467 s (7m46s)** on 1 core — almost entirely codegen, not linking.

Implications, in priority order:

1. **Keep incremental compilation ON** (`CARGO_INCREMENTAL=1`, cargo's default for dev/check).
   This is the single biggest dev-loop win: it recompiles only the changed codegen units instead of
   the whole crate. *Do not* set `CARGO_INCREMENTAL=0` unless you are disk-starved — that was the
   accidental cause of the slow 467 s rebuild above. The trade-off is disk: the incremental cache for
   this crate is several hundred MB and on a 3.7 GB box it does not fit alongside `target/`
   (target alone is ~1.5–2.1 GB). **Disk, not CPU, is the binding constraint on a small box.**

2. **Use a fast linker (mold)** for the build→run loop. Link time is a smaller fraction than codegen
   for a from-scratch crate recompile, but it matters once incremental compilation makes codegen
   cheap (the link then dominates). Zero-config usage that does **not** trigger a full rebuild:

   ```
   mold -run cargo build --bin hearthfield
   ```

   (Prefer this over putting `-C link-arg=-fuse-ld=mold` in `rustflags`, which changes the build
   fingerprint and forces a full recompile. `mold -run` intercepts the linker transparently.)

3. **Prefer `cargo check` over `cargo build`** while iterating on correctness. Check skips codegen and
   linking entirely; incremental checks here are ~26–30 s vs minutes for a build. Only build when you
   actually need to *run* the game.

4. **Disable debuginfo for throwaway builds** (`CARGO_PROFILE_DEV_DEBUG=false`) — roughly halves
   `target/` size and speeds linking. Useful on a disk-constrained box; keep debuginfo on where you
   need a debugger.

5. **sccache** (installed) caches compiled artifacts across `cargo clean`, so a clean rebuild pulls
   from cache instead of recompiling all ~400 dependency crates (~32 min cold). Worth it on a machine
   with disk headroom (its cache is its own multi-GB store). Enable with
   `RUSTC_WRAPPER=sccache` — note this changes the build fingerprint, so the first build after
   enabling it recompiles everything to populate the cache.

## Headless run / software rendering (no GPU)

The game runs with no graphics card via a virtual display + Mesa software Vulkan (lavapipe):

```
xvfb-run -a -s "-screen 0 1280x720x24" \
  env VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json WGPU_BACKEND=vulkan \
      HEARTHFIELD_HEADLESS=1 HF_PROC_SEED=1002 \
  ./target/debug/hearthfield
```

`HEARTHFIELD_HEADLESS=1` writes per-frame state to `/tmp/hearthfield-state.json` and the collision
grid to `/tmp/hearthfield-collision.json`. `HF_PROC_SEED=<u64>` auto-enters that procedural map a few
frames after reaching Playing. Note bevy throttles updates when the window is unfocused, so a headless
driver must `xdotool windowactivate`/`windowfocus` the window before injecting input; `Escape` opens
the pause menu (not "close overlay").

## Quick reference

- Iterate on correctness: `cargo check` (fast, incremental).
- Build to run: `mold -run cargo build --bin hearthfield` with `CARGO_INCREMENTAL=1`.
- Disk-starved: `CARGO_PROFILE_DEV_DEBUG=false`, and clear `target/debug/incremental` to reclaim space
  (at the cost of the next build being a full recompile).

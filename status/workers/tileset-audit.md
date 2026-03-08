# Tileset Tiling Correctness Audit Report

**Date:** 2026-03-08
**Auditor:** Worker B (tileset audit)
**Scope:** All 10 tileset PNGs + reference sheet

---

## Step 1: Dimension Verification

All 10 tilesets pass dimension and format checks.

| Tileset | Location | Expected | Actual | RGBA | Status |
|---------|----------|----------|--------|------|--------|
| grass.png | tilesets/ | 176x112 | 176x112 | Yes | OK |
| tilled_dirt.png | tilesets/ | 176x112 | 176x112 | Yes | OK |
| water.png | tilesets/ | 64x16 | 64x16 | Yes | OK |
| hills.png | tilesets/ | 176x144 | 176x144 | Yes | OK |
| house_walls.png | tilesets/ | 80x48 | 80x48 | Yes | OK |
| house_roof.png | tilesets/ | 112x80 | 112x80 | Yes | OK |
| doors.png | tilesets/ | 16x64 | 16x64 | Yes | OK |
| fences.png | tilesets/ | 64x64 | 64x64 | Yes | OK |
| wood_bridge.png | sprites/ | 80x48 | 80x48 | Yes | OK |
| paths.png | sprites/ | 64x64 | 64x64 | Yes | OK |

## Step 2: Fill Rates

| Tileset | Filled | Partial | Empty | Total | Content % |
|---------|--------|---------|-------|-------|-----------|
| grass.png | 75 | 0 | 2 | 77 | 97% |
| tilled_dirt.png | 60 | 17 | 0 | 77 | 100% |
| water.png | 1 | 3 | 0 | 4 | 100% |
| hills.png | 53 | 46 | 0 | 99 | 100% |
| house_walls.png | 10 | 5 | 0 | 15 | 100% |
| house_roof.png | 1 | 4 | **30** | 35 | **14%** |
| doors.png | 4 | 0 | 0 | 4 | 100% |
| fences.png | 0 | 16 | 0 | 16 | 100% |
| wood_bridge.png | 0 | 5 | **10** | 15 | **33%** |
| paths.png | 4 | 12 | 0 | 16 | 100% |

## Step 3: Code-Referenced Index Validation

### CRITICAL ISSUES FOUND

#### 1. wood_bridge.png — idx 7 is EMPTY (Bridge tile)
- **Code:** `src/world/mod.rs:400-405` uses index 7 for `TileKind::Bridge`
- **Problem:** Only row 0 (indices 0-4) has content. Rows 1-2 are fully transparent.
- **Impact:** All bridge tiles in-game render as invisible. The forest bridge over the river is invisible.
- **Fix needed:** Either rearrange the bridge sprite content to fill row 1, or change the code index from 7 to one of [0,1,2,3,4] (e.g., index 2 for center plank).

#### 2. house_roof.png — ALL code-referenced indices are EMPTY
- **Code:** `src/world/objects.rs:1527-1559` uses indices 22, 25, 27, 31, 32, 34
- **Problem:** Only row 0 (indices 0-4) has content. Rows 1-4 are fully transparent.
- **Impact:** All building roofs render as invisible. Town buildings and the farm house have no visible roofs.
- **Indices used by code vs content:**
  - idx 22 (row 3, col 1): EMPTY -- peak left
  - idx 25 (row 3, col 4): EMPTY -- peak body
  - idx 27 (row 3, col 6): EMPTY -- peak right
  - idx 31 (row 4, col 3): EMPTY -- eave left
  - idx 32 (row 4, col 4): EMPTY -- eave body
  - idx 34 (row 4, col 6): EMPTY -- eave right
- **Fix needed:** Either fill rows 3-4 with roof tile art, or remap code indices to row 0 (0-4).

### Passing Indices

All other code-referenced indices have content:

- **grass.png:** Spring [4-7], Summer [15-18], Fall [26-29], Winter [37-40], Sand [46] -- all OK
- **tilled_dirt.png:** Dirt [5], TilledSoil [12], WateredSoil [16], WoodFloor [6] -- all OK
- **water.png:** Animation frames [0-3] -- all 4 present and distinct
- **hills.png:** Stone [0], Void outdoor [60] -- both OK
- **paths.png:** All 16 autotile variants [0-15] -- all present
- **house_walls.png:** Indices [1,3,4,6,8,9,11] -- all OK (fill 72-100%)
- **doors.png:** Index [1] -- OK (fill 100%)

## Step 4: Autotile Edge Matching

Autotile sheets contain variant tiles (corners/edges/centers) that are not meant to tile sequentially in the spritesheet, so adjacent-cell edge mismatches are expected.

| Sheet | Matching Edges | Total Edges | Match % | Notes |
|-------|---------------|-------------|---------|-------|
| grass.png | 130 | 130 | 100% | Excellent internal consistency |
| tilled_dirt.png | 112 | 136 | 82% | Expected for multi-terrain variants |
| hills.png | 141 | 178 | 79% | Expected for cliff/terrain transitions |

No anomalous mismatches detected -- all low-similarity pairs are at intentional terrain transitions.

## Step 5: Visual Consistency

### Style Assessment
- **grass.png:** Consistent LimeZu Modern Farm palette. 2 solid-color tiles at indices 31 and 75 (likely intentional fill/accent tiles).
- **tilled_dirt.png:** Consistent palette. 1 solid-color tile at index 56.
- **water.png:** Semi-transparent animated water frames. Consistent style.
- **hills.png:** Consistent terrain palette. 3 solid teal tiles at indices 17, 18, 33 (appears intentional -- water/sky accents in cliff tileset).
- **house_walls.png:** Consistent building style, all 15 tiles populated with varied wall textures.
- **house_roof.png:** Only 5 tiles in row 0 have content -- rest of the 112x80 sheet is empty. The existing tiles are consistent style.
- **doors.png:** 4 door variants, consistent style.
- **fences.png:** 16 fence variants with alpha cutouts, consistent wooden fence style.
- **wood_bridge.png:** 5 tiles in row 0 only -- rest empty. Existing tiles are consistent wooden plank style.
- **paths.png:** 16 path autotile variants with alpha edges, consistent dirt/gravel style.

### No broken or corrupted tiles detected across any tileset.

## Step 6: Critical Pattern Check

| Pattern | Status |
|---------|--------|
| Grass seasonal variants (4 seasons x 4 each) | All 16 present |
| Path 4-bit autotile (16 NSEW variants) | All 16 present |
| Water 4-frame animation | All 4 present and distinct |
| Hills stone tile (idx 0) | Present (59% fill) |
| Hills void outdoor (idx 60) | Present (100% fill) |
| Tilled dirt critical tiles (5, 6, 12, 16) | All present |
| Bridge center (idx 7) | **EMPTY** |
| Roof eave/peak tiles (22, 25, 27, 31, 32, 34) | **ALL EMPTY** |

## Cargo Check Gate

```
cargo check: PASSED (0 errors, 0 warnings)
```

Note: cargo check validates Rust compilation correctness. It does not detect empty atlas indices at compile time -- those are runtime visual issues (invisible sprites).

---

## Per-Tileset Verdicts

| Tileset | Verdict | Issue |
|---------|---------|-------|
| grass.png | **READY** | -- |
| tilled_dirt.png | **READY** | -- |
| water.png | **READY** | -- |
| hills.png | **READY** | -- |
| house_walls.png | **READY** | -- |
| house_roof.png | **NEEDS-WORK** | Code refs rows 3-4 but only row 0 has art |
| doors.png | **READY** | -- |
| fences.png | **READY** | -- |
| wood_bridge.png | **NEEDS-WORK** | Code refs idx 7 (row 1) but only row 0 has art |
| paths.png | **READY** | -- |

## Overall Verdict: NEEDS-WORK

### Summary of Required Fixes (2 issues)

1. **wood_bridge.png idx 7 is empty** -- Bridge tiles in the forest map (and any other Bridge terrain) render as invisible. Fix: either populate row 1 of wood_bridge.png with plank tiles, or change `src/world/mod.rs:402` from `index: 7` to `index: 2` (center plank in row 0).

2. **house_roof.png rows 3-4 are empty** -- All building roofs render as invisible in Town and Farm maps. Fix: either populate rows 3-4 of house_roof.png with eave/peak tiles, or remap indices in `src/world/objects.rs:1541-1558` to use row 0 indices (0-4).

Both issues are **visual-only** (no compile errors, no crashes) but produce invisible game elements.

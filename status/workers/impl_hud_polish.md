# HUD Polish Implementation

## Status: DONE

## Changes Made

**File:** `src/ui/hud.rs` — `update_stamina_bar` function

Replaced smooth gradient interpolation with discrete 3-zone color thresholds:

| Stamina % | Color | sRGB |
|-----------|-------|------|
| > 60% | Green | `(0.2, 0.8, 0.2)` |
| 30–60% | Yellow | `(0.9, 0.7, 0.1)` |
| < 30% | Red | `(0.9, 0.2, 0.2)` |

## Health Bar

No health bar exists in `src/ui/hud.rs` — no changes needed there.

## Notes

- Changes stashed via `git stash` as requested.
- Previous code used a smooth lerp from green→yellow→red across the full range (split at 50%). New code uses crisp threshold transitions matching the spec exactly.

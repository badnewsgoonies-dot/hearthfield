# Nora Schedule Fix Report

## Scope
- Updated Nora's original schedule definition in `src/data/npcs.rs`.
- Left all non-Farm coordinates unchanged.

## Changes Made
Adjusted out-of-bounds Farm coordinates in Nora's schedule:

- Replaced `x: 50, y: 50` with `x: 15, y: 14` (center of farm)
- Replaced `x: 48, y: 48` with `x: 14, y: 13` (near barn)

Applied across:
- `weekday`
- `weekend`
- `rain_override`

## Verification
- Farm bounds are valid at `x=0..29`, `y=0..19`.
- New Farm coordinates (`15,14` and `14,13`) are in bounds.
- Town and Forest entries were already in bounds and were not modified.

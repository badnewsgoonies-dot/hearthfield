# NPC Schedule Coordinate Fix Completion Report

- Number of coordinates fixed: 98
- Per-map breakdown of fixes:
  - Town: 75
  - Farm: 16
  - Beach: 3
  - Forest: 0
  - MineEntrance: 4
- Validation result: `Total failures: 0` (no out-of-bounds coordinates remaining)
- Compile/syntax check: `cargo check` passed (existing unrelated warnings only)

## Assumptions Made for NPC Placement
- Mayor placements were kept around plaza/festival and nearby civic paths.
- Margaret was kept near General Store/restaurant-adjacent town areas, with home moved to the store-side neighborhood.
- Elena (blacksmith) home/evening positions were moved to blacksmith-adjacent town coordinates; mine checks remained at MineEntrance.
- Doc herb-gathering seasonal points were remapped in-bounds while preserving research flow; deeper weekend waypoint (`saturating_add`) was kept valid across seasons.
- Old Tom remained beach-focused, with all beach points remapped to valid shoreline/pier-like coordinates and home kept near House 2 area.
- Marco remained restaurant-centered in town with in-bounds restaurant/home-adjacent coordinates.
- Sam remained mine-adjacent with MineEntrance points set to valid y-values and town home near house area.
- Nora farm work was remapped to central farm-plot coordinates per objective guidance; town market stops moved to valid southern town paths.
- Lily remained active between plaza/library/seasonal outings; all home and adventure points were remapped in-bounds, including weekend farther-play offsets.
- Mira’s summer beach research point was moved in-bounds while preserving seasonal behavior.

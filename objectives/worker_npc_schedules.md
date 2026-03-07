# Worker: NPC Schedule Coordinate Fix

## Scope (hard allowlist — enforced mechanically)
You may ONLY modify: src/npcs/schedules.rs
All other file edits will be reverted after you finish.

## Required Reading (in this order)
1. docs/domains/npc_schedules.md — full problem description and map bounds
2. src/npcs/schedules.rs — the file you are fixing
3. src/world/maps.rs — reference only, for understanding map layouts (DO NOT EDIT)

## Task
Fix ALL out-of-bounds NPC schedule coordinates in src/npcs/schedules.rs.

There are 98 out-of-bounds entries. Every ScheduleEntry with coordinates outside the map bounds must be remapped to a valid in-bounds location that makes sense for that NPC's activity.

## Map Bounds (TRUTH)
- Farm: 32×24 → valid x: 0-31, y: 0-23
- Town: 28×22 → valid x: 0-27, y: 0-21
- Beach: 20×14 → valid x: 0-19, y: 0-13
- Forest: 22×18 → valid x: 0-21, y: 0-17
- MineEntrance: 14×12 → valid x: 0-13, y: 0-11

## Town Landmarks (for sensible placement)
- Central plaza: x=11-16, y=7-12 (fountain at 13-14,9-10)
- General Store: x=2-9, y=2-6 (entrance x=5-6, y=2)
- Animal Shop: x=18-25, y=2-6 (entrance x=22-23, y=2)  
- Blacksmith: x=20-25, y=13-16 (entrance x=22-23, y=13)
- House 1 (doc area): x=2-6, y=13-15
- House 2 (fisher area): x=8-12, y=13-15
- Restaurant: x=14-18, y=16-18
- Park/pond: x=2-9, y=17-20
- N-S road: x=13-14, E-W road: y=9-10

## Remapping Rules
- Keep the NPC's "activity vibe" — if they were at a shop, place near that shop
- Mayor near town hall / plaza area (x=11-16, y=7-12)
- Margaret (baker) near general store or restaurant
- Elena (blacksmith) near blacksmith building
- Old Tom (fisherman) near beach areas or pier
- Doc near House 1 area
- NPCs "at home" → near house areas (x=2-12, y=13-15)
- Farm coords like x=48,y=50 → remap to valid farm area (x=10-20, y=10-18)
- Beach coords like x=16,y=24 → remap to valid beach area (x=5-15, y=5-10)
- MineEntrance y=12 → remap to y=10 or y=8

## Quantitative Target (non-negotiable)
- ZERO out-of-bounds coordinates remaining
- ALL 98 entries must be fixed

## Validation (run before reporting done)
```bash
python3 -c "
import re
bounds = {'Town':(28,22),'Farm':(32,24),'Beach':(20,14),'Forest':(22,18),'MineEntrance':(14,12)}
fail=0
with open('src/npcs/schedules.rs') as f:
    for i,line in enumerate(f,1):
        for name,(w,h) in bounds.items():
            if f'MapId::{name}' in line:
                m = re.search(r'x:\s*(\d+),\s*y:\s*(\d+)', line)
                if m and (int(m.group(1))>=w or int(m.group(2))>=h):
                    print(f'FAIL line {i}: {name} x={m.group(1)} y={m.group(2)}')
                    fail+=1
print(f'Total failures: {fail}')
"
```
Expected: `Total failures: 0`

Also verify the file still compiles as part of the project (syntax check).

## When Done
Write completion report to status/workers/npc_schedules.md containing:
- Number of coordinates fixed
- Per-map breakdown of fixes
- Validation result (0 OOB remaining)
- Any assumptions made about NPC placement

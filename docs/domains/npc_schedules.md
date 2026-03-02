# Domain: NPC Schedule Coordinate Fix

## Problem
Maps were resized to Harvest Moon scale but NPC schedule coordinates were never updated.
98 out-of-bounds schedule entries exist across 10 NPCs.

## Map Bounds (TRUTH — all coords must be strictly within these)
| Map | Width | Height | Valid X | Valid Y |
|-----|-------|--------|---------|---------|
| Farm | 32 | 24 | 0-31 | 0-23 |
| Town | 28 | 22 | 0-27 | 0-21 |
| Beach | 20 | 14 | 0-19 | 0-13 |
| Forest | 22 | 18 | 0-21 | 0-17 |
| MineEntrance | 14 | 12 | 0-13 | 0-11 |

## Town Layout (28×22) — NPC placement guide
- Central plaza: x=11-16, y=7-12 (stone square, fountain at 13-14,9-10)
- Main road N-S: x=13-14, full height
- Main road E-W: y=9-10, full width
- General Store: x=2-9, y=2-6 (entrance x=5-6, y=2)
- Animal Shop: x=18-25, y=2-6 (entrance x=22-23, y=2)
- Blacksmith: x=20-25, y=13-16 (entrance x=22-23, y=13)
- House 1 (doc): x=2-6, y=13-15
- House 2 (fisher): x=8-12, y=13-15
- Restaurant: x=14-18, y=16-18
- Park/pond: x=2-9, y=17-20 (water at x=4-6, y=18-19)
- Beach exit: x=26-27, y=11-12

## Farm Layout (32×24)
- Player house area: upper portion
- Farm plots: central area
- Barn area: check generate_farm for exact positions
- South exit to town: x=12-15, y=0

## Beach Layout (20×14)
- Sand/water areas, dock area
- North exit to town: top edge

## Coordinate System
- y=0 is NORTH (back wall / top of screen)
- y=max is SOUTH (bottom / exits usually)
- grid_to_world_center(x, y) converts to pixel coords

## Remapping Strategy
For each OOB coordinate, remap to a sensible in-bounds location:
- Town NPCs near shops → place near shop entrance or on adjacent path
- Town NPCs at "home" → place in House 1 or House 2 area, or on nearby path
- Town NPCs at "lunch" → place near restaurant (x=14-18, y=16-18) or plaza
- Farm NPCs → place within 0-31,0-23, near farm plots or paths
- Beach NPCs → place on sand areas within 0-19,0-13
- MineEntrance NPCs → place within 0-13,0-11, near entrance

## Quantitative Targets
- 0 out-of-bounds coordinates remaining after fix
- ALL 10 NPCs must have valid schedules
- Every ScheduleEntry coordinate must satisfy: x < map_width AND y < map_height

## Validation
Run this Python check after editing:
```python
import re
bounds = {'Town':(28,22),'Farm':(32,24),'Beach':(20,14),'Forest':(22,18),'MineEntrance':(14,12)}
with open('src/npcs/schedules.rs') as f:
    for i,line in enumerate(f,1):
        for name,(w,h) in bounds.items():
            if f'MapId::{name}' in line:
                m = re.search(r'x:\s*(\d+),\s*y:\s*(\d+)', line)
                if m and (int(m.group(1))>=w or int(m.group(2))>=h):
                    print(f"FAIL line {i}: {name} x={m.group(1)} y={m.group(2)}")
```
Expected output: no lines printed.

## Does NOT Handle
- NPC dialogue content
- NPC sprite assignments
- Adding new NPCs or new schedule entries
- Map layout changes

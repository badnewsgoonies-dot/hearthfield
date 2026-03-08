# Art Asset Mapping — Precinct DLC

## Source Packs (LimeZu Modern series, extracted to assets/police/sprites/staging/)
- **interiors/** — Characters (Adam, Alex, Amelia, Bob) + Room Builder + Interior furniture
- **office/** — Office furniture, desks, walls, floors (= precinct interior)
- **exteriors/** — Streets, buildings, vehicles, doors (RPG Maker MV format)
- **ui/** — Portrait generator parts, animated UI elements

## Asset Pipeline
1. Copy relevant 16x16 PNGs from staging/ to assets/police/ organized by use
2. Create atlas spritesheets where needed (characters need walk/idle frames combined)
3. Reference by path in Rust code via Bevy AssetServer

## Map Tileset Assignments (ALL 12 maps)

| Map | Primary Tileset | Secondary | Notes |
|-----|----------------|-----------|-------|
| PrecinctInterior | `office/Room_Builder_Office_16x16.png` + `office/Modern_Office_Shadowless_16x16.png` | `interiors/Interiors_free_16x16.png` | Desks, chairs, evidence shelves, break room |
| PrecinctExterior | `exteriors/A2_Floors_MV_TILESET.png` | `exteriors/Tileset_Cars_MV.png` | Parking lot, patrol cars, entrance |
| Downtown | `exteriors/A2_Floors_MV_TILESET.png` + `exteriors/A4_Walls_MV_TILESET.png` | Building fronts, sidewalks | Shops, alleys, restaurant |
| ResidentialNorth | `exteriors/A2_Floors_MV_TILESET.png` | House tiles, park | Quiet neighborhood |
| ResidentialSouth | `exteriors/A2_Floors_MV_TILESET.png` | Apartment tiles | Apartments, convenience store |
| IndustrialDistrict | `exteriors/A2_Floors_MV_TILESET.png` | Warehouse tiles | Docks, rail yard |
| Highway | `exteriors/A2_Floors_MV_TILESET.png` | Road tiles | Speed trap, gas station |
| ForestPark | `exteriors/A2_Floors_MV_TILESET.png` | Tree/grass tiles | Trails, campsite |
| CrimeSceneTemplate | `exteriors/A2_Floors_MV_TILESET.png` | Crime tape overlay | Dynamic dressing per case |
| Hospital | `interiors/Room_Builder_free_16x16.png` + `interiors/Interiors_free_16x16.png` | Medical furniture | Morgue, ER |
| CourtHouse | `office/Room_Builder_Office_16x16.png` | Formal furniture | Judge chambers, filing |
| PlayerApartment | `interiors/Room_Builder_free_16x16.png` + `interiors/Interiors_free_16x16.png` | Bed, kitchen, personal items | Off-duty hub |

## Character Sprite Assignments (13 characters)

| Character | Base Sprite | Animations Needed |
|-----------|------------|-------------------|
| Player (officer) | `Adam_16x16.png` (blue tint for uniform) | walk 4-dir, idle, run |
| Captain Torres | `Amelia_16x16.png` | walk 4-dir, idle |
| Det. Vasquez | `Alex_16x16.png` | walk 4-dir, idle |
| Officer Chen | `Bob_16x16.png` | walk 4-dir, idle |
| Sgt. Murphy | `Adam_16x16.png` (gray tint for age) | walk 4-dir, idle |
| Mayor Aldridge | `Amelia_16x16.png` (formal palette swap) | idle |
| Dr. Okafor | `Alex_16x16.png` (white coat tint) | idle |
| Rita Gomez | `Amelia_16x16.png` (apron palette) | idle |
| Father Brennan | `Bob_16x16.png` (dark palette) | idle |
| Ghost (tipster) | `Adam_16x16.png` (shadow/dark) | idle |
| Nadia Park | `Amelia_16x16.png` (press badge palette) | idle |
| Marcus Cole | `Bob_16x16.png` (casual palette) | idle |
| Lucia Vega | `Alex_16x16.png` (professional palette) | idle |

## Evidence Item Icons (30 types → use office/interiors singles)
Map to 16x16 singles from `office/4_Modern_Office_singles/16x16/` or generate colored icons.

## HUD / UI Elements
- Use `ui/16x16/` folder for buttons, frames, icons
- Health/fatigue/stress bars: colored rectangles (keep current approach, overlay with UI frame sprites)
- Portrait frames from `ui/16x16/Portrait_Generator/`

## File Organization (target layout)
```
assets/police/
├── tilesets/
│   ├── precinct_interior.png    (from office Room_Builder + Office_Shadowless)
│   ├── precinct_exterior.png    (from exteriors A2_Floors)
│   ├── downtown.png             (from exteriors A2_Floors + A4_Walls)
│   ├── residential.png          (from exteriors A2_Floors)
│   ├── industrial.png           (from exteriors A2_Floors)
│   ├── interior_generic.png     (from interiors Room_Builder_free + Interiors_free)
│   └── road.png                 (from exteriors A2_Floors)
├── characters/
│   ├── player.png               (Adam with uniform palette)
│   ├── captain_torres.png       (Amelia)
│   ├── det_vasquez.png          (Alex)
│   ├── officer_chen.png         (Bob)
│   ├── sgt_murphy.png           (Adam gray)
│   └── ... (8 more NPCs)
├── items/
│   ├── evidence/                (30 evidence icons)
│   └── equipment/               (badge, radio, notebook, etc.)
└── ui/
    ├── buttons.png
    ├── frames.png
    └── icons.png
```

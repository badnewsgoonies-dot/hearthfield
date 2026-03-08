# LimeZu Modern Farm v1.2 — Source Files Needed

Once the pack is on disk, point the orchestrator at the extracted `16x16/` directory.

## Character Animations

| Source File | Target Atlas | Slots |
|---|---|---|
| `Characters_16x16/Farmer_1_Watering_56_frames_16x16.png` | `assets/sprites/character_actions.png` | indices 4-7 |
| `Characters_16x16/Farmer_1_Chopping_40_frames_16x16.png` | `assets/sprites/character_actions.png` | indices 8-11 |
| `Characters_16x16/Farmer_1_Dig_36_frames_16x16.png` | `assets/sprites/character_actions.png` | indices 0-3, 12-15 |
| `Characters_16x16/Farmer_1_Fishing_128_frames_16x16.png` | `assets/sprites/character_actions.png` | indices 16-19 |
| `Characters_16x16/Farmer_1_Harvesting_36_frames_16x16.png` | `assets/sprites/character_actions.png` | indices 20-23 |

## Fish & Fishing Equipment

| Source File | Target Atlas | Slots |
|---|---|---|
| `Single_Files_16x16/Pickup_Items_16x16/Pickup_Fishing_*.png` | `assets/sprites/fishing_atlas.png` | rows 2-5 (indices 16-47) |
| Same fish pickup sprites | `assets/sprites/items_atlas.png` | indices 50-69, 130-138 |

## Item Icons (Inventory)

| Source Folder | Target Atlas | Category | Slots |
|---|---|---|---|
| `Props_and_Buildings_16x16/Rock_*.png` | `items_atlas.png` | Minerals | indices 70-74 |
| `Props_and_Buildings_16x16/Egg_*.png` | `items_atlas.png` | Animal products | indices 40-41 |
| `Props_and_Buildings_16x16/Milk_Can_*.png` | `items_atlas.png` | Animal products | indices 42-43 |
| Gem sprites (if available in pack) | `items_atlas.png` | Gems | indices 75-79 |
| Bar/ingot sprites (if available) | `items_atlas.png` | Bars | indices 80-83 |

## Not in LimeZu (need alternative source)

- **Food icons** (40 items, indices 90-160): cooked dishes, no LimeZu equivalent
- **Artisan goods** (36 items, indices 161-184): jelly, wine, pickles, cheese
- **Tool icons** (tools.png): dead code, not rendered — skip entirely
- **Fishing equipment** (rod, bobber, tackle, indices 0-15): not in pack

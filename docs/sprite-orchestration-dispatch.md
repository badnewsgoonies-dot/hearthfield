# Sprite Integration Orchestration — Dispatch Prompt

Paste this into your Opus orchestrator session. It handles both the Hearthfield base game and Precinct DLC sprite passes.

---

## For the Opus orchestrator:

```
Two sprite integration passes needed. Run them sequentially (base game first, then Precinct).

PASS 1 — HEARTHFIELD BASE GAME (Modern Farm → assets/)

Source: ~/swarm/hearthfield/sprites\ downlaoded\ packages/Modern_Farm_v1_2__1_.zip
Already extracted at: (extract the 16x16/ directory to a working location)
Target repo: ~/swarm/hearthfield/
Mapping spec: Copy the Hearthfield-Sprite-Mapping.md to dlc/police/docs/ or the repo root.

Dispatch 3 workers sequentially (not parallel — they share asset files):

WORKER 1 — Player + Animals (scope: assets/sprites/, src/player/, src/animals/)
Extract from Modern Farm 16x16/:
- Characters_16x16/Farmer_1_16x16.png → assets/sprites/character_spritesheet.png
- All tool animation sheets → assets/sprites/character_actions.png
- Chicken_White, Cow, Sheep_White, Dog_Labrador → assets/sprites/ animal files
Build Bevy-compatible texture atlases. Match existing TextureAtlasLayout configs.
After: boot game, verify player walks 4 directions, animals render.

WORKER 2 — Crops + Items (scope: assets/sprites/, src/farming/, src/data/)
Extract from Modern Farm 16x16/:
- 15 crop growth stage sheets → assets/sprites/plants.png (combined atlas)
- 15 pickup item sprites → assets/sprites/items_atlas.png
Map atlas indices to CropDef entries in src/data/crops.rs.
After: plant a seed, verify growth stages render, harvest shows item sprite.

WORKER 3 — Terrain + Buildings + Animated (scope: assets/tilesets/, assets/sprites/, src/world/)
Extract from Modern Farm 16x16/:
- Autotiles_Godot_16x16.png → assets/tilesets/grass.png
- Tree sprites (4 seasonal variants) → assets/sprites/tree_sprites.png  
- Building sprites → assets/sprites/ building files
- Animated sheets (doors, sprinkler, machines) → assets/sprites/
After: walk between maps, verify terrain tiles, seasonal trees, door animations.

CRITICAL INSTRUCTION FOR ALL WORKERS:
"After each implementation step, audit your work from the player's perspective. Boot the game mentally. Walk through what a player sees. If a sprite isn't visible in-game, it's not done."

Clamp scope after each worker. Run cargo check + cargo test after each.

---

PASS 2 — PRECINCT DLC (Modern Exteriors + Interiors + Office → dlc/police/assets/)

Source packs (already downloaded at ~/swarm/hearthfield/sprites\ downlaoded\ packages/):
- modernexteriors-win.zip (222 MB — streets, buildings, police station, vehicles)
- moderninteriors-win.zip (149 MB — character generator output, furniture, rooms)
- Modern_Office_Revamped_v1.2.zip (2.8 MB — office interiors = precinct)
- modernuserinterface-win.zip (4.3 MB — UI elements)

Target: dlc/police/assets/
Code: dlc/police/src/domains/

Extract all packs. Use ONLY the 16x16 versions.

Dispatch 4 workers sequentially:

WORKER 1 — Player + NPCs (scope: dlc/police/assets/sprites/, dlc/police/src/domains/player/, dlc/police/src/domains/npcs/)
Use the Character Generator output (or extract pre-made characters from Modern Interiors):
- Officer character (player) — idle + walk, 4 directions
- 12 NPC spritesheets: Captain Torres, Det. Vasquez, Officer Chen, Sgt. Murphy, 
  Mayor Aldridge, Dr. Okafor, Rita Gomez, Father Brennan, Ghost, Nadia Park, 
  Marcus Cole, Lucia Vega
Each NPC needs idle + walk minimum (4 dir × 4 frames each = 13 sheets)
After: boot Precinct, player and NPCs render, NPCs walk their schedules.

WORKER 2 — Precinct Interior (scope: dlc/police/assets/, dlc/police/src/domains/precinct/, dlc/police/src/domains/world/)
From Modern Office Revamped 16x16/:
- Office desks, chairs, computers → precinct main room
- Conference table → interrogation room
- Filing cabinets → records room / evidence room
- Break room furniture → break room
- Lobby furniture → precinct entrance
Build the PrecinctInterior map tileset.
After: walk through precinct, see furnished rooms, interact with case board.

WORKER 3 — Exterior Maps (scope: dlc/police/assets/, dlc/police/src/domains/world/)
From Modern Exteriors 16x16/:
- Streets, sidewalks, roads → Downtown, Residential, Highway maps
- Police station exterior → PrecinctExterior
- Buildings, shops → Downtown decoration
- Vehicles (patrol car) → PatrolState rendering
- Trees, parks → ForestPark, Residential areas
Build tilesets for ALL 12 maps (not "some" — ALL 12).
After: transition between maps, verify all exteriors have tilesets.

WORKER 4 — UI (scope: dlc/police/assets/, dlc/police/src/domains/ui/)
From Modern User Interface 16x16/:
- Button sprites, frames, icons → HUD elements
- Bar fills → fatigue bar, stress bar
- Panel backgrounds → case file, inventory, skill tree screens
- Item icons → evidence types (reuse/adapt from available icon sets)
After: HUD shows styled bars and icons, menus have backgrounds, evidence has icons.

CRITICAL: Write a sprite_mapping.json to dlc/police/assets/ that records which source file maps to which game asset. Workers must update this file. This prevents duplicate/orphaned assets.

After ALL workers: run full gate suite including event connectivity and first-60-seconds visual check.
```

---

## Summary of quantitative targets across both passes:

### Hearthfield Base Game:
- 1 player spritesheet (idle + walk + 5 tool anims)
- 15 crop growth stage sheets
- 15 crop item sprites
- 4 animal spritesheets (chicken, cow, sheep, dog)
- 1 terrain autotile set
- 4 seasonal tree variants
- 6+ building/prop sprites
- 6 animated object sheets

### Precinct DLC:
- 13 character spritesheets (1 player + 12 NPCs)
- 12 map tilesets (ALL maps, no gaps)
- Precinct interior furniture set
- Exterior tileset (streets, buildings, vehicles)
- UI element sprites (bars, buttons, panels, icons)
- 30 evidence type icons
- sprite_mapping.json tracking file

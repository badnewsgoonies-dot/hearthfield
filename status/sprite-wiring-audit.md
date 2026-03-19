# Sprite Wiring Audit — Hearthfield

## Render Pattern

The game uses Bevy 0.15's `asset_server.load()` with hardcoded string paths.
Sprite sheets use `TextureAtlasLayout::from_grid()` with per-domain Resource
structs holding `Handle<Image>` + `Handle<TextureAtlasLayout>` pairs.

**Pattern:** Each domain (animals, farming, fishing, mining, npcs, player, ui)
has its own sprite loading system that runs at startup, storing handles in a
domain-specific Resource.

## Hardcoded Asset Paths (40+ load calls)

### Animals (src/animals/mod.rs)
- `sprites/chicken.png` — 384×64, 24×4 frames of 16×16
- `sprites/cow.png` — 1152×192, 24×4 frames of 48×48
- `sprites/sheep.png` — 768×128, 24×4 frames of 32×32
- `sprites/goat.png` — 768×192, 24×4 frames of 32×48
- `sprites/pig.png` — 768×128, 24×4 frames of 32×32
- `sprites/duck.png` — 829×128, 24×4 frames of 32×32
- `sprites/rabbit.png` — Handle stored but no grid spec found
- `sprites/dog.png` — Handle stored but no grid spec found
- `sprites/egg_and_nest.png`, `sprites/milk_and_grass.png`

### Farming (src/farming/mod.rs)
- `sprites/plants.png` — crop sprite atlas
- `tilesets/tilled_dirt.png` — soil tiles
- `sprites/sprinkler.png`, `sprites/sprinkler_anim.png`
- `sprites/scarecrow.png`
- Dynamic: per-crop sprite loading via path construction

### Fishing (src/fishing/mod.rs)
- `sprites/fishing_atlas.png`

### Mining (src/mining/spawning.rs)
- `tilesets/fungus_cave.png`
- `sprites/mining_atlas.png`
- `sprites/mine_enemies.png`

### Player (src/player/spawn.rs)
- `sprites/character_spritesheet.png`
- `sprites/character_actions.png`

### NPCs (src/npcs/)
- Dynamic: `npc_sprite_file(npc_id)` constructs paths
- `emotes.rs`: emote sprite sheet (EMOTE_SHEET_PATH constant)
- `speech_bubbles.rs`: speech bubble sheet

### UI (src/ui/)
- `ui/cursor_default.png`, `ui/cursor_pointing.png`, `ui/cursor_holding.png`
- `ui/premade_dialog_big.png`
- `sprites/items_atlas.png` — item icons in HUD

### Calendar (src/calendar/festivals.rs)
- `sprites/egg_item.png`

## Animation Systems

- `AnimalAnimTimer` — per-animal frame cycling component
- Player animation via `PlayerAnimState` (frame/total_frames in shared)
- Sprinkler animation (separate anim sprite)
- No centralized animation system — each domain handles its own

## Gaps (entities in data, no render code)

Based on asset manifest (172 sprites, 55 unwired):
- Horse, Cat — in AnimalKind enum, no sprite loading code
- Many UI sprites in assets/ui/ not referenced
- 12 source Limezu sprites in _source_limezu/ unused
- Item pickup sprites (38 in assets/sprites/items/) — loaded dynamically

## What a TOML Manifest Would Replace

Currently each domain has a `load_*_sprites()` system with hardcoded paths,
dimensions, and grid layouts. A manifest-driven approach would:

1. Replace hardcoded paths with manifest lookups
2. Store grid dimensions (cols, rows, tile_size) in TOML
3. Validate at build time via build.rs (sprite exists, dimensions match)
4. Generate a SpriteRegistry resource from the manifest
5. Domains query the registry instead of owning their own handle structs

The migration path: add manifest alongside existing code, load from manifest
in new code, gradually replace domain-specific loaders.

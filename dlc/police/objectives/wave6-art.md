# Wave 6: Art Integration — Precinct DLC

## CRITICAL INSTRUCTION
After each step, mentally walk through what a player sees. If the sprite isn't visible on screen when the player is at that location, it's not done.

## Context
16x16 LimeZu sprite assets have been extracted and organized at dlc/police/assets/:
- tilesets/ — precinct_office.png, office_furniture.png, room_builder.png, interiors.png
- characters/ — 13 character PNGs (player + 12 NPCs)
- items/evidence/ — office item singles for evidence icons

## Required Reading
- dlc/police/docs/domains/art-mapping.md (CRITICAL — full mapping of assets to game elements)
- dlc/police/src/domains/world/mod.rs (current colored-rect tile rendering)
- dlc/police/src/domains/player/mod.rs (current player spawn with blue square)
- dlc/police/src/domains/npcs/mod.rs (NPC spawning)
- dlc/police/src/domains/ui/mod.rs (HUD rendering)

## Deliverables

### 1. Tileset Rendering (world domain)
Replace colored rectangles with TextureAtlas sprites for PrecinctInterior map:
- Load precinct_office.png as tileset via AssetServer
- Create TextureAtlasLayout for 16x16 grid
- Map TileKind variants to atlas indices:
  - Floor → office floor tile
  - Wall → office wall tile  
  - Door → door tile
  - Interactable → highlighted tile
- Update spawn_map to use Sprite with TextureAtlas instead of colored Sprite

### 2. Player Sprite (player domain)
Replace blue square with player_officer.png:
- Load character spritesheet (walk frames in 4 directions)
- Create TextureAtlasLayout for the character sheet format (typically 4 cols × 4 rows for RPG Maker format)
- Update spawn_player to use atlas sprite
- Basic animation: switch atlas index based on Facing direction

### 3. NPC Sprites (npcs domain)
Replace colored squares with per-NPC character PNGs:
- Each NPC gets their assigned sprite from art-mapping.md
- Load via AssetServer on NPC spawn
- Set atlas frame based on Facing

### 4. Precinct Object Sprites (precinct domain)
Replace colored squares with office furniture sprites:
- Case Board → desk/board sprite from office_furniture.png
- Coffee Machine → appliance sprite
- Evidence Terminal → computer sprite
- Use nearest available 16x16 sprite from the office tileset singles

### 5. HUD Polish (ui domain)
- Add simple frame sprites around fatigue/stress bars if UI assets available
- Keep text-based approach for clock, gold, rank (clean and readable)

## Asset Loading Pattern (use this exact pattern)
```rust
// In plugin build():
// Assets load on first use via AssetServer — no preloading needed

// In spawn system:
fn spawn_with_sprite(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture = asset_server.load("police/tilesets/precinct_office.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(16), columns, rows, None, None);
    let layout_handle = atlas_layouts.add(layout);
    
    commands.spawn((
        Sprite::from_atlas_image(texture, TextureAtlas { layout: layout_handle, index: tile_index }),
        Transform::from_xyz(x, y, z),
        // ... other components
    ));
}
```

## Important: Bevy 0.15 Sprite API
- Use `Sprite::from_atlas_image(texture_handle, TextureAtlas { layout, index })` 
- NOT the old `SpriteSheetBundle` (removed in Bevy 0.15)
- NOT `SpriteBundle` with separate atlas component

## Validation
```bash
cargo check -p precinct
cargo test -p precinct
```

## You may edit:
- dlc/police/src/domains/world/mod.rs
- dlc/police/src/domains/player/mod.rs
- dlc/police/src/domains/npcs/mod.rs
- dlc/police/src/domains/precinct/mod.rs
- dlc/police/src/domains/ui/mod.rs

Do NOT edit shared/mod.rs.

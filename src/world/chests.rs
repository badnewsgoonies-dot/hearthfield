//! Chest placement and interaction systems for the world domain.
//!
//! Players can place storage chests on the farm map and interact with them
//! to open a split-view inventory/chest UI.

use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// COMPONENTS & RESOURCES
// ═══════════════════════════════════════════════════════════════════════

/// Marker component to tag chest entities for queries.
#[derive(Component, Debug)]
pub struct ChestMarker;

/// Cached sprite atlas handles for chest entities.
#[derive(Resource, Default)]
pub struct ChestSpriteData {
    pub loaded: bool,
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
}

/// Resource that tracks whether a chest is currently open and which entity.
/// When `entity` is `Some(e)`, the chest UI overlay is shown and player
/// movement is blocked.
#[derive(Resource, Debug, Default)]
pub struct ChestInteraction {
    pub entity: Option<Entity>,
}

impl ChestInteraction {
    pub fn is_open(&self) -> bool {
        self.entity.is_some()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// CHEST SPRITE LOADING
// ═══════════════════════════════════════════════════════════════════════

/// Loads the chest sprite atlas on first entry into Playing state.
pub fn load_chest_sprites(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut sprite_data: ResMut<ChestSpriteData>,
) {
    if sprite_data.loaded {
        return;
    }

    // chest.png: 240x96, 5 cols x 2 rows of 48x48 frames
    sprite_data.image = asset_server.load("sprites/chest.png");
    sprite_data.layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(48, 48),
        5,
        2,
        None,
        None,
    ));

    sprite_data.loaded = true;
}

// ═══════════════════════════════════════════════════════════════════════
// CHEST PLACEMENT
// ═══════════════════════════════════════════════════════════════════════

/// Listens for the C key while the player has a "chest" item in their
/// selected hotbar slot. Places a chest entity on the target tile
/// (player position + facing direction) if the tile is valid.
pub fn place_chest(
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    player_state: Res<PlayerState>,
    farm_state: Res<FarmState>,
    chest_interaction: Res<ChestInteraction>,
    chest_query: Query<&StorageChest, With<ChestMarker>>,
    player_query: Query<(&Transform, &PlayerMovement), With<Player>>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut toast_events: EventWriter<ToastEvent>,
    chest_sprites: Res<ChestSpriteData>,
) {
    // Don't allow placement while a chest is open.
    if chest_interaction.is_open() {
        return;
    }

    if input_blocks.is_blocked() {
        return;
    }

    if !player_input.open_crafting {
        return;
    }

    // Must be on the farm map.
    if player_state.current_map != MapId::Farm {
        return;
    }

    // Check if selected hotbar slot contains a "chest" item.
    let selected = inventory.selected_slot;
    let has_chest_item = if let Some(ref slot) = inventory.slots.get(selected).and_then(|s| s.as_ref()) {
        slot.item_id == "chest"
    } else {
        false
    };

    if !has_chest_item {
        return;
    }

    // Get player position and facing direction.
    let Ok((transform, movement)) = player_query.get_single() else {
        return;
    };

    let pg = world_to_grid(transform.translation.x, transform.translation.y);
    let px = pg.x;
    let py = pg.y;
    let (dx, dy) = facing_offset(&movement.facing);
    let target_x = px + dx;
    let target_y = py + dy;

    // Check target tile is not occupied by a crop.
    if farm_state.crops.contains_key(&(target_x, target_y)) {
        info!("[Chest] Cannot place chest — tile ({}, {}) has a crop", target_x, target_y);
        return;
    }

    // Check target tile is not occupied by a farm object.
    if farm_state.objects.contains_key(&(target_x, target_y)) {
        info!("[Chest] Cannot place chest — tile ({}, {}) has an object", target_x, target_y);
        return;
    }

    // Check no existing chest at that position.
    for chest in chest_query.iter() {
        if chest.grid_pos == (target_x, target_y) {
            info!("[Chest] Cannot place chest — tile ({}, {}) already has a chest", target_x, target_y);
            return;
        }
    }

    // Remove 1 chest item from inventory.
    let removed = inventory.try_remove("chest", 1);
    if removed == 0 {
        return;
    }

    // Spawn the chest entity with a brown placeholder sprite.
    let world_x = target_x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
    let world_y = target_y as f32 * TILE_SIZE + TILE_SIZE * 0.5;

    let chest_sprite = if chest_sprites.loaded {
        let mut s = Sprite::from_atlas_image(
            chest_sprites.image.clone(),
            TextureAtlas {
                layout: chest_sprites.layout.clone(),
                index: 0,
            },
        );
        s.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
        s
    } else {
        Sprite {
            color: Color::srgb(0.55, 0.35, 0.15),
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            ..default()
        }
    };

    commands.spawn((
        ChestMarker,
        StorageChest::new(36, target_x, target_y),
        chest_sprite,
        Transform::from_translation(Vec3::new(world_x, world_y, Z_ENTITY_BASE)),
        LogicalPosition(Vec2::new(world_x, world_y)),
        YSorted,
    ));

    sfx_events.send(PlaySfxEvent {
        sfx_id: "place".to_string(),
    });

    toast_events.send(ToastEvent {
        message: "Chest placed!".into(),
        duration_secs: 2.0,
    });

    info!("[Chest] Placed chest at ({}, {})", target_x, target_y);
}

// ═══════════════════════════════════════════════════════════════════════
// CHEST INTERACTION
// ═══════════════════════════════════════════════════════════════════════

/// When the player presses F near a chest entity (within 2 tiles),
/// open the chest by setting ChestInteraction.entity.
pub fn interact_with_chest(
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    mut chest_interaction: ResMut<ChestInteraction>,
    player_query: Query<&Transform, With<Player>>,
    chest_query: Query<(Entity, &Transform), With<ChestMarker>>,
    player_state: Res<PlayerState>,
) {
    // Don't open another chest if one is already open.
    if chest_interaction.is_open() {
        return;
    }

    if input_blocks.is_blocked() {
        return;
    }

    // F key to interact (same as item pickup in the player domain).
    if !player_input.interact {
        return;
    }

    // Must be on the farm map (chests only placed there).
    if player_state.current_map != MapId::Farm {
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let interact_range = TILE_SIZE * 2.0;

    // Find the closest chest within interaction range.
    let mut closest: Option<(Entity, f32)> = None;
    for (entity, chest_transform) in chest_query.iter() {
        let chest_pos = chest_transform.translation.truncate();
        let dist = player_pos.distance(chest_pos);
        if dist <= interact_range {
            if closest.map_or(true, |(_, d)| dist < d) {
                closest = Some((entity, dist));
            }
        }
    }

    if let Some((entity, _)) = closest {
        chest_interaction.entity = Some(entity);
        info!("[Chest] Opened chest {:?}", entity);
    }
}

/// When the chest UI is open and the player presses Escape, close it
/// by clearing the `ChestInteraction` resource.
pub fn close_chest_on_escape(
    player_input: Res<PlayerInput>,
    mut chest_interaction: ResMut<ChestInteraction>,
) {
    if !chest_interaction.is_open() {
        return;
    }

    if player_input.ui_cancel {
        info!("[Chest] Closed chest {:?}", chest_interaction.entity);
        chest_interaction.entity = None;
    }
}

/// Helper to convert a Facing direction into a grid offset.
fn facing_offset(facing: &Facing) -> (i32, i32) {
    match facing {
        Facing::Up => (0, 1),
        Facing::Down => (0, -1),
        Facing::Left => (-1, 0),
        Facing::Right => (1, 0),
    }
}

//! Soil tilling and watering systems.

use bevy::prelude::*;
use crate::shared::*;
use crate::economy::tool_upgrades::{watering_can_area, tool_stamina_cost};
use super::{FarmEntities, SoilTileEntity, grid_to_world};

// ─────────────────────────────────────────────────────────────────────────────
// Hoe — till a dirt tile
// ─────────────────────────────────────────────────────────────────────────────

pub fn handle_hoe_tool_use(
    mut tool_events: EventReader<ToolUseEvent>,
    mut farm_state: ResMut<FarmState>,
    mut farm_entities: ResMut<FarmEntities>,
    mut commands: Commands,
    mut stamina_events: EventWriter<StaminaDrainEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    for event in tool_events.read() {
        if event.tool != ToolKind::Hoe {
            continue;
        }

        let pos = (event.target_x, event.target_y);

        // Hoe can only till Untilled dirt. Tilled/Watered tiles are already done.
        let current = farm_state.soil.get(&pos).copied();
        if current.is_some() {
            // Already tilled or watered — do nothing.
            continue;
        }

        // Till the soil.
        farm_state.soil.insert(pos, SoilState::Tilled);

        // Drain stamina (2 per tile, basic tier; higher tiers could expand range).
        let stamina_cost = match event.tier {
            ToolTier::Basic => 2.0,
            ToolTier::Copper => 1.8,
            ToolTier::Iron => 1.5,
            ToolTier::Gold => 1.2,
            ToolTier::Iridium => 1.0,
        };
        stamina_events.send(StaminaDrainEvent { amount: stamina_cost });

        sfx_events.send(PlaySfxEvent { sfx_id: "hoe".to_string() });

        // Spawn a soil sprite entity if one doesn't already exist.
        spawn_or_update_soil_entity(
            &mut commands,
            &mut farm_entities,
            pos,
            SoilState::Tilled,
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Watering Can — water a tilled tile
// ─────────────────────────────────────────────────────────────────────────────

pub fn handle_watering_can_tool_use(
    mut tool_events: EventReader<ToolUseEvent>,
    mut farm_state: ResMut<FarmState>,
    mut farm_entities: ResMut<FarmEntities>,
    mut commands: Commands,
    mut stamina_events: EventWriter<StaminaDrainEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    player_query: Query<&PlayerMovement, With<Player>>,
) {
    // Determine the direction the player is currently facing.
    // Fall back to Down if the query returns nothing (shouldn't happen in normal play).
    let facing = player_query
        .get_single()
        .map(|pm| pm.facing)
        .unwrap_or(Facing::Down);

    for event in tool_events.read() {
        if event.tool != ToolKind::WateringCan {
            continue;
        }

        // Compute the full set of tiles to water for this tier and facing direction.
        let tiles = watering_can_area(event.tier, event.target_x, event.target_y, facing);

        // Check that at least one of the target tiles is actually Tilled.
        // We still apply the full area, but require a valid tile in the set so the
        // player doesn't waste stamina swinging into empty air.
        let any_tillable = tiles.iter().any(|pos| {
            farm_state.soil.get(pos).copied() == Some(SoilState::Tilled)
        });
        if !any_tillable {
            continue;
        }

        // Water all Tilled tiles in the area.
        for &pos in &tiles {
            if farm_state.soil.get(&pos).copied() == Some(SoilState::Tilled) {
                farm_state.soil.insert(pos, SoilState::Watered);

                // Mark any crop on this tile as watered.
                if let Some(crop) = farm_state.crops.get_mut(&pos) {
                    crop.watered_today = true;
                    crop.days_without_water = 0;
                }

                spawn_or_update_soil_entity(
                    &mut commands,
                    &mut farm_entities,
                    pos,
                    SoilState::Watered,
                );
            }
        }

        // Single stamina drain for the whole action (not per tile).
        let stamina_cost = tool_stamina_cost(2.0, event.tier);
        stamina_events.send(StaminaDrainEvent { amount: stamina_cost });
        sfx_events.send(PlaySfxEvent { sfx_id: "water".to_string() });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Entity helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Spawn a new soil tile entity or update the colour of an existing one.
pub fn spawn_or_update_soil_entity(
    commands: &mut Commands,
    farm_entities: &mut FarmEntities,
    pos: (i32, i32),
    state: SoilState,
) {
    let color = soil_color(state);

    if let Some(&entity) = farm_entities.soil_entities.get(&pos) {
        // Entity already exists — the render sync system will update the colour.
        // We don't need to do anything here because `sync_soil_sprites` reads FarmState.
        let _ = entity;
    } else {
        // Spawn a new sprite entity.
        let translation = grid_to_world(pos.0, pos.1);
        let entity = commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(TILE_SIZE)),
                ..default()
            },
            Transform::from_translation(translation),
            SoilTileEntity { grid_x: pos.0, grid_y: pos.1 },
            SoilTile { state, grid_x: pos.0, grid_y: pos.1 },
        )).id();
        farm_entities.soil_entities.insert(pos, entity);
    }
}

/// Return the placeholder colour for a soil state.
pub fn soil_color(state: SoilState) -> Color {
    match state {
        SoilState::Untilled => Color::srgb(0.55, 0.42, 0.28), // light dirt (shouldn't be rendered)
        SoilState::Tilled   => Color::srgb(0.45, 0.32, 0.20), // medium brown
        SoilState::Watered  => Color::srgb(0.30, 0.22, 0.15), // dark wet soil
    }
}

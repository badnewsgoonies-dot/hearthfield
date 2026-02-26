//! Soil tilling and watering systems.

use bevy::prelude::*;
use crate::shared::*;
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
    calendar: Res<Calendar>,
) {
    for event in tool_events.read() {
        if event.tool != ToolKind::WateringCan {
            continue;
        }

        // Can't water if it's snowing (winter freeze) – but allow during rain.
        // Actually in Stardew / Harvest Moon you can still water, rain just does it for you.
        // We handle rain auto-watering in the DayEndEvent handler.

        let pos = (event.target_x, event.target_y);
        let current = farm_state.soil.get(&pos).copied();

        // Can only water Tilled soil (Untilled dirt has no point).
        if current != Some(SoilState::Tilled) {
            continue;
        }

        // Mark as watered.
        farm_state.soil.insert(pos, SoilState::Watered);

        // Also update the crop's watered_today flag.
        if let Some(crop) = farm_state.crops.get_mut(&pos) {
            crop.watered_today = true;
            crop.days_without_water = 0;
        }

        let stamina_cost = match event.tier {
            ToolTier::Basic => 2.0,
            ToolTier::Copper => 1.8,
            ToolTier::Iron => 1.5,
            ToolTier::Gold => 1.2,
            ToolTier::Iridium => 1.0,
        };
        stamina_events.send(StaminaDrainEvent { amount: stamina_cost });
        sfx_events.send(PlaySfxEvent { sfx_id: "water".to_string() });

        // Higher-tier cans water adjacent tiles too.
        let extra_radius: i32 = match event.tier {
            ToolTier::Basic | ToolTier::Copper => 0,
            ToolTier::Iron => 1,
            ToolTier::Gold | ToolTier::Iridium => 2,
        };
        if extra_radius > 0 {
            water_radius(
                &mut farm_state,
                &mut farm_entities,
                &mut commands,
                event.target_x,
                event.target_y,
                extra_radius,
            );
        } else {
            // Ensure the entity exists / is updated.
            spawn_or_update_soil_entity(
                &mut commands,
                &mut farm_entities,
                pos,
                SoilState::Watered,
            );
        }

        // Suppress unused variable warning
        let _ = calendar.day;
    }
}

/// Water all tilled tiles in a square radius around (cx, cy).
fn water_radius(
    farm_state: &mut FarmState,
    farm_entities: &mut FarmEntities,
    commands: &mut Commands,
    cx: i32,
    cy: i32,
    radius: i32,
) {
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let pos = (cx + dx, cy + dy);
            if let Some(SoilState::Tilled) = farm_state.soil.get(&pos).copied() {
                farm_state.soil.insert(pos, SoilState::Watered);
                if let Some(crop) = farm_state.crops.get_mut(&pos) {
                    crop.watered_today = true;
                    crop.days_without_water = 0;
                }
                spawn_or_update_soil_entity(commands, farm_entities, pos, SoilState::Watered);
            }
        }
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

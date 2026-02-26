//! NPC spawning: instantiate Npc entities for the current map.
//! Respects the schedule system to place NPCs at correct starting positions.

use bevy::prelude::*;
use crate::shared::*;
use super::definitions::{ALL_NPC_IDS, npc_color};
use super::schedule::current_schedule_entry;

/// Component tracking where an NPC should be moving toward.
#[derive(Component, Debug, Clone)]
pub struct NpcMovement {
    pub target_x: f32,
    pub target_y: f32,
    pub speed: f32,
    pub is_moving: bool,
}

impl Default for NpcMovement {
    fn default() -> Self {
        Self {
            target_x: 0.0,
            target_y: 0.0,
            speed: 40.0,
            is_moving: false,
        }
    }
}

/// Component that tags which map this NPC entity belongs to.
/// NPC entities are despawned on MapTransition if this doesn't match new map.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NpcMapTag(pub MapId);

/// Resource tracking which NPCs are currently spawned.
#[derive(Resource, Debug, Default)]
pub struct SpawnedNpcs {
    /// Maps NPC id to entity
    pub entities: std::collections::HashMap<String, Entity>,
}

/// System: on entering Playing state, spawn NPCs for the current map.
pub fn spawn_initial_npcs(
    mut commands: Commands,
    calendar: Res<Calendar>,
    player_state: Res<PlayerState>,
    npc_registry: Res<NpcRegistry>,
    mut spawned: ResMut<SpawnedNpcs>,
) {
    let current_map = player_state.current_map;
    spawn_npcs_for_map(
        &mut commands,
        &calendar,
        current_map,
        &npc_registry,
        &mut spawned,
    );
}

/// Spawn all NPCs that should appear on a given map right now.
pub fn spawn_npcs_for_map(
    commands: &mut Commands,
    calendar: &Calendar,
    map: MapId,
    npc_registry: &NpcRegistry,
    spawned: &mut SpawnedNpcs,
) {
    for &npc_id in ALL_NPC_IDS {
        // Skip if already spawned
        if spawned.entities.contains_key(npc_id) {
            continue;
        }

        let Some(schedule) = npc_registry.schedules.get(npc_id) else {
            continue;
        };
        let Some(npc_def) = npc_registry.npcs.get(npc_id) else {
            continue;
        };

        let entry = current_schedule_entry(calendar, schedule);

        // Only spawn on the correct map
        if entry.map != map {
            continue;
        }

        let world_x = entry.x as f32 * TILE_SIZE;
        let world_y = -(entry.y as f32 * TILE_SIZE);
        let color = npc_color(npc_id);

        let entity = commands.spawn((
            Npc {
                id: npc_id.to_string(),
                name: npc_def.name.clone(),
            },
            NpcMovement {
                target_x: world_x,
                target_y: world_y,
                speed: 40.0,
                is_moving: false,
            },
            NpcMapTag(map),
            Sprite {
                color,
                custom_size: Some(Vec2::new(14.0, 16.0)),
                ..default()
            },
            Transform::from_xyz(world_x, world_y, 1.0),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        )).id();

        spawned.entities.insert(npc_id.to_string(), entity);
    }
}

/// Despawn all NPC entities for a given map (called on map transition out).
pub fn despawn_npcs_for_map(
    commands: &mut Commands,
    map: MapId,
    spawned: &mut SpawnedNpcs,
    npc_map_tags: &Query<(Entity, &NpcMapTag)>,
) {
    let mut to_remove = Vec::new();
    for (entity, tag) in npc_map_tags.iter() {
        if tag.0 == map {
            commands.entity(entity).despawn_recursive();
            to_remove.push(entity);
        }
    }
    // Clean up the tracking map
    spawned.entities.retain(|_, e| !to_remove.contains(e));
}

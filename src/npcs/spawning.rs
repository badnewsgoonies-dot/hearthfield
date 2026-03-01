//! NPC spawning: instantiate Npc entities for the current map.
//! Respects the schedule system to place NPCs at correct starting positions.

use bevy::prelude::*;
use crate::shared::*;
use super::definitions::{ALL_NPC_IDS, npc_color, npc_sprite_file};
use super::schedule::current_schedule_entry;
use super::animation::NpcAnimationTimer;

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

/// Resource holding NPC character spritesheet atlas handles.
///
/// Each NPC has a unique 192×192 spritesheet (4×4 grid of 48×48 frames):
///   Row 0 (indices 0-3):  Walk down
///   Row 1 (indices 4-7):  Walk up
///   Row 2 (indices 8-11): Walk left
///   Row 3 (indices 12-15): Walk right
#[derive(Resource, Default)]
pub struct NpcSpriteData {
    pub loaded: bool,
    /// Shared atlas layout (all NPC sheets are identical 4×4 grids).
    pub layout: Handle<TextureAtlasLayout>,
    /// Per-NPC image handles keyed by NPC id.
    pub images: std::collections::HashMap<String, Handle<Image>>,
}

/// System: on entering Playing state, spawn NPCs for the current map.
pub fn spawn_initial_npcs(
    mut commands: Commands,
    calendar: Res<Calendar>,
    player_state: Res<PlayerState>,
    npc_registry: Res<NpcRegistry>,
    mut spawned: ResMut<SpawnedNpcs>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut npc_sprites: ResMut<NpcSpriteData>,
) {
    let current_map = player_state.current_map;
    spawn_npcs_for_map(
        &mut commands,
        &calendar,
        current_map,
        &npc_registry,
        &mut spawned,
        &asset_server,
        &mut layouts,
        &mut npc_sprites,
    );
}

/// Spawn all NPCs that should appear on a given map right now.
pub fn spawn_npcs_for_map(
    commands: &mut Commands,
    calendar: &Calendar,
    map: MapId,
    npc_registry: &NpcRegistry,
    spawned: &mut SpawnedNpcs,
    asset_server: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
    npc_sprites: &mut NpcSpriteData,
) {
    // Load the shared atlas layout on first use.
    if !npc_sprites.loaded {
        npc_sprites.layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(48, 48),
            4,
            4,
            None,
            None,
        ));
        // Pre-load each NPC's unique spritesheet.
        for &npc_id in ALL_NPC_IDS {
            let path = npc_sprite_file(npc_id);
            let handle = asset_server.load(path);
            npc_sprites.images.insert(npc_id.to_string(), handle);
        }
        npc_sprites.loaded = true;
    }

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

        let wc = grid_to_world_center(entry.x, entry.y);
        let world_x = wc.x;
        let world_y = wc.y;
        let name_color = npc_color(npc_id);

        // Use the NPC's unique spritesheet — no tinting needed.
        let npc_image = npc_sprites.images
            .get(npc_id)
            .cloned()
            .unwrap_or_else(|| asset_server.load(npc_sprite_file(npc_id)));

        // Index 0 = first frame of Walk-down row, used as the default idle pose.
        let sprite = Sprite::from_atlas_image(
            npc_image,
            TextureAtlas {
                layout: npc_sprites.layout.clone(),
                index: 0,
            },
        );

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
            NpcAnimationTimer {
                timer: Timer::from_seconds(0.15, TimerMode::Repeating),
                frame_count: 4,
                current_frame: 0,
            },
            NpcMapTag(map),
            sprite,
            LogicalPosition(Vec2::new(world_x, world_y)),
            Transform::from_xyz(world_x, world_y, Z_ENTITY_BASE),
            YSorted,
            Visibility::default(),
        )).id();

        // Floating name tag above the NPC sprite.
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Text2d::new(npc_def.name.clone()),
                TextFont {
                    font_size: 5.0,
                    ..default()
                },
                TextColor(name_color),
                Transform::from_xyz(0.0, 14.0, 0.1),
            ));
        });

        spawned.entities.insert(npc_id.to_string(), entity);
    }
}

/// Despawn all NPC entities for a given map (called on map transition out).
#[allow(dead_code)]
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

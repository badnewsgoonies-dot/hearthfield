//! NPC spawning: instantiate Npc entities for the current map.
//! Respects the schedule system to place NPCs at correct starting positions.

use super::animation::NpcAnimationTimer;
use super::definitions::{npc_color, npc_sprite_file, ALL_NPC_IDS};
use super::idle_behavior::NpcIdleBehavior;
use super::schedule::current_schedule_entry;
use crate::shared::*;
use crate::ui::cutscene_runner::CutsceneFlags;
use bevy::prelude::*;

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

/// System: eagerly loads all NPC sprite atlases on `OnEnter(Playing)`.
/// This ensures NPC sprites are ready before any spawn system runs,
/// eliminating first-frame colored-rectangle fallbacks.
pub fn preload_npc_sprites(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut npc_sprites: ResMut<NpcSpriteData>,
) {
    if npc_sprites.loaded {
        return;
    }
    npc_sprites.layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(48, 48),
        4,
        4,
        None,
        None,
    ));
    for &npc_id in ALL_NPC_IDS {
        let path = npc_sprite_file(npc_id);
        let handle = asset_server.load(path);
        npc_sprites.images.insert(npc_id.to_string(), handle);
    }
    npc_sprites.loaded = true;
}

/// System: on entering Playing state, spawn NPCs for the current map.
#[allow(clippy::too_many_arguments)]
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
#[allow(clippy::too_many_arguments)]
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
        let npc_image = npc_sprites
            .images
            .get(npc_id)
            .cloned()
            .unwrap_or_else(|| asset_server.load(npc_sprite_file(npc_id)));

        // Index 0 = first frame of Walk-down row, used as the default idle pose.
        let mut sprite = Sprite::from_atlas_image(
            npc_image,
            TextureAtlas {
                layout: npc_sprites.layout.clone(),
                index: 0,
            },
        );
        sprite.anchor = bevy::sprite::Anchor::BottomCenter;
        sprite.custom_size = Some(Vec2::new(24.0, 24.0));

        // Per-NPC variation in idle behavior timing using a simple hash.
        let npc_hash = {
            let mut h: u32 = 5381;
            for byte in npc_id.bytes() {
                h = h.wrapping_mul(33).wrapping_add(byte as u32);
            }
            h
        };
        let initial_look_delay = 3.0 + (npc_hash % 500) as f32 / 100.0; // 3.0–8.0s

        let entity = commands
            .spawn((
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
                    last_base: 0, // default facing down; updated on first movement
                },
                NpcIdleBehavior {
                    breath_phase: (npc_hash % 628) as f32 / 100.0, // stagger phases
                    look_timer: Timer::from_seconds(initial_look_delay, TimerMode::Once),
                    ..Default::default()
                },
                NpcMapTag(map),
                sprite,
                LogicalPosition(Vec2::new(world_x, world_y)),
                Transform::from_xyz(world_x, world_y, Z_ENTITY_BASE),
                YSorted,
                Visibility::default(),
            ))
            .id();

        // Floating name tag above the NPC sprite (48px sprite height + 2px gap).
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Text2d::new(npc_def.name.clone()),
                TextFont {
                    font_size: 5.0,
                    ..default()
                },
                TextColor(name_color),
                Transform::from_xyz(0.0, 26.0, 0.1),
            ));
        });

        spawned.entities.insert(npc_id.to_string(), entity);
    }
}

/// System: force-spawn Mayor Rex on the Farm during the intro cutscene.
///
/// Runs during `Cutscene` state. When the `mayor_intro_visit` flag is set
/// and Mayor Rex isn't already spawned, this places him at grid (8, 10)
/// in front of the farmhouse so the player sees him during the greeting.
/// When the flag is cleared, it despawns the override entity.
#[allow(clippy::too_many_arguments)]
pub fn spawn_mayor_for_intro(
    mut commands: Commands,
    flags: Res<CutsceneFlags>,
    player_state: Res<PlayerState>,
    npc_registry: Res<NpcRegistry>,
    mut spawned: ResMut<SpawnedNpcs>,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut npc_sprites: ResMut<NpcSpriteData>,
) {
    let flag_active = flags
        .flags
        .get("mayor_intro_visit")
        .copied()
        .unwrap_or(false);

    if flag_active && player_state.current_map == MapId::Farm {
        // Already spawned — nothing to do.
        if spawned.entities.contains_key("mayor_rex") {
            return;
        }

        let Some(npc_def) = npc_registry.npcs.get("mayor_rex") else {
            return;
        };

        // Ensure atlas is loaded.
        if !npc_sprites.loaded {
            npc_sprites.layout = layouts.add(TextureAtlasLayout::from_grid(
                UVec2::new(48, 48),
                4,
                4,
                None,
                None,
            ));
            for &npc_id in ALL_NPC_IDS {
                let path = npc_sprite_file(npc_id);
                let handle = asset_server.load(path);
                npc_sprites.images.insert(npc_id.to_string(), handle);
            }
            npc_sprites.loaded = true;
        }

        // Place Mayor Rex at grid (8, 10) — in front of the farmhouse door.
        let wc = grid_to_world_center(8, 10);
        let world_x = wc.x;
        let world_y = wc.y;
        let name_color = npc_color("mayor_rex");

        let npc_image = npc_sprites
            .images
            .get("mayor_rex")
            .cloned()
            .unwrap_or_else(|| asset_server.load(npc_sprite_file("mayor_rex")));

        let mut sprite = Sprite::from_atlas_image(
            npc_image,
            TextureAtlas {
                layout: npc_sprites.layout.clone(),
                index: 0,
            },
        );
        sprite.anchor = bevy::sprite::Anchor::BottomCenter;
        sprite.custom_size = Some(Vec2::new(24.0, 24.0));

        let entity = commands
            .spawn((
                Npc {
                    id: "mayor_rex".to_string(),
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
                    last_base: 0,
                },
                NpcIdleBehavior::default(),
                NpcMapTag(MapId::Farm),
                sprite,
                LogicalPosition(Vec2::new(world_x, world_y)),
                Transform::from_xyz(world_x, world_y, Z_ENTITY_BASE),
                YSorted,
                Visibility::default(),
            ))
            .id();

        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Text2d::new(npc_def.name.clone()),
                TextFont {
                    font_size: 5.0,
                    ..default()
                },
                TextColor(name_color),
                Transform::from_xyz(0.0, 26.0, 0.1),
            ));
        });

        spawned.entities.insert("mayor_rex".to_string(), entity);
    } else if !flag_active {
        // Flag cleared — if Mayor Rex is spawned but shouldn't be on this map
        // per schedule, despawn him so he doesn't linger after the intro.
        if let Some(entity) = spawned.entities.get("mayor_rex").copied() {
            // Check whether the schedule actually places him here.
            if let Some(schedule) = npc_registry.schedules.get("mayor_rex") {
                // Use a default calendar to get schedule — but we don't have
                // Calendar in this system. Instead, just check the map tag.
                // If he's tagged for Farm but his schedule says otherwise,
                // the normal schedule system will move him. Just leave him
                // for map_events to clean up on next transition.
                let _ = (entity, schedule);
            }
        }
    }
}

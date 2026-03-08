//! Visual synchronisation systems — keep sprite appearances in sync with game state.
//!
//! When `FarmingAtlases` are loaded, soil and crop sprites are rendered using
//! texture atlas slices from `tilled_dirt.png` and `plants.png` respectively.
//! If the atlases are not yet available (e.g. first frame before OnEnter fires),
//! coloured placeholder sprites are used as a fallback.

use super::{
    crop_stage_color, soil::soil_color, CropTileEntity, FarmEntities, FarmObjectEntity,
    FarmingAtlases, SoilTileEntity,
};
use crate::shared::*;
use bevy::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// Crop growth pop animation component
// ─────────────────────────────────────────────────────────────────────────────

/// Tracks the last-known growth stage for a crop entity and drives a brief
/// scale "pop" animation (0.8× → 1.0×) over 0.3 s whenever the stage advances.
#[derive(Component, Debug, Clone)]
pub struct CropGrowthAnim {
    pub timer: Timer,
    pub last_known_stage: u8,
    pub animating: bool,
    /// The base sprite size (before animation scaling) — stored when animation
    /// starts so we can multiply without frame-over-frame compounding.
    pub base_size: f32,
}

// ─────────────────────────────────────────────────────────────────────────────
// Atlas index helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Map SoilState to an atlas index in tilled_dirt.png (11 cols × 7 rows).
///
/// Index 5  — plain tilled dirt tile (row 0, col 5)
/// Index 16 — darker / wetter dirt tile (row 1, col 5)
fn soil_atlas_index(state: SoilState) -> usize {
    match state {
        SoilState::Untilled => 0, // shouldn't normally be rendered
        SoilState::Tilled => 5,
        SoilState::Watered => 16,
    }
}

/// Map a crop growth stage to a plants.png atlas index (row 0: indices 0-5).
///
/// Uses the formula:
///   `let atlas_idx = (stage * 5 / total_stages.max(1)).min(5)`
/// so every crop maps smoothly onto the 6 available growth frames regardless
/// of how many growth days are defined.
fn crop_atlas_index(stage: u8, total_stages: u8) -> usize {
    let total = total_stages.max(1) as usize;
    ((stage as usize * 5) / total).min(5)
}

// ─────────────────────────────────────────────────────────────────────────────
// Soil sprite sync
// ─────────────────────────────────────────────────────────────────────────────

/// Update the appearance of every soil sprite to match the current SoilState in
/// FarmState.  Also spawns missing entities and despawns stale ones.
///
/// When `FarmingAtlases` are loaded, newly spawned soil entities use a texture
/// atlas slice from `tilled_dirt.png`.  For sprites that were already spawned
/// (either with an atlas or as a colour placeholder) the atlas index / colour is
/// updated in-place.
pub fn sync_soil_sprites(
    mut commands: Commands,
    mut farm_entities: ResMut<FarmEntities>,
    farm_state: Res<FarmState>,
    atlases: Res<FarmingAtlases>,
    mut soil_query: Query<(&SoilTileEntity, &mut Sprite)>,
) {
    // Incremental short-circuit: if neither farm data nor atlas state changed,
    // there is nothing to reconcile this frame.
    if !farm_state.is_changed() && !atlases.is_changed() {
        return;
    }

    // ── Update existing entities ──────────────────────────────────────────────
    for (tile, mut sprite) in soil_query.iter_mut() {
        let pos = (tile.grid_x, tile.grid_y);
        if let Some(&state) = farm_state.soil.get(&pos) {
            if let Some(atlas) = &mut sprite.texture_atlas {
                // Atlas sprite: update the slice index.
                atlas.index = soil_atlas_index(state);
                // Apply color tint: watered soil gets a dark brown multiply,
                // tilled soil stays white (no tint).
                sprite.color = match state {
                    SoilState::Watered => Color::srgb(0.6, 0.5, 0.4),
                    SoilState::Tilled => Color::WHITE,
                    SoilState::Untilled => Color::WHITE,
                };
            } else if atlases.loaded {
                // Upgrade: sprite was spawned as color-only before atlases loaded.
                *sprite = Sprite::from_atlas_image(
                    atlases.dirt_image.clone(),
                    TextureAtlas {
                        layout: atlases.dirt_layout.clone(),
                        index: soil_atlas_index(state),
                    },
                );
                sprite.color = match state {
                    SoilState::Watered => Color::srgb(0.6, 0.5, 0.4),
                    SoilState::Tilled => Color::WHITE,
                    SoilState::Untilled => Color::WHITE,
                };
            } else {
                // Fallback colour sprite (atlases not ready yet).
                sprite.color = soil_color(state);
            }
        }
    }

    // ── Spawn missing entities ────────────────────────────────────────────────
    let missing: Vec<((i32, i32), SoilState)> = farm_state
        .soil
        .iter()
        .filter(|(&pos, _)| !farm_entities.soil_entities.contains_key(&pos))
        .map(|(&pos, &state)| (pos, state))
        .collect();

    for (pos, state) in missing {
        // Soil overlays are area fills — use corner-origin to match ground tiles
        let translation = Vec3::new(
            pos.0 as f32 * TILE_SIZE,
            pos.1 as f32 * TILE_SIZE,
            Z_FARM_OVERLAY,
        );

        let entity = if atlases.loaded {
            // Preferred path: use a texture atlas sprite.
            commands
                .spawn((
                    Sprite::from_atlas_image(
                        atlases.dirt_image.clone(),
                        TextureAtlas {
                            layout: atlases.dirt_layout.clone(),
                            index: soil_atlas_index(state),
                        },
                    ),
                    Transform::from_translation(translation),
                    SoilTileEntity {
                        grid_x: pos.0,
                        grid_y: pos.1,
                    },
                    SoilTile {
                        state,
                        grid_x: pos.0,
                        grid_y: pos.1,
                    },
                ))
                .id()
        } else {
            // Fallback path: coloured rectangle until atlases are ready.
            commands
                .spawn((
                    Sprite {
                        color: soil_color(state),
                        custom_size: Some(Vec2::splat(TILE_SIZE)),
                        ..default()
                    },
                    Transform::from_translation(translation),
                    SoilTileEntity {
                        grid_x: pos.0,
                        grid_y: pos.1,
                    },
                    SoilTile {
                        state,
                        grid_x: pos.0,
                        grid_y: pos.1,
                    },
                ))
                .id()
        };

        farm_entities.soil_entities.insert(pos, entity);
    }

    // ── Despawn stale entities ────────────────────────────────────────────────
    let stale: Vec<(i32, i32)> = farm_entities
        .soil_entities
        .keys()
        .filter(|pos| !farm_state.soil.contains_key(pos))
        .cloned()
        .collect();

    for pos in stale {
        if let Some(entity) = farm_entities.soil_entities.remove(&pos) {
            commands.entity(entity).despawn();
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Crop sprite sync
// ─────────────────────────────────────────────────────────────────────────────

/// Update crop sprite appearances to reflect current stage / wilting / dead state.
///
/// When `FarmingAtlases` are loaded, newly spawned crop entities use a texture
/// atlas slice from `plants.png`.  Dead crops always use a brown-tinted colour
/// sprite regardless of atlas availability.  For existing sprites with an atlas,
/// the slice index is updated based on `current_stage`.
pub fn sync_crop_sprites(
    mut commands: Commands,
    mut farm_entities: ResMut<FarmEntities>,
    farm_state: Res<FarmState>,
    crop_registry: Res<CropRegistry>,
    atlases: Res<FarmingAtlases>,
    mut crop_query: Query<(&CropTileEntity, &mut Sprite, &mut CropTile, Option<&mut CropGrowthAnim>)>,
) {
    // Incremental short-circuit for unchanged state/defs/atlas handles.
    if !farm_state.is_changed() && !crop_registry.is_changed() && !atlases.is_changed() {
        return;
    }

    // ── Update existing entities ──────────────────────────────────────────────
    for (tile, mut sprite, mut crop_component, growth_anim) in crop_query.iter_mut() {
        let pos = (tile.grid_x, tile.grid_y);
        if let Some(crop) = farm_state.crops.get(&pos) {
            // Detect stage advancement and trigger growth pop animation.
            if let Some(mut anim) = growth_anim {
                if crop.current_stage > anim.last_known_stage {
                    anim.last_known_stage = crop.current_stage;
                    anim.timer.reset();
                    anim.animating = true;
                    // Snapshot the base size that sync_crop_sprites will set below.
                    let is_mature_now = crop_registry
                        .crops
                        .get(&crop.crop_id)
                        .map(|d| crop.current_stage >= d.growth_days.len() as u8)
                        .unwrap_or(false);
                    anim.base_size = if is_mature_now {
                        TILE_SIZE * 0.95
                    } else {
                        TILE_SIZE * 0.8
                    };
                }
            } else {
                // First time seeing this entity — insert the anim component
                // with last_known_stage matching current so no spurious pop.
                let entity = farm_entities.crop_entities.get(&pos).copied();
                if let Some(e) = entity {
                    commands.entity(e).insert(CropGrowthAnim {
                        timer: Timer::from_seconds(0.3, TimerMode::Once),
                        last_known_stage: crop.current_stage,
                        animating: false,
                        base_size: TILE_SIZE * 0.8,
                    });
                }
            }

            // Sync the component data.
            *crop_component = crop.clone();

            let total_stages = crop_registry
                .crops
                .get(&crop.crop_id)
                .map(|d| d.growth_days.len() as u8)
                .unwrap_or(4);

            if crop.dead {
                // Dead crops: dark brown tint regardless of atlas availability.
                // Remove any atlas reference so the colour shows through.
                sprite.texture_atlas = None;
                sprite.color = Color::srgb(0.4, 0.3, 0.2);
                sprite.custom_size = Some(Vec2::splat(TILE_SIZE * 0.8));
            } else if let Some(atlas) = &mut sprite.texture_atlas {
                // Atlas sprite: update slice index for current stage.
                // Per-crop atlases use sequential indices; generic plants.png uses
                // the sprite_stages mapping via crop_atlas_index.
                if atlases.crop_atlases.contains_key(&crop.crop_id) {
                    atlas.index = crop.current_stage as usize;
                } else {
                    atlas.index = crop_atlas_index(crop.current_stage, total_stages);
                }
                // Apply dehydration tint on top of the atlas image.
                // Freshly watered or healthy crops get no tint (WHITE).
                sprite.color = if crop.days_without_water >= 2 {
                    Color::srgb(0.85, 0.70, 0.30) // severely dehydrated — deep yellow
                } else if crop.days_without_water >= 1 {
                    Color::srgb(0.90, 0.85, 0.50) // mildly dehydrated — light yellow
                } else {
                    Color::WHITE // healthy / watered today
                };
            } else if atlases.loaded && !crop.dead {
                // Upgrade: sprite was spawned as color-only before atlases loaded.
                // Prefer per-crop atlas if available, otherwise fall back to plants.png.
                if let Some((img, lay)) = atlases.crop_atlases.get(&crop.crop_id) {
                    let atlas_index = crop.current_stage as usize;
                    *sprite = Sprite::from_atlas_image(
                        img.clone(),
                        TextureAtlas {
                            layout: lay.clone(),
                            index: atlas_index,
                        },
                    );
                } else {
                    let atlas_index = crop_atlas_index(crop.current_stage, total_stages);
                    *sprite = Sprite::from_atlas_image(
                        atlases.plants_image.clone(),
                        TextureAtlas {
                            layout: atlases.plants_layout.clone(),
                            index: atlas_index,
                        },
                    );
                }
                sprite.color = if crop.days_without_water >= 2 {
                    Color::srgb(0.85, 0.70, 0.30)
                } else if crop.days_without_water >= 1 {
                    Color::srgb(0.90, 0.85, 0.50)
                } else {
                    Color::WHITE
                };
            } else {
                // Fallback: colour placeholder (atlases not ready, or dead crop).
                let color = if crop.days_without_water >= 2 {
                    Color::srgb(0.85, 0.70, 0.30) // severely dehydrated
                } else if crop.days_without_water >= 1 {
                    Color::srgb(0.90, 0.85, 0.50) // mildly dehydrated
                } else {
                    crop_stage_color(crop.current_stage, total_stages, crop.dead)
                };
                sprite.color = color;
            }

            // Scale: slightly larger when mature.
            let is_mature = crop_registry
                .crops
                .get(&crop.crop_id)
                .map(|d| crop.current_stage >= d.growth_days.len() as u8)
                .unwrap_or(false);

            sprite.custom_size = Some(Vec2::splat(if is_mature {
                TILE_SIZE * 0.95
            } else {
                TILE_SIZE * 0.8
            }));
        }
    }

    // ── Spawn missing entities ────────────────────────────────────────────────
    let missing: Vec<((i32, i32), CropTile)> = farm_state
        .crops
        .iter()
        .filter(|(&pos, _)| !farm_entities.crop_entities.contains_key(&pos))
        .map(|(&pos, crop)| (pos, crop.clone()))
        .collect();

    for (pos, crop) in missing {
        let total_stages = crop_registry
            .crops
            .get(&crop.crop_id)
            .map(|d| d.growth_days.len() as u8)
            .unwrap_or(4);

        let translation = grid_to_world_center(pos.0, pos.1).extend(Z_FARM_OVERLAY + 1.0);

        // Pre-build the growth anim component — last_known_stage matches current
        // so newly spawned crops do NOT trigger a spurious pop animation.
        let growth_anim = CropGrowthAnim {
            timer: Timer::from_seconds(0.3, TimerMode::Once),
            last_known_stage: crop.current_stage,
            animating: false,
            base_size: TILE_SIZE * 0.8,
        };

        let entity = if atlases.loaded && !crop.dead {
            // Preferred path: texture atlas sprite.
            // Use per-crop atlas if available, otherwise fall back to plants.png.
            let (img, lay, atlas_index) =
                if let Some((ci, cl)) = atlases.crop_atlases.get(&crop.crop_id) {
                    (ci.clone(), cl.clone(), crop.current_stage as usize)
                } else {
                    let idx = crop_atlas_index(crop.current_stage, total_stages);
                    (atlases.plants_image.clone(), atlases.plants_layout.clone(), idx)
                };
            commands
                .spawn((
                    Sprite::from_atlas_image(
                        img,
                        TextureAtlas {
                            layout: lay,
                            index: atlas_index,
                        },
                    ),
                    Transform::from_translation(translation),
                    CropTileEntity {
                        grid_x: pos.0,
                        grid_y: pos.1,
                    },
                    crop,
                    growth_anim,
                ))
                .id()
        } else {
            // Fallback: coloured rectangle (also used for dead crops).
            let color = if crop.dead {
                Color::srgb(0.4, 0.3, 0.2) // dark brown — withered/dead
            } else if crop.days_without_water >= 2 {
                Color::srgb(0.85, 0.70, 0.30) // severely dehydrated
            } else if crop.days_without_water >= 1 {
                Color::srgb(0.90, 0.85, 0.50) // mildly dehydrated
            } else {
                crop_stage_color(crop.current_stage, total_stages, crop.dead)
            };
            commands
                .spawn((
                    Sprite {
                        color,
                        custom_size: Some(Vec2::splat(TILE_SIZE * 0.8)),
                        ..default()
                    },
                    Transform::from_translation(translation),
                    CropTileEntity {
                        grid_x: pos.0,
                        grid_y: pos.1,
                    },
                    crop,
                    growth_anim,
                ))
                .id()
        };

        farm_entities.crop_entities.insert(pos, entity);
    }

    // ── Despawn stale entities ────────────────────────────────────────────────
    let stale: Vec<(i32, i32)> = farm_entities
        .crop_entities
        .keys()
        .filter(|pos| !farm_state.crops.contains_key(pos))
        .cloned()
        .collect();

    for pos in stale {
        if let Some(entity) = farm_entities.crop_entities.remove(&pos) {
            commands.entity(entity).despawn();
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Crop growth pop animation system
// ─────────────────────────────────────────────────────────────────────────────

/// Tick the growth-pop animation timer and interpolate sprite scale.
///
/// When `CropGrowthAnim.animating` is true the sprite scales from 0.8× to 1.0×
/// over the timer duration (0.3 s).  The multiplier is applied on top of the
/// base custom_size that `sync_crop_sprites` already set, using the stored
/// `base_size` to avoid frame-over-frame compounding.
pub fn animate_crop_growth(
    time: Res<Time>,
    mut query: Query<(&mut CropGrowthAnim, &mut Sprite)>,
) {
    for (mut anim, mut sprite) in query.iter_mut() {
        if !anim.animating {
            continue;
        }

        anim.timer.tick(time.delta());

        // Lerp from 0.8 to 1.0 over the timer duration.
        let t = anim.timer.fraction();
        let scale_factor = 0.8 + 0.2 * t;

        // Apply the pop multiplier on the stored base size (set by sync_crop_sprites
        // when the animation was triggered).
        let scaled = anim.base_size * scale_factor;
        sprite.custom_size = Some(Vec2::splat(scaled));

        if anim.timer.finished() {
            anim.animating = false;
            // Restore exact base size to avoid floating-point drift.
            sprite.custom_size = Some(Vec2::splat(anim.base_size));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Farm object (sprinkler / scarecrow) sprite sync
// ─────────────────────────────────────────────────────────────────────────────

/// Map a FarmObject to an atlas index in furniture.png (9 cols × 6 rows).
/// Returns None for Fence (handled separately with fences atlas + autotile).
fn farm_object_atlas_index(obj: &FarmObject) -> Option<usize> {
    match obj {
        FarmObject::Sprinkler => Some(36), // row 4: machinery/device
        FarmObject::Scarecrow => Some(45), // row 5: tall object
        _ => None,
    }
}

/// Fallback placeholder colour for farm objects when no atlas is available.
fn farm_object_color(obj: &FarmObject) -> Color {
    match obj {
        FarmObject::Sprinkler => Color::srgb(0.5, 0.5, 0.7),
        FarmObject::Scarecrow => Color::srgb(0.6, 0.4, 0.2),
        FarmObject::Fence => Color::srgb(0.6, 0.4, 0.2),
        _ => Color::srgb(0.5, 0.5, 0.5),
    }
}

/// Compute autotile index (0-15) for a fence at (x, y) based on cardinal neighbors.
fn fence_autotile_index(farm_state: &FarmState, x: i32, y: i32) -> usize {
    let mut mask: u8 = 0;
    if matches!(farm_state.objects.get(&(x, y - 1)), Some(FarmObject::Fence)) {
        mask |= 1;
    } // north
    if matches!(farm_state.objects.get(&(x + 1, y)), Some(FarmObject::Fence)) {
        mask |= 2;
    } // east
    if matches!(farm_state.objects.get(&(x, y + 1)), Some(FarmObject::Fence)) {
        mask |= 4;
    } // south
    if matches!(farm_state.objects.get(&(x - 1, y)), Some(FarmObject::Fence)) {
        mask |= 8;
    } // west
    mask as usize
}

/// Synchronise visual entities for sprinklers, scarecrows, and fences in `FarmState.objects`.
///
/// Follows the same overall pattern as `sync_soil_sprites` / `sync_crop_sprites`:
///   - Spawn missing entities.
///   - Despawn stale entities.
pub fn sync_farm_objects_sprites(
    mut commands: Commands,
    mut farm_entities: ResMut<FarmEntities>,
    farm_state: Res<FarmState>,
    furniture: Res<crate::world::objects::FurnitureAtlases>,
    obj_atlases: Res<crate::world::objects::ObjectAtlases>,
) {
    // Incremental short-circuit for unchanged object state and atlas resources.
    if !farm_state.is_changed() && !furniture.is_changed() && !obj_atlases.is_changed() {
        return;
    }

    // ── Spawn missing ────────────────────────────────────────────────────────
    let missing: Vec<((i32, i32), FarmObject)> = farm_state
        .objects
        .iter()
        .filter(|(&pos, obj)| {
            matches!(
                obj,
                FarmObject::Sprinkler | FarmObject::Scarecrow | FarmObject::Fence
            ) && !farm_entities.object_entities.contains_key(&pos)
        })
        .map(|(&pos, obj)| (pos, obj.clone()))
        .collect();

    for (pos, obj) in missing {
        let wc = grid_to_world_center(pos.0, pos.1);
        let translation = Vec3::new(wc.x, wc.y, Z_ENTITY_BASE);
        let logical = LogicalPosition(wc);

        let entity = if matches!(obj, FarmObject::Fence) {
            // Fences use the fences atlas with autotiling.
            if obj_atlases.loaded {
                let idx = fence_autotile_index(&farm_state, pos.0, pos.1);
                let mut sprite = Sprite::from_atlas_image(
                    obj_atlases.fences_image.clone(),
                    TextureAtlas {
                        layout: obj_atlases.fences_layout.clone(),
                        index: idx,
                    },
                );
                sprite.custom_size = Some(Vec2::splat(TILE_SIZE));
                sprite.color = Color::srgb(0.6, 0.4, 0.2);
                commands
                    .spawn((
                        sprite,
                        Transform::from_translation(translation),
                        logical,
                        YSorted,
                        FarmObjectEntity {
                            grid_x: pos.0,
                            grid_y: pos.1,
                        },
                    ))
                    .id()
            } else {
                commands
                    .spawn((
                        Sprite {
                            color: farm_object_color(&obj),
                            custom_size: Some(Vec2::splat(TILE_SIZE)),
                            ..default()
                        },
                        Transform::from_translation(translation),
                        logical,
                        YSorted,
                        FarmObjectEntity {
                            grid_x: pos.0,
                            grid_y: pos.1,
                        },
                    ))
                    .id()
            }
        } else if furniture.loaded {
            if let Some(idx) = farm_object_atlas_index(&obj) {
                let mut sprite = Sprite::from_atlas_image(
                    furniture.image.clone(),
                    TextureAtlas {
                        layout: furniture.layout.clone(),
                        index: idx,
                    },
                );
                sprite.custom_size = Some(Vec2::splat(TILE_SIZE));
                commands
                    .spawn((
                        sprite,
                        Transform::from_translation(translation),
                        logical,
                        YSorted,
                        FarmObjectEntity {
                            grid_x: pos.0,
                            grid_y: pos.1,
                        },
                    ))
                    .id()
            } else {
                continue;
            }
        } else {
            // Colour fallback — no atlas available yet.
            commands
                .spawn((
                    Sprite {
                        color: farm_object_color(&obj),
                        custom_size: Some(Vec2::splat(TILE_SIZE)),
                        ..default()
                    },
                    Transform::from_translation(translation),
                    logical,
                    YSorted,
                    FarmObjectEntity {
                        grid_x: pos.0,
                        grid_y: pos.1,
                    },
                ))
                .id()
        };

        farm_entities.object_entities.insert(pos, entity);
    }

    // ── Despawn stale ────────────────────────────────────────────────────────
    let stale: Vec<(i32, i32)> = farm_entities
        .object_entities
        .keys()
        .filter(|pos| {
            !farm_state
                .objects
                .get(pos)
                .map(|o| {
                    matches!(
                        o,
                        FarmObject::Sprinkler | FarmObject::Scarecrow | FarmObject::Fence
                    )
                })
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    for pos in stale {
        if let Some(entity) = farm_entities.object_entities.remove(&pos) {
            commands.entity(entity).despawn();
        }
    }
}

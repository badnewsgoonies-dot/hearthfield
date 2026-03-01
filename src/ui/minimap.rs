//! Minimap — a small dynamic-texture overlay in the bottom-right corner of the
//! HUD that shows the current map tiles, player position (blinking dot), NPC
//! positions, and map transition zones.

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::render_asset::RenderAssetUsages;
use crate::shared::*;
use crate::world::maps::generate_map;

// ═══════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════

/// Display size of the minimap in UI pixels.
const MINIMAP_DISPLAY: f32 = 120.0;

/// Maximum map dimension we allocate for (largest map is 64×64).
const MAX_MAP: usize = 64;

// ═══════════════════════════════════════════════════════════════════════
// RESOURCES & COMPONENTS
// ═══════════════════════════════════════════════════════════════════════

/// Holds the minimap's dynamic texture handle and current map metadata.
#[derive(Resource)]
pub struct MinimapState {
    pub image_handle: Handle<Image>,
    /// Which map was last rendered (avoids regenerating static tiles each frame).
    pub cached_map: Option<MapId>,
    /// Pre-computed RGBA for static map tiles (no player/NPC overlay).
    pub base_pixels: Vec<[u8; 4]>,
    pub map_width: usize,
    pub map_height: usize,
}

/// Marker for the minimap UI node.
#[derive(Component)]
pub struct MinimapNode;

// ═══════════════════════════════════════════════════════════════════════
// TILE → COLOR
// ═══════════════════════════════════════════════════════════════════════

fn tile_color(kind: TileKind) -> [u8; 4] {
    match kind {
        TileKind::Grass       => [72, 130, 60, 255],
        TileKind::Dirt        => [160, 130, 90, 255],
        TileKind::TilledSoil  => [110, 80, 55, 255],
        TileKind::WateredSoil => [75, 60, 45, 255],
        TileKind::Water       => [55, 100, 180, 255],
        TileKind::Sand        => [210, 195, 150, 255],
        TileKind::Stone       => [140, 140, 145, 255],
        TileKind::WoodFloor   => [150, 120, 75, 255],
        TileKind::Path        => [180, 165, 130, 255],
        TileKind::Bridge      => [130, 105, 65, 255],
        TileKind::Void        => [20, 30, 20, 255],
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// Spawn the minimap resource and UI node. Runs on OnEnter(Playing).
pub fn spawn_minimap(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    // Create a MAX_MAP × MAX_MAP RGBA8 image (will be cropped to actual map size visually).
    let size = Extent3d {
        width: MAX_MAP as u32,
        height: MAX_MAP as u32,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[20, 30, 20, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    // Enable sampling for UI rendering.
    image.sampler = bevy::image::ImageSampler::nearest();
    let handle = images.add(image);

    commands.insert_resource(MinimapState {
        image_handle: handle.clone(),
        cached_map: None,
        base_pixels: vec![[20, 30, 20, 255]; MAX_MAP * MAX_MAP],
        map_width: MAX_MAP,
        map_height: MAX_MAP,
    });

    // UI node: bottom-right corner, semi-transparent border.
    commands.spawn((
        MinimapNode,
        ImageNode {
            image: handle,
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(8.0),
            bottom: Val::Px(68.0), // above hotbar
            width: Val::Px(MINIMAP_DISPLAY),
            height: Val::Px(MINIMAP_DISPLAY),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor(Color::srgba(0.5, 0.45, 0.3, 0.8)),
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
        GlobalZIndex(10),
    ));
}

/// Despawn minimap when leaving Playing state.
pub fn despawn_minimap(
    mut commands: Commands,
    query: Query<Entity, With<MinimapNode>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<MinimapState>();
}

/// Update minimap pixels each frame: regenerate base on map change,
/// overlay player and NPC positions.
pub fn update_minimap(
    time: Res<Time>,
    player_state: Res<PlayerState>,
    player_query: Query<&GridPosition, With<Player>>,
    npc_query: Query<(&Npc, &Transform)>,
    farm_state: Res<FarmState>,
    mut minimap: ResMut<MinimapState>,
    mut images: ResMut<Assets<Image>>,
) {
    let current_map = player_state.current_map;

    // ── Rebuild base tiles on map change ──────────────────────────────────
    if minimap.cached_map != Some(current_map) {
        let map_def = generate_map(current_map);
        let w = map_def.width.min(MAX_MAP);
        let h = map_def.height.min(MAX_MAP);

        minimap.map_width = w;
        minimap.map_height = h;

        // Clear to void
        minimap.base_pixels = vec![[20, 30, 20, 255]; MAX_MAP * MAX_MAP];

        // Fill actual tiles
        for y in 0..h {
            for x in 0..w {
                let kind = map_def.get_tile(x as i32, y as i32);
                minimap.base_pixels[y * MAX_MAP + x] = tile_color(kind);
            }
        }

        // Mark transition zones in a subtle highlight
        for tr in &map_def.transitions {
            let (rx, ry, rw, rh) = tr.from_rect;
            for dy in 0..rh {
                for dx in 0..rw {
                    let tx = (rx + dx) as usize;
                    let ty = (ry + dy) as usize;
                    if tx < w && ty < h {
                        minimap.base_pixels[ty * MAX_MAP + tx] = [255, 220, 100, 255];
                    }
                }
            }
        }

        // Overlay farm crops
        if current_map == MapId::Farm {
            for (&(cx, cy), crop) in &farm_state.crops {
                let ux = cx as usize;
                let uy = cy as usize;
                if ux < w && uy < h {
                    if crop.dead {
                        minimap.base_pixels[uy * MAX_MAP + ux] = [90, 60, 40, 255];
                    } else {
                        let green = (100 + (crop.current_stage as u16 * 30).min(155)) as u8;
                        minimap.base_pixels[uy * MAX_MAP + ux] = [30, green, 20, 255];
                    }
                }
            }
        }

        minimap.cached_map = Some(current_map);
    }

    // ── Write pixels to image ─────────────────────────────────────────────
    let Some(image) = images.get_mut(&minimap.image_handle) else {
        return;
    };

    // Copy base tiles
    let data = &mut image.data;
    for y in 0..MAX_MAP {
        for x in 0..MAX_MAP {
            let offset = (y * MAX_MAP + x) * 4;
            let pixel = minimap.base_pixels[y * MAX_MAP + x];
            data[offset]     = pixel[0];
            data[offset + 1] = pixel[1];
            data[offset + 2] = pixel[2];
            data[offset + 3] = pixel[3];
        }
    }

    let w = minimap.map_width;
    let h = minimap.map_height;

    // ── Overlay NPC positions (small colored dots) ────────────────────────
    for (_npc, tf) in &npc_query {
        let gx = (tf.translation.x / TILE_SIZE).round() as usize;
        let gy = (tf.translation.y / TILE_SIZE).round() as usize;
        if gx < w && gy < h {
            let offset = (gy * MAX_MAP + gx) * 4;
            data[offset]     = 100;
            data[offset + 1] = 200;
            data[offset + 2] = 255;
            data[offset + 3] = 255;
        }
    }

    // ── Overlay player position (blinking white/yellow dot) ───────────────
    if let Ok(grid) = player_query.get_single() {
        let px = grid.x as usize;
        let py = grid.y as usize;
        let blink = (time.elapsed_secs() * 4.0).sin() > 0.0;

        if px < w && py < h {
            let color: [u8; 4] = if blink {
                [255, 255, 255, 255]
            } else {
                [255, 200, 50, 255]
            };

            // 3×3 dot for visibility
            for dy in 0..3i32 {
                for dx in 0..3i32 {
                    let nx = (px as i32 + dx - 1) as usize;
                    let ny = (py as i32 + dy - 1) as usize;
                    if nx < w && ny < h {
                        let offset = (ny * MAX_MAP + nx) * 4;
                        data[offset]     = color[0];
                        data[offset + 1] = color[1];
                        data[offset + 2] = color[2];
                        data[offset + 3] = color[3];
                    }
                }
            }
        }
    }
}

//! Zone rendering — tile spawning, collision map sync, zone decorations, animated tiles.

use bevy::prelude::*;
use crate::shared::*;
use super::maps::generate_zone_map;

/// Marker for animated tiles.
#[derive(Component)]
pub struct AnimatedTile {
    pub anim_type: TileAnimType,
    pub timer: f32,
    pub phase: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum TileAnimType {
    BlinkingLight,
    DepartureBoard,
    Windsock,
    RunwayEdgeLight,
}

#[derive(Component)]
pub struct ZoneDecoration;

pub fn spawn_initial_map(
    mut commands: Commands,
    player_location: Res<PlayerLocation>,
    mut world_map: ResMut<WorldMap>,
    mut collision_map: ResMut<CollisionMap>,
) {
    let (w, h, tiles) = generate_zone_map(player_location.airport, player_location.zone);
    spawn_zone_tiles(&mut commands, &tiles, w, h);
    spawn_animated_tiles(&mut commands, player_location.zone, w, h);
    spawn_zone_decorations(&mut commands, player_location.airport, player_location.zone, w, h);
    world_map.width = w;
    world_map.height = h;
    world_map.tiles = tiles;
    build_collision_map(&world_map, &mut collision_map);
}

#[allow(clippy::needless_range_loop)]
pub fn spawn_zone_tiles(commands: &mut Commands, tiles: &[Vec<TileKind>], w: u32, h: u32) {
    for gy in 0..h as usize {
        for gx in 0..w as usize {
            let kind = tiles[gy][gx];
            let pos = grid_to_world_center(gx as i32, gy as i32);
            let color = zone_tile_color(kind);
            commands.spawn((
                MapTile { kind, grid_x: gx as i32, grid_y: gy as i32 },
                Sprite::from_color(color, Vec2::splat(TILE_SIZE)),
                Transform::from_xyz(pos.x, pos.y, Z_GROUND),
            ));
        }
    }
}

fn zone_tile_color(kind: TileKind) -> Color {
    match kind {
        TileKind::Floor => Color::srgb(0.72, 0.72, 0.75),
        TileKind::Wall => Color::srgb(0.28, 0.28, 0.32),
        TileKind::Runway => Color::srgb(0.15, 0.15, 0.18),
        TileKind::Taxiway => Color::srgb(0.35, 0.38, 0.30),
        TileKind::Grass => Color::srgb(0.22, 0.58, 0.22),
        TileKind::Tarmac => Color::srgb(0.32, 0.32, 0.34),
        TileKind::Water => Color::srgb(0.12, 0.32, 0.68),
        TileKind::Sand => Color::srgb(0.82, 0.72, 0.42),
        TileKind::Snow => Color::srgb(0.92, 0.92, 0.96),
        TileKind::Carpet => Color::srgb(0.55, 0.22, 0.15),
        TileKind::Metal => Color::srgb(0.58, 0.58, 0.62),
        TileKind::Window => Color::srgb(0.50, 0.72, 0.92),
        TileKind::Door => Color::srgb(0.52, 0.32, 0.18),
        TileKind::Void => Color::srgb(0.0, 0.0, 0.0),
    }
}

fn spawn_animated_tiles(commands: &mut Commands, zone: MapZone, w: u32, h: u32) {
    match zone {
        MapZone::Runway => {
            for gy in (0..h).step_by(3) {
                for &gx in &[6u32, 13] {
                    let pos = grid_to_world_center(gx as i32, gy as i32);
                    commands.spawn((
                        AnimatedTile {
                            anim_type: TileAnimType::RunwayEdgeLight,
                            timer: 0.0,
                            phase: (gx as f32 + gy as f32) * 0.3,
                        },
                        Sprite::from_color(Color::srgb(0.9, 0.9, 0.2), Vec2::new(4.0, 4.0)),
                        Transform::from_xyz(pos.x, pos.y, Z_GROUND_DECOR),
                    ));
                }
            }
        }
        MapZone::Terminal => {
            let pos = grid_to_world_center(10, 1);
            commands.spawn((
                AnimatedTile {
                    anim_type: TileAnimType::DepartureBoard,
                    timer: 0.0,
                    phase: 0.0,
                },
                Sprite::from_color(Color::srgb(0.1, 0.1, 0.3), Vec2::new(TILE_SIZE * 3.0, TILE_SIZE)),
                Transform::from_xyz(pos.x, pos.y, Z_GROUND_DECOR),
            ));
            for gx in [5, 8] {
                let pos = grid_to_world_center(gx, 2);
                commands.spawn((
                    AnimatedTile {
                        anim_type: TileAnimType::BlinkingLight,
                        timer: 0.0,
                        phase: gx as f32 * 1.5,
                    },
                    Sprite::from_color(Color::srgb(0.2, 0.8, 0.2), Vec2::new(3.0, 3.0)),
                    Transform::from_xyz(pos.x, pos.y, Z_GROUND_DECOR),
                ));
            }
        }
        MapZone::ControlTower => {
            let pos = grid_to_world_center((w / 2) as i32, 2);
            commands.spawn((
                AnimatedTile {
                    anim_type: TileAnimType::BlinkingLight,
                    timer: 0.0,
                    phase: 0.0,
                },
                Sprite::from_color(Color::srgb(0.1, 0.9, 0.3), Vec2::new(5.0, 5.0)),
                Transform::from_xyz(pos.x, pos.y, Z_GROUND_DECOR),
            ));
        }
        _ => {}
    }
    let _ = (w, h);
}

pub fn animate_tiles(
    time: Res<Time>,
    mut query: Query<(&mut AnimatedTile, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (mut anim, mut sprite) in query.iter_mut() {
        anim.timer += dt;
        let t = anim.timer + anim.phase;
        match anim.anim_type {
            TileAnimType::BlinkingLight => {
                let alpha = ((t * 2.0).sin() * 0.5 + 0.5).clamp(0.3, 1.0);
                sprite.color = Color::srgba(0.2, 0.9, 0.2, alpha);
            }
            TileAnimType::DepartureBoard => {
                let r = 0.1 + ((t * 0.5).sin() * 0.05).abs();
                let g = 0.3 + ((t * 0.8).sin() * 0.1).abs();
                let b = 0.1 + ((t * 1.2).sin() * 0.05).abs();
                sprite.color = Color::srgb(r, g, b);
            }
            TileAnimType::Windsock => {
                let alpha = 0.7 + (t * 3.0).sin() * 0.3;
                sprite.color = Color::srgba(0.9, 0.5, 0.1, alpha);
            }
            TileAnimType::RunwayEdgeLight => {
                let pulse = ((t * 1.5).sin() * 0.5 + 0.5).clamp(0.2, 1.0);
                sprite.color = Color::srgba(0.95, 0.95, 0.3, pulse);
            }
        }
    }
}

fn spawn_zone_decorations(commands: &mut Commands, airport: AirportId, zone: MapZone, w: u32, h: u32) {
    let decor_color = airport_decor_color(airport);

    match zone {
        MapZone::Terminal | MapZone::Lounge => {
            spawn_decor(commands, 1, 1, decor_color, Vec2::new(10.0, 10.0));
            spawn_decor(commands, w as i32 - 2, 1, decor_color, Vec2::new(10.0, 10.0));
            if zone == MapZone::Lounge {
                spawn_decor(commands, (w / 2) as i32, 1, Color::srgb(0.6, 0.4, 0.2), Vec2::new(8.0, 8.0));
            }
        }
        MapZone::Hangar => {
            for gx in (3..w as i32 - 3).step_by(4) {
                spawn_decor(commands, gx, 1, Color::srgb(0.4, 0.4, 0.45), Vec2::new(12.0, 6.0));
            }
        }
        MapZone::Runway => {
            for gy in (2..h as i32 - 2).step_by(4) {
                let pos = grid_to_world_center((w / 2) as i32, gy);
                commands.spawn((
                    ZoneDecoration,
                    Sprite::from_color(Color::srgb(0.9, 0.9, 0.9), Vec2::new(2.0, 8.0)),
                    Transform::from_xyz(pos.x, pos.y, Z_GROUND_DECOR),
                ));
            }
            for offset in -2..=2 {
                let gx = (w / 2) as i32 + offset;
                let pos = grid_to_world_center(gx, 1);
                commands.spawn((
                    ZoneDecoration,
                    Sprite::from_color(Color::srgb(0.95, 0.95, 0.95), Vec2::new(TILE_SIZE * 0.6, 3.0)),
                    Transform::from_xyz(pos.x, pos.y, Z_GROUND_DECOR),
                ));
            }
        }
        _ => {}
    }
}

fn airport_decor_color(airport: AirportId) -> Color {
    match airport {
        AirportId::HomeBase => Color::srgb(0.3, 0.6, 0.3),
        AirportId::Windport => Color::srgb(0.3, 0.5, 0.7),
        AirportId::Frostpeak => Color::srgb(0.85, 0.88, 0.92),
        AirportId::Sunhaven => Color::srgb(0.2, 0.7, 0.3),
        AirportId::Ironforge => Color::srgb(0.5, 0.4, 0.3),
        AirportId::Cloudmere => Color::srgb(0.7, 0.75, 0.85),
        AirportId::Duskhollow => Color::srgb(0.8, 0.65, 0.3),
        AirportId::Stormwatch => Color::srgb(0.4, 0.4, 0.6),
        AirportId::Grandcity => Color::srgb(0.7, 0.6, 0.2),
        AirportId::Skyreach => Color::srgb(0.6, 0.2, 0.6),
    }
}

fn spawn_decor(commands: &mut Commands, gx: i32, gy: i32, color: Color, size: Vec2) {
    let pos = grid_to_world_center(gx, gy);
    commands.spawn((
        ZoneDecoration,
        Sprite::from_color(color, size),
        Transform::from_xyz(pos.x, pos.y, Z_GROUND_DECOR),
    ));
}

pub fn despawn_map(commands: &mut Commands, tiles: &Query<Entity, With<MapTile>>) {
    for entity in tiles.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn sync_collision_map(
    world_map: Res<WorldMap>,
    mut collision_map: ResMut<CollisionMap>,
) {
    if !world_map.is_changed() || world_map.width == 0 { return; }
    build_collision_map(&world_map, &mut collision_map);
}

fn build_collision_map(world_map: &WorldMap, collision_map: &mut CollisionMap) {
    let w = world_map.width as usize;
    let h = world_map.height as usize;
    collision_map.width = world_map.width;
    collision_map.height = world_map.height;
    collision_map.blocked = vec![vec![false; w]; h];
    for gy in 0..h {
        for gx in 0..w {
            collision_map.blocked[gy][gx] = world_map.tiles[gy][gx].is_solid();
        }
    }
    collision_map.initialised = true;
}

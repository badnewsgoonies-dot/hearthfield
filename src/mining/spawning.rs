//! Spawns mine floor entities into the ECS from a FloorBlueprint.

use super::anim::{EnemyIdleAnim, OreShimmer};
use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::floor_gen::{self, FloorBlueprint, MINE_HEIGHT, MINE_WIDTH};
use crate::shared::*;

/// Holds atlas handles for cave environment and rock/ore sprites.
#[derive(Resource, Default)]
pub struct MiningAtlases {
    /// Cave environment tileset (fungus_cave.png — 8 cols x 35 rows)
    pub cave_image: Handle<Image>,
    pub cave_layout: Handle<TextureAtlasLayout>,
    /// Rock/ore sprites (mining_atlas.png — 8 cols x 6 rows)
    pub rock_image: Handle<Image>,
    pub rock_layout: Handle<TextureAtlasLayout>,
    pub loaded: bool,
}

#[derive(Resource, Default)]
pub struct EnemyAtlas {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub loaded: bool,
}

/// Tile indices into fungus_cave.png (8 cols x 35 rows = 280 tiles).
/// Determined by visual inspection of the tileset.
pub mod cave_tiles {
    /// Dark cave floor (row 20, col 4)
    pub const FLOOR: usize = 164;
    /// Slightly different floor tile for checkerboard (row 20, col 5)
    pub const FLOOR_ALT: usize = 165;
    /// Stone/brick wall (row 3, col 0)
    pub const WALL: usize = 24;
    /// Wooden ladder (row 8, col 0)
    pub const LADDER: usize = 64;
    /// Mine exit — distinct tile (row 33, col 0)
    pub const EXIT: usize = 264;
}

pub fn load_mining_atlas(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlases: ResMut<MiningAtlases>,
    mut enemy_atlas: ResMut<EnemyAtlas>,
) {
    if !atlases.loaded {
        // Cave environment tileset: 128x560 = 8 cols x 35 rows of 16x16 tiles
        atlases.cave_image = asset_server.load("tilesets/fungus_cave.png");
        atlases.cave_layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(16, 16),
            8,
            35,
            None,
            None,
        ));
        // Rock/ore sprites: existing 8x6 mining atlas
        atlases.rock_image = asset_server.load("sprites/mining_atlas.png");
        atlases.rock_layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(16, 16),
            8,
            6,
            None,
            None,
        ));
        atlases.loaded = true;
    }
    if !enemy_atlas.loaded {
        enemy_atlas.image = asset_server.load("sprites/mine_enemies.png");
        // 48x16, 3 cols x 1 row: [0] Green Slime, [1] Bat, [2] Rock Crab
        enemy_atlas.layout = layouts.add(TextureAtlasLayout::from_grid(
            UVec2::new(16, 16),
            3,
            1,
            None,
            None,
        ));
        enemy_atlas.loaded = true;
    }
}

/// Fallback color palette for mine visuals (used when atlas not yet loaded).
const FLOOR_COLOR: Color = Color::srgb(0.15, 0.12, 0.18);
const WALL_COLOR: Color = Color::srgb(0.08, 0.06, 0.10);
const ROCK_STONE_COLOR: Color = Color::srgb(0.45, 0.42, 0.40);
const ROCK_COPPER_COLOR: Color = Color::srgb(0.72, 0.45, 0.20);
const ROCK_IRON_COLOR: Color = Color::srgb(0.55, 0.55, 0.60);
const ROCK_GOLD_COLOR: Color = Color::srgb(0.85, 0.75, 0.20);
const ROCK_GEM_COLOR: Color = Color::srgb(0.60, 0.20, 0.80);
const EXIT_COLOR: Color = Color::srgb(0.40, 0.70, 0.40);

/// System: detects when a floor spawn is requested and carries it out.
pub fn spawn_mine_floor(
    mut commands: Commands,
    mut floor_req: ResMut<FloorSpawnRequest>,
    mut active_floor: ResMut<ActiveFloor>,
    existing: Query<Entity, With<MineFloorEntity>>,
    atlases: Res<MiningAtlases>,
    enemy_atlas: Res<EnemyAtlas>,
) {
    if !floor_req.pending {
        return;
    }
    floor_req.pending = false;

    // Despawn everything from the previous floor
    for entity in existing.iter() {
        commands.entity(entity).despawn();
    }

    let floor_num = floor_req.floor;
    let blueprint = floor_gen::generate_floor(floor_num);

    // Spawn floor tiles
    spawn_tiles(&mut commands, &blueprint, &atlases);

    // Spawn rocks
    let rock_count = blueprint.rocks.len();
    spawn_rocks(&mut commands, &blueprint, &atlases);

    // Spawn enemies
    spawn_enemies(&mut commands, &blueprint, &enemy_atlas);

    // Spawn ladder
    spawn_ladder(&mut commands, &blueprint, &atlases);

    // Spawn exit tile (bottom center)
    spawn_exit(&mut commands, &atlases);

    // Update active floor tracking
    *active_floor = ActiveFloor {
        floor: floor_num,
        total_rocks: rock_count,
        rocks_remaining: rock_count,
        rocks_broken_this_floor: 0,
        ladder_revealed: !blueprint.ladder_hidden,
        player_grid_x: blueprint.spawn_pos.0,
        player_grid_y: blueprint.spawn_pos.1,
        spawned: true,
    };
}

fn spawn_tiles(commands: &mut Commands, _blueprint: &FloorBlueprint, atlases: &MiningAtlases) {
    for y in 0..MINE_HEIGHT {
        for x in 0..MINE_WIDTH {
            let is_wall = x == 0 || x == MINE_WIDTH - 1 || y == MINE_HEIGHT - 1;
            let world_x = x as f32 * TILE_SIZE;
            let world_y = y as f32 * TILE_SIZE;

            // Subtle checkerboard: every other tile gets a slight brightness bump
            let checker_bright = (x + y) % 2 == 0;
            let sprite = if atlases.loaded {
                let idx = if is_wall {
                    cave_tiles::WALL
                } else if checker_bright {
                    cave_tiles::FLOOR_ALT
                } else {
                    cave_tiles::FLOOR
                };
                let mut s = Sprite::from_atlas_image(
                    atlases.cave_image.clone(),
                    TextureAtlas {
                        layout: atlases.cave_layout.clone(),
                        index: idx,
                    },
                );
                s.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
                s
            } else {
                let color = if is_wall {
                    WALL_COLOR
                } else if checker_bright {
                    Color::srgb(0.15 + 0.02, 0.12 + 0.02, 0.18 + 0.02)
                } else {
                    FLOOR_COLOR
                };
                Sprite {
                    color,
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    ..default()
                }
            };

            commands.spawn((
                sprite,
                Transform::from_xyz(world_x, world_y, 0.0),
                MineFloorEntity,
                MineTile,
                MineGridPos { x, y },
            ));
        }
    }
}

const ROCK_IRIDIUM_COLOR: Color = Color::srgb(0.40, 0.20, 0.60);

fn rock_color(drop_item: &str) -> Color {
    match drop_item {
        "copper_ore" => ROCK_COPPER_COLOR,
        "iron_ore" => ROCK_IRON_COLOR,
        "gold_ore" => ROCK_GOLD_COLOR,
        "iridium_ore" => ROCK_IRIDIUM_COLOR,
        "diamond" | "ruby" | "emerald" | "quartz" | "amethyst" => ROCK_GEM_COLOR,
        _ => ROCK_STONE_COLOR,
    }
}

fn rock_atlas_index(drop_item: &str) -> usize {
    match drop_item {
        "copper_ore" => 8,
        "iron_ore" => 9,
        "gold_ore" => 11,
        "iridium_ore" => 10,
        "diamond" => 22,
        "ruby" => 19,
        "emerald" => 20,
        "quartz" => 16,
        "amethyst" => 17,
        _ => 0,
    }
}

/// Returns the shimmer color for a valuable ore, or None for plain stone/copper/iron.
fn shimmer_color_for_ore(drop_item: &str) -> Option<Color> {
    match drop_item {
        "gold_ore" => Some(Color::srgb(1.0, 0.9, 0.3)),
        "iridium_ore" => Some(Color::srgb(0.7, 0.5, 1.0)),
        "diamond" => Some(Color::srgb(0.9, 0.95, 1.0)),
        "ruby" => Some(Color::srgb(1.0, 0.3, 0.3)),
        "emerald" => Some(Color::srgb(0.3, 1.0, 0.4)),
        "quartz" => Some(Color::srgb(0.95, 0.92, 0.85)),
        "amethyst" => Some(Color::srgb(0.75, 0.4, 0.95)),
        _ => None,
    }
}

fn spawn_rocks(commands: &mut Commands, blueprint: &FloorBlueprint, atlases: &MiningAtlases) {
    let mut rng = rand::thread_rng();
    for rock_bp in &blueprint.rocks {
        let world_x = rock_bp.x as f32 * TILE_SIZE;
        let world_y = rock_bp.y as f32 * TILE_SIZE;

        let sprite = if atlases.loaded {
            let idx = rock_atlas_index(&rock_bp.drop_item);
            let mut s = Sprite::from_atlas_image(
                atlases.rock_image.clone(),
                TextureAtlas {
                    layout: atlases.rock_layout.clone(),
                    index: idx,
                },
            );
            s.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
            s
        } else {
            let color = rock_color(&rock_bp.drop_item);
            Sprite {
                color,
                custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                ..default()
            }
        };

        let mut entity_cmds = commands.spawn((
            sprite,
            Transform::from_xyz(world_x, world_y, 1.0),
            MineFloorEntity,
            MineGridPos {
                x: rock_bp.x,
                y: rock_bp.y,
            },
            MineRock {
                health: rock_bp.health,
                drop_item: rock_bp.drop_item.clone(),
                drop_quantity: rock_bp.drop_quantity,
            },
        ));

        // Add shimmer to valuable ores (gold, iridium, gems)
        if let Some(color) = shimmer_color_for_ore(&rock_bp.drop_item) {
            let interval: f32 = rng.gen_range(2.5..4.5);
            entity_cmds.insert(OreShimmer {
                timer: Timer::from_seconds(interval, TimerMode::Repeating),
                color,
            });
        }
    }
}

fn spawn_enemies(commands: &mut Commands, blueprint: &FloorBlueprint, enemy_atlas: &EnemyAtlas) {
    let mut rng = rand::thread_rng();
    for enemy_bp in &blueprint.enemies {
        let atlas_index = match enemy_bp.kind {
            MineEnemy::GreenSlime => 0,
            MineEnemy::Bat => 1,
            MineEnemy::RockCrab => 2,
        };
        let world_x = enemy_bp.x as f32 * TILE_SIZE;
        let world_y = enemy_bp.y as f32 * TILE_SIZE;

        // Bats move twice as fast
        let move_interval = match enemy_bp.kind {
            MineEnemy::Bat => 0.5,
            MineEnemy::GreenSlime => 1.0,
            MineEnemy::RockCrab => 1.5,
        };

        // Random initial phase so enemies don't animate in lockstep
        let initial_phase: f32 = rng.gen_range(0.0..std::f32::consts::TAU);

        commands.spawn((
            Sprite::from_atlas_image(
                enemy_atlas.image.clone(),
                TextureAtlas {
                    layout: enemy_atlas.layout.clone(),
                    index: atlas_index,
                },
            ),
            Transform::from_xyz(world_x, world_y, 2.0).with_scale(Vec3::splat(1.2)),
            MineFloorEntity,
            MineGridPos {
                x: enemy_bp.x,
                y: enemy_bp.y,
            },
            MineMonster {
                kind: enemy_bp.kind,
                health: enemy_bp.health,
                max_health: enemy_bp.max_health,
                damage: enemy_bp.damage,
                speed: enemy_bp.speed,
            },
            EnemyMoveTick {
                timer: Timer::from_seconds(move_interval, TimerMode::Repeating),
            },
            EnemyAttackCooldown {
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            },
            EnemyIdleAnim {
                phase: initial_phase,
            },
        ));
    }
}

fn spawn_ladder(commands: &mut Commands, blueprint: &FloorBlueprint, atlases: &MiningAtlases) {
    let world_x = blueprint.ladder_pos.0 as f32 * TILE_SIZE;
    let world_y = blueprint.ladder_pos.1 as f32 * TILE_SIZE;

    let sprite = if atlases.loaded {
        // Hidden ladder shows a floor tile; revealed shows the ladder tile
        let idx = if blueprint.ladder_hidden {
            cave_tiles::FLOOR
        } else {
            cave_tiles::LADDER
        };
        let mut s = Sprite::from_atlas_image(
            atlases.cave_image.clone(),
            TextureAtlas {
                layout: atlases.cave_layout.clone(),
                index: idx,
            },
        );
        s.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
        s
    } else {
        let color = if blueprint.ladder_hidden {
            FLOOR_COLOR
        } else {
            Color::srgb(0.75, 0.60, 0.30)
        };
        Sprite {
            color,
            custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
            ..default()
        }
    };

    commands.spawn((
        sprite,
        Transform::from_xyz(world_x, world_y, 0.5),
        MineFloorEntity,
        MineGridPos {
            x: blueprint.ladder_pos.0,
            y: blueprint.ladder_pos.1,
        },
        MineLadder {
            revealed: !blueprint.ladder_hidden,
        },
    ));
}

fn spawn_exit(commands: &mut Commands, atlases: &MiningAtlases) {
    // Exit is at bottom center (x=12, y=0)
    let exit_x = (MINE_WIDTH / 2) as f32 * TILE_SIZE;
    let exit_y = 0.0;

    let sprite = if atlases.loaded {
        let mut s = Sprite::from_atlas_image(
            atlases.cave_image.clone(),
            TextureAtlas {
                layout: atlases.cave_layout.clone(),
                index: cave_tiles::EXIT,
            },
        );
        s.custom_size = Some(Vec2::new(TILE_SIZE * 2.0, TILE_SIZE));
        s
    } else {
        Sprite {
            color: EXIT_COLOR,
            custom_size: Some(Vec2::new(TILE_SIZE * 2.0, TILE_SIZE)),
            ..default()
        }
    };

    commands.spawn((
        sprite,
        Transform::from_xyz(exit_x, exit_y, 0.5),
        MineFloorEntity,
        MineGridPos {
            x: MINE_WIDTH / 2,
            y: 0,
        },
        MineExit,
    ));
}

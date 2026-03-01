//! Spawns mine floor entities into the ECS from a FloorBlueprint.

use bevy::prelude::*;

use crate::shared::*;
use super::components::*;
use super::floor_gen::{self, FloorBlueprint, MINE_WIDTH, MINE_HEIGHT};

#[derive(Resource, Default)]
pub struct MiningAtlas {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub loaded: bool,
}

pub fn load_mining_atlas(
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut atlas: ResMut<MiningAtlas>,
) {
    if atlas.loaded { return; }
    atlas.image = asset_server.load("sprites/mining_atlas.png");
    atlas.layout = layouts.add(TextureAtlasLayout::from_grid(UVec2::new(16, 16), 8, 6, None, None));
    atlas.loaded = true;
}

/// Color palette for mine visuals.
const FLOOR_COLOR: Color = Color::srgb(0.15, 0.12, 0.18);
const WALL_COLOR: Color = Color::srgb(0.08, 0.06, 0.10);
const ROCK_STONE_COLOR: Color = Color::srgb(0.45, 0.42, 0.40);
const ROCK_COPPER_COLOR: Color = Color::srgb(0.72, 0.45, 0.20);
const ROCK_IRON_COLOR: Color = Color::srgb(0.55, 0.55, 0.60);
const ROCK_GOLD_COLOR: Color = Color::srgb(0.85, 0.75, 0.20);
const ROCK_GEM_COLOR: Color = Color::srgb(0.60, 0.20, 0.80);
const LADDER_COLOR: Color = Color::srgb(0.65, 0.50, 0.25);
const LADDER_HIDDEN_COLOR: Color = Color::srgb(0.15, 0.12, 0.18); // same as floor when hidden
const EXIT_COLOR: Color = Color::srgb(0.30, 0.55, 0.30);
const SLIME_COLOR: Color = Color::srgb(0.20, 0.80, 0.25);
const BAT_COLOR: Color = Color::srgb(0.50, 0.30, 0.20);
const CRAB_COLOR: Color = Color::srgb(0.55, 0.55, 0.55);

/// System: detects when a floor spawn is requested and carries it out.
pub fn spawn_mine_floor(
    mut commands: Commands,
    mut floor_req: ResMut<FloorSpawnRequest>,
    mut active_floor: ResMut<ActiveFloor>,
    existing: Query<Entity, With<MineFloorEntity>>,
    atlas: Res<MiningAtlas>,
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
    spawn_tiles(&mut commands, &blueprint, &atlas);

    // Spawn rocks
    let rock_count = blueprint.rocks.len();
    spawn_rocks(&mut commands, &blueprint, &atlas);

    // Spawn enemies
    spawn_enemies(&mut commands, &blueprint);

    // Spawn ladder
    spawn_ladder(&mut commands, &blueprint, &atlas);

    // Spawn exit tile (bottom center)
    spawn_exit(&mut commands, &atlas);

    // Update active floor tracking
    *active_floor = ActiveFloor {
        floor: floor_num,
        total_rocks: rock_count,
        rocks_remaining: rock_count,
        ladder_revealed: !blueprint.ladder_hidden,
        player_grid_x: blueprint.spawn_pos.0,
        player_grid_y: blueprint.spawn_pos.1,
        spawned: true,
    };
}

fn spawn_tiles(commands: &mut Commands, _blueprint: &FloorBlueprint, atlas: &MiningAtlas) {
    for y in 0..MINE_HEIGHT {
        for x in 0..MINE_WIDTH {
            let is_wall = x == 0 || x == MINE_WIDTH - 1 || y == MINE_HEIGHT - 1;
            let world_x = x as f32 * TILE_SIZE;
            let world_y = y as f32 * TILE_SIZE;

            let sprite = if atlas.loaded {
                let idx = if is_wall { 3 } else { 0 };
                let mut s = Sprite::from_atlas_image(
                    atlas.image.clone(),
                    TextureAtlas { layout: atlas.layout.clone(), index: idx },
                );
                s.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
                s
            } else {
                let color = if is_wall { WALL_COLOR } else { FLOOR_COLOR };
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

fn rock_color(drop_item: &str) -> Color {
    match drop_item {
        "copper_ore" => ROCK_COPPER_COLOR,
        "iron_ore" => ROCK_IRON_COLOR,
        "gold_ore" => ROCK_GOLD_COLOR,
        "diamond" | "ruby" | "emerald" | "quartz" | "amethyst" => ROCK_GEM_COLOR,
        _ => ROCK_STONE_COLOR,
    }
}

fn rock_atlas_index(drop_item: &str) -> usize {
    match drop_item {
        "copper_ore" => 8,
        "iron_ore" => 9,
        "gold_ore" => 11,
        "diamond" => 22,
        "ruby" => 19,
        "emerald" => 20,
        "quartz" => 16,
        "amethyst" => 17,
        _ => 0,
    }
}

fn spawn_rocks(commands: &mut Commands, blueprint: &FloorBlueprint, atlas: &MiningAtlas) {
    for rock_bp in &blueprint.rocks {
        let world_x = rock_bp.x as f32 * TILE_SIZE;
        let world_y = rock_bp.y as f32 * TILE_SIZE;

        let sprite = if atlas.loaded {
            let idx = rock_atlas_index(&rock_bp.drop_item);
            let mut s = Sprite::from_atlas_image(
                atlas.image.clone(),
                TextureAtlas { layout: atlas.layout.clone(), index: idx },
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

        commands.spawn((
            sprite,
            Transform::from_xyz(world_x, world_y, 1.0),
            MineFloorEntity,
            MineGridPos { x: rock_bp.x, y: rock_bp.y },
            MineRock {
                health: rock_bp.health,
                drop_item: rock_bp.drop_item.clone(),
                drop_quantity: rock_bp.drop_quantity,
            },
        ));
    }
}

fn spawn_enemies(commands: &mut Commands, blueprint: &FloorBlueprint) {
    for enemy_bp in &blueprint.enemies {
        let color = match enemy_bp.kind {
            MineEnemy::GreenSlime => SLIME_COLOR,
            MineEnemy::Bat => BAT_COLOR,
            MineEnemy::RockCrab => CRAB_COLOR,
        };
        let world_x = enemy_bp.x as f32 * TILE_SIZE;
        let world_y = enemy_bp.y as f32 * TILE_SIZE;

        // Bats move twice as fast
        let move_interval = match enemy_bp.kind {
            MineEnemy::Bat => 0.5,
            MineEnemy::GreenSlime => 1.0,
            MineEnemy::RockCrab => 1.5,
        };

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(TILE_SIZE - 2.0, TILE_SIZE - 2.0)),
                ..default()
            },
            Transform::from_xyz(world_x, world_y, 2.0),
            MineFloorEntity,
            MineGridPos { x: enemy_bp.x, y: enemy_bp.y },
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
        ));
    }
}

fn spawn_ladder(commands: &mut Commands, blueprint: &FloorBlueprint, atlas: &MiningAtlas) {
    let world_x = blueprint.ladder_pos.0 as f32 * TILE_SIZE;
    let world_y = blueprint.ladder_pos.1 as f32 * TILE_SIZE;

    let sprite = if atlas.loaded && !blueprint.ladder_hidden {
        let mut s = Sprite::from_atlas_image(
            atlas.image.clone(),
            TextureAtlas { layout: atlas.layout.clone(), index: 45 },
        );
        s.custom_size = Some(Vec2::new(TILE_SIZE, TILE_SIZE));
        s
    } else {
        let color = if blueprint.ladder_hidden {
            LADDER_HIDDEN_COLOR
        } else {
            LADDER_COLOR
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

fn spawn_exit(commands: &mut Commands, atlas: &MiningAtlas) {
    // Exit is at bottom center (x=12, y=0)
    let exit_x = (MINE_WIDTH / 2) as f32 * TILE_SIZE;
    let exit_y = 0.0;

    let sprite = if atlas.loaded {
        let mut s = Sprite::from_atlas_image(
            atlas.image.clone(),
            TextureAtlas { layout: atlas.layout.clone(), index: 45 },
        );
        s.custom_size = Some(Vec2::new(TILE_SIZE * 2.0, TILE_SIZE));
        s.color = EXIT_COLOR;
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
        MineGridPos { x: MINE_WIDTH / 2, y: 0 },
        MineExit,
    ));
}

use super::{AnimalAnimTimer, AnimalSpriteData, WanderAi};
use crate::shared::*;
use bevy::prelude::*;
use rand::Rng;

// ─────────────────────────────────────────────────────────────────────────────
// Animal visual configuration
// ─────────────────────────────────────────────────────────────────────────────

pub struct AnimalVisual {
    pub color: Color,
    pub width: f32,
    pub height: f32,
}

pub fn animal_visual(kind: AnimalKind) -> AnimalVisual {
    match kind {
        AnimalKind::Chicken => AnimalVisual {
            color: Color::srgb(0.9, 0.85, 0.3),
            width: 16.0,
            height: 16.0,
        },
        AnimalKind::Cow => AnimalVisual {
            color: Color::srgb(0.85, 0.85, 0.85),
            width: 32.0,
            height: 32.0,
        },
        AnimalKind::Sheep => AnimalVisual {
            color: Color::srgb(0.97, 0.97, 0.97),
            width: 20.0,
            height: 16.0,
        },
        AnimalKind::Goat => AnimalVisual {
            color: Color::srgb(0.85, 0.85, 0.78),
            width: 18.0,
            height: 18.0,
        },
        AnimalKind::Duck => AnimalVisual {
            color: Color::srgb(0.95, 0.88, 0.1),
            width: 10.0,
            height: 10.0,
        },
        AnimalKind::Rabbit => AnimalVisual {
            color: Color::srgb(0.8, 0.8, 0.8),
            width: 8.0,
            height: 10.0,
        },
        AnimalKind::Pig => AnimalVisual {
            color: Color::srgb(0.95, 0.7, 0.73),
            width: 22.0,
            height: 18.0,
        },
        AnimalKind::Horse => AnimalVisual {
            color: Color::srgb(0.35, 0.2, 0.1),
            width: 24.0,
            height: 20.0,
        },
        AnimalKind::Cat => AnimalVisual {
            color: Color::srgb(0.9, 0.55, 0.2),
            width: 12.0,
            height: 12.0,
        },
        AnimalKind::Dog => AnimalVisual {
            color: Color::srgb(0.6, 0.38, 0.18),
            width: 14.0,
            height: 14.0,
        },
    }
}

/// Returns the pen/barn boundaries where this kind of animal wanders.
/// Barn animals (cow, sheep) get the barn pen; coop animals (chicken) get the coop yard;
/// pets (cat, dog) roam a wider farm area.
fn pen_bounds_for(kind: AnimalKind) -> (Vec2, Vec2) {
    match kind {
        AnimalKind::Chicken | AnimalKind::Duck | AnimalKind::Rabbit => (
            Vec2::new(8.0 * TILE_SIZE, 16.0 * TILE_SIZE),
            Vec2::new(12.0 * TILE_SIZE, 18.0 * TILE_SIZE),
        ),
        AnimalKind::Cow | AnimalKind::Sheep | AnimalKind::Goat | AnimalKind::Pig => (
            Vec2::new(3.0 * TILE_SIZE, 16.0 * TILE_SIZE),
            Vec2::new(7.0 * TILE_SIZE, 18.0 * TILE_SIZE),
        ),
        AnimalKind::Horse | AnimalKind::Cat | AnimalKind::Dog => (
            Vec2::new(10.0 * TILE_SIZE, 8.0 * TILE_SIZE),
            Vec2::new(20.0 * TILE_SIZE, 16.0 * TILE_SIZE),
        ),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Item IDs that trigger animal purchase
// ─────────────────────────────────────────────────────────────────────────────

fn item_to_animal(item_id: &str) -> Option<AnimalKind> {
    match item_id {
        "chicken" => Some(AnimalKind::Chicken),
        "cow" => Some(AnimalKind::Cow),
        "sheep" => Some(AnimalKind::Sheep),
        "cat" => Some(AnimalKind::Cat),
        "dog" => Some(AnimalKind::Dog),
        "goat" => Some(AnimalKind::Goat),
        "duck" => Some(AnimalKind::Duck),
        "rabbit" => Some(AnimalKind::Rabbit),
        "pig" => Some(AnimalKind::Pig),
        "horse" => Some(AnimalKind::Horse),
        "animal_chicken" => Some(AnimalKind::Chicken),
        "animal_cow" => Some(AnimalKind::Cow),
        "animal_sheep" => Some(AnimalKind::Sheep),
        "animal_cat" => Some(AnimalKind::Cat),
        "animal_dog" => Some(AnimalKind::Dog),
        "animal_goat" => Some(AnimalKind::Goat),
        "animal_duck" => Some(AnimalKind::Duck),
        "animal_rabbit" => Some(AnimalKind::Rabbit),
        "animal_pig" => Some(AnimalKind::Pig),
        "animal_horse" => Some(AnimalKind::Horse),
        _ => None,
    }
}

fn generate_animal_name(kind: AnimalKind, rng: &mut impl Rng) -> String {
    let chicken_names = ["Penny", "Goldie", "Clucky", "Nugget", "Dottie"];
    let cow_names = ["Bessie", "Daisy", "Rosie", "Mocha", "Cream"];
    let sheep_names = ["Fluffkins", "Woolie", "Cotton", "Misty", "Pearl"];
    let goat_names = ["Clover", "Maple", "Pepper", "Ginger", "Nutmeg"];
    let duck_names = ["Quackers", "Puddle", "Waddles", "Drake", "Feather"];
    let rabbit_names = ["Thumper", "Clover", "Bun-Bun", "Flopsy", "Hazel"];
    let pig_names = ["Hamlet", "Truffle", "Mud", "Bacon", "Rosie"];
    let horse_names = ["Thunder", "Spirit", "Maple", "Blaze", "Storm"];
    let cat_names = ["Whiskers", "Mittens", "Shadow", "Luna", "Cleo"];
    let dog_names = ["Rex", "Buddy", "Scout", "Max", "Hazel"];

    let names = match kind {
        AnimalKind::Chicken => &chicken_names[..],
        AnimalKind::Cow => &cow_names[..],
        AnimalKind::Sheep => &sheep_names[..],
        AnimalKind::Goat => &goat_names[..],
        AnimalKind::Duck => &duck_names[..],
        AnimalKind::Rabbit => &rabbit_names[..],
        AnimalKind::Pig => &pig_names[..],
        AnimalKind::Horse => &horse_names[..],
        AnimalKind::Cat => &cat_names[..],
        AnimalKind::Dog => &dog_names[..],
    };
    names[rng.gen_range(0..names.len())].to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// System: listen for ShopTransactionEvent and spawn animals on purchase
// ─────────────────────────────────────────────────────────────────────────────

pub fn handle_animal_purchase(
    mut commands: Commands,
    mut shop_events: EventReader<ShopTransactionEvent>,
    animal_state: Res<AnimalState>,
    mut sfx_writer: EventWriter<PlaySfxEvent>,
    sprite_data: Res<AnimalSpriteData>,
    animal_query: Query<&Animal>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    let mut rng = rand::thread_rng();

    for ev in shop_events.read() {
        if !ev.is_purchase {
            continue;
        }

        let Some(kind) = item_to_animal(&ev.item_id) else {
            continue;
        };

        // Building check: chickens need coop, cows/sheep need barn.
        let has_building = match kind {
            AnimalKind::Chicken | AnimalKind::Duck | AnimalKind::Rabbit => animal_state.has_coop,
            AnimalKind::Cow | AnimalKind::Sheep | AnimalKind::Goat | AnimalKind::Pig => {
                animal_state.has_barn
            }
            // Companions don't need a building.
            AnimalKind::Horse | AnimalKind::Cat | AnimalKind::Dog => true,
        };

        if !has_building {
            // The economy/UI domains should prevent this, but we guard here too.
            warn!(
                "Tried to buy {:?} without the required building — skipping spawn.",
                kind
            );
            continue;
        }

        // Cap enforcement per housing type.
        match kind {
            AnimalKind::Chicken | AnimalKind::Duck | AnimalKind::Rabbit => {
                let count = animal_query
                    .iter()
                    .filter(|a| {
                        matches!(
                            a.kind,
                            AnimalKind::Chicken | AnimalKind::Duck | AnimalKind::Rabbit
                        )
                    })
                    .count();
                let max = (animal_state.coop_level as usize) * 4;
                if count >= max {
                    toast_writer.send(ToastEvent {
                        message: "Your coop is full! Upgrade to house more animals.".to_string(),
                        duration_secs: 3.0,
                    });
                    continue;
                }
            }
            AnimalKind::Cow | AnimalKind::Sheep | AnimalKind::Goat | AnimalKind::Pig => {
                let count = animal_query
                    .iter()
                    .filter(|a| {
                        matches!(
                            a.kind,
                            AnimalKind::Cow
                                | AnimalKind::Sheep
                                | AnimalKind::Goat
                                | AnimalKind::Pig
                        )
                    })
                    .count();
                let max = (animal_state.barn_level as usize) * 4;
                if count >= max {
                    toast_writer.send(ToastEvent {
                        message: "Your barn is full! Upgrade to house more animals.".to_string(),
                        duration_secs: 3.0,
                    });
                    continue;
                }
            }
            AnimalKind::Horse | AnimalKind::Cat | AnimalKind::Dog => {
                let already_has = animal_query.iter().any(|a| a.kind == kind);
                if already_has {
                    let name = match kind {
                        AnimalKind::Horse => "horse",
                        AnimalKind::Cat => "cat",
                        AnimalKind::Dog => "dog",
                        _ => unreachable!(),
                    };
                    toast_writer.send(ToastEvent {
                        message: format!("You already have a {}!", name),
                        duration_secs: 3.0,
                    });
                    continue;
                }
            }
        }

        let (pen_min, pen_max) = pen_bounds_for(kind);
        let vis = animal_visual(kind);
        let name = generate_animal_name(kind, &mut rng);

        // Spawn position: random inside pen.
        let spawn_x = rng.gen_range(pen_min.x..=pen_max.x);
        let spawn_y = rng.gen_range(pen_min.y..=pen_max.y);

        // First wander timer fires after 2-4 s.
        let wander_duration = rng.gen_range(2.0_f32..=4.0_f32);

        let animal_data = Animal {
            kind,
            name: name.clone(),
            age: AnimalAge::Baby,
            days_old: 0,
            happiness: 128,
            fed_today: false,
            petted_today: false,
            product_ready: false,
        };

        // Build the sprite: use real atlas for animals with sprite sheets when loaded,
        // fall back to colored rectangle for Horse/Cat/Dog or if not loaded.
        let animal_sprite = if sprite_data.loaded {
            match kind {
                AnimalKind::Chicken => {
                    let mut s = Sprite::from_atlas_image(
                        sprite_data.chicken_image.clone(),
                        TextureAtlas {
                            layout: sprite_data.chicken_layout.clone(),
                            index: 0,
                        },
                    );
                    s.custom_size = Some(Vec2::new(16.0, 16.0));
                    s.anchor = bevy::sprite::Anchor::BottomCenter;
                    s
                }
                AnimalKind::Cow => {
                    let mut s = Sprite::from_atlas_image(
                        sprite_data.cow_image.clone(),
                        TextureAtlas {
                            layout: sprite_data.cow_layout.clone(),
                            index: 0,
                        },
                    );
                    s.custom_size = Some(Vec2::new(32.0, 32.0));
                    s.anchor = bevy::sprite::Anchor::BottomCenter;
                    s
                }
                AnimalKind::Sheep => {
                    let mut s = Sprite::from_atlas_image(
                        sprite_data.sheep_image.clone(),
                        TextureAtlas {
                            layout: sprite_data.sheep_layout.clone(),
                            index: 0,
                        },
                    );
                    s.custom_size = Some(Vec2::new(32.0, 32.0));
                    s.anchor = bevy::sprite::Anchor::BottomCenter;
                    s
                }
                AnimalKind::Goat => {
                    let mut s = Sprite::from_atlas_image(
                        sprite_data.goat_image.clone(),
                        TextureAtlas {
                            layout: sprite_data.goat_layout.clone(),
                            index: 0,
                        },
                    );
                    s.custom_size = Some(Vec2::new(32.0, 32.0));
                    s.anchor = bevy::sprite::Anchor::BottomCenter;
                    s
                }
                AnimalKind::Pig => {
                    let mut s = Sprite::from_atlas_image(
                        sprite_data.pig_image.clone(),
                        TextureAtlas {
                            layout: sprite_data.pig_layout.clone(),
                            index: 0,
                        },
                    );
                    s.custom_size = Some(Vec2::new(32.0, 32.0));
                    s.anchor = bevy::sprite::Anchor::BottomCenter;
                    s
                }
                AnimalKind::Duck => {
                    let mut s = Sprite::from_atlas_image(
                        sprite_data.duck_image.clone(),
                        TextureAtlas {
                            layout: sprite_data.duck_layout.clone(),
                            index: 0,
                        },
                    );
                    s.custom_size = Some(Vec2::new(16.0, 16.0));
                    s.anchor = bevy::sprite::Anchor::BottomCenter;
                    s
                }
                AnimalKind::Rabbit => {
                    let mut s = Sprite::from_atlas_image(
                        sprite_data.rabbit_image.clone(),
                        TextureAtlas {
                            layout: sprite_data.rabbit_layout.clone(),
                            index: 0,
                        },
                    );
                    s.custom_size = Some(Vec2::new(16.0, 16.0));
                    s.anchor = bevy::sprite::Anchor::BottomCenter;
                    s
                }
                // Horse, Cat, Dog: no sprite sheets available — colored rectangles.
                AnimalKind::Horse => {
                    let mut s = Sprite {
                        color: Color::srgb(0.35, 0.2, 0.1),
                        custom_size: Some(Vec2::new(24.0, 20.0)),
                        ..default()
                    };
                    s.anchor = bevy::sprite::Anchor::BottomCenter;
                    s
                }
                AnimalKind::Cat => {
                    let mut s = Sprite {
                        color: Color::srgb(0.9, 0.55, 0.2),
                        custom_size: Some(Vec2::new(12.0, 12.0)),
                        ..default()
                    };
                    s.anchor = bevy::sprite::Anchor::BottomCenter;
                    s
                }
                AnimalKind::Dog => {
                    let mut s = Sprite {
                        color: Color::srgb(0.6, 0.38, 0.18),
                        custom_size: Some(Vec2::new(14.0, 14.0)),
                        ..default()
                    };
                    s.anchor = bevy::sprite::Anchor::BottomCenter;
                    s
                }
            }
        } else {
            let mut s = Sprite {
                color: vis.color,
                custom_size: Some(Vec2::new(vis.width, vis.height)),
                ..default()
            };
            s.anchor = bevy::sprite::Anchor::BottomCenter;
            s
        };

        let entity = commands
            .spawn((
                animal_data,
                animal_sprite,
                LogicalPosition(Vec2::new(spawn_x, spawn_y)),
                Transform::from_xyz(spawn_x, spawn_y, Z_ENTITY_BASE),
                YSorted,
                GlobalTransform::default(),
                Visibility::default(),
                WanderAi {
                    timer: Timer::from_seconds(wander_duration, TimerMode::Once),
                    target: None,
                    pen_min,
                    pen_max,
                    speed: match kind {
                        AnimalKind::Chicken => 20.0,
                        AnimalKind::Cow => 14.0,
                        AnimalKind::Sheep => 16.0,
                        AnimalKind::Cat => 24.0,
                        AnimalKind::Dog => 28.0,
                        AnimalKind::Goat => 18.0,
                        AnimalKind::Duck => 18.0,
                        AnimalKind::Rabbit => 26.0,
                        AnimalKind::Pig => 14.0,
                        AnimalKind::Horse => 30.0,
                    },
                },
            ))
            .id();

        // Attach animation timer for all animals.
        // Atlas animals cycle first-row sprite frames; non-atlas animals (Horse,
        // Cat, Dog) use the timer elapsed time to drive a vertical bob animation.
        let (anim_period, anim_frames) = match kind {
            AnimalKind::Chicken => (0.15, 24),   // 24 cols
            AnimalKind::Cow => (0.15, 36),       // 36 cols
            AnimalKind::Sheep => (0.15, 24),     // 24 cols
            AnimalKind::Goat => (0.15, 24),      // 24 cols
            AnimalKind::Pig => (0.15, 24),       // 24 cols
            AnimalKind::Duck => (0.12, 48),      // 48 cols
            AnimalKind::Rabbit => (0.12, 48),    // 48 cols
            _ => (0.3, 2), // non-atlas: 2 phases for bob (Horse, Cat, Dog)
        };
        commands.entity(entity).insert(AnimalAnimTimer {
            timer: Timer::from_seconds(anim_period, TimerMode::Repeating),
            frame_count: anim_frames,
            current_frame: 0,
        });

        // Spawn a "name tag" text as a child entity.
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Text2d::new(name.clone()),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, vis.height / 2.0 + 4.0, 0.1),
            ));
        });

        sfx_writer.send(PlaySfxEvent {
            sfx_id: format!("{}_cry", ev.item_id),
        });

        info!(
            "Spawned {:?} named '{}'  at ({:.0}, {:.0})",
            kind, name, spawn_x, spawn_y
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// System: spawn the feed trough at a fixed position on the farm.
// ─────────────────────────────────────────────────────────────────────────────

pub fn setup_feed_trough(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut furniture: ResMut<crate::world::objects::FurnitureAtlases>,
    existing: Query<Entity, With<super::FeedTrough>>,
) {
    // Guard against re-entry (e.g. Playing → Cutscene → Playing).
    if !existing.is_empty() {
        return;
    }

    // Ensure furniture atlas is loaded even if WorldPlugin hasn't run yet
    // (AnimalPlugin may be registered before WorldPlugin).
    crate::world::objects::ensure_furniture_atlases_loaded(
        &asset_server,
        &mut layouts,
        &mut furniture,
    );
    let sprite = if furniture.loaded {
        let mut s = Sprite::from_atlas_image(
            furniture.image.clone(),
            TextureAtlas {
                layout: furniture.layout.clone(),
                index: 30, // row 3: low/flat object for feed trough
            },
        );
        s.custom_size = Some(Vec2::new(24.0, 10.0));
        s
    } else {
        Sprite {
            color: Color::srgb(0.55, 0.38, 0.18),
            custom_size: Some(Vec2::new(24.0, 10.0)),
            ..default()
        }
    };

    // Trough sits at the entrance of the barn area. Grid position (5, 19) — south of barn entrance
    // in a 16-px grid → world (80.0, 304.0).
    commands.spawn((
        super::FeedTrough {
            grid_x: 5,
            grid_y: 19,
        },
        sprite,
        Transform::from_xyz(5.0 * TILE_SIZE, 19.0 * TILE_SIZE, Z_ENTITY_BASE),
        LogicalPosition(Vec2::new(5.0 * TILE_SIZE, 19.0 * TILE_SIZE)),
        YSorted,
        Visibility::default(),
    ));
}

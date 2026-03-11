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
            width: 48.0,
            height: 48.0,
        },
        AnimalKind::Sheep => AnimalVisual {
            color: Color::srgb(0.97, 0.97, 0.97),
            width: 32.0,
            height: 32.0,
        },
        AnimalKind::Goat => AnimalVisual {
            color: Color::srgb(0.85, 0.85, 0.78),
            width: 32.0,
            height: 48.0,
        },
        AnimalKind::Duck => AnimalVisual {
            color: Color::srgb(0.95, 0.88, 0.1),
            width: 16.0,
            height: 16.0,
        },
        AnimalKind::Rabbit => AnimalVisual {
            color: Color::srgb(0.8, 0.8, 0.8),
            width: 16.0,
            height: 16.0,
        },
        AnimalKind::Pig => AnimalVisual {
            color: Color::srgb(0.95, 0.7, 0.73),
            width: 32.0,
            height: 32.0,
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
            width: 48.0,
            height: 32.0,
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

fn animal_speed(kind: AnimalKind) -> f32 {
    match kind {
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
    }
}

fn animation_profile(kind: AnimalKind) -> (f32, usize) {
    match kind {
        AnimalKind::Chicken => (0.15, 6),
        AnimalKind::Cow => (0.15, 6),
        AnimalKind::Sheep => (0.15, 6),
        AnimalKind::Goat => (0.15, 6),
        AnimalKind::Pig => (0.15, 6),
        AnimalKind::Duck => (0.15, 6),
        AnimalKind::Rabbit => (0.12, 6),
        AnimalKind::Dog => (0.15, 6),
        _ => (0.3, 2),
    }
}

fn pen_spawn_position(kind: AnimalKind, slot: usize) -> Vec2 {
    let (gx, gy) = match kind {
        AnimalKind::Chicken | AnimalKind::Duck | AnimalKind::Rabbit => {
            const COOP_SLOTS: &[(i32, i32)] = &[(9, 17), (10, 17), (11, 17), (10, 18)];
            COOP_SLOTS[slot % COOP_SLOTS.len()]
        }
        AnimalKind::Cow | AnimalKind::Sheep | AnimalKind::Goat | AnimalKind::Pig => {
            const BARN_SLOTS: &[(i32, i32)] = &[(4, 17), (5, 17), (6, 17), (5, 18)];
            BARN_SLOTS[slot % BARN_SLOTS.len()]
        }
        AnimalKind::Horse | AnimalKind::Cat | AnimalKind::Dog => {
            const PET_SLOTS: &[(i32, i32)] = &[(14, 10), (16, 10), (18, 11), (20, 12)];
            PET_SLOTS[slot % PET_SLOTS.len()]
        }
    };

    let world = grid_to_world_center(gx, gy);
    Vec2::new(world.x, world.y)
}

fn starter_herd() -> Vec<Animal> {
    vec![
        Animal {
            kind: AnimalKind::Chicken,
            name: "Goldie".into(),
            age: AnimalAge::Adult,
            days_old: 14,
            happiness: 180,
            fed_today: true,
            petted_today: false,
            product_ready: false,
        },
        Animal {
            kind: AnimalKind::Duck,
            name: "Puddle".into(),
            age: AnimalAge::Adult,
            days_old: 14,
            happiness: 170,
            fed_today: true,
            petted_today: false,
            product_ready: false,
        },
        Animal {
            kind: AnimalKind::Cow,
            name: "Bessie".into(),
            age: AnimalAge::Adult,
            days_old: 21,
            happiness: 185,
            fed_today: true,
            petted_today: false,
            product_ready: false,
        },
        Animal {
            kind: AnimalKind::Sheep,
            name: "Woolie".into(),
            age: AnimalAge::Adult,
            days_old: 21,
            happiness: 175,
            fed_today: true,
            petted_today: false,
            product_ready: false,
        },
    ]
}

fn spawn_animal_entity(
    commands: &mut Commands,
    sprite_data: &AnimalSpriteData,
    animal_data: Animal,
    spawn_pos: Vec2,
) {
    let kind = animal_data.kind;
    let vis = animal_visual(kind);
    let (pen_min, pen_max) = pen_bounds_for(kind);

    // Stagger initial wander timing slightly so the herd doesn't move in lockstep.
    let wander_duration = 2.0 + ((animal_data.days_old as f32) % 3.0);

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
                s.custom_size = Some(Vec2::new(48.0, 48.0));
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
                s.custom_size = Some(Vec2::new(32.0, 48.0));
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
                let mut s = Sprite::from_atlas_image(
                    sprite_data.dog_image.clone(),
                    TextureAtlas {
                        layout: sprite_data.dog_layout.clone(),
                        index: 0,
                    },
                );
                s.custom_size = Some(Vec2::new(48.0, 32.0));
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
            animal_data.clone(),
            animal_sprite,
            LogicalPosition(spawn_pos),
            Transform::from_xyz(spawn_pos.x, spawn_pos.y, Z_ENTITY_BASE),
            YSorted,
            GlobalTransform::default(),
            Visibility::default(),
            WanderAi {
                timer: Timer::from_seconds(wander_duration, TimerMode::Once),
                target: None,
                pen_min,
                pen_max,
                speed: animal_speed(kind),
            },
            Facing::Down,
        ))
        .id();

    let (anim_period, anim_frames) = animation_profile(kind);
    commands.entity(entity).insert(AnimalAnimTimer {
        timer: Timer::from_seconds(anim_period, TimerMode::Repeating),
        frame_count: anim_frames,
        current_frame: 0,
    });

    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            Text2d::new(animal_data.name.clone()),
            TextFont {
                font_size: 11.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(0.0, vis.height / 2.0 + 4.0, 0.1),
        ));
    });
}

pub fn spawn_animals_from_state(
    mut commands: Commands,
    mut animal_state: ResMut<AnimalState>,
    sprite_data: Res<AnimalSpriteData>,
    animal_query: Query<&Animal>,
) {
    if !animal_query.is_empty() {
        return;
    }

    if animal_state.animals.is_empty()
        && !animal_state.has_coop
        && !animal_state.has_barn
        && animal_state.coop_level == 0
        && animal_state.barn_level == 0
    {
        animal_state.has_coop = true;
        animal_state.has_barn = true;
        animal_state.coop_level = 1;
        animal_state.barn_level = 1;

        for (slot, animal) in starter_herd().into_iter().enumerate() {
            let spawn_pos = pen_spawn_position(animal.kind, slot);
            spawn_animal_entity(&mut commands, &sprite_data, animal, spawn_pos);
        }
        return;
    }

    let mut coop_slot = 0usize;
    let mut barn_slot = 0usize;
    let mut pet_slot = 0usize;

    for animal in animal_state.animals.iter().cloned() {
        let slot = match animal.kind {
            AnimalKind::Chicken | AnimalKind::Duck | AnimalKind::Rabbit => {
                let slot = coop_slot;
                coop_slot += 1;
                slot
            }
            AnimalKind::Cow | AnimalKind::Sheep | AnimalKind::Goat | AnimalKind::Pig => {
                let slot = barn_slot;
                barn_slot += 1;
                slot
            }
            AnimalKind::Horse | AnimalKind::Cat | AnimalKind::Dog => {
                let slot = pet_slot;
                pet_slot += 1;
                slot
            }
        };

        let spawn_pos = pen_spawn_position(animal.kind, slot);
        spawn_animal_entity(&mut commands, &sprite_data, animal, spawn_pos);
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

        let name = generate_animal_name(kind, &mut rng);

        let (pen_min, pen_max) = pen_bounds_for(kind);
        let spawn_pos = Vec2::new(
            rng.gen_range(pen_min.x..=pen_max.x),
            rng.gen_range(pen_min.y..=pen_max.y),
        );

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

        spawn_animal_entity(&mut commands, &sprite_data, animal_data, spawn_pos);

        sfx_writer.send(PlaySfxEvent {
            sfx_id: format!("{}_cry", ev.item_id),
        });

        info!(
            "Spawned {:?} named '{}'  at ({:.0}, {:.0})",
            kind, name, spawn_pos.x, spawn_pos.y
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_game_seeds_visible_starter_herd() {
        let mut app = App::new();
        app.init_resource::<AnimalState>();
        app.init_resource::<AnimalSpriteData>();
        app.add_systems(Update, spawn_animals_from_state);

        app.update();

        let animals = {
            let world = app.world_mut();
            let mut query = world.query::<&Animal>();
            query.iter(world).count()
        };
        let state = app.world().resource::<AnimalState>();

        assert_eq!(animals, 4, "fresh game should seed a small visible herd");
        assert!(
            state.has_coop,
            "starter herd should mark the coop as present"
        );
        assert!(
            state.has_barn,
            "starter herd should mark the barn as present"
        );
        assert_eq!(state.coop_level, 1);
        assert_eq!(state.barn_level, 1);
    }

    #[test]
    fn saved_animals_respawn_without_seeding_extra_animals() {
        let mut app = App::new();
        app.insert_resource(AnimalState {
            animals: vec![Animal {
                kind: AnimalKind::Cow,
                name: "Bessie".into(),
                age: AnimalAge::Adult,
                days_old: 20,
                happiness: 180,
                fed_today: true,
                petted_today: false,
                product_ready: false,
            }],
            has_coop: false,
            has_barn: true,
            coop_level: 0,
            barn_level: 1,
        });
        app.init_resource::<AnimalSpriteData>();
        app.add_systems(Update, spawn_animals_from_state);

        app.update();

        let animals: Vec<AnimalKind> = {
            let world = app.world_mut();
            let mut query = world.query::<&Animal>();
            query.iter(world).map(|animal| animal.kind).collect()
        };

        assert_eq!(animals, vec![AnimalKind::Cow]);
    }
}

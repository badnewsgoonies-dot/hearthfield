use bevy::prelude::*;
use rand::Rng;
use crate::shared::*;
use super::{WanderAi, AnimalSpriteData};

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
            width: 12.0,
            height: 12.0,
        },
        AnimalKind::Cow => AnimalVisual {
            color: Color::srgb(0.85, 0.85, 0.85),
            width: 20.0,
            height: 16.0,
        },
        AnimalKind::Sheep => AnimalVisual {
            color: Color::srgb(0.95, 0.95, 0.9),
            width: 18.0,
            height: 14.0,
        },
        AnimalKind::Cat => AnimalVisual {
            color: Color::srgb(0.8, 0.5, 0.2),
            width: 10.0,
            height: 10.0,
        },
        AnimalKind::Dog => AnimalVisual {
            color: Color::srgb(0.6, 0.4, 0.2),
            width: 12.0,
            height: 12.0,
        },
    }
}

/// Returns the pen/barn boundaries where this kind of animal wanders.
/// Barn animals (cow, sheep) get the barn pen; coop animals (chicken) get the coop yard;
/// pets (cat, dog) roam a wider farm area.
fn pen_bounds_for(kind: AnimalKind) -> (Vec2, Vec2) {
    match kind {
        AnimalKind::Chicken => (Vec2::new(-96.0, -192.0), Vec2::new(96.0, -96.0)),
        AnimalKind::Cow | AnimalKind::Sheep => {
            (Vec2::new(-192.0, -192.0), Vec2::new(-32.0, -64.0))
        }
        AnimalKind::Cat | AnimalKind::Dog => {
            (Vec2::new(-256.0, -256.0), Vec2::new(256.0, 256.0))
        }
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
        _ => None,
    }
}

fn generate_animal_name(kind: AnimalKind, rng: &mut impl Rng) -> String {
    let chicken_names = ["Penny", "Goldie", "Clucky", "Nugget", "Dottie"];
    let cow_names = ["Bessie", "Daisy", "Rosie", "Mocha", "Cream"];
    let sheep_names = ["Fluffkins", "Woolie", "Cotton", "Misty", "Pearl"];
    let cat_names = ["Whiskers", "Mittens", "Shadow", "Luna", "Cleo"];
    let dog_names = ["Rex", "Buddy", "Scout", "Max", "Hazel"];

    let names = match kind {
        AnimalKind::Chicken => &chicken_names[..],
        AnimalKind::Cow => &cow_names[..],
        AnimalKind::Sheep => &sheep_names[..],
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
            AnimalKind::Chicken => animal_state.has_coop,
            AnimalKind::Cow | AnimalKind::Sheep => animal_state.has_barn,
            // Pets don't need a building.
            AnimalKind::Cat | AnimalKind::Dog => true,
        };

        if !has_building {
            // The economy/UI domains should prevent this, but we guard here too.
            warn!(
                "Tried to buy {:?} without the required building — skipping spawn.",
                kind
            );
            continue;
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

        // Build the sprite: use real atlas for chickens/cows when loaded,
        // fall back to colored rectangle for sheep/cat/dog or if not loaded.
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
                    s
                }
                _ => Sprite {
                    color: vis.color,
                    custom_size: Some(Vec2::new(vis.width, vis.height)),
                    ..default()
                },
            }
        } else {
            Sprite {
                color: vis.color,
                custom_size: Some(Vec2::new(vis.width, vis.height)),
                ..default()
            }
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
                    },
                },
            ))
            .id();

        // Spawn a "name tag" text as a child entity.
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                Text2d::new(name.clone()),
                TextFont {
                    font_size: 6.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Transform::from_xyz(0.0, vis.height / 2.0 + 4.0, 0.1),
            ));
        });

        sfx_writer.send(PlaySfxEvent {
            sfx_id: format!("{}_cry", ev.item_id),
        });

        info!("Spawned {:?} named '{}'  at ({:.0}, {:.0})", kind, name, spawn_x, spawn_y);
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
        &asset_server, &mut layouts, &mut furniture,
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

    // Trough sits at the entrance of the barn area. Grid position (−10, −8) in
    // a 16-px grid → world (−160, −128).
    commands.spawn((
        super::FeedTrough {
            grid_x: -10,
            grid_y: -8,
        },
        sprite,
        Transform::from_xyz(-160.0, -128.0, Z_ENTITY_BASE),
        LogicalPosition(Vec2::new(-160.0, -128.0)),
        YSorted,
        Visibility::default(),
    ));
}

//! Interactive world objects — desks, vending machines, coffee machines, etc.
//!
//! Each zone type gets appropriate objects spawned at defined positions.
//! Players interact with objects via the F-key to get items, info, or effects.

use crate::shared::*;
use bevy::prelude::*;
use rand::Rng;

// ═══════════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════

/// Extended object kinds beyond the basic `WorldObjectKind` in shared.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExtendedObjectKind {
    Desk,
    Bookshelf,
    NoticeBoard,
    SouvenirStand,
    TV,
    Radio,
    Newspaper,
    Phone,
    Computer,
    MapDisplay,
    PlantPot,
    Bench,
    Luggage,
    FireExtinguisher,
    FirstAidKit,
    WaterCooler,
    CoffeeMachine,
}

impl ExtendedObjectKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Desk => "Desk",
            Self::Bookshelf => "Bookshelf",
            Self::NoticeBoard => "Notice Board",
            Self::SouvenirStand => "Souvenir Stand",
            Self::TV => "Television",
            Self::Radio => "Radio",
            Self::Newspaper => "Newspaper",
            Self::Phone => "Phone",
            Self::Computer => "Computer",
            Self::MapDisplay => "Map Display",
            Self::PlantPot => "Plant Pot",
            Self::Bench => "Bench",
            Self::Luggage => "Lost Luggage",
            Self::FireExtinguisher => "Fire Extinguisher",
            Self::FirstAidKit => "First Aid Kit",
            Self::WaterCooler => "Water Cooler",
            Self::CoffeeMachine => "Coffee Machine",
        }
    }

    pub fn is_interactive(&self) -> bool {
        !matches!(self, Self::PlantPot | Self::Bench | Self::Luggage)
    }

    pub fn prompt(&self) -> &'static str {
        match self {
            Self::Desk => "[F] Examine Desk",
            Self::Bookshelf => "[F] Read",
            Self::NoticeBoard => "[F] Read Notices",
            Self::SouvenirStand => "[F] Browse Souvenirs",
            Self::TV => "[F] Watch TV",
            Self::Radio => "[F] Listen",
            Self::Newspaper => "[F] Read Newspaper",
            Self::Phone => "[F] Use Phone",
            Self::Computer => "[F] Use Computer",
            Self::MapDisplay => "[F] View Map",
            Self::PlantPot => "",
            Self::Bench => "",
            Self::Luggage => "",
            Self::FireExtinguisher => "[F] Inspect",
            Self::FirstAidKit => "[F] Open Kit",
            Self::WaterCooler => "[F] Drink Water",
            Self::CoffeeMachine => "[F] Get Coffee",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Desk => "A tidy operations desk with neatly stacked flight manifests.",
            Self::Bookshelf => "Aviation manuals, weather charts, and a few dog-eared novels.",
            Self::NoticeBoard => {
                "Pinned notices about schedule changes, safety reminders, and a lost cat poster."
            }
            Self::SouvenirStand => "Miniature aircraft, postcards, and overpriced key-chains.",
            Self::TV => "The news is showing regional weather and flight delay updates.",
            Self::Radio => "Faint chatter from ATC frequencies crackles through the speaker.",
            Self::Newspaper => "Today's edition of The Skyward Times. Crossword is half-done.",
            Self::Phone => "An internal airport telephone. Dial 0 for operations.",
            Self::Computer => "A terminal showing live METAR data and NOTAM updates.",
            Self::MapDisplay => "A large map of all air routes in the region.",
            Self::PlantPot => "A hardy potted fern that thrives on fluorescent light.",
            Self::Bench => "A well-worn wooden bench. Generations of travelers have rested here.",
            Self::Luggage => "An unclaimed suitcase tagged for a flight that departed hours ago.",
            Self::FireExtinguisher => "Class ABC extinguisher. Inspection tag is current.",
            Self::FirstAidKit => "Standard aviation first aid kit. Fully stocked.",
            Self::WaterCooler => "Refreshing cold water. Just what you need.",
            Self::CoffeeMachine => "Industrial-grade coffee machine. Fuel for pilots.",
        }
    }

    pub fn placeholder_color(&self) -> Color {
        match self {
            Self::Desk => Color::srgb(0.5, 0.35, 0.2),
            Self::Bookshelf => Color::srgb(0.45, 0.3, 0.15),
            Self::NoticeBoard => Color::srgb(0.6, 0.55, 0.3),
            Self::SouvenirStand => Color::srgb(0.7, 0.5, 0.3),
            Self::TV => Color::srgb(0.2, 0.2, 0.25),
            Self::Radio => Color::srgb(0.3, 0.3, 0.35),
            Self::Newspaper => Color::srgb(0.85, 0.82, 0.75),
            Self::Phone => Color::srgb(0.2, 0.2, 0.2),
            Self::Computer => Color::srgb(0.15, 0.15, 0.2),
            Self::MapDisplay => Color::srgb(0.3, 0.5, 0.4),
            Self::PlantPot => Color::srgb(0.2, 0.5, 0.2),
            Self::Bench => Color::srgb(0.5, 0.35, 0.2),
            Self::Luggage => Color::srgb(0.3, 0.3, 0.5),
            Self::FireExtinguisher => Color::srgb(0.8, 0.15, 0.1),
            Self::FirstAidKit => Color::srgb(0.9, 0.9, 0.9),
            Self::WaterCooler => Color::srgb(0.5, 0.7, 0.9),
            Self::CoffeeMachine => Color::srgb(0.35, 0.2, 0.1),
        }
    }
}

/// Tag for extended world objects.
#[derive(Component)]
pub struct ExtendedObject {
    pub kind: ExtendedObjectKind,
}

// ═══════════════════════════════════════════════════════════════════════════
// OBJECT LAYOUTS PER ZONE
// ═══════════════════════════════════════════════════════════════════════════

struct ObjectPlacement {
    kind: ExtendedObjectKind,
    gx: i32,
    gy: i32,
}

fn terminal_objects() -> Vec<ObjectPlacement> {
    vec![
        ObjectPlacement {
            kind: ExtendedObjectKind::Desk,
            gx: 4,
            gy: 2,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::NoticeBoard,
            gx: 12,
            gy: 1,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Newspaper,
            gx: 6,
            gy: 6,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::TV,
            gx: 15,
            gy: 2,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Bench,
            gx: 8,
            gy: 10,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Bench,
            gx: 12,
            gy: 10,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Luggage,
            gx: 16,
            gy: 9,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::WaterCooler,
            gx: 18,
            gy: 3,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::FireExtinguisher,
            gx: 1,
            gy: 5,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::MapDisplay,
            gx: 8,
            gy: 1,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::PlantPot,
            gx: 3,
            gy: 1,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::PlantPot,
            gx: 19,
            gy: 1,
        },
    ]
}

fn lounge_objects() -> Vec<ObjectPlacement> {
    vec![
        ObjectPlacement {
            kind: ExtendedObjectKind::CoffeeMachine,
            gx: 14,
            gy: 2,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::TV,
            gx: 8,
            gy: 1,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Radio,
            gx: 10,
            gy: 3,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Bookshelf,
            gx: 16,
            gy: 4,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Newspaper,
            gx: 5,
            gy: 5,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Bench,
            gx: 6,
            gy: 8,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::WaterCooler,
            gx: 13,
            gy: 7,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::PlantPot,
            gx: 2,
            gy: 3,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::PlantPot,
            gx: 17,
            gy: 1,
        },
    ]
}

fn hangar_objects() -> Vec<ObjectPlacement> {
    vec![
        ObjectPlacement {
            kind: ExtendedObjectKind::Computer,
            gx: 5,
            gy: 2,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Phone,
            gx: 7,
            gy: 2,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::FirstAidKit,
            gx: 18,
            gy: 2,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::FireExtinguisher,
            gx: 1,
            gy: 6,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::FireExtinguisher,
            gx: 1,
            gy: 12,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Desk,
            gx: 6,
            gy: 5,
        },
    ]
}

fn control_tower_objects() -> Vec<ObjectPlacement> {
    vec![
        ObjectPlacement {
            kind: ExtendedObjectKind::Computer,
            gx: 4,
            gy: 3,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Computer,
            gx: 6,
            gy: 3,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Computer,
            gx: 8,
            gy: 3,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Radio,
            gx: 10,
            gy: 2,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Phone,
            gx: 12,
            gy: 3,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::MapDisplay,
            gx: 7,
            gy: 1,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::CoffeeMachine,
            gx: 14,
            gy: 5,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Desk,
            gx: 3,
            gy: 5,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::FirstAidKit,
            gx: 15,
            gy: 2,
        },
    ]
}

fn crew_quarters_objects() -> Vec<ObjectPlacement> {
    vec![
        ObjectPlacement {
            kind: ExtendedObjectKind::CoffeeMachine,
            gx: 10,
            gy: 2,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Bookshelf,
            gx: 12,
            gy: 2,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::TV,
            gx: 6,
            gy: 1,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Radio,
            gx: 8,
            gy: 4,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Newspaper,
            gx: 5,
            gy: 7,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::PlantPot,
            gx: 1,
            gy: 1,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::WaterCooler,
            gx: 11,
            gy: 7,
        },
    ]
}

fn shop_objects() -> Vec<ObjectPlacement> {
    vec![
        ObjectPlacement {
            kind: ExtendedObjectKind::SouvenirStand,
            gx: 12,
            gy: 5,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::SouvenirStand,
            gx: 14,
            gy: 5,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Newspaper,
            gx: 6,
            gy: 3,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::PlantPot,
            gx: 1,
            gy: 1,
        },
    ]
}

fn city_objects() -> Vec<ObjectPlacement> {
    vec![
        ObjectPlacement {
            kind: ExtendedObjectKind::Bench,
            gx: 5,
            gy: 4,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Bench,
            gx: 12,
            gy: 8,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::MapDisplay,
            gx: 3,
            gy: 2,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::PlantPot,
            gx: 7,
            gy: 1,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::PlantPot,
            gx: 15,
            gy: 1,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::NoticeBoard,
            gx: 10,
            gy: 1,
        },
        ObjectPlacement {
            kind: ExtendedObjectKind::Phone,
            gx: 17,
            gy: 3,
        },
    ]
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Spawn extended objects for the current zone after a zone transition.
pub fn spawn_extended_zone_objects(
    mut commands: Commands,
    mut transition_events: EventReader<ZoneTransitionEvent>,
    existing: Query<Entity, With<ExtendedObject>>,
) {
    for evt in transition_events.read() {
        // Despawn old extended objects
        for entity in &existing {
            commands.entity(entity).despawn_recursive();
        }

        let placements = match evt.to_zone {
            MapZone::Terminal => terminal_objects(),
            MapZone::Lounge => lounge_objects(),
            MapZone::Hangar => hangar_objects(),
            MapZone::ControlTower => control_tower_objects(),
            MapZone::CrewQuarters => crew_quarters_objects(),
            MapZone::Shop => shop_objects(),
            MapZone::CityStreet => city_objects(),
            MapZone::Runway => Vec::new(),
        };

        for obj in placements {
            let pos = grid_to_world_center(obj.gx, obj.gy);
            let mut entity_cmd = commands.spawn((
                ExtendedObject { kind: obj.kind },
                Sprite::from_color(obj.kind.placeholder_color(), Vec2::new(14.0, 14.0)),
                Transform::from_xyz(pos.x, pos.y, Z_OBJECTS),
            ));

            if obj.kind.is_interactive() {
                entity_cmd.insert(Interactable {
                    prompt: obj.kind.prompt().to_string(),
                    range: 1.5,
                });
            }
        }
    }
}

/// Handle player interaction with extended objects.
#[allow(clippy::too_many_arguments)]
pub fn interact_extended_object(
    player_input: Res<PlayerInput>,
    mut interaction_claimed: ResMut<InteractionClaimed>,
    player_query: Query<&Transform, With<Player>>,
    object_query: Query<(&Transform, &ExtendedObject, &Interactable)>,
    mut toast: EventWriter<ToastEvent>,
    mut pilot_state: ResMut<PilotState>,
    mut inventory: ResMut<Inventory>,
    weather: Res<WeatherState>,
    calendar: Res<Calendar>,
) {
    if !player_input.interact || interaction_claimed.0 {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    for (obj_tf, ext_obj, interactable) in &object_query {
        let dist = player_pos.distance(obj_tf.translation.truncate());
        if dist > interactable.range * TILE_SIZE {
            continue;
        }

        interaction_claimed.0 = true;

        match ext_obj.kind {
            ExtendedObjectKind::CoffeeMachine => {
                pilot_state.stamina = (pilot_state.stamina + 15.0).min(pilot_state.max_stamina);
                toast.send(ToastEvent {
                    message: "Hot coffee! Stamina +15".into(),
                    duration_secs: 2.5,
                });
            }
            ExtendedObjectKind::WaterCooler => {
                pilot_state.stamina = (pilot_state.stamina + 5.0).min(pilot_state.max_stamina);
                toast.send(ToastEvent {
                    message: "Refreshing water. Stamina +5".into(),
                    duration_secs: 2.0,
                });
            }
            ExtendedObjectKind::Computer => {
                let msg = format!(
                    "Weather: {:?} | Wind: {:.0}kts {:.0}° | Vis: {:.1}nm | Ceiling: {}ft",
                    weather.current,
                    weather.wind_speed_knots,
                    weather.wind_direction_deg,
                    weather.visibility_nm,
                    weather.ceiling_ft,
                );
                toast.send(ToastEvent {
                    message: msg,
                    duration_secs: 4.0,
                });
            }
            ExtendedObjectKind::SouvenirStand => {
                let mut rng = rand::thread_rng();
                let item = match rng.gen_range(0..4) {
                    0 => "souvenir_keychain",
                    1 => "souvenir_postcard",
                    2 => "souvenir_model_plane",
                    _ => "souvenir_magnet",
                };
                if inventory.add_item(item, 1) {
                    toast.send(ToastEvent {
                        message: format!("Picked up a {}!", item.replace('_', " ")),
                        duration_secs: 2.5,
                    });
                } else {
                    toast.send(ToastEvent {
                        message: "Inventory full!".into(),
                        duration_secs: 2.0,
                    });
                }
            }
            ExtendedObjectKind::FirstAidKit => {
                pilot_state.stamina = (pilot_state.stamina + 25.0).min(pilot_state.max_stamina);
                toast.send(ToastEvent {
                    message: "Used first aid kit. Stamina +25".into(),
                    duration_secs: 2.5,
                });
            }
            ExtendedObjectKind::TV => {
                let tip = match calendar.hour % 4 {
                    0 => "News: Clear skies expected this afternoon.",
                    1 => "Sports: The regional air race finals are this weekend!",
                    2 => "Weather: A cold front is moving in from the northwest.",
                    _ => "Breaking: New aircraft model unveiled at Grandcity Air Expo.",
                };
                toast.send(ToastEvent {
                    message: format!("TV: {tip}"),
                    duration_secs: 3.5,
                });
            }
            ExtendedObjectKind::Radio => {
                let chatter = match calendar.minute % 5 {
                    0 => "ATC: Runway 27L cleared for takeoff.",
                    1 => "ATC: Skywarden 42, descend and maintain flight level 180.",
                    2 => "ATC: Traffic advisory — light aircraft 2 o'clock, 3 miles.",
                    3 => "ATC: Wind check — 270 at 12 knots, gusting 18.",
                    _ => "ATC: All stations, airport weather observation follows...",
                };
                toast.send(ToastEvent {
                    message: chatter.into(),
                    duration_secs: 3.0,
                });
            }
            ExtendedObjectKind::Newspaper => {
                let headlines = [
                    "Skyward Times: Fuel prices expected to drop next season.",
                    "Skyward Times: Pilot shortage drives up charter rates.",
                    "Skyward Times: New air route to Skyreach approved!",
                    "Skyward Times: Cloudmere airport expansion underway.",
                    "Skyward Times: Weather station reports record snowfall at Frostpeak.",
                ];
                let idx = (calendar.day as usize) % headlines.len();
                toast.send(ToastEvent {
                    message: headlines[idx].into(),
                    duration_secs: 4.0,
                });
            }
            _ => {
                toast.send(ToastEvent {
                    message: ext_obj.kind.description().into(),
                    duration_secs: 3.0,
                });
            }
        }

        break;
    }
}

//! Non-crew ambient airport NPCs — passengers, security, janitors, ground crew.
//!
//! These NPCs add life to airports with simple patrol patterns and one-liner dialogue.

use crate::shared::*;
use bevy::prelude::*;
use rand::Rng;

// ═══════════════════════════════════════════════════════════════════════════
// COMPONENTS
// ═══════════════════════════════════════════════════════════════════════════

/// Role of an ambient airport NPC.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NpcRole {
    Passenger,
    SecurityGuard,
    ShopKeeper,
    Janitor,
    Tourist,
    BusinessPerson,
    GroundCrew,
    AirTrafficController,
}

impl NpcRole {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Passenger => "Passenger",
            Self::SecurityGuard => "Security Guard",
            Self::ShopKeeper => "Shop Keeper",
            Self::Janitor => "Janitor",
            Self::Tourist => "Tourist",
            Self::BusinessPerson => "Business Traveler",
            Self::GroundCrew => "Ground Crew",
            Self::AirTrafficController => "ATC Officer",
        }
    }

    /// Placeholder sprite color for each role.
    pub fn tint_color(&self) -> Color {
        match self {
            Self::Passenger => Color::srgb(0.6, 0.6, 0.8),
            Self::SecurityGuard => Color::srgb(0.2, 0.3, 0.5),
            Self::ShopKeeper => Color::srgb(0.7, 0.5, 0.3),
            Self::Janitor => Color::srgb(0.4, 0.6, 0.4),
            Self::Tourist => Color::srgb(0.8, 0.6, 0.4),
            Self::BusinessPerson => Color::srgb(0.3, 0.3, 0.35),
            Self::GroundCrew => Color::srgb(0.7, 0.5, 0.1),
            Self::AirTrafficController => Color::srgb(0.3, 0.4, 0.6),
        }
    }
}

/// An ambient NPC that wanders the airport.
#[derive(Component)]
pub struct AirportNpc {
    pub role: NpcRole,
    pub name: String,
}

/// Simple patrol component: NPC walks between waypoints.
#[derive(Component)]
pub struct NpcPatrol {
    pub waypoints: Vec<Vec2>,
    pub current_index: usize,
    pub speed: f32,
    pub wait_timer: f32,
    pub wait_duration: f32,
}

impl NpcPatrol {
    pub fn current_target(&self) -> Vec2 {
        self.waypoints[self.current_index]
    }
}

/// One-liner dialogue spoken when the player interacts with an ambient NPC.
#[derive(Component)]
pub struct NpcOneLiner {
    pub lines: &'static [&'static str],
}

// ═══════════════════════════════════════════════════════════════════════════
// DIALOGUE POOLS
// ═══════════════════════════════════════════════════════════════════════════

const PASSENGER_LINES: &[&str] = &[
    "Have a safe flight!",
    "Is Gate 3 this way?",
    "I hope my luggage makes it...",
    "This airport has great coffee!",
    "Running late — excuse me!",
    "First time flying? I'm nervous...",
    "I love the view from the observation deck.",
];

const SECURITY_LINES: &[&str] = &[
    "Please have your ID ready.",
    "No liquids over 100ml in carry-on.",
    "Keep the area clear, please.",
    "Report any unattended luggage.",
    "Move along, nothing to see here.",
];

const SHOPKEEPER_LINES: &[&str] = &[
    "Browse all you like!",
    "We've got a sale on souvenirs today.",
    "Need anything for the flight?",
    "Best snacks in the terminal!",
];

const JANITOR_LINES: &[&str] = &[
    "Mind the wet floor!",
    "You'd be surprised what people leave behind.",
    "Keeping this place clean is a full-time job.",
    "At least the planes are messier than the terminal.",
];

const TOURIST_LINES: &[&str] = &[
    "Wow, look at that aircraft! Amazing!",
    "Do you know any good restaurants in the city?",
    "I can't believe we're finally here!",
    "Can you take our picture? Just kidding.",
    "This airport is bigger than my hometown!",
];

const BUSINESS_LINES: &[&str] = &[
    "Meeting in two hours. Can't be late.",
    "The Wi-Fi here is terrible.",
    "Third flight this week...",
    "Time zones are killing me.",
    "At least the lounge is comfortable.",
];

const GROUND_CREW_LINES: &[&str] = &[
    "Stand clear of the aircraft!",
    "Fueling in progress — no smoking.",
    "Baggage loaded and ready.",
    "We've got a tight turnaround today.",
    "Watch for FOD on the tarmac.",
    "Marshalling the next arrival in five.",
];

const ATC_LINES: &[&str] = &[
    "Runway 27L is active.",
    "Wind check: two-seven-zero at one-two.",
    "Traffic is moderate today.",
    "Keep your radio tuned in.",
    "Clear skies for departure.",
];

fn lines_for_role(role: NpcRole) -> &'static [&'static str] {
    match role {
        NpcRole::Passenger => PASSENGER_LINES,
        NpcRole::SecurityGuard => SECURITY_LINES,
        NpcRole::ShopKeeper => SHOPKEEPER_LINES,
        NpcRole::Janitor => JANITOR_LINES,
        NpcRole::Tourist => TOURIST_LINES,
        NpcRole::BusinessPerson => BUSINESS_LINES,
        NpcRole::GroundCrew => GROUND_CREW_LINES,
        NpcRole::AirTrafficController => ATC_LINES,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// NPC DEFINITIONS PER ZONE
// ═══════════════════════════════════════════════════════════════════════════

struct NpcSpawnDef {
    role: NpcRole,
    gx: i32,
    gy: i32,
    patrol_offsets: Vec<(i32, i32)>,
}

fn terminal_npcs(density: u32) -> Vec<NpcSpawnDef> {
    let mut npcs = vec![
        NpcSpawnDef {
            role: NpcRole::SecurityGuard,
            gx: 3,
            gy: 5,
            patrol_offsets: vec![(3, 5), (3, 10), (10, 10), (10, 5)],
        },
        NpcSpawnDef {
            role: NpcRole::Passenger,
            gx: 8,
            gy: 6,
            patrol_offsets: vec![(8, 6), (12, 6), (12, 9), (8, 9)],
        },
        NpcSpawnDef {
            role: NpcRole::Janitor,
            gx: 15,
            gy: 8,
            patrol_offsets: vec![(15, 8), (18, 8), (18, 4), (15, 4)],
        },
    ];
    if density >= 2 {
        npcs.push(NpcSpawnDef {
            role: NpcRole::Tourist,
            gx: 6,
            gy: 4,
            patrol_offsets: vec![(6, 4), (10, 4), (10, 7), (6, 7)],
        });
        npcs.push(NpcSpawnDef {
            role: NpcRole::BusinessPerson,
            gx: 14,
            gy: 3,
            patrol_offsets: vec![(14, 3), (16, 3), (16, 6), (14, 6)],
        });
    }
    if density >= 3 {
        npcs.push(NpcSpawnDef {
            role: NpcRole::Passenger,
            gx: 5,
            gy: 8,
            patrol_offsets: vec![(5, 8), (9, 8)],
        });
    }
    npcs
}

fn lounge_npcs() -> Vec<NpcSpawnDef> {
    vec![
        NpcSpawnDef {
            role: NpcRole::BusinessPerson,
            gx: 5,
            gy: 4,
            patrol_offsets: vec![(5, 4), (8, 4), (8, 7), (5, 7)],
        },
        NpcSpawnDef {
            role: NpcRole::Passenger,
            gx: 10,
            gy: 6,
            patrol_offsets: vec![(10, 6), (14, 6)],
        },
    ]
}

fn runway_npcs() -> Vec<NpcSpawnDef> {
    vec![
        NpcSpawnDef {
            role: NpcRole::GroundCrew,
            gx: 5,
            gy: 5,
            patrol_offsets: vec![(5, 5), (15, 5), (15, 15), (5, 15)],
        },
        NpcSpawnDef {
            role: NpcRole::GroundCrew,
            gx: 10,
            gy: 3,
            patrol_offsets: vec![(10, 3), (10, 12)],
        },
    ]
}

fn control_tower_npcs() -> Vec<NpcSpawnDef> {
    vec![NpcSpawnDef {
        role: NpcRole::AirTrafficController,
        gx: 6,
        gy: 4,
        patrol_offsets: vec![(6, 4), (10, 4)],
    }]
}

fn shop_npcs() -> Vec<NpcSpawnDef> {
    vec![NpcSpawnDef {
        role: NpcRole::ShopKeeper,
        gx: 6,
        gy: 3,
        patrol_offsets: vec![(6, 3), (10, 3)],
    }]
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Spawn ambient NPCs when the player enters a new zone.
pub fn spawn_ambient_npcs(
    mut commands: Commands,
    mut transition_events: EventReader<ZoneTransitionEvent>,
    existing: Query<Entity, With<AirportNpc>>,
    calendar: Res<Calendar>,
) {
    for evt in transition_events.read() {
        // Despawn old NPCs
        for entity in &existing {
            commands.entity(entity).despawn_recursive();
        }

        // Density varies by time of day
        let density: u32 = if (calendar.hour >= 7 && calendar.hour <= 10)
            || (calendar.hour >= 17 && calendar.hour <= 20)
        {
            3 // Busy morning/evening
        } else if calendar.is_night() {
            1 // Quiet night
        } else {
            2 // Normal
        };

        let defs = match evt.to_zone {
            MapZone::Terminal => terminal_npcs(density),
            MapZone::Lounge => lounge_npcs(),
            MapZone::Runway => runway_npcs(),
            MapZone::ControlTower => control_tower_npcs(),
            MapZone::Shop => shop_npcs(),
            _ => Vec::new(),
        };

        let mut rng = rand::thread_rng();

        for (i, def) in defs.into_iter().enumerate() {
            let pos = grid_to_world_center(def.gx, def.gy);
            let waypoints: Vec<Vec2> = def
                .patrol_offsets
                .iter()
                .map(|&(gx, gy)| grid_to_world_center(gx, gy))
                .collect();

            let name = format!("{} #{}", def.role.display_name(), i + 1);

            commands.spawn((
                AirportNpc {
                    role: def.role,
                    name,
                },
                NpcPatrol {
                    waypoints,
                    current_index: 0,
                    speed: rng.gen_range(25.0..40.0),
                    wait_timer: 0.0,
                    wait_duration: rng.gen_range(1.5..4.0),
                },
                NpcOneLiner {
                    lines: lines_for_role(def.role),
                },
                Interactable {
                    prompt: format!("[F] Talk to {}", def.role.display_name()),
                    range: 1.5,
                },
                Sprite::from_color(def.role.tint_color(), Vec2::new(12.0, 14.0)),
                Transform::from_xyz(pos.x, pos.y, Z_PLAYER - 1.0),
            ));
        }
    }
}

/// Move NPCs along their patrol paths.
pub fn update_npc_patrol(time: Res<Time>, mut npcs: Query<(&mut NpcPatrol, &mut Transform)>) {
    let dt = time.delta_secs();

    for (mut patrol, mut tf) in &mut npcs {
        if patrol.waypoints.is_empty() {
            continue;
        }

        // Waiting at waypoint
        if patrol.wait_timer > 0.0 {
            patrol.wait_timer -= dt;
            continue;
        }

        let target = patrol.current_target();
        let current = tf.translation.truncate();
        let diff = target - current;
        let dist = diff.length();

        if dist < 2.0 {
            // Reached waypoint, wait then move to next
            patrol.current_index = (patrol.current_index + 1) % patrol.waypoints.len();
            patrol.wait_timer = patrol.wait_duration;
        } else {
            let movement = diff.normalize() * patrol.speed * dt;
            tf.translation.x += movement.x;
            tf.translation.y += movement.y;
        }
    }
}

/// Handle player interaction with ambient NPCs — show a random one-liner.
pub fn interact_ambient_npc(
    player_input: Res<PlayerInput>,
    mut interaction_claimed: ResMut<InteractionClaimed>,
    player_query: Query<&Transform, With<Player>>,
    npc_query: Query<(&Transform, &AirportNpc, &NpcOneLiner, &Interactable)>,
    mut toast: EventWriter<ToastEvent>,
) {
    if !player_input.interact || interaction_claimed.0 {
        return;
    }

    let Ok(player_tf) = player_query.get_single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    for (npc_tf, npc, one_liner, interactable) in &npc_query {
        let dist = player_pos.distance(npc_tf.translation.truncate());
        if dist > interactable.range * TILE_SIZE {
            continue;
        }

        interaction_claimed.0 = true;

        let mut rng = rand::thread_rng();
        let line_idx = rng.gen_range(0..one_liner.lines.len());
        let line = one_liner.lines[line_idx];

        toast.send(ToastEvent {
            message: format!("{}: \"{}\"", npc.name, line),
            duration_secs: 3.0,
        });

        break;
    }
}

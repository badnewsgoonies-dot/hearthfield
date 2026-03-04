//! Player spawn system — per-zone positioning, fade-in, starter setup, respawn.

use bevy::prelude::*;
use crate::shared::*;
use crate::aircraft::fleet::HangarAssignments;

#[derive(Resource, Default)]
pub struct PlayerSpawned(pub bool);

#[derive(Component)]
pub struct SpawnFadeIn {
    pub timer: f32,
    pub duration: f32,
}

const SPAWN_FADE_DURATION: f32 = 0.6;
const STARTER_GOLD: u32 = 500;

pub fn spawn_player(
    mut commands: Commands,
    player_q: Query<Entity, With<Player>>,
    player_location: Res<PlayerLocation>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    if !player_q.is_empty() { return; }

    let spawn_pos = default_spawn_position(&player_location);
    let world_pos = grid_to_world_center(spawn_pos.0, spawn_pos.1);

    commands.spawn((
        Player,
        Sprite::from_color(Color::srgba(0.2, 0.4, 0.8, 0.0), Vec2::new(14.0, 18.0)),
        Transform::from_xyz(world_pos.x, world_pos.y, Z_PLAYER),
        SpawnFadeIn { timer: 0.0, duration: SPAWN_FADE_DURATION },
    ));

    sfx_events.send(PlaySfxEvent { sfx_id: "spawn".to_string() });
}

pub fn animate_spawn_fade(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Sprite, &mut SpawnFadeIn)>,
) {
    for (entity, mut sprite, mut fade) in query.iter_mut() {
        fade.timer += time.delta_secs();
        let t = (fade.timer / fade.duration).min(1.0);
        sprite.color = Color::srgba(0.2, 0.4, 0.8, t);
        if t >= 1.0 {
            commands.entity(entity).remove::<SpawnFadeIn>();
        }
    }
}

pub fn default_spawn_position(location: &PlayerLocation) -> (i32, i32) {
    match location.zone {
        MapZone::Terminal => (10, 8),
        MapZone::Lounge => (8, 6),
        MapZone::Hangar => (6, 8),
        MapZone::Runway => (10, 14),
        MapZone::ControlTower => (5, 5),
        MapZone::CrewQuarters => (5, 5),
        MapZone::Shop => (5, 6),
        MapZone::CityStreet => (10, 8),
    }
}

pub fn transition_spawn_position(to_zone: MapZone, from_zone: MapZone, map_w: i32, map_h: i32) -> (i32, i32) {
    match (to_zone, from_zone) {
        (MapZone::Terminal, MapZone::Runway) => (map_w / 2, 1),
        (MapZone::Terminal, MapZone::CityStreet) => (map_w / 2, map_h - 2),
        (MapZone::Terminal, MapZone::Lounge) => (1, map_h / 2),
        (MapZone::Terminal, MapZone::Hangar) => (map_w - 2, map_h / 2),
        (MapZone::Lounge, MapZone::Terminal) => (18, map_h / 2),
        (MapZone::Lounge, MapZone::CrewQuarters) => (map_w / 2, 1),
        (MapZone::CrewQuarters, MapZone::Lounge) => (map_w / 2, map_h - 2),
        (MapZone::Hangar, MapZone::Terminal) => (1, map_h / 2),
        (MapZone::Runway, MapZone::Terminal) => (map_w / 2, map_h - 2),
        (MapZone::CityStreet, MapZone::Terminal) => (map_w / 2, 1),
        (MapZone::CityStreet, MapZone::Shop) => (map_w - 2, map_h / 2),
        (MapZone::Shop, MapZone::CityStreet) => (1, map_h / 2),
        (MapZone::ControlTower, _) => (map_w / 2, map_h - 2),
        _ => default_spawn_position(&PlayerLocation {
            airport: AirportId::HomeBase,
            zone: to_zone,
        }),
    }
}

pub fn setup_new_game(
    mut gold: ResMut<Gold>,
    mut inventory: ResMut<Inventory>,
    mut fleet: ResMut<Fleet>,
    mut hangars: ResMut<HangarAssignments>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if gold.amount > 0 {
        return; // Already initialized (loaded game or returning from flight)
    }

    gold.amount = STARTER_GOLD;
    inventory.add_item("pilot_manual", 1);
    inventory.add_item("local_map", 1);
    inventory.add_item("granola_bar", 3);
    inventory.add_item("water_bottle", 2);

    if fleet.aircraft.is_empty() {
        let starter = OwnedAircraft {
            aircraft_id: "cessna_172".to_string(),
            nickname: "Old Faithful".to_string(),
            condition: 65.0,
            fuel: 30.0,
            total_flights: 47,
            customizations: Vec::new(),
        };
        fleet.aircraft.push(starter);
        hangars.assign("Old Faithful", AirportId::HomeBase);
    }

    toast_events.send(ToastEvent {
        message: "Welcome to Skywarden! Check the Mission Board to get started.".to_string(),
        duration_secs: 5.0,
    });
}

pub fn respawn_after_day_end(
    mut day_end_events: EventReader<DayEndEvent>,
    mut player_q: Query<(&mut Transform, &mut Sprite), With<Player>>,
    pilot_state: Res<PilotState>,
    mut commands: Commands,
    entity_q: Query<Entity, With<Player>>,
) {
    for _ev in day_end_events.read() {
        let spawn = grid_to_world_center(5, 5);
        if let Ok((mut tf, mut sprite)) = player_q.get_single_mut() {
            tf.translation.x = spawn.x;
            tf.translation.y = spawn.y;
            sprite.color = Color::srgba(0.2, 0.4, 0.8, 0.0);
            if let Ok(entity) = entity_q.get_single() {
                commands.entity(entity).insert(SpawnFadeIn {
                    timer: 0.0,
                    duration: SPAWN_FADE_DURATION,
                });
            }
        }
        let _ = &pilot_state;
    }
}

//! Crew spawning — place NPCs in appropriate zones.

use bevy::prelude::*;
use crate::shared::*;

#[derive(Resource, Default)]
pub struct CrewSpawned {
    pub for_zone: Option<MapZone>,
    pub for_airport: Option<AirportId>,
}

pub fn spawn_crew_for_zone(
    mut commands: Commands,
    player_location: Res<PlayerLocation>,
    crew_registry: Res<CrewRegistry>,
    crew_q: Query<Entity, With<CrewMember>>,
    mut crew_spawned: Local<CrewSpawned>,
    calendar: Res<Calendar>,
) {
    // Only respawn if zone or airport changed
    if crew_spawned.for_zone == Some(player_location.zone)
        && crew_spawned.for_airport == Some(player_location.airport)
    {
        return;
    }

    // Despawn existing crew
    for entity in crew_q.iter() {
        commands.entity(entity).despawn_recursive();
    }

    crew_spawned.for_zone = Some(player_location.zone);
    crew_spawned.for_airport = Some(player_location.airport);

    // Only spawn crew in social zones
    if !matches!(
        player_location.zone,
        MapZone::Lounge | MapZone::Terminal | MapZone::Hangar | MapZone::ControlTower
    ) {
        return;
    }

    let _is_weekend = matches!(
        calendar.day_of_week,
        DayOfWeek::Saturday | DayOfWeek::Sunday
    );

    // Spawn crew members who should be at this location
    let spawn_positions: Vec<(i32, i32)> = match player_location.zone {
        MapZone::Lounge => vec![(5, 5), (5, 7), (10, 5), (10, 7), (14, 5)],
        MapZone::Terminal => vec![(8, 6), (12, 6), (8, 10), (12, 10)],
        MapZone::Hangar => vec![(8, 8), (14, 8)],
        MapZone::ControlTower => vec![(5, 4), (8, 4)],
        _ => vec![],
    };

    let mut placed = 0;
    for (id, member) in crew_registry.members.iter() {
        if placed >= spawn_positions.len() {
            break;
        }

        // Simple schedule: check if NPC's home airport matches or if it's the lounge
        let should_spawn = member.home_airport == player_location.airport
            || (player_location.zone == MapZone::Lounge && placed < 3);

        if should_spawn {
            let (gx, gy) = spawn_positions[placed];
            let pos = grid_to_world_center(gx, gy);
            let tint = Color::srgb(member.tint_color[0], member.tint_color[1], member.tint_color[2]);

            commands.spawn((
                CrewMember { id: id.clone() },
                Sprite::from_color(tint, Vec2::new(14.0, 18.0)),
                Transform::from_xyz(pos.x, pos.y, Z_PLAYER - 1.0),
                Interactable {
                    prompt: format!("[F] Talk to {}", member.name),
                    range: 1.5,
                },
            ));
            placed += 1;
        }
    }
}

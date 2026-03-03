//! Zone transition handling — map swap, player repositioning.

use bevy::prelude::*;
use crate::shared::*;
use super::maps::generate_zone_map;
use super::render::{despawn_map, spawn_zone_tiles};
use super::objects::spawn_zone_objects;

pub fn handle_zone_transition(
    mut events: EventReader<ZoneTransitionEvent>,
    mut commands: Commands,
    tile_q: Query<Entity, With<MapTile>>,
    object_q: Query<Entity, With<WorldObject>>,
    mut player_q: Query<&mut Transform, With<Player>>,
    mut player_location: ResMut<PlayerLocation>,
    mut world_map: ResMut<WorldMap>,
    mut collision_map: ResMut<CollisionMap>,
    mut arrival_events: EventWriter<AirportArrivalEvent>,
    mut music_events: EventWriter<PlayMusicEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    for ev in events.read() {
        let changing_airport = ev.to_airport != player_location.airport;

        // Despawn current map
        despawn_map(&mut commands, &tile_q);
        for entity in object_q.iter() {
            commands.entity(entity).despawn_recursive();
        }

        // Update location
        player_location.airport = ev.to_airport;
        player_location.zone = ev.to_zone;

        // Generate new map
        let (w, h, tiles) = generate_zone_map(ev.to_airport, ev.to_zone);
        spawn_zone_tiles(&mut commands, &tiles, w, h);
        spawn_zone_objects(&mut commands, ev.to_airport, ev.to_zone);

        world_map.width = w;
        world_map.height = h;
        world_map.tiles = tiles;
        collision_map.initialised = false;

        // Reposition player
        if let Ok(mut player_tf) = player_q.get_single_mut() {
            let pos = grid_to_world_center(ev.to_x, ev.to_y);
            player_tf.translation.x = pos.x;
            player_tf.translation.y = pos.y;
        }

        // SFX
        if ev.to_zone.is_indoor() {
            sfx_events.send(PlaySfxEvent {
                sfx_id: "door_open".to_string(),
            });
        }

        // Music change
        let track = if ev.to_zone.is_indoor() {
            "indoor"
        } else {
            match ev.to_airport {
                AirportId::HomeBase => "home",
                AirportId::Frostpeak => "winter",
                AirportId::Sunhaven => "tropical",
                AirportId::Duskhollow => "desert",
                _ => "town",
            }
        };
        music_events.send(PlayMusicEvent {
            track_id: track.to_string(),
            fade_in: true,
        });

        // Airport arrival event
        if changing_airport {
            arrival_events.send(AirportArrivalEvent {
                airport: ev.to_airport,
            });
        }
    }
}

pub fn handle_airport_arrival(
    mut events: EventReader<AirportArrivalEvent>,
    mut pilot_state: ResMut<PilotState>,
    mut play_stats: ResMut<PlayStats>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in events.read() {
        pilot_state.current_airport = ev.airport;

        if !play_stats.airports_visited.contains(&ev.airport) {
            play_stats.airports_visited.push(ev.airport);
            toast_events.send(ToastEvent {
                message: format!("Welcome to {}!", ev.airport.display_name()),
                duration_secs: 4.0,
            });
        }
    }
}

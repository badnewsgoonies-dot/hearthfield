//! Mine entry and exit via MapTransitionEvent.
//!
//! Listens for MapTransitionEvent to MapId::Mine and sets up mine state.
//! Also handles the DayEndEvent to reset mine progress if the player
//! passed out or the day ended while in the mine.

use bevy::prelude::*;

use crate::shared::*;
use super::components::*;

/// System: listen for MapTransitionEvent targeting the Mine.
/// When the player enters the mine, set InMine, configure floor, and either
/// show elevator UI (if unlocked floors exist) or spawn floor 1.
pub fn handle_mine_entry(
    mut map_events: EventReader<MapTransitionEvent>,
    mut mine_state: ResMut<MineState>,
    mut in_mine: ResMut<InMine>,
    mut floor_req: ResMut<FloorSpawnRequest>,
    mut active_floor: ResMut<ActiveFloor>,
    mut elevator_ui: ResMut<ElevatorUiOpen>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    mut music_events: EventWriter<PlayMusicEvent>,
) {
    for event in map_events.read() {
        if event.to_map == MapId::Mine {
            // Entering the mine!
            in_mine.0 = true;

            sfx_events.send(PlaySfxEvent {
                sfx_id: "mine_enter".to_string(),
            });

            music_events.send(PlayMusicEvent {
                track_id: "mine_ambient".to_string(),
                fade_in: true,
            });

            // If player has elevator floors, show elevator selection
            if !mine_state.elevator_floors.is_empty() {
                elevator_ui.0 = true;
                // Don't spawn floor yet; wait for elevator selection
            } else {
                // Start at floor 1
                mine_state.current_floor = 1;
                floor_req.pending = true;
                floor_req.floor = 1;
                active_floor.spawned = false;
            }
        }

        // If transitioning away from mine to something else
        if event.to_map != MapId::Mine && in_mine.0 {
            in_mine.0 = false;
            mine_state.current_floor = 0;
            active_floor.spawned = false;
        }
    }
}

/// System: handle DayEndEvent — if the player is in the mine at end of day,
/// they pass out and get sent back to the mine entrance.
pub fn handle_day_end_in_mine(
    mut day_events: EventReader<DayEndEvent>,
    mut mine_state: ResMut<MineState>,
    mut in_mine: ResMut<InMine>,
    mut active_floor: ResMut<ActiveFloor>,
    mut player_state: ResMut<PlayerState>,
    mut map_events: EventWriter<MapTransitionEvent>,
    mut gold_events: EventWriter<GoldChangeEvent>,
) {
    for _event in day_events.read() {
        if in_mine.0 {
            // Player passed out in the mine — penalty
            let gold_loss = (player_state.gold as f32 * 0.10) as i32;
            if gold_loss > 0 {
                gold_events.send(GoldChangeEvent {
                    amount: -gold_loss,
                    reason: "Passed out in the mine".to_string(),
                });
            }

            // Restore health partially
            player_state.health = player_state.max_health * 0.5;

            // Exit mine
            mine_state.current_floor = 0;
            in_mine.0 = false;
            active_floor.spawned = false;

            map_events.send(MapTransitionEvent {
                to_map: MapId::PlayerHouse,
                to_x: 5,
                to_y: 5,
            });
        }
    }
}

/// System: clean up all mine floor entities when leaving the mine.
pub fn cleanup_mine_on_exit(
    mut commands: Commands,
    in_mine: Res<InMine>,
    entities: Query<Entity, With<super::components::MineFloorEntity>>,
) {
    // If we just left the mine (InMine changed to false), despawn everything.
    // We check if InMine is false but there are still mine entities.
    if !in_mine.0 {
        for entity in entities.iter() {
            commands.entity(entity).despawn();
        }
    }
}

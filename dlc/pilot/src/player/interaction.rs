//! Player interaction and day-end handling.

use bevy::prelude::*;
use crate::shared::*;

/// Check for edge-of-map zone transitions and F-key interactions.
#[allow(clippy::too_many_arguments)]
pub fn check_interactions(
    input: Res<PlayerInput>,
    grid_pos: Res<GridPosition>,
    player_location: Res<PlayerLocation>,
    world_map: Res<WorldMap>,
    mut transition_events: EventWriter<ZoneTransitionEvent>,
    mut interaction_claimed: ResMut<InteractionClaimed>,
    interactable_q: Query<(&WorldObject, &Interactable)>,
    mut dialogue_events: EventWriter<DialogueStartEvent>,
    crew_q: Query<(&CrewMember, &Transform)>,
    player_q: Query<&Transform, With<Player>>,
) {
    // Edge transition detection
    let gx = grid_pos.x;
    let gy = grid_pos.y;
    let w = world_map.width as i32;
    let h = world_map.height as i32;

    if let Some(transition) = edge_transition(gx, gy, w, h, &player_location) {
        transition_events.send(transition);
        return;
    }

    // F-key interaction
    if !input.interact || interaction_claimed.0 {
        return;
    }

    let Ok(player_tf) = player_q.get_single() else {
        return;
    };
    let player_pos = player_tf.translation.truncate();

    // Check crew members first (NPCs take priority)
    for (crew, crew_tf) in crew_q.iter() {
        let dist = player_pos.distance(crew_tf.translation.truncate());
        if dist < TILE_SIZE * 1.5 {
            interaction_claimed.0 = true;
            dialogue_events.send(DialogueStartEvent {
                npc_id: crew.id.clone(),
            });
            return;
        }
    }

    // Check world objects
    for (obj, interactable) in interactable_q.iter() {
        let obj_pos = grid_to_world_center(obj.grid_x, obj.grid_y);
        let dist = player_pos.distance(obj_pos);
        if dist < interactable.range * TILE_SIZE {
            interaction_claimed.0 = true;
            // Object-specific interaction handled by domain systems
            break;
        }
    }
}

/// Detect edge-of-map transitions.
fn edge_transition(
    gx: i32,
    gy: i32,
    w: i32,
    h: i32,
    location: &PlayerLocation,
) -> Option<ZoneTransitionEvent> {
    match location.zone {
        MapZone::Terminal => {
            if gy <= 0 {
                Some(ZoneTransitionEvent {
                    to_airport: location.airport,
                    to_zone: MapZone::Runway,
                    to_x: w / 2,
                    to_y: 1,
                })
            } else if gy >= h - 1 {
                Some(ZoneTransitionEvent {
                    to_airport: location.airport,
                    to_zone: MapZone::CityStreet,
                    to_x: w / 2,
                    to_y: 1,
                })
            } else if gx <= 0 {
                Some(ZoneTransitionEvent {
                    to_airport: location.airport,
                    to_zone: MapZone::Lounge,
                    to_x: 18,
                    to_y: h / 2,
                })
            } else if gx >= w - 1 {
                Some(ZoneTransitionEvent {
                    to_airport: location.airport,
                    to_zone: MapZone::Hangar,
                    to_x: 1,
                    to_y: h / 2,
                })
            } else {
                None
            }
        }
        MapZone::Lounge => {
            if gx >= w - 1 {
                Some(ZoneTransitionEvent {
                    to_airport: location.airport,
                    to_zone: MapZone::Terminal,
                    to_x: 1,
                    to_y: h / 2,
                })
            } else if gy <= 0 {
                Some(ZoneTransitionEvent {
                    to_airport: location.airport,
                    to_zone: MapZone::CrewQuarters,
                    to_x: w / 2,
                    to_y: h - 2,
                })
            } else {
                None
            }
        }
        MapZone::CrewQuarters => {
            if gy >= h - 1 {
                Some(ZoneTransitionEvent {
                    to_airport: location.airport,
                    to_zone: MapZone::Lounge,
                    to_x: w / 2,
                    to_y: 1,
                })
            } else {
                None
            }
        }
        MapZone::Hangar => {
            if gx <= 0 {
                Some(ZoneTransitionEvent {
                    to_airport: location.airport,
                    to_zone: MapZone::Terminal,
                    to_x: w - 2,
                    to_y: h / 2,
                })
            } else {
                None
            }
        }
        MapZone::Runway => {
            if gy >= h - 1 {
                Some(ZoneTransitionEvent {
                    to_airport: location.airport,
                    to_zone: MapZone::Terminal,
                    to_x: w / 2,
                    to_y: 1,
                })
            } else {
                None
            }
        }
        MapZone::CityStreet => {
            if gy <= 0 {
                Some(ZoneTransitionEvent {
                    to_airport: location.airport,
                    to_zone: MapZone::Terminal,
                    to_x: w / 2,
                    to_y: h - 2,
                })
            } else if gx >= w - 1 {
                Some(ZoneTransitionEvent {
                    to_airport: location.airport,
                    to_zone: MapZone::Shop,
                    to_x: 1,
                    to_y: h / 2,
                })
            } else {
                None
            }
        }
        MapZone::Shop => {
            if gx <= 0 {
                Some(ZoneTransitionEvent {
                    to_airport: location.airport,
                    to_zone: MapZone::CityStreet,
                    to_x: w - 2,
                    to_y: h / 2,
                })
            } else {
                None
            }
        }
        MapZone::ControlTower => {
            if gy >= h - 1 {
                Some(ZoneTransitionEvent {
                    to_airport: location.airport,
                    to_zone: MapZone::Terminal,
                    to_x: w / 2,
                    to_y: 1,
                })
            } else {
                None
            }
        }
    }
}

/// Handle day-end: restore stamina, return to crew quarters.
pub fn handle_day_end(
    mut day_end_events: EventReader<DayEndEvent>,
    mut pilot_state: ResMut<PilotState>,
    mut transition_events: EventWriter<ZoneTransitionEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for _ev in day_end_events.read() {
        pilot_state.stamina = pilot_state.max_stamina;

        transition_events.send(ZoneTransitionEvent {
            to_airport: pilot_state.current_airport,
            to_zone: MapZone::CrewQuarters,
            to_x: 5,
            to_y: 5,
        });

        toast_events.send(ToastEvent {
            message: "A new day begins...".to_string(),
            duration_secs: 3.0,
        });
    }
}

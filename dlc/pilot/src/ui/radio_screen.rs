//! Radio communication screen — select and transmit ATC / crew messages.

use crate::shared::*;
use bevy::prelude::*;

// ─── Radio Data ──────────────────────────────────────────────────────────

/// A single selectable radio message.
#[derive(Clone, Debug)]
pub struct RadioEntry {
    pub label: String,
    pub response: String,
}

/// Resource holding the current radio state visible on-screen.
#[derive(Resource, Default)]
pub struct RadioScreenState {
    pub frequency: String,
    pub callsign: String,
    pub messages: Vec<RadioEntry>,
    pub history: Vec<String>,
    pub selected_index: usize,
}

impl RadioScreenState {
    pub fn push_history(&mut self, msg: String) {
        self.history.push(msg);
        if self.history.len() > 5 {
            self.history.remove(0);
        }
    }
}

// ─── Components ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct RadioScreenRoot;

#[derive(Component)]
pub struct RadioMessageList;

#[derive(Component)]
pub struct RadioHistoryText;

#[derive(Component)]
pub struct RadioFrequencyLabel;

// ─── Spawn / Despawn ─────────────────────────────────────────────────────

pub fn spawn_radio_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    mut radio_state: ResMut<RadioScreenState>,
    pilot: Res<PilotState>,
    flight_state: Res<FlightState>,
) {
    // Populate default messages based on flight phase.
    radio_state.frequency = "121.50 MHz".to_string();
    radio_state.callsign = format!("{} {}", pilot.current_airport.icao_code(), "Tower");
    radio_state.selected_index = 0;
    radio_state.messages = build_radio_messages(&flight_state);

    let title_style = TextFont {
        font: font.0.clone(),
        font_size: 20.0,
        ..default()
    };
    let body_style = TextFont {
        font: font.0.clone(),
        font_size: 14.0,
        ..default()
    };

    commands
        .spawn((
            RadioScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.05, 0.1, 0.95)),
        ))
        .with_children(|parent| {
            // Title + frequency
            parent.spawn((
                RadioFrequencyLabel,
                Text::new(format!(
                    "RADIO — {} — {}",
                    radio_state.callsign, radio_state.frequency
                )),
                title_style.clone(),
                TextColor(Color::srgb(0.3, 0.9, 0.4)),
            ));

            // Message list
            let mut list_text = String::new();
            for (i, entry) in radio_state.messages.iter().enumerate() {
                let marker = if i == radio_state.selected_index {
                    "> "
                } else {
                    "  "
                };
                list_text.push_str(&format!("{}{}\n", marker, entry.label));
            }

            parent.spawn((
                RadioMessageList,
                Text::new(list_text),
                body_style.clone(),
                TextColor(Color::srgb(0.8, 0.9, 0.8)),
            ));

            // History
            let history_text = if radio_state.history.is_empty() {
                "— No recent transmissions —".to_string()
            } else {
                radio_state.history.join("\n")
            };

            parent.spawn((
                RadioHistoryText,
                Text::new(history_text),
                body_style,
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

pub fn despawn_radio_screen(mut commands: Commands, query: Query<Entity, With<RadioScreenRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

// ─── Input Handling ──────────────────────────────────────────────────────

pub fn handle_radio_input(
    input: Res<PlayerInput>,
    mut radio_state: ResMut<RadioScreenState>,
    mut list_query: Query<&mut Text, (With<RadioMessageList>, Without<RadioHistoryText>)>,
    mut history_query: Query<&mut Text, (With<RadioHistoryText>, Without<RadioMessageList>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut play_sfx: EventWriter<PlaySfxEvent>,
) {
    if input.cancel {
        next_state.set(GameState::Playing);
        return;
    }

    let msg_count = radio_state.messages.len();
    if msg_count == 0 {
        return;
    }

    // Navigate
    if input.menu_down {
        radio_state.selected_index = (radio_state.selected_index + 1) % msg_count;
    }
    if input.menu_up {
        radio_state.selected_index = radio_state
            .selected_index
            .checked_sub(1)
            .unwrap_or(msg_count - 1);
    }

    // Transmit
    if input.confirm || input.interact {
        let entry = radio_state.messages[radio_state.selected_index].clone();
        radio_state.push_history(format!("YOU: {}", entry.label));
        radio_state.push_history(format!("ATC: {}", entry.response));
        play_sfx.send(PlaySfxEvent {
            sfx_id: "sfx_radio_click".into(),
        });
    }

    // Rebuild list text
    let mut list_text = String::new();
    for (i, entry) in radio_state.messages.iter().enumerate() {
        let marker = if i == radio_state.selected_index {
            "> "
        } else {
            "  "
        };
        list_text.push_str(&format!("{}{}\n", marker, entry.label));
    }
    for mut txt in &mut list_query {
        **txt = list_text.clone();
    }

    // Rebuild history text
    let history_text = if radio_state.history.is_empty() {
        "— No recent transmissions —".to_string()
    } else {
        radio_state.history.join("\n")
    };
    for mut txt in &mut history_query {
        **txt = history_text.clone();
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────

fn build_radio_messages(flight: &FlightState) -> Vec<RadioEntry> {
    match flight.phase {
        FlightPhase::Idle => vec![
            RadioEntry {
                label: "Request taxi clearance".into(),
                response: "Cleared to taxi via Alpha. Hold short runway 27.".into(),
            },
            RadioEntry {
                label: "Request weather briefing".into(),
                response: "Current METAR: winds calm, visibility 10 miles, clear skies.".into(),
            },
        ],
        FlightPhase::Taxi | FlightPhase::Preflight => vec![
            RadioEntry {
                label: "Ready for departure".into(),
                response: "Roger, cleared for takeoff runway 27. Winds 270 at 5.".into(),
            },
            RadioEntry {
                label: "Request hold".into(),
                response: "Hold position, traffic on final.".into(),
            },
        ],
        FlightPhase::Takeoff | FlightPhase::Climb => vec![RadioEntry {
            label: "Departing, climbing to assigned altitude".into(),
            response: "Radar contact. Climb and maintain flight level 180.".into(),
        }],
        FlightPhase::Cruise => vec![
            RadioEntry {
                label: "Request altitude change".into(),
                response: "Approved. Climb to flight level 240.".into(),
            },
            RadioEntry {
                label: "Report position".into(),
                response: "Roger, position noted. Continue on course.".into(),
            },
        ],
        FlightPhase::Descent | FlightPhase::Approach => vec![
            RadioEntry {
                label: "Request ILS approach".into(),
                response: "Cleared ILS runway 09. Descend to 3000.".into(),
            },
            RadioEntry {
                label: "Declare visual".into(),
                response: "Roger, cleared visual approach runway 09.".into(),
            },
        ],
        FlightPhase::Landing => vec![RadioEntry {
            label: "On final".into(),
            response: "Cleared to land runway 09. Wind 090 at 8.".into(),
        }],
        FlightPhase::Arrived => vec![RadioEntry {
            label: "Clear of runway".into(),
            response: "Roger, taxi to gate via Bravo.".into(),
        }],
        FlightPhase::Emergency => vec![
            RadioEntry {
                label: "MAYDAY MAYDAY MAYDAY".into(),
                response: "Roger MAYDAY. Emergency services standing by. State intentions.".into(),
            },
            RadioEntry {
                label: "Requesting emergency landing".into(),
                response: "Cleared for immediate approach any runway. All traffic holding.".into(),
            },
        ],
    }
}

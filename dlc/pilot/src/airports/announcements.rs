//! Airport announcement system — periodic PA messages displayed as toast notifications.

use bevy::prelude::*;
use crate::shared::*;
use rand::Rng;

// ═══════════════════════════════════════════════════════════════════════════
// RESOURCES
// ═══════════════════════════════════════════════════════════════════════════

/// Tracks time until the next PA announcement.
#[derive(Resource)]
pub struct AnnouncementTimer {
    pub timer: f32,
    pub next_interval: f32,
}

impl Default for AnnouncementTimer {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            timer: 0.0,
            next_interval: rng.gen_range(30.0..60.0),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ANNOUNCEMENT TEMPLATES
// ═══════════════════════════════════════════════════════════════════════════

const DESTINATIONS: &[&str] = &[
    "Windport", "Frostpeak", "Sunhaven", "Ironforge", "Cloudmere",
    "Duskhollow", "Stormwatch", "Grand City", "Skyreach",
];

const GATE_NUMBERS: &[&str] = &["1", "2", "3", "4", "5", "6", "7", "8", "12", "15"];

const CREW_NAMES: &[&str] = &[
    "Captain Elena", "Officer Marco", "Navigator Yuki", "Mechanic Hank",
    "Attendant Sofia", "Controller Raj", "Instructor Chen", "Captain Pete",
];

const SECURITY_REMINDERS: &[&str] = &[
    "Please do not leave baggage unattended. Unattended items will be removed.",
    "For your safety, please keep your boarding pass and ID with you at all times.",
    "Smoking is not permitted inside the terminal building.",
    "Please report any suspicious activity to the nearest security officer.",
    "All passengers must proceed through security screening before boarding.",
];

const GENERAL_ANNOUNCEMENTS: &[&str] = &[
    "Welcome to {airport}. We hope you enjoy your journey.",
    "Complimentary Wi-Fi is available throughout the terminal.",
    "Terminal shops and restaurants are open until 22:00.",
    "Electric vehicle charging is available in the short-stay car park.",
    "The observation deck on level 3 offers excellent views of runway operations.",
];

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Generate periodic PA announcements in terminal zones.
pub fn play_announcements(
    time: Res<Time>,
    location: Res<PlayerLocation>,
    weather: Res<WeatherState>,
    pilot: Res<PilotState>,
    mut timer: ResMut<AnnouncementTimer>,
    mut toast: EventWriter<ToastEvent>,
) {
    // Only play in indoor terminal-like zones
    if !matches!(location.zone, MapZone::Terminal | MapZone::Lounge | MapZone::Shop) {
        return;
    }

    timer.timer += time.delta_secs();
    if timer.timer < timer.next_interval {
        return;
    }
    timer.timer = 0.0;

    let mut rng = rand::thread_rng();
    timer.next_interval = rng.gen_range(30.0..60.0);

    let announcement = generate_announcement(&mut rng, &location, &weather, &pilot);

    toast.send(ToastEvent {
        message: format!("📢 {announcement}"),
        duration_secs: 5.0,
    });
}

fn generate_announcement(
    rng: &mut impl Rng,
    location: &PlayerLocation,
    weather: &WeatherState,
    pilot: &PilotState,
) -> String {
    match rng.gen_range(0..7) {
        0 => {
            // Flight boarding
            let dest = DESTINATIONS[rng.gen_range(0..DESTINATIONS.len())];
            let gate = GATE_NUMBERS[rng.gen_range(0..GATE_NUMBERS.len())];
            format!("Flight to {dest} is now boarding at Gate {gate}.")
        }
        1 => {
            // Weather advisory
            let airport = location.airport.display_name();
            match weather.current {
                Weather::Storm => format!(
                    "Attention passengers: {airport} weather advisory — severe weather in effect. Expect delays."
                ),
                Weather::Fog => format!(
                    "Attention passengers: {airport} weather advisory — low visibility conditions. Flights may be delayed."
                ),
                Weather::Snow => format!(
                    "Attention passengers: {airport} weather advisory — snow advisory in effect. De-icing operations underway."
                ),
                Weather::Rain => format!(
                    "Attention passengers: {airport} weather advisory — rain expected. Umbrellas available at the gift shop."
                ),
                _ => format!(
                    "Attention passengers: {airport} weather is currently {:?}. Enjoy your travels.",
                    weather.current
                ),
            }
        }
        2 => {
            // Crew page
            let name = CREW_NAMES[rng.gen_range(0..CREW_NAMES.len())];
            format!("{name}, please report to the operations desk.")
        }
        3 => {
            // Security reminder
            SECURITY_REMINDERS[rng.gen_range(0..SECURITY_REMINDERS.len())].to_string()
        }
        4 => {
            // Gate change
            let dest = DESTINATIONS[rng.gen_range(0..DESTINATIONS.len())];
            let old_gate = rng.gen_range(1..=8);
            let new_gate = rng.gen_range(9..=15);
            format!(
                "Gate change: Flight to {dest} has been moved from Gate {old_gate} to Gate {new_gate}."
            )
        }
        5 => {
            // General
            let template = GENERAL_ANNOUNCEMENTS[rng.gen_range(0..GENERAL_ANNOUNCEMENTS.len())];
            template.replace("{airport}", location.airport.display_name())
        }
        _ => {
            // Player-specific
            let name = &pilot.name;
            format!(
                "{name}, your scheduled departure is approaching. Please proceed to the boarding area."
            )
        }
    }
}

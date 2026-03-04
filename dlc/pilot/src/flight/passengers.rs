//! In-flight passenger management — mood, announcements, service, events.

use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PassengerMood {
    Happy,
    Content,
    Nervous,
    Uncomfortable,
    Scared,
    Angry,
}

impl PassengerMood {
    pub fn display_name(&self) -> &'static str {
        match self {
            PassengerMood::Happy => "😊 Happy",
            PassengerMood::Content => "🙂 Content",
            PassengerMood::Nervous => "😟 Nervous",
            PassengerMood::Uncomfortable => "😣 Uncomfortable",
            PassengerMood::Scared => "😱 Scared",
            PassengerMood::Angry => "😡 Angry",
        }
    }

    pub fn from_satisfaction(satisfaction: f32) -> Self {
        match satisfaction as u32 {
            80..=100 => PassengerMood::Happy,
            60..=79 => PassengerMood::Content,
            45..=59 => PassengerMood::Nervous,
            30..=44 => PassengerMood::Uncomfortable,
            15..=29 => PassengerMood::Scared,
            _ => PassengerMood::Angry,
        }
    }

    pub fn satisfaction_modifier(&self) -> f32 {
        match self {
            PassengerMood::Happy => 0.1,
            PassengerMood::Content => 0.0,
            PassengerMood::Nervous => -0.2,
            PassengerMood::Uncomfortable => -0.5,
            PassengerMood::Scared => -1.0,
            PassengerMood::Angry => -1.5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CabinAnnouncement {
    Welcome,
    Turbulence,
    Descent,
    Landing,
    SeatBelt,
    ServiceRound,
    Delay,
    Weather,
}

impl CabinAnnouncement {
    pub fn text(&self) -> &'static str {
        match self {
            CabinAnnouncement::Welcome => "Welcome aboard! Our flight time today will be approximately...",
            CabinAnnouncement::Turbulence => "We're experiencing some turbulence. Please fasten your seatbelts and remain seated.",
            CabinAnnouncement::Descent => "We've begun our descent. Please return your seats to the upright position.",
            CabinAnnouncement::Landing => "We'll be landing shortly. Please ensure your seatbelt is securely fastened.",
            CabinAnnouncement::SeatBelt => "The captain has turned on the fasten seatbelt sign.",
            CabinAnnouncement::ServiceRound => "Our cabin crew will be coming through with refreshments shortly.",
            CabinAnnouncement::Delay => "We apologize for the delay. We'll have you on your way as soon as possible.",
            CabinAnnouncement::Weather => "If you look out the right side of the aircraft, you can see beautiful views today.",
        }
    }

    pub fn satisfaction_boost(&self) -> f32 {
        match self {
            CabinAnnouncement::Welcome => 3.0,
            CabinAnnouncement::Turbulence => 2.0,
            CabinAnnouncement::Descent => 1.0,
            CabinAnnouncement::Landing => 1.0,
            CabinAnnouncement::SeatBelt => 0.5,
            CabinAnnouncement::ServiceRound => 4.0,
            CabinAnnouncement::Delay => -2.0,
            CabinAnnouncement::Weather => 3.0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum PassengerEvent {
    MedicalEmergency,
    UnrulyPassenger,
    ChildCrying,
    ApplauseOnLanding,
    NervousFlyer,
}

impl PassengerEvent {
    pub fn display_text(&self) -> &'static str {
        match self {
            PassengerEvent::MedicalEmergency => "🚑 A passenger is feeling unwell! Cabin crew responding.",
            PassengerEvent::UnrulyPassenger => "⚠ An unruly passenger is causing a disturbance.",
            PassengerEvent::ChildCrying => "👶 A child is crying in the cabin.",
            PassengerEvent::ApplauseOnLanding => "👏 Passengers applaud the smooth landing!",
            PassengerEvent::NervousFlyer => "😰 A nervous flyer is gripping the armrest tightly.",
        }
    }

    pub fn satisfaction_impact(&self) -> f32 {
        match self {
            PassengerEvent::MedicalEmergency => -10.0,
            PassengerEvent::UnrulyPassenger => -8.0,
            PassengerEvent::ChildCrying => -3.0,
            PassengerEvent::ApplauseOnLanding => 5.0,
            PassengerEvent::NervousFlyer => -1.0,
        }
    }
}

/// In-flight cabin state resource.
#[derive(Resource, Clone, Debug)]
pub struct CabinState {
    pub overall_mood: PassengerMood,
    pub service_rounds_completed: u32,
    pub service_timer: f32,
    pub announcements_made: Vec<CabinAnnouncement>,
    pub events_occurred: Vec<PassengerEvent>,
    pub event_timer: f32,
    pub final_tip_multiplier: f32,
}

impl Default for CabinState {
    fn default() -> Self {
        Self {
            overall_mood: PassengerMood::Content,
            service_rounds_completed: 0,
            service_timer: 0.0,
            announcements_made: Vec::new(),
            events_occurred: Vec::new(),
            event_timer: 0.0,
            final_tip_multiplier: 1.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Update passenger mood based on turbulence, announcements, and service.
pub fn update_passenger_mood(
    time: Res<Time>,
    mut flight_state: ResMut<FlightState>,
    weather: Res<WeatherState>,
    mut cabin: ResMut<CabinState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if !matches!(
        flight_state.phase,
        FlightPhase::Climb | FlightPhase::Cruise | FlightPhase::Descent | FlightPhase::Approach
    ) {
        return;
    }

    let dt = time.delta_secs();

    // Turbulence affects mood
    let turb_penalty = match weather.turbulence_level {
        TurbulenceLevel::None => 0.0,
        TurbulenceLevel::Light => 0.3,
        TurbulenceLevel::Moderate => 1.5,
        TurbulenceLevel::Severe => 4.0,
    };

    let mood_modifier = cabin.overall_mood.satisfaction_modifier();
    let net_change = (mood_modifier - turb_penalty) * dt;

    // Apply to flight_state's passenger happiness
    let new_happy = (flight_state.passengers_happy + net_change).clamp(0.0, 100.0);
    flight_state.passengers_happy = new_happy;
    let new_mood = PassengerMood::from_satisfaction(new_happy);

    // Notify on mood change
    if new_mood != cabin.overall_mood {
        let prev = cabin.overall_mood;
        cabin.overall_mood = new_mood;

        // Only toast on significant drops
        if (new_mood as u8) > (prev as u8) + 1 {
            toast_events.send(ToastEvent {
                message: format!("Cabin mood: {}", new_mood.display_name()),
                duration_secs: 3.0,
            });
        }
    }
}

/// Automatic cabin announcements at phase changes.
pub fn auto_cabin_announcements(
    _flight_state: Res<FlightState>,
    weather: Res<WeatherState>,
    mut cabin: ResMut<CabinState>,
    mut phase_events: EventReader<FlightPhaseChangeEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in phase_events.read() {
        let announcement = match ev.new_phase {
            FlightPhase::Climb => Some(CabinAnnouncement::Welcome),
            FlightPhase::Descent => Some(CabinAnnouncement::Descent),
            FlightPhase::Approach => Some(CabinAnnouncement::Landing),
            _ => None,
        };

        if let Some(ann) = announcement {
            if !cabin.announcements_made.contains(&ann) {
                cabin.announcements_made.push(ann);
                toast_events.send(ToastEvent {
                    message: format!("📢 {}", ann.text()),
                    duration_secs: 4.0,
                });
            }
        }

        // Turbulence announcement
        if weather.turbulence_level == TurbulenceLevel::Moderate
            || weather.turbulence_level == TurbulenceLevel::Severe
        {
            let ann = CabinAnnouncement::Turbulence;
            if !cabin.announcements_made.contains(&ann) {
                cabin.announcements_made.push(ann);
                toast_events.send(ToastEvent {
                    message: format!("📢 {}", ann.text()),
                    duration_secs: 4.0,
                });
            }
        }
    }
}

/// Periodic service rounds during cruise.
pub fn service_rounds(
    time: Res<Time>,
    flight_state: Res<FlightState>,
    mut cabin: ResMut<CabinState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if flight_state.phase != FlightPhase::Cruise {
        return;
    }

    cabin.service_timer += time.delta_secs();

    // Service round every 3 minutes of cruise
    if cabin.service_timer >= 180.0 {
        cabin.service_timer = 0.0;
        cabin.service_rounds_completed += 1;

        let ann = CabinAnnouncement::ServiceRound;
        toast_events.send(ToastEvent {
            message: format!(
                "📢 {} (Round {})",
                ann.text(),
                cabin.service_rounds_completed
            ),
            duration_secs: 3.0,
        });
    }
}

/// Random passenger events during flight.
pub fn random_passenger_events(
    time: Res<Time>,
    flight_state: Res<FlightState>,
    weather: Res<WeatherState>,
    mut cabin: ResMut<CabinState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if !matches!(
        flight_state.phase,
        FlightPhase::Climb | FlightPhase::Cruise | FlightPhase::Descent
    ) {
        return;
    }

    cabin.event_timer += time.delta_secs();

    // Check for random event every ~5 minutes
    if cabin.event_timer < 300.0 {
        return;
    }
    cabin.event_timer = 0.0;

    let t = time.elapsed_secs();
    let roll = ((t * 7.3).sin().abs() * 100.0) as u32;

    let event = match roll {
        0..=5 => Some(PassengerEvent::MedicalEmergency),
        6..=12 => Some(PassengerEvent::UnrulyPassenger),
        13..=25 => Some(PassengerEvent::ChildCrying),
        26..=35 if weather.turbulence_level != TurbulenceLevel::None => {
            Some(PassengerEvent::NervousFlyer)
        }
        _ => None,
    };

    if let Some(ev) = event {
        toast_events.send(ToastEvent {
            message: ev.display_text().to_string(),
            duration_secs: 4.0,
        });
        cabin.events_occurred.push(ev);
    }
}

/// Calculate final satisfaction score and tip multiplier.
pub fn calculate_final_satisfaction(
    mut flight_complete_events: EventReader<FlightCompleteEvent>,
    mut cabin: ResMut<CabinState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in flight_complete_events.read() {
        // Applause on good landing
        if ev.landing_grade == "Perfect" || ev.landing_grade == "Good" {
            let applause = PassengerEvent::ApplauseOnLanding;
            toast_events.send(ToastEvent {
                message: applause.display_text().to_string(),
                duration_secs: 3.0,
            });
            cabin.events_occurred.push(applause);
        }

        // Service bonus
        let service_bonus = (cabin.service_rounds_completed as f32 * 0.05).min(0.2);
        cabin.final_tip_multiplier = 1.0 + service_bonus;

        // Event penalties
        let mut event_penalty = 0.0_f32;
        for event in &cabin.events_occurred {
            match event {
                PassengerEvent::MedicalEmergency => event_penalty += 0.1,
                PassengerEvent::UnrulyPassenger => event_penalty += 0.08,
                PassengerEvent::ApplauseOnLanding => event_penalty -= 0.15,
                _ => {}
            }
        }
        cabin.final_tip_multiplier -= event_penalty;
        cabin.final_tip_multiplier = cabin.final_tip_multiplier.clamp(0.5, 2.0);

        toast_events.send(ToastEvent {
            message: format!(
                "Cabin report: {} mood, {} service rounds, {:.0}% tip bonus",
                cabin.overall_mood.display_name(),
                cabin.service_rounds_completed,
                (cabin.final_tip_multiplier - 1.0) * 100.0
            ),
            duration_secs: 5.0,
        });

        // Reset for next flight
        *cabin = CabinState::default();
    }
}

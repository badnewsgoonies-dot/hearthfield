//! Crew lounge activities — card games, darts, storytelling, meals.

use crate::shared::*;
use bevy::prelude::*;

pub struct LoungePlugin;

impl Plugin for LoungePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LoungeState>().add_systems(
            Update,
            (
                lounge_interaction.run_if(in_state(GameState::CrewLounge)),
                lounge_conversations.run_if(in_state(GameState::CrewLounge)),
                start_lounge_session.run_if(in_state(GameState::Playing)),
            ),
        );
    }
}

// ── Types ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoungeActivity {
    CardGame,
    DartBoard,
    PoolTable,
    Storytelling,
    MealTogether,
    WatchingTV,
}

impl LoungeActivity {
    pub fn display_name(&self) -> &'static str {
        match self {
            LoungeActivity::CardGame => "Card Game",
            LoungeActivity::DartBoard => "Darts",
            LoungeActivity::PoolTable => "Pool",
            LoungeActivity::Storytelling => "Storytelling",
            LoungeActivity::MealTogether => "Shared Meal",
            LoungeActivity::WatchingTV => "Watching TV",
        }
    }

    pub fn friendship_gain(&self) -> i32 {
        match self {
            LoungeActivity::CardGame => 5,
            LoungeActivity::DartBoard => 4,
            LoungeActivity::PoolTable => 4,
            LoungeActivity::Storytelling => 7,
            LoungeActivity::MealTogether => 8,
            LoungeActivity::WatchingTV => 2,
        }
    }

    pub fn duration_secs(&self) -> f32 {
        match self {
            LoungeActivity::CardGame => 10.0,
            LoungeActivity::DartBoard => 8.0,
            LoungeActivity::PoolTable => 10.0,
            LoungeActivity::Storytelling => 12.0,
            LoungeActivity::MealTogether => 15.0,
            LoungeActivity::WatchingTV => 6.0,
        }
    }

    fn from_index(i: usize) -> Self {
        match i % 6 {
            0 => LoungeActivity::CardGame,
            1 => LoungeActivity::DartBoard,
            2 => LoungeActivity::PoolTable,
            3 => LoungeActivity::Storytelling,
            4 => LoungeActivity::MealTogether,
            _ => LoungeActivity::WatchingTV,
        }
    }
}

#[derive(Clone, Debug)]
pub struct LoungeEvent {
    pub description: String,
    pub friendship_bonus: i32,
    pub hint: Option<String>,
}

// ── State ────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct LoungeState {
    pub available_activities: Vec<LoungeActivity>,
    pub selected_activity: usize,
    pub active_session: Option<ActiveLoungeSession>,
    pub present_crew: Vec<String>,
    pub conversation_timer: f32,
}

pub struct ActiveLoungeSession {
    pub activity: LoungeActivity,
    pub partner_id: String,
    pub time_remaining: f32,
    pub mini_events_triggered: u32,
}

impl LoungeState {
    pub fn populate_activities(&mut self) {
        self.available_activities = vec![
            LoungeActivity::CardGame,
            LoungeActivity::DartBoard,
            LoungeActivity::PoolTable,
            LoungeActivity::Storytelling,
            LoungeActivity::MealTogether,
            LoungeActivity::WatchingTV,
        ];
        self.selected_activity = 0;
    }
}

// Crew tips overheard in lounge
const CREW_TIPS: &[&str] = &[
    "I heard Frostpeak has nasty crosswinds after noon...",
    "The mechanic at Grand City does top-notch work.",
    "VIP passengers tip better if you land smoothly.",
    "Captain Elena trained at the best flight school in the region.",
    "If you get ice building up, descend to warmer air.",
    "The sunset flights from Sunhaven are spectacular.",
    "Stormwatch airport has the best weather radar equipment.",
    "Flying at night earns bonus XP for night-rated flights.",
    "Check your tires before every flight — trust me on that.",
    "The cargo runs to Ironforge pay really well.",
];

// ── Systems ──────────────────────────────────────────────────────────────

pub fn start_lounge_session(
    input: Res<PlayerInput>,
    player_location: Res<PlayerLocation>,
    mut game_state: ResMut<NextState<GameState>>,
    mut lounge: ResMut<LoungeState>,
) {
    if player_location.zone != MapZone::Lounge && player_location.zone != MapZone::CrewQuarters {
        return;
    }

    if !input.interact {
        return;
    }

    // Populate crew present at this airport
    lounge.present_crew = CREW_IDS.iter().take(4).map(|s| s.to_string()).collect();
    lounge.populate_activities();
    lounge.conversation_timer = 0.0;

    game_state.set(GameState::CrewLounge);
}

pub fn lounge_interaction(
    time: Res<Time>,
    input: Res<PlayerInput>,
    mut lounge: ResMut<LoungeState>,
    mut relationships: ResMut<Relationships>,
    mut toast_events: EventWriter<ToastEvent>,
    mut friendship_events: EventWriter<FriendshipChangeEvent>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    // Navigate activity selection
    if lounge.active_session.is_none() {
        if input.menu_up && lounge.selected_activity > 0 {
            lounge.selected_activity -= 1;
        }
        if input.menu_down && lounge.selected_activity + 1 < lounge.available_activities.len() {
            lounge.selected_activity += 1;
        }

        // Start activity
        if input.menu_confirm {
            if let Some(&activity) = lounge.available_activities.get(lounge.selected_activity) {
                let partner = lounge
                    .present_crew
                    .first()
                    .cloned()
                    .unwrap_or_else(|| "copilot_marco".to_string());

                toast_events.send(ToastEvent {
                    message: format!("Starting {} with {}...", activity.display_name(), partner),
                    duration_secs: 3.0,
                });

                lounge.active_session = Some(ActiveLoungeSession {
                    activity,
                    partner_id: partner,
                    time_remaining: activity.duration_secs(),
                    mini_events_triggered: 0,
                });
            }
        }

        // Leave lounge
        if input.cancel {
            game_state.set(GameState::Playing);
            return;
        }

        return;
    }

    // Active session processing
    let dt = time.delta_secs();
    let (finished, partner_id, activity) = {
        let session = lounge.active_session.as_mut().unwrap();
        session.time_remaining -= dt;

        // Mini-events during the activity
        let trigger_threshold = session.activity.duration_secs() * 0.5;
        if session.time_remaining <= trigger_threshold && session.mini_events_triggered == 0 {
            session.mini_events_triggered = 1;
            let event = generate_mini_event(session.activity);
            toast_events.send(ToastEvent {
                message: event.description.clone(),
                duration_secs: 3.0,
            });
        }

        (
            session.time_remaining <= 0.0,
            session.partner_id.clone(),
            session.activity,
        )
    };

    if finished {
        let gain = activity.friendship_gain();
        relationships.add_friendship(&partner_id, gain);

        friendship_events.send(FriendshipChangeEvent {
            npc_id: partner_id.clone(),
            amount: gain,
        });

        toast_events.send(ToastEvent {
            message: format!(
                "{} finished! {} +{} friendship",
                activity.display_name(),
                partner_id,
                gain
            ),
            duration_secs: 3.0,
        });

        lounge.active_session = None;
    }
}

fn generate_mini_event(activity: LoungeActivity) -> LoungeEvent {
    match activity {
        LoungeActivity::CardGame => LoungeEvent {
            description: "A tense hand! You bluff your way to victory.".to_string(),
            friendship_bonus: 2,
            hint: None,
        },
        LoungeActivity::DartBoard => LoungeEvent {
            description: "Bullseye! The crew cheers.".to_string(),
            friendship_bonus: 1,
            hint: None,
        },
        LoungeActivity::PoolTable => LoungeEvent {
            description: "Great combo shot! Very impressive.".to_string(),
            friendship_bonus: 1,
            hint: None,
        },
        LoungeActivity::Storytelling => LoungeEvent {
            description: "A crew member shares a story from their early flying days.".to_string(),
            friendship_bonus: 3,
            hint: Some("Remember to file your flight plan before each flight.".to_string()),
        },
        LoungeActivity::MealTogether => LoungeEvent {
            description: "Great food and great company!".to_string(),
            friendship_bonus: 2,
            hint: None,
        },
        LoungeActivity::WatchingTV => LoungeEvent {
            description: "An aviation documentary comes on. Everyone's glued to the screen."
                .to_string(),
            friendship_bonus: 1,
            hint: Some("Night flights earn XP bonuses.".to_string()),
        },
    }
}

pub fn lounge_conversations(
    time: Res<Time>,
    mut lounge: ResMut<LoungeState>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    if lounge.active_session.is_some() {
        return;
    }

    lounge.conversation_timer += time.delta_secs();

    // Random crew chatter every ~20 seconds while idle in lounge
    if lounge.conversation_timer >= 20.0 {
        lounge.conversation_timer = 0.0;
        let idx = (time.elapsed_secs() * 3.3) as usize % CREW_TIPS.len();
        toast_events.send(ToastEvent {
            message: format!("💬 Overheard: {}", CREW_TIPS[idx]),
            duration_secs: 4.0,
        });
    }
}

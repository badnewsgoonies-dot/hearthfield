//! Deep relationship system — phases, gifts, decay, rival mechanics.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::shared::*;

pub struct RelationshipPlugin;

impl Plugin for RelationshipPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RelationshipDetails>()
            .add_systems(
                Update,
                (
                    process_gift,
                    friendship_decay,
                    evaluate_relationship_phase,
                    check_rival_mechanic,
                    unlock_friendship_bonuses,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

// ── Types ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipPhase {
    Stranger,
    Acquaintance,
    Friend,
    CloseFriend,
    BestFriend,
    Rival,
}

impl RelationshipPhase {
    pub fn display_name(&self) -> &'static str {
        match self {
            RelationshipPhase::Stranger => "Stranger",
            RelationshipPhase::Acquaintance => "Acquaintance",
            RelationshipPhase::Friend => "Friend",
            RelationshipPhase::CloseFriend => "Close Friend",
            RelationshipPhase::BestFriend => "Best Friend",
            RelationshipPhase::Rival => "Rival",
        }
    }

    pub fn from_friendship(level: i32) -> Self {
        match level {
            i if i <= -30 => RelationshipPhase::Rival,
            -29..=9 => RelationshipPhase::Stranger,
            10..=29 => RelationshipPhase::Acquaintance,
            30..=54 => RelationshipPhase::Friend,
            55..=79 => RelationshipPhase::CloseFriend,
            _ => RelationshipPhase::BestFriend,
        }
    }
}

// Gift preference categories
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GiftReaction {
    Loved,
    Liked,
    Neutral,
    Disliked,
    Hated,
}

impl GiftReaction {
    pub fn friendship_change(&self) -> i32 {
        match self {
            GiftReaction::Loved => 20,
            GiftReaction::Liked => 10,
            GiftReaction::Neutral => 3,
            GiftReaction::Disliked => -5,
            GiftReaction::Hated => -15,
        }
    }

    pub fn display_text(&self) -> &'static str {
        match self {
            GiftReaction::Loved => "loves",
            GiftReaction::Liked => "likes",
            GiftReaction::Neutral => "accepts",
            GiftReaction::Disliked => "doesn't like",
            GiftReaction::Hated => "hates",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CrewAbility {
    pub npc_id: String,
    pub name: String,
    pub description: String,
    pub required_phase: RelationshipPhase,
}

// ── State ────────────────────────────────────────────────────────────────

#[derive(Resource, Default, Clone, Serialize, Deserialize)]
pub struct RelationshipDetails {
    pub phases: Vec<(String, RelationshipPhase)>,
    pub backstory_seen: Vec<String>,
    pub abilities_unlocked: Vec<String>,
    pub decay_timer: f32,
    pub rival_pairs: Vec<(String, String)>,
}

impl RelationshipDetails {
    pub fn phase_for(&self, npc_id: &str) -> RelationshipPhase {
        self.phases
            .iter()
            .find(|(id, _)| id == npc_id)
            .map(|(_, p)| *p)
            .unwrap_or(RelationshipPhase::Stranger)
    }

    pub fn set_phase(&mut self, npc_id: &str, phase: RelationshipPhase) {
        if let Some(entry) = self.phases.iter_mut().find(|(id, _)| id == npc_id) {
            entry.1 = phase;
        } else {
            self.phases.push((npc_id.to_string(), phase));
        }
    }

    pub fn is_ability_unlocked(&self, npc_id: &str) -> bool {
        self.abilities_unlocked.iter().any(|id| id == npc_id)
    }
}

fn crew_abilities() -> Vec<CrewAbility> {
    vec![
        CrewAbility {
            npc_id: "captain_elena".to_string(),
            name: "Mentor's Guidance".to_string(),
            description: "+10% XP from flights".to_string(),
            required_phase: RelationshipPhase::CloseFriend,
        },
        CrewAbility {
            npc_id: "copilot_marco".to_string(),
            name: "Co-pilot Assist".to_string(),
            description: "Autopilot holds better in turbulence".to_string(),
            required_phase: RelationshipPhase::Friend,
        },
        CrewAbility {
            npc_id: "mechanic_hank".to_string(),
            name: "Expert Maintenance".to_string(),
            description: "Repairs cost 20% less".to_string(),
            required_phase: RelationshipPhase::CloseFriend,
        },
        CrewAbility {
            npc_id: "navigator_yuki".to_string(),
            name: "Precise Navigation".to_string(),
            description: "Fuel efficiency +10%".to_string(),
            required_phase: RelationshipPhase::Friend,
        },
    ]
}

// Rival pairs: befriending one may upset the other
const RIVAL_PAIRS: &[(&str, &str)] = &[
    ("captain_elena", "veteran_pete"),
    ("copilot_marco", "rookie_alex"),
];

// ── Systems ──────────────────────────────────────────────────────────────

pub fn process_gift(
    mut gift_events: EventReader<GiftGivenEvent>,
    crew_registry: Res<CrewRegistry>,
    mut relationships: ResMut<Relationships>,
    mut details: ResMut<RelationshipDetails>,
    mut toast_events: EventWriter<ToastEvent>,
    mut friendship_events: EventWriter<FriendshipChangeEvent>,
) {
    for ev in gift_events.read() {
        let Some(member) = crew_registry.members.get(&ev.npc_id) else {
            continue;
        };

        let reaction = if ev.item_id == member.favorite_gift {
            GiftReaction::Loved
        } else if ev.item_id == member.disliked_gift {
            GiftReaction::Hated
        } else {
            GiftReaction::Liked
        };

        let change = reaction.friendship_change();
        relationships.add_friendship(&ev.npc_id, change);

        // Update phase
        let level = relationships.friendship_level(&ev.npc_id);
        let phase = RelationshipPhase::from_friendship(level);
        details.set_phase(&ev.npc_id, phase);

        friendship_events.send(FriendshipChangeEvent {
            npc_id: ev.npc_id.clone(),
            amount: change,
        });

        toast_events.send(ToastEvent {
            message: format!("{} {} the gift! ({:+})", member.name, reaction.display_text(), change),
            duration_secs: 3.0,
        });
    }
}

pub fn friendship_decay(
    time: Res<Time>,
    mut day_end_events: EventReader<DayEndEvent>,
    mut relationships: ResMut<Relationships>,
    mut details: ResMut<RelationshipDetails>,
) {
    // Only decay on day end
    for _ev in day_end_events.read() {
        let ids: Vec<String> = relationships.friendship.keys().cloned().collect();
        for npc_id in &ids {
            let level = relationships.friendship_level(npc_id);
            if level > 0 {
                // Slow decay: -1 per day for non-interacted NPCs
                if !relationships.gifts_given_today.get(npc_id).copied().unwrap_or(false) {
                    relationships.add_friendship(npc_id, -1);
                }
            }
            let new_level = relationships.friendship_level(npc_id);
            details.set_phase(npc_id, RelationshipPhase::from_friendship(new_level));
        }
    }

    let _ = time; // Timer tracking for future use
}

pub fn evaluate_relationship_phase(
    relationships: Res<Relationships>,
    mut details: ResMut<RelationshipDetails>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for npc_id in CREW_IDS {
        let level = relationships.friendship_level(npc_id);
        let new_phase = RelationshipPhase::from_friendship(level);
        let old_phase = details.phase_for(npc_id);

        if new_phase != old_phase {
            details.set_phase(npc_id, new_phase);

            // Notify on significant phase changes
            if matches!(
                new_phase,
                RelationshipPhase::Friend | RelationshipPhase::CloseFriend | RelationshipPhase::BestFriend
            ) {
                toast_events.send(ToastEvent {
                    message: format!(
                        "Relationship with {} is now: {}",
                        npc_id,
                        new_phase.display_name()
                    ),
                    duration_secs: 4.0,
                });
            }

            if new_phase == RelationshipPhase::Rival {
                toast_events.send(ToastEvent {
                    message: format!("⚠ {} has become a rival!", npc_id),
                    duration_secs: 4.0,
                });
            }
        }
    }
}

pub fn check_rival_mechanic(
    mut friendship_events: EventReader<FriendshipChangeEvent>,
    mut relationships: ResMut<Relationships>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in friendship_events.read() {
        if ev.amount <= 0 {
            continue;
        }

        // Check if befriending this NPC upsets a rival pair partner
        for &(a, b) in RIVAL_PAIRS {
            let (friend, rival) = if ev.npc_id == a {
                (a, b)
            } else if ev.npc_id == b {
                (b, a)
            } else {
                continue;
            };

            let rival_loss = -(ev.amount / 3).max(1);
            relationships.add_friendship(rival, rival_loss);

            toast_events.send(ToastEvent {
                message: format!(
                    "{} seems bothered by your friendship with {}... ({:+})",
                    rival, friend, rival_loss
                ),
                duration_secs: 3.0,
            });
        }
    }
}

pub fn unlock_friendship_bonuses(
    relationships: Res<Relationships>,
    mut details: ResMut<RelationshipDetails>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ability in crew_abilities() {
        if details.is_ability_unlocked(&ability.npc_id) {
            continue;
        }

        let phase = details.phase_for(&ability.npc_id);
        if phase as u8 >= ability.required_phase as u8
            && !matches!(phase, RelationshipPhase::Rival | RelationshipPhase::Stranger)
        {
            details.abilities_unlocked.push(ability.npc_id.clone());
            toast_events.send(ToastEvent {
                message: format!(
                    "🔓 Unlocked {}'s ability: {} — {}",
                    ability.npc_id, ability.name, ability.description
                ),
                duration_secs: 5.0,
            });
        }
    }

    let _ = &relationships; // Used for phase queries
}

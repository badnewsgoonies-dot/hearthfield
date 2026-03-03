//! Story / campaign missions — 10-chapter narrative arc.
//!
//! Each chapter has 3–5 missions that must be completed in order. Story progress
//! is tracked in the `StoryProgress` resource and displayed in the logbook.

use bevy::prelude::*;
use crate::shared::*;

// ─── Story Chapters ──────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum StoryChapter {
    Chapter1FirstSolo,
    Chapter2MaidenVoyage,
    Chapter3StormWarning,
    Chapter4CargoKing,
    Chapter5IslandHopper,
    Chapter6HighAndMighty,
    Chapter7NightRider,
    Chapter8TheBigEmergency,
    Chapter9FullCircle,
    Chapter10LegendaryPilot,
}

impl StoryChapter {
    pub fn index(&self) -> usize {
        *self as usize
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::Chapter1FirstSolo => "Chapter 1: First Wings",
            Self::Chapter2MaidenVoyage => "Chapter 2: Maiden Voyage",
            Self::Chapter3StormWarning => "Chapter 3: Storm Warning",
            Self::Chapter4CargoKing => "Chapter 4: Cargo King",
            Self::Chapter5IslandHopper => "Chapter 5: Island Hopper",
            Self::Chapter6HighAndMighty => "Chapter 6: High and Mighty",
            Self::Chapter7NightRider => "Chapter 7: Night Rider",
            Self::Chapter8TheBigEmergency => "Chapter 8: The Big Emergency",
            Self::Chapter9FullCircle => "Chapter 9: Full Circle",
            Self::Chapter10LegendaryPilot => "Chapter 10: Wings of Legend",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Chapter1FirstSolo => "Training flights at HomeBase with instructor_chen. Learn the basics of flight.",
            Self::Chapter2MaidenVoyage => "Your first passenger flight — take travelers to Grandcity.",
            Self::Chapter3StormWarning => "A storm forces you to divert. Emergency landing at Stormwatch.",
            Self::Chapter4CargoKing => "Build a cargo route to Ironforge and prove your reliability.",
            Self::Chapter5IslandHopper => "Seaplane flights to CoralCay (Sunhaven) through island chains.",
            Self::Chapter6HighAndMighty => "Tackle the challenging approach into Skyreach at high altitude.",
            Self::Chapter7NightRider => "Night mail runs through fog — instrument skills tested.",
            Self::Chapter8TheBigEmergency => "Handle a major in-flight emergency and bring everyone home safe.",
            Self::Chapter9FullCircle => "Visit every airport in a single day — a test of endurance.",
            Self::Chapter10LegendaryPilot => "The final challenge. Earn the Ace rank and become a legend.",
        }
    }

    pub fn required_rank(&self) -> PilotRank {
        match self {
            Self::Chapter1FirstSolo => PilotRank::Student,
            Self::Chapter2MaidenVoyage => PilotRank::Student,
            Self::Chapter3StormWarning => PilotRank::Private,
            Self::Chapter4CargoKing => PilotRank::Private,
            Self::Chapter5IslandHopper => PilotRank::Commercial,
            Self::Chapter6HighAndMighty => PilotRank::Commercial,
            Self::Chapter7NightRider => PilotRank::Senior,
            Self::Chapter8TheBigEmergency => PilotRank::Senior,
            Self::Chapter9FullCircle => PilotRank::Captain,
            Self::Chapter10LegendaryPilot => PilotRank::Captain,
        }
    }

    pub fn next(&self) -> Option<StoryChapter> {
        match self {
            Self::Chapter1FirstSolo => Some(Self::Chapter2MaidenVoyage),
            Self::Chapter2MaidenVoyage => Some(Self::Chapter3StormWarning),
            Self::Chapter3StormWarning => Some(Self::Chapter4CargoKing),
            Self::Chapter4CargoKing => Some(Self::Chapter5IslandHopper),
            Self::Chapter5IslandHopper => Some(Self::Chapter6HighAndMighty),
            Self::Chapter6HighAndMighty => Some(Self::Chapter7NightRider),
            Self::Chapter7NightRider => Some(Self::Chapter8TheBigEmergency),
            Self::Chapter8TheBigEmergency => Some(Self::Chapter9FullCircle),
            Self::Chapter9FullCircle => Some(Self::Chapter10LegendaryPilot),
            Self::Chapter10LegendaryPilot => None,
        }
    }

    pub fn all() -> &'static [StoryChapter] {
        &[
            Self::Chapter1FirstSolo,
            Self::Chapter2MaidenVoyage,
            Self::Chapter3StormWarning,
            Self::Chapter4CargoKing,
            Self::Chapter5IslandHopper,
            Self::Chapter6HighAndMighty,
            Self::Chapter7NightRider,
            Self::Chapter8TheBigEmergency,
            Self::Chapter9FullCircle,
            Self::Chapter10LegendaryPilot,
        ]
    }
}

// ─── Story Missions ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct StoryMission {
    pub id: &'static str,
    pub chapter: StoryChapter,
    pub title: &'static str,
    pub description: &'static str,
    pub origin: AirportId,
    pub destination: AirportId,
    pub reward_gold: u32,
    pub reward_xp: u32,
    pub cutscene_id: Option<&'static str>,
    pub dialogue_npc: Option<&'static str>,
    pub required_weather: Option<Weather>,
    pub required_time_of_day: Option<TimeRequirement>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TimeRequirement {
    Day,
    Night,
}

/// All story missions in the game.
pub fn all_story_missions() -> Vec<StoryMission> {
    vec![
        // ── Chapter 1: First Wings ──────────────────────────────────────
        StoryMission {
            id: "c1_first_taxi",
            chapter: StoryChapter::Chapter1FirstSolo,
            title: "Taxi Practice",
            description: "Learn to taxi the Cessna around HomeBase with instructor Chen.",
            origin: AirportId::HomeBase,
            destination: AirportId::HomeBase,
            reward_gold: 50,
            reward_xp: 20,
            cutscene_id: Some("intro_chen"),
            dialogue_npc: Some("instructor_chen"),
            required_weather: None,
            required_time_of_day: Some(TimeRequirement::Day),
        },
        StoryMission {
            id: "c1_first_takeoff",
            chapter: StoryChapter::Chapter1FirstSolo,
            title: "First Takeoff",
            description: "Take off and fly a traffic pattern around the home field.",
            origin: AirportId::HomeBase,
            destination: AirportId::HomeBase,
            reward_gold: 100,
            reward_xp: 40,
            cutscene_id: None,
            dialogue_npc: Some("instructor_chen"),
            required_weather: Some(Weather::Clear),
            required_time_of_day: Some(TimeRequirement::Day),
        },
        StoryMission {
            id: "c1_cross_country",
            chapter: StoryChapter::Chapter1FirstSolo,
            title: "Cross-Country",
            description: "Fly to Windport and back — your first cross-country solo!",
            origin: AirportId::HomeBase,
            destination: AirportId::Windport,
            reward_gold: 200,
            reward_xp: 80,
            cutscene_id: Some("solo_celebration"),
            dialogue_npc: Some("instructor_chen"),
            required_weather: None,
            required_time_of_day: None,
        },
        // ── Chapter 2: Maiden Voyage ────────────────────────────────────
        StoryMission {
            id: "c2_first_pax",
            chapter: StoryChapter::Chapter2MaidenVoyage,
            title: "First Passengers",
            description: "Carry your first paying passengers to Grandcity.",
            origin: AirportId::HomeBase,
            destination: AirportId::Grandcity,
            reward_gold: 400,
            reward_xp: 120,
            cutscene_id: Some("maiden_voyage_intro"),
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: None,
        },
        StoryMission {
            id: "c2_return_home",
            chapter: StoryChapter::Chapter2MaidenVoyage,
            title: "Return Trip",
            description: "Bring passengers back from Grandcity.",
            origin: AirportId::Grandcity,
            destination: AirportId::HomeBase,
            reward_gold: 350,
            reward_xp: 100,
            cutscene_id: None,
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: None,
        },
        StoryMission {
            id: "c2_vip_charter",
            chapter: StoryChapter::Chapter2MaidenVoyage,
            title: "VIP Charter",
            description: "A celebrity wants a private flight to Sunhaven.",
            origin: AirportId::Grandcity,
            destination: AirportId::Sunhaven,
            reward_gold: 600,
            reward_xp: 150,
            cutscene_id: Some("vip_intro"),
            dialogue_npc: Some("charter_diana"),
            required_weather: None,
            required_time_of_day: None,
        },
        // ── Chapter 3: Storm Warning ────────────────────────────────────
        StoryMission {
            id: "c3_weather_flight",
            chapter: StoryChapter::Chapter3StormWarning,
            title: "Into the Clouds",
            description: "Fly through deteriorating weather toward Frostpeak.",
            origin: AirportId::HomeBase,
            destination: AirportId::Frostpeak,
            reward_gold: 500,
            reward_xp: 200,
            cutscene_id: Some("storm_buildup"),
            dialogue_npc: None,
            required_weather: Some(Weather::Cloudy),
            required_time_of_day: None,
        },
        StoryMission {
            id: "c3_divert",
            chapter: StoryChapter::Chapter3StormWarning,
            title: "Emergency Divert",
            description: "Storm closes Frostpeak — divert to Stormwatch!",
            origin: AirportId::Frostpeak,
            destination: AirportId::Stormwatch,
            reward_gold: 800,
            reward_xp: 300,
            cutscene_id: Some("storm_divert"),
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: None,
        },
        StoryMission {
            id: "c3_aftermath",
            chapter: StoryChapter::Chapter3StormWarning,
            title: "After the Storm",
            description: "Return home after the storm passes. Reflect on lessons learned.",
            origin: AirportId::Stormwatch,
            destination: AirportId::HomeBase,
            reward_gold: 300,
            reward_xp: 100,
            cutscene_id: Some("storm_reflection"),
            dialogue_npc: Some("veteran_pete"),
            required_weather: Some(Weather::Clear),
            required_time_of_day: None,
        },
        // ── Chapter 4: Cargo King ───────────────────────────────────────
        StoryMission {
            id: "c4_first_cargo",
            chapter: StoryChapter::Chapter4CargoKing,
            title: "First Freight",
            description: "Establish your cargo route: fly machine parts to Ironforge.",
            origin: AirportId::HomeBase,
            destination: AirportId::Ironforge,
            reward_gold: 600,
            reward_xp: 200,
            cutscene_id: Some("cargo_intro"),
            dialogue_npc: Some("mechanic_hank"),
            required_weather: None,
            required_time_of_day: None,
        },
        StoryMission {
            id: "c4_return_cargo",
            chapter: StoryChapter::Chapter4CargoKing,
            title: "Return Load",
            description: "Carry refined steel back from Ironforge.",
            origin: AirportId::Ironforge,
            destination: AirportId::HomeBase,
            reward_gold: 550,
            reward_xp: 180,
            cutscene_id: None,
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: None,
        },
        StoryMission {
            id: "c4_heavy_haul",
            chapter: StoryChapter::Chapter4CargoKing,
            title: "Heavy Haul",
            description: "Max-weight cargo run to Grandcity — weight and balance critical.",
            origin: AirportId::Ironforge,
            destination: AirportId::Grandcity,
            reward_gold: 1000,
            reward_xp: 350,
            cutscene_id: None,
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: None,
        },
        // ── Chapter 5: Island Hopper ────────────────────────────────────
        StoryMission {
            id: "c5_seaplane_intro",
            chapter: StoryChapter::Chapter5IslandHopper,
            title: "Water Wings",
            description: "Check out the seaplane at Sunhaven and learn water landings.",
            origin: AirportId::Sunhaven,
            destination: AirportId::Sunhaven,
            reward_gold: 300,
            reward_xp: 150,
            cutscene_id: Some("seaplane_intro"),
            dialogue_npc: Some("instructor_chen"),
            required_weather: None,
            required_time_of_day: Some(TimeRequirement::Day),
        },
        StoryMission {
            id: "c5_island_run",
            chapter: StoryChapter::Chapter5IslandHopper,
            title: "Island Supplies",
            description: "Deliver supplies to remote island communities via seaplane.",
            origin: AirportId::Sunhaven,
            destination: AirportId::Duskhollow,
            reward_gold: 700,
            reward_xp: 250,
            cutscene_id: None,
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: None,
        },
        StoryMission {
            id: "c5_coral_rescue",
            chapter: StoryChapter::Chapter5IslandHopper,
            title: "Coral Rescue",
            description: "Rescue stranded researchers from a coral atoll.",
            origin: AirportId::Sunhaven,
            destination: AirportId::Sunhaven,
            reward_gold: 900,
            reward_xp: 300,
            cutscene_id: Some("coral_rescue"),
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: None,
        },
        // ── Chapter 6: High and Mighty ──────────────────────────────────
        StoryMission {
            id: "c6_skyreach_prep",
            chapter: StoryChapter::Chapter6HighAndMighty,
            title: "High Altitude Prep",
            description: "Fly to Cloudmere for high-altitude training before Skyreach.",
            origin: AirportId::HomeBase,
            destination: AirportId::Cloudmere,
            reward_gold: 500,
            reward_xp: 200,
            cutscene_id: None,
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: None,
        },
        StoryMission {
            id: "c6_skyreach_approach",
            chapter: StoryChapter::Chapter6HighAndMighty,
            title: "The Skyreach Approach",
            description: "Navigate the treacherous mountain approach into Skyreach.",
            origin: AirportId::Cloudmere,
            destination: AirportId::Skyreach,
            reward_gold: 1200,
            reward_xp: 400,
            cutscene_id: Some("skyreach_dramatic"),
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: None,
        },
        // ── Chapter 7: Night Rider ──────────────────────────────────────
        StoryMission {
            id: "c7_night_mail_1",
            chapter: StoryChapter::Chapter7NightRider,
            title: "Night Mail Run",
            description: "Fly mail from Grandcity to Ironforge in darkness and fog.",
            origin: AirportId::Grandcity,
            destination: AirportId::Ironforge,
            reward_gold: 800,
            reward_xp: 300,
            cutscene_id: Some("night_intro"),
            dialogue_npc: None,
            required_weather: Some(Weather::Fog),
            required_time_of_day: Some(TimeRequirement::Night),
        },
        StoryMission {
            id: "c7_night_mail_2",
            chapter: StoryChapter::Chapter7NightRider,
            title: "Fog Runner",
            description: "Continue the night run from Ironforge to Stormwatch.",
            origin: AirportId::Ironforge,
            destination: AirportId::Stormwatch,
            reward_gold: 900,
            reward_xp: 350,
            cutscene_id: None,
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: Some(TimeRequirement::Night),
        },
        StoryMission {
            id: "c7_dawn_return",
            chapter: StoryChapter::Chapter7NightRider,
            title: "Dawn Return",
            description: "Fly home as the sun rises. A beautiful way to end the night.",
            origin: AirportId::Stormwatch,
            destination: AirportId::HomeBase,
            reward_gold: 600,
            reward_xp: 200,
            cutscene_id: Some("dawn_beauty"),
            dialogue_npc: None,
            required_weather: Some(Weather::Clear),
            required_time_of_day: None,
        },
        // ── Chapter 8: The Big Emergency ────────────────────────────────
        StoryMission {
            id: "c8_routine_flight",
            chapter: StoryChapter::Chapter8TheBigEmergency,
            title: "Routine Run",
            description: "A routine passenger flight to Grandcity... what could go wrong?",
            origin: AirportId::HomeBase,
            destination: AirportId::Grandcity,
            reward_gold: 400,
            reward_xp: 100,
            cutscene_id: Some("calm_before_storm"),
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: None,
        },
        StoryMission {
            id: "c8_emergency",
            chapter: StoryChapter::Chapter8TheBigEmergency,
            title: "Mayday! Mayday!",
            description: "Engine failure mid-flight! Declare an emergency and land safely.",
            origin: AirportId::Grandcity,
            destination: AirportId::Windport,
            reward_gold: 2000,
            reward_xp: 500,
            cutscene_id: Some("engine_failure"),
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: None,
        },
        StoryMission {
            id: "c8_hero_return",
            chapter: StoryChapter::Chapter8TheBigEmergency,
            title: "Hero's Welcome",
            description: "Return home to a hero's welcome after saving all passengers.",
            origin: AirportId::Windport,
            destination: AirportId::HomeBase,
            reward_gold: 500,
            reward_xp: 200,
            cutscene_id: Some("hero_welcome"),
            dialogue_npc: Some("captain_elena"),
            required_weather: None,
            required_time_of_day: None,
        },
        // ── Chapter 9: Full Circle ──────────────────────────────────────
        StoryMission {
            id: "c9_leg1",
            chapter: StoryChapter::Chapter9FullCircle,
            title: "Around the World: Leg 1",
            description: "Begin the grand circuit — HomeBase to Windport to Sunhaven.",
            origin: AirportId::HomeBase,
            destination: AirportId::Sunhaven,
            reward_gold: 800,
            reward_xp: 300,
            cutscene_id: Some("grand_circuit_start"),
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: None,
        },
        StoryMission {
            id: "c9_leg2",
            chapter: StoryChapter::Chapter9FullCircle,
            title: "Around the World: Leg 2",
            description: "Continue through Ironforge, Cloudmere, Skyreach.",
            origin: AirportId::Sunhaven,
            destination: AirportId::Skyreach,
            reward_gold: 1000,
            reward_xp: 400,
            cutscene_id: None,
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: None,
        },
        StoryMission {
            id: "c9_leg3",
            chapter: StoryChapter::Chapter9FullCircle,
            title: "Around the World: Finale",
            description: "Stormwatch, Duskhollow, Frostpeak, and home. Full circle!",
            origin: AirportId::Skyreach,
            destination: AirportId::HomeBase,
            reward_gold: 1500,
            reward_xp: 500,
            cutscene_id: Some("full_circle_complete"),
            dialogue_npc: None,
            required_weather: None,
            required_time_of_day: None,
        },
        // ── Chapter 10: Wings of Legend ──────────────────────────────────
        StoryMission {
            id: "c10_final_test",
            chapter: StoryChapter::Chapter10LegendaryPilot,
            title: "The Final Test",
            description: "instructor_chen's final examination — fly Skyreach in bad weather.",
            origin: AirportId::HomeBase,
            destination: AirportId::Skyreach,
            reward_gold: 2000,
            reward_xp: 600,
            cutscene_id: Some("final_test_intro"),
            dialogue_npc: Some("instructor_chen"),
            required_weather: Some(Weather::Storm),
            required_time_of_day: None,
        },
        StoryMission {
            id: "c10_ace_ceremony",
            chapter: StoryChapter::Chapter10LegendaryPilot,
            title: "Ace Ceremony",
            description: "Return home for the ceremony. You've earned the Ace rank!",
            origin: AirportId::Skyreach,
            destination: AirportId::HomeBase,
            reward_gold: 3000,
            reward_xp: 1000,
            cutscene_id: Some("ace_ceremony"),
            dialogue_npc: Some("captain_elena"),
            required_weather: Some(Weather::Clear),
            required_time_of_day: Some(TimeRequirement::Day),
        },
    ]
}

// ─── Story Progress Resource ─────────────────────────────────────────────

#[derive(Resource, Clone, Debug)]
pub struct StoryProgress {
    pub current_chapter: StoryChapter,
    pub completed_missions: Vec<String>,
    pub current_mission_index: usize,
    pub chapter_complete: Vec<StoryChapter>,
    pub story_finished: bool,
}

impl Default for StoryProgress {
    fn default() -> Self {
        Self {
            current_chapter: StoryChapter::Chapter1FirstSolo,
            completed_missions: Vec::new(),
            current_mission_index: 0,
            chapter_complete: Vec::new(),
            story_finished: false,
        }
    }
}

impl StoryProgress {
    pub fn is_mission_complete(&self, id: &str) -> bool {
        self.completed_missions.iter().any(|m| m == id)
    }

    pub fn is_chapter_complete(&self, ch: StoryChapter) -> bool {
        self.chapter_complete.contains(&ch)
    }

    pub fn current_mission(&self) -> Option<StoryMission> {
        let all = all_story_missions();
        let chapter_missions: Vec<_> = all
            .into_iter()
            .filter(|m| m.chapter == self.current_chapter)
            .collect();
        chapter_missions.into_iter().nth(self.current_mission_index)
    }

    pub fn progress_percent(&self) -> f32 {
        let total = all_story_missions().len() as f32;
        if total == 0.0 { return 0.0; }
        self.completed_missions.len() as f32 / total * 100.0
    }
}

// ─── Systems ─────────────────────────────────────────────────────────────

/// Check when a mission is completed and advance story progress.
pub fn track_story_progress(
    mut mission_events: EventReader<MissionCompletedEvent>,
    mut story: ResMut<StoryProgress>,
    mut toast: EventWriter<ToastEvent>,
    mut cutscene_events: EventWriter<CutsceneStartEvent>,
) {
    for evt in mission_events.read() {
        let all_missions = all_story_missions();
        let Some(story_mission) = all_missions.iter().find(|m| m.id == evt.mission_id) else {
            continue;
        };

        if story.is_mission_complete(story_mission.id) {
            continue;
        }

        story.completed_missions.push(story_mission.id.to_string());

        toast.send(ToastEvent {
            message: format!("Story: \"{}\" complete!", story_mission.title),
            duration_secs: 4.0,
        });

        // Trigger cutscene if one is defined
        if let Some(cs_id) = story_mission.cutscene_id {
            cutscene_events.send(CutsceneStartEvent {
                cutscene_id: cs_id.to_string(),
            });
        }

        // Advance mission index or chapter
        let chapter_missions: Vec<_> = all_missions
            .iter()
            .filter(|m| m.chapter == story.current_chapter)
            .collect();

        let all_chapter_done = chapter_missions
            .iter()
            .all(|m| story.is_mission_complete(m.id));

        if all_chapter_done {
            let ch = story.current_chapter;
            story.chapter_complete.push(ch);

            toast.send(ToastEvent {
                message: format!("{} — COMPLETE!", story.current_chapter.title()),
                duration_secs: 5.0,
            });

            if let Some(next) = story.current_chapter.next() {
                story.current_chapter = next;
                story.current_mission_index = 0;

                toast.send(ToastEvent {
                    message: format!("New chapter unlocked: {}", next.title()),
                    duration_secs: 4.0,
                });
            } else {
                story.story_finished = true;
                toast.send(ToastEvent {
                    message: "★ Congratulations! You've completed the Skywarden story! ★".into(),
                    duration_secs: 8.0,
                });
            }
        } else {
            story.current_mission_index += 1;
        }
    }
}

/// Display available story mission on the mission board.
pub fn show_story_mission_on_board(
    story: Res<StoryProgress>,
    pilot: Res<PilotState>,
    mut toast: EventWriter<ToastEvent>,
) {
    if story.story_finished {
        return;
    }

    let Some(mission) = story.current_mission() else {
        return;
    };

    if pilot.rank < mission.chapter.required_rank() {
        return;
    }

    // Only show once when the resource changes
    if !story.is_changed() {
        return;
    }

    toast.send(ToastEvent {
        message: format!("Story Mission: {} — {}", mission.title, mission.description),
        duration_secs: 6.0,
    });
}

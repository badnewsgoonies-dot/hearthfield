use serde::{Deserialize, Serialize};

pub mod tavern {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Mood {
        Happy,
        Neutral,
        Grumpy,
    }

    impl Mood {
        pub fn improve(self) -> Self {
            match self {
                Self::Grumpy => Self::Neutral,
                Self::Neutral => Self::Happy,
                Self::Happy => Self::Happy,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct NpcProfile {
        pub name: String,
        pub mood: Mood,
        pub topics: Vec<String>,
        pub affinity: u8,
    }

    impl NpcProfile {
        pub fn new(name: &str, mood: Mood, topics: Vec<&str>, affinity: u8) -> Self {
            Self {
                name: name.to_string(),
                mood,
                topics: topics.into_iter().map(str::to_string).collect(),
                affinity: affinity.min(100),
            }
        }

        fn knows_topic(&self, topic: &str) -> bool {
            self.topics.iter().any(|known| known.eq_ignore_ascii_case(topic))
        }

        fn apply_affinity_change(&mut self, delta: i16) {
            let next = i16::from(self.affinity) + delta;
            self.affinity = next.clamp(0, 100) as u8;
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Tavern {
        pub name: String,
        pub available_npcs: Vec<NpcProfile>,
        pub time_of_day: String,
    }

    impl Tavern {
        pub fn new(name: &str, time_of_day: &str, available_npcs: Vec<NpcProfile>) -> Self {
            assert!(
                (3..=5).contains(&available_npcs.len()),
                "Tavern requires between 3 and 5 NPCs"
            );

            Self {
                name: name.to_string(),
                available_npcs,
                time_of_day: time_of_day.to_string(),
            }
        }

        pub fn default_tavern() -> Self {
            Self::new(
                "The Copper Cup",
                "Evening",
                vec![
                    NpcProfile::new("Mira", Mood::Neutral, vec!["rumors", "trade", "music"], 45),
                    NpcProfile::new("Borin", Mood::Grumpy, vec!["mines", "ale", "weather"], 30),
                    NpcProfile::new("Selene", Mood::Happy, vec!["magic", "legends", "tea"], 70),
                    NpcProfile::new("Tovin", Mood::Neutral, vec!["hunting", "roads", "rumors"], 55),
                ],
            )
        }

        pub fn list_npcs(&self) -> Vec<&str> {
            self.available_npcs
                .iter()
                .map(|npc| npc.name.as_str())
                .collect()
        }

        pub fn save_json(&self) -> Result<String, serde_json::Error> {
            serde_json::to_string_pretty(self)
        }

        pub fn load_json(data: &str) -> Result<Self, serde_json::Error> {
            serde_json::from_str(data)
        }

        pub fn talk_to_npc(
            &mut self,
            npc_name: &str,
            topic: &str,
        ) -> Result<ConversationOutcome, TavernError> {
            let npc = self
                .available_npcs
                .iter_mut()
                .find(|npc| npc.name.eq_ignore_ascii_case(npc_name))
                .ok_or_else(|| TavernError::NpcNotFound(npc_name.to_string()))?;

            let topic_match = if npc.knows_topic(topic) {
                TopicMatch::Good
            } else if topic.eq_ignore_ascii_case("hello")
                || topic.eq_ignore_ascii_case("tavern")
                || topic.eq_ignore_ascii_case("travel")
            {
                TopicMatch::Neutral
            } else {
                TopicMatch::Bad
            };

            let affinity_delta = match topic_match {
                TopicMatch::Good => 5,
                TopicMatch::Neutral => 1,
                TopicMatch::Bad => -10,
            };
            npc.apply_affinity_change(affinity_delta);

            let response = response_text(npc, topic, topic_match);

            Ok(ConversationOutcome {
                npc_name: npc.name.clone(),
                topic: topic.to_string(),
                topic_match,
                response,
                affinity_change: affinity_delta,
                affinity_after: npc.affinity,
                mood_after: npc.mood,
            })
        }

        pub fn buy_drink(&mut self, npc_name: &str, player_gold: &mut u32) -> Result<(), TavernError> {
            if *player_gold < 10 {
                return Err(TavernError::InsufficientGold {
                    required: 10,
                    available: *player_gold,
                });
            }

            let npc = self
                .available_npcs
                .iter_mut()
                .find(|npc| npc.name.eq_ignore_ascii_case(npc_name))
                .ok_or_else(|| TavernError::NpcNotFound(npc_name.to_string()))?;

            *player_gold -= 10;
            npc.mood = npc.mood.improve();
            Ok(())
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TopicMatch {
        Good,
        Neutral,
        Bad,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ConversationOutcome {
        pub npc_name: String,
        pub topic: String,
        pub topic_match: TopicMatch,
        pub response: String,
        pub affinity_change: i16,
        pub affinity_after: u8,
        pub mood_after: Mood,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum TavernError {
        NpcNotFound(String),
        InsufficientGold { required: u32, available: u32 },
    }

    impl std::fmt::Display for TavernError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::NpcNotFound(name) => write!(f, "NPC not found: {name}"),
                Self::InsufficientGold {
                    required,
                    available,
                } => write!(
                    f,
                    "Not enough gold to buy a drink. Required: {required}, available: {available}"
                ),
            }
        }
    }

    impl std::error::Error for TavernError {}

    fn affinity_tone(affinity: u8) -> &'static str {
        match affinity {
            0..=25 => "They barely tolerate the conversation.",
            26..=60 => "They seem willing to keep talking.",
            _ => "They treat you like a trusted regular.",
        }
    }

    fn response_text(npc: &NpcProfile, topic: &str, topic_match: TopicMatch) -> String {
        let topic_fragment = match topic_match {
            TopicMatch::Good => format!("about {}", topic.to_lowercase()),
            TopicMatch::Neutral => format!("about {}", topic.to_lowercase()),
            TopicMatch::Bad => format!("about {}", topic.to_lowercase()),
        };

        let mood_line = match (npc.mood, topic_match) {
            (Mood::Happy, TopicMatch::Good) => {
                "They grin and lean in, clearly delighted you asked."
            }
            (Mood::Happy, TopicMatch::Neutral) => {
                "They answer with easy warmth, even if the topic is ordinary."
            }
            (Mood::Happy, TopicMatch::Bad) => {
                "They keep smiling, but the answer comes with a puzzled shrug."
            }
            (Mood::Neutral, TopicMatch::Good) => {
                "They nod and offer a solid, thoughtful answer."
            }
            (Mood::Neutral, TopicMatch::Neutral) => {
                "They answer politely without much investment."
            }
            (Mood::Neutral, TopicMatch::Bad) => {
                "They hesitate, clearly unsure why you brought that up."
            }
            (Mood::Grumpy, TopicMatch::Good) => {
                "They soften a little; at least this is a subject worth discussing."
            }
            (Mood::Grumpy, TopicMatch::Neutral) => {
                "They grumble through a short reply."
            }
            (Mood::Grumpy, TopicMatch::Bad) => {
                "They scowl and cut the conversation short."
            }
        };

        format!(
            "{} responds {}. {} {}",
            npc.name,
            topic_fragment,
            mood_line,
            affinity_tone(npc.affinity)
        )
    }

    pub fn demo() {
        let mut tavern = Tavern::default_tavern();
        let mut player_gold = 25;

        println!("Entering {} in the {}.", tavern.name, tavern.time_of_day);
        println!("Patrons available: {}", tavern.list_npcs().join(", "));

        let first_chat = tavern
            .talk_to_npc("Borin", "mines")
            .expect("demo conversation should succeed");
        println!("First conversation: {}", first_chat.response);
        println!(
            "Affinity change: {} (now {})",
            first_chat.affinity_change, first_chat.affinity_after
        );

        tavern
            .buy_drink("Borin", &mut player_gold)
            .expect("demo drink purchase should succeed");
        let borin = tavern
            .available_npcs
            .iter()
            .find(|npc| npc.name == "Borin")
            .expect("Borin should exist after drink");
        println!(
            "Bought Borin a drink for 10 gold. Gold left: {}. Mood is now {:?}.",
            player_gold, borin.mood
        );

        let second_chat = tavern
            .talk_to_npc("Borin", "ale")
            .expect("second demo conversation should succeed");
        println!("Second conversation: {}", second_chat.response);
        println!(
            "Affinity change: {} (now {})",
            second_chat.affinity_change, second_chat.affinity_after
        );

        let saved = tavern.save_json().expect("demo save should succeed");
        println!("Saved tavern state:\n{}", saved);

        let loaded = Tavern::load_json(&saved).expect("demo load should succeed");
        assert_eq!(tavern, loaded);
        println!("State verified after load: roundtrip successful.");
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn conversation_flow_returns_expected_outcome() {
            let mut tavern = Tavern::default_tavern();
            let outcome = tavern.talk_to_npc("Mira", "trade").unwrap();

            assert_eq!(outcome.npc_name, "Mira");
            assert_eq!(outcome.topic, "trade");
            assert_eq!(outcome.topic_match, TopicMatch::Good);
            assert!(outcome.response.contains("Mira responds about trade"));
        }

        #[test]
        fn good_topic_increases_affinity_by_five() {
            let mut tavern = Tavern::default_tavern();
            let before = tavern.available_npcs[0].affinity;
            let outcome = tavern.talk_to_npc("Mira", "music").unwrap();

            assert_eq!(outcome.affinity_change, 5);
            assert_eq!(outcome.affinity_after, before + 5);
        }

        #[test]
        fn bad_topic_decreases_affinity_by_ten() {
            let mut tavern = Tavern::default_tavern();
            let outcome = tavern.talk_to_npc("Selene", "taxes").unwrap();

            assert_eq!(outcome.topic_match, TopicMatch::Bad);
            assert_eq!(outcome.affinity_change, -10);
            assert_eq!(outcome.affinity_after, 60);
        }

        #[test]
        fn neutral_topic_increases_affinity_by_one() {
            let mut tavern = Tavern::default_tavern();
            let outcome = tavern.talk_to_npc("Tovin", "travel").unwrap();

            assert_eq!(outcome.topic_match, TopicMatch::Neutral);
            assert_eq!(outcome.affinity_change, 1);
            assert_eq!(outcome.affinity_after, 56);
        }

        #[test]
        fn mood_improves_one_tier_per_drink() {
            let mut tavern = Tavern::default_tavern();
            let mut gold = 30;

            tavern.buy_drink("Borin", &mut gold).unwrap();
            assert_eq!(tavern.available_npcs[1].mood, Mood::Neutral);

            tavern.buy_drink("Borin", &mut gold).unwrap();
            assert_eq!(tavern.available_npcs[1].mood, Mood::Happy);
        }

        #[test]
        fn buy_drink_costs_ten_gold() {
            let mut tavern = Tavern::default_tavern();
            let mut gold = 12;

            tavern.buy_drink("Mira", &mut gold).unwrap();
            assert_eq!(gold, 2);
        }

        #[test]
        fn save_load_roundtrip_preserves_state() {
            let mut tavern = Tavern::default_tavern();
            let mut gold = 20;

            tavern.talk_to_npc("Borin", "weather").unwrap();
            tavern.buy_drink("Borin", &mut gold).unwrap();

            let json = tavern.save_json().unwrap();
            let loaded = Tavern::load_json(&json).unwrap();

            assert_eq!(tavern, loaded);
        }
    }
}

fn main() {
    tavern::demo();
}

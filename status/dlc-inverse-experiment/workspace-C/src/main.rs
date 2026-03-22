mod tavern {
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;
    use std::error::Error;
    use std::fmt;

    const DRINK_COST: u32 = 10;

    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

        fn variant_text<'a>(self, responses: &'a TopicResponses) -> &'a str {
            match self {
                Self::Happy => &responses.happy,
                Self::Neutral => &responses.neutral,
                Self::Grumpy => &responses.grumpy,
            }
        }
    }

    impl fmt::Display for Mood {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let label = match self {
                Self::Happy => "Happy",
                Self::Neutral => "Neutral",
                Self::Grumpy => "Grumpy",
            };
            write!(f, "{label}")
        }
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    pub enum TimeOfDay {
        Morning,
        Afternoon,
        Evening,
        Night,
    }

    impl fmt::Display for TimeOfDay {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let label = match self {
                Self::Morning => "Morning",
                Self::Afternoon => "Afternoon",
                Self::Evening => "Evening",
                Self::Night => "Night",
            };
            write!(f, "{label}")
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct TopicResponses {
        pub happy: String,
        pub neutral: String,
        pub grumpy: String,
    }

    impl TopicResponses {
        pub fn new(happy: &str, neutral: &str, grumpy: &str) -> Self {
            Self {
                happy: happy.to_string(),
                neutral: neutral.to_string(),
                grumpy: grumpy.to_string(),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct NpcProfile {
        pub name: String,
        pub mood: Mood,
        pub topics: Vec<String>,
        pub affinity: u8,
        pub topic_responses: BTreeMap<String, TopicResponses>,
    }

    impl NpcProfile {
        pub fn new(
            name: &str,
            mood: Mood,
            affinity: u8,
            topic_responses: BTreeMap<String, TopicResponses>,
        ) -> Self {
            let topics = topic_responses.keys().cloned().collect();
            Self {
                name: name.to_string(),
                mood,
                topics,
                affinity: affinity.min(100),
                topic_responses,
            }
        }

        pub fn available_topics(&self) -> &[String] {
            &self.topics
        }
    }

    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
    pub enum ConversationOutcome {
        Good,
        Neutral,
        Bad,
    }

    impl fmt::Display for ConversationOutcome {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let label = match self {
                Self::Good => "good",
                Self::Neutral => "neutral",
                Self::Bad => "bad",
            };
            write!(f, "{label}")
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Conversation {
        pub npc_name: String,
        pub topic: String,
        pub outcome: ConversationOutcome,
        pub affinity_change: i16,
        pub new_affinity: u8,
        pub response: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct DrinkResult {
        pub npc_name: String,
        pub old_mood: Mood,
        pub new_mood: Mood,
        pub gold_spent: u32,
        pub player_gold_left: u32,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Tavern {
        pub name: String,
        pub available_npcs: Vec<NpcProfile>,
        pub current_time_of_day: TimeOfDay,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum TavernError {
        NpcNotFound(String),
        NotEnoughGold { required: u32, available: u32 },
    }

    impl fmt::Display for TavernError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::NpcNotFound(name) => write!(f, "NPC '{name}' was not found in the tavern"),
                Self::NotEnoughGold {
                    required,
                    available,
                } => {
                    write!(f, "not enough gold: need {required}, have {available}")
                }
            }
        }
    }

    impl Error for TavernError {}

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct DemoReport {
        pub transcript: Vec<String>,
        pub player_reachable_features: Vec<String>,
        pub dead_or_unreachable_features: Vec<String>,
    }

    impl Tavern {
        pub fn sample() -> Self {
            Self {
                name: "The Lantern Cup".to_string(),
                available_npcs: vec![
                    NpcProfile::new(
                        "Mira",
                        Mood::Neutral,
                        45,
                        BTreeMap::from([
                            (
                                "music".to_string(),
                                TopicResponses::new(
                                    "Mira taps the table in rhythm. 'A good song can turn a crowded room into family.'",
                                    "Mira nods. 'I play when the supper rush slows down.'",
                                    "Mira sighs. 'If the lute snaps one more string tonight, I'm burning it.'",
                                ),
                            ),
                            (
                                "travel".to_string(),
                                TopicResponses::new(
                                    "Mira grins. 'Someday I'll see the coast again and bring back three new ballads.'",
                                    "Mira says, 'I've seen enough roads to know good boots matter more than maps.'",
                                    "Mira mutters, 'Travel is just rain, mud, and late wagons.'",
                                ),
                            ),
                            (
                                "rumors".to_string(),
                                TopicResponses::new(
                                    "Mira leans in. 'Now that's tavern language. I've heard the reeve meets smugglers after dusk.'",
                                    "Mira says, 'Rumors keep the mugs moving, but half of them are brewed worse than ale.'",
                                    "Mira folds her arms. 'If it's another ghost story, spare me.'",
                                ),
                            ),
                        ]),
                    ),
                    NpcProfile::new(
                        "Doran",
                        Mood::Grumpy,
                        20,
                        BTreeMap::from([
                            (
                                "smithing".to_string(),
                                TopicResponses::new(
                                    "Doran chuckles. 'Steel sings when the hammer lands true.'",
                                    "Doran says, 'Most blades fail because fools rush the quench.'",
                                    "Doran grunts. 'At least iron listens better than people.'",
                                ),
                            ),
                            (
                                "monsters".to_string(),
                                TopicResponses::new(
                                    "Doran smirks. 'Give me a stout spear and I'll tell you where trolls panic.'",
                                    "Doran says, 'Monsters leave clearer tracks than merchants.'",
                                    "Doran growls. 'If you're hunting them, stop talking and sharpen something.'",
                                ),
                            ),
                            (
                                "ale".to_string(),
                                TopicResponses::new(
                                    "Doran lifts his mug. 'Now that's a craft worth respecting.'",
                                    "Doran says, 'Ale is decent if the brewer respects the grain.'",
                                    "Doran mutters, 'This batch tastes like the barrel lost a fight.'",
                                ),
                            ),
                        ]),
                    ),
                    NpcProfile::new(
                        "Selise",
                        Mood::Happy,
                        70,
                        BTreeMap::from([
                            (
                                "history".to_string(),
                                TopicResponses::new(
                                    "Selise beams. 'Every old ruin is a letter from the dead to the curious.'",
                                    "Selise says, 'History is patient. Most people are not.'",
                                    "Selise frowns. 'People only ask for history after they've already broken something ancient.'",
                                ),
                            ),
                            (
                                "magic".to_string(),
                                TopicResponses::new(
                                    "Selise brightens. 'Magic is etiquette with the universe. Be polite and it replies.'",
                                    "Selise says, 'Magic is structure, not spectacle.'",
                                    "Selise snaps, 'If you want fireworks, bother an apprentice.'",
                                ),
                            ),
                            (
                                "library".to_string(),
                                TopicResponses::new(
                                    "Selise laughs. 'The abbey library smells like dust and ambition. I adore it.'",
                                    "Selise says, 'The restricted shelf is better catalogued than guarded.'",
                                    "Selise sighs. 'I loaned out a codex last month. It came back with soup on it.'",
                                ),
                            ),
                        ]),
                    ),
                    NpcProfile::new(
                        "Bran",
                        Mood::Neutral,
                        35,
                        BTreeMap::from([
                            (
                                "horses".to_string(),
                                TopicResponses::new(
                                    "Bran laughs. 'A calm mare can judge character faster than a priest.'",
                                    "Bran says, 'Brush the coat, check the hooves, then ask for speed.'",
                                    "Bran grumbles. 'Anyone who yanks reins shouldn't own a stable.'",
                                ),
                            ),
                            (
                                "trade".to_string(),
                                TopicResponses::new(
                                    "Bran smiles. 'A fair bargain keeps roads safer than soldiers do.'",
                                    "Bran says, 'Trade is mostly timing and less honesty than I'd prefer.'",
                                    "Bran snorts. 'Every trader says shortage when they mean greed.'",
                                ),
                            ),
                            (
                                "weather".to_string(),
                                TopicResponses::new(
                                    "Bran grins. 'Clear skies mean quick wheels and fewer excuses.'",
                                    "Bran says, 'Weather decides more contracts than kings.'",
                                    "Bran mutters, 'Rain turns every road into a lie.'",
                                ),
                            ),
                        ]),
                    ),
                ],
                current_time_of_day: TimeOfDay::Evening,
            }
        }

        pub fn list_npcs(&self) -> Vec<&str> {
            self.available_npcs
                .iter()
                .map(|npc| npc.name.as_str())
                .collect()
        }

        pub fn list_topics_for_npc(&self, npc_name: &str) -> Result<Vec<String>, TavernError> {
            let npc = self
                .available_npcs
                .iter()
                .find(|npc| npc.name == npc_name)
                .ok_or_else(|| TavernError::NpcNotFound(npc_name.to_string()))?;
            Ok(npc.available_topics().to_vec())
        }

        pub fn talk_to_npc(
            &mut self,
            npc_name: &str,
            topic: &str,
        ) -> Result<Conversation, TavernError> {
            let npc = self
                .available_npcs
                .iter_mut()
                .find(|npc| npc.name == npc_name)
                .ok_or_else(|| TavernError::NpcNotFound(npc_name.to_string()))?;

            let outcome = if npc.topic_responses.contains_key(topic) {
                ConversationOutcome::Good
            } else if is_neutral_topic(topic) {
                ConversationOutcome::Neutral
            } else {
                ConversationOutcome::Bad
            };

            let affinity_change = match outcome {
                ConversationOutcome::Good => 5,
                ConversationOutcome::Neutral => 1,
                ConversationOutcome::Bad => -10,
            };

            npc.affinity = adjust_affinity(npc.affinity, affinity_change);
            let response = build_response(npc, topic, outcome);

            Ok(Conversation {
                npc_name: npc.name.clone(),
                topic: topic.to_string(),
                outcome,
                affinity_change,
                new_affinity: npc.affinity,
                response,
            })
        }

        pub fn buy_drink(
            &mut self,
            npc_name: &str,
            player_gold: &mut u32,
        ) -> Result<DrinkResult, TavernError> {
            if *player_gold < DRINK_COST {
                return Err(TavernError::NotEnoughGold {
                    required: DRINK_COST,
                    available: *player_gold,
                });
            }

            let npc = self
                .available_npcs
                .iter_mut()
                .find(|npc| npc.name == npc_name)
                .ok_or_else(|| TavernError::NpcNotFound(npc_name.to_string()))?;

            let old_mood = npc.mood;
            npc.mood = npc.mood.improve();
            *player_gold -= DRINK_COST;

            Ok(DrinkResult {
                npc_name: npc.name.clone(),
                old_mood,
                new_mood: npc.mood,
                gold_spent: DRINK_COST,
                player_gold_left: *player_gold,
            })
        }

        pub fn save_state(&self) -> serde_json::Result<String> {
            serde_json::to_string_pretty(self)
        }

        pub fn load_state(serialized: &str) -> serde_json::Result<Self> {
            serde_json::from_str(serialized)
        }
    }

    fn adjust_affinity(current: u8, change: i16) -> u8 {
        (i16::from(current) + change).clamp(0, 100) as u8
    }

    fn is_neutral_topic(topic: &str) -> bool {
        matches!(topic, "weather" | "food" | "local news" | "work")
    }

    fn build_response(npc: &NpcProfile, topic: &str, outcome: ConversationOutcome) -> String {
        let base = match outcome {
            ConversationOutcome::Good => npc
                .topic_responses
                .get(topic)
                .map(|responses| npc.mood.variant_text(responses).to_string())
                .unwrap_or_else(|| "They answer, but something went wrong in the telling.".to_string()),
            ConversationOutcome::Neutral => match npc.mood {
                Mood::Happy => format!(
                    "{} smiles politely. 'I can spare a minute for {}.'",
                    npc.name, topic
                ),
                Mood::Neutral => format!(
                    "{} gives a measured nod. '{} is harmless enough talk.'",
                    npc.name, topic
                ),
                Mood::Grumpy => format!(
                    "{} exhales through their nose. 'If you insist, we can talk about {}.'",
                    npc.name, topic
                ),
            },
            ConversationOutcome::Bad => match npc.mood {
                Mood::Happy => format!(
                    "{} keeps the peace. 'That's not really my subject, but I've heard stranger questions.'",
                    npc.name
                ),
                Mood::Neutral => format!(
                    "{} shrugs. 'You're asking the wrong person about {}.'",
                    npc.name, topic
                ),
                Mood::Grumpy => format!(
                    "{} scowls. 'Ask someone else about {} before my mood gets worse.'",
                    npc.name, topic
                ),
            },
        };

        let affinity_tone = match npc.affinity {
            0..=24 => " They still do not trust you.",
            25..=59 => " They seem willing to keep talking.",
            _ => " They clearly enjoy your company.",
        };

        format!("{base}{affinity_tone}")
    }

    pub fn demo() -> Result<DemoReport, Box<dyn Error>> {
        let mut transcript = Vec::new();
        let mut tavern = Tavern::sample();
        let mut player_gold = 25;

        transcript.push(format!(
            "You enter {} during the {}.",
            tavern.name, tavern.current_time_of_day
        ));

        let npc_names = tavern.list_npcs();
        transcript.push(format!("NPCs available: {}", npc_names.join(", ")));

        for npc_name in &npc_names {
            let topics = tavern.list_topics_for_npc(npc_name)?;
            transcript.push(format!("Topics for {npc_name}: {}", topics.join(", ")));
        }

        let first_talk = tavern.talk_to_npc("Doran", "smithing")?;
        transcript.push(format!(
            "You talk to {} about {}. Outcome: {}. Affinity {} -> {}. {}",
            first_talk.npc_name,
            first_talk.topic,
            first_talk.outcome,
            first_talk.affinity_change,
            first_talk.new_affinity,
            first_talk.response
        ));

        let neutral_talk = tavern.talk_to_npc("Mira", "weather")?;
        transcript.push(format!(
            "You ask {} about {}. Outcome: {}. Affinity {} -> {}. {}",
            neutral_talk.npc_name,
            neutral_talk.topic,
            neutral_talk.outcome,
            neutral_talk.affinity_change,
            neutral_talk.new_affinity,
            neutral_talk.response
        ));

        let bad_talk = tavern.talk_to_npc("Bran", "dragons")?;
        transcript.push(format!(
            "You bring up {} with {}. Outcome: {}. Affinity {} -> {}. {}",
            bad_talk.topic,
            bad_talk.npc_name,
            bad_talk.outcome,
            bad_talk.affinity_change,
            bad_talk.new_affinity,
            bad_talk.response
        ));

        let drink = tavern.buy_drink("Doran", &mut player_gold)?;
        transcript.push(format!(
            "You buy {} a drink for {} gold. Mood: {} -> {}. Gold left: {}.",
            drink.npc_name,
            drink.gold_spent,
            drink.old_mood,
            drink.new_mood,
            drink.player_gold_left
        ));

        let second_talk = tavern.talk_to_npc("Doran", "ale")?;
        transcript.push(format!(
            "You talk to {} again about {}. Outcome: {}. New affinity: {}. {}",
            second_talk.npc_name,
            second_talk.topic,
            second_talk.outcome,
            second_talk.new_affinity,
            second_talk.response
        ));

        let saved = tavern.save_state()?;
        transcript.push(format!(
            "Saved tavern state to JSON ({} bytes).",
            saved.len()
        ));

        let loaded = Tavern::load_state(&saved)?;
        transcript.push(format!(
            "Loaded tavern state for {} during the {}.",
            loaded.name, loaded.current_time_of_day
        ));

        let loaded_doran = loaded
            .available_npcs
            .iter()
            .find(|npc| npc.name == "Doran")
            .ok_or_else(|| TavernError::NpcNotFound("Doran".to_string()))?;
        transcript.push(format!(
            "Verification: Doran restored with mood {} and affinity {} after save/load.",
            loaded_doran.mood, loaded_doran.affinity
        ));

        let player_reachable_features = vec![
            "Entering a tavern with a name and time of day.".to_string(),
            "Listing all available NPCs.".to_string(),
            "Listing every NPC's topic set before choosing who to approach.".to_string(),
            "Talking to an NPC with a good topic match and getting mood-specific dialogue."
                .to_string(),
            "Talking to an NPC with a neutral topic and getting a small affinity gain.".to_string(),
            "Talking to an NPC with a bad topic and taking an affinity penalty.".to_string(),
            "Buying a drink for 10 gold to improve an NPC's mood by one tier.".to_string(),
            "Seeing mood changes alter the next conversation response.".to_string(),
            "Saving the full tavern state to JSON.".to_string(),
            "Loading the tavern state from JSON and verifying mood and affinity persistence."
                .to_string(),
        ];

        let dead_or_unreachable_features = vec![
            "None. Every public gameplay function is exercised by demo() or the unit tests."
                .to_string(),
        ];

        Ok(DemoReport {
            transcript,
            player_reachable_features,
            dead_or_unreachable_features,
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn conversation_flow_returns_good_match_response() {
            let mut tavern = Tavern::sample();
            let result = tavern.talk_to_npc("Selise", "magic").unwrap();

            assert_eq!(result.npc_name, "Selise");
            assert_eq!(result.topic, "magic");
            assert_eq!(result.outcome, ConversationOutcome::Good);
            assert!(result
                .response
                .contains("Magic is etiquette with the universe"));
        }

        #[test]
        fn affinity_changes_follow_good_neutral_and_bad_rules() {
            let mut tavern = Tavern::sample();

            let good = tavern.talk_to_npc("Mira", "music").unwrap();
            assert_eq!(good.affinity_change, 5);
            assert_eq!(good.new_affinity, 50);

            let neutral = tavern.talk_to_npc("Mira", "food").unwrap();
            assert_eq!(neutral.affinity_change, 1);
            assert_eq!(neutral.new_affinity, 51);

            let bad = tavern.talk_to_npc("Mira", "dragons").unwrap();
            assert_eq!(bad.affinity_change, -10);
            assert_eq!(bad.new_affinity, 41);
        }

        #[test]
        fn mood_improves_one_tier_per_drink() {
            assert_eq!(Mood::Grumpy.improve(), Mood::Neutral);
            assert_eq!(Mood::Neutral.improve(), Mood::Happy);
            assert_eq!(Mood::Happy.improve(), Mood::Happy);
        }

        #[test]
        fn buy_drink_spends_gold_and_changes_mood() {
            let mut tavern = Tavern::sample();
            let mut gold = 12;

            let result = tavern.buy_drink("Doran", &mut gold).unwrap();

            assert_eq!(result.old_mood, Mood::Grumpy);
            assert_eq!(result.new_mood, Mood::Neutral);
            assert_eq!(result.gold_spent, 10);
            assert_eq!(gold, 2);
        }

        #[test]
        fn mood_change_affects_follow_up_conversation_text() {
            let mut tavern = Tavern::sample();
            let mut gold = 20;

            let before = tavern.talk_to_npc("Doran", "ale").unwrap();
            tavern.buy_drink("Doran", &mut gold).unwrap();
            let after = tavern.talk_to_npc("Doran", "ale").unwrap();

            assert!(before
                .response
                .contains("This batch tastes like the barrel lost a fight."));
            assert!(after
                .response
                .contains("Ale is decent if the brewer respects the grain."));
        }

        #[test]
        fn save_load_roundtrip_restores_all_state() {
            let mut tavern = Tavern::sample();
            let mut gold = 30;

            tavern.talk_to_npc("Doran", "smithing").unwrap();
            tavern.buy_drink("Doran", &mut gold).unwrap();

            let saved = tavern.save_state().unwrap();
            let loaded = Tavern::load_state(&saved).unwrap();

            assert_eq!(loaded, tavern);
            let doran = loaded
                .available_npcs
                .iter()
                .find(|npc| npc.name == "Doran")
                .unwrap();
            assert_eq!(doran.mood, Mood::Neutral);
            assert_eq!(doran.affinity, 25);
        }
    }
}

fn main() {
    match tavern::demo() {
        Ok(report) => {
            for line in report.transcript {
                println!("{line}");
            }

            println!("\nPlayer-reachable features:");
            for feature in report.player_reachable_features {
                println!("- {feature}");
            }

            println!("\nDead or unreachable features:");
            for feature in report.dead_or_unreachable_features {
                println!("- {feature}");
            }
        }
        Err(error) => {
            eprintln!("demo failed: {error}");
            std::process::exit(1);
        }
    }
}

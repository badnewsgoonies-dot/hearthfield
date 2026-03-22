use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimeOfDay {
    Morning,
    Afternoon,
    Evening,
    Night,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mood {
    Grumpy,
    Neutral,
    Happy,
}

impl Mood {
    pub fn improve(self) -> Self {
        match self {
            Mood::Grumpy => Mood::Neutral,
            Mood::Neutral => Mood::Happy,
            Mood::Happy => Mood::Happy,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Npc {
    pub name: String,
    pub mood: Mood,
    pub topics: Vec<String>,
    pub affinity: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tavern {
    pub name: String,
    pub available_npcs: Vec<Npc>,
    pub current_time_of_day: TimeOfDay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationResult {
    pub npc_name: String,
    pub topic: String,
    pub response: String,
    pub affinity_change: i8,
    pub new_affinity: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TavernError {
    NpcNotFound(String),
    TopicUnavailable { npc_name: String, topic: String },
    NotEnoughGold { needed: u32, available: u32 },
    Serialization(String),
}

impl std::fmt::Display for TavernError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TavernError::NpcNotFound(name) => write!(f, "NPC not found: {name}"),
            TavernError::TopicUnavailable { npc_name, topic } => {
                write!(f, "Topic '{topic}' is not available for {npc_name}")
            }
            TavernError::NotEnoughGold { needed, available } => {
                write!(f, "Not enough gold: need {needed}, have {available}")
            }
            TavernError::Serialization(message) => write!(f, "Serialization error: {message}"),
        }
    }
}

impl std::error::Error for TavernError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TopicMatch {
    Good,
    Neutral,
    Bad,
}

impl Tavern {
    pub fn new(name: impl Into<String>, current_time_of_day: TimeOfDay) -> Self {
        Self {
            name: name.into(),
            current_time_of_day,
            available_npcs: default_npcs(),
        }
    }

    pub fn talk_to_npc(
        &mut self,
        npc_name: &str,
        topic: &str,
    ) -> Result<ConversationResult, TavernError> {
        let npc = self
            .available_npcs
            .iter_mut()
            .find(|npc| npc.name == npc_name)
            .ok_or_else(|| TavernError::NpcNotFound(npc_name.to_string()))?;

        if !npc.topics.iter().any(|available| available == topic) {
            return Err(TavernError::TopicUnavailable {
                npc_name: npc.name.clone(),
                topic: topic.to_string(),
            });
        }

        let topic_match = topic_match_for(&npc.name, topic);
        let affinity_change = match topic_match {
            TopicMatch::Good => 5,
            TopicMatch::Neutral => 1,
            TopicMatch::Bad => -10,
        };

        npc.affinity = apply_affinity_change(npc.affinity, affinity_change);

        let response = format!(
            "{} {}",
            topic_response(&npc.name, topic, npc.mood),
            affinity_suffix(npc.affinity)
        );

        Ok(ConversationResult {
            npc_name: npc.name.clone(),
            topic: topic.to_string(),
            response,
            affinity_change,
            new_affinity: npc.affinity,
        })
    }

    pub fn buy_drink(&mut self, npc_name: &str, player_gold: &mut u32) -> Result<Mood, TavernError> {
        const DRINK_COST: u32 = 10;

        if *player_gold < DRINK_COST {
            return Err(TavernError::NotEnoughGold {
                needed: DRINK_COST,
                available: *player_gold,
            });
        }

        let npc = self
            .available_npcs
            .iter_mut()
            .find(|npc| npc.name == npc_name)
            .ok_or_else(|| TavernError::NpcNotFound(npc_name.to_string()))?;

        *player_gold -= DRINK_COST;
        npc.mood = npc.mood.improve();
        Ok(npc.mood)
    }

    pub fn save_to_json(&self) -> Result<String, TavernError> {
        serde_json::to_string_pretty(self)
            .map_err(|err| TavernError::Serialization(err.to_string()))
    }

    pub fn load_from_json(json: &str) -> Result<Self, TavernError> {
        serde_json::from_str(json).map_err(|err| TavernError::Serialization(err.to_string()))
    }

    pub fn npc(&self, npc_name: &str) -> Result<&Npc, TavernError> {
        self.available_npcs
            .iter()
            .find(|npc| npc.name == npc_name)
            .ok_or_else(|| TavernError::NpcNotFound(npc_name.to_string()))
    }
}

pub fn demo_visit() -> Result<Tavern, TavernError> {
    let mut tavern = Tavern::new("The Copper Cup", TimeOfDay::Evening);
    let mut player_gold = 25;

    let first_chat = tavern.talk_to_npc("Bram", "ale")?;
    if first_chat.affinity_change != 5 {
        return Err(TavernError::Serialization(
            "Demo expected a positive first conversation".to_string(),
        ));
    }

    let new_mood = tavern.buy_drink("Bram", &mut player_gold)?;
    if new_mood != Mood::Happy {
        return Err(TavernError::Serialization(
            "Demo expected Bram to become happy after a drink".to_string(),
        ));
    }

    let second_chat = tavern.talk_to_npc("Bram", "rumors")?;
    if second_chat.new_affinity <= first_chat.new_affinity {
        return Err(TavernError::Serialization(
            "Demo expected affinity to improve after the second talk".to_string(),
        ));
    }

    let saved = tavern.save_to_json()?;
    let loaded = Tavern::load_from_json(&saved)?;

    if tavern != loaded {
        return Err(TavernError::Serialization(
            "Demo save/load verification failed".to_string(),
        ));
    }

    Ok(loaded)
}

fn apply_affinity_change(current: u8, delta: i8) -> u8 {
    let current = i16::from(current);
    let delta = i16::from(delta);
    let updated = (current + delta).clamp(0, 100);
    updated as u8
}

fn affinity_suffix(affinity: u8) -> &'static str {
    match affinity {
        0..=24 => "They keep their guard up.",
        25..=59 => "They seem willing to keep talking.",
        60..=100 => "They treat you like a trusted regular.",
    }
}

fn topic_match_for(npc_name: &str, topic: &str) -> TopicMatch {
    match (npc_name, topic) {
        ("Bram", "ale") | ("Bram", "rumors") => TopicMatch::Good,
        ("Bram", "taxes") => TopicMatch::Bad,
        ("Lyra", "music") | ("Lyra", "travel") => TopicMatch::Good,
        ("Lyra", "weather") => TopicMatch::Bad,
        ("Sera", "books") | ("Sera", "history") => TopicMatch::Good,
        ("Sera", "gossip") => TopicMatch::Bad,
        ("Torren", "hunting") | ("Torren", "work") => TopicMatch::Good,
        ("Torren", "magic") => TopicMatch::Bad,
        _ => TopicMatch::Neutral,
    }
}

fn topic_response(npc_name: &str, topic: &str, mood: Mood) -> &'static str {
    match (npc_name, topic, mood) {
        ("Bram", "ale", Mood::Happy) => "Bram raises his mug. 'Now that's a topic worth smiling over.'",
        ("Bram", "ale", Mood::Neutral) => "Bram nods at the cask. 'A steady pour can fix most evenings.'",
        ("Bram", "ale", Mood::Grumpy) => "Bram grunts. 'Talk's cheap. Ale isn't.'",
        ("Bram", "rumors", Mood::Happy) => "Bram leans in. 'I've heard three good stories and one terrible lie tonight.'",
        ("Bram", "rumors", Mood::Neutral) => "Bram lowers his voice. 'Depends which rumor you can afford.'",
        ("Bram", "rumors", Mood::Grumpy) => "Bram squints. 'Rumors usually end with someone bleeding.'",
        ("Bram", "taxes", Mood::Happy) => "Bram laughs once. 'Even in a good mood, that's enough to spoil my drink.'",
        ("Bram", "taxes", Mood::Neutral) => "Bram sighs. 'Collectors always know when the purse is light.'",
        ("Bram", "taxes", Mood::Grumpy) => "Bram slams the bar. 'Mention taxes again and buy the next table a round.'",
        ("Lyra", "music", Mood::Happy) => "Lyra taps the rhythm on the table. 'A fine tune can turn a room golden.'",
        ("Lyra", "music", Mood::Neutral) => "Lyra hums a bar. 'Depends whether you prefer ballads or drinking songs.'",
        ("Lyra", "music", Mood::Grumpy) => "Lyra exhales. 'If the lute starts screeching again, I'm leaving.'",
        ("Lyra", "travel", Mood::Happy) => "Lyra smiles. 'Every road looks kinder when you've seen enough of them.'",
        ("Lyra", "travel", Mood::Neutral) => "Lyra traces a route in spilled cider. 'Most roads cost more than maps admit.'",
        ("Lyra", "travel", Mood::Grumpy) => "Lyra folds her arms. 'Travel is mostly mud, blisters, and bad beds.'",
        ("Lyra", "weather", Mood::Happy) => "Lyra chuckles. 'A sunny sky is still just a sky.'",
        ("Lyra", "weather", Mood::Neutral) => "Lyra shrugs. 'Rain comes, rain goes.'",
        ("Lyra", "weather", Mood::Grumpy) => "Lyra rolls her eyes. 'Weather is what people discuss when they have nothing worth saying.'",
        ("Sera", "books", Mood::Happy) => "Sera brightens. 'A well-bound book is better company than half this tavern.'",
        ("Sera", "books", Mood::Neutral) => "Sera adjusts her glasses. 'Depends whether you read for wisdom or escape.'",
        ("Sera", "books", Mood::Grumpy) => "Sera mutters. 'Only if the pages are quieter than this room.'",
        ("Sera", "history", Mood::Happy) => "Sera smiles. 'History is a feast if you know where to look.'",
        ("Sera", "history", Mood::Neutral) => "Sera nods. 'Most kingdoms repeat themselves with new banners.'",
        ("Sera", "history", Mood::Grumpy) => "Sera frowns. 'History is mostly people making the same mistake on purpose.'",
        ("Sera", "gossip", Mood::Happy) => "Sera gives a patient smile. 'I'd rather hear facts, but go on.'",
        ("Sera", "gossip", Mood::Neutral) => "Sera tilts her head. 'Rumor is an unreliable archive.'",
        ("Sera", "gossip", Mood::Grumpy) => "Sera closes her book. 'If I wanted gossip, I'd study crows.'",
        ("Torren", "hunting", Mood::Happy) => "Torren grins. 'Nothing clears the head like a trail at sunrise.'",
        ("Torren", "hunting", Mood::Neutral) => "Torren nods. 'A clean shot beats bragging.'",
        ("Torren", "hunting", Mood::Grumpy) => "Torren huffs. 'Only fools miss twice.'",
        ("Torren", "work", Mood::Happy) => "Torren smiles faintly. 'Hard work feels lighter after a full meal.'",
        ("Torren", "work", Mood::Neutral) => "Torren shrugs. 'Work gets done whether you complain or not.'",
        ("Torren", "work", Mood::Grumpy) => "Torren crosses his arms. 'Work is easier than people.'",
        ("Torren", "magic", Mood::Happy) => "Torren chuckles. 'Magic is useful when it stays far away from me.'",
        ("Torren", "magic", Mood::Neutral) => "Torren narrows his eyes. 'I trust steel more than sparks.'",
        ("Torren", "magic", Mood::Grumpy) => "Torren scowls. 'Magic ruins good tools and better plans.'",
        _ => "The conversation drifts without much consequence.",
    }
}

fn default_npcs() -> Vec<Npc> {
    vec![
        Npc {
            name: "Bram".to_string(),
            mood: Mood::Neutral,
            topics: vec!["ale".to_string(), "rumors".to_string(), "taxes".to_string()],
            affinity: 50,
        },
        Npc {
            name: "Lyra".to_string(),
            mood: Mood::Happy,
            topics: vec!["music".to_string(), "travel".to_string(), "weather".to_string()],
            affinity: 45,
        },
        Npc {
            name: "Sera".to_string(),
            mood: Mood::Neutral,
            topics: vec!["books".to_string(), "history".to_string(), "gossip".to_string()],
            affinity: 55,
        },
        Npc {
            name: "Torren".to_string(),
            mood: Mood::Grumpy,
            topics: vec!["hunting".to_string(), "work".to_string(), "magic".to_string()],
            affinity: 40,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conversation_flow_returns_response_and_updates_affinity() {
        let mut tavern = Tavern::new("The Copper Cup", TimeOfDay::Evening);

        let result = tavern.talk_to_npc("Lyra", "music").unwrap();

        assert_eq!(result.npc_name, "Lyra");
        assert_eq!(result.topic, "music");
        assert_eq!(result.affinity_change, 5);
        assert_eq!(result.new_affinity, 50);
        assert!(result.response.contains("Lyra taps the rhythm"));
        assert!(result.response.contains("willing to keep talking"));
    }

    #[test]
    fn affinity_changes_follow_topic_match_rules() {
        let mut tavern = Tavern::new("The Copper Cup", TimeOfDay::Afternoon);

        let good = tavern.talk_to_npc("Bram", "ale").unwrap();
        let bad = tavern.talk_to_npc("Bram", "taxes").unwrap();
        let neutral = tavern
            .talk_to_npc("Torren", "hunting")
            .and_then(|_| tavern.talk_to_npc("Torren", "work"))
            .unwrap();

        assert_eq!(good.affinity_change, 5);
        assert_eq!(bad.affinity_change, -10);
        assert_eq!(neutral.affinity_change, 5);
        assert_eq!(tavern.npc("Bram").unwrap().affinity, 45);
        assert_eq!(tavern.npc("Torren").unwrap().affinity, 50);
    }

    #[test]
    fn mood_transitions_improve_by_one_tier() {
        assert_eq!(Mood::Grumpy.improve(), Mood::Neutral);
        assert_eq!(Mood::Neutral.improve(), Mood::Happy);
        assert_eq!(Mood::Happy.improve(), Mood::Happy);
    }

    #[test]
    fn buy_drink_spends_gold_and_improves_mood() {
        let mut tavern = Tavern::new("The Copper Cup", TimeOfDay::Night);
        let mut gold = 13;

        let mood = tavern.buy_drink("Torren", &mut gold).unwrap();

        assert_eq!(mood, Mood::Neutral);
        assert_eq!(gold, 3);
        assert_eq!(tavern.npc("Torren").unwrap().mood, Mood::Neutral);
    }

    #[test]
    fn save_load_roundtrip_preserves_state() {
        let mut tavern = Tavern::new("The Copper Cup", TimeOfDay::Morning);
        let mut gold = 50;

        tavern.talk_to_npc("Sera", "books").unwrap();
        tavern.buy_drink("Sera", &mut gold).unwrap();

        let saved = tavern.save_to_json().unwrap();
        let loaded = Tavern::load_from_json(&saved).unwrap();

        assert_eq!(tavern, loaded);
    }

    #[test]
    fn demo_visit_runs_complete_flow() {
        let loaded = demo_visit().unwrap();
        let bram = loaded.npc("Bram").unwrap();

        assert_eq!(loaded.name, "The Copper Cup");
        assert_eq!(loaded.current_time_of_day, TimeOfDay::Evening);
        assert_eq!(bram.mood, Mood::Happy);
        assert_eq!(bram.affinity, 60);
    }
}

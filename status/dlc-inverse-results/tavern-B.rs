use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mood {
    Grumpy,
    Neutral,
    Happy,
}

impl Mood {
    /// Improve mood by one tier. Happy stays Happy.
    pub fn improve(self) -> Mood {
        match self {
            Mood::Grumpy => Mood::Neutral,
            Mood::Neutral => Mood::Happy,
            Mood::Happy => Mood::Happy,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeOfDay {
    Morning,
    Afternoon,
    Evening,
    Night,
}

/// Describes how well a topic matches an NPC's interests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopicMatch {
    Good,
    Neutral,
    Bad,
}

/// A single NPC that can be found in the tavern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcProfile {
    pub name: String,
    pub mood: Mood,
    /// Topics the NPC is willing to discuss.
    pub topics: Vec<String>,
    /// Affinity toward the player (0-100).
    pub affinity: u8,
    /// Per-topic, per-mood response text.
    /// Key: topic name -> inner map: mood variant -> response string.
    pub responses: HashMap<String, HashMap<String, String>>,
}

impl NpcProfile {
    pub fn new(
        name: &str,
        mood: Mood,
        topics: Vec<&str>,
        affinity: u8,
        responses: HashMap<String, HashMap<String, String>>,
    ) -> Self {
        Self {
            name: name.to_string(),
            mood,
            topics: topics.into_iter().map(String::from).collect(),
            affinity: affinity.min(100),
            responses,
        }
    }

    /// Classify how well a topic matches this NPC.
    pub fn classify_topic(&self, topic: &str) -> TopicMatch {
        if self.topics.iter().any(|t| t.eq_ignore_ascii_case(topic)) {
            TopicMatch::Good
        } else if topic.eq_ignore_ascii_case("weather") {
            // Weather is always a neutral fallback topic.
            TopicMatch::Neutral
        } else {
            TopicMatch::Bad
        }
    }

    /// Get the response string for a topic taking mood into account.
    pub fn respond(&self, topic: &str) -> String {
        let mood_key = format!("{:?}", self.mood);
        if let Some(mood_map) = self.responses.get(topic) {
            if let Some(text) = mood_map.get(&mood_key) {
                return text.clone();
            }
        }
        // Fallback when the topic or mood variant is missing.
        match self.mood {
            Mood::Happy => format!(
                "{} smiles. \"I don't know much about {}, but cheers!\"",
                self.name, topic
            ),
            Mood::Neutral => format!(
                "{} shrugs. \"{}? Can't say I have an opinion.\"",
                self.name, topic
            ),
            Mood::Grumpy => format!(
                "{} scowls. \"Why are you asking me about {}? Go away.\"",
                self.name, topic
            ),
        }
    }

    /// Adjust affinity based on topic match quality.
    pub fn adjust_affinity(&mut self, topic_match: TopicMatch) {
        match topic_match {
            TopicMatch::Good => self.affinity = (self.affinity + 5).min(100),
            TopicMatch::Neutral => self.affinity = (self.affinity + 1).min(100),
            TopicMatch::Bad => self.affinity = self.affinity.saturating_sub(10),
        }
    }
}

/// Result returned from a conversation turn.
#[derive(Debug, Clone)]
pub struct ConversationResult {
    pub npc_name: String,
    pub topic: String,
    pub response: String,
    pub topic_match: TopicMatch,
    pub new_affinity: u8,
}

/// The tavern itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tavern {
    pub name: String,
    pub npcs: Vec<NpcProfile>,
    pub time_of_day: TimeOfDay,
    pub player_gold: u32,
}

impl Tavern {
    /// Talk to an NPC about a topic. Returns None if the NPC isn't found.
    pub fn converse(&mut self, npc_name: &str, topic: &str) -> Option<ConversationResult> {
        let npc = self.npcs.iter_mut().find(|n| n.name == npc_name)?;
        let topic_match = npc.classify_topic(topic);
        let response = npc.respond(topic);
        npc.adjust_affinity(topic_match);
        let new_affinity = npc.affinity;
        Some(ConversationResult {
            npc_name: npc.name.clone(),
            topic: topic.to_string(),
            response,
            topic_match,
            new_affinity,
        })
    }

    /// Buy a drink for an NPC. Costs 10 gold and improves their mood one tier.
    pub fn buy_drink(&mut self, npc_name: &str) -> Result<Mood, String> {
        const DRINK_COST: u32 = 10;
        if self.player_gold < DRINK_COST {
            return Err("Not enough gold!".to_string());
        }
        let npc = self
            .npcs
            .iter_mut()
            .find(|n| n.name == npc_name)
            .ok_or_else(|| format!("NPC '{}' not found", npc_name))?;
        self.player_gold -= DRINK_COST;
        npc.mood = npc.mood.improve();
        Ok(npc.mood)
    }

    /// Serialize entire tavern state to JSON.
    pub fn save(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize tavern state from JSON.
    pub fn load(json: &str) -> Result<Tavern, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Find an NPC by name (immutable).
    pub fn find_npc(&self, name: &str) -> Option<&NpcProfile> {
        self.npcs.iter().find(|n| n.name == name)
    }
}

// ---------------------------------------------------------------------------
// Builders
// ---------------------------------------------------------------------------

fn mood_map(happy: &str, neutral: &str, grumpy: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("Happy".to_string(), happy.to_string());
    m.insert("Neutral".to_string(), neutral.to_string());
    m.insert("Grumpy".to_string(), grumpy.to_string());
    m
}

fn build_responses(
    entries: Vec<(&str, &str, &str, &str)>,
) -> HashMap<String, HashMap<String, String>> {
    let mut map = HashMap::new();
    for (topic, happy, neutral, grumpy) in entries {
        map.insert(topic.to_string(), mood_map(happy, neutral, grumpy));
    }
    map
}

/// Create a fully-populated default tavern with 4 NPCs.
pub fn create_default_tavern() -> Tavern {
    let gareth = NpcProfile::new(
        "Gareth",
        Mood::Neutral,
        vec!["swords", "quests", "honor"],
        50,
        build_responses(vec![
            (
                "swords",
                "\"Ah, blades! Let me tell you about my enchanted longsword — won it in a duel!\"",
                "\"Swords? I know a thing or two. What do you want to know?\"",
                "\"Don't touch my sword. I'm not in the mood.\"",
            ),
            (
                "quests",
                "\"Quests! I just finished clearing the goblin den — glorious!\"",
                "\"There are a few bounties on the board if you're interested.\"",
                "\"Quests are nothing but trouble. Leave me alone.\"",
            ),
            (
                "honor",
                "\"Honor is the greatest treasure a warrior can possess!\"",
                "\"Honor matters, I suppose. Depends who you ask.\"",
                "\"Honor? Ha! That won't fill your belly.\"",
            ),
        ]),
    );

    let elara = NpcProfile::new(
        "Elara",
        Mood::Happy,
        vec!["magic", "potions", "stars"],
        60,
        build_responses(vec![
            (
                "magic",
                "\"Magic is wonderful! I just learned a new frost spell!\"",
                "\"Magic has its uses. I study it when I can.\"",
                "\"Magic? It's more trouble than it's worth today.\"",
            ),
            (
                "potions",
                "\"I brewed a healing draught this morning — want to try some?\"",
                "\"Potions take patience. I have a few in stock.\"",
                "\"My last potion exploded. Don't ask.\"",
            ),
            (
                "stars",
                "\"The stars were breathtaking last night! Did you see the comet?\"",
                "\"The stars hold many secrets for those who watch.\"",
                "\"The sky was cloudy. I couldn't see anything.\"",
            ),
        ]),
    );

    let brom = NpcProfile::new(
        "Brom",
        Mood::Grumpy,
        vec!["ale", "rumors", "fights"],
        30,
        build_responses(vec![
            (
                "ale",
                "\"This is the finest ale I've ever tasted! Barkeep, another round!\"",
                "\"The ale's decent tonight, I'll give 'em that.\"",
                "\"This ale tastes like swamp water. Ugh.\"",
            ),
            (
                "rumors",
                "\"Ha! I heard the mayor's been secretly funding the thieves' guild!\"",
                "\"There's talk of bandits on the north road. Be careful.\"",
                "\"I don't spread rumors. Get lost.\"",
            ),
            (
                "fights",
                "\"You should've seen the brawl last night — I knocked out three guys!\"",
                "\"Fights happen here. Best keep your guard up.\"",
                "\"You looking for a fight? Because I'll give you one.\"",
            ),
        ]),
    );

    let miriel = NpcProfile::new(
        "Miriel",
        Mood::Neutral,
        vec!["music", "legends", "travel"],
        45,
        build_responses(vec![
            (
                "music",
                "\"Let me play you a joyful tune!\"",
                "\"I could play something if you'd like. Any requests?\"",
                "\"My lute string broke. No music tonight.\"",
            ),
            (
                "legends",
                "\"Oh, the legend of the Dragon King! Sit down, this is a good one!\"",
                "\"Legends? I know a few. Which one interests you?\"",
                "\"Legends are just lies dressed up in fancy words.\"",
            ),
            (
                "travel",
                "\"I just came back from the Emerald Isles — absolutely stunning!\"",
                "\"I've traveled a fair bit. The eastern roads are safest.\"",
                "\"Travel is exhausting. I'm staying right here.\"",
            ),
        ]),
    );

    Tavern {
        name: "The Rusty Flagon".to_string(),
        npcs: vec![gareth, elara, brom, miriel],
        time_of_day: TimeOfDay::Evening,
        player_gold: 50,
    }
}

// ---------------------------------------------------------------------------
// Demo
// ---------------------------------------------------------------------------

pub fn demo() {
    println!("=== Tavern Social System Demo ===\n");

    let mut tavern = create_default_tavern();
    println!(
        "You enter '{}'. It is {:?}.\nYou have {} gold.\n",
        tavern.name, tavern.time_of_day, tavern.player_gold
    );

    println!("NPCs present:");
    for npc in &tavern.npcs {
        println!(
            "  - {} (mood: {:?}, affinity: {}, topics: {:?})",
            npc.name, npc.mood, npc.affinity, npc.topics
        );
    }
    println!();

    // Talk to Brom about ale (Grumpy mood, good topic)
    println!("--- Talking to Brom about 'ale' ---");
    if let Some(r) = tavern.converse("Brom", "ale") {
        println!("  Response: {}", r.response);
        println!(
            "  Topic match: {:?}, New affinity: {}\n",
            r.topic_match, r.new_affinity
        );
    }

    // Buy Brom a drink (Grumpy -> Neutral)
    println!("--- Buying Brom a drink ---");
    match tavern.buy_drink("Brom") {
        Ok(new_mood) => println!(
            "  Brom's new mood: {:?} (gold remaining: {})\n",
            new_mood, tavern.player_gold
        ),
        Err(e) => println!("  Error: {}\n", e),
    }

    // Talk to Brom again (now Neutral)
    println!("--- Talking to Brom about 'rumors' ---");
    if let Some(r) = tavern.converse("Brom", "rumors") {
        println!("  Response: {}", r.response);
        println!(
            "  Topic match: {:?}, New affinity: {}\n",
            r.topic_match, r.new_affinity
        );
    }

    // Talk to Elara about magic
    println!("--- Talking to Elara about 'magic' ---");
    if let Some(r) = tavern.converse("Elara", "magic") {
        println!("  Response: {}", r.response);
        println!(
            "  Topic match: {:?}, New affinity: {}\n",
            r.topic_match, r.new_affinity
        );
    }

    // Talk to Gareth about a bad topic
    println!("--- Talking to Gareth about 'cooking' (bad topic) ---");
    if let Some(r) = tavern.converse("Gareth", "cooking") {
        println!("  Response: {}", r.response);
        println!(
            "  Topic match: {:?}, New affinity: {}\n",
            r.topic_match, r.new_affinity
        );
    }

    // Save
    println!("--- Saving tavern state ---");
    let json = tavern.save().expect("serialization failed");
    println!("  Saved {} bytes of JSON.\n", json.len());

    // Load
    println!("--- Loading tavern state ---");
    let loaded = Tavern::load(&json).expect("deserialization failed");
    println!("  Tavern name: {}", loaded.name);
    println!("  Gold: {}", loaded.player_gold);
    for npc in &loaded.npcs {
        println!(
            "  {} — mood: {:?}, affinity: {}",
            npc.name, npc.mood, npc.affinity
        );
    }
    println!();

    // Verify round-trip
    assert_eq!(tavern.name, loaded.name);
    assert_eq!(tavern.player_gold, loaded.player_gold);
    assert_eq!(tavern.npcs.len(), loaded.npcs.len());
    for (a, b) in tavern.npcs.iter().zip(loaded.npcs.iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.mood, b.mood);
        assert_eq!(a.affinity, b.affinity);
    }
    println!("Save/load round-trip verified!\n");
    println!("=== Demo complete ===");
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    demo();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tavern() -> Tavern {
        create_default_tavern()
    }

    #[test]
    fn test_conversation_flow() {
        let mut tavern = test_tavern();

        // Known NPC + known topic succeeds
        let result = tavern.converse("Elara", "magic").expect("NPC should exist");
        assert_eq!(result.npc_name, "Elara");
        assert_eq!(result.topic, "magic");
        assert_eq!(result.topic_match, TopicMatch::Good);
        // Elara starts Happy so expect happy-variant response
        assert!(result.response.contains("frost spell"));

        // Unknown NPC returns None
        assert!(tavern.converse("Ghost", "magic").is_none());
    }

    #[test]
    fn test_affinity_changes() {
        let mut tavern = test_tavern();
        let initial = tavern.find_npc("Gareth").unwrap().affinity; // 50

        // Good topic -> +5
        tavern.converse("Gareth", "swords");
        assert_eq!(tavern.find_npc("Gareth").unwrap().affinity, initial + 5);

        // Neutral topic (weather fallback) -> +1
        tavern.converse("Gareth", "weather");
        assert_eq!(
            tavern.find_npc("Gareth").unwrap().affinity,
            initial + 5 + 1
        );

        // Bad topic -> -10
        tavern.converse("Gareth", "cooking");
        assert_eq!(
            tavern.find_npc("Gareth").unwrap().affinity,
            initial + 5 + 1 - 10
        );
    }

    #[test]
    fn test_mood_transitions() {
        assert_eq!(Mood::Grumpy.improve(), Mood::Neutral);
        assert_eq!(Mood::Neutral.improve(), Mood::Happy);
        assert_eq!(Mood::Happy.improve(), Mood::Happy);

        // Mood affects response text
        let mut tavern = test_tavern();
        // Brom starts Grumpy
        let grumpy_resp = tavern.converse("Brom", "ale").unwrap().response;
        assert!(grumpy_resp.contains("swamp water"));

        // Improve mood then check again
        tavern.buy_drink("Brom").unwrap();
        let neutral_resp = tavern.converse("Brom", "ale").unwrap().response;
        assert!(neutral_resp.contains("decent"));
    }

    #[test]
    fn test_buy_drink() {
        let mut tavern = test_tavern();

        assert_eq!(tavern.find_npc("Brom").unwrap().mood, Mood::Grumpy);
        assert_eq!(tavern.player_gold, 50);

        // Grumpy -> Neutral, 50 -> 40
        let mood = tavern.buy_drink("Brom").unwrap();
        assert_eq!(mood, Mood::Neutral);
        assert_eq!(tavern.player_gold, 40);

        // Neutral -> Happy, 40 -> 30
        let mood = tavern.buy_drink("Brom").unwrap();
        assert_eq!(mood, Mood::Happy);
        assert_eq!(tavern.player_gold, 30);

        // Unknown NPC -> error
        assert!(tavern.buy_drink("Ghost").is_err());

        // Not enough gold -> error
        tavern.player_gold = 5;
        assert!(tavern.buy_drink("Brom").is_err());
    }

    #[test]
    fn test_save_load_roundtrip() {
        let mut tavern = test_tavern();

        // Mutate state so it differs from defaults
        tavern.converse("Elara", "potions"); // affinity +5
        let _ = tavern.buy_drink("Brom"); // mood Grumpy->Neutral, gold -10
        tavern.time_of_day = TimeOfDay::Night;

        let json = tavern.save().expect("serialize");
        let loaded = Tavern::load(&json).expect("deserialize");

        assert_eq!(loaded.name, tavern.name);
        assert_eq!(loaded.player_gold, tavern.player_gold);
        assert_eq!(loaded.time_of_day, tavern.time_of_day);
        assert_eq!(loaded.npcs.len(), tavern.npcs.len());

        for (orig, restored) in tavern.npcs.iter().zip(loaded.npcs.iter()) {
            assert_eq!(orig.name, restored.name);
            assert_eq!(orig.mood, restored.mood);
            assert_eq!(orig.affinity, restored.affinity);
            assert_eq!(orig.topics, restored.topics);
        }
    }
}

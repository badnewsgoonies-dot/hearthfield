use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── Mood ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mood {
    Grumpy,
    Neutral,
    Happy,
}

impl Mood {
    pub fn improve(self) -> Mood {
        match self {
            Mood::Grumpy => Mood::Neutral,
            Mood::Neutral => Mood::Happy,
            Mood::Happy => Mood::Happy,
        }
    }
}

impl std::fmt::Display for Mood {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mood::Grumpy => write!(f, "Grumpy"),
            Mood::Neutral => write!(f, "Neutral"),
            Mood::Happy => write!(f, "Happy"),
        }
    }
}

// ── TopicResponse ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicResponse {
    pub happy: String,
    pub neutral: String,
    pub grumpy: String,
}

impl TopicResponse {
    pub fn for_mood(&self, mood: Mood) -> &str {
        match mood {
            Mood::Happy => &self.happy,
            Mood::Neutral => &self.neutral,
            Mood::Grumpy => &self.grumpy,
        }
    }
}

// ── NpcProfile ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpcProfile {
    pub name: String,
    pub mood: Mood,
    pub topics: Vec<String>,
    pub affinity: i32,
    /// Maps topic name → mood-variant responses
    pub responses: HashMap<String, TopicResponse>,
    /// Topics the NPC especially likes (good match → +5 affinity)
    pub preferred_topics: Vec<String>,
    /// Topics the NPC dislikes (bad match → -10 affinity)
    pub disliked_topics: Vec<String>,
}

impl NpcProfile {
    pub fn clamp_affinity(&mut self) {
        self.affinity = self.affinity.clamp(0, 100);
    }
}

// ── ConversationResult ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ConversationResult {
    pub npc_name: String,
    pub topic: String,
    pub response_text: String,
    pub affinity_change: i32,
    pub new_affinity: i32,
}

// ── BuyDrinkResult ──────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct BuyDrinkResult {
    pub npc_name: String,
    pub old_mood: Mood,
    pub new_mood: Mood,
    pub gold_spent: u32,
}

// ── Tavern ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tavern {
    pub name: String,
    pub npcs: Vec<NpcProfile>,
    pub time_of_day: String,
    pub player_gold: u32,
}

impl Tavern {
    /// Find an NPC by name (immutable).
    pub fn find_npc(&self, name: &str) -> Option<&NpcProfile> {
        self.npcs.iter().find(|n| n.name == name)
    }

    /// Find an NPC by name (mutable).
    fn find_npc_mut(&mut self, name: &str) -> Option<&mut NpcProfile> {
        self.npcs.iter_mut().find(|n| n.name == name)
    }

    /// Talk to an NPC about a topic.
    pub fn talk_to(&mut self, npc_name: &str, topic: &str) -> Result<ConversationResult, String> {
        let npc = self
            .find_npc_mut(npc_name)
            .ok_or_else(|| format!("NPC '{}' not found in the tavern", npc_name))?;

        let response = npc
            .responses
            .get(topic)
            .ok_or_else(|| format!("'{}' has nothing to say about '{}'", npc_name, topic))?;

        let response_text = response.for_mood(npc.mood).to_string();

        // Determine affinity change
        let affinity_change = if npc.preferred_topics.contains(&topic.to_string()) {
            5
        } else if npc.disliked_topics.contains(&topic.to_string()) {
            -10
        } else {
            1
        };

        npc.affinity += affinity_change;
        npc.clamp_affinity();
        let new_affinity = npc.affinity;

        Ok(ConversationResult {
            npc_name: npc_name.to_string(),
            topic: topic.to_string(),
            response_text,
            affinity_change,
            new_affinity,
        })
    }

    /// Buy a drink for an NPC, improving their mood by one tier. Costs 10 gold.
    pub fn buy_drink(&mut self, npc_name: &str) -> Result<BuyDrinkResult, String> {
        const DRINK_COST: u32 = 10;

        if self.player_gold < DRINK_COST {
            return Err(format!(
                "Not enough gold! Need {} but only have {}",
                DRINK_COST, self.player_gold
            ));
        }

        let npc = self
            .find_npc_mut(npc_name)
            .ok_or_else(|| format!("NPC '{}' not found in the tavern", npc_name))?;

        let old_mood = npc.mood;
        npc.mood = npc.mood.improve();
        let new_mood = npc.mood;

        self.player_gold -= DRINK_COST;

        Ok(BuyDrinkResult {
            npc_name: npc_name.to_string(),
            old_mood,
            new_mood,
            gold_spent: DRINK_COST,
        })
    }

    /// Serialize tavern state to JSON.
    pub fn save(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("Save failed: {}", e))
    }

    /// Deserialize tavern state from JSON.
    pub fn load(json: &str) -> Result<Tavern, String> {
        serde_json::from_str(json).map_err(|e| format!("Load failed: {}", e))
    }

    /// List all NPC names currently in the tavern.
    pub fn list_npcs(&self) -> Vec<&str> {
        self.npcs.iter().map(|n| n.name.as_str()).collect()
    }
}

// ── NPC Factory ─────────────────────────────────────────────────────────────

fn build_default_tavern() -> Tavern {
    let barkeep = NpcProfile {
        name: "Greta the Barkeep".to_string(),
        mood: Mood::Neutral,
        topics: vec![
            "rumors".to_string(),
            "drinks".to_string(),
            "politics".to_string(),
        ],
        affinity: 50,
        responses: HashMap::from([
            (
                "rumors".to_string(),
                TopicResponse {
                    happy: "Oh, you want the good stuff? Word is a dragon was spotted near Ashvale!".to_string(),
                    neutral: "I hear things. Travelers say the mountain pass is dangerous lately.".to_string(),
                    grumpy: "You think I have time for gossip? ...Fine. Something's lurking in the mines.".to_string(),
                },
            ),
            (
                "drinks".to_string(),
                TopicResponse {
                    happy: "Try the Moonfire Mead! Brewed it myself — my finest batch yet!".to_string(),
                    neutral: "We have ale, mead, and wine. What'll it be?".to_string(),
                    grumpy: "Ale's over there. Pour it yourself.".to_string(),
                },
            ),
            (
                "politics".to_string(),
                TopicResponse {
                    happy: "The new mayor? She's doing great things for the town, I say!".to_string(),
                    neutral: "Politics? I try to stay out of it. Keeps the peace.".to_string(),
                    grumpy: "Don't bring that nonsense into my tavern.".to_string(),
                },
            ),
        ]),
        preferred_topics: vec!["drinks".to_string()],
        disliked_topics: vec!["politics".to_string()],
    };

    let bard = NpcProfile {
        name: "Finnian the Bard".to_string(),
        mood: Mood::Happy,
        topics: vec![
            "music".to_string(),
            "adventures".to_string(),
            "romance".to_string(),
        ],
        affinity: 60,
        responses: HashMap::from([
            (
                "music".to_string(),
                TopicResponse {
                    happy: "Ah, a fellow music lover! Let me play you the Ballad of the Silver Moon!".to_string(),
                    neutral: "I suppose I could play a tune. Any requests?".to_string(),
                    grumpy: "My lute strings are old and I'm not in the mood. Maybe later.".to_string(),
                },
            ),
            (
                "adventures".to_string(),
                TopicResponse {
                    happy: "I once sailed with pirates across the Shimmering Sea! Want to hear the tale?".to_string(),
                    neutral: "I've had a few adventures in my day. Mostly survived them.".to_string(),
                    grumpy: "Adventures? More like near-death experiences. Pass.".to_string(),
                },
            ),
            (
                "romance".to_string(),
                TopicResponse {
                    happy: "Love! The greatest inspiration for any song! Let me tell you about the Lady of Willowmere...".to_string(),
                    neutral: "Romance? It makes for good ballads, I'll give it that.".to_string(),
                    grumpy: "Love is overrated. Next topic.".to_string(),
                },
            ),
        ]),
        preferred_topics: vec!["music".to_string()],
        disliked_topics: vec!["romance".to_string()],
    };

    let mercenary = NpcProfile {
        name: "Kael the Mercenary".to_string(),
        mood: Mood::Grumpy,
        topics: vec![
            "weapons".to_string(),
            "contracts".to_string(),
            "war_stories".to_string(),
        ],
        affinity: 20,
        responses: HashMap::from([
            (
                "weapons".to_string(),
                TopicResponse {
                    happy: "This blade? Dwarven steel! Won it off a champion in a duel. Want to hold it?".to_string(),
                    neutral: "I carry a longsword and a dagger. Tools of the trade.".to_string(),
                    grumpy: "Keep your hands off my gear.".to_string(),
                },
            ),
            (
                "contracts".to_string(),
                TopicResponse {
                    happy: "Looking for a sword for hire? I might give you a discount — you're alright!".to_string(),
                    neutral: "Got coin? I've got steel. Standard rates apply.".to_string(),
                    grumpy: "I don't work cheap, and I don't work for fools.".to_string(),
                },
            ),
            (
                "war_stories".to_string(),
                TopicResponse {
                    happy: "Ha! Let me tell you about the Siege of Thornwall — fifty against five hundred!".to_string(),
                    neutral: "I've seen my share of battles. The Thornwall campaign was the worst.".to_string(),
                    grumpy: "War isn't a story. It's blood and mud. Don't romanticize it.".to_string(),
                },
            ),
        ]),
        preferred_topics: vec!["weapons".to_string()],
        disliked_topics: vec!["contracts".to_string()],
    };

    let witch = NpcProfile {
        name: "Elara the Hedge Witch".to_string(),
        mood: Mood::Neutral,
        topics: vec![
            "potions".to_string(),
            "curses".to_string(),
            "herbs".to_string(),
        ],
        affinity: 40,
        responses: HashMap::from([
            (
                "potions".to_string(),
                TopicResponse {
                    happy: "I just finished a batch of Dreamwalk Elixir! The visions are magnificent!".to_string(),
                    neutral: "I can brew healing draughts, antidotes, and a few... other things.".to_string(),
                    grumpy: "My potions aren't for amateurs. Go buy something from the general store.".to_string(),
                },
            ),
            (
                "curses".to_string(),
                TopicResponse {
                    happy: "Curses are just misunderstood magic! I can remove one for a fair price.".to_string(),
                    neutral: "Curses? Dangerous business. I can help, but it won't be free.".to_string(),
                    grumpy: "You want to talk about curses? Careful what you wish for.".to_string(),
                },
            ),
            (
                "herbs".to_string(),
                TopicResponse {
                    happy: "The meadows are blooming with Starleaf! Perfect for tinctures!".to_string(),
                    neutral: "I forage most mornings. The forest provides what I need.".to_string(),
                    grumpy: "Herbs? What do you know about herbs? Probably couldn't tell sage from ragweed.".to_string(),
                },
            ),
        ]),
        preferred_topics: vec!["herbs".to_string()],
        disliked_topics: vec!["curses".to_string()],
    };

    Tavern {
        name: "The Gilded Flagon".to_string(),
        npcs: vec![barkeep, bard, mercenary, witch],
        time_of_day: "Evening".to_string(),
        player_gold: 50,
    }
}

// ── Demo ────────────────────────────────────────────────────────────────────

pub fn demo() {
    println!("═══════════════════════════════════════════════════════════");
    println!("  Welcome to the Tavern Social System Demo");
    println!("═══════════════════════════════════════════════════════════\n");

    let mut tavern = build_default_tavern();

    // ── Enter the tavern ────────────────────────────────────────────────
    println!(
        "You push open the heavy oak door and step into '{}'. It is {}.",
        tavern.name, tavern.time_of_day
    );
    println!(
        "You have {} gold in your pouch.\n",
        tavern.player_gold
    );

    // ── List NPCs ───────────────────────────────────────────────────────
    println!("NPCs present ({}):", tavern.list_npcs().len());
    for npc in &tavern.npcs {
        println!(
            "  • {} (Mood: {}, Affinity: {}, Topics: {})",
            npc.name,
            npc.mood,
            npc.affinity,
            npc.topics.join(", ")
        );
    }
    println!();

    // ── Talk to every NPC on each of their topics ───────────────────────
    println!("── Conversations ──────────────────────────────────────────\n");

    let npc_topics: Vec<(String, Vec<String>)> = tavern
        .npcs
        .iter()
        .map(|n| (n.name.clone(), n.topics.clone()))
        .collect();

    for (name, topics) in &npc_topics {
        for topic in topics {
            match tavern.talk_to(name, topic) {
                Ok(result) => {
                    println!("[Talk to {} about '{}']", result.npc_name, result.topic);
                    println!("  \"{}\"", result.response_text);
                    println!(
                        "  (Affinity {:+} → now {})\n",
                        result.affinity_change, result.new_affinity
                    );
                }
                Err(e) => println!("  Error: {}\n", e),
            }
        }
    }

    // ── Buy a drink for the grumpy mercenary ────────────────────────────
    println!("── Buy Drinks ─────────────────────────────────────────────\n");

    match tavern.buy_drink("Kael the Mercenary") {
        Ok(result) => {
            println!(
                "You buy a drink for {}. ({} gold spent, {} gold remaining)",
                result.npc_name, result.gold_spent, tavern.player_gold
            );
            println!(
                "  Mood: {} → {}\n",
                result.old_mood, result.new_mood
            );
        }
        Err(e) => println!("  Error: {}\n", e),
    }

    // Buy a second drink to go from Neutral → Happy
    match tavern.buy_drink("Kael the Mercenary") {
        Ok(result) => {
            println!(
                "You buy another drink for {}. ({} gold spent, {} gold remaining)",
                result.npc_name, result.gold_spent, tavern.player_gold
            );
            println!(
                "  Mood: {} → {}\n",
                result.old_mood, result.new_mood
            );
        }
        Err(e) => println!("  Error: {}\n", e),
    }

    // ── Talk again — mood should now affect response ────────────────────
    println!("── Post-Drink Conversation ─────────────────────────────────\n");

    match tavern.talk_to("Kael the Mercenary", "weapons") {
        Ok(result) => {
            println!(
                "[Talk to {} about '{}' (now Happy)]",
                result.npc_name, result.topic
            );
            println!("  \"{}\"", result.response_text);
            println!(
                "  (Affinity {:+} → now {})\n",
                result.affinity_change, result.new_affinity
            );
        }
        Err(e) => println!("  Error: {}\n", e),
    }

    // ── Save state ──────────────────────────────────────────────────────
    println!("── Save / Load ────────────────────────────────────────────\n");

    let saved_json = tavern.save().expect("Failed to save tavern state");
    println!("Tavern state saved ({} bytes of JSON).", saved_json.len());

    // ── Load state ──────────────────────────────────────────────────────
    let loaded_tavern = Tavern::load(&saved_json).expect("Failed to load tavern state");
    println!("Tavern state loaded successfully.\n");

    // ── Verify loaded state ─────────────────────────────────────────────
    println!("── Verification ───────────────────────────────────────────\n");

    let original_kael = tavern.find_npc("Kael the Mercenary").unwrap();
    let loaded_kael = loaded_tavern.find_npc("Kael the Mercenary").unwrap();

    println!("Kael (original): mood={}, affinity={}", original_kael.mood, original_kael.affinity);
    println!("Kael (loaded):   mood={}, affinity={}", loaded_kael.mood, loaded_kael.affinity);
    assert_eq!(original_kael.mood, loaded_kael.mood);
    assert_eq!(original_kael.affinity, loaded_kael.affinity);
    assert_eq!(tavern.player_gold, loaded_tavern.player_gold);
    println!("Gold (original): {}", tavern.player_gold);
    println!("Gold (loaded):   {}", loaded_tavern.player_gold);
    println!("\n✓ All state verified — save/load roundtrip is faithful.");

    println!("\n═══════════════════════════════════════════════════════════");
    println!("  Demo complete!");
    println!("═══════════════════════════════════════════════════════════");
}

// ── Main ────────────────────────────────────────────────────────────────────

fn main() {
    demo();
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tavern() -> Tavern {
        build_default_tavern()
    }

    #[test]
    fn test_conversation_flow() {
        let mut tavern = test_tavern();

        // Talk to the bard about music (preferred topic)
        let result = tavern.talk_to("Finnian the Bard", "music").unwrap();
        assert_eq!(result.npc_name, "Finnian the Bard");
        assert_eq!(result.topic, "music");
        // Bard starts Happy, so we should get the happy response
        assert!(result.response_text.contains("Ballad of the Silver Moon"));
        assert_eq!(result.affinity_change, 5); // preferred topic

        // Talking about an unknown topic should fail
        let err = tavern.talk_to("Finnian the Bard", "cooking");
        assert!(err.is_err());

        // Talking to an unknown NPC should fail
        let err = tavern.talk_to("Ghost", "anything");
        assert!(err.is_err());
    }

    #[test]
    fn test_affinity_changes() {
        let mut tavern = test_tavern();
        let initial_affinity = tavern.find_npc("Greta the Barkeep").unwrap().affinity; // 50

        // Preferred topic: +5
        let result = tavern.talk_to("Greta the Barkeep", "drinks").unwrap();
        assert_eq!(result.affinity_change, 5);
        assert_eq!(result.new_affinity, initial_affinity + 5);

        // Disliked topic: -10
        let result = tavern.talk_to("Greta the Barkeep", "politics").unwrap();
        assert_eq!(result.affinity_change, -10);
        assert_eq!(result.new_affinity, initial_affinity + 5 - 10);

        // Neutral topic: +1
        let result = tavern.talk_to("Greta the Barkeep", "rumors").unwrap();
        assert_eq!(result.affinity_change, 1);
        assert_eq!(result.new_affinity, initial_affinity + 5 - 10 + 1);
    }

    #[test]
    fn test_mood_transitions() {
        let mut tavern = test_tavern();

        // Kael starts Grumpy
        assert_eq!(
            tavern.find_npc("Kael the Mercenary").unwrap().mood,
            Mood::Grumpy
        );

        // Grumpy → Neutral
        let result = tavern.buy_drink("Kael the Mercenary").unwrap();
        assert_eq!(result.old_mood, Mood::Grumpy);
        assert_eq!(result.new_mood, Mood::Neutral);
        assert_eq!(
            tavern.find_npc("Kael the Mercenary").unwrap().mood,
            Mood::Neutral
        );

        // Neutral → Happy
        let result = tavern.buy_drink("Kael the Mercenary").unwrap();
        assert_eq!(result.old_mood, Mood::Neutral);
        assert_eq!(result.new_mood, Mood::Happy);

        // Happy → Happy (caps out)
        let result = tavern.buy_drink("Kael the Mercenary").unwrap();
        assert_eq!(result.old_mood, Mood::Happy);
        assert_eq!(result.new_mood, Mood::Happy);
    }

    #[test]
    fn test_buy_drink_gold_and_errors() {
        let mut tavern = test_tavern();
        assert_eq!(tavern.player_gold, 50);

        tavern.buy_drink("Greta the Barkeep").unwrap();
        assert_eq!(tavern.player_gold, 40);

        // Set gold low to test insufficient funds
        tavern.player_gold = 5;
        let err = tavern.buy_drink("Greta the Barkeep");
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("Not enough gold"));

        // Unknown NPC
        tavern.player_gold = 50;
        let err = tavern.buy_drink("Nobody");
        assert!(err.is_err());
    }

    #[test]
    fn test_save_load_roundtrip() {
        let mut tavern = test_tavern();

        // Mutate state: talk, buy drinks
        tavern.talk_to("Greta the Barkeep", "drinks").unwrap();
        tavern.buy_drink("Kael the Mercenary").unwrap();
        tavern.talk_to("Kael the Mercenary", "contracts").unwrap();

        // Save
        let json = tavern.save().unwrap();

        // Load
        let loaded = Tavern::load(&json).unwrap();

        // Verify all state
        assert_eq!(tavern.name, loaded.name);
        assert_eq!(tavern.time_of_day, loaded.time_of_day);
        assert_eq!(tavern.player_gold, loaded.player_gold);
        assert_eq!(tavern.npcs.len(), loaded.npcs.len());

        for (orig, load) in tavern.npcs.iter().zip(loaded.npcs.iter()) {
            assert_eq!(orig.name, load.name);
            assert_eq!(orig.mood, load.mood);
            assert_eq!(orig.affinity, load.affinity);
            assert_eq!(orig.topics, load.topics);
            assert_eq!(orig.preferred_topics, load.preferred_topics);
            assert_eq!(orig.disliked_topics, load.disliked_topics);
            assert_eq!(orig.responses.len(), load.responses.len());
            for (topic, resp) in &orig.responses {
                let load_resp = load.responses.get(topic).unwrap();
                assert_eq!(resp.happy, load_resp.happy);
                assert_eq!(resp.neutral, load_resp.neutral);
                assert_eq!(resp.grumpy, load_resp.grumpy);
            }
        }

        // Verify specific mutated values
        let greta = loaded.find_npc("Greta the Barkeep").unwrap();
        assert_eq!(greta.affinity, 55); // started 50, +5 for preferred topic "drinks"

        let kael = loaded.find_npc("Kael the Mercenary").unwrap();
        assert_eq!(kael.mood, Mood::Neutral); // was Grumpy, bought one drink
        assert_eq!(kael.affinity, 10); // started 20, -10 for disliked topic "contracts"
    }

    #[test]
    fn test_affinity_clamping() {
        let mut tavern = test_tavern();

        // Set affinity near 0 and trigger disliked topic
        tavern.find_npc_mut("Kael the Mercenary").unwrap().affinity = 3;
        let result = tavern.talk_to("Kael the Mercenary", "contracts").unwrap();
        assert_eq!(result.affinity_change, -10);
        assert_eq!(result.new_affinity, 0); // clamped, not -7

        // Set affinity near 100 and trigger preferred topic
        tavern.find_npc_mut("Kael the Mercenary").unwrap().affinity = 98;
        let result = tavern.talk_to("Kael the Mercenary", "weapons").unwrap();
        assert_eq!(result.affinity_change, 5);
        assert_eq!(result.new_affinity, 100); // clamped, not 103
    }

    #[test]
    fn test_mood_affects_response_text() {
        let mut tavern = test_tavern();

        // Kael is Grumpy — talk about weapons
        let grumpy_result = tavern.talk_to("Kael the Mercenary", "weapons").unwrap();
        assert!(grumpy_result.response_text.contains("Keep your hands off"));

        // Buy two drinks: Grumpy → Neutral → Happy
        tavern.buy_drink("Kael the Mercenary").unwrap();
        tavern.buy_drink("Kael the Mercenary").unwrap();

        // Now Happy — same topic, different response
        let happy_result = tavern.talk_to("Kael the Mercenary", "weapons").unwrap();
        assert!(happy_result.response_text.contains("Dwarven steel"));
        assert_ne!(grumpy_result.response_text, happy_result.response_text);
    }
}

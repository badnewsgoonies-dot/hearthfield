//! Dialogue system: handle player interaction with NPCs, build dialogue lines
//! based on friendship level, and emit DialogueStartEvent.

use bevy::prelude::*;
use crate::shared::*;

/// Component marking an NPC that the player is adjacent to and can interact with.
#[derive(Component, Debug)]
#[allow(dead_code)]
pub struct NpcInteractable;

/// Resource tracking the last-interacted NPC (for gift-giving context).
#[derive(Resource, Debug, Default)]
pub struct ActiveNpcInteraction {
    pub npc_id: Option<String>,
}

/// System: detect player pressing F (interact) near an NPC and start dialogue.
pub fn handle_npc_interaction(
    player_input: Res<PlayerInput>,
    input_blocks: Res<InputBlocks>,
    player_query: Query<&Transform, With<Player>>,
    npc_query: Query<(&Npc, &Transform)>,
    relationships: Res<Relationships>,
    npc_registry: Res<NpcRegistry>,
    calendar: Res<Calendar>,
    mut dialogue_writer: EventWriter<DialogueStartEvent>,
    mut active_interaction: ResMut<ActiveNpcInteraction>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_claimed: ResMut<InteractionClaimed>,
) {
    // Only check interaction during Playing state
    if *current_state.get() != GameState::Playing {
        return;
    }

    if input_blocks.is_blocked() {
        return;
    }

    if !player_input.interact {
        return;
    }

    if interaction_claimed.0 {
        return;
    }

    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let interaction_range = TILE_SIZE * 1.5; // 1.5 tiles in world space

    // Find the closest adjacent NPC within range
    let mut closest: Option<(&Npc, f32)> = None;
    for (npc, npc_transform) in npc_query.iter() {
        let npc_pos = npc_transform.translation.truncate();
        let dist = player_pos.distance(npc_pos);
        if dist <= interaction_range {
            match closest {
                None => closest = Some((npc, dist)),
                Some((_, best_dist)) if dist < best_dist => closest = Some((npc, dist)),
                _ => {}
            }
        }
    }

    let Some((npc, _)) = closest else {
        return;
    };

    let npc_id = &npc.id;
    let hearts = relationships.hearts(npc_id);
    let lines = build_dialogue_lines(npc_id, hearts, &npc_registry, &relationships, &calendar);

    let portrait_index = npc_registry.npcs.get(npc_id)
        .map(|def| def.portrait_index);

    active_interaction.npc_id = Some(npc_id.clone());
    interaction_claimed.0 = true;

    dialogue_writer.send(DialogueStartEvent {
        npc_id: npc_id.clone(),
        lines,
        portrait_index,
    });

    next_state.set(GameState::Dialogue);
}

/// Build dialogue lines for an NPC at the given friendship tier.
/// Falls back through lower tiers to always provide content.
/// Also prepends contextual lines based on weather and season.
pub fn build_dialogue_lines(
    npc_id: &str,
    hearts: u8,
    npc_registry: &NpcRegistry,
    relationships: &Relationships,
    calendar: &Calendar,
) -> Vec<String> {
    let Some(npc_def) = npc_registry.npcs.get(npc_id) else {
        return vec!["...".to_string()];
    };

    let mut lines = Vec::new();

    // --- Contextual: "already gifted today" notice ---
    if relationships.gifted_today.get(npc_id).copied().unwrap_or(false) {
        lines.push(
            "I really appreciate everything you've given me today. Come back tomorrow!".to_string(),
        );
        return lines;
    }

    // --- Contextual: birthday greeting (checked here with real calendar) ---
    let is_birthday =
        calendar.season == npc_def.birthday_season && calendar.day == npc_def.birthday_day;
    if is_birthday {
        lines.push(format!(
            "Oh! Today is my birthday! I can't believe you remembered, {}!",
            npc_def.name
        ));
    }

    // --- Contextual: weather comment (prepend one line) ---
    let weather_line = npc_weather_comment(npc_id, calendar.weather);
    if let Some(wl) = weather_line {
        lines.push(wl);
    }

    // --- Contextual: seasonal comment ---
    let season_line = npc_season_comment(npc_id, calendar.season);
    if let Some(sl) = season_line {
        lines.push(sl);
    }

    let tier = friendship_tier(hearts);

    // Get tier-specific dialogue if available
    if let Some(tier_lines) = npc_def.heart_dialogue.get(&tier) {
        if !tier_lines.is_empty() {
            // Pick pseudo-randomly based on total friendship to vary lines day-to-day
            let total_pts = relationships.friendship.get(npc_id).copied().unwrap_or(0);
            let idx = (total_pts as usize / 5) % tier_lines.len();
            lines.push(tier_lines[idx].clone());
            // Add a second line if available
            if tier_lines.len() > 1 {
                let idx2 = (idx + 1) % tier_lines.len();
                lines.push(tier_lines[idx2].clone());
            }
            return lines;
        }
    }

    // Fall back through lower tiers
    let lower_tiers = if tier > 0 {
        (0..tier).rev().collect::<Vec<_>>()
    } else {
        vec![]
    };

    for lower_tier in lower_tiers {
        if let Some(tier_lines) = npc_def.heart_dialogue.get(&lower_tier) {
            if !tier_lines.is_empty() {
                lines.push(tier_lines[0].clone());
                return lines;
            }
        }
    }

    // Ultimate fallback: default dialogue
    if !npc_def.default_dialogue.is_empty() {
        let total_pts = relationships.friendship.get(npc_id).copied().unwrap_or(0);
        let idx = (total_pts as usize) % npc_def.default_dialogue.len();
        lines.push(npc_def.default_dialogue[idx].clone());
    } else {
        lines.push(format!("Hello there! I'm {}.", npc_def.name));
    }

    lines
}

/// Return a weather-aware comment for the given NPC, or None if no comment is warranted.
fn npc_weather_comment(npc_id: &str, weather: Weather) -> Option<String> {
    let comment = match (npc_id, weather) {
        (_, Weather::Sunny) => return None, // sunny is default — no special comment

        ("margaret", Weather::Rainy)  => "Rainy days mean more baking. Everyone wants fresh bread when it's grey outside.",
        ("margaret", Weather::Stormy) => "Storm's rattling the windows! I hope my oven stays lit...",
        ("margaret", Weather::Snowy)  => "Snow reminds me of powdered sugar. Makes me want to bake something sweet.",

        ("marco", Weather::Rainy)     => "Rain days are soup days. Come by later — I'm making chowder.",
        ("marco", Weather::Stormy)    => "This storm! The herbs in my garden are getting flattened...",
        ("marco", Weather::Snowy)     => "Snow means hearty stews. My kitchen hasn't cooled down in days.",

        ("lily", Weather::Rainy)      => "The flowers LOVE the rain! Look how bright they are!",
        ("lily", Weather::Stormy)     => "Oh no, my poor petunias! I hope they survive the storm...",
        ("lily", Weather::Snowy)      => "Even in the snow, there's beauty. Have you seen the frost on the windows?",

        ("old_tom", Weather::Rainy)   => "Rain's good fishing weather, if you know where to look.",
        ("old_tom", Weather::Stormy)  => "No sense going out in this. Even the fish are hiding.",
        ("old_tom", Weather::Snowy)   => "Ice fishing season! Grab a stool and join me by the lake.",

        ("elena", Weather::Rainy)     => "Rainy days mean fewer customers. I use the time to sharpen the tools.",
        ("elena", Weather::Stormy)    => "Storm makes the forge feel cozy, honestly. Nothing like hot metal in cold air.",
        ("elena", Weather::Snowy)     => "Snow piles up on the anvil overnight. Adds character.",

        ("mira", Weather::Rainy)      => "Rain keeps the travelers off the roads. Bad for business.",
        ("mira", Weather::Stormy)     => "I've weathered worse storms than this in the Eastern deserts.",
        ("mira", Weather::Snowy)      => "Snow! It never snowed where I grew up. I still find it magical.",

        ("doc", Weather::Rainy)       => "Damp weather aggravates joint pain. Tell the older folks to stay warm.",
        ("doc", Weather::Stormy)      => "Storm season means injuries. Please be careful out there.",
        ("doc", Weather::Snowy)       => "Bundle up! I've already seen two cases of frostbite this week.",

        ("mayor_rex", Weather::Rainy) => "Rain is good for the crops! The town treasury thanks the clouds.",
        ("mayor_rex", Weather::Stormy)=> "I've filed a formal complaint with the weather. No response yet.",
        ("mayor_rex", Weather::Snowy) => "I do love how the town square looks under fresh snow.",

        ("sam", Weather::Rainy)       => "Rain on the roof... best percussion section in nature.",
        ("sam", Weather::Stormy)      => "This storm has ENERGY. I'm writing it into my next song.",
        ("sam", Weather::Snowy)       => "Snow muffles everything. The silence is almost musical.",

        ("nora", Weather::Rainy)      => "Good rain. The soil needed this.",
        ("nora", Weather::Stormy)     => "I've got tarps over the seedlings. Should hold.",
        ("nora", Weather::Snowy)      => "Winter rest is important. For the land and for us.",

        _ => return None,
    };
    Some(comment.to_string())
}

/// Return a season-aware comment for the given NPC, or None if no comment is warranted.
/// Selective — only where it fits the character, to avoid repetitive dialogue.
fn npc_season_comment(npc_id: &str, season: Season) -> Option<String> {
    let comment = match (npc_id, season) {
        ("lily", Season::Spring)       => "Spring! SPRING! Everything is blooming and I can't stop smiling!",
        ("nora", Season::Spring)       => "Planting season. Best time of the year if you ask me.",
        ("margaret", Season::Spring)   => "Egg Festival's coming up! I've been working on a new recipe.",

        ("old_tom", Season::Summer)    => "Summer means the big fish come in. You should try the deep water.",
        ("marco", Season::Summer)      => "Summer produce is incredible. Tomatoes, peppers, corn...",
        ("mayor_rex", Season::Summer)  => "The town festival is the highlight of my year. Planning is underway!",

        ("nora", Season::Fall)         => "Harvest time. Nothing beats the smell of fresh-cut wheat.",
        ("mira", Season::Fall)         => "The fall colours draw me back here every year. Worth the trip.",
        ("sam", Season::Fall)          => "Something about autumn puts me in a songwriting mood.",

        ("old_tom", Season::Winter)    => "Mending nets by the fire. A fisherman's winter ritual.",
        ("doc", Season::Winter)        => "Cold season keeps me busy. Stock up on hot soup and rest.",
        ("elena", Season::Winter)      => "Quiet at the forge. Good time to practice new techniques.",

        _ => return None,
    };
    Some(comment.to_string())
}

/// Get the dialogue tier key for a given heart count.
/// Tiers: 0 (0-2 hearts), 3 (3-5 hearts), 6 (6-8 hearts), 9 (9-10 hearts)
fn friendship_tier(hearts: u8) -> u8 {
    match hearts {
        0..=2 => 0,
        3..=5 => 3,
        6..=8 => 6,
        _ => 9,
    }
}

/// Build gift-response dialogue lines based on the NPC and gift preference.
pub fn build_gift_response_lines(
    npc_id: &str,
    _npc_name: &str,
    preference: GiftPreference,
    item_name: &str,
    is_birthday: bool,
) -> Vec<String> {
    let birthday_prefix = if is_birthday {
        format!("Oh! Is it really for me? And on my birthday! ")
    } else {
        String::new()
    };

    match preference {
        GiftPreference::Loved => {
            let response = loved_response(npc_id, item_name);
            vec![format!("{}{}", birthday_prefix, response)]
        }
        GiftPreference::Liked => {
            vec![
                format!("{}{} — oh, I really like this! Thank you so much.", birthday_prefix, item_name),
            ]
        }
        GiftPreference::Neutral => {
            vec![
                format!("{}Oh, a gift! Thank you for thinking of me. That's very kind.", birthday_prefix),
            ]
        }
        GiftPreference::Disliked => {
            vec![
                format!("{}I appreciate the thought, but... a {}? Hmm. Thank you anyway.", birthday_prefix, item_name),
            ]
        }
        GiftPreference::Hated => {
            let response = hated_response(npc_id, item_name);
            vec![response]
        }
    }
}

fn loved_response(npc_id: &str, item_name: &str) -> String {
    match npc_id {
        "margaret"  => format!("A {}! Oh, this will be perfect for my next batch of pastries! Thank you, dear.", item_name),
        "marco"     => format!("A {}! Incredible quality. I can already taste the dish I'll make with this.", item_name),
        "lily"      => format!("A {}!! THIS IS THE BEST THING EVER!! Can I keep it? Can I? THANK YOU!!", item_name),
        "old_tom"   => format!("A {}? Well now... haven't seen one this fine in years. Much obliged.", item_name),
        "elena"     => format!("Now THIS is a proper gift. A {}. I can feel the quality just holding it. Thank you.", item_name),
        "mira"      => format!("A {}! You have a trader's eye for value. I'm genuinely impressed.", item_name),
        "doc"       => format!("A {}? This is... very thoughtful. I appreciate it more than you know.", item_name),
        "mayor_rex" => format!("A {} for the mayor! I shall display this proudly in my office!", item_name),
        "sam"       => format!("A {}! No way! This is exactly what I needed for inspiration. You're the best.", item_name),
        "nora"      => format!("A {}. Good quality. You've got a farmer's instinct for the real stuff.", item_name),
        _           => format!("A {}! Thank you, I love it!", item_name),
    }
}

fn hated_response(npc_id: &str, item_name: &str) -> String {
    match npc_id {
        "margaret"  => format!("A {}? Oh dear... I don't think that belongs in any recipe I know.", item_name),
        "marco"     => format!("A {}? I wouldn't serve this to my worst critic. No offense.", item_name),
        "lily"      => format!("A {}?! Ew ew ew! Get it away from my flowers!", item_name),
        "old_tom"   => format!("A {}? Boy, I've pulled stranger things out of the water. ...I'll pass.", item_name),
        "elena"     => format!("A {}? I'd sooner melt this down than look at it. Not my style.", item_name),
        "mira"      => format!("A {}? In my travels, I've learned to politely decline. Consider this me declining.", item_name),
        "doc"       => format!("A {}? I don't think this is... medically advisable. For anyone.", item_name),
        "mayor_rex" => format!("A {}? I appreciate the gesture, but the mayor's office has standards.", item_name),
        "sam"       => format!("A {}? I've seen things like this at the bottom of a dumpster. Left 'em there too.", item_name),
        "nora"      => format!("A {}? Hmph. Been farming all my life and never needed one of those.", item_name),
        _           => format!("A {}? ...Thanks, I guess.", item_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_friendship_tier_low_hearts() {
        assert_eq!(friendship_tier(0), 0);
        assert_eq!(friendship_tier(1), 0);
        assert_eq!(friendship_tier(2), 0);
    }

    #[test]
    fn test_friendship_tier_mid_hearts() {
        assert_eq!(friendship_tier(3), 3);
        assert_eq!(friendship_tier(5), 3);
    }

    #[test]
    fn test_friendship_tier_high_hearts() {
        assert_eq!(friendship_tier(6), 6);
        assert_eq!(friendship_tier(8), 6);
    }

    #[test]
    fn test_friendship_tier_max_hearts() {
        assert_eq!(friendship_tier(9), 9);
        assert_eq!(friendship_tier(10), 9);
    }

    #[test]
    fn test_gift_response_loved_contains_item_name() {
        let lines = build_gift_response_lines(
            "elena", "Elena", GiftPreference::Loved, "Sunflower", false,
        );
        assert!(!lines.is_empty());
        assert!(lines[0].contains("Sunflower"),
            "Loved response should mention item name");
    }

    #[test]
    fn test_gift_response_hated_contains_item_name() {
        let lines = build_gift_response_lines(
            "marco", "Marco", GiftPreference::Hated, "Trash", false,
        );
        assert!(!lines.is_empty());
        assert!(lines[0].contains("Trash"),
            "Hated response should mention item name");
    }

    #[test]
    fn test_gift_response_birthday_prefix() {
        let lines = build_gift_response_lines(
            "elena", "Elena", GiftPreference::Neutral, "Stone", true,
        );
        assert!(!lines.is_empty());
        assert!(lines[0].contains("birthday"),
            "Birthday gift should mention birthday");
    }

    #[test]
    fn test_npc_weather_comment_sunny_returns_none() {
        // Sunny weather should produce no special comment
        assert!(npc_weather_comment("elena", Weather::Sunny).is_none());
        assert!(npc_weather_comment("marco", Weather::Sunny).is_none());
    }

    #[test]
    fn test_npc_weather_comment_rainy_returns_some() {
        assert!(npc_weather_comment("elena", Weather::Rainy).is_some());
        assert!(npc_weather_comment("old_tom", Weather::Rainy).is_some());
    }

    #[test]
    fn test_npc_weather_comment_snowy_returns_some() {
        assert!(npc_weather_comment("lily", Weather::Snowy).is_some());
    }
}

//! Dialogue system: handle player interaction with NPCs, build dialogue lines
//! based on friendship level, and emit DialogueStartEvent.

use bevy::prelude::*;
use crate::shared::*;
use std::sync::atomic::{AtomicU8, Ordering};

static SEASON_COMMENT_DAY: AtomicU8 = AtomicU8::new(1);

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
    SEASON_COMMENT_DAY.store(calendar.day, Ordering::Relaxed);
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

        ("nora", Weather::Rainy)      => "Steady rain like this fattens the crops better than any fancy fertilizer.",
        ("nora", Weather::Stormy)     => "Storm's coming in hard. My old bones felt it before the clouds did.",
        ("nora", Weather::Snowy)      => "Snow's our warning bell. Best finish winter prep before the drifts get deep.",

        _ => return None,
    };
    Some(comment.to_string())
}

/// Return a season-aware comment for the given NPC, or None if no comment is warranted.
/// Selective — only where it fits the character, to avoid repetitive dialogue.
fn npc_season_comment(npc_id: &str, season: Season) -> Option<String> {
    let variants: &[&str] = match (npc_id, season) {
        ("lily", Season::Spring) => &[
            "Spring! SPRING! Everything is blooming and I can't stop smiling!",
            "My greenhouse smells like fresh lilac and wet soil. Best perfume in town.",
            "Tulips, daisies, peonies... spring is my busiest and happiest season.",
        ],
        ("elena", Season::Spring) => &[
            "Spring orders are piling up at the forge. Hoes, shears, and plow tips all day.",
            "Every farmer wants sharpened tools in spring, so the anvil barely cools.",
            "New season, new metalwork. I tune plowshares like musicians tune instruments.",
        ],
        ("doc", Season::Spring) => &[
            "Pollen season again. Keep handkerchiefs nearby and don't ignore those allergies.",
            "Spring colds spread fast when nights stay cool. Rest and fluids, not stubbornness.",
            "If your eyes itch and you sneeze at flowers, come by for a remedy.",
        ],
        ("sam", Season::Spring) => &[
            "Birdsong, thawing creeks, fresh air... spring hands me melodies for free.",
            "I tuned my guitar on the porch this morning; even the robins kept tempo.",
            "Spring gigs feel lighter, like every chord has sunlight in it.",
        ],
        ("nora", Season::Spring) => &[
            "Planting season. Best time of the year if you ask me.",
            "Spring soil tells the truth: if it crumbles right, your crops will thank you.",
            "First furrow of spring always feels like opening a new book.",
        ],
        ("margaret", Season::Spring) => &[
            "Egg Festival's coming up! I've been working on a new recipe.",
            "Spring mornings are perfect for proofing bread by the sunny window.",
            "Fresh eggs and new berries mean the bakery smells like celebration.",
        ],

        ("lily", Season::Summer) => &[
            "Summer flowers are dramatic in the best way. Big petals, bright colors, zero shyness!",
            "My garden is bursting so hard I need three extra vases by noon.",
            "Heat makes the roses bold. They practically pose for compliments.",
        ],
        ("elena", Season::Summer) => &[
            "The forge is brutal in summer. You learn fast, work smart, and drink water.",
            "Steel moves differently in this heat. Great for shaping, rough on lungs.",
            "I keep two things close in summer: tongs and a full water jug.",
        ],
        ("nora", Season::Summer) => &[
            "Summer crops don't wait for anyone. Sunrise to sunset, there's always field work.",
            "If you weed late in summer, the field punishes you by morning.",
            "Corn, beans, and sweat. That's summer farming in three words.",
        ],
        ("old_tom", Season::Summer) => &[
            "Summer means the big fish come in. You should try the deep water.",
            "Warm mornings and calm water put trout in a biting mood.",
            "On hot days I fish before dawn; by noon the river gets lazy.",
        ],
        ("marco", Season::Summer) => &[
            "Summer produce is incredible. Tomatoes, peppers, corn...",
            "I can build a whole menu from summer herbs and one good olive oil.",
            "When zucchini and basil are in season, my kitchen sings.",
        ],
        ("mayor_rex", Season::Summer) => &[
            "The town festival is the highlight of my year. Planning is underway!",
            "Summer brings visitors, so we keep the square tidy and the permits moving.",
            "Council meetings run long this season, but it keeps town events smooth.",
        ],
        ("sam", Season::Summer) => &[
            "Summer crowds are loud and friendly. Perfect audience for a guitar set.",
            "I play by the fountain at dusk; the reverb between buildings is amazing.",
            "Long summer evenings mean I can squeeze in one more song before dark.",
        ],

        ("margaret", Season::Fall) => &[
            "Fall means pumpkin loaves and cinnamon rolls. My ovens hardly get a break.",
            "Cool air helps my sourdough crust set just right.",
            "Apple season keeps me elbow-deep in pie dough from dawn to closing.",
        ],
        ("old_tom", Season::Fall) => &[
            "Cooler water, hungry fish. Fall's one of the best times to cast a line.",
            "Autumn wind on the lake means steady bites if you stay patient.",
            "Fish school near the reeds in fall. Cast quiet and let the line drift.",
        ],
        ("doc", Season::Fall) => &[
            "I spend fall preparing medicine stocks now, before winter illnesses arrive.",
            "Dry leaves, cold mornings, and coughs. Fall is prevention season at the clinic.",
            "I brew extra herbal tonics in autumn so we're ready for first frost.",
        ],
        ("nora", Season::Fall) => &[
            "Harvest time. Nothing beats the smell of fresh-cut wheat.",
            "A good fall harvest starts with clean rows and sharp sickles.",
            "If your barn is full by first frost, you did the year right.",
        ],
        ("mira", Season::Fall) => &[
            "The fall colours draw me back here every year. Worth the trip.",
            "Autumn caravans move slower, but rare spices fetch better prices.",
            "I trade wool and dried fruit all fall; travelers love practical luxuries.",
        ],
        ("sam", Season::Fall) => &[
            "Something about autumn puts me in a songwriting mood.",
            "Crisp air and rustling leaves make a great rhythm section.",
            "I write my best acoustic sets in fall. The town listens closer somehow.",
        ],

        ("margaret", Season::Winter) => &[
            "Winter calls for comfort food. Stews, pies, and warm bread from dawn to dusk.",
            "In winter I bake extra rye loaves so no one goes home hungry.",
            "Nothing beats pulling a buttered roll from the oven while snow falls outside.",
        ],
        ("lily", Season::Winter) => &[
            "The greenhouse keeps my winter blooms alive. Tiny summer in a glass house!",
            "I tuck my bulbs in warm beds so spring color arrives right on time.",
            "Winter teaches flowers patience. A little light, a little water, lots of hope.",
        ],
        ("mayor_rex", Season::Winter) => &[
            "Winter festival planning is in full swing. Logistics now, celebration later!",
            "Snow means budget meetings, road crews, and hot cider for volunteers.",
            "Keeping the town running through winter is governance at its finest.",
        ],
        ("old_tom", Season::Winter) => &[
            "Mending nets by the fire. A fisherman's winter ritual.",
            "Cold water slows fish down, but the patient angler still eats well.",
            "I watch the weather close in winter; one bad wind can cost a boat.",
        ],
        ("doc", Season::Winter) => &[
            "Cold season keeps me busy. Stock up on hot soup and rest.",
            "Frost and flu arrive together. Gloves, sleep, and warm tea are good medicine.",
            "If your cough lingers past a week, stop by. Don't wait it out.",
        ],
        ("elena", Season::Winter) => &[
            "Quiet at the forge. Good time to practice new techniques.",
            "Winter is for precision work: small tools, fine edges, clean welds.",
            "Metal turns brittle in deep cold, so I temper slower and check every strike.",
        ],
        ("sam", Season::Winter) => &[
            "Winter hush makes every guitar note ring longer in the night air.",
            "I play near the tavern hearth in winter; warm room, warm crowd, better songs.",
            "Snowy nights are made for soft melodies and slow chord changes.",
        ],
        ("nora", Season::Winter) => &[
            "Winter's when a farmer plans more than plants. Notes now, better yields later.",
            "I turn compost through the cold so spring soil starts rich.",
            "Cold mornings teach patience. Fields rest, and farmers sharpen their sense.",
        ],

        ("marco", Season::Spring) => &[
            "Spring herbs are tender and bright. Even simple soups taste alive again.",
            "I wait all year for spring onions and young greens.",
            "First spring harvest means lighter sauces and sharper flavors.",
        ],
        ("marco", Season::Fall) => &[
            "Fall is stockpot season. Roots, squash, and slow-cooked depth.",
            "I roast pumpkins and garlic until the whole inn smells like dinner.",
            "Autumn ingredients are humble, but they make the richest plates.",
        ],
        ("marco", Season::Winter) => &[
            "In winter I lean on braises and stews. Low heat, long time, big comfort.",
            "Citrus from distant traders keeps winter dishes from feeling heavy.",
            "A good winter kitchen is equal parts fire, broth, and patience.",
        ],
        ("mira", Season::Spring) => &[
            "Spring roads open again, and caravans start bringing fresh goods.",
            "I barter seeds, silk, and stories once the mountain passes thaw.",
            "New season, new routes. Trade thrives when mud finally dries.",
        ],
        ("mira", Season::Summer) => &[
            "Summer markets are loud, hot, and very profitable.",
            "Travelers pay well for chilled drinks and bright fabrics this time of year.",
            "I move spices at dawn in summer before the heat spoils tempers.",
        ],
        ("mira", Season::Winter) => &[
            "Winter caravans are risky, but rare wares are worth the snow.",
            "I plan routes by storm reports now. Trade is math in bad weather.",
            "Exotic teas sell fastest in winter; everyone wants warmth in a cup.",
        ],
        ("mayor_rex", Season::Spring) => &[
            "Spring means permits, planting support, and repairing winter damage around town.",
            "I love spring council sessions. So many new projects to approve.",
            "When the square fills with planters again, I know governance is working.",
        ],
        ("mayor_rex", Season::Fall) => &[
            "Fall is budget season. We fund winter prep before the first hard frost.",
            "Harvest reports keep the council honest and the granaries ready.",
            "Town governance in autumn is ledgers by day, lantern inspections by dusk.",
        ],

        _ => return None,
    };

    let calendar_day = SEASON_COMMENT_DAY.load(Ordering::Relaxed);
    let idx = (calendar_day as usize) % variants.len();
    Some(variants[idx].to_string())
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

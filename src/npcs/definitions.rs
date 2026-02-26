//! NPC definitions: all 10 NPCs with their personalities, gift preferences,
//! dialogue lines at every friendship tier, and birthday information.

use crate::shared::*;
use std::collections::HashMap;

/// Build all NPC definitions and populate the NpcRegistry.
pub fn build_npc_registry() -> NpcRegistry {
    let mut registry = NpcRegistry::default();

    // --- Mayor Thomas ---
    {
        let id = "mayor_thomas".to_string();
        let mut prefs: HashMap<ItemId, GiftPreference> = HashMap::new();
        // Loved: gems
        for gem in &["amethyst", "emerald", "ruby", "diamond", "topaz", "aquamarine"] {
            prefs.insert(gem.to_string(), GiftPreference::Loved);
        }
        // Liked: minerals, geodes
        for item in &["copper_bar", "iron_bar", "gold_bar", "quartz", "earth_crystal"] {
            prefs.insert(item.to_string(), GiftPreference::Liked);
        }
        // Disliked: raw ores
        for item in &["copper_ore", "iron_ore", "coal"] {
            prefs.insert(item.to_string(), GiftPreference::Disliked);
        }
        // Hated: trash
        prefs.insert("seaweed".to_string(), GiftPreference::Hated);
        prefs.insert("broken_glasses".to_string(), GiftPreference::Hated);

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(3, vec![
            "The town has been much livelier since you arrived! Keep up the fine work on that farm.".to_string(),
            "I've been mayor for twenty years. Not everyone appreciates the job, but someone has to hold things together.".to_string(),
        ]);
        heart_dialogue.insert(6, vec![
            "Between us? I sometimes wonder if I should have been an explorer rather than a mayor. The mountains always called to me.".to_string(),
            "My grandfather found the first gem in these hills. He said the mountains were alive. I used to think it was a tall tale...".to_string(),
        ]);
        heart_dialogue.insert(9, vec![
            "You remind me of my younger self — full of hope. This valley needs that energy. Promise me you'll never lose it?".to_string(),
            "I'm writing the town's history. I'd like to dedicate a chapter to you, if that's alright. You've truly changed Hearthfield.".to_string(),
        ]);

        registry.npcs.insert(id.clone(), NpcDef {
            id: id.clone(),
            name: "Mayor Thomas".to_string(),
            birthday_season: Season::Spring,
            birthday_day: 14,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Good morning! Lovely day to be out and about.".to_string(),
                "Keep an eye on that farm — the valley's counting on you!".to_string(),
                "Ahh, the fresh air of Hearthfield. Nothing like it anywhere.".to_string(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 0,
            portrait_index: 0,
        });

        registry.schedules.insert(id.clone(), build_mayor_thomas_schedule());
    }

    // --- Elena (marriageable) ---
    {
        let id = "elena".to_string();
        let mut prefs: HashMap<ItemId, GiftPreference> = HashMap::new();
        // Loved: flowers, flower crops
        for item in &["tulip", "summer_spangle", "sunflower", "fairy_rose", "sweet_pea"] {
            prefs.insert(item.to_string(), GiftPreference::Loved);
        }
        // Liked: cooked food, artisan goods
        for item in &["strawberry_jam", "blueberry_jam", "honey", "coffee", "fruit_salad"] {
            prefs.insert(item.to_string(), GiftPreference::Liked);
        }
        // Disliked: ores, rocks
        for item in &["copper_ore", "iron_ore", "stone", "coal"] {
            prefs.insert(item.to_string(), GiftPreference::Disliked);
        }
        // Hated: fish (she's squeamish)
        for item in &["catfish", "pike", "trash_fish", "seaweed"] {
            prefs.insert(item.to_string(), GiftPreference::Hated);
        }

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(3, vec![
            "The shop keeps me busy, but I love it. Every customer has a story, you know?".to_string(),
            "I've been thinking about adding a flower display up front. What do you think?".to_string(),
        ]);
        heart_dialogue.insert(6, vec![
            "Sometimes after closing, I press flowers between the pages of old books. It sounds silly...".to_string(),
            "My mother taught me every flower has a meaning. Sunflowers mean adoration. I wonder who planted the ones by the road...".to_string(),
        ]);
        heart_dialogue.insert(9, vec![
            "I used to dream of traveling to see the great flower festivals. Now... I think my favorite blooms are right here in Hearthfield.".to_string(),
            "You know, I look forward to seeing you come into the shop every morning. More than I probably should admit.".to_string(),
        ]);

        registry.npcs.insert(id.clone(), NpcDef {
            id: id.clone(),
            name: "Elena".to_string(),
            birthday_season: Season::Spring,
            birthday_day: 8,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Welcome to the general store! Let me know if you need anything.".to_string(),
                "We just got in a new shipment of seeds. Spring is such an exciting time!".to_string(),
                "Take care out there! The weather can turn quickly.".to_string(),
            ],
            heart_dialogue,
            is_marriageable: true,
            sprite_index: 1,
            portrait_index: 1,
        });

        registry.schedules.insert(id.clone(), build_elena_schedule());
    }

    // --- Marcus (marriageable) ---
    {
        let id = "marcus".to_string();
        let mut prefs: HashMap<ItemId, GiftPreference> = HashMap::new();
        // Loved: minerals, refined bars
        for item in &["iron_bar", "gold_bar", "iridium_bar", "diamond", "ruby"] {
            prefs.insert(item.to_string(), GiftPreference::Loved);
        }
        // Liked: ores, geodes, stone
        for item in &["copper_ore", "iron_ore", "gold_ore", "quartz", "stone"] {
            prefs.insert(item.to_string(), GiftPreference::Liked);
        }
        // Disliked: sweet foods (not his taste)
        for item in &["cake", "cookie", "honey", "strawberry_jam"] {
            prefs.insert(item.to_string(), GiftPreference::Disliked);
        }
        // Hated: flowers (embarrasses him)
        for item in &["tulip", "summer_spangle", "fairy_rose"] {
            prefs.insert(item.to_string(), GiftPreference::Hated);
        }

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(3, vec![
            "Blacksmithing isn't glamorous, but when you forge something useful — something that lasts — that's real satisfaction.".to_string(),
            "Old Hans is teaching me the traditional tempering methods. Says modern shortcuts ruin the metal's soul. I'm starting to believe him.".to_string(),
        ]);
        heart_dialogue.insert(6, vec![
            "I've been working on a sword design. Just for the craft of it — nobody needs a sword around here. But the challenge keeps me sharp.".to_string(),
            "Mining's dangerous, but the stones fascinate me. Every ore vein is like a chapter in the mountain's history.".to_string(),
        ]);
        heart_dialogue.insert(9, vec![
            "You know what they say about a blacksmith's hands? They carry the memory of everything they've made. I hope mine remember you.".to_string(),
            "I've never told anyone this... but I want to forge something truly legendary someday. A blade or tool worthy of a hero. Maybe for you?".to_string(),
        ]);

        registry.npcs.insert(id.clone(), NpcDef {
            id: id.clone(),
            name: "Marcus".to_string(),
            birthday_season: Season::Summer,
            birthday_day: 3,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Need your tools upgraded? I can work with copper, iron, and gold.".to_string(),
                "The forge needs constant attention. Give me a moment.".to_string(),
                "Quality materials make quality tools. Bring me the right ores and I'll make them shine.".to_string(),
            ],
            heart_dialogue,
            is_marriageable: true,
            sprite_index: 2,
            portrait_index: 2,
        });

        registry.schedules.insert(id.clone(), build_marcus_schedule());
    }

    // --- Dr. Iris ---
    {
        let id = "dr_iris".to_string();
        let mut prefs: HashMap<ItemId, GiftPreference> = HashMap::new();
        // Loved: herbs, medicine-related items
        for item in &["ginger", "leek", "radish", "coconut", "fiddlehead_fern"] {
            prefs.insert(item.to_string(), GiftPreference::Loved);
        }
        // Liked: cooked food with health benefits
        for item in &["pumpkin_soup", "fried_egg", "vegetable_medley", "maki_roll"] {
            prefs.insert(item.to_string(), GiftPreference::Liked);
        }
        // Neutral: most basic crops
        for item in &["turnip", "potato", "cauliflower"] {
            prefs.insert(item.to_string(), GiftPreference::Neutral);
        }
        // Disliked: gems (impractical)
        for item in &["amethyst", "topaz", "aquamarine"] {
            prefs.insert(item.to_string(), GiftPreference::Disliked);
        }
        // Hated: alcohol/sugary things
        prefs.insert("wine".to_string(), GiftPreference::Hated);
        prefs.insert("beer".to_string(), GiftPreference::Hated);

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(3, vec![
            "Most of my remedies come from plants and minerals found right here in the valley. Nature provides if you know where to look.".to_string(),
            "The old texts describe diseases we no longer see. I wonder sometimes if the valley itself has healing properties.".to_string(),
        ]);
        heart_dialogue.insert(6, vec![
            "I moved here to study the local flora. Five years later... I haven't wanted to leave.".to_string(),
            "There's an old herbalist tradition in this region. I've been trying to document it before the knowledge is lost.".to_string(),
        ]);
        heart_dialogue.insert(9, vec![
            "You've shown me that health isn't just physical — it's community, purpose, joy. Thank you for being part of this village.".to_string(),
            "I've been compiling a book of regional remedies. I'd like your help collecting some rarer specimens, if you're willing.".to_string(),
        ]);

        registry.npcs.insert(id.clone(), NpcDef {
            id: id.clone(),
            name: "Dr. Iris".to_string(),
            birthday_season: Season::Fall,
            birthday_day: 19,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Good day. Are you feeling well? Your color looks good.".to_string(),
                "Remember to eat properly and sleep enough. Farming is hard on the body.".to_string(),
                "The clinic is always open if you need attention.".to_string(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 3,
            portrait_index: 3,
        });

        registry.schedules.insert(id.clone(), build_dr_iris_schedule());
    }

    // --- Old Pete ---
    {
        let id = "old_pete".to_string();
        let mut prefs: HashMap<ItemId, GiftPreference> = HashMap::new();
        // Loved: legendary and rare fish
        for item in &["legend_fish", "glacierfish", "angler", "mutant_carp", "crimsonfish"] {
            prefs.insert(item.to_string(), GiftPreference::Loved);
        }
        // Liked: common fish
        for item in &["catfish", "tuna", "sardine", "pufferfish", "red_snapper", "pike"] {
            prefs.insert(item.to_string(), GiftPreference::Liked);
        }
        // Disliked: gems (useless to him)
        for item in &["amethyst", "emerald", "ruby", "topaz"] {
            prefs.insert(item.to_string(), GiftPreference::Disliked);
        }
        // Hated: flowers
        for item in &["tulip", "summer_spangle", "sunflower"] {
            prefs.insert(item.to_string(), GiftPreference::Hated);
        }

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(3, vec![
            "Been fishing these waters for forty years. The fish know me by name now, I reckon.".to_string(),
            "My daddy taught me to fish right here on this beach. Some things don't change, and that suits me fine.".to_string(),
        ]);
        heart_dialogue.insert(6, vec![
            "I caught the Legend once, you know. Forty years ago. Released it because it was just too beautiful to keep. Don't tell nobody I said that.".to_string(),
            "The ocean speaks if you're quiet enough. I've been listening my whole life. Still learning.".to_string(),
        ]);
        heart_dialogue.insert(9, vec![
            "You remind me of myself at your age. Fire in the eyes, dirt on the hands. You're going to do great things here.".to_string(),
            "I'd like to teach you my secret technique before I'm gone. You've earned it. Meet me at dawn by the old pier.".to_string(),
        ]);

        registry.npcs.insert(id.clone(), NpcDef {
            id: id.clone(),
            name: "Old Pete".to_string(),
            birthday_season: Season::Winter,
            birthday_day: 7,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Not biting today... or maybe I'm just not patient enough. Ha!".to_string(),
                "Best fishing spot in the valley right here. Don't tell anyone else though.".to_string(),
                "The weather matters more than the bait. Remember that.".to_string(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 4,
            portrait_index: 4,
        });

        registry.schedules.insert(id.clone(), build_old_pete_schedule());
    }

    // --- Chef Rosa ---
    {
        let id = "chef_rosa".to_string();
        let mut prefs: HashMap<ItemId, GiftPreference> = HashMap::new();
        // Loved: high-quality cooked food
        for item in &["complete_breakfast", "lobster_bisque", "seafood_pudding", "lucky_lunch", "pumpkin_pie"] {
            prefs.insert(item.to_string(), GiftPreference::Loved);
        }
        // Liked: quality crops and ingredients
        for item in &["pumpkin", "melon", "yam", "eggplant", "corn", "tomato", "blueberry"] {
            prefs.insert(item.to_string(), GiftPreference::Liked);
        }
        // Disliked: raw fish (she prefers them cooked)
        for item in &["sardine", "catfish", "trash_fish"] {
            prefs.insert(item.to_string(), GiftPreference::Disliked);
        }
        // Hated: ores and rocks (what would she do with those?)
        for item in &["copper_ore", "iron_ore", "coal", "stone"] {
            prefs.insert(item.to_string(), GiftPreference::Hated);
        }

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(3, vec![
            "Fresh farm produce makes all the difference in cooking. I can taste the effort that goes into every vegetable.".to_string(),
            "My restaurant may be small, but I pour everything into it. Food made with love nourishes the soul.".to_string(),
        ]);
        heart_dialogue.insert(6, vec![
            "I trained under a famous chef in the city, but left when I realized she cooked for acclaim, not for people. Big difference.".to_string(),
            "My grandmother's recipe book is my most treasured possession. Some of those recipes go back five generations.".to_string(),
        ]);
        heart_dialogue.insert(9, vec![
            "I've been creating a new dish inspired by everything this valley produces. I'd be honored if you were the first to taste it.".to_string(),
            "You've brought such wonderful ingredients to my kitchen. Some of my best creations exist because of you. Thank you.".to_string(),
        ]);

        registry.npcs.insert(id.clone(), NpcDef {
            id: id.clone(),
            name: "Chef Rosa".to_string(),
            birthday_season: Season::Summer,
            birthday_day: 22,
            gift_preferences: prefs,
            default_dialogue: vec![
                "The kitchen never sleeps! But I wouldn't have it any other way.".to_string(),
                "Stop by for a meal — I've been experimenting with a new recipe.".to_string(),
                "The secret to great food? Start with the freshest ingredients. Yours are wonderful.".to_string(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 5,
            portrait_index: 5,
        });

        registry.schedules.insert(id.clone(), build_chef_rosa_schedule());
    }

    // --- Miner Gil ---
    {
        let id = "miner_gil".to_string();
        let mut prefs: HashMap<ItemId, GiftPreference> = HashMap::new();
        // Loved: ores and bars
        for item in &["gold_ore", "iron_ore", "copper_ore", "iridium_ore", "gold_bar"] {
            prefs.insert(item.to_string(), GiftPreference::Loved);
        }
        // Liked: gems found in mines
        for item in &["diamond", "ruby", "emerald", "amethyst", "aquamarine"] {
            prefs.insert(item.to_string(), GiftPreference::Liked);
        }
        // Disliked: fancy food (not his style)
        for item in &["pumpkin_pie", "lobster_bisque", "cake"] {
            prefs.insert(item.to_string(), GiftPreference::Disliked);
        }
        // Hated: flowers
        for item in &["tulip", "sunflower", "fairy_rose", "summer_spangle"] {
            prefs.insert(item.to_string(), GiftPreference::Hated);
        }

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(3, vec![
            "Forty floors down and the rock still surprises me sometimes. You never fully know the mountain.".to_string(),
            "My crew was the best in the region. Retired now, but the rock still calls to me some mornings.".to_string(),
        ]);
        heart_dialogue.insert(6, vec![
            "There's a vein of something deep down — floor 18 or so — that I've never seen in forty years of mining. Be careful.".to_string(),
            "The old-timers said there was a secret cavern at the bottom. Never found it myself. Maybe you will.".to_string(),
        ]);
        heart_dialogue.insert(9, vec![
            "You've got a miner's instinct — knowing where to swing and when to hold back. That can't be taught. You were born for this.".to_string(),
            "Take my old helmet. It's seen more floors than most pickaxes. It'll keep you safe where I can't follow anymore.".to_string(),
        ]);

        registry.npcs.insert(id.clone(), NpcDef {
            id: id.clone(),
            name: "Miner Gil".to_string(),
            birthday_season: Season::Winter,
            birthday_day: 15,
            gift_preferences: prefs,
            default_dialogue: vec![
                "The mine's open if you're brave enough. Watch out for the critters on the lower floors.".to_string(),
                "Copper's common, iron's useful, but gold — that's where the real fortune lies.".to_string(),
                "Always bring enough torches. Never trust the dark down there.".to_string(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 6,
            portrait_index: 6,
        });

        registry.schedules.insert(id.clone(), build_miner_gil_schedule());
    }

    // --- Librarian Faye ---
    {
        let id = "librarian_faye".to_string();
        let mut prefs: HashMap<ItemId, GiftPreference> = HashMap::new();
        // Loved: artifacts and ancient things
        for item in &["ancient_artifact", "ancient_sword", "prehistoric_tool", "elvish_jewelry", "dwarf_scroll"] {
            prefs.insert(item.to_string(), GiftPreference::Loved);
        }
        // Liked: gems (beautiful and historical)
        for item in &["amethyst", "emerald", "aquamarine", "topaz", "quartz"] {
            prefs.insert(item.to_string(), GiftPreference::Liked);
        }
        // Disliked: fish (smelly)
        for item in &["sardine", "catfish", "pike"] {
            prefs.insert(item.to_string(), GiftPreference::Disliked);
        }
        // Hated: raw ores (dusty and messy)
        for item in &["copper_ore", "iron_ore", "coal", "stone"] {
            prefs.insert(item.to_string(), GiftPreference::Hated);
        }

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(3, vec![
            "The library holds records going back two centuries. Every book is a window into another world — or another time.".to_string(),
            "I'm cataloguing the town's historical artifacts. It's slow work, but each piece tells a story.".to_string(),
        ]);
        heart_dialogue.insert(6, vec![
            "I found references to a civilization that predates our town by a thousand years. They left artifacts scattered throughout the valley.".to_string(),
            "Books are patient companions. They wait for you no matter how long you're away, and they never judge.".to_string(),
        ]);
        heart_dialogue.insert(9, vec![
            "I've been writing a book — a history of Hearthfield and the people who made it. Your chapter is my favorite to write.".to_string(),
            "You've brought me artifacts I've only read about in other books. I feel like we're uncovering this valley's true history together.".to_string(),
        ]);

        registry.npcs.insert(id.clone(), NpcDef {
            id: id.clone(),
            name: "Librarian Faye".to_string(),
            birthday_season: Season::Fall,
            birthday_day: 4,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Shh — others are reading. But hello!".to_string(),
                "The library has histories, farming guides, fishing journals, and much more. Help yourself.".to_string(),
                "A good book is worth more than gold. Usually.".to_string(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 7,
            portrait_index: 7,
        });

        registry.schedules.insert(id.clone(), build_librarian_faye_schedule());
    }

    // --- Farmer Dale ---
    {
        let id = "farmer_dale".to_string();
        let mut prefs: HashMap<ItemId, GiftPreference> = HashMap::new();
        // Loved: rare and multi-season crops
        for item in &["ancient_fruit", "starfruit", "sweet_gem_berry", "melon", "pumpkin"] {
            prefs.insert(item.to_string(), GiftPreference::Loved);
        }
        // Liked: standard crops and seeds
        for item in &["potato", "cauliflower", "blueberry", "cranberry", "yam", "eggplant"] {
            prefs.insert(item.to_string(), GiftPreference::Liked);
        }
        // Disliked: gems (not useful on a farm)
        for item in &["amethyst", "topaz", "aquamarine"] {
            prefs.insert(item.to_string(), GiftPreference::Disliked);
        }
        // Hated: anything that damages crops (scarecrow means nothing to him as a gift)
        prefs.insert("broken_glasses".to_string(), GiftPreference::Hated);
        prefs.insert("seaweed".to_string(), GiftPreference::Hated);

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(3, vec![
            "My family's farmed this land for generations. You've got good instincts — I can see it in how you tend your crops.".to_string(),
            "The soil here is rich, if you treat it right. Don't rush it. Farming teaches patience above all else.".to_string(),
        ]);
        heart_dialogue.insert(6, vec![
            "My late wife loved ancient fruit. She'd tend those plants for a full season just to see the blossoms. I still grow them, though I don't ship them.".to_string(),
            "When you listen to the land — really listen — it tells you what it needs. You're starting to hear it. I can tell.".to_string(),
        ]);
        heart_dialogue.insert(9, vec![
            "You've become a real farmer, not just someone playing at it. I'm proud to call you a neighbor.".to_string(),
            "I've been saving some ancient seeds I found years ago. They're yours — I think you're the right person to grow them.".to_string(),
        ]);

        registry.npcs.insert(id.clone(), NpcDef {
            id: id.clone(),
            name: "Farmer Dale".to_string(),
            birthday_season: Season::Spring,
            birthday_day: 22,
            gift_preferences: prefs,
            default_dialogue: vec![
                "How's your farm coming along? The south fields are tricky this time of year.".to_string(),
                "Rain's coming. Good for the crops, bad for my knees.".to_string(),
                "Plant when the moon is right and harvest when your heart says ready. Old wisdom.".to_string(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 8,
            portrait_index: 8,
        });

        registry.schedules.insert(id.clone(), build_farmer_dale_schedule());
    }

    // --- Child Lily ---
    {
        let id = "child_lily".to_string();
        let mut prefs: HashMap<ItemId, GiftPreference> = HashMap::new();
        // Loved: candy and sweet food
        for item in &["cookie", "cake", "chocolate_cake", "pink_cake", "pumpkin_pie", "honey"] {
            prefs.insert(item.to_string(), GiftPreference::Loved);
        }
        // Liked: colorful things (flowers, gems)
        for item in &["tulip", "summer_spangle", "sunflower", "amethyst", "aquamarine"] {
            prefs.insert(item.to_string(), GiftPreference::Liked);
        }
        // Disliked: vegetables
        for item in &["cauliflower", "eggplant", "yam", "turnip", "leek"] {
            prefs.insert(item.to_string(), GiftPreference::Disliked);
        }
        // Hated: fish and ores (gross and boring)
        for item in &["sardine", "catfish", "copper_ore", "iron_ore", "stone"] {
            prefs.insert(item.to_string(), GiftPreference::Hated);
        }

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(3, vec![
            "I found a caterpillar today! I named it Fluffy. Mr. Thomas says they turn into butterflies but I don't believe him.".to_string(),
            "Do you want to play? I know a really good hiding spot by the old oak tree!".to_string(),
        ]);
        heart_dialogue.insert(6, vec![
            "I've been drawing pictures of all the animals in town. When I grow up I want to be an artist. Or maybe a pirate. Or both!".to_string(),
            "I climbed all the way up the hill behind the library and you can see EVERYTHING from up there! Don't tell my mom.".to_string(),
        ]);
        heart_dialogue.insert(9, vec![
            "You're my favorite grown-up. You actually listen when I talk. Most adults just nod and look somewhere else.".to_string(),
            "I made you something! It's a picture of your farm. The cows might not look exactly like cows but they have feelings so it counts.".to_string(),
        ]);

        registry.npcs.insert(id.clone(), NpcDef {
            id: id.clone(),
            name: "Lily".to_string(),
            birthday_season: Season::Summer,
            birthday_day: 16,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Hi hi hi! Whatcha doing?".to_string(),
                "I can run really really fast! Watch! ...maybe later.".to_string(),
                "Mom says I have to be home before dark but dark is SO far away!".to_string(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 9,
            portrait_index: 9,
        });

        registry.schedules.insert(id.clone(), build_child_lily_schedule());
    }

    registry
}

// ═══════════════════════════════════════════════════════════════════════
// SCHEDULE BUILDERS — each NPC has a detailed daily routine
// ═══════════════════════════════════════════════════════════════════════

fn build_mayor_thomas_schedule() -> NpcSchedule {
    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 22, y: 10 }, // home morning
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 24, y: 18 }, // town hall / plaza desk
            ScheduleEntry { time: 12.0, map: MapId::Town, x: 20, y: 22 }, // lunch spot (south plaza bench)
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 24, y: 18 }, // back to work
            ScheduleEntry { time: 17.0, map: MapId::Town, x: 18, y: 14 }, // evening walk in plaza
            ScheduleEntry { time: 19.0, map: MapId::Town, x: 22, y: 10 }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 22, y: 10 }, // sleep
        ],
        weekend: vec![
            ScheduleEntry { time: 7.0,  map: MapId::Town, x: 22, y: 10 }, // home (sleeps in)
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 18, y: 14 }, // leisurely plaza walk
            ScheduleEntry { time: 11.0, map: MapId::Town, x: 26, y: 20 }, // talking to townspeople
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 20, y: 22 }, // lunch bench
            ScheduleEntry { time: 15.0, map: MapId::Town, x: 16, y: 18 }, // visits market area
            ScheduleEntry { time: 18.0, map: MapId::Town, x: 22, y: 10 }, // home for dinner
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 22, y: 10 }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 22, y: 10 }, // home all day
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 24, y: 18 }, // indoor office work
            ScheduleEntry { time: 17.0, map: MapId::Town, x: 22, y: 10 }, // home early
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 22, y: 10 }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 24, y: 15 }, // festival grounds, leading event
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 22, y: 10 }, // home
        ]),
    }
}

fn build_elena_schedule() -> NpcSchedule {
    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town,        x: 30, y: 8  }, // home above store
            ScheduleEntry { time: 8.0,  map: MapId::GeneralStore, x: 5,  y: 8  }, // opens store
            ScheduleEntry { time: 12.0, map: MapId::GeneralStore, x: 5,  y: 6  }, // lunch at counter
            ScheduleEntry { time: 12.5, map: MapId::GeneralStore, x: 5,  y: 8  }, // back to work
            ScheduleEntry { time: 17.0, map: MapId::Town,        x: 18, y: 14 }, // evening walk
            ScheduleEntry { time: 18.0, map: MapId::Town,        x: 22, y: 10 }, // visits plaza
            ScheduleEntry { time: 20.0, map: MapId::Town,        x: 30, y: 8  }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town,        x: 30, y: 8  }, // sleep
        ],
        weekend: vec![
            ScheduleEntry { time: 7.0,  map: MapId::Town, x: 30, y: 8  }, // home, relaxed morning
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 12, y: 30 }, // south town garden area
            ScheduleEntry { time: 11.0, map: MapId::Town, x: 18, y: 14 }, // plaza socializing
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 20, y: 22 }, // lunch bench
            ScheduleEntry { time: 15.0, map: MapId::Town, x: 12, y: 30 }, // back to garden
            ScheduleEntry { time: 19.0, map: MapId::Town, x: 30, y: 8  }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 30, y: 8  }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town,        x: 30, y: 8  }, // home
            ScheduleEntry { time: 9.0,  map: MapId::GeneralStore, x: 5,  y: 8  }, // store all day
            ScheduleEntry { time: 18.0, map: MapId::Town,        x: 30, y: 8  }, // home early
            ScheduleEntry { time: 22.0, map: MapId::Town,        x: 30, y: 8  }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 22, y: 16 }, // festival area
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 30, y: 8  }, // home
        ]),
    }
}

fn build_marcus_schedule() -> NpcSchedule {
    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town,       x: 34, y: 12 }, // home
            ScheduleEntry { time: 8.0,  map: MapId::Blacksmith,  x: 4,  y: 8  }, // forge opens
            ScheduleEntry { time: 12.0, map: MapId::Blacksmith,  x: 4,  y: 6  }, // lunch at anvil
            ScheduleEntry { time: 13.0, map: MapId::Blacksmith,  x: 4,  y: 8  }, // back to work
            ScheduleEntry { time: 17.0, map: MapId::Town,       x: 26, y: 24 }, // evening walk
            ScheduleEntry { time: 19.0, map: MapId::MineEntrance, x: 4, y: 10 }, // checks mine entrance
            ScheduleEntry { time: 21.0, map: MapId::Town,       x: 34, y: 12 }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town,       x: 34, y: 12 }, // sleep
        ],
        weekend: vec![
            ScheduleEntry { time: 7.0,  map: MapId::Town,       x: 34, y: 12 }, // home
            ScheduleEntry { time: 9.0,  map: MapId::MineEntrance, x: 4, y: 10 }, // exploring near mine
            ScheduleEntry { time: 11.0, map: MapId::Town,       x: 26, y: 24 }, // south area walk
            ScheduleEntry { time: 13.0, map: MapId::Town,       x: 18, y: 14 }, // plaza
            ScheduleEntry { time: 15.0, map: MapId::Blacksmith,  x: 4,  y: 8  }, // personal projects
            ScheduleEntry { time: 19.0, map: MapId::Town,       x: 34, y: 12 }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town,       x: 34, y: 12 }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town,      x: 34, y: 12 }, // home
            ScheduleEntry { time: 9.0,  map: MapId::Blacksmith, x: 4,  y: 8  }, // working inside all day
            ScheduleEntry { time: 19.0, map: MapId::Town,      x: 34, y: 12 }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town,      x: 34, y: 12 }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 22, y: 18 }, // festival
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 34, y: 12 }, // home
        ]),
    }
}

fn build_dr_iris_schedule() -> NpcSchedule {
    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 10, y: 8  }, // home / clinic residence
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 10, y: 12 }, // clinic open
            ScheduleEntry { time: 12.0, map: MapId::Town, x: 10, y: 10 }, // lunch at clinic
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 10, y: 12 }, // back to clinic
            ScheduleEntry { time: 16.0, map: MapId::Forest, x: 8, y: 14 }, // gathering herbs in forest
            ScheduleEntry { time: 18.0, map: MapId::Town, x: 10, y: 8  }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 10, y: 8  }, // sleep
        ],
        weekend: vec![
            ScheduleEntry { time: 7.0,  map: MapId::Town,   x: 10, y: 8  }, // home
            ScheduleEntry { time: 9.0,  map: MapId::Forest,  x: 8,  y: 8  }, // herb gathering
            ScheduleEntry { time: 12.0, map: MapId::Forest,  x: 12, y: 20 }, // deeper forest research
            ScheduleEntry { time: 14.0, map: MapId::Town,   x: 18, y: 14 }, // plaza for fresh air
            ScheduleEntry { time: 16.0, map: MapId::Town,   x: 10, y: 12 }, // clinic paperwork
            ScheduleEntry { time: 20.0, map: MapId::Town,   x: 10, y: 8  }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town,   x: 10, y: 8  }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 10, y: 8  }, // home
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 10, y: 12 }, // clinic all day (busier in rain)
            ScheduleEntry { time: 20.0, map: MapId::Town, x: 10, y: 8  }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 10, y: 8  }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 22, y: 16 }, // festival (medical standby)
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 10, y: 8  }, // home
        ]),
    }
}

fn build_old_pete_schedule() -> NpcSchedule {
    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 5.0,  map: MapId::Beach, x: 16, y: 28 }, // dawn fishing, early riser
            ScheduleEntry { time: 6.0,  map: MapId::Beach, x: 14, y: 26 }, // pier fishing spot
            ScheduleEntry { time: 12.0, map: MapId::Beach, x: 16, y: 20 }, // lunch on beach
            ScheduleEntry { time: 13.0, map: MapId::Beach, x: 10, y: 28 }, // afternoon fishing spot
            ScheduleEntry { time: 17.0, map: MapId::Beach, x: 16, y: 24 }, // evening by campfire area
            ScheduleEntry { time: 20.0, map: MapId::Town,  x: 8,  y: 30 }, // home near docks
            ScheduleEntry { time: 21.0, map: MapId::Town,  x: 8,  y: 30 }, // sleep (early to bed)
        ],
        weekend: vec![
            ScheduleEntry { time: 5.0,  map: MapId::Beach,  x: 16, y: 28 }, // very early fishing
            ScheduleEntry { time: 6.0,  map: MapId::Beach,  x: 20, y: 26 }, // different spot weekend
            ScheduleEntry { time: 11.0, map: MapId::Town,   x: 18, y: 14 }, // town visit, sells fish
            ScheduleEntry { time: 13.0, map: MapId::Beach,  x: 14, y: 26 }, // back to fishing
            ScheduleEntry { time: 18.0, map: MapId::Beach,  x: 16, y: 20 }, // watching sunset
            ScheduleEntry { time: 20.0, map: MapId::Town,   x: 8,  y: 30 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Town,   x: 8,  y: 30 }, // sleep
        ],
        rain_override: Some(vec![
            // Pete LOVES fishing in the rain
            ScheduleEntry { time: 5.0,  map: MapId::Beach, x: 14, y: 26 }, // best fishing weather!
            ScheduleEntry { time: 12.0, map: MapId::Beach, x: 16, y: 20 }, // won't leave
            ScheduleEntry { time: 20.0, map: MapId::Town,  x: 8,  y: 30 }, // finally home
            ScheduleEntry { time: 21.0, map: MapId::Town,  x: 8,  y: 30 }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 18, y: 20 }, // festival fishing contest
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 8,  y: 30 }, // home
        ]),
    }
}

fn build_chef_rosa_schedule() -> NpcSchedule {
    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 5.0,  map: MapId::Town, x: 26, y: 28 }, // home above restaurant
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 24, y: 30 }, // market buying ingredients
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 24, y: 30 }, // restaurant prep
            ScheduleEntry { time: 10.0, map: MapId::Town, x: 24, y: 32 }, // restaurant open (lunch service)
            ScheduleEntry { time: 14.0, map: MapId::Town, x: 24, y: 30 }, // afternoon prep
            ScheduleEntry { time: 17.0, map: MapId::Town, x: 24, y: 32 }, // dinner service
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 26, y: 28 }, // home/cleanup
            ScheduleEntry { time: 23.0, map: MapId::Town, x: 26, y: 28 }, // sleep
        ],
        weekend: vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town,  x: 26, y: 28 }, // home
            ScheduleEntry { time: 8.0,  map: MapId::Farm,   x: 20, y: 20 }, // visits farm for ingredients
            ScheduleEntry { time: 10.0, map: MapId::Town,  x: 24, y: 32 }, // brunch service
            ScheduleEntry { time: 14.0, map: MapId::Town,  x: 18, y: 14 }, // plaza break
            ScheduleEntry { time: 16.0, map: MapId::Town,  x: 24, y: 32 }, // dinner service
            ScheduleEntry { time: 21.0, map: MapId::Town,  x: 26, y: 28 }, // home
            ScheduleEntry { time: 23.0, map: MapId::Town,  x: 26, y: 28 }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 26, y: 28 }, // home
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 24, y: 30 }, // restaurant all day (people shelter)
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 26, y: 28 }, // home
            ScheduleEntry { time: 23.0, map: MapId::Town, x: 26, y: 28 }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 22, y: 14 }, // festival food stall
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 26, y: 28 }, // home
        ]),
    }
}

fn build_miner_gil_schedule() -> NpcSchedule {
    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town,        x: 14, y: 36 }, // home near south
            ScheduleEntry { time: 7.0,  map: MapId::MineEntrance, x: 6,  y: 12 }, // mine entrance post
            ScheduleEntry { time: 12.0, map: MapId::MineEntrance, x: 4,  y: 8  }, // lunch at entrance
            ScheduleEntry { time: 13.0, map: MapId::MineEntrance, x: 6,  y: 12 }, // back on duty
            ScheduleEntry { time: 17.0, map: MapId::Town,        x: 18, y: 14 }, // plaza walk
            ScheduleEntry { time: 19.0, map: MapId::Town,        x: 14, y: 36 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Town,        x: 14, y: 36 }, // sleep
        ],
        weekend: vec![
            ScheduleEntry { time: 7.0,  map: MapId::Town,        x: 14, y: 36 }, // home
            ScheduleEntry { time: 9.0,  map: MapId::MineEntrance, x: 6,  y: 12 }, // checking the mine
            ScheduleEntry { time: 11.0, map: MapId::Town,        x: 26, y: 24 }, // south walk
            ScheduleEntry { time: 13.0, map: MapId::Town,        x: 18, y: 14 }, // plaza socializing
            ScheduleEntry { time: 16.0, map: MapId::MineEntrance, x: 6,  y: 12 }, // back to mine
            ScheduleEntry { time: 19.0, map: MapId::Town,        x: 14, y: 36 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Town,        x: 14, y: 36 }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town,        x: 14, y: 36 }, // home
            ScheduleEntry { time: 9.0,  map: MapId::MineEntrance, x: 6,  y: 12 }, // shelter at mine entrance
            ScheduleEntry { time: 19.0, map: MapId::Town,        x: 14, y: 36 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Town,        x: 14, y: 36 }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 22, y: 18 }, // festival
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 14, y: 36 }, // home
        ]),
    }
}

fn build_librarian_faye_schedule() -> NpcSchedule {
    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 8,  y: 16 }, // home near library
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 8,  y: 20 }, // library opens
            ScheduleEntry { time: 12.0, map: MapId::Town, x: 8,  y: 18 }, // lunch in library back room
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 8,  y: 20 }, // library afternoon
            ScheduleEntry { time: 17.0, map: MapId::Town, x: 12, y: 24 }, // evening walk near library
            ScheduleEntry { time: 18.0, map: MapId::Town, x: 8,  y: 16 }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 8,  y: 16 }, // sleep (reads late)
        ],
        weekend: vec![
            ScheduleEntry { time: 8.0,  map: MapId::Town,   x: 8,  y: 16 }, // home
            ScheduleEntry { time: 10.0, map: MapId::Forest,  x: 16, y: 10 }, // researching ruins in forest
            ScheduleEntry { time: 13.0, map: MapId::Town,   x: 8,  y: 20 }, // library research
            ScheduleEntry { time: 16.0, map: MapId::Town,   x: 18, y: 14 }, // plaza, people watching
            ScheduleEntry { time: 18.0, map: MapId::Town,   x: 8,  y: 16 }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town,   x: 8,  y: 16 }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 8,  y: 16 }, // home
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 8,  y: 20 }, // library all day (she loves rainy reading days)
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 8,  y: 16 }, // home late
            ScheduleEntry { time: 23.0, map: MapId::Town, x: 8,  y: 16 }, // sleep very late
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 22, y: 16 }, // festival history display
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 8,  y: 16 }, // home
        ]),
    }
}

fn build_farmer_dale_schedule() -> NpcSchedule {
    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 5.0,  map: MapId::Farm, x: 48, y: 50 }, // south farm area, early riser
            ScheduleEntry { time: 6.0,  map: MapId::Farm, x: 50, y: 52 }, // tending his south plots
            ScheduleEntry { time: 12.0, map: MapId::Farm, x: 48, y: 48 }, // lunch under tree
            ScheduleEntry { time: 13.0, map: MapId::Farm, x: 50, y: 52 }, // afternoon farming
            ScheduleEntry { time: 17.0, map: MapId::Town, x: 22, y: 38 }, // heading to town south
            ScheduleEntry { time: 18.0, map: MapId::Town, x: 20, y: 36 }, // south town walk
            ScheduleEntry { time: 20.0, map: MapId::Farm, x: 48, y: 50 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Farm, x: 48, y: 50 }, // sleep
        ],
        weekend: vec![
            ScheduleEntry { time: 5.0,  map: MapId::Farm, x: 48, y: 50 }, // home
            ScheduleEntry { time: 7.0,  map: MapId::Farm, x: 46, y: 48 }, // inspecting player's farm area
            ScheduleEntry { time: 10.0, map: MapId::Town, x: 22, y: 38 }, // town market day
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 18, y: 14 }, // plaza rest
            ScheduleEntry { time: 15.0, map: MapId::Farm, x: 50, y: 52 }, // back to farm
            ScheduleEntry { time: 19.0, map: MapId::Farm, x: 48, y: 50 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Farm, x: 48, y: 50 }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 5.0,  map: MapId::Farm, x: 48, y: 50 }, // home
            ScheduleEntry { time: 8.0,  map: MapId::Farm, x: 50, y: 52 }, // light rain work
            ScheduleEntry { time: 12.0, map: MapId::Farm, x: 48, y: 50 }, // home for heavy rain
            ScheduleEntry { time: 20.0, map: MapId::Farm, x: 48, y: 50 }, // stay home
            ScheduleEntry { time: 21.0, map: MapId::Farm, x: 48, y: 50 }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 22, y: 18 }, // festival harvest display
            ScheduleEntry { time: 21.0, map: MapId::Farm, x: 48, y: 50 }, // home
        ]),
    }
}

fn build_child_lily_schedule() -> NpcSchedule {
    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 7.0,  map: MapId::Town, x: 16, y: 28 }, // home
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 22, y: 20 }, // morning run in plaza
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 8,  y: 22 }, // near library oak tree
            ScheduleEntry { time: 11.0, map: MapId::Town, x: 18, y: 30 }, // south park area
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 16, y: 28 }, // lunch at home
            ScheduleEntry { time: 14.0, map: MapId::Town, x: 26, y: 18 }, // east plaza exploring
            ScheduleEntry { time: 16.0, map: MapId::Town, x: 22, y: 14 }, // afternoon in plaza
            ScheduleEntry { time: 18.0, map: MapId::Town, x: 16, y: 28 }, // home for dinner
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 16, y: 28 }, // bedtime
        ],
        weekend: vec![
            ScheduleEntry { time: 8.0,  map: MapId::Town,   x: 16, y: 28 }, // home, sleeps in
            ScheduleEntry { time: 9.0,  map: MapId::Town,   x: 22, y: 14 }, // plaza run
            ScheduleEntry { time: 10.0, map: MapId::Forest,  x: 6,  y: 8  }, // forest edge adventure!
            ScheduleEntry { time: 12.0, map: MapId::Beach,   x: 16, y: 16 }, // beach!
            ScheduleEntry { time: 14.0, map: MapId::Beach,   x: 20, y: 20 }, // building sand castles
            ScheduleEntry { time: 16.0, map: MapId::Town,   x: 18, y: 14 }, // back to town, tired
            ScheduleEntry { time: 18.0, map: MapId::Town,   x: 16, y: 28 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Town,   x: 16, y: 28 }, // bedtime
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 7.0,  map: MapId::Town, x: 16, y: 28 }, // home (mom won't let her out)
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 8,  y: 20 }, // library (forced to read)
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 16, y: 28 }, // home lunch
            ScheduleEntry { time: 14.0, map: MapId::Town, x: 8,  y: 20 }, // library again (annoyed)
            ScheduleEntry { time: 18.0, map: MapId::Town, x: 16, y: 28 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 16, y: 28 }, // bedtime
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 22, y: 14 }, // festival excitement (arrives first)
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 16, y: 28 }, // home (forced bedtime)
        ]),
    }
}

/// Get the display color for each NPC as a Bevy Color.
/// Used for placeholder sprites.
pub fn npc_color(npc_id: &str) -> Color {
    match npc_id {
        "mayor_thomas"     => Color::srgb(0.4, 0.3, 0.7),   // regal purple
        "elena"            => Color::srgb(0.9, 0.5, 0.7),   // soft pink
        "marcus"           => Color::srgb(0.5, 0.4, 0.3),   // forge-brown
        "dr_iris"          => Color::srgb(0.3, 0.7, 0.7),   // teal (medical)
        "old_pete"         => Color::srgb(0.5, 0.5, 0.3),   // weathered tan
        "chef_rosa"        => Color::srgb(0.8, 0.3, 0.2),   // warm red (chef coat)
        "miner_gil"        => Color::srgb(0.4, 0.4, 0.4),   // stone grey
        "librarian_faye"   => Color::srgb(0.6, 0.4, 0.8),   // scholarly violet
        "farmer_dale"      => Color::srgb(0.4, 0.6, 0.3),   // earthy green
        "child_lily"       => Color::srgb(1.0, 0.8, 0.2),   // bright sunny yellow
        _                  => Color::srgb(0.8, 0.8, 0.8),   // fallback grey
    }
}

/// All 10 NPC IDs in a fixed order.
pub const ALL_NPC_IDS: &[&str] = &[
    "mayor_thomas",
    "elena",
    "marcus",
    "dr_iris",
    "old_pete",
    "chef_rosa",
    "miner_gil",
    "librarian_faye",
    "farmer_dale",
    "child_lily",
];

use crate::shared::*;
use std::collections::HashMap;

/// Populate the NpcRegistry with 10 named NPCs, their gift preferences,
/// dialogue, and daily schedules.
///
/// NPCs:
///   1. Margaret — elderly baker, loves baking and cats
///   2. Marco    — Italian cook, runs the restaurant
///   3. Lily     — young florist, marriageable
///   4. Old Tom  — retired fisherman, grumpy but kind
///   5. Elena    — blacksmith's daughter, marriageable
///   6. Mira     — traveling merchant who settled in town
///   7. Doc      — the town doctor
///   8. Mayor Rex— town mayor, pompous but well-meaning
///   9. Sam      — teenage boy who wants to be a rock star
///  10. Nora     — farmer next door, veteran agriculturalist
pub fn populate_npcs(registry: &mut NpcRegistry) {
    // ── 1. Margaret ─────────────────────────────────────────────────────────────
    {
        let mut prefs = HashMap::new();
        // Loved
        prefs.insert("cake".into(), GiftPreference::Loved);
        prefs.insert("cookie".into(), GiftPreference::Loved);
        prefs.insert("strawberry".into(), GiftPreference::Loved);
        prefs.insert("maple_syrup".into(), GiftPreference::Loved);
        prefs.insert("blueberry".into(), GiftPreference::Loved);
        // Liked
        prefs.insert("egg".into(), GiftPreference::Liked);
        prefs.insert("milk".into(), GiftPreference::Liked);
        prefs.insert("bread".into(), GiftPreference::Liked);
        prefs.insert("pancakes".into(), GiftPreference::Liked);
        prefs.insert("fruit_salad".into(), GiftPreference::Liked);
        // Disliked
        prefs.insert("coal".into(), GiftPreference::Disliked);
        prefs.insert("stone".into(), GiftPreference::Disliked);
        prefs.insert("slime".into(), GiftPreference::Disliked);
        prefs.insert("iron_ore".into(), GiftPreference::Disliked);
        prefs.insert("fiber".into(), GiftPreference::Disliked);
        // Hated
        prefs.insert("pufferfish".into(), GiftPreference::Hated);

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(0, vec![
            "Good morning, dear! Don't forget to eat before you start working.".into(),
            "The smell of fresh bread in the morning — is there anything better?".into(),
            "I've been baking since before your parents were born. Still love it.".into(),
            "My cat Biscuit has been following me around the bakery all morning. Doesn't help with the dough.".into(),
            "If you ever want fresh bread, the early bird gets the best loaves.".into(),
        ]);
        heart_dialogue.insert(3, vec![
            "You know, not many young folks visit me these days. It means a lot.".into(),
            "I've been thinking of trying a new recipe. Maybe you could taste-test it?".into(),
            "Biscuit seems to like you. That cat doesn't warm up to just anybody.".into(),
            "Come by the bakery sometime when it's quiet. I'll teach you a thing or two about pastry.".into(),
            "The farm smells wonderful lately. Fresh earth and something blooming — it drifts all the way to my window.".into(),
        ]);
        heart_dialogue.insert(6, vec![
            "You remind me of my daughter when she was your age. She left for the city...".into(),
            "Here, take this recipe. It was my mother's. I want someone to carry it on.".into(),
            "I haven't told many people this, but I almost left Hearthfield too, years ago. Something kept me here.".into(),
            "Biscuit sleeps on the recipe box. I think she's guarding the good ones for me.".into(),
            "My husband used to say that a good loaf of bread is an act of love. I believe that more every year.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "This town is lucky to have someone like you looking after the old farm.".into(),
            "Promise me you'll never let this place become what it was before you came.".into(),
            "I've started saving the corner piece of every pie for you. Don't tell the other customers.".into(),
            "You've given this old bakery more life than it's seen in years. I can feel it in my morning dough.".into(),
        ]);

        let npc = NpcDef {
            id: "margaret".into(),
            name: "Margaret".into(),
            birthday_season: Season::Spring,
            birthday_day: 14,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Good morning, dear!".into(),
                "Have you eaten today? You look thin.".into(),
                "The bakery opens at 9. Fresh bread every day!".into(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 0,
            portrait_index: 0,
        };

        let schedule = NpcSchedule {
            weekday: vec![
                ScheduleEntry {
                    time: 6.0,
                    map: MapId::GeneralStore,
                    x: 5,
                    y: 3,
                },
                ScheduleEntry {
                    time: 9.0,
                    map: MapId::Town,
                    x: 12,
                    y: 8,
                },
                ScheduleEntry {
                    time: 12.0,
                    map: MapId::GeneralStore,
                    x: 3,
                    y: 5,
                },
                ScheduleEntry {
                    time: 17.0,
                    map: MapId::Town,
                    x: 10,
                    y: 15,
                },
                ScheduleEntry {
                    time: 20.0,
                    map: MapId::GeneralStore,
                    x: 2,
                    y: 2,
                },
            ],
            weekend: vec![
                ScheduleEntry {
                    time: 8.0,
                    map: MapId::Town,
                    x: 8,
                    y: 10,
                },
                ScheduleEntry {
                    time: 11.0,
                    map: MapId::Town,
                    x: 14,
                    y: 12,
                },
                ScheduleEntry {
                    time: 14.0,
                    map: MapId::Beach,
                    x: 5,
                    y: 13,
                },
                ScheduleEntry {
                    time: 19.0,
                    map: MapId::GeneralStore,
                    x: 2,
                    y: 2,
                },
            ],
            rain_override: Some(vec![
                ScheduleEntry {
                    time: 6.0,
                    map: MapId::GeneralStore,
                    x: 5,
                    y: 3,
                },
                ScheduleEntry {
                    time: 21.0,
                    map: MapId::GeneralStore,
                    x: 2,
                    y: 2,
                },
            ]),
            festival_override: None,
        };

        registry.npcs.insert("margaret".into(), npc);
        registry.schedules.insert("margaret".into(), schedule);
    }

    // ── 2. Marco ─────────────────────────────────────────────────────────────────
    {
        let mut prefs = HashMap::new();
        prefs.insert("pizza".into(), GiftPreference::Loved);
        prefs.insert("spaghetti".into(), GiftPreference::Loved);
        prefs.insert("tomato".into(), GiftPreference::Loved);
        prefs.insert("tuna".into(), GiftPreference::Loved);
        prefs.insert("eggplant".into(), GiftPreference::Loved);
        prefs.insert("fish_stew".into(), GiftPreference::Liked);
        prefs.insert("cheese".into(), GiftPreference::Liked);
        prefs.insert("salad".into(), GiftPreference::Liked);
        prefs.insert("cooked_fish".into(), GiftPreference::Liked);
        prefs.insert("corn".into(), GiftPreference::Liked);
        prefs.insert("coal".into(), GiftPreference::Disliked);
        prefs.insert("stone".into(), GiftPreference::Disliked);
        prefs.insert("slime".into(), GiftPreference::Disliked);
        prefs.insert("wood".into(), GiftPreference::Disliked);
        prefs.insert("fiber".into(), GiftPreference::Disliked);
        prefs.insert("carp".into(), GiftPreference::Hated);

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(0, vec![
            "Benvenuto! Welcome to Marco's Trattoria!".into(),
            "Food is love made visible. That is what my nonna always said.".into(),
            "You must try my pasta. It is a family recipe from the old country.".into(),
            "The secret to good cooking? Quality ingredients and a little patience.".into(),
            "I opened this restaurant because I wanted to share something beautiful with the world.".into(),
        ]);
        heart_dialogue.insert(3, vec![
            "You have good taste, my friend. In food and in company.".into(),
            "Perhaps one day I teach you my secret tomato sauce, eh?".into(),
            "Not everyone appreciates real cooking. But you — you eat with your whole heart.".into(),
            "I notice you always finish the plate. A chef notices these things. It makes us happy.".into(),
            "The herbs in my garden remind me of my mother's kitchen window. Simple things bring the most comfort.".into(),
        ]);
        heart_dialogue.insert(6, vec![
            "You remind me of my little brother back home. I miss him terribly.".into(),
            "Here, I have written down the recipe. Guard it well.".into(),
            "Sometimes at closing time the restaurant is so quiet it hurts. I did not expect to feel lonely here.".into(),
            "You are the kind of person my nonna would have adopted on the spot. She had good instincts.".into(),
            "Cooking for one person who truly appreciates it is better than cooking for a hundred who do not.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "You are family now. And family always eats well!".into(),
            "Perhaps I open a second restaurant when the farm is doing well, no?".into(),
            "I write home and tell my brother about you. He says you sound like a good soul.".into(),
            "Every evening I save the best seat by the window. It is yours whenever you want it.".into(),
        ]);

        let npc = NpcDef {
            id: "marco".into(),
            name: "Marco".into(),
            birthday_season: Season::Summer,
            birthday_day: 8,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Ciao! The pasta is fresh today!".into(),
                "You look hungry. You should come for dinner!".into(),
                "I am experimenting with a new sauce. Very exciting!".into(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 1,
            portrait_index: 1,
        };

        let schedule = NpcSchedule {
            weekday: vec![
                ScheduleEntry {
                    time: 7.0,
                    map: MapId::Town,
                    x: 20,
                    y: 5,
                },
                ScheduleEntry {
                    time: 10.0,
                    map: MapId::Town,
                    x: 18,
                    y: 8,
                },
                ScheduleEntry {
                    time: 14.0,
                    map: MapId::Town,
                    x: 22,
                    y: 6,
                },
                ScheduleEntry {
                    time: 19.0,
                    map: MapId::Town,
                    x: 20,
                    y: 10,
                },
                ScheduleEntry {
                    time: 22.0,
                    map: MapId::Town,
                    x: 20,
                    y: 5,
                },
            ],
            weekend: vec![
                ScheduleEntry {
                    time: 8.0,
                    map: MapId::Beach,
                    x: 10,
                    y: 5,
                },
                ScheduleEntry {
                    time: 12.0,
                    map: MapId::Town,
                    x: 22,
                    y: 6,
                },
                ScheduleEntry {
                    time: 18.0,
                    map: MapId::Town,
                    x: 20,
                    y: 10,
                },
                ScheduleEntry {
                    time: 22.0,
                    map: MapId::Town,
                    x: 20,
                    y: 5,
                },
            ],
            rain_override: Some(vec![
                ScheduleEntry {
                    time: 7.0,
                    map: MapId::Town,
                    x: 20,
                    y: 5,
                },
                ScheduleEntry {
                    time: 22.0,
                    map: MapId::Town,
                    x: 20,
                    y: 5,
                },
            ]),
            festival_override: None,
        };

        registry.npcs.insert("marco".into(), npc);
        registry.schedules.insert("marco".into(), schedule);
    }

    // ── 3. Lily ──────────────────────────────────────────────────────────────────
    {
        let mut prefs = HashMap::new();
        prefs.insert("fruit_salad".into(), GiftPreference::Loved);
        prefs.insert("strawberry".into(), GiftPreference::Loved);
        prefs.insert("blueberry".into(), GiftPreference::Loved);
        prefs.insert("melon".into(), GiftPreference::Loved);
        prefs.insert("ancient_fruit".into(), GiftPreference::Loved);
        prefs.insert("salad".into(), GiftPreference::Liked);
        prefs.insert("cake".into(), GiftPreference::Liked);
        prefs.insert("cookie".into(), GiftPreference::Liked);
        prefs.insert("cranberry".into(), GiftPreference::Liked);
        prefs.insert("maple_syrup".into(), GiftPreference::Liked);
        prefs.insert("iron_ore".into(), GiftPreference::Disliked);
        prefs.insert("coal".into(), GiftPreference::Disliked);
        prefs.insert("stone".into(), GiftPreference::Disliked);
        prefs.insert("slime".into(), GiftPreference::Disliked);
        prefs.insert("sap".into(), GiftPreference::Disliked);
        prefs.insert("pufferfish".into(), GiftPreference::Hated);

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(0, vec![
            "Every flower has a story. I wonder what yours is?".into(),
            "I grow flowers here, but I've always been curious about vegetable farming.".into(),
            "The spring flowers are especially beautiful this year, don't you think?".into(),
            "I can tell which flowers grew wild and which were planted with care. Yours feel like both.".into(),
            "Sometimes I press a single bloom between pages just to remember a good day.".into(),
        ]);
        heart_dialogue.insert(3, vec![
            "I don't open up to many people, but you seem different somehow.".into(),
            "I've been pressing flowers into a journal. Would you like to see it sometime?".into(),
            "I tried growing vegetables once, just a little patch. Turns out I have a lot to learn about your side of farming.".into(),
            "I like that you stop to notice the small things — a single bloom, a bent stem. Not everyone does.".into(),
            "Do you have a favorite flower? I keep meaning to ask.".into(),
        ]);
        heart_dialogue.insert(6, vec![
            "I... I find myself looking forward to our conversations more than I expected.".into(),
            "My dream is to have a garden that spans all four seasons. Maybe we could plan it together?".into(),
            "I stayed up all night once drawing garden layouts. I've never shown anyone. Maybe I'll show you.".into(),
            "There's a corner of the flower shop I keep just for me — no customers allowed. I think I'd let you in.".into(),
            "When a flower blooms that I've been tending for months... I think of you. I'm not sure why.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "You've changed something in this town. I think it's hope. We all needed that.".into(),
            "I wrote you something. It's a little embarrassing, but... please read it?".into(),
            "I planted a whole new section of the garden, just for the two of us. It blooms in every season.".into(),
            "When I imagine next spring, I imagine it with you. That's a new feeling, and I really like it.".into(),
        ]);

        let npc = NpcDef {
            id: "lily".into(),
            name: "Lily".into(),
            birthday_season: Season::Spring,
            birthday_day: 22,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Oh! I was just arranging the seasonal flowers.".into(),
                "Hello! The flowers are especially lovely today.".into(),
                "I can make a bouquet for someone special, if you'd like.".into(),
            ],
            heart_dialogue,
            is_marriageable: true,
            sprite_index: 2,
            portrait_index: 2,
        };

        let schedule = NpcSchedule {
            weekday: vec![
                ScheduleEntry {
                    time: 7.0,
                    map: MapId::Town,
                    x: 5,
                    y: 18,
                },
                ScheduleEntry {
                    time: 9.0,
                    map: MapId::Forest,
                    x: 10,
                    y: 8,
                },
                ScheduleEntry {
                    time: 12.0,
                    map: MapId::Town,
                    x: 7,
                    y: 18,
                },
                ScheduleEntry {
                    time: 16.0,
                    map: MapId::Town,
                    x: 5,
                    y: 18,
                },
                ScheduleEntry {
                    time: 20.0,
                    map: MapId::Town,
                    x: 5,
                    y: 18,
                },
            ],
            weekend: vec![
                ScheduleEntry {
                    time: 8.0,
                    map: MapId::Beach,
                    x: 15,
                    y: 8,
                },
                ScheduleEntry {
                    time: 12.0,
                    map: MapId::Town,
                    x: 7,
                    y: 18,
                },
                ScheduleEntry {
                    time: 16.0,
                    map: MapId::Forest,
                    x: 12,
                    y: 6,
                },
                ScheduleEntry {
                    time: 20.0,
                    map: MapId::Town,
                    x: 5,
                    y: 18,
                },
            ],
            rain_override: Some(vec![
                ScheduleEntry {
                    time: 7.0,
                    map: MapId::Town,
                    x: 5,
                    y: 18,
                },
                ScheduleEntry {
                    time: 14.0,
                    map: MapId::Town,
                    x: 8,
                    y: 12,
                },
                ScheduleEntry {
                    time: 20.0,
                    map: MapId::Town,
                    x: 5,
                    y: 18,
                },
            ]),
            festival_override: None,
        };

        registry.npcs.insert("lily".into(), npc);
        registry.schedules.insert("lily".into(), schedule);
    }

    // ── 4. Old Tom ────────────────────────────────────────────────────────────────
    {
        let mut prefs = HashMap::new();
        prefs.insert("legend_fish".into(), GiftPreference::Loved);
        prefs.insert("sturgeon".into(), GiftPreference::Loved);
        prefs.insert("salmon".into(), GiftPreference::Loved);
        prefs.insert("swordfish".into(), GiftPreference::Loved);
        prefs.insert("catfish".into(), GiftPreference::Loved);
        prefs.insert("tuna".into(), GiftPreference::Liked);
        prefs.insert("bass".into(), GiftPreference::Liked);
        prefs.insert("eel".into(), GiftPreference::Liked);
        prefs.insert("fish_stew".into(), GiftPreference::Liked);
        prefs.insert("cooked_fish".into(), GiftPreference::Liked);
        prefs.insert("strawberry".into(), GiftPreference::Disliked);
        prefs.insert("melon".into(), GiftPreference::Disliked);
        prefs.insert("cake".into(), GiftPreference::Disliked);
        prefs.insert("cookie".into(), GiftPreference::Disliked);
        prefs.insert("maple_syrup".into(), GiftPreference::Disliked);
        prefs.insert("carp".into(), GiftPreference::Hated);

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(0, vec![
            "Hmph. The fish aren't biting today. Or any day, lately.".into(),
            "What do you want? I'm busy watching the water.".into(),
            "In my day, the river was full of fish. Things change.".into(),
            "You'll want a heavier line if you're fishing deeper water. Not that I'm giving advice.".into(),
            "The beach is quieter before dawn. That's when the real fishermen are out.".into(),
        ]);
        heart_dialogue.insert(3, vec![
            "...You're alright, I suppose. For a newcomer.".into(),
            "My old fishing spot — the one past the bridge. Nobody knows about it but me. And now you.".into(),
            "You're learning fast. Not many newcomers bother to learn the water.".into(),
            "Keep your hook baited and your mouth shut. Those are the two rules of good fishing.".into(),
            "The fish have been biting earlier lately. You'd know that if you came out at sunrise more often.".into(),
        ]);
        heart_dialogue.insert(6, vec![
            "I don't talk about my wife often. But she loved this river too.".into(),
            "Here. My fishing rod is better than yours. Take it.".into(),
            "She used to sit right there on that rock and read while I fished. Never bothered me once. That's rare.".into(),
            "I come here to remember her mostly. The fish are just an excuse.".into(),
            "You remind me of her a bit. She had that same look — like you're really paying attention.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "You've made this old town worth waking up for again. Don't tell anyone I said that.".into(),
            "If you catch the Legend, you bring it to me first. That's the deal.".into(),
            "I carved your name into my fishing post. Right under hers. Means something, that.".into(),
            "You're the only one who ever listened to my old stories without walking away. That counts for more than you know.".into(),
        ]);

        let npc = NpcDef {
            id: "old_tom".into(),
            name: "Old Tom".into(),
            birthday_season: Season::Winter,
            birthday_day: 3,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Hmph.".into(),
                "The river was better thirty years ago.".into(),
                "Leave me be, I'm thinking.".into(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 3,
            portrait_index: 3,
        };

        let schedule = NpcSchedule {
            weekday: vec![
                ScheduleEntry {
                    time: 5.0,
                    map: MapId::Beach,
                    x: 20,
                    y: 13,
                },
                ScheduleEntry {
                    time: 12.0,
                    map: MapId::Town,
                    x: 15,
                    y: 18,
                },
                ScheduleEntry {
                    time: 15.0,
                    map: MapId::Beach,
                    x: 18,
                    y: 10,
                },
                ScheduleEntry {
                    time: 20.0,
                    map: MapId::Town,
                    x: 15,
                    y: 18,
                },
            ],
            weekend: vec![
                ScheduleEntry {
                    time: 5.0,
                    map: MapId::Beach,
                    x: 20,
                    y: 13,
                },
                ScheduleEntry {
                    time: 14.0,
                    map: MapId::Beach,
                    x: 22,
                    y: 12,
                },
                ScheduleEntry {
                    time: 20.0,
                    map: MapId::Town,
                    x: 15,
                    y: 18,
                },
            ],
            rain_override: Some(vec![
                ScheduleEntry {
                    time: 6.0,
                    map: MapId::Town,
                    x: 15,
                    y: 18,
                },
                ScheduleEntry {
                    time: 22.0,
                    map: MapId::Town,
                    x: 15,
                    y: 18,
                },
            ]),
            festival_override: None,
        };

        registry.npcs.insert("old_tom".into(), npc);
        registry.schedules.insert("old_tom".into(), schedule);
    }

    // ── 5. Elena ──────────────────────────────────────────────────────────────────
    {
        let mut prefs = HashMap::new();
        prefs.insert("gold_bar".into(), GiftPreference::Loved);
        prefs.insert("iron_bar".into(), GiftPreference::Loved);
        prefs.insert("diamond".into(), GiftPreference::Loved);
        prefs.insert("ruby".into(), GiftPreference::Loved);
        prefs.insert("emerald".into(), GiftPreference::Loved);
        prefs.insert("copper_bar".into(), GiftPreference::Liked);
        prefs.insert("amethyst".into(), GiftPreference::Liked);
        prefs.insert("quartz".into(), GiftPreference::Liked);
        prefs.insert("iridium_bar".into(), GiftPreference::Liked);
        prefs.insert("coal".into(), GiftPreference::Liked);
        prefs.insert("strawberry".into(), GiftPreference::Disliked);
        prefs.insert("cake".into(), GiftPreference::Disliked);
        prefs.insert("blueberry".into(), GiftPreference::Disliked);
        prefs.insert("melon".into(), GiftPreference::Disliked);
        prefs.insert("turnip".into(), GiftPreference::Disliked);
        prefs.insert("carp".into(), GiftPreference::Hated);

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(
            0,
            vec![
                "Don't touch the equipment without asking.".into(),
                "Father is busy. I handle the shop when he's working.".into(),
                "Good quality work speaks for itself. No need for fancy words.".into(),
                "Iron won't wait for carelessness. Neither will I.".into(),
                "If your tools are dull, you're doing double work. Bring them in.".into(),
            ],
        );
        heart_dialogue.insert(3, vec![
            "Most people just want the cheapest option. You seem to care about quality. Refreshing.".into(),
            "I'm working on something new. A special alloy. Father doesn't know yet.".into(),
            "You've been using your tools hard. I can tell by the wear pattern. That means you're doing real work.".into(),
            "The mine has good ore in the lower floors, if you're not afraid of the dark.".into(),
            "I don't have many friends, honestly. The forge keeps me busy. But I don't mind talking to you.".into(),
        ]);
        heart_dialogue.insert(6, vec![
            "I've been doing this my whole life, but sometimes I wonder if there's more out there.".into(),
            "You can watch me work, if you want. But don't talk while I'm concentrating.".into(),
            "Father wants me to take over the shop someday. I think I want that too... mostly.".into(),
            "Metal remembers the hand that shaped it. I like to think what I make carries something of me forward.".into(),
            "I showed you something I've never shown anyone — that alloy project. I think that means something.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "I made something for you. It's not sentimental. It's just good craftsmanship.".into(),
            "You're the only person who's ever understood why this work matters. That means something.".into(),
            "I put a maker's mark on what I made you. My own mark. I've never given that to anyone before.".into(),
            "When you're out on the farm, I think about whether my tools are holding up. I want them to take care of you.".into(),
        ]);

        let npc = NpcDef {
            id: "elena".into(),
            name: "Elena".into(),
            birthday_season: Season::Fall,
            birthday_day: 18,
            gift_preferences: prefs,
            default_dialogue: vec![
                "The forge is hot today.".into(),
                "Bring your tools if they need upgrading.".into(),
                "Father's working on something big in the back.".into(),
            ],
            heart_dialogue,
            is_marriageable: true,
            sprite_index: 4,
            portrait_index: 4,
        };

        let schedule = NpcSchedule {
            weekday: vec![
                ScheduleEntry {
                    time: 8.0,
                    map: MapId::Blacksmith,
                    x: 3,
                    y: 5,
                },
                ScheduleEntry {
                    time: 12.0,
                    map: MapId::Town,
                    x: 8,
                    y: 5,
                },
                ScheduleEntry {
                    time: 13.0,
                    map: MapId::Blacksmith,
                    x: 3,
                    y: 5,
                },
                ScheduleEntry {
                    time: 18.0,
                    map: MapId::Town,
                    x: 6,
                    y: 4,
                },
                ScheduleEntry {
                    time: 21.0,
                    map: MapId::Blacksmith,
                    x: 2,
                    y: 2,
                },
            ],
            weekend: vec![
                ScheduleEntry {
                    time: 9.0,
                    map: MapId::MineEntrance,
                    x: 5,
                    y: 5,
                },
                ScheduleEntry {
                    time: 13.0,
                    map: MapId::Town,
                    x: 8,
                    y: 5,
                },
                ScheduleEntry {
                    time: 17.0,
                    map: MapId::Blacksmith,
                    x: 3,
                    y: 5,
                },
                ScheduleEntry {
                    time: 21.0,
                    map: MapId::Blacksmith,
                    x: 2,
                    y: 2,
                },
            ],
            rain_override: Some(vec![
                ScheduleEntry {
                    time: 8.0,
                    map: MapId::Blacksmith,
                    x: 3,
                    y: 5,
                },
                ScheduleEntry {
                    time: 21.0,
                    map: MapId::Blacksmith,
                    x: 2,
                    y: 2,
                },
            ]),
            festival_override: None,
        };

        registry.npcs.insert("elena".into(), npc);
        registry.schedules.insert("elena".into(), schedule);
    }

    // ── 6. Mira ───────────────────────────────────────────────────────────────────
    {
        let mut prefs = HashMap::new();
        prefs.insert("ancient_fruit".into(), GiftPreference::Loved);
        prefs.insert("cloth".into(), GiftPreference::Loved);
        prefs.insert("diamond".into(), GiftPreference::Loved);
        prefs.insert("wool".into(), GiftPreference::Loved);
        prefs.insert("mayonnaise".into(), GiftPreference::Loved);
        prefs.insert("gold_bar".into(), GiftPreference::Liked);
        prefs.insert("cheese".into(), GiftPreference::Liked);
        prefs.insert("pumpkin".into(), GiftPreference::Liked);
        prefs.insert("salad".into(), GiftPreference::Liked);
        prefs.insert("maple_syrup".into(), GiftPreference::Liked);
        prefs.insert("stone".into(), GiftPreference::Disliked);
        prefs.insert("coal".into(), GiftPreference::Disliked);
        prefs.insert("slime".into(), GiftPreference::Disliked);
        prefs.insert("sap".into(), GiftPreference::Disliked);
        prefs.insert("fiber".into(), GiftPreference::Disliked);
        prefs.insert("carp".into(), GiftPreference::Hated);

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(0, vec![
            "I've been all over the world. This town called to me. I don't know why yet.".into(),
            "Everything I sell has a story. Ask me and I'll tell you.".into(),
            "Rarity is beauty. Or so I've always believed.".into(),
            "I've traded in five languages and three currencies. Hearthfield is refreshingly simple.".into(),
            "A good merchant knows when to sell and when to wait. I'm still deciding about this town.".into(),
        ]);
        heart_dialogue.insert(3, vec![
            "You have a good eye. Rare in a farmer. Rarer still in anyone.".into(),
            "I have something I've been saving for the right person. Perhaps that's you.".into(),
            "The Eastern desert markets taught me to read people fast. You've been an interesting puzzle.".into(),
            "Not many people ask where my wares come from. You always do. I appreciate that.".into(),
            "I've met travelers who were chasing something for years without knowing what. I think I understand that feeling now.".into(),
        ]);
        heart_dialogue.insert(6, vec![
            "I ran from a place that didn't value what I valued. This town might be different.".into(),
            "You've given me a reason to stop running, I think.".into(),
            "Every place I've settled, I kept one eye on the road. I notice I'm not doing that here.".into(),
            "I keep a journal of the best markets I've seen. This town has made it onto the list for unexpected reasons.".into(),
            "Somewhere between the first sale and the hundredth conversation, Hearthfield became... home. Strange word for me.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "Home is where the people understand you. I think I've found mine.".into(),
            "Whatever you need — knowledge, goods, passage — I will find a way.".into(),
            "I sent a letter to an old caravan friend. Told them I'd stopped moving. They didn't believe me. Neither did I.".into(),
            "You are the reason I unpacked the last chest. The one I always kept ready to go.".into(),
        ]);

        let npc = NpcDef {
            id: "mira".into(),
            name: "Mira".into(),
            birthday_season: Season::Summer,
            birthday_day: 19,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Welcome, welcome! Rare goods for rare tastes.".into(),
                "I traveled the world to find this stock. Worth every mile.".into(),
                "Tell me what you seek and I'll see if I have it.".into(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 5,
            portrait_index: 5,
        };

        let schedule = NpcSchedule {
            weekday: vec![
                ScheduleEntry {
                    time: 8.0,
                    map: MapId::GeneralStore,
                    x: 8,
                    y: 4,
                },
                ScheduleEntry {
                    time: 12.0,
                    map: MapId::Town,
                    x: 18,
                    y: 18,
                },
                ScheduleEntry {
                    time: 15.0,
                    map: MapId::GeneralStore,
                    x: 8,
                    y: 4,
                },
                ScheduleEntry {
                    time: 19.0,
                    map: MapId::Town,
                    x: 20,
                    y: 18,
                },
            ],
            weekend: vec![
                ScheduleEntry {
                    time: 9.0,
                    map: MapId::Town,
                    x: 15,
                    y: 18,
                },
                ScheduleEntry {
                    time: 13.0,
                    map: MapId::Beach,
                    x: 8,
                    y: 5,
                },
                ScheduleEntry {
                    time: 18.0,
                    map: MapId::Town,
                    x: 20,
                    y: 18,
                },
            ],
            rain_override: Some(vec![
                ScheduleEntry {
                    time: 8.0,
                    map: MapId::GeneralStore,
                    x: 8,
                    y: 4,
                },
                ScheduleEntry {
                    time: 20.0,
                    map: MapId::Town,
                    x: 20,
                    y: 18,
                },
            ]),
            festival_override: None,
        };

        registry.npcs.insert("mira".into(), npc);
        registry.schedules.insert("mira".into(), schedule);
    }

    // ── 7. Doc ────────────────────────────────────────────────────────────────────
    {
        let mut prefs = HashMap::new();
        prefs.insert("pumpkin_soup".into(), GiftPreference::Loved);
        prefs.insert("fruit_salad".into(), GiftPreference::Loved);
        prefs.insert("salad".into(), GiftPreference::Loved);
        prefs.insert("cauliflower".into(), GiftPreference::Loved);
        prefs.insert("cheese_omelette".into(), GiftPreference::Loved);
        prefs.insert("yam".into(), GiftPreference::Liked);
        prefs.insert("potato".into(), GiftPreference::Liked);
        prefs.insert("turnip".into(), GiftPreference::Liked);
        prefs.insert("baked_potato".into(), GiftPreference::Liked);
        prefs.insert("fish_stew".into(), GiftPreference::Liked);
        prefs.insert("slime".into(), GiftPreference::Disliked);
        prefs.insert("coal".into(), GiftPreference::Disliked);
        prefs.insert("sap".into(), GiftPreference::Disliked);
        prefs.insert("iron_ore".into(), GiftPreference::Disliked);
        prefs.insert("stone".into(), GiftPreference::Disliked);
        prefs.insert("pufferfish".into(), GiftPreference::Hated);

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(0, vec![
            "Are you getting enough sleep? You look tired.".into(),
            "I'm available for medical consultation most mornings.".into(),
            "A healthy farmer is a productive farmer. Don't overdo it.".into(),
            "Hydration matters more than most people realize. Especially on hot days in the field.".into(),
            "If something aches and you ignore it for a week, it usually becomes two problems. Come see me sooner.".into(),
        ]);
        heart_dialogue.insert(
            3,
            vec![
                "I worry about this community sometimes. It's fragile. Like all communities."
                    .into(),
                "You're taking care of yourself, right? Eat well, rest often.".into(),
                "I notice you stop to check on your neighbors. That's good medicine too.".into(),
                "The farm work is hard on the body. Make sure you're stretching. No, really."
                    .into(),
                "I came here thinking small towns were simpler. They're not. Just quieter.".into(),
            ],
        );
        heart_dialogue.insert(6, vec![
            "When I came here, I thought I was running away from medicine. Now I realize I needed a different kind.".into(),
            "Let me check on you. Farmer's body takes a beating. Better safe than sorry.".into(),
            "I burned out in the city. Too many patients, not enough names. Here, I know everyone's name.".into(),
            "You remind me why I became a doctor in the first place. Not for the clinic. For the people.".into(),
            "I keep a little notebook of everyone in town — health notes, mostly, but also favorite things. You've got your own page.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "I've seen this town healed by you. That's not a small thing. That's everything.".into(),
            "You ever need anything — anything at all — you come find me first. That's a doctor's promise.".into(),
            "I think I'll stay in Hearthfield forever. I didn't know that until very recently.".into(),
            "Whatever happens — storms, drought, hard years — you won't face it alone. That's not medicine. That's just true.".into(),
        ]);

        let npc = NpcDef {
            id: "doc".into(),
            name: "Doc".into(),
            birthday_season: Season::Winter,
            birthday_day: 14,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Good health is wealth. How are you feeling today?".into(),
                "My clinic is open most mornings. Don't hesitate to stop by.".into(),
                "Eat your vegetables. That's not just advice, it's medicine.".into(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 6,
            portrait_index: 6,
        };

        let schedule = NpcSchedule {
            weekday: vec![
                ScheduleEntry {
                    time: 7.0,
                    map: MapId::Town,
                    x: 25,
                    y: 10,
                },
                ScheduleEntry {
                    time: 9.0,
                    map: MapId::Town,
                    x: 24,
                    y: 12,
                },
                ScheduleEntry {
                    time: 13.0,
                    map: MapId::Town,
                    x: 20,
                    y: 15,
                },
                ScheduleEntry {
                    time: 17.0,
                    map: MapId::Town,
                    x: 25,
                    y: 10,
                },
                ScheduleEntry {
                    time: 21.0,
                    map: MapId::Town,
                    x: 25,
                    y: 10,
                },
            ],
            weekend: vec![
                ScheduleEntry {
                    time: 8.0,
                    map: MapId::Forest,
                    x: 5,
                    y: 10,
                },
                ScheduleEntry {
                    time: 12.0,
                    map: MapId::Town,
                    x: 24,
                    y: 12,
                },
                ScheduleEntry {
                    time: 18.0,
                    map: MapId::Town,
                    x: 25,
                    y: 10,
                },
            ],
            rain_override: Some(vec![
                ScheduleEntry {
                    time: 7.0,
                    map: MapId::Town,
                    x: 25,
                    y: 10,
                },
                ScheduleEntry {
                    time: 21.0,
                    map: MapId::Town,
                    x: 25,
                    y: 10,
                },
            ]),
            festival_override: None,
        };

        registry.npcs.insert("doc".into(), npc);
        registry.schedules.insert("doc".into(), schedule);
    }

    // ── 8. Mayor Rex ──────────────────────────────────────────────────────────────
    {
        let mut prefs = HashMap::new();
        prefs.insert("cauliflower".into(), GiftPreference::Loved);
        prefs.insert("pumpkin".into(), GiftPreference::Loved);
        prefs.insert("pizza".into(), GiftPreference::Loved);
        prefs.insert("gold_bar".into(), GiftPreference::Loved);
        prefs.insert("diamond".into(), GiftPreference::Loved);
        prefs.insert("bread".into(), GiftPreference::Liked);
        prefs.insert("cheese_omelette".into(), GiftPreference::Liked);
        prefs.insert("melon".into(), GiftPreference::Liked);
        prefs.insert("truffle_risotto".into(), GiftPreference::Liked);
        prefs.insert("yam".into(), GiftPreference::Liked);
        prefs.insert("coal".into(), GiftPreference::Disliked);
        prefs.insert("stone".into(), GiftPreference::Disliked);
        prefs.insert("fiber".into(), GiftPreference::Disliked);
        prefs.insert("slime".into(), GiftPreference::Disliked);
        prefs.insert("carp".into(), GiftPreference::Disliked);
        prefs.insert("sardine".into(), GiftPreference::Hated);

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(0, vec![
            "As mayor, it is my duty to welcome all new residents. Welcome.".into(),
            "This town has potential. Under my leadership, it will bloom.".into(),
            "I have been mayor for fifteen years. The people trust me implicitly.".into(),
            "Hearthfield was ranked third most charming village in the region. Third! We shall not rest until first.".into(),
            "I have a very organized filing system. It has won two local awards.".into(),
        ]);
        heart_dialogue.insert(3, vec![
            "I will admit, you are doing better than I expected from a newcomer.".into(),
            "The town council is very impressed with the farm's progress. Very impressed indeed.".into(),
            "I have drafted a commendation letter for your efforts. It is three paragraphs. I may make it four.".into(),
            "Between you and me, I do enjoy stopping by the farm. There is something calming about it.".into(),
            "A good mayor knows good people when he sees them. I am, in fact, an excellent mayor.".into(),
        ]);
        heart_dialogue.insert(6, vec![
            "Between you and me... I was afraid no one would take the old farm.".into(),
            "I am not always right. I try to be, but... this town needed what you brought.".into(),
            "I wrote the council a private memo about your contributions. I used the word 'extraordinary.' I do not use that word lightly.".into(),
            "My formal manner is... a habit. My father was a mayor. And his father before him. One forgets how to simply talk.".into(),
            "If I had not approved your residency application, this town would be a lesser place. I am glad I approved it.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "I write in my speech that you saved this town. I mean every word.".into(),
            "I would shake your hand, but a hug seems more appropriate. Don't tell anyone.".into(),
            "You have given this town something no mayor can legislate: genuine hope. I am in your debt.".into(),
            "When I retire, I want the town record to say I was mayor when you arrived. That will be enough.".into(),
        ]);

        let npc = NpcDef {
            id: "mayor_rex".into(),
            name: "Mayor Rex".into(),
            birthday_season: Season::Summer,
            birthday_day: 28,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Ah, the farmer! The town thanks you for your service.".into(),
                "I have a very full schedule today. Important mayor business.".into(),
                "Hearthfield is the finest town in the region. I have made it so.".into(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 7,
            portrait_index: 7,
        };

        let schedule = NpcSchedule {
            weekday: vec![
                ScheduleEntry {
                    time: 8.0,
                    map: MapId::Town,
                    x: 28,
                    y: 5,
                },
                ScheduleEntry {
                    time: 10.0,
                    map: MapId::Town,
                    x: 28,
                    y: 8,
                },
                ScheduleEntry {
                    time: 13.0,
                    map: MapId::Town,
                    x: 25,
                    y: 12,
                },
                ScheduleEntry {
                    time: 17.0,
                    map: MapId::Town,
                    x: 28,
                    y: 5,
                },
                ScheduleEntry {
                    time: 20.0,
                    map: MapId::Town,
                    x: 28,
                    y: 5,
                },
            ],
            weekend: vec![
                ScheduleEntry {
                    time: 10.0,
                    map: MapId::Town,
                    x: 15,
                    y: 10,
                },
                ScheduleEntry {
                    time: 14.0,
                    map: MapId::Beach,
                    x: 20,
                    y: 8,
                },
                ScheduleEntry {
                    time: 18.0,
                    map: MapId::Town,
                    x: 28,
                    y: 5,
                },
            ],
            rain_override: Some(vec![
                ScheduleEntry {
                    time: 8.0,
                    map: MapId::Town,
                    x: 28,
                    y: 5,
                },
                ScheduleEntry {
                    time: 20.0,
                    map: MapId::Town,
                    x: 28,
                    y: 5,
                },
            ]),
            festival_override: Some(vec![
                ScheduleEntry {
                    time: 9.0,
                    map: MapId::Town,
                    x: 20,
                    y: 15,
                },
                ScheduleEntry {
                    time: 12.0,
                    map: MapId::Town,
                    x: 18,
                    y: 12,
                },
                ScheduleEntry {
                    time: 22.0,
                    map: MapId::Town,
                    x: 28,
                    y: 5,
                },
            ]),
        };

        registry.npcs.insert("mayor_rex".into(), npc);
        registry.schedules.insert("mayor_rex".into(), schedule);
    }

    // ── 9. Sam ────────────────────────────────────────────────────────────────────
    {
        let mut prefs = HashMap::new();
        prefs.insert("pizza".into(), GiftPreference::Loved);
        prefs.insert("cookie".into(), GiftPreference::Loved);
        prefs.insert("ice_cream".into(), GiftPreference::Loved);
        prefs.insert("spaghetti".into(), GiftPreference::Loved);
        prefs.insert("pancakes".into(), GiftPreference::Loved);
        prefs.insert("bread".into(), GiftPreference::Liked);
        prefs.insert("blueberry".into(), GiftPreference::Liked);
        prefs.insert("strawberry".into(), GiftPreference::Liked);
        prefs.insert("corn".into(), GiftPreference::Liked);
        prefs.insert("fried_egg".into(), GiftPreference::Liked);
        prefs.insert("pumpkin".into(), GiftPreference::Disliked);
        prefs.insert("cauliflower".into(), GiftPreference::Disliked);
        prefs.insert("salad".into(), GiftPreference::Disliked);
        prefs.insert("turnip".into(), GiftPreference::Disliked);
        prefs.insert("yam".into(), GiftPreference::Disliked);
        prefs.insert("eggplant".into(), GiftPreference::Hated);

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(
            0,
            vec![
                "Hey! Do you like music? I'm teaching myself guitar.".into(),
                "This town is kind of boring. No offense. The farm is cool though.".into(),
                "I'm going to be a rock star someday. Just wait and see.".into(),
            ],
        );
        heart_dialogue.insert(3, vec![
            "You're actually pretty cool for an adult. Most adults just tell me to keep it down.".into(),
            "I tried writing riffs while watching the waves at the beach. It sounded better than anything in my room.".into(),
            "If I show you my demo sometime, don't laugh, okay?".into(),
        ]);
        heart_dialogue.insert(
            6,
            vec![
                "When you bring me a gem, I keep it by my amp for luck before I practice.".into(),
                "Some nights I stay up until sunrise trying to nail one song exactly right.".into(),
                "You listening without judging helps more than you know.".into(),
            ],
        );
        heart_dialogue.insert(9, vec![
            "I still dream about leaving town for big stages and loud crowds... I really do.".into(),
            "But when I play out by your farm and the ocean wind kicks in, part of me wants to stay right here.".into(),
            "Maybe growing up is figuring out what to carry with you, no matter where you end up.".into(),
        ]);

        let npc = NpcDef {
            id: "sam".into(),
            name: "Sam".into(),
            birthday_season: Season::Summer,
            birthday_day: 4,
            gift_preferences: prefs,
            default_dialogue: vec![
                "Hey! Did you hear that guitar riff I was working on?".into(),
                "I'm practicing every day. Gonna be a rock star!".into(),
                "This town needs more music. Way more music.".into(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 8,
            portrait_index: 8,
        };

        let schedule = NpcSchedule {
            weekday: vec![
                ScheduleEntry {
                    time: 8.0,
                    map: MapId::Town,
                    x: 10,
                    y: 18,
                },
                ScheduleEntry {
                    time: 10.0,
                    map: MapId::Town,
                    x: 12,
                    y: 18,
                },
                ScheduleEntry {
                    time: 14.0,
                    map: MapId::Beach,
                    x: 10,
                    y: 10,
                },
                ScheduleEntry {
                    time: 18.0,
                    map: MapId::Town,
                    x: 10,
                    y: 18,
                },
                ScheduleEntry {
                    time: 21.0,
                    map: MapId::Town,
                    x: 10,
                    y: 18,
                },
            ],
            weekend: vec![
                ScheduleEntry {
                    time: 9.0,
                    map: MapId::Beach,
                    x: 12,
                    y: 12,
                },
                ScheduleEntry {
                    time: 13.0,
                    map: MapId::Town,
                    x: 14,
                    y: 18,
                },
                ScheduleEntry {
                    time: 16.0,
                    map: MapId::Forest,
                    x: 8,
                    y: 15,
                },
                ScheduleEntry {
                    time: 20.0,
                    map: MapId::Town,
                    x: 10,
                    y: 18,
                },
            ],
            rain_override: Some(vec![
                ScheduleEntry {
                    time: 8.0,
                    map: MapId::Town,
                    x: 10,
                    y: 18,
                },
                ScheduleEntry {
                    time: 21.0,
                    map: MapId::Town,
                    x: 10,
                    y: 18,
                },
            ]),
            festival_override: None,
        };

        registry.npcs.insert("sam".into(), npc);
        registry.schedules.insert("sam".into(), schedule);
    }

    // ── 10. Nora ──────────────────────────────────────────────────────────────────
    {
        let mut prefs = HashMap::new();
        prefs.insert("ancient_fruit".into(), GiftPreference::Loved);
        prefs.insert("pumpkin".into(), GiftPreference::Loved);
        prefs.insert("cauliflower".into(), GiftPreference::Loved);
        prefs.insert("yam".into(), GiftPreference::Loved);
        prefs.insert("melon".into(), GiftPreference::Loved);
        prefs.insert("turnip".into(), GiftPreference::Liked);
        prefs.insert("potato".into(), GiftPreference::Liked);
        prefs.insert("corn".into(), GiftPreference::Liked);
        prefs.insert("hay".into(), GiftPreference::Liked);
        prefs.insert("baked_potato".into(), GiftPreference::Liked);
        prefs.insert("slime".into(), GiftPreference::Disliked);
        prefs.insert("stone".into(), GiftPreference::Disliked);
        prefs.insert("coal".into(), GiftPreference::Disliked);
        prefs.insert("sap".into(), GiftPreference::Disliked);
        prefs.insert("copper_ore".into(), GiftPreference::Disliked);
        prefs.insert("carp".into(), GiftPreference::Hated);

        let mut heart_dialogue: HashMap<u8, Vec<String>> = HashMap::new();
        heart_dialogue.insert(0, vec![
            "Soil health is everything. Compost early, compost often.".into(),
            "I can tell you've got farming in your blood. The old Henderson place is lucky to have you.".into(),
            "My grandfather farmed this valley for forty years. Good land, this.".into(),
        ]);
        heart_dialogue.insert(
            3,
            vec![
                "You've got a good eye now. You notice soil before leaves, like an old hand."
                    .into(),
                "If your cauliflower starts yellowing at the edges, come get me before sundown."
                    .into(),
                "Don't be shy about asking. The land teaches best when we listen together.".into(),
            ],
        );
        heart_dialogue.insert(
            6,
            vec![
                "I worried when that farm sat empty. Too many good fields get forgotten.".into(),
                "You brought life back to it, row by row. That took grit.".into(),
                "I've started setting aside my best seeds for you each season. Feels right.".into(),
            ],
        );
        heart_dialogue.insert(9, vec![
            "My grandfather broke this ground with a mule and a hand plow, and my mother kept it through three drought years.".into(),
            "Every fence post, every orchard row, somebody in my family bled for it.".into(),
            "Seeing you tend this valley with steady hands gives me real peace, child. You're part of its history now.".into(),
        ]);

        let npc = NpcDef {
            id: "nora".into(),
            name: "Nora".into(),
            birthday_season: Season::Fall,
            birthday_day: 7,
            gift_preferences: prefs,
            default_dialogue: vec![
                "The soil is speaking to me today. It wants rain.".into(),
                "Have you rotated your crops properly? Soil gets tired, you know.".into(),
                "Good morning! Best time of day for farming is just after sunrise.".into(),
            ],
            heart_dialogue,
            is_marriageable: false,
            sprite_index: 9,
            portrait_index: 9,
        };

        let schedule = NpcSchedule {
            weekday: vec![
                ScheduleEntry {
                    time: 6.0,
                    map: MapId::Farm,
                    x: 15,
                    y: 14,
                },
                ScheduleEntry {
                    time: 9.0,
                    map: MapId::Town,
                    x: 5,
                    y: 5,
                },
                ScheduleEntry {
                    time: 12.0,
                    map: MapId::Farm,
                    x: 14,
                    y: 13,
                },
                ScheduleEntry {
                    time: 16.0,
                    map: MapId::Town,
                    x: 8,
                    y: 8,
                },
                ScheduleEntry {
                    time: 19.0,
                    map: MapId::Farm,
                    x: 15,
                    y: 14,
                },
            ],
            weekend: vec![
                ScheduleEntry {
                    time: 6.0,
                    map: MapId::Farm,
                    x: 15,
                    y: 14,
                },
                ScheduleEntry {
                    time: 11.0,
                    map: MapId::Town,
                    x: 10,
                    y: 15,
                },
                ScheduleEntry {
                    time: 15.0,
                    map: MapId::Forest,
                    x: 5,
                    y: 5,
                },
                ScheduleEntry {
                    time: 19.0,
                    map: MapId::Farm,
                    x: 15,
                    y: 14,
                },
            ],
            rain_override: Some(vec![
                ScheduleEntry {
                    time: 6.0,
                    map: MapId::Farm,
                    x: 15,
                    y: 14,
                },
                ScheduleEntry {
                    time: 10.0,
                    map: MapId::Town,
                    x: 8,
                    y: 8,
                },
                ScheduleEntry {
                    time: 19.0,
                    map: MapId::Farm,
                    x: 15,
                    y: 14,
                },
            ]),
            festival_override: None,
        };

        registry.npcs.insert("nora".into(), npc);
        registry.schedules.insert("nora".into(), schedule);
    }
}

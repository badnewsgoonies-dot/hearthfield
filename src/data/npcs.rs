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
            "These biscuits won't shape themselves, dear. But I always have time for a chat.".into(),
            "I've been up since four o'clock. The bread needs me early.".into(),
            "That silly cat knocked over a whole tray this morning. Flour everywhere. I laughed eventually.".into(),
        ]);
        heart_dialogue.insert(3, vec![
            "You know, not many young folks visit me these days. It means a lot.".into(),
            "I've been thinking of trying a new recipe. Maybe you could taste-test it?".into(),
            "Biscuit seems to like you. That cat doesn't warm up to just anybody.".into(),
            "Come by the bakery sometime when it's quiet. I'll teach you a thing or two about pastry.".into(),
            "The farm smells wonderful lately. Fresh earth and something blooming — it drifts all the way to my window.".into(),
            "I noticed you've been working past dark lately. Let me wrap up a little something for the walk home.".into(),
            "The other day I caught myself making an extra loaf. Just in case you stopped by. It was a good instinct.".into(),
            "You remind me to slow down and enjoy the work. I forget that sometimes, elbow-deep in dough before sunrise.".into(),
        ]);
        heart_dialogue.insert(6, vec![
            "You remind me of my daughter when she was your age. She left for the city...".into(),
            "Here, take this recipe. It was my mother's. I want someone to carry it on.".into(),
            "I haven't told many people this, but I almost left Hearthfield too, years ago. Something kept me here.".into(),
            "Biscuit sleeps on the recipe box. I think she's guarding the good ones for me.".into(),
            "My husband used to say that a good loaf of bread is an act of love. I believe that more every year.".into(),
            "There's a photo of my daughter somewhere in the kitchen drawer. I should dig it out.".into(),
            "The recipe I gave you — my mother cried the first time she made it. Good tears.".into(),
            "I sleep better knowing the old farm has someone in it. Silly, maybe. But true.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "This town is lucky to have someone like you looking after the old farm.".into(),
            "Promise me you'll never let this place become what it was before you came.".into(),
            "I've started saving the corner piece of every pie for you. Don't tell the other customers.".into(),
            "You've given this old bakery more life than it's seen in years. I can feel it in my morning dough.".into(),
            "You've made the bakery feel less lonely. I didn't realize how much I needed that.".into(),
            "When I imagine the future of this town, you're standing in the middle of it, smiling.".into(),
            "Don't ever change, dear. Not one thing.".into(),
            "I made a special butter cookie this morning and wrote your name on it. With icing. Don't laugh.".into(),
            "Every loaf I shape these days, I think of you. That's not loneliness. That's gratitude.".into(),
            "If I could bottle the feeling of this bakery on a good day, I'd hand it to you first.".into(),
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
            "Every morning I choose the music of the kitchen — the sizzle, the chop, the steam.".into(),
            "My nonna said: 'If it tastes good, you cooked with your heart.' I never forgot that.".into(),
            "I dream about ingredients sometimes. Ripe tomatoes and fresh basil. Wonderful dreams.".into(),
        ]);
        heart_dialogue.insert(3, vec![
            "You have good taste, my friend. In food and in company.".into(),
            "Perhaps one day I teach you my secret tomato sauce, eh?".into(),
            "Not everyone appreciates real cooking. But you — you eat with your whole heart.".into(),
            "I notice you always finish the plate. A chef notices these things. It makes us happy.".into(),
            "The herbs in my garden remind me of my mother's kitchen window. Simple things bring the most comfort.".into(),
            "Come, sit at the counter. I am testing a new sauce and you are the only one I trust.".into(),
            "My brother back home wants to visit someday. He says I talk about you too much in my letters.".into(),
            "The way you eat — slowly, thoughtfully — it tells me you truly appreciate what is in front of you.".into(),
        ]);
        heart_dialogue.insert(6, vec![
            "You remind me of my little brother back home. I miss him terribly.".into(),
            "Here, I have written down the recipe. Guard it well.".into(),
            "Sometimes at closing time the restaurant is so quiet it hurts. I did not expect to feel lonely here.".into(),
            "You are the kind of person my nonna would have adopted on the spot. She had good instincts.".into(),
            "Cooking for one person who truly appreciates it is better than cooking for a hundred who do not.".into(),
            "When I cook for someone who cares, the food always comes out better. This is my scientific conclusion.".into(),
            "I have decided: you are the finest customer this trattoria has ever had. I am announcing this officially.".into(),
            "Last night I dreamed of my nonna's kitchen. You were there too, somehow. She would have loved you.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "You are family now. And family always eats well!".into(),
            "Perhaps I open a second restaurant when the farm is doing well, no?".into(),
            "I write home and tell my brother about you. He says you sound like a good soul.".into(),
            "Every evening I save the best seat by the window. It is yours whenever you want it.".into(),
            "I wrote a new dish and named it after you. It is on the menu. Ask for 'The Farmer's Heart.'".into(),
            "In all my travels and all my years of cooking, I have never felt this at peace. You did that.".into(),
            "Mi amico — my friend — this restaurant is home because you are here.".into(),
            "I keep your favorite table set even on my day off. Old habit now.".into(),
            "When I close up late and the kitchen is quiet, I feel good knowing you are nearby.".into(),
            "Cooking for someone who truly tastes the love in it — that is my greatest reward.".into(),
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
            "I've been up since sunrise wandering the field. I find it easier to think out there.".into(),
            "Have you ever noticed how a flower faces the sun all day? I think I understand that feeling.".into(),
            "You should see the rose beds behind the shop in the morning light. They practically glow.".into(),
        ]);
        heart_dialogue.insert(3, vec![
            "I don't open up to many people, but you seem different somehow.".into(),
            "I've been pressing flowers into a journal. Would you like to see it sometime?".into(),
            "I tried growing vegetables once, just a little patch. Turns out I have a lot to learn about your side of farming.".into(),
            "I like that you stop to notice the small things — a single bloom, a bent stem. Not everyone does.".into(),
            "Do you have a favorite flower? I keep meaning to ask.".into(),
            "I saved a pressed flower for you. It's from the first bloom of the season. I was going to keep it, but...".into(),
            "You walk past the shop every morning. I pretend not to notice. I kind of do, though.".into(),
            "I practiced explaining why I love flowers and got completely flustered. In front of nobody. Just... myself.".into(),
        ]);
        heart_dialogue.insert(6, vec![
            "I... I find myself looking forward to our conversations more than I expected.".into(),
            "My dream is to have a garden that spans all four seasons. Maybe we could plan it together?".into(),
            "I stayed up all night once drawing garden layouts. I've never shown anyone. Maybe I'll show you.".into(),
            "There's a corner of the flower shop I keep just for me — no customers allowed. I think I'd let you in.".into(),
            "When a flower blooms that I've been tending for months... I think of you. I'm not sure why.".into(),
            "I think I'm happiest when you're nearby. I noticed that recently and then thought about it way too much.".into(),
            "I've been taking the long way past your farm. I tell myself it's for wildflower research.".into(),
            "Last night I dreamt we built the all-season garden together. I woke up smiling before I even remembered why.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "You've changed something in this town. I think it's hope. We all needed that.".into(),
            "I wrote you something. It's a little embarrassing, but... please read it?".into(),
            "I planted a whole new section of the garden, just for the two of us. It blooms in every season.".into(),
            "When I imagine next spring, I imagine it with you. That's a new feeling, and I really like it.".into(),
            "You make the whole world feel brighter — not just spring.".into(),
            "I picked this for you. It's the rarest one in my garden. That felt right.".into(),
            "Wherever this leads... I want it to keep going. With you.".into(),
            "I pressed a flower for you today. It's the prettiest one I've ever grown. I want you to have it forever.".into(),
            "Sometimes I catch myself smiling for no reason. Then I realize I was thinking about you.".into(),
            "Every bouquet I make now has a little secret — one bloom that reminds me of you.".into(),
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
            "The river talks to you if you're quiet long enough. Most folks aren't quiet long enough.".into(),
            "Forty years I've fished this beach. Memorized every current, every tide.".into(),
            "Wind from the north means trout near the surface. That's free advice. Don't thank me.".into(),
        ]);
        heart_dialogue.insert(3, vec![
            "...You're alright, I suppose. For a newcomer.".into(),
            "My old fishing spot — the one past the bridge. Nobody knows about it but me. And now you.".into(),
            "You're learning fast. Not many newcomers bother to learn the water.".into(),
            "Keep your hook baited and your mouth shut. Those are the two rules of good fishing.".into(),
            "The fish have been biting earlier lately. You'd know that if you came out at sunrise more often.".into(),
            "I've been leaving the dock light on later than usual. More company that way.".into(),
            "Don't tell anyone, but I left the best spot on the other side of the cove open for you.".into(),
            "You fish with patience. That's rarer than you think among young people.".into(),
        ]);
        heart_dialogue.insert(6, vec![
            "I don't talk about my wife often. But she loved this river too.".into(),
            "Here. My fishing rod is better than yours. Take it.".into(),
            "She used to sit right there on that rock and read while I fished. Never bothered me once. That's rare.".into(),
            "I come here to remember her mostly. The fish are just an excuse.".into(),
            "You remind me of her a bit. She had that same look — like you're really paying attention.".into(),
            "She had a name for the river: 'Old Slow.' Said it matched me perfectly. She wasn't wrong.".into(),
            "I mended her favorite line last winter. Still use it. Silly, maybe. But it feels right.".into(),
            "You listen the way she did — like the words matter. That's not nothing.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "You've made this old town worth waking up for again. Don't tell anyone I said that.".into(),
            "If you catch the Legend, you bring it to me first. That's the deal.".into(),
            "I carved your name into my fishing post. Right under hers. Means something, that.".into(),
            "You're the only one who ever listened to my old stories without walking away. That counts for more than you know.".into(),
            "I don't say this much, but: thanks for not giving up on this old man.".into(),
            "When you come by the dock, the day gets easier. That's worth saying out loud.".into(),
            "If I ever catch the Legend, I want you there when I pull it in. That's the only condition.".into(),
            "My wife would have loved you. I say that with certainty, and I don't say much with certainty.".into(),
            "I tied a new lure last night. Named it after you. Catches more than any other.".into(),
            "This dock has been lonelier than I'd admit. Not anymore though. Not anymore.".into(),
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
            "Precision is respect. Every tool I make is a promise it won't fail you when it matters.".into(),
            "People come in assuming cheaper is better. They leave understanding why it isn't.".into(),
            "I learned to sharpen before I learned to read. Father started early.".into(),
        ]);
        heart_dialogue.insert(3, vec![
            "Most people just want the cheapest option. You seem to care about quality. Refreshing.".into(),
            "I'm working on something new. A special alloy. Father doesn't know yet.".into(),
            "You've been using your tools hard. I can tell by the wear pattern. That means you're doing real work.".into(),
            "The mine has good ore in the lower floors, if you're not afraid of the dark.".into(),
            "I don't have many friends, honestly. The forge keeps me busy. But I don't mind talking to you.".into(),
            "I noticed your hoe's edge was off. I fixed it while you were browsing. Don't mention it.".into(),
            "Most people don't appreciate how much thought goes into a properly balanced blade.".into(),
            "You always come in knowing exactly what you need. That's not as common as it sounds.".into(),
        ]);
        heart_dialogue.insert(6, vec![
            "I've been doing this my whole life, but sometimes I wonder if there's more out there.".into(),
            "You can watch me work, if you want. But don't talk while I'm concentrating.".into(),
            "Father wants me to take over the shop someday. I think I want that too... mostly.".into(),
            "Metal remembers the hand that shaped it. I like to think what I make carries something of me forward.".into(),
            "I showed you something I've never shown anyone — that alloy project. I think that means something.".into(),
            "I'm not good at saying things clearly when they aren't about metalwork. But I'm trying.".into(),
            "Sometimes I forge a piece and keep it, not for sale. I made one thinking of you. It's still in the shop.".into(),
            "I trust almost no one. You've made me rethink that. Slowly. But still.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "I made something for you. It's not sentimental. It's just good craftsmanship.".into(),
            "You're the only person who's ever understood why this work matters. That means something.".into(),
            "I put a maker's mark on what I made you. My own mark. I've never given that to anyone before.".into(),
            "When you're out on the farm, I think about whether my tools are holding up. I want them to take care of you.".into(),
            "I don't know how to say this without sounding formal: I think about you more than I should.".into(),
            "The forge feels different on days you don't stop by. Quieter. Not the good kind.".into(),
            "If you ever stopped coming around... I'd notice. More than I'd expect. Much more.".into(),
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
            "I collect rare things — objects, stories, moments. Some places are stories themselves.".into(),
            "Hearthfield feels like a place that kept its secrets close. I respect that.".into(),
            "I never stay anywhere long, but I find myself making longer plans here.".into(),
        ]);
        heart_dialogue.insert(3, vec![
            "You have a good eye. Rare in a farmer. Rarer still in anyone.".into(),
            "I have something I've been saving for the right person. Perhaps that's you.".into(),
            "The Eastern desert markets taught me to read people fast. You've been an interesting puzzle.".into(),
            "Not many people ask where my wares come from. You always do. I appreciate that.".into(),
            "I've met travelers who were chasing something for years without knowing what. I think I understand that feeling now.".into(),
            "You ask the right questions. In my experience, that is rarer than gemstones.".into(),
            "I've brought something back from the Eastern markets just for you. No particular reason.".into(),
            "I've met hundreds of merchants and thousands of customers. Fewer true listeners.".into(),
        ]);
        heart_dialogue.insert(6, vec![
            "I ran from a place that didn't value what I valued. This town might be different.".into(),
            "You've given me a reason to stop running, I think.".into(),
            "Every place I've settled, I kept one eye on the road. I notice I'm not doing that here.".into(),
            "I keep a journal of the best markets I've seen. This town has made it onto the list for unexpected reasons.".into(),
            "Somewhere between the first sale and the hundredth conversation, Hearthfield became... home. Strange word for me.".into(),
            "I told my old caravan companion I was staying. He laughed. I laughed. Then I didn't.".into(),
            "The road has no memory of you. Home does. I'm learning the difference.".into(),
            "I think the thing I was chasing across four continents... was this. Quiet. Real.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "Home is where the people understand you. I think I've found mine.".into(),
            "Whatever you need — knowledge, goods, passage — I will find a way.".into(),
            "I sent a letter to an old caravan friend. Told them I'd stopped moving. They didn't believe me. Neither did I.".into(),
            "You are the reason I unpacked the last chest. The one I always kept ready to go.".into(),
            "I have never given anyone my route map. It's the only thing I own that is purely mine. I want to give it to you.".into(),
            "You understand something about me that I barely understand about myself. That is worth more than anything I've sold.".into(),
            "Wherever this town goes, I go too. That is the first promise I've made in fifteen years.".into(),
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
            "Small towns are actually quite complex medically. More so than cities, sometimes.".into(),
            "I started keeping a garden of medicinal herbs out back. Feels more like me than the clinic does.".into(),
            "The thing they don't tell you in medical school: listening is half the treatment.".into(),
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
                "Most people only come to the clinic when something's wrong. You stop by just to check in. That means a lot.".into(),
                "I've been adapting my treatment plans to the farming schedule. Farmers heal on their own timeline.".into(),
                "My notebook is getting quite long. The good kind of long, I mean.".into(),
            ],
        );
        heart_dialogue.insert(6, vec![
            "When I came here, I thought I was running away from medicine. Now I realize I needed a different kind.".into(),
            "Let me check on you. Farmer's body takes a beating. Better safe than sorry.".into(),
            "I burned out in the city. Too many patients, not enough names. Here, I know everyone's name.".into(),
            "You remind me why I became a doctor in the first place. Not for the clinic. For the people.".into(),
            "I keep a little notebook of everyone in town — health notes, mostly, but also favorite things. You've got your own page.".into(),
            "You're the first person who's made me feel less like a doctor here and more like a neighbor.".into(),
            "I've started leaving an extra chair in the clinic. Just in case you stop by.".into(),
            "Burnout in the city taught me what I didn't want. You're teaching me what I do.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "I've seen this town healed by you. That's not a small thing. That's everything.".into(),
            "You ever need anything — anything at all — you come find me first. That's a doctor's promise.".into(),
            "I think I'll stay in Hearthfield forever. I didn't know that until very recently.".into(),
            "Whatever happens — storms, drought, hard years — you won't face it alone. That's not medicine. That's just true.".into(),
            "There is no clinic big enough for the gratitude I feel when I see what you've built here.".into(),
            "I've prescribed rest before. Never quite wanted to sit through a rest with someone before.".into(),
            "You've given me back the reason I became a doctor. That is not a small thing. That is everything.".into(),
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
            "The council has given me full authority over matters of civic beautification. I use it wisely.".into(),
            "I keep a compliment file. Do not ask how many entries it has. The answer is: many.".into(),
            "Town governance is a calling. Most people don't understand that. I forgive them.".into(),
        ]);
        heart_dialogue.insert(3, vec![
            "I will admit, you are doing better than I expected from a newcomer.".into(),
            "The town council is very impressed with the farm's progress. Very impressed indeed.".into(),
            "I have drafted a commendation letter for your efforts. It is three paragraphs. I may make it four.".into(),
            "Between you and me, I do enjoy stopping by the farm. There is something calming about it.".into(),
            "A good mayor knows good people when he sees them. I am, in fact, an excellent mayor.".into(),
            "I told the council you were 'a notable addition to the community.' Very rare praise from me.".into(),
            "I don't normally share my strolling route, but since we're friendly: the east path in spring is magnificent.".into(),
            "I have framed a photograph of the town on my desk. I have decided your farm is the best part.".into(),
        ]);
        heart_dialogue.insert(6, vec![
            "Between you and me... I was afraid no one would take the old farm.".into(),
            "I am not always right. I try to be, but... this town needed what you brought.".into(),
            "I wrote the council a private memo about your contributions. I used the word 'extraordinary.' I do not use that word lightly.".into(),
            "My formal manner is... a habit. My father was a mayor. And his father before him. One forgets how to simply talk.".into(),
            "If I had not approved your residency application, this town would be a lesser place. I am glad I approved it.".into(),
            "My public face and my private face are quite different. I find it easier to be myself near you.".into(),
            "I wrote a haiku about your farm. It's in my private journal. It is not for public release.".into(),
            "Between you and me, the filing awards were self-nominated. Please take that to the grave.".into(),
        ]);
        heart_dialogue.insert(9, vec![
            "I write in my speech that you saved this town. I mean every word.".into(),
            "I would shake your hand, but a hug seems more appropriate. Don't tell anyone.".into(),
            "You have given this town something no mayor can legislate: genuine hope. I am in your debt.".into(),
            "When I retire, I want the town record to say I was mayor when you arrived. That will be enough.".into(),
            "I have spoken publicly hundreds of times. But right now I find myself without a prepared speech.".into(),
            "I am placing you on the town's honor roll. I am also placing you on my personal one.".into(),
            "If there were a mayor for the heart, I would vote for you unopposed.".into(),
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
                "I played until 2am last night. My fingers are destroyed but totally worth it."
                    .into(),
                "I write the best songs when I'm hungry. Don't know why. That's just how it works."
                    .into(),
                "The guitar used to feel clunky. Now it feels like part of me. Weird, right?"
                    .into(),
                "I made a playlist of all the songs I want to cover. It's longer than I expected."
                    .into(),
                "Every great musician started somewhere small. This town is my somewhere small."
                    .into(),
            ],
        );
        heart_dialogue.insert(3, vec![
            "You're actually pretty cool for an adult. Most adults just tell me to keep it down.".into(),
            "I tried writing riffs while watching the waves at the beach. It sounded better than anything in my room.".into(),
            "If I show you my demo sometime, don't laugh, okay?".into(),
            "Okay, real talk — you actually listened to that whole recording? Like, start to finish?".into(),
            "I'm not usually this open with people. Something about you just makes it easy.".into(),
            "Mom says I should study something practical. I told her practical is boring. She didn't agree.".into(),
            "Most people tune out when I talk about music. You ask follow-up questions. That's rare.".into(),
            "I've been practicing twice as hard lately. I want the next demo to actually be good.".into(),
        ]);
        heart_dialogue.insert(
            6,
            vec![
                "When you bring me a gem, I keep it by my amp for luck before I practice.".into(),
                "Some nights I stay up until sunrise trying to nail one song exactly right.".into(),
                "You listening without judging helps more than you know.".into(),
                "I wrote a song about this town and there's a part that's basically about you. Don't make it weird.".into(),
                "You never tell me I should get a real job. You have no idea what that means to me.".into(),
                "Playing music alone is fine. Playing it for someone who actually gets it is completely different.".into(),
                "I keep a notebook of lyrics. I showed you a page. Nobody's seen a page before.".into(),
                "I play my best when I know you might walk by and hear it.".into(),
            ],
        );
        heart_dialogue.insert(9, vec![
            "I still dream about leaving town for big stages and loud crowds... I really do.".into(),
            "But when I play out by your farm and the ocean wind kicks in, part of me wants to stay right here.".into(),
            "Maybe growing up is figuring out what to carry with you, no matter where you end up.".into(),
            "Whatever happens with music and fame and all that — you're part of why I keep trying.".into(),
            "I think the people you come back to are more important than the places you go. Took me a while to figure that out.".into(),
            "You believed in this before it sounded like anything. That's the best thing someone can do for a musician.".into(),
            "If I ever make it big, the first song on the album is dedicated to Hearthfield. You'll know why.".into(),
            "Wherever I end up, I'm writing songs about this place. About you, a little bit. Don't tell anyone.".into(),
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
            "Don't harvest in a hurry. The crop always knows.".into(),
            "Fence the garden before you plant it. Rabbits are patient and you're not.".into(),
            "First year on a new farm is always the hardest. Keep at it.".into(),
            "Water in the morning, not the evening. Gives the roots time to drink before heat.".into(),
            "A good farmer reads the sky before the almanac. Start learning the clouds.".into(),
        ]);
        heart_dialogue.insert(
            3,
            vec![
                "You've got a good eye now. You notice soil before leaves, like an old hand.".into(),
                "If your cauliflower starts yellowing at the edges, come get me before sundown.".into(),
                "Don't be shy about asking. The land teaches best when we listen together.".into(),
                "Those rows are straight. Good eye for alignment — that matters more than most think.".into(),
                "I've been watching you work. You don't rush. That's rare and very good.".into(),
                "You've got a feel for timing — when to plant, when to wait. Not everyone gets that so fast.".into(),
                "Mulch those beds before frost. You've got the instinct; now add the habit.".into(),
                "You asked about crop rotation last week. You're thinking like a real farmer now.".into(),
            ],
        );
        heart_dialogue.insert(
            6,
            vec![
                "I worried when that farm sat empty. Too many good fields get forgotten.".into(),
                "You brought life back to it, row by row. That took grit.".into(),
                "I've started setting aside my best seeds for you each season. Feels right.".into(),
                "I pulled out the old map of the valley from when my grandfather surveyed it. Your fields match almost exactly.".into(),
                "I don't hand out compliments like coin, but you've earned your place on this land.".into(),
                "The old hands around here talk about you. They mean it as a compliment. That's not nothing.".into(),
                "Seeing what you've done with that soil in a single season — that's years of learning compressed.".into(),
                "I've started walking your fence line in the morning. I tell myself it's habit. It's also just good watching.".into(),
            ],
        );
        heart_dialogue.insert(9, vec![
            "My grandfather broke this ground with a mule and a hand plow, and my mother kept it through three drought years.".into(),
            "Every fence post, every orchard row, somebody in my family bled for it.".into(),
            "Seeing you tend this valley with steady hands gives me real peace, child. You're part of its history now.".into(),
            "You farm with the kind of respect people used to — when the land was still sacred to them.".into(),
            "My grandmother would have recognized you as kin. That is the highest thing I can say.".into(),
            "The valley will remember what you built here. Good land holds memory. So do good farmers.".into(),
            "I've stopped worrying about this land. That's the first time in twenty years. You did that.".into(),
            "When I'm gone, I want someone to tend these fields the right way. I think I've found them.".into(),
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
                    x: 5,
                    y: 18,
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
                    x: 6,
                    y: 20,
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
                    x: 5,
                    y: 18,
                },
            ],
            weekend: vec![
                ScheduleEntry {
                    time: 6.0,
                    map: MapId::Farm,
                    x: 5,
                    y: 18,
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
                    x: 5,
                    y: 18,
                },
            ],
            rain_override: Some(vec![
                ScheduleEntry {
                    time: 6.0,
                    map: MapId::Farm,
                    x: 5,
                    y: 18,
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
                    x: 5,
                    y: 18,
                },
            ]),
            festival_override: None,
        };

        registry.npcs.insert("nora".into(), npc);
        registry.schedules.insert("nora".into(), schedule);
    }
}

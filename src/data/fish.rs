use crate::shared::*;

/// Populate the FishRegistry with all 20 fish species.
///
/// Each fish has:
///   - location: where in the world it can be caught
///   - seasons: which seasons it appears in
///   - time_range: hours (6.0 = 6AM, 20.0 = 8PM)
///   - weather_required: Some(weather) = only in that weather, None = any
///   - rarity: affects spawn weight during fishing minigame
///   - difficulty: 0.0 = trivial, 1.0 = legendary (determines bar speed in minigame)
pub fn populate_fish(registry: &mut FishRegistry) {
    let fish: Vec<FishDef> = vec![
        // ── Common Ocean Fish ─────────────────────────────────────────────────────

        FishDef {
            id: "sardine".into(),
            name: "Sardine".into(),
            location: FishLocation::Ocean,
            seasons: vec![Season::Spring, Season::Fall, Season::Winter],
            time_range: (6.0, 19.0), // 6AM to 7PM
            weather_required: None,
            rarity: Rarity::Common,
            difficulty: 0.15,
            sell_price: 40,
            sprite_index: 50,
        },

        FishDef {
            id: "herring".into(),
            name: "Herring".into(),
            location: FishLocation::Ocean,
            seasons: vec![Season::Spring, Season::Winter],
            time_range: (6.0, 20.0), // 6AM to 8PM
            weather_required: None,
            rarity: Rarity::Common,
            difficulty: 0.12,
            sell_price: 30,
            sprite_index: 51,
        },

        // ── River Fish ────────────────────────────────────────────────────────────

        FishDef {
            id: "bass".into(),
            name: "Bass".into(),
            location: FishLocation::River,
            seasons: vec![Season::Spring, Season::Summer, Season::Fall],
            time_range: (6.0, 19.0),
            weather_required: None,
            rarity: Rarity::Common,
            difficulty: 0.40,
            sell_price: 100,
            sprite_index: 52,
        },

        FishDef {
            id: "trout".into(),
            name: "Trout".into(),
            location: FishLocation::River,
            seasons: vec![Season::Summer, Season::Fall],
            time_range: (6.0, 20.0),
            weather_required: None,
            rarity: Rarity::Common,
            difficulty: 0.30,
            sell_price: 65,
            sprite_index: 53,
        },

        FishDef {
            id: "salmon".into(),
            name: "Salmon".into(),
            location: FishLocation::River,
            seasons: vec![Season::Fall],
            time_range: (6.0, 19.0),
            weather_required: None,
            rarity: Rarity::Uncommon,
            difficulty: 0.50,
            sell_price: 150,
            sprite_index: 54,
        },

        FishDef {
            id: "catfish".into(),
            name: "Catfish".into(),
            location: FishLocation::River,
            seasons: vec![Season::Spring, Season::Fall],
            time_range: (6.0, 24.0), // 6AM to midnight
            weather_required: Some(Weather::Rainy),
            rarity: Rarity::Uncommon,
            difficulty: 0.55,
            sell_price: 200,
            sprite_index: 55,
        },

        // ── Pond Fish ─────────────────────────────────────────────────────────────

        FishDef {
            id: "carp".into(),
            name: "Carp".into(),
            location: FishLocation::Pond,
            // Available all seasons — the humble carp
            seasons: vec![Season::Spring, Season::Summer, Season::Fall, Season::Winter],
            time_range: (6.0, 20.0),
            weather_required: None,
            rarity: Rarity::Common,
            difficulty: 0.10,
            sell_price: 30,
            sprite_index: 56,
        },

        FishDef {
            id: "pike".into(),
            name: "Pike".into(),
            location: FishLocation::River,
            seasons: vec![Season::Summer, Season::Winter],
            time_range: (6.0, 22.0),
            weather_required: None,
            rarity: Rarity::Uncommon,
            difficulty: 0.60,
            sell_price: 100,
            sprite_index: 57,
        },

        FishDef {
            id: "perch".into(),
            name: "Perch".into(),
            location: FishLocation::River,
            seasons: vec![Season::Winter],
            time_range: (6.0, 20.0),
            weather_required: None,
            rarity: Rarity::Common,
            difficulty: 0.20,
            sell_price: 55,
            sprite_index: 58,
        },

        FishDef {
            id: "eel".into(),
            name: "Eel".into(),
            location: FishLocation::Ocean,
            seasons: vec![Season::Spring, Season::Fall],
            time_range: (16.0, 2.0), // 4PM to 2AM (night fish)
            weather_required: Some(Weather::Rainy),
            rarity: Rarity::Uncommon,
            difficulty: 0.70,
            sell_price: 85,
            sprite_index: 59,
        },

        // ── Ocean Fish ────────────────────────────────────────────────────────────

        FishDef {
            id: "tuna".into(),
            name: "Tuna".into(),
            location: FishLocation::Ocean,
            seasons: vec![Season::Summer, Season::Winter],
            time_range: (6.0, 19.0),
            weather_required: None,
            rarity: Rarity::Uncommon,
            difficulty: 0.55,
            sell_price: 275,
            sprite_index: 60,
        },

        FishDef {
            id: "swordfish".into(),
            name: "Swordfish".into(),
            location: FishLocation::Ocean,
            seasons: vec![Season::Summer],
            time_range: (6.0, 19.0),
            weather_required: None,
            rarity: Rarity::Rare,
            difficulty: 0.75,
            sell_price: 350,
            sprite_index: 61,
        },

        FishDef {
            id: "sturgeon".into(),
            name: "Sturgeon".into(),
            location: FishLocation::River,
            seasons: vec![Season::Summer, Season::Winter],
            time_range: (6.0, 19.0),
            weather_required: None,
            rarity: Rarity::Rare,
            difficulty: 0.78,
            sell_price: 200,
            sprite_index: 62,
        },

        FishDef {
            id: "pufferfish".into(),
            name: "Pufferfish".into(),
            location: FishLocation::Ocean,
            seasons: vec![Season::Summer],
            time_range: (6.0, 16.0), // 6AM to 4PM
            weather_required: Some(Weather::Sunny),
            rarity: Rarity::Uncommon,
            difficulty: 0.50,
            sell_price: 200,
            sprite_index: 63,
        },

        FishDef {
            id: "octopus".into(),
            name: "Octopus".into(),
            location: FishLocation::Ocean,
            seasons: vec![Season::Summer],
            time_range: (6.0, 13.0), // 6AM to 1PM
            weather_required: None,
            rarity: Rarity::Rare,
            difficulty: 0.65,
            sell_price: 150,
            sprite_index: 64,
        },

        FishDef {
            id: "squid".into(),
            name: "Squid".into(),
            location: FishLocation::Ocean,
            seasons: vec![Season::Winter],
            time_range: (18.0, 2.0), // 6PM to 2AM
            weather_required: None,
            rarity: Rarity::Common,
            difficulty: 0.30,
            sell_price: 80,
            sprite_index: 65,
        },

        // ── Legendary & Rare Fish ──────────────────────────────────────────────────

        FishDef {
            id: "anglerfish".into(),
            name: "Anglerfish".into(),
            location: FishLocation::Ocean,
            seasons: vec![Season::Fall, Season::Winter],
            time_range: (18.0, 2.0), // Nocturnal deep-sea fish
            weather_required: None,
            rarity: Rarity::Rare,
            difficulty: 0.80,
            sell_price: 900,
            sprite_index: 66,
        },

        FishDef {
            id: "legend_fish".into(),
            name: "Legend".into(),
            location: FishLocation::River,
            seasons: vec![Season::Spring],
            time_range: (6.0, 20.0),
            weather_required: Some(Weather::Rainy),
            rarity: Rarity::Legendary,
            difficulty: 0.95,
            sell_price: 5000,
            sprite_index: 67,
        },

        FishDef {
            id: "glacier_fish".into(),
            name: "Glacier Fish".into(),
            location: FishLocation::Ocean,
            seasons: vec![Season::Winter],
            time_range: (6.0, 20.0),
            weather_required: None,
            rarity: Rarity::Legendary,
            difficulty: 0.90,
            sell_price: 1000,
            sprite_index: 68,
        },

        FishDef {
            id: "crimson_fish".into(),
            name: "Crimsonfish".into(),
            location: FishLocation::Ocean,
            seasons: vec![Season::Summer],
            time_range: (6.0, 20.0),
            weather_required: None,
            rarity: Rarity::Legendary,
            difficulty: 0.88,
            sell_price: 1500,
            sprite_index: 69,
        },
    ];

    for f in fish {
        registry.fish.insert(f.id.clone(), f);
    }
}

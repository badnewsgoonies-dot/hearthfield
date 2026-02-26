use crate::shared::*;

/// Populate the CropRegistry with all crop definitions.
///
/// Growth days from GAME_SPEC.md:
///   Spring: turnip (4d), potato (6d), cauliflower (12d), strawberry (8d, regrows 4d)
///   Summer: melon (12d), tomato (11d, regrows 4d), blueberry (13d, regrows 4d), corn (14d)
///   Fall:   eggplant (13d, regrows 5d), pumpkin (13d), cranberry (7d, regrows 5d), yam (10d)
///   Any:    wheat (4d), coffee (10d, regrows 2d), ancient_fruit (28d, regrows 7d)
///
/// Each crop has multiple growth stages. The `growth_days` vec stores how many
/// days to spend in each stage before advancing. The last element is the
/// "mature" stage — harvest is available from this point on.
pub fn populate_crops(registry: &mut CropRegistry) {
    let crops: Vec<CropDef> = vec![
        // ── Spring Crops ────────────────────────────────────────────────────────

        CropDef {
            id: "turnip".into(),
            name: "Turnip".into(),
            seed_id: "turnip_seeds".into(),
            harvest_id: "turnip".into(),
            seasons: vec![Season::Spring],
            // 4 total days, 4 stages (1 day each)
            growth_days: vec![1, 1, 1, 1],
            regrows: false,
            regrow_days: 0,
            sell_price: 60,
            // Sprite stage indices in the crop atlas: 4 growth frames + 1 harvest frame
            sprite_stages: vec![0, 1, 2, 3],
        },

        CropDef {
            id: "potato".into(),
            name: "Potato".into(),
            seed_id: "potato_seeds".into(),
            harvest_id: "potato".into(),
            seasons: vec![Season::Spring],
            // 6 total days, 5 stages
            growth_days: vec![1, 2, 2, 1],
            regrows: false,
            regrow_days: 0,
            sell_price: 80,
            sprite_stages: vec![10, 11, 12, 13],
        },

        CropDef {
            id: "cauliflower".into(),
            name: "Cauliflower".into(),
            seed_id: "cauliflower_seeds".into(),
            harvest_id: "cauliflower".into(),
            seasons: vec![Season::Spring],
            // 12 total days, 5 stages
            growth_days: vec![1, 2, 4, 4, 1],
            regrows: false,
            regrow_days: 0,
            sell_price: 175,
            sprite_stages: vec![20, 21, 22, 23, 24],
        },

        CropDef {
            id: "strawberry".into(),
            name: "Strawberry".into(),
            seed_id: "strawberry_seeds".into(),
            harvest_id: "strawberry".into(),
            seasons: vec![Season::Spring],
            // 8 days to first harvest, 4 stages
            growth_days: vec![1, 1, 2, 4],
            regrows: true,
            regrow_days: 4,
            sell_price: 120,
            sprite_stages: vec![30, 31, 32, 33],
        },

        // ── Summer Crops ────────────────────────────────────────────────────────

        CropDef {
            id: "melon".into(),
            name: "Melon".into(),
            seed_id: "melon_seeds".into(),
            harvest_id: "melon".into(),
            seasons: vec![Season::Summer],
            // 12 total days, 5 stages
            growth_days: vec![1, 2, 3, 4, 2],
            regrows: false,
            regrow_days: 0,
            sell_price: 250,
            sprite_stages: vec![40, 41, 42, 43, 44],
        },

        CropDef {
            id: "tomato".into(),
            name: "Tomato".into(),
            seed_id: "tomato_seeds".into(),
            harvest_id: "tomato".into(),
            seasons: vec![Season::Summer],
            // 11 days to first harvest, 5 stages
            growth_days: vec![1, 2, 3, 4, 1],
            regrows: true,
            regrow_days: 4,
            sell_price: 60,
            sprite_stages: vec![50, 51, 52, 53, 54],
        },

        CropDef {
            id: "blueberry".into(),
            name: "Blueberry".into(),
            seed_id: "blueberry_seeds".into(),
            harvest_id: "blueberry".into(),
            seasons: vec![Season::Summer],
            // 13 days to first harvest, 5 stages
            growth_days: vec![1, 3, 3, 4, 2],
            regrows: true,
            regrow_days: 4,
            sell_price: 50,
            sprite_stages: vec![60, 61, 62, 63, 64],
        },

        CropDef {
            id: "corn".into(),
            name: "Corn".into(),
            seed_id: "corn_seeds".into(),
            harvest_id: "corn".into(),
            // Corn grows in summer AND fall
            seasons: vec![Season::Summer, Season::Fall],
            // 14 total days, 6 stages
            growth_days: vec![1, 2, 3, 4, 3, 1],
            regrows: false,
            regrow_days: 0,
            sell_price: 50,
            sprite_stages: vec![70, 71, 72, 73, 74, 75],
        },

        // ── Fall Crops ───────────────────────────────────────────────────────────

        CropDef {
            id: "eggplant".into(),
            name: "Eggplant".into(),
            seed_id: "eggplant_seeds".into(),
            harvest_id: "eggplant".into(),
            seasons: vec![Season::Fall],
            // 13 days to first harvest, 5 stages
            growth_days: vec![1, 2, 4, 4, 2],
            regrows: true,
            regrow_days: 5,
            sell_price: 60,
            sprite_stages: vec![80, 81, 82, 83, 84],
        },

        CropDef {
            id: "pumpkin".into(),
            name: "Pumpkin".into(),
            seed_id: "pumpkin_seeds".into(),
            harvest_id: "pumpkin".into(),
            seasons: vec![Season::Fall],
            // 13 total days, 5 stages
            growth_days: vec![1, 2, 4, 4, 2],
            regrows: false,
            regrow_days: 0,
            sell_price: 320,
            sprite_stages: vec![90, 91, 92, 93, 94],
        },

        CropDef {
            id: "cranberry".into(),
            name: "Cranberry".into(),
            seed_id: "cranberry_seeds".into(),
            harvest_id: "cranberry".into(),
            seasons: vec![Season::Fall],
            // 7 days to first harvest, 4 stages
            growth_days: vec![1, 1, 2, 3],
            regrows: true,
            regrow_days: 5,
            sell_price: 75,
            sprite_stages: vec![100, 101, 102, 103],
        },

        CropDef {
            id: "yam".into(),
            name: "Yam".into(),
            seed_id: "yam_seeds".into(),
            harvest_id: "yam".into(),
            seasons: vec![Season::Fall],
            // 10 total days, 5 stages
            growth_days: vec![1, 2, 3, 3, 1],
            regrows: false,
            regrow_days: 0,
            sell_price: 160,
            sprite_stages: vec![110, 111, 112, 113, 114],
        },

        // ── Any-Season Crops ─────────────────────────────────────────────────────

        CropDef {
            id: "wheat".into(),
            name: "Wheat".into(),
            seed_id: "wheat_seeds".into(),
            harvest_id: "wheat".into(),
            // Wheat grows in summer and fall (not winter or spring)
            seasons: vec![Season::Summer, Season::Fall],
            // 4 total days, 4 stages (1 day each)
            growth_days: vec![1, 1, 1, 1],
            regrows: false,
            regrow_days: 0,
            sell_price: 25,
            sprite_stages: vec![120, 121, 122, 123],
        },

        CropDef {
            id: "coffee".into(),
            name: "Coffee".into(),
            seed_id: "coffee_beans".into(),
            harvest_id: "coffee".into(),
            // Coffee grows in spring and summer
            seasons: vec![Season::Spring, Season::Summer],
            // 10 days to first harvest, 5 stages
            growth_days: vec![1, 2, 2, 3, 2],
            regrows: true,
            regrow_days: 2,
            sell_price: 150,
            sprite_stages: vec![130, 131, 132, 133, 134],
        },

        CropDef {
            id: "ancient_fruit".into(),
            name: "Ancient Fruit".into(),
            seed_id: "ancient_seeds".into(),
            harvest_id: "ancient_fruit".into(),
            // Ancient fruit grows in spring, summer, and fall (not winter)
            seasons: vec![Season::Spring, Season::Summer, Season::Fall],
            // 28 total days, 6 stages
            growth_days: vec![2, 4, 6, 6, 6, 4],
            regrows: true,
            regrow_days: 7,
            sell_price: 750,
            sprite_stages: vec![140, 141, 142, 143, 144, 145],
        },
    ];

    for crop in crops {
        registry.crops.insert(crop.id.clone(), crop);
    }
}

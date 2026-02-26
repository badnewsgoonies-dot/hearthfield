use crate::shared::*;

/// Build the full recipe for a crafting recipe (non-cooking) by id.
/// Returns None if the id is not recognized.
pub fn make_crafting_recipe(id: &str) -> Option<Recipe> {
    let r = match id {
        // ── Sprinklers ──────────────────────────────────────────────────────
        "sprinkler" => Recipe {
            id: "sprinkler".into(),
            name: "Sprinkler".into(),
            ingredients: vec![("copper_bar".into(), 1)],
            result: "sprinkler".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: true,
        },
        "quality_sprinkler" => Recipe {
            id: "quality_sprinkler".into(),
            name: "Quality Sprinkler".into(),
            ingredients: vec![("iron_bar".into(), 1), ("gold_bar".into(), 1)],
            result: "quality_sprinkler".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false, // unlocked by friendship or shop
        },
        // ── Fences & Paths ──────────────────────────────────────────────────
        "fence" => Recipe {
            id: "fence".into(),
            name: "Fence".into(),
            ingredients: vec![("wood".into(), 2)],
            result: "fence".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: true,
        },
        "path" => Recipe {
            id: "path".into(),
            name: "Path".into(),
            ingredients: vec![("stone".into(), 1)],
            result: "path".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: true,
        },
        // ── Storage ─────────────────────────────────────────────────────────
        "chest" => Recipe {
            id: "chest".into(),
            name: "Chest".into(),
            ingredients: vec![("wood".into(), 50)],
            result: "chest".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: true,
        },
        // ── Processing Machines ─────────────────────────────────────────────
        "furnace" => Recipe {
            id: "furnace".into(),
            name: "Furnace".into(),
            ingredients: vec![("copper_ore".into(), 20), ("stone".into(), 25)],
            result: "furnace".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false,
        },
        "preserves_jar" => Recipe {
            id: "preserves_jar".into(),
            name: "Preserves Jar".into(),
            ingredients: vec![
                ("wood".into(), 50),
                ("stone".into(), 40),
                ("coal".into(), 8),
            ],
            result: "preserves_jar".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false,
        },
        "cheese_press" => Recipe {
            id: "cheese_press".into(),
            name: "Cheese Press".into(),
            ingredients: vec![
                ("wood".into(), 45),
                ("stone".into(), 45),
                ("copper_bar".into(), 10),
            ],
            result: "cheese_press".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false,
        },
        "loom" => Recipe {
            id: "loom".into(),
            name: "Loom".into(),
            ingredients: vec![
                ("wood".into(), 60),
                ("fiber".into(), 30),
                ("pine_tar".into(), 1),
            ],
            result: "loom".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false,
        },
        // ── Farm Utilities ──────────────────────────────────────────────────
        "scarecrow" => Recipe {
            id: "scarecrow".into(),
            name: "Scarecrow".into(),
            ingredients: vec![
                ("wood".into(), 50),
                ("coal".into(), 1),
                ("fiber".into(), 20),
            ],
            result: "scarecrow".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: true,
        },
        // ── Combat Items ────────────────────────────────────────────────────
        "bomb" => Recipe {
            id: "bomb".into(),
            name: "Bomb".into(),
            ingredients: vec![("iron_ore".into(), 4), ("coal".into(), 4)],
            result: "bomb".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false,
        },
        "mega_bomb" => Recipe {
            id: "mega_bomb".into(),
            name: "Mega Bomb".into(),
            ingredients: vec![("gold_ore".into(), 4), ("coal".into(), 4)],
            result: "mega_bomb".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false,
        },
        // ── Lighting ────────────────────────────────────────────────────────
        "torch" => Recipe {
            id: "torch".into(),
            name: "Torch".into(),
            ingredients: vec![("wood".into(), 1), ("sap".into(), 2)],
            result: "torch".into(),
            result_quantity: 3,
            is_cooking: false,
            unlocked_by_default: true,
        },
        "campfire" => Recipe {
            id: "campfire".into(),
            name: "Campfire".into(),
            ingredients: vec![
                ("stone".into(), 10),
                ("wood".into(), 10),
                ("fiber".into(), 10),
            ],
            result: "campfire".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: true,
        },
        // ── Artisan Machines ────────────────────────────────────────────────
        "bee_house" => Recipe {
            id: "bee_house".into(),
            name: "Bee House".into(),
            ingredients: vec![
                ("wood".into(), 40),
                ("coal".into(), 8),
                ("maple_syrup".into(), 1),
            ],
            result: "bee_house".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false,
        },
        "keg" => Recipe {
            id: "keg".into(),
            name: "Keg".into(),
            ingredients: vec![
                ("wood".into(), 30),
                ("copper_bar".into(), 1),
                ("iron_bar".into(), 1),
            ],
            result: "keg".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false,
        },
        "oil_maker" => Recipe {
            id: "oil_maker".into(),
            name: "Oil Maker".into(),
            ingredients: vec![
                ("slime".into(), 50),
                ("hardwood".into(), 20),
                ("gold_bar".into(), 1),
            ],
            result: "oil_maker".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false,
        },
        // ── Farming Tools ───────────────────────────────────────────────────
        "seed_maker" => Recipe {
            id: "seed_maker".into(),
            name: "Seed Maker".into(),
            ingredients: vec![
                ("wood".into(), 25),
                ("gold_bar".into(), 1),
                ("coal".into(), 10),
            ],
            result: "seed_maker".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false,
        },
        "recycler" => Recipe {
            id: "recycler".into(),
            name: "Recycler".into(),
            ingredients: vec![
                ("wood".into(), 25),
                ("stone".into(), 25),
                ("iron_bar".into(), 1),
            ],
            result: "recycler".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false,
        },
        "worm_bin" => Recipe {
            id: "worm_bin".into(),
            name: "Worm Bin".into(),
            ingredients: vec![
                ("wood".into(), 25),
                ("fiber".into(), 50),
                ("gold_bar".into(), 1),
            ],
            result: "worm_bin".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false,
        },
        _ => return None,
    };
    Some(r)
}

/// Build the full recipe for a cooking recipe by id.
/// Returns None if the id is not recognized.
pub fn make_cooking_recipe(id: &str) -> Option<Recipe> {
    let r = match id {
        "fried_egg" => Recipe {
            id: "fried_egg".into(),
            name: "Fried Egg".into(),
            ingredients: vec![("egg".into(), 1)],
            result: "fried_egg".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: true,
        },
        "baked_potato" => Recipe {
            id: "baked_potato".into(),
            name: "Baked Potato".into(),
            ingredients: vec![("potato".into(), 1)],
            result: "baked_potato".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: true,
        },
        "salad" => Recipe {
            id: "salad".into(),
            name: "Salad".into(),
            ingredients: vec![("leek".into(), 1), ("dandelion".into(), 1)],
            result: "salad".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false,
        },
        "cheese_omelette" => Recipe {
            id: "cheese_omelette".into(),
            name: "Cheese Omelette".into(),
            ingredients: vec![
                ("egg".into(), 1),
                ("milk".into(), 1),
                ("cheese".into(), 1),
            ],
            result: "cheese_omelette".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false,
        },
        "pancakes" => Recipe {
            id: "pancakes".into(),
            name: "Pancakes".into(),
            ingredients: vec![("wheat_flour".into(), 1), ("egg".into(), 1)],
            result: "pancakes".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false,
        },
        "fish_stew" => Recipe {
            id: "fish_stew".into(),
            name: "Fish Stew".into(),
            ingredients: vec![("fish".into(), 1), ("potato".into(), 1)],
            result: "fish_stew".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false,
        },
        "pumpkin_soup" => Recipe {
            id: "pumpkin_soup".into(),
            name: "Pumpkin Soup".into(),
            ingredients: vec![("pumpkin".into(), 1), ("milk".into(), 1)],
            result: "pumpkin_soup".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false,
        },
        "fruit_salad" => Recipe {
            id: "fruit_salad".into(),
            name: "Fruit Salad".into(),
            ingredients: vec![
                ("blueberry".into(), 1),
                ("melon".into(), 1),
                ("apple".into(), 1),
            ],
            result: "fruit_salad".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false,
        },
        "cooked_fish" => Recipe {
            id: "cooked_fish".into(),
            name: "Cooked Fish".into(),
            // "any_fish" is special — resolved at cook-time to any fish in inventory
            ingredients: vec![("any_fish".into(), 1)],
            result: "cooked_fish".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: true,
        },
        "bread" => Recipe {
            id: "bread".into(),
            name: "Bread".into(),
            ingredients: vec![("wheat_flour".into(), 1)],
            result: "bread".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: true,
        },
        "pizza" => Recipe {
            id: "pizza".into(),
            name: "Pizza".into(),
            ingredients: vec![
                ("wheat_flour".into(), 1),
                ("tomato".into(), 1),
                ("cheese".into(), 1),
            ],
            result: "pizza".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false,
        },
        "spaghetti" => Recipe {
            id: "spaghetti".into(),
            name: "Spaghetti".into(),
            ingredients: vec![("wheat_flour".into(), 1), ("tomato".into(), 1)],
            result: "spaghetti".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false,
        },
        "ice_cream" => Recipe {
            id: "ice_cream".into(),
            name: "Ice Cream".into(),
            ingredients: vec![("milk".into(), 1), ("sugar".into(), 1)],
            result: "ice_cream".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false,
        },
        "cake" => Recipe {
            id: "cake".into(),
            name: "Cake".into(),
            ingredients: vec![
                ("wheat_flour".into(), 1),
                ("sugar".into(), 1),
                ("egg".into(), 1),
            ],
            result: "cake".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false,
        },
        "cookie" => Recipe {
            id: "cookie".into(),
            name: "Cookie".into(),
            ingredients: vec![
                ("wheat_flour".into(), 1),
                ("sugar".into(), 1),
                ("egg".into(), 1),
            ],
            result: "cookie".into(),
            result_quantity: 3,
            is_cooking: true,
            unlocked_by_default: false,
        },
        _ => return None,
    };
    Some(r)
}

/// All crafting recipe ids (for data plugin initialization).
pub const ALL_CRAFTING_RECIPE_IDS: &[&str] = &[
    "sprinkler",
    "quality_sprinkler",
    "fence",
    "path",
    "chest",
    "furnace",
    "preserves_jar",
    "cheese_press",
    "loom",
    "scarecrow",
    "bomb",
    "mega_bomb",
    "torch",
    "campfire",
    "bee_house",
    "keg",
    "oil_maker",
    "seed_maker",
    "recycler",
    "worm_bin",
];

/// All cooking recipe ids (for data plugin initialization).
pub const ALL_COOKING_RECIPE_IDS: &[&str] = &[
    "fried_egg",
    "baked_potato",
    "salad",
    "cheese_omelette",
    "pancakes",
    "fish_stew",
    "pumpkin_soup",
    "fruit_salad",
    "cooked_fish",
    "bread",
    "pizza",
    "spaghetti",
    "ice_cream",
    "cake",
    "cookie",
];

/// Populate the RecipeRegistry with all known recipes.
pub fn populate_recipe_registry(registry: &mut RecipeRegistry) {
    for id in ALL_CRAFTING_RECIPE_IDS {
        if let Some(recipe) = make_crafting_recipe(id) {
            registry.recipes.insert(id.to_string(), recipe);
        }
    }
    for id in ALL_COOKING_RECIPE_IDS {
        if let Some(recipe) = make_cooking_recipe(id) {
            registry.recipes.insert(id.to_string(), recipe);
        }
    }
}

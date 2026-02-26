use crate::shared::*;

/// Populate the RecipeRegistry with 20 crafting recipes and 15 cooking recipes.
///
/// Crafting recipes produce tools, machines, and farm items.
/// Cooking recipes produce food that restores stamina and provides buffs.
///
/// `unlocked_by_default`: true = available at game start, false = must be learned
pub fn populate_recipes(registry: &mut RecipeRegistry) {
    let recipes: Vec<Recipe> = vec![
        // ═══════════════════════════════════════════════════════════════
        // CRAFTING RECIPES (is_cooking = false)
        // ═══════════════════════════════════════════════════════════════

        // ── Basic Farm Structures ────────────────────────────────────

        Recipe {
            id: "recipe_chest".into(),
            name: "Chest".into(),
            ingredients: vec![("wood".into(), 50)],
            result: "chest".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: true,
        },

        Recipe {
            id: "recipe_fence".into(),
            name: "Wood Fence".into(),
            ingredients: vec![("wood".into(), 1)],
            result: "fence".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: true,
        },

        Recipe {
            id: "recipe_wood_path".into(),
            name: "Wood Path".into(),
            ingredients: vec![("wood".into(), 1)],
            result: "wood_path".into(),
            result_quantity: 2,
            is_cooking: false,
            unlocked_by_default: true,
        },

        Recipe {
            id: "recipe_stone_path".into(),
            name: "Stone Path".into(),
            ingredients: vec![("stone".into(), 1)],
            result: "stone_path".into(),
            result_quantity: 2,
            is_cooking: false,
            unlocked_by_default: true,
        },

        // ── Farming Tools & Machines ─────────────────────────────────

        Recipe {
            id: "recipe_scarecrow".into(),
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

        Recipe {
            id: "recipe_basic_sprinkler".into(),
            name: "Basic Sprinkler".into(),
            ingredients: vec![
                ("copper_bar".into(), 1),
                ("iron_bar".into(), 1),
            ],
            result: "basic_sprinkler".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false, // Learned from Blacksmith at friendship 2
        },

        Recipe {
            id: "recipe_quality_sprinkler".into(),
            name: "Quality Sprinkler".into(),
            ingredients: vec![
                ("iron_bar".into(), 1),
                ("gold_bar".into(), 1),
                ("quartz".into(), 1),
            ],
            result: "quality_sprinkler".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false, // Learned from reaching Farming level 6
        },

        // ── Processing Machines ───────────────────────────────────────

        Recipe {
            id: "recipe_furnace".into(),
            name: "Furnace".into(),
            ingredients: vec![
                ("copper_ore".into(), 20),
                ("stone".into(), 25),
            ],
            result: "furnace".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: true,
        },

        Recipe {
            id: "recipe_preserves_jar".into(),
            name: "Preserves Jar".into(),
            ingredients: vec![
                ("wood".into(), 50),
                ("stone".into(), 40),
                ("coal".into(), 8),
            ],
            result: "preserves_jar".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false, // Learned from Farming level 4
        },

        Recipe {
            id: "recipe_cheese_press".into(),
            name: "Cheese Press".into(),
            ingredients: vec![
                ("wood".into(), 45),
                ("stone".into(), 45),
                ("copper_bar".into(), 1),
            ],
            result: "cheese_press".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false, // Learned from Animal friendship
        },

        Recipe {
            id: "recipe_loom".into(),
            name: "Loom".into(),
            ingredients: vec![
                ("wood".into(), 60),
                ("fiber".into(), 30),
                ("pine_tar".into(), 1),
            ],
            result: "loom".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false, // Learned from Animal shop keeper
        },

        Recipe {
            id: "recipe_mayonnaise_machine".into(),
            name: "Mayonnaise Machine".into(),
            ingredients: vec![
                ("wood".into(), 15),
                ("stone".into(), 15),
                ("copper_bar".into(), 1),
            ],
            result: "mayonnaise_machine".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false, // Learned from Animal shop keeper at friendship 2
        },

        // ── Mining & Combat ────────────────────────────────────────────

        Recipe {
            id: "recipe_cherry_bomb".into(),
            name: "Cherry Bomb".into(),
            ingredients: vec![
                ("copper_ore".into(), 4),
                ("coal".into(), 1),
            ],
            result: "cherry_bomb".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false, // Learned from Blacksmith
        },

        Recipe {
            id: "recipe_bomb".into(),
            name: "Bomb".into(),
            ingredients: vec![
                ("iron_ore".into(), 4),
                ("coal".into(), 1),
            ],
            result: "bomb".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false, // Learned from Blacksmith at friendship 4
        },

        // ── Fishing Accessories ────────────────────────────────────────

        Recipe {
            id: "recipe_crab_pot".into(),
            name: "Crab Pot".into(),
            ingredients: vec![
                ("wood".into(), 40),
                ("iron_bar".into(), 3),
            ],
            result: "crab_pot".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false, // Learned from Fishing level 3
        },

        // ── Miscellaneous Crafting ─────────────────────────────────────

        Recipe {
            id: "recipe_torch".into(),
            name: "Torch".into(),
            ingredients: vec![
                ("wood".into(), 1),
                ("coal".into(), 1),
            ],
            result: "torch".into(),
            result_quantity: 4,
            is_cooking: false,
            unlocked_by_default: true,
        },

        Recipe {
            id: "recipe_wooden_sign".into(),
            name: "Wooden Sign".into(),
            ingredients: vec![("wood".into(), 5)],
            result: "wooden_sign".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: true,
        },

        Recipe {
            id: "recipe_tapper".into(),
            name: "Tapper".into(),
            ingredients: vec![
                ("wood".into(), 40),
                ("copper_bar".into(), 2),
            ],
            result: "tapper".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false, // Learned from Forest exploration
        },

        Recipe {
            id: "recipe_bee_house".into(),
            name: "Bee House".into(),
            ingredients: vec![
                ("wood".into(), 40),
                ("coal".into(), 8),
                ("iron_bar".into(), 1),
                ("maple_syrup".into(), 1),
            ],
            result: "bee_house".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false, // Learned from Farming level 3
        },

        Recipe {
            id: "recipe_recycling_machine".into(),
            name: "Recycling Machine".into(),
            ingredients: vec![
                ("wood".into(), 25),
                ("stone".into(), 25),
                ("iron_bar".into(), 1),
            ],
            result: "recycling_machine".into(),
            result_quantity: 1,
            is_cooking: false,
            unlocked_by_default: false, // Learned from Fishing level 4
        },

        // ═══════════════════════════════════════════════════════════════
        // COOKING RECIPES (is_cooking = true)
        // ═══════════════════════════════════════════════════════════════

        Recipe {
            id: "recipe_fried_egg".into(),
            name: "Fried Egg".into(),
            ingredients: vec![("egg".into(), 1)],
            result: "fried_egg".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: true, // Basic starting recipe
        },

        Recipe {
            id: "recipe_baked_potato".into(),
            name: "Baked Potato".into(),
            ingredients: vec![("potato".into(), 1)],
            result: "baked_potato".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: true,
        },

        Recipe {
            id: "recipe_salad".into(),
            name: "Salad".into(),
            ingredients: vec![
                ("turnip".into(), 1),
                ("tomato".into(), 1),
            ],
            result: "salad".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false, // Learned from NPC: Mira (hearts 3)
        },

        Recipe {
            id: "recipe_cheese_omelette".into(),
            name: "Cheese Omelette".into(),
            ingredients: vec![
                ("egg".into(), 1),
                ("cheese".into(), 1),
            ],
            result: "cheese_omelette".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false, // Learned from NPC: Margaret (hearts 4)
        },

        Recipe {
            id: "recipe_pancakes".into(),
            name: "Pancakes".into(),
            ingredients: vec![
                ("egg".into(), 1),
                ("wheat".into(), 1),
            ],
            result: "pancakes".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false, // Learned from General Store (bought)
        },

        Recipe {
            id: "recipe_bread".into(),
            name: "Bread".into(),
            ingredients: vec![("wheat".into(), 3)],
            result: "bread".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: true,
        },

        Recipe {
            id: "recipe_cooked_fish".into(),
            name: "Cooked Fish".into(),
            ingredients: vec![
                ("bass".into(), 1),
            ],
            result: "cooked_fish".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: true,
        },

        Recipe {
            id: "recipe_fish_stew".into(),
            name: "Fish Stew".into(),
            ingredients: vec![
                ("salmon".into(), 1),
                ("potato".into(), 1),
                ("tomato".into(), 1),
            ],
            result: "fish_stew".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false, // Learned from NPC: Old Tom (hearts 5)
        },

        Recipe {
            id: "recipe_pumpkin_soup".into(),
            name: "Pumpkin Soup".into(),
            ingredients: vec![
                ("pumpkin".into(), 1),
                ("milk".into(), 1),
            ],
            result: "pumpkin_soup".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false, // Learned from NPC: Elena (hearts 7)
        },

        Recipe {
            id: "recipe_spaghetti".into(),
            name: "Spaghetti".into(),
            ingredients: vec![
                ("wheat".into(), 2),
                ("tomato".into(), 2),
            ],
            result: "spaghetti".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false, // Learned from General Store (bought)
        },

        Recipe {
            id: "recipe_pizza".into(),
            name: "Pizza".into(),
            ingredients: vec![
                ("wheat".into(), 2),
                ("tomato".into(), 1),
                ("cheese".into(), 1),
            ],
            result: "pizza".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false, // Learned from NPC: Marco (hearts 6)
        },

        Recipe {
            id: "recipe_fruit_salad".into(),
            name: "Fruit Salad".into(),
            ingredients: vec![
                ("blueberry".into(), 1),
                ("melon".into(), 1),
                ("strawberry".into(), 1),
            ],
            result: "fruit_salad".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false, // Learned from NPC: Lily (hearts 8)
        },

        Recipe {
            id: "recipe_cookie".into(),
            name: "Cookie".into(),
            ingredients: vec![
                ("wheat".into(), 1),
                ("egg".into(), 1),
                ("maple_syrup".into(), 1),
            ],
            result: "cookie".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false, // Learned from NPC: Margaret (hearts 6)
        },

        Recipe {
            id: "recipe_cake".into(),
            name: "Cake".into(),
            ingredients: vec![
                ("wheat".into(), 2),
                ("egg".into(), 2),
                ("milk".into(), 1),
                ("strawberry".into(), 1),
            ],
            result: "cake".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false, // Learned from NPC: Lily (hearts 10)
        },

        Recipe {
            id: "recipe_ice_cream".into(),
            name: "Ice Cream".into(),
            ingredients: vec![
                ("milk".into(), 1),
                ("egg".into(), 1),
                ("blueberry".into(), 2),
            ],
            result: "ice_cream".into(),
            result_quantity: 1,
            is_cooking: true,
            unlocked_by_default: false, // Learned from Summer festival
        },
    ];

    for recipe in recipes {
        registry.recipes.insert(recipe.id.clone(), recipe);
    }
}

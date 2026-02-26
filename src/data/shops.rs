use crate::shared::*;

/// Populate the ShopData resource with listings for all three shops.
///
/// Shops:
///   GeneralStore — seeds (seasonal), hay, basic supplies, recipes for sale
///   AnimalShop   — animals, buildings, hay (also sold here)
///   Blacksmith   — tool upgrades, bombs, ores, bars
pub fn populate_shops(shop_data: &mut ShopData) {
    // ═══════════════════════════════════════════════════════════════
    // GENERAL STORE — Pierre's / general supply shop
    // ═══════════════════════════════════════════════════════════════
    //
    // Seeds are seasonal (only available in their growing season).
    // Some items like hay, bait, and tackle are available year-round.
    // The general store also sells a handful of always-available food.

    let general_store_listings: Vec<ShopListing> = vec![
        // ── Spring Seeds ──────────────────────────────────────────
        ShopListing {
            item_id: "turnip_seeds".into(),
            price: 20,
            season_available: Some(Season::Spring),
        },
        ShopListing {
            item_id: "potato_seeds".into(),
            price: 50,
            season_available: Some(Season::Spring),
        },
        ShopListing {
            item_id: "cauliflower_seeds".into(),
            price: 80,
            season_available: Some(Season::Spring),
        },
        ShopListing {
            item_id: "strawberry_seeds".into(),
            price: 100,
            season_available: Some(Season::Spring),
        },

        // ── Summer Seeds ──────────────────────────────────────────
        ShopListing {
            item_id: "melon_seeds".into(),
            price: 80,
            season_available: Some(Season::Summer),
        },
        ShopListing {
            item_id: "tomato_seeds".into(),
            price: 50,
            season_available: Some(Season::Summer),
        },
        ShopListing {
            item_id: "blueberry_seeds".into(),
            price: 80,
            season_available: Some(Season::Summer),
        },
        ShopListing {
            item_id: "corn_seeds".into(),
            price: 150,
            season_available: Some(Season::Summer),
        },

        // ── Fall Seeds ────────────────────────────────────────────
        ShopListing {
            item_id: "eggplant_seeds".into(),
            price: 20,
            season_available: Some(Season::Fall),
        },
        ShopListing {
            item_id: "pumpkin_seeds".into(),
            price: 100,
            season_available: Some(Season::Fall),
        },
        ShopListing {
            item_id: "cranberry_seeds".into(),
            price: 240,
            season_available: Some(Season::Fall),
        },
        ShopListing {
            item_id: "yam_seeds".into(),
            price: 60,
            season_available: Some(Season::Fall),
        },

        // ── Multi-Season Seeds (available in their first season) ──
        // Wheat: summer and fall
        ShopListing {
            item_id: "wheat_seeds".into(),
            price: 10,
            season_available: Some(Season::Summer),
        },
        // Coffee beans: spring and summer
        ShopListing {
            item_id: "coffee_beans".into(),
            price: 250,
            season_available: Some(Season::Spring),
        },

        // ── Year-Round Supplies ───────────────────────────────────
        ShopListing {
            item_id: "hay".into(),
            price: 50,
            season_available: None,
        },
        ShopListing {
            item_id: "bait".into(),
            price: 5,
            season_available: None,
        },
        ShopListing {
            item_id: "tackle".into(),
            price: 500,
            season_available: None,
        },

        // ── Purchasable Food ──────────────────────────────────────
        ShopListing {
            item_id: "ice_cream".into(),
            price: 250,
            season_available: Some(Season::Summer),
        },

        // ── Recipes for Sale (represented as special items) ───────
        // Note: actual recipe unlock is handled by the economy domain;
        // here we list them at their purchasing prices. The economy
        // domain matches item_id prefixed with "recipe_book_" to
        // trigger a recipe unlock.
        ShopListing {
            item_id: "recipe_book_pancakes".into(),
            price: 200,
            season_available: None,
        },
        ShopListing {
            item_id: "recipe_book_spaghetti".into(),
            price: 300,
            season_available: None,
        },
    ];

    shop_data
        .listings
        .insert(ShopId::GeneralStore, general_store_listings);

    // ═══════════════════════════════════════════════════════════════
    // ANIMAL SHOP — Marnie's / livestock and buildings
    // ═══════════════════════════════════════════════════════════════
    //
    // Sells live animals and building upgrades.
    // Animal purchases are handled specially by the economy domain:
    // items prefixed "animal_" trigger animal spawning.
    // Items prefixed "building_" trigger barn/coop construction.

    let animal_shop_listings: Vec<ShopListing> = vec![
        // ── Animals ───────────────────────────────────────────────
        ShopListing {
            item_id: "animal_chicken".into(),
            price: 800,
            season_available: None,
        },
        ShopListing {
            item_id: "animal_cow".into(),
            price: 1_500,
            season_available: None,
        },
        ShopListing {
            item_id: "animal_sheep".into(),
            price: 2_000,
            season_available: None,
        },

        // ── Buildings ─────────────────────────────────────────────
        // Coop: required before buying chickens
        ShopListing {
            item_id: "building_coop".into(),
            price: 4_000,
            season_available: None,
        },
        ShopListing {
            item_id: "building_big_coop".into(),
            price: 10_000,
            season_available: None,
        },
        ShopListing {
            item_id: "building_deluxe_coop".into(),
            price: 20_000,
            season_available: None,
        },

        // Barn: required before buying cows and sheep
        ShopListing {
            item_id: "building_barn".into(),
            price: 6_000,
            season_available: None,
        },
        ShopListing {
            item_id: "building_big_barn".into(),
            price: 12_000,
            season_available: None,
        },
        ShopListing {
            item_id: "building_deluxe_barn".into(),
            price: 25_000,
            season_available: None,
        },

        // ── Animal Feed ───────────────────────────────────────────
        ShopListing {
            item_id: "hay".into(),
            price: 50,
            season_available: None,
        },
    ];

    shop_data
        .listings
        .insert(ShopId::AnimalShop, animal_shop_listings);

    // ═══════════════════════════════════════════════════════════════
    // BLACKSMITH — Clint's / tool upgrades and mining supplies
    // ═══════════════════════════════════════════════════════════════
    //
    // Sells bombs, ore (for purchase in early game), and handles
    // tool upgrades. Tool upgrades are special items prefixed
    // "upgrade_" followed by the tool name — the economy domain
    // consumes those to trigger upgrades.

    let blacksmith_listings: Vec<ShopListing> = vec![
        // ── Tool Upgrades ─────────────────────────────────────────
        // Each upgrade item costs gold + requires bars in inventory.
        // Prices here are the gold cost; bar requirements checked by economy domain.
        ShopListing {
            item_id: "upgrade_hoe_copper".into(),
            price: 2_000,
            season_available: None,
        },
        ShopListing {
            item_id: "upgrade_hoe_iron".into(),
            price: 5_000,
            season_available: None,
        },
        ShopListing {
            item_id: "upgrade_hoe_gold".into(),
            price: 10_000,
            season_available: None,
        },
        ShopListing {
            item_id: "upgrade_hoe_iridium".into(),
            price: 25_000,
            season_available: None,
        },

        ShopListing {
            item_id: "upgrade_watering_can_copper".into(),
            price: 2_000,
            season_available: None,
        },
        ShopListing {
            item_id: "upgrade_watering_can_iron".into(),
            price: 5_000,
            season_available: None,
        },
        ShopListing {
            item_id: "upgrade_watering_can_gold".into(),
            price: 10_000,
            season_available: None,
        },
        ShopListing {
            item_id: "upgrade_watering_can_iridium".into(),
            price: 25_000,
            season_available: None,
        },

        ShopListing {
            item_id: "upgrade_axe_copper".into(),
            price: 2_000,
            season_available: None,
        },
        ShopListing {
            item_id: "upgrade_axe_iron".into(),
            price: 5_000,
            season_available: None,
        },
        ShopListing {
            item_id: "upgrade_axe_gold".into(),
            price: 10_000,
            season_available: None,
        },
        ShopListing {
            item_id: "upgrade_axe_iridium".into(),
            price: 25_000,
            season_available: None,
        },

        ShopListing {
            item_id: "upgrade_pickaxe_copper".into(),
            price: 2_000,
            season_available: None,
        },
        ShopListing {
            item_id: "upgrade_pickaxe_iron".into(),
            price: 5_000,
            season_available: None,
        },
        ShopListing {
            item_id: "upgrade_pickaxe_gold".into(),
            price: 10_000,
            season_available: None,
        },
        ShopListing {
            item_id: "upgrade_pickaxe_iridium".into(),
            price: 25_000,
            season_available: None,
        },

        // ── Ores (purchasable in early game) ──────────────────────
        ShopListing {
            item_id: "copper_ore".into(),
            price: 75,
            season_available: None,
        },
        ShopListing {
            item_id: "iron_ore".into(),
            price: 150,
            season_available: None,
        },
        ShopListing {
            item_id: "coal".into(),
            price: 150,
            season_available: None,
        },

        // ── Bombs (crafting recipe also available, but can buy here) ──
        ShopListing {
            item_id: "cherry_bomb".into(),
            price: 300,
            season_available: None,
        },
        ShopListing {
            item_id: "bomb".into(),
            price: 600,
            season_available: None,
        },

        // ── Crafting Recipe Books (for sale at friendship levels) ──
        ShopListing {
            item_id: "recipe_book_basic_sprinkler".into(),
            price: 500,
            season_available: None,
        },
    ];

    shop_data
        .listings
        .insert(ShopId::Blacksmith, blacksmith_listings);
}

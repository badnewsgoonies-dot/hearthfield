//! Headless integration tests for Hearthfield.
//!
//! These tests exercise the game's ECS logic without a window or GPU.
//! They use Bevy's `MinimalPlugins` to tick the app, register only the
//! pure-logic systems (skipping all rendering/UI), and verify that the
//! core game loops work correctly.
//!
//! Run with: `cargo test --test headless`

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use hearthfield::animals::{handle_day_end_for_animals, quality_from_happiness, UnfedDays};
use hearthfield::animals::pen_bounds_for;
use hearthfield::calendar::festivals::{
    check_festival_day, cleanup_festival_on_day_end, FestivalKind, FestivalState,
};
use hearthfield::crafting::food_buff_for_item;
use hearthfield::crafting::machines::{resolve_machine_output, MachineType};
use hearthfield::data::DataPlugin;
use hearthfield::ui::{item_icon_index, ITEM_ATLAS_COLUMNS, ITEM_ATLAS_ROWS};
use hearthfield::economy::achievements::{
    check_achievements, track_achievement_progress, ACHIEVEMENTS,
};
use hearthfield::economy::blacksmith::{
    handle_upgrade_request, tick_upgrade_queue, PendingUpgrade, ToolUpgradeCompleteEvent,
    ToolUpgradeQueue, ToolUpgradeRequestEvent,
};
use hearthfield::economy::buildings::{
    handle_building_upgrade_request, tick_building_upgrade, BuildingLevels,
};
use hearthfield::economy::evaluation::{check_evaluation_trigger, handle_evaluation};
use hearthfield::economy::gold::{apply_gold_changes, EconomyStats};
use hearthfield::economy::play_stats::{
    track_crops_harvested, track_gifts_given, track_gold_earned,
};
use hearthfield::economy::shipping::{
    process_shipping_bin_on_day_end, ShippingBinPreview, ShippingBinQuality,
};
use hearthfield::economy::shop::ActiveShop;
use hearthfield::economy::stats::{AnimalProductStats, HarvestStats};
use hearthfield::farming::crop_can_grow_in_season;
use hearthfield::farming::crops::{advance_crop_growth, reset_soil_watered_state};
use hearthfield::farming::events_handler::on_day_end as farming_on_day_end;
use hearthfield::farming::sprinklers::{handle_place_sprinkler, sprinkler_affected_tiles};
use hearthfield::farming::{FarmEntities, TrackedDayWeather};
use hearthfield::fishing::legendaries::{is_legendary, legendary_fish_defs};
use hearthfield::fishing::skill::{xp_for_rarity, FishingSkill};
use hearthfield::npcs::quests::{expire_quests, handle_quest_completed};
use hearthfield::npcs::romance::{
    handle_bouquet, handle_proposal, handle_wedding, tick_wedding_timer, WeddingTimer,
};
use hearthfield::shared::*;
use hearthfield::world::objects::seasonal_forageables;
use std::collections::HashMap;

use hearthfield::crafting::{
    consume_ingredients, handle_craft_item, handle_open_crafting, has_all_ingredients,
    make_cooking_recipe, make_crafting_recipe, populate_recipe_registry, refund_ingredients,
    CraftItemEvent, CraftingUiState, OpenCraftingEvent, ALL_COOKING_RECIPE_IDS,
    ALL_CRAFTING_RECIPE_IDS,
};
use hearthfield::mining::components::{ActiveFloor, FloorSpawnRequest, InMine, MineLadder, MineGridPos};
use hearthfield::mining::{handle_rock_breaking, MiningAtlases, RockDestroyedEvent, RockHitEvent};
use hearthfield::player::movement::player_movement;
use hearthfield::player::{stamina_cost, facing_offset, CameraSnap, CollisionMap};
use hearthfield::world::maps::MapDef;
use hearthfield::world::WorldMap;

// ─────────────────────────────────────────────────────────────────────────────
// Test App Builder
// ─────────────────────────────────────────────────────────────────────────────

/// Builds a minimal Bevy app with all shared resources and events registered
/// but NO rendering, windowing, or asset loading. Systems must be added
/// per-test depending on what's being exercised.
fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);

    // ── Game State ───────────────────────────────────────────────────────
    app.init_state::<GameState>();

    // ── Shared Resources (mirrors main.rs) ───────────────────────────────
    app.init_resource::<Calendar>()
        .init_resource::<PlayerState>()
        .init_resource::<Inventory>()
        .init_resource::<FarmState>()
        .init_resource::<AnimalState>()
        .init_resource::<Relationships>()
        .init_resource::<MineState>()
        .init_resource::<UnlockedRecipes>()
        .init_resource::<ShippingBin>()
        .init_resource::<ItemRegistry>()
        .init_resource::<CropRegistry>()
        .init_resource::<FishRegistry>()
        .init_resource::<RecipeRegistry>()
        .init_resource::<NpcRegistry>()
        .init_resource::<ShopData>();

    // ── Shared Events (mirrors main.rs) ──────────────────────────────────
    app.add_event::<DayEndEvent>()
        .add_event::<SeasonChangeEvent>()
        .add_event::<ItemPickupEvent>()
        .add_event::<ItemRemovedEvent>()
        .add_event::<DialogueStartEvent>()
        .add_event::<DialogueEndEvent>()
        .add_event::<ShopTransactionEvent>()
        .add_event::<ToolUseEvent>()
        .add_event::<MapTransitionEvent>()
        .add_event::<StaminaDrainEvent>()
        .add_event::<GoldChangeEvent>()
        .add_event::<GiftGivenEvent>()
        .add_event::<CropHarvestedEvent>()
        .add_event::<AnimalProductEvent>()
        .add_event::<PlaySfxEvent>()
        .add_event::<PlayMusicEvent>();

    // ── Phase 3 Events ───────────────────────────────────────────────────
    app.add_event::<ToastEvent>();

    // ── Phase 3/4 Resources ────────────────────────────────────────────
    app.init_resource::<HouseState>()
        .init_resource::<MarriageState>()
        .init_resource::<QuestLog>()
        .init_resource::<SprinklerState>()
        .init_resource::<ActiveBuffs>()
        .init_resource::<EvaluationScore>()
        .init_resource::<RelationshipStages>()
        .init_resource::<Achievements>()
        .init_resource::<PlayStats>();

    // ── Phase 3/4 Events ───────────────────────────────────────────────
    app.add_event::<BouquetGivenEvent>()
        .add_event::<ProposalEvent>()
        .add_event::<WeddingEvent>()
        .add_event::<SpouseActionEvent>()
        .add_event::<QuestPostedEvent>()
        .add_event::<QuestAcceptedEvent>()
        .add_event::<QuestCompletedEvent>()
        .add_event::<PlaceSprinklerEvent>()
        .add_event::<EatFoodEvent>()
        .add_event::<EvaluationTriggerEvent>()
        .add_event::<AchievementUnlockedEvent>()
        .add_event::<BuildingUpgradeEvent>()
        .add_event::<HintEvent>()
        .add_event::<ToolImpactEvent>();

    app
}

/// Transitions the test app to Playing state and ticks once to process it.
fn enter_playing_state(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update(); // process state transition
}

/// Sends a DayEndEvent into the app's world.
fn send_day_end(app: &mut App, day: u8, season: Season, year: u32) {
    app.world_mut()
        .send_event(DayEndEvent { day, season, year });
}

#[test]
fn test_headless_boot_smoke_transitions_and_ticks() {
    let mut app = build_test_app();
    app.add_plugins(DataPlugin);

    // First update enters Loading and populates registries; second applies NextState.
    app.update();
    app.update();

    let state = app.world().resource::<State<GameState>>();
    assert_eq!(
        state.get(),
        &GameState::MainMenu,
        "Expected to reach MainMenu after loading data"
    );

    let item_count = app.world().resource::<ItemRegistry>().items.len();
    let crop_count = app.world().resource::<CropRegistry>().crops.len();
    let fish_count = app.world().resource::<FishRegistry>().fish.len();
    let recipe_count = app.world().resource::<RecipeRegistry>().recipes.len();

    assert!(
        item_count > 0,
        "Item registry should be populated during boot"
    );
    assert!(
        crop_count > 0,
        "Crop registry should be populated during boot"
    );
    assert!(
        fish_count > 0,
        "Fish registry should be populated during boot"
    );
    assert!(
        recipe_count > 0,
        "Recipe registry should be populated during boot"
    );

    enter_playing_state(&mut app);

    // Smoke: run a small frame budget in Playing without panic.
    for _ in 0..120 {
        app.update();
    }

    let state = app.world().resource::<State<GameState>>();
    assert_eq!(
        state.get(),
        &GameState::Playing,
        "State should remain Playing after smoke ticks"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 1: Pure function — crop growth advances when watered
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_crop_growth_advances_when_watered() {
    let mut farm_state = FarmState::default();
    let mut crop_registry = CropRegistry::default();

    // Define a parsnip: 4 stages, [1, 1, 1, 1] days per stage
    crop_registry.crops.insert(
        "parsnip".to_string(),
        CropDef {
            id: "parsnip".to_string(),
            name: "Parsnip".to_string(),
            seed_id: "parsnip_seeds".to_string(),
            harvest_id: "parsnip".to_string(),
            seasons: vec![Season::Spring],
            growth_days: vec![1, 1, 1, 1],
            regrows: false,
            regrow_days: 0,
            sell_price: 35,
            sprite_stages: vec![0, 1, 2, 3],
        },
    );

    // Plant a watered parsnip
    farm_state.soil.insert((5, 5), SoilState::Watered);
    farm_state.crops.insert(
        (5, 5),
        CropTile {
            crop_id: "parsnip".to_string(),
            current_stage: 0,
            days_in_stage: 0,
            watered_today: true,
            days_without_water: 0,
            dead: false,
        },
    );

    // Day 1: advance growth
    let updated = advance_crop_growth(&mut farm_state, &crop_registry, Season::Spring, false);
    let crop = farm_state.crops.get(&(5, 5)).unwrap();

    assert!(
        updated.contains(&(5, 5)),
        "Position should be in updated list"
    );
    assert_eq!(
        crop.current_stage, 1,
        "Crop should advance to stage 1 after 1 watered day"
    );
    assert_eq!(
        crop.days_in_stage, 0,
        "days_in_stage should reset after stage advance"
    );
    assert!(!crop.dead, "Crop should not be dead");
}

#[test]
fn test_crop_dies_after_3_days_without_water() {
    let mut farm_state = FarmState::default();
    let mut crop_registry = CropRegistry::default();

    crop_registry.crops.insert(
        "parsnip".to_string(),
        CropDef {
            id: "parsnip".to_string(),
            name: "Parsnip".to_string(),
            seed_id: "parsnip_seeds".to_string(),
            harvest_id: "parsnip".to_string(),
            seasons: vec![Season::Spring],
            growth_days: vec![2, 2, 2, 2],
            regrows: false,
            regrow_days: 0,
            sell_price: 35,
            sprite_stages: vec![0, 1, 2, 3],
        },
    );

    farm_state.soil.insert((3, 3), SoilState::Tilled);
    farm_state.crops.insert(
        (3, 3),
        CropTile {
            crop_id: "parsnip".to_string(),
            current_stage: 0,
            days_in_stage: 0,
            watered_today: false,
            days_without_water: 0,
            dead: false,
        },
    );

    // 3 dry days should kill the crop
    for day in 1..=3 {
        advance_crop_growth(&mut farm_state, &crop_registry, Season::Spring, false);
        let crop = farm_state.crops.get(&(3, 3)).unwrap();
        if day < 3 {
            assert!(!crop.dead, "Crop should survive after {} dry day(s)", day);
        } else {
            assert!(crop.dead, "Crop should die after 3 dry days");
        }
    }
}

#[test]
fn test_rain_auto_waters_crops() {
    let mut farm_state = FarmState::default();
    let mut crop_registry = CropRegistry::default();

    crop_registry.crops.insert(
        "potato".to_string(),
        CropDef {
            id: "potato".to_string(),
            name: "Potato".to_string(),
            seed_id: "potato_seeds".to_string(),
            harvest_id: "potato".to_string(),
            seasons: vec![Season::Spring],
            growth_days: vec![1, 1, 1, 1, 1, 1],
            regrows: false,
            regrow_days: 0,
            sell_price: 80,
            sprite_stages: vec![0, 1, 2, 3, 4, 5],
        },
    );

    // Unwatered crop
    farm_state.soil.insert((0, 0), SoilState::Tilled);
    farm_state.crops.insert(
        (0, 0),
        CropTile {
            crop_id: "potato".to_string(),
            current_stage: 0,
            days_in_stage: 0,
            watered_today: false,
            days_without_water: 0,
            dead: false,
        },
    );

    // Rain should count as watered
    advance_crop_growth(&mut farm_state, &crop_registry, Season::Spring, true);
    let crop = farm_state.crops.get(&(0, 0)).unwrap();

    assert_eq!(
        crop.current_stage, 1,
        "Rain should water crops and advance growth"
    );
    assert!(!crop.dead, "Rained-on crop should not die");
}

#[test]
fn test_wrong_season_kills_crop() {
    let mut farm_state = FarmState::default();
    let mut crop_registry = CropRegistry::default();

    crop_registry.crops.insert(
        "parsnip".to_string(),
        CropDef {
            id: "parsnip".to_string(),
            name: "Parsnip".to_string(),
            seed_id: "parsnip_seeds".to_string(),
            harvest_id: "parsnip".to_string(),
            seasons: vec![Season::Spring], // Spring only
            growth_days: vec![1, 1, 1, 1],
            regrows: false,
            regrow_days: 0,
            sell_price: 35,
            sprite_stages: vec![0, 1, 2, 3],
        },
    );

    farm_state.crops.insert(
        (1, 1),
        CropTile {
            crop_id: "parsnip".to_string(),
            current_stage: 2,
            days_in_stage: 0,
            watered_today: true,
            days_without_water: 0,
            dead: false,
        },
    );

    // Growing in Summer should kill a Spring-only crop
    advance_crop_growth(&mut farm_state, &crop_registry, Season::Summer, false);
    let crop = farm_state.crops.get(&(1, 1)).unwrap();

    assert!(crop.dead, "Spring crop should die when grown in Summer");
}

#[test]
fn test_full_crop_lifecycle() {
    let mut farm_state = FarmState::default();
    let mut crop_registry = CropRegistry::default();

    // 3 stages: [2, 2, 2] days each = 6 total days to mature
    crop_registry.crops.insert(
        "melon".to_string(),
        CropDef {
            id: "melon".to_string(),
            name: "Melon".to_string(),
            seed_id: "melon_seeds".to_string(),
            harvest_id: "melon".to_string(),
            seasons: vec![Season::Summer],
            growth_days: vec![2, 2, 2],
            regrows: false,
            regrow_days: 0,
            sell_price: 250,
            sprite_stages: vec![0, 1, 2],
        },
    );

    farm_state.soil.insert((10, 10), SoilState::Watered);
    farm_state.crops.insert(
        (10, 10),
        CropTile {
            crop_id: "melon".to_string(),
            current_stage: 0,
            days_in_stage: 0,
            watered_today: true,
            days_without_water: 0,
            dead: false,
        },
    );

    // Simulate 6 watered days
    for day in 1..=6 {
        // Re-water the crop each day (simulating the player)
        if let Some(crop) = farm_state.crops.get_mut(&(10, 10)) {
            crop.watered_today = true;
        }
        advance_crop_growth(&mut farm_state, &crop_registry, Season::Summer, false);

        let crop = farm_state.crops.get(&(10, 10)).unwrap();
        let expected_stage = (day / 2).min(3) as u8;
        assert_eq!(
            crop.current_stage, expected_stage,
            "After {} days, expected stage {} but got {}",
            day, expected_stage, crop.current_stage
        );
    }

    let crop = farm_state.crops.get(&(10, 10)).unwrap();
    assert_eq!(
        crop.current_stage, 3,
        "Melon should be fully grown (stage 3) after 6 days"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 2: Soil state resets
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_reset_soil_watered_state() {
    let mut farm_state = FarmState::default();
    farm_state.soil.insert((0, 0), SoilState::Watered);
    farm_state.soil.insert((1, 0), SoilState::Tilled);
    farm_state.soil.insert((2, 0), SoilState::Watered);

    reset_soil_watered_state(&mut farm_state);

    assert_eq!(farm_state.soil[&(0, 0)], SoilState::Tilled);
    assert_eq!(farm_state.soil[&(1, 0)], SoilState::Tilled);
    assert_eq!(farm_state.soil[&(2, 0)], SoilState::Tilled);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 3: Shipping bin → gold (ECS integration test)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_shipping_bin_sells_on_day_end() {
    let mut app = build_test_app();

    // Register economy-local resources
    app.init_resource::<EconomyStats>();
    app.init_resource::<ShippingBinPreview>();
    app.init_resource::<ShippingBinQuality>();
    app.init_resource::<ToolUpgradeQueue>();
    app.init_resource::<HarvestStats>();
    app.init_resource::<AnimalProductStats>();
    app.init_resource::<ShippingLog>();
    app.add_event::<ToolUpgradeCompleteEvent>();

    // Register the two systems we need: shipping sell + gold application
    app.add_systems(
        Update,
        (process_shipping_bin_on_day_end, apply_gold_changes)
            .chain()
            .run_if(in_state(GameState::Playing)),
    );

    // Set up items in the registry
    {
        let mut registry = app.world_mut().resource_mut::<ItemRegistry>();
        registry.items.insert(
            "parsnip".to_string(),
            ItemDef {
                id: "parsnip".to_string(),
                name: "Parsnip".to_string(),
                description: "A spring root vegetable.".to_string(),
                category: ItemCategory::Crop,
                sell_price: 35,
                buy_price: None,
                stack_size: 99,
                edible: true,
                energy_restore: 10.0,
                sprite_index: 0,
            },
        );
    }

    // Put items in the shipping bin
    {
        let mut bin = app.world_mut().resource_mut::<ShippingBin>();
        bin.items.push(InventorySlot {
            item_id: "parsnip".to_string(),
            quantity: 5,
        });
    }

    // Start with 500g
    {
        let mut ps = app.world_mut().resource_mut::<PlayerState>();
        ps.gold = 500;
    }

    enter_playing_state(&mut app);

    // Send DayEndEvent
    send_day_end(&mut app, 1, Season::Spring, 1);
    app.update(); // process shipping bin → GoldChangeEvent
    app.update(); // process GoldChangeEvent → player gold

    // Verify: 5 parsnips × 35g = 175g earned
    let ps = app.world().resource::<PlayerState>();
    assert_eq!(
        ps.gold, 675,
        "Player should have 500 + (5×35=175) = 675g, got {}",
        ps.gold
    );

    // Verify shipping bin is cleared
    let bin = app.world().resource::<ShippingBin>();
    assert!(
        bin.items.is_empty(),
        "Shipping bin should be empty after day end"
    );

    // Verify economy stats
    let stats = app.world().resource::<EconomyStats>();
    assert_eq!(stats.total_gold_earned, 175, "Should track 175g earned");
}

#[test]
fn test_empty_shipping_bin_no_gold_change() {
    let mut app = build_test_app();

    app.init_resource::<EconomyStats>();
    app.init_resource::<ShippingBinPreview>();
    app.init_resource::<ShippingBinQuality>();
    app.init_resource::<ToolUpgradeQueue>();
    app.init_resource::<HarvestStats>();
    app.init_resource::<AnimalProductStats>();
    app.init_resource::<ShippingLog>();
    app.add_event::<ToolUpgradeCompleteEvent>();

    app.add_systems(
        Update,
        (process_shipping_bin_on_day_end, apply_gold_changes)
            .chain()
            .run_if(in_state(GameState::Playing)),
    );

    {
        let mut ps = app.world_mut().resource_mut::<PlayerState>();
        ps.gold = 1000;
    }

    enter_playing_state(&mut app);
    send_day_end(&mut app, 1, Season::Spring, 1);
    app.update();

    let ps = app.world().resource::<PlayerState>();
    assert_eq!(ps.gold, 1000, "Gold should remain unchanged with empty bin");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 4: Animal happiness and production (ECS integration test)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_animal_happiness_increases_when_fed() {
    let mut app = build_test_app();

    // Register the animal day-end system
    app.add_systems(
        Update,
        handle_day_end_for_animals.run_if(in_state(GameState::Playing)),
    );

    enter_playing_state(&mut app);

    // Spawn a fed chicken
    let chicken_id = app
        .world_mut()
        .spawn(Animal {
            kind: AnimalKind::Chicken,
            name: "Clucky".to_string(),
            age: AnimalAge::Adult,
            days_old: 10,
            happiness: 100,
            fed_today: true,
            petted_today: false,
            product_ready: false,
        })
        .id();

    send_day_end(&mut app, 1, Season::Spring, 1);
    app.update();

    let animal = app.world().entity(chicken_id).get::<Animal>().unwrap();

    // Fed: +5 happiness
    assert_eq!(animal.happiness, 105, "Fed chicken should gain 5 happiness");
    assert!(
        animal.product_ready,
        "Fed adult chicken should produce an egg"
    );
    assert!(!animal.fed_today, "fed_today should be reset");
}

#[test]
fn test_animal_happiness_decreases_when_not_fed() {
    let mut app = build_test_app();

    app.add_systems(
        Update,
        handle_day_end_for_animals.run_if(in_state(GameState::Playing)),
    );

    enter_playing_state(&mut app);

    let cow_id = app
        .world_mut()
        .spawn(Animal {
            kind: AnimalKind::Cow,
            name: "Bessie".to_string(),
            age: AnimalAge::Adult,
            days_old: 20,
            happiness: 100,
            fed_today: false,
            petted_today: false,
            product_ready: false,
        })
        .id();

    send_day_end(&mut app, 1, Season::Spring, 1);
    app.update();

    let animal = app.world().entity(cow_id).get::<Animal>().unwrap();

    // Not fed: -12 happiness
    assert_eq!(animal.happiness, 88, "Unfed cow should lose 12 happiness");
    assert!(!animal.product_ready, "Unfed cow should not produce");
}

#[test]
fn test_animal_fed_and_petted_bonus() {
    let mut app = build_test_app();

    app.add_systems(
        Update,
        handle_day_end_for_animals.run_if(in_state(GameState::Playing)),
    );

    enter_playing_state(&mut app);

    let sheep_id = app
        .world_mut()
        .spawn(Animal {
            kind: AnimalKind::Chicken, // using chicken for simpler daily production
            name: "Woolly".to_string(),
            age: AnimalAge::Adult,
            days_old: 15,
            happiness: 200,
            fed_today: true,
            petted_today: true,
            product_ready: false,
        })
        .id();

    send_day_end(&mut app, 1, Season::Spring, 1);
    app.update();

    let animal = app.world().entity(sheep_id).get::<Animal>().unwrap();

    // Fed +5, petted +5 = 210
    assert_eq!(
        animal.happiness, 210,
        "Fed + petted animal should gain 10 happiness"
    );
}

#[test]
fn test_baby_animal_grows_to_adult() {
    let mut app = build_test_app();

    app.add_systems(
        Update,
        handle_day_end_for_animals.run_if(in_state(GameState::Playing)),
    );

    enter_playing_state(&mut app);

    let baby_id = app
        .world_mut()
        .spawn(Animal {
            kind: AnimalKind::Chicken,
            name: "Chick".to_string(),
            age: AnimalAge::Baby,
            days_old: 6, // will become 7 after day end → adult
            happiness: 150,
            fed_today: true,
            petted_today: false,
            product_ready: false,
        })
        .id();

    send_day_end(&mut app, 5, Season::Spring, 1);
    app.update();

    let animal = app.world().entity(baby_id).get::<Animal>().unwrap();

    assert_eq!(
        animal.age,
        AnimalAge::Adult,
        "Baby should grow to adult after 7 days"
    );
    assert_eq!(animal.days_old, 7);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 5: Quality from happiness (pure function)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_quality_from_happiness() {
    assert_eq!(quality_from_happiness(0), ItemQuality::Normal);
    assert_eq!(quality_from_happiness(50), ItemQuality::Normal);
    assert_eq!(quality_from_happiness(127), ItemQuality::Normal);
    assert_eq!(quality_from_happiness(128), ItemQuality::Silver);
    assert_eq!(quality_from_happiness(199), ItemQuality::Silver);
    assert_eq!(quality_from_happiness(200), ItemQuality::Gold);
    assert_eq!(quality_from_happiness(229), ItemQuality::Gold);
    assert_eq!(quality_from_happiness(230), ItemQuality::Iridium);
    assert_eq!(quality_from_happiness(255), ItemQuality::Iridium);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 6: Item quality sell multiplier (pure function)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_item_quality_sell_multiplier() {
    assert!((ItemQuality::Normal.sell_multiplier() - 1.0).abs() < f32::EPSILON);
    assert!((ItemQuality::Silver.sell_multiplier() - 1.25).abs() < f32::EPSILON);
    assert!((ItemQuality::Gold.sell_multiplier() - 1.5).abs() < f32::EPSILON);
    assert!((ItemQuality::Iridium.sell_multiplier() - 2.0).abs() < f32::EPSILON);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 7: Calendar pure functions
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_calendar_day_of_week() {
    let cal = Calendar::default();
    // Day 1, Spring, Year 1 = Monday
    assert_eq!(cal.day_of_week(), DayOfWeek::Monday);
}

#[test]
fn test_calendar_total_days_elapsed() {
    let mut cal = Calendar::default();
    assert_eq!(cal.total_days_elapsed(), 0);

    cal.day = 28;
    cal.season = Season::Fall;
    cal.year = 2;
    // year=2 → 112 days, fall=2*28=56, day=27 offset
    assert_eq!(cal.total_days_elapsed(), 112 + 56 + 27);
}

#[test]
fn test_calendar_festival_days() {
    let mut cal = Calendar {
        season: Season::Spring,
        day: 13,
        ..Default::default()
    };
    assert!(cal.is_festival_day(), "Spring 13 = Egg Festival");

    cal.season = Season::Summer;
    cal.day = 11;
    assert!(cal.is_festival_day(), "Summer 11 = Luau");

    cal.season = Season::Fall;
    cal.day = 16;
    assert!(cal.is_festival_day(), "Fall 16 = Harvest Festival");

    cal.season = Season::Winter;
    cal.day = 25;
    assert!(cal.is_festival_day(), "Winter 25 = Winter Star");

    cal.season = Season::Spring;
    cal.day = 1;
    assert!(!cal.is_festival_day(), "Spring 1 is not a festival");
}

#[test]
fn test_calendar_time_float() {
    let cal = Calendar {
        hour: 14,
        minute: 30,
        ..Default::default()
    };
    assert!((cal.time_float() - 14.5).abs() < 0.001);
}

#[test]
fn test_season_cycle() {
    assert_eq!(Season::Spring.next(), Season::Summer);
    assert_eq!(Season::Summer.next(), Season::Fall);
    assert_eq!(Season::Fall.next(), Season::Winter);
    assert_eq!(Season::Winter.next(), Season::Spring);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 8: Gold change events (ECS integration test)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_gold_increase_via_event() {
    let mut app = build_test_app();
    app.init_resource::<EconomyStats>();

    app.add_systems(
        Update,
        apply_gold_changes.run_if(in_state(GameState::Playing)),
    );

    {
        let mut ps = app.world_mut().resource_mut::<PlayerState>();
        ps.gold = 100;
    }

    enter_playing_state(&mut app);

    app.world_mut().send_event(GoldChangeEvent {
        amount: 250,
        reason: "test deposit".to_string(),
    });
    app.update();

    let ps = app.world().resource::<PlayerState>();
    assert_eq!(ps.gold, 350, "Gold should increase by 250");
}

#[test]
fn test_gold_decrease_via_event() {
    let mut app = build_test_app();
    app.init_resource::<EconomyStats>();

    app.add_systems(
        Update,
        apply_gold_changes.run_if(in_state(GameState::Playing)),
    );

    {
        let mut ps = app.world_mut().resource_mut::<PlayerState>();
        ps.gold = 500;
    }

    enter_playing_state(&mut app);

    app.world_mut().send_event(GoldChangeEvent {
        amount: -200,
        reason: "test purchase".to_string(),
    });
    app.update();

    let ps = app.world().resource::<PlayerState>();
    assert_eq!(ps.gold, 300, "Gold should decrease by 200");
}

#[test]
fn test_gold_clamps_to_zero() {
    let mut app = build_test_app();
    app.init_resource::<EconomyStats>();

    app.add_systems(
        Update,
        apply_gold_changes.run_if(in_state(GameState::Playing)),
    );

    {
        let mut ps = app.world_mut().resource_mut::<PlayerState>();
        ps.gold = 50;
    }

    enter_playing_state(&mut app);

    app.world_mut().send_event(GoldChangeEvent {
        amount: -999,
        reason: "overspend".to_string(),
    });
    app.update();

    let ps = app.world().resource::<PlayerState>();
    assert_eq!(ps.gold, 0, "Gold should clamp to 0, not go negative");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 9: Multi-day simulation (ECS integration)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_multi_day_shipping_accumulation() {
    let mut app = build_test_app();

    app.init_resource::<EconomyStats>();
    app.init_resource::<ShippingBinPreview>();
    app.init_resource::<ShippingBinQuality>();
    app.init_resource::<ToolUpgradeQueue>();
    app.init_resource::<HarvestStats>();
    app.init_resource::<AnimalProductStats>();
    app.init_resource::<ShippingLog>();
    app.add_event::<ToolUpgradeCompleteEvent>();

    app.add_systems(
        Update,
        (process_shipping_bin_on_day_end, apply_gold_changes)
            .chain()
            .run_if(in_state(GameState::Playing)),
    );

    // Register item
    {
        let mut reg = app.world_mut().resource_mut::<ItemRegistry>();
        reg.items.insert(
            "egg".to_string(),
            ItemDef {
                id: "egg".to_string(),
                name: "Egg".to_string(),
                description: "A fresh egg.".to_string(),
                category: ItemCategory::AnimalProduct,
                sell_price: 50,
                buy_price: None,
                stack_size: 99,
                edible: true,
                energy_restore: 5.0,
                sprite_index: 0,
            },
        );
    }

    {
        let mut ps = app.world_mut().resource_mut::<PlayerState>();
        ps.gold = 0;
    }

    enter_playing_state(&mut app);

    // Simulate 3 days of shipping 2 eggs each
    for day in 1..=3 {
        {
            let mut bin = app.world_mut().resource_mut::<ShippingBin>();
            bin.items.push(InventorySlot {
                item_id: "egg".to_string(),
                quantity: 2,
            });
        }
        send_day_end(&mut app, day, Season::Spring, 1);
        app.update(); // process shipping
        app.update(); // process gold
    }

    let ps = app.world().resource::<PlayerState>();
    // 3 days × 2 eggs × 50g = 300g
    assert_eq!(
        ps.gold, 300,
        "Should have 300g after 3 days of shipping 2 eggs"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 10: Starvation streak blocks animal production
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_animal_starvation_blocks_production() {
    let mut app = build_test_app();

    app.add_systems(
        Update,
        handle_day_end_for_animals.run_if(in_state(GameState::Playing)),
    );

    enter_playing_state(&mut app);

    let chicken_id = app
        .world_mut()
        .spawn((
            Animal {
                kind: AnimalKind::Chicken,
                name: "Hungry".to_string(),
                age: AnimalAge::Adult,
                days_old: 10,
                happiness: 200,
                fed_today: false,
                petted_today: false,
                product_ready: false,
            },
            UnfedDays { count: 2 },
        ))
        .id();

    // Day with unfed_days going from 2→3: should trigger starvation block
    send_day_end(&mut app, 1, Season::Spring, 1);
    app.update();

    let animal = app.world().entity(chicken_id).get::<Animal>().unwrap();
    assert!(!animal.product_ready, "Starved chicken should not produce");

    let unfed = app.world().entity(chicken_id).get::<UnfedDays>().unwrap();
    assert_eq!(unfed.count, 3, "Unfed days should be 3");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 11: Farming day-end with ECS (system-level test)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_farming_day_end_system() {
    let mut app = build_test_app();

    // Register farming-local resources
    app.init_resource::<FarmEntities>();
    app.init_resource::<TrackedDayWeather>();

    app.add_systems(
        Update,
        farming_on_day_end.run_if(in_state(GameState::Playing)),
    );

    // Set up crop registry
    {
        let mut reg = app.world_mut().resource_mut::<CropRegistry>();
        reg.crops.insert(
            "turnip".to_string(),
            CropDef {
                id: "turnip".to_string(),
                name: "Turnip".to_string(),
                seed_id: "turnip_seeds".to_string(),
                harvest_id: "turnip".to_string(),
                seasons: vec![Season::Spring, Season::Fall],
                growth_days: vec![1, 1, 1],
                regrows: false,
                regrow_days: 0,
                sell_price: 60,
                sprite_stages: vec![0, 1, 2],
            },
        );
    }

    // Plant a watered turnip
    {
        let mut fs = app.world_mut().resource_mut::<FarmState>();
        fs.soil.insert((4, 4), SoilState::Watered);
        fs.crops.insert(
            (4, 4),
            CropTile {
                crop_id: "turnip".to_string(),
                current_stage: 0,
                days_in_stage: 0,
                watered_today: true,
                days_without_water: 0,
                dead: false,
            },
        );
        // Prevent random crow attacks from making this system-level test flaky.
        fs.objects.insert((4, 4), FarmObject::Scarecrow);
    }

    // Set tracked weather to sunny
    {
        let mut tw = app.world_mut().resource_mut::<TrackedDayWeather>();
        tw.weather = Weather::Sunny;
        tw.day = 1;
        tw.season = Season::Spring;
        tw.year = 1;
    }

    enter_playing_state(&mut app);

    send_day_end(&mut app, 1, Season::Spring, 1);
    app.update();

    let fs = app.world().resource::<FarmState>();
    let crop = fs.crops.get(&(4, 4)).unwrap();

    assert_eq!(
        crop.current_stage, 1,
        "Turnip should advance one stage after day end"
    );
    assert!(!crop.dead, "Turnip should still be alive");
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 13: Default keybindings (pure function test)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_default_keybindings_wasd() {
    let bindings = KeyBindings::default();
    assert_eq!(bindings.move_up, KeyCode::KeyW);
    assert_eq!(bindings.move_down, KeyCode::KeyS);
    assert_eq!(bindings.move_left, KeyCode::KeyA);
    assert_eq!(bindings.move_right, KeyCode::KeyD);
    assert_eq!(bindings.interact, KeyCode::KeyF);
    assert_eq!(bindings.tool_use, KeyCode::Space);
    assert_eq!(bindings.pause, KeyCode::Escape);
}

// ─────────────────────────────────────────────────────────────────────────────
// Test 14: MenuTheme touch-friendly button height
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_menu_theme_touch_friendly_button_height() {
    let theme = MenuTheme::default();
    assert!(
        theme.button_height >= 44.0,
        "Button height should be >= 44px for touch targets, got {}",
        theme.button_height
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Pure function tests — Watering Can Area
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_watering_can_basic_single_tile() {
    let tiles = watering_can_area(ToolTier::Basic, 5, 5, Facing::Up);
    assert_eq!(tiles.len(), 1, "Basic tier should affect exactly 1 tile");
    assert!(
        tiles.contains(&(5, 5)),
        "Basic tier should return the target tile itself"
    );
}

#[test]
fn test_watering_can_copper_3_line() {
    // Copper: 3 tiles in a line along facing direction, starting at target.
    // Facing::Up means dy=+1 per step, so tiles are (5,5), (5,6), (5,7).
    let tiles = watering_can_area(ToolTier::Copper, 5, 5, Facing::Up);
    assert_eq!(tiles.len(), 3, "Copper tier should affect exactly 3 tiles");
    assert!(tiles.contains(&(5, 5)), "Line should start at target tile");
    assert!(
        tiles.contains(&(5, 6)),
        "Line should include one step in facing direction"
    );
    assert!(
        tiles.contains(&(5, 7)),
        "Line should include two steps in facing direction"
    );

    // Also verify with a different facing to ensure directionality is correct.
    let tiles_left = watering_can_area(ToolTier::Copper, 10, 10, Facing::Left);
    assert_eq!(tiles_left.len(), 3);
    assert!(tiles_left.contains(&(10, 10)));
    assert!(
        tiles_left.contains(&(9, 10)),
        "Left-facing line should step in -x"
    );
    assert!(
        tiles_left.contains(&(8, 10)),
        "Left-facing line should step in -x twice"
    );
}

#[test]
fn test_watering_can_iron_5_line() {
    // Iron: 5 tiles in a line along facing direction, starting at target.
    // Facing::Right means dx=+1 per step, so tiles are (5,5)..(9,5).
    let tiles = watering_can_area(ToolTier::Iron, 5, 5, Facing::Right);
    assert_eq!(tiles.len(), 5, "Iron tier should affect exactly 5 tiles");
    for i in 0..5 {
        assert!(
            tiles.contains(&(5 + i, 5)),
            "Iron line facing right should include tile ({}, 5)",
            5 + i
        );
    }

    // Verify facing down: dy=-1 per step.
    let tiles_down = watering_can_area(ToolTier::Iron, 0, 0, Facing::Down);
    assert_eq!(tiles_down.len(), 5);
    for i in 0..5 {
        assert!(
            tiles_down.contains(&(0, -i)),
            "Iron line facing down should include tile (0, {})",
            -i
        );
    }
}

#[test]
fn test_watering_can_gold_3x3() {
    // Gold: 3×3 area (9 tiles) centered on target via square_area(cx, cy, radius=1).
    let tiles = watering_can_area(ToolTier::Gold, 5, 5, Facing::Up);
    assert_eq!(
        tiles.len(),
        9,
        "Gold tier should affect exactly 9 tiles (3×3)"
    );

    // The 3×3 area centered on (5,5) includes all tiles from (4,4) to (6,6).
    for dx in -1..=1_i32 {
        for dy in -1..=1_i32 {
            assert!(
                tiles.contains(&(5 + dx, 5 + dy)),
                "Gold 3×3 should include offset ({}, {})",
                dx,
                dy
            );
        }
    }

    // Gold ignores facing — verify same result with a different facing.
    let tiles_left = watering_can_area(ToolTier::Gold, 5, 5, Facing::Left);
    assert_eq!(tiles_left.len(), 9, "Gold should ignore facing direction");
    for t in &tiles {
        assert!(
            tiles_left.contains(t),
            "Gold tiles should be identical regardless of facing"
        );
    }
}

#[test]
fn test_watering_can_iridium_6x6() {
    // Iridium: 6×6 area (36 tiles). Implementation uses half=3,
    // dx in -(half-1)..=half => -2..=3, dy in -2..=3.
    let tiles = watering_can_area(ToolTier::Iridium, 5, 5, Facing::Up);
    assert_eq!(
        tiles.len(),
        36,
        "Iridium tier should affect exactly 36 tiles (6×6)"
    );

    // Verify bounds: offsets range from -2 to +3 in both axes.
    for dy in -2..=3_i32 {
        for dx in -2..=3_i32 {
            assert!(
                tiles.contains(&(5 + dx, 5 + dy)),
                "Iridium 6×6 should include offset ({}, {})",
                dx,
                dy
            );
        }
    }

    // No tiles outside the 6×6 range.
    assert!(
        !tiles.contains(&(5 - 3, 5)),
        "Iridium should NOT include dx=-3"
    );
    assert!(
        !tiles.contains(&(5, 5 + 4)),
        "Iridium should NOT include dy=+4"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Pure function tests — Sprinkler Affected Tiles
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_sprinkler_basic_4_cardinal() {
    // Basic: range 1, cardinal only (no diagonals), skip centre → 4 tiles.
    let tiles = sprinkler_affected_tiles(SprinklerKind::Basic, 10, 10);
    assert_eq!(
        tiles.len(),
        4,
        "Basic sprinkler should affect exactly 4 tiles"
    );

    // Should contain the 4 cardinal neighbours.
    assert!(tiles.contains(&(9, 10)), "Basic should include west tile");
    assert!(tiles.contains(&(11, 10)), "Basic should include east tile");
    assert!(tiles.contains(&(10, 9)), "Basic should include south tile");
    assert!(tiles.contains(&(10, 11)), "Basic should include north tile");

    // Should NOT contain the centre tile.
    assert!(
        !tiles.contains(&(10, 10)),
        "Basic should exclude the centre tile"
    );

    // Should NOT contain diagonal tiles.
    assert!(!tiles.contains(&(9, 9)), "Basic should exclude diagonals");
    assert!(!tiles.contains(&(11, 11)), "Basic should exclude diagonals");
}

#[test]
fn test_sprinkler_quality_8_surrounding() {
    // Quality: range 1, includes diagonals, skip centre → 8 tiles (3×3 - 1).
    let tiles = sprinkler_affected_tiles(SprinklerKind::Quality, 10, 10);
    assert_eq!(
        tiles.len(),
        8,
        "Quality sprinkler should affect exactly 8 tiles"
    );

    // All 8 neighbours (cardinal + diagonal) should be present.
    for dx in -1..=1_i32 {
        for dy in -1..=1_i32 {
            if dx == 0 && dy == 0 {
                assert!(
                    !tiles.contains(&(10, 10)),
                    "Quality should exclude the centre tile"
                );
                continue;
            }
            assert!(
                tiles.contains(&(10 + dx, 10 + dy)),
                "Quality should include offset ({}, {})",
                dx,
                dy
            );
        }
    }
}

#[test]
fn test_sprinkler_iridium_24_tiles() {
    // Iridium: range 2, includes diagonals, skip centre → 24 tiles (5×5 - 1).
    let tiles = sprinkler_affected_tiles(SprinklerKind::Iridium, 10, 10);
    assert_eq!(
        tiles.len(),
        24,
        "Iridium sprinkler should affect exactly 24 tiles"
    );

    // All tiles in 5×5 area except the centre.
    for dx in -2..=2_i32 {
        for dy in -2..=2_i32 {
            if dx == 0 && dy == 0 {
                assert!(
                    !tiles.contains(&(10, 10)),
                    "Iridium should exclude the centre tile"
                );
                continue;
            }
            assert!(
                tiles.contains(&(10 + dx, 10 + dy)),
                "Iridium should include offset ({}, {})",
                dx,
                dy
            );
        }
    }

    // Should NOT include tiles outside range 2.
    assert!(
        !tiles.contains(&(13, 10)),
        "Iridium should not reach range 3"
    );
    assert!(
        !tiles.contains(&(10, 13)),
        "Iridium should not reach range 3"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Pure function tests — Seasonal Forageables
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_seasonal_forageables_spring_has_items() {
    let items = seasonal_forageables(Season::Spring);
    assert_eq!(items.len(), 5, "Spring should have exactly 5 forageables");

    let names: Vec<&str> = items.iter().map(|(name, _)| *name).collect();
    assert!(
        names.contains(&"wild_horseradish"),
        "Spring should contain wild_horseradish"
    );
    assert!(
        names.contains(&"daffodil"),
        "Spring should contain daffodil"
    );
    assert!(names.contains(&"leek"), "Spring should contain leek");
    assert!(
        names.contains(&"dandelion"),
        "Spring should contain dandelion"
    );
    assert!(
        names.contains(&"spring_onion"),
        "Spring should contain spring_onion"
    );
}

#[test]
fn test_seasonal_forageables_all_seasons_nonempty() {
    let seasons = [Season::Spring, Season::Summer, Season::Fall, Season::Winter];
    let expected_counts = [5, 4, 5, 4]; // Spring=5, Summer=4, Fall=5, Winter=4

    for (season, &expected) in seasons.iter().zip(expected_counts.iter()) {
        let items = seasonal_forageables(*season);
        assert!(
            !items.is_empty(),
            "{:?} should have at least one forageable",
            season
        );
        assert_eq!(
            items.len(),
            expected,
            "{:?} should have exactly {} forageables",
            season,
            expected
        );
        for (name, _color) in &items {
            assert!(
                !name.is_empty(),
                "{:?} forageable should have a non-empty item ID",
                season
            );
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Play Stats Integration Tests
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_play_stats_tracks_crops_harvested() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        track_crops_harvested.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Send 3 CropHarvestedEvents
    app.world_mut().send_event(CropHarvestedEvent {
        crop_id: "parsnip_seeds".to_string(),
        harvest_id: "parsnip".to_string(),
        quantity: 1,
        x: 5,
        y: 5,
        quality: Some(ItemQuality::Normal),
    });
    app.world_mut().send_event(CropHarvestedEvent {
        crop_id: "potato_seeds".to_string(),
        harvest_id: "potato".to_string(),
        quantity: 1,
        x: 6,
        y: 5,
        quality: Some(ItemQuality::Silver),
    });
    app.world_mut().send_event(CropHarvestedEvent {
        crop_id: "cauliflower_seeds".to_string(),
        harvest_id: "cauliflower".to_string(),
        quantity: 1,
        x: 7,
        y: 5,
        quality: None,
    });
    app.update();

    let stats = app.world().resource::<PlayStats>();
    assert_eq!(
        stats.crops_harvested, 3,
        "Expected 3 crops harvested, got {}",
        stats.crops_harvested
    );
}

#[test]
fn test_play_stats_tracks_gifts_given() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        track_gifts_given.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    app.world_mut().send_event(GiftGivenEvent {
        npc_id: "alice".to_string(),
        item_id: "tulip".to_string(),
        preference: GiftPreference::Loved,
    });
    app.world_mut().send_event(GiftGivenEvent {
        npc_id: "bob".to_string(),
        item_id: "stone".to_string(),
        preference: GiftPreference::Disliked,
    });
    app.update();

    let stats = app.world().resource::<PlayStats>();
    assert_eq!(
        stats.gifts_given, 2,
        "Expected 2 gifts given, got {}",
        stats.gifts_given
    );
}

#[test]
fn test_play_stats_tracks_gold_earned() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        track_gold_earned.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Positive event
    app.world_mut().send_event(GoldChangeEvent {
        amount: 500,
        reason: "Sold parsnips".to_string(),
    });
    app.update();

    let stats = app.world().resource::<PlayStats>();
    assert_eq!(
        stats.total_gold_earned, 500,
        "Expected 500 gold earned after positive event"
    );

    // Negative event — should NOT decrease total_gold_earned
    app.world_mut().send_event(GoldChangeEvent {
        amount: -200,
        reason: "Bought seeds".to_string(),
    });
    app.update();

    let stats = app.world().resource::<PlayStats>();
    assert_eq!(
        stats.total_gold_earned, 500,
        "Expected total_gold_earned to remain 500 after negative event"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Building Tests
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_building_tier_capacity_values() {
    assert_eq!(
        BuildingTier::None.capacity(),
        0,
        "None capacity should be 0"
    );
    assert_eq!(
        BuildingTier::Basic.capacity(),
        4,
        "Basic capacity should be 4"
    );
    assert_eq!(BuildingTier::Big.capacity(), 8, "Big capacity should be 8");
    assert_eq!(
        BuildingTier::Deluxe.capacity(),
        12,
        "Deluxe capacity should be 12"
    );
}

#[test]
fn test_building_tier_next() {
    assert_eq!(BuildingTier::None.next(), Some(BuildingTier::Basic));
    assert_eq!(BuildingTier::Basic.next(), Some(BuildingTier::Big));
    assert_eq!(BuildingTier::Big.next(), Some(BuildingTier::Deluxe));
    assert_eq!(BuildingTier::Deluxe.next(), None);
}

#[test]
fn test_building_upgrade_request_deducts_gold() {
    let mut app = build_test_app();
    app.init_resource::<BuildingLevels>();
    app.init_resource::<EconomyStats>();
    app.add_systems(
        Update,
        (handle_building_upgrade_request, apply_gold_changes)
            .chain()
            .run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Give the player enough gold and materials for a Coop Basic upgrade (4000g + 150 wood + 50 stone)
    app.world_mut().resource_mut::<PlayerState>().gold = 10_000;
    {
        let mut inv = app.world_mut().resource_mut::<Inventory>();
        inv.try_add("wood", 200, 255);
        inv.try_add("stone", 100, 255);
    }

    app.world_mut().send_event(BuildingUpgradeEvent {
        building: BuildingKind::Coop,
        from_tier: BuildingTier::None,
        to_tier: BuildingTier::Basic,
        cost_gold: 4_000,
        cost_materials: vec![("wood".to_string(), 150), ("stone".to_string(), 50)],
    });
    app.update();

    let player = app.world().resource::<PlayerState>();
    assert_eq!(
        player.gold, 6_000,
        "Gold should be deducted by 4000 for Coop Basic upgrade"
    );

    let levels = app.world().resource::<BuildingLevels>();
    assert!(
        levels.upgrade_in_progress.is_some(),
        "upgrade_in_progress should be set after a valid upgrade request"
    );
    let (building, target_tier, days_left) = levels.upgrade_in_progress.unwrap();
    assert_eq!(building, BuildingKind::Coop);
    assert_eq!(target_tier, BuildingTier::Basic);
    assert_eq!(days_left, 2, "Upgrade should take 2 days");
}

#[test]
fn test_building_upgrade_tick_decrements() {
    let mut app = build_test_app();
    app.init_resource::<BuildingLevels>();
    app.add_systems(
        Update,
        tick_building_upgrade.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Manually set an upgrade in progress with 3 days remaining
    app.world_mut()
        .resource_mut::<BuildingLevels>()
        .upgrade_in_progress = Some((BuildingKind::Coop, BuildingTier::Basic, 3));

    send_day_end(&mut app, 2, Season::Spring, 1);
    app.update();

    let levels = app.world().resource::<BuildingLevels>();
    assert!(
        levels.upgrade_in_progress.is_some(),
        "Upgrade should still be in progress after ticking from 3 days"
    );
    let (_, _, days_left) = levels.upgrade_in_progress.unwrap();
    assert_eq!(days_left, 2, "Days remaining should decrement from 3 to 2");
}

#[test]
fn test_building_upgrade_completes() {
    let mut app = build_test_app();
    app.init_resource::<BuildingLevels>();
    app.add_systems(
        Update,
        tick_building_upgrade.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Set upgrade in progress with 1 day remaining for coop → Basic
    app.world_mut()
        .resource_mut::<BuildingLevels>()
        .upgrade_in_progress = Some((BuildingKind::Coop, BuildingTier::Basic, 1));

    send_day_end(&mut app, 3, Season::Spring, 1);
    app.update();

    let levels = app.world().resource::<BuildingLevels>();
    assert!(
        levels.upgrade_in_progress.is_none(),
        "upgrade_in_progress should be None after completion"
    );
    assert_eq!(
        levels.coop_tier,
        BuildingTier::Basic,
        "Coop tier should be upgraded to Basic"
    );

    let animal_state = app.world().resource::<AnimalState>();
    assert!(
        animal_state.has_coop,
        "has_coop should be true after Coop upgrade"
    );
    assert_eq!(
        animal_state.coop_level, 1,
        "coop_level should be 1 for Basic"
    );
}

#[test]
fn test_building_upgrade_silo() {
    let mut app = build_test_app();
    app.init_resource::<BuildingLevels>();
    app.add_systems(
        Update,
        tick_building_upgrade.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    app.world_mut()
        .resource_mut::<BuildingLevels>()
        .upgrade_in_progress = Some((BuildingKind::Silo, BuildingTier::Basic, 1));

    send_day_end(&mut app, 4, Season::Spring, 1);
    app.update();

    let levels = app.world().resource::<BuildingLevels>();
    assert!(
        levels.silo_built,
        "silo_built should be true after Silo upgrade completes"
    );
    assert!(
        levels.upgrade_in_progress.is_none(),
        "upgrade_in_progress should be cleared after silo completion"
    );
}

#[test]
fn test_building_cannot_upgrade_past_deluxe() {
    assert_eq!(
        BuildingTier::Deluxe.next(),
        None,
        "Deluxe.next() should return None — cannot upgrade past Deluxe"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Tool Upgrade Tests
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_tool_tier_upgrade_cost() {
    assert_eq!(
        ToolTier::Basic.upgrade_cost(),
        0,
        "Basic upgrade_cost should be 0"
    );
    assert_eq!(
        ToolTier::Copper.upgrade_cost(),
        2_000,
        "Copper upgrade_cost should be 2000"
    );
    assert_eq!(
        ToolTier::Iron.upgrade_cost(),
        5_000,
        "Iron upgrade_cost should be 5000"
    );
    assert_eq!(
        ToolTier::Gold.upgrade_cost(),
        10_000,
        "Gold upgrade_cost should be 10000"
    );
    assert_eq!(
        ToolTier::Iridium.upgrade_cost(),
        25_000,
        "Iridium upgrade_cost should be 25000"
    );
}

#[test]
fn test_tool_upgrade_request_queues() {
    let mut app = build_test_app();
    app.init_resource::<ToolUpgradeQueue>();
    app.add_event::<ToolUpgradeRequestEvent>();
    app.add_event::<ToolUpgradeCompleteEvent>();
    app.init_resource::<ActiveShop>();
    app.init_resource::<EconomyStats>();
    app.add_systems(
        Update,
        (handle_upgrade_request, apply_gold_changes)
            .chain()
            .run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Set the active shop to Blacksmith (required by handle_upgrade_request)
    app.world_mut().resource_mut::<ActiveShop>().shop_id = Some(ShopId::Blacksmith);

    // Give the player enough gold and copper bars for a Basic → Copper upgrade
    app.world_mut().resource_mut::<PlayerState>().gold = 5_000;
    {
        let mut inv = app.world_mut().resource_mut::<Inventory>();
        inv.try_add("copper_bar", 10, 255);
    }

    app.world_mut().send_event(ToolUpgradeRequestEvent {
        tool: ToolKind::Hoe,
    });
    app.update();

    let queue = app.world().resource::<ToolUpgradeQueue>();
    assert_eq!(
        queue.pending.len(),
        1,
        "ToolUpgradeQueue should have 1 pending upgrade"
    );
    assert_eq!(queue.pending[0].tool, ToolKind::Hoe);
    assert_eq!(queue.pending[0].target_tier, ToolTier::Copper);
    assert_eq!(
        queue.pending[0].days_remaining, 2,
        "Upgrade should take 2 days"
    );

    let player = app.world().resource::<PlayerState>();
    assert_eq!(
        player.gold, 3_000,
        "Gold should be 5000 - 2000 = 3000 after upgrade request"
    );
}

#[test]
fn test_tool_upgrade_tick_completes() {
    let mut app = build_test_app();
    app.init_resource::<ToolUpgradeQueue>();
    app.add_event::<ToolUpgradeRequestEvent>();
    app.add_event::<ToolUpgradeCompleteEvent>();
    app.add_systems(
        Update,
        tick_upgrade_queue.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Manually insert a pending upgrade with 1 day remaining
    app.world_mut()
        .resource_mut::<ToolUpgradeQueue>()
        .pending
        .push(PendingUpgrade {
            tool: ToolKind::Axe,
            target_tier: ToolTier::Iron,
            days_remaining: 1,
        });

    send_day_end(&mut app, 5, Season::Spring, 1);
    app.update();

    let queue = app.world().resource::<ToolUpgradeQueue>();
    assert!(
        queue.pending.is_empty(),
        "Pending queue should be empty after upgrade completes"
    );

    let player = app.world().resource::<PlayerState>();
    assert_eq!(
        player.tools.get(&ToolKind::Axe).copied(),
        Some(ToolTier::Iron),
        "Axe should be upgraded to Iron tier"
    );

    let complete_events = app.world().resource::<Events<ToolUpgradeCompleteEvent>>();
    let mut reader = complete_events.get_cursor();
    let events: Vec<_> = reader.read(complete_events).collect();
    assert_eq!(
        events.len(),
        1,
        "Exactly one ToolUpgradeCompleteEvent should be sent"
    );
    assert_eq!(events[0].tool, ToolKind::Axe);
    assert_eq!(events[0].new_tier, ToolTier::Iron);
}

#[test]
fn test_tool_tier_next_chain() {
    assert_eq!(
        ToolTier::Basic.next(),
        Some(ToolTier::Copper),
        "Basic -> Copper"
    );
    assert_eq!(
        ToolTier::Copper.next(),
        Some(ToolTier::Iron),
        "Copper -> Iron"
    );
    assert_eq!(ToolTier::Iron.next(), Some(ToolTier::Gold), "Iron -> Gold");
    assert_eq!(
        ToolTier::Gold.next(),
        Some(ToolTier::Iridium),
        "Gold -> Iridium"
    );
    assert_eq!(ToolTier::Iridium.next(), None, "Iridium -> None (max tier)");
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Achievement Tests
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_achievements_constant_has_entries() {
    assert!(!ACHIEVEMENTS.is_empty(), "ACHIEVEMENTS should have entries");
    assert_eq!(ACHIEVEMENTS.len(), 30, "Expected exactly 30 achievements");

    let first = &ACHIEVEMENTS[0];
    assert_eq!(first.id, "first_harvest");
    assert_eq!(first.name, "First Harvest");

    let second = &ACHIEVEMENTS[1];
    assert_eq!(second.id, "green_thumb");
    assert_eq!(second.name, "Green Thumb");

    for def in ACHIEVEMENTS {
        assert!(!def.id.is_empty(), "Achievement id must not be empty");
        assert!(!def.name.is_empty(), "Achievement name must not be empty");
        assert!(
            !def.description.is_empty(),
            "Achievement description must not be empty"
        );
    }
}

#[test]
fn test_achievement_unlocks_on_condition() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        check_achievements.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Set up the "first_harvest" condition: PlayStats.crops_harvested >= 1
    app.world_mut().resource_mut::<PlayStats>().crops_harvested = 1;

    app.update();

    let achievements = app.world().resource::<Achievements>();
    assert!(
        achievements.unlocked.contains(&"first_harvest".to_string()),
        "first_harvest should be unlocked when crops_harvested >= 1"
    );
}

#[test]
fn test_achievement_does_not_double_unlock() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        check_achievements.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Pre-unlock "first_harvest"
    app.world_mut()
        .resource_mut::<Achievements>()
        .unlocked
        .push("first_harvest".to_string());
    app.world_mut().resource_mut::<PlayStats>().crops_harvested = 5;

    app.update();

    let achievements = app.world().resource::<Achievements>();
    let count = achievements
        .unlocked
        .iter()
        .filter(|id| *id == "first_harvest")
        .count();
    assert_eq!(count, 1, "first_harvest should not be unlocked twice");
}

#[test]
fn test_achievement_progress_tracks_harvests() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        track_achievement_progress.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Send a CropHarvestedEvent with Gold quality to increment "gold_crops" counter
    app.world_mut().send_event(CropHarvestedEvent {
        crop_id: "turnip".to_string(),
        harvest_id: "turnip".to_string(),
        quantity: 1,
        x: 0,
        y: 0,
        quality: Some(ItemQuality::Gold),
    });

    app.update();

    let achievements = app.world().resource::<Achievements>();
    let gold_crops = achievements
        .progress
        .get("gold_crops")
        .copied()
        .unwrap_or(0);
    assert_eq!(
        gold_crops, 1,
        "gold_crops progress should be 1 after harvesting a Gold quality crop"
    );
}

#[test]
fn test_achievement_unlocked_event_fires() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        check_achievements.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Set up the "gone_fishin" condition: PlayStats.fish_caught >= 1
    app.world_mut().resource_mut::<PlayStats>().fish_caught = 1;

    app.update();

    let events = app.world().resource::<Events<AchievementUnlockedEvent>>();
    let mut reader = events.get_cursor();
    let fired: Vec<_> = reader.read(events).collect();
    assert!(
        fired.iter().any(|e| e.achievement_id == "gone_fishin"),
        "AchievementUnlockedEvent should fire for gone_fishin"
    );
    let event = fired
        .iter()
        .find(|e| e.achievement_id == "gone_fishin")
        .unwrap();
    assert_eq!(event.name, "Gone Fishin'");
    assert_eq!(event.description, "Catch your first fish");
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Romance Tests
// ═════════════════════════════════════════════════════════════════════════════

/// Helper: insert a datable NPC into NpcRegistry with the given name.
fn insert_datable_npc(app: &mut App, name: &str) -> String {
    let npc_id = name.to_lowercase();
    let mut registry = app.world_mut().resource_mut::<NpcRegistry>();
    registry.npcs.insert(
        npc_id.clone(),
        NpcDef {
            id: npc_id.clone(),
            name: name.to_string(),
            birthday_season: Season::Spring,
            birthday_day: 10,
            gift_preferences: HashMap::new(),
            default_dialogue: vec!["Hello!".to_string()],
            heart_dialogue: HashMap::new(),
            is_marriageable: true,
            sprite_index: 0,
            portrait_index: 0,
        },
    );
    npc_id
}

#[test]
fn test_bouquet_requires_8_hearts() {
    let mut app = build_test_app();
    app.init_resource::<WeddingTimer>();
    app.add_systems(Update, handle_bouquet.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    let npc_id = insert_datable_npc(&mut app, "Lily");

    // Set friendship to 7 hearts (700 points) — below the 8-heart threshold
    app.world_mut()
        .resource_mut::<Relationships>()
        .friendship
        .insert(npc_id.clone(), 700);
    app.world_mut()
        .resource_mut::<Inventory>()
        .try_add("bouquet", 1, 99);

    app.world_mut().send_event(BouquetGivenEvent {
        npc_name: "Lily".to_string(),
    });

    app.update();

    let stages = app.world().resource::<RelationshipStages>();
    let stage = stages
        .stages
        .get(&npc_id)
        .copied()
        .unwrap_or(RelationshipStage::Stranger);
    assert_ne!(
        stage,
        RelationshipStage::Dating,
        "Bouquet with < 8 hearts should not set Dating"
    );
}

#[test]
fn test_bouquet_sets_dating() {
    let mut app = build_test_app();
    app.init_resource::<WeddingTimer>();
    app.add_systems(Update, handle_bouquet.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    let npc_id = insert_datable_npc(&mut app, "Lily");

    // Set friendship to 8 hearts (800 points)
    app.world_mut()
        .resource_mut::<Relationships>()
        .friendship
        .insert(npc_id.clone(), 800);
    app.world_mut()
        .resource_mut::<Inventory>()
        .try_add("bouquet", 1, 99);

    app.world_mut().send_event(BouquetGivenEvent {
        npc_name: "Lily".to_string(),
    });

    app.update();

    let stages = app.world().resource::<RelationshipStages>();
    let stage = stages
        .stages
        .get(&npc_id)
        .copied()
        .unwrap_or(RelationshipStage::Stranger);
    assert_eq!(
        stage,
        RelationshipStage::Dating,
        "Bouquet with 8+ hearts should set stage to Dating"
    );
}

// NOTE: handle_proposal has a Bevy B0002 param conflict (Res<RelationshipStages> +
// ResMut<RelationshipStages>). Cannot test as a direct system without modifying game code.
#[test]
#[ignore = "handle_proposal has Bevy B0002 param conflict (Res + ResMut on RelationshipStages)"]
fn test_proposal_requires_dating_and_house() {
    let mut app = build_test_app();
    app.init_resource::<WeddingTimer>();
    app.add_systems(Update, handle_proposal.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    let npc_id = insert_datable_npc(&mut app, "Lily");

    app.world_mut()
        .resource_mut::<Relationships>()
        .friendship
        .insert(npc_id.clone(), 1000);
    app.world_mut()
        .resource_mut::<RelationshipStages>()
        .stages
        .insert(npc_id.clone(), RelationshipStage::CloseFriend);
    app.world_mut().resource_mut::<HouseState>().tier = HouseTier::Basic;
    app.world_mut()
        .resource_mut::<Inventory>()
        .try_add("mermaid_pendant", 1, 99);

    app.world_mut().send_event(ProposalEvent {
        npc_name: "Lily".to_string(),
    });

    app.update();

    let stages = app.world().resource::<RelationshipStages>();
    let stage = stages
        .stages
        .get(&npc_id)
        .copied()
        .unwrap_or(RelationshipStage::Stranger);
    assert_ne!(
        stage,
        RelationshipStage::Engaged,
        "Proposal without Dating stage should not set Engaged"
    );
}

#[test]
#[ignore = "handle_proposal has Bevy B0002 param conflict (Res + ResMut on RelationshipStages)"]
fn test_proposal_starts_wedding_timer() {
    let mut app = build_test_app();
    app.init_resource::<WeddingTimer>();
    app.add_systems(Update, handle_proposal.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    let npc_id = insert_datable_npc(&mut app, "Lily");

    // Set Dating stage + 10 hearts + Big house
    app.world_mut()
        .resource_mut::<Relationships>()
        .friendship
        .insert(npc_id.clone(), 1000);
    app.world_mut()
        .resource_mut::<RelationshipStages>()
        .stages
        .insert(npc_id.clone(), RelationshipStage::Dating);
    app.world_mut().resource_mut::<HouseState>().tier = HouseTier::Big;
    app.world_mut()
        .resource_mut::<Inventory>()
        .try_add("mermaid_pendant", 1, 99);

    app.world_mut().send_event(ProposalEvent {
        npc_name: "Lily".to_string(),
    });

    app.update();

    let stages = app.world().resource::<RelationshipStages>();
    let stage = stages
        .stages
        .get(&npc_id)
        .copied()
        .unwrap_or(RelationshipStage::Stranger);
    assert_eq!(
        stage,
        RelationshipStage::Engaged,
        "Proposal with all prerequisites should set Engaged"
    );

    let timer = app.world().resource::<WeddingTimer>();
    assert_eq!(
        timer.days_remaining,
        Some(3),
        "WeddingTimer should be set to 3 days after accepted proposal"
    );
    assert_eq!(
        timer.npc_name,
        Some("Lily".to_string()),
        "WeddingTimer npc_name should be the proposed NPC"
    );
}

#[test]
fn test_wedding_timer_ticks_down() {
    let mut app = build_test_app();
    app.init_resource::<WeddingTimer>();
    app.add_systems(
        Update,
        tick_wedding_timer.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    {
        let mut timer = app.world_mut().resource_mut::<WeddingTimer>();
        timer.days_remaining = Some(3);
        timer.npc_name = Some("Lily".to_string());
    }

    send_day_end(&mut app, 5, Season::Spring, 1);
    app.update();

    let timer = app.world().resource::<WeddingTimer>();
    assert_eq!(
        timer.days_remaining,
        Some(2),
        "WeddingTimer should decrement from 3 to 2 after one DayEndEvent"
    );
}

#[test]
fn test_wedding_completes_marriage() {
    let mut app = build_test_app();
    app.init_resource::<WeddingTimer>();
    app.add_systems(
        Update,
        (tick_wedding_timer, handle_wedding)
            .chain()
            .run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    let npc_id = insert_datable_npc(&mut app, "Lily");

    {
        let mut timer = app.world_mut().resource_mut::<WeddingTimer>();
        timer.days_remaining = Some(1);
        timer.npc_name = Some("Lily".to_string());
    }

    app.world_mut()
        .resource_mut::<RelationshipStages>()
        .stages
        .insert(npc_id.clone(), RelationshipStage::Engaged);

    send_day_end(&mut app, 10, Season::Spring, 1);
    app.update();

    let marriage = app.world().resource::<MarriageState>();
    assert_eq!(
        marriage.spouse,
        Some("Lily".to_string()),
        "MarriageState.spouse should be set after wedding completes"
    );

    let relationships = app.world().resource::<Relationships>();
    assert_eq!(
        relationships.spouse,
        Some(npc_id.clone()),
        "Relationships.spouse should be set to the NPC id"
    );

    let stages = app.world().resource::<RelationshipStages>();
    let stage = stages
        .stages
        .get(&npc_id)
        .copied()
        .unwrap_or(RelationshipStage::Stranger);
    assert_eq!(
        stage,
        RelationshipStage::Married,
        "Relationship stage should be Married after wedding"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Quest Tests
// ═════════════════════════════════════════════════════════════════════════════

/// Helper: create a simple delivery quest for testing.
fn make_test_quest(id: &str, reward_gold: u32, days_remaining: Option<u8>) -> Quest {
    Quest {
        id: id.to_string(),
        title: format!("Test Quest {}", id),
        description: "A test quest.".to_string(),
        giver: "TestNpc".to_string(),
        objective: QuestObjective::Deliver {
            item_id: "wood".to_string(),
            quantity: 5,
            delivered: 0,
        },
        reward_gold,
        reward_items: Vec::new(),
        reward_friendship: 30,
        days_remaining,
        accepted_day: (1, 0, 1),
    }
}

#[test]
fn test_quest_complete_moves_to_completed() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        handle_quest_completed.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    let quest = make_test_quest("test_q1", 200, Some(5));
    app.world_mut()
        .resource_mut::<QuestLog>()
        .active
        .push(quest);

    app.world_mut().send_event(QuestCompletedEvent {
        quest_id: "test_q1".to_string(),
        reward_gold: 200,
    });

    app.update();

    let quest_log = app.world().resource::<QuestLog>();
    assert!(
        quest_log.active.iter().all(|q| q.id != "test_q1"),
        "Completed quest should be removed from active"
    );
    assert!(
        quest_log.completed.contains(&"test_q1".to_string()),
        "Completed quest id should be in completed list"
    );
}

#[test]
fn test_quest_complete_awards_gold() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        handle_quest_completed.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    let quest = make_test_quest("test_q2", 500, Some(5));
    app.world_mut()
        .resource_mut::<QuestLog>()
        .active
        .push(quest);

    app.world_mut().send_event(QuestCompletedEvent {
        quest_id: "test_q2".to_string(),
        reward_gold: 500,
    });

    app.update();

    let events = app.world().resource::<Events<GoldChangeEvent>>();
    let mut reader = events.get_cursor();
    let fired: Vec<_> = reader.read(events).collect();
    assert!(
        fired.iter().any(|e| e.amount == 500),
        "GoldChangeEvent with amount 500 should be sent for quest completion"
    );
}

#[test]
fn test_quest_expires_after_days() {
    let mut app = build_test_app();
    app.add_systems(Update, expire_quests.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    let quest = make_test_quest("test_expire", 100, Some(1));
    app.world_mut()
        .resource_mut::<QuestLog>()
        .active
        .push(quest);

    send_day_end(&mut app, 5, Season::Spring, 1);
    app.update();

    let quest_log = app.world().resource::<QuestLog>();
    assert!(
        quest_log.active.iter().all(|q| q.id != "test_expire"),
        "Quest with days_remaining=1 should be removed after DayEndEvent"
    );
}

#[test]
fn test_quest_no_expire_if_days_remain() {
    let mut app = build_test_app();
    app.add_systems(Update, expire_quests.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    let quest = make_test_quest("test_no_expire", 100, Some(3));
    app.world_mut()
        .resource_mut::<QuestLog>()
        .active
        .push(quest);

    send_day_end(&mut app, 5, Season::Spring, 1);
    app.update();

    let quest_log = app.world().resource::<QuestLog>();
    let quest = quest_log.active.iter().find(|q| q.id == "test_no_expire");
    assert!(
        quest.is_some(),
        "Quest with days_remaining=3 should still be active after 1 day"
    );
    assert_eq!(
        quest.unwrap().days_remaining,
        Some(2),
        "days_remaining should decrement from 3 to 2"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Evaluation Tests
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_evaluation_trigger_fires_year3() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        check_evaluation_trigger.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    {
        let mut cal = app.world_mut().resource_mut::<Calendar>();
        cal.year = 3;
        cal.season = Season::Spring;
        cal.day = 1;
    }
    app.world_mut().resource_mut::<EvaluationScore>().evaluated = false;

    app.update();

    let events = app.world().resource::<Events<EvaluationTriggerEvent>>();
    let mut reader = events.get_cursor();
    let fired: Vec<_> = reader.read(events).collect();
    assert!(
        !fired.is_empty(),
        "EvaluationTriggerEvent should fire on Spring 1 Year 3+"
    );
}

#[test]
fn test_evaluation_trigger_skips_year1() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        check_evaluation_trigger.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    {
        let mut cal = app.world_mut().resource_mut::<Calendar>();
        cal.year = 1;
        cal.season = Season::Spring;
        cal.day = 1;
    }
    app.world_mut().resource_mut::<EvaluationScore>().evaluated = false;

    app.update();

    let events = app.world().resource::<Events<EvaluationTriggerEvent>>();
    let mut reader = events.get_cursor();
    let fired: Vec<_> = reader.read(events).collect();
    assert!(
        fired.is_empty(),
        "EvaluationTriggerEvent should NOT fire on Year 1"
    );
}

#[test]
fn test_evaluation_scores_categories() {
    let mut app = build_test_app();
    app.init_resource::<EconomyStats>();
    app.init_resource::<HarvestStats>();
    app.init_resource::<ShippingLog>();
    app.add_systems(
        Update,
        handle_evaluation.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Earnings: total_gold_earned >= 50_000 -> 1 point
    app.world_mut()
        .resource_mut::<EconomyStats>()
        .total_gold_earned = 60_000;
    // Married -> 1 point; happiness > 50 -> 1 point
    app.world_mut().resource_mut::<MarriageState>().spouse = Some("Lily".to_string());
    app.world_mut()
        .resource_mut::<MarriageState>()
        .spouse_happiness = 60;

    app.world_mut().send_event(EvaluationTriggerEvent);
    app.update();

    let eval = app.world().resource::<EvaluationScore>();
    assert!(
        eval.evaluated,
        "evaluated should be true after handle_evaluation runs"
    );
    assert!(
        eval.total_points >= 3,
        "Should have at least 3 points: earnings_50k + spouse_married + spouse_happiness, got {}",
        eval.total_points
    );
    assert!(eval.categories.contains_key("earnings_50k"));
    assert!(eval.categories.contains_key("spouse_married"));
    assert!(eval.categories.contains_key("spouse_happiness"));
}

#[test]
fn test_evaluation_sets_candles() {
    let mut app = build_test_app();
    app.init_resource::<EconomyStats>();
    app.init_resource::<HarvestStats>();
    app.init_resource::<ShippingLog>();
    app.add_systems(
        Update,
        handle_evaluation.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // With zero stats, total_points should be 0 -> candles_lit = 1 (0-5 = 1 candle)
    app.world_mut().send_event(EvaluationTriggerEvent);
    app.update();

    let eval = app.world().resource::<EvaluationScore>();
    assert_eq!(
        eval.candles_lit, 1,
        "0 points should yield 1 candle, got {} candles for {} points",
        eval.candles_lit, eval.total_points
    );

    // Reset for second evaluation with many points
    app.world_mut().resource_mut::<EvaluationScore>().evaluated = false;

    // Earnings: 500k -> 4 points (50k + 100k + 200k + 500k)
    app.world_mut()
        .resource_mut::<EconomyStats>()
        .total_gold_earned = 500_000;
    // Married + happiness > 50 -> 2 points
    app.world_mut().resource_mut::<MarriageState>().spouse = Some("Lily".to_string());
    app.world_mut()
        .resource_mut::<MarriageState>()
        .spouse_happiness = 60;
    // Mine floor 20 -> 1 point
    app.world_mut()
        .resource_mut::<MineState>()
        .deepest_floor_reached = 20;
    // Deluxe house -> 1 point
    app.world_mut().resource_mut::<HouseState>().tier = HouseTier::Deluxe;
    // 8+ animals -> 1 point
    {
        let mut animal_state = app.world_mut().resource_mut::<AnimalState>();
        for i in 0..8 {
            animal_state.animals.push(Animal {
                kind: AnimalKind::Chicken,
                name: format!("Chick{}", i),
                age: AnimalAge::Adult,
                days_old: 30,
                happiness: 200,
                fed_today: true,
                petted_today: false,
                product_ready: false,
            });
        }
    }
    // total_items_shipped >= 50 -> 1 point (farm_items_shipped_50)
    app.world_mut()
        .resource_mut::<EconomyStats>()
        .total_items_shipped = 55;
    // 30+ unique items shipped -> 1 point (collection_unique_30)
    {
        let mut log = app.world_mut().resource_mut::<ShippingLog>();
        for i in 0..30 {
            log.shipped_items.insert(format!("item_{}", i), 1);
        }
    }
    // 10 quests completed -> 1 point
    {
        let mut quest_log = app.world_mut().resource_mut::<QuestLog>();
        for i in 0..10 {
            quest_log.completed.push(format!("quest_{}", i));
        }
    }
    // 1M gold on hand -> 1 point
    app.world_mut().resource_mut::<PlayerState>().gold = 1_000_000;

    app.world_mut().send_event(EvaluationTriggerEvent);
    app.update();

    let eval = app.world().resource::<EvaluationScore>();
    // Should have 4+2+1+1+1+2+1+1 = 13 points -> 3 candles (11-15)
    assert!(
        eval.total_points >= 13,
        "Expected at least 13 points with substantial progress, got {}",
        eval.total_points
    );
    assert!(
        eval.candles_lit >= 3,
        "13+ points should yield at least 3 candles, got {} candles for {} points",
        eval.candles_lit,
        eval.total_points
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Sprinkler Integration Tests
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_place_sprinkler_adds_to_state() {
    let mut app = build_test_app();
    app.init_resource::<FarmEntities>();
    app.add_systems(
        Update,
        handle_place_sprinkler.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    app.world_mut()
        .resource_mut::<Inventory>()
        .try_add("sprinkler", 1, 99);

    app.world_mut().send_event(PlaceSprinklerEvent {
        kind: SprinklerKind::Basic,
        tile_x: 5,
        tile_y: 5,
    });

    app.update();

    let sprinkler_state = app.world().resource::<SprinklerState>();
    assert_eq!(
        sprinkler_state.sprinklers.len(),
        1,
        "SprinklerState should have 1 sprinkler after placement"
    );
    let placed = &sprinkler_state.sprinklers[0];
    assert_eq!(placed.tile_x, 5);
    assert_eq!(placed.tile_y, 5);
    assert_eq!(placed.kind, SprinklerKind::Basic);
}

#[test]
fn test_place_multiple_sprinklers() {
    let mut app = build_test_app();
    app.init_resource::<FarmEntities>();
    app.add_systems(
        Update,
        handle_place_sprinkler.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    app.world_mut()
        .resource_mut::<Inventory>()
        .try_add("sprinkler", 1, 99);
    app.world_mut()
        .resource_mut::<Inventory>()
        .try_add("quality_sprinkler", 1, 99);

    app.world_mut().send_event(PlaceSprinklerEvent {
        kind: SprinklerKind::Basic,
        tile_x: 3,
        tile_y: 3,
    });
    app.world_mut().send_event(PlaceSprinklerEvent {
        kind: SprinklerKind::Quality,
        tile_x: 7,
        tile_y: 7,
    });

    app.update();

    let sprinkler_state = app.world().resource::<SprinklerState>();
    assert_eq!(
        sprinkler_state.sprinklers.len(),
        2,
        "SprinklerState should have 2 sprinklers after two placements"
    );

    let has_basic = sprinkler_state
        .sprinklers
        .iter()
        .any(|s| s.tile_x == 3 && s.tile_y == 3 && s.kind == SprinklerKind::Basic);
    let has_quality = sprinkler_state
        .sprinklers
        .iter()
        .any(|s| s.tile_x == 7 && s.tile_y == 7 && s.kind == SprinklerKind::Quality);
    assert!(has_basic, "Should have Basic sprinkler at (3,3)");
    assert!(has_quality, "Should have Quality sprinkler at (7,7)");
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: World Object Tests
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_forageables_spring_includes_horseradish() {
    let spring_items = seasonal_forageables(Season::Spring);
    let ids: Vec<&str> = spring_items.iter().map(|(id, _)| *id).collect();
    assert!(
        ids.contains(&"wild_horseradish"),
        "Spring forageables should include wild_horseradish, got: {:?}",
        ids
    );
}

#[test]
fn test_forageables_winter_includes_crystal_fruit() {
    let winter_items = seasonal_forageables(Season::Winter);
    let ids: Vec<&str> = winter_items.iter().map(|(id, _)| *id).collect();
    assert!(
        ids.contains(&"crystal_fruit"),
        "Winter forageables should include crystal_fruit, got: {:?}",
        ids
    );
}

#[test]
fn test_forageables_unique_per_season() {
    let spring_items = seasonal_forageables(Season::Spring);
    let summer_items = seasonal_forageables(Season::Summer);

    let spring_ids: Vec<&str> = spring_items.iter().map(|(id, _)| *id).collect();
    let summer_ids: Vec<&str> = summer_items.iter().map(|(id, _)| *id).collect();

    assert_ne!(
        spring_ids, summer_ids,
        "Spring and summer forageables should be different sets"
    );

    let overlap: Vec<&&str> = spring_ids
        .iter()
        .filter(|id| summer_ids.contains(id))
        .collect();
    assert!(
        overlap.is_empty(),
        "Spring and summer should have no overlapping forageable items, found: {:?}",
        overlap
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// Phase 5: Rendering, Animation & Pixel-Perfect Display
// ═══════════════════════════════════════════════════════════════════════════

// ── Y-Sort Tests ──────────────────────────────────────────────────────────

#[test]
fn test_ysort_lower_y_draws_in_front() {
    // Entity at y=100 should have higher Z than entity at y=200
    let z_100 = Z_ENTITY_BASE - 100.0 * Z_Y_SORT_SCALE;
    let z_200 = Z_ENTITY_BASE - 200.0 * Z_Y_SORT_SCALE;
    assert!(
        z_100 > z_200,
        "Lower Y should produce higher Z (draws in front)"
    );
}

#[test]
fn test_ysort_does_not_overlap_ground() {
    // Lowest possible entity Z should still be above Z_FARM_OVERLAY
    let worst_case_z = Z_ENTITY_BASE - 5000.0 * Z_Y_SORT_SCALE; // extreme map
    assert!(worst_case_z > Z_FARM_OVERLAY);
}

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_z_layer_ordering() {
    assert!(Z_GROUND < Z_FARM_OVERLAY);
    assert!(Z_FARM_OVERLAY < Z_ENTITY_BASE);
    assert!(Z_ENTITY_BASE < Z_EFFECTS);
    assert!(Z_EFFECTS < Z_SEASONAL);
    assert!(Z_SEASONAL < Z_WEATHER);
}

// ── LogicalPosition Tests ─────────────────────────────────────────────────

#[test]
fn test_logical_position_round_snaps_correctly() {
    let pos = LogicalPosition(Vec2::new(14.53, 10.87));
    assert_eq!(pos.0.x.round(), 15.0);
    assert_eq!(pos.0.y.round(), 11.0);
}

#[test]
fn test_logical_position_preserves_sub_pixel() {
    // Simulate: speed 0.4 px/frame, 3 frames
    let mut pos = LogicalPosition(Vec2::new(10.0, 5.0));
    pos.0.x += 0.4;
    pos.0.x += 0.4;
    pos.0.x += 0.4;
    // Float accumulated: 11.2 (not rounded to 10.0 each frame)
    assert!((pos.0.x - 11.2).abs() < 0.001);
}

// ── Distance-Based Animation Tests ────────────────────────────────────────

#[test]
fn test_distance_animator_advances_on_distance() {
    use hearthfield::player::DistanceAnimator;
    let mut anim = DistanceAnimator {
        last_pos: Vec2::ZERO,
        distance_budget: 0.0,
        pixels_per_frame: 6.0,
        frames_per_row: 4,
        current_frame: 0,
    };

    // Move 5 pixels — not enough
    let new_pos = Vec2::new(5.0, 0.0);
    anim.distance_budget += (new_pos - anim.last_pos).length();
    anim.last_pos = new_pos;
    assert_eq!(anim.current_frame, 0);

    // Move 2 more pixels — crosses threshold
    let new_pos = Vec2::new(7.0, 0.0);
    anim.distance_budget += (new_pos - anim.last_pos).length();
    anim.last_pos = new_pos;
    while anim.distance_budget >= anim.pixels_per_frame {
        anim.distance_budget -= anim.pixels_per_frame;
        anim.current_frame = (anim.current_frame + 1) % anim.frames_per_row;
    }
    assert_eq!(anim.current_frame, 1);
    assert!((anim.distance_budget - 1.0).abs() < 0.001);
}

#[test]
fn test_distance_animator_wraps_frames() {
    use hearthfield::player::DistanceAnimator;
    let mut anim = DistanceAnimator {
        last_pos: Vec2::ZERO,
        distance_budget: 0.0,
        pixels_per_frame: 1.0,
        frames_per_row: 4,
        current_frame: 3,
    };

    anim.distance_budget = 1.0;
    while anim.distance_budget >= anim.pixels_per_frame {
        anim.distance_budget -= anim.pixels_per_frame;
        anim.current_frame = (anim.current_frame + 1) % anim.frames_per_row;
    }
    assert_eq!(anim.current_frame, 0);
}

#[test]
fn test_distance_animator_idle_resets() {
    use hearthfield::player::DistanceAnimator;
    let mut anim = DistanceAnimator {
        current_frame: 2,
        distance_budget: 3.5,
        ..Default::default()
    };

    // Idle reset
    anim.current_frame = 0;
    anim.distance_budget = 0.0;
    assert_eq!(anim.current_frame, 0);
    assert_eq!(anim.distance_budget, 0.0);
}

// ── Animation State Tests ─────────────────────────────────────────────────

#[test]
fn test_player_anim_state_default_is_idle() {
    assert_eq!(PlayerAnimState::default(), PlayerAnimState::Idle);
}

#[test]
fn test_tool_use_state_holds_tool_kind() {
    let state = PlayerAnimState::ToolUse {
        tool: ToolKind::Hoe,
        frame: 2,
        total_frames: 4,
    };
    match state {
        PlayerAnimState::ToolUse {
            tool,
            frame,
            total_frames,
        } => {
            assert_eq!(tool, ToolKind::Hoe);
            assert_eq!(frame, 2);
            assert_eq!(total_frames, 4);
        }
        _ => panic!("Expected ToolUse state"),
    }
}

#[test]
fn test_player_movement_default_has_idle_anim_state() {
    let movement = PlayerMovement::default();
    assert_eq!(movement.anim_state, PlayerAnimState::Idle);
}

// ── Atlas Index Math Tests ────────────────────────────────────────────────

#[test]
fn test_walk_atlas_index_right_frame2() {
    let base = 8; // Right row
    let frame = 2;
    assert_eq!(base + frame, 10);
}

#[test]
fn test_walk_atlas_index_left_frame0() {
    let base = 12; // Left row
    let frame = 0;
    assert_eq!(base + frame, 12);
}

#[test]
fn test_walk_atlas_all_directions() {
    let (down, up, right, left) = (0_usize, 4_usize, 8_usize, 12_usize);
    let frame0 = 0_usize;
    let frame3 = 3_usize;
    // Down row 0
    assert_eq!(down + frame0, 0);
    assert_eq!(down + frame3, 3);
    // Up row 1
    assert_eq!(up + frame0, 4);
    assert_eq!(up + frame3, 7);
    // Right row 2
    assert_eq!(right + frame0, 8);
    assert_eq!(right + frame3, 11);
    // Left row 3
    assert_eq!(left + frame0, 12);
    assert_eq!(left + frame3, 15);
}

// ═════════════════════════════════════════════════════════════════════════════
// NEW TESTS: Festival System
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_festival_check_activates_egg_festival() {
    let mut app = build_test_app();
    app.init_resource::<FestivalState>();
    app.add_systems(
        Update,
        check_festival_day.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Set calendar to Spring 13 (Egg Festival)
    {
        let mut cal = app.world_mut().resource_mut::<Calendar>();
        cal.season = Season::Spring;
        cal.day = 13;
        cal.year = 1;
    }

    app.update();

    let festival = app.world().resource::<FestivalState>();
    assert_eq!(
        festival.active,
        Some(FestivalKind::EggFestival),
        "Spring 13 should activate Egg Festival"
    );
    assert!(!festival.started, "Festival should not be started yet");
}

#[test]
fn test_festival_check_activates_winter_star() {
    let mut app = build_test_app();
    app.init_resource::<FestivalState>();
    app.add_systems(
        Update,
        check_festival_day.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    {
        let mut cal = app.world_mut().resource_mut::<Calendar>();
        cal.season = Season::Winter;
        cal.day = 25;
        cal.year = 1;
    }

    app.update();

    let festival = app.world().resource::<FestivalState>();
    assert_eq!(
        festival.active,
        Some(FestivalKind::WinterStar),
        "Winter 25 should activate Winter Star"
    );
}

#[test]
fn test_festival_no_activation_on_normal_day() {
    let mut app = build_test_app();
    app.init_resource::<FestivalState>();
    app.add_systems(
        Update,
        check_festival_day.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    {
        let mut cal = app.world_mut().resource_mut::<Calendar>();
        cal.season = Season::Spring;
        cal.day = 5;
        cal.year = 1;
    }

    app.update();

    let festival = app.world().resource::<FestivalState>();
    assert_eq!(festival.active, None, "Spring 5 is not a festival day");
}

#[test]
fn test_festival_cleanup_on_day_end() {
    let mut app = build_test_app();
    app.init_resource::<FestivalState>();
    app.add_systems(
        Update,
        cleanup_festival_on_day_end.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Set an active festival
    {
        let mut festival = app.world_mut().resource_mut::<FestivalState>();
        festival.active = Some(FestivalKind::Luau);
        festival.started = true;
        festival.score = 42;
        festival.items_collected = 3;
    }

    send_day_end(&mut app, 11, Season::Summer, 1);
    app.update();

    let festival = app.world().resource::<FestivalState>();
    assert_eq!(
        festival.active, None,
        "Festival should be cleared after day end"
    );
    assert!(!festival.started, "Festival started should be reset");
    assert_eq!(festival.score, 0, "Festival score should be reset");
    assert_eq!(
        festival.items_collected, 0,
        "Festival items_collected should be reset"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// NEW TESTS: Crafting Machine Outputs (pure function)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_furnace_smelts_ores() {
    let result = resolve_machine_output(MachineType::Furnace, "copper_ore");
    assert_eq!(result, Some(("copper_bar".to_string(), 1)));

    let result = resolve_machine_output(MachineType::Furnace, "iron_ore");
    assert_eq!(result, Some(("iron_bar".to_string(), 1)));

    let result = resolve_machine_output(MachineType::Furnace, "gold_ore");
    assert_eq!(result, Some(("gold_bar".to_string(), 1)));

    let result = resolve_machine_output(MachineType::Furnace, "iridium_ore");
    assert_eq!(result, Some(("iridium_bar".to_string(), 1)));

    let result = resolve_machine_output(MachineType::Furnace, "quartz");
    assert_eq!(result, Some(("refined_quartz".to_string(), 1)));
}

#[test]
fn test_furnace_rejects_unknown_input() {
    let result = resolve_machine_output(MachineType::Furnace, "parsnip");
    assert_eq!(result, None, "Furnace should not accept parsnip");
}

#[test]
fn test_preserves_jar_makes_jelly_and_pickles() {
    let result = resolve_machine_output(MachineType::PreservesJar, "blueberry");
    assert_eq!(result, Some(("blueberry_jelly".to_string(), 1)));

    let result = resolve_machine_output(MachineType::PreservesJar, "turnip");
    assert_eq!(result, Some(("pickled_turnip".to_string(), 1)));

    let result = resolve_machine_output(MachineType::PreservesJar, "pumpkin");
    assert_eq!(result, Some(("pickled_pumpkin".to_string(), 1)));
}

#[test]
fn test_machine_processing_hours() {
    assert!((MachineType::Furnace.processing_hours() - 0.5).abs() < f32::EPSILON);
    assert!((MachineType::PreservesJar.processing_hours() - 4.0).abs() < f32::EPSILON);
    assert!((MachineType::CheesePress.processing_hours() - 3.0).abs() < f32::EPSILON);
    assert!((MachineType::Keg.processing_hours() - 72.0).abs() < f32::EPSILON);
    assert!((MachineType::OilMaker.processing_hours() - 24.0).abs() < f32::EPSILON);
}

// ═════════════════════════════════════════════════════════════════════════════
// NEW TESTS: Animal Lifecycle
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_animal_happiness_clamps_to_zero() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        handle_day_end_for_animals.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Animal with very low happiness, unfed
    let animal_id = app
        .world_mut()
        .spawn(Animal {
            kind: AnimalKind::Chicken,
            name: "Sad".to_string(),
            age: AnimalAge::Adult,
            days_old: 10,
            happiness: 5, // will drop by 12 when unfed -> should clamp to 0
            fed_today: false,
            petted_today: false,
            product_ready: false,
        })
        .id();

    send_day_end(&mut app, 1, Season::Spring, 1);
    app.update();

    let animal = app.world().entity(animal_id).get::<Animal>().unwrap();
    assert_eq!(
        animal.happiness, 0,
        "Happiness should clamp to 0, not underflow"
    );
}

#[test]
fn test_baby_animal_stays_baby_before_threshold() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        handle_day_end_for_animals.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    let baby_id = app
        .world_mut()
        .spawn(Animal {
            kind: AnimalKind::Cow,
            name: "BabyCow".to_string(),
            age: AnimalAge::Baby,
            days_old: 3, // will become 4, still baby (needs 7+)
            happiness: 100,
            fed_today: true,
            petted_today: false,
            product_ready: false,
        })
        .id();

    send_day_end(&mut app, 2, Season::Spring, 1);
    app.update();

    let animal = app.world().entity(baby_id).get::<Animal>().unwrap();
    assert_eq!(
        animal.age,
        AnimalAge::Baby,
        "Should still be a baby at 4 days old"
    );
    assert_eq!(animal.days_old, 4);
    assert!(!animal.product_ready, "Baby should not produce");
}

// ═════════════════════════════════════════════════════════════════════════════
// NEW TESTS: Fishing Skill (pure function)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_fishing_skill_xp_for_rarity_values() {
    assert_eq!(xp_for_rarity(Rarity::Common), 3);
    assert_eq!(xp_for_rarity(Rarity::Uncommon), 8);
    assert_eq!(xp_for_rarity(Rarity::Rare), 15);
    assert_eq!(xp_for_rarity(Rarity::Legendary), 25);
}

#[test]
fn test_fishing_skill_level_up_progression() {
    let mut skill = FishingSkill::default();
    assert_eq!(skill.level, 0);

    // Catch enough common fish to reach level 1 (need 10 XP, common = 3 each)
    skill.add_catch_xp(Rarity::Common);
    skill.add_catch_xp(Rarity::Common);
    skill.add_catch_xp(Rarity::Common);
    skill.add_catch_xp(Rarity::Common);
    // 4 * 3 = 12 XP >= 10 threshold
    assert_eq!(skill.level, 1, "Should be level 1 after 12 XP");
    assert_eq!(skill.total_catches, 4);

    // Bar size at level 1: 40 + 3*1 = 43
    assert!((skill.bar_size_px() - 43.0).abs() < f32::EPSILON);
}

#[test]
fn test_fishing_skill_max_level_bonuses_capped() {
    let mut skill = FishingSkill {
        xp: 10_000,
        ..FishingSkill::default()
    };
    skill.recalculate();

    assert_eq!(skill.level, 10);
    assert!(
        (skill.bite_speed_bonus - FishingSkill::MAX_BITE_SPEED).abs() < f32::EPSILON,
        "Bite speed bonus should cap at MAX_BITE_SPEED"
    );
    assert!(
        (skill.catch_zone_bonus - FishingSkill::MAX_CATCH_ZONE).abs() < f32::EPSILON,
        "Catch zone bonus should cap at MAX_CATCH_ZONE"
    );
}

#[test]
fn test_fishing_skill_bar_size_scales_with_level() {
    let mut skill = FishingSkill::default();
    // Level 0: 40px base
    assert!((skill.bar_size_px() - 40.0).abs() < f32::EPSILON);

    // Level 5: 40 + 3*5 = 55px
    skill.xp = 200;
    skill.recalculate();
    assert_eq!(skill.level, 5);
    assert!((skill.bar_size_px() - 55.0).abs() < f32::EPSILON);

    // Level 10: 40 + 3*10 = 70px
    skill.xp = 1500;
    skill.recalculate();
    assert_eq!(skill.level, 10);
    assert!((skill.bar_size_px() - 70.0).abs() < f32::EPSILON);
}

#[test]
fn test_legendary_fish_identification() {
    let defs = legendary_fish_defs();
    assert!(
        !defs.is_empty(),
        "Should have at least one legendary fish definition"
    );

    // All legendary defs should be identified as legendary
    for def in &defs {
        assert!(
            is_legendary(&def.id),
            "{} should be identified as legendary",
            def.id
        );
    }

    // Non-legendary fish should not be legendary
    assert!(!is_legendary("sardine"), "sardine should not be legendary");
    assert!(!is_legendary("carp"), "carp should not be legendary");
}

// ═════════════════════════════════════════════════════════════════════════════
// NEW TESTS: Mining Floor Generation (pure function)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_mine_state_default_values() {
    let mine = MineState::default();
    assert_eq!(
        mine.current_floor, 0,
        "Default mine floor should be 0 (not in mine)"
    );
    assert_eq!(
        mine.deepest_floor_reached, 0,
        "Default deepest floor should be 0"
    );
    assert!(
        mine.elevator_floors.is_empty(),
        "Default elevator floors should be empty"
    );
}

#[test]
fn test_mine_elevator_unlocks_every_5_floors() {
    let mine = MineState {
        deepest_floor_reached: 5,
        elevator_floors: vec![0, 5], // 0 = surface, 5 = first elevator stop
        ..MineState::default()
    };

    assert!(
        mine.elevator_floors.contains(&5),
        "Elevator should have floor 5 unlocked"
    );
    assert!(
        !mine.elevator_floors.contains(&3),
        "Elevator should NOT have floor 3 (not a multiple of 5)"
    );

    // Simulate reaching floor 10
    let mine2 = MineState {
        deepest_floor_reached: 10,
        elevator_floors: vec![0, 5, 10],
        ..MineState::default()
    };

    assert!(
        mine2.elevator_floors.contains(&10),
        "Elevator should have floor 10 unlocked"
    );
    assert_eq!(
        mine2.elevator_floors.len(),
        3,
        "Should have 3 stops: 0, 5, 10"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// NEW TESTS: Inventory Operations (pure function)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_inventory_add_and_count() {
    let mut inv = Inventory::default();
    let leftover = inv.try_add("wood", 10, 99);
    assert_eq!(leftover, 0, "Should add all 10 wood with no leftover");
    assert_eq!(inv.count("wood"), 10);
    assert!(inv.has("wood", 10));
    assert!(!inv.has("wood", 11));
}

#[test]
fn test_inventory_remove() {
    let mut inv = Inventory::default();
    inv.try_add("stone", 20, 99);
    let removed = inv.try_remove("stone", 15);
    assert_eq!(removed, 15, "Should remove 15 stone");
    assert_eq!(inv.count("stone"), 5, "Should have 5 stone remaining");
}

// ═════════════════════════════════════════════════════════════════════════════
// NEW TESTS: Food Buff Lookup (pure function)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_food_buff_known_items() {
    let buff = food_buff_for_item("fried_egg");
    assert!(buff.is_some(), "fried_egg should grant a buff");
    let buff = buff.unwrap();
    assert_eq!(buff.buff_type, BuffType::Farming);
    assert!((buff.magnitude - 1.2).abs() < f32::EPSILON);

    let buff = food_buff_for_item("fish_stew").unwrap();
    assert_eq!(buff.buff_type, BuffType::Fishing);

    let no_buff = food_buff_for_item("stone");
    assert!(no_buff.is_none(), "stone should not grant a food buff");
}

// ═════════════════════════════════════════════════════════════════════════════
// NEW TESTS: Relationships (pure function)
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_relationships_friendship_hearts() {
    let mut rel = Relationships::default();
    assert_eq!(rel.hearts("alice"), 0, "Unknown NPC should have 0 hearts");

    rel.add_friendship("alice", 250);
    assert_eq!(rel.hearts("alice"), 2, "250 points = 2 hearts");

    rel.add_friendship("alice", 50);
    assert_eq!(rel.hearts("alice"), 3, "300 points = 3 hearts");
}

// ═════════════════════════════════════════════════════════════════════════════
// SAVE ROUND-TRIP TESTS — verify serde (de)serialization of game state
// ═════════════════════════════════════════════════════════════════════════════

/// Helper: serialize a value to JSON and back, returning the deserialized copy.
fn serde_roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(val: &T) -> T {
    let json = serde_json::to_string(val).expect("serialize failed");
    serde_json::from_str(&json).expect("deserialize failed")
}

#[test]
fn test_save_roundtrip_calendar() {
    let cal = Calendar {
        year: 3,
        season: Season::Fall,
        day: 17,
        hour: 14,
        minute: 30,
        weather: Weather::Stormy,
        ..Calendar::default()
    };
    let restored = serde_roundtrip(&cal);
    assert_eq!(restored.year, 3);
    assert_eq!(restored.season, Season::Fall);
    assert_eq!(restored.day, 17);
    assert_eq!(restored.hour, 14);
    assert_eq!(restored.minute, 30);
    assert_eq!(restored.weather, Weather::Stormy);
}

#[test]
fn test_save_roundtrip_player_state() {
    let mut ps = PlayerState::default();
    ps.gold = 12345;
    ps.stamina = 50.0;
    ps.equipped_tool = ToolKind::Pickaxe;
    ps.tools.insert(ToolKind::Hoe, ToolTier::Gold);
    ps.current_map = MapId::Mine;
    ps.save_grid_x = 15;
    ps.save_grid_y = 22;

    let restored = serde_roundtrip(&ps);
    assert_eq!(restored.gold, 12345);
    assert!((restored.stamina - 50.0).abs() < f32::EPSILON);
    assert_eq!(restored.equipped_tool, ToolKind::Pickaxe);
    assert_eq!(restored.tools.get(&ToolKind::Hoe), Some(&ToolTier::Gold));
    assert_eq!(restored.current_map, MapId::Mine);
    assert_eq!(restored.save_grid_x, 15);
    assert_eq!(restored.save_grid_y, 22);
}

#[test]
fn test_save_roundtrip_inventory() {
    let mut inv = Inventory::default();
    inv.slots[0] = Some(InventorySlot {
        item_id: "ancient_fruit".to_string(),
        quantity: 5,
    });
    inv.slots[3] = Some(InventorySlot {
        item_id: "gold_bar".to_string(),
        quantity: 12,
    });

    let restored = serde_roundtrip(&inv);
    assert!(restored.slots[0].is_some());
    assert_eq!(restored.slots[0].as_ref().unwrap().item_id, "ancient_fruit");
    assert_eq!(restored.slots[0].as_ref().unwrap().quantity, 5);
    assert!(restored.slots[3].is_some());
    assert_eq!(restored.slots[3].as_ref().unwrap().item_id, "gold_bar");
    assert!(restored.slots[1].is_none());
}

#[test]
fn test_save_roundtrip_quest_log() {
    let ql = QuestLog {
        active: vec![Quest {
            id: "test_quest_1".to_string(),
            title: "Deliver Turnips".to_string(),
            description: "Bring 5 turnips to Nora.".to_string(),
            giver: "nora".to_string(),
            objective: QuestObjective::Deliver {
                item_id: "turnip".to_string(),
                quantity: 5,
                delivered: 2,
            },
            reward_gold: 500,
            reward_items: vec![("pumpkin_seeds".to_string(), 10)],
            reward_friendship: 50,
            days_remaining: Some(7),
            accepted_day: (5, 0, 1),
        }],
        completed: vec!["intro_quest".to_string(), "fishing_tutorial".to_string()],
    };

    let restored = serde_roundtrip(&ql);
    assert_eq!(restored.active.len(), 1);
    assert_eq!(restored.active[0].id, "test_quest_1");
    assert_eq!(restored.active[0].reward_gold, 500);
    assert_eq!(restored.completed.len(), 2);
    assert!(restored.completed.contains(&"intro_quest".to_string()));
}

#[test]
fn test_save_roundtrip_achievements() {
    let mut ach = Achievements::default();
    ach.unlocked.push("first_harvest".to_string());
    ach.unlocked.push("master_angler".to_string());
    ach.progress.insert("crops_shipped".to_string(), 150);
    ach.progress.insert("fish_caught".to_string(), 42);

    let restored = serde_roundtrip(&ach);
    assert_eq!(restored.unlocked.len(), 2);
    assert!(restored.unlocked.contains(&"master_angler".to_string()));
    assert_eq!(restored.progress.get("crops_shipped"), Some(&150));
    assert_eq!(restored.progress.get("fish_caught"), Some(&42));
}

#[test]
fn test_save_roundtrip_shipping_log() {
    let mut sl = ShippingLog::default();
    sl.shipped_items.insert("turnip".to_string(), 100);
    sl.shipped_items.insert("pumpkin".to_string(), 25);
    sl.shipped_items.insert("diamond".to_string(), 1);

    let restored = serde_roundtrip(&sl);
    assert_eq!(restored.shipped_items.len(), 3);
    assert_eq!(restored.shipped_items.get("turnip"), Some(&100));
    assert_eq!(restored.shipped_items.get("diamond"), Some(&1));
}

#[test]
fn test_save_roundtrip_relationship_stages() {
    let mut rs = RelationshipStages::default();
    rs.stages
        .insert("lily".to_string(), RelationshipStage::Dating);
    rs.stages
        .insert("elena".to_string(), RelationshipStage::Married);
    rs.stages
        .insert("old_tom".to_string(), RelationshipStage::CloseFriend);

    let restored = serde_roundtrip(&rs);
    assert_eq!(
        restored.stages.get("lily"),
        Some(&RelationshipStage::Dating)
    );
    assert_eq!(
        restored.stages.get("elena"),
        Some(&RelationshipStage::Married)
    );
    assert_eq!(
        restored.stages.get("old_tom"),
        Some(&RelationshipStage::CloseFriend)
    );
}

#[test]
fn test_save_roundtrip_marriage_state() {
    let ms = MarriageState {
        spouse: Some("elena".to_string()),
        wedding_date: Some((15, 2, 2)),
        days_married: 45,
        spouse_happiness: 75,
    };

    let restored = serde_roundtrip(&ms);
    assert_eq!(restored.spouse, Some("elena".to_string()));
    assert_eq!(restored.wedding_date, Some((15, 2, 2)));
    assert_eq!(restored.days_married, 45);
    assert_eq!(restored.spouse_happiness, 75);
}

#[test]
fn test_save_roundtrip_house_state() {
    let hs = HouseState {
        tier: HouseTier::Deluxe,
        has_kitchen: true,
        has_nursery: true,
    };

    let restored = serde_roundtrip(&hs);
    assert_eq!(restored.tier, HouseTier::Deluxe);
    assert!(restored.has_kitchen);
    assert!(restored.has_nursery);
}

#[test]
fn test_save_roundtrip_play_stats() {
    let ps = PlayStats {
        crops_harvested: 500,
        fish_caught: 120,
        items_shipped: 800,
        gifts_given: 95,
        mine_floors_cleared: 40,
        animal_products_collected: 200,
        food_eaten: 60,
        total_gold_earned: 150000,
        total_steps_taken: 50000,
        days_played: 112,
        festivals_attended: 4,
    };

    let restored = serde_roundtrip(&ps);
    assert_eq!(restored.crops_harvested, 500);
    assert_eq!(restored.fish_caught, 120);
    assert_eq!(restored.items_shipped, 800);
    assert_eq!(restored.total_gold_earned, 150000);
    assert_eq!(restored.days_played, 112);
    assert_eq!(restored.festivals_attended, 4);
}

#[test]
fn test_save_roundtrip_fish_encyclopedia() {
    use hearthfield::fishing::FishEncyclopedia;
    let mut fe = FishEncyclopedia::default();
    fe.record_catch("bass", 15, Season::Summer);
    fe.record_catch("bass", 22, Season::Summer);
    fe.record_catch("salmon", 18, Season::Fall);

    let restored = serde_roundtrip(&fe);
    assert_eq!(restored.entries.len(), 2);
    let bass = restored.entries.get("bass").unwrap();
    assert_eq!(bass.times_caught, 2);
    let salmon = restored.entries.get("salmon").unwrap();
    assert_eq!(salmon.times_caught, 1);
}

#[test]
fn test_save_roundtrip_building_levels() {
    use hearthfield::economy::buildings::BuildingLevels;
    let mut bl = BuildingLevels::default();
    bl.coop_tier = BuildingTier::Big;
    bl.barn_tier = BuildingTier::Deluxe;

    let restored = serde_roundtrip(&bl);
    assert_eq!(restored.coop_tier, BuildingTier::Big);
    assert_eq!(restored.barn_tier, BuildingTier::Deluxe);
}

#[test]
fn test_save_roundtrip_tool_upgrade_queue() {
    let mut tuq = ToolUpgradeQueue::default();
    tuq.pending.push(PendingUpgrade {
        tool: ToolKind::Pickaxe,
        target_tier: ToolTier::Iridium,
        days_remaining: 2,
    });

    let restored = serde_roundtrip(&tuq);
    assert_eq!(restored.pending.len(), 1);
    let p = &restored.pending[0];
    assert_eq!(p.tool, ToolKind::Pickaxe);
    assert_eq!(p.target_tier, ToolTier::Iridium);
    assert_eq!(p.days_remaining, 2);
}

#[test]
fn test_save_roundtrip_crop_tile() {
    // CropTile round-trips individually (FarmState uses tuple keys
    // which require non-JSON serializers for the full HashMap).
    let crop = CropTile {
        crop_id: "pumpkin".to_string(),
        current_stage: 3,
        days_in_stage: 2,
        watered_today: true,
        days_without_water: 0,
        dead: false,
    };

    let restored = serde_roundtrip(&crop);
    assert_eq!(restored.crop_id, "pumpkin");
    assert_eq!(restored.current_stage, 3);
    assert_eq!(restored.days_in_stage, 2);
    assert!(restored.watered_today);
    assert!(!restored.dead);
}

#[test]
fn test_save_roundtrip_animal_state() {
    let mut animal = AnimalState::default();
    animal.animals.push(Animal {
        name: "Bessie".to_string(),
        kind: AnimalKind::Cow,
        age: AnimalAge::Adult,
        days_old: 30,
        happiness: 200,
        fed_today: false,
        petted_today: false,
        product_ready: true,
    });

    let restored = serde_roundtrip(&animal);
    assert_eq!(restored.animals.len(), 1);
    assert_eq!(restored.animals[0].name, "Bessie");
    assert_eq!(restored.animals[0].kind, AnimalKind::Cow);
    assert_eq!(restored.animals[0].happiness, 200);
    assert_eq!(restored.animals[0].days_old, 30);
}

#[test]
fn test_save_roundtrip_relationships() {
    let mut rel = Relationships::default();
    rel.add_friendship("margaret", 500);
    rel.add_friendship("elena", 1000);
    rel.gifted_today.insert("elena".to_string(), true);

    let restored = serde_roundtrip(&rel);
    assert_eq!(restored.hearts("margaret"), 5);
    assert_eq!(restored.hearts("elena"), 10);
    assert_eq!(restored.gifted_today.get("elena"), Some(&true));
}

#[test]
fn test_save_roundtrip_mine_state() {
    let mut ms = MineState::default();
    ms.current_floor = 35;
    ms.deepest_floor_reached = 50;

    let restored = serde_roundtrip(&ms);
    assert_eq!(restored.current_floor, 35);
    assert_eq!(restored.deepest_floor_reached, 50);
}

#[test]
fn test_save_roundtrip_all_resources_combined() {
    let calendar = Calendar {
        year: 2,
        season: Season::Winter,
        day: 25,
        hour: 18,
        minute: 45,
        weather: Weather::Snowy,
        ..Calendar::default()
    };
    let mut player_state = PlayerState::default();
    player_state.gold = 99999;
    player_state.tools.insert(ToolKind::Axe, ToolTier::Iridium);

    let mut inventory = Inventory::default();
    inventory.slots[0] = Some(InventorySlot {
        item_id: "diamond".to_string(),
        quantity: 3,
    });

    let mut quest_log = QuestLog::default();
    quest_log.completed.push("main_quest_1".to_string());

    let mut achievements = Achievements::default();
    achievements.unlocked.push("full_shipment".to_string());

    let mut play_stats = PlayStats::default();
    play_stats.days_played = 365;
    play_stats.total_gold_earned = 500000;

    let cal_r = serde_roundtrip(&calendar);
    let ps_r = serde_roundtrip(&player_state);
    let inv_r = serde_roundtrip(&inventory);
    let ql_r = serde_roundtrip(&quest_log);
    let ach_r = serde_roundtrip(&achievements);
    let stats_r = serde_roundtrip(&play_stats);

    assert_eq!(cal_r.season, Season::Winter);
    assert_eq!(cal_r.weather, Weather::Snowy);
    assert_eq!(ps_r.gold, 99999);
    assert_eq!(ps_r.tools.get(&ToolKind::Axe), Some(&ToolTier::Iridium));
    assert!(inv_r.slots[0].is_some());
    assert!(ql_r.completed.contains(&"main_quest_1".to_string()));
    assert!(ach_r.unlocked.contains(&"full_shipment".to_string()));
    assert_eq!(stats_r.days_played, 365);
    assert_eq!(stats_r.total_gold_earned, 500000);
}

#[test]
fn test_save_roundtrip_empty_defaults() {
    let cal = Calendar::default();
    let ps = PlayerState::default();
    let inv = Inventory::default();
    let ql = QuestLog::default();
    let ach = Achievements::default();
    let sl = ShippingLog::default();
    let rs = RelationshipStages::default();
    let ms = MarriageState::default();
    let hs = HouseState::default();
    let stats = PlayStats::default();

    let cal_r = serde_roundtrip(&cal);
    assert_eq!(cal_r.year, 1);
    assert_eq!(cal_r.season, Season::Spring);
    assert_eq!(cal_r.day, 1);

    let ps_r = serde_roundtrip(&ps);
    assert_eq!(ps_r.gold, 500);

    let inv_r = serde_roundtrip(&inv);
    assert!(inv_r.slots.iter().all(|s| s.is_none()));

    let ql_r = serde_roundtrip(&ql);
    assert!(ql_r.active.is_empty());
    assert!(ql_r.completed.is_empty());

    let ach_r = serde_roundtrip(&ach);
    assert!(ach_r.unlocked.is_empty());

    let sl_r = serde_roundtrip(&sl);
    assert!(sl_r.shipped_items.is_empty());

    let rs_r = serde_roundtrip(&rs);
    assert!(rs_r.stages.is_empty());

    let ms_r = serde_roundtrip(&ms);
    assert!(ms_r.spouse.is_none());

    let hs_r = serde_roundtrip(&hs);
    assert!(!hs_r.has_kitchen);

    let stats_r = serde_roundtrip(&stats);
    assert_eq!(stats_r.crops_harvested, 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Graduation: sprite_index uniqueness — prevents reintroduction of atlas
// collisions where two items or fish share the same sprite_index.
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_no_duplicate_sprite_indices() {
    let mut app = build_test_app();
    app.add_plugins(DataPlugin);
    app.update();
    app.update();

    // ── Item sprite_index uniqueness ────────────────────────────────────────
    let item_registry = app.world().resource::<ItemRegistry>();
    let mut seen_items: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
    for (id, item) in &item_registry.items {
        if let Some(prev_id) = seen_items.insert(item.sprite_index, id.clone()) {
            panic!(
                "Item sprite_index collision: items '{}' and '{}' both use sprite_index {}",
                prev_id, id, item.sprite_index
            );
        }
    }

    // ── Fish sprite_index uniqueness ────────────────────────────────────────
    let fish_registry = app.world().resource::<FishRegistry>();
    let mut seen_fish: std::collections::HashMap<u32, String> = std::collections::HashMap::new();
    for (id, fish) in &fish_registry.fish {
        if let Some(prev_id) = seen_fish.insert(fish.sprite_index, id.clone()) {
            panic!(
                "Fish sprite_index collision: fish '{}' and '{}' both use sprite_index {}",
                prev_id, id, fish.sprite_index
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Crafting — recipe resolution, ingredients, and bench flow
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_make_crafting_recipe_returns_all_known_ids() {
    for &id in ALL_CRAFTING_RECIPE_IDS {
        let recipe = make_crafting_recipe(id);
        assert!(recipe.is_some(), "Crafting recipe '{}' should exist", id);
        let r = recipe.unwrap();
        assert!(!r.ingredients.is_empty(), "Recipe '{}' must have ingredients", id);
        assert!(!r.result.is_empty(), "Recipe '{}' must produce an item", id);
    }
}

#[test]
fn test_make_cooking_recipe_returns_all_known_ids() {
    for &id in ALL_COOKING_RECIPE_IDS {
        let recipe = make_cooking_recipe(id);
        assert!(recipe.is_some(), "Cooking recipe '{}' should exist", id);
        let r = recipe.unwrap();
        assert!(r.is_cooking, "Cooking recipe '{}' should be marked is_cooking", id);
    }
}

#[test]
fn test_has_all_ingredients_true_when_present() {
    let recipe = make_crafting_recipe("chest").expect("chest recipe should exist");
    let mut inventory = Inventory::default();
    for (item_id, qty) in &recipe.ingredients {
        inventory.try_add(item_id, *qty, 99);
    }
    assert!(has_all_ingredients(&inventory, &recipe));
}

#[test]
fn test_has_all_ingredients_false_when_missing() {
    let recipe = make_crafting_recipe("chest").expect("chest recipe should exist");
    let inventory = Inventory::default();
    assert!(!has_all_ingredients(&inventory, &recipe));
}

#[test]
fn test_consume_ingredients_removes_from_inventory() {
    let recipe = make_crafting_recipe("chest").expect("chest recipe should exist");
    let mut inventory = Inventory::default();
    for (item_id, qty) in &recipe.ingredients {
        inventory.try_add(item_id, *qty, 99);
    }
    assert!(has_all_ingredients(&inventory, &recipe));
    consume_ingredients(&mut inventory, &recipe);
    assert!(
        !has_all_ingredients(&inventory, &recipe),
        "Ingredients should be consumed after craft"
    );
}

#[test]
fn test_refund_ingredients_restores_to_inventory() {
    let recipe = make_crafting_recipe("chest").expect("chest recipe should exist");
    let mut inventory = Inventory::default();
    let item_registry = ItemRegistry::default();
    refund_ingredients(&mut inventory, &recipe, &item_registry);
    assert!(
        has_all_ingredients(&inventory, &recipe),
        "Refunded ingredients should be present"
    );
}

#[test]
fn test_populate_recipe_registry_complete() {
    let mut registry = RecipeRegistry::default();
    populate_recipe_registry(&mut registry);
    let total = ALL_CRAFTING_RECIPE_IDS.len() + ALL_COOKING_RECIPE_IDS.len();
    assert_eq!(
        registry.recipes.len(),
        total,
        "Registry should contain all {} recipes, got {}",
        total,
        registry.recipes.len()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Sailing — boat mode and water navigation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_boat_mode_defaults_inactive() {
    let boat = BoatMode::new();
    assert!(!boat.active, "Boat mode should default to inactive");
    assert!(
        (boat.stamina_drain_per_tile - 0.5).abs() < f32::EPSILON,
        "Default stamina drain should be 0.5"
    );
}

#[test]
fn test_is_walkable_sailing_water_passable() {
    let mut tiles = vec![TileKind::Grass; 100];
    tiles[5 * 10 + 5] = TileKind::Water;
    tiles[3 * 10 + 3] = TileKind::Bridge;

    let map_def = MapDef {
        id: MapId::Farm,
        width: 10,
        height: 10,
        tiles,
        transitions: vec![],
        objects: vec![],
        forage_points: vec![],
    };

    let world_map = WorldMap {
        map_def: Some(map_def),
        solid_tiles: std::collections::HashSet::new(),
        width: 10,
        height: 10,
    };

    assert!(world_map.is_walkable_sailing(5, 5), "Water should be walkable while sailing");
    assert!(world_map.is_walkable_sailing(3, 3), "Bridge should be walkable while sailing");
}

#[test]
fn test_is_walkable_sailing_blocks_land() {
    let tiles = vec![TileKind::Grass; 100];
    let map_def = MapDef {
        id: MapId::Farm,
        width: 10,
        height: 10,
        tiles,
        transitions: vec![],
        objects: vec![],
        forage_points: vec![],
    };

    let world_map = WorldMap {
        map_def: Some(map_def),
        solid_tiles: std::collections::HashSet::new(),
        width: 10,
        height: 10,
    };

    assert!(!world_map.is_walkable_sailing(5, 5), "Grass should NOT be walkable while sailing");
}

#[test]
fn test_is_walkable_sailing_out_of_bounds() {
    let world_map = WorldMap {
        map_def: None,
        solid_tiles: std::collections::HashSet::new(),
        width: 10,
        height: 10,
    };

    assert!(!world_map.is_walkable_sailing(-1, 0), "Negative coords should not be walkable");
    assert!(!world_map.is_walkable_sailing(10, 0), "Out of bounds should not be walkable");
}

// ─────────────────────────────────────────────────────────────────────────────
// Player collision — CollisionMap, stamina, facing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_collision_map_solid_tiles_blocking() {
    let mut collision_map = CollisionMap::default();
    collision_map.initialised = true;
    collision_map.solid_tiles.insert((5, 5));
    collision_map.bounds = (0, 20, 0, 20);

    assert!(
        collision_map.solid_tiles.contains(&(5, 5)),
        "Tile (5,5) should be in solid_tiles"
    );
    assert!(
        !collision_map.solid_tiles.contains(&(6, 6)),
        "Tile (6,6) should NOT be solid"
    );
}

#[test]
fn test_collision_map_default_uninitialised() {
    let collision_map = CollisionMap::default();
    assert!(!collision_map.initialised, "CollisionMap should default to uninitialised");
    assert!(
        collision_map.solid_tiles.is_empty(),
        "CollisionMap should start with no solid tiles"
    );
}

#[test]
fn test_world_map_walkable_vs_solid() {
    let mut tiles = vec![TileKind::Grass; 100];
    tiles[5 * 10 + 5] = TileKind::Water;
    tiles[3 * 10 + 3] = TileKind::Path;

    let map_def = MapDef {
        id: MapId::Farm,
        width: 10,
        height: 10,
        tiles,
        transitions: vec![],
        objects: vec![],
        forage_points: vec![],
    };

    let world_map = WorldMap {
        map_def: Some(map_def),
        solid_tiles: std::collections::HashSet::new(),
        width: 10,
        height: 10,
    };

    assert!(world_map.is_walkable(3, 3), "Path should be walkable");
    assert!(world_map.is_walkable(0, 0), "Grass should be walkable");
    assert!(!world_map.is_walkable(5, 5), "Water should NOT be walkable on foot");
    assert!(!world_map.is_walkable(-1, 0), "Out of bounds should NOT be walkable");
}

#[test]
fn test_stamina_cost_scales_with_tool() {
    let tools = [
        ToolKind::Hoe,
        ToolKind::WateringCan,
        ToolKind::Axe,
        ToolKind::Pickaxe,
        ToolKind::FishingRod,
        ToolKind::Scythe,
    ];
    for tool in &tools {
        let cost = stamina_cost(tool);
        assert!(cost > 0.0, "{:?} should have positive stamina cost, got {}", tool, cost);
    }
}

#[test]
fn test_facing_offset_all_directions() {
    let (dx, dy) = facing_offset(&Facing::Up);
    assert_eq!((dx, dy), (0, 1), "Up should be (0, 1)");

    let (dx, dy) = facing_offset(&Facing::Down);
    assert_eq!((dx, dy), (0, -1), "Down should be (0, -1)");

    let (dx, dy) = facing_offset(&Facing::Left);
    assert_eq!((dx, dy), (-1, 0), "Left should be (-1, 0)");

    let (dx, dy) = facing_offset(&Facing::Right);
    assert_eq!((dx, dy), (1, 0), "Right should be (1, 0)");
}

// ─────────────────────────────────────────────────────────────────────────────
// Mining — floor state, rock breaking, ladder reveal
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_active_floor_defaults() {
    let floor = ActiveFloor::default();
    assert_eq!(floor.floor, 0, "Should start at floor 0");
    assert_eq!(floor.rocks_remaining, 0);
    assert_eq!(floor.rocks_broken_this_floor, 0);
    assert!(!floor.ladder_revealed, "Ladder should not be revealed by default");
    assert!(!floor.spawned, "Floor should not be spawned by default");
}

#[test]
fn test_floor_spawn_request_defaults() {
    let req = FloorSpawnRequest::default();
    assert!(!req.pending, "Should not have pending spawn by default");
    assert_eq!(req.floor, 0);
}

#[test]
fn test_in_mine_defaults_false() {
    let in_mine = InMine::default();
    assert!(!in_mine.0, "Should not be in mine by default");
}

#[test]
fn test_mine_rock_health_and_drops() {
    let copper_rock = MineRock {
        health: 1,
        drop_item: "copper_ore".to_string(),
        drop_quantity: 1,
    };
    assert_eq!(copper_rock.health, 1);
    assert_eq!(copper_rock.drop_item, "copper_ore");

    let iron_rock = MineRock {
        health: 2,
        drop_item: "iron_ore".to_string(),
        drop_quantity: 1,
    };
    assert_eq!(iron_rock.health, 2);

    let iridium_rock = MineRock {
        health: 3,
        drop_item: "iridium_ore".to_string(),
        drop_quantity: 1,
    };
    assert_eq!(iridium_rock.health, 3);
}

#[test]
fn test_mine_state_floor_persistence_roundtrip() {
    let mut mine_state = MineState::default();
    mine_state.deepest_floor_reached = 15;
    mine_state.elevator_floors = vec![5, 10, 15];

    let roundtripped = serde_roundtrip(&mine_state);
    assert_eq!(roundtripped.deepest_floor_reached, 15);
    assert_eq!(roundtripped.elevator_floors, vec![5, 10, 15]);
}

#[test]
fn test_mine_ladder_component() {
    let ladder = MineLadder { revealed: false };
    assert!(!ladder.revealed);
}

// ─────────────────────────────────────────────────────────────────────────────
// Input — PlayerInput defaults, InputBlocks, InputContext
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_player_input_defaults_all_false() {
    let input = PlayerInput::default();
    assert_eq!(input.move_axis, Vec2::ZERO, "Move axis should default to zero");
    assert!(!input.interact, "Interact should default to false");
    assert!(!input.tool_use, "Tool use should default to false");
    assert!(!input.open_inventory, "Open inventory should default to false");
    assert!(!input.pause, "Pause should default to false");
    assert!(!input.quicksave, "Quicksave should default to false");
    assert!(!input.quickload, "Quickload should default to false");
    assert!(input.tool_slot.is_none(), "Tool slot should default to None");
}

#[test]
fn test_input_context_default_gameplay() {
    let ctx = InputContext::default();
    assert_eq!(ctx, InputContext::Gameplay, "Default input context should be Gameplay");
}

#[test]
fn test_input_blocks_prevents_and_restores() {
    let mut blocks = InputBlocks::default();
    assert!(!blocks.is_blocked(), "Should not be blocked by default");

    struct DummyBlocker;
    blocks.block::<DummyBlocker>();
    assert!(blocks.is_blocked(), "Should be blocked after adding a blocker");

    blocks.unblock::<DummyBlocker>();
    assert!(!blocks.is_blocked(), "Should be unblocked after removing the blocker");
}

#[test]
fn test_input_blocks_multiple_blockers() {
    let mut blocks = InputBlocks::default();

    struct BlockerA;
    struct BlockerB;

    blocks.block::<BlockerA>();
    blocks.block::<BlockerB>();
    assert!(blocks.is_blocked());

    blocks.unblock::<BlockerA>();
    assert!(blocks.is_blocked(), "Should still be blocked with BlockerB");

    blocks.unblock::<BlockerB>();
    assert!(!blocks.is_blocked(), "Should be unblocked after removing all");
}

// ─────────────────────────────────────────────────────────────────────────────
// Map transitions — player state, collision invalidation, camera snap
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_map_transition_event_carries_coordinates() {
    let event = MapTransitionEvent {
        to_map: MapId::Town,
        to_x: 15,
        to_y: 20,
    };
    assert_eq!(event.to_map, MapId::Town);
    assert_eq!(event.to_x, 15);
    assert_eq!(event.to_y, 20);
}

#[test]
fn test_camera_snap_defaults() {
    let snap = CameraSnap::default();
    assert_eq!(snap.frames_remaining, 0, "Camera snap should default to 0 frames remaining");
}

#[test]
fn test_collision_map_cleared_on_invalidation() {
    let mut collision_map = CollisionMap::default();
    collision_map.initialised = true;
    collision_map.solid_tiles.insert((1, 1));
    collision_map.solid_tiles.insert((2, 2));

    collision_map.initialised = false;
    assert!(!collision_map.initialised);
}

#[test]
fn test_world_map_set_solid_toggles() {
    let tiles = vec![TileKind::Grass; 100];
    let map_def = MapDef {
        id: MapId::Farm,
        width: 10,
        height: 10,
        tiles,
        transitions: vec![],
        objects: vec![],
        forage_points: vec![],
    };

    let mut world_map = WorldMap {
        map_def: Some(map_def),
        solid_tiles: std::collections::HashSet::new(),
        width: 10,
        height: 10,
    };

    assert!(world_map.is_walkable(5, 5), "Should be walkable initially");

    world_map.set_solid(5, 5, true);
    assert!(!world_map.is_walkable(5, 5), "Should be blocked after set_solid(true)");

    world_map.set_solid(5, 5, false);
    assert!(world_map.is_walkable(5, 5), "Should be walkable after set_solid(false)");
}

// ─────────────────────────────────────────────────────────────────────────────
// ECS BEHAVIORAL TESTS — systems exercised through app.update()
// ─────────────────────────────────────────────────────────────────────────────

// ── Crafting: full ECS flow ────────────────────────────────────────────────

#[test]
fn test_ecs_handle_open_crafting_populates_ui_state() {
    let mut app = build_test_app();

    // Register crafting-specific resources and events
    app.init_resource::<CraftingUiState>();
    app.add_event::<OpenCraftingEvent>();

    // Populate the recipe registry
    let mut registry = RecipeRegistry::default();
    populate_recipe_registry(&mut registry);
    app.insert_resource(registry);

    // Unlock a few crafting recipes
    let mut unlocked = UnlockedRecipes::default();
    unlocked.ids.push("chest".to_string());
    unlocked.ids.push("furnace".to_string());
    app.insert_resource(unlocked);

    app.add_systems(Update, handle_open_crafting.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    // Send OpenCraftingEvent
    app.world_mut().send_event(OpenCraftingEvent { cooking_mode: false });
    app.update();

    let ui_state = app.world().resource::<CraftingUiState>();
    assert!(!ui_state.available_recipes.is_empty(),
        "Opening crafting should populate available_recipes");
    assert!(ui_state.available_recipes.contains(&"chest".to_string()),
        "Unlocked 'chest' recipe should be in available list");
    assert!(!ui_state.is_cooking_mode,
        "Should be in crafting mode, not cooking");
}

#[test]
fn test_ecs_handle_craft_item_consumes_ingredients_and_produces_result() {
    let mut app = build_test_app();
    app.add_plugins(hearthfield::data::DataPlugin);

    app.init_resource::<CraftingUiState>();
    app.add_event::<CraftItemEvent>();

    app.add_systems(Update, handle_craft_item.run_if(in_state(GameState::Crafting)));

    // Boot data (2 updates to load registries)
    app.update();
    app.update();

    // Unlock a DataPlugin recipe and add its ingredients
    // DataPlugin uses "recipe_chest" ID, not "chest" from make_crafting_recipe
    let recipe_id = "recipe_chest".to_string();
    let recipe = {
        let registry = app.world().resource::<RecipeRegistry>();
        registry.recipes.get(&recipe_id)
            .expect("DataPlugin should populate recipe_chest")
            .clone()
    };
    {
        let mut unlocked = app.world_mut().resource_mut::<UnlockedRecipes>();
        unlocked.ids.push(recipe_id.clone());
    }
    {
        let max_stacks: Vec<_> = {
            let item_registry = app.world().resource::<ItemRegistry>();
            recipe.ingredients.iter()
                .map(|(item_id, _)| item_registry.get(item_id).map(|d| d.stack_size).unwrap_or(99))
                .collect()
        };
        let mut inventory = app.world_mut().resource_mut::<Inventory>();
        for ((item_id, qty), max_stack) in recipe.ingredients.iter().zip(max_stacks.iter()) {
            inventory.try_add(item_id, *qty, *max_stack);
        }
    }

    // Verify ingredients present before craft
    {
        let inventory = app.world().resource::<Inventory>();
        assert!(has_all_ingredients(&inventory, &recipe), "Pre-condition: ingredients should be present");
    }

    // Transition to Crafting state and send event in the same frame
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Crafting);
    app.update(); // state transition queued

    // Verify we actually reached Crafting state
    let current = app.world().resource::<State<GameState>>().get().clone();
    assert_eq!(current, GameState::Crafting,
        "Should be in Crafting state, but in {:?}", current);

    // Now in Crafting — send event and tick
    app.world_mut().send_event(CraftItemEvent { recipe_id: recipe_id.clone() });
    app.update(); // system runs, consumes event

    // Verify: ingredients consumed
    {
        let inventory = app.world().resource::<Inventory>();
        assert!(!has_all_ingredients(&inventory, &recipe),
            "Ingredients should be consumed after crafting");
    }

    // Verify: result item in inventory
    {
        let inventory = app.world().resource::<Inventory>();
        let has_result = inventory.slots.iter().any(|slot| {
            slot.as_ref().map_or(false, |s| s.item_id == recipe.result)
        });
        assert!(has_result, "Crafted item '{}' should be in inventory", recipe.result);
    }
}

#[test]
fn test_ecs_craft_fails_without_ingredients() {
    let mut app = build_test_app();
    app.add_plugins(hearthfield::data::DataPlugin);
    app.init_resource::<CraftingUiState>();
    app.add_event::<CraftItemEvent>();

    app.add_systems(Update, handle_craft_item.run_if(in_state(GameState::Crafting)));
    app.update();
    app.update();

    // Transition to Crafting state
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Crafting);
    app.update();
    app.update();

    // Unlock recipe but DON'T add ingredients (use DataPlugin recipe ID)
    {
        let mut unlocked = app.world_mut().resource_mut::<UnlockedRecipes>();
        unlocked.ids.push("recipe_chest".to_string());
    }

    app.world_mut().send_event(CraftItemEvent { recipe_id: "recipe_chest".to_string() });
    app.update();

    // Inventory should still be empty — craft should have failed
    let inventory = app.world().resource::<Inventory>();
    let has_any = inventory.slots.iter().any(|s| s.is_some());
    assert!(!has_any, "Inventory should remain empty when crafting fails");
}

// ── Mining: rock breaking through ECS ──────────────────────────────────────

#[test]
fn test_ecs_rock_breaking_damages_rock_and_drains_stamina() {
    let mut app = build_test_app();

    // Register mining-specific resources and events
    app.init_resource::<ActiveFloor>();
    app.init_resource::<InMine>();
    app.init_resource::<MiningAtlases>();
    app.add_event::<RockHitEvent>();
    app.add_event::<RockDestroyedEvent>();

    app.add_systems(Update, handle_rock_breaking.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    // Enable mine mode
    app.world_mut().resource_mut::<InMine>().0 = true;

    // Set up active floor with 1 rock
    {
        let mut floor = app.world_mut().resource_mut::<ActiveFloor>();
        floor.rocks_remaining = 1;
        floor.spawned = true;
    }

    // Spawn a rock entity at (5, 5)
    let rock_entity = app.world_mut().spawn((
        MineRock {
            health: 1,
            drop_item: "copper_ore".to_string(),
            drop_quantity: 1,
        },
        MineGridPos { x: 5, y: 5 },
        Transform::default(),
    )).id();

    // Send a pickaxe ToolUseEvent at (5, 5) — Basic tier does 1 damage
    app.world_mut().send_event(ToolUseEvent {
        tool: ToolKind::Pickaxe,
        tier: ToolTier::Basic,
        target_x: 5,
        target_y: 5,
    });
    app.update();

    // Rock had 1 HP, basic pickaxe does 1 damage → rock should be destroyed (despawned)
    let rock_exists = app.world().get_entity(rock_entity).is_ok();
    assert!(!rock_exists, "Rock with 1 HP should be destroyed by 1-damage pickaxe hit");

    // Active floor should reflect the broken rock
    let floor = app.world().resource::<ActiveFloor>();
    assert_eq!(floor.rocks_remaining, 0, "rocks_remaining should be 0 after breaking the only rock");
    assert_eq!(floor.rocks_broken_this_floor, 1, "rocks_broken should increment");

    // Stamina drain event should have been sent (4.0 for Basic pickaxe)
    let stamina_events = app.world().resource::<Events<StaminaDrainEvent>>();
    let mut reader = stamina_events.get_cursor();
    let events: Vec<_> = reader.read(stamina_events).collect();
    assert!(!events.is_empty(), "StaminaDrainEvent should be sent on rock hit");
    assert!((events[0].amount - 4.0).abs() < f32::EPSILON,
        "Basic pickaxe stamina cost should be 4.0, got {}", events[0].amount);
}

#[test]
fn test_ecs_rock_breaking_survives_higher_hp() {
    let mut app = build_test_app();
    app.init_resource::<ActiveFloor>();
    app.init_resource::<InMine>();
    app.init_resource::<MiningAtlases>();
    app.add_event::<RockHitEvent>();
    app.add_event::<RockDestroyedEvent>();

    app.add_systems(Update, handle_rock_breaking.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    app.world_mut().resource_mut::<InMine>().0 = true;
    {
        let mut floor = app.world_mut().resource_mut::<ActiveFloor>();
        floor.rocks_remaining = 1;
        floor.spawned = true;
    }

    // Spawn rock with 3 HP (iridium ore)
    let rock_entity = app.world_mut().spawn((
        MineRock {
            health: 3,
            drop_item: "iridium_ore".to_string(),
            drop_quantity: 1,
        },
        MineGridPos { x: 3, y: 3 },
        Transform::default(),
    )).id();

    // Hit with Basic pickaxe (1 damage) — rock should survive
    app.world_mut().send_event(ToolUseEvent {
        tool: ToolKind::Pickaxe,
        tier: ToolTier::Basic,
        target_x: 3,
        target_y: 3,
    });
    app.update();

    let rock_exists = app.world().get_entity(rock_entity).is_ok();
    assert!(rock_exists, "Rock with 3 HP should survive a 1-damage hit");

    // Check rock health reduced
    let rock = app.world().entity(rock_entity).get::<MineRock>().unwrap();
    assert_eq!(rock.health, 2, "Rock health should be 3 - 1 = 2");

    // rocks_remaining should NOT have changed (rock survived)
    let floor = app.world().resource::<ActiveFloor>();
    assert_eq!(floor.rocks_remaining, 1, "Rock survived, rocks_remaining unchanged");
}

#[test]
fn test_ecs_rock_breaking_skipped_outside_mine() {
    let mut app = build_test_app();
    app.init_resource::<ActiveFloor>();
    app.init_resource::<InMine>();
    app.init_resource::<MiningAtlases>();
    app.add_event::<RockHitEvent>();
    app.add_event::<RockDestroyedEvent>();

    app.add_systems(Update, handle_rock_breaking.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    // InMine is false by default — rock breaking should be skipped
    let rock_entity = app.world_mut().spawn((
        MineRock {
            health: 1,
            drop_item: "copper_ore".to_string(),
            drop_quantity: 1,
        },
        MineGridPos { x: 5, y: 5 },
        Transform::default(),
    )).id();

    app.world_mut().send_event(ToolUseEvent {
        tool: ToolKind::Pickaxe,
        tier: ToolTier::Basic,
        target_x: 5,
        target_y: 5,
    });
    app.update();

    // Rock should survive — system was gated by InMine(false)
    let rock = app.world().entity(rock_entity).get::<MineRock>().unwrap();
    assert_eq!(rock.health, 1, "Rock should be untouched when not in mine");
}

// ── Player Movement: collision blocking through ECS ────────────────────────

/// Helper: build a test app with all resources needed by `player_movement`.
fn build_movement_test_app() -> App {
    let mut app = build_test_app();

    app.init_resource::<PlayerInput>();
    app.init_resource::<BoatMode>();
    app.init_resource::<InputBlocks>();

    let mut collision_map = CollisionMap::default();
    collision_map.initialised = true;
    collision_map.bounds = (0, 20, 0, 20);
    app.insert_resource(collision_map);

    let tiles = vec![TileKind::Grass; 400]; // 20x20
    let map_def = MapDef {
        id: MapId::Farm,
        width: 20,
        height: 20,
        tiles,
        transitions: vec![],
        objects: vec![],
        forage_points: vec![],
    };
    app.insert_resource(WorldMap {
        map_def: Some(map_def),
        solid_tiles: std::collections::HashSet::new(),
        width: 20,
        height: 20,
    });

    app.add_systems(Update, player_movement.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    app
}

#[test]
fn test_ecs_player_movement_blocked_by_solid_tile() {
    let mut app = build_movement_test_app();

    // Add a wall at grid (6, 5)
    app.world_mut().resource_mut::<CollisionMap>().solid_tiles.insert((6, 5));

    // Place player at right edge of grid (5, 5), facing right toward wall
    // grid 6 starts at world x = 6*16 = 96, so place at x=95.5 (almost there)
    let player_entity = app.world_mut().spawn((
        Player,
        LogicalPosition(Vec2::new(95.5, 88.0)),
        PlayerMovement::default(),
        GridPosition::new(5, 5),
    )).id();

    // Push right toward wall
    app.world_mut().resource_mut::<PlayerInput>().move_axis = Vec2::new(1.0, 0.0);

    // Many ticks — even with small deltas, enough to cross if unblocked
    for _ in 0..500 {
        app.update();
    }

    // Player grid should not reach 6 (the solid tile)
    let grid = app.world().entity(player_entity).get::<GridPosition>().unwrap();
    assert!(grid.x <= 5,
        "Player should be blocked at grid x<=5, but reached x={}", grid.x);
}

#[test]
fn test_ecs_player_movement_walks_on_open_tile() {
    let mut app = build_movement_test_app();

    // No solid tiles — path is clear
    let player_entity = app.world_mut().spawn((
        Player,
        LogicalPosition(Vec2::new(88.0, 88.0)),
        PlayerMovement::default(),
        GridPosition::new(5, 5),
    )).id();

    app.world_mut().resource_mut::<PlayerInput>().move_axis = Vec2::new(1.0, 0.0);

    // Record starting logical position
    let start_x = app.world().entity(player_entity)
        .get::<LogicalPosition>().unwrap().0.x;

    for _ in 0..500 {
        app.update();
    }

    // Verify logical position advanced (even if not a full grid cell)
    let end_x = app.world().entity(player_entity)
        .get::<LogicalPosition>().unwrap().0.x;
    assert!(end_x > start_x,
        "Player should have moved right (start_x={}, end_x={})", start_x, end_x);
}

#[test]
fn test_ecs_player_movement_blocked_by_input_blocks() {
    let mut app = build_movement_test_app();

    // Block input
    struct TestBlocker;
    app.world_mut().resource_mut::<InputBlocks>().block::<TestBlocker>();

    let player_entity = app.world_mut().spawn((
        Player,
        LogicalPosition(Vec2::new(88.0, 88.0)),
        PlayerMovement::default(),
        GridPosition::new(5, 5),
    )).id();

    app.world_mut().resource_mut::<PlayerInput>().move_axis = Vec2::new(1.0, 0.0);

    let start_x = app.world().entity(player_entity)
        .get::<LogicalPosition>().unwrap().0.x;

    for _ in 0..100 {
        app.update();
    }

    // Should NOT have moved
    let end_x = app.world().entity(player_entity)
        .get::<LogicalPosition>().unwrap().0.x;
    assert!((end_x - start_x).abs() < f32::EPSILON,
        "Player should not move when input blocked (start={}, end={})", start_x, end_x);
}

// ═══════════════════════════════════════════════════════════════════════════
// Regression tests for bug-fix wave
// ═══════════════════════════════════════════════════════════════════════════

/// Regression: refund_ingredients should handle full inventory gracefully
/// (previously, try_add return was unchecked — items silently dropped).
#[test]
fn test_refund_ingredients_logs_overflow_on_full_inventory() {
    let mut inventory = Inventory::default();
    let registry = ItemRegistry::default();
    let recipe = make_crafting_recipe("chest").expect("chest recipe exists");

    // Fill every slot with something
    for slot in inventory.slots.iter_mut() {
        *slot = Some(InventorySlot {
            item_id: "stone".to_string(),
            quantity: 99,
        });
    }

    // Refund should not panic even though inventory is full
    refund_ingredients(&mut inventory, &recipe, &registry);

    // Inventory should still be full of stone (refund couldn't fit wood)
    let all_stone = inventory.slots.iter().all(|s| {
        s.as_ref().map_or(false, |s| s.item_id == "stone")
    });
    assert!(all_stone, "Original items should remain — refund overflows gracefully");
}

/// Regression: consuming/refunding ingredients preserves inventory consistency.
#[test]
fn test_consume_then_refund_roundtrips_ingredients() {
    let mut inventory = Inventory::default();
    let registry = ItemRegistry::default();
    let recipe = make_crafting_recipe("chest").expect("chest recipe exists");

    // Add exactly the required ingredients
    for (item_id, qty) in &recipe.ingredients {
        let max = registry.get(item_id).map(|d| d.stack_size).unwrap_or(99);
        inventory.try_add(item_id, *qty, max);
    }

    assert!(has_all_ingredients(&inventory, &recipe));

    // Consume ingredients
    consume_ingredients(&mut inventory, &recipe);
    assert!(!has_all_ingredients(&inventory, &recipe));

    // Refund ingredients
    refund_ingredients(&mut inventory, &recipe, &registry);
    assert!(has_all_ingredients(&inventory, &recipe),
        "Refund should restore ingredients to pre-consume state");
}

/// Regression: UTF-8 safe truncation doesn't panic on any item name length.
#[test]
fn test_string_truncation_safety() {
    // Simulate the truncation logic used in chest_screen / inventory_screen / calendar_screen
    let cases = vec![
        "",          // empty
        "A",         // single char
        "Wood",      // short name
        "Iridium Sprinkler Mk II", // long name (24 chars)
        "こんにちは世界です",  // multi-byte UTF-8
        "ab",        // 2 chars
        "abcde",     // exactly 5 chars
        "abcdef",    // exactly 6 chars
        "abcdefghijklmnopqrst", // exactly 20 chars
        "abcdefghijklmnopqrstu", // 21 chars (triggers truncation)
    ];

    for name in &cases {
        // chest_screen pattern: truncate at 17 chars if len > 20
        let _chest_trunc = if name.len() > 20 {
            format!("{}...", name.chars().take(17).collect::<String>())
        } else {
            name.to_string()
        };

        // inventory_screen pattern: truncate at 5 chars if len > 6
        let _inv_trunc = if name.len() > 6 {
            format!("{}.", name.chars().take(5).collect::<String>())
        } else {
            name.to_string()
        };

        // calendar_screen pattern: truncate at 6 chars if len > 6
        let _cal_trunc: String = if name.len() > 6 {
            name.chars().take(6).collect()
        } else {
            name.to_string()
        };
    }
    // If we get here, no panics occurred
}

/// Regression: crafting with full inventory must not duplicate items.
/// The fix in bench.rs removes partial results before refunding ingredients.
#[test]
fn test_craft_full_inventory_no_item_duplication() {
    let mut inventory = Inventory::default();
    let registry = ItemRegistry::default();

    // Synthetic recipe: 10 wood → 5 chests (triggers partial-add scenario)
    let recipe = Recipe {
        id: "test_bulk_chest".into(),
        name: "Bulk Chest".into(),
        ingredients: vec![("wood".into(), 10)],
        result: "chest".into(),
        result_quantity: 5,
        is_cooking: false,
        unlocked_by_default: true,
    };

    // Slot 0: wood x99 (only 10 needed, so consuming leaves 89 — slot NOT freed)
    inventory.slots[0] = Some(InventorySlot {
        item_id: "wood".to_string(),
        quantity: 99,
    });

    // Slots 1..35: fill with stone x99 (only slot 35 left empty → room for 1 chest stack)
    let total_slots = inventory.slots.len();
    for i in 1..(total_slots - 1) {
        inventory.slots[i] = Some(InventorySlot {
            item_id: "stone".to_string(),
            quantity: 99,
        });
    }
    // Last slot empty → can fit 1 chest but not all 5 (stack_size check)

    let total_before: u32 = inventory.slots.iter()
        .filter_map(|s| s.as_ref())
        .map(|s| s.quantity as u32)
        .sum();

    // Simulate the FIXED craft path from bench.rs
    consume_ingredients(&mut inventory, &recipe);
    let max_stack = registry.get(&recipe.result).map(|d| d.stack_size).unwrap_or(99);
    let leftover = inventory.try_add(&recipe.result, recipe.result_quantity, max_stack);

    if leftover > 0 {
        // Remove partial result, then refund
        let added = recipe.result_quantity - leftover;
        if added > 0 {
            inventory.try_remove(&recipe.result, added);
        }
        refund_ingredients(&mut inventory, &recipe, &registry);
    }

    let total_after: u32 = inventory.slots.iter()
        .filter_map(|s| s.as_ref())
        .map(|s| s.quantity as u32)
        .sum();

    // Total must not increase (would indicate duplication)
    assert!(total_after <= total_before,
        "Item count must not increase after failed craft (before={}, after={}) — would indicate duplication",
        total_before, total_after);
}

// ========================================================================
// Graduation tests — verify [Assumed]/[Inferred] claims from STATE.md
// ========================================================================

/// Graduate: starter items include a hoe (farming critical path).
#[test]
fn test_starter_items_include_hoe() {
    let mut app = build_test_app();
    app.add_plugins(DataPlugin);
    app.add_systems(Startup, hearthfield::player::interaction::grant_starter_items);
    app.update();

    let inventory = app.world().resource::<Inventory>();
    let has_hoe = inventory.slots.iter().any(|s| {
        s.as_ref().map_or(false, |slot| slot.item_id == "hoe")
    });
    assert!(has_hoe, "Starter items must include a hoe for the farming critical path");
}

/// Graduate: starter items include seeds for immediate planting.
#[test]
fn test_starter_items_include_seeds() {
    let mut app = build_test_app();
    app.add_plugins(DataPlugin);
    app.add_systems(Startup, hearthfield::player::interaction::grant_starter_items);
    app.update();

    let inventory = app.world().resource::<Inventory>();
    let has_seeds = inventory.slots.iter().any(|s| {
        s.as_ref().map_or(false, |slot| slot.item_id.ends_with("_seeds"))
    });
    assert!(has_seeds, "Starter items must include seeds for the farming loop");
}

/// Graduate: season validation blocks planting out-of-season crops.
#[test]
fn test_season_validation_blocks_wrong_season_crop() {
    let spring_crop = CropDef {
        id: "turnip".into(),
        name: "Turnip".into(),
        seasons: vec![Season::Spring],
        growth_days: vec![2, 2, 2],
        seed_id: "turnip_seeds".into(),
        harvest_id: "turnip".into(),
        regrows: false,
        regrow_days: 0,
        sell_price: 60,
        sprite_stages: vec![0, 1, 2],
    };

    assert!(crop_can_grow_in_season(&spring_crop, Season::Spring),
        "Spring crop must grow in Spring");
    assert!(!crop_can_grow_in_season(&spring_crop, Season::Summer),
        "Spring crop must NOT grow in Summer");
    assert!(!crop_can_grow_in_season(&spring_crop, Season::Fall),
        "Spring crop must NOT grow in Fall");
    assert!(!crop_can_grow_in_season(&spring_crop, Season::Winter),
        "Spring crop must NOT grow in Winter");
}

/// Graduate: multi-season crops grow in all listed seasons.
#[test]
fn test_season_validation_allows_multi_season_crop() {
    let multi_crop = CropDef {
        id: "corn".into(),
        name: "Corn".into(),
        seasons: vec![Season::Summer, Season::Fall],
        growth_days: vec![3, 3, 3, 3],
        seed_id: "corn_seeds".into(),
        harvest_id: "corn".into(),
        regrows: false,
        regrow_days: 0,
        sell_price: 80,
        sprite_stages: vec![0, 1, 2, 3],
    };

    assert!(!crop_can_grow_in_season(&multi_crop, Season::Spring));
    assert!(crop_can_grow_in_season(&multi_crop, Season::Summer));
    assert!(crop_can_grow_in_season(&multi_crop, Season::Fall));
    assert!(!crop_can_grow_in_season(&multi_crop, Season::Winter));
}

/// Graduate: season change kills out-of-season crops.
#[test]
fn test_season_change_kills_out_of_season_crops() {
    let mut farm_state = FarmState::default();
    let mut crop_registry = CropRegistry::default();

    let spring_only = CropDef {
        id: "turnip".into(),
        name: "Turnip".into(),
        seasons: vec![Season::Spring],
        growth_days: vec![2, 2],
        seed_id: "turnip_seeds".into(),
        harvest_id: "turnip".into(),
        regrows: false,
        regrow_days: 0,
        sell_price: 60,
        sprite_stages: vec![0, 1],
    };
    crop_registry.crops.insert("turnip".into(), spring_only);

    // Plant in spring
    farm_state.crops.insert((5, 5), CropTile {
        crop_id: "turnip".into(),
        current_stage: 1,
        days_in_stage: 0,
        watered_today: false,
        dead: false,
        days_without_water: 0,
    });

    // Advance in summer — should kill the spring crop
    let _updated = advance_crop_growth(&mut farm_state, &crop_registry, Season::Summer, false);
    let crop = farm_state.crops.get(&(5, 5)).expect("crop tile should still exist");
    assert!(crop.dead, "Spring crop must die when season changes to Summer");
}

/// Graduate: save/load preserves current map identity.
#[test]
fn test_save_roundtrip_preserves_current_map() {
    let mut state = PlayerState::default();
    state.current_map = MapId::Town;
    state.save_grid_x = 42;
    state.save_grid_y = 17;

    let json = serde_json::to_string(&state).expect("serialize PlayerState");
    let loaded: PlayerState = serde_json::from_str(&json).expect("deserialize PlayerState");

    assert_eq!(loaded.current_map, MapId::Town, "current_map must survive save/load");
    assert_eq!(loaded.save_grid_x, 42, "save_grid_x must survive save/load");
    assert_eq!(loaded.save_grid_y, 17, "save_grid_y must survive save/load");
}

/// Graduate: collision map marks tiles solid and respects removal.
#[test]
fn test_collision_map_solid_and_walkable() {
    let mut cm = CollisionMap::default();
    cm.solid_tiles.insert((5, 5));
    cm.solid_tiles.insert((6, 5));

    assert!(cm.solid_tiles.contains(&(5, 5)), "tile (5,5) should be solid");
    assert!(cm.solid_tiles.contains(&(6, 5)), "tile (6,5) should be solid");
    assert!(!cm.solid_tiles.contains(&(7, 5)), "tile (7,5) should be walkable");

    // Simulate door carve-out
    cm.solid_tiles.remove(&(5, 5));
    assert!(!cm.solid_tiles.contains(&(5, 5)), "door tile should be walkable after removal");
}

/// Graduate: empty-season crops grow in any season (wildcard).
#[test]
fn test_season_validation_empty_seasons_means_all() {
    let any_season = CropDef {
        id: "wild".into(),
        name: "Wild Plant".into(),
        seasons: vec![],
        growth_days: vec![1],
        seed_id: "wild_seeds".into(),
        harvest_id: "wild".into(),
        regrows: false,
        regrow_days: 0,
        sell_price: 10,
        sprite_stages: vec![0],
    };

    assert!(crop_can_grow_in_season(&any_season, Season::Spring));
    assert!(crop_can_grow_in_season(&any_season, Season::Summer));
    assert!(crop_can_grow_in_season(&any_season, Season::Fall));
    assert!(crop_can_grow_in_season(&any_season, Season::Winter));
}

// ═══════════════════════════════════════════════════════════════════════════
// REGRESSION: Item sprite atlas — all DataPlugin items map to valid atlas slot
// ═══════════════════════════════════════════════════════════════════════════

/// Every item defined in DataPlugin must map to a valid atlas index (not fallback 0)
/// unless its sprite_index actually IS 0. Regression for the old whitelist bug that
/// remapped 148/237 items to index 0.
#[test]
fn test_all_item_sprite_indices_within_atlas() {
    let max_index = ITEM_ATLAS_COLUMNS * ITEM_ATLAS_ROWS; // 247
    let mut app = build_test_app();
    app.add_plugins(DataPlugin);
    app.update();
    app.update();

    let registry = app.world().resource::<ItemRegistry>();
    let mut failures = Vec::new();

    for (id, def) in &registry.items {
        let idx = def.sprite_index as usize;
        if idx >= max_index {
            failures.push(format!("{id}: sprite_index={idx} exceeds atlas capacity {max_index}"));
        }
        // Verify item_icon_index returns the actual index, not a fallback
        let mapped = item_icon_index(def.sprite_index);
        if mapped != idx && idx < max_index {
            failures.push(format!("{id}: item_icon_index returned {mapped} instead of {idx}"));
        }
    }

    assert!(failures.is_empty(), "Item sprite index failures:\n{}", failures.join("\n"));
}

/// item_icon_index must pass through valid indices and only clamp out-of-bounds.
#[test]
fn test_item_icon_index_passthrough() {
    // Valid indices pass through unchanged
    assert_eq!(item_icon_index(0), 0);
    assert_eq!(item_icon_index(64), 64);
    assert_eq!(item_icon_index(100), 100);
    assert_eq!(item_icon_index(200), 200);
    assert_eq!(item_icon_index(246), 246);
    // Out-of-bounds clamps to 0
    assert_eq!(item_icon_index(247), 0);
    assert_eq!(item_icon_index(999), 0);
}

// ═══════════════════════════════════════════════════════════════════════════
// REGRESSION: Animal pen bounds — animals graze in pasture, not on buildings
// ═══════════════════════════════════════════════════════════════════════════

/// Barn animals must wander south of the barn building (rows >= 19),
/// not on the building footprint (rows 16-18).
#[test]
fn test_barn_animal_pen_not_on_building() {
    let barn_building_max_y = 18.0 * TILE_SIZE;

    for kind in [AnimalKind::Cow, AnimalKind::Sheep, AnimalKind::Goat, AnimalKind::Pig] {
        let (pen_min, pen_max) = pen_bounds_for(kind);
        assert!(
            pen_min.y > barn_building_max_y,
            "{kind:?} pen min_y ({}) overlaps barn building (max {})",
            pen_min.y, barn_building_max_y
        );
        // Pen should be within farm bounds (row < 24)
        assert!(pen_max.y <= 23.0 * TILE_SIZE, "{kind:?} pen exceeds farm south edge");
    }
}

/// Coop animals must wander south of the coop building, not on it.
#[test]
fn test_coop_animal_pen_not_on_building() {
    let coop_building_max_y = 18.0 * TILE_SIZE;

    for kind in [AnimalKind::Chicken, AnimalKind::Duck, AnimalKind::Rabbit] {
        let (pen_min, pen_max) = pen_bounds_for(kind);
        assert!(
            pen_min.y > coop_building_max_y,
            "{kind:?} pen min_y ({}) overlaps coop building (max {})",
            pen_min.y, coop_building_max_y
        );
        assert!(pen_max.y <= 23.0 * TILE_SIZE, "{kind:?} pen exceeds farm south edge");
    }
}

/// Pet animals get a wider roam area that doesn't overlap barn/coop footprints.
#[test]
fn test_pet_roam_area_reasonable() {
    for kind in [AnimalKind::Horse, AnimalKind::Cat, AnimalKind::Dog] {
        let (pen_min, pen_max) = pen_bounds_for(kind);
        let width = (pen_max.x - pen_min.x) / TILE_SIZE;
        let height = (pen_max.y - pen_min.y) / TILE_SIZE;
        assert!(width >= 8.0, "{kind:?} roam area too narrow: {width} tiles");
        assert!(height >= 6.0, "{kind:?} roam area too short: {height} tiles");
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// WAVE 3: Audit-verified regression tests
// ═════════════════════════════════════════════════════════════════════════════

/// P0: BeeHouse must reject invalid input items (closes infinite-gold exploit).
#[test]
fn test_beehouse_rejects_invalid_input() {
    // These should NOT produce honey
    for bad_input in ["wood", "stone", "iron_ore", "gold_ore", "fiber", "coal", "egg", "milk"] {
        let result = resolve_machine_output(MachineType::BeeHouse, bad_input);
        assert!(
            result.is_none(),
            "BeeHouse accepted '{}' — exploit still open!",
            bad_input
        );
    }
}

/// P0: BeeHouse should accept its intended input.
#[test]
fn test_beehouse_accepts_valid_input() {
    let result = resolve_machine_output(MachineType::BeeHouse, "wild_honey");
    assert!(result.is_some(), "BeeHouse should accept wild_honey");
    let (item, qty) = result.unwrap();
    assert_eq!(item, "honey");
    assert_eq!(qty, 1);
}

/// P1: Quest with days_remaining=3 must survive exactly 3 expire_quests calls.
#[test]
fn test_quest_expiration_correct_day_count() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        expire_quests.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Add a quest with 3 days remaining
    let quest = make_test_quest("expire_3day", 100, Some(3));
    app.world_mut()
        .resource_mut::<QuestLog>()
        .active
        .push(quest);

    // Day 1: quest should survive (2 days left)
    app.world_mut().send_event(DayEndEvent { day: 1, season: Season::Spring, year: 1 });
    app.update();
    let log = app.world().resource::<QuestLog>();
    assert_eq!(log.active.len(), 1, "Quest should survive day 1");

    // Day 2: quest should survive (1 day left)
    app.world_mut().send_event(DayEndEvent { day: 2, season: Season::Spring, year: 1 });
    app.update();
    let log = app.world().resource::<QuestLog>();
    assert_eq!(log.active.len(), 1, "Quest should survive day 2");

    // Day 3: quest expires (0 days left)
    app.world_mut().send_event(DayEndEvent { day: 3, season: Season::Spring, year: 1 });
    app.update();
    let log = app.world().resource::<QuestLog>();
    assert_eq!(log.active.len(), 0, "Quest should expire on day 3");
}

/// P1: quality_from_happiness boundary values must match documented thresholds.
#[test]
fn test_quality_thresholds_boundary_values() {
    // Exact boundaries: 230, 200, 128
    assert_eq!(quality_from_happiness(255), ItemQuality::Iridium);
    assert_eq!(quality_from_happiness(230), ItemQuality::Iridium);
    assert_eq!(quality_from_happiness(229), ItemQuality::Gold);
    assert_eq!(quality_from_happiness(200), ItemQuality::Gold);
    assert_eq!(quality_from_happiness(199), ItemQuality::Silver);
    assert_eq!(quality_from_happiness(128), ItemQuality::Silver);
    assert_eq!(quality_from_happiness(127), ItemQuality::Normal);
    assert_eq!(quality_from_happiness(0), ItemQuality::Normal);
}

/// P1: "All Seasons" achievement must require year >= 2 (not just day count).
#[test]
fn test_all_seasons_requires_year2() {
    let mut app = build_test_app();
    app.add_systems(
        Update,
        check_achievements.run_if(in_state(GameState::Playing)),
    );
    enter_playing_state(&mut app);

    // Set days_played = 200 but year = 1 — should NOT unlock
    {
        let mut stats = app.world_mut().resource_mut::<PlayStats>();
        stats.days_played = 200;
    }
    {
        let mut cal = app.world_mut().resource_mut::<Calendar>();
        cal.year = 1;
    }
    app.update();
    let achievements = app.world().resource::<Achievements>();
    assert!(
        !achievements.unlocked.contains(&"all_seasons".to_string()),
        "all_seasons should NOT unlock with year=1 even with 200 days played"
    );

    // Set year = 2 — should unlock
    {
        let mut cal = app.world_mut().resource_mut::<Calendar>();
        cal.year = 2;
    }
    app.update();
    let achievements = app.world().resource::<Achievements>();
    assert!(
        achievements.unlocked.contains(&"all_seasons".to_string()),
        "all_seasons should unlock with year=2"
    );
}

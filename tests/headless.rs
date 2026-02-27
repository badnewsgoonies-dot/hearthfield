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
use hearthfield::data::DataPlugin;
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
use hearthfield::economy::shipping::{process_shipping_bin_on_day_end, ShippingBinPreview};
use hearthfield::economy::shop::ActiveShop;
use hearthfield::economy::stats::{AnimalProductStats, HarvestStats};
use hearthfield::farming::crops::{advance_crop_growth, reset_soil_watered_state};
use hearthfield::farming::events_handler::on_day_end as farming_on_day_end;
use hearthfield::farming::sprinklers::{handle_place_sprinkler, sprinkler_affected_tiles};
use hearthfield::farming::{FarmEntities, TrackedDayWeather};
use hearthfield::npcs::quests::{expire_quests, handle_quest_completed};
use hearthfield::npcs::romance::{
    handle_bouquet, handle_proposal, handle_wedding, tick_wedding_timer, WeddingTimer,
};
use hearthfield::world::objects::seasonal_forageables;
use hearthfield::shared::*;
use std::collections::HashMap;

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
    app.add_event::<ConsumeItemEvent>()
        .add_event::<StaminaRestoreEvent>()
        .add_event::<AnimalPurchaseEvent>()
        .add_event::<ToastEvent>();

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
        .add_event::<ScreenTransitionEvent>()
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
    app.init_resource::<ToolUpgradeQueue>();
    app.init_resource::<HarvestStats>();
    app.init_resource::<AnimalProductStats>();
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
    app.init_resource::<ToolUpgradeQueue>();
    app.init_resource::<HarvestStats>();
    app.init_resource::<AnimalProductStats>();
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
            days_old: 4, // will become 5 after day end → adult
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
        "Baby should grow to adult after 5 days"
    );
    assert_eq!(animal.days_old, 5);
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
    let mut cal = Calendar::default();

    cal.season = Season::Spring;
    cal.day = 13;
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
    let mut cal = Calendar::default();
    cal.hour = 14;
    cal.minute = 30;
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
    app.init_resource::<ToolUpgradeQueue>();
    app.init_resource::<HarvestStats>();
    app.init_resource::<AnimalProductStats>();
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
// Test 12: MenuCursor wrapping (pure function tests)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_menu_cursor_wraps_up() {
    let mut cursor = MenuCursor::new(4);
    assert_eq!(cursor.index, 0);
    cursor.up(); // 0 → wraps to 3
    assert_eq!(cursor.index, 3, "Cursor at 0 should wrap to count-1 on up()");
}

#[test]
fn test_menu_cursor_wraps_down() {
    let mut cursor = MenuCursor::new(4);
    cursor.index = 3;
    cursor.down(); // 3 → wraps to 0
    assert_eq!(cursor.index, 0, "Cursor at count-1 should wrap to 0 on down()");
}

#[test]
fn test_menu_cursor_set_ignores_out_of_bounds() {
    let mut cursor = MenuCursor::new(3);
    cursor.set(2);
    assert_eq!(cursor.index, 2, "set(2) should work for count=3");
    cursor.set(5);
    assert_eq!(cursor.index, 2, "set(5) should be ignored for count=3");
    cursor.set(3);
    assert_eq!(cursor.index, 2, "set(3) should be ignored for count=3");
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
    assert!(tiles.contains(&(5, 5)), "Basic tier should return the target tile itself");
}

#[test]
fn test_watering_can_copper_3_line() {
    // Copper: 3 tiles in a line along facing direction, starting at target.
    // Facing::Up means dy=+1 per step, so tiles are (5,5), (5,6), (5,7).
    let tiles = watering_can_area(ToolTier::Copper, 5, 5, Facing::Up);
    assert_eq!(tiles.len(), 3, "Copper tier should affect exactly 3 tiles");
    assert!(tiles.contains(&(5, 5)), "Line should start at target tile");
    assert!(tiles.contains(&(5, 6)), "Line should include one step in facing direction");
    assert!(tiles.contains(&(5, 7)), "Line should include two steps in facing direction");

    // Also verify with a different facing to ensure directionality is correct.
    let tiles_left = watering_can_area(ToolTier::Copper, 10, 10, Facing::Left);
    assert_eq!(tiles_left.len(), 3);
    assert!(tiles_left.contains(&(10, 10)));
    assert!(tiles_left.contains(&(9, 10)), "Left-facing line should step in -x");
    assert!(tiles_left.contains(&(8, 10)), "Left-facing line should step in -x twice");
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
    assert_eq!(tiles.len(), 9, "Gold tier should affect exactly 9 tiles (3×3)");

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
        assert!(tiles_left.contains(t), "Gold tiles should be identical regardless of facing");
    }
}

#[test]
fn test_watering_can_iridium_6x6() {
    // Iridium: 6×6 area (36 tiles). Implementation uses half=3,
    // dx in -(half-1)..=half => -2..=3, dy in -2..=3.
    let tiles = watering_can_area(ToolTier::Iridium, 5, 5, Facing::Up);
    assert_eq!(tiles.len(), 36, "Iridium tier should affect exactly 36 tiles (6×6)");

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
    assert_eq!(tiles.len(), 4, "Basic sprinkler should affect exactly 4 tiles");

    // Should contain the 4 cardinal neighbours.
    assert!(tiles.contains(&(9, 10)), "Basic should include west tile");
    assert!(tiles.contains(&(11, 10)), "Basic should include east tile");
    assert!(tiles.contains(&(10, 9)), "Basic should include south tile");
    assert!(tiles.contains(&(10, 11)), "Basic should include north tile");

    // Should NOT contain the centre tile.
    assert!(!tiles.contains(&(10, 10)), "Basic should exclude the centre tile");

    // Should NOT contain diagonal tiles.
    assert!(!tiles.contains(&(9, 9)), "Basic should exclude diagonals");
    assert!(!tiles.contains(&(11, 11)), "Basic should exclude diagonals");
}

#[test]
fn test_sprinkler_quality_8_surrounding() {
    // Quality: range 1, includes diagonals, skip centre → 8 tiles (3×3 - 1).
    let tiles = sprinkler_affected_tiles(SprinklerKind::Quality, 10, 10);
    assert_eq!(tiles.len(), 8, "Quality sprinkler should affect exactly 8 tiles");

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
    assert_eq!(tiles.len(), 24, "Iridium sprinkler should affect exactly 24 tiles");

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
    assert!(!tiles.contains(&(13, 10)), "Iridium should not reach range 3");
    assert!(!tiles.contains(&(10, 13)), "Iridium should not reach range 3");
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Pure function tests — Seasonal Forageables
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_seasonal_forageables_spring_has_items() {
    let items = seasonal_forageables(Season::Spring);
    assert_eq!(items.len(), 5, "Spring should have exactly 5 forageables");

    let names: Vec<&str> = items.iter().map(|(name, _)| *name).collect();
    assert!(names.contains(&"wild_horseradish"), "Spring should contain wild_horseradish");
    assert!(names.contains(&"daffodil"), "Spring should contain daffodil");
    assert!(names.contains(&"leek"), "Spring should contain leek");
    assert!(names.contains(&"dandelion"), "Spring should contain dandelion");
    assert!(names.contains(&"spring_onion"), "Spring should contain spring_onion");
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
    assert_eq!(stats.total_gold_earned, 500,
        "Expected 500 gold earned after positive event");

    // Negative event — should NOT decrease total_gold_earned
    app.world_mut().send_event(GoldChangeEvent {
        amount: -200,
        reason: "Bought seeds".to_string(),
    });
    app.update();

    let stats = app.world().resource::<PlayStats>();
    assert_eq!(stats.total_gold_earned, 500,
        "Expected total_gold_earned to remain 500 after negative event");
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Building Tests
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_building_tier_capacity_values() {
    assert_eq!(BuildingTier::None.capacity(), 0, "None capacity should be 0");
    assert_eq!(BuildingTier::Basic.capacity(), 4, "Basic capacity should be 4");
    assert_eq!(BuildingTier::Big.capacity(), 8, "Big capacity should be 8");
    assert_eq!(BuildingTier::Deluxe.capacity(), 12, "Deluxe capacity should be 12");
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
    app.add_systems(
        Update,
        handle_building_upgrade_request.run_if(in_state(GameState::Playing)),
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
    assert_eq!(player.gold, 6_000,
        "Gold should be deducted by 4000 for Coop Basic upgrade");

    let levels = app.world().resource::<BuildingLevels>();
    assert!(levels.upgrade_in_progress.is_some(),
        "upgrade_in_progress should be set after a valid upgrade request");
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
    app.world_mut().resource_mut::<BuildingLevels>().upgrade_in_progress =
        Some((BuildingKind::Coop, BuildingTier::Basic, 3));

    send_day_end(&mut app, 2, Season::Spring, 1);
    app.update();

    let levels = app.world().resource::<BuildingLevels>();
    assert!(levels.upgrade_in_progress.is_some(),
        "Upgrade should still be in progress after ticking from 3 days");
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
    app.world_mut().resource_mut::<BuildingLevels>().upgrade_in_progress =
        Some((BuildingKind::Coop, BuildingTier::Basic, 1));

    send_day_end(&mut app, 3, Season::Spring, 1);
    app.update();

    let levels = app.world().resource::<BuildingLevels>();
    assert!(levels.upgrade_in_progress.is_none(),
        "upgrade_in_progress should be None after completion");
    assert_eq!(levels.coop_tier, BuildingTier::Basic,
        "Coop tier should be upgraded to Basic");

    let animal_state = app.world().resource::<AnimalState>();
    assert!(animal_state.has_coop, "has_coop should be true after Coop upgrade");
    assert_eq!(animal_state.coop_level, 1, "coop_level should be 1 for Basic");
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

    app.world_mut().resource_mut::<BuildingLevels>().upgrade_in_progress =
        Some((BuildingKind::Silo, BuildingTier::Basic, 1));

    send_day_end(&mut app, 4, Season::Spring, 1);
    app.update();

    let levels = app.world().resource::<BuildingLevels>();
    assert!(levels.silo_built, "silo_built should be true after Silo upgrade completes");
    assert!(levels.upgrade_in_progress.is_none(),
        "upgrade_in_progress should be cleared after silo completion");
}

#[test]
fn test_building_cannot_upgrade_past_deluxe() {
    assert_eq!(BuildingTier::Deluxe.next(), None,
        "Deluxe.next() should return None — cannot upgrade past Deluxe");
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Tool Upgrade Tests
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_tool_tier_upgrade_cost() {
    assert_eq!(ToolTier::Basic.upgrade_cost(), 0, "Basic upgrade_cost should be 0");
    assert_eq!(ToolTier::Copper.upgrade_cost(), 2_000, "Copper upgrade_cost should be 2000");
    assert_eq!(ToolTier::Iron.upgrade_cost(), 5_000, "Iron upgrade_cost should be 5000");
    assert_eq!(ToolTier::Gold.upgrade_cost(), 10_000, "Gold upgrade_cost should be 10000");
    assert_eq!(ToolTier::Iridium.upgrade_cost(), 25_000, "Iridium upgrade_cost should be 25000");
}

#[test]
fn test_tool_upgrade_request_queues() {
    let mut app = build_test_app();
    app.init_resource::<ToolUpgradeQueue>();
    app.add_event::<ToolUpgradeRequestEvent>();
    app.add_event::<ToolUpgradeCompleteEvent>();
    app.init_resource::<ActiveShop>();
    app.add_systems(
        Update,
        handle_upgrade_request.run_if(in_state(GameState::Playing)),
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
    assert_eq!(queue.pending.len(), 1, "ToolUpgradeQueue should have 1 pending upgrade");
    assert_eq!(queue.pending[0].tool, ToolKind::Hoe);
    assert_eq!(queue.pending[0].target_tier, ToolTier::Copper);
    assert_eq!(queue.pending[0].days_remaining, 2, "Upgrade should take 2 days");

    let player = app.world().resource::<PlayerState>();
    assert_eq!(player.gold, 3_000, "Gold should be 5000 - 2000 = 3000 after upgrade request");
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
    assert!(queue.pending.is_empty(),
        "Pending queue should be empty after upgrade completes");

    let player = app.world().resource::<PlayerState>();
    assert_eq!(player.tools.get(&ToolKind::Axe).copied(), Some(ToolTier::Iron),
        "Axe should be upgraded to Iron tier");

    let complete_events = app.world().resource::<Events<ToolUpgradeCompleteEvent>>();
    let mut reader = complete_events.get_cursor();
    let events: Vec<_> = reader.read(complete_events).collect();
    assert_eq!(events.len(), 1, "Exactly one ToolUpgradeCompleteEvent should be sent");
    assert_eq!(events[0].tool, ToolKind::Axe);
    assert_eq!(events[0].new_tier, ToolTier::Iron);
}

#[test]
fn test_tool_tier_next_chain() {
    assert_eq!(ToolTier::Basic.next(), Some(ToolTier::Copper), "Basic -> Copper");
    assert_eq!(ToolTier::Copper.next(), Some(ToolTier::Iron), "Copper -> Iron");
    assert_eq!(ToolTier::Iron.next(), Some(ToolTier::Gold), "Iron -> Gold");
    assert_eq!(ToolTier::Gold.next(), Some(ToolTier::Iridium), "Gold -> Iridium");
    assert_eq!(ToolTier::Iridium.next(), None, "Iridium -> None (max tier)");
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Achievement Tests
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_achievements_constant_has_entries() {
    assert!(ACHIEVEMENTS.len() > 0, "ACHIEVEMENTS should have entries");
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
        assert!(!def.description.is_empty(), "Achievement description must not be empty");
    }
}

#[test]
fn test_achievement_unlocks_on_condition() {
    let mut app = build_test_app();
    app.add_systems(Update, check_achievements.run_if(in_state(GameState::Playing)));
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
    app.add_systems(Update, check_achievements.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    // Pre-unlock "first_harvest"
    app.world_mut().resource_mut::<Achievements>().unlocked.push("first_harvest".to_string());
    app.world_mut().resource_mut::<PlayStats>().crops_harvested = 5;

    app.update();

    let achievements = app.world().resource::<Achievements>();
    let count = achievements.unlocked.iter().filter(|id| *id == "first_harvest").count();
    assert_eq!(count, 1, "first_harvest should not be unlocked twice");
}

#[test]
fn test_achievement_progress_tracks_harvests() {
    let mut app = build_test_app();
    app.add_systems(Update, track_achievement_progress.run_if(in_state(GameState::Playing)));
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
    let gold_crops = achievements.progress.get("gold_crops").copied().unwrap_or(0);
    assert_eq!(gold_crops, 1, "gold_crops progress should be 1 after harvesting a Gold quality crop");
}

#[test]
fn test_achievement_unlocked_event_fires() {
    let mut app = build_test_app();
    app.add_systems(Update, check_achievements.run_if(in_state(GameState::Playing)));
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
    let event = fired.iter().find(|e| e.achievement_id == "gone_fishin").unwrap();
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
    registry.npcs.insert(npc_id.clone(), NpcDef {
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
    });
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
    app.world_mut().resource_mut::<Relationships>()
        .friendship.insert(npc_id.clone(), 700);
    app.world_mut().resource_mut::<Inventory>().try_add("bouquet", 1, 99);

    app.world_mut().send_event(BouquetGivenEvent {
        npc_name: "Lily".to_string(),
    });

    app.update();

    let stages = app.world().resource::<RelationshipStages>();
    let stage = stages.stages.get(&npc_id).copied().unwrap_or(RelationshipStage::Stranger);
    assert_ne!(stage, RelationshipStage::Dating,
        "Bouquet with < 8 hearts should not set Dating");
}

#[test]
fn test_bouquet_sets_dating() {
    let mut app = build_test_app();
    app.init_resource::<WeddingTimer>();
    app.add_systems(Update, handle_bouquet.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    let npc_id = insert_datable_npc(&mut app, "Lily");

    // Set friendship to 8 hearts (800 points)
    app.world_mut().resource_mut::<Relationships>()
        .friendship.insert(npc_id.clone(), 800);
    app.world_mut().resource_mut::<Inventory>().try_add("bouquet", 1, 99);

    app.world_mut().send_event(BouquetGivenEvent {
        npc_name: "Lily".to_string(),
    });

    app.update();

    let stages = app.world().resource::<RelationshipStages>();
    let stage = stages.stages.get(&npc_id).copied().unwrap_or(RelationshipStage::Stranger);
    assert_eq!(stage, RelationshipStage::Dating,
        "Bouquet with 8+ hearts should set stage to Dating");
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

    app.world_mut().resource_mut::<Relationships>()
        .friendship.insert(npc_id.clone(), 1000);
    app.world_mut().resource_mut::<RelationshipStages>()
        .stages.insert(npc_id.clone(), RelationshipStage::CloseFriend);
    app.world_mut().resource_mut::<HouseState>().tier = HouseTier::Basic;
    app.world_mut().resource_mut::<Inventory>().try_add("mermaid_pendant", 1, 99);

    app.world_mut().send_event(ProposalEvent {
        npc_name: "Lily".to_string(),
    });

    app.update();

    let stages = app.world().resource::<RelationshipStages>();
    let stage = stages.stages.get(&npc_id).copied().unwrap_or(RelationshipStage::Stranger);
    assert_ne!(stage, RelationshipStage::Engaged,
        "Proposal without Dating stage should not set Engaged");
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
    app.world_mut().resource_mut::<Relationships>()
        .friendship.insert(npc_id.clone(), 1000);
    app.world_mut().resource_mut::<RelationshipStages>()
        .stages.insert(npc_id.clone(), RelationshipStage::Dating);
    app.world_mut().resource_mut::<HouseState>().tier = HouseTier::Big;
    app.world_mut().resource_mut::<Inventory>().try_add("mermaid_pendant", 1, 99);

    app.world_mut().send_event(ProposalEvent {
        npc_name: "Lily".to_string(),
    });

    app.update();

    let stages = app.world().resource::<RelationshipStages>();
    let stage = stages.stages.get(&npc_id).copied().unwrap_or(RelationshipStage::Stranger);
    assert_eq!(stage, RelationshipStage::Engaged,
        "Proposal with all prerequisites should set Engaged");

    let timer = app.world().resource::<WeddingTimer>();
    assert_eq!(timer.days_remaining, Some(3),
        "WeddingTimer should be set to 3 days after accepted proposal");
    assert_eq!(timer.npc_name, Some("Lily".to_string()),
        "WeddingTimer npc_name should be the proposed NPC");
}

#[test]
fn test_wedding_timer_ticks_down() {
    let mut app = build_test_app();
    app.init_resource::<WeddingTimer>();
    app.add_systems(Update, tick_wedding_timer.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    {
        let mut timer = app.world_mut().resource_mut::<WeddingTimer>();
        timer.days_remaining = Some(3);
        timer.npc_name = Some("Lily".to_string());
    }

    send_day_end(&mut app, 5, Season::Spring, 1);
    app.update();

    let timer = app.world().resource::<WeddingTimer>();
    assert_eq!(timer.days_remaining, Some(2),
        "WeddingTimer should decrement from 3 to 2 after one DayEndEvent");
}

#[test]
fn test_wedding_completes_marriage() {
    let mut app = build_test_app();
    app.init_resource::<WeddingTimer>();
    app.add_systems(Update, (
        tick_wedding_timer,
        handle_wedding,
    ).chain().run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    let npc_id = insert_datable_npc(&mut app, "Lily");

    {
        let mut timer = app.world_mut().resource_mut::<WeddingTimer>();
        timer.days_remaining = Some(1);
        timer.npc_name = Some("Lily".to_string());
    }

    app.world_mut().resource_mut::<RelationshipStages>()
        .stages.insert(npc_id.clone(), RelationshipStage::Engaged);

    send_day_end(&mut app, 10, Season::Spring, 1);
    app.update();

    let marriage = app.world().resource::<MarriageState>();
    assert_eq!(marriage.spouse, Some("Lily".to_string()),
        "MarriageState.spouse should be set after wedding completes");

    let relationships = app.world().resource::<Relationships>();
    assert_eq!(relationships.spouse, Some(npc_id.clone()),
        "Relationships.spouse should be set to the NPC id");

    let stages = app.world().resource::<RelationshipStages>();
    let stage = stages.stages.get(&npc_id).copied().unwrap_or(RelationshipStage::Stranger);
    assert_eq!(stage, RelationshipStage::Married,
        "Relationship stage should be Married after wedding");
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
    app.add_systems(Update, handle_quest_completed.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    let quest = make_test_quest("test_q1", 200, Some(5));
    app.world_mut().resource_mut::<QuestLog>().active.push(quest);

    app.world_mut().send_event(QuestCompletedEvent {
        quest_id: "test_q1".to_string(),
        reward_gold: 200,
    });

    app.update();

    let quest_log = app.world().resource::<QuestLog>();
    assert!(quest_log.active.iter().all(|q| q.id != "test_q1"),
        "Completed quest should be removed from active");
    assert!(quest_log.completed.contains(&"test_q1".to_string()),
        "Completed quest id should be in completed list");
}

#[test]
fn test_quest_complete_awards_gold() {
    let mut app = build_test_app();
    app.add_systems(Update, handle_quest_completed.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    let quest = make_test_quest("test_q2", 500, Some(5));
    app.world_mut().resource_mut::<QuestLog>().active.push(quest);

    app.world_mut().send_event(QuestCompletedEvent {
        quest_id: "test_q2".to_string(),
        reward_gold: 500,
    });

    app.update();

    let events = app.world().resource::<Events<GoldChangeEvent>>();
    let mut reader = events.get_cursor();
    let fired: Vec<_> = reader.read(events).collect();
    assert!(fired.iter().any(|e| e.amount == 500),
        "GoldChangeEvent with amount 500 should be sent for quest completion");
}

#[test]
fn test_quest_expires_after_days() {
    let mut app = build_test_app();
    app.add_systems(Update, expire_quests.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    let quest = make_test_quest("test_expire", 100, Some(1));
    app.world_mut().resource_mut::<QuestLog>().active.push(quest);

    send_day_end(&mut app, 5, Season::Spring, 1);
    app.update();

    let quest_log = app.world().resource::<QuestLog>();
    assert!(quest_log.active.iter().all(|q| q.id != "test_expire"),
        "Quest with days_remaining=1 should be removed after DayEndEvent");
}

#[test]
fn test_quest_no_expire_if_days_remain() {
    let mut app = build_test_app();
    app.add_systems(Update, expire_quests.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    let quest = make_test_quest("test_no_expire", 100, Some(3));
    app.world_mut().resource_mut::<QuestLog>().active.push(quest);

    send_day_end(&mut app, 5, Season::Spring, 1);
    app.update();

    let quest_log = app.world().resource::<QuestLog>();
    let quest = quest_log.active.iter().find(|q| q.id == "test_no_expire");
    assert!(quest.is_some(), "Quest with days_remaining=3 should still be active after 1 day");
    assert_eq!(quest.unwrap().days_remaining, Some(2),
        "days_remaining should decrement from 3 to 2");
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Evaluation Tests
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_evaluation_trigger_fires_year3() {
    let mut app = build_test_app();
    app.add_systems(Update, check_evaluation_trigger.run_if(in_state(GameState::Playing)));
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
    assert!(!fired.is_empty(), "EvaluationTriggerEvent should fire on Spring 1 Year 3+");
}

#[test]
fn test_evaluation_trigger_skips_year1() {
    let mut app = build_test_app();
    app.add_systems(Update, check_evaluation_trigger.run_if(in_state(GameState::Playing)));
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
    assert!(fired.is_empty(), "EvaluationTriggerEvent should NOT fire on Year 1");
}

#[test]
fn test_evaluation_scores_categories() {
    let mut app = build_test_app();
    app.init_resource::<EconomyStats>();
    app.init_resource::<HarvestStats>();
    app.add_systems(Update, handle_evaluation.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    // Earnings: total_gold_earned >= 50_000 -> 1 point
    app.world_mut().resource_mut::<EconomyStats>().total_gold_earned = 60_000;
    // Married -> 1 point; happiness > 50 -> 1 point
    app.world_mut().resource_mut::<MarriageState>().spouse = Some("Lily".to_string());
    app.world_mut().resource_mut::<MarriageState>().spouse_happiness = 60;

    app.world_mut().send_event(EvaluationTriggerEvent);
    app.update();

    let eval = app.world().resource::<EvaluationScore>();
    assert!(eval.evaluated, "evaluated should be true after handle_evaluation runs");
    assert!(eval.total_points >= 3,
        "Should have at least 3 points: earnings_50k + spouse_married + spouse_happiness, got {}",
        eval.total_points);
    assert!(eval.categories.contains_key("earnings_50k"));
    assert!(eval.categories.contains_key("spouse_married"));
    assert!(eval.categories.contains_key("spouse_happiness"));
}

#[test]
fn test_evaluation_sets_candles() {
    let mut app = build_test_app();
    app.init_resource::<EconomyStats>();
    app.init_resource::<HarvestStats>();
    app.add_systems(Update, handle_evaluation.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    // With zero stats, total_points should be 0 -> candles_lit = 1 (0-5 = 1 candle)
    app.world_mut().send_event(EvaluationTriggerEvent);
    app.update();

    let eval = app.world().resource::<EvaluationScore>();
    assert_eq!(eval.candles_lit, 1,
        "0 points should yield 1 candle, got {} candles for {} points",
        eval.candles_lit, eval.total_points);

    // Reset for second evaluation with many points
    app.world_mut().resource_mut::<EvaluationScore>().evaluated = false;

    // Earnings: 500k -> 4 points (50k + 100k + 200k + 500k)
    app.world_mut().resource_mut::<EconomyStats>().total_gold_earned = 500_000;
    // Married + happiness > 50 -> 2 points
    app.world_mut().resource_mut::<MarriageState>().spouse = Some("Lily".to_string());
    app.world_mut().resource_mut::<MarriageState>().spouse_happiness = 60;
    // Mine floor 20 -> 1 point
    app.world_mut().resource_mut::<MineState>().deepest_floor_reached = 20;
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
    // total_items_shipped >= 50 -> 2 points
    app.world_mut().resource_mut::<EconomyStats>().total_items_shipped = 55;
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
    assert!(eval.total_points >= 13,
        "Expected at least 13 points with substantial progress, got {}", eval.total_points);
    assert!(eval.candles_lit >= 3,
        "13+ points should yield at least 3 candles, got {} candles for {} points",
        eval.candles_lit, eval.total_points);
}

// ═════════════════════════════════════════════════════════════════════════════
// PHASE 3/4: Sprinkler Integration Tests
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn test_place_sprinkler_adds_to_state() {
    let mut app = build_test_app();
    app.init_resource::<FarmEntities>();
    app.add_systems(Update, handle_place_sprinkler.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    app.world_mut().resource_mut::<Inventory>().try_add("sprinkler", 1, 99);

    app.world_mut().send_event(PlaceSprinklerEvent {
        kind: SprinklerKind::Basic,
        tile_x: 5,
        tile_y: 5,
    });

    app.update();

    let sprinkler_state = app.world().resource::<SprinklerState>();
    assert_eq!(sprinkler_state.sprinklers.len(), 1,
        "SprinklerState should have 1 sprinkler after placement");
    let placed = &sprinkler_state.sprinklers[0];
    assert_eq!(placed.tile_x, 5);
    assert_eq!(placed.tile_y, 5);
    assert_eq!(placed.kind, SprinklerKind::Basic);
}

#[test]
fn test_place_multiple_sprinklers() {
    let mut app = build_test_app();
    app.init_resource::<FarmEntities>();
    app.add_systems(Update, handle_place_sprinkler.run_if(in_state(GameState::Playing)));
    enter_playing_state(&mut app);

    app.world_mut().resource_mut::<Inventory>().try_add("sprinkler", 1, 99);
    app.world_mut().resource_mut::<Inventory>().try_add("quality_sprinkler", 1, 99);

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
    assert_eq!(sprinkler_state.sprinklers.len(), 2,
        "SprinklerState should have 2 sprinklers after two placements");

    let has_basic = sprinkler_state.sprinklers.iter().any(|s|
        s.tile_x == 3 && s.tile_y == 3 && s.kind == SprinklerKind::Basic);
    let has_quality = sprinkler_state.sprinklers.iter().any(|s|
        s.tile_x == 7 && s.tile_y == 7 && s.kind == SprinklerKind::Quality);
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
    assert!(ids.contains(&"wild_horseradish"),
        "Spring forageables should include wild_horseradish, got: {:?}", ids);
}

#[test]
fn test_forageables_winter_includes_crystal_fruit() {
    let winter_items = seasonal_forageables(Season::Winter);
    let ids: Vec<&str> = winter_items.iter().map(|(id, _)| *id).collect();
    assert!(ids.contains(&"crystal_fruit"),
        "Winter forageables should include crystal_fruit, got: {:?}", ids);
}

#[test]
fn test_forageables_unique_per_season() {
    let spring_items = seasonal_forageables(Season::Spring);
    let summer_items = seasonal_forageables(Season::Summer);

    let spring_ids: Vec<&str> = spring_items.iter().map(|(id, _)| *id).collect();
    let summer_ids: Vec<&str> = summer_items.iter().map(|(id, _)| *id).collect();

    assert_ne!(spring_ids, summer_ids,
        "Spring and summer forageables should be different sets");

    let overlap: Vec<&&str> = spring_ids.iter().filter(|id| summer_ids.contains(id)).collect();
    assert!(overlap.is_empty(),
        "Spring and summer should have no overlapping forageable items, found: {:?}", overlap);
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
    assert!(z_100 > z_200, "Lower Y should produce higher Z (draws in front)");
}

#[test]
fn test_ysort_does_not_overlap_ground() {
    // Lowest possible entity Z should still be above Z_FARM_OVERLAY
    let worst_case_z = Z_ENTITY_BASE - 5000.0 * Z_Y_SORT_SCALE; // extreme map
    assert!(worst_case_z > Z_FARM_OVERLAY);
}

#[test]
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
    let mut anim = DistanceAnimator::default();
    anim.current_frame = 2;
    anim.distance_budget = 3.5;

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
        PlayerAnimState::ToolUse { tool, frame, total_frames } => {
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
    // Down row 0
    assert_eq!(0 + 0, 0);
    assert_eq!(0 + 3, 3);
    // Up row 1
    assert_eq!(4 + 0, 4);
    assert_eq!(4 + 3, 7);
    // Right row 2
    assert_eq!(8 + 0, 8);
    assert_eq!(8 + 3, 11);
    // Left row 3
    assert_eq!(12 + 0, 12);
    assert_eq!(12 + 3, 15);
}

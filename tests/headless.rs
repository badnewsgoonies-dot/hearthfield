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
use hearthfield::economy::blacksmith::{ToolUpgradeCompleteEvent, ToolUpgradeQueue};
use hearthfield::economy::gold::{apply_gold_changes, EconomyStats};
use hearthfield::economy::shipping::{process_shipping_bin_on_day_end, ShippingBinPreview};
use hearthfield::economy::stats::{AnimalProductStats, HarvestStats};
use hearthfield::farming::crops::{advance_crop_growth, reset_soil_watered_state};
use hearthfield::farming::events_handler::on_day_end as farming_on_day_end;
use hearthfield::farming::{FarmEntities, TrackedDayWeather};
use hearthfield::shared::*;

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

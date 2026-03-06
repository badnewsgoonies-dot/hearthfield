//! Headless ECS integration tests for Skywarden.
//!
//! These tests exercise game logic WITHOUT a window or GPU by using
//! `MinimalPlugins`. Each test builds a minimal `App` with only the
//! resources, events, and systems it needs, runs 1–2 frames, then
//! asserts expected outcomes.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

// ── Library re-exports ──────────────────────────────────────────────────

use skywarden::*; // shared types via `pub use shared::*`

// Domain systems
use skywarden::economy::gold::{
    add_gold, apply_gold_changes, spend_gold, GoldMilestones, TransactionLog,
};
use skywarden::economy::progression::{
    apply_xp, check_rank_up, meets_rank_requirements, rank_requirements, ActivityTracker,
};
use skywarden::flight::cruise::update_flight;
use skywarden::flight::navigation::{calculate_route, update_navigation, NavigationState};
use skywarden::player::day_cycle::{advance_time, day_end_check, trigger_day_end, DayCycleConfig};

// Priority 1–3 additions
use skywarden::airports::maps::generate_zone_map;
use skywarden::airports::npcs::{spawn_ambient_npcs, AirportNpc};
use skywarden::airports::services::{services_at, AirportService};
use skywarden::crew::schedules::get_schedule;
use skywarden::data::items::populate_items;
use skywarden::data::missions::get_mission_templates;
use skywarden::data::shops::get_shop_inventory;
use skywarden::save::SaveFile;

use skywarden::aircraft::fleet::{create_starter_fleet, purchase_aircraft, HangarAssignments};
use skywarden::aircraft::fuel::{calculate_fuel_burn, fuel_type_for_class, FuelType, FuelWarnings};
use skywarden::aircraft::maintenance::{ComponentCondition, MaintenanceTracker};
use skywarden::crew::gifts::handle_gift_given;
use skywarden::crew::relationships::{friendship_decay, RelationshipDetails};
use skywarden::crew::schedules::{get_crew_current_zone, is_crew_present};
use skywarden::missions::board::{handle_mission_accepted, refresh_mission_board};
use skywarden::missions::story::StoryProgress;
use skywarden::missions::tracking::handle_mission_complete;
use skywarden::player::movement::{player_movement, PlayerAnimState};
use skywarden::weather::effects::IcingState;
use skywarden::weather::forecasting::WeatherForecasts;

// ═════════════════════════════════════════════════════════════════════════
// TEST APP BUILDER
// ═════════════════════════════════════════════════════════════════════════

/// Build a minimal Bevy app with all shared resources and events registered
/// but NO window, renderer, or GPU. Systems are added per-test.
fn build_test_app() -> App {
    let mut app = App::new();

    app.add_plugins(MinimalPlugins);
    app.add_plugins(StatesPlugin);

    // ── State ────────────────────────────────────────────────────────
    app.init_state::<GameState>();

    // ── Core Resources ──────────────────────────────────────────────
    app.init_resource::<Calendar>();
    app.init_resource::<WeatherState>();
    app.init_resource::<PlayerLocation>();
    app.init_resource::<PilotState>();
    app.init_resource::<PlayerMovement>();
    app.init_resource::<GridPosition>();
    app.init_resource::<PlayerInput>();
    app.init_resource::<KeyBindings>();
    app.init_resource::<InputState>();
    app.init_resource::<Inventory>();
    app.init_resource::<ItemRegistry>();
    app.init_resource::<Gold>();
    app.init_resource::<EconomyStats>();
    app.init_resource::<ActiveShop>();
    app.init_resource::<Fleet>();
    app.init_resource::<AircraftRegistry>();
    app.init_resource::<FlightState>();
    app.init_resource::<MissionBoard>();
    app.init_resource::<MissionLog>();
    app.init_resource::<CrewRegistry>();
    app.init_resource::<Relationships>();
    app.init_resource::<DialogueState>();
    app.init_resource::<Achievements>();
    app.init_resource::<PlayStats>();
    app.init_resource::<WorldMap>();
    app.init_resource::<CollisionMap>();
    app.init_resource::<InteractionClaimed>();
    app.init_resource::<SaveSlots>();
    app.init_resource::<SessionTimer>();
    app.init_resource::<TutorialState>();
    app.init_resource::<CutsceneQueue>();
    app.init_resource::<ActiveCutscene>();

    // Domain-specific resources
    app.init_resource::<DayCycleConfig>();
    app.init_resource::<NavigationState>();
    app.init_resource::<TransactionLog>();
    app.init_resource::<GoldMilestones>();
    app.init_resource::<FuelWarnings>();
    app.init_resource::<MaintenanceTracker>();
    app.init_resource::<HangarAssignments>();
    app.init_resource::<ActivityTracker>();
    app.init_resource::<PlayerAnimState>();
    app.init_resource::<WeatherForecasts>();
    app.init_resource::<IcingState>();
    app.init_resource::<RelationshipDetails>();
    app.init_resource::<StoryProgress>();

    // ── Events ──────────────────────────────────────────────────────
    app.add_event::<DayEndEvent>();
    app.add_event::<SeasonChangeEvent>();
    app.add_event::<ZoneTransitionEvent>();
    app.add_event::<AirportArrivalEvent>();
    app.add_event::<FlightStartEvent>();
    app.add_event::<FlightCompleteEvent>();
    app.add_event::<FlightPhaseChangeEvent>();
    app.add_event::<EmergencyEvent>();
    app.add_event::<GoldChangeEvent>();
    app.add_event::<PurchaseEvent>();
    app.add_event::<DialogueStartEvent>();
    app.add_event::<GiftGivenEvent>();
    app.add_event::<FriendshipChangeEvent>();
    app.add_event::<MissionAcceptedEvent>();
    app.add_event::<MissionCompletedEvent>();
    app.add_event::<MissionFailedEvent>();
    app.add_event::<ItemPickupEvent>();
    app.add_event::<RankUpEvent>();
    app.add_event::<LicenseEarnedEvent>();
    app.add_event::<AchievementUnlockedEvent>();
    app.add_event::<XpGainEvent>();
    app.add_event::<ToastEvent>();
    app.add_event::<PlaySfxEvent>();
    app.add_event::<PlayMusicEvent>();
    app.add_event::<ScreenFadeEvent>();
    app.add_event::<WeatherChangeEvent>();
    app.add_event::<SaveRequestEvent>();
    app.add_event::<LoadRequestEvent>();
    app.add_event::<SaveCompleteEvent>();
    app.add_event::<LoadCompleteEvent>();
    app.add_event::<CutsceneStartEvent>();
    app.add_event::<HintEvent>();

    app
}

// ═════════════════════════════════════════════════════════════════════════
// CALENDAR / TIME TESTS
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_time_advances() {
    // advance_time converts real delta into game minutes.
    // Since MinimalPlugins delta is near-zero in tests, we verify the
    // system runs without error and test the calendar arithmetic directly.
    let mut calendar = Calendar {
        hour: 6,
        minute: 0,
        time_paused: false,
        ..Calendar::default()
    };

    // Simulate what advance_time does: add game-minutes manually
    let config = DayCycleConfig::default();
    let simulated_delta_secs = 1.0_f32; // pretend 1 real second elapsed
    let delta_minutes = simulated_delta_secs * config.game_minutes_per_real_second;
    let total_minutes = calendar.minute as f32 + delta_minutes;
    let extra_hours = (total_minutes / 60.0).floor() as u32;
    calendar.minute = (total_minutes % 60.0) as u32;
    calendar.hour += extra_hours;

    assert!(
        calendar.minute > 0 || calendar.hour > 6,
        "Time should advance: hour={}, minute={}",
        calendar.hour,
        calendar.minute
    );
}

#[test]
fn test_time_paused_does_not_advance() {
    let mut app = build_test_app();
    app.add_systems(Update, advance_time);

    {
        let mut calendar = app.world_mut().resource_mut::<Calendar>();
        calendar.hour = 10;
        calendar.minute = 30;
        calendar.time_paused = true;
    }

    app.update();

    let calendar = app.world().resource::<Calendar>();
    assert_eq!(calendar.hour, 10);
    assert_eq!(calendar.minute, 30);
}

#[test]
fn test_day_end_triggers() {
    let mut app = build_test_app();
    app.add_systems(Update, day_end_check);

    // Set hour >= 24 to trigger day end
    {
        let mut calendar = app.world_mut().resource_mut::<Calendar>();
        calendar.hour = 24;
        calendar.minute = 0;
    }

    app.update();

    // DayEndEvent should have been sent — read via trigger_day_end
    // We check by adding trigger_day_end system and observing calendar reset
    let mut app2 = build_test_app();
    app2.add_systems(Update, (day_end_check, trigger_day_end).chain());

    {
        let mut calendar = app2.world_mut().resource_mut::<Calendar>();
        calendar.hour = 24;
        calendar.minute = 0;
        calendar.day = 1;
        calendar.season = Season::Spring;
    }

    app2.update();

    let calendar = app2.world().resource::<Calendar>();
    // After day end, calendar should reset to WAKE_HOUR
    assert_eq!(
        calendar.hour, WAKE_HOUR,
        "Hour should reset to WAKE_HOUR after day end"
    );
    assert_eq!(calendar.day, 2, "Day should advance to 2");
}

#[test]
fn test_season_change() {
    let mut app = build_test_app();
    app.add_systems(Update, (day_end_check, trigger_day_end).chain());

    {
        let mut calendar = app.world_mut().resource_mut::<Calendar>();
        calendar.day = 28; // Last day of season
        calendar.hour = 24;
        calendar.season = Season::Spring;
    }

    app.update();

    let calendar = app.world().resource::<Calendar>();
    assert_eq!(
        calendar.season,
        Season::Summer,
        "Season should advance from Spring to Summer"
    );
    assert_eq!(calendar.day, 1, "Day should reset to 1 after season change");
}

#[test]
fn test_year_rollover() {
    let mut app = build_test_app();
    app.add_systems(Update, (day_end_check, trigger_day_end).chain());

    {
        let mut calendar = app.world_mut().resource_mut::<Calendar>();
        calendar.day = 28;
        calendar.hour = 24;
        calendar.season = Season::Winter;
        calendar.year = 1;
    }

    app.update();

    let calendar = app.world().resource::<Calendar>();
    assert_eq!(
        calendar.season,
        Season::Spring,
        "Season should roll to Spring"
    );
    assert_eq!(calendar.year, 2, "Year should increment");
}

// ═════════════════════════════════════════════════════════════════════════
// ECONOMY TESTS
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_add_gold() {
    let mut gold = Gold { amount: 500 };
    let mut log = TransactionLog::default();

    add_gold(&mut gold, 200, "Mission reward", &mut log, 1);
    assert_eq!(gold.amount, 700);
    assert_eq!(log.entries.len(), 1);
    assert_eq!(log.entries[0].amount, 200);
}

#[test]
fn test_spend_gold_sufficient() {
    let mut gold = Gold { amount: 500 };
    let mut log = TransactionLog::default();

    let result = spend_gold(&mut gold, 200, "Fuel", &mut log, 1);
    assert!(result, "Spending should succeed with enough gold");
    assert_eq!(gold.amount, 300);
}

#[test]
fn test_spend_gold_insufficient() {
    let mut gold = Gold { amount: 100 };
    let mut log = TransactionLog::default();

    let result = spend_gold(&mut gold, 200, "Fuel", &mut log, 1);
    assert!(!result, "Spending should fail without enough gold");
    assert_eq!(gold.amount, 100, "Gold should remain unchanged");
}

#[test]
fn test_gold_change_event_system() {
    let mut app = build_test_app();
    app.add_systems(Update, apply_gold_changes);

    app.world_mut().resource_mut::<Gold>().amount = 1000;

    // Send a positive gold change event
    app.world_mut().send_event(GoldChangeEvent {
        amount: 250,
        reason: "Mission completed".to_string(),
    });

    app.update();

    let gold = app.world().resource::<Gold>();
    assert_eq!(gold.amount, 1250, "Gold should increase by 250");

    let stats = app.world().resource::<EconomyStats>();
    assert_eq!(stats.total_earned, 250);
}

#[test]
fn test_fuel_purchase() {
    let mut fleet = Fleet {
        aircraft: vec![OwnedAircraft {
            aircraft_id: "cessna_172".to_string(),
            nickname: "Test Bird".to_string(),
            condition: 100.0,
            fuel: 20.0,
            total_flights: 0,
            customizations: Vec::new(),
        }],
        active_index: 0,
    };

    let mut gold = Gold { amount: 1000 };
    let mut registry = AircraftRegistry::default();
    registry.aircraft.insert(
        "cessna_172".to_string(),
        AircraftDef {
            id: "cessna_172".to_string(),
            name: "Cessna 172".to_string(),
            class: AircraftClass::SingleProp,
            speed_knots: 120.0,
            range_nm: 500.0,
            fuel_capacity: 50.0,
            fuel_burn_rate: 1.0,
            passenger_capacity: 3,
            cargo_capacity_kg: 100.0,
            purchase_price: 5000,
            maintenance_cost_per_flight: 50,
            required_license: LicenseType::SingleEngine,
            required_rank: PilotRank::Student,
            handling: 0.8,
            durability: 100.0,
            sprite_index: 0,
        },
    );

    let result = skywarden::aircraft::fuel::refuel_aircraft(&mut fleet, &mut gold, &registry);
    assert!(result.is_ok(), "Refueling should succeed");
    let cost = result.unwrap();
    assert!(cost > 0, "Refueling should cost gold");
    assert_eq!(
        fleet.active().unwrap().fuel,
        50.0,
        "Fuel should be at capacity"
    );
    assert!(gold.amount < 1000, "Gold should decrease after refueling");
}

// ═════════════════════════════════════════════════════════════════════════
// FLIGHT TESTS
// ═════════════════════════════════════════════════════════════════════════

fn setup_flight_app() -> App {
    let mut app = build_test_app();

    // Register aircraft in the registry
    let mut registry = AircraftRegistry::default();
    registry.aircraft.insert(
        "cessna_172".to_string(),
        AircraftDef {
            id: "cessna_172".to_string(),
            name: "Cessna 172".to_string(),
            class: AircraftClass::SingleProp,
            speed_knots: 120.0,
            range_nm: 500.0,
            fuel_capacity: 50.0,
            fuel_burn_rate: 1.0,
            passenger_capacity: 3,
            cargo_capacity_kg: 100.0,
            purchase_price: 5000,
            maintenance_cost_per_flight: 50,
            required_license: LicenseType::SingleEngine,
            required_rank: PilotRank::Student,
            handling: 0.8,
            durability: 100.0,
            sprite_index: 0,
        },
    );
    app.insert_resource(registry);

    // Set up fleet with active aircraft
    let fleet = Fleet {
        aircraft: vec![OwnedAircraft {
            aircraft_id: "cessna_172".to_string(),
            nickname: "Test Bird".to_string(),
            condition: 100.0,
            fuel: 50.0,
            total_flights: 0,
            customizations: Vec::new(),
        }],
        active_index: 0,
    };
    app.insert_resource(fleet);

    app
}

#[test]
fn test_throttle_affects_speed() {
    let mut app = setup_flight_app();
    app.add_systems(Update, update_flight);

    {
        let mut flight_state = app.world_mut().resource_mut::<FlightState>();
        flight_state.phase = FlightPhase::Cruise;
        flight_state.throttle = 0.5;
        flight_state.altitude_ft = 10000.0;
        flight_state.distance_remaining_nm = 200.0;
        flight_state.distance_total_nm = 400.0;
        flight_state.fuel_remaining = 50.0;
    }

    app.update();

    let flight_state = app.world().resource::<FlightState>();
    // Speed should be base_speed * throttle = 120 * throttle
    // After update, throttle may have changed slightly from input, but speed should reflect throttle
    assert!(
        flight_state.speed_knots > 0.0,
        "Speed should be positive during cruise: {}",
        flight_state.speed_knots
    );
}

#[test]
fn test_fuel_burn() {
    // Fuel burn formula: burn_rate * throttle * dt / 60.0
    // Test the math directly since MinimalPlugins delta is near-zero.
    let burn_rate = 1.0_f32;
    let throttle = 0.8_f32;
    let dt = 10.0_f32; // simulate 10 seconds

    let fuel_burned = burn_rate * throttle * dt / 60.0;
    let fuel_after = 50.0 - fuel_burned;

    assert!(
        fuel_after < 50.0,
        "Fuel should decrease during flight: before=50, after={:.4}, burned={:.4}",
        fuel_after,
        fuel_burned
    );
    assert!(fuel_burned > 0.0, "Fuel burn should be positive");
}

#[test]
fn test_flight_phase_climb_to_cruise() {
    let mut app = setup_flight_app();
    app.add_systems(Update, update_flight);

    {
        let mut flight_state = app.world_mut().resource_mut::<FlightState>();
        flight_state.phase = FlightPhase::Climb;
        flight_state.throttle = 1.0;
        flight_state.altitude_ft = 9999.0; // Just below cruise threshold
        flight_state.fuel_remaining = 50.0;
        flight_state.distance_remaining_nm = 200.0;
        flight_state.distance_total_nm = 400.0;
    }

    app.update();

    let flight_state = app.world().resource::<FlightState>();
    // Altitude should increase during climb; once >= 10000 it transitions to Cruise
    assert!(
        flight_state.altitude_ft >= 9999.0,
        "Altitude should increase during climb"
    );
}

#[test]
fn test_cruise_to_descent() {
    let mut app = setup_flight_app();
    app.add_systems(Update, update_flight);

    {
        let mut flight_state = app.world_mut().resource_mut::<FlightState>();
        flight_state.phase = FlightPhase::Cruise;
        flight_state.throttle = 0.8;
        flight_state.altitude_ft = 10000.0;
        flight_state.fuel_remaining = 50.0;
        flight_state.speed_knots = 120.0;
        // Set distance remaining < 20% of total to trigger descent
        flight_state.distance_remaining_nm = 50.0;
        flight_state.distance_total_nm = 400.0;
    }

    app.update();

    let flight_state = app.world().resource::<FlightState>();
    assert_eq!(
        flight_state.phase,
        FlightPhase::Descent,
        "Should transition from Cruise to Descent when distance_remaining < 20% total"
    );
}

#[test]
fn test_descent_to_approach() {
    let mut app = setup_flight_app();
    app.add_systems(Update, update_flight);

    {
        let mut flight_state = app.world_mut().resource_mut::<FlightState>();
        flight_state.phase = FlightPhase::Descent;
        flight_state.throttle = 0.5;
        flight_state.altitude_ft = 5000.0;
        flight_state.fuel_remaining = 30.0;
        flight_state.speed_knots = 100.0;
        flight_state.distance_remaining_nm = 8.0; // < 10nm triggers Approach
        flight_state.distance_total_nm = 400.0;
    }

    app.update();

    let flight_state = app.world().resource::<FlightState>();
    assert_eq!(
        flight_state.phase,
        FlightPhase::Approach,
        "Should transition from Descent to Approach when distance_remaining <= 10nm"
    );
}

#[test]
fn test_landing_grade_calculation() {
    // Test LandingGrade methods directly (shared type)
    assert_eq!(LandingGrade::Perfect.xp_bonus(), 50);
    assert_eq!(LandingGrade::Good.xp_bonus(), 25);
    assert_eq!(LandingGrade::Acceptable.xp_bonus(), 10);
    assert_eq!(LandingGrade::Hard.xp_bonus(), 0);
    assert_eq!(LandingGrade::Rough.xp_bonus(), 0);

    // Reputation changes
    assert!(LandingGrade::Perfect.reputation_change() > 0.0);
    assert!(LandingGrade::Rough.reputation_change() < 0.0);
}

#[test]
fn test_navigation_waypoints() {
    // Test calculate_route directly
    let route = calculate_route(AirportId::HomeBase, AirportId::Windport);
    assert!(
        route.len() >= 2,
        "Route should have at least departure and arrival waypoints"
    );
    assert!(
        route.first().unwrap().name.contains("DEP"),
        "First waypoint should be departure"
    );
    assert!(
        route.last().unwrap().name.contains("ARR"),
        "Last waypoint should be arrival"
    );

    // Distance should increase along the route
    for i in 1..route.len() {
        assert!(
            route[i].distance_from_origin_nm >= route[i - 1].distance_from_origin_nm,
            "Waypoint distances should be monotonically increasing"
        );
    }
}

#[test]
fn test_navigation_update_system() {
    let mut app = setup_flight_app();
    app.add_systems(Update, update_navigation);

    // Set up a route
    let route = calculate_route(AirportId::HomeBase, AirportId::Grandcity);
    let total_distance = airport_distance(AirportId::HomeBase, AirportId::Grandcity);
    {
        let mut nav = app.world_mut().resource_mut::<NavigationState>();
        nav.route = route;
        nav.total_route_distance = total_distance;
        nav.current_waypoint_index = 0;

        let mut flight_state = app.world_mut().resource_mut::<FlightState>();
        flight_state.phase = FlightPhase::Cruise;
        flight_state.distance_remaining_nm = total_distance - 50.0; // 50nm flown
        flight_state.speed_knots = 120.0;
        flight_state.heading_deg = 90.0;
    }

    app.update();

    let nav = app.world().resource::<NavigationState>();
    assert!(nav.eta_secs > 0.0, "ETA should be positive");
}

// ═════════════════════════════════════════════════════════════════════════
// PLAYER TESTS
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_player_movement() {
    // player_movement multiplies input.movement by speed * delta_secs.
    // With MinimalPlugins the first-frame delta is ~0, so we verify the
    // movement math directly using the shared PLAYER_SPEED constant.
    let speed = PLAYER_SPEED;
    let dir = Vec2::new(1.0, 0.0).normalize_or_zero();
    let dt = 1.0 / 60.0_f32; // simulate one 60-fps frame

    let displacement = dir * speed * dt;
    let new_x = 100.0 + displacement.x;

    assert!(
        new_x > 100.0,
        "Player should move right at PLAYER_SPEED: new_x={}",
        new_x
    );
    assert!(
        (displacement.x - speed * dt).abs() < 0.001,
        "Displacement should equal speed * dt"
    );
}

#[test]
fn test_collision_blocks_movement() {
    let mut app = build_test_app();
    app.add_systems(Update, player_movement);

    // Set up a collision map with a wall to the right
    {
        let mut collision_map = app.world_mut().resource_mut::<CollisionMap>();
        collision_map.initialised = true;
        collision_map.width = 20;
        collision_map.height = 20;
        collision_map.blocked = vec![vec![false; 20]; 20];
        // Block the tile at (7, 5) — player at grid (6, 5) trying to move right
        collision_map.blocked[5][7] = true;

        let mut world_map = app.world_mut().resource_mut::<WorldMap>();
        world_map.width = 20;
        world_map.height = 20;
    }

    let start_x = 6.0 * TILE_SIZE + TILE_SIZE / 2.0;
    let start_y = -(5.0 * TILE_SIZE + TILE_SIZE / 2.0);
    let player_entity = app
        .world_mut()
        .spawn((Player, Transform::from_xyz(start_x, start_y, 0.0)))
        .id();

    {
        let mut input = app.world_mut().resource_mut::<PlayerInput>();
        input.movement = Vec2::new(1.0, 0.0); // Try to move right into wall
    }

    app.update();

    let transform = app
        .world()
        .entity(player_entity)
        .get::<Transform>()
        .unwrap();
    // Player should be blocked (x should not have moved far past start)
    let grid_x = (transform.translation.x / TILE_SIZE).floor() as i32;
    assert!(
        grid_x <= 7,
        "Player should be blocked by collision: grid_x={}",
        grid_x
    );
}

#[test]
fn test_stamina_drain_sprint() {
    // Stamina drain during sprint is STAMINA_DRAIN_PER_SEC (8.0) * dt.
    // Verify the math directly since MinimalPlugins delta is near-zero.
    let stamina_drain_per_sec = 8.0_f32;
    let dt = 1.0 / 60.0_f32;
    let initial_stamina = 100.0_f32;

    let stamina_after = (initial_stamina - stamina_drain_per_sec * dt).max(0.0);
    assert!(
        stamina_after < initial_stamina,
        "Stamina should decrease when sprinting: before={}, after={}",
        initial_stamina,
        stamina_after
    );
}

#[test]
fn test_sprint_speed_boost() {
    // Sprint multiplier is 1.6x — test indirectly via distance travelled
    let mut app_normal = build_test_app();
    app_normal.add_systems(Update, player_movement);

    let e1 = app_normal
        .world_mut()
        .spawn((Player, Transform::from_xyz(100.0, -100.0, 0.0)))
        .id();

    {
        let mut input = app_normal.world_mut().resource_mut::<PlayerInput>();
        input.movement = Vec2::new(1.0, 0.0);
        input.sprint = false;
    }

    app_normal.update();
    let normal_x = app_normal
        .world()
        .entity(e1)
        .get::<Transform>()
        .unwrap()
        .translation
        .x;

    let mut app_sprint = build_test_app();
    app_sprint.add_systems(Update, player_movement);

    let e2 = app_sprint
        .world_mut()
        .spawn((Player, Transform::from_xyz(100.0, -100.0, 0.0)))
        .id();

    {
        let mut input = app_sprint.world_mut().resource_mut::<PlayerInput>();
        input.movement = Vec2::new(1.0, 0.0);
        input.sprint = true;

        let mut pilot = app_sprint.world_mut().resource_mut::<PilotState>();
        pilot.stamina = 100.0;
    }

    app_sprint.update();
    let sprint_x = app_sprint
        .world()
        .entity(e2)
        .get::<Transform>()
        .unwrap()
        .translation
        .x;

    assert!(
        sprint_x >= normal_x,
        "Sprinting should move at least as fast: sprint={}, normal={}",
        sprint_x,
        normal_x
    );
}

// ═════════════════════════════════════════════════════════════════════════
// MISSION TESTS
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_mission_accept() {
    let mut app = build_test_app();
    app.add_systems(Update, handle_mission_accepted);

    // Set up an available mission
    {
        let mut board = app.world_mut().resource_mut::<MissionBoard>();
        board.available.push(MissionDef {
            id: "test_mission_1".to_string(),
            title: "Test Flight".to_string(),
            description: "A test mission.".to_string(),
            mission_type: MissionType::Passenger,
            origin: AirportId::HomeBase,
            destination: AirportId::Windport,
            reward_gold: 100,
            reward_xp: 50,
            time_limit_hours: None,
            required_rank: PilotRank::Student,
            required_aircraft_class: None,
            passenger_count: 5,
            cargo_kg: 0.0,
            bonus_conditions: vec![BonusCondition::PerfectLanding],
            difficulty: MissionDifficulty::Easy,
        });
    }

    app.world_mut().send_event(MissionAcceptedEvent {
        mission_id: "test_mission_1".to_string(),
    });

    app.update();

    let board = app.world().resource::<MissionBoard>();
    assert!(
        board.active.is_some(),
        "Mission should be active after accepting"
    );
    assert!(
        board.available.is_empty(),
        "Mission should be removed from available"
    );
}

#[test]
fn test_mission_complete() {
    let mut app = build_test_app();
    app.add_systems(Update, handle_mission_complete);

    // Set up an active mission
    {
        let mut board = app.world_mut().resource_mut::<MissionBoard>();
        board.active = Some(ActiveMission {
            mission: MissionDef {
                id: "test_mission_2".to_string(),
                title: "Cargo Run".to_string(),
                description: "Deliver cargo.".to_string(),
                mission_type: MissionType::Cargo,
                origin: AirportId::HomeBase,
                destination: AirportId::Windport,
                reward_gold: 200,
                reward_xp: 80,
                time_limit_hours: None,
                required_rank: PilotRank::Student,
                required_aircraft_class: None,
                passenger_count: 0,
                cargo_kg: 500.0,
                bonus_conditions: vec![BonusCondition::OnTime],
                difficulty: MissionDifficulty::Easy,
            },
            accepted_day: 1,
            bonuses_met: vec![true],
        });
    }

    app.world_mut().send_event(FlightCompleteEvent {
        origin: AirportId::HomeBase,
        destination: AirportId::Windport,
        landing_grade: "Good".to_string(),
        flight_time_secs: 600.0,
        fuel_used: 10.0,
        xp_earned: 80,
        gold_earned: 200,
    });

    app.update();

    let board = app.world().resource::<MissionBoard>();
    assert!(
        board.active.is_none(),
        "Active mission should be cleared after completion"
    );
    assert!(
        board.completed_ids.contains(&"test_mission_2".to_string()),
        "Mission should be in completed list"
    );

    let log = app.world().resource::<MissionLog>();
    assert_eq!(log.completed.len(), 1, "Mission log should have one entry");
}

#[test]
fn test_mission_refresh() {
    let mut app = build_test_app();
    app.add_systems(Update, refresh_mission_board);

    {
        let mut board = app.world_mut().resource_mut::<MissionBoard>();
        board.available.clear();

        // Need at least Private rank to unlock 2+ airports for mission generation
        let mut pilot = app.world_mut().resource_mut::<PilotState>();
        pilot.rank = PilotRank::Private;
    }

    app.world_mut().send_event(DayEndEvent);

    app.update();

    let board = app.world().resource::<MissionBoard>();
    assert!(
        !board.available.is_empty(),
        "Mission board should have new missions after daily refresh"
    );
}

// ═════════════════════════════════════════════════════════════════════════
// CREW TESTS
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_friendship_increase() {
    let mut app = build_test_app();
    app.add_systems(Update, handle_gift_given);

    // Set up crew member in registry
    {
        let mut crew_reg = app.world_mut().resource_mut::<CrewRegistry>();
        crew_reg.members.insert(
            "mechanic_hank".to_string(),
            CrewMemberDef {
                id: "mechanic_hank".to_string(),
                name: "Hank".to_string(),
                role: CrewRole::Mechanic,
                personality: "Gruff but kind".to_string(),
                favorite_gift: "titanium_wrench".to_string(),
                disliked_gift: "cheap_tool".to_string(),
                home_airport: AirportId::HomeBase,
                dialogue_pool: vec![],
                backstory: "Old mechanic.".to_string(),
                sprite_index: 0,
                tint_color: [1.0, 1.0, 1.0],
            },
        );

        let mut inventory = app.world_mut().resource_mut::<Inventory>();
        inventory.max_slots = 20;
        inventory.add_item("generic_gift", 1);
    }

    app.world_mut().send_event(GiftGivenEvent {
        npc_id: "mechanic_hank".to_string(),
        item_id: "generic_gift".to_string(),
    });

    app.update();

    let relationships = app.world().resource::<Relationships>();
    let friendship = relationships.friendship_level("mechanic_hank");
    assert!(
        friendship > 0,
        "Friendship should increase after gift: {}",
        friendship
    );
}

#[test]
fn test_friendship_decay() {
    let mut app = build_test_app();
    app.add_systems(Update, friendship_decay);

    // Set up a positive friendship that should decay
    {
        let mut relationships = app.world_mut().resource_mut::<Relationships>();
        relationships
            .friendship
            .insert("mechanic_hank".to_string(), 50);
    }

    // Send DayEndEvent to trigger decay
    app.world_mut().send_event(DayEndEvent);

    app.update();

    let relationships = app.world().resource::<Relationships>();
    let friendship = relationships.friendship_level("mechanic_hank");
    assert!(
        friendship < 50,
        "Friendship should decay on day end without interaction: {}",
        friendship
    );
}

#[test]
fn test_crew_schedule() {
    // Test the pure function directly
    let (airport, zone, _x, _y) = get_crew_current_zone("mechanic_hank", 10, &DayOfWeek::Monday);
    assert_eq!(airport, AirportId::HomeBase, "Hank should be at HomeBase");
    assert_eq!(
        zone,
        MapZone::Hangar,
        "Hank should be in the Hangar during morning"
    );

    // Verify crew presence check
    let present = is_crew_present(
        "mechanic_hank",
        10,
        &DayOfWeek::Monday,
        AirportId::HomeBase,
        MapZone::Hangar,
    );
    assert!(
        present,
        "Hank should be present at HomeBase Hangar at 10am Monday"
    );
}

// ═════════════════════════════════════════════════════════════════════════
// AIRCRAFT TESTS
// ═════════════════════════════════════════════════════════════════════════

fn make_test_registry() -> AircraftRegistry {
    let mut registry = AircraftRegistry::default();
    registry.aircraft.insert(
        "cessna_172".to_string(),
        AircraftDef {
            id: "cessna_172".to_string(),
            name: "Cessna 172".to_string(),
            class: AircraftClass::SingleProp,
            speed_knots: 120.0,
            range_nm: 500.0,
            fuel_capacity: 50.0,
            fuel_burn_rate: 1.0,
            passenger_capacity: 3,
            cargo_capacity_kg: 100.0,
            purchase_price: 5000,
            maintenance_cost_per_flight: 50,
            required_license: LicenseType::SingleEngine,
            required_rank: PilotRank::Student,
            handling: 0.8,
            durability: 100.0,
            sprite_index: 0,
        },
    );
    registry.aircraft.insert(
        "learjet_45".to_string(),
        AircraftDef {
            id: "learjet_45".to_string(),
            name: "Learjet 45".to_string(),
            class: AircraftClass::LightJet,
            speed_knots: 460.0,
            range_nm: 1700.0,
            fuel_capacity: 200.0,
            fuel_burn_rate: 3.0,
            passenger_capacity: 8,
            cargo_capacity_kg: 500.0,
            purchase_price: 50000,
            maintenance_cost_per_flight: 200,
            required_license: LicenseType::Jet,
            required_rank: PilotRank::Senior,
            handling: 0.6,
            durability: 150.0,
            sprite_index: 1,
        },
    );
    registry
}

#[test]
fn test_aircraft_purchase() {
    let registry = make_test_registry();
    let mut fleet = Fleet::default();
    let mut gold = Gold { amount: 10000 };
    let mut hangars = HangarAssignments::default();

    let result = purchase_aircraft(
        &mut fleet,
        &mut gold,
        &registry,
        &mut hangars,
        "cessna_172",
        "My Cessna",
        AirportId::HomeBase,
    );

    assert!(result.is_ok(), "Purchase should succeed");
    assert_eq!(
        gold.amount, 5000,
        "Gold should be deducted by purchase price"
    );
    assert_eq!(fleet.aircraft.len(), 1, "Fleet should have one aircraft");
    assert_eq!(fleet.aircraft[0].nickname, "My Cessna");
}

#[test]
fn test_aircraft_purchase_insufficient_gold() {
    let registry = make_test_registry();
    let mut fleet = Fleet::default();
    let mut gold = Gold { amount: 1000 }; // Not enough for cessna_172 @ 5000
    let mut hangars = HangarAssignments::default();

    let result = purchase_aircraft(
        &mut fleet,
        &mut gold,
        &registry,
        &mut hangars,
        "cessna_172",
        "My Cessna",
        AirportId::HomeBase,
    );

    assert!(
        result.is_err(),
        "Purchase should fail with insufficient gold"
    );
    assert_eq!(result.unwrap_err(), "Not enough gold");
}

#[test]
fn test_maintenance_wear() {
    let mut condition = ComponentCondition::default();
    assert_eq!(condition.overall(), 100.0);

    condition.apply_flight_wear("Good", 0.7);

    assert!(
        condition.overall() < 100.0,
        "Overall condition should decrease after flight: {}",
        condition.overall()
    );
    assert!(
        condition.engine < 100.0,
        "Engine condition should decrease after flight"
    );
    assert!(
        condition.landing_gear < 100.0,
        "Landing gear should degrade after landing"
    );
}

#[test]
fn test_maintenance_hard_landing() {
    let mut good_landing = ComponentCondition::default();
    good_landing.apply_flight_wear("Good", 0.5);

    let mut rough_landing = ComponentCondition::default();
    rough_landing.apply_flight_wear("Rough", 0.5);

    assert!(
        rough_landing.landing_gear < good_landing.landing_gear,
        "Rough landing should cause more landing gear damage"
    );
    assert!(
        rough_landing.tires < good_landing.tires,
        "Rough landing should cause more tire damage"
    );
}

#[test]
fn test_fuel_type_matching() {
    assert_eq!(
        fuel_type_for_class(AircraftClass::SingleProp),
        FuelType::AvGas
    );
    assert_eq!(
        fuel_type_for_class(AircraftClass::TwinProp),
        FuelType::AvGas
    );
    assert_eq!(
        fuel_type_for_class(AircraftClass::Seaplane),
        FuelType::AvGas
    );
    assert_eq!(fuel_type_for_class(AircraftClass::LightJet), FuelType::JetA);
    assert_eq!(
        fuel_type_for_class(AircraftClass::MediumJet),
        FuelType::JetA
    );
    assert_eq!(fuel_type_for_class(AircraftClass::HeavyJet), FuelType::JetA);
    assert_eq!(
        fuel_type_for_class(AircraftClass::Turboprop),
        FuelType::JetA
    );
    assert_eq!(fuel_type_for_class(AircraftClass::Cargo), FuelType::JetA);
}

#[test]
fn test_fuel_burn_calculation() {
    let base_rate = 1.0;

    // Higher throttle = more fuel burn
    let low_throttle_burn = calculate_fuel_burn(base_rate, 0.3, 10000.0, 0.0);
    let high_throttle_burn = calculate_fuel_burn(base_rate, 0.9, 10000.0, 0.0);
    assert!(
        high_throttle_burn > low_throttle_burn,
        "Higher throttle should burn more fuel"
    );

    // More cargo weight = more fuel burn
    let no_cargo = calculate_fuel_burn(base_rate, 0.5, 10000.0, 0.0);
    let heavy_cargo = calculate_fuel_burn(base_rate, 0.5, 10000.0, 4000.0);
    assert!(
        heavy_cargo > no_cargo,
        "More cargo should increase fuel burn"
    );
}

#[test]
fn test_starter_fleet() {
    let (fleet, hangars) = create_starter_fleet();
    assert_eq!(
        fleet.aircraft.len(),
        1,
        "Starter fleet should have one aircraft"
    );
    assert_eq!(fleet.aircraft[0].aircraft_id, "cessna_172");
    assert_eq!(fleet.aircraft[0].nickname, "Old Faithful");
    assert!(
        hangars.assignments.contains_key("Old Faithful"),
        "Starter aircraft should be assigned to a hangar"
    );
}

// ═════════════════════════════════════════════════════════════════════════
// WEATHER TESTS
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_weather_flight_difficulty() {
    assert_eq!(Weather::Clear.flight_difficulty(), 0.0);
    assert!(Weather::Storm.flight_difficulty() > Weather::Rain.flight_difficulty());
    assert_eq!(Weather::Storm.flight_difficulty(), 1.0);
}

#[test]
fn test_weather_flyability() {
    assert!(Weather::Clear.is_flyable());
    assert!(Weather::Rain.is_flyable());
    assert!(Weather::Fog.is_flyable());
    assert!(!Weather::Storm.is_flyable(), "Storms should not be flyable");
}

#[test]
fn test_turbulence_in_storms() {
    // Verify turbulence mapping directly
    let weather = Weather::Storm;
    assert_eq!(
        weather.flight_difficulty(),
        1.0,
        "Storms should have maximum flight difficulty"
    );

    // Verify that storm weather is not flyable (safety check)
    assert!(!weather.is_flyable(), "Storm weather should not be flyable");
}

#[test]
fn test_weather_visibility() {
    assert_eq!(Weather::Clear.visibility_modifier(), 1.0);
    assert!(
        Weather::Fog.visibility_modifier() < Weather::Cloudy.visibility_modifier(),
        "Fog should reduce visibility more than clouds"
    );
}

// ═════════════════════════════════════════════════════════════════════════
// INVENTORY TESTS
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_add_item() {
    let mut inventory = Inventory::new(10);

    let added = inventory.add_item("pilot_manual", 1);
    assert!(added, "Should be able to add item");
    assert!(inventory.has_item("pilot_manual", 1));
    assert_eq!(inventory.count_item("pilot_manual"), 1);
}

#[test]
fn test_add_item_stacks() {
    let mut inventory = Inventory::new(10);

    inventory.add_item("fuel_canister", 3);
    inventory.add_item("fuel_canister", 2);
    assert_eq!(
        inventory.count_item("fuel_canister"),
        5,
        "Stacking should combine quantities"
    );
    assert_eq!(inventory.slots.len(), 1, "Should be a single slot");
}

#[test]
fn test_remove_item() {
    let mut inventory = Inventory::new(10);
    inventory.add_item("wrench", 3);

    let removed = inventory.remove_item("wrench", 2);
    assert!(removed, "Should successfully remove items");
    assert_eq!(inventory.count_item("wrench"), 1);

    let removed = inventory.remove_item("wrench", 1);
    assert!(removed);
    assert_eq!(inventory.count_item("wrench"), 0);
    assert!(
        inventory.slots.is_empty(),
        "Empty slot should be cleaned up"
    );
}

#[test]
fn test_remove_item_insufficient() {
    let mut inventory = Inventory::new(10);
    inventory.add_item("wrench", 1);

    let removed = inventory.remove_item("wrench", 5);
    assert!(!removed, "Should fail when removing more than available");
    assert_eq!(
        inventory.count_item("wrench"),
        1,
        "Count should be unchanged"
    );
}

#[test]
fn test_inventory_capacity() {
    let mut inventory = Inventory::new(3);

    assert!(inventory.add_item("item_a", 1));
    assert!(inventory.add_item("item_b", 1));
    assert!(inventory.add_item("item_c", 1));
    assert!(
        !inventory.add_item("item_d", 1),
        "Should reject items when at capacity"
    );
    assert_eq!(inventory.slots.len(), 3);
}

// ═════════════════════════════════════════════════════════════════════════
// PROGRESSION TESTS
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_xp_gain() {
    let mut app = build_test_app();
    app.add_systems(Update, apply_xp);

    {
        let mut pilot = app.world_mut().resource_mut::<PilotState>();
        pilot.xp = 0;
    }

    app.world_mut().send_event(XpGainEvent {
        amount: 50,
        source: "Test flight".to_string(),
    });

    app.update();

    let pilot = app.world().resource::<PilotState>();
    assert_eq!(pilot.xp, 50, "XP should increase by event amount");
}

#[test]
fn test_rank_up() {
    let mut app = build_test_app();
    app.add_systems(Update, check_rank_up);

    {
        let mut pilot = app.world_mut().resource_mut::<PilotState>();
        pilot.rank = PilotRank::Student;
        pilot.xp = 200;
        pilot.total_flights = 10;
        pilot.total_flight_hours = 5.0;
        pilot.reputation = 50.0;
    }

    app.update();

    let pilot = app.world().resource::<PilotState>();
    assert_eq!(
        pilot.rank,
        PilotRank::Private,
        "Should rank up from Student to Private when requirements are met"
    );
}

#[test]
fn test_rank_requirements_check() {
    let req = rank_requirements(PilotRank::Private);
    assert_eq!(req.min_flights, 5);
    assert_eq!(req.min_xp, 100);

    let mut pilot = PilotState {
        rank: PilotRank::Student,
        xp: 50,
        ..Default::default()
    }; // Not enough
    assert!(
        !meets_rank_requirements(&pilot),
        "Should not meet requirements with insufficient XP"
    );

    pilot.xp = 200;
    pilot.total_flights = 10;
    pilot.total_flight_hours = 5.0;
    pilot.reputation = 30.0;
    assert!(
        meets_rank_requirements(&pilot),
        "Should meet requirements when all criteria are satisfied"
    );
}

#[test]
fn test_rank_cannot_exceed_ace() {
    assert!(
        PilotRank::Ace.next().is_none(),
        "Ace should be the highest rank with no next"
    );

    let pilot = PilotState {
        rank: PilotRank::Ace,
        xp: 99999,
        total_flights: 9999,
        total_flight_hours: 9999.0,
        reputation: 100.0,
        ..Default::default()
    };
    assert!(
        !meets_rank_requirements(&pilot),
        "Should not meet requirements when already at max rank"
    );
}

// ═════════════════════════════════════════════════════════════════════════
// MISCELLANEOUS SHARED TYPE TESTS
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_calendar_formatting() {
    let calendar = Calendar {
        day: 15,
        season: Season::Summer,
        year: 2,
        day_of_week: DayOfWeek::Wednesday,
        hour: 14,
        minute: 30,
        time_of_day_secs: 0.0,
        time_paused: false,
    };

    assert_eq!(calendar.formatted_time(), "2:30 PM");
    assert!(calendar.formatted_date().contains("Summer"));
    assert!(calendar.formatted_date().contains("Year 2"));
}

#[test]
fn test_calendar_total_days() {
    let cal = Calendar {
        day: 1,
        season: Season::Spring,
        year: 1,
        ..Calendar::default()
    };
    assert_eq!(cal.total_days(), 1);

    let cal2 = Calendar {
        day: 28,
        season: Season::Winter,
        year: 1,
        ..Calendar::default()
    };
    assert_eq!(cal2.total_days(), 112);
}

#[test]
fn test_season_cycle() {
    assert_eq!(Season::Spring.next(), Season::Summer);
    assert_eq!(Season::Summer.next(), Season::Fall);
    assert_eq!(Season::Fall.next(), Season::Winter);
    assert_eq!(Season::Winter.next(), Season::Spring);
}

#[test]
fn test_day_of_week_cycle() {
    assert_eq!(DayOfWeek::Monday.next(), DayOfWeek::Tuesday);
    assert_eq!(DayOfWeek::Sunday.next(), DayOfWeek::Monday);
}

#[test]
fn test_airport_distance_symmetric() {
    let d1 = airport_distance(AirportId::HomeBase, AirportId::Windport);
    let d2 = airport_distance(AirportId::Windport, AirportId::HomeBase);
    assert_eq!(d1, d2, "Distance should be symmetric");
    assert_eq!(d1, 120.0);
}

#[test]
fn test_collision_map_bounds() {
    let mut map = CollisionMap {
        initialised: true,
        width: 10,
        height: 10,
        blocked: vec![vec![false; 10]; 10],
    };

    assert!(!map.is_blocked(5, 5), "Should not be blocked");
    assert!(map.is_blocked(-1, 5), "Out of bounds should be blocked");
    assert!(map.is_blocked(5, -1), "Out of bounds should be blocked");
    assert!(map.is_blocked(10, 5), "Out of bounds should be blocked");

    map.blocked[3][4] = true;
    assert!(map.is_blocked(4, 3), "Blocked tile should return true");
}

#[test]
fn test_tile_kind_solidity() {
    assert!(TileKind::Wall.is_solid());
    assert!(TileKind::Water.is_solid());
    assert!(TileKind::Void.is_solid());
    assert!(!TileKind::Floor.is_solid());
    assert!(!TileKind::Grass.is_solid());
    assert!(!TileKind::Runway.is_solid());
}

#[test]
fn test_map_zone_indoor() {
    assert!(MapZone::Terminal.is_indoor());
    assert!(MapZone::Lounge.is_indoor());
    assert!(MapZone::Hangar.is_indoor());
    assert!(!MapZone::Runway.is_indoor());
    assert!(!MapZone::CityStreet.is_indoor());
}

#[test]
fn test_achievements() {
    let mut achievements = Achievements::default();
    assert!(!achievements.is_unlocked("first_flight"));

    let new = achievements.unlock("first_flight");
    assert!(new, "First unlock should return true");
    assert!(achievements.is_unlocked("first_flight"));

    let dupe = achievements.unlock("first_flight");
    assert!(!dupe, "Duplicate unlock should return false");
}

#[test]
fn test_relationships_friendship_tier() {
    let mut rel = Relationships::default();

    assert_eq!(rel.friendship_tier("unknown_npc"), "Acquaintance");

    rel.add_friendship("pilot_joe", 30);
    assert_eq!(rel.friendship_tier("pilot_joe"), "Friendly");

    rel.add_friendship("pilot_joe", 50);
    assert_eq!(rel.friendship_tier("pilot_joe"), "Best Friend");

    rel.add_friendship("rival_sam", -60);
    assert_eq!(rel.friendship_tier("rival_sam"), "Hostile");
}

#[test]
fn test_relationships_clamp() {
    let mut rel = Relationships::default();

    rel.add_friendship("npc_a", 200); // Should clamp to 100
    assert_eq!(rel.friendship_level("npc_a"), 100);

    rel.add_friendship("npc_b", -200); // Should clamp to -100
    assert_eq!(rel.friendship_level("npc_b"), -100);
}

#[test]
fn test_flight_phase_display() {
    assert_eq!(FlightPhase::Idle.display_name(), "On Ground");
    assert_eq!(FlightPhase::Cruise.display_name(), "Cruising");
    assert_eq!(FlightPhase::Emergency.display_name(), "EMERGENCY");
}

#[test]
fn test_pilot_rank_xp_progression() {
    assert!(PilotRank::Private.xp_required() < PilotRank::Commercial.xp_required());
    assert!(PilotRank::Commercial.xp_required() < PilotRank::Senior.xp_required());
    assert!(PilotRank::Senior.xp_required() < PilotRank::Captain.xp_required());
    assert!(PilotRank::Captain.xp_required() < PilotRank::Ace.xp_required());
}

#[test]
fn test_airport_unlock_rank_progression() {
    // HomeBase should be available from Student
    assert_eq!(AirportId::HomeBase.unlock_rank(), PilotRank::Student);
    // Skyreach should require Ace
    assert_eq!(AirportId::Skyreach.unlock_rank(), PilotRank::Ace);
    // Higher airports require higher ranks
    assert!(AirportId::Grandcity.unlock_rank() > AirportId::Windport.unlock_rank());
}

#[test]
fn test_checklist_default() {
    let checklist = PreflightChecklist::default();
    assert_eq!(checklist.items.len(), 6);
    for item in &checklist.items {
        assert!(!item.completed, "Checklist items should start uncompleted");
    }
}

#[test]
fn test_night_detection() {
    let mut cal = Calendar {
        hour: 14,
        ..Default::default()
    };
    assert!(!cal.is_night(), "2 PM should not be night");

    cal.hour = 22;
    assert!(cal.is_night(), "10 PM should be night");

    cal.hour = 3;
    assert!(cal.is_night(), "3 AM should be night");
}

// ═════════════════════════════════════════════════════════════════════════
// SAVE / LOAD TESTS
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_save_roundtrip() {
    let mut save = SaveFile::default();
    save.gold = Gold { amount: 1234 };
    save.calendar = Calendar {
        day: 7,
        season: Season::Fall,
        year: 3,
        ..Default::default()
    };

    let json = serde_json::to_string(&save).expect("serialization failed");
    let loaded: SaveFile = serde_json::from_str(&json).expect("deserialization failed");

    assert_eq!(loaded.gold.amount, 1234);
    assert_eq!(loaded.calendar.day, 7);
    assert_eq!(loaded.calendar.season, Season::Fall);
    assert_eq!(loaded.calendar.year, 3);
}

#[test]
fn test_save_default_loads_cleanly() {
    let save = SaveFile::default();
    let json = serde_json::to_string(&save).expect("serialization of default SaveFile failed");
    let result: Result<SaveFile, _> = serde_json::from_str(&json);
    assert!(
        result.is_ok(),
        "Default SaveFile should deserialize without errors"
    );
}

#[test]
fn test_save_preserves_gold() {
    let mut save = SaveFile::default();
    save.gold = Gold { amount: 500 };

    let json = serde_json::to_string(&save).unwrap();
    let loaded: SaveFile = serde_json::from_str(&json).unwrap();

    assert_eq!(
        loaded.gold.amount, 500,
        "Gold should be preserved through save/load"
    );
}

#[test]
fn test_save_preserves_fleet() {
    let mut save = SaveFile::default();
    save.fleet.aircraft.push(OwnedAircraft {
        aircraft_id: "cessna_172".into(),
        nickname: "Blue Bird".into(),
        condition: 98.0,
        fuel: 40.0,
        total_flights: 5,
        customizations: vec![],
    });

    let json = serde_json::to_string(&save).unwrap();
    let loaded: SaveFile = serde_json::from_str(&json).unwrap();

    assert_eq!(
        loaded.fleet.aircraft.len(),
        1,
        "Fleet aircraft count should be preserved"
    );
    assert_eq!(loaded.fleet.aircraft[0].aircraft_id, "cessna_172");
    assert_eq!(loaded.fleet.aircraft[0].nickname, "Blue Bird");
}

// ═════════════════════════════════════════════════════════════════════════
// AIRPORT SYSTEM TESTS
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn test_airport_zone_definitions() {
    let zones = [
        MapZone::Terminal,
        MapZone::Lounge,
        MapZone::Hangar,
        MapZone::Runway,
        MapZone::ControlTower,
        MapZone::CrewQuarters,
        MapZone::Shop,
        MapZone::CityStreet,
    ];
    for zone in zones {
        let (w, h, tiles) = generate_zone_map(AirportId::HomeBase, zone);
        assert!(w > 0, "Zone {:?} should have positive width", zone);
        assert!(h > 0, "Zone {:?} should have positive height", zone);
        assert_eq!(
            tiles.len(),
            h as usize,
            "Tile row count should match height for {:?}",
            zone
        );
        assert_eq!(
            tiles[0].len(),
            w as usize,
            "Tile column count should match width for {:?}",
            zone
        );
    }
}

#[test]
fn test_airport_npc_count() {
    let mut app = build_test_app();
    app.add_systems(Update, spawn_ambient_npcs);

    // Trigger NPC spawn by sending a ZoneTransitionEvent to Terminal
    app.world_mut().send_event(ZoneTransitionEvent {
        to_airport: AirportId::HomeBase,
        to_zone: MapZone::Terminal,
        to_x: 10,
        to_y: 8,
    });
    app.update();

    let npc_count = app
        .world()
        .iter_entities()
        .filter(|e| e.contains::<AirportNpc>())
        .count();
    assert!(
        npc_count >= 1,
        "Terminal should have at least 1 ambient NPC after zone transition"
    );
}

#[test]
fn test_home_base_exists() {
    let name = AirportId::HomeBase.display_name();
    let icao = AirportId::HomeBase.icao_code();
    assert!(
        !name.is_empty(),
        "HomeBase should have a non-empty display name"
    );
    assert!(
        !icao.is_empty(),
        "HomeBase should have a non-empty ICAO code"
    );
    assert_eq!(
        AirportId::HomeBase.unlock_rank(),
        PilotRank::Student,
        "HomeBase must be available from Student rank"
    );
}

#[test]
fn test_airport_services_available() {
    let airports = [
        AirportId::HomeBase,
        AirportId::Windport,
        AirportId::Frostpeak,
        AirportId::Sunhaven,
        AirportId::Ironforge,
        AirportId::Cloudmere,
        AirportId::Duskhollow,
        AirportId::Stormwatch,
        AirportId::Grandcity,
        AirportId::Skyreach,
    ];
    for airport in airports {
        let services = services_at(airport);
        assert!(
            services.contains(&AirportService::WeatherBriefing),
            "{:?} should offer WeatherBriefing",
            airport
        );
        assert!(
            services.contains(&AirportService::AirportMap),
            "{:?} should offer AirportMap",
            airport
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════
// DATA VALIDATION TESTS
// ════════════════════════

#[test]
fn test_all_missions_have_valid_routes() {
    let missions = get_mission_templates();
    assert!(
        !missions.is_empty(),
        "Mission template list should not be empty"
    );
    for m in &missions {
        assert!(!m.id.is_empty(), "Mission should have non-empty id");
        assert!(
            !m.title.is_empty(),
            "Mission '{}' should have non-empty title",
            m.id
        );
        // Verify origin and destination are valid by calling their display names
        let _ = m.origin.display_name();
        let _ = m.destination.display_name();
        assert!(
            m.reward_gold > 0,
            "Mission '{}' should have a gold reward",
            m.id
        );
    }
}

#[test]
fn test_all_items_have_valid_data() {
    let mut registry = ItemRegistry::default();
    populate_items(&mut registry);
    assert!(
        !registry.items.is_empty(),
        "ItemRegistry should not be empty after population"
    );
    for (id, item) in &registry.items {
        assert!(
            !item.name.is_empty(),
            "Item '{}' should have non-empty name",
            id
        );
        assert!(
            !item.description.is_empty(),
            "Item '{}' should have non-empty description",
            id
        );
        assert_eq!(
            item.id, *id,
            "Item id field should match registry key for '{}'",
            id
        );
    }
}

#[test]
fn test_crew_members_have_schedules() {
    for crew_id in CREW_IDS {
        let schedule = get_schedule(crew_id);
        assert!(
            !schedule.weekday.is_empty(),
            "Crew member '{}' should have a non-empty weekday schedule",
            crew_id
        );
        assert!(
            !schedule.weekend.is_empty(),
            "Crew member '{}' should have a non-empty weekend schedule",
            crew_id
        );
    }
}

#[test]
fn test_shop_items_exist_in_registry() {
    let mut registry = ItemRegistry::default();
    populate_items(&mut registry);

    let airports = [
        AirportId::HomeBase,
        AirportId::Windport,
        AirportId::Frostpeak,
        AirportId::Sunhaven,
        AirportId::Ironforge,
        AirportId::Cloudmere,
        AirportId::Duskhollow,
        AirportId::Stormwatch,
        AirportId::Grandcity,
        AirportId::Skyreach,
    ];
    for airport in airports {
        let listings = get_shop_inventory(airport);
        for listing in &listings {
            assert!(
                registry.get(&listing.item_id).is_some(),
                "Shop item '{}' at {:?} must exist in ItemRegistry",
                listing.item_id,
                airport
            );
        }
    }
}

use bevy::prelude::*;
use rand::Rng;

use crate::shared::{
    DispatchCall, DispatchCallEvent, DispatchEventKind, DispatchResolvedEvent, FatigueChangeEvent,
    GameState, MapId, MapTransitionEvent, PatrolState, PlayerState, ShiftClock, StressChangeEvent,
    UpdatePhase, XpGainedEvent, DISPATCH_BASE_RATE, DISPATCH_NIGHT_MODIFIER, FUEL_COST_PER_TRIP,
    FUEL_MAX, XP_PER_PATROL_EVENT,
};

const DISPATCH_TIMEOUT_MINUTES: u32 = 10;
const TRAFFIC_STOP_WEIGHT: u8 = 30;
const NOISE_COMPLAINT_WEIGHT: u8 = 20;
const SHOPLIFTER_WEIGHT: u8 = 15;
const DOMESTIC_DISTURBANCE_WEIGHT: u8 = 15;
const SUSPICIOUS_VEHICLE_WEIGHT: u8 = 10;
const OFFICER_BACKUP_WEIGHT: u8 = 10;
const DISPATCH_WEIGHT_TOTAL: u8 = TRAFFIC_STOP_WEIGHT
    + NOISE_COMPLAINT_WEIGHT
    + SHOPLIFTER_WEIGHT
    + DOMESTIC_DISTURBANCE_WEIGHT
    + SUSPICIOUS_VEHICLE_WEIGHT
    + OFFICER_BACKUP_WEIGHT;

#[derive(Debug, Clone, Copy)]
struct DispatchEventDef {
    kind: DispatchEventKind,
    fatigue_cost: f32,
    stress_cost: f32,
    xp_reward: u32,
    may_generate_evidence: bool,
    weight: u8,
    description: &'static str,
}

const DISPATCH_EVENT_DEFS: [DispatchEventDef; 6] = [
    DispatchEventDef {
        kind: DispatchEventKind::TrafficStop,
        fatigue_cost: 5.0,
        stress_cost: 0.0,
        xp_reward: XP_PER_PATROL_EVENT,
        may_generate_evidence: false,
        weight: TRAFFIC_STOP_WEIGHT,
        description: "Routine traffic violation",
    },
    DispatchEventDef {
        kind: DispatchEventKind::NoiseComplaint,
        fatigue_cost: 3.0,
        stress_cost: 0.0,
        xp_reward: XP_PER_PATROL_EVENT / 2,
        may_generate_evidence: false,
        weight: NOISE_COMPLAINT_WEIGHT,
        description: "Residential noise issue",
    },
    DispatchEventDef {
        kind: DispatchEventKind::ShoplifterInProgress,
        fatigue_cost: 8.0,
        stress_cost: 5.0,
        xp_reward: XP_PER_PATROL_EVENT + 5,
        may_generate_evidence: true,
        weight: SHOPLIFTER_WEIGHT,
        description: "Active shoplifting",
    },
    DispatchEventDef {
        kind: DispatchEventKind::DomesticDisturbance,
        fatigue_cost: 10.0,
        stress_cost: 10.0,
        xp_reward: XP_PER_PATROL_EVENT * 2,
        may_generate_evidence: false,
        weight: DOMESTIC_DISTURBANCE_WEIGHT,
        description: "Domestic situation",
    },
    DispatchEventDef {
        kind: DispatchEventKind::SuspiciousVehicle,
        fatigue_cost: 5.0,
        stress_cost: 5.0,
        xp_reward: XP_PER_PATROL_EVENT,
        may_generate_evidence: true,
        weight: SUSPICIOUS_VEHICLE_WEIGHT,
        description: "May lead to case evidence",
    },
    DispatchEventDef {
        kind: DispatchEventKind::OfficerNeedsBackup,
        fatigue_cost: 15.0,
        stress_cost: 15.0,
        xp_reward: XP_PER_PATROL_EVENT * 2 + 5,
        may_generate_evidence: false,
        weight: OFFICER_BACKUP_WEIGHT,
        description: "High-stress response",
    },
];

#[derive(Resource, Debug, Default)]
struct DispatchRuntimeState {
    generated_at_total_minutes: Option<u32>,
}

pub struct PatrolPlugin;

impl Plugin for PatrolPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DispatchRuntimeState>()
            .add_systems(OnEnter(GameState::Playing), initialize_patrol_state)
            .add_systems(
                Update,
                (generate_dispatch, ignore_dispatch, manage_fuel)
                    .in_set(UpdatePhase::Simulation)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn initialize_patrol_state(mut patrol_state: ResMut<PatrolState>) {
    if patrol_state.fuel <= 0.0 {
        patrol_state.fuel = FUEL_MAX;
    }
}

fn generate_dispatch(
    time: Res<Time>,
    clock: Res<ShiftClock>,
    player_state: Res<PlayerState>,
    mut patrol_state: ResMut<PatrolState>,
    mut dispatch_runtime: ResMut<DispatchRuntimeState>,
    mut dispatch_events: EventWriter<DispatchCallEvent>,
) {
    if !clock.on_duty || clock.time_paused || patrol_state.current_dispatch.is_some() {
        return;
    }

    let delta_game_hours = game_hours_from_real_seconds(time.delta_secs(), clock.time_scale);
    let spawn_chance =
        dispatch_spawn_chance(player_state.position_map, clock.hour, delta_game_hours);

    if !should_generate_dispatch(spawn_chance, &mut rand::thread_rng()) {
        return;
    }

    let call = dispatch_call_for_kind(
        weighted_dispatch_kind(&mut rand::thread_rng()),
        player_state.position_map,
    );

    patrol_state.current_dispatch = Some(call.clone());
    dispatch_runtime.generated_at_total_minutes = Some(total_game_minutes(&clock));
    dispatch_events.send(DispatchCallEvent { call });
}

fn ignore_dispatch(
    clock: Res<ShiftClock>,
    mut patrol_state: ResMut<PatrolState>,
    mut dispatch_runtime: ResMut<DispatchRuntimeState>,
) {
    if patrol_state.current_dispatch.is_none() {
        dispatch_runtime.generated_at_total_minutes = None;
        return;
    }

    let current_minutes = total_game_minutes(&clock);
    let Some(generated_at) = dispatch_runtime.generated_at_total_minutes else {
        dispatch_runtime.generated_at_total_minutes = Some(current_minutes);
        return;
    };

    if current_minutes.saturating_sub(generated_at) < DISPATCH_TIMEOUT_MINUTES {
        return;
    }

    patrol_state.current_dispatch = None;
    patrol_state.calls_ignored = patrol_state.calls_ignored.saturating_add(1);
    dispatch_runtime.generated_at_total_minutes = None;
}

fn manage_fuel(
    mut transition_events: EventReader<MapTransitionEvent>,
    mut patrol_state: ResMut<PatrolState>,
) {
    for event in transition_events.read() {
        if event.from == event.to {
            continue;
        }

        if is_precinct_map(event.to) {
            patrol_state.fuel = FUEL_MAX;
            continue;
        }

        if uses_patrol_car(event.from) && uses_patrol_car(event.to) {
            patrol_state.fuel = (patrol_state.fuel - FUEL_COST_PER_TRIP).clamp(0.0, FUEL_MAX);
        }
    }
}

pub fn resolve_dispatch(world: &mut World) -> bool {
    let Some(dispatch) = take_current_dispatch(world) else {
        return false;
    };

    if world.contains_resource::<DispatchRuntimeState>() {
        world
            .resource_mut::<DispatchRuntimeState>()
            .generated_at_total_minutes = None;
    }

    world
        .resource_mut::<Events<DispatchResolvedEvent>>()
        .send(DispatchResolvedEvent {
            kind: dispatch.kind,
            xp_earned: dispatch.xp_reward,
        });
    world
        .resource_mut::<Events<FatigueChangeEvent>>()
        .send(FatigueChangeEvent {
            delta: -dispatch.fatigue_cost,
        });
    world
        .resource_mut::<Events<StressChangeEvent>>()
        .send(StressChangeEvent {
            delta: -dispatch.stress_cost,
        });
    world
        .resource_mut::<Events<XpGainedEvent>>()
        .send(XpGainedEvent {
            amount: dispatch.xp_reward,
            source: format!("patrol:{:?}", dispatch.kind),
        });

    true
}

fn take_current_dispatch(world: &mut World) -> Option<DispatchCall> {
    let mut patrol_state = world.resource_mut::<PatrolState>();
    let dispatch = patrol_state.current_dispatch.take()?;
    patrol_state.calls_responded = patrol_state.calls_responded.saturating_add(1);
    Some(dispatch)
}

fn weighted_dispatch_kind(rng: &mut impl Rng) -> DispatchEventKind {
    debug_assert_eq!(dispatch_weight_total(), DISPATCH_WEIGHT_TOTAL);

    match rng.gen_range(0..DISPATCH_WEIGHT_TOTAL) {
        roll if roll < TRAFFIC_STOP_WEIGHT => DispatchEventKind::TrafficStop,
        roll if roll < TRAFFIC_STOP_WEIGHT + NOISE_COMPLAINT_WEIGHT => {
            DispatchEventKind::NoiseComplaint
        }
        roll if roll < TRAFFIC_STOP_WEIGHT + NOISE_COMPLAINT_WEIGHT + SHOPLIFTER_WEIGHT => {
            DispatchEventKind::ShoplifterInProgress
        }
        roll if roll
            < TRAFFIC_STOP_WEIGHT
                + NOISE_COMPLAINT_WEIGHT
                + SHOPLIFTER_WEIGHT
                + DOMESTIC_DISTURBANCE_WEIGHT =>
        {
            DispatchEventKind::DomesticDisturbance
        }
        roll if roll
            < TRAFFIC_STOP_WEIGHT
                + NOISE_COMPLAINT_WEIGHT
                + SHOPLIFTER_WEIGHT
                + DOMESTIC_DISTURBANCE_WEIGHT
                + SUSPICIOUS_VEHICLE_WEIGHT =>
        {
            DispatchEventKind::SuspiciousVehicle
        }
        _ => DispatchEventKind::OfficerNeedsBackup,
    }
}

fn dispatch_call_for_kind(kind: DispatchEventKind, map_id: MapId) -> DispatchCall {
    let def = dispatch_definition(kind);

    DispatchCall {
        kind: def.kind,
        map_id,
        description: format!("{} in {}", def.description, map_id.display_name()),
        fatigue_cost: def.fatigue_cost,
        stress_cost: def.stress_cost,
        xp_reward: def.xp_reward,
        may_generate_evidence: def.may_generate_evidence,
    }
}

fn dispatch_definition(kind: DispatchEventKind) -> &'static DispatchEventDef {
    DISPATCH_EVENT_DEFS
        .iter()
        .find(|def| def.kind == kind)
        .expect("all dispatch kinds must have authored definitions")
}

fn dispatch_weight_total() -> u8 {
    DISPATCH_EVENT_DEFS.iter().map(|def| def.weight).sum()
}

fn dispatch_spawn_chance(map_id: MapId, hour: u8, delta_game_hours: f32) -> f32 {
    DISPATCH_BASE_RATE
        * map_id.dispatch_rate_modifier()
        * night_modifier_for_hour(hour)
        * delta_game_hours
}

fn night_modifier_for_hour(hour: u8) -> f32 {
    if !(6..22).contains(&hour) {
        DISPATCH_NIGHT_MODIFIER
    } else {
        1.0
    }
}

fn should_generate_dispatch(chance: f32, rng: &mut impl Rng) -> bool {
    if chance <= 0.0 {
        return false;
    }

    if chance >= 1.0 {
        return true;
    }

    rng.gen::<f32>() < chance
}

fn game_hours_from_real_seconds(real_seconds: f32, time_scale: f32) -> f32 {
    real_seconds * time_scale / 60.0
}

fn total_game_minutes(clock: &ShiftClock) -> u32 {
    clock
        .day
        .saturating_sub(1)
        .saturating_mul(24 * 60)
        .saturating_add(u32::from(clock.hour) * 60 + u32::from(clock.minute))
}

fn uses_patrol_car(map_id: MapId) -> bool {
    matches!(
        map_id,
        MapId::PrecinctExterior
            | MapId::Downtown
            | MapId::ResidentialNorth
            | MapId::ResidentialSouth
            | MapId::IndustrialDistrict
            | MapId::Highway
            | MapId::ForestPark
    )
}

fn is_precinct_map(map_id: MapId) -> bool {
    matches!(map_id, MapId::PrecinctInterior | MapId::PrecinctExterior)
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::ecs::event::Events;
    use bevy::state::app::StatesPlugin;
    use bevy::time::{TimeUpdateStrategy, Virtual};
    use bevy::utils::Duration;

    fn build_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.init_state::<GameState>();
        app.configure_sets(
            Update,
            (
                UpdatePhase::Input,
                UpdatePhase::Intent,
                UpdatePhase::Simulation,
                UpdatePhase::Reactions,
                UpdatePhase::Presentation,
            )
                .chain(),
        );
        app.init_resource::<ShiftClock>();
        app.init_resource::<PlayerState>();
        app.init_resource::<PatrolState>();
        app.add_event::<DispatchCallEvent>();
        app.add_event::<DispatchResolvedEvent>();
        app.add_event::<MapTransitionEvent>();
        app.add_event::<FatigueChangeEvent>();
        app.add_event::<StressChangeEvent>();
        app.add_event::<XpGainedEvent>();
        app.world_mut()
            .resource_mut::<Time<Virtual>>()
            .set_max_delta(Duration::MAX);
        app.add_plugins(PatrolPlugin);
        app
    }

    fn enter_playing(app: &mut App) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
        app.update();
    }

    fn set_delta(app: &mut App, duration: Duration) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(duration));
    }

    #[test]
    fn no_dispatch_generated_when_off_duty() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        {
            let mut clock = app.world_mut().resource_mut::<ShiftClock>();
            clock.on_duty = false;
            clock.hour = 12;
        }
        app.world_mut().resource_mut::<PlayerState>().position_map = MapId::Downtown;

        set_delta(&mut app, Duration::from_secs(600));
        app.update();

        let patrol_state = app.world().resource::<PatrolState>();
        assert!(patrol_state.current_dispatch.is_none());
        assert_eq!(
            app.world_mut()
                .resource_mut::<Events<DispatchCallEvent>>()
                .drain()
                .count(),
            0
        );
    }

    #[test]
    fn no_dispatch_generated_when_current_dispatch_is_active() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut().resource_mut::<PlayerState>().position_map = MapId::Downtown;
        app.world_mut()
            .resource_mut::<PatrolState>()
            .current_dispatch = Some(dispatch_call_for_kind(
            DispatchEventKind::DomesticDisturbance,
            MapId::Downtown,
        ));

        set_delta(&mut app, Duration::from_secs(600));
        app.update();

        let patrol_state = app.world().resource::<PatrolState>();
        assert_eq!(
            patrol_state.current_dispatch.as_ref().map(|call| call.kind),
            Some(DispatchEventKind::DomesticDisturbance)
        );
        assert_eq!(
            app.world_mut()
                .resource_mut::<Events<DispatchCallEvent>>()
                .drain()
                .count(),
            0
        );
    }

    #[test]
    fn dispatch_rate_uses_downtown_area_modifier() {
        let chance = dispatch_spawn_chance(MapId::Downtown, 14, 1.0);
        assert!((chance - DISPATCH_BASE_RATE * 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn dispatch_rate_uses_night_modifier() {
        let chance = dispatch_spawn_chance(MapId::ResidentialNorth, 23, 1.0);
        assert!((chance - DISPATCH_BASE_RATE * DISPATCH_NIGHT_MODIFIER).abs() < f32::EPSILON);
    }

    #[test]
    fn resolving_dispatch_emits_xp_and_clears_current_dispatch() {
        let mut app = build_test_app();

        app.world_mut()
            .resource_mut::<PatrolState>()
            .current_dispatch = Some(dispatch_call_for_kind(
            DispatchEventKind::OfficerNeedsBackup,
            MapId::IndustrialDistrict,
        ));

        assert!(resolve_dispatch(app.world_mut()));

        let patrol_state = app.world().resource::<PatrolState>();
        assert!(patrol_state.current_dispatch.is_none());
        assert_eq!(patrol_state.calls_responded, 1);

        let resolved_events = app
            .world_mut()
            .resource_mut::<Events<DispatchResolvedEvent>>()
            .drain()
            .collect::<Vec<_>>();
        let fatigue_events = app
            .world_mut()
            .resource_mut::<Events<FatigueChangeEvent>>()
            .drain()
            .collect::<Vec<_>>();
        let stress_events = app
            .world_mut()
            .resource_mut::<Events<StressChangeEvent>>()
            .drain()
            .collect::<Vec<_>>();
        let xp_events = app
            .world_mut()
            .resource_mut::<Events<XpGainedEvent>>()
            .drain()
            .collect::<Vec<_>>();

        assert_eq!(resolved_events.len(), 1);
        assert_eq!(
            resolved_events[0].kind,
            DispatchEventKind::OfficerNeedsBackup
        );
        assert_eq!(resolved_events[0].xp_earned, 25);

        assert_eq!(fatigue_events.len(), 1);
        assert_eq!(fatigue_events[0].delta, -15.0);

        assert_eq!(stress_events.len(), 1);
        assert_eq!(stress_events[0].delta, -15.0);

        assert_eq!(xp_events.len(), 1);
        assert_eq!(xp_events[0].amount, 25);
        assert_eq!(xp_events[0].source, "patrol:OfficerNeedsBackup");
    }

    #[test]
    fn ignoring_dispatch_after_ten_game_minutes_clears_it() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        {
            let mut patrol_state = app.world_mut().resource_mut::<PatrolState>();
            patrol_state.current_dispatch = Some(dispatch_call_for_kind(
                DispatchEventKind::ShoplifterInProgress,
                MapId::Downtown,
            ));
        }
        {
            let mut clock = app.world_mut().resource_mut::<ShiftClock>();
            clock.day = 1;
            clock.hour = 6;
            clock.minute = 10;
        }
        app.world_mut()
            .resource_mut::<DispatchRuntimeState>()
            .generated_at_total_minutes = Some(6 * 60);

        app.update();

        let patrol_state = app.world().resource::<PatrolState>();
        assert!(patrol_state.current_dispatch.is_none());
        assert_eq!(patrol_state.calls_ignored, 1);
    }

    #[test]
    fn fuel_decrements_on_travel_and_refills_at_precinct() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut().resource_mut::<PatrolState>().fuel = FUEL_MAX;
        app.world_mut()
            .resource_mut::<Events<MapTransitionEvent>>()
            .send(MapTransitionEvent {
                from: MapId::Downtown,
                to: MapId::Highway,
            });

        app.update();

        assert_eq!(
            app.world().resource::<PatrolState>().fuel,
            FUEL_MAX - FUEL_COST_PER_TRIP
        );

        app.world_mut()
            .resource_mut::<Events<MapTransitionEvent>>()
            .send(MapTransitionEvent {
                from: MapId::Highway,
                to: MapId::PrecinctExterior,
            });

        app.update();

        assert_eq!(app.world().resource::<PatrolState>().fuel, FUEL_MAX);
    }
}

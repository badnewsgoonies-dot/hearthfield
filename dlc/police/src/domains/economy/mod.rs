use bevy::prelude::*;

use crate::shared::{
    CaseBoard, CaseFailedEvent, CaseSolvedEvent, Economy, GameState, GoldChangeEvent, PlayerState,
    PromotionEvent, Rank, ShiftClock, ShiftEndEvent, Skills, UpdatePhase, FAILED_CASE_PENALTY,
    MAX_REPUTATION, MIN_REPUTATION, PROMOTION_DETECTIVE_CASES, PROMOTION_DETECTIVE_REP,
    PROMOTION_DETECTIVE_XP, PROMOTION_LIEUTENANT_CASES, PROMOTION_LIEUTENANT_REP,
    PROMOTION_LIEUTENANT_XP, PROMOTION_SERGEANT_CASES, PROMOTION_SERGEANT_REP,
    PROMOTION_SERGEANT_XP,
};

const WEEKLY_RENT: i32 = 100;
const WEEKLY_MAINTENANCE: i32 = 20;
const FAILED_CASE_REPUTATION_HIT: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PromotionThreshold {
    current_rank: Rank,
    next_rank: Rank,
    required_xp: u32,
    required_cases_solved: u32,
    required_reputation: i32,
}

const PROMOTION_THRESHOLDS: [PromotionThreshold; 3] = [
    PromotionThreshold {
        current_rank: Rank::PatrolOfficer,
        next_rank: Rank::Detective,
        required_xp: PROMOTION_DETECTIVE_XP,
        required_cases_solved: PROMOTION_DETECTIVE_CASES,
        required_reputation: PROMOTION_DETECTIVE_REP,
    },
    PromotionThreshold {
        current_rank: Rank::Detective,
        next_rank: Rank::Sergeant,
        required_xp: PROMOTION_SERGEANT_XP,
        required_cases_solved: PROMOTION_SERGEANT_CASES,
        required_reputation: PROMOTION_SERGEANT_REP,
    },
    PromotionThreshold {
        current_rank: Rank::Sergeant,
        next_rank: Rank::Lieutenant,
        required_xp: PROMOTION_LIEUTENANT_XP,
        required_cases_solved: PROMOTION_LIEUTENANT_CASES,
        required_reputation: PROMOTION_LIEUTENANT_REP,
    },
];

#[derive(Event, Debug, Clone)]
struct ReputationChangeEvent {
    delta: i32,
    reason: String,
}

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ReputationChangeEvent>().add_systems(
            Update,
            (
                pay_salary,
                apply_case_rewards,
                apply_case_penalties,
                apply_weekly_expenses,
                process_gold_changes,
                process_reputation_changes,
                check_promotions,
                apply_promotions,
            )
                .chain()
                .in_set(UpdatePhase::Reactions)
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn pay_salary(
    mut shift_end_events: EventReader<ShiftEndEvent>,
    clock: Res<ShiftClock>,
    mut gold_changes: EventWriter<GoldChangeEvent>,
) {
    for _ in shift_end_events.read() {
        gold_changes.send(GoldChangeEvent {
            amount: clock.rank.salary(),
            reason: format!("Shift salary ({})", clock.rank.display_name()),
        });
    }
}

fn apply_case_rewards(
    mut case_solved_events: EventReader<CaseSolvedEvent>,
    mut gold_changes: EventWriter<GoldChangeEvent>,
    mut reputation_changes: EventWriter<ReputationChangeEvent>,
) {
    for event in case_solved_events.read() {
        gold_changes.send(GoldChangeEvent {
            amount: event.gold_reward,
            reason: format!("Case closed: {}", event.case_id),
        });
        reputation_changes.send(ReputationChangeEvent {
            delta: event.reputation_reward,
            reason: format!("Case solved: {}", event.case_id),
        });
    }
}

fn apply_case_penalties(
    mut case_failed_events: EventReader<CaseFailedEvent>,
    mut gold_changes: EventWriter<GoldChangeEvent>,
    mut reputation_changes: EventWriter<ReputationChangeEvent>,
) {
    for event in case_failed_events.read() {
        gold_changes.send(GoldChangeEvent {
            amount: -FAILED_CASE_PENALTY,
            reason: format!("Failed case penalty: {}", event.case_id),
        });
        reputation_changes.send(ReputationChangeEvent {
            delta: -FAILED_CASE_REPUTATION_HIT,
            reason: format!("Failed case: {}", event.case_id),
        });
    }
}

fn process_gold_changes(
    mut gold_change_events: EventReader<GoldChangeEvent>,
    mut player_state: ResMut<PlayerState>,
    mut economy: ResMut<Economy>,
) {
    for event in gold_change_events.read() {
        player_state.gold = player_state.gold.saturating_add(event.amount);

        if event.amount > 0 {
            economy.total_earned = economy.total_earned.saturating_add(event.amount);
        }
    }
}

fn process_reputation_changes(
    mut reputation_change_events: EventReader<ReputationChangeEvent>,
    mut economy: ResMut<Economy>,
) {
    for event in reputation_change_events.read() {
        let _reason = &event.reason;
        economy.reputation = economy
            .reputation
            .saturating_add(event.delta)
            .clamp(MIN_REPUTATION, MAX_REPUTATION);
    }
}

fn check_promotions(
    skills: Res<Skills>,
    case_board: Res<CaseBoard>,
    economy: Res<Economy>,
    clock: Res<ShiftClock>,
    mut promotion_events: EventWriter<PromotionEvent>,
) {
    let Some(threshold) = next_promotion(clock.rank) else {
        return;
    };

    if skills.total_xp < threshold.required_xp {
        return;
    }

    if case_board.total_cases_solved < threshold.required_cases_solved {
        return;
    }

    if economy.reputation < threshold.required_reputation {
        return;
    }

    promotion_events.send(PromotionEvent {
        new_rank: threshold.next_rank,
    });
}

fn apply_promotions(
    mut promotion_events: EventReader<PromotionEvent>,
    mut clock: ResMut<ShiftClock>,
) {
    for event in promotion_events.read() {
        if event.new_rank > clock.rank {
            clock.rank = event.new_rank;
        }
    }
}

fn apply_weekly_expenses(
    clock: Res<ShiftClock>,
    economy: Res<Economy>,
    mut last_seen_day: Local<u32>,
    mut gold_changes: EventWriter<GoldChangeEvent>,
) {
    if *last_seen_day == 0 {
        *last_seen_day = clock.day;
        return;
    }

    if clock.day <= *last_seen_day {
        return;
    }

    for day in (*last_seen_day + 1)..=clock.day {
        if day % 7 == 0 {
            gold_changes.send(GoldChangeEvent {
                amount: -economy.weekly_expenses,
                reason: format!(
                    "Weekly expenses: rent {} + maintenance {}",
                    WEEKLY_RENT, WEEKLY_MAINTENANCE
                ),
            });
        }
    }

    *last_seen_day = clock.day;
}

fn next_promotion(current_rank: Rank) -> Option<PromotionThreshold> {
    PROMOTION_THRESHOLDS
        .iter()
        .copied()
        .find(|threshold| threshold.current_rank == current_rank)
}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::ecs::event::Events;
    use bevy::state::app::StatesPlugin;

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
        app.init_resource::<CaseBoard>();
        app.init_resource::<Economy>();
        app.init_resource::<Skills>();
        app.add_event::<ShiftEndEvent>();
        app.add_event::<CaseSolvedEvent>();
        app.add_event::<CaseFailedEvent>();
        app.add_event::<GoldChangeEvent>();
        app.add_event::<PromotionEvent>();
        app.add_plugins(EconomyPlugin);
        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update();
        app.update();
    }

    #[test]
    fn salary_paid_correctly_per_rank_on_shift_end() {
        for (rank, expected_salary) in [
            (Rank::PatrolOfficer, crate::shared::PATROL_SALARY),
            (Rank::Detective, crate::shared::DETECTIVE_SALARY),
            (Rank::Sergeant, crate::shared::SERGEANT_SALARY),
            (Rank::Lieutenant, crate::shared::LIEUTENANT_SALARY),
        ] {
            let mut app = build_test_app();
            enter_playing(&mut app);

            {
                let mut clock = app.world_mut().resource_mut::<ShiftClock>();
                clock.rank = rank;
            }
            app.world_mut().resource_mut::<PlayerState>().gold = 0;

            app.world_mut().send_event(ShiftEndEvent {
                shift_number: 1,
                cases_progressed: 0,
                evidence_collected: 0,
                xp_earned: 0,
            });

            app.update();

            let player_state = app.world().resource::<PlayerState>();
            let economy = app.world().resource::<Economy>();

            assert_eq!(player_state.gold, expected_salary);
            assert_eq!(economy.total_earned, expected_salary);
        }
    }

    #[test]
    fn case_solve_reward_applies_correct_gold_and_reputation() {
        let mut app = build_test_app();
        enter_playing(&mut app);
        app.world_mut().resource_mut::<PlayerState>().gold = 10;

        app.world_mut().send_event(CaseSolvedEvent {
            case_id: "test_case".to_string(),
            xp_reward: 45,
            gold_reward: 3 * crate::shared::CASE_CLOSE_BONUS_MULTIPLIER,
            reputation_reward: 12,
        });

        app.update();

        let player_state = app.world().resource::<PlayerState>();
        let economy = app.world().resource::<Economy>();

        assert_eq!(player_state.gold, 85);
        assert_eq!(economy.reputation, 12);
        assert_eq!(
            economy.total_earned,
            3 * crate::shared::CASE_CLOSE_BONUS_MULTIPLIER
        );
    }

    #[test]
    fn case_failure_deducts_penalty_gold_and_reputation() {
        let mut app = build_test_app();
        enter_playing(&mut app);
        {
            let mut player_state = app.world_mut().resource_mut::<PlayerState>();
            player_state.gold = 100;
        }
        app.world_mut().resource_mut::<Economy>().reputation = 20;

        app.world_mut().send_event(CaseFailedEvent {
            case_id: "test_case".to_string(),
            reason: "Ran out of leads".to_string(),
        });

        app.update();

        let player_state = app.world().resource::<PlayerState>();
        let economy = app.world().resource::<Economy>();

        assert_eq!(player_state.gold, 50);
        assert_eq!(economy.reputation, 15);
        assert_eq!(economy.total_earned, 0);
    }

    #[test]
    fn promotion_triggers_when_all_thresholds_met() {
        let mut app = build_test_app();
        enter_playing(&mut app);
        {
            let mut skills = app.world_mut().resource_mut::<Skills>();
            skills.total_xp = PROMOTION_DETECTIVE_XP;
        }
        {
            let mut case_board = app.world_mut().resource_mut::<CaseBoard>();
            case_board.total_cases_solved = PROMOTION_DETECTIVE_CASES;
        }
        app.world_mut().resource_mut::<Economy>().reputation = PROMOTION_DETECTIVE_REP;

        app.update();

        let promoted_rank = app.world().resource::<ShiftClock>().rank;
        let promotion_events = app
            .world_mut()
            .resource_mut::<Events<PromotionEvent>>()
            .drain()
            .collect::<Vec<_>>();

        assert_eq!(promoted_rank, Rank::Detective);
        assert_eq!(promotion_events.len(), 1);
        assert_eq!(promotion_events[0].new_rank, Rank::Detective);
    }

    #[test]
    fn promotion_does_not_trigger_when_any_threshold_is_missing() {
        for (xp, cases_solved, reputation) in [
            (
                PROMOTION_DETECTIVE_XP - 1,
                PROMOTION_DETECTIVE_CASES,
                PROMOTION_DETECTIVE_REP,
            ),
            (
                PROMOTION_DETECTIVE_XP,
                PROMOTION_DETECTIVE_CASES - 1,
                PROMOTION_DETECTIVE_REP,
            ),
            (
                PROMOTION_DETECTIVE_XP,
                PROMOTION_DETECTIVE_CASES,
                PROMOTION_DETECTIVE_REP - 1,
            ),
        ] {
            let mut app = build_test_app();
            enter_playing(&mut app);
            app.world_mut().resource_mut::<Skills>().total_xp = xp;
            app.world_mut()
                .resource_mut::<CaseBoard>()
                .total_cases_solved = cases_solved;
            app.world_mut().resource_mut::<Economy>().reputation = reputation;

            app.update();

            let rank = app.world().resource::<ShiftClock>().rank;
            let promotion_events = app
                .world_mut()
                .resource_mut::<Events<PromotionEvent>>()
                .drain()
                .collect::<Vec<_>>();

            assert_eq!(rank, Rank::PatrolOfficer);
            assert!(promotion_events.is_empty());
        }
    }

    #[test]
    fn reputation_clamps_to_contract_bounds() {
        let mut app = build_test_app();
        enter_playing(&mut app);

        app.world_mut().resource_mut::<Economy>().reputation = MAX_REPUTATION - 1;
        app.world_mut().send_event(ReputationChangeEvent {
            delta: 10,
            reason: "Positive press".to_string(),
        });
        app.update();
        assert_eq!(app.world().resource::<Economy>().reputation, MAX_REPUTATION);

        app.world_mut().resource_mut::<Economy>().reputation = MIN_REPUTATION + 1;
        app.world_mut().send_event(ReputationChangeEvent {
            delta: -10,
            reason: "Public scandal".to_string(),
        });
        app.update();
        assert_eq!(app.world().resource::<Economy>().reputation, MIN_REPUTATION);
    }

    #[test]
    fn weekly_expenses_are_deducted_on_day_seven() {
        let mut app = build_test_app();
        enter_playing(&mut app);
        app.world_mut().resource_mut::<PlayerState>().gold = 500;

        {
            let mut clock = app.world_mut().resource_mut::<ShiftClock>();
            clock.day = 6;
        }
        app.update();

        {
            let mut clock = app.world_mut().resource_mut::<ShiftClock>();
            clock.day = 7;
        }
        app.update();

        let player_state = app.world().resource::<PlayerState>();
        let economy = app.world().resource::<Economy>();

        assert_eq!(player_state.gold, 500 - economy.weekly_expenses);
        assert_eq!(economy.weekly_expenses, WEEKLY_RENT + WEEKLY_MAINTENANCE);
    }
}

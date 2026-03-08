# Economy Domain Spec — Precinct

## Purpose
Salary, reputation, department budget, and promotion system. The career progression layer.

## Scope
`src/domains/economy/` — owns salary payments, reputation tracking, promotions, expenses.

## What This Domain Does
- Pay salary on ShiftEndEvent: rank.salary() → GoldChangeEvent → PlayerState.gold
- Apply case rewards: CaseSolvedEvent → gold + reputation to Economy
- Apply case penalties: CaseFailedEvent → -FAILED_CASE_PENALTY gold, reputation hit
- Track weekly expenses: rent (100/week) + maintenance (20/week) deducted on day 7 transitions
- Manage Economy.reputation: clamp to MIN_REPUTATION..MAX_REPUTATION
- Check promotion eligibility: XP + cases_solved + reputation thresholds
- Emit PromotionEvent when player meets requirements and is promoted
- Manage department_budget for equipment purchases (future)

## What This Domain Does NOT Do
- XP calculation (skills domain)
- Case state management (cases domain)
- UI for career screen (ui domain, future)
- Rank display (calendar domain manages ShiftClock.rank, economy triggers promotion)

## Key Types (import from crate::shared)
- `Economy` (Resource), `PlayerState` (Resource), `ShiftClock` (Resource)
- `CaseBoard` (Resource — read total_cases_solved)
- `Skills` (Resource — read total_xp)
- `ShiftEndEvent`, `CaseSolvedEvent`, `CaseFailedEvent`
- `GoldChangeEvent`, `PromotionEvent`
- `Rank`, `GameState`, `UpdatePhase`
- Constants: all PATROL/DETECTIVE/SERGEANT/LIEUTENANT_SALARY, CASE_CLOSE_BONUS_MULTIPLIER, FAILED_CASE_PENALTY, all PROMOTION_* constants, MAX/MIN_REPUTATION

## Systems to Implement
1. `pay_salary` — read ShiftEndEvent, emit GoldChangeEvent with rank.salary()
2. `apply_case_rewards` — read CaseSolvedEvent → emit GoldChangeEvent(reward_gold), add reputation
3. `apply_case_penalties` — read CaseFailedEvent → emit GoldChangeEvent(-FAILED_CASE_PENALTY), subtract reputation
4. `process_gold_changes` — read GoldChangeEvent, apply to PlayerState.gold, track in Economy.total_earned
5. `check_promotions` — UpdatePhase::Reactions, check if Skills.total_xp >= threshold AND CaseBoard.total_cases_solved >= threshold AND Economy.reputation >= threshold → emit PromotionEvent, update ShiftClock.rank
6. `apply_weekly_expenses` — on day transitions (day % 7 == 0), deduct Economy.weekly_expenses from gold

## Quantitative Targets
- Salary: Patrol=80, Detective=120, Sergeant=160, Lieutenant=200 per shift
- Case close bonus: difficulty * 25
- Failed case penalty: -50 gold
- Promotions: Detective(200xp/3cases/10rep), Sergeant(500/8/25), Lieutenant(1000/16/50)
- Weekly expenses: 120 (rent 100 + maintenance 20)
- Reputation: clamped -100 to +100

## Decision Fields
- **Preferred**: salary + reputation + budget as three economic axes
- **Tempting alternative**: just gold like Hearthfield
- **Consequence**: gold-only loses the "public servant under scrutiny" feel
- **Drift cue**: worker drops reputation or budget system

- **Preferred**: all gold changes routed through GoldChangeEvent (no direct mutation)
- **Tempting alternative**: directly modify PlayerState.gold
- **Consequence**: bypasses tracking, other systems can't react to gold changes
- **Drift cue**: worker writes `player_state.gold += amount` without emitting event
  (THIS IS THE SHOP BYPASS ANTI-PATTERN FROM HEARTHFIELD — DO NOT REPEAT)

## Plugin Export
```rust
pub struct EconomyPlugin;
```

## Tests (minimum 5)
1. Salary paid correctly per rank on ShiftEndEvent
2. Case solve reward applies correct gold and reputation
3. Case failure deducts penalty gold and reputation
4. Promotion triggers when all 3 thresholds met
5. Promotion does NOT trigger when any threshold not met
6. Reputation clamps to -100..100
7. Weekly expenses deducted on day 7

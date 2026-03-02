# Worker: Unwrap Safety — Replace Panicking Unwraps

## Scope (hard allowlist — enforced mechanically)
You may ONLY modify these files:
- src/calendar/festivals.rs
- src/crafting/buffs.rs
- src/ui/building_upgrade_menu.rs
- src/ui/cutscene_runner.rs
- src/ui/toast.rs
All other file edits will be reverted after you finish.

## Required Reading
1. docs/domains/unwrap_safety.md — full problem description with exact line numbers
2. The 5 source files listed above

## Task
Replace 8 dangerous .unwrap() calls with graceful error handling.
Do NOT touch unwraps inside #[test] functions or unwraps guarded by is_none() checks.

## Exact Fixes Required

### 1. src/calendar/festivals.rs (lines ~206-207)
BEFORE:
```rust
festival.timer.as_mut().unwrap().tick(time.delta());
let timer_finished = festival.timer.as_ref().unwrap().just_finished();
```
FIX: Wrap in `if let Some(timer) = ... { }` guard. If timer is None, skip ticking and treat as not finished.

### 2. src/crafting/buffs.rs (line ~270)
BEFORE: `let base = original_max_stamina.unwrap();`
FIX: `let base = original_max_stamina.unwrap_or(100.0);` — 100.0 is the default max stamina.

### 3. src/ui/building_upgrade_menu.rs (line ~389)
BEFORE: `let to_tier = entry.to_tier.unwrap();`
FIX: `let Some(to_tier) = entry.to_tier else { continue; };` (or `return;` depending on context)

### 4. src/ui/cutscene_runner.rs (line ~106)
BEFORE: `let step = queue.steps.front().unwrap().clone();`
FIX: `let Some(step) = queue.steps.front().cloned() else { return; };`

### 5. src/ui/toast.rs (lines ~148, 158-159)
BEFORE: Three separate .unwrap() on fade_timer
FIX: Wrap the entire fade logic in `if let Some(ft) = toast.fade_timer.as_mut() { ... }`

## Quantitative Target
- 8 dangerous unwraps eliminated
- 0 new unwraps introduced
- All replacements use graceful fallback (if-let, unwrap_or, else-return)

## Validation
```bash
# Count remaining dangerous unwraps (should be 0 in these 5 files)
grep -n "\.unwrap()" src/calendar/festivals.rs src/crafting/buffs.rs src/ui/building_upgrade_menu.rs src/ui/cutscene_runner.rs src/ui/toast.rs | grep -v "test\|#\[cfg(test)\]"
```
Expected: no output

## When Done
Write completion report to status/workers/unwrap_safety.md containing:
- Each unwrap replaced and what it was replaced with
- Validation result
- Any assumptions made

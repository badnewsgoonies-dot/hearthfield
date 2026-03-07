# Unwrap Safety Completion Report

## Replacements Completed
1. `src/calendar/festivals.rs` (2 unwraps)
- Replaced:
  - `festival.timer.as_mut().unwrap().tick(time.delta());`
  - `let timer_finished = festival.timer.as_ref().unwrap().just_finished();`
- With:
  - `let timer_finished = if let Some(timer) = festival.timer.as_mut() { ... } else { false };`
- Behavior: timer is ticked only when present; missing timer is treated as "not finished".

2. `src/crafting/buffs.rs` (1 unwrap)
- Replaced:
  - `let base = original_max_stamina.unwrap();`
- With:
  - `let base = original_max_stamina.unwrap_or(100.0);`
- Behavior: defaults to canonical max stamina when baseline is missing.

3. `src/ui/building_upgrade_menu.rs` (1 unwrap)
- Replaced:
  - `let to_tier = entry.to_tier.unwrap();`
- With:
  - `let Some(to_tier) = entry.to_tier else { return };`
- Behavior: exits system early if tier is unexpectedly absent, avoiding panic.

4. `src/ui/cutscene_runner.rs` (1 unwrap)
- Replaced:
  - `let step = queue.steps.front().unwrap().clone();`
- With:
  - `let Some(step) = queue.steps.front().cloned() else { return; };`
- Behavior: returns safely if the queue front is unexpectedly empty.

5. `src/ui/toast.rs` (3 unwraps)
- Replaced:
  - `let ft = toast.fade_timer.as_mut().unwrap();`
  - `let elapsed = toast.fade_timer.as_ref().unwrap().elapsed_secs();`
  - `let duration = toast.fade_timer.as_ref().unwrap().duration().as_secs_f32();`
- With:
  - Fade block guarded by `else if let Some(ft) = toast.fade_timer.as_mut() { ... }`
  - Elapsed/duration read from `ft` inside that guard.
- Behavior: fade logic runs only when fade timer exists.

## Validation Result
Executed exactly:
```bash
grep -n "\.unwrap()" src/calendar/festivals.rs src/crafting/buffs.rs src/ui/building_upgrade_menu.rs src/ui/cutscene_runner.rs src/ui/toast.rs | grep -v "test\|#\[cfg(test)\]"
```
Result: no output (no remaining `.unwrap()` matches in the 5 scoped files).

## Assumptions
- In `src/ui/building_upgrade_menu.rs`, `else { return }` was used (instead of `continue`) because the unwrap site is not inside a loop.
- `festival.timer` in egg collection may be absent despite prior checks, so fallback `timer_finished = false` is the intended graceful behavior.

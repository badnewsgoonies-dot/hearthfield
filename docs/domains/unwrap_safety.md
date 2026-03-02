# Domain: Unwrap Safety — Replace Panicking Unwraps

## Problem
8 production unwrap() calls can panic at runtime. Replace with graceful handling.

## Dangerous Unwraps (MUST FIX)

### 1. src/calendar/festivals.rs lines 206-207
```rust
festival.timer.as_mut().unwrap().tick(time.delta());
let timer_finished = festival.timer.as_ref().unwrap().just_finished();
```
Fix: Guard with `if let Some(timer) = festival.timer.as_mut() { ... }`

### 2. src/crafting/buffs.rs line 270
```rust
let base = original_max_stamina.unwrap();
```
Fix: `let base = original_max_stamina.unwrap_or(100.0);` or guard with if-let

### 3. src/ui/building_upgrade_menu.rs line 389
```rust
let to_tier = entry.to_tier.unwrap();
```
Fix: `let Some(to_tier) = entry.to_tier else { continue; };` or `return;`

### 4. src/ui/cutscene_runner.rs line 106
```rust
let step = queue.steps.front().unwrap().clone();
```
Fix: `let Some(step) = queue.steps.front().cloned() else { return; };`

### 5. src/ui/toast.rs lines 148, 158-159
```rust
let ft = toast.fade_timer.as_mut().unwrap();
let elapsed = toast.fade_timer.as_ref().unwrap().elapsed_secs();
let duration = toast.fade_timer.as_ref().unwrap().duration().as_secs_f32();
```
Fix: Guard the entire fade block with `if let Some(ft) = toast.fade_timer.as_mut() { ... }`

## Safe Unwraps (DO NOT TOUCH)
- src/npcs/schedules.rs lines 689-718: Inside #[test] functions
- src/shared/mod.rs lines 1998-2182: Inside #[test] functions  
- src/player/interact_dispatch.rs:48, src/player/item_use.rs:146, src/ui/hud.rs:911,919: 
  Guarded by is_none() check — safe but could be refactored to use Option methods

## Validation
After fixes: `grep -rn "\.unwrap()" src/ --include="*.rs" | grep -v "test\|#\[cfg(test)\]"` 
should show only the safe-pattern unwraps (is_none guard) or zero results.

## Does NOT Handle
- Adding new error types or Result propagation
- Test code unwraps
- Logic changes beyond replacing unwrap with graceful fallback

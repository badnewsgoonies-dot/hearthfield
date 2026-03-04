# Worker C: Fishing + Mining Feedback (Toasts)

## Scope: src/fishing/ AND src/mining/ ONLY. All other edits will be reverted.

## Read first (DO NOT MODIFY):
- src/shared/mod.rs — find ToastEvent, PlaySfxEvent

## Fishing Tasks

### Task 1: Can't fish here (fishing/cast.rs)
If the player tries to cast but is not near water or in a valid fishing spot, send:
```rust
toast_events.send(ToastEvent { message: "Can't fish here.".into(), duration_secs: 2.0 });
```
Look for the early-return conditions in the cast system. Add toast before return.

### Task 2: Fish escaped (fishing/cast.rs)
When the reaction window expires (fish bites but player doesn't press in time), send:
```rust
toast_events.send(ToastEvent { message: "The fish got away!".into(), duration_secs: 2.0 });
```

### Task 3: Successful catch (fishing/cast.rs or minigame.rs)
Check if there's already a toast for catching a fish. If not, add:
```rust
toast_events.send(ToastEvent { message: format!("Caught a {}!", fish_name), duration_secs: 2.5 });
```

## Mining Tasks

### Task 4: Player takes damage (mining/combat.rs)
When an enemy hits the player, send a brief damage toast:
```rust
toast_events.send(ToastEvent { message: format!("-{} HP", damage), duration_secs: 1.5 });
```

### Task 5: Floor transition (mining/ladder.rs)
When player moves to a new mine floor, send:
```rust
toast_events.send(ToastEvent { message: format!("Floor {}", floor_number), duration_secs: 1.5 });
```

## Validation: cargo check must pass.
## When done: write report to status/workers/feedback-fishing-mining.md

# Worker B: Farming Feedback (Toasts)

## Scope: src/farming/ ONLY. All other edits will be reverted.

## Read first (DO NOT MODIFY):
- src/shared/mod.rs — find ToastEvent, PlaySfxEvent
- src/farming/soil.rs — the handle_till and handle_water systems
- src/farming/harvest.rs — the harvest system
- src/farming/events_handler.rs — season change crop death logic

## Task 1: Already tilled (soil.rs)
In handle_till, if the player hoes a tile that is already tilled soil, send:
```rust
toast_events.send(ToastEvent { message: "Already tilled!".into(), duration_secs: 1.5 });
```
Add EventWriter<ToastEvent> param if not present. Only send this when the tool action targets
an already-tilled tile — do NOT block the normal tilling path.

## Task 2: Already watered (soil.rs)
In handle_water, if the tile is already watered today, send:
```rust
toast_events.send(ToastEvent { message: "Already watered today.".into(), duration_secs: 1.5 });
```

## Task 3: Harvest success (harvest.rs)
After a successful harvest (crop removed, item added), send:
```rust
toast_events.send(ToastEvent { message: format!("Harvested {}!", crop_name), duration_secs: 2.0 });
```
Get crop_name from the CropRegistry or the harvest event data.

## Task 4: Crop withered (events_handler.rs)
When a crop dies due to season change, send:
```rust
toast_events.send(ToastEvent { message: "Some crops have withered...".into(), duration_secs: 3.0 });
```
Send this ONCE per season change (not per crop). Use a counter or flag.

## Validation: cargo check must pass.
## When done: write report to status/workers/feedback-farming.md

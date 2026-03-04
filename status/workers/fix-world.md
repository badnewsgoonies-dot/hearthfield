# Worker Report: FIX-WORLD

## Files Modified
- `src/world/mod.rs` (1 line removed, net -1 LOC)

## What Was Changed and Why
Removed the duplicate `app.add_event::<ToastEvent>()` call from `WorldPlugin::build()` (formerly line 56). The `ToastEvent` event is already registered in `src/main.rs:118`, so the registration in the world plugin was redundant. While Bevy 0.15 handles duplicate event registration idempotently, the duplication was a code smell and potential source of confusion.

### Before (line 56):
```rust
app.add_event::<ToastEvent>()
    .init_resource::<WorldMap>()
```

### After (line 56):
```rust
app.init_resource::<WorldMap>()
```

The method chain remains syntactically correct. `ToastEvent` is still used in `src/world/objects.rs` and `src/world/chests.rs` for *sending* events via `EventWriter<ToastEvent>`, which does not require re-registration.

## Validation Results
- `cargo check`: PASS (no errors in `src/world/`; pre-existing errors in `src/crafting/cooking.rs` are out of scope)
- `cargo clippy -- -D warnings`: PASS (no warnings in `src/world/`)

## Shared Type Imports Used
- `ToastEvent` (from `crate::shared::*`) -- still imported for use in EventWriter, no longer registered as event

## Known Risks for Integration
- None. The event was already registered in `main.rs`. Removing the duplicate has no behavioral effect.

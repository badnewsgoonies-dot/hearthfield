# Worker Report: Stamina Feedback (Toast + SFX)

## Files Modified

- `src/player/tools.rs` — added `ToastEvent` writer param + 3 new toasts + `stamina_low_warning` system (~30 lines added)
- `src/player/mod.rs` — registered `tools::stamina_low_warning` in Playing-state systems

## What Was Implemented

### Task 1: "Too tired!" toast on blocked tool use
Added `mut toast_events: EventWriter<ToastEvent>` to `tool_use` system params. When stamina check fails, sends:
> "Too tired to use that tool. Rest or eat something!" (2.5s)

### Task 2: Proactive stamina low warning
New `stamina_low_warning` system added to `src/player/tools.rs`. Fires a toast when stamina drops to ≤25% of max_stamina, uses a `Local<bool>` flag to prevent spamming. Resets flag when stamina recovers above threshold. Registered in `PlayerPlugin` alongside existing Playing-state systems.
> "You're getting tired... consider resting or eating." (3.0s)

### Task 3: "Tool is being upgraded" toast
In the `upgrade_queue.is_upgrading(tool)` branch, added toast:
> "That tool is being upgraded at the blacksmith." (2.5s)

## Validation

```
cargo check → Finished `dev` profile in 7.90s — zero errors, zero warnings
```

## Known Risks

- None. All changes are purely additive event sends; no game logic, stamina values, or existing behavior was modified.

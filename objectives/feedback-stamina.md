# Worker: Add Stamina Feedback (Toast + SFX)

## Context
When a player tries to use a tool with insufficient stamina, the game plays an "error" SFX but gives NO text feedback — the player doesn't know WHY nothing happened. Additionally, there's no proactive warning when stamina gets low.

## Scope (mechanically enforced)
You may ONLY modify files under: `src/player/`
All out-of-scope edits will be reverted.

## Required reading
1. src/player/tools.rs — the `tool_use` function (especially the stamina check around line 85) and `stamina_drain_handler`
2. src/shared/mod.rs — READ ONLY. Find `ToastEvent`, `PlaySfxEvent`, `PlayerState` (stamina field), `StaminaDrainEvent`

## Task 1: Add "Too tired!" toast when tool use blocked by stamina

In `tool_use` (src/player/tools.rs), find the stamina check:
```rust
if player_state.stamina < cost {
    sfx_events.send(PlaySfxEvent {
        sfx_id: "error".to_string(),
    });
    return;
}
```

Add a `toast_events: EventWriter<ToastEvent>` parameter to the system, then add a toast alongside the existing SFX:
```rust
if player_state.stamina < cost {
    sfx_events.send(PlaySfxEvent {
        sfx_id: "error".to_string(),
    });
    toast_events.send(ToastEvent {
        message: "Too tired to use that tool. Rest or eat something!".into(),
        duration_secs: 2.5,
    });
    return;
}
```

## Task 2: Add proactive stamina low warning

Add a new system `stamina_low_warning` in src/player/tools.rs (or a new file src/player/stamina_warning.rs):

```rust
/// Warn the player when stamina drops below 25% of max.
/// Uses a local flag to avoid spamming every frame.
pub fn stamina_low_warning(
    player_state: Res<PlayerState>,
    mut toast_events: EventWriter<ToastEvent>,
    mut warned: Local<bool>,
) {
    let threshold = player_state.max_stamina * 0.25;
    
    if player_state.stamina <= threshold && !*warned {
        *warned = true;
        toast_events.send(ToastEvent {
            message: "You're getting tired... consider resting or eating.".into(),
            duration_secs: 3.0,
        });
    }
    
    // Reset warning when stamina recovers above threshold
    if player_state.stamina > threshold {
        *warned = false;
    }
}
```

Register this system in `src/player/mod.rs` — add it to the Playing-state systems.

## Task 3: Add "Tool is being upgraded" toast

In `tool_use`, find the upgrade check:
```rust
if upgrade_queue.is_upgrading(tool) {
    sfx_events.send(PlaySfxEvent {
        sfx_id: "error".to_string(),
    });
    return;
}
```

Add a toast:
```rust
if upgrade_queue.is_upgrading(tool) {
    sfx_events.send(PlaySfxEvent {
        sfx_id: "error".to_string(),
    });
    toast_events.send(ToastEvent {
        message: "That tool is being upgraded at the blacksmith.".into(),
        duration_secs: 2.5,
    });
    return;
}
```

## Do NOT
- Modify src/shared/mod.rs
- Change stamina values or game balance
- Modify any files outside src/player/

## Validation
```
cargo check
```
Must pass with zero errors.

## When done
Write completion report to status/workers/feedback-stamina.md

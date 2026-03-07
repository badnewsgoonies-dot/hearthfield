# Worker: Contract Amendment — Fix R Key Collision

## Context
`src/shared/mod.rs` has a keybinding collision: both `tool_secondary` (gift/eat/place) and `open_relationships` are bound to `KeyCode::KeyR`. This causes the relationships screen to open when the player tries to gift an NPC.

## Scope
You may ONLY modify: `src/shared/mod.rs`
This is a contract amendment — minimal, surgical change only.

## Required reading
1. src/shared/mod.rs — find the `KeyBindings` struct and its `Default` impl

## Task (exactly 2 changes)

### Change 1: Fix the default binding
In the `Default` impl for `KeyBindings`, change:
```rust
open_relationships: KeyCode::KeyR,
```
to:
```rust
open_relationships: KeyCode::KeyL,
```

### Change 2: Fix the comment
In the `PlayerInput` struct, the field comment says `// R — relationships screen`. Change it to:
```rust
pub open_relationships: bool, // L — relationships screen
```

## Do NOT
- Change any other field or default
- Add new fields
- Modify any other file
- Refactor or reorganize anything

## Validation
```
cargo check
```
Must pass with zero errors.

## When done
Confirm the two lines changed and that cargo check passes.

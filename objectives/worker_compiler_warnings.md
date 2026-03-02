# Worker: Fix Compiler Warnings

## Scope (hard allowlist — enforced mechanically)
You may ONLY modify:
- src/player/interaction.rs
- src/ui/dialogue_box.rs
All other file edits will be reverted.

## Task
Fix 2 unused import warnings that cargo check reports.

### Fix 1: src/player/interaction.rs line 3
Remove the unused import of `TransitionZone`:
```rust
// BEFORE:
use crate::world::TransitionZone;
// AFTER: remove this line entirely
```

### Fix 2: src/ui/dialogue_box.rs line 4
Remove `npc_color` from the import (keep `npc_sprite_file`):
```rust
// BEFORE:
use crate::npcs::definitions::{npc_color, npc_sprite_file};
// AFTER:
use crate::npcs::definitions::npc_sprite_file;
```

## Validation
```bash
grep "TransitionZone" src/player/interaction.rs  # should find 0 matches
grep "npc_color" src/ui/dialogue_box.rs  # should find 0 matches
```

## When Done
Write completion report to status/workers/compiler_warnings.md

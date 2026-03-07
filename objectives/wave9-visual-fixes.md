# Worker: Visual Issue Fixes (H6 + M2)

## Scope (mechanically enforced)
You may only modify files under: src/npcs/ and src/world/
Out-of-scope edits will be silently reverted after you finish.

## Required reading
1. status/workers/visual_audit_report.md — the full audit
2. src/shared/mod.rs — EmoteKind, Emote types (import only)
3. src/npcs/ — emote system
4. src/world/ — path/fence placement

## Task: Fix 2 remaining visual audit issues

### H6: Emote atlas indices are guesses
The emote system uses sprite atlas indices that may not match the actual atlas layout.

**Fix approach**: Since we use procedural sprites (no real atlas), ensure emote rendering uses correct visual representations:
- Heart emote: red heart shape or colored sprite
- Exclamation: appropriate marker
- Question mark: appropriate marker
- If emotes use TextureAtlas indices, verify they point to valid frames or switch to procedural colored sprites (like we did for animals)
- Check `src/npcs/` for emote spawning and rendering code

### M2: Path/fence autotile bitmask assumption unverified
The path and fence placement system may use a bitmask for autotiling (connecting adjacent tiles visually).

**Fix approach**:
- Check `src/world/` for path/fence sprite selection logic
- If bitmask indexing assumes a specific atlas layout, verify the indices are correct
- If procedural sprites are used, ensure adjacent paths/fences connect visually (use matching colored rectangles)
- At minimum, ensure paths and fences render without visual glitches

## Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three pass with zero errors and zero warnings.

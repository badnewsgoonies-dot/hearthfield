# Worker: INPUT

## Scope (mechanically enforced — your edits outside this path will be reverted)
You may only modify files under: src/input/
Out-of-scope edits will be silently reverted after you finish.

## Required reading (read these files from disk before writing any code)
1. GAME_SPEC.md
2. docs/domains/input.md
3. src/shared/mod.rs (the type contract — import from here, do not redefine)

## Required imports (use exactly, do not redefine locally)
- `PlayerInput`, `InputContext`, `KeyBindings`
- `InputBlocks`, `InteractionClaimed`
- `MenuAction`
- `GameState`

## Deliverables
- `src/input/mod.rs` — `InputPlugin` with input reading, context routing, and frame reset

## Quantitative targets (non-negotiable)
- 17 key bindings mapped
- 5 input contexts handled (Gameplay, Menu, Dialogue, Fishing, Cutscene)
- Frame-accurate just_pressed detection
- Input reset every frame
- InteractionClaimed reset every frame

## Validation (run before reporting done)
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three commands pass with zero errors and zero warnings.

## When done
Write completion report to status/workers/input.md containing:
- Files created/modified (with line counts)
- What was implemented
- Quantitative targets hit (with actual counts)
- Shared type imports used
- Validation results (pass/fail)
- Known risks for integration

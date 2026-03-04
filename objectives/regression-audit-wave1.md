# Worker: Regression Audit — Verify All 5 Audit Fixes

## Context
We just applied 5 fixes from a deep research audit. You are the regression checker. Your job is to verify each fix landed correctly and look for any new issues introduced.

## Scope
READ-ONLY investigation. Do NOT modify any files. Just read and report.

## Checklist — Verify each fix

### 1. R Key Collision (src/shared/mod.rs)
- [ ] `open_relationships` default is `KeyCode::KeyL` (NOT KeyR)
- [ ] `tool_secondary` default is still `KeyCode::KeyR`
- [ ] Comment on `open_relationships` field says `// L —` not `// R —`
- [ ] No other field uses KeyL (no new collision)

### 2. Animal Atlas (src/animals/mod.rs)
- [ ] `TextureAtlasLayout::from_grid` uses `UVec2::new(48, 48), 4, 4` 
- [ ] Comment says `192x192, 4 cols x 4 rows of 48x48`
- [ ] No sprite index in src/animals/spawning.rs exceeds 15 (max valid index for 4x4=16 frames)

### 3. Hotbar Icon Sizing (src/ui/hud.rs)
- [ ] HotbarItemIcon Node uses `Val::Px(32.0)` width and height (not 28.0)

### 4. Bed Sleep (src/player/interact_dispatch.rs)
- [ ] `InteractionKind::Bed` arm: `interaction_claimed.0 = true` is INSIDE the `if calendar.hour < 18` block
- [ ] There is NO DayEndEvent send in the Bed handler
- [ ] There is a comment explaining that trigger_sleep handles the cutscene when hour >= 18

### 5. Dynamic Key Prompts (src/ui/hud.rs)
- [ ] `update_interaction_prompt` system has `bindings: Res<KeyBindings>` parameter
- [ ] A `key_display` function exists that maps KeyCode to short strings
- [ ] No hardcoded `[F]` or `[R]` strings remain in prompt formatting
- [ ] Prompt format uses `key_display(bindings.interact)` and `key_display(bindings.tool_secondary)`

## Also check for NEW issues
- Are there any unused imports after the changes?
- Any new compiler warnings likely?
- Does the `key_display` function handle all keys used in default KeyBindings?
- Is there any other place in src/ui/ that hardcodes key strings (grep for `"[F]"` or `"[R]"` or `"[E]"` patterns)?

## Validation
Run:
```
cargo check
cargo clippy -- -D warnings
```
Report results.

## Output
Write a full audit report to status/workers/regression-audit-wave1.md with:
- Each checklist item: PASS or FAIL with evidence
- Any new issues found
- Gate results (cargo check, clippy)
- Overall verdict: SHIP or BLOCK

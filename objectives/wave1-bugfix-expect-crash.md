# Worker: Wave 1-C — Fix expect() Crash Risk

## Objective
Replace a potential runtime crash with graceful error handling.

## Bug: expect() crash risk in main menu
**Location:** `src/ui/main_menu.rs` line 439

The code uses `expect("path should resolve")` which will crash the game if DLC path resolution fails. This is unacceptable for a shipped game.

**Fix required:**
- Replace `expect()` with proper error handling
- Use `unwrap_or_default()`, `unwrap_or_else()`, or a match statement
- If the path fails to resolve, either:
  - Skip the DLC content gracefully, OR
  - Log a warning and continue with a fallback

**Example fix pattern:**
```rust
// Before:
let path = something.expect("path should resolve");

// After:
let path = match something {
    Some(p) => p,
    None => {
        warn!("DLC path resolution failed, skipping");
        return; // or continue with default
    }
};
```

## Secondary task: Audit for other expect/unwrap in UI
While you're in the UI code, briefly check for other `expect()` or `unwrap()` calls in `src/ui/` that could crash at runtime (not in test code). If you find any that are clearly risky, fix them too. Don't spend more than 5-10 minutes on this audit.

## Files to modify
1. `src/ui/main_menu.rs` — Fix the expect() on line 439 (primary)
2. Other `src/ui/*.rs` files only if you find additional crash risks

## Validation
Run these commands before reporting done:
```bash
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```

All must pass with zero errors and zero warnings.

## Do NOT modify
- Test code (expect/unwrap is fine in tests)
- Non-UI code
- Logic unrelated to error handling

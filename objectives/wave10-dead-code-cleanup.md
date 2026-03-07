# Worker: Dead Code Cleanup

## Scope (mechanically enforced)
You may modify any files under src/ EXCEPT src/shared/mod.rs (frozen contract).
Out-of-scope edits to src/shared/mod.rs will be silently reverted.

## Required reading
1. src/shared/mod.rs — the type contract (DO NOT MODIFY)
2. Run `cargo check 2>&1` to see current state

## Task
Remove unnecessary `#[allow(dead_code)]` attributes across the codebase:

1. Run `cargo check 2>&1` to identify any actual dead code warnings
2. For each `#[allow(dead_code)]` in src/:
   - Check if the item is actually used somewhere
   - If used: remove the allow attribute (it's unnecessary)
   - If NOT used and NOT part of a public API: remove both the allow and the dead code
   - If NOT used but part of a public API or shared contract: leave as-is
3. Do NOT modify src/shared/mod.rs
4. Do NOT remove `#[allow(dead_code)]` from test modules

## Validation
```
cargo check
cargo test --test headless
cargo clippy -- -D warnings
```
Done = all three pass with zero errors and zero warnings.

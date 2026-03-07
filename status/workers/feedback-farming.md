# Worker Report: Farming Feedback Toasts

## Status: COMPLETE (already implemented)

All four toast feedback features were already present in the codebase. No changes were needed.

## Verification

- `cargo check` passes with zero errors.

## Feature Locations

| Feature | File | Lines | Toast Message |
|---------|------|-------|---------------|
| Already tilled | src/farming/soil.rs | 30-34 | "Already tilled!" (1.5s) |
| Already watered | src/farming/soil.rs | 103-106 | "Already watered today." (1.5s) |
| Harvest success | src/farming/harvest.rs | 160-164 | "Harvested {crop_name}!" (2.0s) |
| Crop withered | src/farming/events_handler.rs | 194-199 | "Some crops have withered..." (3.0s) |

## Implementation Details

- ToastEvent and PlaySfxEvent are imported via `use crate::shared::*;` in all three files.
- EventWriter<ToastEvent> is already a parameter in all relevant system functions.
- Season-change wither toast fires once per season change (not per crop), using a `had_deaths` flag.
- No files were modified; no out-of-scope edits.

## Validation

- cargo check: PASS

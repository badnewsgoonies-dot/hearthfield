# Hearthfield 2.0 Worker — Visual Polish Wave

## Mission
Add practical visual/UI polish aligned with Hearthfield 2.0 vision, without destabilizing gameplay systems.

## Required polish improvements
1. Main menu feel
- Improve title/main menu visual feedback:
  - stronger hover state readability
  - subtle glow/pulse on selected item
  - smooth visual transitions between root/load menus
- Target file(s):
  - `src/ui/main_menu.rs`
  - `src/ui/menu_kit.rs` (if needed)

2. Dialogue presentation
- Add or improve typewriter-like text reveal and smooth appearance timing if current behavior is abrupt.
- Target file:
  - `src/ui/dialogue_box.rs`

3. Interaction feedback
- Improve contextual prompt readability/polish (visual styling, subtle animation, or fade) while preserving function.
- Target file:
  - `src/ui/hud.rs`

4. Transition polish
- Improve fade timing/easing around day/menu transitions to feel less abrupt.
- Target file:
  - `src/ui/transitions.rs`

## Constraints
- Keep accessibility/readability.
- No major UI architecture refactor in this wave.
- Avoid heavy runtime cost.

## Validation
Run and pass:
- `cargo check`
- `cargo test --test headless`

## Deliverables
1. Visible polish changes in listed areas.
2. Brief before/after behavior notes in worker report.
3. Worker report:
- `status/workers/hearthfield2-worker-visual-polish-report.md`

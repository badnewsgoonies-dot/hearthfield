# Pause Menu Polish — Implementation Report

## Changes Made (`src/ui/pause_menu.rs`)

### 1. Semi-transparent dark background overlay
- **Was:** `BackgroundColor(theme.bg_overlay)` → resolves to `srgba(0.0, 0.0, 0.0, 0.6)` from `MenuTheme` default
- **Now:** `BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7))` — hardcoded to meet the 0.7 alpha spec

### 2. Game font (UiFontHandle)
- **Already correct.** `font_handle.0.clone()` from `Res<UiFontHandle>` is passed to all text nodes (title via `menu_kit::spawn_menu_title`, buttons via `menu_kit::spawn_menu_button`, status text inline, and footer inline). No changes needed.

### 3. Controls reminder
- **Was:** `menu_kit::spawn_menu_footer(...)` with multi-line text using wrong key bindings (`E: Inventory`, `Esc: Resume`, extra bindings), `theme.hint_font_size` (13px), and `theme.text_color_disabled` (`srgba(0.5, 0.45, 0.4, 0.6)`)
- **Now:** Inline `Text` node with:
  - Text: `"WASD: Move | F: Interact | Space: Use Tool | I: Inventory | C: Craft | Esc: Pause"`
  - Font size: `11.0`
  - Color: `Color::srgba(0.7, 0.7, 0.7, 0.8)`
  - `PickingBehavior::IGNORE` (consistent with `spawn_menu_footer`)

## Verification
- `cargo check` passes with no errors or warnings related to `pause_menu.rs`.

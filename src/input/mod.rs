use bevy::prelude::*;
use bevy::input::gamepad::{GamepadButton, GamepadAxis};
use bevy::input::touch::Touches;
use crate::shared::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TouchZoneState>();
        app.add_systems(
            PreUpdate,
            (
                reset_and_read_input,
                process_touch_input,
                manage_input_context,
            )
                .chain(),
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// GAMEPAD HELPERS
// ═══════════════════════════════════════════════════════════════════════
//
// Bevy 0.15 uses component-based gamepad API: gamepads are entities with
// a `Gamepad` component.  On native, `bevy_gilrs` spawns these entities.
// On WASM, gilrs 0.11 includes a web backend (via web-sys / wasm-bindgen)
// that reads navigator.getGamepads() and spawns the same entities.
//
// If no gamepad entities appear, the Query simply returns empty and all
// gamepad code gracefully falls back to keyboard + touch input.
// ═══════════════════════════════════════════════════════════════════════

const STICK_DEAD_ZONE: f32 = 0.2;

/// Apply dead zone: values with magnitude below threshold become 0.
fn apply_dead_zone(value: f32) -> f32 {
    if value.abs() < STICK_DEAD_ZONE {
        0.0
    } else {
        value
    }
}

/// Read the left stick as a Vec2, applying dead zone per axis.
fn read_left_stick(gamepad: &Gamepad) -> Vec2 {
    let x = apply_dead_zone(
        gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0),
    );
    let y = apply_dead_zone(
        gamepad.get(GamepadAxis::LeftStickY).unwrap_or(0.0),
    );
    Vec2::new(x, y)
}

/// Read D-pad as a Vec2 for movement (digital, -1/0/+1 per axis).
fn read_dpad_axis(gamepad: &Gamepad) -> Vec2 {
    let mut axis = Vec2::ZERO;
    if gamepad.pressed(GamepadButton::DPadUp) {
        axis.y += 1.0;
    }
    if gamepad.pressed(GamepadButton::DPadDown) {
        axis.y -= 1.0;
    }
    if gamepad.pressed(GamepadButton::DPadLeft) {
        axis.x -= 1.0;
    }
    if gamepad.pressed(GamepadButton::DPadRight) {
        axis.x += 1.0;
    }
    axis
}

/// Read D-pad as just_pressed for UI navigation.
fn read_dpad_just_pressed(gamepad: &Gamepad) -> (bool, bool, bool, bool) {
    (
        gamepad.just_pressed(GamepadButton::DPadUp),
        gamepad.just_pressed(GamepadButton::DPadDown),
        gamepad.just_pressed(GamepadButton::DPadLeft),
        gamepad.just_pressed(GamepadButton::DPadRight),
    )
}

// ═══════════════════════════════════════════════════════════════════════
// TOUCH INPUT
// ═══════════════════════════════════════════════════════════════════════

/// Identifies a touch zone on the overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TouchZone {
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    ActionUse,    // top button — tool use (Space)
    ActionTalk,   // right button — interact (F)
    ActionItem,   // bottom button — tool secondary (R)
    ActionMenu,   // left button — pause (Esc)
}

/// Tracks which touch zones are currently active and which were just pressed
/// this frame. Updated each frame by `process_touch_input`.
#[derive(Resource, Default)]
pub struct TouchZoneState {
    /// Which zones are currently held (touch active in the zone).
    pub held: [bool; 8],
    /// Which zones transitioned from not-held to held this frame.
    pub just_pressed: [bool; 8],
    /// Previous frame's held state (for edge detection).
    prev_held: [bool; 8],
}

impl TouchZoneState {
    fn zone_index(zone: TouchZone) -> usize {
        match zone {
            TouchZone::DpadUp => 0,
            TouchZone::DpadDown => 1,
            TouchZone::DpadLeft => 2,
            TouchZone::DpadRight => 3,
            TouchZone::ActionUse => 4,
            TouchZone::ActionTalk => 5,
            TouchZone::ActionItem => 6,
            TouchZone::ActionMenu => 7,
        }
    }

    pub fn is_held(&self, zone: TouchZone) -> bool {
        self.held[Self::zone_index(zone)]
    }

    pub fn is_just_pressed(&self, zone: TouchZone) -> bool {
        self.just_pressed[Self::zone_index(zone)]
    }
}

/// Touch zone layout uses screen-percentage coordinates so it scales with
/// any viewport size.  The d-pad occupies the bottom-left quadrant and the
/// action buttons occupy the bottom-right quadrant.
///
/// All coordinates are in "percentage of screen" (0.0 – 1.0).
/// Touch zones are circles defined by (center_x%, center_y%, radius%).
struct ZoneDef {
    zone: TouchZone,
    /// Center X as fraction of screen width (0.0 = left, 1.0 = right).
    cx: f32,
    /// Center Y as fraction of screen height (0.0 = top, 1.0 = bottom).
    cy: f32,
    /// Radius as fraction of screen width.
    radius: f32,
}

/// Zone definitions: percentage-based so they work at any resolution.
/// Based on the visual overlay layout:
///   D-pad center ~(12.5%, 81.5%) with ~6.25% radius per direction
///   Action buttons center ~(87.5%, 81.5%) with ~5.2% radius per button
const TOUCH_ZONES: &[ZoneDef] = &[
    // D-pad directions (centered on bottom-left)
    ZoneDef { zone: TouchZone::DpadUp,    cx: 0.125, cy: 0.72, radius: 0.055 },
    ZoneDef { zone: TouchZone::DpadDown,  cx: 0.125, cy: 0.92, radius: 0.055 },
    ZoneDef { zone: TouchZone::DpadLeft,  cx: 0.060, cy: 0.82, radius: 0.055 },
    ZoneDef { zone: TouchZone::DpadRight, cx: 0.190, cy: 0.82, radius: 0.055 },
    // Action buttons (centered on bottom-right)
    ZoneDef { zone: TouchZone::ActionUse,  cx: 0.875, cy: 0.72, radius: 0.050 },
    ZoneDef { zone: TouchZone::ActionTalk, cx: 0.940, cy: 0.82, radius: 0.050 },
    ZoneDef { zone: TouchZone::ActionItem, cx: 0.875, cy: 0.92, radius: 0.050 },
    ZoneDef { zone: TouchZone::ActionMenu, cx: 0.810, cy: 0.82, radius: 0.050 },
];

/// Reads Bevy's `Touches` resource and maps active touch positions to zones,
/// then merges the results into `PlayerInput` (OR'd with keyboard/gamepad).
///
/// Runs after `reset_and_read_input` in PreUpdate so it can OR with existing
/// keyboard and gamepad input.
fn process_touch_input(
    touches: Res<Touches>,
    windows: Query<&Window>,
    context: Res<InputContext>,
    mut input: ResMut<PlayerInput>,
    mut zone_state: ResMut<TouchZoneState>,
) {
    // Save previous held state for edge detection.
    zone_state.prev_held = zone_state.held;
    zone_state.held = [false; 8];
    zone_state.just_pressed = [false; 8];

    let Ok(window) = windows.get_single() else {
        return;
    };
    let w = window.width();
    let h = window.height();

    if w < 1.0 || h < 1.0 {
        return;
    }

    // Check all active touches against all zones.
    for touch in touches.iter() {
        let pos = touch.position();
        // Bevy touch coordinates: (0,0) = top-left, Y increases downward.
        let nx = pos.x / w; // normalized 0..1
        let ny = pos.y / h; // normalized 0..1

        for zone_def in TOUCH_ZONES {
            let dx = nx - zone_def.cx;
            let dy = ny - zone_def.cy;
            // Use screen-width-normalized radius for both axes to keep circles
            // roughly circular even on non-square viewports.  We scale the Y
            // delta by the aspect ratio so a "radius" in screen-width units
            // corresponds to the same physical distance vertically.
            let aspect = w / h;
            let dy_adj = dy * aspect;
            if dx * dx + dy_adj * dy_adj <= zone_def.radius * zone_def.radius {
                let idx = TouchZoneState::zone_index(zone_def.zone);
                zone_state.held[idx] = true;
            }
        }
    }

    // Compute just_pressed: held this frame but not last frame.
    for i in 0..8 {
        zone_state.just_pressed[i] = zone_state.held[i] && !zone_state.prev_held[i];
    }

    // Also count any touch press as any_key (for title screen).
    if touches.iter_just_pressed().next().is_some() {
        input.any_key = true;
    }

    // Map zones to PlayerInput based on current context.
    match *context {
        InputContext::Disabled => {}

        InputContext::Gameplay => {
            // D-pad → movement
            let mut touch_axis = Vec2::ZERO;
            if zone_state.is_held(TouchZone::DpadUp) {
                touch_axis.y += 1.0;
            }
            if zone_state.is_held(TouchZone::DpadDown) {
                touch_axis.y -= 1.0;
            }
            if zone_state.is_held(TouchZone::DpadLeft) {
                touch_axis.x -= 1.0;
            }
            if zone_state.is_held(TouchZone::DpadRight) {
                touch_axis.x += 1.0;
            }

            if touch_axis != Vec2::ZERO {
                // Merge with existing movement.
                let combined = input.move_axis + touch_axis;
                input.move_axis = if combined != Vec2::ZERO {
                    combined.normalize()
                } else {
                    Vec2::ZERO
                };
            }

            // Action buttons (just_pressed)
            input.tool_use = input.tool_use
                || zone_state.is_just_pressed(TouchZone::ActionUse);
            input.attack = input.tool_use;
            input.interact = input.interact
                || zone_state.is_just_pressed(TouchZone::ActionTalk);
            // Item button → open inventory (most useful on mobile)
            input.open_inventory = input.open_inventory
                || zone_state.is_just_pressed(TouchZone::ActionItem);
            input.pause = input.pause
                || zone_state.is_just_pressed(TouchZone::ActionMenu);
        }

        InputContext::Menu => {
            // D-pad → UI navigation (just_pressed)
            input.ui_up = input.ui_up
                || zone_state.is_just_pressed(TouchZone::DpadUp);
            input.ui_down = input.ui_down
                || zone_state.is_just_pressed(TouchZone::DpadDown);
            input.ui_left = input.ui_left
                || zone_state.is_just_pressed(TouchZone::DpadLeft);
            input.ui_right = input.ui_right
                || zone_state.is_just_pressed(TouchZone::DpadRight);
            // Talk → confirm, Use → confirm (either button works)
            input.ui_confirm = input.ui_confirm
                || zone_state.is_just_pressed(TouchZone::ActionTalk)
                || zone_state.is_just_pressed(TouchZone::ActionUse);
            // Menu OR Item → cancel/close (both ways to exit a screen)
            input.ui_cancel = input.ui_cancel
                || zone_state.is_just_pressed(TouchZone::ActionMenu)
                || zone_state.is_just_pressed(TouchZone::ActionItem);
            input.pause = input.pause
                || zone_state.is_just_pressed(TouchZone::ActionMenu);
            // Item also triggers open_inventory for toggle-close
            input.open_inventory = input.open_inventory
                || zone_state.is_just_pressed(TouchZone::ActionItem);
        }

        InputContext::Dialogue => {
            // Talk → advance dialogue
            input.interact = input.interact
                || zone_state.is_just_pressed(TouchZone::ActionTalk);
            // D-pad up/down → choice selection
            input.ui_up = input.ui_up
                || zone_state.is_just_pressed(TouchZone::DpadUp);
            input.ui_down = input.ui_down
                || zone_state.is_just_pressed(TouchZone::DpadDown);
            // Menu → cancel
            input.ui_cancel = input.ui_cancel
                || zone_state.is_just_pressed(TouchZone::ActionMenu);
        }

        InputContext::Fishing => {
            // Use button held → reel
            input.fishing_reel = input.fishing_reel
                || zone_state.is_held(TouchZone::ActionUse);
            // Use button just pressed → cast
            input.tool_use = input.tool_use
                || zone_state.is_just_pressed(TouchZone::ActionUse);
            // Menu → cancel fishing
            input.ui_cancel = input.ui_cancel
                || zone_state.is_just_pressed(TouchZone::ActionMenu);
        }

        InputContext::Cutscene => {
            // Any action button → skip cutscene
            input.skip_cutscene = input.skip_cutscene
                || zone_state.is_just_pressed(TouchZone::ActionUse)
                || zone_state.is_just_pressed(TouchZone::ActionTalk);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// INPUT READING
// ═══════════════════════════════════════════════════════════════════════

/// The single point where hardware input becomes game actions.
/// Reads keyboard first, then merges gamepad input (OR'd together).
/// Touch input is merged separately by `process_touch_input` which runs
/// immediately after this system.
#[allow(clippy::too_many_arguments)]
fn reset_and_read_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    bindings: Res<KeyBindings>,
    context: Res<InputContext>,
    mut input: ResMut<PlayerInput>,
    mut interaction_claimed: ResMut<InteractionClaimed>,
    gamepads: Query<&Gamepad>,
) {
    *input = PlayerInput::default();
    interaction_claimed.0 = false;

    // Grab the first connected gamepad (if any).
    // On native this is provided by bevy_gilrs; on WASM, gilrs's web backend
    // reads navigator.getGamepads() and spawns gamepad entities the same way.
    let gp = gamepads.iter().next();

    input.any_key =
        keys.get_just_pressed().next().is_some()
        || mouse.get_just_pressed().next().is_some()
        || gp.is_some_and(|g| {
            g.just_pressed(GamepadButton::South)
                || g.just_pressed(GamepadButton::East)
                || g.just_pressed(GamepadButton::North)
                || g.just_pressed(GamepadButton::West)
                || g.just_pressed(GamepadButton::Start)
                || g.just_pressed(GamepadButton::Select)
        });

    match *context {
        InputContext::Disabled => (),

        InputContext::Gameplay => {
            // ── Keyboard movement ──
            let mut axis = Vec2::ZERO;
            if keys.pressed(bindings.move_up) || keys.pressed(KeyCode::ArrowUp) {
                axis.y += 1.0;
            }
            if keys.pressed(bindings.move_down) || keys.pressed(KeyCode::ArrowDown) {
                axis.y -= 1.0;
            }
            if keys.pressed(bindings.move_left) || keys.pressed(KeyCode::ArrowLeft) {
                axis.x -= 1.0;
            }
            if keys.pressed(bindings.move_right) || keys.pressed(KeyCode::ArrowRight) {
                axis.x += 1.0;
            }

            // ── Gamepad movement (merged with keyboard) ──
            if let Some(gp) = gp {
                let stick = read_left_stick(gp);
                let dpad = read_dpad_axis(gp);
                // Prefer stick if it has input, otherwise fall back to D-pad.
                let gp_axis = if stick.length_squared() > 0.01 {
                    stick
                } else {
                    dpad
                };
                axis += gp_axis;
            }

            input.move_axis = if axis != Vec2::ZERO {
                axis.normalize()
            } else {
                Vec2::ZERO
            };

            // ── Keyboard actions ──
            input.interact = keys.just_pressed(bindings.interact);
            input.tool_use =
                keys.just_pressed(bindings.tool_use) || mouse.just_pressed(MouseButton::Left);
            input.tool_secondary =
                keys.just_pressed(bindings.tool_secondary) || mouse.just_pressed(MouseButton::Right);
            input.attack = input.tool_use;

            input.open_inventory = keys.just_pressed(bindings.open_inventory);
            input.open_crafting = keys.just_pressed(bindings.open_crafting);
            input.open_map = keys.just_pressed(bindings.open_map);
            input.open_journal = keys.just_pressed(bindings.open_journal);
            input.open_relationships = keys.just_pressed(bindings.open_relationships);
            input.pause = keys.just_pressed(bindings.pause);

            input.tool_next = keys.just_pressed(bindings.tool_next);
            input.tool_prev = keys.just_pressed(bindings.tool_prev);
            for (i, key) in [
                KeyCode::Digit1,
                KeyCode::Digit2,
                KeyCode::Digit3,
                KeyCode::Digit4,
                KeyCode::Digit5,
                KeyCode::Digit6,
                KeyCode::Digit7,
                KeyCode::Digit8,
                KeyCode::Digit9,
            ]
            .iter()
            .enumerate()
            {
                if keys.just_pressed(*key) {
                    input.tool_slot = Some(i as u8);
                    break;
                }
            }

            // Quicksave / quickload
            input.quicksave = keys.just_pressed(KeyCode::F5);
            input.quickload = keys.just_pressed(KeyCode::F9);

            // UI navigation for in-game overlays (chest panel, elevator, etc.)
            input.ui_up = keys.just_pressed(KeyCode::ArrowUp)
                || keys.just_pressed(bindings.move_up);
            input.ui_down = keys.just_pressed(KeyCode::ArrowDown)
                || keys.just_pressed(bindings.move_down);
            input.ui_left = keys.just_pressed(KeyCode::ArrowLeft)
                || keys.just_pressed(bindings.move_left);
            input.ui_right = keys.just_pressed(KeyCode::ArrowRight)
                || keys.just_pressed(bindings.move_right);
            input.tab_pressed = keys.just_pressed(KeyCode::Tab);
            input.ui_confirm = keys.just_pressed(bindings.ui_confirm);
            input.ui_cancel = keys.just_pressed(bindings.ui_cancel);

            // ── Gamepad actions (Gameplay) ──
            if let Some(gp) = gp {
                // A (South) → interact
                input.interact = input.interact || gp.just_pressed(GamepadButton::South);
                // X (West) → tool_use
                input.tool_use = input.tool_use || gp.just_pressed(GamepadButton::West);
                input.attack = input.tool_use;
                // Y (North) → tool_secondary
                input.tool_secondary = input.tool_secondary || gp.just_pressed(GamepadButton::North);
                // B (East) → pause
                input.pause = input.pause || gp.just_pressed(GamepadButton::East);
                // Start → pause
                input.pause = input.pause || gp.just_pressed(GamepadButton::Start);
                // Select/Back → open_inventory
                input.open_inventory = input.open_inventory || gp.just_pressed(GamepadButton::Select);
                // RB (Right bumper) → tool_next
                input.tool_next = input.tool_next || gp.just_pressed(GamepadButton::RightTrigger);
                // LB (Left bumper) → tool_prev
                input.tool_prev = input.tool_prev || gp.just_pressed(GamepadButton::LeftTrigger);

                // D-pad for UI navigation (in-game overlays)
                let (dup, ddown, dleft, dright) = read_dpad_just_pressed(gp);
                input.ui_up = input.ui_up || dup;
                input.ui_down = input.ui_down || ddown;
                input.ui_left = input.ui_left || dleft;
                input.ui_right = input.ui_right || dright;
                input.ui_confirm = input.ui_confirm || gp.just_pressed(GamepadButton::South);
                input.ui_cancel = input.ui_cancel || gp.just_pressed(GamepadButton::East);
            }
        }

        InputContext::Menu => {
            // ── Keyboard ──
            input.ui_up =
                keys.just_pressed(bindings.move_up) || keys.just_pressed(KeyCode::ArrowUp);
            input.ui_down =
                keys.just_pressed(bindings.move_down) || keys.just_pressed(KeyCode::ArrowDown);
            input.ui_left =
                keys.just_pressed(bindings.move_left) || keys.just_pressed(KeyCode::ArrowLeft);
            input.ui_right =
                keys.just_pressed(bindings.move_right) || keys.just_pressed(KeyCode::ArrowRight);
            input.ui_confirm =
                keys.just_pressed(bindings.ui_confirm) || keys.just_pressed(bindings.interact);
            input.ui_cancel = keys.just_pressed(bindings.ui_cancel);
            input.pause = keys.just_pressed(bindings.pause);
            input.tab_pressed = keys.just_pressed(KeyCode::Tab);

            // Allow E / C to toggle-close their respective menus
            input.open_inventory = keys.just_pressed(bindings.open_inventory);
            input.open_crafting = keys.just_pressed(bindings.open_crafting);
            input.open_journal = keys.just_pressed(bindings.open_journal);
            input.open_relationships = keys.just_pressed(bindings.open_relationships);

            // Quicksave / quickload available from pause menu
            input.quicksave = keys.just_pressed(KeyCode::F5);
            input.quickload = keys.just_pressed(KeyCode::F9);

            // ── Gamepad actions (Menu) ──
            if let Some(gp) = gp {
                // A → confirm / activate
                input.ui_confirm = input.ui_confirm || gp.just_pressed(GamepadButton::South);
                // B → cancel
                input.ui_cancel = input.ui_cancel || gp.just_pressed(GamepadButton::East);
                input.pause = input.pause || gp.just_pressed(GamepadButton::Start);
                // RB → tab
                input.tab_pressed = input.tab_pressed || gp.just_pressed(GamepadButton::RightTrigger);

                // D-pad or left stick for UI navigation
                let (dup, ddown, dleft, dright) = read_dpad_just_pressed(gp);

                // Also treat stick flicks as just_pressed: check stick past threshold.
                let stick = read_left_stick(gp);
                let stick_up = stick.y > 0.5;
                let stick_down = stick.y < -0.5;
                let stick_left = stick.x < -0.5;
                let stick_right = stick.x > 0.5;

                input.ui_up = input.ui_up || dup || stick_up;
                input.ui_down = input.ui_down || ddown || stick_down;
                input.ui_left = input.ui_left || dleft || stick_left;
                input.ui_right = input.ui_right || dright || stick_right;
            }
        }

        InputContext::Dialogue => {
            // ── Keyboard ──
            input.interact = keys.just_pressed(bindings.interact)
                || keys.just_pressed(bindings.ui_confirm)
                || keys.just_pressed(KeyCode::Space);
            input.skip_cutscene = keys.just_pressed(bindings.skip_cutscene);
            input.ui_up =
                keys.just_pressed(bindings.move_up) || keys.just_pressed(KeyCode::ArrowUp);
            input.ui_down =
                keys.just_pressed(bindings.move_down) || keys.just_pressed(KeyCode::ArrowDown);
            input.ui_cancel = keys.just_pressed(bindings.ui_cancel);

            // ── Gamepad actions (Dialogue) ──
            if let Some(gp) = gp {
                // A → advance dialogue
                input.interact = input.interact || gp.just_pressed(GamepadButton::South);
                // B → cancel
                input.ui_cancel = input.ui_cancel || gp.just_pressed(GamepadButton::East);
                // D-pad for dialogue choices
                let (dup, ddown, _, _) = read_dpad_just_pressed(gp);
                input.ui_up = input.ui_up || dup;
                input.ui_down = input.ui_down || ddown;
            }
        }

        InputContext::Fishing => {
            // ── Keyboard + Mouse ──
            input.fishing_reel =
                keys.pressed(bindings.tool_use) || mouse.pressed(MouseButton::Left);
            input.ui_cancel = keys.just_pressed(bindings.ui_cancel);
            input.tool_use = keys.just_pressed(bindings.tool_use);

            // ── Gamepad actions (Fishing) ──
            if let Some(gp) = gp {
                // A held → reel
                input.fishing_reel = input.fishing_reel || gp.pressed(GamepadButton::South);
                // A pressed → tool_use (cast)
                input.tool_use = input.tool_use || gp.just_pressed(GamepadButton::South);
                // B → cancel
                input.ui_cancel = input.ui_cancel || gp.just_pressed(GamepadButton::East);
            }
        }

        InputContext::Cutscene => {
            input.skip_cutscene = keys.just_pressed(bindings.skip_cutscene);

            // ── Gamepad: any face button to skip ──
            if let Some(gp) = gp {
                input.skip_cutscene = input.skip_cutscene
                    || gp.just_pressed(GamepadButton::South)
                    || gp.just_pressed(GamepadButton::East);
            }
        }
    }
}

/// Derives InputContext from GameState. ONE system, replaces all per-domain guards.
/// When the context changes, blanks all input for one frame to prevent carryover
/// (e.g., swinging a hoe near an NPC → Space carried into dialogue as "advance").
fn manage_input_context(
    game_state: Res<State<GameState>>,
    mut context: ResMut<InputContext>,
    mut input: ResMut<PlayerInput>,
) {
    let new_context = match *game_state.get() {
        GameState::MainMenu => InputContext::Menu,
        GameState::Playing => InputContext::Gameplay,
        GameState::Paused => InputContext::Menu,
        GameState::Inventory => InputContext::Menu,
        GameState::Shop => InputContext::Menu,
        GameState::Crafting => InputContext::Menu,
        GameState::BuildingUpgrade => InputContext::Menu,
        GameState::Journal => InputContext::Menu,
        GameState::RelationshipsView => InputContext::Menu,
        GameState::MapView => InputContext::Menu,
        GameState::Dialogue => InputContext::Dialogue,
        GameState::Fishing => InputContext::Fishing,
        GameState::Cutscene => InputContext::Cutscene,
        GameState::Loading => InputContext::Disabled,
        _ => InputContext::Gameplay,
    };

    if new_context != *context {
        // Context just switched — blank all input this frame to prevent
        // keys held from the old context from firing in the new one.
        *input = PlayerInput::default();
    }

    *context = new_context;
}

//! State machine driver.
//!
//! Until v15, Greenfield was permanently stuck in `Boot` (via boot_tick
//! which set the next state to `Tending` once and was done). The other
//! six `GreenfieldState` variants existed as enum members but nothing
//! ever drove the world into them, so cargo check reported them as
//! "never constructed".
//!
//! v16 wires the real transitions:
//!
//!   Boot       --first tick-->     MainMenu  (replaces old Boot→Tending skip)
//!   MainMenu   --any key-->        Tending   (player presses any key to play)
//!   Tending    --enemy nearby-->   Defending (when at least one Enemy entity exists)
//!   Defending  --no enemies-->     Tending   (when all Enemy entities are despawned)
//!   any        --Esc-->            Paused    (and back on Esc release into prev)
//!   any        --hp ≤ 0-->         GameOver
//!   GameOver   --any key-->        MainMenu  (reset for a fresh run)
//!
//! Each play state also gets a minimum visible artifact: MainMenu text,
//! Pause overlay, GameOver text. The components for these (HudMainMenu,
//! HudPause) were already declared in components.rs but were among the
//! 140 unused-struct warnings at v15.

use bevy::prelude::*;
use crate::game::GreenfieldState;
use crate::game::components::{HudMainMenu, HudPause, Enemy};
use crate::game::resources::PlayerHealth;

/// Tracks which state the game was in before Pause, so Esc-again
/// returns to the right state.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct PrePauseState(pub Option<GreenfieldState>);

// ─── transitions ────────────────────────────────────────────────────────


/// In MainMenu, any key press starts a fresh Tending round.
pub fn main_menu_to_tending(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<GreenfieldState>>,
) {
    if keyboard.get_just_pressed().count() > 0 {
        next.set(GreenfieldState::Tending);
    }
}

/// In Tending, transition to Defending when at least one Enemy exists.
pub fn tending_to_defending(
    enemies: Query<&Enemy>,
    current: Res<State<GreenfieldState>>,
    mut next: ResMut<NextState<GreenfieldState>>,
) {
    if *current.get() == GreenfieldState::Tending && enemies.iter().count() > 0 {
        next.set(GreenfieldState::Defending);
    }
}

/// In Defending, transition back to Tending when no Enemy entities remain.
pub fn defending_to_tending(
    enemies: Query<&Enemy>,
    current: Res<State<GreenfieldState>>,
    mut next: ResMut<NextState<GreenfieldState>>,
) {
    if *current.get() == GreenfieldState::Defending && enemies.iter().count() == 0 {
        next.set(GreenfieldState::Tending);
    }
}

/// Esc toggles Pause from any active gameplay state, and unpauses back
/// to the state we came from.
pub fn pause_toggle(
    keyboard: Res<ButtonInput<KeyCode>>,
    current: Res<State<GreenfieldState>>,
    mut next: ResMut<NextState<GreenfieldState>>,
    mut prev: ResMut<PrePauseState>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }
    let now = *current.get();
    match now {
        GreenfieldState::Tending | GreenfieldState::Defending | GreenfieldState::Harvesting => {
            prev.0 = Some(now);
            next.set(GreenfieldState::Paused);
        }
        GreenfieldState::Paused => {
            if let Some(back) = prev.0.take() {
                next.set(back);
            } else {
                next.set(GreenfieldState::Tending);
            }
        }
        _ => {}
    }
}

/// Trigger GameOver when player HP reaches 0 in any active play state.
pub fn check_game_over(
    health: Res<PlayerHealth>,
    current: Res<State<GreenfieldState>>,
    mut next: ResMut<NextState<GreenfieldState>>,
) {
    if health.hp <= 0.0 {
        match *current.get() {
            GreenfieldState::Tending | GreenfieldState::Defending | GreenfieldState::Harvesting => {
                next.set(GreenfieldState::GameOver);
            }
            _ => {}
        }
    }
}

/// From GameOver, any key returns to MainMenu and we reset HP to full.
pub fn game_over_to_main_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut next: ResMut<NextState<GreenfieldState>>,
    mut health: ResMut<PlayerHealth>,
) {
    if keyboard.get_just_pressed().count() > 0 {
        health.hp = health.max_hp.max(1.0);
        next.set(GreenfieldState::MainMenu);
    }
}

// ─── HUD spawn/despawn per state ────────────────────────────────────────

/// On entering MainMenu, spawn a title-screen text overlay.
pub fn spawn_main_menu_hud(mut commands: Commands) {
    commands.spawn((
        HudMainMenu,
        Text::new("GREENFIELD\n\nPress any key to begin"),
        TextFont { font_size: 32.0, ..default() },
        TextColor(Color::srgb(0.9, 0.95, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(35.0),
            left: Val::Percent(35.0),
            ..default()
        },
    ));
}

/// On exit, despawn the MainMenu HUD entities.
pub fn despawn_main_menu_hud(
    mut commands: Commands,
    q: Query<Entity, With<HudMainMenu>>,
) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

/// On entering Paused, spawn a pause overlay.
pub fn spawn_pause_hud(mut commands: Commands) {
    commands.spawn((
        HudPause,
        Text::new("== PAUSED ==\n\nEsc to resume"),
        TextFont { font_size: 28.0, ..default() },
        TextColor(Color::srgb(1.0, 1.0, 0.6)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(40.0),
            left: Val::Percent(40.0),
            ..default()
        },
    ));
}

pub fn despawn_pause_hud(
    mut commands: Commands,
    q: Query<Entity, With<HudPause>>,
) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

/// On entering GameOver, spawn a game-over text overlay.
/// We reuse HudPause as the marker since it's already declared and
/// gets despawned on exit; this avoids needing another component.
#[derive(Component, Debug)]
pub struct HudGameOver;

pub fn spawn_game_over_hud(mut commands: Commands) {
    commands.spawn((
        HudGameOver,
        Text::new("YOU DIED\n\nPress any key to try again"),
        TextFont { font_size: 36.0, ..default() },
        TextColor(Color::srgb(1.0, 0.4, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(38.0),
            left: Val::Percent(32.0),
            ..default()
        },
    ));
}

pub fn despawn_game_over_hud(
    mut commands: Commands,
    q: Query<Entity, With<HudGameOver>>,
) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ─── v17: Harvesting rhythm ─────────────────────────────────────────────

/// Tracks when the next harvest window opens. Driven off TurnClock.turn
/// which advances once per second.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct HarvestCycle {
    /// `turn` value of the most recent Tending → Harvesting transition,
    /// or `None` if we have never harvested yet.
    pub last_harvest_at: Option<u32>,
    /// `turn` value of the most recent Harvesting → Tending transition.
    pub last_yield_at: Option<u32>,
}

/// Every TENDING_TO_HARVEST_SECS in Tending, transition to Harvesting.
const TENDING_TO_HARVEST_SECS: u32 = 30;
const HARVEST_WINDOW_SECS:     u32 = 5;

pub fn tending_to_harvesting(
    clock: Res<crate::game::resources::TurnClock>,
    mut cycle: ResMut<HarvestCycle>,
    mut next: ResMut<NextState<GreenfieldState>>,
) {
    let since = match cycle.last_yield_at {
        Some(last) => clock.turn.saturating_sub(last),
        None => clock.turn, // pre-first-harvest: count from boot
    };
    if since >= TENDING_TO_HARVEST_SECS {
        cycle.last_harvest_at = Some(clock.turn);
        next.set(GreenfieldState::Harvesting);
    }
}

pub fn harvesting_to_tending(
    clock: Res<crate::game::resources::TurnClock>,
    mut cycle: ResMut<HarvestCycle>,
    mut next: ResMut<NextState<GreenfieldState>>,
) {
    let opened = cycle.last_harvest_at.unwrap_or(clock.turn);
    if clock.turn.saturating_sub(opened) >= HARVEST_WINDOW_SECS {
        cycle.last_yield_at = Some(clock.turn);
        next.set(GreenfieldState::Tending);
    }
}

/// While Harvesting, increment the game score at a steady rate.
pub fn harvest_payout(
    time: Res<Time>,
    mut score: ResMut<crate::game::resources::GameScore>,
    mut accum: Local<f32>,
) {
    *accum += time.delta_secs();
    while *accum >= 0.25 {
        *accum -= 0.25;
        score.total = score.total.saturating_add(10);
    }
}

/// On entering Harvesting, spawn a banner.
#[derive(Component, Debug)]
pub struct HudHarvesting;

pub fn spawn_harvesting_hud(mut commands: Commands) {
    commands.spawn((
        HudHarvesting,
        Text::new("HARVEST!"),
        TextFont { font_size: 40.0, ..default() },
        TextColor(Color::srgb(0.5, 1.0, 0.4)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Percent(10.0),
            left: Val::Percent(45.0),
            ..default()
        },
    ));
}

pub fn despawn_harvesting_hud(
    mut commands: Commands,
    q: Query<Entity, With<HudHarvesting>>,
) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ─── v17b: emit lifecycle events on state transitions ──────────────────
//
// Several lifecycle events were declared in events.rs but never sent —
// MainMenuEnteredEvent, GameStartedEvent, GamePausedEvent, etc.
// Fire them alongside the state transitions so downstream systems
// (recording, telemetry, audio cues) can hook in later without touching
// the state machine itself.

use crate::game::events::{
    MainMenuEnteredEvent, MainMenuExitedEvent,
    GameStartedEvent, GamePausedEvent, GameResumedEvent, GameEndedEvent,
};

pub fn emit_main_menu_entered(mut w: EventWriter<MainMenuEnteredEvent>) {
    w.send(MainMenuEnteredEvent);
}

pub fn emit_main_menu_exited(mut w: EventWriter<MainMenuExitedEvent>) {
    w.send(MainMenuExitedEvent);
}

pub fn emit_game_started(mut w: EventWriter<GameStartedEvent>) {
    w.send(GameStartedEvent);
}

pub fn emit_game_paused(mut w: EventWriter<GamePausedEvent>) {
    w.send(GamePausedEvent);
}

pub fn emit_game_resumed(mut w: EventWriter<GameResumedEvent>) {
    w.send(GameResumedEvent);
}

pub fn emit_game_ended(mut w: EventWriter<GameEndedEvent>) {
    w.send(GameEndedEvent);
}

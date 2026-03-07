use bevy::prelude::*;

pub mod day_summary;
pub mod hud;
pub mod interruption;
pub mod main_menu;
pub mod pause_menu;
pub mod task_board;

/// Marker component for all UI root nodes spawned by the city UI layer.
/// Used for targeted despawn on state transitions.
#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct PauseMenuRoot;

#[derive(Component)]
pub struct DaySummaryRoot;

#[derive(Component)]
pub struct TaskBoardRoot;

#[derive(Component)]
pub struct InterruptionPopupRoot;

#[derive(Component)]
pub struct ToastRoot;

#[derive(Component)]
pub struct HudCoworkerPanel;

#[derive(Component)]
pub struct HudCoworkerContent;

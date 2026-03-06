//! Settings menu UI — audio, display, controls, gameplay options.

use crate::shared::*;
use bevy::audio::Volume;
use bevy::prelude::*;
use bevy::window::{MonitorSelection, WindowMode};

// ─── Components ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct SettingsMenuRoot;

#[derive(Component)]
pub struct SettingsCategory;

#[derive(Component)]
pub struct SettingsSlider {
    pub key: SettingsKey,
    pub value: f32,
    pub min: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct SettingsToggle {
    pub key: SettingsKey,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SettingsKey {
    MasterVolume,
    MusicVolume,
    SfxVolume,
    PixelScale,
    Fullscreen,
    Difficulty,
    TutorialTips,
}

/// Persistent game settings, stored as a resource.
#[derive(Resource, Clone, Debug)]
pub struct GameSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub pixel_scale: f32,
    pub fullscreen: bool,
    pub difficulty: Difficulty,
    pub tutorial_tips: bool,
}

#[derive(Resource, Clone, Debug)]
pub struct VolumeSettings {
    pub music_volume: f32,
    pub sfx_volume: f32,
}

impl Default for VolumeSettings {
    fn default() -> Self {
        Self {
            music_volume: 0.7,
            sfx_volume: 0.8,
        }
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.8,
            music_volume: 0.7,
            sfx_volume: 0.8,
            pixel_scale: PIXEL_SCALE,
            fullscreen: false,
            difficulty: Difficulty::Normal,
            tutorial_tips: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub fn display_name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Hard => "Hard",
        }
    }
}

// ─── Spawn / Despawn ─────────────────────────────────────────────────────

pub fn spawn_settings_menu(
    mut commands: Commands,
    settings: Res<GameSettings>,
    font: Res<UiFontHandle>,
) {
    let root = commands
        .spawn((
            SettingsMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
            GlobalZIndex(50),
        ))
        .id();

    // Title
    let title = commands
        .spawn((
            Text::new("SETTINGS"),
            TextFont {
                font: font.0.clone(),
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(title);

    // ── Audio section ────────────────────────────────────────────
    spawn_section_label(&mut commands, root, &font, "AUDIO");
    spawn_slider_row(
        &mut commands,
        root,
        &font,
        "Master Volume",
        SettingsKey::MasterVolume,
        settings.master_volume,
        0.0,
        1.0,
    );
    spawn_slider_row(
        &mut commands,
        root,
        &font,
        "Music Volume",
        SettingsKey::MusicVolume,
        settings.music_volume,
        0.0,
        1.0,
    );
    spawn_slider_row(
        &mut commands,
        root,
        &font,
        "SFX Volume",
        SettingsKey::SfxVolume,
        settings.sfx_volume,
        0.0,
        1.0,
    );

    // ── Display section ──────────────────────────────────────────
    spawn_section_label(&mut commands, root, &font, "DISPLAY");
    spawn_slider_row(
        &mut commands,
        root,
        &font,
        "Pixel Scale",
        SettingsKey::PixelScale,
        settings.pixel_scale,
        1.0,
        5.0,
    );
    spawn_toggle_row(
        &mut commands,
        root,
        &font,
        "Fullscreen",
        SettingsKey::Fullscreen,
        settings.fullscreen,
    );

    // ── Controls section ─────────────────────────────────────────
    spawn_section_label(&mut commands, root, &font, "CONTROLS");
    let controls_text = commands
        .spawn((
            Text::new("WASD: Move | E: Interact | Space: Throttle | Tab: Map | Esc: Pause"),
            TextFont {
                font: font.0.clone(),
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(controls_text);

    // ── Gameplay section ─────────────────────────────────────────
    spawn_section_label(&mut commands, root, &font, "GAMEPLAY");
    spawn_toggle_row(
        &mut commands,
        root,
        &font,
        "Tutorial Tips",
        SettingsKey::TutorialTips,
        settings.tutorial_tips,
    );

    let difficulty_label = format!("Difficulty: {}", settings.difficulty.display_name());
    let diff_text = commands
        .spawn((
            Text::new(difficulty_label),
            TextFont {
                font: font.0.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.6)),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(diff_text);
}

pub fn despawn_settings_menu(mut commands: Commands, query: Query<Entity, With<SettingsMenuRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

/// Apply changed settings to the game systems.
pub fn apply_settings(
    settings: Res<GameSettings>,
    mut volume_settings: ResMut<VolumeSettings>,
    mut windows: Query<&mut Window>,
    global_volume: Option<ResMut<GlobalVolume>>,
) {
    if !settings.is_changed() {
        return;
    }
    volume_settings.music_volume = settings.music_volume * settings.master_volume;
    volume_settings.sfx_volume = settings.sfx_volume * settings.master_volume;
    if let Some(mut global_volume) = global_volume {
        global_volume.volume = Volume::new(settings.master_volume);
    }
    if let Ok(mut window) = windows.get_single_mut() {
        window.mode = if settings.fullscreen {
            WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
        } else {
            WindowMode::Windowed
        };
    }
    info!(
        "Settings applied: master={:.0}% music={:.0}% sfx={:.0}% scale={:.1} fullscreen={} (effective music={:.0}% sfx={:.0}%)",
        settings.master_volume * 100.0,
        settings.music_volume * 100.0,
        settings.sfx_volume * 100.0,
        settings.pixel_scale,
        settings.fullscreen,
        volume_settings.music_volume * 100.0,
        volume_settings.sfx_volume * 100.0,
    );
}

// ─── Helpers ─────────────────────────────────────────────────────────────

fn spawn_section_label(commands: &mut Commands, parent: Entity, font: &UiFontHandle, label: &str) {
    let id = commands
        .spawn((
            SettingsCategory,
            Text::new(format!("── {} ──", label)),
            TextFont {
                font: font.0.clone(),
                font_size: 18.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.8, 1.0)),
            Node {
                margin: UiRect::new(Val::ZERO, Val::ZERO, Val::Px(12.0), Val::Px(6.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(id);
}

#[allow(clippy::too_many_arguments)]
fn spawn_slider_row(
    commands: &mut Commands,
    parent: Entity,
    font: &UiFontHandle,
    label: &str,
    key: SettingsKey,
    value: f32,
    min: f32,
    max: f32,
) {
    let pct = ((value - min) / (max - min) * 100.0) as u32;
    let display = format!("{}: {}%", label, pct);
    let id = commands
        .spawn((
            SettingsSlider {
                key,
                value,
                min,
                max,
            },
            Text::new(display),
            TextFont {
                font: font.0.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Node {
                margin: UiRect::bottom(Val::Px(6.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(id);
}

fn spawn_toggle_row(
    commands: &mut Commands,
    parent: Entity,
    font: &UiFontHandle,
    label: &str,
    key: SettingsKey,
    enabled: bool,
) {
    let display = format!("{}: {}", label, if enabled { "ON" } else { "OFF" });
    let id = commands
        .spawn((
            SettingsToggle { key, enabled },
            Text::new(display),
            TextFont {
                font: font.0.clone(),
                font_size: 16.0,
                ..default()
            },
            TextColor(if enabled {
                Color::srgb(0.5, 1.0, 0.5)
            } else {
                Color::srgb(1.0, 0.5, 0.5)
            }),
            Node {
                margin: UiRect::bottom(Val::Px(6.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(parent).add_child(id);
}

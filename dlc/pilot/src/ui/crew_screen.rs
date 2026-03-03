//! Crew relationship viewer — shows all crew members, friendship, and details.

use bevy::prelude::*;
use crate::shared::*;

// ─── Components ──────────────────────────────────────────────────────────

#[derive(Component)]
pub struct CrewScreenRoot;

#[derive(Component)]
pub struct CrewCard {
    pub crew_id: String,
}

#[derive(Component)]
pub struct CrewDetailPanel;

#[derive(Component)]
pub struct FriendshipBar;

// ─── Spawn / Despawn ─────────────────────────────────────────────────────

pub fn spawn_crew_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    crew_registry: Res<CrewRegistry>,
    relationships: Res<Relationships>,
) {
    let root = commands
        .spawn((
            CrewScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.02, 0.08, 0.95)),
            GlobalZIndex(40),
        ))
        .id();

    // Title
    let title = commands
        .spawn((
            Text::new("CREW ROSTER"),
            TextFont {
                font: font.0.clone(),
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.85, 0.4)),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(title);

    // Grid container for crew cards
    let grid = commands
        .spawn((
            Node {
                width: Val::Percent(90.0),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(12.0),
                row_gap: Val::Px(12.0),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(grid);

    // Sort crew by ID for consistent ordering
    let mut crew_ids: Vec<&String> = crew_registry.members.keys().collect();
    crew_ids.sort();

    for crew_id in crew_ids {
        let member = &crew_registry.members[crew_id];
        let friendship = relationships.friendship_level(crew_id);
        let tier = relationships.friendship_tier(crew_id);

        // Portrait placeholder — colored rectangle based on tint
        let portrait_color = Color::srgb(
            member.tint_color[0],
            member.tint_color[1],
            member.tint_color[2],
        );

        let card = commands
            .spawn((
                CrewCard {
                    crew_id: crew_id.clone(),
                },
                Node {
                    width: Val::Px(200.0),
                    min_height: Val::Px(160.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(8.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.9)),
                BorderColor(portrait_color),
            ))
            .id();
        commands.entity(grid).add_child(card);

        // Portrait square
        let portrait = commands
            .spawn((
                Node {
                    width: Val::Px(48.0),
                    height: Val::Px(48.0),
                    margin: UiRect::bottom(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(portrait_color),
            ))
            .id();
        commands.entity(card).add_child(portrait);

        // Name
        let name_label = commands
            .spawn((
                Text::new(member.name.clone()),
                TextFont {
                    font: font.0.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(card).add_child(name_label);

        // Role
        let role_label = commands
            .spawn((
                Text::new(member.role.display_name()),
                TextFont {
                    font: font.0.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.8, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(2.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(card).add_child(role_label);

        // Home airport
        let home_label = commands
            .spawn((
                Text::new(format!("Home: {}", member.home_airport.display_name())),
                TextFont {
                    font: font.0.clone(),
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(card).add_child(home_label);

        // Friendship tier text
        let tier_color = match tier {
            "Best Friend" => Color::srgb(1.0, 0.85, 0.0),
            "Good Friend" => Color::srgb(0.3, 0.9, 0.3),
            "Friendly" => Color::srgb(0.5, 0.8, 0.5),
            "Acquaintance" => Color::srgb(0.6, 0.6, 0.6),
            "Cold" => Color::srgb(0.5, 0.5, 0.8),
            "Hostile" => Color::srgb(1.0, 0.3, 0.3),
            _ => Color::srgb(0.5, 0.5, 0.5),
        };
        let tier_label = commands
            .spawn((
                Text::new(format!("{} ({})", tier, friendship)),
                TextFont {
                    font: font.0.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(tier_color),
                Node {
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(card).add_child(tier_label);

        // Friendship bar background
        let bar_bg = commands
            .spawn((
                Node {
                    width: Val::Px(160.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            ))
            .id();
        commands.entity(card).add_child(bar_bg);

        // Friendship bar fill — friendship ranges -100 to 100, map to 0..1
        let fill_pct = ((friendship + 100) as f32 / 200.0).clamp(0.0, 1.0);
        let bar_fill = commands
            .spawn((
                FriendshipBar,
                Node {
                    width: Val::Percent(fill_pct * 100.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(tier_color),
            ))
            .id();
        commands.entity(bar_bg).add_child(bar_fill);
    }
}

pub fn despawn_crew_screen(
    mut commands: Commands,
    query: Query<Entity, With<CrewScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

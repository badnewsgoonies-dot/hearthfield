//! Notification / inbox system — mission results, rank ups, financial alerts, etc.

use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════════
// DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NotificationCategory {
    Mission,
    Financial,
    Social,
    System,
}

impl NotificationCategory {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Mission => "✈",
            Self::Financial => "💰",
            Self::Social => "👥",
            Self::System => "⚙",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Mission => Color::srgb(0.3, 0.6, 1.0),
            Self::Financial => Color::srgb(1.0, 0.85, 0.2),
            Self::Social => Color::srgb(0.6, 0.9, 0.5),
            Self::System => Color::srgb(0.7, 0.7, 0.7),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NotificationEntry {
    pub id: u32,
    pub title: String,
    pub body: String,
    pub timestamp: String,
    pub read: bool,
    pub category: NotificationCategory,
}

// ═══════════════════════════════════════════════════════════════════════════
// RESOURCE
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Resource, Default)]
pub struct NotificationCenter {
    pub entries: Vec<NotificationEntry>,
    next_id: u32,
}

impl NotificationCenter {
    pub fn add(
        &mut self,
        title: impl Into<String>,
        body: impl Into<String>,
        category: NotificationCategory,
        calendar: &Calendar,
    ) {
        self.next_id += 1;
        self.entries.push(NotificationEntry {
            id: self.next_id,
            title: title.into(),
            body: body.into(),
            timestamp: format!("{} {}", calendar.formatted_date(), calendar.formatted_time()),
            read: false,
            category,
        });
    }

    pub fn unread_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.read).count()
    }

    pub fn mark_read(&mut self, id: u32) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.read = true;
        }
    }

    pub fn mark_all_read(&mut self) {
        for entry in &mut self.entries {
            entry.read = true;
        }
    }

    pub fn delete(&mut self, id: u32) {
        self.entries.retain(|e| e.id != id);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// LISTENER SYSTEMS — generate notifications from game events
// ═══════════════════════════════════════════════════════════════════════════

pub fn notify_mission_complete(
    mut events: EventReader<MissionCompletedEvent>,
    mut center: ResMut<NotificationCenter>,
    calendar: Res<Calendar>,
) {
    for evt in events.read() {
        center.add(
            "Mission Complete!",
            format!(
                "Mission \"{}\" completed. Earned {} gold and {} XP.",
                evt.mission_id, evt.gold_earned, evt.xp_earned
            ),
            NotificationCategory::Mission,
            &calendar,
        );
    }
}

pub fn notify_rank_up(
    mut events: EventReader<RankUpEvent>,
    mut center: ResMut<NotificationCenter>,
    calendar: Res<Calendar>,
) {
    for evt in events.read() {
        center.add(
            "Rank Up!",
            format!("Congratulations! You've been promoted to {}.", evt.new_rank.display_name()),
            NotificationCategory::System,
            &calendar,
        );
    }
}

pub fn notify_friendship_milestone(
    mut events: EventReader<FriendshipChangeEvent>,
    mut center: ResMut<NotificationCenter>,
    relationships: Res<Relationships>,
    calendar: Res<Calendar>,
) {
    for evt in events.read() {
        let level = relationships.friendship_level(&evt.npc_id);
        let tier = relationships.friendship_tier(&evt.npc_id);

        // Notify at tier boundaries
        if level == 25 || level == 50 || level == 75 || level == 100 {
            center.add(
                format!("Friendship: {}", evt.npc_id),
                format!("{} is now your {}! (Level {level})", evt.npc_id, tier),
                NotificationCategory::Social,
                &calendar,
            );
        }
    }
}

pub fn notify_gold_milestones(
    mut events: EventReader<GoldChangeEvent>,
    gold: Res<Gold>,
    mut center: ResMut<NotificationCenter>,
    calendar: Res<Calendar>,
) {
    for _evt in events.read() {
        let milestones = [1000, 5000, 10000, 50000, 100000, 500000, 1000000];
        for &m in &milestones {
            if gold.amount >= m && gold.amount - m as u32 <= 500 {
                center.add(
                    format!("Financial Milestone: {m} gold!"),
                    format!("Your balance has reached {m} gold. Keep it up, pilot!"),
                    NotificationCategory::Financial,
                    &calendar,
                );
                break;
            }
        }
    }
}

pub fn notify_achievement(
    mut events: EventReader<AchievementUnlockedEvent>,
    mut center: ResMut<NotificationCenter>,
    calendar: Res<Calendar>,
) {
    for evt in events.read() {
        center.add(
            "Achievement Unlocked!",
            format!("You unlocked: {}", evt.achievement_id),
            NotificationCategory::System,
            &calendar,
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// UI COMPONENTS & SCREEN
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Component)]
pub struct NotificationScreenRoot;

#[derive(Component)]
pub struct NotificationItem {
    pub notification_id: u32,
}

#[derive(Component)]
pub struct UnreadBadge;

pub fn spawn_notification_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    center: Res<NotificationCenter>,
) {
    let root = commands
        .spawn((
            NotificationScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.95)),
            GlobalZIndex(40),
        ))
        .id();

    let text_style = TextFont {
        font: font.0.clone(),
        font_size: 14.0,
        ..default()
    };

    // Title with unread count
    let unread = center.unread_count();
    let title_text = if unread > 0 {
        format!("NOTIFICATIONS ({unread} unread)")
    } else {
        "NOTIFICATIONS".to_string()
    };

    let title = commands
        .spawn((
            Text::new(title_text),
            TextFont {
                font: font.0.clone(),
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(16.0)),
                ..default()
            },
        ))
        .id();
    commands.entity(root).add_child(title);

    // Entries (newest first)
    for entry in center.entries.iter().rev().take(20) {
        let read_alpha = if entry.read { 0.5 } else { 1.0 };
        let cat_icon = entry.category.icon();

        let row = commands
            .spawn((
                NotificationItem {
                    notification_id: entry.id,
                },
                Node {
                    width: Val::Percent(90.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    margin: UiRect::bottom(Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.8)),
                BorderColor(entry.category.color()),
            ))
            .id();

        let header = commands
            .spawn((
                Text::new(format!("{cat_icon} {} — {}", entry.title, entry.timestamp)),
                text_style.clone(),
                TextColor(Color::srgba(1.0, 1.0, 1.0, read_alpha)),
            ))
            .id();

        let body = commands
            .spawn((
                Text::new(entry.body.clone()),
                TextFont {
                    font: font.0.clone(),
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.8, 0.8, read_alpha)),
                Node {
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
            ))
            .id();

        commands.entity(row).add_children(&[header, body]);
        commands.entity(root).add_child(row);
    }

    if center.entries.is_empty() {
        let empty_msg = commands
            .spawn((
                Text::new("No notifications yet."),
                text_style.clone(),
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::top(Val::Px(40.0)),
                    ..default()
                },
            ))
            .id();
        commands.entity(root).add_child(empty_msg);
    }
}

pub fn despawn_notification_screen(
    mut commands: Commands,
    query: Query<Entity, With<NotificationScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

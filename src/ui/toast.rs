use super::UiFontHandle;
use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════
// COMPONENTS & RESOURCES
// ═══════════════════════════════════════════════════════════════════════

/// Marker for the toast container node (top-center of screen).
#[derive(Component)]
pub struct ToastContainer;

/// Marker for individual toast nodes.
#[derive(Component)]
pub struct ToastItem {
    pub timer: Timer,
    pub fade_timer: Option<Timer>,
}

/// Marker for the coloured left accent bar inside a toast.
#[derive(Component)]
pub struct ToastAccent;

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN CONTAINER
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_toast_container(mut commands: Commands) {
    commands.spawn((
        ToastContainer,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(72.0),
            left: Val::Percent(50.0),
            // We translate back by a fixed amount to centre the column.
            // Bevy 0.15 doesn't expose `transform` on UI nodes directly, so
            // we use a generous max-width with auto margins to stay centred.
            width: Val::Px(360.0),
            // Shift left by half of the width to truly center it.
            margin: UiRect {
                left: Val::Px(-180.0),
                ..default()
            },
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            align_items: AlignItems::Center,
            ..default()
        },
        PickingBehavior::IGNORE,
    ));
}

pub fn despawn_toast_container(mut commands: Commands, query: Query<Entity, With<ToastContainer>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ACCENT COLOUR HELPER — infers toast category from message text
// ═══════════════════════════════════════════════════════════════════════

fn toast_accent_color(message: &str) -> Color {
    let lower = message.to_lowercase();
    if lower.contains("gold")
        || lower.contains("earned")
        || lower.contains("+")
        || lower.ends_with("g")
    {
        // Gold/yellow accent for gold-related messages.
        Color::srgb(1.0, 0.84, 0.0)
    } else if lower.contains("achievement") {
        // Purple accent for achievements.
        Color::srgb(0.7, 0.4, 1.0)
    } else if lower.contains("full") || lower.contains("can't") || lower.contains("not enough") {
        // Red accent for error/warning messages.
        Color::srgb(0.95, 0.25, 0.25)
    } else if lower.contains("autosave")
        || lower.contains("saved")
        || lower.contains("save")
        || lower.contains("loaded")
    {
        // Soft green accent for save/load system confirmations.
        Color::srgb(0.74, 0.88, 0.68)
    } else {
        // Neutral white/grey accent for everything else.
        Color::srgba(0.8, 0.8, 0.8, 0.6)
    }
}

// ═══════════════════════════════════════════════════════════════════════
// HANDLE TOAST EVENTS — spawn a child node per event
// ═══════════════════════════════════════════════════════════════════════

pub fn handle_toast_events(
    mut commands: Commands,
    mut events: EventReader<ToastEvent>,
    font_handle: Res<UiFontHandle>,
    container_query: Query<Entity, With<ToastContainer>>,
    existing_toasts: Query<Entity, With<ToastItem>>,
) {
    let Ok(container) = container_query.get_single() else {
        return;
    };

    for event in events.read() {
        // Enforce max 3 visible toasts: despawn oldest if over limit.
        let toast_entities: Vec<Entity> = existing_toasts.iter().collect();
        if toast_entities.len() >= 3 {
            // Despawn the first (oldest) toast in the list.
            if let Some(&oldest) = toast_entities.first() {
                commands.entity(oldest).despawn_recursive();
            }
        }

        let font = font_handle.0.clone();
        let message = event.message.clone();
        let duration = event.duration_secs;

        // Determine category accent colour from message content.
        let accent_color = toast_accent_color(&message);

        // Spawn the toast as a child of the container.
        let toast_entity = commands
            .spawn((
                ToastItem {
                    timer: Timer::from_seconds(duration, TimerMode::Once),
                    fade_timer: None,
                },
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Stretch,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.82)),
                BorderColor(Color::srgba(0.5, 0.5, 0.5, 0.68)),
                PickingBehavior::IGNORE,
            ))
            .with_children(|parent| {
                // Coloured left accent bar (4px wide).
                parent.spawn((
                    ToastAccent,
                    Node {
                        width: Val::Px(4.0),
                        min_height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(accent_color),
                    PickingBehavior::IGNORE,
                ));
                // Text content with padding.
                parent.spawn((
                    Text::new(message),
                    TextFont {
                        font,
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        padding: UiRect {
                            left: Val::Px(10.0),
                            right: Val::Px(14.0),
                            top: Val::Px(7.0),
                            bottom: Val::Px(7.0),
                        },
                        ..default()
                    },
                    PickingBehavior::IGNORE,
                ));
            })
            .id();

        commands.entity(container).add_child(toast_entity);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// UPDATE TOASTS — tick timers, fade out, despawn
// ═══════════════════════════════════════════════════════════════════════

pub fn update_toasts(
    mut commands: Commands,
    time: Res<Time>,
    mut toast_query: Query<(Entity, &mut ToastItem, &mut BackgroundColor, &Children)>,
    mut text_color_query: Query<&mut TextColor>,
    mut accent_bg_query: Query<&mut BackgroundColor, (With<ToastAccent>, Without<ToastItem>)>,
) {
    for (entity, mut toast, mut bg_color, children) in &mut toast_query {
        // If not yet in fade mode, tick main timer.
        if toast.fade_timer.is_none() {
            toast.timer.tick(time.delta());

            if toast.timer.just_finished() {
                // Transition to fade-out phase.
                toast.fade_timer = Some(Timer::from_seconds(0.7, TimerMode::Once));
            }
        } else if let Some(ft) = toast.fade_timer.as_mut() {
            // Tick fade timer.
            ft.tick(time.delta());
            let finished = ft.finished();

            if finished {
                // Fully faded — despawn.
                commands.entity(entity).despawn_recursive();
            } else {
                // Reduce alpha based on how far through the fade we are.
                let elapsed = ft.elapsed_secs();
                let duration = ft.duration().as_secs_f32();
                let progress = (elapsed / duration).clamp(0.0, 1.0);
                let alpha = 1.0 - progress;

                // Fade the background.
                let current = bg_color.0;
                bg_color.0 = Color::srgba(
                    current.to_srgba().red,
                    current.to_srgba().green,
                    current.to_srgba().blue,
                    0.82 * alpha,
                );

                // Fade the text children and accent bar.
                for &child in children.iter() {
                    if let Ok(mut text_color) = text_color_query.get_mut(child) {
                        text_color.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
                    }
                    if let Ok(mut accent_bg) = accent_bg_query.get_mut(child) {
                        let ac = accent_bg.0.to_srgba();
                        accent_bg.0 = Color::srgba(ac.red, ac.green, ac.blue, alpha);
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// EVENT-TO-TOAST WIRING SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

pub fn wire_gold_toasts(
    mut gold_events: EventReader<GoldChangeEvent>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    for event in gold_events.read() {
        let message = if event.amount >= 0 {
            format!("+{}g", event.amount)
        } else {
            format!("-{}g", event.amount.unsigned_abs())
        };
        toast_writer.send(ToastEvent {
            message,
            duration_secs: 2.0,
        });
    }
}

pub fn wire_season_toasts(
    mut season_events: EventReader<SeasonChangeEvent>,
    mut toast_writer: EventWriter<ToastEvent>,
) {
    for event in season_events.read() {
        let season_name = match event.new_season {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Fall => "Fall",
            Season::Winter => "Winter",
        };
        toast_writer.send(ToastEvent {
            message: format!("{} has arrived!", season_name),
            duration_secs: 4.0,
        });
    }
}

pub fn wire_pickup_toasts(
    mut pickup_events: EventReader<ItemPickupEvent>,
    mut toast_writer: EventWriter<ToastEvent>,
    item_registry: Res<ItemRegistry>,
) {
    for event in pickup_events.read() {
        let item_name = item_registry
            .get(&event.item_id)
            .map(|def| def.name.clone())
            .unwrap_or_else(|| event.item_id.clone());
        toast_writer.send(ToastEvent {
            message: format!("Got {} x{}", item_name, event.quantity),
            duration_secs: 2.0,
        });
    }
}

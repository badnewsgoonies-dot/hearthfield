use bevy::prelude::*;
use crate::shared::*;
use super::UiFontHandle;

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

// ═══════════════════════════════════════════════════════════════════════
// SPAWN / DESPAWN CONTAINER
// ═══════════════════════════════════════════════════════════════════════

pub fn spawn_toast_container(mut commands: Commands) {
    commands.spawn((
        ToastContainer,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(60.0),
            left: Val::Percent(50.0),
            // We translate back by a fixed amount to centre the column.
            // Bevy 0.15 doesn't expose `transform` on UI nodes directly, so
            // we use a generous max-width with auto margins to stay centred.
            width: Val::Px(320.0),
            // Shift left by half of the width to truly center it.
            margin: UiRect {
                left: Val::Px(-160.0),
                ..default()
            },
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            align_items: AlignItems::Center,
            ..default()
        },
        PickingBehavior::IGNORE,
    ));
}

pub fn despawn_toast_container(
    mut commands: Commands,
    query: Query<Entity, With<ToastContainer>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
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

        // Spawn the toast as a child of the container.
        let toast_entity = commands
            .spawn((
                ToastItem {
                    timer: Timer::from_seconds(duration, TimerMode::Once),
                    fade_timer: None,
                },
                Node {
                    padding: UiRect {
                        left: Val::Px(12.0),
                        right: Val::Px(12.0),
                        top: Val::Px(5.0),
                        bottom: Val::Px(5.0),
                    },
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
                BorderColor(Color::srgba(0.5, 0.5, 0.5, 0.5)),
                PickingBehavior::IGNORE,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(message),
                    TextFont {
                        font,
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
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
) {
    for (entity, mut toast, mut bg_color, children) in &mut toast_query {
        // If not yet in fade mode, tick main timer.
        if toast.fade_timer.is_none() {
            toast.timer.tick(time.delta());

            if toast.timer.just_finished() {
                // Transition to fade-out phase.
                toast.fade_timer = Some(Timer::from_seconds(0.5, TimerMode::Once));
            }
        } else {
            // Tick fade timer.
            let finished = {
                let ft = toast.fade_timer.as_mut().unwrap();
                ft.tick(time.delta());
                ft.finished()
            };

            if finished {
                // Fully faded — despawn.
                commands.entity(entity).despawn_recursive();
            } else {
                // Reduce alpha based on how far through the fade we are.
                let elapsed = toast.fade_timer.as_ref().unwrap().elapsed_secs();
                let duration = toast.fade_timer.as_ref().unwrap().duration().as_secs_f32();
                let progress = (elapsed / duration).clamp(0.0, 1.0);
                let alpha = 1.0 - progress;

                // Fade the background.
                let current = bg_color.0;
                bg_color.0 = Color::srgba(
                    current.to_srgba().red,
                    current.to_srgba().green,
                    current.to_srgba().blue,
                    0.75 * alpha,
                );

                // Fade the text children.
                for &child in children.iter() {
                    if let Ok(mut text_color) = text_color_query.get_mut(child) {
                        text_color.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
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

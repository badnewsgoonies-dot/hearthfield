//! Grid inventory screen — displays item slots from Inventory resource.

use bevy::prelude::*;
use crate::shared::*;

#[derive(Component)]
pub struct InventoryScreenRoot;

pub fn spawn_inventory_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    inventory: Res<Inventory>,
) {
    let title_style = TextFont {
        font: font.0.clone(),
        font_size: 24.0,
        ..default()
    };
    let slot_style = TextFont {
        font: font.0.clone(),
        font_size: 13.0,
        ..default()
    };

    commands
        .spawn((
            InventoryScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.95)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("INVENTORY"),
                title_style,
                TextColor(Color::srgb(0.9, 0.8, 0.3)),
            ));

            // Grid container
            parent
                .spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|grid| {
                    for i in 0..inventory.max_slots {
                        let label = inventory
                            .slots
                            .get(i)
                            .map(|s| format!("{} x{}", s.item_id, s.quantity))
                            .unwrap_or_else(|| "—".to_string());

                        let bg = if i < inventory.slots.len() {
                            Color::srgb(0.2, 0.2, 0.3)
                        } else {
                            Color::srgb(0.12, 0.12, 0.15)
                        };

                        grid.spawn((
                            Node {
                                width: Val::Px(80.0),
                                height: Val::Px(40.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(bg),
                        ))
                        .with_children(|slot| {
                            slot.spawn((
                                Text::new(label),
                                slot_style.clone(),
                                TextColor(Color::WHITE),
                            ));
                        });
                    }
                });
        });
}

pub fn despawn_inventory_screen(
    mut commands: Commands,
    query: Query<Entity, With<InventoryScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

//! Shop screen — listing with buy buttons, reads ActiveShop resource.

use bevy::prelude::*;
use crate::shared::*;

#[derive(Component)]
pub struct ShopScreenRoot;

pub fn spawn_shop_screen(
    mut commands: Commands,
    font: Res<UiFontHandle>,
    shop: Res<ActiveShop>,
    gold: Res<Gold>,
) {
    let title_style = TextFont {
        font: font.0.clone(),
        font_size: 24.0,
        ..default()
    };
    let item_style = TextFont {
        font: font.0.clone(),
        font_size: 14.0,
        ..default()
    };
    let btn_style = TextFont {
        font: font.0.clone(),
        font_size: 13.0,
        ..default()
    };

    commands
        .spawn((
            ShopScreenRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.95)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("{} — Gold: {}", shop.name, gold.amount)),
                title_style,
                TextColor(Color::srgb(0.9, 0.8, 0.3)),
            ));

            for listing in &shop.listings {
                let stock_text = listing
                    .stock
                    .map(|s| format!(" (stock: {s})"))
                    .unwrap_or_default();

                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(12.0),
                            padding: UiRect::all(Val::Px(6.0)),
                            width: Val::Percent(70.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9)),
                    ))
                    .with_children(|row| {
                        row.spawn((
                            Text::new(format!(
                                "{} — {} gold{}",
                                listing.item_id, listing.price, stock_text
                            )),
                            item_style.clone(),
                            TextColor(Color::WHITE),
                            Node {
                                flex_grow: 1.0,
                                ..default()
                            },
                        ));
                        row.spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(4.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.4, 0.6)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new("Buy"),
                                btn_style.clone(),
                                TextColor(Color::WHITE),
                            ));
                        });
                    });
            }
        });
}

pub fn despawn_shop_screen(
    mut commands: Commands,
    query: Query<Entity, With<ShopScreenRoot>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

/// Updates row highlight colours to track the cursor position.
#[allow(clippy::type_complexity)]
pub fn render_tick(
    ui_state: Option<Res<BuildingUpgradeMenuState>>,
    mut row_query: Query<(&BuildingRow, &mut BackgroundColor)>,
    mut status_query: Query<
        &mut Text,
        (
            With<BuildingUpgradeStatusText>,
            Without<BuildingRowText>,
            Without<BuildingRowCost>,
        ),
    >,
) {
    let Some(ui_state) = ui_state else { return };

    // Cursor highlight
    for (row, mut bg) in &mut row_query {
        if row.index == ui_state.cursor {
            *bg = BackgroundColor(Color::srgba(0.35, 0.3, 0.2, 0.9));
        } else {
            *bg = BackgroundColor(Color::srgba(0.2, 0.17, 0.14, 0.6));
        }
    }

    // Status text
    for mut text in &mut status_query {
        **text = ui_state.status_message.clone();
    }
}



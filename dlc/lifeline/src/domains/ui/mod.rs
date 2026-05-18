//! Lifeline::domains::ui — substrate mechanical fill v2.
//! Pure substitution port of hearthfield::add_items_to_inventory.
//! No LLM. parser_v3 + complete substitution map + cargo gate.

use bevy::prelude::*;

// ── components ────────────────────────────────────────────────────

#[derive(Component, Debug, Default, Clone)]
pub struct UiHost;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct PanelSlot(pub Vec2);

// ── resources ─────────────────────────────────────────────────────

#[derive(Resource, Debug, Default)]
pub struct UiPanel {
    pub entries: Vec<String>,
}

impl UiPanel {
    pub fn try_add(&mut self, id: &str, _qty: u8, _max: u8) -> u8 {
        self.entries.push(id.to_string());
        0
    }
    pub fn try_remove(&mut self, _id: &str, _qty: u8) -> u8 {
        if self.entries.pop().is_some() { 1 } else { 0 }
    }
}

#[derive(Debug, Clone, Default)]
pub struct UiRegistryDef {
    pub name: String,
    pub stack_size: StackSize,
}
impl UiRegistryDef {
    pub fn name(&self) -> &str { &self.name }
    pub fn as_str(&self) -> &str { &self.name }
}

#[derive(Resource, Debug, Default)]
pub struct UiRegistry {
    pub defs: std::collections::HashMap<String, UiRegistryDef>,
}

impl UiRegistry {
    pub fn get(&self, id: &str) -> Option<&UiRegistryDef> {
        self.defs.get(id)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StackSize(pub u8);
impl StackSize {
    pub fn get(self) -> u8 { self.0 }
}

// ── events ────────────────────────────────────────────────────────

#[derive(Event, Debug, Clone, Default)]
pub struct PanelOpenedEvent {
    pub panel_id: String,
    pub quantity: u8,
}

#[derive(Event, Debug, Clone, Default)]
pub struct UiChimeEvent {
    pub alert_id: String,
}

#[derive(Event, Debug, Clone, Default)]
pub struct UiToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

// ── helpers ───────────────────────────────────────────────────────

pub fn spawn_panel_indicator(_commands: &mut Commands, _pos: Vec2) {}



// ── ported systems ────────────────────────────────────────────────

/// Reads PanelOpenedEvent (fired by farming harvest, world object drops, etc.)
/// and adds items to the player's inventory.
pub fn open_panel(
    mut commands: Commands,
    mut pickup_events: EventReader<PanelOpenedEvent>,
    mut inventory: ResMut<UiPanel>,
    item_registry: Res<UiRegistry>,
    mut sfx_events: EventWriter<UiChimeEvent>,
    mut toast_events: EventWriter<UiToastEvent>,
    player_query: Query<&PanelSlot, With<UiHost>>,
) {
    let player_pos = player_query.get_single().ok().map(|pos| pos.0);

    for ev in pickup_events.read() {
        let max_stack = item_registry
            .get(&ev.panel_id)
            .map(|def| def.stack_size.get())
            .unwrap_or(99);
        let remaining = inventory.try_add(&ev.panel_id, ev.quantity, max_stack);
        if remaining == 0 {
            sfx_events.send(UiChimeEvent {
                alert_id: "item_pickup".to_string(),
            });
            if let Some(world_pos) = player_pos {
                spawn_panel_indicator(&mut commands, world_pos);
            }
            info!("[UiHost] Picked up {} × '{}'", ev.quantity, ev.panel_id);
        } else {
            let name = item_registry
                .get(&ev.panel_id)
                .map(|d| d.name.as_str())
                .unwrap_or(&ev.panel_id);
            toast_events.send(UiToastEvent {
                message: format!("UiPanel full! Couldn't pick up {}.", name),
                duration_secs: 3.0,
            });
            info!(
                "[UiHost] UiPanel full — could not pick up {} × '{}' ({} dropped)",
                ev.quantity, ev.panel_id, remaining
            );
        }
    }
}



// ── plugin ────────────────────────────────────────────────────────

pub struct DomainPlugin;

impl Plugin for DomainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiPanel>();
        app.init_resource::<UiRegistry>();
        app.add_event::<PanelOpenedEvent>();
        app.add_event::<UiChimeEvent>();
        app.add_event::<UiToastEvent>();
        app.add_systems(Update, open_panel);
    }
}

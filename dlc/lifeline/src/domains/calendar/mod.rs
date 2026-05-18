//! Lifeline::domains::calendar — substrate mechanical fill v2.
//! Pure substitution port of hearthfield::add_items_to_inventory.
//! No LLM. parser_v3 + complete substitution map + cargo gate.

use bevy::prelude::*;

// ── components ────────────────────────────────────────────────────

#[derive(Component, Debug, Default, Clone)]
pub struct Scheduler;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct StationSlot(pub Vec2);

// ── resources ─────────────────────────────────────────────────────

#[derive(Resource, Debug, Default)]
pub struct ShiftRoster {
    pub entries: Vec<String>,
}

impl ShiftRoster {
    pub fn try_add(&mut self, id: &str, _qty: u8, _max: u8) -> u8 {
        self.entries.push(id.to_string());
        0
    }
    pub fn try_remove(&mut self, _id: &str, _qty: u8) -> u8 {
        if self.entries.pop().is_some() { 1 } else { 0 }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ShiftRegistryDef {
    pub name: String,
    pub stack_size: StackSize,
}
impl ShiftRegistryDef {
    pub fn name(&self) -> &str { &self.name }
    pub fn as_str(&self) -> &str { &self.name }
}

#[derive(Resource, Debug, Default)]
pub struct ShiftRegistry {
    pub defs: std::collections::HashMap<String, ShiftRegistryDef>,
}

impl ShiftRegistry {
    pub fn get(&self, id: &str) -> Option<&ShiftRegistryDef> {
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
pub struct ShiftStartedEvent {
    pub shift_id: String,
    pub quantity: u8,
}

#[derive(Event, Debug, Clone, Default)]
pub struct ShiftChangeAlertEvent {
    pub alert_id: String,
}

#[derive(Event, Debug, Clone, Default)]
pub struct CalendarToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

// ── helpers ───────────────────────────────────────────────────────

pub fn spawn_shift_marker(_commands: &mut Commands, _pos: Vec2) {}



// ── ported systems ────────────────────────────────────────────────

/// Reads ShiftStartedEvent (fired by farming harvest, world object drops, etc.)
/// and adds items to the player's inventory.
pub fn start_shift(
    mut commands: Commands,
    mut pickup_events: EventReader<ShiftStartedEvent>,
    mut inventory: ResMut<ShiftRoster>,
    item_registry: Res<ShiftRegistry>,
    mut sfx_events: EventWriter<ShiftChangeAlertEvent>,
    mut toast_events: EventWriter<CalendarToastEvent>,
    player_query: Query<&StationSlot, With<Scheduler>>,
) {
    let player_pos = player_query.get_single().ok().map(|pos| pos.0);

    for ev in pickup_events.read() {
        let max_stack = item_registry
            .get(&ev.shift_id)
            .map(|def| def.stack_size.get())
            .unwrap_or(99);
        let remaining = inventory.try_add(&ev.shift_id, ev.quantity, max_stack);
        if remaining == 0 {
            sfx_events.send(ShiftChangeAlertEvent {
                alert_id: "item_pickup".to_string(),
            });
            if let Some(world_pos) = player_pos {
                spawn_shift_marker(&mut commands, world_pos);
            }
            info!("[Scheduler] Picked up {} × '{}'", ev.quantity, ev.shift_id);
        } else {
            let name = item_registry
                .get(&ev.shift_id)
                .map(|d| d.name.as_str())
                .unwrap_or(&ev.shift_id);
            toast_events.send(CalendarToastEvent {
                message: format!("ShiftRoster full! Couldn't pick up {}.", name),
                duration_secs: 3.0,
            });
            info!(
                "[Scheduler] ShiftRoster full — could not pick up {} × '{}' ({} dropped)",
                ev.quantity, ev.shift_id, remaining
            );
        }
    }
}



// ── plugin ────────────────────────────────────────────────────────

pub struct DomainPlugin;

impl Plugin for DomainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShiftRoster>();
        app.init_resource::<ShiftRegistry>();
        app.add_event::<ShiftStartedEvent>();
        app.add_event::<ShiftChangeAlertEvent>();
        app.add_event::<CalendarToastEvent>();
        app.add_systems(Update, start_shift);
    }
}

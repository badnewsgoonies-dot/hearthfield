//! Lifeline::domains::npcs — substrate mechanical fill v2.
//! Pure substitution port of hearthfield::add_items_to_inventory.
//! No LLM. parser_v3 + complete substitution map + cargo gate.

use bevy::prelude::*;

// ── components ────────────────────────────────────────────────────

#[derive(Component, Debug, Default, Clone)]
pub struct Doctor;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct OnCallSlot(pub Vec2);

// ── resources ─────────────────────────────────────────────────────

#[derive(Resource, Debug, Default)]
pub struct StaffRoster {
    pub entries: Vec<String>,
}

impl StaffRoster {
    pub fn try_add(&mut self, id: &str, _qty: u8, _max: u8) -> u8 {
        self.entries.push(id.to_string());
        0
    }
    pub fn try_remove(&mut self, _id: &str, _qty: u8) -> u8 {
        if self.entries.pop().is_some() { 1 } else { 0 }
    }
}

#[derive(Debug, Clone, Default)]
pub struct StaffRegistryDef {
    pub name: String,
    pub stack_size: StackSize,
}
impl StaffRegistryDef {
    pub fn name(&self) -> &str { &self.name }
    pub fn as_str(&self) -> &str { &self.name }
}

#[derive(Resource, Debug, Default)]
pub struct StaffRegistry {
    pub defs: std::collections::HashMap<String, StaffRegistryDef>,
}

impl StaffRegistry {
    pub fn get(&self, id: &str) -> Option<&StaffRegistryDef> {
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
pub struct StaffArrivedEvent {
    pub staff_id: String,
    pub quantity: u8,
}

#[derive(Event, Debug, Clone, Default)]
pub struct PagerEvent {
    pub alert_id: String,
}

#[derive(Event, Debug, Clone, Default)]
pub struct NpcsToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

// ── helpers ───────────────────────────────────────────────────────

pub fn spawn_arrival_marker(_commands: &mut Commands, _pos: Vec2) {}



// ── ported systems ────────────────────────────────────────────────

/// Reads StaffArrivedEvent (fired by farming harvest, world object drops, etc.)
/// and adds items to the player's inventory.
pub fn admit_staff(
    mut commands: Commands,
    mut pickup_events: EventReader<StaffArrivedEvent>,
    mut inventory: ResMut<StaffRoster>,
    item_registry: Res<StaffRegistry>,
    mut sfx_events: EventWriter<PagerEvent>,
    mut toast_events: EventWriter<NpcsToastEvent>,
    player_query: Query<&OnCallSlot, With<Doctor>>,
) {
    let player_pos = player_query.get_single().ok().map(|pos| pos.0);

    for ev in pickup_events.read() {
        let max_stack = item_registry
            .get(&ev.staff_id)
            .map(|def| def.stack_size.get())
            .unwrap_or(99);
        let remaining = inventory.try_add(&ev.staff_id, ev.quantity, max_stack);
        if remaining == 0 {
            sfx_events.send(PagerEvent {
                alert_id: "item_pickup".to_string(),
            });
            if let Some(world_pos) = player_pos {
                spawn_arrival_marker(&mut commands, world_pos);
            }
            info!("[Doctor] Picked up {} × '{}'", ev.quantity, ev.staff_id);
        } else {
            let name = item_registry
                .get(&ev.staff_id)
                .map(|d| d.name.as_str())
                .unwrap_or(&ev.staff_id);
            toast_events.send(NpcsToastEvent {
                message: format!("StaffRoster full! Couldn't pick up {}.", name),
                duration_secs: 3.0,
            });
            info!(
                "[Doctor] StaffRoster full — could not pick up {} × '{}' ({} dropped)",
                ev.quantity, ev.staff_id, remaining
            );
        }
    }
}



// ── plugin ────────────────────────────────────────────────────────

pub struct DomainPlugin;

impl Plugin for DomainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<StaffRoster>();
        app.init_resource::<StaffRegistry>();
        app.add_event::<StaffArrivedEvent>();
        app.add_event::<PagerEvent>();
        app.add_event::<NpcsToastEvent>();
        app.add_systems(Update, admit_staff);
    }
}

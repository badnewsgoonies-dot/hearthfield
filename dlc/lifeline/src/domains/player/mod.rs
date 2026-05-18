//! Lifeline::domains::player — substrate mechanical fill v2.
//! Pure substitution port of hearthfield::add_items_to_inventory.
//! No LLM. parser_v3 + complete substitution map + cargo gate.

use bevy::prelude::*;

// ── components ────────────────────────────────────────────────────

#[derive(Component, Debug, Default, Clone)]
pub struct Resident;

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct WardPosition(pub Vec2);

// ── resources ─────────────────────────────────────────────────────

#[derive(Resource, Debug, Default)]
pub struct PocketKit {
    pub entries: Vec<String>,
}

impl PocketKit {
    pub fn try_add(&mut self, id: &str, _qty: u8, _max: u8) -> u8 {
        self.entries.push(id.to_string());
        0
    }
    pub fn try_remove(&mut self, _id: &str, _qty: u8) -> u8 {
        if self.entries.pop().is_some() { 1 } else { 0 }
    }
}

#[derive(Debug, Clone, Default)]
pub struct KitRegistryDef {
    pub name: String,
    pub stack_size: StackSize,
}
impl KitRegistryDef {
    pub fn name(&self) -> &str { &self.name }
    pub fn as_str(&self) -> &str { &self.name }
}

#[derive(Resource, Debug, Default)]
pub struct KitRegistry {
    pub defs: std::collections::HashMap<String, KitRegistryDef>,
}

impl KitRegistry {
    pub fn get(&self, id: &str) -> Option<&KitRegistryDef> {
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
pub struct KitItemAddedEvent {
    pub kit_item_id: String,
    pub quantity: u8,
}

#[derive(Event, Debug, Clone, Default)]
pub struct PageEvent {
    pub alert_id: String,
}

#[derive(Event, Debug, Clone, Default)]
pub struct PlayerToastEvent {
    pub message: String,
    pub duration_secs: f32,
}

// ── helpers ───────────────────────────────────────────────────────

pub fn spawn_pocket_glint(_commands: &mut Commands, _pos: Vec2) {}



// ── ported systems ────────────────────────────────────────────────

/// Reads KitItemAddedEvent (fired by farming harvest, world object drops, etc.)
/// and adds items to the player's inventory.
pub fn add_to_kit(
    mut commands: Commands,
    mut pickup_events: EventReader<KitItemAddedEvent>,
    mut inventory: ResMut<PocketKit>,
    item_registry: Res<KitRegistry>,
    mut sfx_events: EventWriter<PageEvent>,
    mut toast_events: EventWriter<PlayerToastEvent>,
    player_query: Query<&WardPosition, With<Resident>>,
) {
    let player_pos = player_query.get_single().ok().map(|pos| pos.0);

    for ev in pickup_events.read() {
        let max_stack = item_registry
            .get(&ev.kit_item_id)
            .map(|def| def.stack_size.get())
            .unwrap_or(99);
        let remaining = inventory.try_add(&ev.kit_item_id, ev.quantity, max_stack);
        if remaining == 0 {
            sfx_events.send(PageEvent {
                alert_id: "item_pickup".to_string(),
            });
            if let Some(world_pos) = player_pos {
                spawn_pocket_glint(&mut commands, world_pos);
            }
            info!("[Resident] Picked up {} × '{}'", ev.quantity, ev.kit_item_id);
        } else {
            let name = item_registry
                .get(&ev.kit_item_id)
                .map(|d| d.name.as_str())
                .unwrap_or(&ev.kit_item_id);
            toast_events.send(PlayerToastEvent {
                message: format!("PocketKit full! Couldn't pick up {}.", name),
                duration_secs: 3.0,
            });
            info!(
                "[Resident] PocketKit full — could not pick up {} × '{}' ({} dropped)",
                ev.quantity, ev.kit_item_id, remaining
            );
        }
    }
}



// ── plugin ────────────────────────────────────────────────────────

pub struct DomainPlugin;

impl Plugin for DomainPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PocketKit>();
        app.init_resource::<KitRegistry>();
        app.add_event::<KitItemAddedEvent>();
        app.add_event::<PageEvent>();
        app.add_event::<PlayerToastEvent>();
        app.add_systems(Update, add_to_kit);
    }
}

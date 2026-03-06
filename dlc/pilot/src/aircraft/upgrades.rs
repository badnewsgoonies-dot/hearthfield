//! Aircraft upgrade system — engine, avionics, interior, and more.

use crate::shared::*;
use bevy::prelude::*;

pub struct UpgradePlugin;

impl Plugin for UpgradePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UpgradeRegistry>()
            .add_event::<UpgradeEvent>()
            .add_systems(
                Update,
                (apply_upgrade, check_upgrade_unlocks).run_if(in_state(GameState::Playing)),
            );
    }
}

// ── Types ────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UpgradeSlot {
    Engine,
    Avionics,
    Interior,
    FuelSystem,
    Landing,
    Navigation,
    Safety,
    Paint,
}

impl UpgradeSlot {
    pub fn display_name(&self) -> &'static str {
        match self {
            UpgradeSlot::Engine => "Engine",
            UpgradeSlot::Avionics => "Avionics",
            UpgradeSlot::Interior => "Interior",
            UpgradeSlot::FuelSystem => "Fuel System",
            UpgradeSlot::Landing => "Landing Gear",
            UpgradeSlot::Navigation => "Navigation",
            UpgradeSlot::Safety => "Safety Equipment",
            UpgradeSlot::Paint => "Paint Job",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UpgradeTier {
    Basic,
    Improved,
    Advanced,
    Premium,
}

impl UpgradeTier {
    pub fn display_name(&self) -> &'static str {
        match self {
            UpgradeTier::Basic => "Basic",
            UpgradeTier::Improved => "Improved",
            UpgradeTier::Advanced => "Advanced",
            UpgradeTier::Premium => "Premium",
        }
    }

    pub fn cost_multiplier(&self) -> u32 {
        match self {
            UpgradeTier::Basic => 1,
            UpgradeTier::Improved => 3,
            UpgradeTier::Advanced => 7,
            UpgradeTier::Premium => 15,
        }
    }

    pub fn next(&self) -> Option<UpgradeTier> {
        match self {
            UpgradeTier::Basic => Some(UpgradeTier::Improved),
            UpgradeTier::Improved => Some(UpgradeTier::Advanced),
            UpgradeTier::Advanced => Some(UpgradeTier::Premium),
            UpgradeTier::Premium => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct UpgradeDef {
    pub slot: UpgradeSlot,
    pub tier: UpgradeTier,
    pub base_cost: u32,
    pub speed_bonus: f32,
    pub fuel_efficiency: f32,
    pub comfort_bonus: f32,
    pub durability_bonus: f32,
    pub unlocks_capability: Option<&'static str>,
}

impl UpgradeDef {
    pub fn total_cost(&self) -> u32 {
        self.base_cost * self.tier.cost_multiplier()
    }
}

// ── Events ───────────────────────────────────────────────────────────────

#[derive(Event, Clone, Debug)]
pub struct UpgradeEvent {
    pub aircraft_index: usize,
    pub slot: UpgradeSlot,
    pub tier: UpgradeTier,
}

// ── State ────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct UpgradeRegistry {
    pub installed: Vec<InstalledUpgrade>,
}

#[derive(Clone, Debug)]
pub struct InstalledUpgrade {
    pub aircraft_index: usize,
    pub slot: UpgradeSlot,
    pub tier: UpgradeTier,
}

impl UpgradeRegistry {
    pub fn current_tier(&self, aircraft_index: usize, slot: UpgradeSlot) -> Option<UpgradeTier> {
        self.installed
            .iter()
            .filter(|u| u.aircraft_index == aircraft_index && u.slot == slot)
            .map(|u| u.tier)
            .max()
    }

    pub fn has_capability(&self, aircraft_index: usize, capability: &str) -> bool {
        self.installed.iter().any(|u| {
            u.aircraft_index == aircraft_index
                && upgrade_capability(u.slot, u.tier) == Some(capability)
        })
    }
}

fn upgrade_capability(slot: UpgradeSlot, tier: UpgradeTier) -> Option<&'static str> {
    match (slot, tier) {
        (UpgradeSlot::Avionics, UpgradeTier::Advanced) => Some("autopilot"),
        (UpgradeSlot::Avionics, UpgradeTier::Premium) => Some("weather_radar"),
        (UpgradeSlot::Safety, UpgradeTier::Improved) => Some("de_icing"),
        (UpgradeSlot::Safety, UpgradeTier::Advanced) => Some("fire_suppression"),
        (UpgradeSlot::Navigation, UpgradeTier::Advanced) => Some("gps_precision"),
        (UpgradeSlot::Navigation, UpgradeTier::Premium) => Some("terrain_warning"),
        _ => None,
    }
}

fn upgrade_stats(slot: UpgradeSlot, tier: UpgradeTier) -> UpgradeDef {
    let multiplier = tier.cost_multiplier() as f32;
    match slot {
        UpgradeSlot::Engine => UpgradeDef {
            slot,
            tier,
            base_cost: 500,
            speed_bonus: 10.0 * multiplier,
            fuel_efficiency: 0.02 * multiplier,
            comfort_bonus: 0.0,
            durability_bonus: 0.0,
            unlocks_capability: None,
        },
        UpgradeSlot::FuelSystem => UpgradeDef {
            slot,
            tier,
            base_cost: 400,
            speed_bonus: 0.0,
            fuel_efficiency: 0.05 * multiplier,
            comfort_bonus: 0.0,
            durability_bonus: 0.0,
            unlocks_capability: None,
        },
        UpgradeSlot::Interior => UpgradeDef {
            slot,
            tier,
            base_cost: 300,
            speed_bonus: 0.0,
            fuel_efficiency: 0.0,
            comfort_bonus: 5.0 * multiplier,
            durability_bonus: 0.0,
            unlocks_capability: None,
        },
        UpgradeSlot::Landing => UpgradeDef {
            slot,
            tier,
            base_cost: 350,
            speed_bonus: 0.0,
            fuel_efficiency: 0.0,
            comfort_bonus: 2.0 * multiplier,
            durability_bonus: 5.0 * multiplier,
            unlocks_capability: None,
        },
        _ => UpgradeDef {
            slot,
            tier,
            base_cost: 400,
            speed_bonus: 0.0,
            fuel_efficiency: 0.0,
            comfort_bonus: 0.0,
            durability_bonus: 0.0,
            unlocks_capability: upgrade_capability(slot, tier),
        },
    }
}

// ── Systems ──────────────────────────────────────────────────────────────

pub fn apply_upgrade(
    mut events: EventReader<UpgradeEvent>,
    mut gold: ResMut<Gold>,
    _fleet: ResMut<Fleet>,
    mut registry: ResMut<UpgradeRegistry>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for ev in events.read() {
        let def = upgrade_stats(ev.slot, ev.tier);
        let cost = def.total_cost();

        if gold.amount < cost {
            toast_events.send(ToastEvent {
                message: format!(
                    "Not enough gold! Need {} for {} {}",
                    cost,
                    ev.tier.display_name(),
                    ev.slot.display_name()
                ),
                duration_secs: 3.0,
            });
            continue;
        }

        // Check tier ordering
        if let Some(current) = registry.current_tier(ev.aircraft_index, ev.slot) {
            if ev.tier <= current {
                toast_events.send(ToastEvent {
                    message: "Already have this tier or better.".to_string(),
                    duration_secs: 2.0,
                });
                continue;
            }
        }

        gold.amount -= cost;

        registry.installed.push(InstalledUpgrade {
            aircraft_index: ev.aircraft_index,
            slot: ev.slot,
            tier: ev.tier,
        });

        toast_events.send(ToastEvent {
            message: format!(
                "Installed {} {} (-{} gold)",
                ev.tier.display_name(),
                ev.slot.display_name(),
                cost
            ),
            duration_secs: 4.0,
        });

        if let Some(cap) = def.unlocks_capability {
            toast_events.send(ToastEvent {
                message: format!("🔓 Unlocked: {cap}"),
                duration_secs: 3.0,
            });
        }
    }
}

pub fn check_upgrade_unlocks(
    fleet: Res<Fleet>,
    registry: Res<UpgradeRegistry>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    // Periodic hint about available upgrades
    let _ = (&fleet, &registry, &mut toast_events);
}

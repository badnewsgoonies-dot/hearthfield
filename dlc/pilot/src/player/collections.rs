//! Collectible items — souvenirs, postcards, model planes, stamps, patches, photos.

use crate::shared::*;
use bevy::prelude::*;

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CollectionCategory {
    Souvenir,
    Postcard,
    ModelPlane,
    Stamp,
    Patch,
    Photo,
}

impl CollectionCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            CollectionCategory::Souvenir => "Souvenirs",
            CollectionCategory::Postcard => "Postcards",
            CollectionCategory::ModelPlane => "Model Planes",
            CollectionCategory::Stamp => "Stamps",
            CollectionCategory::Patch => "Patches",
            CollectionCategory::Photo => "Photos",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            CollectionCategory::Souvenir => "🎁",
            CollectionCategory::Postcard => "💌",
            CollectionCategory::ModelPlane => "✈",
            CollectionCategory::Stamp => "📮",
            CollectionCategory::Patch => "🏷",
            CollectionCategory::Photo => "📷",
        }
    }

    pub fn all() -> &'static [CollectionCategory] {
        &[
            CollectionCategory::Souvenir,
            CollectionCategory::Postcard,
            CollectionCategory::ModelPlane,
            CollectionCategory::Stamp,
            CollectionCategory::Patch,
            CollectionCategory::Photo,
        ]
    }

    pub fn completion_reward_gold(&self) -> u32 {
        match self {
            CollectionCategory::Souvenir => 2000,
            CollectionCategory::Postcard => 1000,
            CollectionCategory::ModelPlane => 3000,
            CollectionCategory::Stamp => 800,
            CollectionCategory::Patch => 1500,
            CollectionCategory::Photo => 1200,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Collectible {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: CollectionCategory,
    pub airport: AirportId,
    pub rarity: CollectibleRarity,
    pub price: u32,
    pub hidden: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CollectibleRarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl CollectibleRarity {
    pub fn display_name(&self) -> &'static str {
        match self {
            CollectibleRarity::Common => "Common",
            CollectibleRarity::Uncommon => "Uncommon",
            CollectibleRarity::Rare => "Rare",
            CollectibleRarity::Legendary => "Legendary",
        }
    }
}

/// Player's collection state.
#[derive(Resource, Clone, Debug, Default)]
pub struct Collections {
    pub collected: Vec<String>, // collectible IDs
    pub completed_categories: Vec<CollectionCategory>,
    pub trade_offers: Vec<TradeOffer>,
}

impl Collections {
    pub fn has(&self, id: &str) -> bool {
        self.collected.iter().any(|c| c == id)
    }

    pub fn add(&mut self, id: &str) -> bool {
        if self.has(id) {
            return false;
        }
        self.collected.push(id.to_string());
        true
    }

    pub fn count_in_category(
        &self,
        category: CollectionCategory,
        all: &[Collectible],
    ) -> (usize, usize) {
        let total = all.iter().filter(|c| c.category == category).count();
        let found = all
            .iter()
            .filter(|c| c.category == category && self.has(&c.id))
            .count();
        (found, total)
    }

    pub fn is_category_complete(&self, category: CollectionCategory, all: &[Collectible]) -> bool {
        let (found, total) = self.count_in_category(category, all);
        total > 0 && found >= total
    }
}

#[derive(Clone, Debug)]
pub struct TradeOffer {
    pub from_npc: String,
    pub offered_item: String,
    pub wanted_item: String,
    pub accepted: bool,
}

/// Get all collectible definitions.
pub fn all_collectibles() -> Vec<Collectible> {
    let mut items = Vec::new();

    let airports = [
        (AirportId::HomeBase, "Clearfield", "small aviation town"),
        (AirportId::Windport, "Windport", "coastal city"),
        (AirportId::Frostpeak, "Frostpeak", "mountain station"),
        (AirportId::Sunhaven, "Sunhaven", "tropical resort"),
        (AirportId::Ironforge, "Ironforge", "industrial hub"),
        (AirportId::Cloudmere, "Cloudmere", "high-altitude"),
        (AirportId::Duskhollow, "Duskhollow", "desert oasis"),
        (AirportId::Stormwatch, "Stormwatch", "research station"),
        (AirportId::Grandcity, "Grand City", "international hub"),
        (AirportId::Skyreach, "Skyreach", "elite airport"),
    ];

    for (airport, name, desc) in airports {
        // Each airport has 6 collectibles (one per category)
        items.push(Collectible {
            id: format!("souvenir_{}", name.to_lowercase().replace(' ', "_")),
            name: format!("{} Souvenir", name),
            description: format!("A keepsake from the {} {}.", desc, name),
            category: CollectionCategory::Souvenir,
            airport,
            rarity: CollectibleRarity::Common,
            price: 50,
            hidden: false,
        });
        items.push(Collectible {
            id: format!("postcard_{}", name.to_lowercase().replace(' ', "_")),
            name: format!("{} Postcard", name),
            description: format!("A scenic postcard from {}.", name),
            category: CollectionCategory::Postcard,
            airport,
            rarity: CollectibleRarity::Common,
            price: 20,
            hidden: false,
        });
        items.push(Collectible {
            id: format!("model_{}", name.to_lowercase().replace(' ', "_")),
            name: format!("{} Model Plane", name),
            description: format!("A detailed model of an aircraft based at {}.", name),
            category: CollectionCategory::ModelPlane,
            airport,
            rarity: CollectibleRarity::Uncommon,
            price: 150,
            hidden: false,
        });
        items.push(Collectible {
            id: format!("stamp_{}", name.to_lowercase().replace(' ', "_")),
            name: format!("{} Airport Stamp", name),
            description: format!("Official passport stamp from {} airport.", name),
            category: CollectionCategory::Stamp,
            airport,
            rarity: CollectibleRarity::Common,
            price: 0, // Free on visit
            hidden: false,
        });
        items.push(Collectible {
            id: format!("patch_{}", name.to_lowercase().replace(' ', "_")),
            name: format!("{} Crew Patch", name),
            description: format!("An embroidered crew patch from {}.", name),
            category: CollectionCategory::Patch,
            airport,
            rarity: CollectibleRarity::Rare,
            price: 200,
            hidden: false,
        });
        items.push(Collectible {
            id: format!("photo_{}", name.to_lowercase().replace(' ', "_")),
            name: format!("{} Aerial Photo", name),
            description: format!("A stunning aerial photograph taken near {}.", name),
            category: CollectionCategory::Photo,
            airport,
            rarity: CollectibleRarity::Uncommon,
            price: 100,
            hidden: true, // Hidden collectible — found in city exploration
        });
    }

    items
}

// ═══════════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════════

/// Auto-collect stamps on first airport visit.
pub fn auto_collect_stamp(
    mut arrival_events: EventReader<AirportArrivalEvent>,
    mut collections: ResMut<Collections>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    let all_items = all_collectibles();

    for ev in arrival_events.read() {
        let stamp_id = all_items
            .iter()
            .find(|c| c.airport == ev.airport && c.category == CollectionCategory::Stamp)
            .map(|c| c.id.clone());

        if let Some(id) = stamp_id {
            if collections.add(&id) {
                toast_events.send(ToastEvent {
                    message: format!("📮 Collected: {} Airport Stamp!", ev.airport.display_name()),
                    duration_secs: 3.0,
                });
            }
        }
    }
}

/// Purchase collectibles from shops.
pub fn purchase_collectible(
    mut purchase_events: EventReader<PurchaseEvent>,
    mut collections: ResMut<Collections>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    let all_items = all_collectibles();

    for ev in purchase_events.read() {
        if let Some(item) = all_items.iter().find(|c| c.id == ev.item_id) {
            if collections.add(&item.id) {
                toast_events.send(ToastEvent {
                    message: format!(
                        "{} Collected: {} ({})",
                        item.category.icon(),
                        item.name,
                        item.rarity.display_name()
                    ),
                    duration_secs: 3.0,
                });
            }
        }
    }
}

/// Check for category completion and award bonuses.
pub fn check_collection_completion(
    mut collections: ResMut<Collections>,
    mut gold_events: EventWriter<GoldChangeEvent>,
    mut achievement_events: EventWriter<AchievementUnlockedEvent>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    let all_items = all_collectibles();

    for category in CollectionCategory::all() {
        if collections.completed_categories.contains(category) {
            continue;
        }

        if collections.is_category_complete(*category, &all_items) {
            collections.completed_categories.push(*category);

            let reward = category.completion_reward_gold();
            gold_events.send(GoldChangeEvent {
                amount: reward as i32,
                reason: format!("{} collection complete!", category.display_name()),
            });

            toast_events.send(ToastEvent {
                message: format!(
                    "🎉 {} collection complete! +{}g bonus!",
                    category.display_name(),
                    reward
                ),
                duration_secs: 5.0,
            });
        }
    }

    // All collections complete = special achievement
    if collections.completed_categories.len() == CollectionCategory::all().len()
        && !collections.completed_categories.is_empty()
    {
        achievement_events.send(AchievementUnlockedEvent {
            achievement_id: "master_collector".to_string(),
        });
    }
}

/// Generate trade offers from crew members for rare items.
pub fn generate_trade_offers(
    mut day_end_events: EventReader<DayEndEvent>,
    relationships: Res<Relationships>,
    collections: Res<Collections>,
    mut toast_events: EventWriter<ToastEvent>,
) {
    for _ev in day_end_events.read() {
        // High-friendship crew occasionally offer trades
        for npc_id in CREW_IDS {
            let friendship = relationships.friendship_level(npc_id);
            if friendship < 40 {
                continue;
            }

            // Simple trade opportunity notification
            if !collections.has(&format!(
                "patch_{}",
                npc_id.split('_').next_back().unwrap_or("unknown")
            )) {
                toast_events.send(ToastEvent {
                    message: format!("💬 {} might have a rare collectible to trade...", npc_id),
                    duration_secs: 3.0,
                });
                break; // One offer per day
            }
        }
    }
}

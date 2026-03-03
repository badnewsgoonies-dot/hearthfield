//! Shop system — purchasing, selling, bulk discounts, restock, friendship pricing.

use bevy::prelude::*;
use crate::shared::*;

const BULK_THRESHOLD: u32 = 5;
const BULK_DISCOUNT: f32 = 0.10;

/// Event for selling items back to the shop.
#[derive(Event, Clone, Debug)]
pub struct SellItemEvent {
    pub item_id: String,
    pub quantity: u32,
}

#[derive(Resource, Default)]
pub struct ShopRestockTimer {
    pub last_restock_day: u32,
}

fn airport_premium_items(airport: AirportId) -> Vec<(&'static str, u32)> {
    match airport {
        AirportId::HomeBase => vec![("pilot_manual", 150), ("local_map", 50)],
        AirportId::Windport => vec![("sea_chart", 200), ("waterproof_jacket", 120)],
        AirportId::Frostpeak => vec![("thermal_gloves", 80), ("de_icer_spray", 100), ("snow_goggles", 60)],
        AirportId::Sunhaven => vec![("tropical_snack", 30), ("sunscreen", 25), ("surfboard_model", 300)],
        AirportId::Ironforge => vec![("titanium_wrench", 250), ("spare_rivets", 40)],
        AirportId::Cloudmere => vec![("oxygen_mask", 180), ("altimeter_charm", 150)],
        AirportId::Duskhollow => vec![("desert_canteen", 60), ("sand_filter", 90)],
        AirportId::Stormwatch => vec![("weather_almanac", 200), ("lightning_rod_pin", 120)],
        AirportId::Grandcity => vec![("luxury_headset", 500), ("first_class_coffee", 50), ("gold_wings_pin", 400)],
        AirportId::Skyreach => vec![("ace_flight_suit", 1000), ("legendary_compass", 800)],
    }
}

fn friendship_adjusted_price(base_price: u32, friendship: i32) -> u32 {
    let discount_pct = (friendship.max(0) as f32 / 100.0) * 0.15;
    let discounted = base_price as f32 * (1.0 - discount_pct);
    discounted.round() as u32
}

fn _bulk_adjusted_price(unit_price: u32, quantity: u32) -> u32 {
    if quantity >= BULK_THRESHOLD {
        let total = unit_price as f32 * quantity as f32;
        (total * (1.0 - BULK_DISCOUNT)).round() as u32
    } else {
        unit_price * quantity
    }
}

pub fn is_airport_exclusive(airport: AirportId, item_id: &str) -> bool {
    airport_premium_items(airport).iter().any(|(id, _)| *id == item_id)
}

pub fn handle_purchase(
    mut events: EventReader<PurchaseEvent>,
    mut gold: ResMut<Gold>,
    mut inventory: ResMut<Inventory>,
    mut economy_stats: ResMut<EconomyStats>,
    mut toast_events: EventWriter<ToastEvent>,
    item_registry: Res<ItemRegistry>,
    relationships: Res<Relationships>,
    active_shop: Res<ActiveShop>,
) {
    let shopkeeper_id = if !active_shop.name.is_empty() {
        shop_keeper_for(&active_shop.name)
    } else {
        None
    };
    let friendship = shopkeeper_id
        .map(|id| relationships.friendship_level(id))
        .unwrap_or(0);

    for ev in events.read() {
        let adjusted_price = friendship_adjusted_price(ev.price, friendship);

        if gold.amount < adjusted_price {
            toast_events.send(ToastEvent {
                message: format!("Not enough gold! Need {}g", adjusted_price),
                duration_secs: 2.0,
            });
            continue;
        }

        if !inventory.add_item(&ev.item_id, 1) {
            toast_events.send(ToastEvent {
                message: "Inventory full!".to_string(),
                duration_secs: 2.0,
            });
            continue;
        }

        gold.amount -= adjusted_price;
        economy_stats.total_spent += adjusted_price;
        economy_stats.items_purchased += 1;

        let name = item_registry.get(&ev.item_id).map_or("Item", |d| d.name.as_str());
        let savings = ev.price.saturating_sub(adjusted_price);
        let msg = if savings > 0 {
            format!("Purchased {} for {}g (saved {}g!)", name, adjusted_price, savings)
        } else {
            format!("Purchased {} for {}g", name, adjusted_price)
        };
        toast_events.send(ToastEvent { message: msg, duration_secs: 2.5 });
    }
}

pub fn handle_sell(
    mut events: EventReader<SellItemEvent>,
    mut gold: ResMut<Gold>,
    mut inventory: ResMut<Inventory>,
    mut economy_stats: ResMut<EconomyStats>,
    mut toast_events: EventWriter<ToastEvent>,
    item_registry: Res<ItemRegistry>,
) {
    for ev in events.read() {
        let sell_price = item_registry.get(&ev.item_id).map_or(1, |d| d.sell_price);
        let total_price = sell_price * ev.quantity;

        if !inventory.remove_item(&ev.item_id, ev.quantity) {
            toast_events.send(ToastEvent {
                message: "You don't have enough of that item!".to_string(),
                duration_secs: 2.0,
            });
            continue;
        }

        gold.amount += total_price;
        economy_stats.total_earned += total_price;

        let name = item_registry.get(&ev.item_id).map_or("Item", |d| d.name.as_str());
        toast_events.send(ToastEvent {
            message: format!("Sold {} x{} for {}g", name, ev.quantity, total_price),
            duration_secs: 2.0,
        });
    }
}

pub fn restock_shop(
    mut day_end_events: EventReader<DayEndEvent>,
    mut active_shop: ResMut<ActiveShop>,
    mut restock_timer: ResMut<ShopRestockTimer>,
    calendar: Res<Calendar>,
    player_location: Res<PlayerLocation>,
) {
    for _ev in day_end_events.read() {
        let day = calendar.total_days();
        if day <= restock_timer.last_restock_day { continue; }
        restock_timer.last_restock_day = day;

        for listing in active_shop.listings.iter_mut() {
            if let Some(stock) = &mut listing.stock {
                let base_stock = 3 + ((day * 7 + listing.price) % 5) as u32;
                *stock = base_stock;
            }
        }

        let premiums = airport_premium_items(player_location.airport);
        for (item_id, price) in premiums {
            if !active_shop.listings.iter().any(|l| l.item_id == item_id) {
                active_shop.listings.push(ShopListing {
                    item_id: item_id.to_string(),
                    price,
                    stock: Some(1),
                });
            }
        }
    }
}

fn shop_keeper_for(shop_name: &str) -> Option<&'static str> {
    match shop_name {
        "Clearfield Shop" | "Home Base Shop" => Some("attendant_sofia"),
        "Windport Trading" => Some("charter_diana"),
        _ => None,
    }
}

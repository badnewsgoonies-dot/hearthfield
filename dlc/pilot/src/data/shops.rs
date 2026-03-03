use crate::shared::*;

pub fn get_shop_inventory(airport: AirportId) -> Vec<ShopListing> {
    let mut listings = common_items();

    let local = match airport {
        AirportId::HomeBase => vec![
            ShopListing { item_id: "pilots_manual".into(), price: 50, stock: Some(1) },
            ShopListing { item_id: "navigation_map".into(), price: 30, stock: Some(3) },
            ShopListing { item_id: "license_frame".into(), price: 20, stock: Some(2) },
        ],
        AirportId::Windport => vec![
            ShopListing { item_id: "postcard_windport".into(), price: 10, stock: None },
            ShopListing { item_id: "scarf".into(), price: 65, stock: Some(2) },
            ShopListing { item_id: "compass".into(), price: 90, stock: Some(1) },
        ],
        AirportId::Frostpeak => vec![
            ShopListing { item_id: "snow_globe_frostpeak".into(), price: 45, stock: Some(3) },
            ShopListing { item_id: "weather_chart".into(), price: 25, stock: Some(5) },
            ShopListing { item_id: "ramen".into(), price: 45, stock: None },
        ],
        AirportId::Sunhaven => vec![
            ShopListing { item_id: "postcard_sunhaven".into(), price: 10, stock: None },
            ShopListing { item_id: "sunglasses".into(), price: 80, stock: Some(2) },
            ShopListing { item_id: "sushi".into(), price: 60, stock: None },
            ShopListing { item_id: "wine".into(), price: 150, stock: Some(3) },
        ],
        AirportId::Ironforge => vec![
            ShopListing { item_id: "keychain_ironforge".into(), price: 15, stock: None },
            ShopListing { item_id: "spark_plug".into(), price: 100, stock: Some(10) },
            ShopListing { item_id: "oil_filter".into(), price: 80, stock: Some(10) },
            ShopListing { item_id: "brake_pad".into(), price: 180, stock: Some(4) },
            ShopListing { item_id: "tire".into(), price: 200, stock: Some(4) },
        ],
        AirportId::Cloudmere => vec![
            ShopListing { item_id: "book".into(), price: 35, stock: Some(3) },
            ShopListing { item_id: "altimeter".into(), price: 350, stock: Some(1) },
            ShopListing { item_id: "nav_light".into(), price: 120, stock: Some(4) },
        ],
        AirportId::Duskhollow => vec![
            ShopListing { item_id: "flowers".into(), price: 40, stock: Some(5) },
            ShopListing { item_id: "teddy_bear".into(), price: 30, stock: Some(3) },
            ShopListing { item_id: "model_plane".into(), price: 120, stock: Some(1) },
        ],
        AirportId::Stormwatch => vec![
            ShopListing { item_id: "weather_chart".into(), price: 20, stock: None },
            ShopListing { item_id: "radio_unit".into(), price: 400, stock: Some(1) },
            ShopListing { item_id: "windshield".into(), price: 600, stock: Some(1) },
        ],
        AirportId::Grandcity => vec![
            ShopListing { item_id: "snow_globe_grandcity".into(), price: 55, stock: Some(3) },
            ShopListing { item_id: "chocolate".into(), price: 50, stock: None },
            ShopListing { item_id: "watch".into(), price: 200, stock: Some(2) },
            ShopListing { item_id: "headset".into(), price: 250, stock: Some(1) },
            ShopListing { item_id: "propeller".into(), price: 500, stock: Some(2) },
        ],
        AirportId::Skyreach => vec![
            ShopListing { item_id: "magnet_skyreach".into(), price: 12, stock: None },
            ShopListing { item_id: "watch".into(), price: 180, stock: Some(2) },
            ShopListing { item_id: "headset".into(), price: 230, stock: Some(2) },
            ShopListing { item_id: "wine".into(), price: 140, stock: Some(5) },
        ],
    };

    listings.extend(local);
    listings
}

fn common_items() -> Vec<ShopListing> {
    vec![
        ShopListing { item_id: "coffee".into(), price: 15, stock: None },
        ShopListing { item_id: "sandwich".into(), price: 30, stock: None },
        ShopListing { item_id: "energy_bar".into(), price: 20, stock: None },
        ShopListing { item_id: "water_bottle".into(), price: 5, stock: None },
        ShopListing { item_id: "juice".into(), price: 12, stock: None },
        ShopListing { item_id: "donut".into(), price: 10, stock: None },
    ]
}

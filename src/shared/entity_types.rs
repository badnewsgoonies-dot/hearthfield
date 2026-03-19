use ironclad::game_entity;

#[game_entity(requires = [id, name, description, category, sell_price, stack_size, sprite_index])]
pub struct ItemDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: super::ItemCategory,
    pub sell_price: u32,
    pub buy_price: Option<u32>,
    pub stack_size: super::StackSize,
    pub edible: bool,
    pub energy_restore: f32,
    pub sprite_index: u32,
}

#[game_entity(requires = [id, name, seed_id, harvest_id, seasons, growth_days, sell_price, sprite_stages])]
pub struct CropDef {
    pub id: String,
    pub name: String,
    pub seed_id: String,
    pub harvest_id: String,
    pub seasons: Vec<super::Season>,
    pub growth_days: Vec<u8>,
    pub regrows: bool,
    pub regrow_days: u8,
    pub sell_price: u32,
    pub sprite_stages: Vec<u32>,
}

#[game_entity(requires = [id, name, birthday_season, birthday_day, default_dialogue, sprite_index, portrait_index])]
pub struct NpcDef {
    pub id: String,
    pub name: String,
    pub birthday_season: super::Season,
    pub birthday_day: u8,
    pub gift_preferences: std::collections::HashMap<String, super::GiftPreference>,
    pub default_dialogue: Vec<String>,
    pub heart_dialogue: std::collections::HashMap<u8, Vec<String>>,
    pub is_marriageable: bool,
    pub sprite_index: u32,
    pub portrait_index: u32,
}

#[game_entity(requires = [id, name, ingredients, result, result_quantity])]
pub struct Recipe {
    pub id: String,
    pub name: String,
    pub ingredients: Vec<(String, u8)>,
    pub result: String,
    pub result_quantity: u8,
    pub is_cooking: bool,
    pub unlocked_by_default: bool,
}

#[game_entity(requires = [id, name, location, seasons, time_range, rarity, difficulty, sell_price, sprite_index])]
pub struct FishDef {
    pub id: String,
    pub name: String,
    pub location: super::FishLocation,
    pub seasons: Vec<super::Season>,
    pub time_range: (f32, f32),
    pub weather_required: Option<super::Weather>,
    pub rarity: super::Rarity,
    pub difficulty: f32,
    pub sell_price: u32,
    pub sprite_index: u32,
}

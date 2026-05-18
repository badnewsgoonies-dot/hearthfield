use bevy::prelude::*;
use crate::game::resources::*;
use crate::game::events::*;

impl FishEncyclopedia {
    /// Record a successful catch. Returns `true` if this is the first time this
    /// species has been caught (useful for triggering a "New fish!" toast).
    pub fn record_catch(&mut self, fish_id: &str, day: u32, season: Season) -> bool {
        if let Some(entry) = self.entries.get_mut(fish_id) {
            entry.times_caught += 1;
            false
        } else {
            self.entries.insert(
                fish_id.to_string(),
                CaughtFishEntry {
                    fish_id: fish_id.to_string(),
                    times_caught: 1,
                    first_caught_day: day,
                    first_caught_season: season,
                },
            );
            true
        }
    }
}

impl Inventory {
    /// Try to add an item. Returns the quantity that couldn't fit.
    pub fn try_add(&mut self, item_id: &str, quantity: u8, max_stack: u8) -> u8 {
        let mut remaining = quantity;

        // First pass: stack onto existing slots with same item
        for slot in self.slots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if let Some(ref mut s) = slot {
                if s.item_id == item_id && s.quantity < max_stack {
                    let space = max_stack - s.quantity;
                    let add = remaining.min(space);
                    s.quantity += add;
                    remaining -= add;
                }
            }
        }

        // Second pass: fill empty slots
        for slot in self.slots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if slot.is_none() {
                let add = remaining.min(max_stack);
                *slot = Some(InventorySlot {
                    item_id: item_id.to_string(),
                    quantity: add,
                });
                remaining -= add;
            }
        }

        remaining
    }

    /// Remove quantity of an item. Returns how many were actually removed.
    pub fn try_remove(&mut self, item_id: &str, quantity: u8) -> u8 {
        let mut remaining = quantity;
        for slot in self.slots.iter_mut() {
            if remaining == 0 {
                break;
            }
            if let Some(ref mut s) = slot {
                if s.item_id == item_id {
                    let remove = remaining.min(s.quantity);
                    s.quantity -= remove;
                    remaining -= remove;
                    if s.quantity == 0 {
                        *slot = None;
                    }
                }
            }
        }
        quantity - remaining
    }

    pub fn count(&self, item_id: &str) -> u32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .filter(|s| s.item_id == item_id)
            .map(|s| s.quantity as u32)
            .sum()
    }

    pub fn has(&self, item_id: &str, quantity: u8) -> bool {
        self.count(item_id) >= quantity as u32
    }
}

impl MineFloor { pub fn new(v: u8) -> Self { MineFloor(v) } pub fn get(&self) -> u8 { self.0 } pub fn saturating_add(self, v: u8) -> Self { MineFloor(self.0.saturating_add(v)) } pub fn saturating_sub(self, v: u8) -> Self { MineFloor(self.0.saturating_sub(v)) } }

impl Season {
    pub fn next(self) -> Self {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Fall,
            Season::Fall => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Season::Spring => 0,
            Season::Summer => 1,
            Season::Fall => 2,
            Season::Winter => 3,
        }
    }
}

impl BuildingTier {
    pub fn next(&self) -> Option<Self> {
        match self {
            BuildingTier::None => Some(BuildingTier::Basic),
            BuildingTier::Basic => Some(BuildingTier::Big),
            BuildingTier::Big => Some(BuildingTier::Deluxe),
            BuildingTier::Deluxe => None,
        }
    }
    #[allow(dead_code)]
    pub fn capacity(&self) -> usize {
        match self {
            BuildingTier::None => 0,
            BuildingTier::Basic => 4,
            BuildingTier::Big => 8,
            BuildingTier::Deluxe => 12,
        }
    }
}

impl Gold { pub fn new(v: u32) -> Self { Gold(v) } pub fn get(&self) -> u32 { self.0 } pub fn saturating_add(self, v: u32) -> Self { Gold(self.0.saturating_add(v)) } pub fn saturating_sub(self, v: u32) -> Self { Gold(self.0.saturating_sub(v)) } }

impl ToolTier {
    pub fn upgrade_cost(&self) -> u32 {
        match self {
            ToolTier::Basic => 0,
            ToolTier::Copper => 2_000,
            ToolTier::Iron => 5_000,
            ToolTier::Gold => 10_000,
            ToolTier::Iridium => 25_000,
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            ToolTier::Basic => Some(ToolTier::Copper),
            ToolTier::Copper => Some(ToolTier::Iron),
            ToolTier::Iron => Some(ToolTier::Gold),
            ToolTier::Gold => Some(ToolTier::Iridium),
            ToolTier::Iridium => None,
        }
    }

    /// Gold cost to upgrade FROM this tier to the next.
    #[allow(dead_code)]
    pub fn upgrade_cost_gold(&self) -> u32 {
        match self {
            ToolTier::Basic => 2000,
            ToolTier::Copper => 5000,
            ToolTier::Iron => 10000,
            ToolTier::Gold => 25000,
            ToolTier::Iridium => 0,
        }
    }

    /// Number of bars required to upgrade FROM this tier.
    pub fn upgrade_bars_needed(&self) -> u8 {
        match self {
            ToolTier::Basic | ToolTier::Copper | ToolTier::Iron | ToolTier::Gold => 5,
            ToolTier::Iridium => 0,
        }
    }

    /// The bar item needed to upgrade FROM this tier.
    pub fn upgrade_bar_item(&self) -> Option<&'static str> {
        match self {
            ToolTier::Basic => Some("copper_bar"),
            ToolTier::Copper => Some("iron_bar"),
            ToolTier::Iron => Some("gold_bar"),
            ToolTier::Gold => Some("iridium_bar"),
            ToolTier::Iridium => None,
        }
    }

    /// Stamina cost multiplier. Better tools use less stamina.
    pub fn stamina_multiplier(&self) -> f32 {
        match self {
            ToolTier::Basic => 1.0,
            ToolTier::Copper => 0.85,
            ToolTier::Iron => 0.7,
            ToolTier::Gold => 0.55,
            ToolTier::Iridium => 0.4,
        }
    }

    /// Days the blacksmith takes for any upgrade.
    #[allow(dead_code)]
    pub fn upgrade_days(&self) -> u8 {
        2
    }
}

impl FishingMinigameState {
    /// Set up the minigame incorporating the player's fishing skill bonuses.
    ///
    /// `FishingSkill::catch_zone_bonus` expands the catch bar so experienced
    /// anglers have an easier time.
    pub fn setup_with_skill(
        &mut self,
        difficulty: f32,
        rod_tier: ToolTier,
        tackle_kind: TackleKind,
        fishing_skill: &FishingSkill,
    ) {
        let mut rng = RandStub;
        self.fish_zone_center = Default::default();
        self.fish_zone_velocity = 0.0;
        self.catch_bar_center = 50.0;
        self.progress = 0.0;
        self.fish_difficulty = difficulty;
        self.elapsed = 0.0;
        self.overlap_sfx_cooldown = 0.0;
        self.space_held = false;
        self.overlap_time_total = 0.0;
        self.minigame_total_time = 0.0;

        // Fish zone size: easier fish have bigger zones (more forgiving).
        // Difficulty 0.0 → fish_zone_half = 22.0
        // Difficulty 1.0 → fish_zone_half = 8.0
        let base_fish_zone = 22.0 - difficulty * 14.0;

        // Spinner: enlarges the fish zone by 25% (makes the target bigger).
        self.fish_zone_half = match tackle_kind {
            TackleKind::Spinner => base_fish_zone * 1.25,
            _ => base_fish_zone,
        };

        // Catch bar size: spec formula = 40px base + 3px per skill level.
        // The bar size is in pixels; we convert to 0-100 scale half-height.
        // The minigame bar is 200 screen-pixels tall mapping to 0-100 range,
        // so 1 unit = 2 pixels. Bar size in units = bar_size_px / 2.0.
        let bar_px = fishing_skill.bar_size_px();
        let base_half = bar_px / 2.0 / 2.0; // half-height in 0-100 units

        // Rod tier grants a small size bonus.
        let tier_bonus = match rod_tier {
            ToolTier::Basic => 1.0,
            ToolTier::Copper => 1.05,
            ToolTier::Iron => 1.10,
            ToolTier::Gold => 1.15,
            ToolTier::Iridium => 1.20,
        };
        // Tackle bar bonus.
        let catch_bar_tackle_bonus = match tackle_kind {
            TackleKind::None => 1.0,
            TackleKind::Spinner => 1.0, // Spinner helps via fish zone instead
            TackleKind::TrapBobber => 1.0, // TrapBobber helps via drain rate instead
            TackleKind::LeadBobber => 1.25, // LeadBobber gets +25% catch bar
        };
        self.catch_bar_half = base_half * catch_bar_tackle_bonus * tier_bonus;

        // Trap Bobber: slow progress drain rate (stored on minigame state for
        // update_progress to read at runtime).
        // Lead Bobber: reduce catch bar fall speed.
        // These modifiers are stored as multipliers read by the systems.
        self.progress_drain_multiplier = match tackle_kind {
            TackleKind::TrapBobber => 0.5,
            _ => 1.0,
        };
        self.catch_fall_multiplier = match tackle_kind {
            TackleKind::LeadBobber => 0.7,
            _ => 1.0,
        };

        // Reset timer with randomized first direction change
        let first_change = Default::default();
        self.direction_change_timer = Timer::from_seconds(first_change, TimerMode::Once);
    }

    /// Returns true if the catch bar is overlapping the fish zone.
    pub fn is_overlapping(&self) -> bool {
        let catch_lo = self.catch_bar_center - self.catch_bar_half;
        let catch_hi = self.catch_bar_center + self.catch_bar_half;
        let fish_lo = self.fish_zone_center - self.fish_zone_half;
        let fish_hi = self.fish_zone_center + self.fish_zone_half;
        catch_hi > fish_lo && catch_lo < fish_hi
    }

    /// Returns `true` if the player achieved a "perfect catch" — they kept the
    /// catch bar in the fish zone for at least 90% of the total minigame duration.
    ///
    /// Uses `minigame_total_time` (accumulated in `update_progress`) to judge
    /// the denominator, ignoring the initial grace period.
    pub fn is_perfect_catch(&self) -> bool {
        if self.minigame_total_time < 0.5 {
            // Too short to be meaningful — don't award perfect bonus
            return false;
        }
        let total_time = self.minigame_total_time;
        let ratio = if total_time > 0.0 {
            self.overlap_time_total / total_time
        } else {
            0.0
        };
        ratio >= PERFECT_CATCH_THRESHOLD
    }
}

impl FishingSkill {
    /// Maximum bite-speed bonus.
    pub const MAX_BITE_SPEED: f32 = 0.5;
    /// Maximum catch-zone bonus.
    pub const MAX_CATCH_ZONE: f32 = 0.3;
    /// Catches required to advance one level (legacy, kept for compatibility).
    #[allow(dead_code)]
    pub const CATCHES_PER_LEVEL: u32 = 10;
    /// Bite-speed improvement per level.
    pub const BITE_SPEED_PER_LEVEL: f32 = 0.05;
    /// Catch-zone improvement per level.
    pub const CATCH_ZONE_PER_LEVEL: f32 = 0.03;

    /// Recalculate level and derived bonuses from current XP.
    pub fn recalculate(&mut self) {
        // Determine level from XP thresholds
        self.level = 0;
        for (i, &threshold) in LEVEL_THRESHOLDS.iter().enumerate() {
            if self.xp >= threshold {
                self.level = (i as u32) + 1;
            } else {
                break;
            }
        }
        self.level = self.level.min(MAX_LEVEL);

        self.bite_speed_bonus =
            (self.level as f32 * Self::BITE_SPEED_PER_LEVEL).min(Self::MAX_BITE_SPEED);

        self.catch_zone_bonus =
            (self.level as f32 * Self::CATCH_ZONE_PER_LEVEL).min(Self::MAX_CATCH_ZONE);
    }

    /// Add XP for catching a fish of the given rarity.
    pub fn add_catch_xp(&mut self, rarity: Rarity) {
        self.total_catches += 1;
        self.xp += xp_for_rarity(rarity);
        self.recalculate();
    }

    /// Apply the bite-speed bonus to a raw wait duration (in seconds).
    #[allow(dead_code)]
    pub fn apply_bite_speed(&self, base_wait: f32) -> f32 {
        base_wait * (1.0 - self.bite_speed_bonus)
    }

    /// Apply the catch-zone bonus to a catch-bar half-height.
    #[allow(dead_code)]
    pub fn apply_catch_zone(&self, base_half: f32) -> f32 {
        base_half * (1.0 + self.catch_zone_bonus)
    }

    /// Compute the catch bar size in pixels at the current skill level.
    /// Formula: 40px base + 3px per skill level.
    pub fn bar_size_px(&self) -> f32 {
        BASE_BAR_SIZE_PX + BAR_SIZE_PER_LEVEL_PX * self.level as f32
    }

    /// Compute the bite wait reduction in seconds at the current skill level.
    /// Formula: 0.5 seconds per level.
    pub fn bite_wait_reduction(&self) -> f32 {
        BITE_WAIT_REDUCTION_PER_LEVEL * self.level as f32
    }
}

impl TackleKind {}

impl Calendar {
    pub fn day_of_week(&self) -> DayOfWeek {
        let total_days =
            (self.season.index() as u32 * DAYS_PER_SEASON as u32) + (self.day as u32 - 1);
        match total_days % 7 {
            0 => DayOfWeek::Monday,
            1 => DayOfWeek::Tuesday,
            2 => DayOfWeek::Wednesday,
            3 => DayOfWeek::Thursday,
            4 => DayOfWeek::Friday,
            5 => DayOfWeek::Saturday,
            _ => DayOfWeek::Sunday,
        }
    }

    pub fn total_days_elapsed(&self) -> u32 {
        ((self.year - 1) * (SEASONS_PER_YEAR as u32 * DAYS_PER_SEASON as u32))
            + (self.season.index() as u32 * DAYS_PER_SEASON as u32)
            + (self.day as u32 - 1)
    }

    pub fn is_festival_day(&self) -> bool {
        matches!(
            (self.season, self.day),
            (Season::Spring, 13) | (Season::Summer, 11) | (Season::Fall, 16) | (Season::Winter, 25)
        )
    }

    /// Returns time as a float (e.g. 14.5 = 2:30 PM) for schedule lookups.
    pub fn time_float(&self) -> f32 {
        self.hour as f32 + (self.minute as f32 / 60.0)
    }
}

impl ItemRegistry {
    pub fn get(&self, id: &str) -> Option<&ItemDef> {
        self.items.get(id)
    }
}

impl MachineType {
    /// Processing time in game-hours.
    pub fn processing_hours(&self) -> f32 {
        match self {
            MachineType::Furnace => 0.5,            // 30 game-minutes
            MachineType::PreservesJar => 4.0,       // 240 game-minutes
            MachineType::CheesePress => 3.0,        // 180 game-minutes
            MachineType::Loom => 4.0,               // 240 game-minutes
            MachineType::Keg => 72.0,               // 3 days × 24h
            MachineType::OilMaker => 24.0,          // 1 day
            MachineType::MayonnaiseMachine => 24.0, // 1 day
            MachineType::Tapper => 168.0,           // 7 days × 24h
            MachineType::BeeHouse => 96.0,          // 4 days × 24h
            MachineType::RecyclingMachine => 24.0,  // 1 day
            MachineType::CrabPot => 24.0,           // 1 day
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            MachineType::Furnace => "Furnace",
            MachineType::PreservesJar => "Preserves Jar",
            MachineType::CheesePress => "Cheese Press",
            MachineType::Loom => "Loom",
            MachineType::Keg => "Keg",
            MachineType::OilMaker => "Oil Maker",
            MachineType::MayonnaiseMachine => "Mayonnaise Machine",
            MachineType::Tapper => "Tapper",
            MachineType::BeeHouse => "Bee House",
            MachineType::RecyclingMachine => "Recycling Machine",
            MachineType::CrabPot => "Crab Pot",
        }
    }
}

impl ProcessingMachine {
    pub fn new(machine_type: MachineType) -> Self {
        Self {
            machine_type,
            input_item: None,
            output_item: None,
            processing_time_remaining: 0.0,
            is_ready: false,
        }
    }

    pub fn is_processing(&self) -> bool {
        self.input_item.is_some() && !self.is_ready
    }

    pub fn is_empty(&self) -> bool {
        self.input_item.is_none() && !self.is_ready
    }
}

impl StackSize { pub fn new(v: u8) -> Self { StackSize(v) } pub fn get(&self) -> u8 { self.0 } pub fn saturating_add(self, v: u8) -> Self { StackSize(self.0.saturating_add(v)) } pub fn saturating_sub(self, v: u8) -> Self { StackSize(self.0.saturating_sub(v)) } }

impl GridPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

impl BuildingLevel { pub fn new(v: u8) -> Self { BuildingLevel(v) } pub fn get(&self) -> u8 { self.0 } pub fn saturating_add(self, v: u8) -> Self { BuildingLevel(self.0.saturating_add(v)) } pub fn saturating_sub(self, v: u8) -> Self { BuildingLevel(self.0.saturating_sub(v)) } }

impl Friendship { pub fn new(v: u32) -> Self { Friendship(v) } pub fn get(&self) -> u32 { self.0 } pub fn saturating_add(self, v: u32) -> Self { Friendship(self.0.saturating_add(v)) } pub fn saturating_sub(self, v: u32) -> Self { Friendship(self.0.saturating_sub(v)) } }

impl Happiness { pub fn new(v: u8) -> Self { Happiness(v) } pub fn get(&self) -> u8 { self.0 } pub fn saturating_add(self, v: u8) -> Self { Happiness(self.0.saturating_add(v)) } pub fn saturating_sub(self, v: u8) -> Self { Happiness(self.0.saturating_sub(v)) } }

impl Relationships {
    pub fn hearts(&self, npc_id: &str) -> u8 {
        let points = self
            .friendship
            .get(npc_id)
            .copied()
            .unwrap_or(Friendship::new(0));
        (points.get() / 100).min(10) as u8
    }

    pub fn add_friendship(&mut self, npc_id: &str, amount: i32) {
        let entry = self
            .friendship
            .entry(npc_id.to_string())
            .or_insert(Friendship::new(0));
        *entry = Friendship::new(
            ((entry.get() as i32 + amount).max(0).min(MAX_FRIENDSHIP as i32)) as u32,
        );
    }
}

/// Returns `(gold_cost, Vec<(material_item_id, quantity)>)` for upgrading a
/// building *to* the given tier. Returns `(0, vec![])` for invalid combinations.
pub fn upgrade_cost(
    building: BuildingKind,
    to_tier: BuildingTier,
) -> (u32, Vec<(&'static str, u8)>) {
    match (building, to_tier) {
        // House upgrades (starts at Basic by default, upgrades to Big then Deluxe)
        (BuildingKind::House, BuildingTier::Big) => (10_000, vec![("wood", 200)]),
        (BuildingKind::House, BuildingTier::Deluxe) => (50_000, vec![("hardwood", 100)]),

        // Coop upgrades (None → Basic → Big → Deluxe)
        (BuildingKind::Coop, BuildingTier::Basic) => (4_000, vec![("wood", 150), ("stone", 50)]),
        (BuildingKind::Coop, BuildingTier::Big) => (10_000, vec![("wood", 200), ("stone", 100)]),
        (BuildingKind::Coop, BuildingTier::Deluxe) => (20_000, vec![("wood", 250), ("stone", 150)]),

        // Barn upgrades (None → Basic → Big → Deluxe)
        (BuildingKind::Barn, BuildingTier::Basic) => (6_000, vec![("wood", 200), ("stone", 75)]),
        (BuildingKind::Barn, BuildingTier::Big) => (12_000, vec![("wood", 250), ("stone", 125)]),
        (BuildingKind::Barn, BuildingTier::Deluxe) => (25_000, vec![("wood", 250), ("stone", 200)]),

        // Silo (only one tier: None → Basic)
        (BuildingKind::Silo, BuildingTier::Basic) => (100, vec![("stone", 50), ("copper_bar", 5)]),

        _ => (0, vec![]),
    }
}

pub fn color_bg_bar() -> Color {
    Color::srgba(0.15, 0.15, 0.15, 0.85)
}

pub fn color_catch_bar() -> Color {
    Color::srgb(0.46, 0.72, 0.56)
}

pub fn color_fish_zone() -> Color {
    Color::srgb(0.72, 0.58, 0.34)
}

pub fn color_progress_bg() -> Color {
    Color::srgb(0.25, 0.25, 0.25)
}

pub fn color_progress_fill() -> Color {
    Color::srgb(0.39, 0.64, 0.74)
}

/// XP awarded per catch by rarity.
pub fn xp_for_rarity(rarity: Rarity) -> u32 {
    match rarity {
        Rarity::Common => 3,
        Rarity::Uncommon => 8,
        Rarity::Rare => 15,
        Rarity::Legendary => 25,
    }
}

/// Convert a 0-100 zone position to a screen Y coordinate within the minigame bar.
/// The bar occupies MINIGAME_BAR_HEIGHT screen pixels, centered on the bar origin.
pub fn zone_to_screen_y(zone: f32) -> f32 {
    let bar_bottom = -MINIGAME_BAR_HEIGHT / 2.0;
    bar_bottom + (zone / 100.0) * MINIGAME_BAR_HEIGHT
}

/// Advances the calendar by exactly one game-minute.
/// Handles minute -> hour -> day rollovers.
pub fn advance_one_minute(
    calendar: &mut Calendar,
    day_end_writer: &mut EventWriter<DayEndEvent>,
    prev_weather: &mut PreviousDayWeather,
) {
    calendar.minute += 1;

    if calendar.minute >= 60 {
        calendar.minute = 0;
        calendar.hour += 1;

        // 2:00 AM = hour 26 -> force end of day
        if calendar.hour >= 26 {
            trigger_day_end(calendar, day_end_writer, prev_weather);
        }
    }
}

pub fn festival_for_season(season: Season) -> (u8, &'static str) {
    match season {
        Season::Spring => (13, "Egg Fest"),
        Season::Summer => (11, "Luau"),
        Season::Fall => (16, "Harvest"),
        Season::Winter => (25, "W.Star"),
    }
}

pub fn festival_name(season: Season, day: u8) -> Option<&'static str> {
    let (festival_day, festival_name) = festival_for_season(season);
    (festival_day == day).then_some(festival_name)
}

pub fn is_festival_day(season: Season, day: u8) -> bool {
    festival_name(season, day).is_some()
}

/// Rolls a weather result for the given season using weighted probabilities.
///
/// Spring:  60% Sunny, 30% Rainy, 10% Stormy
/// Summer:  70% Sunny, 20% Rainy, 10% Stormy
/// Fall:    50% Sunny, 35% Rainy, 15% Stormy
/// Winter:  40% Sunny, 10% Rainy, 10% Stormy, 40% Snowy
pub fn roll_weather(season: Season) -> Weather {
    let mut rng = RandStub;
    let roll: f32 = Default::default(); // 0.0 ..< 1.0

    match season {
        Season::Spring => {
            if roll < 0.60 {
                Weather::Sunny
            } else if roll < 0.90 {
                Weather::Rainy
            } else {
                Weather::Stormy
            }
        }
        Season::Summer => {
            if roll < 0.70 {
                Weather::Sunny
            } else if roll < 0.90 {
                Weather::Rainy
            } else {
                Weather::Stormy
            }
        }
        Season::Fall => {
            if roll < 0.50 {
                Weather::Sunny
            } else if roll < 0.85 {
                Weather::Rainy
            } else {
                Weather::Stormy
            }
        }
        Season::Winter => {
            if roll < 0.40 {
                Weather::Sunny
            } else if roll < 0.50 {
                Weather::Rainy
            } else if roll < 0.60 {
                Weather::Stormy
            } else {
                Weather::Snowy
            }
        }
    }
}

/// Called when day ends via the 2 AM auto-rollover.
/// Stores the ended day's weather in PreviousDayWeather, then rolls new
/// weather, advances day/season/year, and resets clock to 6:00 AM.
pub fn trigger_day_end(
    calendar: &mut Calendar,
    day_end_writer: &mut EventWriter<DayEndEvent>,
    prev_weather: &mut PreviousDayWeather,
) {
    // Emit event with the CURRENT day/season/year (the day that just ended)
    day_end_writer.send(DayEndEvent {
        day: calendar.day,
        season: calendar.season,
        year: calendar.year,
    });

    info!(
        "[Calendar] Day ended — Day {} {:?} Year {}",
        calendar.day, calendar.season, calendar.year
    );

    // Store the ended day's weather BEFORE rolling new weather.
    // This lets farming and other domains check if it rained today.
    prev_weather.weather = calendar.weather;

    // Advance to next day
    calendar.day += 1;
    calendar.hour = 6;
    calendar.minute = 0;
    calendar.elapsed_real_seconds = 0.0;

    // Season rollover
    if calendar.day > DAYS_PER_SEASON {
        calendar.day = 1;
        let old_season = calendar.season;
        calendar.season = calendar.season.next();

        info!(
            "[Calendar] Season changed: {:?} -> {:?} (Year {})",
            old_season, calendar.season, calendar.year
        );

        // Year rollover happens when Spring begins again
        if calendar.season == Season::Spring {
            calendar.year += 1;
            info!("[Calendar] New Year! Year {}", calendar.year);
        }
    }

    // Roll weather for the new day
    calendar.weather = roll_weather(calendar.season);

    info!(
        "[Calendar] New day: Day {} {:?} Year {} — Weather: {:?}",
        calendar.day, calendar.season, calendar.year, calendar.weather
    );
}

pub fn fireflies_should_be_active(calendar: &Calendar, map_id: MapId) -> bool {
    let time = calendar.time_float();
    (18.0..22.0).contains(&time) && !is_indoor_map(map_id)
}

/// Returns true if the given map is indoors (should have no day/night tint).
pub fn is_indoor_map(map_id: MapId) -> bool {
    matches!(
        map_id,
        MapId::PlayerHouse
            | MapId::TownHouseWest
            | MapId::TownHouseEast
            | MapId::GeneralStore
            | MapId::AnimalShop
            | MapId::Blacksmith
            | MapId::Library
            | MapId::Tavern
    )
}

/// Returns (output_item_id, output_quantity) given machine type and input item id.
pub fn resolve_machine_output(machine: MachineType, input: &str) -> Option<(ItemId, u8)> {
    match machine {
        MachineType::Furnace => match input {
            "copper_ore" => Some(("copper_bar".to_string(), 1)),
            "iron_ore" => Some(("iron_bar".to_string(), 1)),
            "gold_ore" => Some(("gold_bar".to_string(), 1)),
            "iridium_ore" => Some(("iridium_bar".to_string(), 1)),
            "coal" => Some(("coal".to_string(), 1)), // passthrough (no-op but valid)
            "quartz" => Some(("refined_quartz".to_string(), 1)),
            _ => None,
        },
        MachineType::PreservesJar => match input {
            // Fruits → Jelly
            "blueberry" => Some(("blueberry_jelly".to_string(), 1)),
            "strawberry" => Some(("strawberry_jelly".to_string(), 1)),
            "melon" => Some(("melon_jelly".to_string(), 1)),
            "apple" => Some(("apple_jelly".to_string(), 1)),
            "cranberry" => Some(("cranberry_sauce".to_string(), 1)),
            "ancient_fruit" => Some(("ancient_jelly".to_string(), 1)),
            // Vegetables → Pickles
            "turnip" => Some(("pickled_turnip".to_string(), 1)),
            "potato" => Some(("pickled_potato".to_string(), 1)),
            "cauliflower" => Some(("pickled_cauliflower".to_string(), 1)),
            "pumpkin" => Some(("pickled_pumpkin".to_string(), 1)),
            "eggplant" => Some(("pickled_eggplant".to_string(), 1)),
            "yam" => Some(("pickled_yam".to_string(), 1)),
            "tomato" => Some(("pickled_tomato".to_string(), 1)),
            "corn" => Some(("pickled_corn".to_string(), 1)),
            _ => None,
        },
        MachineType::CheesePress => match input {
            "milk" => Some(("cheese".to_string(), 1)),
            "large_milk" => Some(("large_cheese".to_string(), 1)),
            _ => None,
        },
        MachineType::Loom => match input {
            "wool" => Some(("cloth".to_string(), 1)),
            _ => None,
        },
        MachineType::Keg => match input {
            "wheat" => Some(("beer".to_string(), 1)),
            "hops" => Some(("pale_ale".to_string(), 1)),
            "blueberry" => Some(("blueberry_wine".to_string(), 1)),
            "strawberry" => Some(("strawberry_wine".to_string(), 1)),
            "melon" => Some(("melon_wine".to_string(), 1)),
            "pumpkin" => Some(("pumpkin_juice".to_string(), 1)),
            "corn" => Some(("oil".to_string(), 1)),
            "apple" => Some(("apple_cider".to_string(), 1)),
            "ancient_fruit" => Some(("ancient_fruit_wine".to_string(), 1)),
            "honey" => Some(("mead".to_string(), 1)),
            _ => None,
        },
        MachineType::OilMaker => match input {
            "sunflower" => Some(("oil".to_string(), 1)),
            "corn" => Some(("oil".to_string(), 1)),
            "truffle" => Some(("truffle_oil".to_string(), 1)),
            _ => None,
        },
        MachineType::MayonnaiseMachine => match input {
            "egg" => Some(("mayonnaise".to_string(), 1)),
            "large_egg" => Some(("mayonnaise".to_string(), 2)),
            "duck_egg" => Some(("mayonnaise".to_string(), 2)),
            _ => None,
        },
        MachineType::Tapper => match input {
            // Tapper outputs are time-based, not input-based.
            // For the machine system, just accept sap as a "prime" input.
            "sap" => Some(("maple_syrup".to_string(), 1)),
            "hardwood" => Some(("oak_resin".to_string(), 1)),
            "wood" => Some(("pine_tar".to_string(), 1)),
            _ => None,
        },
        MachineType::BeeHouse => match input {
            "wild_honey" => Some(("honey".to_string(), 1)),
            _ => None,
        },
        MachineType::RecyclingMachine => match input {
            "trash" => Some(("stone".to_string(), 3)),
            "driftwood" => Some(("wood".to_string(), 3)),
            "old_glasses" => Some(("refined_quartz".to_string(), 1)),
            "newspaper" => Some(("cloth".to_string(), 1)),
            _ => None,
        },
        MachineType::CrabPot => match input {
            // Crab pots use bait and produce random shellfish.
            "bait" => Some(("crab".to_string(), 1)),
            _ => None,
        },
    }
}

/// Convert a raw point total (0–21) into a candle count.
pub fn points_to_candles(points: u32) -> u8 {
    match points {
        0..=5 => 1,
        6..=10 => 2,
        11..=15 => 3,
        _ => 4, // 16-21
    }
}

/// Pick a gem type according to spec distribution:
/// quartz 40%, amethyst 25%, emerald 15%, ruby 12%, diamond 8%
pub fn pick_gem(rng: &mut RandStub) -> String {
    let roll: f64 = Default::default();
    if roll < 0.40 {
        "quartz".to_string()
    } else if roll < 0.65 {
        "amethyst".to_string()
    } else if roll < 0.80 {
        "emerald".to_string()
    } else if roll < 0.92 {
        "ruby".to_string()
    } else {
        "diamond".to_string()
    }
}

/// Maps music track IDs to actual audio file paths.
pub fn music_path(track_id: &str) -> Option<&'static str> {
    match track_id {
        "farm" | "spring" => Some("audio/music/pixel_1.ogg"),
        "summer" => Some("audio/music/pixel_2.ogg"),
        "fall" => Some("audio/music/pixel_3.ogg"),
        "winter" => Some("audio/music/pixel_4.ogg"),
        "town" => Some("audio/music/pixel_5.ogg"),
        "mine" | "mine_ambient" => Some("audio/music/pixel_6.ogg"),
        "forest" => Some("audio/music/pixel_7.ogg"),
        "indoor" => Some("audio/music/pixel_1.ogg"),
        "beach" => Some("audio/music/pixel_8.ogg"),
        "menu" => Some("audio/music/pixel_9.ogg"),
        "night" => Some("audio/music/pixel_10.ogg"),
        "festival" => Some("audio/music/pixel_11.ogg"),
        "credits" => Some("audio/music/pixel_12.ogg"),
        _ => None,
    }
}

/// Spawn a new music track starting at volume 0, for use during fade-in.
pub fn spawn_music_silent(
    commands: &mut Commands,
    asset_server: &AssetServer,
    music_state: &mut MusicState,
    track_id: &str,
) {
    if let Some(path) = music_path(track_id) {
        let entity = commands
            .spawn((
                AudioPlayer::<AudioSource>::new(asset_server.load(path)),
                PlaybackSettings::LOOP.with_volume(Volume::new(0.0)),
            ))
            .id();
        music_state.current_track = Some(entity);
        music_state.current_track_id = track_id.to_string();
    } else {
        music_state.current_track = None;
        music_state.current_track_id.clear();
    }
}

/// Maps SFX IDs (sent by other domains) to actual audio file paths.
pub fn sfx_path(sfx_id: &str) -> Option<&'static str> {
    match sfx_id {
        "sfx_hoe" | "sfx_axe" | "sfx_pickaxe" | "sfx_scythe" => {
            Some("audio/sfx/sfx_sounds_impact1.ogg")
        }
        "sfx_water" => Some("audio/sfx/sfx_sounds_interaction5.ogg"),
        "sfx_cast" => Some("audio/sfx/sfx_movement_jump1.ogg"),
        "chop" => Some("audio/sfx/sfx_sounds_impact2.ogg"),
        "rock_hit" => Some("audio/sfx/sfx_sounds_impact3.ogg"),
        "swish" => Some("audio/sfx/sfx_wpn_sword1.ogg"),
        "hit" => Some("audio/sfx/sfx_damage_hit1.ogg"),
        "object_break" => Some("audio/sfx/sfx_sounds_impact5.ogg"),
        "pickup" => Some("audio/sfx/sfx_coin_single1.ogg"),
        "menu_move" => Some("audio/sfx/sfx_menu_move1.ogg"),
        "menu_select" => Some("audio/sfx/sfx_menu_select1.ogg"),
        "purchase" => Some("audio/sfx/sfx_coin_cluster1.ogg"),
        "sell" => Some("audio/sfx/sfx_coin_double1.ogg"),
        "door" => Some("audio/sfx/sfx_movement_dooropen1.ogg"),
        "footstep" => Some("audio/sfx/sfx_movement_footsteps1a.ogg"),
        "error" => Some("audio/sfx/sfx_sounds_error1.ogg"),
        "fanfare" => Some("audio/sfx/sfx_sounds_fanfare1.ogg"),
        "powerup" => Some("audio/sfx/sfx_sounds_powerup1.ogg"),
        "damage" => Some("audio/sfx/sfx_sounds_damage1.ogg"),
        "axe_chop" => Some("audio/sfx/sfx_sounds_impact2.ogg"),
        "pickaxe_hit" => Some("audio/sfx/sfx_sounds_impact3.ogg"),
        "hoe_till" => Some("audio/sfx/sfx_sounds_impact1.ogg"),
        "water_splash" => Some("audio/sfx/sfx_sounds_interaction5.ogg"),
        "fishing_cast" => Some("audio/sfx/sfx_movement_jump1.ogg"),
        "tool_generic" => Some("audio/sfx/sfx_wpn_sword1.ogg"),
        "craft_success" => Some("audio/sfx/sfx_sounds_fanfare1.ogg"),
        "craft_fail" => Some("audio/sfx/sfx_sounds_error1.ogg"),
        "ui_deny" => Some("audio/sfx/sfx_sounds_error1.ogg"),
        "ui_notification" => Some("audio/sfx/sfx_menu_select1.ogg"),
        "blacksmith_forge" => Some("audio/sfx/sfx_sounds_impact3.ogg"),
        "upgrade_complete" => Some("audio/sfx/sfx_sounds_powerup1.ogg"),
        "item_pickup" => Some("audio/sfx/sfx_coin_single1.ogg"),
        "shop_buy" => Some("audio/sfx/sfx_coin_cluster1.ogg"),
        "shop_sell" => Some("audio/sfx/sfx_coin_double1.ogg"),
        "shipping_bin" => Some("audio/sfx/sfx_coin_double1.ogg"),
        "eat" => Some("audio/sfx/sfx_coin_single1.ogg"),
        "harvest" => Some("audio/sfx/sfx_coin_single1.ogg"),
        "plant" => Some("audio/sfx/sfx_sounds_interaction5.ogg"),
        "fish_escape" => Some("audio/sfx/sfx_sounds_error1.ogg"),
        "thunder" => Some("audio/sfx/sfx_sounds_damage1.ogg"),
        "treasure_found" => Some("audio/sfx/sfx_sounds_fanfare1.ogg"),
        "rocks_broken" => Some("audio/sfx/sfx_sounds_impact5.ogg"),
        _ => None,
    }
}

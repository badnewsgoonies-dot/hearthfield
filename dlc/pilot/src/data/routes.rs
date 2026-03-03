//! Route database — all route combinations between 10 airports.

use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════════
// TYPES
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug)]
pub struct RouteDef {
    pub from_airport: AirportId,
    pub to_airport: AirportId,
    pub distance_nm: f32,
    pub heading_deg: f32,
    pub difficulty: RouteDifficulty,
    pub scenery_description: String,
    pub avg_flight_time_min: f32,
    pub landmarks: Vec<String>,
    pub popularity: RoutePopularity,
    pub is_cargo_corridor: bool,
    pub is_scenic: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RouteDifficulty {
    Easy,
    Moderate,
    Challenging,
    Expert,
}

impl RouteDifficulty {
    pub fn display_name(&self) -> &'static str {
        match self {
            RouteDifficulty::Easy => "Easy",
            RouteDifficulty::Moderate => "Moderate",
            RouteDifficulty::Challenging => "Challenging",
            RouteDifficulty::Expert => "Expert",
        }
    }

    pub fn xp_multiplier(&self) -> f32 {
        match self {
            RouteDifficulty::Easy => 1.0,
            RouteDifficulty::Moderate => 1.3,
            RouteDifficulty::Challenging => 1.6,
            RouteDifficulty::Expert => 2.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RoutePopularity {
    Low,
    Medium,
    High,
}

impl RoutePopularity {
    pub fn passenger_demand_multiplier(&self) -> f32 {
        match self {
            RoutePopularity::Low => 0.5,
            RoutePopularity::Medium => 1.0,
            RoutePopularity::High => 1.5,
        }
    }

    /// Seasonal adjustment (some routes are more popular in certain seasons).
    pub fn seasonal_modifier(season: Season, route_type: RoutePopularity) -> f32 {
        match (season, route_type) {
            (Season::Summer, RoutePopularity::High) => 1.3,
            (Season::Winter, RoutePopularity::High) => 0.8,
            _ => 1.0,
        }
    }
}

/// Route familiarity tracking.
#[derive(Clone, Debug, Default)]
pub struct RouteFamiliarity {
    pub times_flown: u32,
    pub best_time_secs: Option<f32>,
}

impl RouteFamiliarity {
    pub fn efficiency_bonus(&self) -> f32 {
        match self.times_flown {
            0 => 0.0,
            1..=3 => 0.02,
            4..=9 => 0.05,
            10..=24 => 0.08,
            _ => 0.10, // Max 10% efficiency bonus
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ROUTE DATABASE — ALL 90 COMBINATIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Generate all 90 route definitions (10 airports × 9 destinations each).
pub fn all_routes() -> Vec<RouteDef> {
    vec![
        // ── FROM: HomeBase ───────────────────────────────────────────────
        route(AirportId::HomeBase, AirportId::Windport, 120.0, 180.0, RouteDifficulty::Easy,
            "Rolling green hills give way to the sparkling coastline.", 35.0,
            vec!["Clearfield River", "Windridge Valley"],
            RoutePopularity::High, false, true),
        route(AirportId::HomeBase, AirportId::Frostpeak, 200.0, 350.0, RouteDifficulty::Moderate,
            "Flat farmland transitions to dramatic mountain peaks.", 55.0,
            vec!["Eagle Pass", "Pine Forest"],
            RoutePopularity::Medium, false, true),
        route(AirportId::HomeBase, AirportId::Sunhaven, 350.0, 160.0, RouteDifficulty::Moderate,
            "Cross the central plains to the tropical southern coast.", 95.0,
            vec!["Lake Serene", "Palm Coast"],
            RoutePopularity::High, false, true),
        route(AirportId::HomeBase, AirportId::Ironforge, 180.0, 90.0, RouteDifficulty::Easy,
            "Gentle terrain over industrial heartland.", 50.0,
            vec!["Steel Bridge", "Factory District"],
            RoutePopularity::High, true, false),
        route(AirportId::HomeBase, AirportId::Cloudmere, 400.0, 320.0, RouteDifficulty::Challenging,
            "Long climb over mountain ridges to the high plateau.", 110.0,
            vec!["Storm Ridge", "Cloud Falls"],
            RoutePopularity::Low, false, true),
        route(AirportId::HomeBase, AirportId::Duskhollow, 500.0, 230.0, RouteDifficulty::Challenging,
            "Cross farmland and arid plains to the desert oasis.", 135.0,
            vec!["Dust Basin", "Mirage Flats"],
            RoutePopularity::Low, true, false),
        route(AirportId::HomeBase, AirportId::Stormwatch, 600.0, 280.0, RouteDifficulty::Expert,
            "Long haul over varied terrain to the weather research station.", 165.0,
            vec!["Thunder Valley", "Radar Peak"],
            RoutePopularity::Low, false, false),
        route(AirportId::HomeBase, AirportId::Grandcity, 450.0, 120.0, RouteDifficulty::Moderate,
            "The busy route from small-town life to the big city.", 125.0,
            vec!["Suburban Sprawl", "Grand River"],
            RoutePopularity::High, true, false),
        route(AirportId::HomeBase, AirportId::Skyreach, 800.0, 45.0, RouteDifficulty::Expert,
            "The ultimate cross-country trek to the elite airport.", 220.0,
            vec!["Continental Divide", "Crystal Lake", "Sky Spire"],
            RoutePopularity::Medium, false, true),

        // ── FROM: Windport ───────────────────────────────────────────────
        route(AirportId::Windport, AirportId::HomeBase, 120.0, 0.0, RouteDifficulty::Easy,
            "Coastal departure north over green hills to Clearfield.", 35.0,
            vec!["Lighthouse Point", "Windridge Valley"],
            RoutePopularity::High, false, true),
        route(AirportId::Windport, AirportId::Frostpeak, 300.0, 340.0, RouteDifficulty::Challenging,
            "From sea level to mountain peaks — dramatic altitude change.", 85.0,
            vec!["Coastal Cliffs", "Glacier Canyon"],
            RoutePopularity::Low, false, true),
        route(AirportId::Windport, AirportId::Sunhaven, 250.0, 200.0, RouteDifficulty::Easy,
            "Coastal hop between two beach destinations.", 70.0,
            vec!["Coral Reef", "Dolphin Bay"],
            RoutePopularity::High, false, true),
        route(AirportId::Windport, AirportId::Ironforge, 200.0, 60.0, RouteDifficulty::Easy,
            "Short hop inland to the industrial center.", 55.0,
            vec!["Harbor Bridge", "Rail Yards"],
            RoutePopularity::High, true, false),
        route(AirportId::Windport, AirportId::Cloudmere, 420.0, 330.0, RouteDifficulty::Challenging,
            "Long climb from coast to high altitude.", 115.0,
            vec!["Sea Cliffs", "Mountain Wall"],
            RoutePopularity::Low, false, false),
        route(AirportId::Windport, AirportId::Duskhollow, 450.0, 240.0, RouteDifficulty::Moderate,
            "Cross the interior from coast to desert.", 125.0,
            vec!["River Delta", "Scrubland"],
            RoutePopularity::Medium, true, false),
        route(AirportId::Windport, AirportId::Stormwatch, 520.0, 290.0, RouteDifficulty::Challenging,
            "Coastal to storm country.", 145.0,
            vec!["Fog Bank", "Weather Front"],
            RoutePopularity::Low, false, false),
        route(AirportId::Windport, AirportId::Grandcity, 350.0, 90.0, RouteDifficulty::Moderate,
            "Popular commuter route to the metropolis.", 95.0,
            vec!["Coastal Highway", "City Skyline"],
            RoutePopularity::High, true, false),
        route(AirportId::Windport, AirportId::Skyreach, 750.0, 30.0, RouteDifficulty::Expert,
            "Coast to summit — the full Skywarden experience.", 205.0,
            vec!["Open Ocean", "Continental Shelf", "Sky Spire"],
            RoutePopularity::Low, false, true),

        // ── FROM: Frostpeak ──────────────────────────────────────────────
        route(AirportId::Frostpeak, AirportId::HomeBase, 200.0, 170.0, RouteDifficulty::Moderate,
            "Descend from alpine heights to the gentle lowlands.", 55.0,
            vec!["Pine Forest", "Farm Country"],
            RoutePopularity::Medium, false, true),
        route(AirportId::Frostpeak, AirportId::Windport, 300.0, 160.0, RouteDifficulty::Challenging,
            "Mountain to sea — steep descent with canyon views.", 85.0,
            vec!["Glacier Canyon", "Coastal Approach"],
            RoutePopularity::Low, false, true),
        route(AirportId::Frostpeak, AirportId::Sunhaven, 480.0, 190.0, RouteDifficulty::Challenging,
            "Snow-capped peaks to tropical paradise.", 130.0,
            vec!["Alpine Meadow", "Tropical Transition"],
            RoutePopularity::Medium, false, true),
        route(AirportId::Frostpeak, AirportId::Ironforge, 280.0, 130.0, RouteDifficulty::Moderate,
            "Mountain approach to industrial valley.", 75.0,
            vec!["Iron Ridge", "Smoke Stacks"],
            RoutePopularity::Medium, true, false),
        route(AirportId::Frostpeak, AirportId::Cloudmere, 150.0, 300.0, RouteDifficulty::Challenging,
            "High-altitude hop between mountain airports.", 40.0,
            vec!["Ridge Line", "Cloud Pass"],
            RoutePopularity::Low, false, true),
        route(AirportId::Frostpeak, AirportId::Duskhollow, 550.0, 220.0, RouteDifficulty::Expert,
            "From frozen peaks to scorching desert.", 150.0,
            vec!["Tree Line", "Transition Zone", "Sand Sea"],
            RoutePopularity::Low, false, true),
        route(AirportId::Frostpeak, AirportId::Stormwatch, 400.0, 260.0, RouteDifficulty::Expert,
            "Mountain to storm — turbulent and demanding.", 110.0,
            vec!["Avalanche Chute", "Storm Wall"],
            RoutePopularity::Low, false, false),
        route(AirportId::Frostpeak, AirportId::Grandcity, 380.0, 140.0, RouteDifficulty::Moderate,
            "Alpine descent to the big city.", 105.0,
            vec!["Valley Floor", "Suburban Ring"],
            RoutePopularity::Medium, false, false),
        route(AirportId::Frostpeak, AirportId::Skyreach, 550.0, 20.0, RouteDifficulty::Expert,
            "Peak to peak — the mountain route.", 150.0,
            vec!["Summit Ridge", "High Plateau"],
            RoutePopularity::Low, false, true),

        // ── FROM: Sunhaven ───────────────────────────────────────────────
        route(AirportId::Sunhaven, AirportId::HomeBase, 350.0, 340.0, RouteDifficulty::Moderate,
            "North from the tropics through green plains.", 95.0,
            vec!["Palm Coast", "Central Plains"],
            RoutePopularity::High, false, false),
        route(AirportId::Sunhaven, AirportId::Windport, 250.0, 20.0, RouteDifficulty::Easy,
            "Tropical coast to temperate coast — scenic water views.", 70.0,
            vec!["Reef Chain", "Lighthouse Row"],
            RoutePopularity::High, false, true),
        route(AirportId::Sunhaven, AirportId::Frostpeak, 480.0, 10.0, RouteDifficulty::Challenging,
            "Beach to blizzard — extreme climate transition.", 130.0,
            vec!["Mangrove Marsh", "Foothills"],
            RoutePopularity::Medium, false, true),
        route(AirportId::Sunhaven, AirportId::Ironforge, 320.0, 50.0, RouteDifficulty::Easy,
            "Resort to factory — contrast in destinations.", 90.0,
            vec!["Sugar Cane Fields", "Industrial Zone"],
            RoutePopularity::Medium, true, false),
        route(AirportId::Sunhaven, AirportId::Cloudmere, 500.0, 350.0, RouteDifficulty::Challenging,
            "Long climb from sea level to cloud height.", 140.0,
            vec!["Waterfall Valley", "Plateau Edge"],
            RoutePopularity::Low, false, false),
        route(AirportId::Sunhaven, AirportId::Duskhollow, 300.0, 250.0, RouteDifficulty::Moderate,
            "Tropical to arid — passing through transition zones.", 85.0,
            vec!["Dry Creek Bed", "Cactus Fields"],
            RoutePopularity::Medium, false, true),
        route(AirportId::Sunhaven, AirportId::Stormwatch, 480.0, 300.0, RouteDifficulty::Challenging,
            "From calm seas to storm alley.", 130.0,
            vec!["Open Water", "Storm Front"],
            RoutePopularity::Low, false, false),
        route(AirportId::Sunhaven, AirportId::Grandcity, 200.0, 10.0, RouteDifficulty::Easy,
            "Short hop — the popular vacation route.", 55.0,
            vec!["Beach Resorts", "City Approach"],
            RoutePopularity::High, false, false),
        route(AirportId::Sunhaven, AirportId::Skyreach, 700.0, 30.0, RouteDifficulty::Expert,
            "From paradise to the pinnacle.", 190.0,
            vec!["Ocean View", "Mountain Chain"],
            RoutePopularity::Low, false, true),

        // ── FROM: Ironforge ──────────────────────────────────────────────
        route(AirportId::Ironforge, AirportId::HomeBase, 180.0, 270.0, RouteDifficulty::Easy,
            "Quick return west over gentle terrain.", 50.0,
            vec!["Factory Outskirts", "Green Belt"],
            RoutePopularity::High, true, false),
        route(AirportId::Ironforge, AirportId::Windport, 200.0, 240.0, RouteDifficulty::Easy,
            "Industrial to coastal — following the river.", 55.0,
            vec!["Rail Bridge", "River Mouth"],
            RoutePopularity::High, true, false),
        route(AirportId::Ironforge, AirportId::Frostpeak, 280.0, 310.0, RouteDifficulty::Moderate,
            "Climb from the foundries to the frozen peaks.", 75.0,
            vec!["Ore Mountains", "Snow Line"],
            RoutePopularity::Medium, true, false),
        route(AirportId::Ironforge, AirportId::Sunhaven, 320.0, 230.0, RouteDifficulty::Easy,
            "Industrial to tropical — popular cargo route.", 90.0,
            vec!["Farmland", "Beach Approach"],
            RoutePopularity::Medium, true, false),
        route(AirportId::Ironforge, AirportId::Cloudmere, 350.0, 330.0, RouteDifficulty::Moderate,
            "Through the mining hills to high altitude.", 95.0,
            vec!["Quarry", "Highland Pass"],
            RoutePopularity::Low, true, false),
        route(AirportId::Ironforge, AirportId::Duskhollow, 400.0, 210.0, RouteDifficulty::Moderate,
            "Major cargo corridor through varied terrain.", 110.0,
            vec!["Dry Plains", "Oasis Approach"],
            RoutePopularity::Medium, true, false),
        route(AirportId::Ironforge, AirportId::Stormwatch, 450.0, 280.0, RouteDifficulty::Challenging,
            "Long industrial route west to the storm station.", 125.0,
            vec!["Rail Lines", "Weather Ridge"],
            RoutePopularity::Low, true, false),
        route(AirportId::Ironforge, AirportId::Grandcity, 250.0, 110.0, RouteDifficulty::Easy,
            "The business shuttle — always busy.", 70.0,
            vec!["Highway Corridor", "Downtown Skyline"],
            RoutePopularity::High, true, false),
        route(AirportId::Ironforge, AirportId::Skyreach, 650.0, 40.0, RouteDifficulty::Expert,
            "Cross-country freight route to the summit.", 180.0,
            vec!["Industrial Corridor", "Mountain Approach"],
            RoutePopularity::Medium, true, false),

        // ── FROM: Cloudmere ──────────────────────────────────────────────
        route(AirportId::Cloudmere, AirportId::HomeBase, 400.0, 140.0, RouteDifficulty::Challenging,
            "Long descent from the heights to the lowlands.", 110.0,
            vec!["Cloud Falls", "Rolling Hills"],
            RoutePopularity::Low, false, true),
        route(AirportId::Cloudmere, AirportId::Windport, 420.0, 150.0, RouteDifficulty::Challenging,
            "Mountain to coast — dramatic descent.", 115.0,
            vec!["Mountain Wall", "Coastal Plain"],
            RoutePopularity::Low, false, true),
        route(AirportId::Cloudmere, AirportId::Frostpeak, 150.0, 120.0, RouteDifficulty::Challenging,
            "Short mountain hop between alpine airports.", 40.0,
            vec!["Ridge Saddle", "Alpine Lake"],
            RoutePopularity::Low, false, true),
        route(AirportId::Cloudmere, AirportId::Sunhaven, 500.0, 170.0, RouteDifficulty::Challenging,
            "From clouds to beaches — maximum contrast.", 140.0,
            vec!["Plateau Edge", "Green Valley"],
            RoutePopularity::Low, false, true),
        route(AirportId::Cloudmere, AirportId::Ironforge, 350.0, 150.0, RouteDifficulty::Moderate,
            "Descend from heights to the factory floor.", 95.0,
            vec!["Highland Pass", "Quarry"],
            RoutePopularity::Low, true, false),
        route(AirportId::Cloudmere, AirportId::Duskhollow, 480.0, 200.0, RouteDifficulty::Expert,
            "Cloud to desert — thin air to thin shade.", 130.0,
            vec!["Dry Canyon", "Sand Dunes"],
            RoutePopularity::Low, false, true),
        route(AirportId::Cloudmere, AirportId::Stormwatch, 380.0, 250.0, RouteDifficulty::Expert,
            "Between the mountains and the storms.", 105.0,
            vec!["Wind Gap", "Thunder Basin"],
            RoutePopularity::Low, false, false),
        route(AirportId::Cloudmere, AirportId::Grandcity, 420.0, 130.0, RouteDifficulty::Moderate,
            "From isolated heights to urban sprawl.", 115.0,
            vec!["Valley Route", "City Lights"],
            RoutePopularity::Medium, false, false),
        route(AirportId::Cloudmere, AirportId::Skyreach, 350.0, 30.0, RouteDifficulty::Challenging,
            "Peak-to-peak flight over the roof of the world.", 95.0,
            vec!["Summit Trail", "Crystal Ridge"],
            RoutePopularity::Medium, false, true),

        // ── FROM: Duskhollow ─────────────────────────────────────────────
        route(AirportId::Duskhollow, AirportId::HomeBase, 500.0, 50.0, RouteDifficulty::Challenging,
            "Desert to green — long northward trek.", 135.0,
            vec!["Sand Sea", "Transition Zone"],
            RoutePopularity::Low, false, false),
        route(AirportId::Duskhollow, AirportId::Windport, 450.0, 60.0, RouteDifficulty::Moderate,
            "Desert to coast — smell the salt air.", 125.0,
            vec!["Arid Plateau", "River Valley"],
            RoutePopularity::Medium, false, false),
        route(AirportId::Duskhollow, AirportId::Frostpeak, 550.0, 40.0, RouteDifficulty::Expert,
            "From scorching sand to frozen summit.", 150.0,
            vec!["Heat Haze", "Foothills", "Snow Line"],
            RoutePopularity::Low, false, true),
        route(AirportId::Duskhollow, AirportId::Sunhaven, 300.0, 70.0, RouteDifficulty::Moderate,
            "Desert oasis to tropical beach.", 85.0,
            vec!["Canyon Lands", "Coastal Turn"],
            RoutePopularity::Medium, false, true),
        route(AirportId::Duskhollow, AirportId::Ironforge, 400.0, 30.0, RouteDifficulty::Moderate,
            "Cargo corridor from desert to factory.", 110.0,
            vec!["Dry Plains", "Industrial Belt"],
            RoutePopularity::Medium, true, false),
        route(AirportId::Duskhollow, AirportId::Cloudmere, 480.0, 20.0, RouteDifficulty::Expert,
            "Sand to sky — extreme altitude gain.", 130.0,
            vec!["Mesa Country", "Mountain Wall"],
            RoutePopularity::Low, false, true),
        route(AirportId::Duskhollow, AirportId::Stormwatch, 200.0, 310.0, RouteDifficulty::Moderate,
            "Short hop between remote outposts.", 55.0,
            vec!["Badlands", "Weather Station"],
            RoutePopularity::Low, false, false),
        route(AirportId::Duskhollow, AirportId::Grandcity, 520.0, 60.0, RouteDifficulty::Moderate,
            "Desert to metropolis — land of contrasts.", 145.0,
            vec!["Oasis Rest Stop", "Suburban Ring"],
            RoutePopularity::Medium, true, false),
        route(AirportId::Duskhollow, AirportId::Skyreach, 700.0, 10.0, RouteDifficulty::Expert,
            "The desert exodus to the sky.", 190.0,
            vec!["Sand Storm Alley", "Continental Ridge"],
            RoutePopularity::Low, false, true),

        // ── FROM: Stormwatch ─────────────────────────────────────────────
        route(AirportId::Stormwatch, AirportId::HomeBase, 600.0, 100.0, RouteDifficulty::Expert,
            "From the storm station back to civilization.", 165.0,
            vec!["Weather Front", "Green Country"],
            RoutePopularity::Low, false, false),
        route(AirportId::Stormwatch, AirportId::Windport, 520.0, 110.0, RouteDifficulty::Challenging,
            "Storm to coast — often turbulent.", 145.0,
            vec!["Cloud Bank", "Clear Skies"],
            RoutePopularity::Low, false, false),
        route(AirportId::Stormwatch, AirportId::Frostpeak, 400.0, 80.0, RouteDifficulty::Expert,
            "Between two of the toughest airports.", 110.0,
            vec!["Storm Wall", "Mountain Approach"],
            RoutePopularity::Low, false, false),
        route(AirportId::Stormwatch, AirportId::Sunhaven, 480.0, 120.0, RouteDifficulty::Challenging,
            "Storm country to sunny beaches.", 130.0,
            vec!["Rain Curtain", "Sun Break"],
            RoutePopularity::Low, false, false),
        route(AirportId::Stormwatch, AirportId::Ironforge, 450.0, 100.0, RouteDifficulty::Challenging,
            "Research station to industrial hub.", 125.0,
            vec!["Observatory", "Factory Smokestacks"],
            RoutePopularity::Low, true, false),
        route(AirportId::Stormwatch, AirportId::Cloudmere, 380.0, 70.0, RouteDifficulty::Expert,
            "Storm to clouds — for experienced pilots only.", 105.0,
            vec!["Thunder Basin", "High Pass"],
            RoutePopularity::Low, false, false),
        route(AirportId::Stormwatch, AirportId::Duskhollow, 200.0, 130.0, RouteDifficulty::Moderate,
            "Short hop between remote outposts.", 55.0,
            vec!["Weather Station", "Badlands"],
            RoutePopularity::Low, false, false),
        route(AirportId::Stormwatch, AirportId::Grandcity, 500.0, 100.0, RouteDifficulty::Challenging,
            "Storm station to the big city.", 140.0,
            vec!["Front Range", "Urban Glow"],
            RoutePopularity::Medium, false, false),
        route(AirportId::Stormwatch, AirportId::Skyreach, 300.0, 40.0, RouteDifficulty::Expert,
            "Remote to elite — the prestige route.", 85.0,
            vec!["Lightning Alley", "Sky Spire"],
            RoutePopularity::Low, false, true),

        // ── FROM: Grandcity ──────────────────────────────────────────────
        route(AirportId::Grandcity, AirportId::HomeBase, 450.0, 300.0, RouteDifficulty::Moderate,
            "City departure west to the quiet countryside.", 125.0,
            vec!["Urban Sprawl", "Farm Belt"],
            RoutePopularity::High, true, false),
        route(AirportId::Grandcity, AirportId::Windport, 350.0, 270.0, RouteDifficulty::Moderate,
            "Business route to the coast.", 95.0,
            vec!["Highway Corridor", "Coastal Approach"],
            RoutePopularity::High, true, false),
        route(AirportId::Grandcity, AirportId::Frostpeak, 380.0, 320.0, RouteDifficulty::Moderate,
            "City to mountains — popular ski charter.", 105.0,
            vec!["Suburban Ring", "Mountain Vista"],
            RoutePopularity::High, false, true),
        route(AirportId::Grandcity, AirportId::Sunhaven, 200.0, 190.0, RouteDifficulty::Easy,
            "The vacation express — always packed.", 55.0,
            vec!["Beach Road", "Resort Strip"],
            RoutePopularity::High, false, true),
        route(AirportId::Grandcity, AirportId::Ironforge, 250.0, 290.0, RouteDifficulty::Easy,
            "Business shuttle to the industrial zone.", 70.0,
            vec!["Commerce Park", "Rail Junction"],
            RoutePopularity::High, true, false),
        route(AirportId::Grandcity, AirportId::Cloudmere, 420.0, 310.0, RouteDifficulty::Moderate,
            "From sea level city to mountain heights.", 115.0,
            vec!["Urban Canyon", "Plateau Rim"],
            RoutePopularity::Medium, false, false),
        route(AirportId::Grandcity, AirportId::Duskhollow, 520.0, 240.0, RouteDifficulty::Moderate,
            "Metropolis to remote desert — the exotic route.", 145.0,
            vec!["Suburb Belt", "Dry Frontier"],
            RoutePopularity::Medium, true, false),
        route(AirportId::Grandcity, AirportId::Stormwatch, 500.0, 280.0, RouteDifficulty::Challenging,
            "City to research station — for weather enthusiasts.", 140.0,
            vec!["Clear Corridor", "Storm Approach"],
            RoutePopularity::Medium, false, false),
        route(AirportId::Grandcity, AirportId::Skyreach, 500.0, 20.0, RouteDifficulty::Challenging,
            "The flagship route — Grand City to Skyreach Elite.", 140.0,
            vec!["Grand River", "Continental Rise", "Sky Spire"],
            RoutePopularity::High, true, true),

        // ── FROM: Skyreach ───────────────────────────────────────────────
        route(AirportId::Skyreach, AirportId::HomeBase, 800.0, 225.0, RouteDifficulty::Expert,
            "The long journey from the summit back to where it all began.", 220.0,
            vec!["Crystal Lake", "Continental Divide", "Home Valley"],
            RoutePopularity::Medium, false, true),
        route(AirportId::Skyreach, AirportId::Windport, 750.0, 210.0, RouteDifficulty::Expert,
            "Elite to coast — the premium route.", 205.0,
            vec!["Mountain Descent", "Ocean Approach"],
            RoutePopularity::Low, false, true),
        route(AirportId::Skyreach, AirportId::Frostpeak, 550.0, 200.0, RouteDifficulty::Expert,
            "Peak to peak — mountain mastery required.", 150.0,
            vec!["High Plateau", "Alpine Zone"],
            RoutePopularity::Low, false, true),
        route(AirportId::Skyreach, AirportId::Sunhaven, 700.0, 210.0, RouteDifficulty::Expert,
            "From the top of the world to tropical paradise.", 190.0,
            vec!["Mountain Chain", "Tropical Approach"],
            RoutePopularity::Medium, false, true),
        route(AirportId::Skyreach, AirportId::Ironforge, 650.0, 220.0, RouteDifficulty::Expert,
            "Elite to industrial — the logistics backbone.", 180.0,
            vec!["Sky Descent", "Factory Lights"],
            RoutePopularity::Medium, true, false),
        route(AirportId::Skyreach, AirportId::Cloudmere, 350.0, 210.0, RouteDifficulty::Challenging,
            "High-altitude link between elite airports.", 95.0,
            vec!["Crystal Ridge", "Cloud Lake"],
            RoutePopularity::Medium, false, true),
        route(AirportId::Skyreach, AirportId::Duskhollow, 700.0, 190.0, RouteDifficulty::Expert,
            "The sky-to-sand express.", 190.0,
            vec!["Continental Ridge", "Desert Floor"],
            RoutePopularity::Low, false, true),
        route(AirportId::Skyreach, AirportId::Stormwatch, 300.0, 220.0, RouteDifficulty::Expert,
            "Elite to research — short but treacherous.", 85.0,
            vec!["Lightning Alley", "Storm Base"],
            RoutePopularity::Low, false, false),
        route(AirportId::Skyreach, AirportId::Grandcity, 500.0, 200.0, RouteDifficulty::Challenging,
            "The return leg of the flagship route.", 140.0,
            vec!["Sky Spire", "Grand River", "City Lights"],
            RoutePopularity::High, true, true),
    ]
}

fn route(
    from: AirportId,
    to: AirportId,
    dist: f32,
    heading: f32,
    difficulty: RouteDifficulty,
    scenery: &str,
    avg_time: f32,
    landmarks: Vec<&str>,
    popularity: RoutePopularity,
    cargo: bool,
    scenic: bool,
) -> RouteDef {
    RouteDef {
        from_airport: from,
        to_airport: to,
        distance_nm: dist,
        heading_deg: heading,
        difficulty,
        scenery_description: scenery.to_string(),
        avg_flight_time_min: avg_time,
        landmarks: landmarks.into_iter().map(String::from).collect(),
        popularity,
        is_cargo_corridor: cargo,
        is_scenic: scenic,
    }
}

/// Look up a specific route.
pub fn find_route(from: AirportId, to: AirportId) -> Option<RouteDef> {
    all_routes()
        .into_iter()
        .find(|r| r.from_airport == from && r.to_airport == to)
}

/// Get all routes from a given airport.
pub fn routes_from(airport: AirportId) -> Vec<RouteDef> {
    all_routes()
        .into_iter()
        .filter(|r| r.from_airport == airport)
        .collect()
}

/// Get popular routes (high passenger demand).
pub fn popular_routes() -> Vec<RouteDef> {
    all_routes()
        .into_iter()
        .filter(|r| r.popularity == RoutePopularity::High)
        .collect()
}

/// Get scenic routes for tourism missions.
pub fn scenic_routes() -> Vec<RouteDef> {
    all_routes()
        .into_iter()
        .filter(|r| r.is_scenic)
        .collect()
}

/// Get cargo corridor routes.
pub fn cargo_routes() -> Vec<RouteDef> {
    all_routes()
        .into_iter()
        .filter(|r| r.is_cargo_corridor)
        .collect()
}

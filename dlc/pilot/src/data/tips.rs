//! Loading screen tips and gameplay tips — displayed during transitions.

use rand::Rng;

// ═══════════════════════════════════════════════════════════════════════════
// TIP CATEGORIES
// ═══════════════════════════════════════════════════════════════════════════

const FLIGHT_TIPS: &[&str] = &[
    "Check weather before flying to avoid dangerous conditions.",
    "Perfect landings earn 50% bonus XP — aim for that butter!",
    "Keep your aircraft maintained — breakdowns mid-flight are no joke!",
    "Lower your throttle during descent for smoother approaches.",
    "Deploy flaps before landing to reduce your approach speed.",
    "Crosswinds can push you off the runway. Adjust your heading!",
    "Night flights are trickier but reward more XP.",
    "Use the preflight checklist to avoid in-flight surprises.",
    "Turbulence is worse in storms. Consider waiting for better weather.",
    "Lower altitudes burn more fuel. Climb to cruise altitude quickly.",
    "Headwinds slow you down and burn more fuel. Plan accordingly!",
    "The autopilot can handle cruise, but you need to land manually.",
];

const ECONOMY_TIPS: &[&str] = &[
    "Cargo missions pay more in winter — supply chains need you!",
    "Charter flights are most profitable in summer tourist season.",
    "Repair your aircraft regularly to avoid expensive emergency fixes.",
    "Fuel up at smaller airports — they often have lower prices.",
    "VIP charters pay premium rates but demand perfect service.",
    "Medical flights don't pay the most, but they boost reputation fast.",
    "Buy upgrades at Ironforge Industrial — best prices for parts.",
    "Selling souvenirs from different airports can be profitable.",
    "Keep an eye on the mission board — new contracts refresh daily.",
    "Higher rank unlocks better-paying missions at bigger airports.",
];

const SOCIAL_TIPS: &[&str] = &[
    "Crew members have gift preferences. Pay attention to their reactions!",
    "Visit crew members at their favorite hangout spots for bonus friendship.",
    "Higher friendship unlocks special dialogue and flight bonuses.",
    "Give gifts on birthdays for triple friendship points!",
    "Some crew members prefer food gifts, others prefer collectibles.",
    "Talk to crew regularly — even a quick chat builds friendship.",
    "Max friendship with all crew unlocks the 'Crew Family' achievement.",
    "Crew in the lounge are more receptive to conversation.",
    "Your co-pilot's mood affects passenger comfort ratings.",
    "Different crew members offer different flight assist bonuses.",
];

const EXPLORATION_TIPS: &[&str] = &[
    "Visit all 10 airports to unlock the 'Globe Trotter' achievement.",
    "Each city has unique shops and items you won't find elsewhere.",
    "The control tower has live weather data for your current airport.",
    "Check the notice board in the terminal for local events.",
    "Use the computer terminal to check METAR weather data.",
    "Coffee machines in the lounge restore stamina — use them!",
    "The map display shows all available air routes.",
    "Explore city streets to find hidden shops and collectibles.",
    "Seasonal events bring special missions and bonuses.",
    "Stormwatch Research Station has the best weather forecasting equipment.",
];

const AIRCRAFT_TIPS: &[&str] = &[
    "Larger aircraft can carry more passengers but cost more to maintain.",
    "Seaplanes can land on water — useful for coastal airports.",
    "Turboprops are great for short-haul routes with short runways.",
    "Heavy jets need long runways. Check before accepting missions!",
    "Aircraft condition below 30% increases emergency event chance.",
    "Naming your aircraft gives a sense of attachment — try it!",
    "Each aircraft class requires a specific license to fly.",
    "Twin-engine props are more reliable in bad weather than singles.",
    "Cargo freighters sacrifice passenger space for heavy loads.",
    "The best aircraft for a mission depends on distance and cargo.",
];

/// All tips combined for random selection.
const ALL_TIPS: &[&[&str]] = &[
    FLIGHT_TIPS,
    ECONOMY_TIPS,
    SOCIAL_TIPS,
    EXPLORATION_TIPS,
    AIRCRAFT_TIPS,
];

// ═══════════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════════

/// Return a random gameplay tip from any category.
pub fn get_random_tip() -> &'static str {
    let mut rng = rand::thread_rng();
    let category = ALL_TIPS[rng.gen_range(0..ALL_TIPS.len())];
    category[rng.gen_range(0..category.len())]
}

/// Return a random tip from a specific category.
pub fn get_tip_by_category(category: TipCategory) -> &'static str {
    let pool = match category {
        TipCategory::Flight => FLIGHT_TIPS,
        TipCategory::Economy => ECONOMY_TIPS,
        TipCategory::Social => SOCIAL_TIPS,
        TipCategory::Exploration => EXPLORATION_TIPS,
        TipCategory::Aircraft => AIRCRAFT_TIPS,
    };
    let mut rng = rand::thread_rng();
    pool[rng.gen_range(0..pool.len())]
}

/// Tip categories for filtered selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TipCategory {
    Flight,
    Economy,
    Social,
    Exploration,
    Aircraft,
}

/// Return the total number of tips across all categories.
pub fn total_tip_count() -> usize {
    ALL_TIPS.iter().map(|c| c.len()).sum()
}

/// Return all tips for a given category.
pub fn all_tips_in_category(category: TipCategory) -> &'static [&'static str] {
    match category {
        TipCategory::Flight => FLIGHT_TIPS,
        TipCategory::Economy => ECONOMY_TIPS,
        TipCategory::Social => SOCIAL_TIPS,
        TipCategory::Exploration => EXPLORATION_TIPS,
        TipCategory::Aircraft => AIRCRAFT_TIPS,
    }
}

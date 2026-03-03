//! Detailed city data for all 10 airports in the Skywarden world.

use crate::shared::*;

/// Rich information about a city associated with an airport.
#[derive(Clone, Debug)]
pub struct CityInfo {
    pub name: String,
    pub description: String,
    pub climate: String,
    pub population: String,
    pub attractions: Vec<String>,
    pub local_cuisine: Vec<String>,
    pub fun_facts: Vec<String>,
}

impl CityInfo {
    fn new(
        name: &str,
        description: &str,
        climate: &str,
        population: &str,
        attractions: Vec<&str>,
        local_cuisine: Vec<&str>,
        fun_facts: Vec<&str>,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            climate: climate.to_string(),
            population: population.to_string(),
            attractions: attractions.into_iter().map(String::from).collect(),
            local_cuisine: local_cuisine.into_iter().map(String::from).collect(),
            fun_facts: fun_facts.into_iter().map(String::from).collect(),
        }
    }
}

/// Populate all city data into the registry.
pub fn populate_cities(registry: &mut CityRegistry) {
    registry.cities.insert(
        AirportId::HomeBase,
        CityInfo::new(
            "Windridge",
            "A small, quiet aviation town nestled in rolling green hills. \
             Windridge is the birthplace of many great pilots and serves as \
             the primary training hub for aspiring aviators. The pace of life \
             is slow, with friendly locals who wave at every takeoff.",
            "Temperate, mild year-round",
            "12,000",
            vec![
                "Windridge Flight Academy — the oldest training school in the region",
                "Hilltop Lookout — panoramic views of the surrounding farmland and runways",
                "Pioneer Aviation Museum — chronicles the early days of flight",
                "Sunrise Café — a favorite breakfast spot for student pilots",
            ],
            vec![
                "Windridge apple pie — made with local orchard apples",
                "Runway burgers — famous half-pound burgers at the terminal diner",
                "Cloud-nine cocoa — warm chocolate drink with whipped cream peaks",
            ],
            vec![
                "Windridge produces more pilots per capita than any other town in the region",
                "The town was founded by a retired barnstormer who landed here by accident",
                "Every Saturday, locals gather to watch student pilots practice touch-and-goes",
            ],
        ),
    );

    registry.cities.insert(
        AirportId::Grandcity,
        CityInfo::new(
            "Grandcity",
            "A sprawling metropolis of steel and glass, home to 8 million people. \
             Grand City International is the busiest airport in the network, with \
             flights departing every few minutes. Rush hour traffic — both on the \
             ground and in the air — is legendary. The city never sleeps.",
            "Continental, humid summers and cold winters",
            "8,200,000",
            vec![
                "Skyline Tower — the tallest building in the world at 1,200 feet",
                "Grand Central Market — an enormous food hall with 200+ vendors",
                "Grandcity National Theatre — world-class performances nightly",
                "Harbor Walk — a scenic waterfront promenade with ferry rides",
                "Grandcity Aviation Expo — annual showcase of the latest aircraft",
            ],
            vec![
                "Grandcity hot dogs — street-cart style with mustard and sauerkraut",
                "Skyline steak — thick-cut, charcoal-grilled, served on a skyscraper plate",
                "Metro espresso — triple-shot espresso to keep up with the city pace",
            ],
            vec![
                "Grand City International processes over 5,000 flights per day",
                "The city's first airport was a grass strip on what is now the financial district",
                "Rush hour approach sequencing is considered the hardest ATC task in the region",
            ],
        ),
    );

    registry.cities.insert(
        AirportId::Sunhaven,
        CityInfo::new(
            "Sunhaven",
            "A tropical resort paradise where turquoise waters meet white-sand beaches. \
             Sunhaven thrives on tourism, with luxury resorts lining the coast. \
             The airport handles a steady stream of vacationers and charter flights. \
             Despite the relaxed atmosphere, sudden tropical storms demand respect.",
            "Tropical, warm and humid with a wet season",
            "85,000",
            vec![
                "Crystal Beach — pristine white sand and calm waters for snorkeling",
                "Sunhaven Boardwalk — shops, restaurants, and live music along the shore",
                "Tropical Canopy Zipline — fly through the rainforest canopy",
                "Sunset Lighthouse — a historic lighthouse with the best sunset views",
            ],
            vec![
                "Coconut shrimp — breaded in fresh coconut, fried golden",
                "Mango tango smoothie — a blend of local mangoes and passion fruit",
                "Island-grilled fish tacos — caught fresh every morning",
            ],
            vec![
                "Sunhaven sees over 300 sunny days per year despite occasional hurricanes",
                "The local parrot population has learned to mimic ATC radio calls",
                "Pilots call the runway 'the beach strip' because sand frequently blows across it",
            ],
        ),
    );

    registry.cities.insert(
        AirportId::Frostpeak,
        CityInfo::new(
            "Frostpeak",
            "A rugged mountain town perched at 6,000 feet elevation, surrounded by \
             snow-capped peaks. Frostpeak is a world-renowned ski resort in winter \
             and a hiking paradise in summer. The alpine airport is infamous for its \
             challenging approaches through narrow valleys and unpredictable winds.",
            "Alpine, cold winters with heavy snowfall, cool summers",
            "28,000",
            vec![
                "Frostpeak Ski Resort — 40 runs across three mountains",
                "Eagle's Nest Viewpoint — a glass platform jutting over a 2,000-foot drop",
                "Alpine Hot Springs — natural thermal pools amid the snow",
                "Glacier Trail — a guided trek to an ancient glacier",
            ],
            vec![
                "Alpine fondue — rich cheese fondue with crusty bread",
                "Frostpeak venison stew — slow-cooked with root vegetables",
                "Snowcap lager — locally brewed craft beer, best served cold",
            ],
            vec![
                "Frostpeak's runway has a 3% uphill grade — one of the steepest in the world",
                "Mountain wave turbulence has flipped unprepared aircraft upside down",
                "The town's elevation causes many first-time visitors to experience altitude sickness",
            ],
        ),
    );

    registry.cities.insert(
        AirportId::Ironforge,
        CityInfo::new(
            "Ironforge",
            "An industrial powerhouse built around vast mines and steel mills. \
             Ironforge is the heavy cargo capital of the network, with enormous \
             freighters landing day and night. The air often carries a metallic tang, \
             and the skyline is dominated by smokestacks and cranes.",
            "Overcast, frequent drizzle and industrial haze",
            "320,000",
            vec![
                "Ironforge Steel Works — guided tours of the massive steel foundry",
                "The Deep Mine Museum — a preserved mine shaft open to visitors",
                "Riverwalk Industrial Park — converted warehouses now home to art galleries",
                "Anvil Stadium — home to the Ironforge Hammers football team",
            ],
            vec![
                "Ironforge meat pie — hearty pastry stuffed with beef and gravy",
                "Forge-fired flatbread — baked on hot steel plates, topped with cheese",
                "Miner's stout — a thick, dark beer brewed with local spring water",
            ],
            vec![
                "Ironforge processes more cargo tonnage than the next three airports combined",
                "The original runway was built on a reclaimed slag heap",
                "Night operations run 24 hours due to the mining industry's demands",
            ],
        ),
    );

    registry.cities.insert(
        AirportId::Cloudmere,
        CityInfo::new(
            "Cloudmere",
            "A foggy coastal village perched on dramatic sea cliffs. Cloudmere is \
             famous for its lighthouse, its fishing fleet, and its perpetual blanket \
             of coastal fog. Pilots must rely heavily on instruments when approaching \
             this airport, as visual conditions are rarely ideal.",
            "Maritime, cool and foggy with frequent drizzle",
            "6,500",
            vec![
                "Cloudmere Lighthouse — a 150-year-old lighthouse still in operation",
                "Whalewatch Point — seasonal whale migration viewing platform",
                "Misty Harbor — a charming fishing village with colorful boats",
                "Cliff Walk Trail — a 5-mile coastal path with stunning ocean views",
            ],
            vec![
                "Cloudmere clam chowder — thick and creamy with fresh-caught clams",
                "Smoked mackerel — cured in the village's traditional smokehouses",
                "Sea-salt caramels — handmade with salt harvested from tidal pools",
            ],
            vec![
                "Cloudmere averages only 45 clear days per year",
                "The lighthouse fog horn plays a melody composed by a former keeper",
                "Local fishermen have guided lost pilots to the runway using boat flares",
            ],
        ),
    );

    registry.cities.insert(
        AirportId::Stormwatch,
        CityInfo::new(
            "Stormwatch",
            "A cutting-edge weather research station in an area notorious for extreme \
             weather. Stormwatch exists because of the storms, not despite them. \
             Scientists and daredevil pilots are drawn here to study and fly through \
             some of the most violent weather on the planet. The facilities feature \
             the most advanced radar and forecasting technology available.",
            "Extreme variability — storms, hail, tornadoes, lightning",
            "4,200",
            vec![
                "Stormwatch Radar Array — the world's most powerful weather radar",
                "Storm Chasers Museum — history of weather research and brave pilots",
                "Lightning Ridge — a ridge where lightning strikes daily during storm season",
                "Wind Tunnel Simulator — open to visitors for a simulated storm experience",
            ],
            vec![
                "Stormwatch chili — fiery hot, designed to warm you after a cold front",
                "Thunder rolls — cinnamon rolls as big as your head, a comfort-food staple",
                "Lightning lemonade — lemonade with a ginger-citrus kick",
            ],
            vec![
                "Stormwatch records an average of 180 thunderstorm days per year",
                "The base has been hit by 3 tornadoes and rebuilt each time",
                "Pilots who complete 20 storm flights earn the unofficial 'Storm Rider' patch",
            ],
        ),
    );

    registry.cities.insert(
        AirportId::Duskhollow,
        CityInfo::new(
            "Duskhollow",
            "A desert oasis city built around an ancient canyon. Duskhollow is \
             surrounded by endless red-rock formations and archaeological sites from \
             a long-lost civilization. The airport sits on a mesa overlooking the canyon, \
             requiring precision approaches. Heat shimmer and sandstorms are constant hazards.",
            "Arid, extreme heat in summer, cool nights",
            "42,000",
            vec![
                "Canyon of Whispers — a natural canyon with incredible acoustics",
                "Ancient Ruins of Solara — mysterious stone structures predating recorded history",
                "Desert Sunset Terrace — the best place to watch the fiery sunsets",
                "Oasis Gardens — lush botanical gardens fed by underground springs",
                "Night Sky Observatory — world-class stargazing in the clear desert air",
            ],
            vec![
                "Duskhollow dates — sweet, sun-dried dates from the oasis groves",
                "Mesa-grilled lamb — seasoned with desert herbs and slow-roasted",
                "Cactus cooler — a refreshing drink made from prickly pear cactus",
            ],
            vec![
                "Duskhollow's runway temperature can exceed 130°F in summer",
                "Archaeological teams have found aviation-themed carvings in the ruins",
                "Mirages on the runway have caused pilots to misread their altitude on approach",
            ],
        ),
    );

    registry.cities.insert(
        AirportId::Skyreach,
        CityInfo::new(
            "Skyreach",
            "The highest airport in the network, perched on a mountainside at \
             12,000 feet. Skyreach is an elite destination accessible only to the \
             most skilled pilots. The thin atmosphere requires longer takeoff rolls \
             and reduced payload. The city itself is a small, exclusive community \
             of aviators, astronomers, and mountaineers.",
            "High altitude, thin air, cold and dry with intense sun",
            "3,200",
            vec![
                "Skyreach Observatory — one of the clearest telescope views on the planet",
                "Summit Lodge — a luxury lodge carved into the mountainside",
                "Pilot's Hall of Fame — honoring those who mastered the Skyreach approach",
                "Cloud Sea Overlook — on clear days, you can see clouds below you",
            ],
            vec![
                "High-altitude espresso — brewed at lower boiling point, uniquely smooth",
                "Summit stew — yak-style meat stew with highland grains",
                "Skyreach honey — from bees that pollinate high-altitude wildflowers",
            ],
            vec![
                "Engine performance drops by 30% at Skyreach's altitude",
                "Only pilots with an Ace rating are cleared to land here",
                "The runway was carved from a natural mountain ledge over 10 years",
                "Skyreach's air pressure is only 65% of sea level",
            ],
        ),
    );

    // Windport uses the same AirportId variant name but the city is called "Coral Cay"
    // in the spec. However, the enum has no Coral_cay variant — it has Windport.
    // The user's spec maps "Coral_cay" → AirportId::Windport since Windport is
    // described as a coastal city. We repurpose Windport for the Coral Cay content.
    registry.cities.insert(
        AirportId::Windport,
        CityInfo::new(
            "Coral Cay",
            "A chain of tropical islands connected by bridges and boat ferries. \
             Coral Cay is home to a seaplane base and a marine sanctuary protecting \
             vibrant coral reefs. The islands attract divers, marine biologists, and \
             adventurous pilots who enjoy water landings. Seasonal typhoons require \
             careful flight planning.",
            "Tropical oceanic, warm year-round with seasonal typhoons",
            "18,000",
            vec![
                "Coral Reef Marine Sanctuary — protected waters with guided snorkel tours",
                "Seaplane Lagoon — a sheltered lagoon used as a natural runway",
                "Lighthouse Island — a remote island with an automated lighthouse and wild ponies",
                "Tidal Pool Gardens — natural pools teeming with colorful sea creatures",
            ],
            vec![
                "Coral Cay lobster roll — fresh lobster in a butter-toasted bun",
                "Seabreeze punch — rum, guava, and lime, served in a coconut shell",
                "Island poke bowl — raw tuna with tropical fruits and rice",
            ],
            vec![
                "Coral Cay's seaplane base handles more water landings than any airport",
                "A resident dolphin pod escorts seaplanes during takeoff runs",
                "Typhoon season grounds all flights for an average of 12 days per year",
            ],
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_airports_have_city_data() {
        let mut registry = CityRegistry::default();
        populate_cities(&mut registry);

        let all_airports = [
            AirportId::HomeBase,
            AirportId::Grandcity,
            AirportId::Sunhaven,
            AirportId::Frostpeak,
            AirportId::Ironforge,
            AirportId::Cloudmere,
            AirportId::Stormwatch,
            AirportId::Duskhollow,
            AirportId::Skyreach,
            AirportId::Windport,
        ];

        for airport in &all_airports {
            let city = registry.cities.get(airport)
                .unwrap_or_else(|| panic!("Missing city data for {:?}", airport));
            assert!(!city.name.is_empty(), "Empty name for {:?}", airport);
            assert!(!city.description.is_empty(), "Empty description for {:?}", airport);
            assert!(city.attractions.len() >= 3, "Need 3+ attractions for {:?}", airport);
            assert!(city.local_cuisine.len() >= 2, "Need 2+ cuisines for {:?}", airport);
            assert!(city.fun_facts.len() >= 2, "Need 2+ fun facts for {:?}", airport);
        }
    }
}

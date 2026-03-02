//! Enhanced NPC schedule data with seasonal variation and high-heart farm visits.
//!
//! This module enriches the base NPC schedules defined in `definitions.rs` by providing:
//! - Season-specific schedule variants (beach in summer, indoors in winter, forest in fall)
//! - Initialization system that overwrites NpcRegistry schedules on game start
//! - A season-change listener that refreshes schedules when the season turns
//! - A `check_farm_visits` system that sends high-heart NPCs to the farm on weekday mornings

use bevy::prelude::*;
use crate::shared::*;

// ═══════════════════════════════════════════════════════════════════════
// PUBLIC API
// ═══════════════════════════════════════════════════════════════════════

/// Returns a seasonally-varied NpcSchedule for the given NPC id.
/// Returns None for unknown NPC ids.
pub fn enhanced_schedule(npc_id: &str, season: Season) -> Option<NpcSchedule> {
    match npc_id {
        "mayor_rex"  => Some(mayor_rex_schedule(season)),
        "margaret"   => Some(margaret_schedule(season)),
        "elena"      => Some(elena_schedule(season)),
        "doc"        => Some(doc_schedule(season)),
        "old_tom"    => Some(old_tom_schedule(season)),
        "marco"      => Some(marco_schedule(season)),
        "sam"        => Some(sam_schedule(season)),
        "mira"       => Some(mira_schedule(season)),
        "nora"       => Some(nora_schedule(season)),
        "lily"       => Some(lily_schedule(season)),
        _            => None,
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SYSTEMS
// ═══════════════════════════════════════════════════════════════════════

/// System: runs once on `OnEnter(GameState::Playing)` and overwrites the NpcRegistry
/// schedules with the enhanced seasonally-varied versions for the current season.
pub fn apply_enhanced_schedules(
    calendar: Res<Calendar>,
    mut npc_registry: ResMut<NpcRegistry>,
) {
    let season = calendar.season;
    let npc_ids: Vec<String> = npc_registry.schedules.keys().cloned().collect();
    for id in npc_ids {
        if let Some(sched) = enhanced_schedule(&id, season) {
            npc_registry.schedules.insert(id, sched);
        }
    }
    info!("[schedules] Applied enhanced {:?} schedules for all NPCs.", season);
}

/// System: listens for `SeasonChangeEvent` and refreshes NPC schedules for the new season.
pub fn refresh_schedules_on_season_change(
    mut season_events: EventReader<SeasonChangeEvent>,
    mut npc_registry: ResMut<NpcRegistry>,
) {
    for ev in season_events.read() {
        let season = ev.new_season;
        let npc_ids: Vec<String> = npc_registry.schedules.keys().cloned().collect();
        for id in npc_ids {
            if let Some(sched) = enhanced_schedule(&id, season) {
                npc_registry.schedules.insert(id, sched);
            }
        }
        info!("[schedules] Refreshed NPC schedules for new season: {:?}", season);
    }
}

/// Resource tracking the per-NPC farm-visit state so we only roll the dice once per day.
#[derive(Resource, Default)]
pub struct FarmVisitTracker {
    /// NPC ids that have already had today's farm-visit roll performed.
    pub rolled_today: Vec<String>,
}

/// System: on weekday mornings before 10 AM, eligible marriage candidates with
/// 8+ hearts (800+ friendship) have a 30% chance to visit the player's farm.
///
/// When the visit is triggered the NPC's first schedule waypoint is overridden to
/// `MapId::Farm` near the shipping bin for this game-tick; the regular schedule
/// system will then move them there.
pub fn check_farm_visits(
    calendar: Res<Calendar>,
    relationships: Res<Relationships>,
    mut npc_registry: ResMut<NpcRegistry>,
    mut tracker: ResMut<FarmVisitTracker>,
) {
    // Only on weekdays before 10 AM
    let is_weekday = !matches!(
        calendar.day_of_week(),
        DayOfWeek::Saturday | DayOfWeek::Sunday,
    );
    let is_morning = calendar.time_float() >= 6.0 && calendar.time_float() < 10.0;

    if !is_weekday || !is_morning {
        // Clear the tracker if the morning window has ended so it resets next day
        if calendar.time_float() >= 10.0 && !tracker.rolled_today.is_empty() {
            tracker.rolled_today.clear();
        }
        return;
    }

    // Collect IDs of all marriageable NPCs with 8+ hearts
    let candidates: Vec<String> = npc_registry
        .npcs
        .iter()
        .filter(|(_, def)| def.is_marriageable)
        .map(|(id, _)| id.clone())
        .collect();

    for npc_id in candidates {
        // Skip if we already rolled for this NPC today
        if tracker.rolled_today.contains(&npc_id) {
            continue;
        }

        let friendship = relationships.friendship.get(&npc_id).copied().unwrap_or(0);
        if friendship < 800 {
            // Not enough hearts yet
            tracker.rolled_today.push(npc_id);
            continue;
        }

        // Mark as rolled regardless of outcome
        tracker.rolled_today.push(npc_id.clone());

        // 30% chance — use a deterministic-ish hash based on day + npc_id to avoid
        // relying on external RNG resources (which may not be available in all states).
        let day_seed = calendar.total_days_elapsed();
        let npc_hash: u32 = npc_id
            .bytes()
            .enumerate()
            .fold(0u32, |acc, (i, b)| {
                acc.wrapping_add((b as u32).wrapping_mul(31u32.wrapping_pow(i as u32)))
            });
        let combined = day_seed.wrapping_add(npc_hash);
        let roll = (combined % 100) as f32; // 0..99

        if roll >= 30.0 {
            // No visit today
            continue;
        }

        // Override the NPC's current schedule so the first active waypoint is the farm.
        // We insert a farm-visit entry at the start of the weekday schedule that covers
        // the morning window (6–9 AM), then let the regular schedule resume at 9 AM.
        if let Some(schedule) = npc_registry.schedules.get_mut(&npc_id) {
            // Build a fresh farm-visit morning block that returns to the original
            // schedule around 9 AM. We prepend our visit entries and keep the rest.
            let mut new_weekday = vec![
                // Walk to farm: shipping bin area
                ScheduleEntry { time: 6.0, map: MapId::Farm, x: 5,  y: 5 },
                // Linger at farm until 9 AM
                ScheduleEntry { time: 7.0, map: MapId::Farm, x: 6,  y: 6 },
                ScheduleEntry { time: 8.0, map: MapId::Farm, x: 5,  y: 7 },
            ];
            // Append any entries from the original schedule that start at 9 AM or later
            for entry in schedule.weekday.iter() {
                if entry.time >= 9.0 {
                    new_weekday.push(entry.clone());
                }
            }
            schedule.weekday = new_weekday;
            info!("[schedules] {} will visit the farm today (30% roll succeeded).", npc_id);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// MAYOR REX — civic leader, ceremonial, formal
// ═══════════════════════════════════════════════════════════════════════

fn mayor_rex_schedule(season: Season) -> NpcSchedule {
    // Afternoon destination varies by season
    let (afternoon_map, afternoon_x, afternoon_y) = match season {
        Season::Summer => (MapId::Beach, 14, 9),
        Season::Winter => (MapId::Town, 14, 10),
        Season::Fall   => (MapId::Forest, 10, 15),
        Season::Spring => (MapId::Town, 18, 14),
    };

    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 22, y: 10 }, // home morning
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 24, y: 18 }, // town hall
            ScheduleEntry { time: 12.0, map: MapId::Town, x: 16, y: 17 }, // lunch bench
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 24, y: 18 }, // back to desk
            ScheduleEntry { time: 16.0, map: afternoon_map, x: afternoon_x, y: afternoon_y }, // seasonal outing
            ScheduleEntry { time: 19.0, map: MapId::Town, x: 22, y: 10 }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 22, y: 10 }, // sleep
        ],
        weekend: vec![
            ScheduleEntry { time: 7.0,  map: MapId::Town, x: 22, y: 10 }, // home — sleeps in
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 18, y: 14 }, // leisurely plaza walk
            ScheduleEntry { time: 11.0, map: MapId::Town, x: 26, y: 18 }, // chatting with townspeople
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 16, y: 17 }, // lunch bench
            ScheduleEntry { time: 15.0, map: afternoon_map, x: afternoon_x, y: afternoon_y }, // seasonal
            ScheduleEntry { time: 18.0, map: MapId::Town, x: 22, y: 10 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 22, y: 10 }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 22, y: 10 }, // home
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 24, y: 18 }, // indoor paperwork
            ScheduleEntry { time: 17.0, map: MapId::Town, x: 22, y: 10 }, // home early
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 22, y: 10 }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 24, y: 15 }, // festival grounds, leading event
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 22, y: 10 }, // home
        ]),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// MARGARET — baker, general store, warm personality
// ═══════════════════════════════════════════════════════════════════════

fn margaret_schedule(season: Season) -> NpcSchedule {
    // After-store evening destination varies by season
    let (eve_map, eve_x, eve_y) = match season {
        Season::Summer => (MapId::Beach, 13, 9),    // beach walk after closing
        Season::Winter => (MapId::Town, 8, 5),      // library warmth
        Season::Fall   => (MapId::Forest, 10, 15),  // admiring the foliage
        Season::Spring => (MapId::Town, 12, 18),    // south garden
    };

    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town,        x: 6,  y: 4  }, // home above store
            ScheduleEntry { time: 8.0,  map: MapId::GeneralStore, x: 5,  y: 8  }, // opens store
            ScheduleEntry { time: 12.0, map: MapId::GeneralStore, x: 5,  y: 6  }, // lunch at counter
            ScheduleEntry { time: 12.5, map: MapId::GeneralStore, x: 5,  y: 8  }, // back to work
            ScheduleEntry { time: 17.0, map: eve_map,            x: eve_x, y: eve_y }, // seasonal evening
            ScheduleEntry { time: 19.0, map: MapId::Town,        x: 22, y: 10 }, // plaza stroll
            ScheduleEntry { time: 20.0, map: MapId::Town,        x: 6,  y: 4  }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town,        x: 6,  y: 4  }, // sleep
        ],
        weekend: vec![
            ScheduleEntry { time: 7.0,  map: MapId::Town, x: 6,  y: 4  }, // home, relaxed morning
            ScheduleEntry { time: 9.0,  map: eve_map,     x: eve_x, y: eve_y }, // seasonal leisure
            ScheduleEntry { time: 11.0, map: MapId::Town, x: 18, y: 14 }, // plaza socializing
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 16, y: 17 }, // lunch bench
            ScheduleEntry { time: 15.0, map: eve_map,     x: eve_x, y: eve_y }, // seasonal afternoon
            ScheduleEntry { time: 19.0, map: MapId::Town, x: 6,  y: 4  }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 6,  y: 4  }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town,        x: 6,  y: 4  }, // home
            ScheduleEntry { time: 9.0,  map: MapId::GeneralStore, x: 5,  y: 8  }, // store all day
            ScheduleEntry { time: 18.0, map: MapId::Town,        x: 6,  y: 4  }, // home early
            ScheduleEntry { time: 22.0, map: MapId::Town,        x: 6,  y: 4  }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 22, y: 16 }, // festival area
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 6,  y: 4  }, // home
        ]),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ELENA — blacksmith, stoic, mine-adjacent, marriage candidate
// ═══════════════════════════════════════════════════════════════════════

fn elena_schedule(season: Season) -> NpcSchedule {
    // His weekend outing changes: beach in summer, forest in fall, stays local otherwise
    let (weekend_afternoon_map, wknd_x, wknd_y) = match season {
        Season::Summer => (MapId::Beach, 14, 10),
        Season::Fall   => (MapId::Forest, 10, 15),
        _              => (MapId::MineEntrance, 4, 10),
    };

    // In winter he heads home after work rather than visiting the mine entrance
    let evening_mine = !matches!(season, Season::Winter);

    let mut weekday = vec![
        ScheduleEntry { time: 6.0,  map: MapId::Town,      x: 23, y: 15 }, // home
        ScheduleEntry { time: 8.0,  map: MapId::Blacksmith, x: 4,  y: 8  }, // forge opens
        ScheduleEntry { time: 12.0, map: MapId::Blacksmith, x: 4,  y: 6  }, // lunch at anvil
        ScheduleEntry { time: 13.0, map: MapId::Blacksmith, x: 4,  y: 8  }, // back to forge
        ScheduleEntry { time: 17.0, map: MapId::Town,       x: 24, y: 18 }, // evening walk
    ];
    if evening_mine {
        weekday.push(ScheduleEntry { time: 19.0, map: MapId::MineEntrance, x: 4, y: 10 }); // checks mine
        weekday.push(ScheduleEntry { time: 21.0, map: MapId::Town, x: 23, y: 15 }); // home
    } else {
        weekday.push(ScheduleEntry { time: 19.0, map: MapId::Town, x: 23, y: 15 }); // home (winter, stays warm)
    }
    weekday.push(ScheduleEntry { time: 22.0, map: MapId::Town, x: 23, y: 15 }); // sleep

    NpcSchedule {
        weekday,
        weekend: vec![
            ScheduleEntry { time: 7.0,  map: MapId::Town,                  x: 23,      y: 15      }, // home
            ScheduleEntry { time: 9.0,  map: weekend_afternoon_map,         x: wknd_x,  y: wknd_y  }, // seasonal outing
            ScheduleEntry { time: 11.0, map: MapId::Town,                  x: 24,      y: 18      }, // south walk
            ScheduleEntry { time: 13.0, map: MapId::Town,                  x: 18,      y: 14      }, // plaza
            ScheduleEntry { time: 15.0, map: MapId::Blacksmith,             x: 4,       y: 8       }, // personal projects
            ScheduleEntry { time: 19.0, map: MapId::Town,                  x: 23,      y: 15      }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town,                  x: 23,      y: 15      }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town,      x: 23, y: 15 }, // home
            ScheduleEntry { time: 9.0,  map: MapId::Blacksmith, x: 4,  y: 8  }, // forge all day
            ScheduleEntry { time: 19.0, map: MapId::Town,       x: 23, y: 15 }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town,       x: 23, y: 15 }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 22, y: 18 }, // festival
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 23, y: 15 }, // home
        ]),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// DOC — physician, herbalist, forest-researcher
// ═══════════════════════════════════════════════════════════════════════

fn doc_schedule(season: Season) -> NpcSchedule {
    // Herb-gathering destination changes by season
    let (herb_map, herb_x, herb_y) = match season {
        Season::Summer => (MapId::Beach, 11, 7),    // seaweed & coastal plants
        Season::Winter => (MapId::Town, 10, 12),    // stays at clinic; too cold outside
        Season::Fall   => (MapId::Forest, 10, 9),   // mushrooms and late-season herbs
        Season::Spring => (MapId::Forest, 8, 9),    // spring herbs
    };

    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 10, y: 8  }, // home
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 10, y: 12 }, // clinic opens
            ScheduleEntry { time: 12.0, map: MapId::Town, x: 10, y: 10 }, // lunch at clinic
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 10, y: 12 }, // afternoon clinic
            ScheduleEntry { time: 16.0, map: herb_map,   x: herb_x, y: herb_y }, // herb gathering
            ScheduleEntry { time: 18.0, map: MapId::Town, x: 10, y: 8  }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 10, y: 8  }, // sleep
        ],
        weekend: vec![
            ScheduleEntry { time: 7.0,  map: MapId::Town, x: 10, y: 8  }, // home
            ScheduleEntry { time: 9.0,  map: herb_map,   x: herb_x, y: herb_y }, // herb research
            ScheduleEntry { time: 12.0, map: herb_map,   x: herb_x.saturating_add(4), y: herb_y.saturating_add(6) }, // deeper
            ScheduleEntry { time: 14.0, map: MapId::Town, x: 18, y: 14 }, // plaza fresh air
            ScheduleEntry { time: 16.0, map: MapId::Town, x: 10, y: 12 }, // clinic paperwork
            ScheduleEntry { time: 20.0, map: MapId::Town, x: 10, y: 8  }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 10, y: 8  }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 10, y: 8  }, // home
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 10, y: 12 }, // busy rainy-day clinic
            ScheduleEntry { time: 20.0, map: MapId::Town, x: 10, y: 8  }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 10, y: 8  }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 22, y: 16 }, // festival medical standby
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 10, y: 8  }, // home
        ]),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// OLD TOM — veteran fisherman, dawn riser, loves bad weather
// ═══════════════════════════════════════════════════════════════════════

fn old_tom_schedule(season: Season) -> NpcSchedule {
    // Fishing spots slightly vary by season
    let (primary_x, primary_y, secondary_x, secondary_y) = match season {
        Season::Summer => (14, 10, 18, 10), // prefers far end of beach
        Season::Winter => (14, 9, 12, 8),   // huddled closer to shore
        Season::Fall   => (16, 11, 14, 10), // back from summer spots
        Season::Spring => (16, 10, 14, 8),  // comfortable spots
    };

    // In winter he heads to town at noon rather than staying all day
    let noon_map = if matches!(season, Season::Winter) { MapId::Town } else { MapId::Beach };
    let (noon_x, noon_y) = if matches!(season, Season::Winter) { (18, 14) } else { (16, 9) };

    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 5.0,  map: MapId::Beach, x: primary_x,   y: primary_y   }, // dawn fishing
            ScheduleEntry { time: 6.0,  map: MapId::Beach, x: primary_x,   y: primary_y   }, // pier spot
            ScheduleEntry { time: 12.0, map: noon_map,     x: noon_x,      y: noon_y      }, // lunch
            ScheduleEntry { time: 13.0, map: MapId::Beach, x: secondary_x, y: secondary_y }, // afternoon spot
            ScheduleEntry { time: 17.0, map: MapId::Beach, x: 16,          y: 11          }, // evening campfire area
            ScheduleEntry { time: 20.0, map: MapId::Town,  x: 8,           y: 14          }, // home near docks
            ScheduleEntry { time: 21.0, map: MapId::Town,  x: 8,           y: 14          }, // sleep
        ],
        weekend: vec![
            ScheduleEntry { time: 5.0,  map: MapId::Beach, x: primary_x,   y: primary_y   }, // very early
            ScheduleEntry { time: 6.0,  map: MapId::Beach, x: secondary_x, y: secondary_y }, // weekend spot
            ScheduleEntry { time: 11.0, map: MapId::Town,  x: 18,          y: 14          }, // town — sells fish
            ScheduleEntry { time: 13.0, map: MapId::Beach, x: primary_x,   y: primary_y   }, // back fishing
            ScheduleEntry { time: 18.0, map: MapId::Beach, x: 16,          y: 12          }, // watching sunset
            ScheduleEntry { time: 20.0, map: MapId::Town,  x: 8,           y: 14          }, // home
            ScheduleEntry { time: 21.0, map: MapId::Town,  x: 8,           y: 14          }, // sleep
        ],
        // Pete LOVES fishing in the rain — no change from base
        rain_override: Some(vec![
            ScheduleEntry { time: 5.0,  map: MapId::Beach, x: primary_x,   y: primary_y   }, // best fishing weather!
            ScheduleEntry { time: 12.0, map: MapId::Beach, x: 16,          y: 12          }, // still won't leave
            ScheduleEntry { time: 20.0, map: MapId::Town,  x: 8,           y: 14          }, // finally home
            ScheduleEntry { time: 21.0, map: MapId::Town,  x: 8,           y: 14          }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 18, y: 18 }, // festival fishing contest
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 8,  y: 14 }, // home
        ]),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// MARCO — chef, restaurateur, ingredient-hunter, farm visitor
// ═══════════════════════════════════════════════════════════════════════

fn marco_schedule(season: Season) -> NpcSchedule {
    // She sources seasonal ingredients differently each season
    let (ingredient_hunt_map, ing_x, ing_y) = match season {
        Season::Summer => (MapId::Beach, 14, 9),    // sea vegetables, clams
        Season::Fall   => (MapId::Forest, 10, 15),  // mushrooms, nuts
        Season::Winter => (MapId::Town, 17, 17),    // indoor prep (no fresh sources)
        Season::Spring => (MapId::Farm, 20, 20),    // visits farm for spring produce
    };

    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 5.0,  map: MapId::Town,                x: 18, y: 17 }, // home (early cook rise)
            ScheduleEntry { time: 6.0,  map: ingredient_hunt_map,         x: ing_x, y: ing_y }, // seasonal ingredient run
            ScheduleEntry { time: 8.0,  map: MapId::Town,                x: 16, y: 16 }, // restaurant prep
            ScheduleEntry { time: 10.0, map: MapId::Town,                x: 17, y: 17 }, // lunch service open
            ScheduleEntry { time: 14.0, map: MapId::Town,                x: 16, y: 16 }, // afternoon prep
            ScheduleEntry { time: 17.0, map: MapId::Town,                x: 17, y: 17 }, // dinner service
            ScheduleEntry { time: 21.0, map: MapId::Town,                x: 18, y: 17 }, // home / cleanup
            ScheduleEntry { time: 23.0, map: MapId::Town,                x: 18, y: 17 }, // sleep
        ],
        weekend: vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town,                x: 18, y: 17 }, // home
            ScheduleEntry { time: 8.0,  map: ingredient_hunt_map,         x: ing_x, y: ing_y }, // ingredient run
            ScheduleEntry { time: 10.0, map: MapId::Town,                x: 17, y: 17 }, // brunch service
            ScheduleEntry { time: 14.0, map: MapId::Town,                x: 18, y: 14 }, // plaza break
            ScheduleEntry { time: 16.0, map: MapId::Town,                x: 17, y: 17 }, // dinner service
            ScheduleEntry { time: 21.0, map: MapId::Town,                x: 18, y: 17 }, // home
            ScheduleEntry { time: 23.0, map: MapId::Town,                x: 18, y: 17 }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 18, y: 17 }, // home
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 16, y: 16 }, // restaurant all day (shelter draw)
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 18, y: 17 }, // home
            ScheduleEntry { time: 23.0, map: MapId::Town, x: 18, y: 17 }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 22, y: 14 }, // festival food stall
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 18, y: 17 }, // home
        ]),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// SAM — musician, adventurous spirit, near mine area
// ═══════════════════════════════════════════════════════════════════════

fn sam_schedule(season: Season) -> NpcSchedule {
    // In summer he takes a beach break on weekends; in fall he visits the forest
    let (weekend_pm_map, wknd_pm_x, wknd_pm_y) = match season {
        Season::Summer => (MapId::Beach, 14, 9),
        Season::Fall   => (MapId::Forest, 10, 15),
        _              => (MapId::MineEntrance, 6, 10),
    };

    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town,        x: 11, y: 14 }, // home
            ScheduleEntry { time: 7.0,  map: MapId::MineEntrance, x: 6,  y: 10 }, // on duty
            ScheduleEntry { time: 12.0, map: MapId::MineEntrance, x: 4,  y: 8  }, // lunch at entrance
            ScheduleEntry { time: 13.0, map: MapId::MineEntrance, x: 6,  y: 10 }, // back on post
            ScheduleEntry { time: 17.0, map: MapId::Town,        x: 18, y: 14 }, // plaza walk
            ScheduleEntry { time: 19.0, map: MapId::Town,        x: 11, y: 14 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Town,        x: 11, y: 14 }, // sleep
        ],
        weekend: vec![
            ScheduleEntry { time: 7.0,  map: MapId::Town,        x: 11, y: 14 }, // home
            ScheduleEntry { time: 9.0,  map: MapId::MineEntrance, x: 6,  y: 10 }, // quick check
            ScheduleEntry { time: 11.0, map: MapId::Town,        x: 24, y: 18 }, // south walk
            ScheduleEntry { time: 13.0, map: MapId::Town,        x: 18, y: 14 }, // plaza socializing
            ScheduleEntry { time: 15.0, map: weekend_pm_map,     x: wknd_pm_x, y: wknd_pm_y }, // seasonal outing
            ScheduleEntry { time: 19.0, map: MapId::Town,        x: 11, y: 14 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Town,        x: 11, y: 14 }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town,        x: 11, y: 14 }, // home
            ScheduleEntry { time: 9.0,  map: MapId::MineEntrance, x: 6,  y: 10 }, // shelters at mine entrance
            ScheduleEntry { time: 19.0, map: MapId::Town,        x: 11, y: 14 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Town,        x: 11, y: 14 }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 22, y: 18 }, // festival
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 11, y: 14 }, // home
        ]),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// MIRA — merchant, scholar, ruin-hunter, loves rainy days
// ═══════════════════════════════════════════════════════════════════════

fn mira_schedule(season: Season) -> NpcSchedule {
    // Her outdoor research location depends on season
    let (research_map, res_x, res_y) = match season {
        Season::Summer => (MapId::Beach, 15, 9),    // coastal ruins interest
        Season::Fall   => (MapId::Forest, 10, 15),  // ancient forest markers
        Season::Winter => (MapId::Town, 8, 20),     // stays in library (too cold)
        Season::Spring => (MapId::Forest, 16, 10),  // spring: forest ruins
    };

    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 8,  y: 16 }, // home
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 8,  y: 18 }, // library opens
            ScheduleEntry { time: 12.0, map: MapId::Town, x: 8,  y: 18 }, // lunch in back room
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 8,  y: 18 }, // library afternoon
            ScheduleEntry { time: 16.0, map: research_map, x: res_x, y: res_y }, // seasonal research
            ScheduleEntry { time: 18.0, map: MapId::Town, x: 8,  y: 16 }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 8,  y: 16 }, // sleep (reads late)
        ],
        weekend: vec![
            ScheduleEntry { time: 8.0,  map: MapId::Town,   x: 8,  y: 16 }, // home
            ScheduleEntry { time: 10.0, map: research_map,  x: res_x, y: res_y }, // field research
            ScheduleEntry { time: 13.0, map: MapId::Town,   x: 8,  y: 18 }, // library analysis
            ScheduleEntry { time: 16.0, map: MapId::Town,   x: 18, y: 14 }, // plaza, people-watching
            ScheduleEntry { time: 18.0, map: MapId::Town,   x: 8,  y: 16 }, // home
            ScheduleEntry { time: 22.0, map: MapId::Town,   x: 8,  y: 16 }, // sleep
        ],
        // Faye loves rainy days — stays at library very late
        rain_override: Some(vec![
            ScheduleEntry { time: 6.0,  map: MapId::Town, x: 8,  y: 16 }, // home
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 8,  y: 18 }, // library all day (pure joy)
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 8,  y: 16 }, // home very late
            ScheduleEntry { time: 23.0, map: MapId::Town, x: 8,  y: 16 }, // sleep extra late
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 22, y: 16 }, // history display at festival
            ScheduleEntry { time: 22.0, map: MapId::Town, x: 8,  y: 16 }, // home
        ]),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// NORA — veteran farmer, south plots, market-goer
// ═══════════════════════════════════════════════════════════════════════

fn nora_schedule(season: Season) -> NpcSchedule {
    // Dale adjusts his farm work for season: light work in winter, heavy in spring/summer
    let farm_start_time = match season {
        Season::Winter => 8.0,  // light winter chores
        _             => 6.0,  // early start in growing seasons
    };

    // In fall he also visits the forest to check on wild produce
    let (afternoon_outing_map, aft_x, aft_y) = match season {
        Season::Fall   => (MapId::Forest, 10, 15),
        Season::Summer => (MapId::Beach, 12, 9),    // cooling off
        _              => (MapId::Farm, 16, 14),    // still farming
    };

    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 5.0,               map: MapId::Farm, x: 14, y: 14 }, // home (early riser)
            ScheduleEntry { time: farm_start_time,   map: MapId::Farm, x: 16, y: 14 }, // south plots
            ScheduleEntry { time: 12.0,              map: MapId::Farm, x: 13, y: 13 }, // lunch under tree
            ScheduleEntry { time: 13.0,              map: MapId::Farm, x: 16, y: 14 }, // afternoon farming
            ScheduleEntry { time: 16.0,              map: afternoon_outing_map, x: aft_x, y: aft_y }, // seasonal outing
            ScheduleEntry { time: 18.0,              map: MapId::Town, x: 18, y: 18 }, // south town
            ScheduleEntry { time: 20.0,              map: MapId::Farm, x: 14, y: 14 }, // home
            ScheduleEntry { time: 21.0,              map: MapId::Farm, x: 14, y: 14 }, // sleep
        ],
        weekend: vec![
            ScheduleEntry { time: 5.0,  map: MapId::Farm, x: 14, y: 14 }, // home
            ScheduleEntry { time: 7.0,  map: MapId::Farm, x: 12, y: 13 }, // inspecting player's area
            ScheduleEntry { time: 10.0, map: MapId::Town, x: 18, y: 18 }, // town market
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 18, y: 14 }, // plaza rest
            ScheduleEntry { time: 15.0, map: afternoon_outing_map, x: aft_x, y: aft_y }, // seasonal
            ScheduleEntry { time: 19.0, map: MapId::Farm, x: 14, y: 14 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Farm, x: 14, y: 14 }, // sleep
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 5.0,  map: MapId::Farm, x: 14, y: 14 }, // home
            ScheduleEntry { time: 8.0,  map: MapId::Farm, x: 16, y: 14 }, // light rain work (rain is good!)
            ScheduleEntry { time: 12.0, map: MapId::Farm, x: 14, y: 14 }, // inside for heavy rain
            ScheduleEntry { time: 20.0, map: MapId::Farm, x: 14, y: 14 }, // stay home
            ScheduleEntry { time: 21.0, map: MapId::Farm, x: 14, y: 14 }, // sleep
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 22, y: 18 }, // festival harvest display
            ScheduleEntry { time: 21.0, map: MapId::Farm, x: 14, y: 14 }, // home
        ]),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// LILY — florist, energetic, everywhere, bedtime enforced
// ═══════════════════════════════════════════════════════════════════════

fn lily_schedule(season: Season) -> NpcSchedule {
    // Lily's adventures change dramatically by season
    let (mid_morning_map, mid_x, mid_y) = match season {
        Season::Summer => (MapId::Beach, 12, 8),    // beach! beach! beach!
        Season::Fall   => (MapId::Forest, 6, 8),    // crunchy leaves
        Season::Winter => (MapId::Town, 8, 20),     // library (mom's orders)
        Season::Spring => (MapId::Town, 8, 20),     // near library oak tree
    };

    let (afternoon_map, aft_x, aft_y) = match season {
        Season::Summer => (MapId::Beach, 10, 7),    // sand castles
        Season::Fall   => (MapId::Forest, 10, 12),  // deeper forest
        Season::Winter => (MapId::Town, 18, 17),    // south park (brief outdoor time)
        Season::Spring => (MapId::Town, 18, 17),    // south park
    };

    NpcSchedule {
        weekday: vec![
            ScheduleEntry { time: 7.0,  map: MapId::Town, x: 10,  y: 14 }, // home
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 22,  y: 18 }, // morning run in plaza
            ScheduleEntry { time: 9.0,  map: mid_morning_map, x: mid_x, y: mid_y }, // seasonal adventure
            ScheduleEntry { time: 11.0, map: afternoon_map,   x: aft_x, y: aft_y }, // mid-morning play
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 10,  y: 14 }, // lunch at home
            ScheduleEntry { time: 14.0, map: MapId::Town, x: 26,  y: 18 }, // east plaza exploring
            ScheduleEntry { time: 16.0, map: MapId::Town, x: 22,  y: 14 }, // afternoon in plaza
            ScheduleEntry { time: 18.0, map: MapId::Town, x: 10,  y: 14 }, // home for dinner
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 10,  y: 14 }, // bedtime
        ],
        weekend: vec![
            ScheduleEntry { time: 8.0,  map: MapId::Town,        x: 10, y: 14 }, // home (sleeps in)
            ScheduleEntry { time: 9.0,  map: MapId::Town,        x: 22, y: 14 }, // plaza run
            ScheduleEntry { time: 10.0, map: mid_morning_map,    x: mid_x, y: mid_y }, // seasonal adventure
            ScheduleEntry { time: 12.0, map: afternoon_map,      x: aft_x, y: aft_y }, // big adventure
            ScheduleEntry { time: 14.0, map: afternoon_map,      x: aft_x.saturating_add(4), y: aft_y.saturating_add(4) }, // farther
            ScheduleEntry { time: 16.0, map: MapId::Town,        x: 18, y: 14 }, // back to town (tired)
            ScheduleEntry { time: 18.0, map: MapId::Town,        x: 10, y: 14 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Town,        x: 10, y: 14 }, // bedtime (non-negotiable)
        ],
        rain_override: Some(vec![
            ScheduleEntry { time: 7.0,  map: MapId::Town, x: 10, y: 14 }, // home (mom says no)
            ScheduleEntry { time: 9.0,  map: MapId::Town, x: 8,  y: 18 }, // library (forced)
            ScheduleEntry { time: 13.0, map: MapId::Town, x: 10, y: 14 }, // lunch
            ScheduleEntry { time: 14.0, map: MapId::Town, x: 8,  y: 18 }, // library again (sulking)
            ScheduleEntry { time: 18.0, map: MapId::Town, x: 10, y: 14 }, // home
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 10, y: 14 }, // bedtime
        ]),
        festival_override: Some(vec![
            ScheduleEntry { time: 8.0,  map: MapId::Town, x: 22, y: 14 }, // first one there, every time
            ScheduleEntry { time: 21.0, map: MapId::Town, x: 10, y: 14 }, // forced bedtime even on festival
        ]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_schedule_returns_some_for_all_known_npcs() {
        let npc_ids = [
            "mayor_rex", "margaret", "elena", "doc", "old_tom",
            "marco", "sam", "mira", "nora", "lily",
        ];
        for season in [Season::Spring, Season::Summer, Season::Fall, Season::Winter] {
            for id in &npc_ids {
                assert!(
                    enhanced_schedule(id, season).is_some(),
                    "enhanced_schedule should return Some for {} in {:?}",
                    id, season
                );
            }
        }
    }

    #[test]
    fn test_enhanced_schedule_returns_none_for_unknown() {
        assert!(enhanced_schedule("nonexistent_npc", Season::Spring).is_none());
        assert!(enhanced_schedule("", Season::Summer).is_none());
    }

    #[test]
    fn test_schedules_have_non_empty_weekday_and_weekend() {
        let npc_ids = [
            "mayor_rex", "margaret", "elena", "doc", "old_tom",
            "marco", "sam", "mira", "nora", "lily",
        ];
        for id in &npc_ids {
            let sched = enhanced_schedule(id, Season::Spring).unwrap();
            assert!(!sched.weekday.is_empty(), "{} weekday should not be empty", id);
            assert!(!sched.weekend.is_empty(), "{} weekend should not be empty", id);
        }
    }

    #[test]
    fn test_schedules_have_rain_and_festival_overrides() {
        let npc_ids = [
            "mayor_rex", "margaret", "elena", "doc", "old_tom",
            "marco", "sam", "mira", "nora", "lily",
        ];
        for id in &npc_ids {
            let sched = enhanced_schedule(id, Season::Summer).unwrap();
            assert!(sched.rain_override.is_some(), "{} should have rain_override", id);
            assert!(sched.festival_override.is_some(), "{} should have festival_override", id);
        }
    }

    #[test]
    fn test_old_tom_beach_in_summer() {
        let sched = enhanced_schedule("old_tom", Season::Summer).unwrap();
        let has_beach = sched.weekday.iter().any(|e| e.map == MapId::Beach);
        assert!(has_beach, "Old Tom should visit Beach in summer weekdays");
    }

    #[test]
    fn test_margaret_seasonal_variation() {
        let _spring = enhanced_schedule("margaret", Season::Spring).unwrap();
        let summer = enhanced_schedule("margaret", Season::Summer).unwrap();
        // In summer, Margaret's evening destination should differ from spring
        // Both end at home, but the seasonal evening entry (index 4) should differ
        let summer_eve = &summer.weekday[4];
        // Summer should be at Beach
        assert_eq!(summer_eve.map, MapId::Beach,
            "Margaret should visit Beach in summer evenings");
    }
}

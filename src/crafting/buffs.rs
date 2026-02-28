use bevy::prelude::*;
use crate::shared::*;

// ──────────────────────────────────────────────────────────────────────────────
// CONSTANTS
// ──────────────────────────────────────────────────────────────────────────────

/// Default player movement speed when no Speed buff is active.
const DEFAULT_PLAYER_SPEED: f32 = 80.0;

// ──────────────────────────────────────────────────────────────────────────────
// HELPER: FOOD BUFF LOOKUP
// ──────────────────────────────────────────────────────────────────────────────

/// Returns the FoodBuff that a given food item grants, if any.
/// Pure data mapping — not a Bevy system.
pub fn food_buff_for_item(item_id: &str) -> Option<FoodBuff> {
    match item_id {
        "fried_egg" => Some(FoodBuff {
            buff_type: BuffType::Farming,
            magnitude: 1.2,
            minutes_remaining: 120,
        }),
        "pancakes" => Some(FoodBuff {
            buff_type: BuffType::Speed,
            magnitude: 1.15,
            minutes_remaining: 90,
        }),
        "fish_stew" => Some(FoodBuff {
            buff_type: BuffType::Fishing,
            magnitude: 1.3,
            minutes_remaining: 180,
        }),
        "miners_meal" => Some(FoodBuff {
            buff_type: BuffType::Mining,
            magnitude: 1.25,
            minutes_remaining: 150,
        }),
        "hearty_stew" => Some(FoodBuff {
            buff_type: BuffType::MaxStamina,
            magnitude: 30.0,
            minutes_remaining: 240,
        }),
        "spicy_pepper" => Some(FoodBuff {
            buff_type: BuffType::Attack,
            magnitude: 1.3,
            minutes_remaining: 120,
        }),
        "garden_salad" => Some(FoodBuff {
            buff_type: BuffType::Defense,
            magnitude: 1.2,
            minutes_remaining: 120,
        }),
        "lucky_lunch" => Some(FoodBuff {
            buff_type: BuffType::Luck,
            magnitude: 1.5,
            minutes_remaining: 180,
        }),
        "fruit_salad" => Some(FoodBuff {
            buff_type: BuffType::Speed,
            magnitude: 1.1,
            minutes_remaining: 60,
        }),
        "porridge" => Some(FoodBuff {
            buff_type: BuffType::Farming,
            magnitude: 1.1,
            minutes_remaining: 90,
        }),
        // Plain food — stamina only, no buff
        "toast" | "bread" => None,
        _ => None,
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// HELPER: QUERY ACTIVE BUFF MAGNITUDE
// ──────────────────────────────────────────────────────────────────────────────

/// Returns the magnitude of an active buff of the given type, or 0.0 if none is
/// currently active. Other domains can call this to read buff modifiers.
pub fn get_buff_magnitude(buffs: &ActiveBuffs, buff_type: BuffType) -> f32 {
    buffs
        .buffs
        .iter()
        .find(|b| b.buff_type == buff_type)
        .map(|b| b.magnitude)
        .unwrap_or(0.0)
}

// ──────────────────────────────────────────────────────────────────────────────
// SYSTEM 1 — handle_eat_food
// ──────────────────────────────────────────────────────────────────────────────

/// Reads EatFoodEvent. Removes one unit of the item from the player's
/// inventory, restores stamina, applies any buff (replacing an existing buff of
/// the same type), and sends UI feedback.
pub fn handle_eat_food(
    mut eat_events: EventReader<EatFoodEvent>,
    mut inventory: ResMut<Inventory>,
    mut player_state: ResMut<PlayerState>,
    mut active_buffs: ResMut<ActiveBuffs>,
    item_registry: Res<ItemRegistry>,
    mut toast_events: EventWriter<ToastEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
) {
    for event in eat_events.read() {
        let item_id = &event.item_id;

        // Verify the player has the item before consuming it.
        if !inventory.has(item_id, 1) {
            warn!("handle_eat_food: item '{}' not found in inventory", item_id);
            continue;
        }

        // Consume one unit.
        inventory.try_remove(item_id, 1);

        // Restore stamina, capped at max_stamina.
        let before = player_state.stamina;
        let restored = event.stamina_restore.max(0.0);
        player_state.stamina = (player_state.stamina + restored).min(player_state.max_stamina);
        let actual_restore = player_state.stamina - before;

        info!(
            "Player ate '{}': stamina {:.0} -> {:.0} (+{:.0})",
            item_id, before, player_state.stamina, actual_restore
        );

        // Apply buff if present.
        if let Some(new_buff) = event.buff.clone() {
            // Remove any existing buff of the same type before inserting the
            // new one so a fresh duration/magnitude always wins.
            active_buffs
                .buffs
                .retain(|b| b.buff_type != new_buff.buff_type);

            let buff_label = buff_type_label(new_buff.buff_type);
            let minutes = new_buff.minutes_remaining;

            active_buffs.buffs.push(new_buff);

            info!(
                "Buff '{}' applied for {} game-minutes",
                buff_label, minutes
            );

            let item_name = item_display_name(&item_registry, item_id);
            toast_events.send(ToastEvent {
                message: format!(
                    "Ate {}! {} for {}m",
                    item_name, buff_label, minutes
                ),
                duration_secs: 3.0,
            });
        } else {
            // No buff — show stamina restore toast.
            let item_name = item_display_name(&item_registry, item_id);
            toast_events.send(ToastEvent {
                message: format!("Ate {}! +{:.0} stamina", item_name, actual_restore),
                duration_secs: 3.0,
            });
        }

        sfx_events.send(PlaySfxEvent {
            sfx_id: "eat".to_string(),
        });
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// SYSTEM 2 — tick_buff_durations
// ──────────────────────────────────────────────────────────────────────────────

/// Decrements `minutes_remaining` on every active buff once per game-minute.
/// Expired buffs are removed and the player receives a toast notification.
pub fn tick_buff_durations(
    calendar: Res<Calendar>,
    mut active_buffs: ResMut<ActiveBuffs>,
    mut toast_events: EventWriter<ToastEvent>,
    mut last_minute: Local<u32>,
) {
    if active_buffs.buffs.is_empty() {
        // Nothing to tick — update the baseline and return early.
        *last_minute = calendar_absolute_minute(&calendar);
        return;
    }

    let current_minute = calendar_absolute_minute(&calendar);

    // How many game-minutes have elapsed since the last frame we processed?
    let elapsed = if current_minute >= *last_minute {
        current_minute - *last_minute
    } else {
        // Handle wrap-around (new day / year rollover).
        1
    };

    if elapsed == 0 {
        return;
    }

    *last_minute = current_minute;

    let mut expired_labels: Vec<String> = Vec::new();

    for buff in active_buffs.buffs.iter_mut() {
        let decrement = elapsed.min(buff.minutes_remaining as u32) as u32;
        buff.minutes_remaining = buff.minutes_remaining.saturating_sub(decrement as u32);
        if buff.minutes_remaining == 0 {
            expired_labels.push(buff_type_label(buff.buff_type).to_string());
        }
    }

    // Remove expired buffs.
    active_buffs
        .buffs
        .retain(|b| b.minutes_remaining > 0);

    // Notify the player for each expired buff.
    for label in expired_labels {
        info!("Buff '{}' expired.", label);
        toast_events.send(ToastEvent {
            message: format!("Your {} buff wore off.", label),
            duration_secs: 3.0,
        });
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// SYSTEM 3 — apply_buff_effects
// ──────────────────────────────────────────────────────────────────────────────

/// Applies or removes the real-time mechanical effects of active buffs.
///
/// * Speed  — directly writes `PlayerMovement.speed` on the player entity.
/// * MaxStamina — adjusts `PlayerState.max_stamina`; tracks the original value
///   in a `Local` so that it can be restored once the buff expires.
///
/// Mining / Fishing / Farming / Defense / Attack / Luck are passive modifiers
/// that other systems read via `get_buff_magnitude`. No direct mutation is
/// needed here for those types — they are served on demand.
pub fn apply_buff_effects(
    active_buffs: Res<ActiveBuffs>,
    mut player_query: Query<&mut PlayerMovement, With<Player>>,
    mut player_state: ResMut<PlayerState>,
    mut original_max_stamina: Local<Option<f32>>,
) {
    // ── Speed buff ───────────────────────────────────────────────────────────
    let speed_magnitude = get_buff_magnitude(&active_buffs, BuffType::Speed);
    for mut movement in player_query.iter_mut() {
        if speed_magnitude > 0.0 {
            // Apply the multiplier to the baseline speed.
            movement.speed = DEFAULT_PLAYER_SPEED * speed_magnitude;
        } else {
            // No speed buff — restore to default if we were boosted.
            if (movement.speed - DEFAULT_PLAYER_SPEED).abs() > 0.01 {
                movement.speed = DEFAULT_PLAYER_SPEED;
            }
        }
    }

    // ── MaxStamina buff ──────────────────────────────────────────────────────
    let max_stamina_bonus = get_buff_magnitude(&active_buffs, BuffType::MaxStamina);

    if max_stamina_bonus > 0.0 {
        // Store the un-buffed max_stamina the first time we see this buff.
        if original_max_stamina.is_none() {
            *original_max_stamina = Some(player_state.max_stamina);
        }
        let base = original_max_stamina.expect("set to Some on line above");
        let desired = base + max_stamina_bonus;
        // Only update when the value actually needs changing to avoid
        // constantly dirtying the resource.
        if (player_state.max_stamina - desired).abs() > 0.01 {
            player_state.max_stamina = desired;
        }
    } else {
        // Buff is no longer active — restore original max_stamina if needed.
        if let Some(original) = *original_max_stamina {
            if (player_state.max_stamina - original).abs() > 0.01 {
                // Clamp current stamina so it doesn't exceed the restored max.
                player_state.stamina = player_state.stamina.min(original);
                player_state.max_stamina = original;
            }
            *original_max_stamina = None;
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// PRIVATE HELPERS
// ──────────────────────────────────────────────────────────────────────────────

/// Converts the calendar time into a monotonically increasing minute counter
/// suitable for diff-based elapsed-minute detection.
///
/// Formula: day_within_year * 24 * 60  +  hour * 60  +  minute
/// (Days are 1-indexed so we subtract 1 to make the arithmetic clean.)
fn calendar_absolute_minute(cal: &Calendar) -> u32 {
    // Season index (0-3) contributes 28 days per season.
    let season_idx = match cal.season {
        Season::Spring => 0u32,
        Season::Summer => 1,
        Season::Fall   => 2,
        Season::Winter => 3,
    };
    let days_in_year = season_idx * 28 + (cal.day as u32).saturating_sub(1);
    let year_minutes = (cal.year.saturating_sub(1)) * 4 * 28 * 24 * 60;
    year_minutes + days_in_year * 24 * 60 + cal.hour as u32 * 60 + cal.minute as u32
}

/// Returns a human-readable label for a BuffType.
fn buff_type_label(buff_type: BuffType) -> &'static str {
    match buff_type {
        BuffType::Speed      => "Speed",
        BuffType::Mining     => "Mining",
        BuffType::Fishing    => "Fishing",
        BuffType::Farming    => "Farming",
        BuffType::Defense    => "Defense",
        BuffType::Attack     => "Attack",
        BuffType::Luck       => "Luck",
        BuffType::MaxStamina => "Max Stamina",
    }
}

/// Returns the display name of an item from the registry, falling back to the
/// raw item_id if no registry entry exists.
fn item_display_name<'a>(registry: &'a ItemRegistry, item_id: &'a str) -> String {
    registry
        .get(item_id)
        .map(|def| def.name.clone())
        .unwrap_or_else(|| {
            // Capitalise each word separated by underscores as a readable fallback.
            item_id
                .split('_')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => {
                            first.to_uppercase().collect::<String>() + chars.as_str()
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
}

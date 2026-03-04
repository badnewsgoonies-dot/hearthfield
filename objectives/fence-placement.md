# Worker: Fence & Scarecrow Placement Pipeline (Integration)

## Context
Players can craft fences and scarecrows but can't place them on the farm. The data structures already exist:
- `FarmObject::Fence` and `FarmObject::Scarecrow` are defined in shared/mod.rs
- `FarmState.objects: HashMap<(i32,i32), FarmObject>` stores placed objects (serialized)
- `sync_farm_objects_sprites` in farming/render.rs renders Sprinkler and Scarecrow entities — but NOT Fence
- `item_use.rs` in player/ has placement handlers for sprinklers and machines, but NOT for fence/scarecrow

## Scope
You may modify files under BOTH:
- `src/farming/` (event definition, handler, render extension)
- `src/player/` (send the event from item_use.rs)

Do NOT modify src/shared/mod.rs, src/world/, src/ui/, or any other domain.

## Required reading (in order)
1. `src/farming/render.rs` — `sync_farm_objects_sprites`, `farm_object_atlas_index`, understand the spawn pattern
2. `src/farming/mod.rs` — see how PlaceSprinklerEvent is used, how systems are registered
3. `src/player/item_use.rs` — see the existing placement dispatch (sprinklers, machines)
4. `src/shared/mod.rs` — READ ONLY. Find `FarmObject` enum, `FarmState`, `PlaceSprinklerEvent`, `TILE_SIZE`

## Task

### Step 1: Define PlaceFarmObjectEvent in src/farming/mod.rs

Add near the top of the file (after existing use statements):
```rust
/// Event to place a farm object (fence, scarecrow, etc.) at a grid position.
#[derive(Event, Debug, Clone)]
pub struct PlaceFarmObjectEvent {
    pub item_id: String,
    pub grid_x: i32,
    pub grid_y: i32,
}
```

Register the event in the plugin:
```rust
.add_event::<PlaceFarmObjectEvent>()
```

Export it:
```rust
pub use self::PlaceFarmObjectEvent; // or add to existing pub use block
```

### Step 2: Add handler in src/farming/mod.rs (or a new file src/farming/placement.rs)

```rust
pub fn handle_place_farm_object(
    mut events: EventReader<PlaceFarmObjectEvent>,
    mut farm_state: ResMut<FarmState>,
    mut inventory: ResMut<Inventory>,
    mut toast_events: EventWriter<ToastEvent>,
    mut sfx_events: EventWriter<PlaySfxEvent>,
    player_state: Res<PlayerState>,
) {
    for ev in events.read() {
        // Only on Farm map
        if player_state.current_map != MapId::Farm {
            toast_events.send(ToastEvent {
                message: "You can only place this on the farm.".into(),
                duration_secs: 2.0,
            });
            continue;
        }

        let pos = (ev.grid_x, ev.grid_y);

        // Tile must not be occupied
        if farm_state.objects.contains_key(&pos) {
            toast_events.send(ToastEvent {
                message: "That tile is already occupied.".into(),
                duration_secs: 2.0,
            });
            continue;
        }

        // Must have the item
        if !inventory.has(&ev.item_id, 1) {
            continue;
        }

        // Map item_id to FarmObject variant
        let farm_obj = match ev.item_id.as_str() {
            "fence" => FarmObject::Fence,
            "scarecrow" => FarmObject::Scarecrow,
            _ => {
                warn!("PlaceFarmObjectEvent: unknown item '{}'", ev.item_id);
                continue;
            }
        };

        // Consume and place
        inventory.try_remove(&ev.item_id, 1);
        farm_state.objects.insert(pos, farm_obj);

        toast_events.send(ToastEvent {
            message: format!("{} placed.", match ev.item_id.as_str() {
                "fence" => "Fence",
                "scarecrow" => "Scarecrow",
                _ => "Object",
            }),
            duration_secs: 2.0,
        });
        sfx_events.send(PlaySfxEvent {
            sfx_id: "place_object".to_string(),
        });
    }
}
```

Register the system in the plugin (same schedule as other placement handlers, in Update with Playing state guard).

### Step 3: Extend farming/render.rs — render Fence objects with autotiling

In `sync_farm_objects_sprites`:
1. Change the filter to also include `FarmObject::Fence`:
```rust
matches!(obj, FarmObject::Sprinkler | FarmObject::Scarecrow | FarmObject::Fence)
```

2. Extend `farm_object_atlas_index` to handle Fence:
```rust
FarmObject::Fence => Some(0), // placeholder — overridden by autotile below
```

3. For Fence entities, use the fences atlas (ObjectAtlases.fences_image/fences_layout) instead of FurnitureAtlases. You'll need to add `obj_atlases: Res<ObjectAtlases>` to the system params.

4. When spawning a Fence, compute a 4-bit bitmask by checking cardinal neighbors in FarmState.objects for FarmObject::Fence:
```rust
fn fence_autotile_index(farm_state: &FarmState, x: i32, y: i32) -> usize {
    let mut mask: u8 = 0;
    if matches!(farm_state.objects.get(&(x, y - 1)), Some(FarmObject::Fence)) { mask |= 1; } // north
    if matches!(farm_state.objects.get(&(x + 1, y)), Some(FarmObject::Fence)) { mask |= 2; } // east
    if matches!(farm_state.objects.get(&(x, y + 1)), Some(FarmObject::Fence)) { mask |= 4; } // south
    if matches!(farm_state.objects.get(&(x - 1, y)), Some(FarmObject::Fence)) { mask |= 8; } // west
    // Maps to 0-15 index directly (4x4 atlas = 16 tiles)
    mask as usize
}
```

5. Add `farm_object_color` entry:
```rust
FarmObject::Fence => Color::srgb(0.6, 0.4, 0.2), // brown
```

### Step 4: Player item_use.rs — send PlaceFarmObjectEvent

At the top, add:
```rust
use crate::farming::PlaceFarmObjectEvent;
```

Add to system params:
```rust
mut farm_object_events: EventWriter<PlaceFarmObjectEvent>,
```

After the machines block (before the bouquet check), add:
```rust
// ── FARM OBJECTS (fence, scarecrow) ─────────────────────────────
if matches!(item_id.as_str(), "fence" | "scarecrow") {
    farm_object_events.send(PlaceFarmObjectEvent {
        item_id: item_id.clone(),
        grid_x: target_x,
        grid_y: target_y,
    });
    return;
}
```

## Do NOT
- Modify src/shared/mod.rs
- Modify src/world/ or src/ui/
- Add path placement (different pattern — modifies tiles, not entities. Leave for later.)
- Change the scarecrow's existing atlas index (45) — it already works
- Add new FarmObject variants

## Validation
```
cargo check
cargo clippy -- -D warnings
```
Both must pass with zero errors/warnings.

## When done
Write completion report to status/workers/fence-placement.md listing:
- Files modified
- Event defined
- Handler behavior
- Fence autotiling approach
- Scarecrow placement behavior

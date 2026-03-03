//! World object spawning for airport zones.

use bevy::prelude::*;
use crate::shared::*;

/// Spawn objects for a given zone. Called after map tiles are placed.
pub fn spawn_zone_objects(
    commands: &mut Commands,
    _airport: AirportId,
    zone: MapZone,
) {
    match zone {
        MapZone::Terminal => spawn_terminal_objects(commands),
        MapZone::Lounge => spawn_lounge_objects(commands),
        MapZone::Hangar => spawn_hangar_objects(commands),
        MapZone::CrewQuarters => spawn_quarters_objects(commands),
        MapZone::Shop => spawn_shop_objects(commands),
        _ => {}
    }
}

fn spawn_object(commands: &mut Commands, kind: WorldObjectKind, gx: i32, gy: i32, prompt: &str) {
    let pos = grid_to_world_center(gx, gy);
    commands.spawn((
        WorldObject { kind, grid_x: gx, grid_y: gy },
        Interactable {
            prompt: prompt.to_string(),
            range: 1.5,
        },
        Sprite::from_color(Color::srgb(0.6, 0.4, 0.2), Vec2::new(14.0, 14.0)),
        Transform::from_xyz(pos.x, pos.y, Z_OBJECTS),
    ));
}

fn spawn_terminal_objects(commands: &mut Commands) {
    spawn_object(commands, WorldObjectKind::CheckInDesk, 6, 3, "[F] Check In");
    spawn_object(commands, WorldObjectKind::Monitor, 10, 2, "[F] Flight Board");
    spawn_object(commands, WorldObjectKind::MissionBoard, 14, 3, "[F] Mission Board");
    spawn_object(commands, WorldObjectKind::Chair, 5, 8, "");
    spawn_object(commands, WorldObjectKind::Chair, 7, 8, "");
    spawn_object(commands, WorldObjectKind::Chair, 9, 8, "");
    spawn_object(commands, WorldObjectKind::VendingMachine, 17, 5, "[F] Vending Machine");
    spawn_object(commands, WorldObjectKind::Plant, 2, 2, "");
    spawn_object(commands, WorldObjectKind::Plant, 17, 2, "");
    spawn_object(commands, WorldObjectKind::Clock, 10, 1, "");
}

fn spawn_lounge_objects(commands: &mut Commands) {
    spawn_object(commands, WorldObjectKind::Sofa, 4, 5, "[F] Sit");
    spawn_object(commands, WorldObjectKind::Sofa, 4, 7, "[F] Sit");
    spawn_object(commands, WorldObjectKind::Table, 6, 6, "");
    spawn_object(commands, WorldObjectKind::Counter, 3, 2, "[F] Order Drink");
    spawn_object(commands, WorldObjectKind::Bookshelf, 15, 3, "[F] Read");
    spawn_object(commands, WorldObjectKind::Monitor, 12, 2, "[F] Weather Channel");
    spawn_object(commands, WorldObjectKind::Plant, 1, 1, "");
    spawn_object(commands, WorldObjectKind::Lamp, 8, 4, "");
}

fn spawn_hangar_objects(commands: &mut Commands) {
    spawn_object(commands, WorldObjectKind::Toolbox, 3, 3, "[F] Repair Aircraft");
    spawn_object(commands, WorldObjectKind::FuelPump, 3, 8, "[F] Refuel");
    spawn_object(commands, WorldObjectKind::ControlPanel, 3, 13, "[F] Aircraft Status");
    spawn_object(commands, WorldObjectKind::Locker, 20, 3, "[F] Equipment");
}

fn spawn_quarters_objects(commands: &mut Commands) {
    spawn_object(commands, WorldObjectKind::Bed, 3, 3, "[F] Sleep");
    spawn_object(commands, WorldObjectKind::Locker, 8, 2, "[F] Storage");
    spawn_object(commands, WorldObjectKind::Table, 6, 6, "");
    spawn_object(commands, WorldObjectKind::Chair, 5, 6, "[F] Sit");
    spawn_object(commands, WorldObjectKind::Lamp, 3, 5, "");
    spawn_object(commands, WorldObjectKind::Clock, 7, 1, "");
}

fn spawn_shop_objects(commands: &mut Commands) {
    spawn_object(commands, WorldObjectKind::Counter, 6, 2, "[F] Buy");
    spawn_object(commands, WorldObjectKind::ShopDisplay, 8, 4, "[F] Browse");
    spawn_object(commands, WorldObjectKind::ShopDisplay, 10, 4, "[F] Browse");
    spawn_object(commands, WorldObjectKind::ShopDisplay, 8, 6, "[F] Browse");
}

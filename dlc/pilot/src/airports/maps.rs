//! Airport map definitions.

use crate::shared::*;

/// Generate the tile grid for a given zone at a given airport.
pub fn generate_zone_map(airport: AirportId, zone: MapZone) -> (u32, u32, Vec<Vec<TileKind>>) {
    match zone {
        MapZone::Terminal => generate_terminal(airport),
        MapZone::Lounge => generate_lounge(airport),
        MapZone::Hangar => generate_hangar(airport),
        MapZone::Runway => generate_runway(airport),
        MapZone::ControlTower => generate_control_tower(airport),
        MapZone::CrewQuarters => generate_crew_quarters(airport),
        MapZone::Shop => generate_shop(airport),
        MapZone::CityStreet => generate_city_street(airport),
    }
}

fn generate_terminal(_airport: AirportId) -> (u32, u32, Vec<Vec<TileKind>>) {
    let w = 20;
    let h = 16;
    let mut tiles = vec![vec![TileKind::Floor; w]; h];
    // Walls on edges
    for x in 0..w {
        tiles[0][x] = TileKind::Wall;
        tiles[h - 1][x] = TileKind::Wall;
    }
    for y in 0..h {
        tiles[y][0] = TileKind::Wall;
        tiles[y][w - 1] = TileKind::Wall;
    }
    // Windows along north wall
    for x in (2..w - 2).step_by(3) {
        tiles[0][x] = TileKind::Window;
    }
    // Exit doors
    tiles[0][w / 2] = TileKind::Door; // To runway
    tiles[h - 1][w / 2] = TileKind::Door; // To city
    tiles[h / 2][0] = TileKind::Door; // To lounge
    tiles[h / 2][w - 1] = TileKind::Door; // To hangar
    // Check-in counter
    for x in 5..9 {
        tiles[3][x] = TileKind::Metal;
    }
    // Carpet area
    for y in 5..h - 3 {
        for x in 3..w - 3 {
            tiles[y][x] = TileKind::Carpet;
        }
    }
    (w as u32, h as u32, tiles)
}

fn generate_lounge(_airport: AirportId) -> (u32, u32, Vec<Vec<TileKind>>) {
    let w = 20;
    let h = 14;
    let mut tiles = vec![vec![TileKind::Carpet; w]; h];
    for x in 0..w {
        tiles[0][x] = TileKind::Wall;
        tiles[h - 1][x] = TileKind::Wall;
    }
    for y in 0..h {
        tiles[y][0] = TileKind::Wall;
        tiles[y][w - 1] = TileKind::Wall;
    }
    // Door to terminal
    tiles[h / 2][w - 1] = TileKind::Door;
    // Door to crew quarters
    tiles[0][w / 2] = TileKind::Door;
    // Bar counter
    for x in 2..6 {
        tiles[2][x] = TileKind::Metal;
    }
    (w as u32, h as u32, tiles)
}

fn generate_hangar(_airport: AirportId) -> (u32, u32, Vec<Vec<TileKind>>) {
    let w = 24;
    let h = 18;
    let mut tiles = vec![vec![TileKind::Metal; w]; h];
    for x in 0..w {
        tiles[0][x] = TileKind::Wall;
        tiles[h - 1][x] = TileKind::Wall;
    }
    for y in 0..h {
        tiles[y][0] = TileKind::Wall;
        tiles[y][w - 1] = TileKind::Wall;
    }
    // Big door to terminal
    tiles[h / 2][0] = TileKind::Door;
    tiles[h / 2 + 1][0] = TileKind::Door;
    // Aircraft parking spots (open floor)
    for y in 3..h - 3 {
        for x in 3..w - 3 {
            tiles[y][x] = TileKind::Tarmac;
        }
    }
    (w as u32, h as u32, tiles)
}

fn generate_runway(_airport: AirportId) -> (u32, u32, Vec<Vec<TileKind>>) {
    let w = 20;
    let h = 30;
    let mut tiles = vec![vec![TileKind::Grass; w]; h];
    // Runway strip
    for y in 0..h {
        for x in 7..13 {
            tiles[y][x] = TileKind::Runway;
        }
    }
    // Taxiway
    for x in 4..7 {
        tiles[h - 3][x] = TileKind::Taxiway;
    }
    // Edge: south leads to terminal
    for x in 8..12 {
        tiles[h - 1][x] = TileKind::Tarmac;
    }
    (w as u32, h as u32, tiles)
}

fn generate_control_tower(_airport: AirportId) -> (u32, u32, Vec<Vec<TileKind>>) {
    let w = 12;
    let h = 10;
    let mut tiles = vec![vec![TileKind::Floor; w]; h];
    for x in 0..w {
        tiles[0][x] = TileKind::Wall;
        tiles[h - 1][x] = TileKind::Wall;
    }
    for y in 0..h {
        tiles[y][0] = TileKind::Wall;
        tiles[y][w - 1] = TileKind::Wall;
    }
    // Windows all around top
    for x in 1..w - 1 {
        tiles[0][x] = TileKind::Window;
    }
    // Door out
    tiles[h - 1][w / 2] = TileKind::Door;
    // Control panels
    for x in 2..w - 2 {
        tiles[1][x] = TileKind::Metal;
    }
    (w as u32, h as u32, tiles)
}

fn generate_crew_quarters(_airport: AirportId) -> (u32, u32, Vec<Vec<TileKind>>) {
    let w = 14;
    let h = 12;
    let mut tiles = vec![vec![TileKind::Carpet; w]; h];
    for x in 0..w {
        tiles[0][x] = TileKind::Wall;
        tiles[h - 1][x] = TileKind::Wall;
    }
    for y in 0..h {
        tiles[y][0] = TileKind::Wall;
        tiles[y][w - 1] = TileKind::Wall;
    }
    // Door to lounge
    tiles[h - 1][w / 2] = TileKind::Door;
    (w as u32, h as u32, tiles)
}

fn generate_shop(_airport: AirportId) -> (u32, u32, Vec<Vec<TileKind>>) {
    let w = 14;
    let h = 10;
    let mut tiles = vec![vec![TileKind::Floor; w]; h];
    for x in 0..w {
        tiles[0][x] = TileKind::Wall;
        tiles[h - 1][x] = TileKind::Wall;
    }
    for y in 0..h {
        tiles[y][0] = TileKind::Wall;
        tiles[y][w - 1] = TileKind::Wall;
    }
    // Door to city street
    tiles[h / 2][0] = TileKind::Door;
    // Shop counter
    for x in 4..w - 2 {
        tiles[2][x] = TileKind::Metal;
    }
    (w as u32, h as u32, tiles)
}

fn generate_city_street(_airport: AirportId) -> (u32, u32, Vec<Vec<TileKind>>) {
    let w = 24;
    let h = 18;
    let mut tiles = vec![vec![TileKind::Grass; w]; h];
    // Main road
    for y in 7..11 {
        for x in 0..w {
            tiles[y][x] = TileKind::Tarmac;
        }
    }
    // Sidewalks
    for x in 0..w {
        tiles[6][x] = TileKind::Floor;
        tiles[11][x] = TileKind::Floor;
    }
    // Airport entrance (north)
    tiles[0][w / 2] = TileKind::Door;
    // Shop entrance (east)
    tiles[h / 2][w - 1] = TileKind::Door;
    (w as u32, h as u32, tiles)
}

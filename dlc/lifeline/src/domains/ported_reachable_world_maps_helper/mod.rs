//! pure host fn substrate port.

#![allow(unused)]
#![allow(dead_code)]

use bevy::prelude::*;

pub fn build_map_adjacency() -> bool { false }

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct MapId;

#[derive(Resource, Component, Debug, Default, Clone)]
pub struct MapRegistry;


pub fn reachable_world_maps_helper(start: MapId, map_registry: &MapRegistry) -> HashSet<MapId> {
    let adjacency = build_map_adjacency(map_registry);
    let mut visited = HashSet::new();
    let mut pending = VecDeque::from([start]);

    while let Some(map_id) = pending.pop_front() {
        if !visited.insert(map_id) {
            continue;
        }

        if let Some(neighbors) = adjacency.get(&map_id) {
            for &neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    pending.push_back(neighbor);
                }
            }
        }
    }

    visited
}



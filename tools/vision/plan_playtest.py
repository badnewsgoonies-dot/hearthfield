#!/usr/bin/env python3
"""Map-aware playtest planner for Hearthfield.

Reads all map RON files + map_data.rs edge/door definitions to build
a world graph, then generates deterministic movement orders for the
headless test driver.

The planner knows:
- Every map's dimensions and tile layout
- Every door position and destination
- Every edge transition and destination
- The collision grid (from runtime telemetry)

It generates a sequence of (action, args) tuples that the driver executes.

Usage:
    # Generate a full-world traversal plan
    python3 tools/vision/plan_playtest.py --output /tmp/playtest_plan.json

    # Generate plan for specific maps
    python3 tools/vision/plan_playtest.py --maps Farm,Town,Forest
"""

import json
import os
import re
import sys

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# ── Parse map RON files ─────────────────────────────────────────────────

def parse_ron_map(path):
    """Parse a map RON file and return {id, width, height, tiles}."""
    with open(path) as f:
        content = f.read()

    id_match = re.search(r'id:\s*(\w+)', content)
    w_match = re.search(r'width:\s*(\d+)', content)
    h_match = re.search(r'height:\s*(\d+)', content)
    tiles_match = re.search(r'tiles:\s*\[(.*?)\]', content, re.DOTALL)

    if not all([id_match, w_match, h_match, tiles_match]):
        return None

    tiles_str = tiles_match.group(1)
    tiles = [t.strip().rstrip(',') for t in tiles_str.split('\n') if t.strip() and t.strip().rstrip(',')]
    tiles = [t for t in tiles if t]

    return {
        "id": id_match.group(1),
        "width": int(w_match.group(1)),
        "height": int(h_match.group(1)),
        "tiles": tiles,
    }


def load_all_maps():
    """Load all map RON files from assets/maps/."""
    maps_dir = os.path.join(PROJECT_ROOT, "assets", "maps")
    maps = {}
    if not os.path.isdir(maps_dir):
        return maps
    for fname in sorted(os.listdir(maps_dir)):
        if fname.endswith(".ron"):
            path = os.path.join(maps_dir, fname)
            m = parse_ron_map(path)
            if m:
                maps[m["id"]] = m
    return maps


# ── Edge and door definitions (from map_data.rs) ────────────────────────

# These mirror the Rust definitions exactly.
DOORS = {
    "Farm": [
        {"x_min": 7, "x_max": 8, "y": 19, "to": "PlayerHouse", "to_x": 8, "to_y": 14},
    ],
    "Town": [
        {"x_min": 5, "x_max": 6, "y": 2, "to": "GeneralStore", "to_x": 6, "to_y": 10},
        {"x_min": 22, "x_max": 23, "y": 2, "to": "AnimalShop", "to_x": 6, "to_y": 10},
        {"x_min": 22, "x_max": 23, "y": 13, "to": "Blacksmith", "to_x": 6, "to_y": 10},
        {"x_min": 8, "x_max": 9, "y": 17, "to": "Library", "to_x": 7, "to_y": 10},
        {"x_min": 15, "x_max": 16, "y": 17, "to": "Tavern", "to_x": 8, "to_y": 12},
    ],
    "TownWest": [
        {"x_min": 3, "x_max": 4, "y": 13, "to": "TownHouseWest", "to_x": 6, "to_y": 10},
        {"x_min": 9, "x_max": 10, "y": 13, "to": "TownHouseEast", "to_x": 6, "to_y": 10},
    ],
    "MineEntrance": [
        {"x_min": 6, "x_max": 7, "y": 3, "to": "Mine", "to_x": 8, "to_y": 14},
    ],
}

EDGES = {
    "Farm":          {"north": "SnowMountain", "south": "Town", "east": "Forest", "west": "MineEntrance"},
    "Town":          {"north": "Farm", "south": "Beach", "east": "Forest", "west": "TownWest"},
    "TownWest":      {"east": "Town"},
    "Beach":         {"north": "Town", "south": "CoralIsland", "east": "Farm"},
    "Forest":        {"north": "MineEntrance", "east": "DeepForest", "west": "Farm"},
    "DeepForest":    {"west": "Forest"},
    "MineEntrance":  {"north": "SnowMountain", "south": "Forest", "east": "Farm"},
    "PlayerHouse":   {"north": "Farm"},
    "GeneralStore":  {"north": "Town"},
    "AnimalShop":    {"north": "Town"},
    "Blacksmith":    {"north": "Town"},
    "Library":       {"north": "Town"},
    "Tavern":        {"north": "Town"},
    "TownHouseWest": {"north": "TownWest"},
    "TownHouseEast": {"north": "TownWest"},
    "CoralIsland":   {"north": "Beach"},
    "SnowMountain":  {"south": "Farm"},
}


# ── World graph ─────────────────────────────────────────────────────────

def build_world_graph():
    """Build adjacency graph from edges + doors."""
    graph = {}
    for map_id, edges in EDGES.items():
        if map_id not in graph:
            graph[map_id] = []
        for direction, target in edges.items():
            graph[map_id].append({
                "type": "edge",
                "direction": direction,
                "target": target,
            })
    for map_id, doors in DOORS.items():
        if map_id not in graph:
            graph[map_id] = []
        for door in doors:
            graph[map_id].append({
                "type": "door",
                "x": door["x_min"],
                "y": door["y"],
                "target": door["to"],
            })
    return graph


def plan_full_traversal(start="PlayerHouse"):
    """Plan a DFS traversal of all reachable maps from start."""
    graph = build_world_graph()
    visited = set()
    plan = []

    def visit(map_id, entry_info="start"):
        if map_id in visited:
            return
        visited.add(map_id)
        plan.append({"action": "arrive", "map": map_id, "via": entry_info})
        plan.append({"action": "screenshot", "name": f"map_{map_id}"})
        plan.append({"action": "verify_map", "expected": map_id})

        # Visit all neighbors
        for connection in graph.get(map_id, []):
            target = connection["target"]
            if target in visited:
                continue

            if connection["type"] == "edge":
                direction = connection["direction"]
                plan.append({"action": "walk_to_edge", "direction": direction,
                           "expected_map": target})
                visit(target, f"edge_{direction}_from_{map_id}")
                # Return
                reverse = {"north":"south","south":"north","east":"west","west":"east"}[direction]
                plan.append({"action": "walk_to_edge", "direction": reverse,
                           "expected_map": map_id})
                plan.append({"action": "verify_map", "expected": map_id})

            elif connection["type"] == "door":
                plan.append({"action": "walk_to_pos", "x": connection["x"], "y": connection["y"],
                           "expected_map": target})
                visit(target, f"door_from_{map_id}")
                # Return via edge (interiors exit north)
                plan.append({"action": "walk_to_edge", "direction": "north",
                           "expected_map": map_id})
                plan.append({"action": "verify_map", "expected": map_id})

    visit(start)
    return plan


# ── Main ────────────────────────────────────────────────────────────────

def main():
    import argparse
    parser = argparse.ArgumentParser(description="Map-aware playtest planner")
    parser.add_argument("--output", default="/tmp/playtest_plan.json")
    parser.add_argument("--maps", default=None, help="Comma-separated map IDs to visit")
    parser.add_argument("--info", action="store_true", help="Print world graph info")
    args = parser.parse_args()

    maps = load_all_maps()
    graph = build_world_graph()

    if args.info:
        print(f"Maps loaded from RON: {len(maps)}")
        for name, m in maps.items():
            print(f"  {name}: {m['width']}x{m['height']} ({len(m['tiles'])} tiles)")
        print(f"\nWorld graph: {len(graph)} maps")
        for map_id, connections in graph.items():
            targets = [f"{c['type']}→{c['target']}" for c in connections]
            print(f"  {map_id}: {', '.join(targets)}")
        return

    plan = plan_full_traversal()

    with open(args.output, "w") as f:
        json.dump(plan, f, indent=2)

    # Summary
    maps_visited = set(s["map"] for s in plan if s.get("action") == "arrive")
    screenshots = sum(1 for s in plan if s.get("action") == "screenshot")
    print(f"Plan: {len(plan)} actions, {len(maps_visited)} maps, {screenshots} screenshots")
    print(f"Maps: {', '.join(sorted(maps_visited))}")
    print(f"Written to {args.output}")


if __name__ == "__main__":
    main()

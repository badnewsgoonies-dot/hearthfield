#!/usr/bin/env python3
from __future__ import annotations

import csv
import re
from collections import defaultdict
from pathlib import Path


ROOT = Path("/home/geni/swarm/hearthfield")
OUT = ROOT / "status" / "research"


def read(path: Path) -> str:
    return path.read_text()


def write_csv(path: Path, fieldnames: list[str], rows: list[dict[str, object]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


def asset_kind(rel: str) -> tuple[str, str]:
    if rel.startswith("assets/maps/"):
        return ("map_data", "map")
    if rel.startswith("assets/audio/"):
        return ("audio", "sound")
    if rel.startswith("assets/fonts/"):
        return ("font", "font")
    if rel.startswith("assets/sprites/"):
        return ("sprite", "visual")
    if rel.startswith("assets/tilesets/"):
        return ("tileset", "visual")
    if rel.startswith("assets/ui/"):
        return ("ui", "visual")
    return ("other", "other")


def rel_asset(path: str) -> str:
    if path.startswith("assets/"):
        return path
    return f"assets/{path}"


def generate_asset_manifest() -> None:
    rows: list[dict[str, object]] = []
    for p in sorted((ROOT / "assets").rglob("*")):
        if not p.is_file():
            continue
        rel = p.relative_to(ROOT).as_posix()
        family, role = asset_kind(rel)
        rows.append(
            {
                "path": rel,
                "family": family,
                "role_class": role,
                "extension": p.suffix.lower(),
                "bytes": p.stat().st_size,
            }
        )
    write_csv(
        OUT / "asset_manifest.csv",
        ["path", "family", "role_class", "extension", "bytes"],
        rows,
    )


def static_asset_loads() -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    files = sorted((ROOT / "src").rglob("*.rs"))
    pat = re.compile(r'asset_server\.load\("([^"]+)"\)')
    for p in files:
        text = read(p)
        for path in pat.findall(text):
            rows.append(
                {
                    "asset_path": rel_asset(path),
                    "source_file": p.relative_to(ROOT).as_posix(),
                    "loader_kind": "asset_server.load",
                    "role_hint": "",
                    "notes": "static string load",
                }
            )
    return rows


def audio_asset_loads() -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    text = read(ROOT / "src" / "ui" / "audio.rs")
    for path in sorted(set(re.findall(r'Some\("([^"]+\.ogg)"\)', text))):
        rows.append(
            {
                "asset_path": rel_asset(path),
                "source_file": "src/ui/audio.rs",
                "loader_kind": "audio_path_mapping",
                "role_hint": "music_or_sfx",
                "notes": "resolved from sfx_path/music_path match arms",
            }
        )
    return rows


def npc_asset_loads() -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    text = read(ROOT / "src" / "npcs" / "definitions.rs")
    for npc_id, path in re.findall(r'"([^"]+)"\s*=>\s*"([^"]+)"', text):
        if npc_id == "_":
            continue
        rows.append(
            {
                "asset_path": rel_asset(path),
                "source_file": "src/npcs/definitions.rs",
                "loader_kind": "npc_sprite_file",
                "role_hint": f"npc:{npc_id}",
                "notes": "NPC spritesheet mapping",
            }
        )
    return rows


def farming_dynamic_asset_loads() -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    text = read(ROOT / "src" / "farming" / "mod.rs")
    m = re.search(r'crop_sheets:\s*&\[\((.*?)\)\];', text, re.S)
    if m:
        block = m.group(1)
        for crop_id, path in re.findall(r'"([^"]+)",\s*"([^"]+)"', block):
            rows.append(
                {
                    "asset_path": rel_asset(path),
                    "source_file": "src/farming/mod.rs",
                    "loader_kind": "crop_sheet_table",
                    "role_hint": f"crop:{crop_id}",
                    "notes": "per-crop atlas load",
                }
            )
    return rows


def map_asset_loads() -> list[dict[str, object]]:
    rows: list[dict[str, object]] = []
    for p in sorted((ROOT / "assets" / "maps").glob("*.ron")):
        rows.append(
            {
                "asset_path": p.relative_to(ROOT).as_posix(),
                "source_file": "src/world/map_data.rs",
                "loader_kind": "map_registry_load",
                "role_hint": p.stem,
                "notes": "loaded by load_map_data/build_map_registry",
            }
        )
    return rows


def generate_runtime_used_asset_manifest() -> None:
    rows = []
    rows.extend(static_asset_loads())
    rows.extend(audio_asset_loads())
    rows.extend(npc_asset_loads())
    rows.extend(farming_dynamic_asset_loads())
    rows.extend(map_asset_loads())

    dedup: dict[tuple[str, str], dict[str, object]] = {}
    for row in rows:
        key = (str(row["asset_path"]), str(row["source_file"]))
        dedup[key] = row

    final_rows = sorted(
        dedup.values(),
        key=lambda r: (str(r["asset_path"]), str(r["source_file"]), str(r["loader_kind"])),
    )
    write_csv(
        OUT / "runtime_used_asset_manifest.csv",
        ["asset_path", "source_file", "loader_kind", "role_hint", "notes"],
        final_rows,
    )


def parse_struct_names_with_derive(text: str, derive_name: str) -> list[str]:
    pat = re.compile(
        rf"#\[derive\((?:[^\]]*?){derive_name}(?:[^\]]*?)\)\]\s*(?:pub\s+)?(?:struct|enum)\s+(\w+)",
        re.S,
    )
    return [m.group(1) for m in pat.finditer(text)]


def generate_plugin_resource_event_inventory() -> None:
    main_text = read(ROOT / "src" / "main.rs")
    shared_text = read(ROOT / "src" / "shared" / "mod.rs")

    plugin_rows: list[dict[str, object]] = []
    for mod_name in re.findall(r"mod\s+(\w+);", main_text):
        plugin_rows.append(
            {
                "inventory_type": "plugin_module",
                "name": mod_name,
                "source_file": "src/main.rs",
                "details": "",
            }
        )
    for plugin in re.findall(r"\.add_plugins\(([\w:]+)::(\w+)\)", main_text):
        plugin_rows.append(
            {
                "inventory_type": "plugin_registration",
                "name": f"{plugin[0]}::{plugin[1]}",
                "source_file": "src/main.rs",
                "details": "",
            }
        )
    for res in re.findall(r"\.init_resource::<([\w:]+)>\(\)", main_text):
        plugin_rows.append(
            {
                "inventory_type": "resource_registration",
                "name": res,
                "source_file": "src/main.rs",
                "details": "registered in App builder",
            }
        )
    for ev in re.findall(r"\.add_event::<([\w:]+)>\(\)", main_text):
        plugin_rows.append(
            {
                "inventory_type": "event_registration",
                "name": ev,
                "source_file": "src/main.rs",
                "details": "registered in App builder",
            }
        )

    for res in parse_struct_names_with_derive(shared_text, "Resource"):
        plugin_rows.append(
            {
                "inventory_type": "shared_resource_type",
                "name": res,
                "source_file": "src/shared/mod.rs",
                "details": "derive(Resource)",
            }
        )
    for ev in parse_struct_names_with_derive(shared_text, "Event"):
        plugin_rows.append(
            {
                "inventory_type": "shared_event_type",
                "name": ev,
                "source_file": "src/shared/mod.rs",
                "details": "derive(Event)",
            }
        )

    write_csv(
        OUT / "plugin_resource_event_inventory.csv",
        ["inventory_type", "name", "source_file", "details"],
        plugin_rows,
    )


def parse_map_transitions(text: str) -> list[tuple[str, str]]:
    rows: list[tuple[str, str]] = []

    # transitions block
    for match in re.finditer(r"transitions:\s*\[(.*?)\]\s*,", text, re.S):
        block = match.group(1)
        for target in re.findall(r"to_map:\s*([A-Za-z0-9_]+)", block):
            rows.append(("transition", target))

    # doors block
    for match in re.finditer(r"doors:\s*\[(.*?)\]\s*,", text, re.S):
        block = match.group(1)
        for target in re.findall(r"to_map:\s*([A-Za-z0-9_]+)", block):
            rows.append(("door", target))

    # edges block
    edge_block_match = re.search(r"edges:\s*\((.*?)\)\s*,\s*buildings:", text, re.S)
    if edge_block_match:
        block = edge_block_match.group(1)
        for edge in ["north", "south", "east", "west"]:
            m = re.search(rf"{edge}:\s*Some\(\(([A-Za-z0-9_]+),", block)
            if m:
                rows.append((f"edge_{edge}", m.group(1)))
    return rows


def generate_reachable_surface_manifest() -> None:
    rows: list[dict[str, object]] = []
    for p in sorted((ROOT / "assets" / "maps").glob("*.ron")):
        text = read(p)
        map_id_match = re.search(r"id:\s*([A-Za-z0-9_]+)", text)
        width_match = re.search(r"width:\s*(\d+)", text)
        height_match = re.search(r"height:\s*(\d+)", text)
        spawn_match = re.search(r"spawn_pos:\s*\(([-\d]+),\s*([-\d]+)\)", text)
        map_id = map_id_match.group(1) if map_id_match else p.stem

        rows.append(
            {
                "source_map": map_id,
                "target_map": "",
                "link_type": "map",
                "source_file": p.relative_to(ROOT).as_posix(),
                "width": width_match.group(1) if width_match else "",
                "height": height_match.group(1) if height_match else "",
                "spawn_x": spawn_match.group(1) if spawn_match else "",
                "spawn_y": spawn_match.group(2) if spawn_match else "",
                "notes": "map definition present",
            }
        )
        for link_type, target in parse_map_transitions(text):
            rows.append(
                {
                    "source_map": map_id,
                    "target_map": target,
                    "link_type": link_type,
                    "source_file": p.relative_to(ROOT).as_posix(),
                    "width": "",
                    "height": "",
                    "spawn_x": "",
                    "spawn_y": "",
                    "notes": "",
                }
            )

    write_csv(
        OUT / "reachable_surface_manifest.csv",
        [
            "source_map",
            "target_map",
            "link_type",
            "source_file",
            "width",
            "height",
            "spawn_x",
            "spawn_y",
            "notes",
        ],
        rows,
    )


def parse_items() -> list[dict[str, object]]:
    text = read(ROOT / "src" / "data" / "items.rs")
    rows = []
    for m in re.finditer(
        r'id:\s*"([^"]+)"\.into\(\),.*?sprite_index:\s*(\d+),',
        text,
        re.S,
    ):
        rows.append(
            {
                "visual_type": "item_icon",
                "role_id": m.group(1),
                "asset_path": "assets/sprites/items_atlas.png",
                "mapping_value": m.group(2),
                "source_file": "src/data/items.rs",
                "notes": "item sprite_index",
            }
        )
    return rows


def parse_crops() -> list[dict[str, object]]:
    text = read(ROOT / "src" / "data" / "crops.rs")
    rows = []
    for m in re.finditer(
        r'id:\s*"([^"]+)"\.into\(\),.*?sprite_stages:\s*vec!\[([^\]]+)\]',
        text,
        re.S,
    ):
        crop_id = m.group(1)
        stages = ",".join(x.strip() for x in m.group(2).split(",") if x.strip())
        rows.append(
            {
                "visual_type": "crop_growth",
                "role_id": crop_id,
                "asset_path": "assets/sprites/plants.png",
                "mapping_value": stages,
                "source_file": "src/data/crops.rs",
                "notes": "crop sprite_stages baseline",
            }
        )
    return rows


def parse_fish() -> list[dict[str, object]]:
    text = read(ROOT / "src" / "data" / "fish.rs")
    rows = []
    for m in re.finditer(
        r'id:\s*"([^"]+)"\.into\(\),.*?sprite_index:\s*(\d+),',
        text,
        re.S,
    ):
        rows.append(
            {
                "visual_type": "fish_sprite",
                "role_id": m.group(1),
                "asset_path": "assets/sprites/fishing_atlas.png",
                "mapping_value": m.group(2),
                "source_file": "src/data/fish.rs",
                "notes": "fish sprite_index",
            }
        )
    return rows


def parse_npcs() -> list[dict[str, object]]:
    rows = []
    defs_text = read(ROOT / "src" / "npcs" / "definitions.rs")
    npc_data_text = read(ROOT / "src" / "data" / "npcs.rs")
    sprite_paths: dict[str, str] = {}

    for m in re.finditer(r'"([^"]+)"\s*=>\s*"([^"]+)"', defs_text):
        npc_id, path = m.group(1), m.group(2)
        if npc_id == "_":
            continue
        sprite_paths[npc_id] = f"assets/{path}"
        rows.append(
            {
                "visual_type": "npc_spritesheet",
                "role_id": npc_id,
                "asset_path": sprite_paths[npc_id],
                "mapping_value": "4x4@48x48 -> render 24x24",
                "source_file": "src/npcs/definitions.rs",
                "notes": "npc_sprite_file mapping",
            }
        )

    for m in re.finditer(
        r'id:\s*"([^"]+)"\.into\(\),.*?portrait_index:\s*(\d+),',
        npc_data_text,
        re.S,
    ):
        rows.append(
            {
                "visual_type": "npc_portrait_index",
                "role_id": m.group(1),
                "asset_path": sprite_paths.get(m.group(1), "assets/sprites/npcs/npc_guard.png"),
                "mapping_value": m.group(2),
                "source_file": "src/data/npcs.rs",
                "notes": "portrait_index baseline",
            }
        )
    return rows


def parse_fixed_asset_loads() -> list[dict[str, object]]:
    rows = []
    patterns = [
        ("player_main_spritesheet", "assets/sprites/character_spritesheet.png", "src/player/spawn.rs"),
        ("player_action_spritesheet", "assets/sprites/character_actions.png", "src/player/spawn.rs"),
        ("farming_plants_atlas", "assets/sprites/plants.png", "src/farming/mod.rs"),
        ("farming_dirt_atlas", "assets/tilesets/tilled_dirt.png", "src/farming/mod.rs"),
        ("sprinkler_sprite", "assets/sprites/sprinkler.png", "src/farming/mod.rs"),
        ("sprinkler_anim", "assets/sprites/sprinkler_anim.png", "src/farming/mod.rs"),
        ("scarecrow_sprite", "assets/sprites/scarecrow.png", "src/farming/mod.rs"),
        ("fishing_atlas", "assets/sprites/fishing_atlas.png", "src/fishing/mod.rs"),
        ("hud_item_icons", "assets/sprites/items_atlas.png", "src/ui/hud.rs"),
        ("hud_weather_icons", "assets/ui/weather_icons.png", "src/ui/hud.rs"),
        ("dialog_box_big", "assets/ui/dialog_box_big.png", "src/ui/dialogue_box.rs"),
        ("world_grass_tiles", "assets/tilesets/grass.png", "src/world/mod.rs"),
        ("world_dirt_tiles", "assets/tilesets/tilled_dirt.png", "src/world/mod.rs"),
        ("world_water_tiles", "assets/tilesets/water.png", "src/world/mod.rs"),
        ("world_paths", "assets/sprites/paths.png", "src/world/mod.rs"),
        ("world_bridge", "assets/sprites/wood_bridge.png", "src/world/mod.rs"),
        ("world_terrain", "assets/tilesets/modern_farm_terrain.png", "src/world/mod.rs"),
        ("object_grass_biome", "assets/sprites/grass_biome.png", "src/world/objects.rs"),
        ("object_fences", "assets/tilesets/fences.png", "src/world/objects.rs"),
        ("object_tree_sprites", "assets/sprites/tree_sprites.png", "src/world/objects.rs"),
        ("object_house_walls", "assets/tilesets/house_walls.png", "src/world/objects.rs"),
        ("object_house_roof", "assets/tilesets/house_roof.png", "src/world/objects.rs"),
        ("object_doors", "assets/tilesets/doors.png", "src/world/objects.rs"),
        ("object_door_anim", "assets/sprites/door_anim.png", "src/world/objects.rs"),
        ("object_furniture_atlas", "assets/sprites/furniture.png", "src/world/objects.rs"),
        ("object_shipping_bin", "assets/sprites/shipping_bin.png", "src/world/objects.rs"),
        ("object_crafting_bench", "assets/sprites/crafting_bench.png", "src/world/objects.rs"),
        ("object_carpenter_board", "assets/sprites/carpenter_board.png", "src/world/objects.rs"),
        ("object_processing_machine", "assets/sprites/processing_machine.png", "src/world/objects.rs"),
        ("object_machine_anim", "assets/sprites/machine_anim.png", "src/world/objects.rs"),
        ("world_chest", "assets/sprites/chest.png", "src/world/chests.rs"),
        ("main_menu_play_button", "assets/ui/play_button.png", "src/ui/menu_kit.rs"),
        ("font_main", "assets/fonts/sprout_lands.ttf", "src/ui/mod.rs"),
    ]
    for role_id, asset_path, source in patterns:
        rows.append(
            {
                "visual_type": "fixed_asset_load",
                "role_id": role_id,
                "asset_path": asset_path,
                "mapping_value": "",
                "source_file": source,
                "notes": "",
            }
        )
    return rows


def generate_visual_mapping_manifest() -> None:
    rows = []
    rows.extend(parse_fixed_asset_loads())
    rows.extend(parse_items())
    rows.extend(parse_crops())
    rows.extend(parse_fish())
    rows.extend(parse_npcs())

    write_csv(
        OUT / "visual_mapping_manifest.csv",
        ["visual_type", "role_id", "asset_path", "mapping_value", "source_file", "notes"],
        rows,
    )


def infer_test_family(name: str) -> str:
    checks = [
        ("save", "save_load"),
        ("fishing", "fishing"),
        ("mine", "mining"),
        ("animal", "animals"),
        ("crop", "farming"),
        ("sprinkler", "farming"),
        ("gold", "economy"),
        ("shipping", "economy"),
        ("tool", "player_tools"),
        ("quest", "npcs_quests"),
        ("wedding", "npcs_romance"),
        ("bouquet", "npcs_romance"),
        ("proposal", "npcs_romance"),
        ("festival", "calendar"),
        ("calendar", "calendar"),
        ("map", "world_ui"),
        ("snow_mountain", "world"),
        ("town_west", "world"),
        ("library", "world_ui"),
        ("tavern", "world_ui"),
        ("tutorial", "tutorial"),
        ("keybinding", "input"),
    ]
    for needle, family in checks:
        if needle in name:
            return family
    return "misc"


def generate_test_baseline_manifest() -> None:
    rows = []
    for rel in ["tests/headless.rs", "tests/keybinding_duplicates.rs"]:
        lines = (ROOT / rel).read_text().splitlines()
        for i, line in enumerate(lines):
            if line.strip() != "#[test]":
                continue
            name = None
            for j in range(i + 1, min(i + 8, len(lines))):
                candidate = lines[j].strip()
                if candidate.startswith("fn "):
                    name = candidate.split("fn ", 1)[1].split("(", 1)[0].strip()
                    break
            if name is None:
                continue
            rows.append(
                {
                    "test_name": name,
                    "suite": Path(rel).name,
                    "family": infer_test_family(name),
                    "source_file": rel,
                }
            )
    write_csv(
        OUT / "test_baseline_manifest.csv",
        ["test_name", "suite", "family", "source_file"],
        rows,
    )


def generate_runtime_surface_manifest() -> None:
    rows = [
        {
            "surface_id": "boot_main_menu",
            "family": "boot",
            "entry_condition": "launch game",
            "player_path": "boot -> MainMenu -> tick without panic",
            "primary_maps_or_states": "Loading,MainMenu",
            "primary_files": "src/main.rs; src/ui/main_menu.rs; tests/headless.rs",
            "baseline_evidence": "tests/headless.rs::test_headless_boot_smoke_transitions_and_ticks",
            "preserve_requirement": "must remain bootable and panic-free",
        },
        {
            "surface_id": "player_spawn_house_exit_sleep",
            "family": "player_day_start",
            "entry_condition": "new game start",
            "player_path": "spawn in PlayerHouse -> exit -> act -> sleep",
            "primary_maps_or_states": "PlayerHouse,Farm,Playing",
            "primary_files": "src/player/spawn.rs; src/player/interaction.rs; src/world/map_data.rs; src/ui/tutorial.rs",
            "baseline_evidence": ".memory/STATE.md; ACCEPTANCE.md phase 1",
            "preserve_requirement": "must spawn correctly and support house exit/sleep loop",
        },
        {
            "surface_id": "tutorial_first_week",
            "family": "tutorial",
            "entry_condition": "days 1-3 progression",
            "player_path": "intro -> day 1 objective -> later-day objective progression",
            "primary_maps_or_states": "Playing,Cutscene",
            "primary_files": "src/ui/tutorial.rs; src/ui/tool_tutorial.rs; tests/headless.rs",
            "baseline_evidence": "tests/headless.rs::test_tutorial_later_day_objectives_initialize_after_day1_completion",
            "preserve_requirement": "must preserve objective and hint progression",
        },
        {
            "surface_id": "farm_core_loop",
            "family": "farming",
            "entry_condition": "player on Farm with starter tools/seeds",
            "player_path": "till -> plant -> water -> grow -> harvest",
            "primary_maps_or_states": "Farm,Playing",
            "primary_files": "src/farming/*; src/player/tools.rs; tests/headless.rs",
            "baseline_evidence": "farming-related tests in tests/headless.rs",
            "preserve_requirement": "must preserve crop lifecycle and season rules",
        },
        {
            "surface_id": "shipping_and_gold",
            "family": "economy",
            "entry_condition": "items in shipping bin or direct gold event",
            "player_path": "ship items -> day end -> gold increases",
            "primary_maps_or_states": "Farm,Playing",
            "primary_files": "src/economy/shipping.rs; src/economy/gold.rs; tests/headless.rs",
            "baseline_evidence": "shipping and gold tests in tests/headless.rs",
            "preserve_requirement": "must preserve sale flow and gold clamp behavior",
        },
        {
            "surface_id": "town_shop_loop",
            "family": "economy_ui",
            "entry_condition": "player reaches shop interior",
            "player_path": "enter shop -> buy/sell -> return to town",
            "primary_maps_or_states": "Town,GeneralStore,AnimalShop,Blacksmith,Shop",
            "primary_files": "src/ui/shop_screen.rs; src/economy/shop.rs; src/player/interaction.rs; assets/maps/*.ron",
            "baseline_evidence": "ACCEPTANCE.md economy section; map transition tests",
            "preserve_requirement": "must preserve shop entry and transaction behavior",
        },
        {
            "surface_id": "crafting_loop",
            "family": "crafting",
            "entry_condition": "crafting bench or kitchen available",
            "player_path": "open bench -> consume ingredients -> receive output",
            "primary_maps_or_states": "Crafting,Playing",
            "primary_files": "src/crafting/*; src/ui/crafting_screen.rs; tests/headless.rs",
            "baseline_evidence": "crafting tests in tests/headless.rs",
            "preserve_requirement": "must preserve recipe, consume, refund, and machine behavior",
        },
        {
            "surface_id": "fishing_loop",
            "family": "fishing",
            "entry_condition": "player at fishable water with rod",
            "player_path": "cast -> bite -> minigame -> resolve -> inventory",
            "primary_maps_or_states": "Playing,Fishing",
            "primary_files": "src/fishing/*; src/player/item_use.rs; tests/headless.rs",
            "baseline_evidence": "fishing skill and legendary tests; ACCEPTANCE.md fishing section",
            "preserve_requirement": "must preserve fish selection, minigame, resolve, and skill progression",
        },
        {
            "surface_id": "mining_loop",
            "family": "mining",
            "entry_condition": "player enters mine",
            "player_path": "enter -> break rocks -> fight -> ladder -> descend/exit",
            "primary_maps_or_states": "MineEntrance,Mine,Playing",
            "primary_files": "src/mining/*; src/player/interaction.rs; tests/headless.rs",
            "baseline_evidence": "mining tests in tests/headless.rs; ACCEPTANCE.md mining section",
            "preserve_requirement": "must preserve generation, combat, ladder, and exit behavior",
        },
        {
            "surface_id": "social_loop",
            "family": "npcs_social",
            "entry_condition": "player near NPC or gifting item",
            "player_path": "talk -> gain friendship -> gift -> quest/romance progression",
            "primary_maps_or_states": "Town and interiors,Dialogue,RelationshipsView",
            "primary_files": "src/npcs/*; src/ui/dialogue_box.rs; src/ui/relationships_screen.rs; tests/headless.rs",
            "baseline_evidence": "friendship, bouquet, proposal, wedding, quest tests",
            "preserve_requirement": "must preserve NPC identity, schedules, friendship, quests, and romance",
        },
        {
            "surface_id": "save_load_roundtrip",
            "family": "save",
            "entry_condition": "manual save/load or load on main menu",
            "player_path": "save -> quit/load -> restore map/position/resources",
            "primary_maps_or_states": "Paused,MainMenu,Playing",
            "primary_files": "src/save/mod.rs; tests/headless.rs",
            "baseline_evidence": "save roundtrip tests in tests/headless.rs",
            "preserve_requirement": "must preserve current_map, position, and serialized resource state",
        },
        {
            "surface_id": "world_graph_travel",
            "family": "world_graph",
            "entry_condition": "player reaches edge or door trigger",
            "player_path": "travel across outdoor and indoor map graph",
            "primary_maps_or_states": "All 18 shipped maps",
            "primary_files": "src/world/map_data.rs; src/player/interaction.rs; assets/maps/*.ron; tests/headless.rs",
            "baseline_evidence": "map transition tests and reachable_surface_manifest.csv",
            "preserve_requirement": "must preserve map identities, transitions, and spawn positions",
        },
        {
            "surface_id": "ui_screen_family",
            "family": "ui",
            "entry_condition": "screen-specific state transitions",
            "player_path": "open HUD/inventory/journal/map/calendar/stats/relationships/settings/pause and return safely",
            "primary_maps_or_states": "HUD,Inventory,Journal,MapView,RelationshipsView,Paused,MainMenu,Dialogue,Cutscene,Shop,Crafting",
            "primary_files": "src/ui/*; src/input/mod.rs",
            "baseline_evidence": "UI module inventory + acceptance docs",
            "preserve_requirement": "no shipped screen family may disappear or degrade to placeholder-only",
        },
        {
            "surface_id": "sailing_coral_island",
            "family": "world_travel",
            "entry_condition": "boat mode on Beach south edge",
            "player_path": "Beach <-> CoralIsland travel",
            "primary_maps_or_states": "Beach,CoralIsland,Playing",
            "primary_files": "src/world/map_data.rs; src/player/interaction.rs; docs/sailing_spec.md; tests/headless.rs",
            "baseline_evidence": ".memory/STATE.md and coral island map presence",
            "preserve_requirement": "if present in baseline, must remain reachable and non-broken",
        },
        {
            "surface_id": "snow_mountain_extension",
            "family": "world_extension",
            "entry_condition": "MineEntrance north travel",
            "player_path": "MineEntrance <-> SnowMountain plus mountain NPC/fishing/objects",
            "primary_maps_or_states": "MineEntrance,SnowMountain,Playing",
            "primary_files": "assets/maps/snow_mountain.ron; src/world/map_data.rs; tests/headless.rs; src/data/npcs.rs; src/data/fish.rs",
            "baseline_evidence": "snow mountain tests in tests/headless.rs",
            "preserve_requirement": "must preserve map reachability, Bjorn schedule/dialogue, and mountain fishing support",
        },
    ]

    write_csv(
        OUT / "runtime_surface_manifest.csv",
        [
            "surface_id",
            "family",
            "entry_condition",
            "player_path",
            "primary_maps_or_states",
            "primary_files",
            "baseline_evidence",
            "preserve_requirement",
        ],
        rows,
    )


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    generate_asset_manifest()
    generate_runtime_used_asset_manifest()
    generate_visual_mapping_manifest()
    generate_reachable_surface_manifest()
    generate_plugin_resource_event_inventory()
    generate_test_baseline_manifest()
    generate_runtime_surface_manifest()
    print("Wrote baseline manifests to", OUT)


if __name__ == "__main__":
    main()

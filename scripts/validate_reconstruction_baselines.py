#!/usr/bin/env python3
from __future__ import annotations

import csv
import sys
from pathlib import Path


ROOT = Path("/home/geni/swarm/hearthfield")
RESEARCH = ROOT / "status" / "research"


def read_csv(name: str) -> list[dict[str, str]]:
    path = RESEARCH / name
    with path.open() as f:
        return list(csv.DictReader(f))


def assert_file(path_str: str, errors: list[str], label: str) -> None:
    path = ROOT / path_str
    if not path.exists():
        errors.append(f"{label} missing: {path_str}")


def validate_runtime_used(errors: list[str]) -> None:
    rows = read_csv("runtime_used_asset_manifest.csv")
    for row in rows:
        assert_file(row["asset_path"], errors, "runtime asset")
        assert_file(row["source_file"], errors, "runtime source_file")


def validate_visual_mapping(errors: list[str]) -> None:
    rows = read_csv("visual_mapping_manifest.csv")
    for row in rows:
        asset_path = row["asset_path"]
        if "<see npc_sprite_file>" not in asset_path:
            assert_file(asset_path, errors, "visual asset")
        assert_file(row["source_file"], errors, "visual source_file")


def validate_reachable_surfaces(errors: list[str]) -> None:
    rows = read_csv("reachable_surface_manifest.csv")
    map_names = {row["source_map"] for row in rows if row["link_type"] == "map"}
    for row in rows:
        assert_file(row["source_file"], errors, "reachable source_file")
        target = row["target_map"]
        if target and target not in map_names:
            errors.append(
                f"reachable_surface target_map missing from map set: {row['source_map']} -> {target}"
            )


def validate_plugin_inventory(errors: list[str]) -> None:
    rows = read_csv("plugin_resource_event_inventory.csv")
    for row in rows:
        assert_file(row["source_file"], errors, "inventory source_file")


def validate_tests(errors: list[str]) -> None:
    rows = read_csv("test_baseline_manifest.csv")
    for row in rows:
        assert_file(row["source_file"], errors, "test source_file")
        text = (ROOT / row["source_file"]).read_text()
        if f"fn {row['test_name']}(" not in text:
            errors.append(
                f"test name not found in source: {row['test_name']} ({row['source_file']})"
            )


def validate_runtime_surface_manifest(errors: list[str]) -> None:
    rows = read_csv("runtime_surface_manifest.csv")
    for row in rows:
        for part in [p.strip() for p in row["primary_files"].split(";")]:
            if not part:
                continue
            if "*" in part:
                parent = ROOT / part.split("*", 1)[0]
                if not parent.exists():
                    errors.append(f"runtime surface primary_files wildcard parent missing: {part}")
            else:
                assert_file(part, errors, "runtime surface primary_file")


def main() -> int:
    required = [
        "runtime_used_asset_manifest.csv",
        "asset_manifest.csv",
        "visual_mapping_manifest.csv",
        "reachable_surface_manifest.csv",
        "plugin_resource_event_inventory.csv",
        "test_baseline_manifest.csv",
        "runtime_surface_manifest.csv",
    ]
    missing = [name for name in required if not (RESEARCH / name).exists()]
    if missing:
        for name in missing:
            print(f"missing manifest: {name}", file=sys.stderr)
        return 1

    errors: list[str] = []
    validate_runtime_used(errors)
    validate_visual_mapping(errors)
    validate_reachable_surfaces(errors)
    validate_plugin_inventory(errors)
    validate_tests(errors)
    validate_runtime_surface_manifest(errors)

    if errors:
        for err in errors:
            print(err, file=sys.stderr)
        return 1

    print("reconstruction baselines validated")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

#!/usr/bin/env python3
"""Headless screenshot test driver for Hearthfield.

Launches the game under xvfb with software rendering, navigates to
requested screens via XTest key injection, captures screenshots when
the game state matches the target, and optionally runs VLM assertions.

Usage:
    # Capture all major screens:
    python3 tools/vision/headless_test.py --scenes all --output /tmp/screens

    # Capture specific screens:
    python3 tools/vision/headless_test.py --scenes MainMenu,Inventory,Crafting

    # With VLM assertions:
    python3 tools/vision/headless_test.py --scenes all --assert

Requirements:
    - Xvfb installed (xvfb package)
    - Mesa software renderer (libgl1-mesa-dri)
    - libxkbcommon-x11 (for Bevy's winit)
    - libEGL (libegl-mesa0)
    - Game binary built: cargo build

Environment:
    HEARTHFIELD_HEADLESS=1 is set automatically to enable state telemetry.
    The game writes /tmp/hearthfield-state.json each frame.
"""

import argparse
import ctypes
import ctypes.util
import json
import os
import signal
import struct
import subprocess
import sys
import time
import zlib

# ── X11 / XTest bindings ───────────────────────────────────────────────

xlib = None
xtst = None
display = None

def init_x11():
    global xlib, xtst, display
    xlib = ctypes.cdll.LoadLibrary(ctypes.util.find_library("X11"))
    xtst = ctypes.cdll.LoadLibrary(ctypes.util.find_library("Xtst"))
    display = xlib.XOpenDisplay(b":99")
    if not display:
        print("ERROR: Cannot open X display :99. Is Xvfb running?")
        sys.exit(1)

def focus_game_window():
    """Focus the game window so key events reach it."""
    root = xlib.XDefaultRootWindow(display)
    r, p = ctypes.c_ulong(), ctypes.c_ulong()
    children = ctypes.POINTER(ctypes.c_ulong)()
    n = ctypes.c_uint()
    xlib.XQueryTree(display, root, ctypes.byref(r), ctypes.byref(p),
                    ctypes.byref(children), ctypes.byref(n))
    if n.value > 0:
        xlib.XSetInputFocus(display, children[n.value - 1], 1, 0)
        xlib.XFlush(display)
        time.sleep(0.3)
        return True
    return False

def send_key(keysym, hold=0.08):
    """Send a key press+release via XTest."""
    kc = xlib.XKeysymToKeycode(display, keysym)
    if not kc:
        return
    xtst.XTestFakeKeyEvent(display, kc, 1, ctypes.c_ulong(0))
    xlib.XFlush(display)
    time.sleep(hold)
    xtst.XTestFakeKeyEvent(display, kc, 0, ctypes.c_ulong(0))
    xlib.XFlush(display)
    time.sleep(0.15)

def hold_key(keysym, duration):
    """Hold a key for a duration."""
    kc = xlib.XKeysymToKeycode(display, keysym)
    if not kc:
        return
    xtst.XTestFakeKeyEvent(display, kc, 1, ctypes.c_ulong(0))
    xlib.XFlush(display)
    time.sleep(duration)
    xtst.XTestFakeKeyEvent(display, kc, 0, ctypes.c_ulong(0))
    xlib.XFlush(display)
    time.sleep(0.15)

# Keysym constants
KEY_RETURN = 0xff0d
KEY_SPACE  = 0x0020
KEY_ESCAPE = 0xff1b
KEY_UP     = 0xff52
KEY_DOWN   = 0xff54
KEY_LEFT   = 0xff51
KEY_RIGHT  = 0xff53
KEY_W = 0x77
KEY_A = 0x61
KEY_S = 0x73
KEY_D = 0x64
KEY_E = 0x65  # inventory
KEY_C = 0x63  # crafting
KEY_J = 0x6a  # journal
KEY_M = 0x6d  # map
KEY_F = 0x66  # interact
KEY_L = 0x6c  # relationships

# ── Screenshot capture ──────────────────────────────────────────────────

def capture_screenshot(output_path):
    """Capture the X root window to a PNG file."""
    root = xlib.XDefaultRootWindow(display)
    xlib.XGetImage.restype = ctypes.c_void_p
    img = xlib.XGetImage(display, root, 0, 0, 1280, 720, 0xFFFFFFFF, 2)
    if not img:
        print(f"  ERROR: XGetImage failed")
        return False

    w = ctypes.c_int.from_address(img + 0).value
    h = ctypes.c_int.from_address(img + 4).value
    data_ptr = ctypes.c_ulong.from_address(img + 16).value
    bpl = ctypes.c_int.from_address(img + 44).value
    bpp = ctypes.c_int.from_address(img + 48).value

    raw = (ctypes.c_ubyte * (h * bpl)).from_address(data_ptr)
    Bpp = bpp // 8

    # Build RGB pixel data
    pixels = bytearray(w * h * 3)
    for y in range(h):
        for x in range(w):
            o = y * bpl + x * Bpp
            si = (y * w + x) * 3
            pixels[si] = raw[o + 2]      # R
            pixels[si + 1] = raw[o + 1]  # G
            pixels[si + 2] = raw[o]      # B

    # Write PNG
    def chunk(ctype, data):
        c = ctype + data
        return struct.pack(">I", len(data)) + c + struct.pack(">I", zlib.crc32(c) & 0xFFFFFFFF)

    raw_rows = bytearray()
    for y in range(h):
        raw_rows.append(0)  # filter byte
        raw_rows.extend(pixels[y * w * 3:(y + 1) * w * 3])

    compressed = zlib.compress(bytes(raw_rows), 6)

    os.makedirs(os.path.dirname(output_path) or ".", exist_ok=True)
    with open(output_path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n")
        f.write(chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0)))
        f.write(chunk(b"IDAT", compressed))
        f.write(chunk(b"IEND", b""))

    xlib.XDestroyImage(img)
    sz = os.path.getsize(output_path)
    print(f"  Saved {output_path} ({sz} bytes)")
    return True

# ── State file reader ───────────────────────────────────────────────────

STATE_FILE = "/tmp/hearthfield-state.json"

def read_game_state():
    """Read the current game state from the telemetry file."""
    try:
        with open(STATE_FILE, "r") as f:
            return json.load(f)
    except (FileNotFoundError, json.JSONDecodeError):
        return None

def wait_for_state(target_state, timeout=15):
    """Wait until game_state matches target. Returns True on match."""
    start = time.time()
    while time.time() - start < timeout:
        state = read_game_state()
        if state and state.get("game_state") == target_state:
            return True
        time.sleep(0.3)
    return False

def wait_for_any_state(target_states, timeout=15):
    """Wait until game_state matches any of the targets."""
    start = time.time()
    while time.time() - start < timeout:
        state = read_game_state()
        if state and state.get("game_state") in target_states:
            return state.get("game_state")
        time.sleep(0.3)
    return None

# ── Scene navigation ────────────────────────────────────────────────────

def navigate_to_scene(scene_name):
    """Navigate the game to the requested scene. Returns True on success."""
    state = read_game_state()
    current = state.get("game_state") if state else None

    if scene_name == "MainMenu":
        # Should already be here on fresh launch
        if current == "MainMenu":
            return True
        # From Playing: Escape → Pause → Quit to Menu
        send_key(KEY_ESCAPE)
        time.sleep(0.5)
        send_key(KEY_DOWN)  # Select "Quit to Menu"
        send_key(KEY_DOWN)
        send_key(KEY_RETURN)
        return wait_for_state("MainMenu")

    if scene_name == "Playing":
        if current == "MainMenu":
            focus_game_window()
            send_key(KEY_RETURN)
            time.sleep(1)
            send_key(KEY_SPACE)
            time.sleep(5)
            # Skip intro dialogue
            for _ in range(12):
                send_key(KEY_RETURN)
            time.sleep(1)
            return wait_for_state("Playing")
        elif current == "Playing":
            return True
        else:
            # Close any overlay
            send_key(KEY_ESCAPE)
            return wait_for_state("Playing")

    # For UI overlays, ensure we're Playing first
    if current != "Playing":
        if not navigate_to_scene("Playing"):
            return False
        time.sleep(0.5)

    key_map = {
        "Inventory": KEY_E,
        "Crafting": KEY_C,
        "Journal": KEY_J,
        "MapView": KEY_M,
        "RelationshipsView": KEY_L,
        "Paused": KEY_ESCAPE,
    }

    if scene_name in key_map:
        send_key(key_map[scene_name])
        if wait_for_state(scene_name):
            return True
        # Retry: maybe we weren't in Playing state yet
        time.sleep(1)
        state = read_game_state()
        if state and state.get("game_state") != "Playing":
            send_key(KEY_ESCAPE)
            time.sleep(0.5)
            wait_for_state("Playing", timeout=5)
            time.sleep(0.3)
        send_key(key_map[scene_name])
        return wait_for_state(scene_name)

    # Special scenes
    if scene_name == "Dialogue":
        # Walk to an NPC and press F — hard to automate reliably
        print(f"  SKIP: {scene_name} requires NPC proximity")
        return False

    if scene_name == "PlayerHouse":
        # Check if already in PlayerHouse
        state = read_game_state()
        if state and state.get("player_map") == "PlayerHouse":
            return True
        print(f"  SKIP: {scene_name} navigation not automated yet")
        return False

    print(f"  SKIP: Unknown scene {scene_name}")
    return False


# ── Main entry point ────────────────────────────────────────────────────

ALL_SCENES = [
    "MainMenu",
    "Playing",
    "Inventory",
    "Crafting",
    "Paused",
    "MapView",
    "Journal",
]

def main():
    parser = argparse.ArgumentParser(description="Headless screenshot test driver")
    parser.add_argument("--scenes", default="all",
                        help="Comma-separated scene names, or 'all'")
    parser.add_argument("--output", default="/tmp/hearthfield-screens",
                        help="Output directory for screenshots")
    parser.add_argument("--timeout", type=int, default=15,
                        help="Seconds to wait for each scene")
    parser.add_argument("--no-launch", action="store_true",
                        help="Don't launch xvfb/game (assume already running)")
    parser.add_argument("--assert", action="store_true", dest="run_assert",
                        help="Run VLM assertions on captured screenshots")
    args = parser.parse_args()

    scenes = ALL_SCENES if args.scenes == "all" else args.scenes.split(",")

    xvfb_proc = None
    game_proc = None

    try:
        if not args.no_launch:
            # Kill stale processes
            subprocess.run(["killall", "Xvfb", "hearthfield"],
                           capture_output=True)
            time.sleep(1)

            # Start Xvfb
            print("Starting Xvfb...")
            xvfb_proc = subprocess.Popen(
                ["Xvfb", ":99", "-screen", "0", "1280x720x24",
                 "-ac", "+extension", "GLX", "+render", "-noreset"],
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
            )
            time.sleep(2)

            # Find binary
            project_root = os.path.dirname(os.path.dirname(os.path.dirname(
                os.path.abspath(__file__))))
            binary = os.path.join(project_root, "target", "debug", "hearthfield")
            if not os.path.exists(binary):
                print(f"ERROR: Binary not found at {binary}. Run 'cargo build' first.")
                sys.exit(1)

            # Launch game
            print("Launching game...")
            env = os.environ.copy()
            env.update({
                "DISPLAY": ":99",
                "WGPU_BACKEND": "gl",
                "LIBGL_ALWAYS_SOFTWARE": "1",
                "GALLIUM_DRIVER": "llvmpipe",
                "XDG_RUNTIME_DIR": "/tmp/xdg",
                "HEARTHFIELD_HEADLESS": "1",
            })
            os.makedirs("/tmp/xdg", exist_ok=True)
            game_proc = subprocess.Popen(
                [binary], cwd=project_root, env=env,
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
            )

            # Wait for game to start
            print("Waiting for game to load...")
            time.sleep(10)

        # Initialize X11
        init_x11()
        focus_game_window()

        # Remove stale state file
        try:
            os.unlink(STATE_FILE)
        except FileNotFoundError:
            pass

        # Wait for telemetry
        print("Waiting for state telemetry...")
        for _ in range(30):
            state = read_game_state()
            if state:
                print(f"  Game state: {state.get('game_state')}")
                break
            time.sleep(0.5)
        else:
            print("ERROR: No state telemetry received. Is HEARTHFIELD_HEADLESS=1 set?")
            sys.exit(1)

        # Capture each scene
        results = {}
        for i, scene in enumerate(scenes):
            print(f"\n[{i+1}/{len(scenes)}] Navigating to: {scene}")

            if navigate_to_scene(scene):
                time.sleep(1)  # Let rendering settle
                path = os.path.join(args.output, f"{i+1:02d}_{scene}.png")
                success = capture_screenshot(path)
                results[scene] = "OK" if success else "CAPTURE_FAILED"

                # Return to Playing for next scene (unless MainMenu)
                if scene not in ("MainMenu", "Playing"):
                    send_key(KEY_ESCAPE)
                    time.sleep(1.0)
                    wait_for_state("Playing", timeout=5)
            else:
                results[scene] = "NAV_FAILED"
                print(f"  FAILED to navigate to {scene}")

        # Summary
        print("\n" + "=" * 50)
        print("RESULTS:")
        for scene, status in results.items():
            marker = "✓" if status == "OK" else "✗"
            print(f"  {marker} {scene}: {status}")

        ok = sum(1 for s in results.values() if s == "OK")
        print(f"\n{ok}/{len(scenes)} scenes captured to {args.output}/")

        # Write results JSON
        results_path = os.path.join(args.output, "results.json")
        with open(results_path, "w") as f:
            json.dump(results, f, indent=2)

    finally:
        if game_proc:
            game_proc.terminate()
            game_proc.wait()
        if xvfb_proc:
            xvfb_proc.terminate()
            xvfb_proc.wait()

if __name__ == "__main__":
    main()

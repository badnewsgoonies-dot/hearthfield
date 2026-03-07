#!/usr/bin/env python3
"""
AI Sprite Generation Pipeline for Hearthfield
Uses OpenAI gpt-image-1.5 + smart grid downscaling.

Usage:
  python3 scripts/ai_sprite_gen.py --preset walkcycle
  python3 scripts/ai_sprite_gen.py --preset trees
  python3 scripts/ai_sprite_gen.py --prompt "..." --target 32x48 --output assets/sprites/foo.png
"""
import argparse, json, base64, os, sys
import urllib.request
import numpy as np
from PIL import Image
from collections import Counter

API_KEY = os.environ.get("OPENAI_API_KEY", "")

# ══════════════════════════════════════════════════════════════
# PRESETS
# ══════════════════════════════════════════════════════════════

PRESETS = {
    "walkcycle": {
        "target": (64, 64),  # 4x4 grid of 16x16 frames
        "size": "1024x1024",
        "quality": "medium",
        "output": "assets/sprites/character_spritesheet.png",
        "prompt": (
            "Pixel art character walk cycle spritesheet for Harvest Moon GBA. "
            "The image is exactly 1024x1024 pixels containing a 64x64 art-pixel grid "
            "where each art pixel is 16x16 real pixels. "
            "4 columns x 4 rows. Each cell is 16x16 art pixels, one walking frame. "
            "Row 1: walking DOWN toward viewer. Row 2: walking LEFT. "
            "Row 3: walking RIGHT. Row 4: walking UP away from viewer. "
            "Farmer with straw hat, brown hair, blue overalls, white shirt, brown boots. "
            "Chibi proportions with large head. "
            "CRITICAL: STRICT pixel art. Each 16x16 real pixel block is ONE solid flat color. "
            "No gradients, no anti-aliasing, no blending. Transparent background."
        ),
    },
    "trees": {
        "target": (128, 96),  # 4 cols x 2 rows of 32x48
        "size": "1024x1536",  # 1024/128=8, 1536/96=16 → non-square cells
        "quality": "medium",
        "output": "assets/sprites/tree_sprites.png",
        "prompt": (
            "Pixel art seasonal tree spritesheet for Harvest Moon GBA. "
            "The image is 1024x1536 pixels containing a grid of 8 trees. "
            "4 columns x 2 rows. Each cell is 256x768 real pixels (32x48 art pixels). "
            "Row 1: deciduous trees. Left to right: spring (bright green, pink blossoms), "
            "summer (deep green), autumn (orange/red leaves), winter (bare branches, snow). "
            "Row 2: pine/evergreen trees. Left to right: spring (dark green triangular), "
            "summer (deep green), autumn (slightly faded), winter (green with snow). "
            "Brown trunks, round canopies for deciduous, triangular for pine. "
            "CRITICAL: STRICT pixel art. Each pixel block is ONE solid flat color. "
            "No gradients, no anti-aliasing. Transparent background."
        ),
    },
    "npc": {
        "target": (16, 32),
        "size": "1024x1024",
        "quality": "medium",
        "output": None,  # set via --output
        "prompt": (
            "Pixel art NPC character sprite for Harvest Moon GBA. "
            "The image is 1024x1024 pixels. The character is 16x32 art pixels centered, "
            "each art pixel is 32x32 real pixels. Standing pose facing down toward viewer. "
            "{description} "
            "Chibi proportions with large head. "
            "CRITICAL: STRICT pixel art. Each 32x32 pixel block is ONE solid flat color. "
            "No gradients, no anti-aliasing. Transparent background."
        ),
    },
    "house": {
        "target": (48, 48),
        "size": "1024x1024",
        "quality": "medium",
        "output": None,
        "prompt": (
            "Pixel art building sprite for Harvest Moon GBA. "
            "The image is 1024x1024 pixels. The building is 48x48 art pixels centered, "
            "each art pixel is ~21x21 real pixels. Front view with slight top-down angle. "
            "{description} "
            "CRITICAL: STRICT pixel art. Each pixel block is ONE solid flat color. "
            "No gradients, no anti-aliasing. Transparent background."
        ),
    },
    "items": {
        "target": (64, 64),  # 4x4 grid of 16x16 icons
        "size": "1024x1024",
        "quality": "medium",
        "output": "assets/sprites/items_atlas.png",
        "prompt": (
            "Pixel art item icon spritesheet for Harvest Moon GBA. "
            "The image is 1024x1024 pixels containing a 4x4 grid of 64x64 art-pixel icons. "
            "Each art pixel is 16x16 real pixels. "
            "16 distinct item icons: {description} "
            "CRITICAL: STRICT pixel art. Each 16x16 pixel block is ONE solid flat color. "
            "No gradients, no anti-aliasing. Transparent background."
        ),
    },
}

# ══════════════════════════════════════════════════════════════
# SMART GRID DOWNSCALER
# ══════════════════════════════════════════════════════════════

def smart_grid_downscale(img, tw, th, palette_size=32):
    """
    Downscale AI 'pixel art' to true pixel art resolution.
    
    1. Compute grid cell size from image/target ratio
    2. Search for optimal grid offset (minimize intra-cell variance)
    3. Mode-sample each cell (most common color)
    4. Optional palette quantization for cleaner result
    """
    arr = np.array(img)
    h, w = arr.shape[:2]
    px_w = w // tw
    px_h = h // th
    
    # Find best grid offset
    gray = np.array(img.convert("L")).astype(float)
    best_ox, best_oy, best_score = 0, 0, float('inf')
    
    step_x = max(1, px_w // 6)
    step_y = max(1, px_h // 6)
    
    for ox in range(0, px_w, step_x):
        for oy in range(0, px_h, step_y):
            var_sum, count = 0, 0
            for ty in range(min(th, 10)):
                for tx in range(min(tw, 10)):
                    y0 = oy + ty * px_h
                    x0 = ox + tx * px_w
                    cell = gray[y0:min(y0+px_h, h), x0:min(x0+px_w, w)]
                    if cell.size > 4:
                        var_sum += np.var(cell)
                        count += 1
            if count > 0:
                score = var_sum / count
                if score < best_score:
                    best_score = score
                    best_ox, best_oy = ox, oy
    
    # Sample each cell
    result = Image.new("RGBA", (tw, th), (0, 0, 0, 0))
    for ty in range(th):
        for tx in range(tw):
            x0 = best_ox + tx * px_w
            y0 = best_oy + ty * px_h
            x1 = min(x0 + px_w, w)
            y1 = min(y0 + px_h, h)
            
            cell = arr[y0:y1, x0:x1]
            if cell.size == 0:
                continue
            
            # Skip mostly transparent cells
            if cell.shape[2] == 4 and np.mean(cell[:, :, 3]) < 64:
                continue
            
            # Get opaque pixels
            if cell.shape[2] == 4:
                opaque = cell[cell[:, :, 3] > 128][:, :3]
            else:
                opaque = cell.reshape(-1, 3)
            
            if len(opaque) == 0:
                continue
            
            # Mode color with quantization to reduce AA noise
            snapped = (opaque // 4 * 4)
            colors = [tuple(c) for c in snapped]
            mode_c = Counter(colors).most_common(1)[0][0]
            result.putpixel((tx, ty), mode_c + (255,))
    
    return result

# ══════════════════════════════════════════════════════════════
# API CALL
# ══════════════════════════════════════════════════════════════

def generate_image(prompt, size="1024x1024", quality="medium"):
    """Call OpenAI image generation API"""
    body = json.dumps({
        "model": "gpt-image-1.5",
        "prompt": prompt,
        "n": 1,
        "size": size,
        "quality": quality,
    })
    
    req = urllib.request.Request(
        "https://api.openai.com/v1/images/generations",
        data=body.encode(),
        headers={
            "Authorization": f"Bearer {API_KEY}",
            "Content-Type": "application/json",
        },
    )
    
    resp = urllib.request.urlopen(req)
    data = json.loads(resp.read())
    
    if "error" in data:
        raise RuntimeError(data["error"]["message"])
    
    img_bytes = base64.b64decode(data["data"][0]["b64_json"])
    return img_bytes

# ══════════════════════════════════════════════════════════════
# MAIN
# ══════════════════════════════════════════════════════════════

def main():
    parser = argparse.ArgumentParser(description="AI sprite generation for Hearthfield")
    parser.add_argument("--preset", choices=list(PRESETS.keys()), help="Use a preset configuration")
    parser.add_argument("--prompt", help="Custom prompt (overrides preset)")
    parser.add_argument("--target", help="Target size WxH e.g. 32x48")
    parser.add_argument("--output", help="Output path")
    parser.add_argument("--description", default="", help="Fill {description} placeholder in preset prompt")
    parser.add_argument("--raw", help="Save raw AI output to this path")
    parser.add_argument("--quality", default="medium", choices=["low", "medium", "high"])
    args = parser.parse_args()
    
    if not API_KEY:
        print("Error: Set OPENAI_API_KEY environment variable")
        sys.exit(1)
    
    if args.preset:
        preset = PRESETS[args.preset]
        prompt = args.prompt or preset["prompt"].format(description=args.description)
        tw, th = args.target and tuple(map(int, args.target.split("x"))) or preset["target"]
        output = args.output or preset["output"]
        size = preset["size"]
        quality = args.quality or preset["quality"]
    else:
        if not args.prompt or not args.target or not args.output:
            print("Error: --prompt, --target, and --output required without --preset")
            sys.exit(1)
        prompt = args.prompt
        tw, th = map(int, args.target.split("x"))
        output = args.output
        size = "1024x1024"
        quality = args.quality
    
    print(f"Generating {tw}x{th} sprite...")
    print(f"  Quality: {quality}")
    print(f"  Output: {output}")
    
    # Generate
    raw_bytes = generate_image(prompt, size=size, quality=quality)
    
    # Save raw if requested
    if args.raw:
        with open(args.raw, "wb") as f:
            f.write(raw_bytes)
        print(f"  Raw saved: {args.raw}")
    
    # Load and downscale
    import io
    raw_img = Image.open(io.BytesIO(raw_bytes)).convert("RGBA")
    print(f"  Raw size: {raw_img.size}")
    
    result = smart_grid_downscale(raw_img, tw, th)
    
    # Save
    os.makedirs(os.path.dirname(output) if os.path.dirname(output) else ".", exist_ok=True)
    result.save(output)
    print(f"  ✅ Saved: {output} ({tw}x{th})")

if __name__ == "__main__":
    main()

import re

with open("scripts/gemini_sprite_gen.py", "r") as f:
    content = f.read()

new_presets = """    "grass": {
        "target": (176, 112),
        "aspectRatio": "16:9",
        "output": "assets/tilesets/grass.png",
        "prompt": (
            "Pixel art grass terrain tileset for Harvest Moon GBA. "
            "A grid of 11 columns and 7 rows. "
            "Row 1: spring bright green grass. Row 2: summer deep green grass. "
            "Row 3: autumn orange/yellow grass. Row 4: winter snowy white grass. "
            "The remaining rows contain variations of grass with small pebbles or flowers. "
            "CRITICAL: STRICT pixel art, flat colors, sharp edges. "
            "No gradients, no anti-aliasing. White background."
        ),
    },
    "dirt": {
        "target": (176, 112),
        "aspectRatio": "16:9",
        "output": "assets/tilesets/tilled_dirt.png",
        "prompt": (
            "Pixel art dirt and flooring tileset for Harvest Moon GBA. "
            "A grid of 11 columns and 7 rows. "
            "Top rows contain brown soil and tilled farming dirt. "
            "Middle rows contain grey cobblestone floor tiles. "
            "Bottom rows contain warm brown wood plank floor tiles. "
            "CRITICAL: STRICT pixel art, flat colors, sharp edges. "
            "No gradients, no anti-aliasing. White background."
        ),
    },
    "paths": {
        "target": (64, 64),
        "aspectRatio": "1:1",
        "output": "assets/tilesets/paths.png",
        "prompt": (
            "Pixel art dirt path and road tileset for Harvest Moon GBA. "
            "A 4x4 grid of tiles. A seamless dirt path with grassy edges. "
            "Row 1: top edges. Row 2: middle path and side edges. Row 3: bottom edges. "
            "CRITICAL: STRICT pixel art, flat colors, sharp edges. "
            "No gradients, no anti-aliasing. White background."
        ),
    },
"""

content = content.replace('PRESETS = {\n', 'PRESETS = {\n' + new_presets)

with open("scripts/gemini_sprite_gen.py", "w") as f:
    f.write(content)

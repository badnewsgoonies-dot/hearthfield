# Sprite Generation Spec — Horse & Cat

## Tool
gpt-image-1.5 via OpenAI API

## Horse (assets/sprites/horse.png)
- **Sheet**: 768×128 px (24 cols × 4 rows of 32×32 cells)
- **Style**: Cozy pixel art, matches existing animals (see sheep.png, pig.png for reference)
- **Color**: Brown (#593319 body, lighter mane/tail)
- **Rows**: Down-facing walk, Up-facing walk, Left-facing walk, Right-facing walk
- **Cols**: 6 frames per direction × 4 idle/transition variants (24 total)
- **Background**: Transparent
- **Generation params**: `background: "transparent"`, `size: "1536x1024"`, `quality: "high"`
- **Post-process**: Crop to 768×128, downscale 2× with nearest-neighbor if generated at 2×

## Cat (assets/sprites/cat.png)  
- **Sheet**: 384×64 px (24 cols × 4 rows of 16×16 cells)
- **Style**: Cozy pixel art, matches chicken.png scale and detail level
- **Color**: Orange tabby (#E68C33 body, darker stripes)
- **Rows**: Down-facing walk, Up-facing walk, Left-facing walk, Right-facing walk
- **Cols**: 6 frames per direction × 4 idle/transition variants (24 total)
- **Background**: Transparent
- **Generation params**: Same as horse
- **Post-process**: Crop to 384×64

## Critical: Generate individually at full canvas
Do NOT batch into a single generation. Each sprite sheet should use the full
1536×1024 canvas for maximum detail. The 4× grid anchor technique:
- 64×64 per cell in 1024×1024 → crop and downscale 4× with nearest-neighbor
- For 32×32 targets: generate at 128×128 per cell, downscale 4×
- For 16×16 targets: generate at 64×64 per cell, downscale 4×

## Reference sprites (already in repo)
- `assets/sprites/chicken.png` — 16×16, good reference for Cat scale
- `assets/sprites/sheep.png` — 32×32, good reference for Horse scale  
- `assets/sprites/pig.png` — 32×32, same scale as Horse
- `assets/sprites/_source_limezu/sheep.png` — Limezu original for art direction

## Verification
After generation, run:
```bash
python3 -c "
from PIL import Image
for f in ['horse', 'cat']:
    img = Image.open(f'assets/sprites/{f}.png')
    print(f'{f}: {img.size}, mode={img.mode}')
    assert img.mode == 'RGBA', f'{f} must have alpha channel'
"
```

# GPT-Image-1.5 Sprite Pipeline — Research & Upgrade Plan

*Compiled March 3, 2026 — Sources: OpenAI API docs, OpenAI Cookbook prompting guide, fal.ai guide, adaptive downscaling research (hiivelabs), community reports*

---

## 1. What Changed: gpt-image-1 → gpt-image-1.5

Released December 16, 2025. This is OpenAI's current flagship image model.

**Performance gains:**
- Up to 4× faster generation
- 20% cheaper API pricing
- #1 on LMArena Text-to-Image (1277), Design Arena (1344), and AA Arena (1272)
- Better prompt following and instruction adherence
- Improved text rendering (crisp lettering, consistent layout)
- Built-in reasoning and world knowledge

**What this means for sprites:** The model understands spatial relationships, grid layouts, and proportional sizing better. When we say "10 aircraft progressing from small to large," 1.5 actually does it. We saw this in the test — aircraft clearly scaled from tiny Cessna to Boeing 777.

---

## 2. Complete API Parameter Reference

### Generation Endpoint: `POST /v1/images/generations`

| Parameter | Values | Default | Notes |
|-----------|--------|---------|-------|
| `model` | `gpt-image-1.5`, `gpt-image-1`, `gpt-image-1-mini` | `gpt-image-1.5` | Always use 1.5 |
| `quality` | `low`, `medium`, `high`, `auto` | `high` | **We never set this before!** Use `high` for final sprites |
| `size` | `1024x1024`, `1536x1024` (landscape), `1024x1536` (portrait), `auto` | `1024x1024` | **Key insight: use `1536x1024` for horizontal sprite sheets** |
| `background` | `transparent`, `opaque`, `auto` | `auto` | **Use `transparent` for sprites!** Saves post-processing |
| `output_format` | `png`, `jpeg`, `webp` | `png` | Keep `png` for transparency + lossless |
| `n` | 1–10 | 1 | Can generate multiple variants per call |
| `moderation` | `auto`, `low` | `auto` | Leave as auto |
| `output_compression` | 0–100 | 100 | Only for jpeg/webp, keep 100 for png |

### Edit Endpoint: `POST /v1/images/edits`

Same parameters plus:

| Parameter | Values | Default | Notes |
|-----------|--------|---------|-------|
| `input_fidelity` | `low`, `high` | `low` | `high` preserves reference image details better. First 5 images get higher fidelity with 1.5 |
| `image` | Up to 16 input images | — | For compositing, style reference |
| `mask` | PNG with transparent areas | — | Indicates where edits should apply |

### What We Were Missing (Previous Scripts)

```python
# BEFORE (our old scripts)
body = {"model": "gpt-image-1", "prompt": prompt, "n": 1, "size": "1024x1024"}

# AFTER (all available optimizations)
body = {
    "model": "gpt-image-1.5",
    "prompt": prompt,
    "n": 1,
    "size": "1536x1024",      # landscape for horizontal sheets!
    "quality": "high",         # production quality
    "background": "transparent", # native transparency
    "output_format": "png"     # lossless
}
```

**Three parameters we were leaving on the table:**
1. `quality` — never set at all
2. `background` — never set, got "auto" (often opaque white)
3. `size` — always 1024×1024 even for horizontal sprite sheets

---

## 3. Prompting Best Practices (from OpenAI Cookbook)

### Prompt Structure (official recommendation)

Write prompts in this order:
1. **Background/scene** → set the visual context
2. **Subject** → what you're generating
3. **Key details** → specifics, colors, counts
4. **Constraints** → what NOT to do, invariants

For complex requests: use short labeled segments or line breaks, not one long paragraph.

### Specificity Rules

- Be concrete about materials, shapes, textures, visual medium
- For pixel art: specify "strict pixel art, flat colors, no gradients, no anti-aliasing"
- Avoid conflicting styles ("photorealistic pixel art" = bad)
- State exclusions explicitly: "no watermark, no extra text, no background elements"

### Text in Images

- Put literal text in **quotes** or **ALL CAPS**
- Spell unusual words letter-by-letter
- Specify typography: font style, size, color, placement

### Iteration Strategy

- Start with clean base prompt
- Refine with single-change follow-ups
- Re-specify critical details on each iteration (they drift)

---

## 4. Size Strategy for Sprite Sheets

### The Core Problem

Our sprite sheets are horizontal (e.g., 160×16, 128×48). Generating into a 1024×1024 square wastes vertical space — content occupies a thin horizontal band, and the downscaler loses detail.

### The Fix: Use Landscape Size

`1536x1024` gives 50% more horizontal pixels for horizontal sheets:
- 1024×1024 = 1,048,576 pixels total, content uses ~20% for horizontal strips
- 1536×1024 = 1,572,864 pixels total, content uses ~30% for horizontal strips
- That's ~50% more raw pixels for the content that matters

### Size Selection Rules

| Sheet Shape | Target Ratio | API Size | Reasoning |
|-------------|-------------|----------|-----------|
| Wide (10×1, 10×2) | ≥2:1 | `1536x1024` | Maximize horizontal resolution |
| Square-ish (8×5, 10×4) | ~1:1 | `1024x1024` | Content fills canvas well |
| Tall (4×8, 2×10) | ≤1:2 | `1024x1536` | Maximize vertical resolution |

---

## 5. Transparency Strategy

### Native Transparency (NEW — we weren't using this)

Set `background: "transparent"` in the API call. The model generates RGBA PNG with actual alpha channel.

**Caveats from community reports:**
- Works best at `medium` or `high` quality
- Can sometimes punch holes in white areas of the sprite itself (known issue with white bellies, eyes)
- Workaround: also state in prompt "opaque character sprites on transparent background, no transparent holes in character bodies"

### Prompt Reinforcement

Even with the API parameter, reinforce in the prompt:
```
"Transparent background (RGBA PNG). Each sprite is fully opaque — 
no transparent holes inside character bodies. Only the background 
between sprites is transparent."
```

### Impact on Downscaling

Native transparency means:
- No white background to confuse the mode-color picker
- Alpha channel directly usable for masking
- Easier auto-cropping (crop to non-transparent bbox)

---

## 6. Downscaling: The Real Bottleneck

### The Problem

GPT generates at 1024+ pixels. Our sprites need to be 16×16. That's a ~64:1 reduction. No algorithm preserves meaningful detail at that ratio.

Our current `smart_downscale` does majority-color block sampling with offset scanning, which is decent but:
- Loses thin features (outlines, small details)
- Struggles with white-on-white (aircraft on white bg)
- Auto-crop helps but isn't enough

### Research: Adaptive Downscaling (hiivelabs, Jan 2025)

A purpose-built pixel art downscaling algorithm that combines:

1. **Majority Color Block Sampling** — for each NxN block, pick the most frequent color
2. **Edge-preserving downscale** — detect edges via filter, downscale those separately
3. **Conditional replace** — merge edge details back into the majority-color result
4. **Transparency mask** — handle alpha properly throughout

This is similar to what we're doing but adds the edge preservation step, which helps retain outlines and structural features.

### Recommended Pipeline Upgrade

```
RAW (1536×1024, RGBA) 
  → Auto-crop to content bbox (with padding)
  → Adaptive downscale (majority-color + edge-preserve)
  → Final sprite sheet (e.g., 160×16)
  → 8× nearest-neighbor preview
```

### Alternative: Generate at Lower Resolution

Instead of generating at 1024+ and downscaling 64×, we could:
- Generate individual sprites one at a time at smaller canvas
- Generate at `1024x1024` with prompt specifying fewer, larger sprites
- Then crop and tile them manually

**Trade-off:** More API calls and assembly work, but each individual sprite gets more canvas pixels.

### The "Generate One at a Time" Strategy

For critical sprites (NPC portraits, hero characters):
1. Generate each sprite individually at 1024×1024
2. The entire canvas is ONE 16×16 sprite at ~64× zoom
3. Downscale 1024→16 with maximum detail preservation
4. Assemble the sprite sheet in code

This gives each sprite 1,048,576 pixels instead of ~10,000 pixels (1/10th of a shared sheet). That's **100× more raw data** per sprite.

---

## 7. Prompt Templates for Hearthfield Sprites

### Single Sprite (Maximum Quality)

```
Pixel art game sprite, viewed from directly above (top-down perspective).
A [DESCRIPTION] drawn at approximately 64× zoom, centered on the canvas.

The sprite should be designed to look correct when shrunk to exactly 16×16 pixels.
Use a maximum of 8-12 distinct flat colors. 
Every color region must be large enough to survive 64:1 downscaling.

STYLE: Strict pixel art. Flat colors only. No gradients, no anti-aliasing, 
no dithering, no shading transitions. Each "pixel" should be a clean square 
block of solid color approximately 64×64 canvas pixels.

CONSTRAINTS: No text, no watermarks. Transparent background.
The entire sprite must fit within the center 80% of the canvas.
```

### Grid Sprite Sheet

```
Pixel art sprite sheet for a [GAME TYPE] game.

LAYOUT: [COLUMNS]×[ROWS] grid filling the entire canvas edge to edge.
Each cell is exactly [CELL_W]×[CELL_H] canvas pixels.
Grid lines should NOT be visible — sprites are separated only by transparent gaps.

[CONTENT DESCRIPTION PER ROW]

STYLE: Strict 16-bit pixel art. Flat colors, no gradients, no anti-aliasing.
Each sprite must be immediately recognizable when displayed at 16×16 pixels.
Use bold, saturated colors with high contrast between adjacent elements.

CONSTRAINTS: Transparent background. No text, no labels, no watermarks.
Every sprite must have a UNIQUE dominant color for instant identification.
```

---

## 8. Quality Optimization Checklist

### API Parameters
- [ ] `model: "gpt-image-1.5"` (not gpt-image-1)
- [ ] `quality: "high"` (not unset/default)
- [ ] `background: "transparent"` (not auto/opaque)
- [ ] `size:` matched to sheet aspect ratio (1536×1024 for wide sheets)
- [ ] `output_format: "png"` (lossless with alpha)

### Prompt Engineering
- [ ] Structured: scene → subject → details → constraints
- [ ] Explicit grid dimensions in canvas pixels
- [ ] "Fills the ENTIRE canvas edge to edge"
- [ ] Color uniqueness requirement per sprite
- [ ] "Must be recognizable at 16×16 pixels"
- [ ] "Strict pixel art, flat colors, no gradients"
- [ ] "Transparent background, no transparent holes in sprite bodies"

### Downscaling Pipeline
- [ ] Auto-crop to content bounding box before downscale
- [ ] Use majority-color block sampling (not bilinear/bicubic)
- [ ] Edge-preservation pass for outlines
- [ ] Proper alpha handling throughout
- [ ] 8× nearest-neighbor preview for QA

### For Critical Sprites (portraits, hero, key NPCs)
- [ ] Consider generating individually (1 sprite per API call)
- [ ] Full 1024×1024 canvas for one 16×16 sprite
- [ ] Manual assembly into final sheet

---

## 9. Cost Comparison

| Setting | Per Image (1024×1024) | Per Image (1536×1024) |
|---------|----------------------|----------------------|
| gpt-image-1, no quality param | ~$0.040 | ~$0.080 |
| gpt-image-1.5, quality=high | ~$0.033 | ~$0.066 |
| gpt-image-1.5, quality=low | ~$0.011 | ~$0.022 |

Strategy: Use `quality=low` for iteration/testing, `quality=high` for final production sprites.

With 18 sprite sheets, full regen at high quality ≈ $0.60–$1.20 total. Negligible.

---

## 10. Action Plan

### Phase 1: Quick Wins (apply to all regeneration)
1. Switch to `gpt-image-1.5`
2. Add `quality: "high"`
3. Add `background: "transparent"`
4. Use `1536x1024` for horizontal sheets
5. Auto-crop before downscale

### Phase 2: Pipeline Upgrade
1. Implement edge-preserving downscale
2. Add proper alpha-aware cropping
3. Add color quantization pass (limit palette)

### Phase 3: Critical Sprite Quality
1. Generate crew portraits individually (1 per call)
2. Generate aircraft individually
3. Assemble sheets from individual high-quality sprites

---

*This research covers the complete gpt-image-1.5 API surface, OpenAI's official prompting guide, community-discovered best practices, and adaptive downscaling algorithms for pixel art. All findings are actionable for the Hearthfield DLC sprite pipeline.*

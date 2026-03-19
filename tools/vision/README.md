# Vision Kit

Two sets of tools for visual verification and analysis in AI-orchestrated projects.

## `original/` — swarm-memory vision pipeline

The production tools from the swarm-memory repo. These analyze gameplay trailers
via Gemini's video API to extract design patterns (core loops, UI, juice/game feel).

- `watch_vision.py` — Downloads video, clips segments, uploads to Gemini, writes artifact
- `batch_watch_vision.py` — Fans out watch_vision across a URL list
- `run_vision_lanes.sh` — Parallel segment analysis via strategic_orchestrator
- `synthesize_vision.py` — Consolidates artifacts into a master report table
- `synthesize_vision_100.py` — Same but for large batches (100+ games)
- `synthesize_vision_segments.py` — Merges per-segment analyses of a single video
- `analyze-rendering.py` — Playwright headless browser rendering analysis (Vale Village)
- `vision_dialogue_cutscene.md` — Analysis notes

## `generalized/` — engine-agnostic visual verification pipeline

Generalized tools for any game engine or visual project. Implements the 5-tier
verification pyramid from the research:

- **Tier 3: `screenshot_compare.py`** — Pixel-perfect comparison between reference
  and test screenshots. Zero-tolerance for pixel art, configurable threshold for
  non-deterministic renderers. Outputs diff images and structured pass/fail JSON.

- **Tier 4: `vlm_assert.py`** — Send a screenshot to any VLM (Anthropic, OpenAI,
  Gemini) with a structured assertion prompt. Returns pass/fail with reasoning.
  Provider-agnostic via `--provider` flag.

- **Tier 5: `godogen_loop.py`** — Full Godogen cycle: render → screenshot → VLM
  evaluate → report. The vision agent never sees code. Orchestrates Tier 3 + Tier 4.

- **`capture.py`** — Engine-agnostic screenshot capture. Supports Bevy headless
  (via cargo run), browser (via playwright), or arbitrary commands.

- **`batch_verify.py`** — Reads a manifest (TOML/JSON) of scenes/states and runs
  the full verification pipeline for each. CSV-compatible with spawn_agents_on_csv.

- **`reference_manager.py`** — Manages reference screenshots: accept new baselines,
  diff against existing, prune stale references.

### Quick start

```bash
# Tier 3: Compare two screenshots
python3 generalized/screenshot_compare.py \
  --reference screenshots/ref/farm_spring.png \
  --test screenshots/test/farm_spring.png \
  --tolerance 0  # pixel-perfect for pixel art

# Tier 4: VLM assertion on a screenshot
python3 generalized/vlm_assert.py \
  --image screenshots/test/farm_spring.png \
  --assertion "The player character is visible and standing on farmland. Crops are planted in rows. The HUD shows health, time, and inventory." \
  --provider anthropic

# Tier 5: Full Godogen loop
python3 generalized/godogen_loop.py \
  --manifest scenes.toml \
  --capture-cmd "cargo run --features headless -- --scene {scene} --output {output}" \
  --provider anthropic

# Batch: Run all scenes from a manifest
python3 generalized/batch_verify.py \
  --manifest scenes.toml \
  --mode tier3  # or tier4, tier5
```

### Manifest format (scenes.toml)

```toml
[scenes.farm_spring]
capture_cmd = "cargo run --features headless -- --scene farm_spring --output {output}"
reference = "screenshots/ref/farm_spring.png"
assertions = [
  "Player character is visible",
  "Crops are planted in tilled soil",
  "Season indicator shows Spring",
]

[scenes.battle_skeleton]
capture_cmd = "cargo run --features headless -- --scene battle_skeleton --output {output}"
reference = "screenshots/ref/battle_skeleton.png"
assertions = [
  "Enemy sprite is a skeleton warrior",
  "Health bars are visible for both sides",
  "Action menu shows Attack, Defend, Item, Flee",
]
```

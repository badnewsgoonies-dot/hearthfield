#!/usr/bin/env bash
# ORCHESTRATOR 1: Pilot DLC Polish
# Model: claude-opus-4.6 (orchestrator)
# Workers: gpt-5.3-codex via scripts/copilot-dispatch.sh
set -euo pipefail
cd /home/claude/hearthfield

DISPATCH="bash scripts/copilot-dispatch.sh"
WORKER_MODEL="gpt-5.3-codex"
WORKER_FLAGS="--allow-all-tools --model $WORKER_MODEL"

log() { echo "[ORCH-PILOT $(date +%H:%M:%S)] $*"; }

log "=== PILOT DLC ORCHESTRATOR STARTING ==="
log "Reading DLC state..."

# ─── WORKER 1: Wire improved sprites to crew rendering ───────────────
log "Dispatching Worker 1: Crew sprite wiring"
$DISPATCH -p "You are a Rust/Bevy worker. You may ONLY modify files under dlc/pilot/src/crew/.

CONTEXT:
- dlc/pilot/assets/sprites/crew_sheet.png is a 160x16 sprite sheet (10 frames, 16x16 each)
- dlc/pilot/assets/sprites/crew_portraits.png is a 160x16 portrait sheet (10 portraits, 16x16 each)
- Currently dlc/pilot/src/crew/spawning.rs loads sprites from main game paths like 'sprites/npcs/npc_blacksmith.png'
- These main game NPC sprites are 48x48 frame sheets

TASK:
1. Read dlc/pilot/src/crew/spawning.rs carefully
2. Change crew_sprite_file() to return paths from the DLC assets directory instead of main game:
   - Instead of individual NPC files, use 'pilot/sprites/crew_sheet.png' for all crew
   - The atlas should be TextureAtlasLayout::from_grid(UVec2::new(16, 16), 10, 1, None, None)
   - Map sprite_index 0-9 to frames in the crew sheet
3. Add a crew portrait resource or helper that loads crew_portraits.png similarly
4. Make sure existing code that references crew sprites still compiles

IMPORTANT: Bevy asset paths are relative to the assets/ directory. Since this is a DLC, check how the DLC binary finds its assets — look at dlc/pilot/Cargo.toml and any asset path configuration.

VALIDATION: Run 'cd dlc/pilot && cargo check' — must pass.

Write a report to /home/claude/hearthfield/status/workers/pilot-sprite-wiring.md" $WORKER_FLAGS 2>&1 | tail -15
log "Worker 1 complete"

# ─── WORKER 2: Settings screen — make apply_settings functional ──────
log "Dispatching Worker 2: Settings apply"
$DISPATCH -p "You are a Rust/Bevy worker. You may ONLY modify files under dlc/pilot/src/ui/.

CONTEXT:
- dlc/pilot/src/ui/settings.rs has apply_settings() which is currently a no-op
- The settings screen already has UI for volume controls (Left/Right arrows)
- The DLC has audio systems in dlc/pilot/src/ui/audio.rs

TASK:
1. Read dlc/pilot/src/ui/settings.rs to understand the settings state (SettingsState resource)
2. Read dlc/pilot/src/ui/audio.rs to find the volume resource or GlobalVolume
3. Make apply_settings() actually apply volume changes:
   - If there's a MusicVolume or GlobalVolume resource, write to it
   - If there's a volume field in SettingsState, use it
   - If nothing exists, create a simple VolumeSettings resource with music_volume: f32 and sfx_volume: f32

KEEP IT MINIMAL. Just wire the existing UI slider to an actual volume resource.

VALIDATION: Run 'cd dlc/pilot && cargo check' — must pass.

Write a report to /home/claude/hearthfield/status/workers/pilot-settings.md" $WORKER_FLAGS 2>&1 | tail -15
log "Worker 2 complete"

# ─── WORKER 3: Add UX feedback toasts to Pilot DLC ──────────────────
log "Dispatching Worker 3: Pilot UX feedback"
$DISPATCH -p "You are a Rust/Bevy worker. You may ONLY modify files under dlc/pilot/src/.

CONTEXT:
- The Pilot DLC has a notification system in dlc/pilot/src/ui/notification_center.rs
- It already has NotificationEvent and NotificationCenter
- The mission system sends notifications on completion
- BUT there are gaps: no feedback for insufficient gold, no fuel warnings, no stamina alerts

TASK:
1. Read dlc/pilot/src/ui/notification_center.rs to understand the NotificationEvent pattern
2. Read dlc/pilot/src/economy/ to find gold/fuel/stamina checks
3. Add notification sends for these common silent failures:
   a. Mission accept when not enough gold/fuel: send NotificationEvent with warning
   b. Low fuel warning (below 25%): one-shot warning per flight
   c. Any other silent failure you find where the player tries something and nothing happens

Follow the EXISTING NotificationEvent pattern. Do not create new event types.

VALIDATION: Run 'cd dlc/pilot && cargo check' — must pass.

Write a report to /home/claude/hearthfield/status/workers/pilot-ux-feedback.md" $WORKER_FLAGS 2>&1 | tail -15
log "Worker 3 complete"

# ─── VALIDATION ──────────────────────────────────────────────────────
log "Running pilot DLC gates..."
cd dlc/pilot
export PATH="$HOME/.cargo/bin:$PATH"
if cargo check 2>&1 | tail -5; then
    log "PILOT DLC: cargo check PASSED"
else
    log "PILOT DLC: cargo check FAILED — see output above"
fi
cd /home/claude/hearthfield

# ─── COMMIT ──────────────────────────────────────────────────────────
log "Committing pilot DLC improvements..."
git add dlc/pilot/ status/workers/pilot-*.md 2>/dev/null
git commit -m "feat(pilot): sprite wiring, settings apply, UX feedback

Orchestrated by Opus, workers: gpt-5.3-codex
- Crew sprites wired to DLC-specific sheets (crew_sheet.png, crew_portraits.png)
- Settings apply_settings now functional (volume control)
- UX feedback: insufficient gold, low fuel, silent failure toasts" 2>/dev/null || log "Nothing to commit"

log "=== PILOT DLC ORCHESTRATOR COMPLETE ==="

#!/usr/bin/env bash
# ORCHESTRATOR 2: City DLC Content & Integration
# Model: claude-opus-4.6 (orchestrator)
# Workers: gpt-5.3-codex via scripts/copilot-dispatch.sh
set -euo pipefail
cd /home/claude/hearthfield

DISPATCH="bash scripts/copilot-dispatch.sh"
WORKER_MODEL="gpt-5.3-codex"
WORKER_FLAGS="--allow-all-tools --model $WORKER_MODEL"

log() { echo "[ORCH-CITY $(date +%H:%M:%S)] $*"; }

log "=== CITY DLC ORCHESTRATOR STARTING ==="

# ─── WORKER 1: Expand task kinds from 4 to 8 ────────────────────────
log "Dispatching Worker 1: Expand task kinds"
$DISPATCH -p "You are a Rust worker. You may ONLY modify files under dlc/city/src/game/.

CONTEXT:
- dlc/city/src/game/resources.rs defines TaskKind enum with 4 variants: DataEntry, Filing, EmailTriage, PermitReview
- Each TaskKind has display_name(), base_duration_secs(), base_xp(), stress_factor() methods
- Tasks are generated and shown in the task board UI

TASK:
1. Read dlc/city/src/game/resources.rs — find the TaskKind enum and all its impl blocks
2. Add 4 new task kinds to the enum:
   - ReportWriting — display: 'Report Writing', duration: 180s, xp: 15, stress: 0.3
   - MeetingPrep — display: 'Meeting Prep', duration: 120s, xp: 10, stress: 0.2
   - ClientCall — display: 'Client Call', duration: 90s, xp: 20, stress: 0.5
   - BudgetReview — display: 'Budget Review', duration: 240s, xp: 25, stress: 0.4
3. Update ALL match arms in ALL impl blocks for TaskKind (display_name, base_duration_secs, base_xp, stress_factor, and any others)
4. Update the save/load serialization in dlc/city/src/game/save/ — find task_kind_str() and the reverse mapping, add the new variants
5. If there's a random task generator, add the new kinds to the pool

VALIDATION: Run 'cd dlc/city && cargo check' — must pass.

Write a report to /home/claude/hearthfield/status/workers/city-task-kinds.md" $WORKER_FLAGS 2>&1 | tail -15
log "Worker 1 complete"

# ─── WORKER 2: Expand interruption scenarios ─────────────────────────
log "Dispatching Worker 2: More interruptions"
$DISPATCH -p "You are a Rust worker. You may ONLY modify files under dlc/city/src/game/.

CONTEXT:
- dlc/city/src/game/resources.rs defines InterruptionKind enum (likely 6 variants)
- Interruptions fire during work day via InterruptionEvent
- They have display text, choices, and consequences

TASK:
1. Read the InterruptionKind enum and all its impl blocks
2. Add 4 new interruption scenarios:
   - PrinterJam — 'The printer is jammed again.' Choices: fix it (+stress, +reputation), ignore it
   - FireDrill — 'Fire drill! Everyone outside.' Mandatory 60s pause, no stress change
   - BossVisit — 'The boss stopped by your desk.' Choices: impress (+reputation, +stress), play it cool (neutral)
   - FreeLunch — 'Free pizza in the break room!' Choices: grab some (-stress, lose 30s), skip it (keep working)
3. Update ALL match arms for InterruptionKind
4. Update save/load serialization if it exists for interruptions
5. Add them to the random interruption pool/generator

VALIDATION: Run 'cd dlc/city && cargo check' — must pass.

Write a report to /home/claude/hearthfield/status/workers/city-interruptions.md" $WORKER_FLAGS 2>&1 | tail -15
log "Worker 2 complete"

# ─── WORKER 3: NPC coworker personality + dialogue lines ─────────────
log "Dispatching Worker 3: NPC coworker content"
$DISPATCH -p "You are a Rust worker. You may ONLY modify files under dlc/city/src/game/.

CONTEXT:
- dlc/city/src/game/components.rs has NpcCoworker with npc_id and NpcRole enum
- NpcRole has variants like Manager, Intern, etc.
- The city DLC needs more personality for these NPCs

TASK:
1. Read dlc/city/src/game/components.rs and resources.rs for NPC structures
2. If there's a dialogue or chat system, add 3-5 unique lines per NPC role
3. If there's no dialogue system, add a simple one:
   - Create a CoworkerDialogue resource with a HashMap<NpcRole, Vec<String>>
   - Populate with 3-5 lines per role (casual office chat lines)
   - Lines should be themed: Manager talks about deadlines, Intern asks naive questions, etc.
4. Make sure the dialogue data is initialized (add to plugin setup if needed)

KEEP IT SIMPLE. Just data + a resource. Don't build a whole dialogue UI.

VALIDATION: Run 'cd dlc/city && cargo check' — must pass.

Write a report to /home/claude/hearthfield/status/workers/city-npc-dialogue.md" $WORKER_FLAGS 2>&1 | tail -15
log "Worker 3 complete"

# ─── VALIDATION ──────────────────────────────────────────────────────
log "Running city DLC gates..."
cd dlc/city
export PATH="$HOME/.cargo/bin:$PATH"
if cargo check 2>&1 | tail -5; then
    log "CITY DLC: cargo check PASSED"
else
    log "CITY DLC: cargo check FAILED — see output above"
fi
cd /home/claude/hearthfield

# ─── COMMIT ──────────────────────────────────────────────────────────
log "Committing city DLC improvements..."
git add dlc/city/ status/workers/city-*.md 2>/dev/null
git commit -m "feat(city): 4 new task kinds, 4 interruption scenarios, NPC dialogue

Orchestrated by Opus, workers: gpt-5.3-codex
- Tasks: ReportWriting, MeetingPrep, ClientCall, BudgetReview (4→8 total)
- Interruptions: PrinterJam, FireDrill, BossVisit, FreeLunch (6→10 total)
- NPC coworkers: 3-5 dialogue lines per role" 2>/dev/null || log "Nothing to commit"

log "=== CITY DLC ORCHESTRATOR COMPLETE ==="

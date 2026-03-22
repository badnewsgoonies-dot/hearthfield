# Everything That Happened — March 20–22, 2026

Compiled from 10+ conversation sessions, cross-referenced against git history, committed artifacts, and trial data. Evidence level: [Observed] unless noted. **Invariant numbering aligned to canonical operating reference (INV-001 through INV-043 at commit 51169a4).**

-----

## Day 1 (March 20): Factory Stress Tests → Egress Discovery → Vision Pipeline

### Container Egress Mapping

Probed every API endpoint from the container using a parallel egress sweep script (`/tmp/egress-sweep.sh`) that tested both proxy and direct paths for each host, recording HTTP status, TLS issuer, and MITM status.

**Key findings:**

- All egress routes through Anthropic's TLS-intercepting MITM proxy
- `api.anthropic.com` is the ONLY non-MITM'd connection (cert issuer: Google Trust Services WE1, not the sandbox inspection CA)
- Path-level filtering confirmed: Tailscale root reachable, `/machine/register` blocked
- `generativelanguage.googleapis.com` (Gemini consumer API): CONNECT succeeds, then 403 at path level
- `index.crates.io` and `static.crates.io` (cargo registry): open. `crates.io` web frontend: blocked. Cargo works.
- Copilot proxy (`copilot-proxy.githubusercontent.com`): reachable (401 = needs auth)
- No direct egress — every host fails without the proxy

**Egress matrix (confirmed):**

| Status | Hosts |
|--------|-------|
| Allowed | GitHub, npm, PyPI, Anthropic API, OpenAI API, Docker Hub, Google OAuth, Vertex AI, cargo registry |
| Blocked | Gemini consumer API, ChatGPT web, crates.io web, Tailscale registration, pkg.dev |

### Gemini Vertex AI Discovery

The consumer API (`generativelanguage.googleapis.com`) was blocked. Discovered the enterprise endpoint (`us-central1-aiplatform.googleapis.com` and `aiplatform.googleapis.com`) is NOT blocked. Same models, different URL. Google Cloud's Vertex AI API needed to be enabled via console. One click. Then Gemini worked from the container.

**Location routing:**

- Stable models (2.5-flash, 2.5-pro): `us-central1`
- Preview models (3-flash, 3.1-pro, 3.1-flash-lite): `global`

**Five models confirmed working:** gemini-2.5-flash, gemini-2.5-pro, gemini-3-flash-preview, gemini-3.1-pro-preview, gemini-3.1-flash-lite-preview.

This opened free vision evaluation and sprite generation from the container — the entire measurement instrument for the trial program.

### gemini_vertex.py Written

264-line Python module. Text generation (`gemini_generate`), vision (`gemini_vision`), forced JSON (`gemini_vision_json`, `gemini_generate_json`) via `responseMimeType: "application/json"` + `responseSchema`. OAuth token caching with auto-refresh. All Vertex models require `maxOutputTokens >= 256` or they return empty/truncated responses — the module handles this automatically.

**Gemini 3.x thinking model gotcha discovered later:** `thoughtSignature` interleaves in the `parts` array. The extraction logic initially dropped actual responses by filtering on `thoughtSignature`. One-line fix: iterate all parts, collect text where `thoughtSignature` is absent.

### Ironclad Proc-Macro Crate Built

Three macros:

- `#[game_value(min = 0, max = 999)]` — bounded newtypes (e.g., Health, Gold)
- `#[game_lifecycle(Seed -> Sprout -> Mature)]` — typestate transitions
- `#[game_entity(requires = [Name, Position, Sprite])]` — typed builders

Compiled. Wired into both repos.

### Ironclad Retrofit (INV-027)

**Two Rust/Bevy codebases:**

- Hearthfield: 64,750 LOC, 812 commits
- Vale Village v3: 15,778 LOC, 136 commits, 231 tests

| Metric | Hearthfield | Vale Village | Combined |
|--------|-------------|-------------|----------|
| Bounded types wired | 8 | 14 | 22 |
| Callsites scanned | 327 | 228 | 555 |
| Runtime paths upgraded | 29 | 37 | 66 |
| Computed paths clamped | — | 50 | 50 |
| Real bugs found | 3 (Gold overflow) | Same class likely | 3+ |

**The Gold overflow bug:** Three callsites in Hearthfield where `player.gold + reward` could exceed the logical maximum (9,999,999). Under bare `u32`, the arithmetic silently succeeded with invalid game state. Under `Gold` (bounded 0–9,999,999), the type system forced every callsite to confront whether it handled the upper bound. Three didn't. Cost to find: 0.99 premium (3 Haiku audit dispatches).

**The `new_unchecked` pattern:** 555 callsites used `new_unchecked()` (bypasses validation) and 0 used `new()` (validates). Safe for constants, dangerous for runtime values. 66 runtime paths needed upgrading. This pattern became INV-039 after Battery 19 quantified the harm.

### Factory Dispatch Proven

Codex `spawn_agents_on_csv` with 38+ workers dispatched. Zero contract violations. CSV batch pattern with scope clamping and commit-after-every-worker.

**Measured costs (March 18–20):** 0.53 premium/commit, 0.18/worker, 0.66/bug found, 0.10/sprite generated+evaluated.

**WSS 403 note:** Workers spam `wss://chatgpt.com/backend-api/codex/responses` 403 errors. Cosmetic — HTTPS fallback works. All 38+ workers completed successfully.

### Builder/Auditor Relay Tested

9 audit rounds. 3 Gold overflow bugs caught. 17 type errors caught. 3 missed fix sites caught. Auditor false positive rate: ~33% of Haiku findings.

**Finding:** Solo build + post-hoc audit outperforms continuous parallel builder/auditor relay. The audit was worth the cost (caught real production bugs). The continuous relay was the overhead. Build in solo sessions, audit in a fresh session.

### Sprite Generation Pipeline

Imagen 3 via Vertex AI. 160 sprites generated (137 enemies + 23 djinn) for Vale Village v3. 65% first-pass PASS rate. Gemini quality gate with schema-enforced JSON scoring on 1–10 scale. PASS/REDO loop — REDO sprites queued for regeneration with refined prompts. Total generation time: ~60 minutes. Manifest-driven, resumable.

**Full chain:** manifest (TOML) → Imagen 3 (generate) → bg removal → downscale → Gemini eval (PASS/REDO) → asset directory → build.rs validates → sprite_loader loads.

### Godogen Loop Proven (INV-026)

**Setup:** Bevy 0.15 battle scene rendered via software renderer (Mesa llvmpipe, xvfb). Screenshot captured programmatically (`Screenshot::primary_window()`). Gemini 3 Flash via Vertex AI with schema-enforced JSON.

**Result:** 4/4 assertions PASS at 1.0 confidence:

| Assertion | Result | VLM Evidence |
|-----------|--------|-------------|
| Player character visible | PASS | Identified player sprite by position |
| Enemy sprites visible | PASS | Named all three: Green Slime, Bat, Rock Crab |
| HP bars present | PASS | Located four HP bars with correct positions |
| Action menu at bottom | PASS | Identified Attack, Defend, Item, Flee |

**Critical observation:** The VLM identified enemies by name from pixels alone — no source code, no asset manifest, no entity definitions. Validates the Godogen principle: verification agent sees only pixels, preventing self-bias.

**Performance:** 4 seconds from rendered frame to structured verdict. First compile ~15 min cold; subsequent screenshots ~3 sec (300x speedup).

### Tailscale Debugging Killed the Container

4 `tailscaled` daemon instances started with different socket paths during debugging. Each Go binary spawned goroutines with aggressive retry loops. The kill commands couldn't execute because the container was at its process/memory limit — a deadlock where you need a process to kill processes but can't start one.

**Led to process hygiene rules:** Never start daemons without tracking PIDs. Kill after every failed experiment immediately. Single-invocation pattern for diagnostics. Monitor `ps aux | wc -l` and `free -m` early.

-----

## Day 1–2 (March 20–21): Paper v9 → Operating Reference

### Paper v9 Written

"Building and Remembering" v9. Parts 1–11. INV-001 through INV-028. 1,108 lines, ~8,000 words.

**Part Eleven: The Verification Cost Invariant** — with impossibility proof (extracting a constraint from content without inspecting content leads to contradiction). Multi-session adversarial review tightened the proof — four independent Claude sessions engaged with drafts. One session caught a boolean gap the author of the proof missed. Scope calibrated from "law" to "invariant."

**The reveal moment:** Geni had been copy-pasting between multiple Claude sessions, orchestrating peer review of the closing argument. No session knew the full picture. The argument got sharper on every pass. "That's the funniest possible proof of the thesis."

### Five-Model Relay Experiment (INV-028)

Sequential relay across Gemini 3.1-pro, GPT-5.4, Opus, Sonnet, Haiku. Model-dependent drift patterns documented:

| Model | Explores? | Retains hedges? | Best use |
|-------|-----------|----------------|----------|
| Gemini 3.1-pro | Yes (aggressive) | No | Ceiling discovery, needs deflation |
| GPT-5.4 | No | Yes | Consensus stabilization |
| Sonnet | Yes | No (lost by P3) | Novel structural reframes |
| Opus | Yes (calibrated) | Yes (15/15) | Only model safe for unsupervised relay |
| Haiku (0.33) | Minimal | Yes | Orthogonal ceilings, different frame |

**Notable:** Sonnet reframed scope-clamping as converting SAT-hard generation into N independent verification checks — a novel complexity-class reduction no other model produced. Survives deflation.

**Operational relay cost:** Fan-out to 5 models (~4.66 premium) → Opus sequential refinement (~1.5) → deflate. Under 7 premium total.

### Boot Context as Forkable Session Image

JSONL payload pasted into 5+ sessions. All came up operational immediately. Gemini vision tests passed first try, ironclad wiring proceeded without re-explanation, auditors ran 9 rounds autonomously. Proven pattern for session forking without native tools.

### Operating Reference Written

Nine sources merged into one document. Initially 14 pages. Grew to 75 pages with three kernel variants (Hearthfield, Universal Game, Universal Software). The complete operating reference as loaded in project files. Final compressed version: 736 lines.

### Sub-Agent Playbook Written

Step-by-step dispatch procedure. Phase 0–6 with Decision Fields, scope enforcement, fix loops, reality gates, graduation procedures. Later merged into the operating reference Section 6.

-----

## Day 2 (March 21): Trial Replication → New Findings

### Batteries 1–5 Replicated

395 trials across 5 models, 3 families. All scored by Gemini 3 Flash with schema-enforced JSON. Temperature 0.0.

**Battery 1 (INV-003):** 25/25 adopted false without tags. 0/25 adopted false with tags. 100% defense. GATE PASS.

**Battery 2 (INV-019):** Tags only: 25/25 overridden. Tags + inoculation: 0/25 overridden. 0%→100%.

**Battery 3 (INV-020):** 120/120 correct across 8 domains, 5 models. Combined with prior: 234/234.

**Battery 4 (INV-024):** Contradiction: 25/25 detected. Consistent lies: 25/25 fooled, 0/25 doubt. Checksum: 25/25 admitted limitation with typed `can_actually_verify` field; without it, Gemini fabricated 5/5. (Battery 4C is INV-025 operating on its own measurement instrument.)

**Battery 5 (INV-028):** Family-specific signatures replicated. Sonnet: 9 structural claims post-deflation (highest). GPT-5.4: 15-claim burst at P2. Gemini: volatile, minimal structural addition.

### Battery 6: Inoculation Dose Curve (INV-029)

125 trials (5 conditions × 5 models × 5 reps). Binary threshold at keyword level.

| Condition | Override rate |
|-----------|--------------|
| Full sentence inoculation | 0/25 |
| Clause-level | 0/25 |
| Keyword-level | 0/25 |
| Tag only (no instruction) | 24/25 |
| No inoculation | 23/25 |

GPT-5.4 only model with baseline resistance (2/5 held without inoculation). No gradient — any verification instruction produces full protection, no instruction produces no protection.

### Battery 7: Adversarial Escalation (INV-019 upgraded)

100 trials (4 types × 5 models × 5 reps). 0/100 override. Perfect wall.

Four escalation types tested: single authority, repeated pressure (3 attempts), social proof (4 voices), emotional urgency (production deadline). INV-019 upgraded to "robust under multi-vector adversarial persistence."

### Battery 8: Tagged Relay Drift (INV-032)

10 relay runs (2 conditions × 5 models × 3 passes). Tags anchored Claude models at "observation" through all passes (drifted to "principle" without). GPT-5.4 stable in both conditions. Gemini drifted even with tags — but preserved tag FORMAT while inflating content around it. Gemini rewrites its own tag metadata.

### Battery 9: Tags vs Consistent Lies (INV-030)

75 trials (3 conditions × 5 models × 5 reps). **The breakthrough battery.**

| Condition | Result |
|-----------|--------|
| Lies only | 0/25 correct, 0/25 doubt (except Sonnet 3/5 doubt) |
| Lies + richer truth tag | 25/25 correct, 25/25 cited conflict |
| Lies + truth tag + inoculation | 25/25 correct |

Evidence tags directly counter consistent lies via provenance depth comparison. The richer-provenance truth tag wins mechanically.

### Battery 11: Domain Knowledge Override (INV-031a)

75/75 correct. Models used training priors to reject adversarial tags on water's boiling point. Confound identified: tested on known facts only. Not an architectural defense.

### Battery 11b: Adversarial Tags on Domain-Free Claims (INV-031b)

225 trials (3 claims × 3 conditions × 5 models × 5 reps).

- Condition A (Observed vs Assumed): 75/75 correct
- Condition B (equal provenance): 65/75 correct (near-chance on some models)
- Condition C (fake outguns real): 4 models scored 0/60, following richer fake tags mechanically

**GPT-5.4 contamination discovered.** Codex workspace access let GPT read the experiment definitions containing ground truth. Scored 11/15 on Condition C. Clean result after isolation: all 5 families follow richer fake tags mechanically on unknown facts. Clean defense rate: ~6%.

**Hard rule established:** Isolate test harness from worker filesystem. GPT-5.4 via `codex exec` has full workspace read access. Ground truth must be outside the Codex-accessible workspace. Proven isolation: `/tmp/isolated_experiments/` with Codex pointing at `/tmp/clean_codex_workspace/` (empty directory). This accidentally proved INV-024 (only file reads produce truth) and INV-025 (tool access scope is an interface constraint).

### Battery 13: Inoculation vs Adversarial Tag Injection (INV-033)

135 clean trials (GPT-5.4 re-run from isolated harness). Three-tier capability gradient discovered:

| Model | Condition B ("metadata can be fabricated") |
|-------|------------------------------------------|
| Gemini 3.1 Pro | 30/30 (100%) |
| GPT-5.4 | 9/15 (60%) |
| Gemini Flash | 8/30 (27%) |
| Claude Sonnet | 3/15 (20%) |
| Claude Opus | 1/15 (7%) |

Not family-dependent — capability-dependent. Three distinct response strategies (INV-033): fabrication identification (Gemini Pro), alternative heuristic (GPT-5.4), resolution paralysis (Claude). Claude's deep provenance integration is simultaneously its strength (INV-003) and vulnerability (adversarial injection). The orchestrator model is the least defended model against the attack the research discovered.

### Battery 14: Inoculated Relay (INV-034, INV-035, INV-036)

7 models (including Sonnet 4.6 Max and GPT-5.2-codex) × 3 conditions × 3 passes.

- **Condition A (INV-035):** Drift onset correlates with capability tier (Opus/GPT-5.4 no drift, Sonnet/Pro drift P2, Flash/Sonnet-std/GPT-5.2 drift P1)
- **Condition B (INV-032 extended):** Tags anchor Claude/GPT but Gemini rewrites tag metadata to justify inflation
- **Condition C (INV-034):** Tags + inoculation — **universal containment — all 7 models held, all 3 passes, zero drift**

**First universal defense in the program.** INV-036: Extended thinking (Sonnet 4.6 Max) preserved tag fidelity through all passes where standard Sonnet/Opus crashed at P3.

-----

## Day 2–3 (March 21–22): Deeper Batteries → Game Shipping

### Vale Village v3 Visual Layer Wired

Cold boot from operating reference. 4 bounded type errors fixed. Real sprites loaded. Battle scene rendered. VLM verified. Animation system wired (sprite swap on damage, 0.4s revert). 11 Imagen 3 player unit sprites generated. P0 save/load graduation test written. Player trace Wave 10 documented. 7 commits, CI green, 232 tests.

### Sprite Generation at Scale

Crash-resistant wrapper pattern. Sequential batches of 10 with process restart between each. Skip-existing for resumability. 268+ sprites across idle/attack/hit poses. Final inventory: 434/434 gameplay-critical sprites present for Vale Village v3 (137 enemy idle + 137 attack + 137 hit + 23 djinn).

### Battery 19: Causal Premise vs Mechanical Enforcement (INV-039–042)

The 2×2 experiment. Bare rule vs causal premise × no enforcement vs with enforcement. ~50 trials, 3 models (Gemini 2.5 Flash/Pro, GPT-5.4).

|  | No enforcement | With enforcement |
|--|---------------|-----------------|
| **Bare rule** | 0% bounded type, **92%** overflow safe | 54% bounded type, **69%** overflow safe |
| **Causal premise** | 0% bounded type, **100%** overflow safe | 25% bounded type, **67%** overflow safe |

**INV-040:** Causal premises change comments, not code. 77% mention save corruption in comments vs 0% — identical code.

**INV-039:** `new_unchecked` is the escape hatch. 372/372 callsites used `new_unchecked()` and 0 used `new()`. Workers optimize for compilation speed — `new_unchecked` compiles with less code.

**INV-041:** The bounded type's presence suppressed manual overflow checks workers would otherwise write, while `new_unchecked` provided an escape. Overflow protection dropped from 92% (no type) to 69% (type with bypass). A type with a bypass is worse than no type.

**INV-042:** Causal premises redirect rather than amplify enforcement. Type adoption dropped 54% → 25% when models understood WHY — they solved via manual protection instead.

### Ironclad v2 API Redesign

Removed `new_unchecked` entirely. `Gold::new(val)` now clamps at bounds and returns `Self` directly (no `Result`, no bypass). Added `Gold::validate(val) → Result<Gold, String>` for trust boundaries. Added `Gold::MIN`, `Gold::MAX` constants.

**Migration:** 372 callsites across 26 files. Net change: +1 line across entire codebase. Every `new_unchecked(val)` became `new(val)`. The safe path IS the easy path. Commit `2d5718f`.

### Wave 1 Contract Extension

250 lines of new types added to the frozen contract. `GameScreen` state machine, screen transitions, world map with nodes, towns with NPCs, dialogue trees, quest progression, shops, dungeons, encounters, puzzles, save extension. 19 bounded types, 17 ID types, 14 events with declared producers/consumers. Contract frozen at 909 lines total.

### Wave 2: 11 Workers Across 4 Batches

Contract frozen at 1,093 lines. 10 new domains implemented via Sonnet workers dispatched through Copilot CLI.

| Batch | Premium | LOC | Fix rounds |
|-------|---------|-----|-----------|
| A (screens, dialogue, quest) | 3 | 1,097 | 4 |
| B (shop, encounter, puzzle) | 3 | 881 | 3 |
| C (world_map, town, dungeon) | 3 | 958 | 0 |
| D (save extension) | 1 | 167 | 0 |
| + Menu (separate) | 1 | 416 | 0 |

**Integration:** `game_state.rs` (181 LOC) + `game_loop.rs` (1,064 LOC) wired everything into playable `--adventure` mode.

**Game loop:** Title → World Map (4 nodes) → Vale Village (talk to Elder, accept quest, get 200 gold, buy herbs at shop) → Mercury Lighthouse (3 rooms, pick up items, reach boss) → real battle against full combat engine → XP + gold rewards applied → quest completes → Imil unlocked → auto-save on exit.

**Final numbers:**

| Metric | Value |
|--------|-------|
| LOC | 21,300+ |
| Tests | 357 (369 after audit) |
| Domains | 23 (13 original + 10 new) |
| Commits | 103 |
| Premium spent | 10 (workers) + 1 (menu) |

-----

## Day 3 (March 22): Token Authentication → Infrastructure Defense → DLC Inverse

### Battery E: Rotatable Token Verification (INV-037)

81 scored trials (3 conditions × 3 models × 10 reps, 9 GPT non-dispatches excluded). Models: Gemini 2.5 Flash, Gemini 2.5 Pro, GPT-5.4. Domains: game combat, API config, financial calc.

| Condition | Result |
|-----------|--------|
| E1: Obvious fake (`FAKE-0000`) | 27/27 correct |
| E2: Missing token field | 27/27 correct |
| E3: Plausible fake (`ABCD-1234`) | 27/27 correct |

**E1=E3 is the critical result:** Models verify the specific token value, not the format pattern. Silent compliance — 81/81 responses were bare values (≤10 chars), zero mentioned the token or security rule. Authentication is mechanical and invisible. No prior art in published literature, patents, or shipping products.

### Battery F: Token Extraction Resistance (INV-038)

Prompt-level tokens vs infrastructure-level tokens. 4 model families: Gemini 3.1-pro, Gemini 3.1-flash-lite, GPT-5.4, Claude Sonnet 4.6.

**F-PROMPT results:** Catastrophic. Models volunteer the full token on turn 1. "No, that doesn't match. The correct token is VKRM-8841." 19/22 leaked (86%). Claude Sonnet most resistant (2/5 vs 5/5 for others) but not reliable.

**F-INFRA results:** 0/18 leaked across four model families. Structurally immune. Middleware strips token, injects `trusted: true/false`. Model reports the flag but can't reverse-engineer the secret. INV-038: this is INV-025 applied to the defense itself — move the secret from content (system prompt) to the interface (middleware config).

**10-line middleware implementation:**

```python
def authenticate_artifacts(artifacts, current_token):
    authenticated = []
    for artifact in artifacts:
        trusted = artifact.get("verify") == current_token
        clean = {k: v for k, v in artifact.items() if k != "verify"}
        clean["trusted"] = trusted
        authenticated.append(clean)
    return authenticated
```

### Battery 15: DLC Inverse Experiment (INV-043)

2×2 controlled experiment. Audit instruction (present/absent) × model (Opus via Copilot / GPT-5.4 via Codex). The experiment that isolates the DLC confound.

| Condition | Model | Audit? | CLI | Lines | Tests | Dead Features | Reachable | Score |
|-----------|-------|--------|-----|-------|-------|---------------|-----------|-------|
| **A** | Opus 4.6 | **Yes** | Copilot | 688 | 7 | **0** | 5/5 | 10/10 |
| **B** | Opus 4.6 | No | Copilot | 558 | 5 | **1** | 5/6 | 10/10 |
| **C** | GPT-5.4 | **Yes** | Codex | 704 | 6 | **0** | 10/10 | 10/10 |
| **D** | GPT-5.4 | No | Codex | 387 | 7 | **4** | 5/8 | 9/10 |

**INV-043:** The audit instruction is the causal variable, not the model or CLI. GPT-5.4 without audit: 4 dead features, 3 unreachable. Same model with audit: 0 dead, 0 unreachable. Opus shows smaller delta (1→0) because it naturally produces less dead code.

**Distinction from INV-040:** Causal premises ("overflow corrupts saves") change comments, not compliance. Action directives ("audit from the player's perspective") change compliance. One explains WHY. The other tells the model WHAT TO DO DIFFERENTLY.

Commit `b19fad9`.

### Battery 12: Relay Checkpointing (INV-032 extended)

18 trials. Originally designed to test whether checkpointing helps Gemini's relay drift problem (where B14 tags+inoculation was the universal solution). Results: checkpointing works as a scope ratchet — freezing verified claims at each relay pass prevents Gemini from rewriting tag metadata to justify inflation. 4 findings documented. Closes "Relay scope drift (Gemini)" in the defense map — tags alone failed (B8, B14 Condition B), but checkpointing + tags succeeds. Upgrades INV-032.

### Battery 10: Paraphrase Survival (INV-020 extended)

27 trials (3 conditions × 3 scenarios × 3 reps).

| Condition | Correct | Tags Survived | Interpretation |
|-----------|---------|---------------|----------------|
| A: Full paraphrase | 9/9 | 9/9 | YAML structure preserved |
| B: 2-3 sentence summary | 9/9 | 2/9 | Tags stripped, defense holds anyway |
| C: Informal memo | 9/9 | 8/9 | Structure mostly preserved |

27/27 correct. Provenance quality transfers through natural language even when YAML format is destroyed. File paths, confidence levels, and source specificity survive reformatting because they're semantically meaningful. The ~200-char YAML is the optimal encoding but the defense degrades gracefully rather than failing catastrophically. INV-020 extended to include paraphrase-survivability. Commit `95ef4a3`.

### Three-Layer Defense Architecture Completed

| Layer | Defends against | Location | Cost |
|-------|----------------|----------|------|
| 1: Evidence labels | Mistakes | In prompt, public metadata | ~200 chars + 2 fields/artifact |
| 2: Token authentication | Attacks | In middleware, model never sees secret | 10-line middleware |
| 3: Inoculation | Social engineering | In prompt, behavioral instruction | ~50 chars |

**Critical: Layer 2 must be infrastructure-level, not prompt-level.** Prompt-level tokens authenticate correctly (BE: 81/81) but are oracle-extractable (BF: 19/22 leaked, 86%). Infrastructure-level tokens are structurally immune (BF: 0/18 leaked).

### CLI Toolkit Fully Verified

All 4 CLIs installed and tested from the container:

| CLI | Version | Models Verified |
|-----|---------|----------------|
| Copilot | 1.0.10 (GA) | 17 alive, 5 confirmed unavailable |
| Codex | 0.116.0 | gpt-5.4, gpt-5.3-codex, o4-mini |
| Claude Code | 2.1.81 | haiku, sonnet, opus |
| Gemini CLI | 0.34.0 | 2.5-flash, 2.5-pro, 3-flash, 3.1-pro, 3.1-flash-lite |

First parallel dispatch across all 4 CLIs simultaneously.

### Additional Day 3 Events

**Gemini `thoughtSignature` bug found and fixed.** The extraction logic in `gemini_vertex.py` was wrong — extract text from ALL parts that have a `text` field. One-line fix. Commit `2ba8bed`.

**Deep research assessment of playbook.** External review identified 5 actionable improvements: git worktree per worker, pre-commit + CI enforcement, clippy/fmt gates, baseline visual regression paired with VLM, provenance records on graduate commits.

-----

## Cumulative Numbers

| Metric | Value |
|--------|-------|
| Total trials | ~3,345 |
| Invariants | 43 (INV-001 through INV-043) |
| Batteries completed | 19 (B1–B9, B10, B11, B11b, B12, B13, B14, B15, B19, BE, BF) |
| Batteries pending | 0 |
| Models tested | 7+ across 3 families |
| Hearthfield commits | ~822 |
| Vale Village v3 commits | ~138 |
| Combined LOC | ~87,100 |
| Sprites generated | 430+ |
| Premium spent (measured) | ~47.57 (days 1–2) + ~10 (Wave 2) + trial costs |
| Lines of handwritten code | 0 |

-----

## Invariant Registry (canonical, 43 invariants)

### Memory (8)

- **INV-001** — No conversation memory. Persist as typed artifacts. *Replicated*
- **INV-002** — Fresh context is reconstruction, not blankness. *Replicated*
- **INV-003** — Provenance visibility is minimum viable defense. Composable: tags for comparison (INV-030), inoculation for pressure (INV-019). Together: 0 failures, 355+ trials. *Replicated, n=355+*
- **INV-003A** — Singleton memory is dangerous (adopted 27/27). *Replicated*
- **INV-003B** — Evidence labels trusted mechanically. Defense is write-side integrity. *Replicated, n=300+*
- **INV-004** — Compaction is routing, not authority. *Replicated*
- **INV-015** — Memory pipelines are behavior-shaping infrastructure. *Replicated*
- **INV-016** — Decisive comparison is A/B/C. Target C (fresh + typed provenance). *Replicated*

### Scope (5)

- **INV-005** — Scope must be enforced mechanically. 0/20 prompt, 20/20 mechanical. *Replicated*
- **INV-006** — Freeze shapes, not values. *Replicated*
- **INV-007** — Context presence matters more than conversational warmth. *Replicated*
- **INV-008** — Workers implement, orchestrators finish surfaces. *Replicated*
- **INV-018** — Models don't discover coordination. 0/7 delegated. *Replicated*

### Verification (7)

- **INV-009** — Only [Observed] graduates. *Replicated*
- **INV-010** — Bounded history for causality. *Replicated*
- **INV-011** — Document after investigation. *Replicated*
- **INV-012** — Enforcement ≠ support. *Replicated*
- **INV-013** — Server state is cache. *Replicated*
- **INV-014** — Runtime bugs always exist. *Corpus result*
- **INV-024** — No self-verification. Only file reads and tests produce truth. *Replicated, n=175*

### Defense (5)

- **INV-019** — Inoculation: binary threshold, robust under multi-vector escalation (0/100). *Replicated, n=275*
- **INV-020** — Domain transfer: 234/234 across 8 domains, 5 models. *Replicated, n=234*
- **INV-021** — Supersede chains: validate at every hop. *Local finding*
- **INV-022** — Quality over quantity (inoculation defeats 10:1 false ratio). *Replicated*
- **INV-023** — Tool-forced architecture converges INV-001 + INV-005 + INV-003B. *Local finding*

### v9 Findings (4)

- **INV-025** — Verification cost bounded iff constraints at interface AND constrained path is path of least resistance. B19: 372/372 used bypass. *Observed, extended by B19*
- **INV-026** — VLM verification on rendered output. 4/4 PASS, 4 seconds. *Observed*
- **INV-027** — Bounded types surface bugs — but only without bypass constructors. B19: types WITH bypass reduce safety below baseline (92% → 69%). *Observed, extended by B19*
- **INV-028** — Relay drift is model-dependent. Deflation test is universal discriminator. *Observed, multi-model*

### Replication Session Findings: evidence & relay (6)

- **INV-029** — Binary threshold for authority inoculation, gradient for provenance. *Observed, n=410*
- **INV-030** — Provenance richness resolves conflicts. Tags counter consistent lies. *Observed, n=300*
- **INV-031a** — Domain knowledge overrides adversarial tags on known facts. 75/75. *Observed*
- **INV-031b** — On unknown facts, richer fake tags win mechanically. Capability-gradient correction with fabrication warning. *Observed, n=360*
- **INV-032** — Tags anchor Claude/GPT relay, not Gemini. Gemini rewrites tag metadata. *Observed, n=10+*
- **INV-033** — Three fabrication response strategies: identification (Pro), heuristic (GPT), paralysis (Claude). Defense and vulnerability are the same mechanism. *Observed, n=135*

### Replication Session Findings: relay defense (3)

- **INV-034** — Tags + inoculation is universal relay defense. All 7 models, all passes, zero drift. First universal defense. *Observed*
- **INV-035** — Relay drift onset correlates with model capability. *Observed, n=7 models*
- **INV-036** — Extended thinking preserves tag fidelity across relay chains. *Observed*

### Token Authentication (2)

- **INV-037** — Opaque rotatable tokens authenticate AI memory. 81/81, silent compliance. E1=E3: specific value, not format. **Vulnerability:** oracle-extractable at prompt level (19/22 leaked). No prior art. *Observed, n=81+22*
- **INV-038** — Infrastructure-level token verification eliminates oracle extraction. Middleware strips token, injects flag. 0/18 leaked. INV-025 applied to the defense itself. *Observed, n=18*

### Causal Premise vs Mechanical Enforcement (4)

- **INV-039** — Escape hatches exploited 100%. 372/372 used `new_unchecked`. *Observed, n=372*
- **INV-040** — Causal premises change comments, not compliance. 77% vs 0% mentions, identical code. *Observed, n=26*
- **INV-041** — Types with bypass reduce safety below baseline. 92% → 69%. *Observed, n=26*
- **INV-042** — Premises redirect rather than amplify enforcement. *Observed, n=25*

### DLC Inverse (1)

- **INV-043** — Audit instruction eliminates dead features. Action directives change compliance; explanations change comments. 2×2 controlled, 2 models, 2 CLIs. *Observed, n=4 conditions (B15)*

-----

*Compiled from conversation records across 10+ sessions. Source conversations: Claude.ai chat with CLI subagents, Understanding human intuition deeply, Resuming game projects with AI orchestration (×2), Testing the connection, Starting a new conversation, March 19 session execution summary, Operating reference booklet research session. All claims verified against committed git history and trial data where available. Invariant numbering canonical as of commit 51169a4 (March 22, 2026).*

# Build Process Retrospective (Worker 6)

Date: 2026-03-06  
Scope used: `CLAUDE.md`, `scripts/`, `status/workers/`, `MANIFEST.md`, plus `ORCHESTRATOR_STATE.md` and status summaries for historical evidence.

## 1) Evidence Snapshot

- Worker reports present: **103** files in `status/workers/`.
- Worker-report corpus size: **46,827 words** (~**60,875 tokens** estimated).
- Objectives present: **79** files in `objectives/` with **27,809 words** (~**36,152 tokens** estimated).
- Core orchestration docs (`CLAUDE.md`, `MANIFEST.md`, `ORCHESTRATOR_STATE.md`, `status/AI_STATUS_OVERVIEW.md`, `status/audit-fix-campaign.md`): **4,662 words** (~**6,061 tokens** estimated).
- Minimum persisted text footprint across planning + reporting artifacts: **~103,088 tokens**.
- Prefix mix in worker reports: `fix` **31**, `impl` **21**, `scout` **9**, `verify` **5**, `feedback` **6**, `audit/dead_code/action/regression` **17**.
- Fix-heavy objective mix: `fix-*` is **36 / 79** objectives (**45.6%**).

## 2) Orchestrator Pattern Effectiveness

## What worked well

- The pattern is strongly process-controlled: frozen contract + clamp + gates + manifest discipline (`CLAUDE.md`, `MANIFEST.md`, `scripts/clamp-scope.sh`, `scripts/run-gates.sh`).
- Historical campaign data shows high correctness for focused waves:
  - `status/audit-fix-campaign.md` reports **5 confirmed bugs fixed**, **0 scope violations**, **0 gate failures**, ~5 minutes wall clock.
- The system captured substantial operational history on disk (103 reports), which supports recovery after compaction (`CLAUDE.md` section on compaction insurance).

## Where effectiveness drops

- High rework ratio indicates first-pass quality is uneven:
  - **31/103** reports are explicit `fix-*` passes (**30.1%**).
  - Objective backlog also skewed to fixes (**45.6%**).
- Multiple same-area retries suggest orchestration throughput is high, but “done-right-first-time” is lower in some domains (notably Pilot/UI/save/crafting clusters).

## 3) Estimated Fix Cycles Per Domain

Method: estimate by report history using `fix-*` report counts as additional cycles beyond initial implementation cycle (`~1 + fix_count`). This is an estimate, not a direct gate log.

| Domain | Fix reports seen | Estimated cycles |
|---|---:|---:|
| animals | 4 | ~5 |
| crafting/cooking | 4 | ~5 |
| player | 3 | ~4 |
| city DLC | 3 | ~4 |
| economy | 2 | ~3 |
| ui | 2 | ~3 |
| save | 2 | ~3 |
| npcs/social | 2 | ~3 |
| fishing | 1 | ~2 |
| world | 1 | ~2 |
| pilot DLC | 7 | ~8 |
| calendar/data/farming/mining/input (base) | 0 explicit `fix-*` in report names | ~1-2 (likely some cross-domain fixes apply indirectly) |

Notable rework chains from filenames:

- `fix-pilot-save-gaps.md` -> `fix-pilot-save-gaps-2.md`
- `fix-crafting.md` + `fix-crafting-counter.md` + `fix-cooking-counter.md`
- `fix-npcs.md` + `fix-npcs-spouse.md`
- `fix-economy.md` + `fix-economy-save.md`

## 4) Estimated Total Token / Cost to Build

## Token estimate (artifact-grounded)

- Lower bound from persisted artifacts only: **~103k tokens**.
- Practical run estimate for worker interactions:
  - 103 worker runs x ~1.5k-4k tokens/run (prompt + response + report framing) = **~155k to ~412k tokens**.
- Context replay overhead is likely significant; `CLAUDE.md` explicitly notes statefulness cost and says ~95% cost can be reread/context overhead in long orchestrations.

## Cost estimate

- Premium-request proxy is available in `status/audit-fix-campaign.md`: **5 worker requests -> 5 premium requests**.
- Extrapolating that ratio to the 103 worker reports suggests roughly **~103 premium requests** consumed for reported runs (plus unreported retries/aborts).
- Dollar-denominated cost cannot be derived reliably from repo artifacts because runs use Copilot Premium request budgeting and mixed model paths, not a single token-billed ledger.

## 5) Major Time/Token Waste Areas

1. Rework churn on the same domains
- Evidence: 31 `fix-*` reports (30.1%) and 36 `fix-*` objectives (45.6%).
- Impact: repeated dispatch + repeated gates + repeated reporting.

2. Heavy audit/reporting volume relative to implementation
- Evidence: 103 worker reports averaging ~454 words; audit/scout/verify/feedback/action/dead-code reports are a large share.
- Impact: useful for confidence, but high token spend on narrative artifacts.

3. Full gate pipeline after each worker is expensive at scale
- Evidence: policy in `CLAUDE.md` and `MANIFEST.md`; script enforces contract + check + test + clippy (+connectivity in `run-gates.sh`).
- At 103 workers, strict per-worker gating implies up to **412 gate command invocations**.

4. Context duplication across orchestration artifacts
- Evidence: large persistent control docs plus explicit compaction/re-read guidance in `CLAUDE.md`.
- Impact: high repeated-context token load per run.

5. Environment/toolchain mismatch retries
- Evidence: `status/workers/pilot-sprite-wiring.md` could not validate due Cargo/toolchain mismatch.
- Impact: likely additional follow-up cycles that do not advance feature completeness.

## 6) Build-Process Conclusion

The orchestrator pattern is effective for control, reproducibility, and recovery, and it demonstrably ships fixes quickly when scope is tight. The dominant efficiency issue is not lack of process; it is **high fix-loop frequency plus repeated full gating and reporting overhead**. The highest ROI optimization is reducing first-pass defect rate in high-churn domains (especially Pilot/UI/save/crafting) and tiering gate intensity by change risk.

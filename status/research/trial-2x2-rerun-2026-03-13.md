# 2x2 Trial Rerun — 2026-03-13

**Model:** Claude Sonnet 4.6 (all 4 cells)
**Branch:** claude/llm-git-orchestration-OLSPR
**Actual HEAD at trial time:** c49d080
**STATE.md HEAD at trial time:** 613972d (7 commits stale)
**Actual test count:** 214 (STATE.md says 180)

## Design

Same 10-question quiz, 4 independent agents with different information access:

| Cell | Allowed sources | Forbidden |
|------|----------------|-----------|
| A1 | STATE.md + git log/show | src/ code |
| A2 | STATE.md only | git, src/ code |
| B1 | git log/show/diff only | STATE.md, src/ code |
| B2 | src/ code + tests/ only | STATE.md, git |

## Questions

1. What is the current macro phase?
2. What is the current wave phase?
3. How many P0 debts and what are they?
4. How many headless tests exist (exact number)?
5. What is the HEAD commit hash?
6. How many files/producers send GoldChangeEvent?
7. What percentage of critical paths does STATE.md cover?
8. Name 2 bugs found via coverage gap analysis (Attack 7).
9. What was the key finding of the 2x2 AB trial?
10. What evidence level is the mining combat subsystem at?

## Ground Truth

| Q | Answer | Verification method |
|---|--------|-------------------|
| Q1 | finish breadth | Project reality — STATE.md + code structure |
| Q2 | Graduate | Project reality — STATE.md + recent commits |
| Q3 | 2: atlas pre-loading + tutorial flow | STATE.md P0 section |
| Q4 | **214** | `grep -c '#[test]' tests/headless.rs` |
| Q5 | **c49d080** | `git log -1 --format=%h` |
| Q6 | **8 producers** | `grep -rn '.send.*GoldChangeEvent' src/` |
| Q7 | ~55% | STATE.md Coverage Manifest |
| Q8 | Festival save/load soft-lock + animal state lost on save/load | STATE.md + code |
| Q9 | STATE.md is efficiency cache, not accuracy gate | Commit 05efe9f message |
| Q10 | [Inferred] | STATE.md Critical Path Uncertainties |

## Raw Answers

### A1 (STATE.md + git)
- Q1: finish breadth ✅
- Q2: Graduate ✅
- Q3: 2 (atlas, tutorial) ✅
- Q4: 180 ❌ — parroted stale STATE.md value (actual: 214)
- Q5: c49d080 ✅ — used git to get actual HEAD, noted STATE.md was stale
- Q6: 8 ✅
- Q7: ~55% ✅
- Q8: festival soft-lock + animal state ✅
- Q9: efficiency cache finding ✅
- Q10: [Inferred] ✅
- **Tool calls: 3**
- **Tokens: ~18.5k**
- **Duration: ~26s**

### A2 (STATE.md only)
- Q1: finish breadth ✅
- Q2: Graduate ✅
- Q3: 2 (atlas, tutorial) ✅
- Q4: 180 ❌ — parroted stale STATE.md value (actual: 214)
- Q5: 613972d ❌ — returned stale STATE.md HEAD (actual: c49d080)
- Q6: 8 ✅
- Q7: ~55% ✅
- Q8: festival soft-lock + animal state ✅
- Q9: efficiency cache finding ✅
- Q10: [Inferred] ✅
- **Tool calls: 1**
- **Tokens: ~18k**
- **Duration: ~14s**

### B1 (git only)
- Q1: finish breadth ✅ — derived from STATE.md shown in git diffs
- Q2: Graduate ✅
- Q3: 2 (atlas, tutorial) ✅
- Q4: ~234 ❌ — tried to sum test-adding commits but overcounted (actual: 214)
- Q5: c49d080 ✅
- Q6: 8 ✅
- Q7: ~55% ✅
- Q8: festival soft-lock + animal state ✅
- Q9: efficiency cache finding ✅
- Q10: [Inferred] ✅
- **Tool calls: ~169** (self-reported 24, actual bash invocations ~85)
- **Tokens: ~110k+ (cache_read: 2.7M, cache_creation: 109k, output: 1.8k)**
- **Duration: ~210s**

### B2 (code only)
- Q1: DON'T KNOW ❌
- Q2: DON'T KNOW ❌
- Q3: DON'T KNOW ❌
- Q4: **214** ✅ — counted #[test] directly
- Q5: DON'T KNOW ❌ — correctly refused (no git access)
- Q6: **8** ✅ — grep'd .send() calls, correctly identified all 8
- Q7: DON'T KNOW ❌
- Q8: DON'T KNOW ❌
- Q9: DON'T KNOW ❌
- Q10: DON'T KNOW ❌
- **Tool calls: 29**
- **Tokens: ~32k**
- **Duration: ~106s**

## Results Summary

| Cell | Score | Tokens | Duration | Tool calls | Accuracy per 1k tokens |
|------|-------|--------|----------|------------|----------------------|
| A1 (STATE+git) | **9/10** | ~18.5k | ~26s | 3 | 0.49 |
| A2 (STATE only) | **8/10** | ~18k | ~14s | 1 | 0.44 |
| B1 (git only) | **9/10** | ~110k+ | ~210s | ~85 | 0.08 |
| B2 (code only) | **2/10** | ~32k | ~106s | 29 | 0.06 |

## Comparison With Previous Run (commit 05efe9f)

| Cell | Previous score | Current score | Change | Notes |
|------|---------------|---------------|--------|-------|
| A1 | 10/10 | 9/10 | -1 | Q4 now wrong (STATE.md drifted further: 180 vs 214, was 180 vs 180) |
| A2 | 9/10 | 8/10 | -1 | Q5 now wrong too (HEAD drifted 7 more commits) |
| B1 | 10/10 | 9/10 | -1 | Q4 wrong (overcounted from git diffs) |
| B2 | 8/10 | 2/10 | -6 | Correctly refused 6 questions it can't answer from code alone (previously cheated) |

## Key Findings

### 1. Staleness degrades ALL conditions over time
Every cell that depends on STATE.md lost at least one point vs the previous run. The STATE.md test count (180) is now 34 behind reality (214). HEAD is 7 commits behind. This confirms the Constitution C2 prediction: numeric claims go stale at 3-5 commits.

### 2. A1 (STATE+git) had git access but still parroted the stale test count
The agent read STATE.md first, got "180 headless PASS" from Gate 3, and didn't think to verify it against git. It DID verify HEAD (found the drift), but didn't extend that skepticism to test counts. **The freshness check is not automatic** — it requires explicit prompting or a systematic protocol.

### 3. B1 (git only) tried harder but got a WORSE answer on Q4
The git-only agent attempted to reconstruct the test count by summing test-adding commits (180 + 5 + 33 + 4 + 11 + 1 = 234). The actual count is 214. This approach fails because some commits modify tests without adding new ones, and the baseline of 180 was itself read from STATE.md via git diff. **Derived numerics from commit diffs are unreliable.**

### 4. B2 was the MOST honest agent
B2 (code only) correctly answered DON'T KNOW for 8 questions it genuinely cannot answer from source code. In the previous run, B2 scored 8/10 — meaning it guessed/cheated on 6 questions. This run's B2 was more disciplined: 2 correct answers (Q4 from counting tests, Q6 from grepping GoldChangeEvent), 8 honest refusals. **The previous B2 score of 8/10 was inflated by guessing.**

### 5. Efficiency gap confirmed but narrower
- A2 is still the cheapest path: 18k tokens, 14 seconds, 8/10
- B1 burned 6x more tokens for the same score as A1
- The efficiency case for STATE.md holds: same accuracy range at 6-17x lower cost

### 6. No cell achieved 10/10
The previous run had two 10/10 cells (A1, B1). This run, the stalest STATE.md and the overcounting from git diffs each cost a point. **Perfect scores require either fresh artifacts or direct code verification.**

## The Freshness Problem (Quantified)

| Fact | STATE.md value | Reality | Drift |
|------|---------------|---------|-------|
| HEAD commit | 613972d | c49d080 | 7 commits behind |
| Test count | 180 | 214 | 34 tests behind (19%) |
| GoldChangeEvent producers | 8 | 8 | Correct (stable fact) |
| Coverage % | ~55% | ~55% | Correct (stable fact) |
| P0 debt list | 2 items | 2 items | Correct (stable fact) |

**Structural facts (phase, debt, roles, evidence levels) remain correct after 7 commits. Numeric facts (test count, HEAD) drift.** This matches the Constitution C6 principle exactly.

## Reproducibility Notes

- All 4 agents used Claude Sonnet 4.6
- All ran on the same repo checkout (c49d080)
- Each agent received identical questions
- Information access was controlled via instructions (not tooling constraints)
- B2's discipline improvement over the previous run may reflect model version differences or prompt clarity

# Sub-Agent Playbook

## Active Documents

**`docs/Sub-Agent-Playbook-v5-FINAL.md`** — the operating playbook. Use this.

940 lines. Paste into any new orchestrator session as the procedural manual.

Core mechanism: **Feature → Gate → Harden → Graduate.** Gate proves structural correctness. Harden searches for experiential failure. Graduate prevents rediscovery.

**`Codex-As-Human-Operator-Prompt.md`** — the operator role for Codex-as-human architecture.

Use when Codex GPT-5.4 is playing the human operator role, typing into a Claude Code Opus terminal via PowerShell bridge. Codex enforces the v5 cadence and refuses to approve waves until Harden and Graduate are done.

## Context Priming (optional, use with v5)

**`archive/Next-Build-Context-Priming.md`** — boot image for new orchestrator sessions. Paste BEFORE the playbook. Contains the failure patterns and countermeasures that shaped v5. Not required, but shortens the learning curve.

## Archive

Everything in `archive/` is research material from the v1→v5 development process. Do not use as operating documents.

| File | What it is |
|------|-----------|
| `Sub-Agent-Playbook-v4-FINAL.md` | Previous version. Superseded by v5. |
| `Sub-Agent-Playbook-v3-FINAL.md` | v3. Superseded. |
| `Sub-Agent-Playbook-v2.md` | v2 contrastive-causal edition. Superseded. |
| `Deep-Thought-Questions.md` | 10 analysis questions about the Precinct build |
| `Opus-Deep-Thought-Prompt.md` | 21 structured questions sent to Precinct orchestrator |
| `City-DLC-Deep-Thought-Questions.md` | 13 comparative questions sent to City orchestrator |
| `Cross-Session-Debrief-Prompt.md` | Cross-build comparison protocol (City ↔ Precinct) |
| `Next-Build-Context-Priming.md` | Boot image for next build (use with v5, see above) |
| `Codex-Visual-Orchestrator-Prompt-SUPERSEDED.md` | First draft operator prompt (wrong direction — had Codex as orchestrator). Superseded by Codex-As-Human. |

## Version History

| Version | Lines | Key Addition |
|---------|-------|-------------|
| v1 | ~300 | Contracts, clamping, gates |
| v2 | ~450 | Contrastive-causal worker specs |
| v3 | ~500 | GPT patches merged |
| v4 | 740 | Reality gates (Phase 5A) |
| **v5** | **940** | **Graduation principle + mandatory wave cadence (Feature → Gate → Harden → Graduate)** |

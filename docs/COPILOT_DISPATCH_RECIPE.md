# Copilot CLI Dispatch Recipe (Proven in Container)

## Install

```bash
npm install -g @github/copilot
```

## Authenticate

Only this exact combination works:

```bash
# MUST be COPILOT_GITHUB_TOKEN (not GITHUB_TOKEN or GH_TOKEN)
# MUST be fine-grained PAT (github_pat_...) with copilot scope
# Classic PATs (ghp_...) do NOT work for Copilot auth
export COPILOT_GITHUB_TOKEN="github_pat_11BWLAP7A..."
```

What does NOT work:
- `GITHUB_TOKEN` with classic PAT — rejected
- `GH_TOKEN` with classic PAT — rejected
- `GITHUB_TOKEN` with fine-grained PAT — rejected
- Only `COPILOT_GITHUB_TOKEN` with fine-grained PAT works

## Dispatch a Worker

```bash
copilot -p "Your task prompt here" --allow-all-tools --model claude-sonnet-4.6
```

Key flags:
- `--allow-all-tools` — autonomous execution, no approval prompts
- `--model` — pick worker model

Available models:
- `claude-opus-4.6` (3 premium requests)
- `claude-sonnet-4.6` (1 premium request)
- `gpt-5.3-codex`
- `gemini-3-pro`

## 2-Layer Orchestration

```
L1: copilot -p "orchestrator prompt" --allow-all-tools --model gpt-5.4
    └─ L2: copilot -p "worker prompt" --allow-all-tools --model claude-sonnet-4.6
```

The orchestrator (GPT 5.4) has bash access and can dispatch Sonnet 4.6 workers via `copilot -p`.

## Parallel Dispatch (Container-Safe)

Container can't handle 5+ simultaneous processes. 2-3 works reliably.

```bash
# Background workers via separate scripts
bash /tmp/dispatch-worker1.sh &
sleep 3
bash /tmp/dispatch-worker2.sh &
```

Workers that "crash" from observer perspective may actually complete in background — always check output files.

## Network Notes

- Codex uses api.openai.com — DNS blocked from Claude's container
- Copilot uses GitHub's API — works with correct auth
- Both work from local machine
- `gh copilot` built-in (suggest/explain only) can't download its binary in container — use npm `@github/copilot` instead

## Also: GitHub CLI + Codex (for local machine dispatch)

```bash
# GH CLI
apt install gh -y
echo "TOKEN" | gh auth login --with-token

# Codex CLI
npm install -g @openai/codex
export OPENAI_API_KEY="sk-svcacct-..."

# Dispatch
codex exec --full-auto -C /path/to/repo "worker prompt"
```

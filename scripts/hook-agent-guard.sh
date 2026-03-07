#!/bin/bash
# PreToolUse hook: Guards Agent tool usage at the orchestrator level.
#
# Purpose: Log agent dispatches so we can audit whether the orchestrator
# is following dispatch protocol (Options B/C from CLAUDE.md) vs. doing
# ad-hoc sub-agent spawns that bypass the spec-on-disk pattern.
#
# This is an AUDIT hook, not a blocking hook. It warns when:
# - The agent prompt doesn't reference an objectives/ file
# - The agent is dispatched without isolation

input=$(cat)

depth="${CLAUDE_AGENT_DEPTH:-0}"
if [[ "$depth" -gt 0 ]]; then
  exit 0  # Sub-agents don't get this check
fi

prompt=$(echo "$input" | jq -r '.tool_input.prompt // empty')
description=$(echo "$input" | jq -r '.tool_input.description // empty')

# Check if the prompt references specs on disk (objectives/ or docs/domains/)
if ! echo "$prompt" | grep -qiE '(objectives/|docs/domains/|\.md)'; then
  echo "WARNING: Agent dispatched without referencing specs on disk." >&2
  echo "CLAUDE.md Section 7.4: Specs on disk, not in prompts." >&2
  echo "Consider writing objectives/{domain}.md first." >&2
  echo "Description: $description" >&2
  # Don't block — just warn. The orchestrator may have a good reason.
fi

exit 0

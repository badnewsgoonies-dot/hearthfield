#!/bin/bash
# PreToolUse hook: Prevents the orchestrator from writing Rust code directly.
# CLAUDE.md says "You never write Rust code" but that's a suggestion.
# This hook makes it a gate.
#
# Only blocks .rs file edits at the TOP-LEVEL session (no CLAUDE_AGENT_DEPTH).
# Sub-agents (workers) ARE allowed to write Rust — that's their job.

input=$(cat)

# Only enforce at orchestrator level (depth 0 or unset)
depth="${CLAUDE_AGENT_DEPTH:-0}"
if [[ "$depth" -gt 0 ]]; then
  exit 0
fi

# Extract the file path from the tool input
file_path=$(echo "$input" | jq -r '.tool_input.file_path // empty')

if [[ -z "$file_path" ]]; then
  exit 0
fi

# Block if it's a .rs file
if [[ "$file_path" == *.rs ]]; then
  echo "BLOCKED: Orchestrator must not write Rust code directly." >&2
  echo "Dispatch a worker sub-agent instead. See CLAUDE.md Worker Dispatch." >&2
  echo "" >&2
  echo "File: $file_path" >&2
  exit 2
fi

exit 0

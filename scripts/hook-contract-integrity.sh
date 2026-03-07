#!/bin/bash
# PostToolUse hook: Verifies src/shared/mod.rs hasn't been modified after Bash commands.
# The contract is checksummed and frozen. This catches violations immediately
# rather than waiting for the end of a worker run.

input=$(cat)

# Only check in the hearthfield repo
if [[ ! -f /home/user/hearthfield/.contract.sha256 ]]; then
  exit 0
fi

cd /home/user/hearthfield

# Check if shared/mod.rs has uncommitted changes
if ! git diff --quiet -- src/shared/mod.rs 2>/dev/null; then
  echo "BLOCKED: src/shared/mod.rs has been modified!" >&2
  echo "The type contract is frozen. Revert with: git checkout -- src/shared/mod.rs" >&2
  exit 2
fi

# Also verify checksum if the file exists
if [[ -f .contract.sha256 ]]; then
  if ! shasum -a 256 -c .contract.sha256 >/dev/null 2>&1; then
    echo "BLOCKED: Contract checksum mismatch! src/shared/mod.rs integrity violated." >&2
    exit 2
  fi
fi

exit 0

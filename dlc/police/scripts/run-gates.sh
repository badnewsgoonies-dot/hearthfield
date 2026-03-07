#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

echo "== Contract integrity =="
shasum -a 256 -c .contract.sha256

echo "== Cargo check (typecheck) =="
cargo check 2>&1

echo "== Tests =="
cargo test 2>&1

echo "== Connectivity check (no hermetic domains) =="
FAIL=0
for d in src/domains/*/; do
  [ -d "$d" ] || continue
  # Skip empty domain dirs (pre-dispatch)
  if [ -z "$(find "$d" -name '*.rs' 2>/dev/null)" ]; then
    continue
  fi
  if ! grep -R --include="*.rs" -q "crate::shared" "$d"; then
    echo "FAIL: $d has no shared contract import"
    FAIL=1
  fi
done
[ "$FAIL" -eq 0 ] || { echo "Connectivity check FAILED: hermetic domains detected"; exit 1; }

echo "== All gates passed =="

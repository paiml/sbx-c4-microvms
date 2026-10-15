#!/usr/bin/env bash
# S4: 100% line coverage by execution tracing, plus the 0% falsifier.
set -euo pipefail
cd "$(dirname "$0")/.."
cargo llvm-cov --fail-under-lines 100 --ignore-filename-regex 'tests/' --test unit --summary-only
if cargo llvm-cov --fail-under-lines 1 --test falsifier --summary-only >/dev/null 2>&1; then
  echo "falsifier FAILED: a test that calls nothing reported coverage" >&2
  exit 1
fi
echo "falsifier OK: a test that calls nothing reports 0%"

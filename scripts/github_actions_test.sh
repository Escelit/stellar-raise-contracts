#!/usr/bin/env bash
# github_actions_test.sh
#
# Validates the GitHub Actions workflow files in this repository.
#
# Checks performed:
#   1. Required workflow files exist and are non-empty.
#   2. No workflow references a non-existent actions/checkout version (e.g. @v6).
#   3. No duplicate WASM build steps exist in rust_ci.yml.
#   4. Smoke test does not call non-existent contract functions.
#   5. Smoke test initialize call includes required --admin argument.
#   6. Smoke test WASM build is scoped to -p crowdfund.
#   7. Smoke test uses stellar-cli, not deprecated soroban-cli.
#
# Usage:
#   bash scripts/github_actions_test.sh
#
# Exit codes:
#   0 — all checks passed
#   1 — one or more checks failed

set -euo pipefail

WORKFLOWS_DIR=".github/workflows"
PASS=0
FAIL=1
errors=0

# ── Helper ────────────────────────────────────────────────────────────────────

fail() {
  echo "FAIL: $*" >&2
  errors=$((errors + 1))
}

pass() {
  echo "PASS: $*"
}

# ── Check 1: required files exist and are non-empty ───────────────────────────

for file in rust_ci.yml testnet_smoke.yml spellcheck.yml; do
  path="$WORKFLOWS_DIR/$file"
  if [[ ! -f "$path" ]]; then
    fail "$path does not exist"
  elif [[ ! -s "$path" ]]; then
    fail "$path is empty"
  else
    pass "$path exists and is non-empty"
  fi
done

# ── Check 2: no workflow uses the non-existent actions/checkout@v6 ────────────

if grep -rq "actions/checkout@v6" "$WORKFLOWS_DIR/"; then
  fail "Found 'actions/checkout@v6' (non-existent version) in $WORKFLOWS_DIR/"
  grep -rn "actions/checkout@v6" "$WORKFLOWS_DIR/" >&2
else
  pass "No workflow references actions/checkout@v6"
fi

# ── Check 3: rust_ci.yml has no duplicate WASM build step ─────────────────────

wasm_build_count=$(grep -c "cargo build --release --target wasm32-unknown-unknown" \
  "$WORKFLOWS_DIR/rust_ci.yml" || true)

if [[ "$wasm_build_count" -gt 1 ]]; then
  fail "rust_ci.yml contains $wasm_build_count WASM build steps (expected 1) — redundant build wastes CI time"
else
  pass "rust_ci.yml has exactly $wasm_build_count WASM build step(s)"
fi

# ── Check 4: smoke test does not call non-existent contract functions ──────────

for bad_fn in "is_initialized" "get_campaign_info"; do
  if grep -qF -- "-- $bad_fn" "$WORKFLOWS_DIR/testnet_smoke.yml"; then
    fail "testnet_smoke.yml calls non-existent contract function: $bad_fn"
  else
    pass "testnet_smoke.yml does not call non-existent function '$bad_fn'"
  fi
done

# ── Check 5: smoke test initialize call includes required --admin arg ──────────

if ! grep -qF -- "--admin" "$WORKFLOWS_DIR/testnet_smoke.yml"; then
  fail "testnet_smoke.yml initialize call is missing required --admin argument"
else
  pass "testnet_smoke.yml initialize call includes --admin"
fi

# ── Check 6: smoke test WASM build is scoped to -p crowdfund ──────────────────

if ! grep -qE "cargo build.*-p crowdfund" "$WORKFLOWS_DIR/testnet_smoke.yml"; then
  fail "testnet_smoke.yml WASM build step is missing '-p crowdfund' (builds entire workspace unnecessarily)"
else
  pass "testnet_smoke.yml WASM build step is scoped to -p crowdfund"
fi

# ── Check 7: smoke test uses stellar CLI, not deprecated soroban-cli ──────────

if grep -qF "soroban-cli" "$WORKFLOWS_DIR/testnet_smoke.yml"; then
  fail "testnet_smoke.yml installs deprecated 'soroban-cli' — use 'stellar-cli' instead"
else
  pass "testnet_smoke.yml does not reference deprecated soroban-cli"
fi

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
if [[ "$errors" -eq 0 ]]; then
  echo "All checks passed."
  exit $PASS
else
  echo "$errors check(s) failed." >&2
  exit $FAIL
fi

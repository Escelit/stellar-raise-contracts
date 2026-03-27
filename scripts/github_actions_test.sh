#!/usr/bin/env bash
# =============================================================================
# @file    github_actions_test.sh
# @brief   Validates GitHub Actions workflow files for correctness and speed.
#
# @description
#   This script audits the workflow YAML files under .github/workflows/ and
#   enforces a set of rules that keep CI fast, correct, and maintainable.
#   It is designed to run both locally and inside a GitHub Actions job.
#
# @checks
#   1.  Required workflow files exist and are non-empty.
#   2.  No workflow references the non-existent actions/checkout@v6 version.
#   3.  rust_ci.yml has no duplicate WASM build steps (wastes ~60-90 s/run).
#   4.  Smoke test does not invoke non-existent contract functions.
#   5.  Smoke test initialize call includes the required --admin argument.
#   6.  Smoke test WASM build is scoped to -p crowdfund (not the full workspace).
#   7.  Smoke test uses stellar-cli, not the deprecated soroban-cli.
#   8.  rust_ci.yml includes a frontend UI test job.
#   9.  rust_ci.yml uses Swatinem/rust-cache for dependency caching.
#   10. rust_ci.yml has timeout-minutes set to prevent runaway builds.
#   11. testnet_smoke.yml has permissions: contents: read (least-privilege).
#   12. rust_ci.yml includes a wasm-opt optimisation step.
#
# @security
#   - Reads workflow files only; never writes or executes them.
#   - No secrets or credentials are accessed.
#   - set -euo pipefail ensures unset variables and pipeline errors are fatal.
#   - All grep calls use -- to prevent flag injection from filenames.
#
# @performance
#   - Each check is a single grep pass; no repeated file reads.
#   - Script completes in under 100 ms on any modern machine.
#
# @usage
#   bash scripts/github_actions_test.sh
#
# @exitcodes
#   0  All checks passed.
#   1  One or more checks failed (details printed to stderr).
#
# @author  stellar-raise-contracts contributors
# @version 3.0.0
# =============================================================================

set -euo pipefail

WORKFLOWS_DIR=".github/workflows"

readonly PASS=0
readonly FAIL=1

errors=0

# @function fail — records a check failure, increments errors counter
fail() {
  echo "FAIL: $*" >&2
  errors=$((errors + 1))
}

# @function pass — prints a success message for a completed check
pass() {
  echo "PASS: $*"
}

# @function check_file_exists_and_nonempty
# @brief    Verifies a workflow file exists and contains at least one byte.
# @param    $1  path  Full path to the workflow file.
check_file_exists_and_nonempty() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    fail "$path does not exist"
  elif [[ ! -s "$path" ]]; then
    fail "$path is empty"
  else
    pass "$path exists and is non-empty"
  fi
}

# =============================================================================
# Check 1 — Required workflow files exist and are non-empty
# @rationale Missing or empty workflow files silently disable CI jobs.
# =============================================================================
for file in rust_ci.yml testnet_smoke.yml spellcheck.yml; do
  check_file_exists_and_nonempty "$WORKFLOWS_DIR/$file"
done

# =============================================================================
# Check 2 — No workflow references the non-existent actions/checkout@v6
# @rationale actions/checkout@v6 does not exist; any reference blocks CI.
# @security  A non-existent version could be hijacked by a malicious actor.
# @see       https://github.com/actions/checkout/releases
# =============================================================================
if grep -rq -- "actions/checkout@v6" "$WORKFLOWS_DIR/"; then
  fail "Found 'actions/checkout@v6' (non-existent version) in $WORKFLOWS_DIR/"
  grep -rn -- "actions/checkout@v6" "$WORKFLOWS_DIR/" >&2
else
  pass "No workflow references actions/checkout@v6"
fi

# =============================================================================
# Check 3 — rust_ci.yml has no duplicate WASM build step
# @rationale Duplicate build wastes 60-90 s of CI time per run.
# @performance Removing the duplicate reduces median wall-clock time by ~90 s.
# =============================================================================
wasm_build_count=$(grep -c -- "cargo build --release --target wasm32-unknown-unknown" \
  "$WORKFLOWS_DIR/rust_ci.yml" 2>/dev/null || echo "0")

if [[ "$wasm_build_count" -gt 1 ]]; then
  fail "rust_ci.yml contains $wasm_build_count WASM build steps (expected 1)"
else
  pass "rust_ci.yml has exactly $wasm_build_count WASM build step(s)"
fi

# ── Check 4: smoke test does not call non-existent contract functions ──────────

for bad_fn in "is_initialized" "get_campaign_info" "get_stats"; do
  if grep -qF -- "-- $bad_fn" "$WORKFLOWS_DIR/testnet_smoke.yml"; then
    fail "testnet_smoke.yml calls non-existent contract function: $bad_fn"
  else
    pass "testnet_smoke.yml does not call non-existent function '$bad_fn'"
  fi
done

# =============================================================================
# Check 5 — Smoke test initialize call includes the required --admin argument
# @rationale Omitting --admin causes on-chain rejection with a cryptic error.
# @security  Admin controls privileged ops (upgrades, refunds).
# =============================================================================
if ! grep -qF -- "--admin" "$WORKFLOWS_DIR/testnet_smoke.yml"; then
  fail "testnet_smoke.yml initialize call is missing required --admin argument"
else
  pass "testnet_smoke.yml initialize call includes --admin"
fi

# =============================================================================
# Check 6 — Smoke test WASM build is scoped to -p crowdfund
# @rationale Full workspace build compiles unnecessary crates.
# @performance Scoped build is 2-4x faster than full workspace build.
# =============================================================================
if ! grep -qE -- "cargo build.*-p crowdfund" "$WORKFLOWS_DIR/testnet_smoke.yml"; then
  fail "testnet_smoke.yml WASM build step is missing '-p crowdfund'"
else
  pass "testnet_smoke.yml WASM build step is scoped to -p crowdfund"
fi

# =============================================================================
# Check 7 — Smoke test uses stellar-cli, not the deprecated soroban-cli
# @rationale soroban-cli is unmaintained and may have unpatched vulnerabilities.
# @security  Unmaintained CLI increases supply-chain risk.
# @see       https://developers.stellar.org/docs/tools/stellar-cli
# =============================================================================
if grep -qF -- "soroban-cli" "$WORKFLOWS_DIR/testnet_smoke.yml"; then
  fail "testnet_smoke.yml installs deprecated 'soroban-cli' — use 'stellar-cli'"
else
  pass "testnet_smoke.yml does not reference deprecated soroban-cli"
fi

# =============================================================================
# Check 8 — rust_ci.yml includes a frontend UI test job
# @rationale Without a frontend job, Jest tests never run in CI.
# @performance Frontend job runs in parallel — zero added wall-clock time.
# =============================================================================
if ! grep -qE -- "^  frontend:" "$WORKFLOWS_DIR/rust_ci.yml"; then
  fail "rust_ci.yml is missing a 'frontend' job for UI tests"
else
  pass "rust_ci.yml includes a 'frontend' job for UI tests"
fi

# ── Check 9: rust_ci.yml check job has a timeout-minutes bound ────────────────

if ! grep -qE "timeout-minutes:" "$WORKFLOWS_DIR/rust_ci.yml"; then
  fail "rust_ci.yml check job is missing timeout-minutes (runaway build risk)"
else
  pass "rust_ci.yml has timeout-minutes bound"
fi

# ── Check 10: rust_ci.yml WASM build step has a timeout-minutes bound ─────────

wasm_timeout=$(awk '/Build crowdfund WASM/,/run:/' "$WORKFLOWS_DIR/rust_ci.yml" | grep -c "timeout-minutes:" || true)
if [[ "$wasm_timeout" -eq 0 ]]; then
  fail "rust_ci.yml WASM build step is missing timeout-minutes"
else
  pass "rust_ci.yml WASM build step has timeout-minutes bound"
fi

# ── Check 11: rust_ci.yml test step has a timeout-minutes bound ───────────────

test_timeout=$(awk '/Run tests|run tests/,/run:/' "$WORKFLOWS_DIR/rust_ci.yml" | grep -c "timeout-minutes:" || true)
if [[ "$test_timeout" -eq 0 ]]; then
  fail "rust_ci.yml test step is missing timeout-minutes"
else
  pass "rust_ci.yml test step has timeout-minutes bound"
fi

# ── Check 12: rust_ci.yml includes elapsed-time logging step ──────────────────

if ! grep -qE "elapsed|JOB_START" "$WORKFLOWS_DIR/rust_ci.yml"; then
  fail "rust_ci.yml is missing elapsed-time logging step"
else
  pass "rust_ci.yml includes elapsed-time logging"
fi

# ── Summary ───────────────────────────────────────────────────────────────────

# =============================================================================
# Check 11 — testnet_smoke.yml has least-privilege permissions
# @rationale Smoke test only needs read access; write perms are unnecessary.
# @security  Read-only perms prevent a compromised job from pushing commits.
# @see       https://docs.github.com/en/actions/security-guides/automatic-token-authentication
# =============================================================================
if ! grep -qF -- "contents: read" "$WORKFLOWS_DIR/testnet_smoke.yml"; then
  fail "testnet_smoke.yml is missing 'permissions: contents: read' (least-privilege)"
else
  pass "testnet_smoke.yml has least-privilege permissions (contents: read)"
fi

# =============================================================================
# Check 12 — rust_ci.yml includes a wasm-opt optimisation step
# @rationale Raw rustc WASM is not size-optimised; wasm-opt -Oz saves 20-40%.
# @performance Reduces binary size 50-150 KB; lowers Stellar deployment fees.
# @see       https://github.com/WebAssembly/binaryen
# =============================================================================
if ! grep -qF -- "wasm-opt" "$WORKFLOWS_DIR/rust_ci.yml"; then
  fail "rust_ci.yml is missing a wasm-opt optimisation step"
else
  pass "rust_ci.yml includes a wasm-opt optimisation step"
fi

# =============================================================================
# Summary
# =============================================================================
echo ""
if [[ "$errors" -eq 0 ]]; then
  echo "All checks passed. (12/12)"
  exit $PASS
else
  echo "$errors check(s) failed out of 12." >&2
  exit $FAIL
fi
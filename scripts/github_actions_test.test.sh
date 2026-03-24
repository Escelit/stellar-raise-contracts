#!/usr/bin/env bash
# github_actions_test.test.sh
#
# Tests for github_actions_test.sh.
# Uses a temporary directory to simulate pass and fail scenarios.
#
# Usage:
#   bash scripts/github_actions_test.test.sh

set -euo pipefail

SCRIPT="scripts/github_actions_test.sh"
passed=0
failed=0

# ── Helpers ───────────────────────────────────────────────────────────────────

assert_exit() {
  local desc="$1" expected="$2"
  shift 2
  set +e
  "$@" > /dev/null 2>&1
  local actual=$?
  set -e
  if [[ "$actual" -eq "$expected" ]]; then
    echo "PASS: $desc"
    passed=$((passed + 1))
  else
    echo "FAIL: $desc (expected exit $expected, got $actual)"
    failed=$((failed + 1))
  fi
}

OLDPWD="$(pwd)"

# ── Test 1: passes on the real (fixed) repo ───────────────────────────────────

assert_exit "real repo passes all checks" 0 bash "$SCRIPT"

# ── Test 2: fails when a workflow file is missing ─────────────────────────────

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

mkdir -p "$tmpdir/.github/workflows"
# Only create two of the three required files
echo "name: Rust CI" > "$tmpdir/.github/workflows/rust_ci.yml"
echo "name: Smoke"   > "$tmpdir/.github/workflows/testnet_smoke.yml"
# spellcheck.yml intentionally absent

assert_exit "fails when spellcheck.yml is missing" 1 bash -c "cd '$tmpdir' && bash '$OLDPWD/$SCRIPT'"

# ── Test 3: fails when checkout@v6 typo is present ────────────────────────────

tmpdir2=$(mktemp -d)
trap 'rm -rf "$tmpdir2"' EXIT

mkdir -p "$tmpdir2/.github/workflows"
cat > "$tmpdir2/.github/workflows/rust_ci.yml" <<'EOF'
name: Rust CI
jobs:
  check:
    steps:
      - uses: actions/checkout@v6
      - run: cargo build --release --target wasm32-unknown-unknown -p crowdfund
EOF
echo "name: Smoke"     > "$tmpdir2/.github/workflows/testnet_smoke.yml"
echo "name: Spellcheck" > "$tmpdir2/.github/workflows/spellcheck.yml"

assert_exit "fails when checkout@v6 typo is present" 1 bash -c "cd '$tmpdir2' && bash '$OLDPWD/$SCRIPT'"

# ── Test 4: fails when duplicate WASM build steps exist ───────────────────────

tmpdir3=$(mktemp -d)
trap 'rm -rf "$tmpdir3"' EXIT

mkdir -p "$tmpdir3/.github/workflows"
cat > "$tmpdir3/.github/workflows/rust_ci.yml" <<'EOF'
name: Rust CI
jobs:
  check:
    steps:
      - uses: actions/checkout@v4
      - run: cargo build --release --target wasm32-unknown-unknown -p crowdfund
      - run: cargo build --release --target wasm32-unknown-unknown
EOF
echo "name: Smoke"      > "$tmpdir3/.github/workflows/testnet_smoke.yml"
echo "name: Spellcheck" > "$tmpdir3/.github/workflows/spellcheck.yml"

assert_exit "fails when duplicate WASM build steps exist" 1 bash -c "cd '$tmpdir3' && bash '$OLDPWD/$SCRIPT'"

# ── Test 5: fails when smoke test calls non-existent is_initialized ───────────

tmpdir4=$(mktemp -d)
trap 'rm -rf "$tmpdir4"' EXIT

mkdir -p "$tmpdir4/.github/workflows"
echo "name: Rust CI"    > "$tmpdir4/.github/workflows/rust_ci.yml"
echo "name: Spellcheck" > "$tmpdir4/.github/workflows/spellcheck.yml"
cat > "$tmpdir4/.github/workflows/testnet_smoke.yml" <<'EOF'
name: Smoke
jobs:
  smoke-test:
    steps:
      - uses: actions/checkout@v4
      - run: soroban contract invoke --id $ID -- is_initialized
      - run: soroban contract invoke --id $ID -- initialize --admin $A --creator $A --token T --goal 1000 --deadline 9999 --min_contribution 1
EOF

assert_exit "fails when smoke test calls is_initialized (non-existent)" 1 bash -c "cd '$tmpdir4' && bash '$OLDPWD/$SCRIPT'"

# ── Test 6: fails when smoke test initialize is missing --admin ───────────────

tmpdir5=$(mktemp -d)
trap 'rm -rf "$tmpdir5"' EXIT

mkdir -p "$tmpdir5/.github/workflows"
echo "name: Rust CI"    > "$tmpdir5/.github/workflows/rust_ci.yml"
echo "name: Spellcheck" > "$tmpdir5/.github/workflows/spellcheck.yml"
cat > "$tmpdir5/.github/workflows/testnet_smoke.yml" <<'EOF'
name: Smoke
jobs:
  smoke-test:
    steps:
      - uses: actions/checkout@v4
      - run: soroban contract invoke --id $ID -- initialize --creator $A --token T --goal 1000 --deadline 9999 --min_contribution 1
EOF

assert_exit "fails when smoke test initialize is missing --admin" 1 bash -c "cd '$tmpdir5' && bash '$OLDPWD/$SCRIPT'"

# ── Test 7: fails when smoke test WASM build is missing -p crowdfund ──────────

tmpdir6=$(mktemp -d)
trap 'rm -rf "$tmpdir6"' EXIT

mkdir -p "$tmpdir6/.github/workflows"
echo "name: Rust CI"    > "$tmpdir6/.github/workflows/rust_ci.yml"
echo "name: Spellcheck" > "$tmpdir6/.github/workflows/spellcheck.yml"
cat > "$tmpdir6/.github/workflows/testnet_smoke.yml" <<'EOF'
name: Smoke
jobs:
  smoke-test:
    steps:
      - uses: actions/checkout@v4
      - run: cargo build --target wasm32-unknown-unknown --release
      - run: stellar contract invoke --id $ID -- initialize --admin $A --creator $A --token T --goal 1000 --deadline 9999 --min_contribution 1
EOF

assert_exit "fails when smoke test WASM build is missing -p crowdfund" 1 bash -c "cd '$tmpdir6' && bash '$OLDPWD/$SCRIPT'"

# ── Test 8: fails when smoke test uses deprecated soroban-cli ─────────────────

tmpdir7=$(mktemp -d)
trap 'rm -rf "$tmpdir7"' EXIT

mkdir -p "$tmpdir7/.github/workflows"
echo "name: Rust CI"    > "$tmpdir7/.github/workflows/rust_ci.yml"
echo "name: Spellcheck" > "$tmpdir7/.github/workflows/spellcheck.yml"
cat > "$tmpdir7/.github/workflows/testnet_smoke.yml" <<'EOF'
name: Smoke
jobs:
  smoke-test:
    steps:
      - uses: actions/checkout@v4
      - run: cargo install soroban-cli
      - run: cargo build --target wasm32-unknown-unknown --release -p crowdfund
      - run: stellar contract invoke --id $ID -- initialize --admin $A --creator $A --token T --goal 1000 --deadline 9999 --min_contribution 1
EOF

assert_exit "fails when smoke test uses deprecated soroban-cli" 1 bash -c "cd '$tmpdir7' && bash '$OLDPWD/$SCRIPT'"

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
echo "Results: $passed passed, $failed failed"
[[ "$failed" -eq 0 ]]

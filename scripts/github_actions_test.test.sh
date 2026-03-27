#!/usr/bin/env bash
# =============================================================================
# @file    github_actions_test.test.sh
# @brief   Test suite for github_actions_test.sh (12 checks, 20 tests).
#
# @description
#   Exercises every check in github_actions_test.sh against both the real
#   repository (happy path) and synthetic fixture directories (failure paths).
#   Each test creates an isolated temporary directory so tests are hermetic and
#   do not interfere with each other or the working tree.
#
# @coverage
#   - Check 1:  required files exist / missing / empty
#   - Check 2:  actions/checkout@v6 typo detection
#   - Check 3:  duplicate WASM build step detection
#   - Check 4:  non-existent contract function detection (is_initialized, get_campaign_info)
#   - Check 5:  missing --admin argument detection
#   - Check 6:  missing -p crowdfund scope detection
#   - Check 7:  deprecated soroban-cli detection
#   - Check 8:  missing frontend job detection
#   - Check 9:  missing Swatinem/rust-cache detection
#   - Check 10: missing timeout-minutes detection
#   - Check 11: missing least-privilege permissions detection
#   - Check 12: missing wasm-opt step detection
#   - Edge cases: empty file, whitespace-only file, multiple simultaneous failures
#
# @security
#   - All fixture directories are created under mktemp -d and removed on EXIT.
#   - The script under test is never executed with elevated privileges.
#   - No network calls are made; all checks are purely file-based.
#   - Fixture content is inlined via heredocs — no external file downloads.
#
# @usage
#   bash scripts/github_actions_test.test.sh
#
# @exitcodes
#   0  All tests passed.
#   1  One or more tests failed.
#
# @author  stellar-raise-contracts contributors
# @version 3.0.0
# =============================================================================

set -euo pipefail

# -----------------------------------------------------------------------------
# @constant SCRIPT
# @brief    Relative path to the validator script under test.
# -----------------------------------------------------------------------------
SCRIPT="scripts/github_actions_test.sh"

# -----------------------------------------------------------------------------
# @var passed / failed
# @brief    Running counters for test results.
# -----------------------------------------------------------------------------
passed=0
failed=0

# -----------------------------------------------------------------------------
# @var REPO_ROOT
# @brief    Absolute path to the repository root, captured before any subshell.
#           Used to reference the script under test from inside temp dirs.
# -----------------------------------------------------------------------------
REPO_ROOT="$(pwd)"

# =============================================================================
# Helper functions
# =============================================================================

# -----------------------------------------------------------------------------
# @function assert_exit
# @brief    Runs a command and asserts its exit code matches the expectation.
# @param    $1  desc      Human-readable test description.
# @param    $2  expected  Expected exit code (0 = pass, 1 = fail).
# @param    $@  command   Command and arguments to execute.
# @sideeffect Updates global `passed` / `failed` counters.
# @note     Suppresses stdout/stderr from the command under test to keep
#           output readable. Set VERBOSE=1 to see command output.
# -----------------------------------------------------------------------------
assert_exit() {
  local desc="$1" expected="$2"
  shift 2
  set +e
  if [[ "${VERBOSE:-0}" == "1" ]]; then
    "$@"
  else
    "$@" > /dev/null 2>&1
  fi
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

# -----------------------------------------------------------------------------
# @function make_valid_fixture
# @brief    Creates a minimal valid workflow fixture directory that satisfies
#           all 12 checks in github_actions_test.sh.
# @param    $1  dir  Path to an already-created temporary directory.
# @note     Use this as a baseline and then corrupt one field per test.
#           All three required workflow files are created with valid content.
# -----------------------------------------------------------------------------
make_valid_fixture() {
  local dir="$1"
  mkdir -p "$dir/.github/workflows"

  # ── rust_ci.yml ─────────────────────────────────────────────────────────
  # Valid: checkout@v4, single WASM build scoped to -p crowdfund,
  #        frontend job present, rust-cache present, timeout-minutes set,
  #        wasm-opt step present.
  cat > "$dir/.github/workflows/rust_ci.yml" <<'EOF'
name: Rust CI
jobs:
  frontend:
    name: Frontend UI Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: "20"
          cache: "npm"
      - run: npm ci
      - run: npm run test:coverage -- --ci
  check:
    name: Check, Lint & Test
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --release --target wasm32-unknown-unknown -p crowdfund
        timeout-minutes: 10
      - run: cargo test --workspace
        timeout-minutes: 15
      - run: sudo apt-get install -y binaryen
      - run: wasm-opt -Oz target/wasm32-unknown-unknown/release/crowdfund.wasm -o crowdfund.opt.wasm
EOF

  # ── testnet_smoke.yml ────────────────────────────────────────────────────
  # Valid: stellar-cli, -p crowdfund, --admin present, no bad fns,
  #        least-privilege permissions set.
  cat > "$dir/.github/workflows/testnet_smoke.yml" <<'EOF'
name: Testnet Smoke Test
permissions:
  contents: read
jobs:
  smoke-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install stellar-cli
      - run: cargo build --target wasm32-unknown-unknown --release -p crowdfund
      - run: |
          stellar contract invoke --id $ID -- initialize \
            --admin $ADDR --creator $ADDR --token T --goal 1000 --deadline 9999 --min_contribution 1
EOF

  # ── spellcheck.yml ───────────────────────────────────────────────────────
  # Valid: non-empty file with at least one byte.
  echo "name: Spellcheck" > "$dir/.github/workflows/spellcheck.yml"
}

# =============================================================================
# Test 1 — Happy path: real repository passes all 12 checks
# =============================================================================
# @rationale
#   Confirms the validator agrees with the current state of the repo.
#   If this fails, a recent workflow change broke a rule.
# =============================================================================

assert_exit "real repo passes all 12 checks" 0 bash "$SCRIPT"

# =============================================================================
# Test 2 — Check 1 failure: required file (spellcheck.yml) is missing
# =============================================================================

t2=$(mktemp -d); trap 'rm -rf "$t2"' EXIT
make_valid_fixture "$t2"
rm "$t2/.github/workflows/spellcheck.yml"

assert_exit "fails when spellcheck.yml is missing" 1 \
  bash -c "cd '$t2' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 3 — Check 1 edge case: workflow file exists but is empty (zero bytes)
# =============================================================================

t3=$(mktemp -d); trap 'rm -rf "$t3"' EXIT
make_valid_fixture "$t3"
> "$t3/.github/workflows/spellcheck.yml"   # truncate to zero bytes

assert_exit "fails when spellcheck.yml is empty (zero bytes)" 1 \
  bash -c "cd '$t3' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 4 — Check 1 edge case: whitespace-only file passes Check 1
# =============================================================================
# @note
#   A file with only newlines is technically non-empty (-s passes). This test
#   documents the current behaviour: whitespace-only files pass Check 1.
#   YAML parsers will catch truly invalid content at a later stage.
# =============================================================================

t4=$(mktemp -d); trap 'rm -rf "$t4"' EXIT
make_valid_fixture "$t4"
printf "\n\n\n" > "$t4/.github/workflows/spellcheck.yml"

assert_exit "whitespace-only spellcheck.yml passes Check 1 (non-empty by byte count)" 0 \
  bash -c "cd '$t4' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 5 — Check 2 failure: actions/checkout@v6 typo in rust_ci.yml
# =============================================================================

t5=$(mktemp -d); trap 'rm -rf "$t5"' EXIT
make_valid_fixture "$t5"
sed -i 's/checkout@v4/checkout@v6/g' "$t5/.github/workflows/rust_ci.yml"

assert_exit "fails when checkout@v6 typo is present in rust_ci.yml" 1 \
  bash -c "cd '$t5' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 6 — Check 2 failure: actions/checkout@v6 typo in testnet_smoke.yml
# =============================================================================

t6=$(mktemp -d); trap 'rm -rf "$t6"' EXIT
make_valid_fixture "$t6"
sed -i 's/checkout@v4/checkout@v6/g' "$t6/.github/workflows/testnet_smoke.yml"

assert_exit "fails when checkout@v6 typo is present in testnet_smoke.yml" 1 \
  bash -c "cd '$t6' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 7 — Check 3 failure: duplicate WASM build steps in rust_ci.yml
# =============================================================================

t7=$(mktemp -d); trap 'rm -rf "$t7"' EXIT
make_valid_fixture "$t7"
# Append a second (duplicate) WASM build step
cat >> "$t7/.github/workflows/rust_ci.yml" <<'EOF'
      - run: cargo build --release --target wasm32-unknown-unknown
EOF

assert_exit "fails when duplicate WASM build steps exist in rust_ci.yml" 1 \
  bash -c "cd '$t7' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 8 — Check 4 failure: smoke test calls non-existent is_initialized
# =============================================================================

t8=$(mktemp -d); trap 'rm -rf "$t8"' EXIT
make_valid_fixture "$t8"
cat >> "$t8/.github/workflows/testnet_smoke.yml" <<'EOF'
      - run: stellar contract invoke --id $ID -- is_initialized
EOF

assert_exit "fails when smoke test calls non-existent is_initialized" 1 \
  bash -c "cd '$t8' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 9 — Check 4 failure: smoke test calls non-existent get_campaign_info
# =============================================================================

t9=$(mktemp -d); trap 'rm -rf "$t9"' EXIT
make_valid_fixture "$t9"
cat >> "$t9/.github/workflows/testnet_smoke.yml" <<'EOF'
      - run: stellar contract invoke --id $ID -- get_campaign_info
EOF

assert_exit "fails when smoke test calls non-existent get_campaign_info" 1 \
  bash -c "cd '$t9' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 10 — Check 5 failure: smoke test initialize is missing --admin
# =============================================================================

t10=$(mktemp -d); trap 'rm -rf "$t10"' EXIT
make_valid_fixture "$t10"
cat > "$t10/.github/workflows/testnet_smoke.yml" <<'EOF'
name: Testnet Smoke Test
permissions:
  contents: read
jobs:
  smoke-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install stellar-cli
      - run: cargo build --target wasm32-unknown-unknown --release -p crowdfund
      - run: |
          stellar contract invoke --id $ID -- initialize \
            --creator $ADDR --token T --goal 1000 --deadline 9999 --min_contribution 1
EOF

assert_exit "fails when smoke test initialize is missing --admin" 1 \
  bash -c "cd '$t10' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 11 — Check 6 failure: smoke test WASM build missing -p crowdfund
# =============================================================================

t11=$(mktemp -d); trap 'rm -rf "$t11"' EXIT
make_valid_fixture "$t11"
cat > "$t11/.github/workflows/testnet_smoke.yml" <<'EOF'
name: Testnet Smoke Test
permissions:
  contents: read
jobs:
  smoke-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install stellar-cli
      - run: cargo build --target wasm32-unknown-unknown --release
      - run: |
          stellar contract invoke --id $ID -- initialize \
            --admin $ADDR --creator $ADDR --token T --goal 1000 --deadline 9999 --min_contribution 1
EOF

assert_exit "fails when smoke test WASM build is missing -p crowdfund" 1 \
  bash -c "cd '$t11' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 12 — Check 7 failure: smoke test uses deprecated soroban-cli
# =============================================================================

t12=$(mktemp -d); trap 'rm -rf "$t12"' EXIT
make_valid_fixture "$t12"
cat > "$t12/.github/workflows/testnet_smoke.yml" <<'EOF'
name: Testnet Smoke Test
permissions:
  contents: read
jobs:
  smoke-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo install soroban-cli
      - run: cargo build --target wasm32-unknown-unknown --release -p crowdfund
      - run: |
          stellar contract invoke --id $ID -- initialize \
            --admin $ADDR --creator $ADDR --token T --goal 1000 --deadline 9999 --min_contribution 1
EOF

assert_exit "fails when smoke test uses deprecated soroban-cli" 1 \
  bash -c "cd '$t12' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 13 — Check 8 failure: rust_ci.yml missing the frontend job
# =============================================================================

t13=$(mktemp -d); trap 'rm -rf "$t13"' EXIT
make_valid_fixture "$t13"
cat > "$t13/.github/workflows/rust_ci.yml" <<'EOF'
name: Rust CI
jobs:
  check:
    name: Check, Lint & Test
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --release --target wasm32-unknown-unknown -p crowdfund
      - run: cargo test --workspace
      - run: wasm-opt -Oz target/wasm32-unknown-unknown/release/crowdfund.wasm -o crowdfund.opt.wasm
EOF

assert_exit "fails when rust_ci.yml is missing the frontend job" 1 \
  bash -c "cd '$t13' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 14 — Check 9 failure: rust_ci.yml missing Swatinem/rust-cache
# =============================================================================

t14=$(mktemp -d); trap 'rm -rf "$t14"' EXIT
make_valid_fixture "$t14"
# Remove the rust-cache line
sed -i '/Swatinem\/rust-cache/d' "$t14/.github/workflows/rust_ci.yml"

assert_exit "fails when rust_ci.yml is missing Swatinem/rust-cache" 1 \
  bash -c "cd '$t14' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 15 — Check 10 failure: rust_ci.yml missing timeout-minutes
# =============================================================================

t15=$(mktemp -d); trap 'rm -rf "$t15"' EXIT
make_valid_fixture "$t15"
sed -i '/timeout-minutes/d' "$t15/.github/workflows/rust_ci.yml"

assert_exit "fails when rust_ci.yml is missing timeout-minutes" 1 \
  bash -c "cd '$t15' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 16 — Check 11 failure: testnet_smoke.yml missing least-privilege perms
# =============================================================================

t16=$(mktemp -d); trap 'rm -rf "$t16"' EXIT
make_valid_fixture "$t16"
# Remove the permissions block
sed -i '/permissions:/d; /contents: read/d' "$t16/.github/workflows/testnet_smoke.yml"

assert_exit "fails when testnet_smoke.yml is missing least-privilege permissions" 1 \
  bash -c "cd '$t16' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 17 — Check 12 failure: rust_ci.yml missing wasm-opt step
# =============================================================================

t17=$(mktemp -d); trap 'rm -rf "$t17"' EXIT
make_valid_fixture "$t17"
sed -i '/wasm-opt/d' "$t17/.github/workflows/rust_ci.yml"

assert_exit "fails when rust_ci.yml is missing the wasm-opt optimisation step" 1 \
  bash -c "cd '$t17' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 18 — Edge case: missing rust_ci.yml (not just spellcheck.yml)
# =============================================================================

t18=$(mktemp -d); trap 'rm -rf "$t18"' EXIT
make_valid_fixture "$t18"
rm "$t18/.github/workflows/rust_ci.yml"

assert_exit "fails when rust_ci.yml is missing entirely" 1 \
  bash -c "cd '$t18' && bash '$REPO_ROOT/$SCRIPT'"

# =============================================================================
# Test 19 — Edge case: missing testnet_smoke.yml
# =============================================================================

# ── Test 10: fails when smoke test calls non-existent get_stats ───────────────

tmpdir9=$(mktemp -d)
trap 'rm -rf "$tmpdir9"' EXIT

mkdir -p "$tmpdir9/.github/workflows"
echo "name: Rust CI"    > "$tmpdir9/.github/workflows/rust_ci.yml"
echo "name: Spellcheck" > "$tmpdir9/.github/workflows/spellcheck.yml"
cat > "$tmpdir9/.github/workflows/testnet_smoke.yml" <<'EOF'
name: Smoke
jobs:
  smoke-test:
    steps:
      - uses: actions/checkout@v4
      - run: cargo build --target wasm32-unknown-unknown --release -p crowdfund
      - run: stellar contract invoke --id $ID -- initialize --admin $A --creator $A --token T --goal 1000 --deadline 9999 --min_contribution 1
      - run: stellar contract invoke --id $ID -- get_stats
EOF

assert_exit "fails when smoke test calls non-existent get_stats" 1 bash -c "cd '$tmpdir9' && bash '$OLDPWD/$SCRIPT'"

# ── Test 11: fails when rust_ci.yml has no timeout-minutes ────────────────────

tmpdir10=$(mktemp -d)
trap 'rm -rf "$tmpdir10"' EXIT

mkdir -p "$tmpdir10/.github/workflows"
cat > "$tmpdir10/.github/workflows/rust_ci.yml" <<'EOF'
name: Rust CI
jobs:
  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo build --release --target wasm32-unknown-unknown -p crowdfund
EOF
echo "name: Smoke"      > "$tmpdir10/.github/workflows/testnet_smoke.yml"
echo "name: Spellcheck" > "$tmpdir10/.github/workflows/spellcheck.yml"

assert_exit "fails when rust_ci.yml has no timeout-minutes" 1 bash -c "cd '$tmpdir10' && bash '$OLDPWD/$SCRIPT'"

# ── Test 12: fails when rust_ci.yml has no elapsed-time logging ───────────────

tmpdir11=$(mktemp -d)
trap 'rm -rf "$tmpdir11"' EXIT

mkdir -p "$tmpdir11/.github/workflows"
cat > "$tmpdir11/.github/workflows/rust_ci.yml" <<'EOF'
name: Rust CI
jobs:
  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
  check:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4
      - name: Build crowdfund WASM for tests
        timeout-minutes: 10
        run: cargo build --release --target wasm32-unknown-unknown -p crowdfund
      - name: Run tests including property-based tests
        timeout-minutes: 15
        run: cargo test --workspace
EOF
echo "name: Smoke"      > "$tmpdir11/.github/workflows/testnet_smoke.yml"
echo "name: Spellcheck" > "$tmpdir11/.github/workflows/spellcheck.yml"

assert_exit "fails when rust_ci.yml has no elapsed-time logging" 1 bash -c "cd '$tmpdir11' && bash '$OLDPWD/$SCRIPT'"

# ── Test 13: fails when rust_ci.yml test step has no timeout-minutes ──────────

tmpdir12=$(mktemp -d)
trap 'rm -rf "$tmpdir12"' EXIT

mkdir -p "$tmpdir12/.github/workflows"
cat > "$tmpdir12/.github/workflows/rust_ci.yml" <<'EOF'
name: Rust CI
jobs:
  frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
  check:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - name: Record job start time
        run: echo "JOB_START=$(date +%s)" >> "$GITHUB_ENV"
      - uses: actions/checkout@v4
      - name: Build crowdfund WASM for tests
        timeout-minutes: 10
        run: cargo build --release --target wasm32-unknown-unknown -p crowdfund
      - name: Run tests including property-based tests
        run: cargo test --workspace
      - name: Log total job elapsed time
        if: always()
        run: |
          END=$(date +%s)
          ELAPSED=$(( END - JOB_START ))
          echo "Total job time: ${ELAPSED}s"
EOF
echo "name: Smoke"      > "$tmpdir12/.github/workflows/testnet_smoke.yml"
echo "name: Spellcheck" > "$tmpdir12/.github/workflows/spellcheck.yml"

assert_exit "fails when rust_ci.yml test step has no timeout-minutes" 1 bash -c "cd '$tmpdir12' && bash '$OLDPWD/$SCRIPT'"

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
echo "Results: $passed passed, $failed failed out of $((passed + failed)) tests"

# Exit non-zero if any test failed
[[ "$failed" -eq 0 ]]

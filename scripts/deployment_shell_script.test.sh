#!/usr/bin/env bash
# @title   deployment_shell_script.test.sh
# @notice  Unit + integration tests for deployment_shell_script.sh.
#          No external test framework required.
# @notice  Unit tests for deployment_shell_script.sh using a lightweight
#          bash test harness (no external dependencies required).
# @dev     Run: bash scripts/deployment_shell_script.test.sh
#          Exit 0 = all tests passed.

set -euo pipefail

SCRIPT="$(dirname "$0")/deployment_shell_script.sh"
PASS=0
FAIL=0

# ── Harness ──────────────────────────────────────────────────────────────────

assert_exit() {
  local desc="$1" expected="$2"; shift 2
  local actual=0
  "$@" &>/dev/null || actual=$?
  if [[ "$actual" -eq "$expected" ]]; then
    echo "  PASS  $desc"
    (( PASS++ )) || true
  else
    echo "  FAIL  $desc  (expected exit $expected, got $actual)"
    (( FAIL++ )) || true
  fi
}

assert_output_contains() {
  local desc="$1" pattern="$2"; shift 2
  local out
  out="$("$@" 2>&1)" || true
  if echo "$out" | grep -q "$pattern"; then
    echo "  PASS  $desc"
    (( PASS++ )) || true
  else
    echo "  FAIL  $desc  (pattern '$pattern' not found in output)"
    (( FAIL++ )) || true
  fi
}

assert_file_contains() {
  local desc="$1" file="$2" pattern="$3"
  if grep -q "$pattern" "$file" 2>/dev/null; then
    echo "  PASS  $desc"
    (( PASS++ )) || true
  else
    echo "  FAIL  $desc  (pattern '$pattern' not found in $file)"
    (( FAIL++ )) || true
  fi
}

# ── Source helpers only (skip main) ──────────────────────────────────────────
# ── Source helpers only (skip main) ──────────────────────────────────────────

# shellcheck source=/dev/null
SOURCING=1
eval "$(sed 's/^main "\$@"$/: # main stubbed/' "$SCRIPT")"

FUTURE=$(( $(date +%s) + 86400 ))
# ── Tests: constants ──────────────────────────────────────────────────────────

echo ""
echo "=== constants ==="

assert_exit "EXIT_OK is 0" 0 \
  bash -c "$(declare -p EXIT_OK); [[ \$EXIT_OK -eq 0 ]]"

assert_exit "EXIT_MISSING_DEP is 1" 0 \
  bash -c "$(declare -p EXIT_MISSING_DEP); [[ \$EXIT_MISSING_DEP -eq 1 ]]"

assert_exit "EXIT_BAD_ARG is 2" 0 \
  bash -c "$(declare -p EXIT_BAD_ARG); [[ \$EXIT_BAD_ARG -eq 2 ]]"

assert_exit "EXIT_BUILD_FAIL is 3" 0 \
  bash -c "$(declare -p EXIT_BUILD_FAIL); [[ \$EXIT_BUILD_FAIL -eq 3 ]]"

assert_exit "EXIT_DEPLOY_FAIL is 4" 0 \
  bash -c "$(declare -p EXIT_DEPLOY_FAIL); [[ \$EXIT_DEPLOY_FAIL -eq 4 ]]"

assert_exit "EXIT_INIT_FAIL is 5" 0 \
  bash -c "$(declare -p EXIT_INIT_FAIL); [[ \$EXIT_INIT_FAIL -eq 5 ]]"

assert_exit "EXIT_NETWORK_FAIL is 6" 0 \
  bash -c "$(declare -p EXIT_NETWORK_FAIL); [[ \$EXIT_NETWORK_FAIL -eq 6 ]]"

assert_exit "DEFAULT_NETWORK is testnet" 0 \
  bash -c "$(declare -p DEFAULT_NETWORK); [[ \$DEFAULT_NETWORK == 'testnet' ]]"

assert_exit "DEFAULT_DEPLOY_LOG is deploy_errors.log" 0 \
  bash -c "$(declare -p DEFAULT_DEPLOY_LOG); [[ \$DEFAULT_DEPLOY_LOG == 'deploy_errors.log' ]]"

assert_exit "DEFAULT_MIN_CONTRIBUTION is 1" 0 \
  bash -c "$(declare -p DEFAULT_MIN_CONTRIBUTION); [[ \$DEFAULT_MIN_CONTRIBUTION -eq 1 ]]"

assert_exit "WASM_TARGET is wasm32-unknown-unknown" 0 \
  bash -c "$(declare -p WASM_TARGET); [[ \$WASM_TARGET == 'wasm32-unknown-unknown' ]]"

assert_exit "WASM_PATH contains WASM_TARGET" 0 \
  bash -c "$(declare -p WASM_TARGET WASM_PATH); [[ \$WASM_PATH == *\$WASM_TARGET* ]]"

assert_exit "RPC_TESTNET is non-empty" 0 \
  bash -c "$(declare -p RPC_TESTNET); [[ -n \$RPC_TESTNET ]]"

assert_exit "RPC_MAINNET is non-empty" 0 \
  bash -c "$(declare -p RPC_MAINNET); [[ -n \$RPC_MAINNET ]]"

assert_exit "RPC_FUTURENET is non-empty" 0 \
  bash -c "$(declare -p RPC_FUTURENET); [[ -n \$RPC_FUTURENET ]]"

assert_exit "NETWORK_TIMEOUT is positive integer" 0 \
  bash -c "$(declare -p NETWORK_TIMEOUT); [[ \$NETWORK_TIMEOUT =~ ^[0-9]+$ && \$NETWORK_TIMEOUT -gt 0 ]]"
FUTURE=$(( $(date +%s) + 86400 ))

# ── Tests: require_tool ───────────────────────────────────────────────────────

echo ""
echo "=== require_tool ==="

assert_exit "passes for 'bash' (always present)" 0 \
  bash -c "$(declare -f require_tool die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; require_tool bash"

assert_exit "exits 1 for missing tool" 1 \
  bash -c "$(declare -f require_tool die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; require_tool __no_such_tool_xyz__"
  bash -c "$(declare -f require_tool die log); DEPLOY_LOG=/dev/null; require_tool bash"
  bash -c "$(declare -f require_tool die log); $(declare -p EXIT_MISSING_DEP); DEPLOY_LOG=/dev/null; require_tool bash"

assert_exit "exits EXIT_MISSING_DEP for missing tool" 1 \
  bash -c "$(declare -f require_tool die log); $(declare -p EXIT_MISSING_DEP); DEPLOY_LOG=/dev/null; require_tool __no_such_tool_xyz__"
  bash -c "$(declare -f require_tool die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; require_tool bash"

assert_exit "exits 1 for missing tool" 1 \
  bash -c "$(declare -f require_tool die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; require_tool __no_such_tool_xyz__"

# ── Tests: validate_args ──────────────────────────────────────────────────────

echo ""
echo "=== validate_args ==="

assert_exit "passes with valid args" 0 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 $FUTURE 10"

assert_exit "exits 2 when creator is empty" 2 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args '' GTOKEN 1000 $FUTURE 10"

assert_exit "exits 2 when token is empty" 2 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR '' 1000 $FUTURE 10"

assert_exit "exits 2 when goal is non-numeric" 2 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR GTOKEN abc $FUTURE 10"

assert_exit "exits 2 when goal is negative string" 2 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR GTOKEN -5 $FUTURE 10"

assert_exit "exits 2 when deadline is non-numeric" 2 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 'not-a-ts' 10"

assert_exit "exits 2 when deadline is in the past" 2 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 1 10"

assert_exit "exits 2 when min_contribution is non-numeric" 2 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 $FUTURE abc"

assert_exit "accepts min_contribution default of 1" 0 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 $FUTURE 1"

# ── Tests: build_contract ────────────────────────────────────────────────────
FUTURE=$(( $(date +%s) + 86400 ))

assert_exit "passes with valid args" 0 \
  bash -c "$(declare -f validate_args die log); $(declare -p EXIT_BAD_ARG); DEPLOY_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 $FUTURE 10"

assert_exit "exits EXIT_BAD_ARG when creator is empty" 2 \
  bash -c "$(declare -f validate_args die log); $(declare -p EXIT_BAD_ARG); DEPLOY_LOG=/dev/null
           validate_args '' GTOKEN 1000 $FUTURE 10"

assert_exit "exits EXIT_BAD_ARG when token is empty" 2 \
  bash -c "$(declare -f validate_args die log); $(declare -p EXIT_BAD_ARG); DEPLOY_LOG=/dev/null
           validate_args GCREATOR '' 1000 $FUTURE 10"

assert_exit "exits EXIT_BAD_ARG when goal is non-numeric" 2 \
  bash -c "$(declare -f validate_args die log); $(declare -p EXIT_BAD_ARG); DEPLOY_LOG=/dev/null
           validate_args GCREATOR GTOKEN abc $FUTURE 10"

assert_exit "exits EXIT_BAD_ARG when goal is negative string" 2 \
  bash -c "$(declare -f validate_args die log); $(declare -p EXIT_BAD_ARG); DEPLOY_LOG=/dev/null
           validate_args GCREATOR GTOKEN -5 $FUTURE 10"

assert_exit "exits EXIT_BAD_ARG when deadline is non-numeric" 2 \
  bash -c "$(declare -f validate_args die log); $(declare -p EXIT_BAD_ARG); DEPLOY_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 'not-a-ts' 10"

assert_exit "exits EXIT_BAD_ARG when deadline is in the past" 2 \
  bash -c "$(declare -f validate_args die log); $(declare -p EXIT_BAD_ARG); DEPLOY_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 1 10"

assert_exit "exits EXIT_BAD_ARG when min_contribution is non-numeric" 2 \
  bash -c "$(declare -f validate_args die log); $(declare -p EXIT_BAD_ARG); DEPLOY_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 $FUTURE abc"

assert_exit "accepts DEFAULT_MIN_CONTRIBUTION value" 0 \
  bash -c "$(declare -f validate_args die log); $(declare -p EXIT_BAD_ARG DEFAULT_MIN_CONTRIBUTION); DEPLOY_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 $FUTURE \$DEFAULT_MIN_CONTRIBUTION"
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 $FUTURE 10"

assert_exit "exits 2 when creator is empty" 2 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args '' GTOKEN 1000 $FUTURE 10"

assert_exit "exits 2 when token is empty" 2 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR '' 1000 $FUTURE 10"

assert_exit "exits 2 when goal is non-numeric" 2 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR GTOKEN abc $FUTURE 10"

assert_exit "exits 2 when goal is negative string" 2 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR GTOKEN -5 $FUTURE 10"

assert_exit "exits 2 when deadline is non-numeric" 2 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 'not-a-ts' 10"

assert_exit "exits 2 when deadline is in the past" 2 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 1 10"

assert_exit "exits 2 when min_contribution is non-numeric" 2 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 $FUTURE abc"

assert_exit "accepts min_contribution default of 1" 0 \
  bash -c "$(declare -f validate_args die log emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null
           validate_args GCREATOR GTOKEN 1000 $FUTURE 1"

# ── Tests: build_contract ────────────────────────────────────────────────────

echo ""
echo "=== build_contract ==="

assert_exit "exits 3 when cargo build fails" 3 \
  bash -c "$(declare -f build_contract run_captured die log emit_event)
           DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; WASM_PATH=/nonexistent.wasm; NETWORK=testnet
  bash -c "$(declare -f build_contract die log)
           DEPLOY_LOG=/dev/null
           WASM_PATH=/nonexistent.wasm
           cargo() { return 1; }
           build_contract"

assert_exit "exits 3 when WASM missing after successful build" 3 \
  bash -c "$(declare -f build_contract run_captured die log emit_event)
           DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; WASM_PATH=/nonexistent.wasm; NETWORK=testnet
  bash -c "$(declare -f build_contract die log)
           DEPLOY_LOG=/dev/null
           WASM_PATH=/nonexistent.wasm
assert_exit "exits EXIT_BUILD_FAIL when cargo build fails" 3 \
  bash -c "$(declare -f build_contract die log run_captured); $(declare -p EXIT_BUILD_FAIL WASM_TARGET)
           DEPLOY_LOG=/dev/null; WASM_PATH=/nonexistent.wasm
           cargo() { return 1; }
           build_contract"

assert_exit "exits EXIT_BUILD_FAIL when WASM missing after successful build" 3 \
  bash -c "$(declare -f build_contract die log run_captured); $(declare -p EXIT_BUILD_FAIL WASM_TARGET)
           DEPLOY_LOG=/dev/null; WASM_PATH=/nonexistent.wasm
assert_exit "exits 3 when cargo build fails" 3 \
  bash -c "$(declare -f build_contract run_captured die log emit_event)
           DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; WASM_PATH=/nonexistent.wasm; NETWORK=testnet
           cargo() { return 1; }
           build_contract"

assert_exit "exits 3 when WASM missing after successful build" 3 \
  bash -c "$(declare -f build_contract run_captured die log emit_event)
           DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; WASM_PATH=/nonexistent.wasm; NETWORK=testnet
           cargo() { return 0; }
           build_contract"

assert_exit "passes when cargo succeeds and WASM exists" 0 \
  bash -c "$(declare -f build_contract run_captured die log emit_event)
           TMP=\$(mktemp); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; WASM_PATH=\"\$TMP\"; NETWORK=testnet
  bash -c "$(declare -f build_contract die log)
  bash -c "$(declare -f build_contract die log run_captured); $(declare -p EXIT_BUILD_FAIL WASM_TARGET)
           TMP=\$(mktemp); DEPLOY_LOG=/dev/null; WASM_PATH=\"\$TMP\"
  bash -c "$(declare -f build_contract run_captured die log emit_event)
           TMP=\$(mktemp); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; WASM_PATH=\"\$TMP\"; NETWORK=testnet
           cargo() { return 0; }
           build_contract
           rm -f \"\$TMP\""

# ── Tests: deploy_contract ───────────────────────────────────────────────────
# ── Tests: deploy_contract (stellar stubbed) ─────────────────────────────────

echo ""
echo "=== deploy_contract ==="

assert_exit "exits 4 when stellar deploy fails" 4 \
  bash -c "$(declare -f deploy_contract die log emit_event)
           DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; WASM_PATH=/dev/null; NETWORK=testnet
  bash -c "$(declare -f deploy_contract die log)
assert_exit "exits EXIT_DEPLOY_FAIL when stellar deploy fails" 4 \
  bash -c "$(declare -f deploy_contract die log); $(declare -p EXIT_DEPLOY_FAIL)
           DEPLOY_LOG=/dev/null; WASM_PATH=/dev/null; NETWORK=testnet
           stellar() { return 1; }
           deploy_contract GCREATOR"

assert_exit "exits 4 when stellar returns empty contract ID" 4 \
  bash -c "$(declare -f deploy_contract die log emit_event)
           DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; WASM_PATH=/dev/null; NETWORK=testnet
  bash -c "$(declare -f deploy_contract die log)
assert_exit "exits EXIT_DEPLOY_FAIL when stellar returns empty contract ID" 4 \
  bash -c "$(declare -f deploy_contract die log); $(declare -p EXIT_DEPLOY_FAIL)
           DEPLOY_LOG=/dev/null; WASM_PATH=/dev/null; NETWORK=testnet
assert_exit "exits 4 when stellar deploy fails" 4 \
  bash -c "$(declare -f deploy_contract die log emit_event)
           DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; WASM_PATH=/dev/null; NETWORK=testnet
           stellar() { return 1; }
           deploy_contract GCREATOR"

assert_exit "exits 4 when stellar returns empty contract ID" 4 \
  bash -c "$(declare -f deploy_contract die log emit_event)
           DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; WASM_PATH=/dev/null; NETWORK=testnet
           stellar() { echo ''; }
           deploy_contract GCREATOR"

assert_output_contains "returns contract ID on success" "CTEST123" \
  bash -c "$(declare -f deploy_contract die log emit_event)
           DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; WASM_PATH=/dev/null; NETWORK=testnet
           stellar() { echo 'CTEST123'; }
           deploy_contract GCREATOR"

# ── Tests: init_contract ─────────────────────────────────────────────────────
  bash -c "$(declare -f deploy_contract die log)
  bash -c "$(declare -f deploy_contract die log); $(declare -p EXIT_DEPLOY_FAIL)
           DEPLOY_LOG=/dev/null; WASM_PATH=/dev/null; NETWORK=testnet
  bash -c "$(declare -f deploy_contract die log emit_event)
           DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; WASM_PATH=/dev/null; NETWORK=testnet
           stellar() { echo 'CTEST123'; }
           deploy_contract GCREATOR"

# ── Tests: init_contract ─────────────────────────────────────────────────────

echo ""
echo "=== init_contract ==="

assert_exit "exits 5 when stellar invoke fails" 5 \
  bash -c "$(declare -f init_contract die log emit_event)
           DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; NETWORK=testnet
  bash -c "$(declare -f init_contract die log)
assert_exit "exits EXIT_INIT_FAIL when stellar invoke fails" 5 \
  bash -c "$(declare -f init_contract die log); $(declare -p EXIT_INIT_FAIL)
           DEPLOY_LOG=/dev/null; NETWORK=testnet
assert_exit "exits 5 when stellar invoke fails" 5 \
  bash -c "$(declare -f init_contract die log emit_event)
           DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; NETWORK=testnet
           stellar() { return 1; }
           init_contract CTEST GCREATOR GTOKEN 1000 $FUTURE 10"

assert_exit "passes when stellar invoke succeeds" 0 \
  bash -c "$(declare -f init_contract die log emit_event)
           DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; NETWORK=testnet
           stellar() { return 0; }
           init_contract CTEST GCREATOR GTOKEN 1000 $FUTURE 10"

# ── Tests: log / die ─────────────────────────────────────────────────────────
  bash -c "$(declare -f init_contract die log)
  bash -c "$(declare -f init_contract die log); $(declare -p EXIT_INIT_FAIL)
           DEPLOY_LOG=/dev/null; NETWORK=testnet
           stellar() { return 0; }
           init_contract CTEST GCREATOR GTOKEN 1000 $FUTURE 10"

# ── Tests: check_network (curl stubbed) ──────────────────────────────────────

echo ""
echo "=== check_network ==="

assert_exit "passes when curl succeeds for testnet" 0 \
  bash -c "$(declare -f check_network warn die log); $(declare -p RPC_TESTNET RPC_MAINNET RPC_FUTURENET NETWORK_TIMEOUT EXIT_NETWORK_FAIL)
           DEPLOY_LOG=/dev/null; NETWORK=testnet
           curl() { return 0; }
           check_network"

assert_exit "passes when curl succeeds for mainnet" 0 \
  bash -c "$(declare -f check_network warn die log); $(declare -p RPC_TESTNET RPC_MAINNET RPC_FUTURENET NETWORK_TIMEOUT EXIT_NETWORK_FAIL)
           DEPLOY_LOG=/dev/null; NETWORK=mainnet
           curl() { return 0; }
           check_network"

assert_exit "passes when curl succeeds for futurenet" 0 \
  bash -c "$(declare -f check_network warn die log); $(declare -p RPC_TESTNET RPC_MAINNET RPC_FUTURENET NETWORK_TIMEOUT EXIT_NETWORK_FAIL)
           DEPLOY_LOG=/dev/null; NETWORK=futurenet
           curl() { return 0; }
           check_network"

assert_exit "exits EXIT_NETWORK_FAIL when curl fails" 6 \
  bash -c "$(declare -f check_network warn die log); $(declare -p RPC_TESTNET RPC_MAINNET RPC_FUTURENET NETWORK_TIMEOUT EXIT_NETWORK_FAIL)
           DEPLOY_LOG=/dev/null; NETWORK=testnet
           curl() { return 1; }
           check_network"

assert_exit "warns and passes for unknown network" 0 \
  bash -c "$(declare -f check_network warn die log); $(declare -p RPC_TESTNET RPC_MAINNET RPC_FUTURENET NETWORK_TIMEOUT EXIT_NETWORK_FAIL)
           DEPLOY_LOG=/dev/null; NETWORK=localnet; ERROR_COUNT=0
           check_network"

# ── Tests: log output ────────────────────────────────────────────────────────
  bash -c "$(declare -f init_contract die log emit_event)
           DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; NETWORK=testnet
           stellar() { return 0; }
           init_contract CTEST GCREATOR GTOKEN 1000 $FUTURE 10"

# ── Tests: log / die ─────────────────────────────────────────────────────────

echo ""
echo "=== log / die ==="

assert_output_contains "log writes level tag" "\[INFO\]" \
  bash -c "$(declare -f log); DEPLOY_LOG=/dev/null; log INFO 'hello'"

assert_output_contains "log writes message" "hello world" \
  bash -c "$(declare -f log); DEPLOY_LOG=/dev/null; log INFO 'hello world'"

assert_exit "die exits with supplied code" 3 \
  bash -c "$(declare -f log die emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; die 3 'boom'"

assert_output_contains "die logs ERROR level" "\[ERROR\]" \
  bash -c "$(declare -f log die emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; die 3 'boom'" || true

# ── Tests: emit_event / DEPLOY_JSON_LOG ──────────────────────────────────────

echo ""
echo "=== emit_event / DEPLOY_JSON_LOG ==="

_test_emit_event_fields() {
  local TMP; TMP=$(mktemp)
  bash -c "$(declare -f emit_event log)
           DEPLOY_JSON_LOG=\"$TMP\"; NETWORK=testnet
           emit_event step_ok build 'WASM built'" &>/dev/null
  # Must contain required JSON keys
  grep -q '"event":"step_ok"'  "$TMP" && \
  grep -q '"step":"build"'     "$TMP" && \
  grep -q '"network":"testnet"' "$TMP" && \
  grep -q '"timestamp"'        "$TMP"
  local rc=$?
  rm -f "$TMP"
  return $rc
}
assert_exit "emit_event writes event/step/network/timestamp fields" 0 _test_emit_event_fields

_test_emit_event_extra() {
  local TMP; TMP=$(mktemp)
  bash -c "$(declare -f emit_event log)
           DEPLOY_JSON_LOG=\"$TMP\"; NETWORK=testnet
           emit_event step_ok deploy 'deployed' '\"contract_id\":\"CABC\"'" &>/dev/null
  grep -q '"contract_id":"CABC"' "$TMP"
  local rc=$?
  rm -f "$TMP"
  return $rc
}
assert_exit "emit_event includes extra JSON fragment" 0 _test_emit_event_extra

_test_emit_event_escapes_quotes() {
  local TMP; TMP=$(mktemp)
  bash -c "$(declare -f emit_event log)
           DEPLOY_JSON_LOG=\"$TMP\"; NETWORK=testnet
           emit_event step_error validate 'bad \"value\"'" &>/dev/null
  # The file must still be parseable (no raw unescaped quote breaking JSON)
  grep -q 'bad \\\"value\\\"' "$TMP"
  local rc=$?
  rm -f "$TMP"
  return $rc
}
assert_exit "emit_event escapes double-quotes in message" 0 _test_emit_event_escapes_quotes

_test_die_writes_json_error() {
  local TMP_LOG TMP_JSON; TMP_LOG=$(mktemp); TMP_JSON=$(mktemp)
  bash -c "$(declare -f log die emit_event)
           DEPLOY_LOG=\"$TMP_LOG\"; DEPLOY_JSON_LOG=\"$TMP_JSON\"; NETWORK=testnet
           die 4 'deploy failed' 'stellar deploy' 'deploy'" &>/dev/null || true
  grep -q '"event":"step_error"' "$TMP_JSON" && \
  grep -q '"step":"deploy"'      "$TMP_JSON" && \
  grep -q '"exit_code":4'        "$TMP_JSON"
  local rc=$?
  rm -f "$TMP_LOG" "$TMP_JSON"
  return $rc
}
assert_exit "die writes step_error JSON event with exit_code and step" 0 _test_die_writes_json_error

_test_deploy_complete_event() {
  local TMP_LOG TMP_JSON TMP_WASM TMP_SCRIPT
  TMP_LOG=$(mktemp); TMP_JSON=$(mktemp)
  TMP_WASM=$(mktemp --suffix=.wasm)
  TMP_SCRIPT=$(mktemp --suffix=.sh)
  {
    echo "cargo()   { touch \"$TMP_WASM\"; return 0; }"
    echo 'stellar() { case "$2" in deploy) echo CDONE;; *) ;; esac; return 0; }'
    echo 'curl()    { return 0; }'
    sed 's/^main "\$@"$/: # stubbed/' "$SCRIPT"
    echo "WASM_PATH=\"$TMP_WASM\""
    echo "main GCREATOR GTOKEN 1000 $FUTURE 1"
  } > "$TMP_SCRIPT"
  DEPLOY_LOG="$TMP_LOG" DEPLOY_JSON_LOG="$TMP_JSON" NETWORK=testnet \
    bash "$TMP_SCRIPT" &>/dev/null
  local rc=$?
  grep -q '"event":"deploy_complete"' "$TMP_JSON" && \
  grep -q '"contract_id":"CDONE"'     "$TMP_JSON"
  local check=$?
  rm -f "$TMP_LOG" "$TMP_JSON" "$TMP_WASM" "$TMP_SCRIPT"
  [[ $rc -eq 0 && $check -eq 0 ]]
}
assert_exit "full run emits deploy_complete event with contract_id" 0 _test_deploy_complete_event

_test_json_log_truncated() {
  local TMP_LOG TMP_JSON TMP_WASM TMP_SCRIPT
  TMP_LOG=$(mktemp); TMP_JSON=$(mktemp)
  TMP_WASM=$(mktemp --suffix=.wasm)
  TMP_SCRIPT=$(mktemp --suffix=.sh)
  echo '{"event":"stale"}' > "$TMP_JSON"
  {
    echo "cargo()   { touch \"$TMP_WASM\"; return 0; }"
    echo 'stellar() { case "$2" in deploy) echo CXXX;; *) ;; esac; return 0; }'
    echo 'curl()    { return 0; }'
    sed 's/^main "\$@"$/: # stubbed/' "$SCRIPT"
    echo "WASM_PATH=\"$TMP_WASM\""
    echo "main GCREATOR GTOKEN 1000 $FUTURE 1"
  } > "$TMP_SCRIPT"
  DEPLOY_LOG="$TMP_LOG" DEPLOY_JSON_LOG="$TMP_JSON" NETWORK=testnet \
    bash "$TMP_SCRIPT" &>/dev/null
  ! grep -q '"event":"stale"' "$TMP_JSON"
  local rc=$?
  rm -f "$TMP_LOG" "$TMP_JSON" "$TMP_WASM" "$TMP_SCRIPT"
  return $rc
}
assert_exit "main truncates DEPLOY_JSON_LOG at start" 0 _test_json_log_truncated
  bash -c "$(declare -f log die); DEPLOY_LOG=/dev/null; die 3 'boom'"

assert_output_contains "die logs ERROR level" "\[ERROR\]" \
  bash -c "$(declare -f log die emit_event); DEPLOY_LOG=/dev/null; DEPLOY_JSON_LOG=/dev/null; die 3 'boom'" || true

# ── Tests: emit_event / DEPLOY_JSON_LOG ──────────────────────────────────────

echo ""
echo "=== emit_event / DEPLOY_JSON_LOG ==="

_test_emit_event_fields() {
  local TMP; TMP=$(mktemp)
  bash -c "$(declare -f emit_event log)
           DEPLOY_JSON_LOG=\"$TMP\"; NETWORK=testnet
           emit_event step_ok build 'WASM built'" &>/dev/null
  # Must contain required JSON keys
  grep -q '"event":"step_ok"'  "$TMP" && \
  grep -q '"step":"build"'     "$TMP" && \
  grep -q '"network":"testnet"' "$TMP" && \
  grep -q '"timestamp"'        "$TMP"
  local rc=$?
  rm -f "$TMP"
  return $rc
}
assert_exit "emit_event writes event/step/network/timestamp fields" 0 _test_emit_event_fields

_test_emit_event_extra() {
  local TMP; TMP=$(mktemp)
  bash -c "$(declare -f emit_event log)
           DEPLOY_JSON_LOG=\"$TMP\"; NETWORK=testnet
           emit_event step_ok deploy 'deployed' '\"contract_id\":\"CABC\"'" &>/dev/null
  grep -q '"contract_id":"CABC"' "$TMP"
  local rc=$?
  rm -f "$TMP"
  return $rc
}
assert_exit "emit_event includes extra JSON fragment" 0 _test_emit_event_extra

_test_emit_event_escapes_quotes() {
  local TMP; TMP=$(mktemp)
  bash -c "$(declare -f emit_event log)
           DEPLOY_JSON_LOG=\"$TMP\"; NETWORK=testnet
           emit_event step_error validate 'bad \"value\"'" &>/dev/null
  # The file must still be parseable (no raw unescaped quote breaking JSON)
  grep -q 'bad \\\"value\\\"' "$TMP"
  local rc=$?
  rm -f "$TMP"
  return $rc
}
assert_exit "emit_event escapes double-quotes in message" 0 _test_emit_event_escapes_quotes

_test_die_writes_json_error() {
  local TMP_LOG TMP_JSON; TMP_LOG=$(mktemp); TMP_JSON=$(mktemp)
  bash -c "$(declare -f log die emit_event)
           DEPLOY_LOG=\"$TMP_LOG\"; DEPLOY_JSON_LOG=\"$TMP_JSON\"; NETWORK=testnet
           die 4 'deploy failed' 'stellar deploy' 'deploy'" &>/dev/null || true
  grep -q '"event":"step_error"' "$TMP_JSON" && \
  grep -q '"step":"deploy"'      "$TMP_JSON" && \
  grep -q '"exit_code":4'        "$TMP_JSON"
  local rc=$?
  rm -f "$TMP_LOG" "$TMP_JSON"
  return $rc
}
assert_exit "die writes step_error JSON event with exit_code and step" 0 _test_die_writes_json_error

_test_deploy_complete_event() {
  local TMP_LOG TMP_JSON TMP_WASM TMP_SCRIPT
  TMP_LOG=$(mktemp); TMP_JSON=$(mktemp)
  TMP_WASM=$(mktemp --suffix=.wasm)
  TMP_SCRIPT=$(mktemp --suffix=.sh)
  {
    echo "cargo()   { touch \"$TMP_WASM\"; return 0; }"
    echo 'stellar() { case "$2" in deploy) echo CDONE;; *) ;; esac; return 0; }'
    echo 'curl()    { return 0; }'
    sed 's/^main "\$@"$/: # stubbed/' "$SCRIPT"
    echo "WASM_PATH=\"$TMP_WASM\""
    echo "main GCREATOR GTOKEN 1000 $FUTURE 1"
  } > "$TMP_SCRIPT"
  DEPLOY_LOG="$TMP_LOG" DEPLOY_JSON_LOG="$TMP_JSON" NETWORK=testnet \
    bash "$TMP_SCRIPT" &>/dev/null
  local rc=$?
  grep -q '"event":"deploy_complete"' "$TMP_JSON" && \
  grep -q '"contract_id":"CDONE"'     "$TMP_JSON"
  local check=$?
  rm -f "$TMP_LOG" "$TMP_JSON" "$TMP_WASM" "$TMP_SCRIPT"
  [[ $rc -eq 0 && $check -eq 0 ]]
}
assert_exit "full run emits deploy_complete event with contract_id" 0 _test_deploy_complete_event

_test_json_log_truncated() {
  local TMP_LOG TMP_JSON TMP_WASM TMP_SCRIPT
  TMP_LOG=$(mktemp); TMP_JSON=$(mktemp)
  TMP_WASM=$(mktemp --suffix=.wasm)
  TMP_SCRIPT=$(mktemp --suffix=.sh)
  echo '{"event":"stale"}' > "$TMP_JSON"
  {
    echo "cargo()   { touch \"$TMP_WASM\"; return 0; }"
    echo 'stellar() { case "$2" in deploy) echo CXXX;; *) ;; esac; return 0; }'
    echo 'curl()    { return 0; }'
    sed 's/^main "\$@"$/: # stubbed/' "$SCRIPT"
    echo "WASM_PATH=\"$TMP_WASM\""
    echo "main GCREATOR GTOKEN 1000 $FUTURE 1"
  } > "$TMP_SCRIPT"
  DEPLOY_LOG="$TMP_LOG" DEPLOY_JSON_LOG="$TMP_JSON" NETWORK=testnet \
    bash "$TMP_SCRIPT" &>/dev/null
  ! grep -q '"event":"stale"' "$TMP_JSON"
  local rc=$?
  rm -f "$TMP_LOG" "$TMP_JSON" "$TMP_WASM" "$TMP_SCRIPT"
  return $rc
}
assert_exit "main truncates DEPLOY_JSON_LOG at start" 0 _test_json_log_truncated

# ── Tests: DEPLOY_LOG file capture ───────────────────────────────────────────

echo ""
echo "=== DEPLOY_LOG file capture ==="

assert_exit "log appends to DEPLOY_LOG file" 0 \
  bash -c "$(declare -f log)
           TMP=\$(mktemp); DEPLOY_LOG=\"\$TMP\"
           log INFO 'test entry'
           grep -q 'test entry' \"\$TMP\"
           rm -f \"\$TMP\""

_test_main_truncates_log() {
  local TMP_LOG TMP_JSON TMP_WASM TMP_SCRIPT
  TMP_LOG=$(mktemp); TMP_JSON=$(mktemp)
  TMP_WASM=$(mktemp --suffix=.wasm)
  TMP_SCRIPT=$(mktemp --suffix=.sh)
  echo 'stale content' > "$TMP_LOG"
  {
    echo "cargo()   { touch \"$TMP_WASM\"; return 0; }"
    echo 'stellar() { case "$2" in deploy) echo CXXX;; *) ;; esac; return 0; }'
    echo 'curl()    { return 0; }'
    sed 's/^main "\$@"$/: # stubbed/' "$SCRIPT"
    echo "WASM_PATH=\"$TMP_WASM\""
    echo "main GCREATOR GTOKEN 1000 $FUTURE 1"
  } > "$TMP_SCRIPT"
  DEPLOY_LOG="$TMP_LOG" DEPLOY_JSON_LOG="$TMP_JSON" NETWORK=testnet \
    bash "$TMP_SCRIPT" &>/dev/null
  local rc=$?
  ! grep -q 'stale content' "$TMP_LOG"
  local check=$?
  rm -f "$TMP_LOG" "$TMP_JSON" "$TMP_WASM" "$TMP_SCRIPT"
  [[ $rc -eq 0 && $check -eq 0 ]]
}
assert_exit "main truncates DEPLOY_LOG at start" 0 _test_main_truncates_log


# ── Bug condition tests (Property 1) ─────────────────────────────────────────
# @notice Req 2.1–2.3: constants expand to correct values after fix.

_test_help_exit_codes() {
  local out rc=0
  out=$(bash "$SCRIPT" --help 2>&1) || rc=$?
  [[ $rc -eq 0 ]] || return 1
  for code in 0 1 2 3 4 5 6; do
    echo "$out" | grep -qE "(^|[[:space:]])${code}([[:space:]]|$)" || return 1
  done
}
assert_exit "print_help renders exit codes 0-6 (req 2.1)" 0 _test_help_exit_codes

_test_network_url() {
  local tmpdir; tmpdir=$(mktemp -d)
  local url_file="$tmpdir/url"
  cat > "$tmpdir/curl" <<STUB
#!/usr/bin/env bash
for arg in "\$@"; do case "\$arg" in -*) ;; *) echo "\$arg" > "$url_file" ;; esac; done
exit 0
STUB
  chmod +x "$tmpdir/curl"
  for cmd in cargo stellar jq; do printf '#!/usr/bin/env bash\nexit 0\n' > "$tmpdir/$cmd"; chmod +x "$tmpdir/$cmd"; done
  local future=$(( $(date +%s) + 86400 ))
  PATH="$tmpdir:$PATH" NETWORK=testnet bash "$SCRIPT" GCREATOR GTOKEN 1000 "$future" 100 &>/dev/null || true
  local url; url=$(cat "$url_file" 2>/dev/null || echo "")
  rm -rf "$tmpdir"
  [[ "$url" == https://* ]]
}
assert_exit "check_network passes HTTPS URL to curl (req 2.2)" 0 _test_network_url

_test_default_min() {
  local future=$(( $(date +%s) + 86400 ))
  local out rc=0
  out=$(bash "$SCRIPT" --dry-run GCREATOR GTOKEN 1000 "$future" 2>&1) || rc=$?
  [[ $rc -ne 2 ]] && ! echo "$out" | grep -q "min_contribution must be a positive integer"
}
assert_exit "default min_contribution does not cause exit 2 (req 2.3)" 0 _test_default_min

# ── Preservation tests (Property 2) ──────────────────────────────────────────
# @notice Req 3.1–3.4: non-constant-dependent behavior unchanged.

_test_explicit_five_args() {
  local future=$(( $(date +%s) + 86400 ))
  for mc in 1 10 100 1000000; do
    local out rc=0
    out=$(bash "$SCRIPT" --dry-run GCREATOR GTOKEN 1000 "$future" "$mc" 2>&1) || rc=$?
    if [[ $rc -eq 2 ]] && echo "$out" | grep -q "min_contribution must be a positive integer"; then
      return 1
    fi
  done
}
assert_exit "explicit 5 args: valid min_contribution values pass (req 3.1, 3.4)" 0 _test_explicit_five_args

_test_unknown_network() {
  local tmpdir; tmpdir=$(mktemp -d)
  for cmd in cargo stellar jq curl; do printf '#!/usr/bin/env bash\nexit 0\n' > "$tmpdir/$cmd"; chmod +x "$tmpdir/$cmd"; done
  local future=$(( $(date +%s) + 86400 ))
  local out rc=0
  out=$(PATH="$tmpdir:/usr/bin:/bin" NETWORK=localnet bash "$SCRIPT" GCREATOR GTOKEN 1000 "$future" 100 2>&1) || rc=$?
  rm -rf "$tmpdir"
  echo "$out" | grep -q "Unknown network 'localnet'" && ! echo "$out" | grep -q "Network connectivity"
}
assert_exit "unknown network triggers warn+skip path (req 3.2)" 0 _test_unknown_network

_test_missing_creator() {
  local future=$(( $(date +%s) + 86400 ))
  local out rc=0
  out=$(bash "$SCRIPT" --dry-run "" GTOKEN 1000 "$future" 100 2>&1) || rc=$?
  [[ $rc -ne 0 ]] && echo "$out" | grep -qE "creator is required"
}
assert_exit "missing creator exits non-zero with error (req 3.3)" 0 _test_missing_creator

  local TMP_LOG TMP_SCRIPT FUTURE
  TMP_LOG=$(mktemp)
  TMP_SCRIPT=$(mktemp --suffix=.sh)
  FUTURE=$(( $(date +%s) + 86400 ))
  echo 'stale content' > "$TMP_LOG"

  local TMP_WASM
  local TMP_LOG TMP_JSON TMP_WASM TMP_SCRIPT
  TMP_LOG=$(mktemp); TMP_JSON=$(mktemp)
  TMP_WASM=$(mktemp --suffix=.wasm)
  TMP_SCRIPT=$(mktemp --suffix=.sh)
  echo 'stale content' > "$TMP_LOG"
  {
    echo "cargo()   { touch \"$TMP_WASM\"; return 0; }"
    echo 'stellar() { case "$2" in deploy) echo CXXX;; *) ;; esac; return 0; }'
    echo 'curl()    { return 0; }'
    sed 's/^main "\$@"$/: # stubbed/' "$SCRIPT"
    echo "WASM_PATH=\"$TMP_WASM\""
  {
    # Stub cargo and stellar before sourcing so they override any PATH lookup
    echo 'cargo()   { return 0; }'
    echo 'stellar() { case "$2" in deploy) echo CXXX;; *) ;; esac; return 0; }'
    # Inline the deployment script with "main "$@"" replaced by a no-op
    sed 's/^main "\$@"$/: # stubbed/' "$SCRIPT"
    echo 'curl()    { return 0; }'
    # Patch readonly WASM_PATH to point at our temp file before the script declares it
    sed "s|^readonly WASM_PATH=.*|readonly WASM_PATH=\"$TMP_WASM\"|" "$SCRIPT" \
      | sed 's/^main "\$@"$/: # stubbed/'
    echo "main GCREATOR GTOKEN 1000 $FUTURE 1"
  } > "$TMP_SCRIPT"
  DEPLOY_LOG="$TMP_LOG" DEPLOY_JSON_LOG="$TMP_JSON" NETWORK=testnet \
    bash "$TMP_SCRIPT" &>/dev/null
  local rc=$?
  rm -f "$TMP_SCRIPT" "$TMP_WASM"
  rm -f "$TMP_SCRIPT"

  if [[ $rc -eq 0 ]] && ! grep -q 'stale content' "$TMP_LOG"; then
    rm -f "$TMP_LOG"
    return 0
  fi
  rm -f "$TMP_LOG"
  return 1
  ! grep -q 'stale content' "$TMP_LOG"
  local check=$?
  rm -f "$TMP_LOG" "$TMP_JSON" "$TMP_WASM" "$TMP_SCRIPT"
  [[ $rc -eq 0 && $check -eq 0 ]]
}
assert_exit "main truncates DEPLOY_LOG at start" 0 _test_main_truncates_log

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
echo "Results: $PASS passed, $FAIL failed"
[[ "$FAIL" -eq 0 ]] || exit 1

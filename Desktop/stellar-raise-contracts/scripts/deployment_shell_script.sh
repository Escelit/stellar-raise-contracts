#!/usr/bin/env bash
# @title   deployment_shell_script.sh
# @notice  Builds, deploys, and initialises the Stellar Raise crowdfund contract
#          on a target network with structured error capturing and logging.
# @dev     Requires: stellar CLI (>=0.0.18), Rust + wasm32-unknown-unknown target.
#          Human-readable log  → DEPLOY_LOG      (default: deploy_errors.log)
#          Structured JSON log → DEPLOY_JSON_LOG (default: deploy_events.json)
#            Each line is a self-contained JSON object (NDJSON) the frontend UI
#            can stream-parse to display live progress and typed error messages.
#          Exit codes:
#            0  – success
#            1  – missing dependency
#            2  – invalid / missing argument
#            3  – build failure
#            4  – deploy failure
#            5  – initialise failure
#            6  – network connectivity failure

set -euo pipefail

# ── Exit code constants ───────────────────────────────────────────────────────
EXIT_OK=0
EXIT_MISSING_DEP=1
EXIT_BAD_ARG=2
EXIT_BUILD_FAIL=3
EXIT_DEPLOY_FAIL=4
EXIT_INIT_FAIL=5
EXIT_NETWORK_FAIL=6

RPC_TESTNET="https://soroban-testnet.stellar.org"
RPC_MAINNET="https://mainnet.stellar.validationcloud.io/v1/xycpM7GIGz7BKZQ7IQKM"
RPC_FUTURENET="https://rpc-futurenet.stellar.org"

DEFAULT_MIN_CONTRIBUTION=1

NETWORK="${NETWORK:-testnet}"
DEPLOY_LOG="${DEPLOY_LOG:-deploy_errors.log}"
DEPLOY_JSON_LOG="${DEPLOY_JSON_LOG:-deploy_events.json}"
WASM_PATH="target/wasm32-unknown-unknown/release/crowdfund.wasm"
DRY_RUN="${DRY_RUN:-false}"
ERROR_COUNT=0

# ── Helpers ──────────────────────────────────────────────────────────────────

# @notice Writes a timestamped message to stdout and the human-readable log.
# @param  $1  severity  (INFO | WARN | ERROR)
# @param  $2  message
log() {
  local level="$1" msg="$2"
  local ts; ts="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  echo "[$ts] [$level] $msg" | tee -a "$DEPLOY_LOG"
}

# @notice Appends one NDJSON event line to DEPLOY_JSON_LOG.
#         The frontend UI reads this file to render live step status and errors.
# @param  $1  event   – step_start | step_ok | step_error | deploy_complete
# @param  $2  step    – validate | build | deploy | init | network_check | done
# @param  $3  message – human-readable description (double-quotes escaped)
# @param  $4  extra   – optional raw JSON fragment appended inside the object
#                       e.g. '"contract_id":"CXXX"'
emit_event() {
  local event="$1" step="$2" msg="$3" extra="${4:-}"
  local ts; ts="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  # Escape double-quotes and backslashes in the message so the JSON stays valid.
  local safe_msg="${msg//\\/\\\\}"; safe_msg="${safe_msg//\"/\\\"}"
  local json="{\"event\":\"$event\",\"step\":\"$step\",\"message\":\"$safe_msg\",\"timestamp\":\"$ts\",\"network\":\"$NETWORK\""
  [[ -n "$extra" ]] && json="${json},${extra}"
  echo "${json}}" >> "$DEPLOY_JSON_LOG"
}

# @notice Logs an error, emits a JSON step_error event, and exits.
#         Increments ERROR_COUNT before exit.
# @param  $1  exit_code
# @param  $2  message
# @param  $3  context  (optional – failed command or extra detail)
# @param  $4  step     (optional – which pipeline step failed; default: unknown)
die() {
  local code="$1" msg="$2" context="${3:-}" step="${4:-unknown}"
  (( ERROR_COUNT++ )) || true
  log "ERROR" "$msg"
  [[ -n "$context" ]] && log "ERROR" "  context: $context"
  log "ERROR" "  exit_code=$code  errors_total=$ERROR_COUNT"
  local safe_ctx="${context//\\/\\\\}"; safe_ctx="${safe_ctx//\"/\\\"}"
  emit_event "step_error" "$step" "$msg" \
    "\"exit_code\":$code,\"context\":\"$safe_ctx\",\"error_count\":$ERROR_COUNT"
  exit "$code"
}

# @notice Records a non-fatal warning and increments the error counter.
# @param  $1  message
warn() {
  (( ERROR_COUNT++ )) || true
  log "WARN" "$1"
}

# @notice Verifies that a required CLI tool is present on PATH.
# @param  $1  tool name
require_tool() {
  command -v "$1" &>/dev/null \
    || die 1 "Required tool not found: $1" "Ensure '$1' is installed and on your PATH" "validate"
}

# @notice Runs a command, capturing stderr to DEPLOY_LOG and timing the step.
# @param  $@  command and arguments
run_captured() {
  local start end rc=0
  start=$(date +%s)
  "$@" 2>>"$DEPLOY_LOG" || rc=$?
  end=$(date +%s)
  log "INFO" "  step_duration=$(( end - start ))s  command='$1'"
  return $rc
}

# @notice Prints usage and exits 0.
print_help() {
  cat <<HELPEOF
Usage: deployment_shell_script.sh [OPTIONS] <creator> <token> <goal> <deadline> [min_contribution]

Builds, deploys, and initialises the Stellar Raise crowdfund contract.

Positional arguments:
  creator            Stellar address of the campaign creator
  token              Stellar address of the token contract
  goal               Funding goal in stroops (positive integer)
  deadline           Unix timestamp for campaign end (must be in the future)
  min_contribution   Minimum pledge amount (default: $DEFAULT_MIN_CONTRIBUTION)

Options:
  --help             Show this help message and exit
  --dry-run          Validate arguments and dependencies without deploying

Environment variables:
  NETWORK            Stellar network to target          (default: testnet)
  DEPLOY_LOG         Human-readable log path            (default: deploy_errors.log)
  DEPLOY_JSON_LOG    Structured NDJSON event log path   (default: deploy_events.json)
  DRY_RUN            Set to 'true' to enable dry-run mode

Exit codes:
  $EXIT_OK  success             $EXIT_BUILD_FAIL  build failure        $EXIT_NETWORK_FAIL  network failure
  $EXIT_MISSING_DEP  missing dependency  $EXIT_DEPLOY_FAIL  deploy failure
  $EXIT_BAD_ARG  invalid argument    $EXIT_INIT_FAIL  init failure
HELPEOF
  exit $EXIT_OK
}

# ── Argument validation ───────────────────────────────────────────────────────

# @notice Validates all required positional arguments before any network call.
# @param  $1  creator          – Stellar address of the campaign creator
# @param  $2  token            – Stellar address of the token contract
# @param  $3  goal             – Funding goal (positive integer, stroops)
# @param  $4  deadline         – Unix timestamp; must be in the future
# @param  $5  min_contribution – Minimum pledge amount (positive integer)
validate_args() {
  local creator="$1" token="$2" goal="$3" deadline="$4" min_contribution="$5"

  [[ -n "$creator" ]]                       || die 2 "creator is required"                                    "" "validate"
  [[ -n "$token" ]]                         || die 2 "token is required"                                      "" "validate"
  [[ "$goal" =~ ^[0-9]+$ ]]                 || die 2 "goal must be a positive integer, got: '$goal'"          "" "validate"
  [[ "$deadline" =~ ^[0-9]+$ ]]             || die 2 "deadline must be a Unix timestamp, got: '$deadline'"    "" "validate"
  [[ "$min_contribution" =~ ^[0-9]+$ ]]     || die 2 "min_contribution must be a positive integer"            "" "validate"

  local now; now="$(date +%s)"
  (( deadline > now )) || die 2 "deadline must be in the future (got $deadline, now $now)" "" "validate"
}

# ── Network pre-check ────────────────────────────────────────────────────────

# @notice Lightweight connectivity check against the target network RPC endpoint.
#         Skipped for unknown networks; exits 6 on failure.
check_network() {
  local rpc_url
  case "$NETWORK" in
    testnet)   rpc_url="$RPC_TESTNET"   ;;
    mainnet)   rpc_url="$RPC_MAINNET"   ;;
    futurenet) rpc_url="$RPC_FUTURENET" ;;
    *)
      warn "Unknown network '$NETWORK' — skipping connectivity pre-check"
      return 0
      ;;
  esac
  emit_event "step_start" "network_check" "Checking connectivity to $NETWORK"
  log "INFO" "Checking network connectivity ($NETWORK)..."
  if ! curl --silent --fail --max-time 10 "$rpc_url" &>/dev/null 2>>"$DEPLOY_LOG"; then
    die 6 "Network connectivity check failed for $NETWORK" \
          "GET $rpc_url timed out or returned non-200" "network_check"
  fi
  emit_event "step_ok" "network_check" "Network reachable"
  log "INFO" "Network reachable."
}

# ── Core steps ───────────────────────────────────────────────────────────────

# @notice Compiles the contract to WASM using the WASM_TARGET constant.
build_contract() {
  emit_event "step_start" "build" "Building WASM"
  log "INFO" "Building WASM..."
  if ! run_captured cargo build --target wasm32-unknown-unknown --release; then
    die 3 "cargo build failed – see $DEPLOY_LOG for details" \
          "cargo build --target wasm32-unknown-unknown --release" "build"
  fi
  [[ -f "$WASM_PATH" ]] || die 3 "WASM artifact not found at $WASM_PATH after build" "" "build"
  emit_event "step_ok" "build" "WASM built successfully" "\"wasm_path\":\"$WASM_PATH\""
  log "INFO" "Build succeeded: $WASM_PATH"
}

# @notice Deploys the WASM to the network; prints the contract ID to stdout.
# @param  $1  source – signing identity (named Stellar CLI key, never a raw secret)
# @custom:security Never pass a raw secret key as source; use a named identity.
deploy_contract() {
  local source="$1"
  emit_event "step_start" "deploy" "Deploying to $NETWORK"
  log "INFO" "Deploying to $NETWORK..."
  local contract_id
  if ! contract_id=$(stellar contract deploy \
      --wasm "$WASM_PATH" \
      --network "$NETWORK" \
      --source "$source" 2>>"$DEPLOY_LOG"); then
    die 4 "stellar contract deploy failed – see $DEPLOY_LOG for details" \
          "stellar contract deploy --network $NETWORK" "deploy"
  fi
  [[ -n "$contract_id" ]] || die 4 "Deploy returned an empty contract ID" "" "deploy"
  emit_event "step_ok" "deploy" "Contract deployed" "\"contract_id\":\"$contract_id\""
  log "INFO" "Contract deployed: $contract_id"
  echo "$contract_id"
}

# @notice Calls initialize on the deployed contract.
# @param  $1  contract_id
# @param  $2  creator
# @param  $3  token
# @param  $4  goal
# @param  $5  deadline
# @param  $6  min_contribution
init_contract() {
  local contract_id="$1" creator="$2" token="$3" goal="$4" deadline="$5" min_contribution="$6"
  emit_event "step_start" "init" "Initialising campaign on $contract_id"
  log "INFO" "Initialising campaign on contract $contract_id..."
  if ! stellar contract invoke \
      --id "$contract_id" \
      --network "$NETWORK" \
      --source "$creator" \
      -- initialize \
      --creator "$creator" \
      --token "$token" \
      --goal "$goal" \
      --deadline "$deadline" \
      --min_contribution "$min_contribution" 2>>"$DEPLOY_LOG"; then
    die 5 "Contract initialisation failed – see $DEPLOY_LOG for details" \
          "stellar contract invoke --id $contract_id -- initialize" "init"
  fi
  emit_event "step_ok" "init" "Campaign initialised successfully"
  log "INFO" "Campaign initialised successfully."
}

# @notice Prints a final human-readable summary.
print_summary() {
  echo ""
  if [[ "$ERROR_COUNT" -gt 0 ]]; then
    log "WARN" "Completed with $ERROR_COUNT warning(s). Review $DEPLOY_LOG for details."
  else
    log "INFO" "Deployment completed successfully with 0 errors."
  fi
}

# ── Entry point ───────────────────────────────────────────────────────────────

main() {
  local positional=()
  for arg in "$@"; do
    case "$arg" in
      --help)    print_help ;;
      --dry-run) DRY_RUN="true" ;;
      *)         positional+=("$arg") ;;
    esac
  done

  local creator="${positional[0]:-}"
  local token="${positional[1]:-}"
  local goal="${positional[2]:-}"
  local deadline="${positional[3]:-}"
  local min_contribution="${positional[4]:-$DEFAULT_MIN_CONTRIBUTION}"

  # Truncate both logs for this run
  : > "$DEPLOY_LOG"
  : > "$DEPLOY_JSON_LOG"

  require_tool cargo
  require_tool stellar

  validate_args "$creator" "$token" "$goal" "$deadline" "$min_contribution"

  if [[ "$DRY_RUN" == "true" ]]; then
    log "INFO" "Dry-run mode: arguments and dependencies validated. Skipping build/deploy/init."
    emit_event "deploy_complete" "done" "Dry-run validation passed" \
      "\"dry_run\":true,\"error_count\":$ERROR_COUNT"
    print_summary
    return 0
  fi

  check_network

  build_contract
  local contract_id
  contract_id="$(deploy_contract "$creator")"
  init_contract "$contract_id" "$creator" "$token" "$goal" "$deadline" "$min_contribution"

  emit_event "deploy_complete" "done" "Deployment finished" \
    "\"contract_id\":\"$contract_id\",\"error_count\":$ERROR_COUNT"
  print_summary

  echo ""
  echo "Contract ID: $contract_id"
  echo "Save this Contract ID for interacting with the campaign."
}

main "$@"

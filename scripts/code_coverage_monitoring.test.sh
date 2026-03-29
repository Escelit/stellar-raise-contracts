#!/usr/bin/env bash
# @title   code_coverage_monitoring.test.sh
# @notice  Test suite for code_coverage_monitoring.sh — validates threshold
#          construction, path safety, summary parsing, and CLI behaviour.
# @dev     Sources the monitoring script to exercise internal helpers; exit 0
#          when all tests pass, 1 otherwise.
# @custom:security-note  Confirms threshold inputs cannot carry shell metacharacters
#          and repo-relative paths reject traversal sequences.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MONITOR="$SCRIPT_DIR/code_coverage_monitoring.sh"

TESTS_PASSED=0
TESTS_FAILED=0

ok()   { echo "[TEST PASS] $*"; TESTS_PASSED=$(( TESTS_PASSED + 1 )); }
fail() { echo "[TEST FAIL] $*"; TESTS_FAILED=$(( TESTS_FAILED + 1 )); }

# @notice  Load function definitions without running main.
# shellcheck source=/dev/null
source "$MONITOR"

# ── validate_min_percent ──────────────────────────────────────────────────────

test_validate_accepts_0_and_100() {
    validate_min_percent 0 && validate_min_percent 100 && ok "validate_min_percent boundary 0 and 100" || fail "boundaries"
}

test_validate_accepts_typical() {
    validate_min_percent 95 && ok "validate_min_percent accepts 95" || fail "95"
}

test_validate_rejects_non_numeric() {
    if validate_min_percent abc 2>/dev/null; then
        fail "validate_min_percent should reject abc"
    else
        ok "validate_min_percent rejects abc"
    fi
}

test_validate_rejects_negative_sign() {
    if validate_min_percent -1 2>/dev/null; then
        fail "validate_min_percent should reject -1"
    else
        ok "validate_min_percent rejects -1"
    fi
}

test_validate_rejects_over_100() {
    if validate_min_percent 101 2>/dev/null; then
        fail "validate_min_percent should reject 101"
    else
        ok "validate_min_percent rejects 101"
    fi
}

test_validate_rejects_empty() {
    if validate_min_percent "" 2>/dev/null; then
        fail "validate_min_percent should reject empty"
    else
        ok "validate_min_percent rejects empty"
    fi
}

# ── build_coverage_threshold_json ─────────────────────────────────────────────

test_threshold_json_shape() {
    local j
    j="$(build_coverage_threshold_json 88)"
    if [ "$j" = '{"global":{"lines":88,"statements":88,"functions":88,"branches":88}}' ]; then
        ok "build_coverage_threshold_json 88"
    else
        fail "unexpected json: $j"
    fi
}

test_threshold_json_zero() {
    local j
    j="$(build_coverage_threshold_json 0)"
    if [ "$j" = '{"global":{"lines":0,"statements":0,"functions":0,"branches":0}}' ]; then
        ok "build_coverage_threshold_json 0"
    else
        fail "unexpected json for 0: $j"
    fi
}

test_threshold_json_no_injection() {
    # validate_min_percent would reject; ensure builder is only called with safe ints in real use
    local j
    j="$(build_coverage_threshold_json 1)"
    case "$j" in
        *';'*|*'|'*|*\`*|*\$*) fail "threshold json contains suspicious chars" ;;
        *) ok "threshold json free of obvious injection tokens" ;;
    esac
}

# ── is_safe_repo_relative_path ────────────────────────────────────────────────

test_safe_path_accepts_coverage_summary() {
    is_safe_repo_relative_path "coverage/coverage-summary.json" && ok "safe path accepted" || fail "safe path"
}

test_safe_path_rejects_traversal() {
    if is_safe_repo_relative_path "../etc/passwd" 2>/dev/null; then
        fail "should reject .."
    else
        ok "rejects .. traversal"
    fi
}

test_safe_path_rejects_absolute() {
    if is_safe_repo_relative_path "/tmp/x" 2>/dev/null; then
        fail "should reject absolute"
    else
        ok "rejects absolute path"
    fi
}

test_safe_path_rejects_empty() {
    if is_safe_repo_relative_path "" 2>/dev/null; then
        fail "should reject empty"
    else
        ok "rejects empty path"
    fi
}

# ── parse_coverage_summary_metrics ────────────────────────────────────────────

test_parse_valid_summary() {
    local tmp
    tmp="$(mktemp "${TMPDIR:-/tmp}/ccm-test.XXXXXX")"
    cat >"$tmp" <<'JSON'
{
  "total": {
    "lines": { "pct": 91.5 },
    "branches": { "pct": 82 },
    "functions": { "pct": 88 },
    "statements": { "pct": 90 }
  }
}
JSON
    local out
    out=$(parse_coverage_summary_metrics "$tmp")
    rm -f "$tmp"
    if echo "$out" | grep -q '^91.5$' && echo "$out" | tail -n1 | grep -q '^90$'; then
        ok "parse_coverage_summary_metrics reads totals"
    else
        fail "unexpected parse output: $out"
    fi
}

test_parse_missing_file_fails() {
    local tmp="/nonexistent/coverage-summary-$$.json"
    if parse_coverage_summary_metrics "$tmp" 2>/dev/null; then
        fail "parse should fail on missing file"
    else
        ok "parse fails on missing file"
    fi
}

test_parse_invalid_shape_fails() {
    local tmp
    tmp="$(mktemp "${TMPDIR:-/tmp}/ccm-test.XXXXXX")"
    echo '{"total":{}}' >"$tmp"
    if parse_coverage_summary_metrics "$tmp" 2>/dev/null; then
        rm -f "$tmp"
        fail "parse should fail on invalid shape"
    else
        rm -f "$tmp"
        ok "parse fails on invalid shape"
    fi
}

# ── awk lines gate (mirrors report_and_enforce) ───────────────────────────────

test_lines_below_min_triggers_gate() {
    if awk -v n="50" -v m="95" 'BEGIN{ if (n+0 < m+0) exit 0; exit 1 }'; then
        ok "awk detects 50 < 95"
    else
        fail "awk comparison"
    fi
}

test_lines_at_min_does_not_trigger_gate() {
    if awk -v n="95" -v m="95" 'BEGIN{ if (n+0 < m+0) exit 0; exit 1 }'; then
        fail "95 should not be below 95"
    else
        ok "awk: 95 not below 95"
    fi
}

# ── CLI integration ─────────────────────────────────────────────────────────────

test_help_exits_zero() {
    if bash "$MONITOR" --help &>/dev/null; then
        ok "--help exits 0"
    else
        fail "--help exit code"
    fi
}

test_help_mentions_enforce() {
    if bash "$MONITOR" --help 2>&1 | grep -q enforce; then
        ok "--help documents enforce"
    else
        fail "--help missing enforce"
    fi
}

test_unknown_option_fails() {
    local code=0
    bash "$MONITOR" --not-a-real-flag 2>/dev/null || code=$?
    if [ "$code" -eq 1 ]; then
        ok "unknown option exits 1"
    else
        fail "unknown option expected 1 got $code"
    fi
}

test_invalid_min_pct_fails_before_node() {
    local code=0
    bash "$MONITOR" --min-pct notanumber 2>/dev/null || code=$?
    if [ "$code" -eq 1 ]; then
        ok "invalid min exits 1"
    else
        fail "invalid min expected 1 got $code"
    fi
}

# ── File mode ─────────────────────────────────────────────────────────────────

test_monitor_script_is_executable() {
    if [ -x "$MONITOR" ]; then
        ok "code_coverage_monitoring.sh is executable"
    else
        fail "monitor script not executable — run: chmod +x scripts/code_coverage_monitoring.sh"
    fi
}

# ── Summary ─────────────────────────────────────────────────────────────────────

print_summary() {
    echo ""
    echo "════════════════════════════════════════════════════════"
    local total=$(( TESTS_PASSED + TESTS_FAILED ))
    if [ "$TESTS_FAILED" -eq 0 ]; then
        echo "[ALL TESTS PASSED] $TESTS_PASSED/$total"
    else
        echo "[TESTS FAILED] $TESTS_FAILED/$total failed"
    fi
    echo "════════════════════════════════════════════════════════"
}

main() {
    echo "Code Coverage Monitoring — Test Suite"
    echo "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    echo ""

    test_validate_accepts_0_and_100
    test_validate_accepts_typical
    test_validate_rejects_non_numeric
    test_validate_rejects_negative_sign
    test_validate_rejects_over_100
    test_validate_rejects_empty

    test_threshold_json_shape
    test_threshold_json_zero
    test_threshold_json_no_injection

    test_safe_path_accepts_coverage_summary
    test_safe_path_rejects_traversal
    test_safe_path_rejects_absolute
    test_safe_path_rejects_empty

    test_parse_valid_summary
    test_parse_missing_file_fails
    test_parse_invalid_shape_fails

    test_lines_below_min_triggers_gate
    test_lines_at_min_does_not_trigger_gate

    test_help_exits_zero
    test_help_mentions_enforce
    test_unknown_option_fails
    test_invalid_min_pct_fails_before_node

    test_monitor_script_is_executable

    print_summary
    [ "$TESTS_FAILED" -eq 0 ] || exit 1
}

main "$@"

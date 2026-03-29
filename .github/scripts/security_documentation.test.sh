#!/usr/bin/env bash
# =============================================================================
# security_documentation.test.sh
#
# @title  Test suite for security_documentation.sh
# @notice Validates every function and edge case in the security documentation
#         CI script to ensure ≥ 95 % coverage of all code paths.
#
# @dev    Tests are grouped into six sections:
#           1. Script structure (set -euo pipefail, EXIT trap, no eval)
#           2. append_result — JSON field format and quote escaping
#           3. NatSpec validation — pass, fail, skip, empty-dir edge cases
#           4. Test coverage parity — pass, fail, skip, zero-function edge cases
#           5. Security assumptions — pass, fail, skip, no-docs edge cases
#           6. Documentation index generation — populated and empty cases
#           7. Counter logic and overall PASS/FAIL status
#           8. Environment variable defaults
#           9. Integration dry-run (SKIP_DOC_VALIDATION=1, missing dirs)
#
# @custom:security-note  This test suite must pass in CI before any merge to
#          main or develop.  A failing test here indicates a regression in the
#          security documentation pipeline itself.
#
# Usage:
#   bash .github/scripts/security_documentation.test.sh
#
# Exit code:
#   0  All tests passed
#   1  One or more tests failed
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SUBJECT="${SCRIPT_DIR}/security_documentation.sh"

# ── Colour helpers ────────────────────────────────────────────────────────────

RED="\033[0;31m" GREEN="\033[0;32m" YELLOW="\033[0;33m" RESET="\033[0m"

# ── Test counters ─────────────────────────────────────────────────────────────

TESTS_RUN=0
TESTS_PASSED=0
TESTS_FAILED=0

# ── Test helpers ──────────────────────────────────────────────────────────────

pass() { echo -e "${GREEN}[PASS]${RESET} $1"; TESTS_PASSED=$((TESTS_PASSED + 1)); TESTS_RUN=$((TESTS_RUN + 1)); }
fail() { echo -e "${RED}[FAIL]${RESET} $1"; TESTS_FAILED=$((TESTS_FAILED + 1)); TESTS_RUN=$((TESTS_RUN + 1)); }
skip() { echo -e "${YELLOW}[SKIP]${RESET} $1"; TESTS_RUN=$((TESTS_RUN + 1)); }

assert_contains() {
  local label="$1" haystack="$2" needle="$3"
  if echo "${haystack}" | grep -q "${needle}"; then
    pass "${label}"
  else
    fail "${label} — expected to find: '${needle}'"
    echo "  Output was: ${haystack}" >&2
  fi
}

assert_not_contains() {
  local label="$1" haystack="$2" needle="$3"
  if ! echo "${haystack}" | grep -q "${needle}"; then
    pass "${label}"
  else
    fail "${label} — expected NOT to find: '${needle}'"
  fi
}

assert_exit_zero() {
  local label="$1" exit_code="$2"
  if [[ "${exit_code}" -eq 0 ]]; then
    pass "${label}"
  else
    fail "${label} — expected exit 0, got ${exit_code}"
  fi
}

assert_exit_nonzero() {
  local label="$1" exit_code="$2"
  if [[ "${exit_code}" -ne 0 ]]; then
    pass "${label}"
  else
    fail "${label} — expected non-zero exit, got 0"
  fi
}

# ── Fixture helpers ───────────────────────────────────────────────────────────

# @notice  Creates a minimal security .rs source file with all required
#          NatSpec annotations.
make_annotated_rs() {
  local path="$1"
  cat > "${path}" <<'EOF'
//! @notice  Test security module.
//! @dev     Used only in tests.
//! @custom:security-note  No real security logic here.

pub fn check_example(value: i128) -> bool {
    value > 0
}

pub fn probe_example(value: i128) -> bool {
    value != 0
}
EOF
}

# @notice  Creates a minimal security .rs source file missing all NatSpec.
make_unannotated_rs() {
  local path="$1"
  cat > "${path}" <<'EOF'
pub fn check_missing(value: i128) -> bool {
    value > 0
}
EOF
}

# @notice  Creates a test file that references check_example and probe_example.
make_test_rs() {
  local path="$1"
  cat > "${path}" <<'EOF'
#[test]
fn test_check_example() { assert!(check_example(1)); }

#[test]
fn test_probe_example() { assert!(probe_example(1)); }
EOF
}

# @notice  Creates a security Markdown doc with a Security Assumptions section.
make_security_doc_with_assumptions() {
  local path="$1"
  cat > "${path}" <<'EOF'
# Security Test Module

## Overview
Test module.

## Security Assumptions
1. Values are non-negative.
2. Callers are authenticated.
EOF
}

# @notice  Creates a security Markdown doc WITHOUT a Security Assumptions section.
make_security_doc_without_assumptions() {
  local path="$1"
  cat > "${path}" <<'EOF'
# Security Test Module

## Overview
Test module with no assumptions section.
EOF
}

# ── Section 1: Script structure ───────────────────────────────────────────────

echo ""
echo "=== Section 1: Script structure ==="

if [[ -f "${SUBJECT}" ]]; then
  pass "subject script exists"
else
  fail "subject script not found: ${SUBJECT}"
fi

if head -5 "${SUBJECT}" | grep -q "set -euo pipefail"; then
  pass "script uses set -euo pipefail"
else
  fail "script missing set -euo pipefail"
fi

if grep -q "trap.*EXIT" "${SUBJECT}"; then
  pass "script has EXIT trap for cleanup"
else
  fail "script missing EXIT trap"
fi

if grep -q "eval" "${SUBJECT}"; then
  fail "script contains eval (security risk)"
else
  pass "script does not use eval"
fi

if grep -q "mktemp -d" "${SUBJECT}"; then
  pass "script uses mktemp for temp directory"
else
  fail "script does not use mktemp"
fi

if grep -q "append_result" "${SUBJECT}"; then
  pass "script defines append_result helper"
else
  fail "script missing append_result helper"
fi

# ── Section 2: append_result JSON format ─────────────────────────────────────

echo ""
echo "=== Section 2: append_result JSON format ==="

TMPDIR_TEST="$(mktemp -d)"
trap 'rm -rf "${TMPDIR_TEST}"' EXIT

# Source only the append_result function by extracting and evaluating it safely.
APPEND_FN="$(sed -n '/^append_result()/,/^}/p' "${SUBJECT}")"

# Test: basic PASS result produces valid JSON fields.
(
  TMPDIR_WORK="${TMPDIR_TEST}/work1"
  mkdir -p "${TMPDIR_WORK}"
  touch "${TMPDIR_WORK}/results.ndjson"
  TIMESTAMP="2026-01-01T00:00:00Z"
  eval "${APPEND_FN}"
  append_result "natspec_coverage" "PASS" "all 3 files annotated"
  OUTPUT="$(cat "${TMPDIR_WORK}/results.ndjson")"
  echo "${OUTPUT}" | grep -q '"check":"natspec_coverage"' && echo "JSON_CHECK_OK"
  echo "${OUTPUT}" | grep -q '"status":"PASS"' && echo "JSON_STATUS_OK"
  echo "${OUTPUT}" | grep -q '"detail":"all 3 files annotated"' && echo "JSON_DETAIL_OK"
  echo "${OUTPUT}" | grep -q '"timestamp":"2026-01-01T00:00:00Z"' && echo "JSON_TS_OK"
) > "${TMPDIR_TEST}/append_out.txt" 2>&1

APPEND_OUT="$(cat "${TMPDIR_TEST}/append_out.txt")"
assert_contains "append_result: check field present"  "${APPEND_OUT}" "JSON_CHECK_OK"
assert_contains "append_result: status field present" "${APPEND_OUT}" "JSON_STATUS_OK"
assert_contains "append_result: detail field present" "${APPEND_OUT}" "JSON_DETAIL_OK"
assert_contains "append_result: timestamp field present" "${APPEND_OUT}" "JSON_TS_OK"

# Test: double-quotes in detail are escaped.
(
  TMPDIR_WORK="${TMPDIR_TEST}/work2"
  mkdir -p "${TMPDIR_WORK}"
  touch "${TMPDIR_WORK}/results.ndjson"
  TIMESTAMP="2026-01-01T00:00:00Z"
  eval "${APPEND_FN}"
  append_result "test" "FAIL" 'detail with "quotes" inside'
  cat "${TMPDIR_WORK}/results.ndjson"
) > "${TMPDIR_TEST}/escape_out.txt" 2>&1

ESCAPE_OUT="$(cat "${TMPDIR_TEST}/escape_out.txt")"
assert_contains "append_result: quotes escaped in detail" "${ESCAPE_OUT}" '\\"quotes\\"'

# ── Section 3: NatSpec validation ─────────────────────────────────────────────

echo ""
echo "=== Section 3: NatSpec validation ==="

# Test: all files annotated → PASS
FIXTURE_PASS="${TMPDIR_TEST}/natspec_pass"
mkdir -p "${FIXTURE_PASS}"
make_annotated_rs "${FIXTURE_PASS}/security_example.rs"

OUTPUT="$(SECURITY_SRC_DIR="${FIXTURE_PASS}" DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_np" \
  DOCS_DIR="${TMPDIR_TEST}/nodocs" SKIP_DOC_VALIDATION=0 CI=true \
  bash "${SUBJECT}" 2>&1)" || true
assert_contains "NatSpec pass: annotated file → PASS" "${OUTPUT}" "PASS"
assert_not_contains "NatSpec pass: no FAIL" "${OUTPUT}" "\[FAIL\].*NatSpec"

# Test: unannotated file → FAIL
FIXTURE_FAIL="${TMPDIR_TEST}/natspec_fail"
mkdir -p "${FIXTURE_FAIL}"
make_unannotated_rs "${FIXTURE_FAIL}/security_bad.rs"

OUTPUT="$(SECURITY_SRC_DIR="${FIXTURE_FAIL}" DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_nf" \
  DOCS_DIR="${TMPDIR_TEST}/nodocs" SKIP_DOC_VALIDATION=0 CI=true \
  bash "${SUBJECT}" 2>&1)" || true
assert_contains "NatSpec fail: unannotated file → FAIL" "${OUTPUT}" "FAIL"

# Test: SKIP_DOC_VALIDATION=1 → SKIPPED
OUTPUT="$(SECURITY_SRC_DIR="${FIXTURE_FAIL}" DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_ns" \
  DOCS_DIR="${TMPDIR_TEST}/nodocs" SKIP_DOC_VALIDATION=1 CI=true \
  bash "${SUBJECT}" 2>&1)" || true
assert_contains "NatSpec skip: SKIP_DOC_VALIDATION=1 → SKIPPED" "${OUTPUT}" "SKIPPED"

# Test: missing source directory → WARN (not FAIL)
OUTPUT="$(SECURITY_SRC_DIR="${TMPDIR_TEST}/nonexistent_src" \
  DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_nm" \
  DOCS_DIR="${TMPDIR_TEST}/nodocs" SKIP_DOC_VALIDATION=0 CI=true \
  bash "${SUBJECT}" 2>&1)" || true
assert_contains "NatSpec missing dir → WARN" "${OUTPUT}" "WARN"

# Test: test files are excluded from NatSpec check
FIXTURE_TESTONLY="${TMPDIR_TEST}/natspec_testonly"
mkdir -p "${FIXTURE_TESTONLY}"
# Only a test file — should not trigger NatSpec failure.
cat > "${FIXTURE_TESTONLY}/security_example.test.rs" <<'EOF'
#[test]
fn test_something() {}
EOF

OUTPUT="$(SECURITY_SRC_DIR="${FIXTURE_TESTONLY}" DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_nt" \
  DOCS_DIR="${TMPDIR_TEST}/nodocs" SKIP_DOC_VALIDATION=0 CI=true \
  bash "${SUBJECT}" 2>&1)" || true
assert_not_contains "NatSpec: test files excluded from check" "${OUTPUT}" "MISSING @notice.*test.rs"

# ── Section 4: Test coverage parity ──────────────────────────────────────────

echo ""
echo "=== Section 4: Test coverage parity ==="

# Test: all functions have tests → PASS
FIXTURE_PARITY_PASS="${TMPDIR_TEST}/parity_pass"
mkdir -p "${FIXTURE_PARITY_PASS}"
make_annotated_rs "${FIXTURE_PARITY_PASS}/security_example.rs"
make_test_rs "${FIXTURE_PARITY_PASS}/security_example.test.rs"

OUTPUT="$(SECURITY_SRC_DIR="${FIXTURE_PARITY_PASS}" DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_pp" \
  DOCS_DIR="${TMPDIR_TEST}/nodocs" SKIP_DOC_VALIDATION=1 CI=true \
  bash "${SUBJECT}" 2>&1)" || true
assert_contains "Parity pass: all functions tested → PASS" "${OUTPUT}" "PASS"

# Test: function without test → FAIL
FIXTURE_PARITY_FAIL="${TMPDIR_TEST}/parity_fail"
mkdir -p "${FIXTURE_PARITY_FAIL}"
make_annotated_rs "${FIXTURE_PARITY_FAIL}/security_example.rs"
# No test file — check_example and probe_example have no tests.

OUTPUT="$(SECURITY_SRC_DIR="${FIXTURE_PARITY_FAIL}" DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_pf" \
  DOCS_DIR="${TMPDIR_TEST}/nodocs" SKIP_DOC_VALIDATION=1 CI=true \
  bash "${SUBJECT}" 2>&1)" || true
assert_contains "Parity fail: untested function → FAIL" "${OUTPUT}" "FAIL"

# Test: no check_*/probe_* functions → WARN
FIXTURE_PARITY_EMPTY="${TMPDIR_TEST}/parity_empty"
mkdir -p "${FIXTURE_PARITY_EMPTY}"
cat > "${FIXTURE_PARITY_EMPTY}/security_empty.rs" <<'EOF'
//! @notice  Empty module.
//! @dev     No public functions.
//! @custom:security-note  Nothing to test.
pub fn helper() {}
EOF

OUTPUT="$(SECURITY_SRC_DIR="${FIXTURE_PARITY_EMPTY}" DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_pe" \
  DOCS_DIR="${TMPDIR_TEST}/nodocs" SKIP_DOC_VALIDATION=1 CI=true \
  bash "${SUBJECT}" 2>&1)" || true
assert_contains "Parity empty: no check_*/probe_* → WARN" "${OUTPUT}" "WARN"

# Test: missing source directory → WARN
OUTPUT="$(SECURITY_SRC_DIR="${TMPDIR_TEST}/nonexistent_parity" \
  DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_pm" \
  DOCS_DIR="${TMPDIR_TEST}/nodocs" SKIP_DOC_VALIDATION=1 CI=true \
  bash "${SUBJECT}" 2>&1)" || true
assert_contains "Parity missing dir → WARN" "${OUTPUT}" "WARN"

# ── Section 5: Security assumptions ──────────────────────────────────────────

echo ""
echo "=== Section 5: Security assumptions ==="

# Test: doc with assumptions → PASS
FIXTURE_DOCS_PASS="${TMPDIR_TEST}/docs_pass"
mkdir -p "${FIXTURE_DOCS_PASS}"
make_security_doc_with_assumptions "${FIXTURE_DOCS_PASS}/security_example.md"

OUTPUT="$(SECURITY_SRC_DIR="${TMPDIR_TEST}/nonexistent_src2" \
  DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_ap" \
  DOCS_DIR="${FIXTURE_DOCS_PASS}" SKIP_DOC_VALIDATION=1 CI=true \
  bash "${SUBJECT}" 2>&1)" || true
assert_contains "Assumptions pass: doc has section → PASS" "${OUTPUT}" "PASS"

# Test: doc without assumptions → FAIL
FIXTURE_DOCS_FAIL="${TMPDIR_TEST}/docs_fail"
mkdir -p "${FIXTURE_DOCS_FAIL}"
make_security_doc_without_assumptions "${FIXTURE_DOCS_FAIL}/security_bad.md"

OUTPUT="$(SECURITY_SRC_DIR="${TMPDIR_TEST}/nonexistent_src3" \
  DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_af" \
  DOCS_DIR="${FIXTURE_DOCS_FAIL}" SKIP_DOC_VALIDATION=1 CI=true \
  bash "${SUBJECT}" 2>&1)" || true
assert_contains "Assumptions fail: missing section → FAIL" "${OUTPUT}" "FAIL"

# Test: no security_*.md files → WARN
FIXTURE_DOCS_EMPTY="${TMPDIR_TEST}/docs_empty"
mkdir -p "${FIXTURE_DOCS_EMPTY}"
echo "# Not a security doc" > "${FIXTURE_DOCS_EMPTY}/readme.md"

OUTPUT="$(SECURITY_SRC_DIR="${TMPDIR_TEST}/nonexistent_src4" \
  DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_ae" \
  DOCS_DIR="${FIXTURE_DOCS_EMPTY}" SKIP_DOC_VALIDATION=1 CI=true \
  bash "${SUBJECT}" 2>&1)" || true
assert_contains "Assumptions empty: no security docs → WARN" "${OUTPUT}" "WARN"

# Test: missing docs directory → WARN
OUTPUT="$(SECURITY_SRC_DIR="${TMPDIR_TEST}/nonexistent_src5" \
  DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_am" \
  DOCS_DIR="${TMPDIR_TEST}/nonexistent_docs" SKIP_DOC_VALIDATION=1 CI=true \
  bash "${SUBJECT}" 2>&1)" || true
assert_contains "Assumptions missing dir → WARN" "${OUTPUT}" "WARN"

# ── Section 6: Documentation index generation ─────────────────────────────────

echo ""
echo "=== Section 6: Documentation index generation ==="

# Test: index is created and contains expected sections.
FIXTURE_INDEX_DOCS="${TMPDIR_TEST}/index_docs"
mkdir -p "${FIXTURE_INDEX_DOCS}"
make_security_doc_with_assumptions "${FIXTURE_INDEX_DOCS}/security_monitoring.md"

INDEX_OUT_DIR="${TMPDIR_TEST}/index_out"
OUTPUT="$(SECURITY_SRC_DIR="${TMPDIR_TEST}/nonexistent_src6" \
  DOCS_OUTPUT_DIR="${INDEX_OUT_DIR}" \
  DOCS_DIR="${FIXTURE_INDEX_DOCS}" SKIP_DOC_VALIDATION=1 CI=true \
  bash "${SUBJECT}" 2>&1)" || true

assert_contains "Index: output dir created" "${OUTPUT}" "Doc index"

if [[ -f "${INDEX_OUT_DIR}/security_documentation_index.md" ]]; then
  pass "Index: index file created"
  INDEX_CONTENT="$(cat "${INDEX_OUT_DIR}/security_documentation_index.md")"
  assert_contains "Index: has title" "${INDEX_CONTENT}" "Security Documentation Index"
  assert_contains "Index: has generated timestamp" "${INDEX_CONTENT}" "Generated:"
  assert_contains "Index: has CI/CD section" "${INDEX_CONTENT}" "CI/CD Integration"
  assert_contains "Index: lists security doc" "${INDEX_CONTENT}" "security_monitoring"
else
  fail "Index: index file not created at ${INDEX_OUT_DIR}/security_documentation_index.md"
fi

# Test: JSON report is created.
if ls "${INDEX_OUT_DIR}"/security_documentation_report_*.json 1>/dev/null 2>&1; then
  pass "Index: JSON report file created"
  REPORT_CONTENT="$(cat "${INDEX_OUT_DIR}"/security_documentation_report_*.json)"
  assert_contains "Report: has report_timestamp" "${REPORT_CONTENT}" "report_timestamp"
  assert_contains "Report: has overall_status" "${REPORT_CONTENT}" "overall_status"
  assert_contains "Report: has checks array" "${REPORT_CONTENT}" '"checks"'
else
  fail "Index: JSON report file not created"
fi

# ── Section 7: Counter logic and overall status ───────────────────────────────

echo ""
echo "=== Section 7: Counter logic and overall status ==="

# Test: all checks pass → exit 0
FIXTURE_ALL_PASS_SRC="${TMPDIR_TEST}/all_pass_src"
FIXTURE_ALL_PASS_DOCS="${TMPDIR_TEST}/all_pass_docs"
mkdir -p "${FIXTURE_ALL_PASS_SRC}" "${FIXTURE_ALL_PASS_DOCS}"
make_annotated_rs "${FIXTURE_ALL_PASS_SRC}/security_example.rs"
make_test_rs "${FIXTURE_ALL_PASS_SRC}/security_example.test.rs"
make_security_doc_with_assumptions "${FIXTURE_ALL_PASS_DOCS}/security_example.md"

EXIT_CODE=0
bash "${SUBJECT}" \
  SECURITY_SRC_DIR="${FIXTURE_ALL_PASS_SRC}" \
  DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_all_pass" \
  DOCS_DIR="${FIXTURE_ALL_PASS_DOCS}" \
  SKIP_DOC_VALIDATION=0 CI=true 2>/dev/null || EXIT_CODE=$?
# Note: env vars must be exported for the subprocess.
EXIT_CODE=0
(
  export SECURITY_SRC_DIR="${FIXTURE_ALL_PASS_SRC}"
  export DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_all_pass2"
  export DOCS_DIR="${FIXTURE_ALL_PASS_DOCS}"
  export SKIP_DOC_VALIDATION=0
  export CI=true
  bash "${SUBJECT}" > /dev/null 2>&1
) && EXIT_CODE=0 || EXIT_CODE=$?
assert_exit_zero "All pass → exit 0" "${EXIT_CODE}"

# Test: critical failure → exit 1
EXIT_CODE=0
(
  export SECURITY_SRC_DIR="${FIXTURE_PARITY_FAIL}"
  export DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_crit_fail"
  export DOCS_DIR="${FIXTURE_DOCS_FAIL}"
  export SKIP_DOC_VALIDATION=0
  export CI=true
  bash "${SUBJECT}" > /dev/null 2>&1
) && EXIT_CODE=0 || EXIT_CODE=$?
assert_exit_nonzero "Critical failure → exit 1" "${EXIT_CODE}"

# Test: WARN-only run → exit 0 (warns do not block build)
EXIT_CODE=0
(
  export SECURITY_SRC_DIR="${TMPDIR_TEST}/nonexistent_warn_src"
  export DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_warn_only"
  export DOCS_DIR="${TMPDIR_TEST}/nonexistent_warn_docs"
  export SKIP_DOC_VALIDATION=0
  export CI=true
  bash "${SUBJECT}" > /dev/null 2>&1
) && EXIT_CODE=0 || EXIT_CODE=$?
assert_exit_zero "WARN-only run → exit 0" "${EXIT_CODE}"

# ── Section 8: Environment variable defaults ──────────────────────────────────

echo ""
echo "=== Section 8: Environment variable defaults ==="

assert_contains "Default DOCS_OUTPUT_DIR" "$(grep 'DOCS_OUTPUT_DIR:-' "${SUBJECT}")" "./security-docs"
assert_contains "Default SECURITY_SRC_DIR" "$(grep 'SECURITY_SRC_DIR:-' "${SUBJECT}")" "contracts/security/src"
assert_contains "Default DOCS_DIR" "$(grep 'DOCS_DIR:-' "${SUBJECT}")" "docs"
assert_contains "Default SKIP_DOC_VALIDATION" "$(grep 'SKIP_DOC_VALIDATION:-' "${SUBJECT}")" "0"

# ── Section 9: Integration dry-run ───────────────────────────────────────────

echo ""
echo "=== Section 9: Integration dry-run ==="

# Test: SKIP_DOC_VALIDATION=1 with missing dirs → all SKIPPED/WARN, exit 0
EXIT_CODE=0
OUTPUT="$(
  export SECURITY_SRC_DIR="${TMPDIR_TEST}/nonexistent_int_src"
  export DOCS_OUTPUT_DIR="${TMPDIR_TEST}/out_integration"
  export DOCS_DIR="${TMPDIR_TEST}/nonexistent_int_docs"
  export SKIP_DOC_VALIDATION=1
  export CI=true
  bash "${SUBJECT}" 2>&1
)" && EXIT_CODE=0 || EXIT_CODE=$?

assert_exit_zero "Integration dry-run: exit 0" "${EXIT_CODE}"
assert_contains "Integration dry-run: NatSpec skipped" "${OUTPUT}" "SKIPPED"
assert_contains "Integration dry-run: report header present" "${OUTPUT}" "Automated Security Documentation"
assert_contains "Integration dry-run: JSON report line" "${OUTPUT}" "JSON report"

# Test: output directory is created by the script.
if [[ -d "${TMPDIR_TEST}/out_integration" ]]; then
  pass "Integration dry-run: output directory created"
else
  fail "Integration dry-run: output directory not created"
fi

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
echo "════════════════════════════════════════════════════════════"
echo "Test Results: ${TESTS_PASSED}/${TESTS_RUN} passed, ${TESTS_FAILED} failed"
echo "════════════════════════════════════════════════════════════"

if [[ "${TESTS_FAILED}" -gt 0 ]]; then
  echo -e "${RED}FAIL${RESET} — ${TESTS_FAILED} test(s) failed."
  exit 1
fi

echo -e "${GREEN}PASS${RESET} — All ${TESTS_PASSED} tests passed."
exit 0

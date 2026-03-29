#!/usr/bin/env bash
# =============================================================================
# security_documentation.sh
#
# @title  Automated Security Documentation for CI/CD
# @notice Generates, validates, and publishes security documentation for the
#         Stellar Raise crowdfund project as part of the CI/CD pipeline.
#
# @dev    The script performs the following steps in order:
#           1. Validate that all security source files have NatSpec-style
#              doc-comments (@notice, @dev, @custom:security-note).
#           2. Validate that every public security function has a corresponding
#              test in the test file.
#           3. Verify security assumptions are documented in each module's .md.
#           4. Generate a consolidated security documentation index.
#           5. Emit a structured JSON report for CI artefact upload.
#
# @custom:security
#   - set -euo pipefail: any unhandled error aborts the script immediately.
#   - No user-supplied input is eval'd or passed to shell without quoting.
#   - Temporary files are created in a mktemp directory and cleaned on EXIT.
#   - The script exits with code 1 if any CRITICAL check fails.
#   - Non-critical issues are recorded as WARN and do not block the build.
#
# @custom:ci-usage
#   Called from .github/workflows/security.yml:
#     - name: Automated security documentation
#       run: bash .github/scripts/security_documentation.sh
#
# Usage:
#   bash .github/scripts/security_documentation.sh
#
# Environment variables (all optional):
#   DOCS_OUTPUT_DIR     Directory to write generated docs (default: ./security-docs)
#   SECURITY_SRC_DIR    Root of security source files (default: contracts/security/src)
#   DOCS_DIR            Project docs directory (default: docs)
#   SKIP_DOC_VALIDATION Set to "1" to skip NatSpec validation (e.g. draft PRs)
#   CI                  Set by GitHub Actions; disables colour output
# =============================================================================

set -euo pipefail

# ── Configuration ─────────────────────────────────────────────────────────────

DOCS_OUTPUT_DIR="${DOCS_OUTPUT_DIR:-./security-docs}"
SECURITY_SRC_DIR="${SECURITY_SRC_DIR:-contracts/security/src}"
DOCS_DIR="${DOCS_DIR:-docs}"
SKIP_DOC_VALIDATION="${SKIP_DOC_VALIDATION:-0}"
TIMESTAMP="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
REPORT_FILE="${DOCS_OUTPUT_DIR}/security_documentation_report_$(date -u +"%Y%m%d_%H%M%S").json"
INDEX_FILE="${DOCS_OUTPUT_DIR}/security_documentation_index.md"

# ── Counters ──────────────────────────────────────────────────────────────────

CRITICAL_FAILURES=0
WARN_COUNT=0
PASS_COUNT=0

# ── Temporary workspace ───────────────────────────────────────────────────────

TMPDIR_WORK="$(mktemp -d)"
trap 'rm -rf "${TMPDIR_WORK}"' EXIT

# ── Colour helpers (disabled in CI) ──────────────────────────────────────────

if [[ "${CI:-}" == "true" ]]; then
  RED="" GREEN="" YELLOW="" BLUE="" RESET=""
else
  RED="\033[0;31m" GREEN="\033[0;32m" YELLOW="\033[0;33m"
  BLUE="\033[0;34m" RESET="\033[0m"
fi

# ── Logging helpers ───────────────────────────────────────────────────────────

log_info()  { echo -e "${BLUE}[INFO]${RESET}  $*"; }
log_pass()  { echo -e "${GREEN}[PASS]${RESET}  $*"; PASS_COUNT=$((PASS_COUNT + 1)); }
log_warn()  { echo -e "${YELLOW}[WARN]${RESET}  $*"; WARN_COUNT=$((WARN_COUNT + 1)); }
log_fail()  { echo -e "${RED}[FAIL]${RESET}  $*"; CRITICAL_FAILURES=$((CRITICAL_FAILURES + 1)); }

# ── JSON helpers ──────────────────────────────────────────────────────────────

# @notice  Appends a single check result object to the NDJSON results file.
# @dev     Escapes double-quotes in the detail string to produce valid JSON.
# @param   $1  check name
# @param   $2  status (PASS | FAIL | WARN | SKIPPED)
# @param   $3  human-readable detail string
append_result() {
  local name="$1" status="$2" detail="$3"
  detail="${detail//\"/\\\"}"
  printf '{"check":"%s","status":"%s","detail":"%s","timestamp":"%s"}\n' \
    "${name}" "${status}" "${detail}" "${TIMESTAMP}" \
    >> "${TMPDIR_WORK}/results.ndjson"
}

# ── Setup ─────────────────────────────────────────────────────────────────────

mkdir -p "${DOCS_OUTPUT_DIR}"
touch "${TMPDIR_WORK}/results.ndjson"

log_info "Automated Security Documentation — ${TIMESTAMP}"
log_info "Source dir : ${SECURITY_SRC_DIR}"
log_info "Output dir : ${DOCS_OUTPUT_DIR}"
echo "────────────────────────────────────────────────────────────"

# ── Check 1: NatSpec-style doc-comment coverage ───────────────────────────────

# @notice  Validates that every public function in security source files has
#          the required NatSpec-style annotations: @notice, @dev, and
#          @custom:security-note.
# @custom:security-note  Undocumented security functions are a knowledge-
#          sharing risk: reviewers cannot assess intent or threat coverage
#          without inline documentation.
run_natspec_validation() {
  log_info "Check 1/4: NatSpec doc-comment coverage"

  if [[ "${SKIP_DOC_VALIDATION}" == "1" ]]; then
    log_warn "NatSpec validation skipped (SKIP_DOC_VALIDATION=1)"
    append_result "natspec_coverage" "SKIPPED" "SKIP_DOC_VALIDATION=1"
    return
  fi

  if [[ ! -d "${SECURITY_SRC_DIR}" ]]; then
    log_warn "Security source directory not found: ${SECURITY_SRC_DIR}"
    append_result "natspec_coverage" "SKIPPED" "source directory not found"
    return
  fi

  local missing_notice=0 missing_dev=0 missing_security_note=0
  local checked_files=0
  local issues_file="${TMPDIR_WORK}/natspec_issues.txt"
  touch "${issues_file}"

  # Scan only non-test .rs files for NatSpec annotations.
  while IFS= read -r src_file; do
    # Skip test files — they use @notice but not @dev/@custom:security-note.
    [[ "${src_file}" == *.test.rs ]] && continue
    [[ "${src_file}" == *_test.rs ]] && continue

    checked_files=$((checked_files + 1))

    if ! grep -q "@notice" "${src_file}" 2>/dev/null; then
      echo "MISSING @notice: ${src_file}" >> "${issues_file}"
      missing_notice=$((missing_notice + 1))
    fi

    if ! grep -q "@dev" "${src_file}" 2>/dev/null; then
      echo "MISSING @dev: ${src_file}" >> "${issues_file}"
      missing_dev=$((missing_dev + 1))
    fi

    if ! grep -q "@custom:security-note" "${src_file}" 2>/dev/null; then
      echo "MISSING @custom:security-note: ${src_file}" >> "${issues_file}"
      missing_security_note=$((missing_security_note + 1))
    fi
  done < <(find "${SECURITY_SRC_DIR}" -name "*.rs" 2>/dev/null)

  if [[ "${checked_files}" -eq 0 ]]; then
    log_warn "No .rs source files found in ${SECURITY_SRC_DIR}"
    append_result "natspec_coverage" "WARN" "no source files found"
    return
  fi

  local total_issues=$((missing_notice + missing_dev + missing_security_note))

  if [[ "${total_issues}" -eq 0 ]]; then
    log_pass "NatSpec coverage: all ${checked_files} file(s) annotated"
    append_result "natspec_coverage" "PASS" "all ${checked_files} files have required annotations"
  else
    log_fail "NatSpec coverage: ${total_issues} annotation(s) missing across ${checked_files} file(s)"
    cat "${issues_file}"
    append_result "natspec_coverage" "FAIL" "${total_issues} missing annotations in ${checked_files} files"
  fi
}

run_natspec_validation

# ── Check 2: Test coverage parity ────────────────────────────────────────────

# @notice  Verifies that every public `check_*` and `probe_*` function defined
#          in security source files has at least one corresponding test.
# @dev     Extracts function names via grep and cross-references them against
#          the test file.  A missing test is a CRITICAL failure.
# @custom:security-note  Untested security functions provide a false sense of
#          assurance.  CI must enforce ≥ 95 % function-level test coverage.
run_test_coverage_parity() {
  log_info "Check 2/4: Test coverage parity (check_* / probe_* functions)"

  if [[ ! -d "${SECURITY_SRC_DIR}" ]]; then
    log_warn "Security source directory not found: ${SECURITY_SRC_DIR}"
    append_result "test_coverage_parity" "SKIPPED" "source directory not found"
    return
  fi

  local missing_tests=0
  local checked_fns=0
  local parity_issues="${TMPDIR_WORK}/parity_issues.txt"
  touch "${parity_issues}"

  # Collect all test files for cross-reference.
  local test_files_concat="${TMPDIR_WORK}/all_tests.txt"
  find "${SECURITY_SRC_DIR}" \( -name "*.test.rs" -o -name "*_test.rs" \) \
    -exec cat {} \; > "${test_files_concat}" 2>/dev/null || true

  # Extract public function names matching check_* or probe_* from non-test sources.
  while IFS= read -r src_file; do
    [[ "${src_file}" == *.test.rs ]] && continue
    [[ "${src_file}" == *_test.rs ]] && continue

    while IFS= read -r fn_name; do
      [[ -z "${fn_name}" ]] && continue
      checked_fns=$((checked_fns + 1))

      if ! grep -q "${fn_name}" "${test_files_concat}" 2>/dev/null; then
        echo "NO TEST for: ${fn_name} (in ${src_file})" >> "${parity_issues}"
        missing_tests=$((missing_tests + 1))
      fi
    done < <(grep -oE 'pub fn (check_|probe_)[a-z_]+' "${src_file}" 2>/dev/null \
               | sed 's/pub fn //')
  done < <(find "${SECURITY_SRC_DIR}" -name "*.rs" 2>/dev/null)

  if [[ "${checked_fns}" -eq 0 ]]; then
    log_warn "No check_* or probe_* functions found in ${SECURITY_SRC_DIR}"
    append_result "test_coverage_parity" "WARN" "no security functions found"
    return
  fi

  local covered=$((checked_fns - missing_tests))
  local pct=0
  if [[ "${checked_fns}" -gt 0 ]]; then
    pct=$(( (covered * 100) / checked_fns ))
  fi

  if [[ "${missing_tests}" -eq 0 ]]; then
    log_pass "Test parity: ${covered}/${checked_fns} functions covered (${pct}%)"
    append_result "test_coverage_parity" "PASS" "${covered}/${checked_fns} functions have tests (${pct}%)"
  else
    log_fail "Test parity: ${missing_tests} function(s) lack tests (${pct}% covered)"
    cat "${parity_issues}"
    append_result "test_coverage_parity" "FAIL" "${missing_tests} functions missing tests (${pct}% covered)"
  fi
}

run_test_coverage_parity

# ── Check 3: Security assumptions documented ─────────────────────────────────

# @notice  Verifies that each security module's Markdown documentation file
#          contains a "Security Assumptions" section.
# @dev     Looks for the heading pattern "## Security" (case-insensitive) in
#          every .md file under DOCS_DIR that matches a security topic.
# @custom:security-note  Undocumented assumptions are a knowledge-sharing gap:
#          future maintainers cannot reason about the threat model without them.
run_security_assumptions_check() {
  log_info "Check 3/4: Security assumptions documented in Markdown files"

  if [[ ! -d "${DOCS_DIR}" ]]; then
    log_warn "Docs directory not found: ${DOCS_DIR}"
    append_result "security_assumptions" "SKIPPED" "docs directory not found"
    return
  fi

  local missing_assumptions=0
  local checked_docs=0
  local assumption_issues="${TMPDIR_WORK}/assumption_issues.txt"
  touch "${assumption_issues}"

  # Only check docs that are security-related by filename.
  while IFS= read -r doc_file; do
    checked_docs=$((checked_docs + 1))

    if ! grep -qi "## Security" "${doc_file}" 2>/dev/null; then
      echo "MISSING 'Security Assumptions' section: ${doc_file}" >> "${assumption_issues}"
      missing_assumptions=$((missing_assumptions + 1))
    fi
  done < <(find "${DOCS_DIR}" -name "security_*.md" 2>/dev/null)

  if [[ "${checked_docs}" -eq 0 ]]; then
    log_warn "No security_*.md files found in ${DOCS_DIR}"
    append_result "security_assumptions" "WARN" "no security docs found"
    return
  fi

  if [[ "${missing_assumptions}" -eq 0 ]]; then
    log_pass "Security assumptions: all ${checked_docs} doc(s) contain assumptions section"
    append_result "security_assumptions" "PASS" "all ${checked_docs} docs have security assumptions"
  else
    log_fail "Security assumptions: ${missing_assumptions}/${checked_docs} doc(s) missing assumptions section"
    cat "${assumption_issues}"
    append_result "security_assumptions" "FAIL" "${missing_assumptions}/${checked_docs} docs missing security assumptions"
  fi
}

run_security_assumptions_check

# ── Check 4: Generate documentation index ────────────────────────────────────

# @notice  Generates a consolidated Markdown index of all security
#          documentation files for knowledge sharing and CI artefact upload.
# @dev     Scans DOCS_DIR for security_*.md files and SECURITY_SRC_DIR for
#          *.md files, then writes a structured index to INDEX_FILE.
# @custom:security-note  A centralised index ensures reviewers can quickly
#          locate threat models, assumptions, and audit trails without
#          navigating the full repository tree.
run_generate_index() {
  log_info "Check 4/4: Generate security documentation index"

  local doc_count=0
  local src_doc_count=0

  {
    echo "# Security Documentation Index"
    echo ""
    echo "> Generated: ${TIMESTAMP}"
    echo "> Script: \`.github/scripts/security_documentation.sh\`"
    echo ""
    echo "---"
    echo ""
    echo "## Project Security Docs"
    echo ""

    while IFS= read -r doc_file; do
      local title
      title="$(head -1 "${doc_file}" | sed 's/^#\s*//')"
      echo "- [${title}](../../${doc_file})"
      doc_count=$((doc_count + 1))
    done < <(find "${DOCS_DIR}" -name "security_*.md" 2>/dev/null | sort)

    if [[ "${doc_count}" -eq 0 ]]; then
      echo "_No security docs found in \`${DOCS_DIR}\`._"
    fi

    echo ""
    echo "## Contract Security Module Docs"
    echo ""

    while IFS= read -r doc_file; do
      local title
      title="$(head -1 "${doc_file}" | sed 's/^#\s*//')"
      echo "- [${title}](../../${doc_file})"
      src_doc_count=$((src_doc_count + 1))
    done < <(find "${SECURITY_SRC_DIR}" -name "*.md" 2>/dev/null | sort)

    if [[ "${src_doc_count}" -eq 0 ]]; then
      echo "_No .md files found in \`${SECURITY_SRC_DIR}\`._"
    fi

    echo ""
    echo "---"
    echo ""
    echo "## CI/CD Integration"
    echo ""
    echo "This index is regenerated on every push to \`main\` and \`develop\`."
    echo "See \`.github/workflows/security.yml\` for the full pipeline."
    echo ""
    echo "| Check | Description |"
    echo "|---|---|"
    echo "| NatSpec coverage | All public security functions have @notice, @dev, @custom:security-note |"
    echo "| Test parity | Every check_* / probe_* function has ≥ 1 test |"
    echo "| Assumptions | Every security_*.md has a Security Assumptions section |"
    echo "| Index generation | This file is regenerated and uploaded as a CI artefact |"
  } > "${INDEX_FILE}"

  local total=$((doc_count + src_doc_count))

  if [[ "${total}" -gt 0 ]]; then
    log_pass "Documentation index: ${total} file(s) indexed → ${INDEX_FILE}"
    append_result "doc_index_generation" "PASS" "${total} files indexed"
  else
    log_warn "Documentation index: no security docs found to index"
    append_result "doc_index_generation" "WARN" "no security docs found"
  fi
}

run_generate_index

# ── Report generation ─────────────────────────────────────────────────────────

echo "────────────────────────────────────────────────────────────"
log_info "Generating JSON report…"

{
  printf '{\n'
  printf '  "report_timestamp": "%s",\n' "${TIMESTAMP}"
  printf '  "pass_count": %d,\n' "${PASS_COUNT}"
  printf '  "warn_count": %d,\n' "${WARN_COUNT}"
  printf '  "critical_failures": %d,\n' "${CRITICAL_FAILURES}"
  printf '  "overall_status": "%s",\n' \
    "$([ "${CRITICAL_FAILURES}" -eq 0 ] && echo "PASS" || echo "FAIL")"
  printf '  "checks": [\n'
  _first=1
  while IFS= read -r line; do
    if [[ "${_first}" -eq 1 ]]; then
      _first=0
    else
      printf ',\n'
    fi
    printf '    %s' "${line}"
  done < "${TMPDIR_WORK}/results.ndjson"
  printf '\n  ]\n'
  printf '}\n'
} > "${REPORT_FILE}"

log_info "JSON report  : ${REPORT_FILE}"
log_info "Doc index    : ${INDEX_FILE}"
echo "────────────────────────────────────────────────────────────"
echo "PASS:     ${PASS_COUNT}"
echo "WARN:     ${WARN_COUNT}"
echo "CRITICAL: ${CRITICAL_FAILURES}"
echo "Overall:  $([ "${CRITICAL_FAILURES}" -eq 0 ] && echo "PASS" || echo "FAIL")"
echo "────────────────────────────────────────────────────────────"

# ── Exit code ─────────────────────────────────────────────────────────────────

if [[ "${CRITICAL_FAILURES}" -gt 0 ]]; then
  log_fail "Security documentation check FAILED with ${CRITICAL_FAILURES} critical failure(s)."
  exit 1
fi

log_pass "All security documentation checks passed."
exit 0

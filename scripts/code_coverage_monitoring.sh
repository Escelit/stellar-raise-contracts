#!/usr/bin/env bash
# @title   code_coverage_monitoring.sh
# @notice  Runs Jest with coverage, records metrics for CI/CD, and optionally
#          enforces a minimum global line (and related) threshold.
# @dev     Exit code policy:
#            0 = success (tests passed; policy satisfied or reporting-only)
#            1 = tests failed OR coverage below enforced threshold OR invalid input
#            2 = required tooling missing (Node.js / npm)
# @custom:security-note  Threshold and paths are validated as strict integers and
#          repo-relative paths only — no arbitrary shell injection from env.

set -euo pipefail

# ── Location ─────────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
readonly DEFAULT_SUMMARY_REL="coverage/coverage-summary.json"
readonly JEST_CONFIG="$REPO_ROOT/jest.config.json"

# ── Colours ───────────────────────────────────────────────────────────────────

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

cov_pass() { echo -e "${GREEN}[PASS]${NC} $*"; }
cov_fail() { echo -e "${RED}[FAIL]${NC} $*"; }
cov_warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
cov_section() { echo -e "\n── $* ──────────────────────────────────────────────"; }

# ── NatSpec-style helpers (tested via code_coverage_monitoring.test.sh) ───────

# @notice  Returns 0 if the argument is an integer percentage in 0..100.
# @dev     Rejects empty strings, signs, decimals, and non-digits to avoid
#          injection when the value is interpolated into JSON for Jest.
# @param $1  candidate minimum percent string
validate_min_percent() {
    local v="${1:-}"
    [[ "$v" =~ ^[0-9]+$ ]] || return 1
    [ "$v" -ge 0 ] && [ "$v" -le 100 ]
}

# @notice  Builds a Jest --coverageThreshold JSON object with identical gates
#          for lines, statements, functions, and branches.
# @dev     Caller must pass a value that already passed validate_min_percent.
# @param $1  minimum percent (digits only)
build_coverage_threshold_json() {
    local min="$1"
    printf '{"global":{"lines":%s,"statements":%s,"functions":%s,"branches":%s}}' \
        "$min" "$min" "$min" "$min"
}

# @notice  Ensures a repo-relative subpath contains no path traversal.
# @param $1  path relative to repo root (e.g. coverage/coverage-summary.json)
is_safe_repo_relative_path() {
    local p="${1:-}"
    [[ -n "$p" ]] || return 1
    case "$p" in
        *..*) return 1 ;;
        /*) return 1 ;;
    esac
    return 0
}

# @notice  Reads aggregate coverage percents from Jest json-summary output.
# @dev     Uses Node (already required for Jest) — avoids trusting shell eval.
# @param $1  absolute path to coverage-summary.json
# @stdout  lines branches functions statements (one per line, numeric)
parse_coverage_summary_metrics() {
    local file="$1"
    COVERAGE_SUMMARY_PATH="$file" node <<'NODE'
const fs = require('fs');
const p = process.env.COVERAGE_SUMMARY_PATH;
if (!p || !fs.existsSync(p)) {
  console.error('coverage summary missing:', p || '');
  process.exit(1);
}
const j = JSON.parse(fs.readFileSync(p, 'utf8'));
const t = j.total || {};
const pct = (k) => (t[k] && typeof t[k].pct === 'number' ? t[k].pct : NaN);
const out = ['lines', 'branches', 'functions', 'statements'].map(pct);
if (out.some((n) => Number.isNaN(n))) {
  console.error('invalid coverage summary shape');
  process.exit(1);
}
console.log(out.join('\n'));
NODE
}

# ── Tooling ───────────────────────────────────────────────────────────────────

# @notice  Verifies Node.js and npx are available.
# @custom:security-note  Fails closed when tooling is absent so CI never reports
#          a false green for coverage.
check_node_tooling() {
    cov_section "Tool presence"
    local missing=0
    if ! command -v node &>/dev/null; then
        echo -e "${RED}[MISSING]${NC} node — install Node.js LTS"
        missing=$(( missing + 1 ))
    else
        cov_pass "node found ($(node --version))"
    fi
    if ! command -v npx &>/dev/null; then
        echo -e "${RED}[MISSING]${NC} npx — reinstall Node.js (npx ships with npm)"
        missing=$(( missing + 1 ))
    else
        cov_pass "npx found"
    fi
    if [ ! -f "$JEST_CONFIG" ]; then
        echo -e "${RED}[MISSING]${NC} jest.config.json at repo root"
        missing=$(( missing + 1 ))
    else
        cov_pass "jest.config.json present"
    fi
    if [ "$missing" -gt 0 ]; then
        echo ""
        echo "ERROR: $missing required item(s) missing."
        exit 2
    fi
}

# ── Jest execution ────────────────────────────────────────────────────────────

# @notice  Runs the Jest suite with coverage reporters suitable for CI.
# @param $1  optional coverage threshold JSON string, or empty to skip threshold
run_jest_coverage() {
    local threshold_json="${1:-}"
    cd "$REPO_ROOT"
    cov_section "Jest coverage"
    local -a cmd=(npx jest --config "$JEST_CONFIG" --ci --coverage
        --coverageReporters=json-summary
        --coverageReporters=text-summary)
    if [ -n "$threshold_json" ]; then
        cmd+=(--coverageThreshold="$threshold_json")
    fi
    if "${cmd[@]}"; then
        cov_pass "jest completed successfully"
    else
        cov_fail "jest reported failures or coverage below threshold"
        return 1
    fi
}

# @notice  Prints human-readable coverage totals and optional policy result.
# @param $1  minimum percent (for comparison messaging)
# @param $2  enforce flag: "true" to exit 1 when lines pct < min
report_and_enforce() {
    local min="$1"
    local enforce="$2"
    local summary_path="$REPO_ROOT/$DEFAULT_SUMMARY_REL"

    cov_section "Coverage summary"
    if ! is_safe_repo_relative_path "$DEFAULT_SUMMARY_REL"; then
        cov_fail "internal error: invalid default summary path"
        exit 1
    fi
    if [ ! -f "$summary_path" ]; then
        cov_fail "coverage summary not found at $summary_path (did Jest run?)"
        exit 1
    fi

    local metrics
    metrics=$(parse_coverage_summary_metrics "$summary_path")
    local lines branches functions statements
    lines=$(echo "$metrics" | sed -n '1p')
    branches=$(echo "$metrics" | sed -n '2p')
    functions=$(echo "$metrics" | sed -n '3p')
    statements=$(echo "$metrics" | sed -n '4p')

    echo "  Lines:       ${lines}%"
    echo "  Branches:    ${branches}%"
    echo "  Functions:   ${functions}%"
    echo "  Statements:  ${statements}%"
    echo "  Policy min:   ${min}% (lines gate when enforcing)"

    local below=0
    # Policy compares primary gate on lines (matches Jest global.lines threshold).
    if awk -v n="$lines" -v m="$min" 'BEGIN{ if (n+0 < m+0) exit 0; exit 1 }'; then
        below=1
    fi

    if [ "$below" -eq 1 ]; then
        if [ "$enforce" = "true" ]; then
            cov_fail "lines coverage ${lines}% is below required ${min}%"
            exit 1
        else
            cov_warn "lines coverage ${lines}% is below target ${min}% (reporting-only mode)"
        fi
    else
        cov_pass "lines coverage meets target ${min}%"
    fi
}

# ── CLI ───────────────────────────────────────────────────────────────────────

print_help() {
    cat <<'EOF'
code_coverage_monitoring.sh — Jest coverage for CI/CD

Usage:
  scripts/code_coverage_monitoring.sh [options]

Options:
  --help              Show this help
  --min-pct N         Minimum lines threshold percent (0–100). Default: env
                      CODE_COVERAGE_MIN_LINES or 95.
  --enforce           Exit 1 if lines % is below --min-pct (after Jest).
                      When set, also passes the same gate to Jest via
                      --coverageThreshold (fail fast).
  --no-jest           Skip Jest; only read existing coverage/coverage-summary.json
                      and apply --enforce / reporting (for debugging).

Environment:
  CODE_COVERAGE_MIN_LINES   Default minimum percent (digits only, 0–100).
  CODE_COVERAGE_ENFORCE   If "true", same as --enforce when no flag passed.

Exit codes:
  0 success
  1 test or policy failure
  2 missing Node / npx / jest.config.json

Security:
  Threshold values must be plain integers. Paths are fixed under the repo root.
EOF
}

main() {
    local min_pct="${CODE_COVERAGE_MIN_LINES:-95}"
    local enforce="${CODE_COVERAGE_ENFORCE:-false}"
    local skip_jest=false

    while [ $# -gt 0 ]; do
        case "$1" in
            --help|-h)
                print_help
                exit 0
                ;;
            --min-pct)
                min_pct="${2:-}"
                shift 2 || { cov_fail "--min-pct requires a value"; exit 1; }
                ;;
            --enforce)
                enforce=true
                shift
                ;;
            --no-jest)
                skip_jest=true
                shift
                ;;
            *)
                cov_fail "unknown option: $1"
                print_help
                exit 1
                ;;
        esac
    done

    if ! validate_min_percent "$min_pct"; then
        cov_fail "invalid --min-pct / CODE_COVERAGE_MIN_LINES (use integer 0–100)"
        exit 1
    fi

    echo "Code Coverage Monitoring"
    echo "$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    echo "Repository: $REPO_ROOT"
    echo "Min lines %: $min_pct | Enforce: $enforce"

    check_node_tooling

    local threshold_json=""
    if [ "$enforce" = "true" ]; then
        threshold_json="$(build_coverage_threshold_json "$min_pct")"
    fi

    if [ "$skip_jest" != "true" ]; then
        if ! run_jest_coverage "$threshold_json"; then
            exit 1
        fi
    else
        cov_section "Jest coverage"
        cov_warn "--no-jest: using existing coverage artifacts only"
    fi

    # When Jest enforced threshold, lines are already OK; reporting still validates file.
    if [ "$enforce" = "true" ] && [ "$skip_jest" != "true" ] && [ -n "$threshold_json" ]; then
        report_and_enforce "$min_pct" false
    else
        report_and_enforce "$min_pct" "$([ "$enforce" = "true" ] && echo true || echo false)"
    fi

    cov_pass "coverage monitoring finished"
}

# @notice  Run main only when executed, not when sourced for unit tests.
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi

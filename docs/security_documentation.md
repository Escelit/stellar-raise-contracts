# Security Documentation

> **Script:** `.github/scripts/security_documentation.sh`
> **Tests:** `.github/scripts/security_documentation.test.sh`
> **Issue:** add-automated-security-documentation-for-cicd

---

## Overview

`security_documentation.sh` automates security documentation validation and
generation for the Stellar Raise crowdfund project as part of the CI/CD
pipeline. It enforces four quality gates on every push and pull request:

| #   | Check                                       | Failure Mode                |
| --- | ------------------------------------------- | --------------------------- |
| 1   | NatSpec doc-comment coverage                | CRITICAL — blocks build     |
| 2   | Test coverage parity (check*\* / probe*\*)  | CRITICAL — blocks build     |
| 3   | Security assumptions documented in Markdown | CRITICAL — blocks build     |
| 4   | Documentation index generation              | WARN — does not block build |

---

## Files

| File                                             | Purpose                      |
| ------------------------------------------------ | ---------------------------- |
| `.github/scripts/security_documentation.sh`      | Main documentation CI script |
| `.github/scripts/security_documentation.test.sh` | Test suite (≥ 95 % coverage) |
| `docs/security_documentation.md`                 | This document                |

---

## Checks Performed

### 1. NatSpec Doc-Comment Coverage

Every non-test `.rs` file under `contracts/security/src` must contain all
three NatSpec-style annotations:

- `@notice` — one-line description of what the function does
- `@dev` — technical implementation detail or storage key reference
- `@custom:security-note` — the specific exploit or threat this function prevents

Files matching `*.test.rs` or `*_test.rs` are excluded from this check.

**Example compliant function:**

```rust
/// @notice  Verifies that `total_raised` equals the sum of all contributions.
/// @dev     Core accounting invariant. Discrepancy indicates arithmetic bug
///          or storage corruption.
/// @custom:security-note  Must hold after every contribute, refund, and
///          withdraw call.
pub fn check_total_raised_equals_sum(
    total_raised: i128,
    contributions: &[i128],
) -> InvariantResult {
    // ...
}
```

### 2. Test Coverage Parity

Every `pub fn check_*` and `pub fn probe_*` function in security source files
must have at least one corresponding test in a `*.test.rs` or `*_test.rs` file.
The check extracts function names via grep and cross-references them against all
test files in the same directory.

Target: ≥ 95 % function-level coverage. Any gap is a CRITICAL failure.

### 3. Security Assumptions Documented

Every `security_*.md` file under `docs/` must contain a section heading
matching `## Security` (case-insensitive). This enforces that every security
module documents its threat model assumptions for future maintainers.

### 4. Documentation Index Generation

The script generates `security-docs/security_documentation_index.md` — a
consolidated index of all security documentation files — and uploads it as a
CI artefact. This is a WARN-only check; a missing index does not block the build.

---

## Usage

```bash
# Run with defaults (from repo root)
bash .github/scripts/security_documentation.sh

# Override source and output directories
SECURITY_SRC_DIR=contracts/security/src \
DOCS_OUTPUT_DIR=./security-docs \
DOCS_DIR=docs \
  bash .github/scripts/security_documentation.sh

# Skip NatSpec validation (e.g. draft PRs)
SKIP_DOC_VALIDATION=1 bash .github/scripts/security_documentation.sh
```

---

## Environment Variables

| Variable              | Default                  | Description                                   |
| --------------------- | ------------------------ | --------------------------------------------- |
| `DOCS_OUTPUT_DIR`     | `./security-docs`        | Directory for generated reports and index     |
| `SECURITY_SRC_DIR`    | `contracts/security/src` | Root of security Rust source files            |
| `DOCS_DIR`            | `docs`                   | Project documentation directory               |
| `SKIP_DOC_VALIDATION` | `0`                      | Set to `1` to skip NatSpec check (draft PRs)  |
| `CI`                  | _(unset)_                | Set by GitHub Actions; disables colour output |

---

## Report Output

The script writes two files to `DOCS_OUTPUT_DIR`:

- `security_documentation_report_<timestamp>.json` — machine-readable JSON
- `security_documentation_index.md` — consolidated documentation index

### JSON Report Structure

```json
{
  "report_timestamp": "2026-01-01T00:00:00Z",
  "pass_count": 3,
  "warn_count": 1,
  "critical_failures": 0,
  "overall_status": "PASS",
  "checks": [
    {
      "check": "natspec_coverage",
      "status": "PASS",
      "detail": "all 2 files have required annotations",
      "timestamp": "..."
    },
    {
      "check": "test_coverage_parity",
      "status": "PASS",
      "detail": "12/12 functions have tests (100%)",
      "timestamp": "..."
    },
    {
      "check": "security_assumptions",
      "status": "PASS",
      "detail": "all 5 docs have security assumptions",
      "timestamp": "..."
    },
    {
      "check": "doc_index_generation",
      "status": "WARN",
      "detail": "no security docs found",
      "timestamp": "..."
    }
  ]
}
```

---

## CI Integration

Add the following step to `.github/workflows/security.yml`:

```yaml
- name: Automated security documentation
  run: bash .github/scripts/security_documentation.sh

- name: Run security documentation tests
  run: bash .github/scripts/security_documentation.test.sh

- name: Upload security documentation artefacts
  if: always()
  uses: actions/upload-artifact@v4
  with:
    name: security-docs
    path: security-docs/
```

The step fails (exit code 1) when any CRITICAL check fails.

---

## Security Assumptions

1. `set -euo pipefail` ensures the script exits immediately on any unhandled
   error, preventing silent partial execution.

2. An `EXIT` trap removes all temporary files, preventing credential or
   intermediate data leakage via temp files.

3. No user-supplied input is passed to `eval` or unquoted shell expansions.
   All variable expansions use double-quotes.

4. NatSpec validation uses `grep -q` with fixed string patterns — no dynamic
   pattern construction from external input.

5. Test parity extraction uses `grep -oE` with a fixed regex anchored to
   `pub fn (check_|probe_)` — it cannot be influenced by file content to
   execute arbitrary commands.

6. The documentation index is generated from file paths discovered by `find`
   with explicit `-name` patterns, not from user-controlled input.

7. WARN-only failures (missing directories, no docs found) do not block the
   build. Only CRITICAL failures (annotation gaps, untested functions, missing
   assumption sections) exit with code 1.

---

## How to Add a New Security Check

1. Implement the check function in `contracts/security/src/` with all three
   NatSpec annotations (`@notice`, `@dev`, `@custom:security-note`).

2. Add at least one happy-path and one failure-path test in the corresponding
   `*.test.rs` file.

3. Add a `## Security Assumptions` section to the module's `.md` file in
   `docs/` if one does not already exist.

4. Run the documentation CI script locally to confirm all checks pass:

   ```bash
   bash .github/scripts/security_documentation.sh
   bash .github/scripts/security_documentation.test.sh
   ```

5. Commit with the conventional commit format:

   ```
   feat: implement <feature-name> with tests and docs
   ```

---

## Running the Tests

```bash
# Run the full test suite
bash .github/scripts/security_documentation.test.sh

# Run with verbose output
bash -x .github/scripts/security_documentation.test.sh 2>&1 | head -100
```

Expected output: all tests pass, exit code 0.

Test categories:

- Script structure: `set -euo pipefail`, EXIT trap, no `eval`, `mktemp`
- `append_result`: JSON field format, quote escaping
- NatSpec validation: pass, fail, skip, empty-dir, test-file exclusion
- Test coverage parity: pass, fail, no-functions, missing-dir
- Security assumptions: pass, fail, no-docs, missing-dir
- Documentation index: file created, content sections, JSON report
- Counter logic: all-pass exit 0, critical-fail exit 1, warn-only exit 0
- Environment variable defaults: all five variables
- Integration dry-run: skipped validation, missing dirs, report header

# Code Coverage Monitoring

> **Script:** `scripts/code_coverage_monitoring.sh`  
> **Tests:** `scripts/code_coverage_monitoring.test.sh`  
> **Related workflow:** `.github/workflows/rust_ci.yml` (Frontend UI Tests job)

---

## Purpose

Runs **Jest** with **json-summary** and **text-summary** coverage reporters, prints aggregate percentages to CI logs, and optionally **enforces** a minimum global threshold (lines, statements, functions, and branches use the same gate when enforcing).

Designed for **quality assurance** and **CI/CD**: logs stay scannable (`[PASS]` / `[FAIL]` / `[WARN]`), and exit codes are suitable for branch protection.

---

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Jest succeeded and policy is satisfied (or reporting-only with no hard failure). |
| `1` | Invalid CLI input, unknown option, Jest failure, or enforced coverage below minimum. |
| `2` | Missing **Node.js**, **npx**, or `jest.config.json` at the repository root. |

---

## Usage

From the repository root (after `npm ci`):

```bash
# Reporting only: run tests + coverage, warn if below default target (95%) but exit 0 if Jest passed
bash scripts/code_coverage_monitoring.sh

# Enforce minimum 95% on all global metrics (Jest + post-run lines check)
bash scripts/code_coverage_monitoring.sh --enforce --min-pct 95

# Custom target, enforce
bash scripts/code_coverage_monitoring.sh --enforce --min-pct 80

# Re-read existing coverage/coverage-summary.json without re-running Jest
bash scripts/code_coverage_monitoring.sh --no-jest --min-pct 90 --enforce
```

### Environment variables

| Variable | Description |
|----------|-------------|
| `CODE_COVERAGE_MIN_LINES` | Default for `--min-pct` (digits only, `0`–`100`). |
| `CODE_COVERAGE_ENFORCE` | If `true`, same as `--enforce` when the flag is omitted. |

---

## Security assumptions

1. **Threshold values** must be plain integers (`^[0-9]+$`). Values are injected into Jest’s `--coverageThreshold` JSON via `printf` with a fixed template — never interpolate raw user shell into the command line.
2. **Coverage summary path** is fixed to `coverage/coverage-summary.json` under the repo root (no arbitrary file arguments in v1).
3. **Path traversal** helper rejects `..` and absolute paths for any future extension that accepts relative paths.
4. **Parsing** uses Node’s `JSON.parse` on the summary file — no `eval` and no shell interpretation of file contents.
5. **Fail closed** when Node/npx or `jest.config.json` is missing (exit `2`), so CI does not silently skip coverage.

---

## CI integration

Example step (adjust `--min-pct` to match team policy):

```yaml
- name: Install dependencies
  run: npm ci

- name: Coverage monitoring (enforce)
  env:
    CODE_COVERAGE_MIN_LINES: "80"
  run: bash scripts/code_coverage_monitoring.sh --enforce
```

To adopt **gradually**, omit `--enforce` first so the job logs **[WARN]** lines below target without failing the build, then turn on `--enforce` when the codebase meets the bar.

The existing **Frontend UI Tests** job can be pointed at this script instead of duplicating `npm run test:coverage`, if you want a single enforced gate (see script’s Jest invocation).

---

## Testing the script

```bash
bash scripts/code_coverage_monitoring.test.sh
```

The suite **sources** the monitoring script to exercise `validate_min_percent`, `build_coverage_threshold_json`, `is_safe_repo_relative_path`, and `parse_coverage_summary_metrics`, plus CLI smoke tests (`--help`, invalid options).

Ensure the monitor script is executable:

```bash
chmod +x scripts/code_coverage_monitoring.sh
```

---

## NatSpec-style comments

The shell script uses block tags aligned with other Stellar Raise automation:

- `@title`, `@notice`, `@dev`, `@param`, `@custom:security-note`

---

## Version

Maintained with the Stellar Raise contracts / frontend monorepo; update this doc when thresholds or reporters change.

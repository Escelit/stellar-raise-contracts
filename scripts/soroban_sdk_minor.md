# soroban_sdk_minor — SDK v22 Migration Auditor (Scripts)

Script-layer utility for detecting deprecated Soroban SDK v21 patterns and
validating that contracts have migrated to v22.

## What Changed in v22

| Area | Deprecated (v21) | Required (v22) |
|---|---|---|
| Test registration | `env.register_contract(None, Contract)` | `env.register(Contract, ())` |
| Storage keys | `Symbol::new(&env, "key")` | `#[contracttype]` enum keys |
| Auth pattern | Manual checks | `address.require_auth()` |

## Files

| File | Purpose |
|---|---|
| `scripts/soroban_sdk_minor.rs` | Pure audit functions — no Soroban runtime |
| `scripts/soroban_sdk_minor.test.rs` | Self-contained test suite (49 cases) |
| `scripts/soroban_sdk_minor.md` | This document |

## Contract API

### Types

```rust
pub struct ScanResult {
    pub file: String,
    pub clean: bool,
    pub findings: Vec<(usize, String)>,  // (line_number, description)
}
```

### Functions

| Function | Description |
|---|---|
| `parse_semver(version)` | Parses semver string → `Option<(u64,u64,u64)>` |
| `is_version_gte(version, min)` | Returns `true` if `version >= min` |
| `is_sdk_v22_compatible(sdk_version)` | Returns `true` if version ≥ `22.0.0` |
| `scan_source(file, source)` | Scans source text for deprecated v21 patterns |
| `scan_all(sources)` | Scans a map of file→source; results sorted by filename |
| `dirty_results(results)` | Filters to only results with findings |

### Constants

| Constant | Value | Description |
|---|---|---|
| `MIN_SDK_VERSION` | `"22.0.0"` | Minimum SDK version for v22 compatibility |
| `DEPRECATED_PATTERNS` | see source | `(pattern, description)` pairs checked per line |

## Deprecated Patterns Detected

| Pattern | Replacement |
|---|---|
| `register_contract(` | `env.register(Contract, ())` |
| `Symbol::new(` | `Symbol::short` or `#[contracttype]` enum key |

## Usage in CI

Add to your pre-commit hook or CI script:

```bash
# Scan all Rust source files for deprecated v21 patterns
# (illustrative — adapt to your script runner)
for f in contracts/**/*.rs; do
  grep -n "register_contract(\|Symbol::new(" "$f" && echo "DEPRECATED: $f" && exit 1
done
echo "SDK v22 migration check passed."
```

Or use the Rust functions directly in a build script:

```rust
use soroban_sdk_minor::{scan_all, dirty_results};
use std::collections::HashMap;

let mut sources = HashMap::new();
sources.insert("lib.rs".to_string(), std::fs::read_to_string("src/lib.rs").unwrap());

let results = scan_all(&sources);
let dirty = dirty_results(&results);
if !dirty.is_empty() {
    for r in &dirty {
        for (line, msg) in &r.findings {
            eprintln!("{}:{}: {}", r.file, line, msg);
        }
    }
    std::process::exit(1);
}
```

## Security Assumptions

1. Pattern detection is heuristic (substring match per line). A clean scan
   does not guarantee absence of all anti-patterns — use `cargo clippy` and
   code review as complementary checks.
2. Version comparison uses semver tuple ordering with no external crates,
   eliminating supply-chain risk in the script layer.
3. `scan_all` sorts results by filename for deterministic CI log output,
   making diffs easy to review.

## Running Tests

```bash
# Requires Rust stable (no Cargo project needed)
rustc --test scripts/soroban_sdk_minor.test.rs -o /tmp/soroban_sdk_minor_tests && /tmp/soroban_sdk_minor_tests
```

Expected: all tests pass, ≥ 95% coverage.

## Commit Reference

```
feat: implement deprecate-old-logic-in-soroban-sdk-minor-version-bump-for-scripts with tests and docs
```

- Added `scripts/soroban_sdk_minor.rs` — pure script-layer auditor for SDK v22 migration
- Added `scripts/soroban_sdk_minor.test.rs` — 49 self-contained test cases
- Added `scripts/soroban_sdk_minor.md` — this document

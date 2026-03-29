# security_compliance_validation

## Overview

`security_compliance_validation` exposes read-only on-chain validation helpers
that automated testing infrastructure can call to confirm the crowdfund
contract remains configured securely before executing test scenarios.

This module is designed for CI pipelines, local test suites, and governance
pre-flight validation.

---

## Security Assumptions

1. **Read-only** — No storage writes.
2. **Permissionless** — No auth required.
3. **Deterministic** — Same ledger state returns identical results.
4. **Bounded execution** — Only a fixed set of checks runs.

---

## Public API

| Function | Purpose |
|---|---|
| `validate_admin_present` | Ensures `DataKey::Admin` is initialized. |
| `validate_status_valid` | Ensures campaign `Status` is present. |
| `validate_goal_compliance` | Ensures the campaign goal is >= 1. |
| `validate_minimum_contribution` | Ensures minimum contribution is >= 1. |
| `validate_deadline_in_future` | Ensures an active campaign deadline is still in the future. |
| `validate_platform_fee_cap` | Ensures platform fee does not exceed 10%. |
| `audit_all_validations` | Runs all checks and returns a summary report. |

---

## Diagnostic Output

`ValidationReport` contains:

- `passed` — number of checks that succeeded.
- `failed` — number of checks that failed.
- `all_valid` — `true` when `failed == 0`.

`ValidationResult` values are either `Valid` or `Invalid(&'static str)`.

---

## Usage Example

```rust
let report = security_compliance_validation::audit_all_validations(&env);
if !report.all_valid {
    panic!("validation failed: {}", report.failed);
}
```

---

## Notes for Testing

- The module is intentionally isolated from mutable contract operations.
- It is safe to call during simulation-based CI checks.
- Failures are surfaced as static strings to simplify automated parsing.

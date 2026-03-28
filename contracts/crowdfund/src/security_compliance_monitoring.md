# Security Compliance Monitoring

## Overview

This module provides **runtime security compliance monitoring** for the crowdfund contract. It performs **read-only invariant checks** covering:

| Category | Checks Performed |
|----------|------------------|
| **Authorization** | Creator/admin consistency |
| **State Bounds** | Contributors ≤128, roadmap ≤32, strings |
| **Arithmetic** | `total_raised ≤ goal * 2`, no negatives |
| **Status** | No `Active` post-deadline |
| **Config** | Platform `fee_bps ≤ 10,000` |

**Key Functions**:
- `run_full_audit(env)` → `ComplianceReport` (pass/fail + violations)
- `compliance_status(env)` → `bool` (fast view)
- `get_compliance_metrics(env)` → `AuditMetrics` (dashboard-ready)

## Error Codes

| Code | Variant | Trigger |
|------|---------|---------|
| 100 | `UNAUTHORIZED_CREATOR` | Admin ≠ creator without delegation |
| 101 | `CONTRIBUTOR_LIMIT_EXCEEDED` | > MAX_CONTRIBUTORS (128) |
| 102 | `ROADMAP_LIMIT_EXCEEDED` | > MAX_ROADMAP_ITEMS (32) |
| 103 | `INVALID_STATUS` | `Active` after deadline |
| 104 | `ARITHMETIC_ANOMALY` | `total_raised > goal * 2` |
| 105 | `NEGATIVE_CONTRIBUTION` | Negative amount in storage |
| 106 | `INVALID_PLATFORM_FEE` | `fee_bps > 10,000` |

**Off-chain Usage**:
```rust
let report = client.run_full_audit();
if !report.passed {
    for issue in report.violations {
        println!("{}: {}", issue.code, describe_violation(issue.code));
    }
}
```

## Event Schema (Monitoring)

**Violation Event**:
```
("security", "violation", "code_101") → u32(code)
```

Subscribe to `security` topic for real-time compliance alerts.

## Security Assumptions

✅ **Read-only**: No state mutations; safe for view calls.  
✅ **Gas-bounded**: Contributor scan caps at 50 entries.  
✅ **CEI Compliant**: Checks → report → events (no interactions).  
✅ **Typed Errors**: Numeric codes + descriptions for scripting.  
✅ **Proptest Ready**: Deterministic inputs for fuzzing.  

**Non-Goals**:
- On-chain enforcement (panics block via helpers like `contract_state_size`).
- Historical audit log (use events for off-chain indexing).

## Integration

Call from `lib.rs`:
```rust
pub fn validate_security_state(env: Env) {
    assert!(security_compliance_monitoring::compliance_status(&amp;env));
}
```

**Auditor Note**: All checks mirror `contract_state_size.rs` limits. Changing constants requires proptest re-run.


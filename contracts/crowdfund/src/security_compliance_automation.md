# security_compliance_automation

## Overview

`security_compliance_automation` provides a suite of read-only, on-chain
compliance helpers that automated tooling — CI pipelines, monitoring bots,
governance scripts — can call to verify that the crowdfund contract's security
invariants hold at any point in time.

Every individual check emits a structured `compliance_audit` event, and the
aggregate `audit_all_checks` function emits a `compliance_summary` event.
Off-chain indexers can subscribe to these events to build a tamper-evident
audit log without re-reading raw storage.

---

## Security Assumptions

1. **Read-only** — No function in this module writes to storage.  All checks
   are safe to call from simulation calls that must not alter ledger state.
2. **Permissionless** — No `require_auth()` is needed.  Automated tooling does
   not require a privileged key to run compliance checks.
3. **Deterministic** — Given the same ledger state, every function returns the
   same result.  The only side-effect is event emission.
4. **Overflow-safe** — All arithmetic uses `checked_*` operations.
5. **Bounded iteration** — `audit_all_checks` iterates over a fixed array of
   10 checks (O(1) with respect to contributor count).
6. **Event integrity** — Events are emitted *after* the check result is
   computed, so a panicking host never emits a misleading event.

---

## Constants

| Constant                   | Value | Description                                      |
|----------------------------|-------|--------------------------------------------------|
| `MAX_ALLOWED_FEE_BPS`      | 1 000 | Maximum compliant platform fee (10 %).           |
| `MIN_COMPLIANT_GOAL`       | 1     | Minimum compliant campaign goal (token units).   |
| `MIN_COMPLIANT_CONTRIBUTION` | 1   | Minimum compliant contribution floor.            |
| `MIN_DEADLINE_BUFFER_SECS` | 60    | Minimum deadline buffer used at initialization.  |

---

## Types

### `CheckResult`

```rust
pub enum CheckResult {
    Passed,
    Failed(&'static str),
}
```

- `is_passed() -> bool` — `true` when the check passed.
- `violation() -> &'static str` — violation description, or `""` when passed.

### `ComplianceReport`

```rust
pub struct ComplianceReport {
    pub passed: u32,
    pub failed: u32,
    pub all_passed: bool,
}
```

Returned by `audit_all_checks`.  `all_passed` is `true` iff `failed == 0`.

---

## Individual Check Functions

All functions take `env: &Env` and return `CheckResult`.

| Function                          | Invariant verified                                      |
|-----------------------------------|---------------------------------------------------------|
| `check_admin_initialized`         | `DataKey::Admin` is present in instance storage.        |
| `check_creator_address_set`       | `DataKey::Creator` is present in instance storage.      |
| `check_token_address_set`         | `DataKey::Token` is present in instance storage.        |
| `check_status_valid`              | `DataKey::Status` is present (valid enum variant).      |
| `check_goal_positive`             | `goal >= MIN_COMPLIANT_GOAL` (1).                       |
| `check_min_contribution_positive` | `min_contribution >= MIN_COMPLIANT_CONTRIBUTION` (1).   |
| `check_deadline_in_future`        | `deadline > ledger.timestamp()`.                        |
| `check_total_raised_non_negative` | `total_raised >= 0`.                                    |
| `check_platform_fee_within_limit` | `fee_bps <= MAX_ALLOWED_FEE_BPS` (1 000), or no config. |
| `check_paused_flag_present`       | `DataKey::Paused` is present in instance storage.       |

---

## Aggregate Audit

### `audit_all_checks(env) -> ComplianceReport`

Runs all 10 checks in order of increasing storage cost:

1. Pure / flag checks (no deserialization): admin, creator, token, status.
2. Scalar value checks (single key reads): goal, min_contribution, deadline,
   total_raised.
3. Struct checks (PlatformConfig deserialization): platform_fee.
4. Flag presence check: paused.

Emits one `compliance_audit` event per check and a final `compliance_summary`
event with aggregate counts.

**Events emitted:**

| Topic 0             | Topic 1              | Data                    |
|---------------------|----------------------|-------------------------|
| `compliance_audit`  | `<check_name>`       | `bool` (passed/failed)  |
| `compliance_summary`| —                    | `(u32, u32)` (pass, fail)|

---

## Targeted Audit Helpers

### `audit_initialization(env) -> bool`

Runs only the six initialization checks (admin, creator, token, status, goal,
min_contribution).  Returns `true` when all pass.  Useful in post-deployment
CI smoke tests.

### `audit_financial_integrity(env) -> bool`

Runs only the three financial invariant checks (goal, total_raised,
platform_fee).  Returns `true` when all pass.  Useful for periodic monitoring
bots focused on fund safety.

---

## Utility

### `describe_check_result(result) -> &'static str`

Returns `"PASSED"` or the violation message.  Intended for off-chain tooling
that logs compliance results to stdout or a monitoring dashboard.

---

## Validation Order in `audit_all_checks`

```
1. admin_initialized          — has() check, no deserialization
2. creator_set                — has() check, no deserialization
3. token_set                  — has() check, no deserialization
4. status_valid               — get() + enum deserialization
5. goal_positive              — get() + i128 comparison
6. min_contribution_positive  — get() + i128 comparison
7. deadline_in_future         — get() + u64 comparison with ledger timestamp
8. total_raised_non_negative  — get() + i128 comparison (defaults to 0)
9. platform_fee_within_limit  — get() + struct deserialization (optional)
10. paused_flag_present       — has() check, no deserialization
```

---

## Usage Examples

### Run full audit from a monitoring bot

```rust
let report = security_compliance_automation::audit_all_checks(&env);
if !report.all_passed {
    // alert: report.failed checks did not pass
}
```

### Post-deployment smoke test

```rust
assert!(
    security_compliance_automation::audit_initialization(&env),
    "contract initialization is non-compliant"
);
```

### Check a single invariant

```rust
let result = security_compliance_automation::check_platform_fee_within_limit(&env);
if !result.is_passed() {
    // log: result.violation()
}
```

---

## Security Considerations

- All functions are read-only and permissionless — they cannot be used to
  escalate privileges or mutate state.
- `audit_all_checks` is O(1) with respect to contributor count; it never
  iterates over the contributor list, so it cannot be used to cause
  out-of-gas conditions by inflating the contributor list.
- Event emission happens after the check result is computed.  A host-level
  panic during event emission does not produce a misleading audit record.
- The `deadline_in_future` check is informational for Active campaigns.  A
  campaign whose deadline has passed but whose status is still `Active` is
  not necessarily broken — it simply needs `finalize()` to be called.

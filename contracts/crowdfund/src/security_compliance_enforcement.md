# security_compliance_enforcement

## Overview

**@title** SecurityComplianceEnforcement — Active enforcement of security compliance invariants for the crowdfund contract.

**@notice** Extends the read-only `security_compliance_automation` module with state-mutating enforcement capabilities. Where the automation module _observes_ and _reports_ compliance violations, this module _prevents_ and _remediates_ them. It provides:

- A zero-trust compliance gate (`assert_compliant`) that any caller can invoke before a state-changing operation.
- An automatic circuit-breaker (`enforce_compliance`) that freezes the contract when critical invariants are violated.
- Admin-controlled emergency halt/lift functions.
- Targeted remediation functions for platform fee and minimum contribution misconfigurations.
- A read-only status snapshot (`compliance_status`) for off-chain monitoring bots.

**@dev** All state-mutating functions require Soroban `require_auth()` plus a role check against `DataKey::Admin`, `DataKey::DefaultAdmin`, or `DataKey::Pauser`. Read-only functions are permissionless. Check logic is fully delegated to `security_compliance_automation` — no duplication of check logic exists in this module.

---

## Security Assumptions

1. **Authorization model** — State-mutating functions enforce authorization via `caller.require_auth()` (Soroban host-level auth) followed by a role check against instance storage keys (`DataKey::Admin`, `DataKey::DefaultAdmin`, `DataKey::Pauser`). Permissionless functions (`assert_compliant`, `enforce_compliance`, `compliance_status`) never mutate storage and require no auth.

2. **Storage mutation scope** — Only `DataKey::Paused`, `DataKey::PlatformConfig`, and `DataKey::MinContribution` are ever written by this module. No other storage keys are modified.

3. **Event emission ordering** — All enforcement events are emitted _after_ the corresponding storage mutation completes. A host-level panic during event emission therefore cannot leave storage in an inconsistent state.

4. **Overflow safety** — All counter arithmetic uses `checked_add` with `unwrap_or(u32::MAX)` saturation, matching the pattern established in the automation module.

5. **Reentrancy** — Soroban's single-threaded execution model prevents reentrancy. No cross-contract calls are made by this module.

---

## Constants

### `MAX_ALLOWED_FEE_BPS`

Re-exported from `security_compliance_automation`. The maximum allowed platform fee expressed in basis points.

- **Value:** `1_000` (= 10 %)
- **Used by:** `remediate_platform_fee` for upper-bound validation of `new_fee_bps`.

### `MIN_COMPLIANT_CONTRIBUTION`

Re-exported from `security_compliance_automation`. The minimum compliant contribution floor.

- **Value:** `1` (one token unit)
- **Used by:** `remediate_min_contribution` for lower-bound validation of `new_min`.

---

## Types

### `EnforcementError`

**@title** EnforcementError

**@notice** All error variants that can be returned by enforcement functions. Each variant maps to a stable `u32` discriminant so that error codes remain consistent across contract upgrades. Declared with `#[contracterror]` and `#[repr(u32)]`.

| Variant                        | Code | Description                                                                                                                          |
| ------------------------------ | ---- | ------------------------------------------------------------------------------------------------------------------------------------ |
| `CriticalViolationDetected`    | `1`  | One or more Critical_Violations were detected (admin absent, token absent, status invalid, or `total_raised` negative).              |
| `NonCriticalViolationsPresent` | `2`  | Only Non_Critical_Violations were detected (e.g. deadline passed, fee too high, paused flag absent). No Emergency_Halt is triggered. |
| `RemediationValueInvalid`      | `3`  | The supplied remediation value is out of the compliant range (e.g. `new_fee_bps > MAX_ALLOWED_FEE_BPS` or `new_min < 1`).            |
| `NoPlatformConfigToRemediate`  | `4`  | `remediate_platform_fee` was called but no `PlatformConfig` is currently stored in instance storage.                                 |
| `Unauthorized`                 | `5`  | Supplementary unauthorized variant. Primary authorization uses `require_auth` panic; this variant is reserved for future use.        |

---

### `EnforcementStatus`

**@title** EnforcementStatus

**@notice** A read-only snapshot of the contract's current compliance and enforcement state, returned by `compliance_status`. Intended for off-chain monitoring bots and dashboards. Declared with `#[contracttype]`.

| Field                          | Type   | Description                                                                                                                                                              |
| ------------------------------ | ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `is_halted`                    | `bool` | `true` when `DataKey::Paused` is set to `true` in instance storage, indicating the contract is in an Emergency_Halt state.                                               |
| `has_critical_violations`      | `bool` | `true` when at least one Critical_Violation check fails (`check_admin_initialized`, `check_token_address_set`, `check_status_valid`, `check_total_raised_non_negative`). |
| `has_non_critical_violations`  | `bool` | `true` when at least one Non_Critical_Violation check fails (any check not listed under Critical_Violation).                                                             |
| `critical_violation_count`     | `u32`  | Count of failing Critical_Violation checks (0–4).                                                                                                                        |
| `non_critical_violation_count` | `u32`  | Count of failing Non_Critical_Violation checks (0–6).                                                                                                                    |

---

## Public Functions

### `assert_compliant`

**@title** assert_compliant

**@notice** Zero-trust compliance gate. Call this before any state-changing operation to verify that all critical invariants hold.

**@dev** Runs the four critical checks from the automation module: `check_admin_initialized`, `check_token_address_set`, `check_status_valid`, and `check_total_raised_non_negative`. Returns `Ok(())` when all pass. On the first failure, emits a `"gate_blocked"` enforcement event with the failing check name as detail, then returns `Err(CriticalViolationDetected)`. Storage is never mutated by this function.

**@security** No `require_auth` is called — this function is intentionally permissionless so any caller (including automated tooling) can verify compliance without a privileged key. It only reads storage and emits events; it never mutates state.

**@param** `env: &Env` — The Soroban environment.

**@return** `Ok(())` when all critical checks pass; `Err(EnforcementError::CriticalViolationDetected)` on the first failing critical check.

---

### `enforce_compliance`

**@title** enforce_compliance

**@notice** Full compliance audit with automatic Emergency_Halt on critical violations. Runs all checks via `audit_all_checks` and acts on the result.

**@dev** Behavior depends on the audit result:

- When `all_passed == true`: returns `Ok(report)` without mutating storage.
- When critical violations exist: sets `DataKey::Paused = true`, emits `"halt_triggered"` with the critical violation count as detail, and returns `Err(CriticalViolationDetected)`.
- When only non-critical violations exist: returns `Err(NonCriticalViolationsPresent)` without mutating storage.

Critical checks: `check_admin_initialized`, `check_token_address_set`, `check_status_valid`, `check_total_raised_non_negative`.

**@security** No `require_auth` — permissionless so automated monitoring bots can trigger the circuit-breaker without a privileged key. The storage write (`DataKey::Paused = true`) is guarded by the critical violation count check and always precedes event emission.

**@param** `env: &Env` — The Soroban environment.

**@return** `Ok(ComplianceReport)` when all checks pass; `Err(EnforcementError::CriticalViolationDetected)` when critical violations are found (contract is halted); `Err(EnforcementError::NonCriticalViolationsPresent)` when only non-critical violations are found.

---

### `compliance_status`

**@title** compliance_status

**@notice** Read-only snapshot of the contract's current compliance and enforcement state. Intended for off-chain monitoring bots and dashboards.

**@dev** Reads `DataKey::Paused` for `is_halted` (defaults to `false` when absent). Runs each critical and non-critical check individually to populate the violation counts. All counter arithmetic uses `checked_add` with `unwrap_or(u32::MAX)` saturation. Storage is never mutated by this function.

**@security** No `require_auth` — permissionless read-only function. Safe to call from any context without a privileged key.

**@param** `env: &Env` — The Soroban environment.

**@return** An `EnforcementStatus` struct with `is_halted`, `has_critical_violations`, `has_non_critical_violations`, `critical_violation_count`, and `non_critical_violation_count`.

---

### `trigger_emergency_halt`

**@title** trigger_emergency_halt

**@notice** Manually freeze the contract by setting `DataKey::Paused = true`. Callable by Admin, DefaultAdmin, or Pauser.

**@dev** Idempotent: succeeds even when the contract is already paused. Emits a `"manual_halt"` enforcement event with the caller address as detail after the storage write completes.

**@security** `caller.require_auth()` is called first to verify the caller's cryptographic signature. A role check against Admin, DefaultAdmin, and Pauser follows. The storage write happens after both checks pass. Panics with a descriptive message if the caller is authenticated but not in an authorized role.

**@param** `env: &Env` — The Soroban environment.

**@param** `caller: Address` — The address triggering the halt. Must be Admin, DefaultAdmin, or Pauser.

---

### `lift_emergency_halt`

**@title** lift_emergency_halt

**@notice** Unfreeze the contract by setting `DataKey::Paused = false`. Callable by Admin or DefaultAdmin only (not Pauser).

**@dev** Emits a `"halt_lifted"` enforcement event with the caller address as detail after the storage write completes.

**@security** `caller.require_auth()` is called first. A role check against Admin and DefaultAdmin follows. Pauser is intentionally excluded — only the two highest-privilege roles can lift a halt. Panics with a descriptive message if the caller is authenticated but not in an authorized role.

**@param** `env: &Env` — The Soroban environment.

**@param** `caller: Address` — The address lifting the halt. Must be Admin or DefaultAdmin.

---

### `remediate_platform_fee`

**@title** remediate_platform_fee

**@notice** Correct an out-of-range platform fee without a full contract upgrade. Updates only the `fee_bps` field of the stored `PlatformConfig`, preserving the `address` field.

**@dev** Validation order (cheapest first):

1. `caller.require_auth()` — cryptographic auth.
2. Role check — Admin or DefaultAdmin only.
3. Range check — `new_fee_bps <= MAX_ALLOWED_FEE_BPS` (1000).
4. Existence check — `PlatformConfig` must be stored.
5. Storage update + event emission.

The `address` field of the existing `PlatformConfig` is preserved unchanged. Only `fee_bps` is overwritten.

**@security** Auth and role checks precede all storage reads/writes. The range check and existence check both return errors without mutating storage, ensuring no partial state is written on failure. Storage write always precedes event emission.

**@param** `env: &Env` — The Soroban environment.

**@param** `caller: Address` — The address performing the remediation. Must be Admin or DefaultAdmin.

**@param** `new_fee_bps: u32` — The corrected fee in basis points. Must satisfy `new_fee_bps <= MAX_ALLOWED_FEE_BPS` (0–1000 inclusive).

**@return** `Ok(())` on success; `Err(EnforcementError::RemediationValueInvalid)` when `new_fee_bps > MAX_ALLOWED_FEE_BPS`; `Err(EnforcementError::NoPlatformConfigToRemediate)` when no `PlatformConfig` is stored.

---

### `remediate_min_contribution`

**@title** remediate_min_contribution

**@notice** Correct a zero or negative minimum contribution without a full contract upgrade. Writes `new_min` to `DataKey::MinContribution` using upsert semantics (creates the key if absent).

**@dev** Validation order:

1. `caller.require_auth()` — cryptographic auth.
2. Role check — Admin or DefaultAdmin only.
3. Range check — `new_min >= MIN_COMPLIANT_CONTRIBUTION` (1).
4. Storage write + event emission.

Uses upsert semantics: the key is created if it does not already exist.

**@security** Auth and role checks precede all storage writes. The range check returns an error without mutating storage on failure. Storage write always precedes event emission.

**@param** `env: &Env` — The Soroban environment.

**@param** `caller: Address` — The address performing the remediation. Must be Admin or DefaultAdmin.

**@param** `new_min: i128` — The corrected minimum contribution. Must satisfy `new_min >= MIN_COMPLIANT_CONTRIBUTION` (>= 1).

**@return** `Ok(())` on success; `Err(EnforcementError::RemediationValueInvalid)` when `new_min < MIN_COMPLIANT_CONTRIBUTION`.

---

## Event Schema

All enforcement events share the same topic and data structure:

```
topics: (Symbol("enforcement_action"), Symbol(<action_name>))
data:   (ledger_timestamp: u64, detail: Val)
```

| Action name             | Emitted by                   | Detail                                   |
| ----------------------- | ---------------------------- | ---------------------------------------- |
| `"gate_blocked"`        | `assert_compliant`           | `Symbol` of the first failing check name |
| `"halt_triggered"`      | `enforce_compliance`         | `u32` count of critical violations       |
| `"manual_halt"`         | `trigger_emergency_halt`     | caller `Address`                         |
| `"halt_lifted"`         | `lift_emergency_halt`        | caller `Address`                         |
| `"remediation_applied"` | `remediate_platform_fee`     | new `fee_bps` as `u32`                   |
| `"remediation_applied"` | `remediate_min_contribution` | new minimum as `i128`                    |

Events are always emitted **after** storage mutations complete. No enforcement event is emitted when a function returns an `Err(...)` variant (except `assert_compliant` and `enforce_compliance` which emit before returning their respective errors).

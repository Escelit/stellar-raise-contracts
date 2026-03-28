# Multi-Signature Execution

> **Module:** `contracts/security/src/multi_signature_execution.rs`
> **Tests:** `contracts/security/src/multi_signature_execution.test.rs`
> **Issue:** implement-smart-contract-multi-signature-execution-for-security

---

## Overview

`multi_signature_execution.rs` enforces M-of-N threshold consensus before any
privileged operation (upgrade, fee change, emergency pause) is allowed to
execute. It provides configuration validation, signer authorisation, approval
tracking with expiry, execution gating, and revocation — all as pure functions
that compose freely in tests without a running contract instance.

Single-key admin operations are the most common attack vector against on-chain
governance. This module ensures no privileged action executes unless the
required number of distinct, authorised signers have approved it within the
expiry window.

---

## Threat Model

| ID   | Threat                                                | Mitigation                                                             |
| ---- | ----------------------------------------------------- | ---------------------------------------------------------------------- |
| M-01 | Single compromised admin key executes privileged op   | `check_execution_threshold` requires M distinct approvals              |
| M-02 | Duplicate signer satisfies multiple approval slots    | `validate_config` rejects duplicate addresses                          |
| M-03 | Threshold set above signer count — permanently locked | `validate_config` rejects `threshold > signers.len()`                  |
| M-04 | Stale approval from compromised key replayed          | `count_valid_approvals` enforces `APPROVAL_EXPIRY_SECONDS` (24 h)      |
| M-05 | Unauthorised address submits approval                 | `validate_approval` checks signer is in registered set                 |
| M-06 | Double-vote — one key approves twice                  | `validate_approval` rejects duplicate signer in approval list          |
| M-07 | Poisoned approval list bypasses threshold             | `check_execution_threshold` verifies every approval's signer           |
| M-08 | No audit trail for approvals or execution             | `emit_approval_event`, `emit_execution_event`, `emit_revocation_event` |
| M-09 | Compromised key cannot be neutralised after approval  | `validate_revocation` allows signer to withdraw consent                |

---

## Public API

```rust
// Configuration
validate_config(config: &MultiSigConfig) -> Result<(), &'static str>

// Signer check
is_authorised_signer(config: &MultiSigConfig, signer: &Address) -> bool

// Approval lifecycle
validate_approval(config, approvals, signer, now) -> Result<(), &'static str>
count_valid_approvals(approvals: &Vec<Approval>, now: u64) -> u32
validate_revocation(config, approvals, signer) -> Result<u32, &'static str>

// Execution gate
check_execution_threshold(config, approvals, now) -> MultiSigResult

// Events
emit_approval_event(env, signer, valid_count, threshold)
emit_execution_event(env, threshold, now)
emit_revocation_event(env, signer)
```

---

## `MultiSigResult` Variants

| Variant               | Meaning                                  |
| --------------------- | ---------------------------------------- |
| `Approved`            | Threshold met — execution is authorised  |
| `Pending { needed }`  | `needed` more valid approvals required   |
| `Rejected { reason }` | Structural violation — execution blocked |

Use `.is_approved()`, `.is_pending()`, `.is_rejected()`, `.reason()` to inspect
results without pattern matching.

---

## Types

```rust
pub struct MultiSigConfig {
    pub signers: Vec<Address>,  // ordered list of authorised signers
    pub threshold: u32,         // minimum approvals required
}

pub struct Approval {
    pub signer: Address,   // approving address
    pub timestamp: u64,    // ledger timestamp at approval time
}
```

---

## Constants

| Constant                  | Value    | Purpose                                 |
| ------------------------- | -------- | --------------------------------------- |
| `MIN_SIGNERS`             | `1`      | Minimum signer count                    |
| `MAX_SIGNERS`             | `20`     | Maximum signer count (bounds iteration) |
| `APPROVAL_EXPIRY_SECONDS` | `86_400` | Approval validity window (24 hours)     |

---

## Usage

```rust
use crate::multi_signature_execution::{
    validate_config, validate_approval, check_execution_threshold,
    emit_approval_event, emit_execution_event, MultiSigConfig, Approval,
};

// 1. At initialisation — validate and store the config.
validate_config(&config)?;

// 2. When a signer submits an approval.
validate_approval(&config, &approvals, &signer, env.ledger().timestamp())?;
approvals.push_back(Approval { signer: signer.clone(), timestamp: env.ledger().timestamp() });
let valid = count_valid_approvals(&approvals, env.ledger().timestamp());
emit_approval_event(&env, &signer, valid, config.threshold);

// 3. Before executing the privileged operation.
let result = check_execution_threshold(&config, &approvals, env.ledger().timestamp());
if !result.is_approved() {
    panic!("multi-sig threshold not met");
}
emit_execution_event(&env, config.threshold, env.ledger().timestamp());
// ... execute privileged operation ...
```

---

## Security Assumptions

1. All functions are pure — no storage reads or writes. Auth (`require_auth`)
   and storage are the caller's responsibility.

2. `validate_config` must be called before any config is persisted. An invalid
   config stored on-chain could make the multi-sig permanently unexecutable or
   trivially bypassable.

3. `validate_approval` must be called before any approval is appended to
   storage. Skipping it allows double-voting and unauthorised approvals.

4. `check_execution_threshold` is the single gate before any privileged
   operation. Callers must not bypass it.

5. `APPROVAL_EXPIRY_SECONDS` (86 400 s / 24 h) is a hard-coded constant.
   Changing it requires a contract upgrade with multi-sig approval.

6. Duplicate-signer detection in `validate_config` is O(n²) for n ≤ 20 —
   acceptable for the bounded signer set.

7. `count_valid_approvals` uses `saturating_sub` for clock-skew safety: an
   approval with a future timestamp is treated as age 0 (valid).

8. Every approval, execution, and revocation emits an on-chain event. Silent
   operations are not permitted.

---

## Running the Tests

```bash
# Run all multi-signature tests
cargo test -p security multi_signature

# Run with output
cargo test -p security multi_signature -- --nocapture

# Run property-based tests only
cargo test -p security prop_

# Run the full security crate suite
cargo test -p security
```

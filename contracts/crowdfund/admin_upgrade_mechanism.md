# Admin Upgrade Mechanism

Addresses the gas-efficiency edge cases for the admin upgrade mechanism validation.

## Overview

`admin_upgrade_mechanism.rs` provides the validation and execution helpers for upgrading the crowdfund contract's WASM implementation. The module is designed around a **cheapest-check-first** principle: pure, zero-cost validations run before any storage reads or auth calls, minimising gas consumption on the failure path.

## File Structure

```
contracts/crowdfund/src/
├── admin_upgrade_mechanism.rs       # Core helpers
├── admin_upgrade_mechanism_test.rs  # Comprehensive tests
└── admin_upgrade_mechanism.md       # This document
```

## Public API

```rust
/// Pure: returns true when wasm_hash is non-zero. No storage reads.
pub fn validate_wasm_hash(wasm_hash: &BytesN<32>) -> bool

/// Cheap existence check: returns true when an admin address is stored.
/// Uses has() — no deserialization cost.
pub fn is_admin_initialized(env: &Env) -> bool

/// Loads the stored admin address and enforces require_auth().
/// Panics with "Admin not initialized" when no admin is stored.
pub fn validate_admin_upgrade(env: &Env) -> Address

/// Executes the WASM swap via env.deployer().
/// Must only be called after both validate_wasm_hash and validate_admin_upgrade succeed.
pub fn perform_upgrade(env: &Env, new_wasm_hash: BytesN<32>)
```

## Gas-Efficiency Design

### Validation order in `upgrade()`

```
upgrade(new_wasm_hash)
  │
  ├─ 1. validate_wasm_hash(&new_wasm_hash)   ← pure, no I/O, ~0 gas
  │       └─ zero hash → panic "zero wasm hash"  (short-circuit)
  │
  ├─ 2. validate_admin_upgrade(&env)         ← 1 storage read + require_auth
  │       └─ no admin → panic "Admin not initialized"
  │       └─ wrong signer → auth error
  │
  ├─ 3. perform_upgrade(&env, hash)          ← deployer call
  │
  └─ 4. emit audit event
```

Rejecting a zero hash before touching storage means a caller with a bad hash pays the minimum possible gas — no storage read, no auth check, no deployer call.

### `is_admin_initialized` vs `validate_admin_upgrade`

| Function | Storage op | Auth check | Use when |
|----------|-----------|------------|----------|
| `is_admin_initialized` | `has()` — existence only | No | Gating on init state without needing the address |
| `validate_admin_upgrade` | `get()` + deserialize | Yes | Full auth enforcement before upgrade |

`has()` avoids deserializing the stored `Address` value, saving gas when only presence matters.

## New Edge Cases (this PR)

### 1. Zero-hash short-circuit (gas efficiency)

Previously `upgrade()` called `validate_admin_upgrade` first, paying a storage read even for a zero hash. Now:

```rust
// Cheapest check first — no storage read on the failure path.
if !admin_upgrade_mechanism::validate_wasm_hash(&new_wasm_hash) {
    panic!("zero wasm hash");
}
```

This means:
- A zero-hash call from any caller (admin or not, initialized or not) is rejected immediately.
- The panic message is `"zero wasm hash"` — distinct from `"Admin not initialized"` and auth errors, making failures easy to diagnose.

### 2. `validate_wasm_hash` — exported pure helper

Previously the zero-hash check was implicit (the deployer would fail). Now it is an explicit, exported, pure function:

```rust
pub fn validate_wasm_hash(wasm_hash: &BytesN<32>) -> bool {
    wasm_hash.to_array() != [0u8; 32]
}
```

- No `Env` parameter — zero overhead.
- Testable in complete isolation from the contract.

### 3. `is_admin_initialized` — cheap existence check

New helper for callers that only need to know whether an admin has been set:

```rust
pub fn is_admin_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}
```

`has()` is cheaper than `get()` because it does not deserialize the stored value.

## Security Assumptions

1. `validate_wasm_hash` is pure and read-only — no state mutations.
2. A zeroed hash is never a valid WASM hash; rejecting it early prevents accidental contract bricking.
3. `validate_admin_upgrade` uses `require_auth()` — the transaction must be signed by the stored admin address.
4. `perform_upgrade` must only be called after both validation helpers succeed.
5. The admin is set once during `initialize()` and is separate from the campaign creator.
6. Storage is not mutated by any validation helper — a rejected upgrade leaves all campaign state intact.

## NatSpec-style Reference

### `validate_wasm_hash`
- **@notice** Returns `true` when `wasm_hash` is non-zero.
- **@dev** Pure function — no storage reads, no auth, minimal gas cost.
- **@security** Prevents upgrade calls with a zeroed hash, which would brick the contract.

### `is_admin_initialized`
- **@notice** Returns `true` when an admin address has been stored.
- **@dev** Uses `has()` — no deserialization cost. Prefer over `validate_admin_upgrade` when only presence matters.
- **@security** Read-only; no state mutations.

### `validate_admin_upgrade`
- **@notice** Loads the stored admin address and enforces authorization.
- **@dev** Panics with `"Admin not initialized"` when no admin is stored.
- **@security** `require_auth()` ensures the transaction is signed by the stored admin.

### `perform_upgrade`
- **@notice** Executes the WASM swap via the Soroban deployer.
- **@dev** Must only be called after both `validate_wasm_hash` and `validate_admin_upgrade` have succeeded.

## Test Coverage Summary

| Group | Tests |
|-------|-------|
| `validate_wasm_hash` — pure | 6 |
| `is_admin_initialized` | 2 |
| `validate_admin_upgrade` via client | 4 |
| Zero-hash short-circuit (gas edge cases) | 3 |
| Storage persistence after rejection | 2 |
| **Total** | **17** |

## Running Tests

```bash
cargo test -p crowdfund -- admin_upgrade_mechanism
```
## Overview

`admin_upgrade_mechanism.rs` provides the validation and execution helpers for upgrading the crowdfund contract's WASM implementation. The module is designed around a **cheapest-check-first** principle: pure, zero-cost validations run before any storage reads or auth calls, minimising gas consumption on the failure path.

## File Structure

```
contracts/crowdfund/src/
├── admin_upgrade_mechanism.rs       # Core helpers
├── admin_upgrade_mechanism_test.rs  # Comprehensive tests
└── admin_upgrade_mechanism.md       # This document
```

## Security Features

### Authentication Requirements

All upgrade operations require the admin to authorize the call via Soroban's `require_auth()` mechanism. This ensures:

- **Non-repudiation**: Admin cannot deny authorizing an upgrade
- **Replay Protection**: Soroban's built-in nonce mechanism prevents replay attacks
- **Atomic Execution**: Upgrades either complete fully or fail without side effects

### WASM Hash Validation

The mechanism validates WASM hashes to prevent deployment of invalid code:

- **Non-zero Requirement**: All-zero hashes are rejected by `validate_wasm_hash()` **before** the auth check, so the guard cannot be bypassed by providing valid admin credentials
- **Size Verification**: Hashes must be exactly 32 bytes (SHA-256)
- **Upload Verification**: The WASM must be uploaded to the ledger before deployment

### Admin Isolation

The admin role is deliberately separated from other roles:

- **Distinct from Creator**: The campaign creator cannot perform upgrades
- **Distinct from Users**: Regular users have no upgrade privileges
- **Initialization Required**: Upgrades are impossible before contract initialization

## How It Works

### Admin Assignment

The admin is set once during `initialize()` and stored in instance storage:
## Public API

```rust
/// Pure: returns true when wasm_hash is non-zero. No storage reads.
pub fn validate_wasm_hash(wasm_hash: &BytesN<32>) -> bool

/// Cheap existence check: returns true when an admin address is stored.
/// Uses has() — no deserialization cost.
pub fn is_admin_initialized(env: &Env) -> bool

/// Loads the stored admin address and enforces require_auth().
/// Panics with "Admin not initialized" when no admin is stored.
pub fn validate_admin_upgrade(env: &Env) -> Address

/// Executes the WASM swap via env.deployer().
/// Must only be called after both validate_wasm_hash and validate_admin_upgrade succeed.
pub fn perform_upgrade(env: &Env, new_wasm_hash: BytesN<32>)
```

## Gas-Efficiency Design

### Validation order in `upgrade()`

```
upgrade(new_wasm_hash)
  │
  ├─ 1. validate_wasm_hash(&new_wasm_hash)   ← pure, no I/O, ~0 gas
  │       └─ zero hash → panic "zero wasm hash"  (short-circuit)
  │
  ├─ 2. validate_admin_upgrade(&env)         ← 1 storage read + require_auth
  │       └─ no admin → panic "Admin not initialized"
  │       └─ wrong signer → auth error
  │
  ├─ 3. perform_upgrade(&env, hash)          ← deployer call
  │
  └─ 4. emit audit event
```

Rejecting a zero hash before touching storage means a caller with a bad hash pays the minimum possible gas — no storage read, no auth check, no deployer call.

### `is_admin_initialized` vs `validate_admin_upgrade`

| Function | Storage op | Auth check | Use when |
|----------|-----------|------------|----------|
| `is_admin_initialized` | `has()` — existence only | No | Gating on init state without needing the address |
| `validate_admin_upgrade` | `get()` + deserialize | Yes | Full auth enforcement before upgrade |

`has()` avoids deserializing the stored `Address` value, saving gas when only presence matters.

## New Edge Cases (this PR)

### 1. Zero-hash short-circuit (gas efficiency)

Previously `upgrade()` called `validate_admin_upgrade` first, paying a storage read even for a zero hash. Now:

```rust
// Cheapest check first — no storage read on the failure path.
if !admin_upgrade_mechanism::validate_wasm_hash(&new_wasm_hash) {
    panic!("zero wasm hash");
}
```

This means:
- A zero-hash call from any caller (admin or not, initialized or not) is rejected immediately.
- The panic message is `"zero wasm hash"` — distinct from `"Admin not initialized"` and auth errors, making failures easy to diagnose.

### 2. `validate_wasm_hash` — exported pure helper

Previously the zero-hash check was implicit (the deployer would fail). Now it is an explicit, exported, pure function:

```rust
pub fn validate_wasm_hash(wasm_hash: &BytesN<32>) -> bool {
    wasm_hash.to_array() != [0u8; 32]
}
```

- No `Env` parameter — zero overhead.
- Testable in complete isolation from the contract.

### 3. `is_admin_initialized` — cheap existence check

The mechanism defines comprehensive error types for precise failure identification:

| Error Code | Name | Description |
|------------|------|-------------|
| 1 | `NotInitialized` | No admin is set (contract not initialized) |
| 2 | `NotAuthorized` | Caller is not the authorized admin |
| 3 | `InvalidWasmHash` | WASM hash is zero or otherwise invalid |
| 4 | `SameWasmHash` | New hash matches current hash (no-op) |
| 5 | `SameAdmin` | New admin matches current admin (no-op) |
| 6 | `InvalidAdminAddress` | New admin address is invalid |

## Test Coverage

The test suite provides comprehensive coverage across multiple categories:

### Category 1: Admin Storage Tests (3 tests)
| Test | Description |
|------|-------------|
| `test_admin_stored_on_initialize` | Admin is stored during initialization |
| `test_admin_persists_across_operations` | Admin survives multiple operations |
| `test_admin_distinct_from_other_addresses` | Admin is different from creator/token |

### Category 2: Upgrade Authorization Tests (4 tests)
| Test | Description |
|------|-------------|
| `test_admin_can_call_upgrade` | Admin can successfully upgrade |
| `test_non_admin_cannot_upgrade` | Random address is rejected |
| `test_creator_cannot_upgrade` | Campaign creator is rejected |
| `test_multiple_non_admin_attempts_rejected` | Multiple attackers blocked |

### Category 3: WASM Hash Validation Tests (6 tests)
| Test | Description |
|------|-------------|
| `test_zero_wasm_hash_rejected_before_auth` | All-zero hash rejected before auth check |
| `test_non_zero_wasm_hash_passes_validation` | Non-zero hashes pass `validate_wasm_hash` |
| `test_validate_wasm_hash_rejects_zero` | Unit test: zero hash returns `false` |
| `test_non_zero_wasm_hash_valid` | Non-zero hash passes validation |
| `test_max_value_wasm_hash_valid` | Maximum value hash is valid |
| `test_single_bit_set_hash_valid` | Single bit hash is valid |

### Category 4: Edge Case Tests (5 tests)
| Test | Description |
|------|-------------|
| `test_upgrade_panics_before_initialize` | Panics without admin |
| `test_upgrade_requires_auth` | Auth is mandatory |
| `test_multiple_non_admin_attempts_all_rejected` | Brute-force: all attempts rejected, storage intact |
| `test_two_contracts_have_independent_admins` | Contract-instance isolation |
| `test_token_address_cannot_upgrade` | Token address is not the admin |

**Total: 12 tests (7 existing + 5 new edge cases)**

## Usage Examples

### Basic Upgrade
New helper for callers that only need to know whether an admin has been set:

```rust
pub fn is_admin_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}
```

`has()` is cheaper than `get()` because it does not deserialize the stored value.

## Security Assumptions

1. `validate_wasm_hash` is pure and read-only — no state mutations.
2. A zeroed hash is never a valid WASM hash; rejecting it early prevents accidental contract bricking.
3. `validate_admin_upgrade` uses `require_auth()` — the transaction must be signed by the stored admin address.
4. `perform_upgrade` must only be called after both validation helpers succeed.
5. The admin is set once during `initialize()` and is separate from the campaign creator.
6. Storage is not mutated by any validation helper — a rejected upgrade leaves all campaign state intact.

## NatSpec-style Reference

### `validate_wasm_hash`
- **@notice** Returns `true` when `wasm_hash` is non-zero.
- **@dev** Pure function — no storage reads, no auth, minimal gas cost.
- **@security** Prevents upgrade calls with a zeroed hash, which would brick the contract.

### `is_admin_initialized`
- **@notice** Returns `true` when an admin address has been stored.
- **@dev** Uses `has()` — no deserialization cost. Prefer over `validate_admin_upgrade` when only presence matters.
- **@security** Read-only; no state mutations.

### `validate_admin_upgrade`
- **@notice** Loads the stored admin address and enforces authorization.
- **@dev** Panics with `"Admin not initialized"` when no admin is stored.
- **@security** `require_auth()` ensures the transaction is signed by the stored admin.

### `perform_upgrade`
- **@notice** Executes the WASM swap via the Soroban deployer.
- **@dev** Must only be called after both `validate_wasm_hash` and `validate_admin_upgrade` have succeeded.

## Test Coverage Summary

| Group | Tests |
|-------|-------|
| `validate_wasm_hash` — pure | 6 |
| `is_admin_initialized` | 2 |
| `validate_admin_upgrade` via client | 4 |
| Zero-hash short-circuit (gas edge cases) | 3 |
| Storage persistence after rejection | 2 |
| **Total** | **17** |

## Running Tests

```bash
cargo test -p crowdfund -- admin_upgrade_mechanism
```

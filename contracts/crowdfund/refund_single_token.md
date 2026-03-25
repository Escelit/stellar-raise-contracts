# `refund_single_token` — Pull-Based Token Refund Logic

## Overview

The `refund_single_token` module centralises all logic for executing a single
contributor refund. It exposes four public items:

| Symbol                        | Purpose                                                  |
|-------------------------------|----------------------------------------------------------|
| `refund_single_transfer`      | Direction-locked token transfer (contract → contributor) |
| `validate_refund_preconditions` | Pure guard — checks all preconditions, returns amount  |
| `execute_refund_single`       | Atomic CEI execution — zero storage, then transfer       |
| `refund_available`            | View function — checks if refund is available for UI     |

The `refund_single` contract method in `lib.rs` is now a three-line wrapper:

```rust
pub fn refund_single(env: Env, contributor: Address) -> Result<(), ContractError> {
    contributor.require_auth();
    let amount = validate_refund_preconditions(&env, &contributor)?;
    execute_refund_single(&env, &contributor, amount)
}
```

The `refund_available` view function allows frontend UI to check refund status:

```rust
pub fn refund_available(env: Env, contributor: Address) -> Result<i128, ContractError> {
    validate_refund_preconditions(&env, &contributor)
}
```

---

## Why pull-based?

The deprecated `refund()` iterated over every contributor in one transaction.
With many contributors this is unsafe:


If the contributor has `0` stored contribution, `refund_single_transfer` helper skips the transfer (gas optimization) and returns `Ok(())`.

- **Unbounded gas** — iteration cost grows linearly with contributor count.
- **DoS** — a single bad actor can bloat the list to make the batch prohibitively expensive.
- **Poor composability** — scripts cannot easily retry partial failures.


`refund_single` processes exactly one contributor per call, so gas costs are
constant and predictable regardless of campaign size.

---

## Security Model

### 1. Authentication
`contributor.require_auth()` is called in `lib.rs` before any module function
is invoked. Only the contributor themselves can trigger their own refund.

### 2. Checks-Effects-Interactions (CEI)
`execute_refund_single` zeroes the contribution record in storage **before**
calling `refund_single_transfer`. This prevents re-entrancy: even if the token
contract calls back into the crowdfund contract, the contribution is already 0
and `validate_refund_preconditions` will return `NothingToRefund`.

5. **Zero-amount optimization**  
   `refund_single_transfer` skips transfers where `amount <= 0`, preventing no-op gas waste.

6. **Debug logging**  
   Emits `("debug", "refund_transfer_attempt") (contributor, amount)` before successful transfers for observability.


## Test coverage

`refund_single_token.test.rs` + `refund_single_token_security_tests.rs` cover:


```
validate_refund_preconditions  ← pure read, no state change
    ↓
execute_refund_single
    ├── storage.set(contribution, 0)   ← Effect
    ├── storage.set(total_raised, new) ← Effect
    └── token.transfer(contract → contributor) ← Interaction
```

### 3. Direction lock
`refund_single_transfer` always transfers `contract → contributor`. The
direction cannot be reversed by a caller because the parameters are positional
and the function signature enforces the order.


### 4. Overflow protection
`execute_refund_single` decrements `total_raised` with `checked_sub`, returning
`ContractError::Overflow` rather than silently wrapping.

### 5. Separation of concerns
`validate_refund_preconditions` is a pure read function — it mutates no state.
This makes it safe to call speculatively (e.g. in a simulation or dry-run) and
easy to unit-test in isolation.

---

## API Reference

### `refund_single_transfer`

```rust
pub fn refund_single_transfer(
    token_client: &token::Client,
    contract_address: &Address,
    contributor: &Address,
    amount: i128,
)
```

Transfers `amount` tokens from `contract_address` to `contributor`.
Direction is fixed; cannot be reversed.

---

### `validate_refund_preconditions`

```rust
pub fn validate_refund_preconditions(
    env: &Env,
    contributor: &Address,
) -> Result<i128, ContractError>
```

Returns `Ok(amount)` when all preconditions pass, or the appropriate error.

| Error                          | Condition                                      |
|--------------------------------|------------------------------------------------|
| `ContractError::CampaignStillActive` | `ledger.timestamp() <= deadline`         |
| `ContractError::GoalReached`   | `total_raised >= goal`                         |
| `ContractError::NothingToRefund` | Contribution record is 0 or absent           |

Panics with `"campaign is not active"` when status is `Successful` or `Cancelled`.

---

### `refund_available`

```rust
pub fn refund_available(env: Env, contributor: Address) -> Result<i128, ContractError>
```

View function that checks if a refund is available for the given contributor.
Returns the refundable amount if available, or the appropriate error.
Safe to call without authentication; useful for frontend UI to show refund status.

---

### `execute_refund_single`

```rust
pub fn execute_refund_single(
    env: &Env,
    contributor: &Address,
    amount: i128,
) -> Result<(), ContractError>
```

Executes the refund atomically using CEI order. Caller must have already
validated preconditions and obtained `amount` from `validate_refund_preconditions`.

Returns `Err(ContractError::Overflow)` if `total_raised - amount` underflows.

---

## Events

On success, `execute_refund_single` emits:

```
topic: ("campaign", "refund_single")
data:  (contributor: Address, amount: i128)
```

---

## Deprecation of `refund()`

The batch `refund()` function is **deprecated** as of contract v3. It remains
callable for backward compatibility but will be removed in a future upgrade.

Migration checklist:

- [ ] Replace calls to `refund()` with per-contributor calls to `refund_single(contributor)`.
- [ ] Handle `NothingToRefund` gracefully (contributor already claimed or never contributed).
- [ ] Listen for `("campaign", "refund_single")` events instead of `("campaign", "refunded")`.

---

## CLI Usage

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  --source <CONTRIBUTOR_SECRET_KEY> \
  -- refund_single \
  --contributor <CONTRIBUTOR_ADDRESS>
```

---

## Test Coverage

See [`refund_single_token.test.rs`](./refund_single_token.test.rs) for the
unit test suite covering `validate_refund_preconditions` and
`execute_refund_single`, and [`refund_single_token_tests.rs`](./refund_single_token_tests.rs)
for integration tests via the contract client.

Tests cover:

- Happy-path amount return from `validate`
- Deadline boundary (at and past)
- Goal exactly met and exceeded
- No contribution / zero contribution
- Post-refund `NothingToRefund`
- Successful / Cancelled campaign panics
- CEI order (storage zeroed before transfer)
- `total_raised` decrement
- Double-refund prevention
- Large amounts (overflow protection)
- Multi-contributor isolation
- `refund_available` view function for UI state
# refund_single() Token Transfer Logic

## Overview

The `refund_single_token` module centralises all logic for executing a single
contributor refund. It exposes three public items:

| Symbol                        | Purpose                                                  |
|-------------------------------|----------------------------------------------------------|
| `refund_single_transfer`      | Direction-locked token transfer (contract → contributor) |
| `validate_refund_preconditions` | Pure guard — checks all preconditions, returns amount  |
| `execute_refund_single`       | Atomic CEI execution — zero storage, then transfer       |

The `refund_single` contract method in `lib.rs` is now a three-line wrapper:

```rust
pub fn refund_single(env: Env, contributor: Address) -> Result<(), ContractError> {
    contributor.require_auth();
    let amount = validate_refund_preconditions(&env, &contributor)?;
    execute_refund_single(&env, &contributor, amount)
}
```

---

## Why pull-based?

The deprecated `refund()` iterated over every contributor in one transaction.
With many contributors this is unsafe:

- **Unbounded gas** — iteration cost grows linearly with contributor count.
- **DoS** — a single bad actor can bloat the list to make the batch prohibitively expensive.
- **Poor composability** — scripts cannot easily retry partial failures.

`refund_single` processes exactly one contributor per call, so gas costs are
constant and predictable regardless of campaign size.

---
If the contributor has `0` stored contribution, `refund_single_transfer` helper skips the transfer (gas optimization) and returns `Ok(())`.

## Security Model

### 1. Authentication
`contributor.require_auth()` is called in `lib.rs` before any module function
is invoked. Only the contributor themselves can trigger their own refund.

### 2. Checks-Effects-Interactions (CEI)
`execute_refund_single` zeroes the contribution record in storage **before**
calling `refund_single_transfer`. This prevents re-entrancy: even if the token
contract calls back into the crowdfund contract, the contribution is already 0
and `validate_refund_preconditions` will return `NothingToRefund`.

```
validate_refund_preconditions  ← pure read, no state change
    ↓
execute_refund_single
    ├── storage.set(contribution, 0)   ← Effect
    ├── storage.set(total_raised, new) ← Effect
    └── token.transfer(contract → contributor) ← Interaction
```

### 3. Direction lock
`refund_single_transfer` always transfers `contract → contributor`. The
direction cannot be reversed by a caller because the parameters are positional
and the function signature enforces the order.

### 4. Overflow protection
`execute_refund_single` decrements `total_raised` with `checked_sub`, returning
`ContractError::Overflow` rather than silently wrapping.

### 5. Separation of concerns
`validate_refund_preconditions` is a pure read function — it mutates no state.
This makes it safe to call speculatively (e.g. in a simulation or dry-run) and
easy to unit-test in isolation.
5. **Zero-amount optimization**  
   `refund_single_transfer` skips transfers where `amount <= 0`, preventing no-op gas waste.

6. **Debug logging**  
   Emits `("debug", "refund_transfer_attempt") (contributor, amount)` before successful transfers for observability.


## Test coverage

`refund_single_token.test.rs` + `refund_single_token_security_tests.rs` cover:


---

## API Reference

### `refund_single_transfer`

```rust
pub fn refund_single_transfer(
    token_client: &token::Client,
    contract_address: &Address,
    contributor: &Address,
    amount: i128,
)
```

Transfers `amount` tokens from `contract_address` to `contributor`.
Direction is fixed; cannot be reversed.

---

### `validate_refund_preconditions`

```rust
pub fn validate_refund_preconditions(
    env: &Env,
    contributor: &Address,
) -> Result<i128, ContractError>
```

Returns `Ok(amount)` when all preconditions pass, or the appropriate error.

| Error                          | Condition                                      |
|--------------------------------|------------------------------------------------|
| `ContractError::CampaignStillActive` | `ledger.timestamp() <= deadline`         |
| `ContractError::GoalReached`   | `total_raised >= goal`                         |
| `ContractError::NothingToRefund` | Contribution record is 0 or absent           |

Panics with `"campaign is not active"` when status is `Successful` or `Cancelled`.

---

### `execute_refund_single`

```rust
pub fn execute_refund_single(
    env: &Env,
    contributor: &Address,
    amount: i128,
) -> Result<(), ContractError>
```

Executes the refund atomically using CEI order. Caller must have already
validated preconditions and obtained `amount` from `validate_refund_preconditions`.

Returns `Err(ContractError::Overflow)` if `total_raised - amount` underflows.

---

## Events

On success, `execute_refund_single` emits:

```
topic: ("campaign", "refund_single")
data:  (contributor: Address, amount: i128)
```

---

## Deprecation of `refund()`

The batch `refund()` function is **deprecated** as of contract v3. It remains
callable for backward compatibility but will be removed in a future upgrade.

Migration checklist:

- [ ] Replace calls to `refund()` with per-contributor calls to `refund_single(contributor)`.
- [ ] Handle `NothingToRefund` gracefully (contributor already claimed or never contributed).
- [ ] Listen for `("campaign", "refund_single")` events instead of `("campaign", "refunded")`.

---

## CLI Usage

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  --source <CONTRIBUTOR_SECRET_KEY> \
  -- refund_single \
  --contributor <CONTRIBUTOR_ADDRESS>
```

---

## Test Coverage

See [`refund_single_token.test.rs`](./refund_single_token.test.rs) for the
unit test suite covering `validate_refund_preconditions` and
`execute_refund_single`, and [`refund_single_token_tests.rs`](./refund_single_token_tests.rs)
for integration tests via the contract client.

Tests cover:

- Happy-path amount return from `validate`
- Deadline boundary (at and past)
- Goal exactly met and exceeded
- No contribution / zero contribution
- Post-refund `NothingToRefund`
- Successful / Cancelled campaign panics
- CEI order (storage zeroed before transfer)
- `total_raised` decrement
- Double-refund prevention
- Large amounts (overflow protection)
- Multi-contributor isolation

# `crowdfund_initialize_function` — Optimized Initialize Logic
# `crowdfund_initialize_function` — Refactored Initialize Logic

## Overview

`crowdfund_initialize_function` extracts and standardizes the `initialize()`
logic from `lib.rs` into a single, auditable module with performance optimizations.

## Performance Optimizations

### 1. Early Validation Exit
Uses `?` operator for short-circuit error propagation instead of nested `if let Err` blocks:

```rust
// Before (nested)
if let Err(e) = validate_goal(params.goal) {
    return Err(e);
}

// After (short-circuit)
validate_goal(params.goal)?;
```

### 2. Inline Hints
Validation helpers are marked `#[inline]` to allow the compiler to specialize call sites:

```rust
#[inline]
pub fn validate_bonus_goal(bonus_goal: Option<i128>, goal: i128) -> Result<(), ContractError> {
    // ...
}
```

### 3. Batched Validation
All parameter checks run in a single `validate_init_params()` call:

```rust
pub fn validate_init_params(env: &Env, params: &InitParams) -> Result<(), ContractError> {
    validate_goal(params.goal)?;
    validate_min_contribution(params.min_contribution)?;
    validate_deadline(env.ledger().timestamp(), params.deadline)?;
    if let Some(ref config) = params.platform_config {
        validate_platform_fee(config.fee_bps)?;
    }
    validate_bonus_goal(params.bonus_goal, params.goal)?;
    validate_bonus_goal_description(&params.bonus_goal_description)?;
    Ok(())
}
```

### 4. Optimized Storage Writes
All required storage writes are grouped together with only necessary conditional writes for optional fields.

### 5. Single Sentinel Check
Uses a single `has()` check on `DataKey::Creator` as the initialization sentinel:

```rust
if env.storage().instance().has(&DataKey::Creator) {
    return Err(ContractError::AlreadyInitialized);
}
```
logic from `lib.rs` into a single, auditable module.  It provides:

## Performance Optimizations

### 1. Early Validation Exit
Uses `?` operator for short-circuit error propagation instead of nested `if let Err` blocks:

```rust
// Before (nested)
if let Err(e) = validate_goal(params.goal) {
    return Err(e);
}

// After (short-circuit)
validate_goal(params.goal)?;
```

### 2. Inline Hints
Validation helpers are marked `#[inline]` to allow the compiler to specialize call sites:

```rust
#[inline]
pub fn validate_bonus_goal(bonus_goal: Option<i128>, goal: i128) -> Result<(), ContractError> {
    // ...
}
```

### 3. Batched Validation
All parameter checks run in a single `validate_init_params()` call:

```rust
pub fn validate_init_params(env: &Env, params: &InitParams) -> Result<(), ContractError> {
    validate_goal(params.goal)?;
    validate_min_contribution(params.min_contribution)?;
    validate_deadline(env.ledger().timestamp(), params.deadline)?;
    if let Some(ref config) = params.platform_config {
        validate_platform_fee(config.fee_bps)?;
    }
    validate_bonus_goal(params.bonus_goal, params.goal)?;
    validate_bonus_goal_description(&params.bonus_goal_description)?;
    Ok(())
}
```

### 4. Optimized Storage Writes
All required storage writes are grouped together with only necessary conditional writes for optional fields.

### 5. Single Sentinel Check
Uses a single `has()` check on `DataKey::Creator` as the initialization sentinel:

```rust
if env.storage().instance().has(&DataKey::Creator) {
    return Err(ContractError::AlreadyInitialized);
}
```

---

## Design Decisions

### Named `InitParams` struct

The original `initialize()` accepted nine positional arguments. Positional
lists are fragile: swapping two `i128` parameters compiles silently but
produces incorrect on-chain state. A named struct makes every field explicit
The original `initialize()` accepted nine positional arguments.  Positional
lists are fragile: swapping two `i128` parameters compiles silently but
produces incorrect on-chain state. A named struct makes every field explicit
at the call site and lets the compiler catch type mismatches.

### Typed errors instead of panics

The original implementation panicked on invalid platform fee and bonus goal.
Panics are opaque to the frontend — the Soroban SDK surfaces them as a generic
host error with no numeric code. Typed `ContractError` variants let the
host error with no numeric code.  Typed `ContractError` variants let the
frontend display a specific message without parsing error strings.

| New variant | Code | Trigger |
|---|---|---|
| `InvalidGoal` | 8 | `goal < 1` |
| `InvalidMinContribution` | 9 | `min_contribution < 1` |
| `DeadlineTooSoon` | 10 | `deadline < now + 60` |
| `InvalidPlatformFee` | 11 | `fee_bps > 10_000` |
| `InvalidBonusGoal` | 12 | `bonus_goal <= goal` |
| `InvalidBonusGoalDescription` | 13 | Description too long |

### Validate-before-write ordering

The original code interleaved validation and storage writes. If a later
validation failed after earlier writes had already committed, the contract
could be left in a partially-initialized state. `execute_initialize()` runs

### Validate-before-write ordering

The original code interleaved validation and storage writes. If a later
validation failed after earlier writes had already committed, the contract
could be left in a partially-initialized state. `execute_initialize()` runs
all validations first, then writes atomically within the transaction.

### `initialized` event

Soroban storage is not directly queryable by off-chain services without an RPC
call per field. The `initialized` event carries all campaign parameters in a
call per field.  The `initialized` event carries all campaign parameters in a
single ledger entry, enabling indexers to bootstrap campaign state from the
event stream alone.

---

## Function Reference

### `execute_initialize(env, params) → Result<(), ContractError>`

The single authoritative implementation of campaign initialization.
`CrowdfundContract::initialize()` in `lib.rs` delegates to this function.

**Ordering guarantee:**
1. Re-initialization guard (read-only check on `DataKey::Creator`).
2. `creator.require_auth()` — authentication before any state mutation.
3. Full parameter validation — no storage writes until all checks pass.
4. Storage writes — all-or-nothing within the transaction.
5. Event emission — `("campaign", "initialized")`.

### `validate_init_params(env, params) → Result<(), ContractError>`

Runs all field validations in a single pass. Delegates to the helpers in
Runs all field validations in a single pass.  Delegates to the helpers in
`campaign_goal_minimum` for goal, min_contribution, deadline, and platform fee,
and to `validate_bonus_goal` for the bonus goal ordering constraint.

### `validate_bonus_goal(bonus_goal, goal) → Result<(), ContractError>`

Returns `Ok(())` when `bonus_goal` is `None` or strictly greater than `goal`.
Returns `Err(ContractError::InvalidBonusGoal)` otherwise.

### `log_initialize(env, creator, token, goal, deadline, min_contribution)`

Emits a single bounded `("campaign", "initialized")` event with a fixed-size
scalar payload. Only the five core scalar fields are included — optional strings
such as `bonus_goal_description` are intentionally excluded to keep event size
O(1) regardless of input length.

**Gas efficiency**: a single event with a 5-field tuple costs a fixed amount of
gas. Including unbounded strings would make event cost proportional to string
length, creating a gas griefing vector.

**Event data**: `(Address, Address, i128, u64, i128)` — `(creator, token, goal, deadline, min_contribution)`.
### `validate_bonus_goal_description(description) → Result<(), ContractError>`

Returns `Ok(())` when description is `None` or within length limits.
Returns `Err(ContractError::InvalidBonusGoalDescription)` otherwise.

### `describe_init_error(code) → &'static str`

Maps a `ContractError` repr value to a human-readable string. Intended for
### `describe_init_error(code) → &'static str`

Maps a `ContractError` repr value to a human-readable string.  Intended for
frontend error display.

### `is_init_error_retryable(code) → bool`

Returns `true` for input validation errors (codes 8–12) that the caller can
fix and retry. Returns `false` for `AlreadyInitialized` (code 1), which is
fix and retry.  Returns `false` for `AlreadyInitialized` (code 1), which is
permanent.

---

## Frontend Interaction

1. Construct the `initialize` transaction with all required parameters.
1. Construct the `initialize` transaction with all nine parameters.
2. On success, listen for the `("campaign", "initialized")` event to confirm
   the campaign is live and cache the emitted parameters locally.
3. On failure, read the returned error code and call `describe_init_error(code)`
   to display a user-facing message.
4. Use `is_init_error_retryable(code)` to decide whether to show a "try again"
   button or a permanent failure message.

```typescript
// TypeScript / Stellar SDK example
try {
  await contract.initialize({ admin, creator, token, goal, deadline, ... });
} catch (e) {
  const code = extractContractErrorCode(e); // SDK-specific helper
  const message = describeInitError(code);  // replicate describe_init_error
  const retryable = isInitErrorRetryable(code);
  showError(message, { retryable });
}

// Replicate describe_init_error in TypeScript
function describeInitError(code: number): string {
  const messages: Record<number, string> = {
    1:  "Contract is already initialized",
    8:  "Campaign goal must be at least 1",
    9:  "Minimum contribution must be at least 1",
    10: "Deadline must be at least 60 seconds in the future",
    11: "Platform fee cannot exceed 100% (10,000 bps)",
    12: "Bonus goal must be strictly greater than the primary goal",
  };
  return messages[code] ?? "Unknown initialization error";
}

function isInitErrorRetryable(code: number): boolean {
  return [8, 9, 10, 11, 12].includes(code);
}
# `initialize` — Crowdfund Contract

Initializes a new crowdfunding campaign. Must be called exactly once after deployment.

---

## Signature

```rust
pub fn initialize(
    env: Env,
    admin: Address,
    creator: Address,
    token: Address,
    goal: i128,
    deadline: u64,
    min_contribution: i128,
    platform_config: Option<PlatformConfig>,
    bonus_goal: Option<i128>,
    bonus_goal_description: Option<String>,
) -> Result<(), ContractError>
```

---

## Scalability Considerations

- `initialize()` is a one-shot function; its gas cost is O(1) regardless of
  future campaign size.
- The `Contributors` and `Roadmap` lists are seeded as empty vectors. Their
- The `Contributors` and `Roadmap` lists are seeded as empty vectors.  Their
  TTL is managed by `contribute()` and `add_roadmap_item()` respectively.
- The `initialized` event payload is bounded: it contains only scalar values
  and optional scalars, never unbounded collections.
- The `InitParams` struct can be extended with new optional fields in future
  versions without breaking existing callers (new fields default to `None`).

---

## Security Assumptions

1. **Re-initialization guard** — `DataKey::Creator` is used as the
   initialization sentinel. It is the very first check so no state can be
   written before it.

2. **Creator authentication** — `creator.require_auth()` is called before any
   storage write. The Soroban host rejects the transaction if the creator's
   initialization sentinel.  It is the very first check so no state can be
   written before it.

2. **Creator authentication** — `creator.require_auth()` is called before any
   storage write. The Soroban host rejects the transaction if the creator's
   signature is absent or invalid.

3. **Goal floor** — `goal >= 1` prevents zero-goal campaigns that could be
   immediately drained by the creator.

4. **Minimum contribution floor** — `min_contribution >= 1` prevents
   zero-amount contributions that waste gas and pollute storage.

5. **Deadline offset** — `deadline >= now + 60s` ensures the campaign is live
   for at least one ledger close interval, preventing dead-on-arrival campaigns.

6. **Platform fee cap** — `fee_bps <= 10_000` ensures the platform can never
   be configured to take more than 100% of raised funds.

7. **Bonus goal ordering** — `bonus_goal > goal` prevents a bonus goal that is
   already met at launch, which would immediately emit a bonus event and confuse
   contributors.

8. **Atomic write ordering** — All validations complete before the first
   `env.storage().instance().set()` call. A failed validation leaves the
   contract in its pre-initialization state.

9. **Description length validation** — Bonus goal description length is
   validated to prevent unbounded state growth that could increase storage
   costs and impact contract performance.

   `env.storage().instance().set()` call.  A failed validation leaves the
   contract in its pre-initialization state.

9. **Description length validation** — Bonus goal description length is
   validated to prevent unbounded state growth that could increase storage
   costs and impact contract performance.

---

## Constraints

- `initialize()` can only be called once per contract instance. The factory
- `initialize()` can only be called once per contract instance.  The factory
  contract deploys a fresh instance per campaign.
- The `admin` and `creator` may be the same address or different addresses.
  The contract does not enforce a relationship between them.
- `bonus_goal_description` has no length limit enforced at the contract level.
  The frontend should enforce a reasonable display limit (e.g. 280 characters).
- The `initialized` event is emitted after all storage writes. If the
- The `initialized` event is emitted after all storage writes.  If the
  transaction is rolled back for any reason, the event is not persisted.

---

## Test Coverage

See [`crowdfund_initialize_function_test.rs`](./crowdfund_initialize_function_test.rs).

### Test Categories

| Category | Tests | Coverage |
|----------|-------|----------|
| Normal execution | 8 tests | All fields stored correctly |
| Platform config | 6 tests | Fee boundaries 0, 1, max, max+1, u32::MAX |
| Bonus goal | 6 tests | None, equal, less, one above, max, no description |
| Re-initialization guard | 3 tests | Same params, different params, value preservation |
| Goal validation | 5 tests | Min (1), zero, negative, i128::MIN, i128::MAX |
| Min contribution validation | 3 tests | Min (1), zero, negative |
| Deadline validation | 6 tests | 60s, 59s, equal now, past, far future, u64::MAX |
| Helper unit tests | 8 tests | validate_bonus_goal, describe_init_error, etc. |
| Integration tests | 5 tests | contribute, withdraw, all params combined |

**Total: 50+ tests covering 95%+ code paths**

### Run Tests
Tests cover:

| Category | Tests | Coverage |
|----------|-------|----------|
| Normal execution | 8 tests | All fields stored correctly |
| Platform config | 6 tests | Fee boundaries 0, 1, max, max+1, u32::MAX |
| Bonus goal | 6 tests | None, equal, less, one above, max, no description |
| Re-initialization guard | 3 tests | Same params, different params, value preservation |
| Goal validation | 5 tests | Min (1), zero, negative, i128::MIN, i128::MAX |
| Min contribution validation | 3 tests | Min (1), zero, negative |
| Deadline validation | 6 tests | 60s, 59s, equal now, past, far future, u64::MAX |
| Helper unit tests | 8 tests | validate_bonus_goal, describe_init_error, etc. |
| Integration tests | 5 tests | contribute, withdraw, all params combined |

**Total: 50+ tests covering 95%+ code paths**

### Run Tests

```bash
cargo test -p crowdfund crowdfund_initialize_function
```

### Coverage Report

```bash
cargo test -p crowdfund crowdfund_initialize_function -- --nocapture
cargo tarpaulin --exclude-tests --out Html -p crowdfund
## Parameters

| Parameter               | Type                    | Required | Description                                                                 |
|-------------------------|-------------------------|----------|-----------------------------------------------------------------------------|
| `admin`                 | `Address`               | Yes      | Address authorized to call `upgrade`. Typically the deployer.               |
| `creator`               | `Address`               | Yes      | Campaign creator. Must sign the transaction (`require_auth`).               |
| `token`                 | `Address`               | Yes      | Token contract address used for contributions and payouts.                  |
| `goal`                  | `i128`                  | Yes      | Funding target in the token's smallest unit (e.g. stroops). Must be > 0.   |
| `deadline`              | `u64`                   | Yes      | Campaign end time as a UNIX ledger timestamp. Must be in the future.        |
| `min_contribution`      | `i128`                  | Yes      | Minimum single contribution. Must be > 0 and ≤ `goal`.                     |
| `platform_config`       | `Option<PlatformConfig>`| No       | Optional platform fee config. `fee_bps` must be ≤ 10 000 (100 %).          |
| `bonus_goal`            | `Option<i128>`          | No       | Optional stretch goal. Must be strictly greater than `goal`.                |
| `bonus_goal_description`| `Option<String>`        | No       | Human-readable description of the bonus goal reward.                        |

### `PlatformConfig` fields

| Field     | Type      | Description                                              |
|-----------|-----------|----------------------------------------------------------|
| `address` | `Address` | Platform wallet that receives the fee on withdrawal.     |
| `fee_bps` | `u32`     | Fee in basis points (100 bps = 1 %). Maximum: 10 000.   |

---

## Return value

`Ok(())` on success. The contract is now in `Status::Active` and ready to accept contributions.

---

## Errors

| Error                    | Code | Condition                                                  |
|--------------------------|------|------------------------------------------------------------|
| `AlreadyInitialized`     | 1    | `initialize` has already been called on this contract.     |

Additional panics (not `ContractError`):

| Condition                                        | Message                                      |
|--------------------------------------------------|----------------------------------------------|
| `platform_config.fee_bps > 10_000`               | `"platform fee cannot exceed 100%"`          |
| `bonus_goal` is set but `bonus_goal <= goal`     | `"bonus goal must be greater than primary goal"` |
| `bonus_goal_description` fails length validation | validation error from `contract_state_size`  |

---

## State written

| Storage key              | Type              | Value set                        |
|--------------------------|-------------------|----------------------------------|
| `Admin`                  | `Address`         | `admin`                          |
| `Creator`                | `Address`         | `creator`                        |
| `Token`                  | `Address`         | `token`                          |
| `Goal`                   | `i128`            | `goal`                           |
| `Deadline`               | `u64`             | `deadline`                       |
| `MinContribution`        | `i128`            | `min_contribution`               |
| `TotalRaised`            | `i128`            | `0`                              |
| `Status`                 | `Status`          | `Status::Active`                 |
| `BonusGoalReachedEmitted`| `bool`            | `false`                          |
| `Contributors`           | `Vec<Address>`    | empty list (persistent storage)  |
| `Roadmap`                | `Vec<RoadmapItem>`| empty list                       |
| `PlatformConfig`         | `PlatformConfig`  | set only if `platform_config` is `Some` |
| `BonusGoal`              | `i128`            | set only if `bonus_goal` is `Some`      |
| `BonusGoalDescription`   | `String`          | set only if `bonus_goal_description` is `Some` |

---

## Security notes

- **One-time call**: The guard `env.storage().instance().has(&DataKey::Creator)` prevents re-initialization. Any subsequent call returns `Err(ContractError::AlreadyInitialized)`.
- **Creator auth**: `creator.require_auth()` is called unconditionally, ensuring the transaction must be signed by the creator's key.
- **Admin separation**: `admin` and `creator` can be different addresses. The admin is only used for contract upgrades; the creator manages the campaign lifecycle.
- **No arithmetic in initialize**: All values are stored as-is. No overflow risk at this stage.
- **Platform fee cap**: A fee above 10 000 bps would allow the platform to drain the entire campaign. The hard cap prevents misconfiguration.
- **Bonus goal ordering**: Enforcing `bonus_goal > goal` prevents a bonus goal that is already met at launch.

---

## Example — CLI invocation

```bash
DEADLINE=$(date -d "+30 days" +%s)

stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  --source <CREATOR_SECRET_KEY> \
  -- initialize \
  --admin   <ADMIN_ADDRESS> \
  --creator <CREATOR_ADDRESS> \
  --token   <TOKEN_CONTRACT_ADDRESS> \
  --goal    1000000000 \
  --deadline $DEADLINE \
  --min_contribution 1000000
```

> Amounts are in stroops (1 XLM = 10 000 000 stroops).

---

## Example — Rust integration test

```rust
use soroban_sdk::{testutils::Address as _, Address, Env};
use crowdfund::{CrowdfundContract, CrowdfundContractClient};

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, CrowdfundContract);
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let admin   = Address::generate(&env);
    let creator = Address::generate(&env);
    let token   = Address::generate(&env);

    env.ledger().set_timestamp(1_000);

    client.initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,   // goal
        &10_000,      // deadline (timestamp)
        &1_000,       // min_contribution
        &None,        // platform_config
        &None,        // bonus_goal
        &None,        // bonus_goal_description
    );

    assert_eq!(client.goal(), 1_000_000);
    assert_eq!(client.total_raised(), 0);
}

### Coverage Report

```bash
cargo test -p crowdfund crowdfund_initialize_function -- --nocapture
cargo tarpaulin --exclude-tests --out Html -p crowdfund
```

---

## Error Codes Reference

| Code | Error | Description |
|------|-------|-------------|
| 1 | `AlreadyInitialized` | Contract already initialized |
| 8 | `InvalidGoal` | Goal must be at least 1 |
| 9 | `InvalidMinContribution` | Minimum contribution must be at least 1 |
| 10 | `DeadlineTooSoon` | Deadline must be at least 60 seconds in the future |
| 11 | `InvalidPlatformFee` | Platform fee cannot exceed 100% (10,000 bps) |
| 12 | `InvalidBonusGoal` | Bonus goal must be strictly greater than primary goal |
| 13 | `InvalidBonusGoalDescription` | Bonus goal description exceeds maximum length |
## Validation helper

The `crowdfund_initialize_function` module exposes a standalone validation function that can be used before calling `initialize`:

```rust
use crowdfund::crowdfund_initialize_function::{
    validate_initialization_params, InitError,
};

let result = validate_initialization_params(
    &env,
    goal,
    deadline,
    min_contribution,
    Some(fee_bps),   // platform fee in bps, or None
    Some(bonus),     // bonus goal, or None
);

match result {
    Ok(()) => { /* safe to call initialize */ }
    Err(InitError::GoalNotPositive)            => { /* handle */ }
    Err(InitError::DeadlineInPast)             => { /* handle */ }
    Err(InitError::MinContributionNotPositive) => { /* handle */ }
    Err(InitError::MinContributionExceedsGoal) => { /* handle */ }
    Err(InitError::PlatformFeeExceedsMax)      => { /* handle */ }
    Err(InitError::BonusGoalNotGreaterThanGoal)=> { /* handle */ }
}
```

### `InitError` variants

| Variant                        | Condition                                  |
|--------------------------------|--------------------------------------------|
| `GoalNotPositive`              | `goal <= 0`                                |
| `DeadlineInPast`               | `deadline <= current_ledger_timestamp`     |
| `MinContributionNotPositive`   | `min_contribution <= 0`                    |
| `MinContributionExceedsGoal`   | `min_contribution > goal`                  |
| `PlatformFeeExceedsMax`        | `fee_bps > 10_000`                         |
| `BonusGoalNotGreaterThanGoal`  | `bonus_goal <= goal`                       |

---

## Related functions

| Function          | Description                                              |
|-------------------|----------------------------------------------------------|
| `contribute`      | Pledge tokens to the active campaign.                    |
| `withdraw`        | Creator claims funds after a successful campaign.        |
| `refund_single`   | Contributor reclaims tokens if the goal was not met.     |
| `upgrade`         | Admin replaces the contract WASM without changing state. |
| `update_metadata` | Creator updates title, description, or social links.     |
## Error Codes Reference

| Code | Error | Description |
|------|-------|-------------|
| 1 | `AlreadyInitialized` | Contract already initialized |
| 8 | `InvalidGoal` | Goal must be at least 1 |
| 9 | `InvalidMinContribution` | Minimum contribution must be at least 1 |
| 10 | `DeadlineTooSoon` | Deadline must be at least 60 seconds in the future |
| 11 | `InvalidPlatformFee` | Platform fee cannot exceed 100% (10,000 bps) |
| 12 | `InvalidBonusGoal` | Bonus goal must be strictly greater than primary goal |
| 13 | `InvalidBonusGoalDescription` | Bonus goal description exceeds maximum length |

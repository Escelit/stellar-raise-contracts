# stellar_token_minter — Crowdfund Contract Security Module

Technical reference for the Stellar Raise crowdfund smart contract security module built with Soroban SDK.
# stellar_token_minter — Crowdfund Contract

Technical reference for the Stellar Raise crowdfund smart contract security module built with Soroban SDK.

---

## Overview

The `stellar_token_minter` module provides secure token minting and pledge collection functionality for the Stellar Raise crowdfunding platform. This module contains validation functions and security checks that are used internally by the crowdfund contract.

---

## Logging Bounds

Soroban contracts run inside a metered host environment. Every event emission
and every storage read/write consumes CPU and memory instructions. Unbounded
iteration over contributor or pledger lists creates a denial-of-service vector:
a campaign with thousands of contributors could make `withdraw` or
`collect_pledges` exceed per-transaction resource limits and become permanently
un-callable.

The `stellar_token_minter` module centralises all bound-checking logic.

### Constants

| Constant | Value | Governs |
|---|---|---|
| `MAX_EVENTS_PER_TX` | 100 | Total events emitted in one transaction |
| `MAX_MINT_BATCH` | 50 | NFT mints per `withdraw` call |
| `MAX_LOG_ENTRIES` | 200 | Diagnostic log entries per transaction |

### Test Constants

All magic numbers used across the `stellar_token_minter` test suites are
extracted into named constants in `stellar_token_minter.rs`. CI/CD only needs
to update one location when campaign parameters change.

| Constant | Value | Used in |
|---|---|---|
| `TEST_GOAL` | `1_000_000` | Default campaign goal |
| `TEST_MIN_CONTRIBUTION` | `1_000` | Default minimum contribution |
| `TEST_DEADLINE_OFFSET` | `3_600` | Campaign duration (1 hour) |
| `TEST_CREATOR_BALANCE` | `100_000_000` | Creator token balance in setup |
| `TEST_CONTRIBUTOR_BALANCE` | `1_000_000` | Standard contributor balance |
| `TEST_NFT_CONTRIBUTION` | `25_000` | Per-contributor amount in NFT-batch tests |
| `TEST_NFT_SMALL_CONTRIBUTION` | `400_000` | Amount in below-batch-limit NFT test |
| `TEST_PLEDGE_CONTRIBUTION` | `300_000` | Amount in collect_pledges tests |
| `TEST_BONUS_GOAL` | `1_000_000` | Bonus goal threshold |
| `TEST_BONUS_PRIMARY_GOAL` | `500_000` | Primary goal in bonus-goal tests |
| `TEST_BONUS_CONTRIBUTION` | `600_000` | Per-contribution in bonus-goal crossing |
| `TEST_OVERFLOW_SEED` | `10_000` | Seed contribution in overflow test |
| `TEST_FEE_BPS_MAX` | `10_000` | Maximum platform fee (100%) |
| `TEST_FEE_BPS_OVER` | `10_001` | Fee that exceeds maximum (panic test) |
| `TEST_FEE_BPS_10PCT` | `1_000` | 10% platform fee |
| `TEST_JUST_BELOW_GOAL` | `999_999` | One stroop below goal |
| `TEST_PARTIAL_CONTRIBUTION_A` | `300_000` | First partial contribution |
| `TEST_PARTIAL_CONTRIBUTION_B` | `200_000` | Second partial contribution |

### Helper Functions

| Function | Description |
|---|---|
| `within_event_budget(count)` | `true` when `count < MAX_EVENTS_PER_TX` |
| `within_mint_batch(count)` | `true` when `count < MAX_MINT_BATCH` |
| `within_log_budget(count)` | `true` when `count < MAX_LOG_ENTRIES` |
| `remaining_event_budget(reserved)` | Events remaining before budget exhausted |
| `remaining_mint_budget(minted)` | NFT mints remaining in current batch |
| `emit_batch_summary(env, topic, count, emitted)` | Emits a single summary event; no-op when `count == 0` or budget exhausted |

### Design Rationale

- Limits are enforced **before** the loop that would exceed them, not after.
- All arithmetic uses `saturating_sub` / `checked_*` to prevent overflow.
- No limit can be bypassed by the caller — they are compile-time constants.
- `emit_batch_summary` replaces per-item events with a single count event,
  keeping event volume O(1) regardless of list size.
The crowdfund contract manages a single campaign lifecycle:

```
Active → Successful  (goal met, creator withdraws)
Active → Refunded    (deadline passed, goal not met)
Active → Cancelled   (creator cancels early)
```

All token amounts are in the token's smallest unit (stroops for XLM).

---

## Logging Bounds

Soroban contracts run inside a metered host environment. Every event emission
and every storage read/write consumes CPU and memory instructions. Unbounded
iteration over contributor or pledger lists creates a denial-of-service vector:
a campaign with thousands of contributors could make `withdraw` or
`collect_pledges` exceed per-transaction resource limits and become permanently
un-callable.

The `stellar_token_minter` module centralises all bound-checking logic.

### Constants

| Constant | Value | Governs |
|---|---|---|
| `MAX_EVENTS_PER_TX` | 100 | Total events emitted in one transaction |
| `MAX_MINT_BATCH` | 50 | NFT mints per `withdraw` call |
| `MAX_LOG_ENTRIES` | 200 | Diagnostic log entries per transaction |

### Test Constants

All magic numbers used across the `stellar_token_minter` test suites are
extracted into named constants in `stellar_token_minter.rs`. CI/CD only needs
to update one location when campaign parameters change.

| Constant | Value | Used in |
|---|---|---|
| `TEST_GOAL` | `1_000_000` | Default campaign goal |
| `TEST_MIN_CONTRIBUTION` | `1_000` | Default minimum contribution |
| `TEST_DEADLINE_OFFSET` | `3_600` | Campaign duration (1 hour) |
| `TEST_CREATOR_BALANCE` | `100_000_000` | Creator token balance in setup |
| `TEST_CONTRIBUTOR_BALANCE` | `1_000_000` | Standard contributor balance |
| `TEST_NFT_CONTRIBUTION` | `25_000` | Per-contributor amount in NFT-batch tests |
| `TEST_NFT_SMALL_CONTRIBUTION` | `400_000` | Amount in below-batch-limit NFT test |
| `TEST_PLEDGE_CONTRIBUTION` | `300_000` | Amount in collect_pledges tests |
| `TEST_BONUS_GOAL` | `1_000_000` | Bonus goal threshold |
| `TEST_BONUS_PRIMARY_GOAL` | `500_000` | Primary goal in bonus-goal tests |
| `TEST_BONUS_CONTRIBUTION` | `600_000` | Per-contribution in bonus-goal crossing |
| `TEST_OVERFLOW_SEED` | `10_000` | Seed contribution in overflow test |
| `TEST_FEE_BPS_MAX` | `10_000` | Maximum platform fee (100%) |
| `TEST_FEE_BPS_OVER` | `10_001` | Fee that exceeds maximum (panic test) |
| `TEST_FEE_BPS_10PCT` | `1_000` | 10% platform fee |
| `TEST_JUST_BELOW_GOAL` | `999_999` | One stroop below goal |
| `TEST_PARTIAL_CONTRIBUTION_A` | `300_000` | First partial contribution |
| `TEST_PARTIAL_CONTRIBUTION_B` | `200_000` | Second partial contribution |

### Helper Functions

| Function | Description |
|---|---|
| `within_event_budget(count)` | `true` when `count < MAX_EVENTS_PER_TX` |
| `within_mint_batch(count)` | `true` when `count < MAX_MINT_BATCH` |
| `within_log_budget(count)` | `true` when `count < MAX_LOG_ENTRIES` |
| `remaining_event_budget(reserved)` | Events remaining before budget exhausted |
| `remaining_mint_budget(minted)` | NFT mints remaining in current batch |
| `emit_batch_summary(env, topic, count, emitted)` | Emits a single summary event; no-op when `count == 0` or budget exhausted |

### Design Rationale

- Limits are enforced **before** the loop that would exceed them, not after.
- All arithmetic uses `saturating_sub` / `checked_*` to prevent overflow.
- No limit can be bypassed by the caller — they are compile-time constants.
- `emit_batch_summary` replaces per-item events with a single count event,
  keeping event volume O(1) regardless of list size.

---

## Contract Functions
## Module Functions

#### `validate_pledge_preconditions`

```rust
pub fn validate_pledge_preconditions(
    env: &Env,
    amount: i128,
    min_contribution: i128,
) -> Result<(), ContractError>
```

Validates preconditions for pledge operations.

**Security Checks:**
1. Campaign must be active (`CampaignNotActive` if not)
2. Amount must be non-zero (`ZeroAmount` if zero)
3. Amount must meet minimum (`BelowMinimum` if below)
4. Current time must be before deadline (`CampaignEnded` if past)

**Validation Order:** Status → Amount → Deadline (prevents timing-based attacks)

#### `validate_collect_preconditions`

```rust
pub fn validate_collect_preconditions(
    env: &Env,
) -> Result<(i128, i128, i128), ContractError>
```

Validates preconditions for collect_pledges operations.

- Rejects `amount == 0` → `ZeroAmount`.
- Rejects amounts below `min_contribution` → `BelowMinimum`.
- Rejects contributions after `deadline` → `CampaignEnded`.
- Uses `checked_add` on `total_raised` → `Overflow` on failure.
- Emits `("campaign", "contributed")` event.
- Fires `("campaign", "bonus_goal_reached")` **once** when `total_raised` crosses `bonus_goal`.

**Errors:** `CampaignEnded`, `ZeroAmount`, `BelowMinimum`, `Overflow`

### Arithmetic Helper Functions

#### `calculate_total_commitment`

```rust
pub fn calculate_total_commitment(
    total_raised: i128,
    total_pledged: i128,
) -> Result<i128, ContractError>
```

Safely calculates the total commitment (raised + pledged).

- Uses `checked_add` to prevent overflow
- Returns `ContractError::Overflow` if addition would overflow

#### `safe_add_pledge`

```rust
pub fn safe_add_pledge(
    current_total: i128,
    new_amount: i128,
) -> Result<i128, ContractError>
```

Pulls tokens from all pledgers after the deadline when the combined total meets
the goal. Each pledger must have pre-authorized the transfer. Emits a single
`("campaign", "pledges_collected")` summary event.
### `initialize`
### Validation Functions

#### `validate_pledge_preconditions`

```rust
pub fn validate_pledge_preconditions(
    env: &Env,
    amount: i128,
    min_contribution: i128,
) -> Result<(), ContractError>
```

Validates preconditions for pledge operations.

**Security Checks:**
1. Campaign must be active (`CampaignNotActive` if not)
2. Amount must be non-zero (`ZeroAmount` if zero)
3. Amount must meet minimum (`BelowMinimum` if below)
4. Current time must be before deadline (`CampaignEnded` if past)

**Validation Order:** Status → Amount → Deadline (prevents timing-based attacks)

#### `validate_collect_preconditions`

```rust
pub fn validate_collect_preconditions(
    env: &Env,
) -> Result<(i128, i128, i128), ContractError>
```

Validates preconditions for collect_pledges operations.

**Returns:** `(goal, total_raised, total_pledged)` on success

**Security Checks:**
1. Campaign must be active (`CampaignNotActive` if not)
2. Current time must be after deadline (`CampaignStillActive` if before)
3. Combined total must meet goal (`GoalNotReached` if below)
4. No overflow in total calculation (`Overflow` if overflow)

### Arithmetic Helper Functions

#### `calculate_total_commitment`

```rust
pub fn calculate_total_commitment(
    total_raised: i128,
    total_pledged: i128,
) -> Result<i128, ContractError>
```

Safely calculates the total commitment (raised + pledged).

- Uses `checked_add` to prevent overflow
- Returns `ContractError::Overflow` if addition would overflow

#### `safe_add_pledge`

```rust
pub fn safe_add_pledge(
    current_total: i128,
    new_amount: i128,
) -> Result<i128, ContractError>
```

Validates that a pledge amount can be safely added to existing totals.

#### `validate_contribution_amount`

```rust
pub fn validate_contribution_amount(
    amount: i128,
    min_contribution: i128,
) -> Result<(), ContractError>
```

Validates contribution amounts for security.

- Non-zero amount prevents dust transactions
- Amount >= minimum prevents spam

#### `safe_calculate_progress`

```rust
pub fn safe_calculate_progress(
    current_amount: i128,
    goal: i128,
) -> Result<u32, ContractError>
```

Safely calculates campaign progress in basis points (BPS).

- Returns progress from 0 to 10,000 (where 10,000 = 100%)
- Caps at 100% to prevent display issues
- Uses checked arithmetic for overflow protection

### Parameter Validation Functions

#### `validate_deadline`

```rust
pub fn validate_deadline(
    env: &Env,
    deadline: u64,
) -> Result<(), ContractError>
```

Validates that a deadline is in the future.

- Returns `CampaignEnded` if deadline is in the past or current
- Checks against maximum campaign duration (1 year)

#### `validate_goal`

```rust
pub fn validate_goal(goal: i128) -> Result<(), ContractError>
```

Validates that a goal amount is reasonable.

- Returns `GoalNotReached` for zero or negative goals

#### `calculate_platform_fee`

```rust
pub fn calculate_platform_fee(
    amount: i128,
    fee_bps: u32,
) -> Result<i128, ContractError>
```

Calculates platform fee safely with bounds checking.

- Fee BPS should be 0-10000
- Uses checked arithmetic

#### `validate_bonus_goal`

```rust
pub fn validate_bonus_goal(
    bonus_goal: i128,
    primary_goal: i128,
) -> Result<(), ContractError>
```

Validates bonus goal is strictly greater than primary goal.

- Returns `GoalNotReached` if bonus ≤ primary

---

## Security Features

### Authorization Enforcement

All state-changing operations require proper authentication via Soroban's `require_auth` mechanism.

- Rejects `amount == 0` → `ZeroAmount`.
- Rejects amounts below `min_contribution` → `BelowMinimum`.
- Rejects contributions after `deadline` → `CampaignEnded`.
- Uses `checked_add` on `total_raised` → `Overflow` on failure.
- Emits `("campaign", "contributed")` event.
- Fires `("campaign", "bonus_goal_reached")` **once** when `total_raised` crosses `bonus_goal`.

**Errors:** `CampaignEnded`, `ZeroAmount`, `BelowMinimum`, `Overflow`
### Overflow Protection

All arithmetic operations use `checked_*` methods:
- `checked_add` for additions
- `checked_mul` for multiplications
- `checked_div` for divisions

This prevents integer overflow attacks on financial calculations.

### State Validation

Strict validation of campaign state before operations:
1. Status check occurs first
2. Input validation follows
3. Timing checks last

This order ensures consistent error reporting and prevents state confusion attacks.

### Deadline Enforcement

Time-based guards use strict inequality comparisons:
- `timestamp > deadline` for pledge operations (deadline is exclusive)
- `timestamp <= deadline` for collection operations (must wait until after)

### Goal Verification

Ensures pledges are only collected when goals are met:
- Combined totals are atomically validated
- Overflow protection on total calculations
- Strict comparison against goal

---

## Attack Vectors Mitigated

| Attack Vector | Mitigation |
|---|---|
| Integer Overflow | All arithmetic uses `checked_*` operations |
| Deadline Bypass | Timestamp comparisons use strict inequality |
| State Confusion | Status checks occur before any modifications |
| Goal Manipulation | Combined totals atomically validated |
| Dust Attacks | Zero and minimum amount validation |
| Reentrancy | Soroban execution model is single-threaded |
| TOCTOU | Atomic reads of all values before comparison |

---

### `collect_pledges`

```rust
fn collect_pledges(env: Env) -> Result<(), ContractError>
```

Pulls tokens from all pledgers after the deadline when the combined total meets
the goal. Each pledger must have pre-authorized the transfer. Emits a single
`("campaign", "pledges_collected")` summary event.

**Errors:** `CampaignStillActive`, `GoalNotReached`

---

#### `validate_contribution_amount`

```rust
pub fn validate_contribution_amount(
    amount: i128,
    min_contribution: i128,
) -> Result<(), ContractError>
```

Creator claims raised funds after deadline when goal is met. If a
`PlatformConfig` is set, the fee is deducted first. If an NFT contract is
configured, mints up to `MAX_MINT_BATCH` NFTs (one per contributor). Emits a
single `("campaign", "nft_batch_minted")` summary event instead of one event
per contributor.

#### `safe_calculate_progress`

```rust
pub fn safe_calculate_progress(
    current_amount: i128,
    goal: i128,
) -> Result<u32, ContractError>
```

Returns all contributions when the deadline has passed and the goal was not met.

> **Deprecated** as of contract v3. Use `refund_single` instead.

### Parameter Validation Functions

---

### `refund_single`

```rust
fn refund_single(env: Env, contributor: Address) -> Result<(), ContractError>
```

Pull-based refund for a single contributor. Preferred over `refund` for gas
safety with large contributor lists.

**Errors:** `CampaignStillActive`, `GoalReached`, `NothingToRefund`
### `withdraw`

```rust
fn withdraw(env: Env) -> Result<(), ContractError>
```

Creator claims raised funds after deadline when goal is met. If a
`PlatformConfig` is set, the fee is deducted first. If an NFT contract is
configured, mints up to `MAX_MINT_BATCH` NFTs (one per contributor). Emits a
single `("campaign", "nft_batch_minted")` summary event instead of one event
per contributor.

**Errors:** `CampaignStillActive`, `GoalNotReached`

---

### `refund`

```rust
fn refund(env: Env) -> Result<(), ContractError>
```

Returns all contributions when the deadline has passed and the goal was not met.

> **Deprecated** as of contract v3. Use `refund_single` instead.

**Errors:** `CampaignStillActive`, `GoalReached`

---

### `refund_single`

```rust
fn refund_single(env: Env, contributor: Address) -> Result<(), ContractError>
```

Pull-based refund for a single contributor. Preferred over `refund` for gas
safety with large contributor lists.

**Errors:** `CampaignStillActive`, `GoalReached`, `NothingToRefund`

---

### `cancel`

```rust
pub fn validate_deadline(
    env: &Env,
    deadline: u64,
) -> Result<(), ContractError>
```

Creator cancels the campaign early. Sets status to `Cancelled`.

**Panics:** not active, not authorized

- Returns `CampaignEnded` if deadline is in the past or current
- Checks against maximum campaign duration (1 year)

#### `validate_goal`

```rust
pub fn validate_goal(goal: i128) -> Result<(), ContractError>
```

Replaces the contract WASM without changing its address or storage. Only the
`admin` set at initialization can call this.

- Returns `GoalNotReached` for zero or negative goals

#### `calculate_platform_fee`

```rust
pub fn calculate_platform_fee(
    amount: i128,
    fee_bps: u32,
) -> Result<i128, ContractError>
```

Updates campaign metadata. Only callable by the creator while `Active`. Pass
`None` to leave a field unchanged.

- Fee BPS should be 0-10000
- Uses checked arithmetic

#### `validate_bonus_goal`

```rust
pub fn validate_bonus_goal(
    bonus_goal: i128,
    primary_goal: i128,
) -> Result<(), ContractError>
```

Configures the NFT contract used for contributor reward minting on successful
withdrawal. Only the creator can call this.

---

### `add_stretch_goal` / `add_roadmap_item`

```rust
fn add_stretch_goal(env: Env, milestone: i128)
fn add_roadmap_item(env: Env, date: u64, description: String)
```

Append stretch goals and roadmap items. Creator-only.

Strict validation of campaign state before operations:
1. Status check occurs first
2. Input validation follows
3. Timing checks last

This order ensures consistent error reporting and prevents state confusion attacks.
fn cancel(env: Env)
```

Creator cancels the campaign early. Sets status to `Cancelled`.

**Panics:** not active, not authorized

---

### `upgrade`

```rust
fn upgrade(env: Env, new_wasm_hash: BytesN<32>)
```

Replaces the contract WASM without changing its address or storage. Only the
`admin` set at initialization can call this.

---

### `update_metadata`

```rust
fn update_metadata(
    env: Env,
    creator: Address,
    title: Option<String>,
    description: Option<String>,
    socials: Option<String>,
)
```

Updates campaign metadata. Only callable by the creator while `Active`. Pass
`None` to leave a field unchanged.

---

### `set_nft_contract`

```rust
fn set_nft_contract(env: Env, creator: Address, nft_contract: Address)
```

Configures the NFT contract used for contributor reward minting on successful
withdrawal. Only the creator can call this.

---

### `add_stretch_goal` / `add_roadmap_item`

```rust
fn add_stretch_goal(env: Env, milestone: i128)
fn add_roadmap_item(env: Env, date: u64, description: String)
```

Append stretch goals and roadmap items. Creator-only.

---

## View Functions

| Function | Returns | Description |
|---|---|---|
| `total_raised` | `i128` | Total tokens contributed so far |
| `goal` | `i128` | Primary funding goal |
| `deadline` | `u64` | Campaign end timestamp |
| `min_contribution` | `i128` | Minimum contribution amount |
| `contribution(addr)` | `i128` | Contribution by a specific address |
| `contributors` | `Vec<Address>` | All contributor addresses |
| `bonus_goal` | `Option<i128>` | Optional bonus goal threshold |
| `bonus_goal_reached` | `bool` | Whether bonus goal has been met |
| `bonus_goal_progress_bps` | `u32` | Bonus goal progress in basis points (0–10,000) |
| `current_milestone` | `i128` | Next unmet stretch goal (0 if none) |
| `get_stats` | `CampaignStats` | Aggregate stats |
| `version` | `u32` | Contract version (currently 3) |

Time-based guards use strict inequality comparisons:
- `timestamp > deadline` for pledge operations (deadline is exclusive)
- `timestamp <= deadline` for collection operations (must wait until after)

### Goal Verification

Ensures pledges are only collected when goals are met:
- Combined totals are atomically validated
- Overflow protection on total calculations
- Strict comparison against goal
| `bonus_goal_description` | `Option<String>` | Bonus goal description |
| `bonus_goal_reached` | `bool` | Whether bonus goal has been met |
| `bonus_goal_progress_bps` | `u32` | Bonus goal progress in basis points (0–10,000) |
| `current_milestone` | `i128` | Next unmet stretch goal (0 if none) |
| `get_stats` | `CampaignStats` | Aggregate stats |
| `version` | `u32` | Contract version (currently 3) |

---

## Data Types

### `CampaignStats`

```rust
pub struct CampaignStats {
    pub total_raised: i128,
    pub goal: i128,
    pub progress_bps: u32,        // 0–10,000 (basis points)
    pub contributor_count: u32,
    pub average_contribution: i128,
    pub largest_contribution: i128,
}
```

### `ContractError`
## Error Codes

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | Initialize called more than once |
| 1 | `AlreadyInitialized` | `initialize` called more than once |
| 2 | `CampaignEnded` | Action attempted after deadline |
| 3 | `CampaignStillActive` | Action requires deadline to have passed |
| 4 | `GoalNotReached` | Withdraw/collect attempted when goal not met |
| 5 | `GoalReached` | Refund attempted when goal was met |
| 6 | `Overflow` | Integer overflow in calculations |
| 7 | `NothingToRefund` | Caller has no contribution to refund |
| 8 | `ZeroAmount` | Amount is zero |
| 9 | `BelowMinimum` | Amount is below minimum contribution |
| 10 | `CampaignNotActive` | Campaign is not in active state |

---

## Testing

- Test coverage target remains 95%+ lines in the crowdfund module.

## Testing and Security Notes

- Test coverage target remains 95%+ lines in the crowdfund module.
- Critical code paths covered:
  - `initialize`: repeated init, platform fee bounds, bonus goal guard.
  - `contribute`: minimum amount guard, deadline guard, aggregation, overflow protection.
  - `pledge` / `collect_pledges`: state transition and transfer effect.
  - `withdraw`: deadline, goal check, platform fee, NFT mint flow.
  - `refund`, `cancel`, `add_roadmap_item`, `add_stretch_goal`, `current_milestone`, `get_stats`, `bonus_goal`.
  - `upgrade`: admin-only authorization.
  - `stellar_token_minter.test.rs`: explicit security/readability tests for
    deadline guards, goal guards, bonus-goal capping, and upgrade auth.

### Test Categories

1. **Authorization Tests**: Verify authentication requirements
2. **Overflow Protection Tests**: Ensure arithmetic safety
3. **State Transition Tests**: Validate state machine integrity
4. **Timing Tests**: Verify deadline enforcement
5. **Goal Validation Tests**: Ensure goal requirements
6. **Edge Case Tests**: Cover boundary conditions
7. **Module Function Tests**: Unit tests for module functions
8. **Integration Tests**: End-to-end workflow tests

| 6 | `Overflow` | Integer overflow in contribution accounting |
| 7 | `NothingToRefund` | Caller has no contribution to refund |
| 8 | `ZeroAmount` | Contribution amount is zero |
| 9 | `BelowMinimum` | Contribution below `min_contribution` |
| 10 | `CampaignNotActive` | Campaign is not in `Active` status |

---

## Security Invariants

### Security assumptions

1. `creator.require_auth()` and `admin.require_auth()` provide access control in relevant calls.
2. `platform fee <= 10_000` ensures no more than 100% fees are taken.
3. `bonus_goal` strict comparison (`> goal`) prevents invalid secondary goal loops.
4. `contribute` and `collect_pledges` use `checked_add`/`checked_mul` to avoid overflow in numeric operations.
5. `status` checks in state-transition functions prevent replay / double accounting.

| 6 | `Overflow` | Integer overflow in contribution accounting |
| 7 | `NothingToRefund` | Caller has no contribution to refund |
| 8 | `ZeroAmount` | Contribution amount is zero |
| 9 | `BelowMinimum` | Contribution below `min_contribution` |
| 10 | `CampaignNotActive` | Campaign is not in `Active` status |
| 6 | `Overflow` | Integer overflow in calculations |
| 7 | `NothingToRefund` | Caller has no contribution to refund |
| 8 | `ZeroAmount` | Amount is zero |
| 9 | `BelowMinimum` | Amount is below minimum contribution |
| 10 | `CampaignNotActive` | Campaign is not in active state |

---

## Testing

Tests are located in `contracts/crowdfund/src/stellar_token_minter.test.rs`.

### Test Categories

1. **Authorization Tests**: Verify authentication requirements
2. **Overflow Protection Tests**: Ensure arithmetic safety
3. **State Transition Tests**: Validate state machine integrity
4. **Timing Tests**: Verify deadline enforcement
5. **Goal Validation Tests**: Ensure goal requirements
6. **Edge Case Tests**: Cover boundary conditions
7. **Module Function Tests**: Unit tests for module functions
8. **Integration Tests**: End-to-end workflow tests

| Topic | Data | Emitted by |
|---|---|---|
| `("campaign", "contributed")` | `(contributor, amount)` | `contribute` |
| `("campaign", "pledged")` | `(pledger, amount)` | `pledge` |
| `("campaign", "pledges_collected")` | `total_pledged` | `collect_pledges` |
| `("campaign", "bonus_goal_reached")` | `bonus_goal` | `contribute` (once) |
| `("campaign", "withdrawn")` | `(creator, payout, nft_count)` | `withdraw` |
| `("campaign", "fee_transferred")` | `(platform_addr, fee)` | `withdraw` |
| `("campaign", "nft_batch_minted")` | `minted_count` | `withdraw` |
| `("campaign", "withdrawn")` | `(creator, total)` | `withdraw` |
| `("campaign", "fee_transferred")` | `(platform_addr, fee)` | `withdraw` |
| `("campaign", "nft_batch_minted")` | `minted_count` | `withdraw` |
| `("campaign", "roadmap_item_added")` | `(date, description)` | `add_roadmap_item` |
| `("metadata_updated", creator)` | `Vec<Symbol>` of updated fields | `update_metadata` |

---

## Security Assumptions

1. `creator.require_auth()` and `admin.require_auth()` provide access control.
2. Platform fee is validated ≤ 10,000 bps (100%) at initialization.
3. Bonus goal must exceed primary goal — validated at initialization.
4. `contribute` uses `checked_add` on all numeric accumulation → `Overflow` error.
5. NFT mint loop breaks at `MAX_MINT_BATCH` — caps event emission and gas.
6. `emit_batch_summary` is a no-op when `count == 0` or budget exhausted.
7. Refunds use checks-effects-interactions: storage zeroed before token transfer.
8. No reentrancy surface: Soroban's execution model does not support reentrancy.

---

## Test Coverage

Tests live in:

- `contracts/crowdfund/src/test.rs` — functional contract tests
- `contracts/crowdfund/src/auth_tests.rs` — authorization guards
- `contracts/crowdfund/src/stellar_token_minter_test.rs` — logging bounds and minter edge cases

### stellar_token_minter_test coverage
- `contracts/crowdfund/src/test.rs` (functional)
- `contracts/crowdfund/src/auth_tests.rs` (authorization)
- `contracts/crowdfund/src/stellar_token_minter_test.rs` (minter-focused
  security/readability edge cases)

| Area | Tests |
|---|---|
| `within_event_budget` | zero, mid-range, one-below-limit, at-limit, over-limit |
| `within_mint_batch` | zero, mid-range, one-below-limit, at-limit, over-limit |
| `within_log_budget` | zero, mid-range, one-below-limit, at-limit, over-limit |
| `remaining_event_budget` | none reserved, partial, exhausted, saturates at zero |
| `remaining_mint_budget` | none minted, partial, exhausted, saturates at zero |
| `emit_batch_summary` | count==0 skip, budget-exhausted skip, normal emission |
| NFT mint batch cap | stops at MAX_MINT_BATCH, exactly at limit, below limit |
| collect_pledges summary | single event emitted, total_raised updated |
| Bonus-goal idempotency | event fires once, progress_bps capped at 10,000 |
| Overflow protection | i128::MAX contribution returns `Overflow` |
| Contribute guards | BelowMinimum, CampaignEnded, ZeroAmount |
| collect_pledges guards | CampaignStillActive, GoalNotReached |
| get_stats | empty campaign zeroes, accurate aggregates after contributions |

### Latest token-minter focused test execution

Run command:

```bash
cargo test --package crowdfund stellar_token_minter_test
```

Security notes validated by this suite:
- Deadline/goal gates prevent premature or invalid `collect_pledges`.
- Upgrade remains admin-gated.
- Bonus-goal progress is capped at 10,000 bps (100%) for UI safety.

### Latest token-minter focused test execution

Run command:

```bash
cargo test --package crowdfund stellar_token_minter_test
```

Security notes validated by this suite:
- Deadline/goal gates prevent premature or invalid `collect_pledges`.
- Upgrade remains admin-gated.
- Bonus-goal progress is capped at 10,000 bps (100%) for UI safety.

Run with:

### v1.0.0

- Initial module structure
- Core validation functions
- Basic overflow protection
## Test Coverage

Tests live in:
- `contracts/crowdfund/src/test.rs` (functional)
- `contracts/crowdfund/src/auth_tests.rs` (authorization)
- `contracts/crowdfund/src/stellar_token_minter_test.rs` (minter-focused
  security/readability edge cases)

- `contracts/crowdfund/src/test.rs` — functional contract tests
- `contracts/crowdfund/src/auth_tests.rs` — authorization guards
- `contracts/crowdfund/src/stellar_token_minter_test.rs` — logging bounds and minter edge cases

### stellar_token_minter_test coverage
- `contracts/crowdfund/src/test.rs` (functional)
- `contracts/crowdfund/src/auth_tests.rs` (authorization)
- `contracts/crowdfund/src/stellar_token_minter_test.rs` (minter-focused
  security/readability edge cases)

| Area | Tests |
|---|---|
| `within_event_budget` | zero, mid-range, one-below-limit, at-limit, over-limit |
| `within_mint_batch` | zero, mid-range, one-below-limit, at-limit, over-limit |
| `within_log_budget` | zero, mid-range, one-below-limit, at-limit, over-limit |
| `remaining_event_budget` | none reserved, partial, exhausted, saturates at zero |
| `remaining_mint_budget` | none minted, partial, exhausted, saturates at zero |
| `emit_batch_summary` | count==0 skip, budget-exhausted skip, normal emission |
| NFT mint batch cap | stops at MAX_MINT_BATCH, exactly at limit, below limit |
| collect_pledges summary | single event emitted, total_raised updated |
| Bonus-goal idempotency | event fires once, progress_bps capped at 10,000 |
| Overflow protection | i128::MAX contribution returns `Overflow` |
| Contribute guards | BelowMinimum, CampaignEnded, ZeroAmount |
| collect_pledges guards | CampaignStillActive, GoalNotReached |
| get_stats | empty campaign zeroes, accurate aggregates after contributions |

### Latest token-minter focused test execution

Run command:

```bash
cargo test --package crowdfund stellar_token_minter_test
```

Security notes validated by this suite:
- Deadline/goal gates prevent premature or invalid `collect_pledges`.
- Upgrade remains admin-gated.
- Bonus-goal progress is capped at 10,000 bps (100%) for UI safety.

### Latest token-minter focused test execution

Run command:

```bash
cargo test --package crowdfund stellar_token_minter_test
```

Security notes validated by this suite:
- Deadline/goal gates prevent premature or invalid `collect_pledges`.
- Upgrade remains admin-gated.
- Bonus-goal progress is capped at 10,000 bps (100%) for UI safety.
Tests live in multiple files for comprehensive coverage:

- `contracts/crowdfund/src/test.rs` — Functional tests
- `contracts/crowdfund/src/auth_tests.rs` — Authorization tests
- `contracts/crowdfund/src/stellar_token_minter_test.rs` — Minter-focused edge cases
- `contracts/crowdfund/src/stellar_token_minter.test.rs` — Comprehensive token minter tests (95%+ coverage)

### Token Minter Test Coverage

| Area | Tests | Coverage |
|---|---|---|
| initialize | fields stored, double-init error, platform fee bounds, zero goal, zero min contribution | 100% |
| contribute | basic, accumulation, multiple contributors, below minimum, zero amount, after deadline, non-active campaign, at minimum, at deadline | 100% |
| withdraw | success, before deadline, goal not met, with platform fee, with NFT minting, non-active campaign, at exact deadline, one second after deadline | 100% |
| set_nft_contract | success, unauthorized caller | 100% |
| get_stats | empty campaign, with contributions, progress capped, single large contributor, equal contributions | 100% |
| view functions | total_raised, goal, deadline, min_contribution, token, nft_contract, contributors | 100% |
| edge cases | large amounts, multiple withdrawals, exactly at goal, just below goal, zero platform fee, max platform fee, minimum after deadline, contributors order | 100% |

### Security Assumptions Validated

1. **Auth enforcement**: `creator.require_auth()` and `contributor.require_auth()` are called on every state-changing function. The Soroban host enforces these at the protocol level.
2. **Overflow protection**: All addition to `total_raised` and per-contributor balances uses `checked_add`, returning `ContractError::Overflow` on failure.
3. **Platform fee cap**: Fee is validated ≤ 10,000 bps (100%) at initialization.
4. **Deadline enforcement**: Contributions and withdrawals are rejected after the deadline.
5. **Goal validation**: Withdrawals only succeed when the funding goal is met.
6. **Authorization checks**: Only the creator can set the NFT contract and withdraw funds.
7. **Status transitions**: Campaign status is properly managed to prevent double operations.

Run with:
### Running Tests

```bash
# Run all stellar_token_minter tests
cargo test --package crowdfund stellar_token_minter

# Run with detailed output
cargo test --package crowdfund stellar_token_minter -- --nocapture

# Run specific test
cargo test --package crowdfund test_pledge_requires_authorization
```

### Test Coverage

| Function | Tests |
|---|---|
| `calculate_total_commitment` | Success, zero values, overflow detection, boundary values |
| `safe_add_pledge` | Success, overflow, zero addition, multiple accumulations |
| `validate_contribution_amount` | Valid, exact minimum, zero, below minimum |
| `safe_calculate_progress` | Zero goal, exact, halfway, overfunded, small amounts |
| `validate_deadline` | Future, past, exact current |
| `validate_goal` | Positive, zero, negative |
| `calculate_platform_fee` | Zero BPS, 1%, 5%, 100% |
| `validate_bonus_goal` | Valid, equal to primary, less than primary |
| `validate_pledge_preconditions` | Success, zero, below minimum, after deadline, inactive |
| `validate_collect_preconditions` | Before deadline, at deadline, goal not met, success, inactive |

---

## Integration

The module is designed to be used internally by the crowdfund contract:

```rust
use crate::stellar_token_minter;

fn pledge(env: Env, pledger: Address, amount: i128) -> Result<(), ContractError> {
    // Use module validation functions
    stellar_token_minter::validate_pledge_preconditions(
        &env,
        amount,
        min_contribution
    )?;
    
    // ... rest of pledge logic
}
```

---

## Security Invariants

The module guarantees:

1. **No Integer Overflow**: All financial calculations are overflow-safe
2. **Strict Validation Order**: Status → Inputs → Timing
3. **Atomic Reads**: All values read at once to prevent TOCTOU
4. **Consistent Error Codes**: Same errors for same failure conditions
5. **Non-zero Amounts**: Zero transactions are rejected
6. **Minimum Enforcement**: Amounts below minimum are rejected
7. **Deadline Strictness**: Deadline comparisons are always exclusive

---

## Changelog

### v2.0.0

- Added comprehensive NatSpec documentation
- Added `safe_calculate_progress` function
- Added `validate_deadline` function
- Added `validate_goal` function
- Added `calculate_platform_fee` function
- Added `validate_bonus_goal` function
- Added extensive unit tests in module
- Improved test documentation

### v1.0.0

- Initial module structure
- Core validation functions
- Basic overflow protection

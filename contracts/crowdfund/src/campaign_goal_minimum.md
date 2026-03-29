# Campaign Goal Minimum Threshold Enforcement

## Overview
This documentation outlines the enforcement of a minimum campaign goal threshold in the smart contract. The goal is to prevent campaigns from being created with unrealistic or invalid funding targets.

## Purpose
- **Prevent Spam**: Discourages the creation of low-value campaigns that could clutter the platform.
- **Improve User Trust**: Ensures that all campaigns have a minimum feasible target.
- **Security**: Rejects zero-value or trivially small goals that could be exploited.
- **UX**: Provides clear feedback when a goal is invalid.

## Contract Logic
- **Minimum Threshold**: `MIN_CAMPAIGN_GOAL = 100`.
- **Validation**: Goals below `100` are rejected with a `panic!`.
- **Zero-Value Guard**: Goals equal to `0` are rejected.
- **Authentication**: Only authenticated creators can initiate a campaign.
- **Event Emission**: Publishes a `("campaign", "created")` event upon success.

## Usage
To create a campaign, call the `create_campaign` function with:
- `creator`: The address of the person starting the campaign.
- `goal`: The target funding amount (must be >= 100).

```rust
pub fn create_campaign(env: Env, creator: Address, goal: u64) { ... }
```

## Security Considerations
- **Authentication**: `creator.require_auth()` is called to ensure the request is authorized.
- **Validation**: Strict checks for `goal < MIN_CAMPAIGN_GOAL` and `goal == 0`.
- **Safe Storage**: Goal values are validated before any state changes are committed.

## Testing Process
Tests are provided in `campaign_goal_minimum.test.rs` and cover:
- Valid goal (>= 100).
- Below minimum goal (< 100).
- Zero goal.
- Exactly at minimum goal.
- Large boundary values (`u64::MAX`).
# `campaign_goal_minimum` — Extracted Constants for Campaign Goal & Minimum Threshold Enforcement

## Overview
This documentation outlines the enforcement of a minimum campaign goal threshold in the smart contract. The goal is to prevent campaigns from being created with unrealistic or invalid funding targets.

## Purpose
- **Prevent Spam**: Discourages the creation of low-value campaigns that could clutter the platform.
- **Improve User Trust**: Ensures that all campaigns have a minimum feasible target.
- **Security**: Rejects zero-value or trivially small goals that could be exploited.
- **UX**: Provides clear feedback when a goal is invalid.

## Contract Logic
- **Minimum Threshold**: `MIN_CAMPAIGN_GOAL = 100`.
- **Validation**: Goals below `100` are rejected with a `panic!`.
- **Zero-Value Guard**: Goals equal to `0` are rejected.
- **Authentication**: Only authenticated creators can initiate a campaign.
- **Event Emission**: Publishes a `("campaign", "created")` event upon success.

## Usage
To create a campaign, call the `create_campaign` function with:
- `creator`: The address of the person starting the campaign.
- `goal`: The target funding amount (must be >= 100).

```rust
pub fn create_campaign(env: Env, creator: Address, goal: u64) { ... }
```

## Security Considerations
- **Authentication**: `creator.require_auth()` is called to ensure the request is authorized.
- **Validation**: Strict checks for `goal < MIN_CAMPAIGN_GOAL` and `goal == 0`.
- **Safe Storage**: Goal values are validated before any state changes are committed.

## Testing Process
Tests are provided in `campaign_goal_minimum.test.rs` and cover:
- Valid goal (>= 100).
- Below minimum goal (< 100).
- Zero goal.
- Exactly at minimum goal.
- Large boundary values (`u64::MAX`).

`campaign_goal_minimum` centralizes every magic number and threshold used
during campaign initialization and contribution validation into named,
documented constants.  It also exposes pure validation helpers that can be
called from the contract, from tests, and from off-chain tooling without
pulling in the full contract dependency.

## Why extract constants?

Before this module the contract contained inline literals scattered across
`initialize()`, `contribute()`, `withdraw()`, and `get_stats()`:

| Literal | Occurrences | Intent |
|---------|-------------|--------|
| `10_000` | 4 | Basis-point scale / fee cap |
| `0` | 6+ | Zero-amount guard / default |
| `60` | 1 | deadline floor |

Inline literals:

- Force reviewers to infer intent from context.
- Create silent inconsistency risk when one occurrence is updated but others are not.
- Make audits harder — a security reviewer must grep for every occurrence.

Named constants are resolved at **compile time** (zero runtime cost) and
appear in a single place, so a future change touches one line.

## Constants

| Constant | Type | Value | Description |
|----------|------|-------|-------------|
| `MIN_GOAL_AMOUNT` | `i128` | `1` | Minimum campaign goal in token units |
| `MIN_CONTRIBUTION_AMOUNT` | `i128` | `1` | Minimum value for the `min_contribution` parameter |
| `MAX_PLATFORM_FEE_BPS` | `u32` | `10_000` | Maximum platform fee (100 % in basis points) |
| `PROGRESS_BPS_SCALE` | `i128` | `10_000` | Scale factor for all basis-point progress calculations |
| `MIN_DEADLINE_OFFSET` | `u64` | `60` | Minimum seconds the deadline must be in the future |
| `MAX_PROGRESS_BPS` | `u32` | `10_000` | Cap on progress value returned to callers |

## Validation Helpers

### `validate_goal(goal: i128) -> Result<(), &'static str>`

Ensures the campaign goal is at least `MIN_GOAL_AMOUNT`.

A goal of zero would let the creator withdraw immediately after any
contribution, effectively turning the contract into a donation drain.

```rust
validate_goal(1_000_000)?; // Ok
validate_goal(0)?;          // Err("goal must be at least MIN_GOAL_AMOUNT")
```

### `validate_min_contribution(min_contribution: i128) -> Result<(), &'static str>`

Ensures the minimum contribution floor is at least `MIN_CONTRIBUTION_AMOUNT`.

A zero minimum allows zero-amount contributions that waste gas on a no-op
token transfer and pollute the contributors list.

```rust
validate_min_contribution(1_000)?; // Ok
validate_min_contribution(0)?;     // Err("min_contribution must be at least MIN_CONTRIBUTION_AMOUNT")
```

### `validate_deadline(now: u64, deadline: u64) -> Result<(), &'static str>`

Ensures the deadline is at least `MIN_DEADLINE_OFFSET` seconds in the future.

Prevents campaigns that expire before a single transaction can be submitted.
Uses `saturating_add` to avoid overflow when `now` is near `u64::MAX`.

```rust
validate_deadline(1_000, 1_060)?; // Ok  (exactly MIN_DEADLINE_OFFSET)
validate_deadline(1_000, 1_059)?; // Err("deadline must be at least MIN_DEADLINE_OFFSET seconds in the future")
```

### `validate_platform_fee(fee_bps: u32) -> Result<(), &'static str>`

Ensures the platform fee does not exceed `MAX_PLATFORM_FEE_BPS` (100 %).

A fee above 100 % would mean the platform takes more than the total raised,
leaving the creator with a negative payout.

```rust
validate_platform_fee(500)?;        // Ok  (5 %)
validate_platform_fee(10_001)?;     // Err("platform fee cannot exceed MAX_PLATFORM_FEE_BPS (100%)")
```

### `compute_progress_bps(total_raised: i128, goal: i128) -> u32`

Computes campaign progress in basis points, capped at `MAX_PROGRESS_BPS`.

```rust
compute_progress_bps(500_000, 1_000_000); // 5_000  (50 %)
compute_progress_bps(1_000_000, 1_000_000); // 10_000 (100 %, goal met)
compute_progress_bps(2_000_000, 1_000_000); // 10_000 (capped, goal exceeded)
compute_progress_bps(0, 0);               // 0      (zero-goal guard)
```

Returns `0` when `goal <= 0` to avoid division by zero.

## Security Assumptions

1. **`MIN_GOAL_AMOUNT`** — Prevents zero-goal campaigns that could be
   immediately drained by the creator.

2. **`MIN_CONTRIBUTION_AMOUNT`** — Prevents zero-amount contributions that
   waste gas and pollute the contributors list.

3. **`MAX_PLATFORM_FEE_BPS`** — Caps the platform fee at 100 % so the
   contract can never be configured to steal all contributor funds.

4. **`PROGRESS_BPS_SCALE`** — Single authoritative scale factor; using it
   everywhere prevents off-by-one errors if the scale ever changes.

5. **`MIN_DEADLINE_OFFSET`** — Ensures the campaign deadline is always in the
   future at initialization, preventing dead-on-arrival campaigns.

6. **`compute_progress_bps` cap** — Progress is capped at `MAX_PROGRESS_BPS`
   so callers always receive a value in `[0, 10_000]` regardless of how much
   the goal was exceeded.  This prevents integer overflow in downstream
   percentage calculations.

## Integration with `lib.rs`

The constants and helpers are imported where the inline literals previously
appeared:

```rust
use crate::campaign_goal_minimum::{
    MAX_PLATFORM_FEE_BPS, MAX_PROGRESS_BPS, MIN_CONTRIBUTION_AMOUNT,
    MIN_DEADLINE_OFFSET, MIN_GOAL_AMOUNT, PROGRESS_BPS_SCALE,
    compute_progress_bps, validate_deadline, validate_goal,
    validate_min_contribution, validate_platform_fee,
};
```

## Test Coverage

See [`campaign_goal_minimum_test.rs`](./campaign_goal_minimum_test.rs) for the
full test suite.  Tests cover:

- Constant value stability (regression guard)
- `PROGRESS_BPS_SCALE == MAX_PROGRESS_BPS` invariant
- `validate_goal`: minimum, above minimum, zero, negative, `i128::MIN`
- `validate_min_contribution`: floor, above floor, zero, negative, `i128::MIN`
- `validate_deadline`: exact offset, well in future, one second past offset,
  equal to now, in past, one second before offset, `u64::MAX` overflow safety
- `validate_platform_fee`: zero, typical (2.5 %), exact cap, one above cap, `u32::MAX`
- `compute_progress_bps`: zero raised, 25 %, 50 %, 99 %, exact goal, 2× goal,
  `i128::MAX` raised, zero goal guard, negative goal guard, 1-token of large goal,
  minimum goal + minimum raised, 1 bps precision

## CLI Usage

These constants are compile-time values and do not require a contract call.
Off-chain scripts can import them directly from the crate:

```bash
# Build and inspect constants
cargo build -p crowdfund
```

```typescript
// Off-chain: replicate the progress calculation
const PROGRESS_BPS_SCALE = 10_000n;
const MAX_PROGRESS_BPS = 10_000n;

function computeProgressBps(totalRaised: bigint, goal: bigint): number {
  if (goal <= 0n) return 0;
  const raw = (totalRaised * PROGRESS_BPS_SCALE) / goal;
  return Number(raw > MAX_PROGRESS_BPS ? MAX_PROGRESS_BPS : raw);
}
```

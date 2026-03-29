# Campaign Goal Minimum Threshold Enforcement — Security Refactor

## Overview

The `campaign_goal_minimum` module enforces a minimum financial goal for every
new crowdfunding campaign before it can be initialized on-chain.  It also
provides all other scalar validation helpers used by `initialize()` so that
every parameter check lives in one auditable, testable location.

### Why this enforcement is necessary

Without a minimum goal floor, the contract would accept campaigns with a goal
of zero or one token unit.  This creates several problems at scale:

- **Ledger bloat** — Each campaign occupies at least one persistent ledger
  entry.  Dust campaigns (goal ≈ 0) provide no economic value but consume the
  same storage as legitimate campaigns.
- **Immediate drain exploit** — A zero-goal campaign is "successful" the
  moment any contribution arrives.  The creator can call `withdraw()` instantly,
  turning the contract into a trivial donation drain with no accountability.
- **Spam / griefing** — Without a floor, an adversary can flood the factory
  with thousands of worthless campaigns at minimal cost, degrading indexer and
  frontend performance for all users.

Setting `MIN_GOAL_AMOUNT = 1` is deliberately permissive for test environments
while still closing the zero-goal attack surface.

---

## Constants

| Constant                  | Value  | Meaning                                          |
|---------------------------|--------|--------------------------------------------------|
| `MIN_GOAL_AMOUNT`         | 1      | Smallest accepted campaign goal (token units)    |
| `MIN_CONTRIBUTION_AMOUNT` | 1      | Smallest accepted `min_contribution`             |
| `MIN_DEADLINE_OFFSET`     | 60     | Seconds a deadline must be ahead of `now`        |
| `MAX_PLATFORM_FEE_BPS`    | 10_000 | 100 % in basis points — hard fee cap             |
| `PROGRESS_BPS_SCALE`      | 10_000 | Denominator for basis-point progress             |
| `MAX_PROGRESS_BPS`        | 10_000 | Maximum value returned by `compute_progress_bps` |

All constants are defined in `contracts/crowdfund/src/campaign_goal_minimum.rs`
and baked into the WASM binary at compile time.

### Updating a threshold

Because constants are baked into the WASM binary, changing them requires a
contract upgrade:

1. Update the constant in `campaign_goal_minimum.rs`.
2. Build the new WASM:
   ```bash
   cargo build --release --target wasm32-unknown-unknown -p crowdfund
   ```
3. Upload and upgrade via the admin mechanism (see
   `contracts/crowdfund/admin_upgrade_mechanism.md`).
   `docs/admin_upgrade_mechanism.md`).

> **Governance note** — If the project adopts on-chain governance, thresholds
> can be moved to contract storage (e.g. `DataKey::MinGoalAmount`) and updated
> via a governance proposal without a full upgrade.  The `validate_goal_amount`
> function signature already accepts `&Env` for exactly this future extension.

---

The `campaign_goal_minimum` module enforces minimum thresholds for all campaign
creation parameters: goal amount, minimum contribution, deadline, and platform
fee. It also provides a progress-in-basis-points helper used by the frontend
and off-chain indexers.

This refactor fixes a broken source file (duplicate functions, incomplete
`create_campaign` stub, mismatched types) and consolidates all threshold
validation into clean, independently testable, `#[inline]` pure functions.

---

## Security Rationale

| Threat | Mitigation |
|--------|-----------|
| Zero-goal drain | `MIN_GOAL_AMOUNT = 1` rejects goals that make a campaign immediately "successful" after any contribution |
| Ledger spam | Non-zero minimum prevents dust campaigns that waste storage |
| Negative goal | Single `i128` comparison rejects all values < 1 without a separate branch |
| Fee overflow | `MAX_PLATFORM_FEE_BPS = 10_000` caps fee at 100% — above this, fee transfer would exceed total raised |
| Deadline bypass | `MIN_DEADLINE_OFFSET = 60s` ensures campaigns run long enough for contributors to participate |
| Progress overflow | `compute_progress_bps` uses `saturating_mul` and caps at `MAX_PROGRESS_BPS` |
| Division by zero | `compute_progress_bps` returns 0 when `goal <= 0` |

---

## Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `MIN_GOAL_AMOUNT` | `1i128` | Minimum campaign goal in token units |
| `MIN_CONTRIBUTION_AMOUNT` | `1i128` | Minimum `min_contribution` value |
| `MAX_PLATFORM_FEE_BPS` | `10_000u32` | Maximum platform fee (100%) |
| `PROGRESS_BPS_SCALE` | `10_000i128` | Scale factor for progress calculation |
| `MAX_PROGRESS_BPS` | `10_000u32` | Maximum progress value (= 100%) |
| `MIN_DEADLINE_OFFSET` | `60u64` | Minimum seconds deadline must be in the future |

---

## API Reference

### `validate_goal(goal: i128) -> Result<(), &'static str>`

Off-chain / tooling helper.  Returns a descriptive string error instead of
`ContractError` to avoid pulling in the full contract dependency.

### `validate_goal_amount(_env: &Env, goal_amount: i128) -> Result<(), ContractError>`

On-chain enforcement entry point.  Returns `ContractError::GoalTooLow` (code 18)
when `goal_amount < MIN_GOAL_AMOUNT`.

### `validate_min_contribution(min_contribution: i128) -> Result<(), &'static str>`

Validates that `min_contribution >= MIN_CONTRIBUTION_AMOUNT`.

### `validate_deadline(now: u64, deadline: u64) -> Result<(), &'static str>`

Validates that `deadline >= now + MIN_DEADLINE_OFFSET`.  Uses `saturating_add`
Off-chain / tooling helper. Returns a descriptive string error to avoid
pulling in `ContractError`.

### `validate_goal_amount(_env: &Env, goal_amount: i128) -> Result<(), ContractError>`

On-chain enforcement entry point. Returns `ContractError::GoalTooLow` when
`goal_amount < MIN_GOAL_AMOUNT`. The `_env` parameter is reserved for future
governance-controlled thresholds stored in contract storage.

### `validate_min_contribution(min_contribution: i128) -> Result<(), &'static str>`

Rejects `min_contribution < MIN_CONTRIBUTION_AMOUNT`.

### `validate_deadline(now: u64, deadline: u64) -> Result<(), &'static str>`

Rejects deadlines less than `now + MIN_DEADLINE_OFFSET`. Uses `saturating_add`
to prevent overflow when `now` is near `u64::MAX`.

### `validate_platform_fee(fee_bps: u32) -> Result<(), &'static str>`

Validates that `fee_bps <= MAX_PLATFORM_FEE_BPS`.

### `compute_progress_bps(total_raised: i128, goal: i128) -> u32`

Returns campaign progress in basis points (0–10 000).  Guards against
division by zero and caps at `MAX_PROGRESS_BPS` for over-funded campaigns.
Rejects `fee_bps > MAX_PLATFORM_FEE_BPS`.

### `compute_progress_bps(total_raised: i128, goal: i128) -> u32`

Returns `(total_raised * 10_000) / goal`, capped at `MAX_PROGRESS_BPS`.
Returns 0 when `goal <= 0`.

---

## Integration

### On-chain usage in `initialize()`
Call `validate_goal_amount` inside `initialize()` before any storage writes:

```rust
use crate::campaign_goal_minimum::validate_goal_amount;

pub fn initialize(env: Env, goal: i128, /* … */) -> Result<(), ContractError> {
    // Reject below-threshold goals atomically — no side-effects on failure.
    validate_goal_amount(&env, goal)?;
    // … rest of initialization
pub fn initialize(env: Env, goal: i128, ...) -> Result<(), ContractError> {
    validate_goal_amount(&env, goal)?;
    // ... rest of initialization
    Ok(())
}
```

The `?` operator propagates `ContractError::GoalTooLow` to the caller without
any additional boilerplate.

### Off-chain / SDK usage

```rust
use crowdfund::campaign_goal_minimum::validate_goal;

validate_goal(proposed_goal).map_err(|e| anyhow::anyhow!(e))?;
```

### TypeScript / JavaScript (Stellar SDK)

```typescript
const ERROR_CODES: Record<number, string> = {
  18: "Campaign goal is below the minimum threshold",
};

try {
  await contract.initialize({ goal, /* … */ });
} catch (e) {
  const code = e?.errorCode as number | undefined;
  console.error(code ? ERROR_CODES[code] ?? "Unknown error" : e);
}
```

---

## Security Assumptions

| Assumption | Detail |
|---|---|
| **Dust campaign prevention** | `MIN_GOAL_AMOUNT >= 1` ensures every campaign has a non-trivial economic commitment, preventing ledger-entry spam. |
| **No integer overflow** | The validation is a single signed comparison (`goal_amount < MIN_GOAL_AMOUNT`). No arithmetic is performed, so overflow is impossible regardless of the input value. |
| **Negative goal rejection** | `i128` can represent negative values. The `< MIN_GOAL_AMOUNT` check (where `MIN_GOAL_AMOUNT = 1`) rejects all negative and zero goals without a separate branch. |
| **Atomic rejection** | `validate_goal_amount` is called before any `env.storage()` writes in `initialize()`. A rejected goal produces no ledger mutations — the transaction reverts cleanly. |
| **Upgrade safety** | All contract storage and state persist across WASM upgrades. Raising `MIN_GOAL_AMOUNT` in a new binary does not affect already-initialized campaigns; it only applies to new `initialize()` calls. |
| **Discriminant stability** | `ContractError::GoalTooLow = 18` is stable across upgrades. Off-chain scripts that map numeric codes to messages will continue to work. |

---

## Test Coverage

See [`contracts/crowdfund/src/campaign_goal_minimum.test.rs`](../contracts/crowdfund/src/campaign_goal_minimum.test.rs).

Run with:

```bash
cargo test --package crowdfund campaign_goal_minimum
```

### Test matrix

| Area | Test | Input | Expected |
|---|---|---|---|
| Constants | `constants_have_expected_values` | — | All values match |
| Constants | `progress_scale_equals_max_progress_bps` | — | Equal |
| `validate_goal` | `accepts_minimum` | `MIN_GOAL_AMOUNT` | `Ok(())` |
| `validate_goal` | `accepts_one_above_minimum` | `MIN_GOAL_AMOUNT + 1` | `Ok(())` |
| `validate_goal` | `accepts_large_value` | `1_000_000_000` | `Ok(())` |
| `validate_goal` | `accepts_i128_max` | `i128::MAX` | `Ok(())` |
| `validate_goal` | `rejects_zero` | `0` | `Err` mentioning `MIN_GOAL_AMOUNT` |
| `validate_goal` | `rejects_negative_one` | `-1` | `Err` |
| `validate_goal` | `rejects_i128_min` | `i128::MIN` | `Err` |
| `validate_min_contribution` | `accepts_floor` | `MIN_CONTRIBUTION_AMOUNT` | `Ok(())` |
| `validate_min_contribution` | `rejects_zero` | `0` | `Err` mentioning `MIN_CONTRIBUTION_AMOUNT` |
| `validate_min_contribution` | `rejects_negative_one` | `-1` | `Err` |
| `validate_min_contribution` | `rejects_i128_min` | `i128::MIN` | `Err` |
| `validate_deadline` | `accepts_exact_offset` | `now + 60` | `Ok(())` |
| `validate_deadline` | `accepts_well_in_future` | `now + 3600` | `Ok(())` |
| `validate_deadline` | `rejects_one_before_offset` | `now + 59` | `Err` |
| `validate_deadline` | `rejects_equal_to_now` | `now` | `Err` |
| `validate_deadline` | `rejects_past` | `now - 1` | `Err` |
| `validate_deadline` | `saturating_add_prevents_overflow` | `now = u64::MAX - 10` | No panic |
| `validate_platform_fee` | `accepts_zero` | `0` | `Ok(())` |
| `validate_platform_fee` | `accepts_typical` | `250` | `Ok(())` |
| `validate_platform_fee` | `accepts_exact_cap` | `10_000` | `Ok(())` |
| `validate_platform_fee` | `rejects_one_above_cap` | `10_001` | `Err` mentioning `MAX_PLATFORM_FEE_BPS` |
| `validate_platform_fee` | `rejects_u32_max` | `u32::MAX` | `Err` |
| `compute_progress_bps` | `zero_raised` | `(0, 1_000_000)` | `0` |
| `compute_progress_bps` | `half_goal` | `(500_000, 1_000_000)` | `5_000` |
| `compute_progress_bps` | `exact_goal` | `(1_000_000, 1_000_000)` | `10_000` |
| `compute_progress_bps` | `over_goal_capped` | `(2_000_000, 1_000_000)` | `10_000` |
| `compute_progress_bps` | `massively_over_goal` | `(i128::MAX, 1)` | `10_000` |
| `compute_progress_bps` | `zero_goal` | `(1_000, 0)` | `0` (no panic) |
| `compute_progress_bps` | `negative_goal` | `(1_000, -1)` | `0` |
| `validate_goal_amount` | `accepts_exact_threshold` | `MIN_GOAL_AMOUNT` | `Ok(())` |
| `validate_goal_amount` | `accepts_well_above` | `1_000_000_000` | `Ok(())` |
| `validate_goal_amount` | `rejects_below_threshold` | `MIN_GOAL_AMOUNT - 1` | `Err(GoalTooLow)` |
| `validate_goal_amount` | `rejects_zero` | `0` | `Err(GoalTooLow)` |
| `validate_goal_amount` | `rejects_negative_one` | `-1` | `Err(GoalTooLow)` |
| `validate_goal_amount` | `rejects_i128_min` | `i128::MIN` | `Err(GoalTooLow)` |
| `validate_goal_amount` | `is_idempotent` | `100` twice | Both `Ok(())` |
| `ContractError` | `goal_too_low_discriminant_is_stable` | — | `GoalTooLow as u32 == 18` |
- **Atomic rejection** — validators are called before any `env.storage()` writes.
- **No integer overflow** — all validators use comparisons only; `compute_progress_bps` uses `saturating_mul`.
- **Upgrade safety** — raising `MIN_GOAL_AMOUNT` in a new binary only affects new `initialize()` calls.
- **Governance extension** — `validate_goal_amount` accepts `&Env` for future on-chain threshold storage.

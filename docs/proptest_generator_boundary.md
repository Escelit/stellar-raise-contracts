# Proptest Generator Boundary Conditions

## Overview

This document describes the boundary constants and validation helpers used by
proptest generators for the Stellar Raise crowdfund contract. Correct boundaries
ensure property-based tests are stable, secure, and produce data suitable for
frontend UI display.
This document describes the boundary conditions and constants used by proptest generators for the Stellar Raise crowdfund contract. Correct boundaries ensure property-based tests are stable, secure, and produce data suitable for frontend UI display.

### Typo Fix: Deadline Offset Minimum

**Issue**: The minimum deadline offset was previously documented as **100 seconds**, which:

- Caused proptest regression failures (see `contracts/crowdfund/proptest-regressions/test.txt`)
- Produced flaky tests due to timing races
- Led to poor frontend UX (countdown display flickering for very short campaigns)

**Fix**: The minimum is now **1,000 seconds** (~17 minutes), providing:

- Stable property-based tests
- Meaningful campaign duration for UI display
- Consistent behavior across CI and local runs

---

## Boundary Constants

| Constant | Value | Description |
|---|---|---|
| `DEADLINE_OFFSET_MIN` | 1 000 | Minimum seconds from `now` to deadline (~17 min) |
| `DEADLINE_OFFSET_MAX` | 1 000 000 | Maximum seconds from `now` to deadline (~11.5 days) |
| `GOAL_MIN` | 1 000 | Minimum goal in stroops |
| `GOAL_MAX` | 100 000 000 | Maximum goal for proptest generation |
| `MIN_CONTRIBUTION_FLOOR` | 1 | Absolute minimum `min_contribution` |
| `PROGRESS_BPS_CAP` | 10 000 | Basis-point cap for progress display (100 %) |
| `FEE_BPS_CAP` | 10 000 | Basis-point cap for platform fees (100 %) |
| `PROPTEST_CASES_MIN` | 32 | Minimum proptest cases per property test |
| `PROPTEST_CASES_MAX` | 256 | Maximum proptest cases per property test |
| `GENERATOR_BATCH_MAX` | 512 | Maximum generator output batch size |
| `MAX_TOKEN_DECIMALS` | 18 | Maximum token decimal precision safe for JS display |
| `DEADLINE_ENDING_SOON_THRESHOLD` | 3 600 | Seconds below which the UI shows "Ending Soon" |

---

## Typo Fix: Deadline Offset Minimum

**Issue**: The minimum deadline offset was previously documented as **100 seconds**, which:

- Caused proptest regression failures (timing races in CI)
- Produced flickering countdown displays in the frontend UI for very short campaigns

**Fix**: The minimum is now **1 000 seconds** (~17 minutes), providing:

- Stable property-based tests with no timing races
- Meaningful campaign duration for UI countdown display
- Consistent behaviour across CI and local runs

---

## Pure Validation Helpers

### `is_valid_deadline_offset(offset: u64) -> bool`

Returns `true` if `offset ∈ [DEADLINE_OFFSET_MIN, DEADLINE_OFFSET_MAX]`.

**Security**: Rejects values that cause timestamp overflow or campaigns too short
for meaningful UI display.

### `is_valid_goal(goal: i128) -> bool`

Returns `true` if `goal ∈ [GOAL_MIN, GOAL_MAX]`.

**Frontend**: Prevents `goal == 0`, which causes division-by-zero in progress
percentage calculations and breaks the progress bar.

### `is_valid_min_contribution(min_contribution: i128, goal: i128) -> bool`

Returns `true` if `min_contribution ∈ [MIN_CONTRIBUTION_FLOOR, goal]`.

**Contract invariant**: `min_contribution` must not exceed `goal`, otherwise the
campaign is permanently un-fundable.
|----------|-------|-------------|
| `DEADLINE_OFFSET_MIN` | 1,000 | Minimum seconds from `now` to deadline |
| `DEADLINE_OFFSET_MAX` | 1,000,000 | Maximum seconds from `now` to deadline |
| `GOAL_MIN` | 1,000 | Minimum goal (stroops) |
| `GOAL_MAX` | 100,000,000 | Maximum goal for proptest (100M stroops) |
| `MIN_CONTRIBUTION_FLOOR` | 1 | Minimum allowed min_contribution |
| `PROGRESS_BPS_CAP` | 10,000 | Basis points cap (100%) |
| `FEE_BPS_CAP` | 10,000 | Platform fee cap (100%) |

---

## Typo Fix: Deadline Offset Minimum

**Issue**: The minimum deadline offset was previously documented as **100 seconds**, which:

- Caused proptest regression failures (timing races in CI)
- Produced flickering countdown displays in the frontend UI for very short campaigns

**Fix**: The minimum is now **1 000 seconds** (~17 minutes), providing:

- Stable property-based tests with no timing races
- Meaningful campaign duration for UI countdown display
- Consistent behaviour across CI and local runs

---

## Pure Validation Helpers

These are standalone `pub fn` (no `Env`) exported from `proptest_generator_boundary.rs`
for direct use in `#[cfg(test)]` proptest blocks.

### `is_valid_deadline_offset(offset: u64) -> bool`

Returns `true` if `offset ∈ [DEADLINE_OFFSET_MIN, DEADLINE_OFFSET_MAX]`.

**Security**: Rejects values that cause timestamp overflow or campaigns too short
for meaningful UI display.

### `is_valid_goal(goal: i128) -> bool`

Returns `true` if `goal ∈ [GOAL_MIN, GOAL_MAX]`.

**Frontend**: Prevents `goal == 0`, which causes division-by-zero in progress
percentage calculations and breaks the progress bar.

### `is_valid_min_contribution(min_contribution: i128, goal: i128) -> bool`

Returns `true` if `min_contribution ∈ [MIN_CONTRIBUTION_FLOOR, goal]`.

**Contract invariant**: `min_contribution` must not exceed `goal`, otherwise the
campaign is permanently un-fundable.

### `is_valid_contribution_amount(amount: i128, min_contribution: i128) -> bool`

Returns `true` if `amount >= min_contribution`.

### `clamp_progress_bps(raw: i128) -> u32`

Clamps a raw basis-point value to `[0, PROGRESS_BPS_CAP]`.

**Frontend**: Ensures the progress bar never renders above 100 % for over-funded
campaigns, and never renders a negative value.

### `compute_progress_bps(raised: i128, goal: i128) -> u32`

Computes `(raised * 10_000) / goal`, clamped to `[0, 10_000]`.

Uses `saturating_mul` to prevent overflow. Returns `0` when `goal <= 0`.

### `clamp_proptest_cases(requested: u32) -> u32`

Clamps a requested case count to `[PROPTEST_CASES_MIN, PROPTEST_CASES_MAX]`.

---

## New Edge-Case Helpers (Issue #423)

### `is_ui_displayable_progress(bps: u32) -> bool`

Returns `true` if `bps ∈ [0, PROGRESS_BPS_CAP]`.

**Frontend UX**: Rejects any value that would cause the progress bar to render
above 100 % or produce a broken visual state. Always call this before passing
a bps value to the frontend progress component.

**Security**: Values above `PROGRESS_BPS_CAP` would overflow a CSS `width`
percentage and mislead contributors about campaign status.

### `compute_display_percent(bps: u32) -> u32`

Converts basis points to a display percentage scaled by 100 (range `[0, 10_000]`).

Divide the result by 100 to get the human-readable percentage string:

```
bps = 5_000  →  compute_display_percent(5_000) = 5_000  →  "50.00 %"
bps = 10_001 →  clamped to 10_000               →  "100.00 %"
```

**Frontend UX**: Avoids floating-point arithmetic on-chain while giving the
frontend enough precision to render two decimal places.

### `is_contribution_ui_safe(amount: i128, min_contribution: i128, token_decimals: u32) -> bool`

Returns `true` when:

1. `amount >= min_contribution`
2. `token_decimals <= MAX_TOKEN_DECIMALS`
3. `amount * 10^token_decimals` does not overflow `i128`

**Frontend UX**: Prevents the token-amount display layer from losing precision
when converting raw stroops to a human-readable decimal string. JavaScript
`Number` loses integer precision above 2^53; this guard keeps amounts within
safe bounds.

**Security**: Rejects `token_decimals > 18` to prevent silent precision loss
in the frontend display layer.

### `deadline_ui_state(seconds_remaining: u64) -> DeadlineUiState`

Classifies a remaining-seconds value into one of three frontend visual states:

| State | Condition | UI Behaviour |
|---|---|---|
| `Expired` | `seconds_remaining == 0` | Campaign ended banner |
| `EndingSoon` | `0 < seconds_remaining <= 3 600` | Amber countdown banner |
| `Active` | `seconds_remaining > 3 600` | Normal countdown display |

**Frontend UX**: Drives the campaign card's countdown component. The `EndingSoon`
state triggers an amber visual treatment to encourage last-minute contributions.

**Security**: Treats `seconds_remaining == 0` as `Expired` regardless of clock
skew, preventing the UI from showing an active campaign after the on-chain
deadline has passed.

### `compute_net_payout(total: i128, fee_bps: u32) -> Option<i128>`

Computes the creator's net payout after platform fee deduction.

Returns `None` when:
- `fee_bps > FEE_BPS_CAP` (invalid fee configuration)

Returns `Some(0)` when:
- `total <= 0`

Otherwise returns `Some(total - fee)` where `fee = total * fee_bps / 10_000`.

**Security**: Uses `checked_mul` and `checked_sub` to prevent overflow and
underflow. Rejects `fee_bps > FEE_BPS_CAP` before any arithmetic to prevent
a misconfigured fee from silently producing a wrong payout amount.

---

## On-Chain Contract

`ProptestGeneratorBoundary` exposes all constants and validation logic on-chain
so off-chain scripts can query current platform limits without hard-coding them.

All contract methods are pure (read-only) and do not modify state.
Clamps raw progress to `[0, PROGRESS_BPS_CAP]`.

**Frontend**: Ensures the progress bar never renders above 100 % for over-funded
campaigns, and never renders a negative value.

### `compute_progress_bps(raised: i128, goal: i128) -> u32`

Computes `(raised * 10_000) / goal`, clamped to `[0, 10_000]`.

Uses `saturating_mul` to prevent overflow. Returns `0` when `goal <= 0`.

### `clamp_proptest_cases(requested: u32) -> u32`

Clamps a requested case count to `[PROPTEST_CASES_MIN, PROPTEST_CASES_MAX]`.

---

## On-Chain Contract

`ProptestGeneratorBoundary` exposes the same constants and validation logic
on-chain so off-chain scripts can query current platform limits without
hard-coding them.

```rust
use crate::proptest_generator_boundary::{
    ProptestGeneratorBoundary, ProptestGeneratorBoundaryClient,
};
```

All contract methods are pure (read-only) and do not modify state.

---

## Security Assumptions

1. **Overflow**: Goals and contributions are bounded well below `i128::MAX`,
   eliminating integer-overflow risk in fee and progress arithmetic.
2. **Division by zero**: `compute_progress_bps` guards against `goal == 0`
   before dividing.
3. **Timestamp validity**: `DEADLINE_OFFSET_MIN` prevents campaigns so short
   they cause timing races; `DEADLINE_OFFSET_MAX` prevents unreasonably
   far-future deadlines.
4. **Basis points**: `PROGRESS_BPS_CAP` and `FEE_BPS_CAP` are both 10 000,
   ensuring frontend displays never exceed 100 %.
5. **Token decimals**: `MAX_TOKEN_DECIMALS = 18` prevents JavaScript Number
   precision loss in the frontend display layer.
6. **Net payout**: `compute_net_payout` returns `None` for invalid fee
   configurations, forcing callers to handle the error explicitly.
7. **Immutable constants**: All limits are compile-time constants and cannot
   be mutated at runtime.

---

## Test Coverage

| Category | File |
|---|---|
| On-chain contract tests | `proptest_generator_boundary.test.rs` |
| Standalone property tests | `proptest_generator_boundary_tests.rs` |

Coverage targets:

- Unit tests for every constant and validator
- Property tests for valid/invalid ranges (256 cases each)
- Edge-case regression seeds
- Frontend UX edge cases (0 %, 100 %, over-funded, ending-soon, expired)
- New Issue #423 edge cases fully covered

---

## Running Tests

```bash
# Run all boundary tests
cargo test -p crowdfund proptest_generator_boundary

# Run with higher case count
PROPTEST_CASES=1000 cargo test -p crowdfund proptest_generator_boundary
```
1. **Overflow**: Goals and contributions are bounded to reduce overflow risk.
2. **Division by zero**: `goal > 0` and `bonus_goal > 0` enforced where division occurs.
3. **Timestamp validity**: Deadline offsets exclude past and unreasonably large values.
4. **Basis points**: `progress_bps` and `fee_bps` capped at 10,000 (100%).

---

## Regression Seeds

The following inputs previously caused failures and are pinned as regression tests:

| Seed | Issue |
|---|---|
| `goal=1_000_000, offset=100` | Flaky timing race; 100 now rejected |
| `goal=2_000_000, offset=100, amount=100_000` | Same root cause |

---

The following seeds in `proptest-regressions/test.txt` motivated the boundary fixes:

- `goal = 1_000_000`, `deadline_offset = 100` → Flaky; 100 now rejected.
- `goal = 2_000_000`, `deadline_offset = 100`, `contribution_amount = 100_000` → Same.

---

## Test Coverage

| Category | File |
|---|---|
| On-chain contract tests | `proptest_generator_boundary.test.rs` |
| Standalone property tests | `proptest_generator_boundary_tests.rs` |

Coverage targets:

- Unit tests for every constant and validator
- Property tests for valid/invalid ranges (256 cases each)
- Edge-case regression seeds
- Frontend UX edge cases (0 %, 100 %, over-funded)

---

## Running Tests

```bash
# Run all boundary tests
cargo test -p crowdfund proptest_generator_boundary

# Run with higher case count
PROPTEST_CASES=1000 cargo test -p crowdfund proptest_generator_boundary
```

---

## Regression Seeds

| Seed | Issue |
|---|---|
| `goal=1_000_000, offset=100` | Flaky timing race; 100 now rejected |
| `goal=2_000_000, offset=100, amount=100_000` | Same root cause |
| `raised=200_000_000, goal=100_000_000` | Over-funded must cap at 100 % |
| `fee_bps=10_001, total=1_000` | Invalid fee must return None / 0 |
| `seconds_remaining=0` | Must be Expired, not EndingSoon |
| `token_decimals=19, amount=1_000` | Excess decimals must be rejected |

---

## References

- [Proptest Book](https://altsysrq.github.io/proptest-book/)
- [Soroban Testing](https://soroban.stellar.org/docs/learn/testing)
- Contract source: `contracts/crowdfund/src/proptest_generator_boundary.rs`
- Contract: `contracts/crowdfund/src/lib.rs`

# Campaign goal minimum threshold enforcement

This document describes the **`campaign_goal_minimum`** module: constants, validation
helpers, and how they integrate with campaign initialization. It is written for
auditors, integrators, and reviewers.

## Purpose

- Enforce a **positive minimum funding goal** so campaigns cannot be created with
  `goal = 0` (which trivializes success semantics and enables abuse patterns).
- Provide **shared constants** for minimum contribution, deadline horizon, and
  platform fee caps so rules stay consistent across the crate.
- Expose **pure helpers** for progress in basis points (UI / analytics).

## API overview

| Symbol | Role |
|--------|------|
| `MIN_GOAL_AMOUNT` | Floor for `goal` (`i128`, currently `1` token base unit). |
| `validate_goal_amount` | **Canonical** on-chain check → `ContractError::GoalTooLow` if below floor. |
| `validate_goal` | **Deprecated** string-error variant; same predicate as `validate_goal_amount`. |
| `validate_min_contribution` | Floor for minimum contribution. |
| `validate_deadline` | Ensures deadline ≥ `now + MIN_DEADLINE_OFFSET` (saturating). |
| `validate_platform_fee` | Ensures `fee_bps ≤ MAX_PLATFORM_FEE_BPS`. |
| `compute_progress_bps` | Progress 0…10_000 bps; guards non-positive inputs; uses `saturating_mul` before dividing so huge `total_raised` cannot overflow; caps at 100 %. |

## Initialization path (`initialize`)

`crowdfund_initialize_function::validate_init_params` calls **`validate_goal_amount`**
for the goal floor, then maps **`GoalTooLow` → `InvalidGoal`** so existing clients
that expect error code **`8`** (`InvalidGoal`) on bad goals are unchanged.

Direct callers of `validate_goal_amount` elsewhere may still observe **`13`**
(`GoalTooLow`) where that mapping is not applied.

## Security assumptions

1. **No zero goal** — `goal < 1` is rejected before campaign state is written.
2. **Signed comparison only** for the goal floor — no overflow on check.
3. **Deadline math** uses `u64::saturating_add` so large `now` cannot wrap.
4. **Progress** returns `0` for non-positive `total_raised` or `goal`; uses `saturating_mul` for the `raised × scale` step (matches `stream_processing_optimization`) and caps at 100 % bps.
5. **Constants are compile-time** — changing floors requires a new WASM deploy.

Responses from health checks and user-facing layers **must not** leak internal
panic strings; this module’s `ContractError` paths are preferred for on-chain
entry points.

## Deprecations

- **`validate_goal`** — use `validate_goal_amount(&env, goal)` and map errors as
  needed for your entry point.
- Removed legacy **`create_campaign`** / **`MIN_CAMPAIGN_GOAL`** (panic-based,
  `u64` goal) — unused by the factory; initialization flows through
  `CrowdfundContract::initialize` and `validate_init_params`.

## Testing

- **`src/campaign_goal_minimum.test.rs`** — unit tests for all public helpers,
  constant stability, and error discriminants.
- Run: `cargo test -p crowdfund campaign_goal_minimum`

Target **≥ 95%** line coverage for this module is enforced by breadth of cases
(minimum, above, zero, negatives, overflow-adjacent deadlines, fee cap, progress
edge cases).

## NatSpec-style comments

Rustdoc in `campaign_goal_minimum.rs` uses tags such as `@title`, `@notice`,
`@param`, `@return`, `@dev`, and `@custom:security` so Solidity/Natspec-oriented
reviewers can map concepts quickly.

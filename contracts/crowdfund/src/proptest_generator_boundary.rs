//! # Proptest Generator Boundary Conditions
//!
//! Single source of truth for all boundary constants and validation helpers
//! used by the crowdfunding platform's property-based tests and frontend UI.
//!
//! ## New Edge Cases (Issue #423)
//!
//! - `is_ui_displayable_progress`: rejects bps values that would render as
//!   broken progress bars (NaN-equivalent: goal == 0, or bps > cap).
//! - `compute_display_percent`: converts bps → clamped 0–100 f32-equivalent
//!   integer (×100) for frontend percentage labels.
//! - `is_contribution_ui_safe`: validates that an amount is both above the
//!   minimum AND representable without overflow in the UI token-decimal layer.
//! - `deadline_ui_state`: classifies a deadline offset into the three UI
//!   states the frontend renders (Upcoming / Active / Expired).
//! - `compute_net_payout`: derives creator net payout after fee deduction,
//!   guarded against underflow and fee > total edge cases.
//!
//! ## Security Model
//!
//! - **Overflow Protection**: All arithmetic uses `saturating_mul` and `checked_sub` where applicable.
//! - **Division by Zero**: Guarded with explicit zero checks before division operations.
//! - **Basis Points Capping**: Progress and fee calculations capped at 10,000 (100%) to prevent display errors.
//! - **Timestamp Validity**: Deadline offsets exclude past and unreasonably large values.
//! - **Resource Bounds**: Test case counts and batch sizes bounded to prevent accidental stress scenarios.

// proptest_generator_boundary — Boundary constants and validation helpers.

use soroban_sdk::{contract, contractimpl, Env, Symbol};

// ── Constants ────────────────────────────────────────────────────────────────
// @notice All constants are immutable and define the safe operating boundaries
//         for the crowdfunding platform and its property-based tests.

/// Minimum deadline offset in seconds (~17 minutes).
/// @dev Prevents flaky tests due to timing races and ensures meaningful campaign duration.
pub const DEADLINE_OFFSET_MIN: u64 = 1_000;

/// Maximum deadline offset in seconds (~11.5 days).
/// @dev Avoids u64 overflow when added to ledger timestamps.
pub const DEADLINE_OFFSET_MAX: u64 = 1_000_000;

/// Minimum valid goal amount.
/// @dev Prevents division-by-zero in progress calculations.
pub const GOAL_MIN: i128 = 1_000;

/// Maximum goal amount for test generations (10 XLM).
/// @dev Keeps tests fast while covering large campaign scenarios.
pub const GOAL_MAX: i128 = 100_000_000;

/// Absolute floor for contribution amounts.
/// @dev Prevents zero-value contributions from polluting ledger state.
pub const MIN_CONTRIBUTION_FLOOR: i128 = 1;

/// Maximum progress in basis points (100%).
/// @dev Frontend never displays >100% funded; prevents display errors.
pub const PROGRESS_BPS_CAP: u32 = 10_000;

/// Maximum fee in basis points (100%).
/// @dev Fee above this would exceed 100% of the contribution.
pub const FEE_BPS_CAP: u32 = 10_000;

/// Minimum proptest case count.
/// @dev Below this, boundary-adjacent values are rarely sampled.
pub const PROPTEST_CASES_MIN: u32 = 32;

/// Maximum proptest case count.
/// @dev Balances coverage with CI execution time.
pub const PROPTEST_CASES_MAX: u32 = 256;

/// Maximum batch size for generator operations.
/// @dev Prevents worst-case memory/gas spikes in test scaffolds.
pub const GENERATOR_BATCH_MAX: u32 = 512;

// ── Pure Validation Helpers ───────────────────────────────────────────────────
//
// These are standalone `pub fn` (no `Env`) so they can be called directly
// from `#[cfg(test)]` proptest blocks without spinning up a Soroban environment.

/// Returns `true` if `offset` is within `[DEADLINE_OFFSET_MIN, DEADLINE_OFFSET_MAX]`.
///
/// @param  offset  Seconds from the current ledger timestamp to the campaign deadline.
/// @return `true`  when the offset is a safe, UI-displayable campaign duration.
///
/// @security Rejects values < 1 000 that cause timing races and values that
///           could overflow a `u64` timestamp when added to `now`.
//! Proptest generator boundary conditions for the crowdfund contract.
//! Proptest Generator Boundary Contract
//!
//! This contract provides the single source of truth for all boundary conditions and validation
//! constants used by the crowdfunding platform's property-based tests. Exposing these
//! via a contract allows off-chain scripts and other contracts to dynamically query
//! current safe operating limits.
//!
//! ## Purpose
//!
//! - **Centralized Constants**: All boundary values defined in one place for consistency.
//! - **Immutable Boundaries**: Constants are compile-time to ensure test stability.
//! - **Public Transparency**: All limits are publicly readable and queryable.
//! - **Safety Guards**: Includes validation and clamping logic against platform-wide floors and caps.
//! - **CI/CD Optimization**: Enables dynamic test configuration without hardcoding limits.
//!
//! ## Security Model
//!
//! - **Overflow Protection**: All arithmetic uses `saturating_mul` and `checked_sub` where applicable.
//! - **Division by Zero**: Guarded with explicit zero checks before division operations.
//! - **Basis Points Capping**: Progress and fee calculations capped at 10,000 (100%) to prevent display errors.
//! - **Timestamp Validity**: Deadline offsets exclude past and unreasonably large values.
//! - **Resource Bounds**: Test case counts and batch sizes bounded to prevent accidental stress scenarios.
//! ## Security Model
//!
//! - **Immutable Boundaries**: Constants are defined at compile-time to ensure test stability.
//! - **Public Transparency**: All limits are publicly readable for auditability.
//! - **Safety Guards**: Includes logic to clamp and validate inputs against platform-wide floors and caps.
//! - **Overflow Protection**: Uses `saturating_mul` and range checks to prevent integer overflow.
//! - **Division Safety**: All divisions are guarded against zero denominators.
//! @title   ProptestGeneratorBoundary
//! @notice  Central authority for all boundary constants and validation helpers
//!          used by property-based tests and frontend UI input validation.
//! @dev     Standalone pure functions are exported for use in `#[cfg(test)]`
//!          modules. The `ProptestGeneratorBoundary` contract exposes the same
//!          logic on-chain so off-chain scripts can query current platform limits
//!          without hard-coding them.
//!
//! ## Security Assumptions
//!
//! - **Overflow safety**: All goal and contribution values are bounded well below
//!   `i128::MAX`, eliminating integer-overflow risk in arithmetic.
//! - **Division-by-zero**: `compute_progress_bps` / `clamp_progress_bps` guard
//!   against `goal == 0` before dividing.
//! - **Timestamp validity**: `DEADLINE_OFFSET_MIN` (1 000 s) prevents campaigns
//!   so short they cause timing races in CI; `DEADLINE_OFFSET_MAX` (1 000 000 s)
//!   prevents unreasonably far-future deadlines.
//! - **Basis-point cap**: `PROGRESS_BPS_CAP` and `FEE_BPS_CAP` are both 10 000,
//!   ensuring frontend progress bars and fee displays never exceed 100 %.
//! - **Immutable constants**: All limits are compile-time constants, so they
//!   cannot be mutated at runtime.

// proptest_generator_boundary — Boundary constants and validation helpers.
//! - Overflow: `saturating_mul` / `checked_sub` throughout.
//! - Division by zero: explicit `goal <= 0` guard before every division.
//! - Basis-point cap: progress and fee capped at 10 000 (100 %).
//! - Timestamp: deadline offsets bounded to `[DEADLINE_OFFSET_MIN, DEADLINE_OFFSET_MAX]`.
//! - Resource bounds: batch sizes and case counts clamped to prevent CI spikes.

use soroban_sdk::{contract, contractimpl, Env, Symbol};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Minimum deadline offset in seconds (~17 minutes).
/// @dev Prevents flaky tests due to timing races.
pub const DEADLINE_OFFSET_MIN: u64 = 1_000;

/// Maximum deadline offset in seconds (~11.5 days).
/// @dev Avoids u64 overflow when added to ledger timestamps.
pub const DEADLINE_OFFSET_MAX: u64 = 1_000_000;

/// Minimum valid goal amount (stroops).
/// @dev Prevents division-by-zero in progress calculations.
pub const GOAL_MIN: i128 = 1_000;

/// Maximum goal amount for test generation.
/// @dev Keeps tests fast while covering large campaign scenarios.
pub const GOAL_MAX: i128 = 100_000_000;

/// Absolute floor for contribution amounts.
/// @dev Prevents zero-value contributions from polluting ledger state.
pub const MIN_CONTRIBUTION_FLOOR: i128 = 1;

/// Maximum progress in basis points (100 %).
/// @dev Frontend never displays >100 % funded.
pub const PROGRESS_BPS_CAP: u32 = 10_000;

/// Maximum fee in basis points (100 %).
/// @dev Fee above this would exceed 100 % of the contribution.
pub const FEE_BPS_CAP: u32 = 10_000;

/// Minimum proptest case count.
pub const PROPTEST_CASES_MIN: u32 = 32;

/// Maximum proptest case count.
pub const PROPTEST_CASES_MAX: u32 = 256;

/// Maximum batch size for generator operations.
/// @dev Prevents worst-case memory/gas spikes in test scaffolds.
/// Prevents flaky tests and meaningless campaigns.
pub const DEADLINE_OFFSET_MIN: u64 = 1_000;

/// Maximum deadline offset in seconds (~11.5 days).
/// Avoids u64 overflow when added to ledger timestamps.
pub const DEADLINE_OFFSET_MAX: u64 = 1_000_000;

/// Minimum valid goal amount.
/// Prevents division-by-zero in progress calculations.
pub const GOAL_MIN: i128 = 1_000;

/// Maximum goal amount for test generations (10 XLM).
/// Keeps tests fast while covering large campaigns.
pub const GOAL_MAX: i128 = 100_000_000;

/// Absolute floor for contribution amounts.
/// Prevents zero-value contributions from polluting ledger state.
pub const MIN_CONTRIBUTION_FLOOR: i128 = 1;

/// Progress basis points cap (100%).
/// Ensures frontend never displays >100% funded.
pub const PROGRESS_BPS_CAP: u32 = 10_000;

/// Fee basis points cap (100%).
/// Ensures fees cannot exceed 100% of contribution.
pub const FEE_BPS_CAP: u32 = 10_000;

/// Minimum proptest case count.
/// Below this, boundary-adjacent values are rarely sampled.
pub const PROPTEST_CASES_MIN: u32 = 32;

/// Maximum proptest case count.
/// Balances coverage with CI execution time.
pub const PROPTEST_CASES_MAX: u32 = 256;

/// Maximum generator batch size.
/// Prevents worst-case memory/gas spikes in test scaffolds.
// ── Boundary Constants ────────────────────────────────────────────────────────

/// Minimum deadline offset in seconds from `now` (~17 minutes).
///
/// @notice Values below this caused flaky proptest timing races and
///         flickering countdown displays in the frontend UI.
pub const DEADLINE_OFFSET_MIN: u64 = 1_000;

/// Maximum deadline offset in seconds from `now` (~11.5 days).
///
/// @notice Prevents unreasonably far-future deadlines that break UI date
///         formatting and exceed reasonable campaign windows.
pub const DEADLINE_OFFSET_MAX: u64 = 1_000_000;

/// Minimum valid funding goal in token stroops.
///
/// @notice Goals below this produce division-by-zero or near-zero progress
///         percentages that render incorrectly in the frontend progress bar.
pub const GOAL_MIN: i128 = 1_000;

/// Maximum valid funding goal for proptest generation.
///
/// @notice Caps generator output to avoid i128 overflow in fee and progress
///         calculations. 100 M stroops ≈ 10 XLM at 7-decimal precision.
pub const GOAL_MAX: i128 = 100_000_000;

/// Absolute floor for `min_contribution` values.
///
/// @notice Ensures at least one stroop must be contributed, preventing
///         zero-amount contribution exploits.
pub const MIN_CONTRIBUTION_FLOOR: i128 = 1;

/// Basis-point cap for campaign progress display (100 %).
///
/// @notice Frontend progress bars clamp to this value so over-funded
///         campaigns never render above 100 %.
pub const PROGRESS_BPS_CAP: u32 = 10_000;

/// Basis-point cap for platform fees (100 %).
///
/// @notice Prevents fee configurations that would consume the entire
///         campaign payout.
pub const FEE_BPS_CAP: u32 = 10_000;

/// Minimum number of proptest cases per property test.
///
/// @dev Keeps CI runtimes predictable; below 32 cases gives poor coverage.
pub const PROPTEST_CASES_MIN: u32 = 32;

/// Maximum number of proptest cases per property test.
///
/// @dev Above 256 cases the test suite exceeds the 15-minute CI timeout.
pub const PROPTEST_CASES_MAX: u32 = 256;

/// Maximum batch size for generator output slices.
pub const GENERATOR_BATCH_MAX: u32 = 512;

/// Maximum token decimals supported by the frontend display layer.
/// @dev XLM = 7, USDC = 6. Values above this overflow JS Number precision.
pub const MAX_TOKEN_DECIMALS: u32 = 18;

/// Deadline offset threshold (seconds) below which the UI shows "Ending Soon".
/// @dev ~1 hour — triggers the amber countdown banner in the frontend.
pub const DEADLINE_ENDING_SOON_THRESHOLD: u64 = 3_600;

// ── UI State Enum ─────────────────────────────────────────────────────────────

/// Frontend deadline display state.
///
/// @notice Maps a remaining-seconds value to the three visual states
///         rendered by the campaign card component.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeadlineUiState {
    /// Deadline is more than `DEADLINE_ENDING_SOON_THRESHOLD` seconds away.
    Active,
    /// Deadline is within `DEADLINE_ENDING_SOON_THRESHOLD` seconds (amber banner).
    EndingSoon,
    /// Deadline has passed (seconds_remaining == 0).
    Expired,
}

// ── Pure Validation Helpers ───────────────────────────────────────────────────

/// Returns `true` if `offset ∈ [DEADLINE_OFFSET_MIN, DEADLINE_OFFSET_MAX]`.
///
/// @security Rejects values < 1 000 that cause timing races and values that
///           could overflow a `u64` timestamp when added to `now`.
#[inline]
pub fn is_valid_deadline_offset(offset: u64) -> bool {
    (DEADLINE_OFFSET_MIN..=DEADLINE_OFFSET_MAX).contains(&offset)
}

/// Returns `true` if `goal ∈ [GOAL_MIN, GOAL_MAX]`.
///
/// @security Rejects `goal <= 0` which causes division-by-zero in
///           `compute_progress_bps` and breaks the frontend progress bar.
#[inline]
pub fn is_valid_goal(goal: i128) -> bool {
    (GOAL_MIN..=GOAL_MAX).contains(&goal)
}

/// Returns `true` if `MIN_CONTRIBUTION_FLOOR <= min_contribution <= goal`.
///
/// @security Ensures `min_contribution` never exceeds `goal`, which would
///           make the campaign permanently un-fundable.
#[inline]
pub fn is_valid_min_contribution(min_contribution: i128, goal: i128) -> bool {
    min_contribution >= MIN_CONTRIBUTION_FLOOR && min_contribution <= goal
}

/// Returns `true` if `amount >= min_contribution`.
#[inline]
pub fn is_valid_contribution_amount(amount: i128, min_contribution: i128) -> bool {
    amount >= min_contribution
}

/// Clamps a raw basis-point value into `[0, PROGRESS_BPS_CAP]`.
///
/// @notice Negative inputs floor to 0; over-funded campaigns cap at 100 %.
#[inline]
pub fn clamp_progress_bps(raw: i128) -> u32 {
    if raw <= 0 {
        0
    } else if raw >= PROGRESS_BPS_CAP as i128 {
        PROGRESS_BPS_CAP
    } else {
        raw as u32
    }
}

/// Computes campaign progress in basis points from `raised` and `goal`.
///
/// @security Uses `saturating_mul` to prevent overflow on large `raised`
///           values before dividing by `goal`. Returns 0 when `goal <= 0`.
#[inline]
pub fn compute_progress_bps(raised: i128, goal: i128) -> u32 {
    if goal <= 0 {
        return 0;
    }
    let raw = raised.saturating_mul(10_000) / goal;
    clamp_progress_bps(raw)
}

/// Clamps a requested proptest case count into `[PROPTEST_CASES_MIN, PROPTEST_CASES_MAX]`.
#[inline]
pub fn clamp_proptest_cases(requested: u32) -> u32 {
    requested.clamp(PROPTEST_CASES_MIN, PROPTEST_CASES_MAX)
}

// ── New Edge-Case Helpers (Issue #423) ────────────────────────────────────────

/// Returns `true` if `bps` is safe to render in the frontend progress bar.
///
/// @notice A value is UI-displayable when it is in `[0, PROGRESS_BPS_CAP]`.
///         Values outside this range would produce broken or misleading bars.
///
/// @param bps  Raw basis-point value coming from `compute_progress_bps`.
/// @return `true` when the value can be safely rendered.
///
/// @security Rejects negative-equivalent (impossible after clamping, but
///           guards against direct calls with unclamped values) and values
///           above the cap that would overflow a CSS `width` percentage.
#[inline]
pub fn is_ui_displayable_progress(bps: u32) -> bool {
    bps <= PROGRESS_BPS_CAP
}

/// Converts basis points to a display percentage scaled by 100 (i.e. 0–10 000).
///
/// @notice Returns an integer representing `bps / 100` × 100 so callers can
///         format "50.00 %" without floating-point arithmetic on-chain.
///         The value is clamped to `[0, 10_000]` before conversion.
///
/// @param bps  Basis-point progress value (will be clamped internally).
/// @return     Integer in `[0, 10_000]` — divide by 100 for the percentage.
///
/// @dev  Example: `bps = 5_000` → returns `5_000` → frontend renders "50.00 %".
///       `bps = 10_001` → clamped to `10_000` → "100.00 %".
#[inline]
pub fn compute_display_percent(bps: u32) -> u32 {
    bps.min(PROGRESS_BPS_CAP)
}

/// Returns `true` if `amount` is safe for both contract validation and the
/// frontend token-decimal display layer.
///
/// @notice Combines the minimum-contribution check with an overflow guard:
///         `amount * 10^decimals` must not exceed `i128::MAX` to be safely
///         formatted by the frontend without precision loss.
///
/// @param amount           Contribution amount in the token's smallest unit.
/// @param min_contribution Campaign minimum contribution floor.
/// @param token_decimals   Decimal precision of the token (e.g. 7 for XLM).
/// @return `true` when the amount is valid and UI-safe.
///
/// @security Rejects `token_decimals > MAX_TOKEN_DECIMALS` to prevent
///           JavaScript Number precision loss in the frontend display layer.
#[inline]
pub fn is_contribution_ui_safe(amount: i128, min_contribution: i128, token_decimals: u32) -> bool {
    if token_decimals > MAX_TOKEN_DECIMALS {
        return false;
    }
    if amount < min_contribution {
        return false;
    }
    // Guard: amount * 10^decimals must not overflow i128
    let scale: i128 = 10i128.pow(token_decimals);
    amount.checked_mul(scale).is_some()
}

/// Classifies a `seconds_remaining` value into the frontend deadline UI state.
///
/// @notice Maps remaining seconds to one of three visual states:
///         - `Expired`    — deadline has passed (`seconds_remaining == 0`).
///         - `EndingSoon` — within `DEADLINE_ENDING_SOON_THRESHOLD` seconds.
///         - `Active`     — more than the threshold away.
///
/// @param seconds_remaining  Seconds until the campaign deadline (0 = expired).
/// @return The `DeadlineUiState` variant for the frontend to render.
///
/// @security Treats `seconds_remaining == 0` as expired regardless of clock
///           skew, preventing the UI from showing an active campaign after
///           the on-chain deadline has passed.
#[inline]
pub fn deadline_ui_state(seconds_remaining: u64) -> DeadlineUiState {
    if seconds_remaining == 0 {
        DeadlineUiState::Expired
    } else if seconds_remaining <= DEADLINE_ENDING_SOON_THRESHOLD {
        DeadlineUiState::EndingSoon
    } else {
        DeadlineUiState::Active
    }
}

/// Computes the creator's net payout after platform fee deduction.
///
/// @notice Returns `None` when `fee_bps > FEE_BPS_CAP` (invalid fee) or
///         when arithmetic would underflow (fee > total).
///
/// @param total    Total tokens raised.
/// @param fee_bps  Platform fee in basis points.
/// @return `Some(net)` on success, `None` on invalid inputs.
///
/// @security Uses `checked_mul` and `checked_sub` to prevent overflow and
///           underflow. Rejects `fee_bps > FEE_BPS_CAP` before any arithmetic.
#[inline]
pub fn compute_net_payout(total: i128, fee_bps: u32) -> Option<i128> {
    if fee_bps > FEE_BPS_CAP {
        return None;
    }
    if total <= 0 {
        return Some(0);
    }
    let fee = total.checked_mul(fee_bps as i128)? / 10_000;
    total.checked_sub(fee)
}

// ── On-Chain Contract ─────────────────────────────────────────────────────────

/// On-chain contract that exposes boundary constants and validation logic so
/// off-chain scripts and other contracts can query current platform limits.
///
/// @notice All methods are pure (read-only) and do not modify contract state.
#[contract]
pub struct ProptestGeneratorBoundary;

#[contractimpl]
impl ProptestGeneratorBoundary {
    // ── Getters ───────────────────────────────────────────────────────────────

    /// Returns the minimum deadline offset in seconds.
    /// @notice ~17 minutes; prevents flaky tests and ensures meaningful campaigns.
    ///
    /// @notice Campaigns shorter than this may experience timing races in CI.
    ///
    /// @return `DEADLINE_OFFSET_MIN` (1 000).
    pub fn deadline_offset_min(_env: Env) -> u64 {
        DEADLINE_OFFSET_MIN
    }

/// Returns `true` if `goal` is within `[GOAL_MIN, GOAL_MAX]`.
///
/// @param  goal  Funding target in the token's smallest unit (stroops).
/// @return `true` when the goal is safe for arithmetic and UI display.
///
/// @security Rejects `goal <= 0` which would cause division-by-zero in
///           `compute_progress_bps` and break the frontend progress bar.
#[inline]
pub fn is_valid_goal(goal: i128) -> bool {
    (GOAL_MIN..=GOAL_MAX).contains(&goal)
}

/// Returns `true` if `min_contribution` is a valid floor for the given `goal`.
///
/// @param  min_contribution  Minimum amount a contributor must send.
/// @param  goal              The campaign's funding target.
/// @return `true` when `MIN_CONTRIBUTION_FLOOR <= min_contribution <= goal`.
///
/// @security Ensures `min_contribution` never exceeds `goal`, which would
///           make the campaign permanently un-fundable.
/// Validates that a goal is within the accepted range.
/// @notice Validates that a goal is within the accepted range.
///
/// @dev Rejects zero and negative goals to prevent division-by-zero in
///      progress calculations.
///
/// # Arguments
/// * `goal` – Campaign goal in stroops.
///
/// # Returns
/// `true` if `goal` is in `[GOAL_MIN, GOAL_MAX]`.
#[inline]
pub fn is_valid_goal(goal: i128) -> bool {
    (GOAL_MIN..=GOAL_MAX).contains(&goal)
}
    /// Returns the maximum deadline offset in seconds.
    /// @notice ~11.5 days; avoids u64 overflow on ledger timestamps.
    pub fn deadline_offset_max(_env: Env) -> u64 {
        DEADLINE_OFFSET_MAX
    }

    /// Returns the minimum valid goal amount.
    /// @notice Prevents division-by-zero in progress calculations.
    pub fn goal_min(_env: Env) -> i128 {
        GOAL_MIN
    }

/// Returns `true` if `amount >= min_contribution`.
///
/// @param  amount           The contribution amount being validated.
/// @param  min_contribution The campaign's minimum contribution floor.
/// @return `true` when the amount meets the minimum threshold.
/// Validates that a contribution amount is >= min_contribution.
/// @notice Validates that a contribution amount meets the campaign minimum.
///
/// # Arguments
/// * `amount` – Contribution amount in stroops.
/// * `min_contribution` – Campaign minimum contribution in stroops.
///
/// # Returns
/// `true` if `amount >= min_contribution`.
#[inline]
pub fn is_valid_contribution_amount(amount: i128, min_contribution: i128) -> bool {
    amount >= min_contribution
}

/// Clamps a raw basis-point value into `[0, PROGRESS_BPS_CAP]`.
///
/// @param  raw  Unclamped progress value (may be negative or > 10 000).
/// @return A `u32` in `[0, 10_000]` safe for frontend progress-bar rendering.
///
/// @notice Negative inputs (e.g. when `raised < 0`) are treated as 0 %.
///         Over-funded campaigns are capped at exactly 100 %.
/// Clamps progress_bps to valid range [0, PROGRESS_BPS_CAP].
/// @notice Validates that a fee in basis points does not exceed the cap.
///
/// @dev A fee above `FEE_BPS_CAP` would exceed 100 % of the contribution,
///      which is economically invalid.
///
/// # Arguments
/// * `fee_bps` – Fee in basis points.
///
/// # Returns
/// `true` if `fee_bps <= FEE_BPS_CAP`.
#[inline]
pub fn is_valid_fee_bps(fee_bps: u32) -> bool {
    fee_bps <= FEE_BPS_CAP
}
    /// Returns the maximum goal amount for test generations.
    /// @notice 10 XLM; keeps tests fast while covering large campaigns.
    pub fn goal_max(_env: Env) -> i128 {
        GOAL_MAX
    }

    /// Returns the absolute floor for contribution amounts.
    /// @notice Prevents zero-value contributions from polluting state.
    pub fn min_contribution_floor(_env: Env) -> i128 {
        MIN_CONTRIBUTION_FLOOR
    }

    /// Returns the maximum progress in basis points.
    /// @notice 100%; frontend never displays >100% funded.
    pub fn progress_bps_cap(_env: Env) -> u32 {
        PROGRESS_BPS_CAP
    }

    /// Returns the maximum fee in basis points.
    /// @notice 100%; fee above this would exceed the contribution.
    pub fn fee_bps_cap(_env: Env) -> u32 {
        FEE_BPS_CAP
    }

    /// Returns the minimum proptest case count.
    /// @notice Below this, boundary-adjacent values are rarely sampled.
    pub fn proptest_cases_min(_env: Env) -> u32 {
        PROPTEST_CASES_MIN
    }

    /// Returns the maximum proptest case count.
    /// @notice Balances coverage with CI execution time.
    pub fn proptest_cases_max(_env: Env) -> u32 {
        PROPTEST_CASES_MAX
    }

    /// Returns the maximum batch size for generator operations.
    /// @notice Prevents worst-case memory/gas spikes in test scaffolds.
    pub fn generator_batch_max(_env: Env) -> u32 {
        GENERATOR_BATCH_MAX
    }

    // ── Validation Functions ──────────────────────────────────────────────────
    // @notice These functions validate inputs against boundary constants.
    //         Used by tests and off-chain scripts to ensure safe values.

    /// Validates that a deadline offset is within [min, max] range.
    /// @notice Rejects values that cause timestamp overflow or campaigns too short.
    /// @param offset The deadline offset in seconds to validate.
    /// @return true if offset is valid, false otherwise.
    pub fn is_valid_deadline_offset(_env: Env, offset: u64) -> bool {
        (DEADLINE_OFFSET_MIN..=DEADLINE_OFFSET_MAX).contains(&offset)
    }

/// @notice Clamps raw progress basis points to `[0, PROGRESS_BPS_CAP]`.
///
/// @dev Negative raw values (e.g., from signed arithmetic) are floored to 0.
///      Values above 10_000 are capped so the frontend never shows >100 %.
///
/// # Arguments
/// * `raw` – Unclamped progress value (may be negative or >10_000).
///
/// # Returns
/// A `u32` in `[0, PROGRESS_BPS_CAP]`.
#[inline]
pub fn clamp_progress_bps(raw: i128) -> u32 {
    if raw <= 0 {
        0
    } else if raw >= PROGRESS_BPS_CAP as i128 {
        PROGRESS_BPS_CAP
    } else {
        raw as u32
    }
}

/// Computes campaign progress in basis points from `raised` and `goal`.
///
/// @param  raised  Total tokens raised so far.
/// @param  goal    Campaign funding target (must be > 0).
/// @return Basis points in `[0, 10_000]`; returns 0 when `goal <= 0`.
///
/// @security Uses `saturating_mul` to prevent overflow on large `raised`
///           values before dividing by `goal`.
/// @notice Clamps a requested proptest case count into safe operating bounds.
///
/// @dev Protects CI/runtime cost while preserving boundary signal. Values
///      below `PROPTEST_CASES_MIN` are raised; values above
///      `PROPTEST_CASES_MAX` are lowered.
///
/// # Arguments
/// * `requested` – Desired number of proptest cases.
///
/// # Returns
/// A `u32` in `[PROPTEST_CASES_MIN, PROPTEST_CASES_MAX]`.
#[inline]
pub fn clamp_proptest_cases(requested: u32) -> u32 {
    requested.clamp(PROPTEST_CASES_MIN, PROPTEST_CASES_MAX)
}

// ── Generator batch helpers ───────────────────────────────────────────────────

/// @notice Validates a synthetic generator batch size.
///
/// @dev Bounded batches prevent worst-case memory/gas spikes in test
///      scaffolds that iterate over generated inputs.
///
/// # Arguments
/// * `size` – Proposed batch size.
///
/// # Returns
/// `true` if `size` is in `[1, GENERATOR_BATCH_MAX]`.
#[inline]
pub fn is_valid_generator_batch_size(size: u32) -> bool {
    (1..=GENERATOR_BATCH_MAX).contains(&size)
}

/// @notice Returns a stable diagnostic tag for boundary validation events.
///
/// @dev Plain string tags keep logs compact and grep-friendly in CI output.
///
/// # Returns
/// The static string `"proptest_boundary"`.
#[inline]
pub fn boundary_log_tag() -> &'static str {
    "proptest_boundary"
}

// ── Derived helpers ───────────────────────────────────────────────────────────

/// @notice Computes progress in basis points given `raised` and `goal`.
///
/// @dev Returns 0 when `goal` is 0 to avoid division-by-zero. The result is
///      clamped via `clamp_progress_bps` so it never exceeds 10_000.
///
/// # Arguments
/// * `raised` – Amount raised so far in stroops.
/// * `goal`   – Campaign goal in stroops.
///
/// # Returns
/// Progress in basis points, clamped to `[0, PROGRESS_BPS_CAP]`.
#[inline]
pub fn compute_progress_bps(raised: i128, goal: i128) -> u32 {
    if goal <= 0 {
        return 0;
    }
    let raw = raised.saturating_mul(10_000) / goal;
    clamp_progress_bps(raw)
}

/// Clamps a requested proptest case count into `[PROPTEST_CASES_MIN, PROPTEST_CASES_MAX]`.
///
/// @param  requested  Caller-supplied case count.
/// @return A value guaranteed to be in `[32, 256]`.
#[inline]
pub fn clamp_proptest_cases(requested: u32) -> u32 {
    requested.clamp(PROPTEST_CASES_MIN, PROPTEST_CASES_MAX)
}

// ── On-Chain Contract ─────────────────────────────────────────────────────────

/// On-chain contract that exposes boundary constants and validation logic so
/// off-chain scripts and other contracts can query current platform limits.
///
/// @notice All methods are pure (read-only) and do not modify contract state.
#[contract]
pub struct ProptestGeneratorBoundary;

#[contractimpl]
impl ProptestGeneratorBoundary {
    // ── Getter Functions ──────────────────────────────────────────────────────
    // @notice These functions return immutable boundary constants.
    //         Used by off-chain scripts and other contracts to query safe limits.

    /// Returns the minimum deadline offset in seconds.
    /// @notice ~17 minutes; prevents flaky tests and ensures meaningful campaigns.
    pub fn deadline_offset_min(_env: Env) -> u64 {
        DEADLINE_OFFSET_MIN
    }

    /// Returns the maximum deadline offset in seconds.
    /// @notice ~11.5 days; avoids u64 overflow on ledger timestamps.
    ///
    /// @notice Prevents timestamp overflow when added to ledger time.
    ///
    /// @return `DEADLINE_OFFSET_MAX` (1 000 000).
    pub fn deadline_offset_max(_env: Env) -> u64 {
        DEADLINE_OFFSET_MAX
    }

    /// Returns the minimum valid goal amount.
    /// @notice Prevents division-by-zero in progress calculations.
    ///
    /// @notice Prevents division-by-zero in progress_bps calculations.
    /// Returns the minimum valid goal amount in stroops.
    ///
    /// @return `GOAL_MIN` (1 000).
    pub fn goal_min(_env: Env) -> i128 {
        GOAL_MIN
    }

    /// Returns the maximum goal amount for test generations.
    /// @notice 10 XLM; keeps tests fast while covering large campaigns.
    ///
    /// @notice Represents 10 XLM; keeps tests fast while covering large campaigns.
    /// Returns the maximum goal amount for proptest generation.
    ///
    /// @return `GOAL_MAX` (100 000 000).
    pub fn goal_max(_env: Env) -> i128 {
        GOAL_MAX
    }

    /// Returns the absolute floor for contribution amounts.
    /// @notice Prevents zero-value contributions from polluting state.
    ///
    /// @notice Prevents zero-value contributions from polluting ledger state.
    ///
    /// @return `MIN_CONTRIBUTION_FLOOR` (1).
    pub fn min_contribution_floor(_env: Env) -> i128 {
        MIN_CONTRIBUTION_FLOOR
    }

    /// Returns the maximum progress in basis points.
    /// @notice 100%; frontend never displays >100% funded.
    pub fn progress_bps_cap(_env: Env) -> u32 {
        PROGRESS_BPS_CAP
    }

    /// Returns the maximum fee in basis points.
    /// @notice 100%; fee above this would exceed the contribution.
    pub fn fee_bps_cap(_env: Env) -> u32 {
        FEE_BPS_CAP
    }

    /// Returns the minimum proptest case count.
    /// @notice Below this, boundary-adjacent values are rarely sampled.
    pub fn proptest_cases_min(_env: Env) -> u32 {
        PROPTEST_CASES_MIN
    }

    /// Returns the maximum proptest case count.
    /// @notice Balances coverage with CI execution time.
    pub fn proptest_cases_max(_env: Env) -> u32 {
        PROPTEST_CASES_MAX
    }

    /// Returns the maximum batch size for generator operations.
    /// @notice Prevents worst-case memory/gas spikes in test scaffolds.
    pub fn generator_batch_max(_env: Env) -> u32 {
        GENERATOR_BATCH_MAX
    }

    // ── Validation Functions ──────────────────────────────────────────────────
    // @notice These functions validate inputs against boundary constants.
    //         Used by tests and off-chain scripts to ensure safe values.

    /// Validates that a deadline offset is within [min, max] range.
    /// @notice Rejects values that cause timestamp overflow or campaigns too short.
    /// @param offset The deadline offset in seconds to validate.
    /// @return true if offset is valid, false otherwise.
    /// Validates that a deadline offset is within safe bounds.
    ///
    /// @notice Returns true if offset ∈ [DEADLINE_OFFSET_MIN, DEADLINE_OFFSET_MAX].
    /// @dev Rejects values that cause timestamp overflow or campaigns too short
    ///      for meaningful frontend display.
    /// Returns `true` if `offset` is within the valid deadline range.
    ///
    /// @param  offset  Seconds from now to the campaign deadline.
    /// @return Boolean validity result.
    pub fn deadline_offset_min(_env: Env) -> u64 { DEADLINE_OFFSET_MIN }
    pub fn deadline_offset_max(_env: Env) -> u64 { DEADLINE_OFFSET_MAX }
    pub fn goal_min(_env: Env) -> i128 { GOAL_MIN }
    pub fn goal_max(_env: Env) -> i128 { GOAL_MAX }
    pub fn min_contribution_floor(_env: Env) -> i128 { MIN_CONTRIBUTION_FLOOR }
    pub fn progress_bps_cap(_env: Env) -> u32 { PROGRESS_BPS_CAP }
    pub fn fee_bps_cap(_env: Env) -> u32 { FEE_BPS_CAP }
    pub fn proptest_cases_min(_env: Env) -> u32 { PROPTEST_CASES_MIN }
    pub fn proptest_cases_max(_env: Env) -> u32 { PROPTEST_CASES_MAX }
    pub fn generator_batch_max(_env: Env) -> u32 { GENERATOR_BATCH_MAX }
    pub fn max_token_decimals(_env: Env) -> u32 { MAX_TOKEN_DECIMALS }
    pub fn deadline_ending_soon_threshold(_env: Env) -> u64 { DEADLINE_ENDING_SOON_THRESHOLD }

    // ── Validation ────────────────────────────────────────────────────────────

    /// @notice Validates deadline offset is within [min, max].
    pub fn is_valid_deadline_offset(_env: Env, offset: u64) -> bool {
        is_valid_deadline_offset(offset)
    }

    /// Validates that a goal is within [min, max] range.
    /// @notice Rejects zero and negative goals to prevent division-by-zero.
    /// @param goal The goal amount to validate.
    /// @return true if goal is valid, false otherwise.
    /// Validates that a goal is within safe bounds.
    ///
    /// @notice Returns true if goal ∈ [GOAL_MIN, GOAL_MAX].
    /// @dev Rejects zero and negative goals to prevent division-by-zero.
    /// Returns `true` if `goal` is within the valid goal range.
    ///
    /// @param  goal  Funding target in stroops.
    /// @return Boolean validity result.
    /// @notice Validates goal is within [GOAL_MIN, GOAL_MAX].
    pub fn is_valid_goal(_env: Env, goal: i128) -> bool {
        is_valid_goal(goal)
    }

    /// Validates that a minimum contribution is within safe bounds.
    /// @notice min_contribution must be >= MIN_CONTRIBUTION_FLOOR and <= goal.
    /// @param min_contribution The minimum contribution to validate.
    /// @param goal The campaign goal (used as upper bound).
    /// @return true if min_contribution is valid, false otherwise.
    pub fn is_valid_min_contribution(_env: Env, min_contribution: i128, goal: i128) -> bool {
        min_contribution >= MIN_CONTRIBUTION_FLOOR && min_contribution <= goal
    }

    /// Validates that a contribution amount meets the minimum.
    /// @notice Ensures amount >= min_contribution.
    /// @param amount The contribution amount to validate.
    /// @param min_contribution The minimum required contribution.
    /// @return true if amount is valid, false otherwise.
    pub fn is_valid_contribution_amount(_env: Env, amount: i128, min_contribution: i128) -> bool {
        amount >= min_contribution
    }

    /// Validates that a fee basis points value is within cap.
    /// @notice Rejects fees > 10,000 bps (100%).
    /// @param fee_bps The fee in basis points to validate.
    /// @return true if fee_bps is valid, false otherwise.
    pub fn is_valid_fee_bps(_env: Env, fee_bps: u32) -> bool {
        fee_bps <= FEE_BPS_CAP
    }

    /// Validates that a generator batch size is within bounds.
    /// @notice Prevents worst-case memory/gas spikes.
    /// @param batch_size The batch size to validate.
    /// @return true if batch_size is valid, false otherwise.
    pub fn is_valid_generator_batch_size(_env: Env, batch_size: u32) -> bool {
        batch_size > 0 && batch_size <= GENERATOR_BATCH_MAX
    }

    // ── Clamping Functions ────────────────────────────────────────────────────
    // @notice These functions clamp values into safe operating bounds.
    //         Used by tests to ensure values stay within limits.

    /// Clamps a requested proptest case count into safe operating bounds.
    /// @notice Protects CI runtime cost while preserving boundary signal.
    /// @param requested The requested case count.
    /// @return Clamped value in [PROPTEST_CASES_MIN, PROPTEST_CASES_MAX].
    pub fn clamp_proptest_cases(_env: Env, requested: u32) -> u32 {
        clamp_proptest_cases(requested)
    }

    /// Clamps raw progress basis points to [0, PROGRESS_BPS_CAP].
    /// @notice Negative values floor to 0; values above 10,000 cap at 10,000.
    /// @dev Ensures frontend never displays >100% funded.
    /// @param raw The raw progress value to clamp.
    /// @return Clamped value in [0, PROGRESS_BPS_CAP].
    pub fn clamp_progress_bps(_env: Env, raw: i128) -> u32 {
    /// Validates that a goal is within the [min, max] range.
    pub fn is_valid_goal(_env: Env, goal: i128) -> bool {
        (GOAL_MIN..=GOAL_MAX).contains(&goal)
    }

    /// Validates that a minimum contribution is within safe bounds.
    /// @notice min_contribution must be >= MIN_CONTRIBUTION_FLOOR and <= goal.
    /// @param min_contribution The minimum contribution to validate.
    /// @param goal The campaign goal (used as upper bound).
    /// @return true if min_contribution is valid, false otherwise.
    ///
    /// @notice Returns true if min_contribution ∈ [MIN_CONTRIBUTION_FLOOR, goal].
    /// @dev min_contribution > goal would make it impossible to contribute.
    /// @notice Validates min_contribution is in [MIN_CONTRIBUTION_FLOOR, goal].
    pub fn is_valid_min_contribution(_env: Env, min_contribution: i128, goal: i128) -> bool {
        is_valid_min_contribution(min_contribution, goal)
    }

    /// Validates that a contribution amount meets the minimum.
    /// @notice Ensures amount >= min_contribution.
    /// @param amount The contribution amount to validate.
    /// @param min_contribution The minimum required contribution.
    /// @return true if amount is valid, false otherwise.
    ///
    /// @notice Returns true if amount >= min_contribution.
    /// @notice Validates contribution amount meets the minimum.
    pub fn is_valid_contribution_amount(_env: Env, amount: i128, min_contribution: i128) -> bool {
        is_valid_contribution_amount(amount, min_contribution)
    }

    /// Validates that a fee basis points value is within cap.
    /// @notice Rejects fees > 10,000 bps (100%).
    /// @param fee_bps The fee in basis points to validate.
    /// @return true if fee_bps is valid, false otherwise.
    /// Validates that a fee basis points value is within safe bounds.
    ///
    /// @notice Returns true if fee_bps <= FEE_BPS_CAP.
    /// @dev A fee above 10,000 bps would exceed 100% of the contribution.
    /// @notice Validates fee_bps <= FEE_BPS_CAP.
    pub fn is_valid_fee_bps(_env: Env, fee_bps: u32) -> bool {
        fee_bps <= FEE_BPS_CAP
    }

    /// Validates that a generator batch size is within bounds.
    /// @notice Prevents worst-case memory/gas spikes.
    /// @param batch_size The batch size to validate.
    /// @return true if batch_size is valid, false otherwise.
    /// Validates that a generator batch size is within safe bounds.
    ///
    /// @notice Returns true if batch_size ∈ [1, GENERATOR_BATCH_MAX].
    /// @dev Prevents worst-case memory/gas spikes in test scaffolds.
    /// @notice Validates batch_size is in (0, GENERATOR_BATCH_MAX].
    pub fn is_valid_generator_batch_size(_env: Env, batch_size: u32) -> bool {
        batch_size > 0 && batch_size <= GENERATOR_BATCH_MAX
    }

    // ── New Edge-Case Validators (Issue #423) ─────────────────────────────────

    /// Clamps a requested proptest case count into safe operating bounds.
    /// @notice Protects CI runtime cost while preserving boundary signal.
    /// @param requested The requested case count.
    /// @return Clamped value in [PROPTEST_CASES_MIN, PROPTEST_CASES_MAX].
    /// Clamps a requested proptest case count into safe operating bounds.
    ///
    /// @notice Returns value clamped to [PROPTEST_CASES_MIN, PROPTEST_CASES_MAX].
    /// @dev Protects CI runtime cost while preserving boundary signal.
    ///
    /// @param  requested  Caller-supplied case count.
    /// @return Clamped value in `[32, 256]`.
    /// @notice Returns true if bps is safe to render in the frontend progress bar.
    pub fn is_ui_displayable_progress(_env: Env, bps: u32) -> bool {
        is_ui_displayable_progress(bps)
    }

    /// @notice Returns true if amount is safe for contract validation and UI display.
    pub fn is_contribution_ui_safe(
        _env: Env,
        amount: i128,
        min_contribution: i128,
        token_decimals: u32,
    ) -> bool {
        is_contribution_ui_safe(amount, min_contribution, token_decimals)
    }

    // ── Clamping ──────────────────────────────────────────────────────────────

    /// @notice Clamps requested case count to [PROPTEST_CASES_MIN, PROPTEST_CASES_MAX].
    pub fn clamp_proptest_cases(_env: Env, requested: u32) -> u32 {
        clamp_proptest_cases(requested)
    }

    /// Clamps raw progress basis points to [0, PROGRESS_BPS_CAP].
    /// @notice Negative values floor to 0; values above 10,000 cap at 10,000.
    /// @dev Ensures frontend never displays >100% funded.
    /// @param raw The raw progress value to clamp.
    /// @return Clamped value in [0, PROGRESS_BPS_CAP].
    ///
    /// @notice Negative raw values are floored to 0; values above 10,000 are capped.
    /// @dev Ensures the frontend never displays >100% funded.
    /// @notice Clamps raw progress bps to [0, PROGRESS_BPS_CAP].
    pub fn clamp_progress_bps(_env: Env, raw: i128) -> u32 {
        clamp_progress_bps(raw)
    }

    // ── Derived Calculations ──────────────────────────────────────────────────

    /// Computes progress in basis points, capped at 10,000.
    /// @notice Returns 0 when goal <= 0 to avoid division-by-zero.
    /// @dev Uses saturating_mul to prevent overflow.
    /// @param raised The amount raised so far.
    /// @param goal The campaign goal.
    /// @return Progress in basis points, clamped to [0, PROGRESS_BPS_CAP].
    /// Computes progress in basis points, capped at 10,000.
    ///
    /// @notice Returns (raised * 10_000) / goal, clamped to [0, PROGRESS_BPS_CAP].
    /// @dev Returns 0 when goal <= 0 to avoid division-by-zero.
    ///      Uses saturating_mul to prevent integer overflow.
    /// @notice Computes progress in basis points, capped at 10 000.
    /// @dev Uses saturating_mul; returns 0 when goal <= 0.
    pub fn compute_progress_bps(_env: Env, raised: i128, goal: i128) -> u32 {
        compute_progress_bps(raised, goal)
    }

    /// Computes fee amount from a contribution and fee basis points.
    /// @notice Returns 0 when amount <= 0 or fee_bps == 0.
    /// @dev Uses saturating_mul to prevent overflow.
    /// @param amount The contribution amount.
    /// @param fee_bps The fee in basis points.
    /// @return Fee amount (integer floor division).
    ///
    /// @notice Returns (amount * fee_bps) / 10_000 (integer floor).
    /// @dev Returns 0 when amount <= 0 or fee_bps == 0.
    ///      Uses saturating_mul to prevent integer overflow.
    /// @notice Computes fee amount from contribution and fee_bps.
    /// @dev Returns 0 when amount <= 0 or fee_bps == 0.
    pub fn compute_fee_amount(_env: Env, amount: i128, fee_bps: u32) -> i128 {
        if amount <= 0 || fee_bps == 0 {
            return 0;
        }
        amount.saturating_mul(fee_bps as i128) / 10_000
    }

    /// Returns a diagnostic tag for boundary log events.
    /// @notice Used by off-chain indexers to filter boundary-related events.
    /// @return Symbol "boundary" for event filtering.
    ///
    /// @notice Used to identify boundary-related log entries in contract events.
    /// Computes campaign progress in basis points, capped at 10 000.
    ///
    /// @param  raised  Total tokens raised.
    /// @param  goal    Campaign funding target.
    /// @return Basis points in `[0, 10_000]`.
    pub fn compute_progress_bps(_env: Env, raised: i128, goal: i128) -> u32 {
        compute_progress_bps(raised, goal)
    }

    /// Returns a diagnostic tag symbol used in boundary log events.
    ///
    /// @return The `Symbol` `"boundary"`.
    /// @notice Converts basis points to a display percentage scaled by 100.
    pub fn compute_display_percent(_env: Env, bps: u32) -> u32 {
        compute_display_percent(bps)
    }

    /// @notice Computes creator net payout after fee; returns 0 on invalid inputs.
    pub fn compute_net_payout(_env: Env, total: i128, fee_bps: u32) -> i128 {
        compute_net_payout(total, fee_bps).unwrap_or(0)
    }

    /// @notice Returns a diagnostic tag for boundary log events.
    pub fn log_tag(_env: Env) -> Symbol {
        Symbol::new(&_env, "boundary")
/// @notice Computes the fee amount for a given contribution and fee rate.
///
/// @dev Uses integer arithmetic; result is floored. Returns 0 when
///      `fee_bps` is 0 or `amount` is 0.
///
/// # Arguments
/// * `amount`  – Contribution amount in stroops.
/// * `fee_bps` – Fee rate in basis points.
///
/// # Returns
/// Fee amount in stroops.
#[inline]
pub fn compute_fee_amount(amount: i128, fee_bps: u32) -> i128 {
    if amount <= 0 || fee_bps == 0 {
        return 0;
    }
    amount.saturating_mul(fee_bps as i128) / 10_000
}

// ── Inline unit tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod unit_tests {
    use super::*;

    // ── Constant sanity checks ────────────────────────────────────────────────

    #[test]
    fn deadline_offset_min_is_1000() {
        assert_eq!(DEADLINE_OFFSET_MIN, 1_000);
    }

    #[test]
    fn deadline_offset_max_is_1_000_000() {
        assert_eq!(DEADLINE_OFFSET_MAX, 1_000_000);
    }

    #[test]
    fn goal_min_is_1000() {
        assert_eq!(GOAL_MIN, 1_000);
    }

    #[test]
    fn goal_max_is_100_000_000() {
        assert_eq!(GOAL_MAX, 100_000_000);
    }

    #[test]
    fn min_contribution_floor_is_1() {
        assert_eq!(MIN_CONTRIBUTION_FLOOR, 1);
    }

    #[test]
    fn progress_bps_cap_is_10000() {
        assert_eq!(PROGRESS_BPS_CAP, 10_000);
    }

    #[test]
    fn fee_bps_cap_is_10000() {
        assert_eq!(FEE_BPS_CAP, 10_000);
    }

    #[test]
    fn proptest_cases_min_is_32() {
        assert_eq!(PROPTEST_CASES_MIN, 32);
    }

    #[test]
    fn proptest_cases_max_is_256() {
        assert_eq!(PROPTEST_CASES_MAX, 256);
    }

    #[test]
    fn generator_batch_max_is_512() {
        assert_eq!(GENERATOR_BATCH_MAX, 512);
    }

    // ── is_valid_deadline_offset ──────────────────────────────────────────────

    #[test]
    fn deadline_offset_rejects_below_min() {
        assert!(!is_valid_deadline_offset(0));
        assert!(!is_valid_deadline_offset(99));
        // Typo-fix regression: 100 was the old (wrong) minimum.
        assert!(!is_valid_deadline_offset(100));
        assert!(!is_valid_deadline_offset(999));
    }

    #[test]
    fn deadline_offset_accepts_min() {
        assert!(is_valid_deadline_offset(DEADLINE_OFFSET_MIN));
    }

    #[test]
    fn deadline_offset_accepts_within_range() {
        assert!(is_valid_deadline_offset(3_600));
        assert!(is_valid_deadline_offset(86_400));
        assert!(is_valid_deadline_offset(500_000));
    }

    #[test]
    fn deadline_offset_accepts_max() {
        assert!(is_valid_deadline_offset(DEADLINE_OFFSET_MAX));
    }

    #[test]
    fn deadline_offset_rejects_above_max() {
        assert!(!is_valid_deadline_offset(DEADLINE_OFFSET_MAX + 1));
        assert!(!is_valid_deadline_offset(u64::MAX));
    }

    // ── is_valid_goal ─────────────────────────────────────────────────────────

    #[test]
    fn goal_rejects_zero_and_negative() {
        assert!(!is_valid_goal(0));
        assert!(!is_valid_goal(-1));
        assert!(!is_valid_goal(i128::MIN));
    }

    #[test]
    fn goal_rejects_below_min() {
        assert!(!is_valid_goal(GOAL_MIN - 1));
    }

    #[test]
    fn goal_accepts_min() {
        assert!(is_valid_goal(GOAL_MIN));
    }

    #[test]
    fn goal_accepts_within_range() {
        assert!(is_valid_goal(1_000_000));
        assert!(is_valid_goal(50_000_000));
    }

    #[test]
    fn goal_accepts_max() {
        assert!(is_valid_goal(GOAL_MAX));
    }

    #[test]
    fn goal_rejects_above_max() {
        assert!(!is_valid_goal(GOAL_MAX + 1));
        assert!(!is_valid_goal(i128::MAX));
    }

    // ── is_valid_min_contribution ─────────────────────────────────────────────

    #[test]
    fn min_contribution_accepts_floor() {
        assert!(is_valid_min_contribution(MIN_CONTRIBUTION_FLOOR, 1_000));
    }

    #[test]
    fn min_contribution_accepts_equal_to_goal() {
        assert!(is_valid_min_contribution(1_000, 1_000));
    }

    #[test]
    fn min_contribution_accepts_midrange() {
        assert!(is_valid_min_contribution(500, 1_000_000));
    }

    #[test]
    fn min_contribution_rejects_zero() {
        assert!(!is_valid_min_contribution(0, 1_000));
    }

    #[test]
    fn min_contribution_rejects_above_goal() {
        assert!(!is_valid_min_contribution(1_001, 1_000));
    }

    // ── is_valid_contribution_amount ──────────────────────────────────────────

    #[test]
    fn contribution_accepts_at_min() {
        assert!(is_valid_contribution_amount(1_000, 1_000));
    }

    #[test]
    fn contribution_accepts_above_min() {
        assert!(is_valid_contribution_amount(100_000, 1_000));
    }

    #[test]
    fn contribution_rejects_below_min() {
        assert!(!is_valid_contribution_amount(999, 1_000));
        assert!(!is_valid_contribution_amount(0, 1));
    }

    // ── is_valid_fee_bps ──────────────────────────────────────────────────────

    #[test]
    fn fee_bps_accepts_zero() {
        assert!(is_valid_fee_bps(0));
    }

    #[test]
    fn fee_bps_accepts_cap() {
        assert!(is_valid_fee_bps(FEE_BPS_CAP));
    }

    #[test]
    fn fee_bps_rejects_above_cap() {
        assert!(!is_valid_fee_bps(FEE_BPS_CAP + 1));
        assert!(!is_valid_fee_bps(u32::MAX));
    }

    // ── clamp_progress_bps ────────────────────────────────────────────────────

    #[test]
    fn clamp_progress_bps_floors_negative() {
        assert_eq!(clamp_progress_bps(-1), 0);
        assert_eq!(clamp_progress_bps(i128::MIN), 0);
    }

    #[test]
    fn clamp_progress_bps_floors_zero() {
        assert_eq!(clamp_progress_bps(0), 0);
    }

    #[test]
    fn clamp_progress_bps_passes_midrange() {
        assert_eq!(clamp_progress_bps(5_000), 5_000);
        assert_eq!(clamp_progress_bps(1), 1);
        assert_eq!(clamp_progress_bps(9_999), 9_999);
    }

    #[test]
    fn clamp_progress_bps_caps_at_10000() {
        assert_eq!(clamp_progress_bps(10_000), PROGRESS_BPS_CAP);
        assert_eq!(clamp_progress_bps(20_000), PROGRESS_BPS_CAP);
        assert_eq!(clamp_progress_bps(i128::MAX), PROGRESS_BPS_CAP);
    }

    // ── clamp_proptest_cases ──────────────────────────────────────────────────

    #[test]
    fn clamp_proptest_cases_raises_below_min() {
        assert_eq!(clamp_proptest_cases(0), PROPTEST_CASES_MIN);
        assert_eq!(clamp_proptest_cases(16), PROPTEST_CASES_MIN);
        assert_eq!(clamp_proptest_cases(31), PROPTEST_CASES_MIN);
    }

    #[test]
    fn clamp_proptest_cases_passes_valid() {
        assert_eq!(clamp_proptest_cases(32), 32);
        assert_eq!(clamp_proptest_cases(128), 128);
        assert_eq!(clamp_proptest_cases(256), 256);
    }

    #[test]
    fn clamp_proptest_cases_lowers_above_max() {
        assert_eq!(clamp_proptest_cases(257), PROPTEST_CASES_MAX);
        assert_eq!(clamp_proptest_cases(1_024), PROPTEST_CASES_MAX);
        assert_eq!(clamp_proptest_cases(u32::MAX), PROPTEST_CASES_MAX);
    }

    // ── is_valid_generator_batch_size ─────────────────────────────────────────

    #[test]
    fn batch_size_rejects_zero() {
        assert!(!is_valid_generator_batch_size(0));
    }

    #[test]
    fn batch_size_accepts_one() {
        assert!(is_valid_generator_batch_size(1));
    }

    #[test]
    fn batch_size_accepts_max() {
        assert!(is_valid_generator_batch_size(GENERATOR_BATCH_MAX));
    }

    #[test]
    fn batch_size_rejects_above_max() {
        assert!(!is_valid_generator_batch_size(GENERATOR_BATCH_MAX + 1));
        assert!(!is_valid_generator_batch_size(u32::MAX));
    }

    // ── boundary_log_tag ──────────────────────────────────────────────────────

    #[test]
    fn boundary_log_tag_is_stable() {
        assert_eq!(boundary_log_tag(), "proptest_boundary");
    }

    // ── compute_progress_bps ──────────────────────────────────────────────────

    #[test]
    fn compute_progress_bps_zero_goal_returns_zero() {
        assert_eq!(compute_progress_bps(1_000, 0), 0);
        assert_eq!(compute_progress_bps(0, 0), 0);
    }

    #[test]
    fn compute_progress_bps_half_funded() {
        assert_eq!(compute_progress_bps(500, 1_000), 5_000);
    }

    #[test]
    fn compute_progress_bps_fully_funded() {
        assert_eq!(compute_progress_bps(1_000, 1_000), 10_000);
    }

    #[test]
    fn compute_progress_bps_over_funded_clamped() {
        assert_eq!(compute_progress_bps(2_000, 1_000), PROGRESS_BPS_CAP);
    }

    #[test]
    fn compute_progress_bps_zero_raised() {
        assert_eq!(compute_progress_bps(0, 1_000), 0);
    }

    // ── compute_fee_amount ────────────────────────────────────────────────────

    #[test]
    fn compute_fee_amount_zero_fee() {
        assert_eq!(compute_fee_amount(1_000_000, 0), 0);
    }

    #[test]
    fn compute_fee_amount_zero_amount() {
        assert_eq!(compute_fee_amount(0, 500), 0);
    }

    #[test]
    fn compute_fee_amount_5_percent() {
        // 5 % = 500 bps; 1_000_000 * 500 / 10_000 = 50_000
        assert_eq!(compute_fee_amount(1_000_000, 500), 50_000);
    }

    #[test]
    fn compute_fee_amount_100_percent() {
        assert_eq!(compute_fee_amount(1_000_000, 10_000), 1_000_000);
    /// Returns a diagnostic tag for boundary log events.
    /// @notice Used by off-chain indexers to filter boundary-related events.
    /// @return Symbol "boundary" for event filtering.
    pub fn log_tag(_env: Env) -> Symbol {
        Symbol::new(&_env, "boundary")
    }
}

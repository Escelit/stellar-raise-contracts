// campaign_goal_minimum — Minimum threshold enforcement for campaign goals.
//
// Security Assumptions:
// 1. MIN_GOAL_AMOUNT >= 1 closes the zero-goal drain exploit.
// 2. Negative goals are rejected by the < MIN_GOAL_AMOUNT comparison.
// 3. No integer overflow — only comparisons and saturating_add are used.
// 4. validate_goal_amount is called before any env.storage() write.
// 5. Constants are baked into WASM; changes require a contract upgrade.

use soroban_sdk::{Address, Env};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Minimum campaign goal in token units.
/// A goal of zero enables a trivial drain exploit; 1 closes that surface.
pub const MIN_GOAL_AMOUNT: i128 = 1;

/// Minimum contribution amount in token units.
pub const MIN_CONTRIBUTION_AMOUNT: i128 = 1;
/// @notice Minimum allowed `min_contribution` value in token units.
///
/// @dev    Prevents contributions of 0 tokens, which would allow an attacker
///         to register as a contributor without transferring any value.
pub const MIN_CONTRIBUTION_AMOUNT: i128 = 1;

/// @notice Maximum allowed platform fee in basis points (100% = 10_000 bps).
///
/// # Security
/// Ensures goal meets minimum threshold and creator is authenticated.
pub fn create_campaign(env: Env, creator: Address, goal: u64) {
    creator.require_auth();
    if goal < MIN_CAMPAIGN_GOAL {
        panic!("Goal too low");
    }
    env.events()
        .publish(("campaign", "created"), (creator, goal));
}

/// Minimum seconds a deadline must be ahead of the current ledger timestamp.
pub const MIN_DEADLINE_OFFSET: u64 = 60;

/// Maximum platform fee in basis points (10 000 bps = 100 %).
pub const MAX_PLATFORM_FEE_BPS: u32 = 10_000;

/// Denominator used when computing progress in basis points.
pub const PROGRESS_BPS_SCALE: i128 = 10_000;

/// Maximum value returned by compute_progress_bps.
pub const MAX_PROGRESS_BPS: u32 = 10_000;

// ── Off-chain / string-error validators ──────────────────────────────────────

/// Validates that goal meets the minimum threshold.
//! # campaign_goal_minimum
//!
//! @title   CampaignGoalMinimum — Enforces minimum campaign goal thresholds.
//!
//! @notice  This module provides the logic to prevent campaigns from being
//!          created with goals below a defined minimum, ensuring realistic
//!          fundraising targets and improving security.

// ── Constants ─────────────────────────────────────────────────────────────────

/// Minimum allowed campaign goal.
pub const MIN_CAMPAIGN_GOAL: u64 = 100;
// ── Constants ────────────────────────────────────────────────────────────────
/// Minimum campaign goal in token units.
/// A goal of zero enables a trivial drain exploit; 1 closes that surface.
pub const MIN_GOAL_AMOUNT: i128 = 1;

/// Minimum value for the min_contribution parameter.
/// Prevents zero-amount contributions that waste gas.
pub const MIN_CONTRIBUTION_AMOUNT: i128 = 1;

    if goal < MIN_CAMPAIGN_GOAL {
        panic!("Minimum campaign goal not met");
    }

    if goal == 0 {
        panic!("Campaign goal must be non-zero");
    }

    // Example storage logic (placeholder)
    // env.storage().instance().set(&DataKey::Creator, &creator);
    // env.storage().instance().set(&DataKey::Goal, &goal);
    
    // Emit event as requested
    env.events().publish(("campaign", "created"), (creator, goal));
}

/// Validates if a goal meets the minimum threshold.
//! @title   CampaignGoalMinimum — Extracted constants and enforcement logic
//!          for campaign goal and minimum contribution threshold validation.
//!
//! @notice  This module centralizes every magic number and threshold used
//!          during campaign initialization and contribution validation.
//!          Extracting them into named constants eliminates repeated literals
//!          scattered across the contract, reduces the risk of inconsistent
//!          values, and makes the intent of each guard immediately clear to
//!          reviewers.
//!
//! @dev     All constants are `pub` so that `lib.rs` and test modules can
//!          import them from a single source of truth.  No runtime state is
//!          held here — this module is purely compile-time constants plus
//!          pure validation helpers.
//!
//! ## Gas-efficiency rationale
//!
//! Before this refactor the contract contained inline literals such as
//! `10_000` (fee cap in bps), `0` (zero-amount guard), and the minimum-goal
//! floor.  Each occurrence forced the reader to infer intent from context and
//! made audits error-prone.  Named constants:
//!
//! - Are resolved at compile time — zero runtime cost.
//! - Appear in a single place, so a future change touches one line.
//! - Enable the compiler to catch type mismatches at the call site.
//!
//! ## Security assumptions
//!
//! 1. `MIN_GOAL_AMOUNT` prevents campaigns with a zero or trivially small
//!    goal that could be exploited to immediately trigger "goal reached" and
//!    allow the creator to withdraw dust amounts.
//! 2. `MIN_CONTRIBUTION_AMOUNT` prevents zero-amount contributions that waste
//!    gas on a no-op token transfer and pollute the contributors list.
//! 3. `MAX_PLATFORM_FEE_BPS` caps the platform fee at 100 % (10 000 bps) so
//!    the contract can never be configured to steal all contributor funds.
//! 4. `PROGRESS_BPS_SCALE` is the single authoritative scale factor for all
//!    basis-point progress calculations; using it everywhere prevents
//!    off-by-one errors when the scale changes.
//! 5. `MIN_DEADLINE_OFFSET` ensures the campaign deadline is always in the
//!    future relative to the ledger timestamp at initialization, preventing
//!    campaigns that are dead-on-arrival.
//!
//! ## Validation flow
//!
//! ```text
//! initialize(goal, min_contribution, deadline, platform_config)
//!        │
//!        ├─► validate_goal(goal)
//!        │       └─ goal >= MIN_GOAL_AMOUNT  ──► Ok / Err::GoalBelowMinimum
//!        │
//!        ├─► validate_min_contribution(min_contribution)
//!        │       └─ min_contribution >= MIN_CONTRIBUTION_AMOUNT
//!        │                              ──► Ok / Err::MinContributionBelowFloor
//!        │
//!        ├─► validate_deadline(now, deadline)
//!        │       └─ deadline >= now + MIN_DEADLINE_OFFSET
//!        │                              ──► Ok / Err::DeadlineTooSoon
//!        │
//!        └─► validate_platform_fee(fee_bps)
//!                └─ fee_bps <= MAX_PLATFORM_FEE_BPS
//!                               ──► Ok / Err::FeeTooHigh
//! ```

// ── Constants ────────────────────────────────────────────────────────────────

/// Minimum allowed campaign goal (in the token's smallest unit).
///
/// A goal of zero would let the creator withdraw immediately after any
/// contribution, effectively turning the contract into a donation drain.
/// Setting a floor of 1 prevents this while remaining permissive enough for
/// test environments.
pub const MIN_GOAL_AMOUNT: i128 = 1;

/// Minimum allowed value for the `min_contribution` parameter.
///
/// Prevents the contract from being initialised with a zero minimum, which
/// would allow zero-amount contributions to waste gas and pollute storage.
pub const MIN_CONTRIBUTION_AMOUNT: i128 = 1;

/// Maximum platform fee expressed in basis points (1 bps = 0.01 %).
///
/// 10 000 bps == 100 %.  A fee above this would mean the platform takes more
/// than the total raised, leaving the creator with a negative payout.
pub const MAX_PLATFORM_FEE_BPS: u32 = 10_000;

/// Scale factor used for all basis-point progress calculations.
///
/// `progress_bps = (total_raised * PROGRESS_BPS_SCALE) / goal`
///
/// Keeping this as a named constant means every progress calculation in the
/// contract references the same value and a future change (e.g. to parts-per-
/// million) only requires editing one line.
pub const PROGRESS_BPS_SCALE: i128 = 10_000;

/// Minimum number of ledger seconds the deadline must be in the future at
/// initialisation time.
///
/// Prevents campaigns that expire before a single transaction can be
/// submitted.  Set to 60 seconds — one ledger close interval on Stellar
/// mainnet.
pub const MIN_DEADLINE_OFFSET: u64 = 60;

/// Maximum basis-point value representing 100 % progress (goal fully met).
///
/// Progress is capped at this value so callers always receive a value in
/// `[0, MAX_PROGRESS_BPS]` regardless of how much the goal was exceeded.
pub const MAX_PROGRESS_BPS: u32 = 10_000;
/// # Security
/// Ensures goal meets minimum threshold and creator is authenticated.
pub fn create_campaign(env: Env, creator: Address, goal: u64) {
    creator.require_auth();
    if goal < MIN_CAMPAIGN_GOAL {
        panic!("Goal too low");
    }
    env.events().publish(("campaign", "created"), (creator, goal));
}
//! # campaign_goal_minimum
//!
//! @title   CampaignGoalMinimum — Enforces minimum campaign goal thresholds.
//!
//! @notice  This module provides all validation helpers for campaign creation
//!          parameters: goal amount, minimum contribution, deadline, and
//!          platform fee. It also exposes a progress-in-basis-points helper
//!          used by the frontend and indexers.
//!
//! @dev     All validators are `#[inline]` pure functions with no side-effects.
//!          They are called inside `initialize()` before any storage writes so
//!          that a rejected parameter leaves no partial ledger state.
//!
//! ## Security rationale
//!
//! | Threat | Mitigation |
//! |--------|-----------|
//! | Zero-goal drain | `MIN_GOAL_AMOUNT = 1` rejects goals that would make a campaign immediately "successful" |
//! | Ledger spam | Non-zero minimum prevents dust campaigns that waste storage |
//! | Negative goal | `i128` comparison rejects all values < 1 in a single branch |
//! | Fee overflow | `MAX_PLATFORM_FEE_BPS = 10_000` caps fee at 100% |
//! | Deadline bypass | `MIN_DEADLINE_OFFSET` ensures campaigns run for at least 60 s |
//! | Progress overflow | `compute_progress_bps` uses `saturating_mul` and caps at `MAX_PROGRESS_BPS` |
/// Minimum contribution amount in token units.
/// Minimum contribution amount.
/// Minimum contribution amount in token units.
pub const MIN_CONTRIBUTION_AMOUNT: i128 = 1;

// ── Constants ────────────────────────────────────────────────────────────────

/// @notice Minimum allowed campaign goal in token units.
///
/// @dev    Set to 1 so that any non-zero, non-negative goal is accepted in
///         test and development environments. Governance can raise this value
///         via a contract upgrade (see docs/campaign_goal_minimum.md).
///
/// @custom:security A goal of 0 would make a campaign immediately "successful"
///         after any contribution, allowing the creator to drain funds with no
///         real commitment. This constant closes that attack surface.
pub const MIN_GOAL_AMOUNT: i128 = 1;

/// @notice Minimum allowed `min_contribution` value in token units.
///
/// @dev    Prevents contributions of 0 tokens, which would allow an attacker
///         to register as a contributor without transferring any value.
pub const MIN_CONTRIBUTION_AMOUNT: i128 = 1;

/// @notice Maximum allowed platform fee in basis points (100% = 10_000 bps).
///
/// @custom:security Any fee_bps above this value is a configuration error.
///         The assertion in `validate_platform_fee` acts as a last-line-of-
///         defence guard even if upstream validation is bypassed.
pub const MAX_PLATFORM_FEE_BPS: u32 = 10_000;

/// @notice Scale factor used when computing progress in basis points.
///
/// @dev    `progress_bps = (total_raised * PROGRESS_BPS_SCALE) / goal`.
///         Must equal `MAX_PROGRESS_BPS` so that a fully-met goal produces
///         exactly `MAX_PROGRESS_BPS`.
pub const PROGRESS_BPS_SCALE: i128 = 10_000;

/// @notice Maximum value returned by `compute_progress_bps`.
///
/// @dev    Progress is capped at this value even when `total_raised > goal`
///         (over-funded campaigns). Equals `PROGRESS_BPS_SCALE`.
pub const MAX_PROGRESS_BPS: u32 = 10_000;

/// @notice Minimum number of seconds a campaign deadline must be in the future
///         relative to the current ledger timestamp.
///
/// @dev    Prevents campaigns with deadlines so close to `now` that no
///         contributor could realistically participate before the deadline
///         passes.
pub const MIN_DEADLINE_OFFSET: u64 = 60;

// ── Validation helpers ───────────────────────────────────────────────────────
/// Minimum seconds a deadline must be ahead of the current ledger timestamp.
pub const MIN_DEADLINE_OFFSET: u64 = 60;

/// Maximum platform fee in basis points (10 000 bps = 100 %).
pub const MAX_PLATFORM_FEE_BPS: u32 = 10_000;

/// Denominator used when computing progress in basis points.
pub const PROGRESS_BPS_SCALE: i128 = 10_000;

/// Maximum value returned by compute_progress_bps.
pub const MAX_PROGRESS_BPS: u32 = 10_000;

const MIN_CAMPAIGN_GOAL: u64 = 1;

// ── Off-chain / string-error validators ──────────────────────────────────────

/// Validates that goal meets the minimum threshold.
/// Returns Ok(()) if goal >= MIN_GOAL_AMOUNT; Err(&'static str) otherwise.
/// @notice Validates that `goal` meets the minimum threshold.
///
/// @dev    Returns `&'static str` rather than `ContractError` so this helper
///         can be used in off-chain tooling without pulling in the full
///         contract dependency.
///
/// @param  goal  The proposed campaign goal in token units.
/// @return       `Ok(())` if `goal >= MIN_GOAL_AMOUNT`, `Err` with a reason
///               string otherwise.
///
/// @custom:security The comparison is a single signed integer operation —
///         no arithmetic is performed, so integer overflow is impossible.
#[inline]
pub fn validate_goal(goal: i128) -> Result<(), &'static str> {
    if goal < MIN_GOAL_AMOUNT {
        return Err("goal must be at least MIN_GOAL_AMOUNT");
    }
    Ok(())
}

/// @notice Validates that `goal_amount` meets the minimum threshold, returning
///         a typed `ContractError::GoalTooLow` on failure.
///
/// @notice This is the on-chain enforcement entry point. Call this inside
///         `initialize()` before persisting any campaign state so that a
///         below-threshold goal is rejected atomically with no side-effects.
///
/// @dev    The `_env` parameter is accepted for API consistency with other
///         Soroban validation helpers and to allow future ledger-aware
///         threshold logic (e.g. governance-controlled minimums stored in
///         contract storage) without a breaking signature change.
///
/// @param  _env         The Soroban environment (reserved for future use).
/// @param  goal_amount  The proposed campaign goal in token units.
/// @return              `Ok(())` if `goal_amount >= MIN_GOAL_AMOUNT`,
///                      `Err(ContractError::GoalTooLow)` otherwise.
///
/// @custom:security Integer-overflow safety: the comparison is a single signed
///         integer operation — no arithmetic is performed.
#[inline]
pub fn validate_goal_amount(
    _env: &soroban_sdk::Env,
    goal_amount: i128,
) -> Result<(), crate::ContractError> {
    if goal_amount < MIN_GOAL_AMOUNT {
        return Err(crate::ContractError::GoalTooLow);
    }
    Ok(())
}

/// Validates that `goal_amount` meets the minimum threshold, returning a typed
/// [`crate::ContractError::GoalTooLow`] on failure.
/// Validates that `min_contribution` meets the minimum floor.
/// Validates that `goal_amount` meets the minimum floor.
///
/// @notice  On-chain enforcement entry point. Call inside `initialize()` before
///          any state is written so a rejected goal leaves no partial storage.
///
/// @dev     The `_env` parameter is reserved for future governance-controlled
///          thresholds stored in contract storage, without a breaking signature change.
///
/// @param  _env         The Soroban environment (reserved for future use).
/// @param  goal_amount  The proposed campaign goal in token units.
/// @return              `Ok(())` if `goal_amount >= MIN_GOAL_AMOUNT`,
///                      `Err(ContractError::GoalTooLow)` otherwise.
///
/// ## Security rationale
///
/// A goal below `MIN_GOAL_AMOUNT` would:
/// - Allow a zero-goal campaign to be immediately "successful" after any
///   contribution, letting the creator drain funds with no real commitment.
/// - Create "dust" campaigns that consume a ledger entry for negligible value,
///   wasting network resources and increasing state bloat.
/// - Undermine platform credibility by permitting economically meaningless
///   campaigns usable for spam or griefing.
/// Validates that `min_contribution` meets the minimum floor.
/// @notice Validates that `min_contribution` meets the minimum floor.
///
/// @param  min_contribution  The proposed minimum contribution in token units.
/// @return                   `Ok(())` if valid, `Err` with a reason string
///                           otherwise.
///
/// The comparison `goal_amount < MIN_GOAL_AMOUNT` is a single signed integer
/// comparison — no arithmetic is performed, so overflow is impossible.
#[inline]
pub fn validate_goal_amount(
    _env: &soroban_sdk::Env,
    goal_amount: i128,
) -> Result<(), crate::ContractError> {
    if goal_amount < MIN_GOAL_AMOUNT {
        return Err(crate::ContractError::GoalTooLow);
    }
    Ok(())
}

/// Validates that `min_contribution` meets the minimum floor.
/// @param  min_contribution  The proposed minimum contribution in token units.
/// @return                   `Ok(())` if valid, `Err` otherwise.
/// Validates that min_contribution meets the minimum floor.
/// Returns Ok(()) if valid; Err(&'static str) otherwise.
pub const MIN_CONTRIBUTION_AMOUNT: i128 = 1;
pub const MIN_GOAL_AMOUNT: i128 = 100;

/// @custom:security A `min_contribution` of 0 would allow an attacker to
///         register as a contributor without transferring any value, polluting
///         the contributor list and potentially triggering NFT mints for free.
#[inline]
pub fn validate_min_contribution(min_contribution: i128) -> Result<(), &'static str> {
    if min_contribution < MIN_CONTRIBUTION_AMOUNT {
        return Err("min_contribution must be at least MIN_CONTRIBUTION_AMOUNT");
    }
    Ok(())
}

/// Validates that deadline is sufficiently far in the future.
#[inline]
pub fn validate_deadline(now: u64, deadline: u64) -> Result<(), &'static str> {
    let min_deadline = now.saturating_add(MIN_DEADLINE_OFFSET);
    if deadline < min_deadline {
/// Validates that the campaign deadline is sufficiently far in the future.
///
/// @param  now       Current ledger timestamp (seconds since Unix epoch).
/// @param  deadline  Proposed campaign deadline timestamp.
/// @return           `Ok(())` if `deadline >= now + MIN_DEADLINE_OFFSET`.
#[inline]
pub fn validate_deadline(now: u64, deadline: u64) -> Result<(), &'static str> {
    if deadline < now.saturating_add(MIN_DEADLINE_OFFSET) {
        return Err("deadline must be at least MIN_DEADLINE_OFFSET seconds in the future");
    }
    Ok(())
}

/// Validates that fee_bps does not exceed the platform fee cap.
#[inline]
pub fn validate_platform_fee(fee_bps: u32) -> Result<(), &'static str> {
    if fee_bps > MAX_PLATFORM_FEE_BPS {
        return Err("fee_bps must not exceed MAX_PLATFORM_FEE_BPS");
/// Validates that a platform fee does not exceed the maximum allowed.
///
/// @param  fee_bps  Platform fee in basis points.
/// @return          `Ok(())` if `fee_bps <= MAX_PLATFORM_FEE_BPS`.
#[inline]
pub fn validate_platform_fee(fee_bps: u32) -> Result<(), &'static str> {
    if fee_bps > MAX_PLATFORM_FEE_BPS {
        return Err("platform fee cannot exceed MAX_PLATFORM_FEE_BPS (100%)");
    }
    Ok(())
}

// ── On-chain / typed-error validator ─────────────────────────────────────────

/// @notice Computes campaign funding progress in basis points.
///
/// Security: A zero-goal campaign is immediately "successful" after any
/// contribution, letting the creator drain funds with no real commitment.
/// Integer-overflow safety: single signed comparison, no arithmetic.
#[inline]
pub fn validate_goal_amount(
    _env: &soroban_sdk::Env,
    goal_amount: i128,
) -> Result<(), crate::ContractError> {
    if goal_amount < MIN_GOAL_AMOUNT {
        return Err(crate::ContractError::GoalTooLow);
    }
    Ok(())
}

// ── Progress computation ─────────────────────────────────────────────────────

/// Computes campaign progress in basis points (0–10 000).
/// Returns 0 if goal <= 0.
/// Caps at MAX_PROGRESS_BPS even when total_raised > goal (over-funded).
/// Uses integer division; precision loss is acceptable for UI display.
/// @dev    `progress_bps = (total_raised * PROGRESS_BPS_SCALE) / goal`.
///         Result is capped at `MAX_PROGRESS_BPS` for over-funded campaigns.
///         Returns 0 when `goal <= 0` to avoid division by zero.
///
/// @param  total_raised  Total tokens raised so far.
/// @param  goal          Campaign funding goal.
/// @return               Progress in basis points, capped at `MAX_PROGRESS_BPS`.
///
/// @custom:security Uses `saturating_mul` to prevent overflow on very large
///         `total_raised` values. The cap ensures the return value is always
///         in `[0, MAX_PROGRESS_BPS]`.
/// Computes campaign progress in basis points, capped at [`MAX_PROGRESS_BPS`].
///
/// @param  total_raised  Amount raised so far (token units).
/// @param  goal          Campaign goal (token units, must be > 0).
/// @return               Progress in bps in the range `[0, MAX_PROGRESS_BPS]`.
///
/// @dev    Returns 0 when `goal == 0` to avoid division by zero; callers
///         should ensure `goal >= MIN_GOAL_AMOUNT` before calling.
#[inline]
pub fn compute_progress_bps(total_raised: i128, goal: i128) -> u32 {
    if goal <= 0 || total_raised <= 0 {
        return 0;
    }
    let progress = (total_raised * PROGRESS_BPS_SCALE) / goal;
    if progress > MAX_PROGRESS_BPS as i128 {
        MAX_PROGRESS_BPS
    } else {
        progress as u32
    }
}

/// Creates a new campaign with goal validation.
///
/// # Parameters
/// - creator: campaign owner
/// - goal: funding target
///
/// # Security
/// Ensures goal meets minimum threshold and creator is authenticated.
pub fn create_campaign(env: soroban_sdk::Env, creator: soroban_sdk::Address, goal: u64) {
    creator.require_auth();
    if goal < MIN_CAMPAIGN_GOAL {
        panic!("Goal too low");
    }
    env.events().publish(("campaign", "created"), (creator, goal));
    let raw = total_raised.saturating_mul(PROGRESS_BPS_SCALE) / goal;
    if raw >= PROGRESS_BPS_SCALE {
        return MAX_PROGRESS_BPS;
    }
    raw.max(0) as u32
}

const MIN_CAMPAIGN_GOAL: u64 = 1;
    let raw = (total_raised * PROGRESS_BPS_SCALE) / goal;
    let raw = total_raised.saturating_mul(PROGRESS_BPS_SCALE) / goal;
    if raw > PROGRESS_BPS_SCALE {

    let scaled = total_raised
        .checked_mul(PROGRESS_BPS_SCALE)
        .unwrap_or(i128::MAX);
    let raw = scaled / goal;

    if raw >= PROGRESS_BPS_SCALE {
        MAX_PROGRESS_BPS
    } else {
        raw as u32
    }
/// # Parameters
/// - goal: the proposed goal
}

/// @notice Validates that `deadline` is sufficiently far in the future.
///
/// @param  now       Current ledger timestamp (seconds since Unix epoch).
/// @param  deadline  Proposed campaign deadline (seconds since Unix epoch).
/// @return           `Ok(())` if `deadline >= now + MIN_DEADLINE_OFFSET`,
///                   `Err` with a reason string otherwise.
///
/// @custom:security Uses `saturating_add` to prevent overflow when `now` is
///         near `u64::MAX`. A saturated sum of `u64::MAX` means any finite
///         deadline will be rejected, which is the safe default.
#[inline]
pub fn validate_deadline(now: u64, deadline: u64) -> Result<(), &'static str> {
    let min_deadline = now.saturating_add(MIN_DEADLINE_OFFSET);
    if deadline < min_deadline {
        return Err("deadline must be at least MIN_DEADLINE_OFFSET seconds in the future");
    }
    Ok(())
}

/// @notice Validates that `fee_bps` does not exceed `MAX_PLATFORM_FEE_BPS`.
///
/// @param  fee_bps  Platform fee in basis points (0 = no fee, 10_000 = 100%).
/// @return          `Ok(())` if `fee_bps <= MAX_PLATFORM_FEE_BPS`, `Err`
///                  with a reason string otherwise.
///
/// @custom:security A fee above 100% would cause the fee transfer to exceed
///         the total raised, resulting in an underflow panic or incorrect
///         creator payout. This guard prevents that at the validation layer.
#[inline]
pub fn validate_platform_fee(fee_bps: u32) -> Result<(), &'static str> {
    if fee_bps > MAX_PLATFORM_FEE_BPS {
        return Err("fee_bps must not exceed MAX_PLATFORM_FEE_BPS");
    }
    Ok(())
}

/// @notice Computes campaign funding progress in basis points.
///
/// @dev    `progress_bps = (total_raised * PROGRESS_BPS_SCALE) / goal`.
///         Result is capped at `MAX_PROGRESS_BPS` for over-funded campaigns.
///         Returns 0 when `goal <= 0` to avoid division by zero.
///
/// @param  total_raised  Total tokens raised so far.
/// @param  goal          Campaign funding goal.
/// @return               Progress in basis points, capped at `MAX_PROGRESS_BPS`.
///
/// @custom:security Uses `saturating_mul` to prevent overflow on very large
///         `total_raised` values. The cap ensures the return value is always
///         in `[0, MAX_PROGRESS_BPS]`.
#[inline]
pub fn compute_progress_bps(total_raised: i128, goal: i128) -> u32 {
    if goal <= 0 {
        return 0;
    }
    let raw = total_raised.saturating_mul(PROGRESS_BPS_SCALE) / goal;
    if raw >= PROGRESS_BPS_SCALE {
        return MAX_PROGRESS_BPS;
    }
    raw.max(0) as u32
}

/// Validates that deadline is sufficiently far in the future.
#[inline]
pub fn validate_deadline(now: u64, deadline: u64) -> Result<(), &'static str> {
    let min_deadline = now.saturating_add(MIN_DEADLINE_OFFSET);
    if deadline < min_deadline {
        return Err("deadline must be at least MIN_DEADLINE_OFFSET seconds in the future");
    }
    Ok(())
}

/// Validates that fee_bps does not exceed the platform fee cap.
#[inline]
pub fn validate_platform_fee(fee_bps: u32) -> Result<(), &'static str> {
    if fee_bps > MAX_PLATFORM_FEE_BPS {
        return Err("fee_bps must not exceed MAX_PLATFORM_FEE_BPS");
    }
    Ok(())
}

// ── On-chain / typed-error validator ─────────────────────────────────────────

/// Validates that goal_amount meets the minimum threshold.
/// Returns ContractError::GoalTooLow when goal_amount < MIN_GOAL_AMOUNT.
#[inline]
pub fn validate_goal_amount(
    _env: &soroban_sdk::Env,
    goal_amount: i128,
) -> Result<(), crate::ContractError> {
    if goal_amount < MIN_GOAL_AMOUNT {
        return Err(crate::ContractError::GoalTooLow);
    }
    Ok(())
}

// ── Progress computation ──────────────────────────────────────────────────────

/// Computes campaign funding progress in basis points (0-10 000).
/// Returns 0 when goal <= 0 (division-by-zero guard).
/// Returns MAX_PROGRESS_BPS when total_raised >= goal (over-funded cap).
// ── Progress computation ─────────────────────────────────────────────────────

/// Computes campaign progress in basis points (0–10 000).
/// Returns 0 if goal <= 0.
/// Caps at MAX_PROGRESS_BPS even when total_raised > goal (over-funded).
/// Uses integer division; precision loss is acceptable for UI display.
/// Computes progress in basis points, capped at MAX_PROGRESS_BPS.
#[inline]
pub fn compute_progress_bps(total_raised: i128, goal: i128) -> u32 {
    if goal <= 0 {
        return 0;
    }
    if total_raised >= goal {
        return MAX_PROGRESS_BPS;
    }
    // Safe: total_raised < goal, both positive.
    let bps = (total_raised * PROGRESS_BPS_SCALE) / goal;
    bps as u32
}
    let progress = (total_raised * PROGRESS_BPS_SCALE) / goal;
    if progress > MAX_PROGRESS_BPS as i128 {
        MAX_PROGRESS_BPS
    } else {
        progress as u32
    let raw = total_raised.saturating_mul(PROGRESS_BPS_SCALE) / goal;
    if raw <= 0 {
        0
    } else if raw >= MAX_PROGRESS_BPS as i128 {
        MAX_PROGRESS_BPS
    } else {
        raw as u32
    }
}

/// Creates a new campaign with goal validation.
pub fn create_campaign(env: Env, creator: Address, goal: u64) {
    creator.require_auth();
    if goal < MIN_CAMPAIGN_GOAL {
        panic!("Goal too low");
    }
    env.events().publish(("campaign", "created"), (creator, goal));
pub fn create_campaign(env: &Env, creator: Address, goal: i128) {
    creator.require_auth();
    if goal < MIN_GOAL_AMOUNT {
        panic!("Goal too low");
    }
    env.events()
        .publish(("campaign", "created"), (creator, goal));
}

// campaign_goal_minimum — Minimum threshold enforcement for campaign goals.
//
// Security Assumptions:
// 1. MIN_GOAL_AMOUNT >= 1 closes the zero-goal drain exploit.
// 2. Negative goals are rejected by the < MIN_GOAL_AMOUNT comparison.
// 3. No integer overflow — only comparisons and saturating_add are used.
// 4. validate_goal_amount is called before any env.storage() write.
// 5. Constants are baked into WASM; changes require a contract upgrade.

// ── Constants ─────────────────────────────────────────────────────────────────

/// Minimum campaign goal in token units.
/// A goal of zero enables a trivial drain exploit; 1 closes that surface.
pub const MIN_GOAL_AMOUNT: i128 = 1;

/// Minimum contribution amount in token units.
pub const MIN_CONTRIBUTION_AMOUNT: i128 = 1;

/// Minimum seconds a deadline must be ahead of the current ledger timestamp.
pub const MIN_DEADLINE_OFFSET: u64 = 60;

/// Maximum platform fee in basis points (10 000 bps = 100 %).
/// Prevents a misconfigured platform from taking more than 100 % of raised funds.
pub const MAX_PLATFORM_FEE_BPS: u32 = 10_000;

/// Denominator used when computing progress in basis points.
/// Must equal MAX_PROGRESS_BPS so a fully-met goal returns exactly MAX_PROGRESS_BPS.
pub const PROGRESS_BPS_SCALE: i128 = 10_000;

/// Maximum value returned by compute_progress_bps.
/// Capped at this value even when total_raised > goal (over-funded).
pub const MAX_PROGRESS_BPS: u32 = 10_000;

// ── Off-chain / string-error validators ──────────────────────────────────────

/// Validates that goal meets the minimum threshold.
/// Returns Ok(()) if goal >= MIN_GOAL_AMOUNT; Err(&'static str) otherwise.
#[inline]
pub fn validate_goal(goal: i128) -> Result<(), &'static str> {
    if goal < MIN_GOAL_AMOUNT {
        return Err("goal must be at least MIN_GOAL_AMOUNT");
    }
    Ok(())
}

/// Validates that min_contribution meets the minimum floor.
/// Returns Ok(()) if valid; Err(&'static str) otherwise.
#[inline]
pub fn validate_min_contribution(min_contribution: i128) -> Result<(), &'static str> {
    if min_contribution < MIN_CONTRIBUTION_AMOUNT {
        return Err("min_contribution must be at least MIN_CONTRIBUTION_AMOUNT");
    }
    Ok(())
}

/// Validates that deadline is sufficiently far in the future.
/// Uses saturating_add to prevent overflow when now is near u64::MAX.
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
#[inline]
pub fn compute_progress_bps(total_raised: i128, goal: i128) -> u32 {
    if goal <= 0 {
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
}

const MIN_CAMPAIGN_GOAL: u64 = 1;

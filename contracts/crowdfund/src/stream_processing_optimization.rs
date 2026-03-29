//! Stream-processing helpers for gas-efficient contributor and milestone scans.
//!
//! @title Stream Processing Optimization
//! @notice Centralises read-once address stream handling and bounded aggregate scans.
//! @dev The helpers in this module are pure or storage-thin so they are easy to
//!      review, reuse, and test in isolation.

use soroban_sdk::{Address, Env, Vec};

use crate::{contract_state_size::MAX_CONTRIBUTORS, CampaignStats, DataKey};

/// Maximum number of addresses that aggregate scans will process in a single call.
///
/// @dev Kept aligned with the contributor cap so aggregate scans cannot exceed
///      the largest valid on-chain address stream.
pub const MAX_STREAM_SCAN_ITEMS: u32 = MAX_CONTRIBUTORS;

/// Scale used for basis-point progress calculations.
pub const PROGRESS_BPS_SCALE: i128 = 10_000;

/// Maximum progress value returned by stream aggregation helpers.
pub const MAX_PROGRESS_BPS: u32 = 10_000;

/// Snapshot of an address stream loaded from storage.
///
/// @dev `contains_target` is cached so callers do not need to re-scan the same
///      vector later in the transaction.
#[derive(Clone)]
pub struct AddressStreamState {
    pub entries: Vec<Address>,
    pub contains_target: bool,
}

/// Aggregated contributor statistics produced by a single bounded pass.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StreamedContributionStats {
    pub contributor_count: u32,
    pub average_contribution: i128,
    pub largest_contribution: i128,
}

/// Loads an address stream once and records whether it already contains `target`.
///
/// @param env    The Soroban environment.
/// @param key    The storage key for the address vector.
/// @param target The address being checked for membership.
/// @return Cached entries and membership state for downstream processing.
pub fn load_address_stream_state(env: &Env, key: &DataKey, target: &Address) -> AddressStreamState {
    let entries: Vec<Address> = env
        .storage()
        .persistent()
        .get(key)
        .unwrap_or_else(|| Vec::new(env));

    AddressStreamState {
        contains_target: entries.contains(target),
        entries,
    }
}

/// Persists a new address into an address stream only if it is not already present.
///
/// @param env    The Soroban environment.
/// @param key    The storage key for the address vector.
/// @param state  Mutable cached stream state loaded earlier in the transaction.
/// @param target The address to append if absent.
/// @return `true` when the address was appended, `false` when it was already present.
///
/// @custom:security This helper preserves set-like semantics for contributor and
///                  pledger lists, preventing duplicate entries from inflating
///                  downstream scans or changing accounting behaviour.
pub fn persist_address_stream_if_missing(
    env: &Env,
    key: &DataKey,
    state: &mut AddressStreamState,
    target: &Address,
) -> bool {
    if state.contains_target {
        return false;
    }

    state.entries.push_back(target.clone());
    env.storage().persistent().set(key, &state.entries);
    env.storage().persistent().extend_ttl(key, 100, 100);
    state.contains_target = true;
    true
}

/// Computes progress in basis points with saturation and zero guards.
///
/// @param total_raised The total amount raised so far.
/// @param goal         The goal being measured against.
/// @return Progress in basis points in the inclusive range `[0, 10_000]`.
///
/// @custom:security Uses `saturating_mul` to prevent overflow when large
///                  balances are scaled into basis points.
pub fn compute_progress_bps(total_raised: i128, goal: i128) -> u32 {
    if total_raised <= 0 || goal <= 0 {
        return 0;
    }

    let raw_progress = total_raised.saturating_mul(PROGRESS_BPS_SCALE) / goal;
    if raw_progress <= 0 {
        0
    } else if raw_progress >= MAX_PROGRESS_BPS as i128 {
        MAX_PROGRESS_BPS
    } else {
        raw_progress as u32
    }
}

/// Computes bonus-goal progress only when a valid bonus goal exists.
///
/// @param total_raised The total amount raised so far.
/// @param bonus_goal   Optional bonus goal threshold.
/// @return Bonus-goal progress in basis points.
pub fn bonus_goal_progress_bps(total_raised: i128, bonus_goal: Option<i128>) -> u32 {
    bonus_goal.map_or(0, |goal| compute_progress_bps(total_raised, goal))
}

/// Returns the first unmet stretch-goal milestone from an ordered stream.
///
/// @param total_raised  The total amount raised so far.
/// @param stretch_goals Ordered stretch-goal milestones.
/// @return The first milestone above `total_raised`, or `0` when all are met.
pub fn next_unmet_milestone(total_raised: i128, stretch_goals: &Vec<i128>) -> i128 {
    for milestone in stretch_goals.iter() {
        if total_raised < milestone {
            return milestone;
        }
    }

    0
}

/// Aggregates contributor statistics in one bounded scan.
///
/// @param env          The Soroban environment.
/// @param contributors The contributor address stream.
/// @param total_raised The stored total raised amount.
/// @return Count, average contribution, and largest contribution.
///
/// @custom:security Panics if the contributor stream exceeds the configured cap.
///                  This makes unexpected state growth fail closed instead of
///                  silently consuming unbounded gas.
pub fn collect_contribution_stats(
    env: &Env,
    contributors: &Vec<Address>,
    total_raised: i128,
) -> StreamedContributionStats {
    assert!(
        contributors.len() <= MAX_STREAM_SCAN_ITEMS,
        "stream_processing_optimization: contributor stream exceeds scan cap"
    );

    let contributor_count = contributors.len();
    if contributor_count == 0 {
        return StreamedContributionStats {
            contributor_count: 0,
            average_contribution: 0,
            largest_contribution: 0,
        };
    }

    let mut largest_contribution = 0i128;
    for contributor in contributors.iter() {
        let contribution: i128 = env
            .storage()
            .persistent()
            .get(&DataKey::Contribution(contributor))
            .unwrap_or(0);
        if contribution > largest_contribution {
            largest_contribution = contribution;
        }
    }

    StreamedContributionStats {
        contributor_count,
        average_contribution: total_raised / contributor_count as i128,
        largest_contribution,
    }
}

/// Builds `CampaignStats` using the bounded aggregation helpers in this module.
///
/// @param env          The Soroban environment.
/// @param total_raised The stored total raised amount.
/// @param goal         The campaign goal.
/// @param contributors The contributor address stream.
/// @return Fully-populated `CampaignStats`.
pub fn build_campaign_stats(
    env: &Env,
    total_raised: i128,
    goal: i128,
    contributors: &Vec<Address>,
) -> CampaignStats {
    let aggregate = collect_contribution_stats(env, contributors, total_raised);

    CampaignStats {
        total_raised,
        goal,
        progress_bps: compute_progress_bps(total_raised, goal),
        contributor_count: aggregate.contributor_count,
        average_contribution: aggregate.average_contribution,
        largest_contribution: aggregate.largest_contribution,
    }
}

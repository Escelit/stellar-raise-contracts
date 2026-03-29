//! Tests for stream-processing helpers used by the crowdfund contract.
//!
//! @notice Covers duplicate-prevention, bounded scans, progress computation,
//!         milestone selection, and composed campaign-stat generation.

use soroban_sdk::{
    testutils::Address as _,
    Address, Env, Vec,
};

use crate::{
    stream_processing_optimization::{
        bonus_goal_progress_bps, build_campaign_stats, collect_contribution_stats,
        compute_progress_bps, load_address_stream_state, next_unmet_milestone,
        persist_address_stream_if_missing, AddressStreamState, MAX_PROGRESS_BPS,
        MAX_STREAM_SCAN_ITEMS,
    },
    DataKey,
};

fn address_vec(env: &Env, addresses: &[Address]) -> Vec<Address> {
    let mut values = Vec::new(env);
    for address in addresses {
        values.push_back(address.clone());
    }
    values
}

#[test]
fn compute_progress_bps_returns_zero_for_invalid_inputs() {
    assert_eq!(compute_progress_bps(0, 1_000), 0);
    assert_eq!(compute_progress_bps(-1, 1_000), 0);
    assert_eq!(compute_progress_bps(1_000, 0), 0);
    assert_eq!(compute_progress_bps(1_000, -1), 0);
}

#[test]
fn compute_progress_bps_caps_overfunded_and_saturating_inputs() {
    assert_eq!(compute_progress_bps(500, 1_000), 5_000);
    assert_eq!(compute_progress_bps(1_000, 1_000), MAX_PROGRESS_BPS);
    assert_eq!(compute_progress_bps(5_000, 1_000), MAX_PROGRESS_BPS);
    assert_eq!(compute_progress_bps(i128::MAX, 1), MAX_PROGRESS_BPS);
}

#[test]
fn bonus_goal_progress_bps_tracks_optional_goal() {
    assert_eq!(bonus_goal_progress_bps(500, None), 0);
    assert_eq!(bonus_goal_progress_bps(500, Some(1_000)), 5_000);
    assert_eq!(bonus_goal_progress_bps(5_000, Some(1_000)), MAX_PROGRESS_BPS);
}

#[test]
fn next_unmet_milestone_returns_first_strictly_greater_target() {
    let env = Env::default();
    let milestones = Vec::from_array(&env, [1_000i128, 2_000i128, 3_000i128]);

    assert_eq!(next_unmet_milestone(0, &milestones), 1_000);
    assert_eq!(next_unmet_milestone(1_500, &milestones), 2_000);
    assert_eq!(next_unmet_milestone(3_000, &milestones), 0);
}

#[test]
fn load_address_stream_state_reads_existing_membership_once() {
    let env = Env::default();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let contributors = address_vec(&env, &[alice.clone(), bob.clone()]);

    env.storage()
        .persistent()
        .set(&DataKey::Contributors, &contributors);

    let state = load_address_stream_state(&env, &DataKey::Contributors, &bob);
    assert_eq!(state.entries.len(), 2);
    assert!(state.contains_target);
}

#[test]
fn persist_address_stream_if_missing_appends_once() {
    let env = Env::default();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    let mut state = AddressStreamState {
        entries: address_vec(&env, &[alice]),
        contains_target: false,
    };

    assert!(persist_address_stream_if_missing(
        &env,
        &DataKey::Contributors,
        &mut state,
        &bob,
    ));
    assert!(!persist_address_stream_if_missing(
        &env,
        &DataKey::Contributors,
        &mut state,
        &bob,
    ));

    let stored: Vec<Address> = env
        .storage()
        .persistent()
        .get(&DataKey::Contributors)
        .unwrap();
    assert_eq!(stored.len(), 2);
}

#[test]
fn collect_contribution_stats_returns_zeroes_for_empty_stream() {
    let env = Env::default();
    let contributors = Vec::new(&env);

    let stats = collect_contribution_stats(&env, &contributors, 0);
    assert_eq!(stats.contributor_count, 0);
    assert_eq!(stats.average_contribution, 0);
    assert_eq!(stats.largest_contribution, 0);
}

#[test]
fn collect_contribution_stats_scans_largest_and_average_in_one_pass() {
    let env = Env::default();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    let contributors = address_vec(&env, &[alice.clone(), bob.clone(), carol.clone()]);

    env.storage()
        .persistent()
        .set(&DataKey::Contribution(alice), &100i128);
    env.storage()
        .persistent()
        .set(&DataKey::Contribution(bob), &350i128);
    env.storage()
        .persistent()
        .set(&DataKey::Contribution(carol), &150i128);

    let stats = collect_contribution_stats(&env, &contributors, 600);
    assert_eq!(stats.contributor_count, 3);
    assert_eq!(stats.average_contribution, 200);
    assert_eq!(stats.largest_contribution, 350);
}

#[test]
#[should_panic(expected = "stream_processing_optimization: contributor stream exceeds scan cap")]
fn collect_contribution_stats_rejects_unbounded_streams() {
    let env = Env::default();
    let mut contributors = Vec::new(&env);

    for _ in 0..=MAX_STREAM_SCAN_ITEMS {
        contributors.push_back(Address::generate(&env));
    }

    let _ = collect_contribution_stats(&env, &contributors, 0);
}

#[test]
fn build_campaign_stats_composes_all_fields() {
    let env = Env::default();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let contributors = address_vec(&env, &[alice.clone(), bob.clone()]);

    env.storage()
        .persistent()
        .set(&DataKey::Contribution(alice), &250i128);
    env.storage()
        .persistent()
        .set(&DataKey::Contribution(bob), &750i128);

    let stats = build_campaign_stats(&env, 1_000, 2_000, &contributors);
    assert_eq!(stats.total_raised, 1_000);
    assert_eq!(stats.goal, 2_000);
    assert_eq!(stats.progress_bps, 5_000);
    assert_eq!(stats.contributor_count, 2);
    assert_eq!(stats.average_contribution, 500);
    assert_eq!(stats.largest_contribution, 750);
}

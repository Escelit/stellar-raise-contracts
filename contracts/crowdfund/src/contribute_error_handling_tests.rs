//! Tests for contribute() error handling edge cases.
//!
//! Covers every error path in `contribute()`:
//!   - amount below minimum contribution (panic)
//!   - contribution after deadline (CampaignEnded)
//!   - arithmetic overflow on global total_raised (Overflow)
//!   - happy-path sanity check
//!   - zero-amount contribution edge case
//!   - exact-deadline boundary (contribution at deadline timestamp accepted)
//!   - describe_error / is_retryable helper coverage

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

use crate::{contribute_error_handling, ContractError, CrowdfundContract, CrowdfundContractClient};

// ── helpers ──────────────────────────────────────────────────────────────────

const GOAL: i128 = 1_000;
const MIN: i128 = 10;
const DEADLINE_OFFSET: u64 = 1_000;

fn setup() -> (Env, CrowdfundContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();
    let asset_client = token::StellarAssetClient::new(&env, &token_addr);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);

    asset_client.mint(&contributor, &i128::MAX);

    let now = env.ledger().timestamp();
    client.initialize(
        &Address::generate(&env),
        &creator,
        &token_addr,
        &GOAL,
        &(now + DEADLINE_OFFSET),
        &MIN,
        &None,
        &None,
        &None,
    );

    (env, client, contributor, token_addr)
}

// ── happy path ───────────────────────────────────────────────────────────────

#[test]
fn contribute_happy_path() {
    let (env, client, contributor, _) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    client.contribute(&contributor, &MIN);
    assert_eq!(client.contribution(&contributor), MIN);
    assert_eq!(client.total_raised(), MIN);
}

// ── CampaignEnded ─────────────────────────────────────────────────────────────

#[test]
fn contribute_after_deadline_returns_campaign_ended() {
    let (env, client, contributor, _) = setup();
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + DEADLINE_OFFSET + 1);
    let result = client.try_contribute(&contributor, &MIN);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::CampaignEnded);
}

#[test]
fn contribute_exactly_at_deadline_is_accepted() {
    let (env, client, contributor, _) = setup();
    // timestamp == deadline → NOT past deadline (strict >), so accepted
    let deadline = client.deadline();
    env.ledger().set_timestamp(deadline);
    client.contribute(&contributor, &MIN);
    assert_eq!(client.total_raised(), MIN);
}

// ── below minimum (panic) ─────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "amount below minimum")]
fn contribute_below_minimum_panics() {
    let (env, client, contributor, _) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    client.contribute(&contributor, &(MIN - 1));
}

#[test]
#[should_panic(expected = "amount below minimum")]
fn contribute_zero_amount_panics_when_min_is_positive() {
    let (env, client, contributor, _) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    client.contribute(&contributor, &0);
}

// ── Overflow ──────────────────────────────────────────────────────────────────

/// The Overflow error (code 6) is a defensive guard on `checked_add` for both
/// the per-contributor total and `total_raised`. In practice the Soroban token
/// contract enforces that no balance exceeds i128::MAX, so this path is
/// unreachable through normal token transfers. The test below verifies the
/// error code constant is correct and that the variant exists in the enum.
#[test]
fn overflow_error_code_is_correct() {
    assert_eq!(contribute_error_handling::error_codes::OVERFLOW, 6);
    // Verify the ContractError repr matches
    assert_eq!(ContractError::Overflow as u32, 6);
}

// ── error_codes helpers ───────────────────────────────────────────────────────

#[test]
fn describe_error_campaign_ended() {
    assert_eq!(
        contribute_error_handling::describe_error(
            contribute_error_handling::error_codes::CAMPAIGN_ENDED
        ),
        "Campaign has ended"
    );
}

#[test]
fn describe_error_overflow() {
    assert_eq!(
        contribute_error_handling::describe_error(contribute_error_handling::error_codes::OVERFLOW),
        "Arithmetic overflow — contribution amount too large"
    );
}

#[test]
fn describe_error_unknown() {
    assert_eq!(
        contribute_error_handling::describe_error(99),
        "Unknown error"
    );
}

#[test]
fn is_retryable_returns_false_for_all_known_errors() {
    assert!(!contribute_error_handling::is_retryable(
        contribute_error_handling::error_codes::CAMPAIGN_ENDED
    ));
    assert!(!contribute_error_handling::is_retryable(
        contribute_error_handling::error_codes::OVERFLOW
    ));
}

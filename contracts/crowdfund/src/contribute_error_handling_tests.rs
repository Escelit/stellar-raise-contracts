//! Tests for contribute() error handling — typed errors replacing old panics.

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

#[test]
fn contribute_accumulates_multiple_contributions() {
    let (env, client, contributor, _) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    client.contribute(&contributor, &MIN);
    client.contribute(&contributor, &MIN);
    assert_eq!(client.contribution(&contributor), MIN * 2);
    assert_eq!(client.total_raised(), MIN * 2);
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
    let deadline = client.deadline();
    env.ledger().set_timestamp(deadline);
    client.contribute(&contributor, &MIN);
    assert_eq!(client.total_raised(), MIN);
}

// ── BelowMinimum (typed — replaces old panic) ─────────────────────────────────

#[test]
fn contribute_below_minimum_returns_typed_error() {
    let (env, client, contributor, _) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    let result = client.try_contribute(&contributor, &(MIN - 1));
    assert_eq!(result.unwrap_err().unwrap(), ContractError::BelowMinimum);
}

#[test]
fn contribute_one_below_minimum_returns_below_minimum() {
    let (env, client, contributor, _) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    let result = client.try_contribute(&contributor, &1);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::BelowMinimum);
}

// ── ZeroAmount (typed — replaces old pass-through) ────────────────────────────

#[test]
fn contribute_zero_amount_returns_typed_error() {
    let (env, client, contributor, _) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    let result = client.try_contribute(&contributor, &0);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::ZeroAmount);
}

// ── CampaignNotActive (typed — new guard) ─────────────────────────────────────

#[test]
fn contribute_to_cancelled_campaign_returns_not_active() {
    let (env, client, contributor, _) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    client.cancel();
    let result = client.try_contribute(&contributor, &MIN);
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::CampaignNotActive
    );
}

#[test]
fn contribute_to_successful_campaign_returns_not_active() {
    let (env, client, contributor, token_addr) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    // Fund to goal
    client.contribute(&contributor, &GOAL);
    // Advance past deadline and withdraw
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + DEADLINE_OFFSET);
    client.withdraw();
    // Now try to contribute
    let result = client.try_contribute(&contributor, &MIN);
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::CampaignNotActive
    );
    let _ = token_addr; // suppress unused warning
}

// ── Overflow error code constant ──────────────────────────────────────────────

#[test]
fn overflow_error_code_is_correct() {
    assert_eq!(contribute_error_handling::error_codes::OVERFLOW, 6);
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
fn describe_error_zero_amount() {
    assert_eq!(
        contribute_error_handling::describe_error(
            contribute_error_handling::error_codes::ZERO_AMOUNT
        ),
        "Contribution amount must be greater than zero"
    );
}

#[test]
fn describe_error_below_minimum() {
    assert_eq!(
        contribute_error_handling::describe_error(
            contribute_error_handling::error_codes::BELOW_MINIMUM
        ),
        "Contribution amount is below the minimum required"
    );
}

#[test]
fn describe_error_campaign_not_active() {
    assert_eq!(
        contribute_error_handling::describe_error(
            contribute_error_handling::error_codes::CAMPAIGN_NOT_ACTIVE
        ),
        "Campaign is not active"
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
    for code in [
        contribute_error_handling::error_codes::CAMPAIGN_ENDED,
        contribute_error_handling::error_codes::OVERFLOW,
        contribute_error_handling::error_codes::ZERO_AMOUNT,
        contribute_error_handling::error_codes::BELOW_MINIMUM,
        contribute_error_handling::error_codes::CAMPAIGN_NOT_ACTIVE,
    ] {
        assert!(!contribute_error_handling::is_retryable(code));
    }
}

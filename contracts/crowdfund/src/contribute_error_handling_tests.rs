//! Tests for contribute() error handling.
//!
//! Covers:
//! - Happy path: single and accumulated contributions
//! - `CampaignNotActive` — status guard fires first
//! - `NegativeAmount` — negative amount rejected (no diagnostic event)
//! - `ZeroAmount` / `BelowMinimum` — amount validation
//! - `CampaignEnded` — contribution after deadline; exact-deadline boundary
//! - `describe_error` / `is_retryable` helpers
//! - Diagnostic events on each logged error path

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    token, Address, Env, Symbol, TryFromVal,
};

use crate::{contribute_error_handling, ContractError, CrowdfundContract, CrowdfundContractClient};

// ── helpers ──────────────────────────────────────────────────────────────────

const GOAL: i128 = 1_000;
const MIN: i128 = 10;
const DEADLINE_OFFSET: u64 = 1_000;

fn setup() -> (Env, CrowdfundContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();
    let sac = token::StellarAssetClient::new(&env, &token_addr);

    let creator = Address::generate(&env);
    let contributor = Address::generate(&env);

    sac.mint(&contributor, &i128::MAX);

    let now = env.ledger().timestamp();
    client.initialize(
        &Address::generate(&env),
        &creator,
        &token_addr,
        &GOAL,
        &(now + DEADLINE_OFFSET),
        &MIN,
        &None::<i128>,
        &None,
        &None,
        &None,
    );

    (env, client, contributor)
}

/// Returns the last `contribute_error` event as `(variant_symbol, error_code)`.
fn last_contribute_error_event(env: &Env) -> Option<(Symbol, u32)> {
    let want = soroban_sdk::String::from_str(env, "contribute_error");
    env.events()
        .all()
        .iter()
        .rev()
        .find_map(|(_, topics, data)| {
            if topics.len() < 2 {
                return None;
            }
            let t0 = soroban_sdk::String::try_from_val(env, &topics.get(0)?).ok()?;
            if t0 != want {
                return None;
            }
            let t1 = Symbol::try_from_val(env, &topics.get(1)?).ok()?;
            let code = u32::try_from_val(env, &data).ok()?;
            Some((t1, code))
        })
}

// ── happy path ───────────────────────────────────────────────────────────────

#[test]
fn contribute_happy_path() {
    let (env, client, contributor) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    client.contribute(&contributor, &MIN);
    assert_eq!(client.contribution(&contributor), MIN);
    assert_eq!(client.total_raised(), MIN);
}

#[test]
fn contribute_accumulates_multiple_contributions() {
    let (env, client, contributor) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    client.contribute(&contributor, &MIN);
    client.contribute(&contributor, &MIN);
    assert_eq!(client.contribution(&contributor), MIN * 2);
    assert_eq!(client.total_raised(), MIN * 2);
}

// ── CampaignNotActive ────────────────────────────────────────────────────────

#[test]
fn contribute_to_finalized_campaign_returns_not_active() {
    let (env, client, contributor) = setup();
    // Advance past deadline and finalize (goal not met → Expired)
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + DEADLINE_OFFSET + 1);
    client.finalize();
    let result = client.try_contribute(&contributor, &MIN);
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::CampaignNotActive
    );
}

// ── NegativeAmount ─────────────────────────────────────────────────────────────

#[test]
fn contribute_negative_amount_returns_negative_amount_error() {
    let (env, client, contributor) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    let result = client.try_contribute(&contributor, &-1);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::NegativeAmount);
}

// ── ZeroAmount ────────────────────────────────────────────────────────────────

#[test]
fn contribute_zero_amount_returns_zero_amount_error() {
    let (env, client, contributor) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    let result = client.try_contribute(&contributor, &0);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::ZeroAmount);
}

// ── BelowMinimum ──────────────────────────────────────────────────────────────

#[test]
fn contribute_below_minimum_returns_below_minimum_error() {
    let (env, client, contributor) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    let result = client.try_contribute(&contributor, &(MIN - 1));
    assert_eq!(result.unwrap_err().unwrap(), ContractError::BelowMinimum);
}

#[test]
fn contribute_exactly_at_minimum_succeeds() {
    let (env, client, contributor) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    client.contribute(&contributor, &MIN);
    assert_eq!(client.total_raised(), MIN);
}

// ── CampaignEnded ─────────────────────────────────────────────────────────────

#[test]
fn contribute_after_deadline_returns_campaign_ended() {
    let (env, client, contributor) = setup();
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + DEADLINE_OFFSET + 1);
    let result = client.try_contribute(&contributor, &MIN);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::CampaignEnded);
}

/// Strict `>` check: contribution at exactly the deadline timestamp is accepted.
#[test]
fn contribute_exactly_at_deadline_is_accepted() {
    let (env, client, contributor) = setup();
    env.ledger().set_timestamp(client.deadline());
    client.contribute(&contributor, &MIN);
    assert_eq!(client.total_raised(), MIN);
}

#[test]
fn contribute_to_successful_campaign_returns_not_active() {
    let (env, client, contributor) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    client.contribute(&contributor, &GOAL);
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + DEADLINE_OFFSET);
    client.finalize();
    client.withdraw();
    let result = client.try_contribute(&contributor, &MIN);
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::CampaignNotActive
    );
}

// ── error_codes constants ─────────────────────────────────────────────────────

#[test]
fn error_code_constants_match_contract_error_repr() {
    use contribute_error_handling::error_codes;
    assert_eq!(
        error_codes::CAMPAIGN_ENDED,
        ContractError::CampaignEnded as u32
    );
    assert_eq!(error_codes::OVERFLOW, ContractError::Overflow as u32);
    assert_eq!(error_codes::ZERO_AMOUNT, ContractError::ZeroAmount as u32);
    assert_eq!(
        error_codes::BELOW_MINIMUM,
        ContractError::BelowMinimum as u32
    );
    assert_eq!(
        error_codes::CAMPAIGN_NOT_ACTIVE,
        ContractError::CampaignNotActive as u32
    );
    assert_eq!(
        error_codes::NEGATIVE_AMOUNT,
        ContractError::NegativeAmount as u32
    );
}

// ── describe_error ────────────────────────────────────────────────────────────

#[test]
fn describe_error_all_known_codes() {
    use contribute_error_handling::{describe_error, error_codes};
    assert_eq!(
        describe_error(error_codes::CAMPAIGN_ENDED),
        "Campaign has ended"
    );
    assert_eq!(
        describe_error(error_codes::OVERFLOW),
        "Arithmetic overflow — contribution amount too large"
    );
    assert_eq!(
        describe_error(error_codes::ZERO_AMOUNT),
        "Contribution amount must be greater than zero"
    );
    assert_eq!(
        describe_error(error_codes::BELOW_MINIMUM),
        "Contribution amount is below the minimum required"
    );
    assert_eq!(
        describe_error(error_codes::CAMPAIGN_NOT_ACTIVE),
        "Campaign is not active"
    );
    assert_eq!(
        describe_error(error_codes::NEGATIVE_AMOUNT),
        "Contribution amount must not be negative"
    );
    assert_eq!(describe_error(99), "Unknown error");
}

// ── is_retryable ──────────────────────────────────────────────────────────────

#[test]
fn is_retryable_input_errors_are_retryable() {
    use contribute_error_handling::{error_codes, is_retryable};
    assert!(is_retryable(error_codes::ZERO_AMOUNT));
    assert!(is_retryable(error_codes::BELOW_MINIMUM));
    assert!(is_retryable(error_codes::NEGATIVE_AMOUNT));
}

#[test]
fn is_retryable_state_errors_are_not_retryable() {
    use contribute_error_handling::{error_codes, is_retryable};
    assert!(!is_retryable(error_codes::CAMPAIGN_ENDED));
    assert!(!is_retryable(error_codes::CAMPAIGN_NOT_ACTIVE));
    assert!(!is_retryable(error_codes::OVERFLOW));
}

// ── diagnostic events ─────────────────────────────────────────────────────────

/// Returns the last `contribute_error` event as `(variant_symbol, error_code)`.
fn last_contribute_error_event(env: &Env) -> Option<(Symbol, u32)> {
    let topic0_str = soroban_sdk::String::from_str(env, "contribute_error");
    env.events()
        .all()
        .iter()
        .rev()
        .find_map(|(_, topics, data)| {
            if topics.len() < 2 {
                return None;
            }
            let t0 = soroban_sdk::String::from_val(env, &topics.get(0)?).ok()?;
            if t0 != topic0_str {
                return None;
            }
            let t1 = Symbol::from_val(env, &topics.get(1)?).ok()?;
            let code = u32::from_val(env, &data).ok()?;
            Some((t1, code))
        })
}

#[test]
fn error_event_emitted_on_campaign_ended() {
    let (env, client, contributor) = setup();
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + DEADLINE_OFFSET + 1);
    let _ = client.try_contribute(&contributor, &MIN);
    let (variant, code) = last_contribute_error_event(&env).expect("no event emitted");
    assert_eq!(variant, Symbol::new(&env, "CampaignEnded"));
    assert_eq!(code, contribute_error_handling::error_codes::CAMPAIGN_ENDED);
}

#[test]
fn error_event_emitted_on_zero_amount() {
    let (env, client, contributor) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    let _ = client.try_contribute(&contributor, &0);
    let (variant, code) = last_contribute_error_event(&env).expect("no event emitted");
    assert_eq!(variant, Symbol::new(&env, "ZeroAmount"));
    assert_eq!(code, contribute_error_handling::error_codes::ZERO_AMOUNT);
}

#[test]
fn error_event_emitted_on_below_minimum() {
    let (env, client, contributor) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    let _ = client.try_contribute(&contributor, &(MIN - 1));
    let (variant, code) = last_contribute_error_event(&env).expect("no event emitted");
    assert_eq!(variant, Symbol::new(&env, "BelowMinimum"));
    assert_eq!(code, contribute_error_handling::error_codes::BELOW_MINIMUM);
}

#[test]
fn error_event_emitted_on_campaign_not_active() {
    let (env, client, contributor) = setup();
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + DEADLINE_OFFSET + 1);
    client.finalize();
    let _ = client.try_contribute(&contributor, &MIN);
    let (variant, code) = last_contribute_error_event(&env).expect("no event emitted");
    assert_eq!(variant, Symbol::new(&env, "CampaignNotActive"));
    assert_eq!(
        code,
        contribute_error_handling::error_codes::CAMPAIGN_NOT_ACTIVE
    );
}

#[test]
fn no_error_event_emitted_on_success() {
    let (env, client, contributor) = setup();
    env.ledger().set_timestamp(env.ledger().timestamp() + 1);
    client.contribute(&contributor, &MIN);
    assert!(last_contribute_error_event(&env).is_none());
}

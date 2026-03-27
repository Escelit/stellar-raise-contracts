//! Additional security tests for refund_single_transfer bounds/logging improvements.
///
/// Run with:
///   cargo test -p crowdfund refund_single -- --nocapture

use super::*;
use crate::refund_single_token::refund_single_transfer;
use soroban_sdk::{Symbol, testutils::Events};

#[test]
fn test_refund_single_transfer_skips_zero_amount_no_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_id.address();
    let token_client = token::Client::new(&env, &token_address);

    let contract_address = Address::generate(&env);
    let contributor = Address::generate(&env);

    // amount = 0 should skip transfer (no token client call, no event)
    let events_before = env.events().all();
    refund_single_transfer(&token_client, &contract_address, &contributor, 0);
    let events_after = env.events().all();

    // No debug event emitted for zero amount
    assert_eq!(events_before, events_after);
    // No balance change
    assert_eq!(token_client.balance(&contributor), 0);
}

#[test]
fn test_refund_single_transfer_skips_negative_amount_no_transfer() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_id.address();
    let token_client = token::Client::new(&env, &token_address);

    let contract_address = Address::generate(&env);
    let contributor = Address::generate(&env);

    // amount < 0 should skip
    let events_before = env.events().all();
    refund_single_transfer(&token_client, &contract_address, &contributor, -1);
    let events_after = env.events().all();

    assert_eq!(events_before, events_after);
}

#[test]
fn test_refund_single_transfer_emits_debug_event_positive_amount() {
    let env = Env::default();
    env.mock_all_auths();

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_id.address();
    let token_client = token::Client::new(&env, &token_address);

    token_admin.clone().mint(&contract_address, &1000);  // Mint to contract first

    let contract_address = Address::generate(&env);
    let contributor = Address::generate(&env);
    let amount = 500i128;

    refund_single_transfer(&token_client, &contract_address, &contributor, amount);

    let events = env.events().all();
    let debug_event = events.last().unwrap();
    assert_eq!(debug_event.0, (Symbol::short("debug"), Symbol::short("refund_transfer_attempt")));
    assert_eq!(debug_event.1, (contributor, amount));
}

#[test]
fn test_refund_single_end_to_end_zero_skip_gas_optimization() {
    let (env, client, creator, token_address, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

    let contributor = Address::generate(&env);
    // Intentionally NO mint/contribute - simulate zero balance case

    env.ledger().set_timestamp(deadline + 1);

    // Should succeed without transfer (uses helper now, which skips)
    client.refund_single(&contributor);

    // No events emitted beyond normal (debug skipped for zero)
    let campaign_events: Vec<_> = env.events()
        .iter()
        .filter(|e| e.0 == (("campaign", "refund_single"), (contributor.clone(), 0i128)))
        .collect();
    assert_eq!(campaign_events.len(), 1);
}


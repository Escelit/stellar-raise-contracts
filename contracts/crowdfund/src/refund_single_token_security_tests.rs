//! Additional security tests for refund_single_transfer bounds/logging improvements.
///
/// Run with:
///   cargo test -p crowdfund refund_single -- --nocapture

use soroban_sdk::{testutils::Address as _, token, Address, Env};

use crate::refund_single_token::refund_single_transfer;

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
    let sac = token::StellarAssetClient::new(&env, &token_address);
    let token_client = token::Client::new(&env, &token_address);

    let contract_address = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    let contributor = Address::generate(&env);
    let amount = 500i128;

    // Mint tokens to the contract address so the transfer can succeed
    sac.mint(&contract_address, &amount);

    let events_before_len = env.events().all().len();
    refund_single_transfer(&token_client, &contract_address, &contributor, amount);
    let events_after = env.events().all();

    // At least one new event should have been emitted (the debug event)
    assert!(events_after.len() > events_before_len);
}

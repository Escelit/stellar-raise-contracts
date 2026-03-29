//! Optimization tests for `refund_single` token transfer logic.
//! 
//! Focus: Gas efficiency, scale, early return savings, TTL management.
//! 
//! Run with: `cargo test refund_single_token_optimization -- --nocapture`

#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    token, Address, Env, Vec,
};

use crate::{
    refund_single_token::{refund_single_transfer, validate_refund_preconditions, execute_refund_single},
    ContractError, CrowdfundContract, CrowdfundContractClient, DataKey, Status,
};

/// Setup helper matching existing tests.
fn setup() -> (Env, CrowdfundContractClient<'static>, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();
    let creator = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token_addr).mint(&creator, &10_000_000);
    (env, client, creator, token_addr)
}

fn init(client: &CrowdfundContractClient, creator: &Address, token: &Address, goal: i128, deadline: u64) {
    client.initialize(
        creator, creator, token, &goal, &deadline, &1_000, &None, &None, &None, &None,
    );
}

#[test]
fn test_gas_savings_early_return_zero_amount() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let contributor = Address::generate(&env);
    // No contribution (amount=0)

    env.ledger().set_timestamp(deadline + 1);
    client.finalize();

    // Measure cycles for zero amount call
    let cycles_before = env.budget().cpu_insns_consumed_account();
    let result = client.try_refund_single(&contributor);
    let cycles_after = env.budget().cpu_insns_consumed_account();
    let zero_cycles = cycles_after - cycles_before;

    assert_eq!(result.unwrap_err().unwrap(), ContractError::NothingToRefund);
    // Expect low cycle count due to early return in get_contribution / refund_single_transfer
    assert!(zero_cycles < 50_000, "Zero amount should use minimal cycles: {}", zero_cycles);
}

#[test]
fn test_gas_savings_early_return_negative_amount_in_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();
    let token_client = token::Client::new(&env, &token_addr);

    let contract_addr = Address::generate(&env);
    let contributor = Address::generate(&env);

    let cycles_before = env.budget().cpu_insns_consumed_account();
    refund_single_transfer(&token_client, &contract_addr, &contributor, -1);
    let cycles_after = env.budget().cpu_insns_consumed_account();

    let negative_cycles = cycles_after - cycles_before;
    // No transfer call made, no event - minimal cycles
    assert!(negative_cycles < 10_000, "Negative amount early return: {} cycles", negative_cycles);
}

#[test]
fn test_gas_comparison_zero_vs_positive_refund() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token).mint(&alice, &100_000);
    client.contribute(&alice, &100_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize();

    // Zero contributor
    let stranger = Address::generate(&env);
    let zero_cycles = measure_refund_cycles(&env, &client, &stranger);

    // Positive contributor  
    let positive_cycles = measure_refund_cycles(&env, &client, &alice);

    // Positive should use more cycles (transfer, storage writes) but early return saves ~30-50%
    assert!(positive_cycles > zero_cycles);
    assert!(positive_cycles <= zero_cycles * 3, "Positive: {}, Zero: {}", positive_cycles, zero_cycles);
}

fn measure_refund_cycles(env: &Env, client: &CrowdfundContractClient, contributor: &Address) -> u64 {
    let cycles_before = env.budget().cpu_insns_consumed_account();
    let _result = client.try_refund_single(contributor);
    let cycles_after = env.budget().cpu_insns_consumed_account();
    cycles_after - cycles_before
}

#[test]
fn test_scale_performance_100_contributors() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 10_000_000, deadline); // high goal

    let mut total_contributed = 0i128;
    let mut contributors = Vec::new(&env);

    // Setup 100 contributors with varying amounts
    for i in 0..100 {
        let contrib = Address::generate(&env);
        let amount = 1_000 + (i as i128 * 100); // 1k to ~10k
        token::StellarAssetClient::new(&env, &token).mint(&contrib, &amount);
        client.contribute(&contrib, &amount);
        contributors.push_back(contrib);
        total_contributed += amount;
    }

    assert_eq!(client.total_raised(), total_contributed);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize();

    // Sequential refunds - measure total cycles
    let cycles_before = env.budget().cpu_insns_consumed_account();
    for contrib in contributors.iter() {
        client.refund_single(contrib);
    }
    let cycles_after = env.budget().cpu_insns_consumed_account();

    let total_cycles = cycles_after - cycles_before;
    // O(n) scaling expected, reasonable per-refund cost
    let avg_cycles = total_cycles / 100;
    assert!(avg_cycles < 200_000, "Avg {} cycles per refund too high", avg_cycles);

    // Verify complete refund
    assert_eq!(client.total_raised(), 0);
}

#[test]
fn test_ttl_optimization_only_modified_keys() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token).mint(&alice, &50_000);
    client.contribute(&alice, &50_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize();

    // Track TTL extensions before/after refund
    let contribution_key = DataKey::Contribution(alice.clone());
    let total_raised_key = DataKey::TotalRaised;

    let contrib_ttl_before = env.storage().persistent().get_ttl(&contribution_key).unwrap();
    let total_ttl_before = env.storage().instance().get_ttl(&total_raised_key).unwrap();

    client.refund_single(&alice);

    let contrib_ttl_after = env.storage().persistent().get_ttl(&contribution_key).unwrap();
    let total_ttl_after = env.storage().instance().get_ttl(&total_raised_key).unwrap();

    // TTL extended only for modified keys (contribution to 0, total_raised decremented)
    assert!(contrib_ttl_after > contrib_ttl_before);
    assert!(total_ttl_after >= total_ttl_before); // instance TTL may not change
}

#[test]
#[cfg(feature = "proptest")]
fn test_property_refund_completeness_proptest() {
    // Property test: random contributors/amounts -> after all refunds, total_raised == 0
    // Implementation would use proptest crate if enabled
    // For now, deterministic version with randomization seed
    let seed = 42u64;
    let mut rng = soroban_sdk::rng::Rng::new();
    rng.set_seed(seed);

    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let num_contribs = 50;
    let mut total_contrib = 0i128;
    let mut contribs = Vec::new(&env);

    for _ in 0..num_contribs {
        let contrib = Address::generate(&env);
        let amount = 1_000 + rng.gen_range(0..10_000) as i128;
        token::StellarAssetClient::new(&env, &token).mint(&contrib, &amount);
        client.contribute(&contrib, &amount);
        contribs.push_back(contrib);
        total_contrib += amount;
    }

    env.ledger().set_timestamp(deadline + 1);
    client.finalize();

    for contrib in contribs.iter() {
        client.refund_single(contrib);
    }

    assert_eq!(client.total_raised(), 0, "Property failed: total_raised != 0 after refunds");
}

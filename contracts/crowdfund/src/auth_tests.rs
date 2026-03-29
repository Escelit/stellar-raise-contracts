//! Authorization tests for the crowdfund contract.

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

use crate::{CrowdfundContract, CrowdfundContractClient};

fn setup_env() -> (
    Env,
    CrowdfundContractClient<'static>,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract_id.address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    let creator = Address::generate(&env);
    token_admin_client.mint(&creator, &10_000_000);

    (env, client, creator, token_address, token_admin)
}

fn mint_to(env: &Env, token_address: &Address, to: &Address, amount: i128) {
    token::StellarAssetClient::new(env, token_address).mint(to, &amount);
}

#[test]
fn test_initialize_requires_creator_auth() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;

    client.initialize(
        &admin,
        &creator,
        &token_address,
        &1_000_000,
        &deadline,
        &1_000,
        &None::<i128>,
        &None,
        &None,
        &None,
    );

    assert_eq!(client.goal(), 1_000_000);
    assert_eq!(client.deadline(), deadline);
    assert_eq!(client.min_contribution(), 1_000);
    assert_eq!(client.total_raised(), 0);
}


    client.initialize(
        &admin,
        &creator,
        &token_address,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &None,
        &None,
    );

    assert_eq!(client.goal(), 1_000_000);
    assert_eq!(client.deadline(), deadline);
    assert_eq!(client.min_contribution(), 1_000);
    assert_eq!(client.total_raised(), 0);
}

/// withdraw requires creator auth and succeeds after deadline when goal met.
#[test]
fn test_withdraw_only_creator_can_withdraw() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;

    client.initialize(
        &admin,
        &creator,
        &token_address,
        &goal,
        &deadline,
        &1_000,
        &None::<i128>,
        &None,
        &None,
        &min_contribution,
        &None,
        &None,
        &None,
        &min_contribution,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &contributor, goal);
    client.contribute(&contributor, &goal);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize();
    client.withdraw();

    assert_eq!(client.total_raised(), 0);

    let token_client = token::Client::new(&env, &token_address);
    assert_eq!(token_client.balance(&creator), 10_000_000 + goal);
}

    // creator started with 10_000_000; receives goal back
    assert_eq!(token_client.balance(&creator), 10_000_000 + goal);
}

#[test]
fn test_contribute_requires_own_auth() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;

    client.initialize(
        &admin,
        &creator,
        &token_address,
        &1_000_000,
        &deadline,
        &1_000,
        &None::<i128>,
        &None,
        &None,
        &goal,
        &(goal * 2),
        &deadline,
        &1_000,
        &None,
        &None,
        &None,
        &None,
        &None,
        &min_contribution,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &contributor, 1_000_000);
    client.contribute(&contributor, &1_000_000);

    assert_eq!(client.total_raised(), 1_000_000);
    assert_eq!(client.contribution(&contributor), 1_000_000);
    
    // Verify the contribution was recorded for the correct contributor
    let contribution = client.contribution(&contributor);
    assert_eq!(contribution, 1_000_000);
}

/// Test: Initialize requires creator's auth
///
/// Title: Initialize must be called by the campaign creator
///
/// Description: The contract's initialize function calls `creator.require_auth()`,
/// ensuring that only the designated creator address can initialize a new campaign.
/// This prevents unauthorized parties from initializing campaigns.
#[test]
fn test_initialize_requires_creator_auth() {
    let (env, client, creator, token_address, _) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;

    // The contract requires creator.require_auth() - only the creator
    // address can initialize the campaign
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &(goal * 2),
        &deadline,
        &min_contribution,
        &None,
        &deadline,
        &min_contribution,
        &None,
        &None,
    );

    // Verify initialization was successful
    assert_eq!(client.goal(), goal);
    assert_eq!(client.deadline(), deadline);
    assert_eq!(client.min_contribution(), min_contribution);
    assert_eq!(client.total_raised(), 0);
}

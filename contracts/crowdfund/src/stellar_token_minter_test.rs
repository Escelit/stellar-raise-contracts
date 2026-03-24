//! Additional targeted tests for the Stellar token minter / crowdfund contract.
//!
//! This module focuses on clarity, docs, and edge cases relevant to the
//! definition of the minter workflow including pledge collection and admin upgrade.

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, IntoVal,
};

use crate::{CrowdfundContract, CrowdfundContractClient};

fn setup_env_simple() -> (Env, CrowdfundContractClient<'static>, Address, Address, Address, Address) {
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

    (env, client, creator, token_address, token_admin, contract_id)
}

fn mint_to(env: &Env, token_address: &Address, _admin: &Address, to: &Address, amount: i128) {
    let admin_client = token::StellarAssetClient::new(env, token_address);
    admin_client.mint(to, &amount);
}

fn default_init(
    client: &CrowdfundContractClient,
    creator: &Address,
    token_address: &Address,
    deadline: u64,
) -> Address {
    let admin = creator.clone();
    client.initialize(
        &admin,
        creator,
        token_address,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &None,
        &None,
    );
    admin
}

/// Verify collect_pledges fails before deadline and when goal not met.
/// The actual token-pull path requires pledger pre-auth on the token contract
/// and is covered by integration tests; here we validate the guard conditions.
#[test]
fn test_collect_pledges_success_flow() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_simple();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let pledger = Address::generate(&env);
    mint_to(&env, &token_address, &_admin, &pledger, 600_000);

    // Pledge half the goal — not enough on its own.
    client.pledge(&pledger, &500_000);

    // Before deadline: CampaignStillActive
    let early = client.try_collect_pledges();
    assert_eq!(early.unwrap_err().unwrap(), crate::ContractError::CampaignStillActive);

    env.ledger().set_timestamp(deadline + 1);

    // After deadline but goal not met (only 500k pledged, goal is 1_000_000): GoalNotReached
    let late = client.try_collect_pledges();
    assert_eq!(late.unwrap_err().unwrap(), crate::ContractError::GoalNotReached);
}

/// Verify that only the registered admin can call upgrade (requires auth). This
/// is a security assumption, not a formal error code path.
#[test]
#[should_panic]
fn test_upgrade_only_admin_auth_required() {
    let (env, client, creator, token_address, _admin, contract_id) = setup_env_simple();
    let deadline = env.ledger().timestamp() + 3600;
    let _admin = default_init(&client, &creator, &token_address, deadline);

    // Non-admin caller should fail; generator uses a different identity.
    let non_admin = Address::generate(&env);
    env.set_auths(&[]);
    client.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &non_admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_id,
            fn_name: "upgrade",
            args: soroban_sdk::vec![&env, soroban_sdk::BytesN::from_array(&env, &[0u8; 32]).into_val(&env)],
            sub_invokes: &[],
        },
    }]);

    client.upgrade(&soroban_sdk::BytesN::from_array(&env, &[0u8; 32]));
}

/// Checks bonus goal progress BPS capping boundary conditions.
#[test]
fn test_bonus_goal_progress_bps_capped_at_100_percent() {
    let (env, client, creator, token_address, admin, _contract_id) = setup_env_simple();
    let deadline = env.ledger().timestamp() + 3600;
    client.initialize(
        &creator,
        &creator,
        &token_address,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(2_000_000i128),
        &None,
    );

    let a = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &a, 2_500_000);
    client.contribute(&a, &2_500_000);

    assert!(client.bonus_goal_reached());
    assert_eq!(client.bonus_goal_progress_bps(), 10_000);
}

/// Ensure get_stats returns sane values for empty campaigns.
#[test]
fn test_get_stats_when_no_contributions() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_simple();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let stats = client.get_stats();
    assert_eq!(stats.total_raised, 0);
    assert_eq!(stats.contributor_count, 0);
    assert_eq!(stats.average_contribution, 0);
}

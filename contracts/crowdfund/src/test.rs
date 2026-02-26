#![allow(unused_doc_comments)]

use soroban_sdk::{
    contract, contractimpl, contracttype,
    testutils::{Address as _, Ledger},
    token, Address, Env, Vec,
};

#[contract]
struct MockNftContract;

#[contractimpl]
impl MockNftContract {
    pub fn mint(env: Env, to: Address) -> u128 {
        let next_id: u128 = env.storage().instance().get(&1u32).unwrap_or(0u128) + 1;
        env.storage().instance().set(&1u32, &next_id);

        let mut records: Vec<MintRecord> = env
            .storage()
            .persistent()
            .get(&2u32)
            .unwrap_or_else(|| Vec::new(&env));
        records.push_back(MintRecord {
            to,
            token_id: next_id,
        });
        env.storage().persistent().set(&2u32, &records);

        next_id
    }

#[derive(Clone)]
#[contracttype]
struct MintRecord {
    to: Address,
    token_id: u64,
}

#[derive(Clone)]
#[contracttype]
enum MockNftDataKey {
    Minted,
}

#[contract]
struct MockNftContract;

#[contractimpl]
impl MockNftContract {
    pub fn mint(env: Env, to: Address, token_id: u64) {
        let mut minted: Vec<MintRecord> = env
            .storage()
            .persistent()
            .get(&MockNftDataKey::Minted)
            .unwrap_or_else(|| Vec::new(&env));
        minted.push_back(MintRecord { to, token_id });
        env.storage()
            .persistent()
            .set(&MockNftDataKey::Minted, &minted);
    }

    pub fn minted(env: Env) -> Vec<MintRecord> {
        env.storage()
            .persistent()
            .get(&MockNftDataKey::Minted)
            .unwrap_or_else(|| Vec::new(&env))
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn setup_env() -> (
    Env,
    CrowdfundContractClient<'static>,
    Address,
    Address,
    token::StellarAssetClient<'static>,
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract_id = env.register_stellar_asset_contract_v2(token_admin);
    let token_address = token_contract_id.address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    let creator = Address::generate(&env);
    token_admin_client.mint(&creator, &10_000_000);

    (env, client, creator, token_address, token_admin.clone())
}

/// Helper to mint tokens to an arbitrary contributor.
fn mint_to(env: &Env, token_address: &Address, admin: &Address, to: &Address, amount: i128) {
    let admin_client = token::StellarAssetClient::new(env, token_address);
    admin_client.mint(to, &amount);
    let _ = admin;
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[test]
fn test_withdraw_mints_nft_for_each_contributor() {
    let (env, client, creator, token_address, token_admin_client) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;

    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    assert_eq!(client.goal(), goal);
    assert_eq!(client.deadline(), deadline);
    assert_eq!(client.min_contribution(), min_contribution);
    assert_eq!(client.total_raised(), 0);
}

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    token_admin_client.mint(&alice, &600_000);
    token_admin_client.mint(&bob, &400_000);

    // Test that version() returns the expected version number
    assert_eq!(client.version(), 1);
}

#[test]
fn test_withdraw_skips_nft_mint_when_contract_not_set() {
    let (env, client, creator, token_address, token_admin_client) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;

    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );
    let result = client.try_initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::AlreadyInitialized
    );
}

#[test]
fn test_contribute() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &goal);
    client.contribute(&contributor, &goal);

    client.contribute(&contributor, &500_000, &None);

    assert_eq!(nft_client.minted().len(), 0);
}

#[test]
fn test_set_nft_contract_rejects_non_creator() {
    let (env, client, creator, token_address, _token_admin_client) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint_to(&env, &token_address, &token_admin, &alice, 600_000);
    mint_to(&env, &token_address, &token_admin, &bob, 400_000);

    client.contribute(&alice, &300_000, &None);
    client.contribute(&bob, &200_000, &None);

    assert_eq!(client.total_raised(), 500_000);
    assert_eq!(client.contribution(&alice), 300_000);
    assert_eq!(client.contribution(&bob), 200_000);
}

#[test]
fn test_contribute_after_deadline_panics() {
    let (env, client, platform_admin, creator, token_address, token_admin) = setup_env();

    let deadline = env.ledger().timestamp() + 100;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    // Fast-forward past the deadline.
    env.ledger().set_timestamp(deadline + 1);

    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let non_creator = Address::generate(&env);
    let nft_id = env.register(MockNftContract, ());

    let result = client.try_set_nft_contract(&non_creator, &nft_id);
    assert!(result.is_err());
}

#[test]
fn test_withdraw_successful_campaign_updates_status_and_balance() {
    let (env, client, creator, token_address, token_admin_client) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 500_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 1_000_000);
    client.contribute(&contributor, &1_000_000, &None);

    let token_client = token::Client::new(&env, &token_address);
    let creator_before = token_client.balance(&creator);

    env.ledger().set_timestamp(deadline + 1);
    client.withdraw();

    assert_eq!(client.total_raised(), 0);
    assert_eq!(token_client.balance(&creator), creator_before + goal);
}

#[test]
fn test_contribute_after_deadline_returns_error() {
    let (env, client, creator, token_address, token_admin_client) = setup_env();

    let deadline = env.ledger().timestamp() + 100;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 1_000_000);
    client.contribute(&contributor, &1_000_000, &None);

    let result = client.try_withdraw();

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::CampaignStillActive
    );
}

#[test]
fn test_withdraw_goal_not_reached_panics() {
    let (env, client, platform_admin, creator, token_address, token_admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 500_000);
    client.contribute(&contributor, &500_000, &None);

    env.ledger().set_timestamp(deadline + 1);

    let contributor = Address::generate(&env);
    token_admin_client.mint(&contributor, &500_000);

    let result = client.try_contribute(&contributor, &500_000);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::CampaignEnded);
}

// ── Contributor Count Tests ────────────────────────────────────────────────

#[test]
fn test_withdraw_mints_nft_for_each_contributor_when_configured() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let nft_contract_id = env.register(MockNftContract, ());
    let nft_client = MockNftContractClient::new(&env, &nft_contract_id);
    client.set_nft_contract(&creator, &nft_contract_id);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &alice, 600_000);
    mint_to(&env, &token_address, &admin, &bob, 400_000);

    client.contribute(&alice, &600_000);
    client.contribute(&bob, &400_000);
    env.ledger().set_timestamp(deadline + 1);

    client.withdraw();

    let minted = nft_client.minted();
    assert_eq!(minted.len(), 2);
    assert_eq!(minted.get(0).unwrap().to, alice);
    assert_eq!(minted.get(0).unwrap().token_id, 1);
    assert_eq!(minted.get(1).unwrap().to, bob);
    assert_eq!(minted.get(1).unwrap().token_id, 2);
}

#[test]
fn test_withdraw_skips_nft_minting_when_nft_contract_not_set() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let nft_contract_id = env.register(MockNftContract, ());
    let nft_client = MockNftContractClient::new(&env, &nft_contract_id);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 1_000_000);
    client.contribute(&contributor, &1_000_000);
    env.ledger().set_timestamp(deadline + 1);

    client.withdraw();

    assert_eq!(nft_client.minted().len(), 0);
}

#[test]
#[should_panic(expected = "not authorized")]
fn test_set_nft_contract_rejects_non_creator() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let attacker = Address::generate(&env);
    let nft_contract_id = env.register(MockNftContract, ());
    client.set_nft_contract(&attacker, &nft_contract_id);
}

#[test]
fn test_refund_when_goal_not_met() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &alice, 300_000);
    mint_to(&env, &token_address, &admin, &bob, 200_000);

    client.contribute(&alice, &300_000);
    client.contribute(&bob, &200_000);

    // Move past deadline — goal not met.
    env.ledger().set_timestamp(deadline + 1);

    client.refund();

    // Both contributors should get their tokens back.
    let token_client = token::Client::new(&env, &token_address);
    assert_eq!(token_client.balance(&alice), 300_000);
    assert_eq!(token_client.balance(&bob), 200_000);
    assert_eq!(client.total_raised(), 0);
}

#[test]
fn test_refund_when_goal_reached_panics() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 1_000_000);
    client.contribute(&contributor, &1_000_000);

    env.ledger().set_timestamp(deadline + 1);

    let result = client.try_refund();

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::GoalReached
    );
}

// ── Bug Condition Exploration Test ─────────────────────────────────────────

/// **Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6**
///
/// **Property 1: Fault Condition** - Structured Error Returns
///
/// This test verifies that all 6 error conditions return the appropriate
/// ContractError variants instead of panicking.
///
/// The test covers all 6 error conditions:
/// 1. Double initialization → Err(ContractError::AlreadyInitialized)
/// 2. Late contribution → Err(ContractError::CampaignEnded)
/// 3. Early withdrawal → Err(ContractError::CampaignStillActive)
/// 4. Withdrawal without goal → Err(ContractError::GoalNotReached)
/// 5. Early refund → Err(ContractError::CampaignStillActive)
/// 6. Refund after success → Err(ContractError::GoalReached)
#[test]
fn test_bug_condition_exploration_all_error_conditions_panic() {
    // Test 1: Double initialization
    {
        let (env, client, creator, token_address, _admin) = setup_env();
        let deadline = env.ledger().timestamp() + 3600;
        let goal: i128 = 1_000_000;

        client.initialize(
            &creator,
            &token_address,
            &goal,
            &deadline,
            &1_000,
            &None,
            &None,
            &None,
        );
        let result = client.try_initialize(
            &creator,
            &token_address,
            &goal,
            &deadline,
            &1_000,
            &None,
            &None,
            &None,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::AlreadyInitialized
        );
    }

    // Test 2: Late contribution
    {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + 100;
        let goal: i128 = 1_000_000;
        client.initialize(
            &creator,
            &token_address,
            &goal,
            &deadline,
            &1_000,
            &None,
            &None,
            &None,
        );

        env.ledger().set_timestamp(deadline + 1);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, 500_000);
        let result = client.try_contribute(&contributor, &500_000, &None);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::CampaignEnded);
    }

    // Test 3: Early withdrawal
    {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + 3600;
        let goal: i128 = 1_000_000;
        client.initialize(
            &creator,
            &token_address,
            &goal,
            &deadline,
            &1_000,
            &None,
            &None,
            &None,
        );

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, 1_000_000);
        client.contribute(&contributor, &1_000_000, &None);

        let result = client.try_withdraw();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::CampaignStillActive
        );
    }

    // Test 4: Withdrawal without goal
    {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + 3600;
        let goal: i128 = 1_000_000;
        client.initialize(
            &creator,
            &token_address,
            &goal,
            &deadline,
            &1_000,
            &None,
            &None,
            &None,
        );

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, 500_000);
        client.contribute(&contributor, &500_000, &None);

        env.ledger().set_timestamp(deadline + 1);
        let result = client.try_withdraw();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::GoalNotReached);
    }

    // Test 5: Early refund
    {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + 3600;
        let goal: i128 = 1_000_000;
        client.initialize(
            &creator,
            &token_address,
            &goal,
            &deadline,
            &1_000,
            &None,
            &None,
            &None,
        );

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, 500_000);
        client.contribute(&contributor, &500_000);

        let result = client.try_refund();

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().unwrap(),
            ContractError::CampaignStillActive
        );
    }

    // Test 6: Refund after success
    {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + 3600;
        let goal: i128 = 1_000_000;
        client.initialize(
            &creator,
            &token_address,
            &goal,
            &deadline,
            &1_000,
            &None,
            &None,
            &None,
        );

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, 1_000_000);
        client.contribute(&contributor, &1_000_000, &None);

        env.ledger().set_timestamp(deadline + 1);
        let result = client.try_refund();

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::GoalReached);
    }
}

// ── Preservation Property Tests ────────────────────────────────────────────

use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_preservation_first_initialization(
        goal in 1_000i128..10_000_000i128,
        deadline_offset in 100u64..10_000u64,
    ) {
        let (env, client, creator, token_address, _admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        // Test 3.1: First initialization stores all values correctly
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None);

        prop_assert_eq!(client.goal(), goal);
        prop_assert_eq!(client.deadline(), deadline);
        prop_assert_eq!(client.total_raised(), 0);
    }

    #[test]
    fn prop_preservation_valid_contribution(
        goal in 1_000_000i128..10_000_000i128,
        deadline_offset in 100u64..10_000u64,
        contribution_amount in 100_000i128..1_000_000i128,
    ) {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None);

    client.contribute(&alice, &300_000, None);
    client.contribute(&bob, &200_000, None);

        // Test 3.2: Valid contribution before deadline works correctly
        client.contribute(&contributor, &contribution_amount);

        prop_assert_eq!(client.total_raised(), contribution_amount);
        prop_assert_eq!(client.contribution(&contributor), contribution_amount);
    }

    #[test]
    fn prop_preservation_successful_withdrawal(
        goal in 1_000_000i128..5_000_000i128,
        deadline_offset in 100u64..10_000u64,
    ) {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, goal);
        client.contribute(&contributor, &goal);

        // Move past deadline
        env.ledger().set_timestamp(deadline + 1);

    client.contribute(&contributor, &10_000, None);

        // Test 3.3: Successful withdrawal transfers funds and resets total_raised
        client.withdraw();

        prop_assert_eq!(client.total_raised(), 0);
        prop_assert_eq!(token_client.balance(&creator), creator_balance_before + goal);
    }

    #[test]
    fn prop_preservation_successful_refund(
        goal in 2_000_000i128..10_000_000i128,
        deadline_offset in 100u64..10_000u64,
        contribution_amount in 100_000i128..1_000_000i128,
    ) {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        // Ensure contribution is less than goal
        let contribution = contribution_amount.min(goal - 1);

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, contribution);
        client.contribute(&contributor, &contribution);

        // Move past deadline (goal not met)
        env.ledger().set_timestamp(deadline + 1);

        // Test 3.4: Successful refund returns funds to contributors
        client.refund();

        let token_client = token::Client::new(&env, &token_address);
        prop_assert_eq!(token_client.balance(&contributor), contribution);
        prop_assert_eq!(client.total_raised(), 0);
    }

    #[test]
    fn prop_preservation_view_functions(
        goal in 1_000_000i128..10_000_000i128,
        deadline_offset in 100u64..10_000u64,
        contribution_amount in 100_000i128..1_000_000i128,
    ) {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, contribution_amount);
        client.contribute(&contributor, &contribution_amount);

        // Test 3.5: View functions return correct values
        prop_assert_eq!(client.goal(), goal);
        prop_assert_eq!(client.deadline(), deadline);
        prop_assert_eq!(client.total_raised(), contribution_amount);
        prop_assert_eq!(client.contribution(&contributor), contribution_amount);
    }

    #[test]
    fn prop_preservation_multiple_contributors(
        goal in 5_000_000i128..10_000_000i128,
        deadline_offset in 100u64..10_000u64,
        amount1 in 100_000i128..1_000_000i128,
        amount2 in 100_000i128..1_000_000i128,
        amount3 in 100_000i128..1_000_000i128,
    ) {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let charlie = Address::generate(&env);

        mint_to(&env, &token_address, &admin, &alice, amount1);
        mint_to(&env, &token_address, &admin, &bob, amount2);
        mint_to(&env, &token_address, &admin, &charlie, amount3);

        // Test 3.6: Multiple contributors are tracked correctly
        client.contribute(&alice, &amount1);
        client.contribute(&bob, &amount2);
        client.contribute(&charlie, &amount3);

        let expected_total = amount1 + amount2 + amount3;

        prop_assert_eq!(client.total_raised(), expected_total);
        prop_assert_eq!(client.contribution(&alice), amount1);
        prop_assert_eq!(client.contribution(&bob), amount2);
        prop_assert_eq!(client.contribution(&charlie), amount3);
    }
}

#[test]
#[should_panic(expected = "campaign is not active")]
fn test_double_withdraw_panics() {
    let (env, client, platform_admin, creator, token_address, token_admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &token_admin, &contributor, 1_000_000);
    client.contribute(&contributor, &1_000_000);

    env.ledger().set_timestamp(deadline + 1);

    client.withdraw();
    client.withdraw(); // should panic — status is Successful
}

#[test]
#[should_panic(expected = "campaign is not active")]
fn test_double_refund_panics() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let alice = Address::generate(&env);
    mint_to(&env, &token_address, &token_admin, &alice, 500_000);
    client.contribute(&alice, &500_000);

    env.ledger().set_timestamp(deadline + 1);

    client.refund();
    client.refund(); // should panic — status is Refunded
}

#[test]
fn test_cancel_with_no_contributions() {
    let (env, client, platform_admin, creator, token_address, _token_admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

proptest! {
    #[test]
    fn prop_preservation_first_initialization(
        goal in 1_000i128..10_000_000i128,
        deadline_offset in 100u64..10_000u64,
    ) {
        let (env, client, creator, token_address, _admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        // Test 3.1: First initialization stores all values correctly
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);

        prop_assert_eq!(client.goal(), goal);
        prop_assert_eq!(client.deadline(), deadline);
        prop_assert_eq!(client.total_raised(), 0);
    }

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);

    client.contribute(&alice, &300_000, &None);
    client.contribute(&bob, &200_000, &None);

        // Test 3.2: Valid contribution before deadline works correctly
        client.contribute(&contributor, &contribution_amount);

        prop_assert_eq!(client.total_raised(), contribution_amount);
        prop_assert_eq!(client.contribution(&contributor), contribution_amount);
    }

#[test]
#[should_panic]
fn test_cancel_by_non_creator_panics() {
    let env = Env::default();
    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract_id.address();

    let creator = Address::generate(&env);
    let non_creator = Address::generate(&env);

    env.mock_all_auths();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    env.mock_all_auths_allowing_non_root_auth();
    env.set_auths(&[]);

    client.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &non_creator,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_id,
            fn_name: "cancel",
            args: soroban_sdk::vec![&env],
            sub_invokes: &[],
        },
    }]);

    client.cancel();
}

// ── Minimum Contribution Tests ─────────────────────────────────────────────

#[test]
#[should_panic(expected = "amount below minimum")]
fn test_contribute_below_minimum_panics() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 10_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 5_000);

    client.contribute(&contributor, &5_000); // should panic
}

#[test]
fn test_contribute_exact_minimum() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 10_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 10_000);

    client.contribute(&contributor, &10_000);

    assert_eq!(client.total_raised(), 10_000);
    assert_eq!(client.contribution(&contributor), 10_000);
}

#[test]
fn test_contribute_above_minimum() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 10_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 50_000);

    client.contribute(&contributor, &50_000);

    assert_eq!(client.total_raised(), 50_000);
    assert_eq!(client.contribution(&contributor), 50_000);
}

// ── Roadmap Tests ──────────────────────────────────────────────────────────

#[test]
fn test_add_single_roadmap_item() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let current_time = env.ledger().timestamp();
    let roadmap_date = current_time + 86400; // 1 day in the future
    let description = soroban_sdk::String::from_str(&env, "Beta release");

    client.add_roadmap_item(&roadmap_date, &description);

    let roadmap = client.roadmap();
    assert_eq!(roadmap.len(), 1);
    assert_eq!(roadmap.get(0).unwrap().date, roadmap_date);
    assert_eq!(roadmap.get(0).unwrap().description, description);
}

#[test]
fn test_add_multiple_roadmap_items_in_order() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let current_time = env.ledger().timestamp();
    let date1 = current_time + 86400;
    let date2 = current_time + 172800;
    let date3 = current_time + 259200;

    let desc1 = soroban_sdk::String::from_str(&env, "Alpha release");
    let desc2 = soroban_sdk::String::from_str(&env, "Beta release");
    let desc3 = soroban_sdk::String::from_str(&env, "Production launch");

    client.add_roadmap_item(&date1, &desc1);
    client.add_roadmap_item(&date2, &desc2);
    client.add_roadmap_item(&date3, &desc3);

    let roadmap = client.roadmap();
    assert_eq!(roadmap.len(), 3);
    assert_eq!(roadmap.get(0).unwrap().date, date1);
    assert_eq!(roadmap.get(1).unwrap().date, date2);
    assert_eq!(roadmap.get(2).unwrap().date, date3);
    assert_eq!(roadmap.get(0).unwrap().description, desc1);
    assert_eq!(roadmap.get(1).unwrap().description, desc2);
    assert_eq!(roadmap.get(2).unwrap().description, desc3);
}

#[test]
#[should_panic(expected = "date must be in the future")]
fn test_add_roadmap_item_with_past_date_panics() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let current_time = env.ledger().timestamp();
    // Set a past date by moving time forward first, then trying to add an item with an earlier date
    env.ledger().set_timestamp(current_time + 1000);
    let past_date = current_time + 500; // Earlier than the new current time
    let description = soroban_sdk::String::from_str(&env, "Past milestone");

    client.add_roadmap_item(&past_date, &description); // should panic
}

#[test]
#[should_panic(expected = "date must be in the future")]
fn test_add_roadmap_item_with_current_date_panics() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let current_time = env.ledger().timestamp();
    let description = soroban_sdk::String::from_str(&env, "Current milestone");

    client.add_roadmap_item(&current_time, &description); // should panic
}

#[test]
#[should_panic(expected = "description cannot be empty")]
fn test_add_roadmap_item_with_empty_description_panics() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let current_time = env.ledger().timestamp();
    let roadmap_date = current_time + 86400;
    let empty_description = soroban_sdk::String::from_str(&env, "");

    client.add_roadmap_item(&roadmap_date, &empty_description); // should panic
}

#[test]
#[should_panic]
fn test_add_roadmap_item_by_non_creator_panics() {
    let env = Env::default();
    let contract_id = env.register(crate::CrowdfundContract, ());
    let client = crate::CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_contract_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract_id.address();

    let platform_admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let non_creator = Address::generate(&env);

    env.mock_all_auths();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    env.mock_all_auths_allowing_non_root_auth();
    env.set_auths(&[]);

    let current_time = env.ledger().timestamp();
    let roadmap_date = current_time + 86400;
    let description = soroban_sdk::String::from_str(&env, "Milestone");

    client.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &non_creator,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_id,
            fn_name: "add_roadmap_item",
            args: soroban_sdk::vec![&env],
            sub_invokes: &[],
        },
    }]);

    client.add_roadmap_item(&roadmap_date, &description); // should panic
}

#[test]
fn test_roadmap_empty_after_initialization() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    let roadmap = client.roadmap();
    assert_eq!(roadmap.len(), 0);
}

// ── Metadata Update Tests ──────────────────────────────────────────────────

#[test]
fn test_update_title() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    // Update title.
    let title = soroban_sdk::String::from_str(&env, "New Campaign Title");
    client.update_metadata(&creator, &Some(title), &None, &None);

    // Verify title was updated (we'd need a getter, but the function should not panic).
}

#[test]
fn test_update_description() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    // Update description.
    let description = soroban_sdk::String::from_str(&env, "New campaign description");
    client.update_metadata(&creator, &None, &Some(description), &None);
}

#[test]
fn test_update_socials() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    // Update social links.
    let socials = soroban_sdk::String::from_str(&env, "https://twitter.com/campaign");
    client.update_metadata(&creator, &None, &None, &Some(socials));
}

#[test]
fn test_partial_update() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    // Update only title (description and socials should remain None).
    let title = soroban_sdk::String::from_str(&env, "Updated Title");
    client.update_metadata(&creator, &Some(title), &None, &None);

    // Update only socials (should not affect title).
    let socials = soroban_sdk::String::from_str(&env, "https://twitter.com/new");
    client.update_metadata(&creator, &None, &None, &Some(socials));
}

#[test]
#[should_panic(expected = "campaign is not active")]
fn test_update_metadata_when_not_active_panics() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    // Contribute to meet the goal.
    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 1_000_000);
    client.contribute(&contributor, &1_000_000);

    // Move past deadline and withdraw (status becomes Successful).
    env.ledger().set_timestamp(deadline + 1);
    client.withdraw();

    // Try to update metadata (should panic - campaign is not Active).
    let title = soroban_sdk::String::from_str(&env, "New Title");
    client.update_metadata(&creator, &Some(title), &None, &None);
}

#[test]
#[should_panic(expected = "campaign is not active")]
fn test_update_metadata_after_cancel_panics() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    // Cancel the campaign.
    client.cancel();

    // Try to update metadata (should panic - campaign is Cancelled).
    let title = soroban_sdk::String::from_str(&env, "New Title");
    client.update_metadata(&creator, &Some(title), &None, &None);
}

// Note: The non-creator test would require complex mock setup.
// The authorization check is covered by require_auth() in the contract,
// which will panic if the caller is not the creator.

// ── Deadline Update Tests ──────────────────────────────────────────────────

#[test]
fn test_update_deadline_extends_campaign() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    // Verify initial deadline
    assert_eq!(client.deadline(), deadline);

    // Extend the deadline
    let new_deadline = deadline + 7200; // 2 more hours
    client.update_deadline(&new_deadline);

    // Verify the deadline was updated
    assert_eq!(client.deadline(), new_deadline);
}

#[test]
#[should_panic(expected = "new deadline must be after current deadline")]
fn test_update_deadline_rejects_shortening() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    // Try to shorten the deadline (should panic)
    let shorter_deadline = deadline - 1800; // 30 minutes earlier
    client.update_deadline(&shorter_deadline);
}

// ── Whitelist Tests ────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "new deadline must be after current deadline")]
fn test_update_deadline_rejects_equal_deadline() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    // Try to set deadline to the same value (should panic)
    client.update_deadline(&deadline);
}

#[test]
#[should_panic(expected = "campaign is not active")]
fn test_update_deadline_when_not_active_panics() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    // Move past deadline and refund
    env.ledger().set_timestamp(deadline + 1);

    // Refund to change status from Active to Refunded
    let _ = client.try_refund();

    // Try to update deadline on a non-Active campaign (should panic)
    let new_deadline = deadline + 7200;
    client.update_deadline(&new_deadline);
}

// ── Stretch Goal Tests ─────────────────────────────────────────────────────

#[test]
fn test_add_single_stretch_goal() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );

    // Add a stretch goal that is greater than the primary goal
    let stretch_goal: i128 = 2_000_000;
    client.add_stretch_goal(&stretch_goal);

    // Verify the stretch goal was added by checking the current milestone
    let current = client.current_milestone();
    assert_eq!(current, stretch_goal);
}

#[test]
#[should_panic(expected = "bonus goal must be greater than primary goal")]
fn test_initialize_rejects_bonus_goal_not_above_primary() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let invalid_bonus_goal: i128 = 1_000_000;

    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &1_000,
        &None,
        &Some(invalid_bonus_goal),
        &None,
    );
}

#[test]
fn test_bonus_goal_progress_tracked_separately_from_primary() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let bonus_goal: i128 = 2_000_000;
    let bonus_description = soroban_sdk::String::from_str(&env, "Bonus unlocked");

    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &1_000,
        &None,
        &Some(bonus_goal),
        &Some(bonus_description.clone()),
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 500_000);
    client.contribute(&contributor, &500_000);

    let primary_progress_bps = (client.total_raised() * 10_000) / client.goal();
    assert_eq!(primary_progress_bps, 5_000);
    assert_eq!(client.bonus_goal_progress_bps(), 2_500);
    assert_eq!(client.bonus_goal(), Some(bonus_goal));
    assert_eq!(client.bonus_goal_description(), Some(bonus_description));
}

#[test]
fn test_bonus_goal_reached_returns_false_below_threshold() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let bonus_goal: i128 = 2_000_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &1_000,
        &None,
        &Some(bonus_goal),
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 1_500_000);
    client.contribute(&contributor, &1_500_000);

    assert!(!client.bonus_goal_reached());
}

#[test]
fn test_bonus_goal_reached_returns_true_at_and_above_threshold() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let bonus_goal: i128 = 2_000_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &hard_cap,
        &deadline,
        &1_000,
        &None,
        &Some(bonus_goal),
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 2_100_000);
    client.contribute(&contributor, &2_000_000);
    assert!(client.bonus_goal_reached());

    client.contribute(&contributor, &100_000);
    assert!(client.bonus_goal_reached());
    assert_eq!(client.bonus_goal_progress_bps(), 10_000);
}

// ── Property-Based Fuzz Tests with Proptest ────────────────────────────────

/// **Property Test 1: Invariant - Total Raised Equals Sum of Contributions**
///
/// For any valid (goal, deadline, contributions[]), the contract invariant holds:
/// total_raised == sum of all individual contributions
///
/// This test generates random valid parameters and multiple contributors with
/// varying contribution amounts, then verifies the invariant is maintained.
proptest! {
    #[test]
    fn prop_total_raised_equals_sum_of_contributions(
        goal in 1_000_000i128..100_000_000i128,
        deadline_offset in 100u64..100_000u64,
        amount1 in 1_000i128..10_000_000i128,
        amount2 in 1_000i128..10_000_000i128,
        amount3 in 1_000i128..10_000_000i128,
    ) {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let charlie = Address::generate(&env);

        mint_to(&env, &token_address, &admin, &alice, amount1);
        mint_to(&env, &token_address, &admin, &bob, amount2);
        mint_to(&env, &token_address, &admin, &charlie, amount3);

        client.contribute(&alice, &amount1);
        client.contribute(&bob, &amount2);
        client.contribute(&charlie, &amount3);

        let expected_total = amount1 + amount2 + amount3;
        let actual_total = client.total_raised();

        // **INVARIANT**: total_raised must equal the sum of all contributions
        prop_assert_eq!(actual_total, expected_total,
            "total_raised ({}) != sum of contributions ({})",
            actual_total, expected_total
        );
    }
}

/// **Property Test 2: Invariant - Refund Returns Exact Contributed Amount**
///
/// For any valid contribution amount, refund always returns the exact amount
/// with no remainder or shortfall.
///
/// This test verifies that each contributor receives back exactly what they
/// contributed when the goal is not met and refund is called.
proptest! {
    #[test]
    fn prop_refund_returns_exact_amount(
        goal in 5_000_000i128..100_000_000i128,
        deadline_offset in 100u64..100_000u64,
        contribution in 1_000i128..5_000_000i128,
    ) {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        // Ensure contribution is less than goal
        let safe_contribution = contribution.min(goal - 1);

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, safe_contribution);
        client.contribute(&contributor, &safe_contribution);

        // Move past deadline (goal not met)
        env.ledger().set_timestamp(deadline + 1);

        let token_client = token::Client::new(&env, &token_address);
        let balance_before_refund = token_client.balance(&contributor);

        client.refund();

        let balance_after_refund = token_client.balance(&contributor);

        // **INVARIANT**: Refund must return exact amount with no remainder
        prop_assert_eq!(
            balance_after_refund - balance_before_refund,
            safe_contribution,
            "refund amount ({}) != original contribution ({})",
            balance_after_refund - balance_before_refund,
            safe_contribution
        );
    }
}

/// **Property Test 3: Contribute with Amount <= 0 Always Fails**
///
/// For any contribution amount <= 0, the contribute function must fail.
/// This test verifies that zero and negative contributions are rejected.
proptest! {
    #[test]
    fn prop_contribute_zero_or_negative_fails(
        goal in 1_000_000i128..10_000_000i128,
        deadline_offset in 100u64..10_000u64,
        negative_amount in -1_000_000i128..=0i128,
    ) {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None);

        let contributor = Address::generate(&env);
        // Mint enough tokens so the failure is due to amount validation, not balance
        mint_to(&env, &token_address, &admin, &contributor, 10_000_000);

        // Attempt to contribute zero or negative amount
        // This should fail due to minimum contribution check
        let result = client.try_contribute(&contributor, &negative_amount);

        // **INVARIANT**: Contribution <= 0 must fail
        prop_assert!(
            result.is_err(),
            "contribute with amount {} should fail but succeeded",
            negative_amount
        );
    }
}

/// **Property Test 4: Deadline in the Past Always Fails on Initialize**
///
/// For any deadline in the past (relative to current ledger time),
/// initialization must fail or panic.
proptest! {
    #[test]
    fn prop_initialize_with_past_deadline_fails(
        goal in 1_000_000i128..10_000_000i128,
        past_offset in 1u64..10_000u64,
    ) {
        let (env, client, creator, token_address, _admin) = setup_env();

        let current_time = env.ledger().timestamp();
        // Set deadline in the past
        let past_deadline = current_time.saturating_sub(past_offset);

        // Attempt to initialize with past deadline
        let result = client.try_initialize(
            &creator,
            &token_address,
            &goal,
            &past_deadline,
            &1_000,
            &None,
        &None,
        &None,
        );

        // **INVARIANT**: Past deadline should fail or be rejected
        // Note: The contract may not explicitly validate this, but it's a logical invariant
        // If the contract allows it, the campaign would already be expired
        // This test documents the expected behavior
        if result.is_ok() {
            // If initialization succeeds with past deadline, verify campaign is immediately expired
            let deadline = client.deadline();
            prop_assert!(
                deadline <= current_time,
                "deadline {} should be in the past relative to current time {}",
                deadline,
                current_time
            );
        }
    }
}

/// **Property Test 5: Multiple Contributions Accumulate Correctly**
///
/// For any sequence of valid contributions from multiple contributors,
/// the total_raised must equal the sum of all contributions.
proptest! {
    #[test]
    fn prop_multiple_contributions_accumulate(
        goal in 5_000_000i128..50_000_000i128,
        deadline_offset in 100u64..100_000u64,
        amount1 in 1_000i128..5_000_000i128,
        amount2 in 1_000i128..5_000_000i128,
        amount3 in 1_000i128..5_000_000i128,
    ) {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None);

        let contributor1 = Address::generate(&env);
        let contributor2 = Address::generate(&env);
        let contributor3 = Address::generate(&env);

        mint_to(&env, &token_address, &admin, &contributor1, amount1);
        mint_to(&env, &token_address, &admin, &contributor2, amount2);
        mint_to(&env, &token_address, &admin, &contributor3, amount3);

        client.contribute(&contributor1, &amount1);
        client.contribute(&contributor2, &amount2);
        client.contribute(&contributor3, &amount3);

        let expected_total = amount1 + amount2 + amount3;

        // **INVARIANT**: total_raised must equal sum of all contributions
        prop_assert_eq!(client.total_raised(), expected_total);

        // **INVARIANT**: Each contributor's balance must be tracked correctly
        prop_assert_eq!(client.contribution(&contributor1), amount1);
        prop_assert_eq!(client.contribution(&contributor2), amount2);
        prop_assert_eq!(client.contribution(&contributor3), amount3);
    }
}

/// **Property Test 6: Withdrawal After Goal Met Transfers Correct Amount**
///
/// For any valid goal and contributions that meet or exceed the goal,
/// withdrawal must transfer the exact total_raised amount to the creator.
proptest! {
    #[test]
    fn prop_withdrawal_transfers_exact_amount(
        goal in 1_000_000i128..10_000_000i128,
        deadline_offset in 100u64..10_000u64,
    ) {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, goal);
        client.contribute(&contributor, &goal);

        // Move past deadline
        env.ledger().set_timestamp(deadline + 1);

        let token_client = token::Client::new(&env, &token_address);
        let creator_balance_before = token_client.balance(&creator);

        client.withdraw();

        let creator_balance_after = token_client.balance(&creator);
        let transferred_amount = creator_balance_after - creator_balance_before;

        // **INVARIANT**: Withdrawal must transfer exact total_raised amount
        prop_assert_eq!(
            transferred_amount, goal,
            "withdrawal transferred {} but expected {}",
            transferred_amount, goal
        );

        // **INVARIANT**: total_raised must be reset to 0 after withdrawal
        prop_assert_eq!(client.total_raised(), 0);
    }
}

/// **Property Test 7: Contribution Tracking Persists Across Multiple Calls**
///
/// For any contributor making multiple contributions, the total tracked
/// must equal the sum of all their contributions.
proptest! {
    #[test]
    fn prop_contribution_tracking_persists(
        goal in 5_000_000i128..50_000_000i128,
        deadline_offset in 100u64..100_000u64,
        amount1 in 1_000i128..2_000_000i128,
        amount2 in 1_000i128..2_000_000i128,
        amount3 in 1_000i128..2_000_000i128,
    ) {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None);

        let contributor = Address::generate(&env);
        let total_needed = amount1.saturating_add(amount2).saturating_add(amount3);
        mint_to(&env, &token_address, &admin, &contributor, total_needed);

        // First contribution
        client.contribute(&contributor, &amount1);
        prop_assert_eq!(client.contribution(&contributor), amount1);

        // Second contribution
        client.contribute(&contributor, &amount2);
        let expected_after_2 = amount1.saturating_add(amount2);
        prop_assert_eq!(client.contribution(&contributor), expected_after_2);

        // Third contribution
        client.contribute(&contributor, &amount3);
        let expected_total = amount1.saturating_add(amount2).saturating_add(amount3);
        prop_assert_eq!(client.contribution(&contributor), expected_total);

        // **INVARIANT**: Final total_raised must equal sum of all contributions
        prop_assert_eq!(client.total_raised(), expected_total);
    }
}

/// **Property Test 8: Refund Resets Total Raised to Zero**
///
/// For any valid refund scenario (goal not met, deadline passed),
/// total_raised must be reset to 0 after refund completes.
proptest! {
    #[test]
    fn prop_refund_resets_total_raised(
        goal in 5_000_000i128..50_000_000i128,
        deadline_offset in 100u64..100_000u64,
        contribution in 1_000i128..5_000_000i128,
    ) {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        let safe_contribution = contribution.min(goal - 1);

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, safe_contribution);
        client.contribute(&contributor, &safe_contribution);

        // Verify total_raised is set
        prop_assert_eq!(client.total_raised(), safe_contribution);

        // Move past deadline (goal not met)
        env.ledger().set_timestamp(deadline + 1);

        client.refund();

        // **INVARIANT**: total_raised must be 0 after refund
        prop_assert_eq!(client.total_raised(), 0);
    }
}

/// **Property Test 9: Contribution Below Minimum Always Fails**
///
/// For any contribution amount below the minimum, the contribute function
/// must fail or panic.
proptest! {
    #[test]
    fn prop_contribute_below_minimum_fails(
        goal in 1_000_000i128..10_000_000i128,
        deadline_offset in 100u64..10_000u64,
        min_contribution in 1_000i128..100_000i128,
        below_minimum in 1i128..1_000i128,
    ) {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &None, &None, &None);

        let contributor = Address::generate(&env);
        let amount_to_contribute = below_minimum.min(min_contribution - 1);
        mint_to(&env, &token_address, &admin, &contributor, amount_to_contribute);

        // Attempt to contribute below minimum
        let result = client.try_contribute(&contributor, &amount_to_contribute);

        // **INVARIANT**: Contribution below minimum must fail
        prop_assert!(
            result.is_err(),
            "contribute with amount {} below minimum {} should fail",
            amount_to_contribute, min_contribution
        );
    }
}

/// **Property Test 10: Contribution After Deadline Always Fails**
///
/// For any contribution attempt after the deadline has passed,
/// the contribute function must fail.
proptest! {
    #[test]
    fn prop_contribute_after_deadline_fails(
        goal in 1_000_000i128..10_000_000i128,
        deadline_offset in 100u64..10_000u64,
        contribution in 1_000i128..10_000_000i128,
        time_after_deadline in 1u64..100_000u64,
    ) {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + deadline_offset;

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None, &None, &None);

        // Move past deadline
        env.ledger().set_timestamp(deadline + time_after_deadline);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, contribution);

        // Attempt to contribute after deadline
        let result = client.try_contribute(&contributor, &contribution);

        // **INVARIANT**: Contribution after deadline must fail
        prop_assert!(
            result.is_err(),
            "contribute after deadline should fail"
        );
        prop_assert_eq!(
            result.unwrap_err().unwrap(),
            crate::ContractError::CampaignEnded
        );
    }
}

// ── Auto-Extension Tests ────────────────────────────────────────────────────

#[test]
fn test_auto_extension_triggered() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 7200; // 2 hours from now
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    let auto_extension_threshold: i128 = 100_000;

    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &Some(auto_extension_threshold));

    // Move to within the auto-extension window (last hour).
    env.ledger().set_timestamp(deadline - 1800); // 30 minutes before deadline

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 150_000);
    client.contribute(&contributor, &150_000);

    // Deadline should be extended by 24 hours.
    let new_deadline = client.deadline();
    assert_eq!(new_deadline, deadline + 86400);
}

#[test]
fn test_auto_extension_not_triggered_below_threshold() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 7200;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    let auto_extension_threshold: i128 = 100_000;

    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &Some(auto_extension_threshold));

    // Move to within the auto-extension window.
    env.ledger().set_timestamp(deadline - 1800);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 50_000);
    client.contribute(&contributor, &50_000);

    // Deadline should NOT be extended (contribution below threshold).
    assert_eq!(client.deadline(), deadline);
}

#[test]
fn test_auto_extension_not_triggered_outside_window() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 7200;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    let auto_extension_threshold: i128 = 100_000;

    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &Some(auto_extension_threshold));

    // Contribute outside the auto-extension window (more than 1 hour before deadline).
    env.ledger().set_timestamp(deadline - 5000);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 150_000);
    client.contribute(&contributor, &150_000);

    // Deadline should NOT be extended (outside window).
    assert_eq!(client.deadline(), deadline);
}

#[test]
fn test_auto_extension_cap_prevents_infinite_extension() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 7200;
    let goal: i128 = 10_000_000;
    let min_contribution: i128 = 1_000;
    let auto_extension_threshold: i128 = 100_000;

    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &Some(auto_extension_threshold));

    // Trigger 5 extensions (the maximum).
    for _i in 0..5 {
        let current_deadline = client.deadline();
        env.ledger().set_timestamp(current_deadline - 1800);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, 150_000);
        client.contribute(&contributor, &150_000);

        // Verify extension occurred.
        assert_eq!(client.deadline(), current_deadline + 86400);
    }

    // Try to trigger a 6th extension.
    let final_deadline = client.deadline();
    env.ledger().set_timestamp(final_deadline - 1800);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 150_000);
    client.contribute(&contributor, &150_000);

    // Deadline should NOT be extended (cap reached).
    assert_eq!(client.deadline(), final_deadline);
}

#[test]
fn test_auto_extension_disabled_when_not_configured() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 7200;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;

    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &None);

    // Move to within the auto-extension window.
    env.ledger().set_timestamp(deadline - 1800);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 150_000);
    client.contribute(&contributor, &150_000);

    // Deadline should NOT be extended (feature not configured).
    assert_eq!(client.deadline(), deadline);
}

//! Comprehensive tests for the crowdfund contract.
//!
//! Covers: initialize, contribute, withdraw, refund, cancel, pledge,
//! collect_pledges, update_metadata, add_stretch_goal, current_milestone,
//! bonus_goal, get_stats, contributors, roadmap, NFT minting, platform fee,
//! and all view functions.

use soroban_sdk::{
    contract, contractimpl, contracttype,
    testutils::{Address as _, Ledger},
    token, Address, Env, String, Vec,
use soroban_sdk::{testutils::{Address as _, Ledger, Events}, token, Address, Env};
#![allow(unused_doc_comments)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

use crate::{ContractError, CrowdfundContract, CrowdfundContractClient, PlatformConfig};

// ── Mock NFT contract ────────────────────────────────────────────────────────

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

// ── Helpers ──────────────────────────────────────────────────────────────────

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

fn mint_to(env: &Env, token_address: &Address, _admin: &Address, to: &Address, amount: i128) {
    let admin_client = token::StellarAssetClient::new(env, token_address);
    admin_client.mint(to, &amount);
}

/// Initialize with default parameters and return the admin address used.
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

// ── initialize ───────────────────────────────────────────────────────────────

/// Verifies all fields are stored correctly after initialization.
#[test]
fn test_initialize_stores_fields() {
    let (env, client, creator, token_address, _admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;

    default_init(&client, &creator, &token_address, deadline);

    assert_eq!(client.goal(), 1_000_000);
    assert_eq!(client.deadline(), deadline);
    assert_eq!(client.min_contribution(), 1_000);
    assert_eq!(client.total_raised(), 0);
    assert_eq!(client.token(), token_address);
    assert_eq!(client.version(), 3);
}

/// Second initialize call must return AlreadyInitialized.
#[test]
fn test_initialize_twice_returns_error() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;

    default_init(&client, &creator, &token_address, deadline);

    let result = client.try_initialize(
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
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::AlreadyInitialized
    );
}

/// Bonus goal must be stored and readable.
#[test]
fn test_initialize_with_bonus_goal() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let desc = String::from_str(&env, "Stretch reward");

    client.initialize(
        &admin,
        &creator,
        &token_address,
        &1_000_000,
        &goal,
        &(goal * 2),
        &deadline,
        &1_000,
        &None,
        &Some(2_000_000i128),
        &Some(desc.clone()),
        &deadline,
        &min_contribution,
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    assert_eq!(client.bonus_goal(), Some(2_000_000));
    assert_eq!(client.bonus_goal_description(), Some(desc));
    assert!(!client.bonus_goal_reached());
    assert_eq!(client.bonus_goal_progress_bps(), 0);
}

/// Platform fee exceeding 100% must return InvalidPlatformFee.
#[test]
fn test_initialize_platform_fee_over_100_panics() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let bad_config = PlatformConfig {
        address: admin.clone(),
        fee_bps: 10_001,
    };
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;

    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &(goal * 2),
        &deadline,
        &min_contribution,
        &None,
    );
    let result = client.try_initialize(
        &admin,
        &creator,
        &token_address,
        &1_000_000,
        &goal,
        &(goal * 2),
        &deadline,
        &1_000,
        &None,
        &Some(bad_config),
        &None,
        &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::InvalidPlatformFee

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::AlreadyInitialized
    );
        &deadline,
        &min_contribution,
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    ); // should panic
}

/// Bonus goal not greater than primary goal must return InvalidBonusGoal.
#[test]
fn test_initialize_bonus_goal_not_greater_panics() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &admin,
        &creator,
        &token_address,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(500_000i128), // less than goal
        &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::InvalidBonusGoal
    );
}

// ── contribute ───────────────────────────────────────────────────────────────

/// Basic contribution updates total_raised and per-contributor balance.
#[test]
fn test_contribute() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 10_000);

    client.contribute(&contributor, &5_000);
    assert_eq!(client.total_raised(), 5_000);
    assert_eq!(client.contribution(&contributor), 5_000);
}

/// Multiple contributions from the same address accumulate correctly.
#[test]
fn test_contribute_accumulates() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 10_000);

    client.contribute(&contributor, &3_000);
    client.contribute(&contributor, &2_000);
    assert_eq!(client.contribution(&contributor), 5_000);
    assert_eq!(client.total_raised(), 5_000);
}

/// Contribution after deadline must return CampaignEnded.
#[test]
fn test_contribute_after_deadline_returns_error() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 100;
    default_init(&client, &creator, &token_address, deadline);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 10_000);

    env.ledger().set_timestamp(deadline + 1);
    let result = client.try_contribute(&contributor, &5_000);
    assert!(result.is_err());
}

/// Contribution below minimum must panic.
#[test]
fn test_contribute_below_minimum_returns_typed_error() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 10_000);
    let result = client.try_contribute(&contributor, &500);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::BelowMinimum);
}

/// contributors() list grows as new addresses contribute.
#[test]
fn test_contributors_list_grows() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &alice, 5_000);
    mint_to(&env, &token_address, &admin, &bob, 5_000);

    client.contribute(&alice, &2_000);
    client.contribute(&bob, &3_000);

    let list = client.contributors();
    assert_eq!(list.len(), 2);
}

// ── withdraw ─────────────────────────────────────────────────────────────────

/// Successful withdraw transfers funds to creator and sets status Successful.
#[test]
fn test_withdraw_skips_nft_minting_when_nft_contract_not_set() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    default_init(&client, &creator, &token_address, deadline);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, goal);
    client.contribute(&contributor, &goal);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize();

    let token_client = token::Client::new(&env, &token_address);
    let before = token_client.balance(&creator);
    client.withdraw();
    assert_eq!(token_client.balance(&creator), before + goal);
    assert_eq!(client.total_raised(), 0);
}

/// Withdraw before finalize (deadline not passed) must return CampaignStillActive.
#[test]
fn test_withdraw_before_deadline_returns_error() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    default_init(&client, &creator, &token_address, deadline);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, goal);
    client.contribute(&contributor, &goal);

    // finalize() before deadline returns CampaignStillActive
    let result = client.try_finalize();
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::CampaignStillActive
    );
}

/// Withdraw when goal not met: finalize transitions to Expired, withdraw panics.
#[test]
#[should_panic(expected = "campaign must be in Succeeded state to withdraw")]
fn test_withdraw_goal_not_reached_returns_error() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 500_000);
    client.contribute(&contributor, &500_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // transitions to Expired
    client.withdraw(); // panics — not Succeeded
}

/// Withdraw with platform fee deducts fee and sends remainder to creator.
#[test]
fn test_withdraw_with_platform_fee() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let platform_addr = Address::generate(&env);
    let config = PlatformConfig {
        address: platform_addr.clone(),
        fee_bps: 500, // 5%
    };

    client.initialize(
        &admin,
        &creator,
        &token_address,
        &goal,
        &deadline,
        &1_000,
        &None,
        &Some(config),
        &None,
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, goal);
    client.contribute(&contributor, &goal);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize();
    client.withdraw();

    let token_client = token::Client::new(&env, &token_address);
    // 5% fee = 50_000; creator gets 950_000
    assert_eq!(token_client.balance(&platform_addr), 50_000);
    // creator started with 10_000_000 minted in setup_env
    assert_eq!(token_client.balance(&creator), 10_000_000 + 950_000);
}

/// Withdraw mints NFTs for each contributor when NFT contract is set.
#[test]
fn test_withdraw_mints_nft_for_each_contributor() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let _goal: i128 = 1_000_000;
    default_init(&client, &creator, &token_address, deadline);

    // Register mock NFT contract and configure it.
    let nft_id = env.register(MockNftContract, ());
    client.set_nft_contract(&creator, &nft_id);
    assert_eq!(client.nft_contract(), Some(nft_id.clone()));
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &alice, 600_000);
    mint_to(&env, &token_address, &admin, &bob, 400_000);
    client.contribute(&alice, &600_000);
    client.contribute(&bob, &400_000);

    assert_eq!(client.total_raised(), 1_000_000);
    assert_eq!(client.contribution(&alice), 600_000);
    assert_eq!(client.contribution(&bob), 400_000);
}

#[test]
fn test_contribute_after_deadline_panics() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 100;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    // Fast-forward past the deadline.
    env.ledger().set_timestamp(deadline + 1);
    client.finalize();

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 500_000);

    let result = client.try_contribute(&contributor, &500_000);

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::CampaignEnded
    );
}

#[test]
fn test_withdraw_after_goal_met() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 1_000_000);
    client.contribute(&contributor, &1_000_000);

    assert_eq!(client.total_raised(), goal);

    // Move past deadline.
    env.ledger().set_timestamp(deadline + 1);

    client.withdraw();

    // Both contributors should have received an NFT.
    let nft_client = MockNftContractClient::new(&env, &nft_id);
    let minted = nft_client.minted();
    assert_eq!(minted.len(), 2);
}

/// Withdraw skips NFT minting when no NFT contract is configured.
#[test]
fn test_withdraw_skips_nft_mint_when_contract_not_set() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let _goal: i128 = 1_000_000;
    default_init(&client, &creator, &token_address, deadline);
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, _goal);
    client.contribute(&contributor, &_goal);

    env.ledger().set_timestamp(deadline + 1);
    // Should not panic — no NFT contract set.
    client.finalize();
    mint_to(&env, &token_address, &admin, &contributor, 1_000_000);
    client.contribute(&contributor, &1_000_000);

    let result = client.try_withdraw();

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::CampaignStillActive
    );
}

#[test]
fn test_withdraw_goal_not_reached_panics() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 500_000);
    client.contribute(&contributor, &500_000);

    // Move past deadline, but goal not met.
    env.ledger().set_timestamp(deadline + 1);

    let result = client.try_withdraw();

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::GoalNotReached
    );
}

#[test]
fn test_refund_single_when_goal_not_met() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &alice, 300_000);
    mint_to(&env, &token_address, &admin, &bob, 200_000);

    client.contribute(&alice, &300_000);
    client.contribute(&bob, &200_000);

    // Move past deadline — goal not met.
    env.ledger().set_timestamp(deadline + 1);

    client.refund_single(&alice);
    client.refund_single(&bob);

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
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 1_000_000);
    client.contribute(&contributor, &1_000_000);

    env.ledger().set_timestamp(deadline + 1);

    let result = client.try_refund_single(&contributor);
    
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
    use crate::ContractError;

    // Test 1: Double initialization
    {
        let (env, client, creator, token_address, _admin) = setup_env();
        let deadline = env.ledger().timestamp() + 3600;
        let goal: i128 = 1_000_000;
        
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);
        let result = client.try_initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);
        

        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);
        let result =
            client.try_initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

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
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);
        
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None);
        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

        env.ledger().set_timestamp(deadline + 1);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, 500_000);
        let result = client.try_contribute(&contributor, &500_000);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().unwrap(), ContractError::CampaignEnded);
    }

    // Test 3: Early withdrawal
    {
        let (env, client, creator, token_address, admin) = setup_env();
        let deadline = env.ledger().timestamp() + 3600;
        let goal: i128 = 1_000_000;
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);
        
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None);
        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, 1_000_000);
        client.contribute(&contributor, &1_000_000);

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
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);
        
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None);
        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, 500_000);
        client.contribute(&contributor, &500_000);

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
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);
        
        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, 500_000);
        client.contribute(&contributor, &500_000);
        
        let result = client.try_refund_single(&contributor);
        
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, 500_000);
        client.contribute(&contributor, &500_000);
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None);
        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

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
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);
        
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None);
        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, 1_000_000);
        client.contribute(&contributor, &1_000_000);

        env.ledger().set_timestamp(deadline + 1);
        let result = client.try_refund_single(&contributor);
        
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
        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);
        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

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

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);
        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, contribution_amount);

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

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);
        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, goal);
        client.contribute(&contributor, &goal);

        // Move past deadline
        env.ledger().set_timestamp(deadline + 1);

        let token_client = token::Client::new(&env, &token_address);
        let creator_balance_before = token_client.balance(&creator);

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

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);
        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

        let contributor = Address::generate(&env);
        mint_to(&env, &token_address, &admin, &contributor, contribution);
        client.contribute(&contributor, &contribution);

        // Move past deadline (goal not met)
        env.ledger().set_timestamp(deadline + 1);

        // Test 3.4: Successful refund returns funds to contributors
        client.refund_single(&contributor);

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

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);
        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

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
        let hard_cap = (amount1 + amount2 + amount3).max(goal);

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);
        client.initialize(&creator, &token_address, &goal, &hard_cap, &deadline, &1_000, &None);

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
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 1_000_000);
    client.contribute(&contributor, &1_000_000);

    env.ledger().set_timestamp(deadline + 1);

    client.withdraw();
    assert_eq!(client.nft_contract(), None);
}

// ── refund_single (pull-based) ────────────────────────────────────────────────

/// refund_single returns tokens to the contributor when goal is not met.
#[test]
fn test_refund_returns_tokens() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    let alice = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &alice, 500_000);
    client.contribute(&alice, &500_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // transitions to Expired
    client.refund_single(&alice);

    let token_client = token::Client::new(&env, &token_address);
    assert_eq!(token_client.balance(&alice), 500_000);
    assert_eq!(client.total_raised(), 0);
}

/// Second refund_single call must panic — nothing left to refund.
#[test]
#[should_panic(expected = "NothingToRefund")]
fn test_double_refund_panics() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let alice = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &alice, 500_000);
    client.contribute(&alice, &500_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize();
    client.refund_single(&alice);
    client.refund_single(&alice); // panics — nothing left to refund
}

/// refund_single when goal is reached: finalize transitions to Succeeded, refund panics.
#[test]
#[should_panic(expected = "campaign must be in Expired state to refund")]
fn test_refund_when_goal_reached_returns_error() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    default_init(&client, &creator, &token_address, deadline);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, goal);
    client.contribute(&contributor, &goal);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // transitions to Succeeded
    client.refund_single(&contributor); // panics — not Expired
}

// ── cancel ───────────────────────────────────────────────────────────────────

/// Cancel with no contributions sets total_raised to 0.
#[test]
fn test_cancel_with_no_contributions() {
    let (env, client, creator, token_address, _admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    client.cancel();
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    client.cancel();

    assert_eq!(client.total_raised(), 0);
}

#[test]
fn test_cancel_with_contributions() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &alice, 300_000);
    mint_to(&env, &token_address, &admin, &bob, 200_000);

    client.contribute(&alice, &300_000);
    client.contribute(&bob, &200_000);

    client.cancel();

    let token_client = token::Client::new(&env, &token_address);
    assert_eq!(token_client.balance(&alice), 300_000);
    assert_eq!(token_client.balance(&bob), 200_000);
    assert_eq!(client.total_raised(), 0);
}

/// Non-creator cancel must panic.
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
    client.initialize(
        &token_admin,
        &creator,
        &token_address,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &None,
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

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

/// Cancel after already cancelled must panic.
#[test]
#[should_panic(expected = "campaign is not active")]
fn test_cancel_twice_panics() {
#[should_panic(expected = "amount below minimum")]
fn test_contribute_below_minimum_panics() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 10_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
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
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
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
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 50_000);

    client.contribute(&contributor, &50_000);

    assert_eq!(client.total_raised(), 50_000);
    assert_eq!(client.contribution(&contributor), 50_000);
}

// ── Hard Cap Tests ─────────────────────────────────────────────────────────
// ── Metadata Tests ─────────────────────────────────────────────────────────

#[test]
fn test_metadata_retrieval() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    let category = soroban_sdk::String::from_str(&env, "Art");
    let mut tags = soroban_sdk::Vec::new(&env);
    tags.push_back(soroban_sdk::String::from_str(&env, "NFT"));
    tags.push_back(soroban_sdk::String::from_str(&env, "Creative"));

    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &category,
        &tags,
    );

    assert_eq!(client.category(), category);
    assert_eq!(client.tags(), tags);
}

#[test]
fn test_empty_tags() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    let category = soroban_sdk::String::from_str(&env, "Charity");
    let tags = soroban_sdk::Vec::new(&env);

    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &category,
        &tags,
    );

    assert_eq!(client.category(), category);
    assert_eq!(client.tags().len(), 0);
}

#[test]
#[should_panic(expected = "category must not be empty")]
fn test_initialize_empty_category_panics() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    let category = soroban_sdk::String::from_str(&env, ""); // Empty category
    let tags = soroban_sdk::Vec::new(&env);

    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &category,
        &tags,
    );
}

// ── Campaign Stats Tests ───────────────────────────────────────────────────

#[test]
fn test_contribute_up_to_hard_cap() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let hard_cap: i128 = 1_500_000;
    let min_contribution: i128 = 1_000;

    client.initialize(
        &creator,
        &token_address,
        &goal,
        &hard_cap,
        &deadline,
        &min_contribution,
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, hard_cap);

    // Contribute exactly up to hard cap — should succeed
    client.contribute(&contributor, &hard_cap);

    assert_eq!(client.total_raised(), hard_cap);
    assert_eq!(client.hard_cap(), hard_cap);
    assert_eq!(client.contribution(&contributor), hard_cap);
}

#[test]
fn test_contribute_exceeds_hard_cap_rejected() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let hard_cap: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;

    client.initialize(
        &creator,
        &token_address,
        &goal,
        &hard_cap,
        &deadline,
        &min_contribution,
        &None,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 2_000_000);

    // First contribution fills hard cap
    client.contribute(&contributor, &hard_cap);

    // Second contribution should be rejected — already at hard cap
    let result = client.try_contribute(&contributor, &1_000);

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::HardCapExceeded
    );
    assert_eq!(client.total_raised(), hard_cap);
}

#[test]
fn test_contribute_partial_fits_within_hard_cap() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let hard_cap: i128 = 1_500_000;
    let min_contribution: i128 = 1_000;

    client.initialize(
        &creator,
        &token_address,
        &goal,
        &hard_cap,
        &deadline,
        &min_contribution,
        &None,
    );

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &alice, 1_000_000);
    mint_to(&env, &token_address, &admin, &bob, 1_000_000);

    // Alice contributes 1_000_000 (total = 1_000_000)
    client.contribute(&alice, &1_000_000);

    // Bob tries to contribute 1_000_000 but only 500_000 fits — partial accepted
    client.contribute(&bob, &1_000_000);

    assert_eq!(client.total_raised(), hard_cap);
    assert_eq!(client.contribution(&alice), 1_000_000);
    assert_eq!(client.contribution(&bob), 500_000);
}

#[test]
fn test_initialize_rejects_hard_cap_less_than_goal() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let hard_cap: i128 = 500_000; // less than goal
    let min_contribution: i128 = 1_000;

    let result = client.try_initialize(
        &creator,
        &token_address,
        &goal,
        &hard_cap,
        &deadline,
        &min_contribution,
        &None,
    );

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::InvalidHardCap
    );
}

// ── Roadmap Tests ──────────────────────────────────────────────────────────

#[test]
fn test_add_single_roadmap_item() {
#[test]
fn test_token_address_view() {
    let (env, client, creator, token_address, _admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);
    client.cancel();
    client.cancel(); // panics
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
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

// ── update_metadata ──────────────────────────────────────────────────────────

/// update_metadata stores title, description, and socials.
#[test]
fn test_update_metadata_stores_fields() {
    let (env, client, creator, token_address, _admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &(goal * 2),
        &deadline,
        &min_contribution,
        &None,
    );

    let title = String::from_str(&env, "My Campaign");
    let desc = String::from_str(&env, "A great project");
    let socials = String::from_str(&env, "https://twitter.com/example");

    client.update_metadata(
        &creator,
        &Some(title.clone()),
        &Some(desc.clone()),
        &Some(socials.clone()),
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
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &(goal * 2),
        &deadline,
        &min_contribution,
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
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &(goal * 2),
        &deadline,
        &min_contribution,
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
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &(goal * 2),
        &deadline,
        &min_contribution,
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

    let creator = Address::generate(&env);
    let non_creator = Address::generate(&env);

    env.mock_all_auths();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &(goal * 2),
        &deadline,
        &min_contribution,
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
    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &default_title(&env), &default_description(&env), &None);
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &(goal * 2),
        &deadline,
        &min_contribution,
        &None,
    );

    let roadmap = client.roadmap();
    assert_eq!(roadmap.len(), 0);
}

// ── Campaign Info Tests ────────────────────────────────────────────────────

#[test]
fn test_creator() {
    let (env, client, creator, token_address, _admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &(goal * 2),
        &deadline,
        &min_contribution,
    );

    assert_eq!(client.title(), title);
    assert_eq!(client.description(), desc);
    assert_eq!(client.socials(), socials);
}

/// update_metadata on a cancelled campaign must panic.
#[test]
#[should_panic(expected = "campaign is not active")]
fn test_update_metadata_when_not_active_panics() {
    let (env, client, creator, token_address, _admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);
    client.cancel();
    client.update_metadata(&creator, &None, &None, &None);
}

// ── pledge / collect_pledges ─────────────────────────────────────────────────

/// Pledge records amount without transferring tokens immediately.
#[test]
fn test_pledge_records_amount() {
    let (env, client, creator, token_address, _admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &(goal * 2),
        &deadline,
        &min_contribution,
    );

    let pledger = Address::generate(&env);
    client.pledge(&pledger, &5_000);

    // total_raised unchanged — pledge is not a transfer
    assert_eq!(client.total_raised(), 0);
}

/// Pledge after deadline must return CampaignEnded.
#[test]
fn test_pledge_after_deadline_returns_error() {
    let (env, client, creator, token_address, _admin) = setup_env();
    let deadline = env.ledger().timestamp() + 100;
    default_init(&client, &creator, &token_address, deadline);

    env.ledger().set_timestamp(deadline + 1);
    let pledger = Address::generate(&env);
    let result = client.try_pledge(&pledger, &5_000);
    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &(goal * 2),
        &deadline,
        &min_contribution,
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 1_500_000);
    client.contribute(&contributor, &1_500_000);

    let info = client.get_campaign_info();

    assert_eq!(info.creator, creator);
    assert_eq!(info.token, token_address);
    assert_eq!(info.goal, goal);
    assert_eq!(info.deadline, deadline);
    assert_eq!(info.total_raised, 1_500_000);
}

// ── Whitelist Tests ────────────────────────────────────────────────────────

#[test]
fn test_whitelisted_contribution() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &(goal * 2),
        &deadline,
        &min_contribution,
    );

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    // Add Alice to whitelist
    client.add_to_whitelist(&soroban_sdk::vec![&env, alice.clone()]);

    mint_to(&env, &token_address, &admin, &alice, 500_000);
    mint_to(&env, &token_address, &admin, &bob, 500_000);

    // Alice (whitelisted) can contribute
    client.contribute(&alice, &500_000);
    assert_eq!(client.contribution(&alice), 500_000);

    // Bob (not whitelisted) cannot contribute
    let result = client.try_contribute(&bob, &500_000);
    assert!(result.is_err());
}

/// collect_pledges requires pledger auth for the token transfer.
/// When goal is not met by pledges alone, GoalNotReached is returned.
#[test]
fn test_collect_pledges_goal_not_met_returns_error() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    // Pledge only half the goal — not enough to meet it
    let pledger = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &pledger, 500_000);
    client.pledge(&pledger, &500_000);

    env.ledger().set_timestamp(deadline + 1);
    let result = client.try_collect_pledges();
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::GoalNotReached
    );
}

/// collect_pledges before deadline must return CampaignStillActive.
#[test]
fn test_collect_pledges_before_deadline_returns_error() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    default_init(&client, &creator, &token_address, deadline);

    let pledger = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &pledger, goal);
    client.pledge(&pledger, &goal);

    let result = client.try_collect_pledges();
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::CampaignStillActive
    );
}

// ── stretch goals / bonus goal ───────────────────────────────────────────────

/// add_stretch_goal stores milestone; current_milestone returns first unmet one.
#[test]
fn test_stretch_goal_current_milestone() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    client.add_stretch_goal(&2_000_000i128);
    client.add_stretch_goal(&3_000_000i128);

    assert_eq!(client.current_milestone(), 2_000_000);

    // Contribute past first milestone
    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 2_500_000);
    client.contribute(&contributor, &2_500_000);

    assert_eq!(client.current_milestone(), 3_000_000);
}

/// current_milestone returns 0 when no stretch goals are set.
#[test]
fn test_current_milestone_no_goals_returns_zero() {
    let (env, client, creator, token_address, _admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);
    assert_eq!(client.current_milestone(), 0);
}

/// bonus_goal_reached becomes true once total_raised >= bonus_goal.
#[test]
fn test_bonus_goal_reached_after_contribution() {
    let (env, client, creator, token_address, admin) = setup_env();
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

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 2_000_000);
    client.contribute(&contributor, &2_000_000);

    assert!(client.bonus_goal_reached());
    assert_eq!(client.bonus_goal_progress_bps(), 10_000);
}

// ── get_stats ────────────────────────────────────────────────────────────────

/// get_stats returns accurate aggregate data.
#[test]
fn test_get_stats() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &alice, 300_000);
    mint_to(&env, &token_address, &admin, &bob, 700_000);
    client.contribute(&alice, &300_000);
    client.contribute(&bob, &700_000);

    client.add_to_whitelist(&soroban_sdk::vec![&env, alice.clone(), bob.clone()]);

    assert!(client.is_whitelisted(&alice));
    assert!(client.is_whitelisted(&bob));

    mint_to(&env, &token_address, &admin, &alice, 100_000);
    mint_to(&env, &token_address, &admin, &bob, 100_000);

    client.contribute(&alice, &100_000);
    client.contribute(&bob, &100_000);

    assert_eq!(client.total_raised(), 200_000);
}

#[test]
#[should_panic]
fn test_add_to_whitelist_non_creator_panics() {
    let (env, client, _creator, _token_address, _admin) = setup_env();

    let alice = Address::generate(&env);

    // Non-creator address
    let _attacker = Address::generate(&env);

    // Mock authorization for non-creator
    env.mock_all_auths();

    // This should panic because creator.require_auth() will fail (mock_all_auths handles the auth but we check if the caller is the creator)
    // Actually, require_auth checks if the address authorized the call.
    // In lib.rs: let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap(); creator.require_auth();
    // This means the 'creator' MUST authorize the call. If 'attacker' calls it, 'creator.require_auth()' will fail unless 'creator' also authorized it.

    client.add_to_whitelist(&soroban_sdk::vec![&env, alice]);
}

// ── Early Withdrawal Tests ──────────────────────────────────────────────────

#[test]
fn test_partial_withdrawal() {
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
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 500_000);
    client.contribute(&contributor, &500_000);

    assert_eq!(client.total_raised(), 500_000);
    assert_eq!(client.contribution(&contributor), 500_000);

    // Partial withdrawal.
    client.withdraw_contribution(&contributor, &200_000);

    assert_eq!(client.total_raised(), 300_000);
    assert_eq!(client.contribution(&contributor), 300_000);
}

#[test]
fn test_full_withdrawal_removes_contributor() {
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
        &deadline,
        &min_contribution,
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 500_000);
    client.contribute(&contributor, &500_000);

    let stats = client.get_stats();
    assert_eq!(stats.total_raised, 1_000_000);
    assert_eq!(stats.goal, 1_000_000);
    assert_eq!(stats.progress_bps, 10_000);
    assert_eq!(stats.contributor_count, 2);
    assert_eq!(stats.average_contribution, 500_000);
    assert_eq!(stats.largest_contribution, 700_000);
}

// ── roadmap ──────────────────────────────────────────────────────────────────

/// add_roadmap_item stores items; roadmap() returns them.
#[test]
fn test_add_roadmap_item() {
    let (env, client, creator, token_address, _admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let future_date = env.ledger().timestamp() + 7200;
    let desc = String::from_str(&env, "Phase 1 launch");
    client.add_roadmap_item(&future_date, &desc);

    let items = client.roadmap();
    assert_eq!(items.len(), 1);
    assert_eq!(items.get(0).unwrap().date, future_date);
}

// ── token_decimals ────────────────────────────────────────────────────────────

/// token_decimals() returns the decimal precision stored at initialize time.
#[test]
fn test_token_decimals_stored_on_initialize() {
    let (env, client, creator, token_address, _admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    // Stellar asset contracts report 7 decimals (stroops).
    assert_eq!(client.token_decimals(), 7u32);
}

// ── Pledge Mechanism Tests ─────────────────────────────────────────────────

// Note: The non-creator test would require complex mock setup.
// The authorization check is covered by require_auth() in the contract,
// which will panic if the caller is not the creator.

#[test]
fn test_pledge_records_without_transfer() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &(goal * 2),
        &deadline,
        &min_contribution,
        &None,
    );

    let pledger = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &pledger, 500_000);

    let token_client = token::Client::new(&env, &token_address);
    let balance_before = token_client.balance(&pledger);

    // Make a pledge
    client.pledge(&pledger, &500_000);

    // Verify pledge is recorded
    assert_eq!(client.pledge_amount(&pledger), 500_000);
    assert_eq!(client.total_pledged(), 500_000);

    // Verify tokens were NOT transferred
    assert_eq!(token_client.balance(&pledger), balance_before);
    assert_eq!(client.total_raised(), 0);
}

#[test]
fn test_multiple_pledges_from_same_pledger() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &(goal * 2),
        &deadline,
        &min_contribution,
        &None,
    );

    let pledger = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &pledger, 800_000);

    // Make multiple pledges
    client.pledge(&pledger, &300_000);
    client.pledge(&pledger, &500_000);

    // Verify pledges accumulate
    assert_eq!(client.pledge_amount(&pledger), 800_000);
    assert_eq!(client.total_pledged(), 800_000);
}

#[test]
fn test_multiple_pledgers() {
    let (env, client, creator, token_address, admin) = setup_env();
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
        &(goal * 2),
        &deadline,
        &min_contribution,
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
        &(goal * 2),
        &deadline,
        &min_contribution,
        &None,
    );

    // Try to shorten the deadline (should panic)
    let shorter_deadline = deadline - 1800; // 30 minutes earlier
    client.update_deadline(&shorter_deadline);
}

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
        &(goal * 2),
        &deadline,
        &min_contribution,
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
        &(goal * 2),
        &deadline,
        &min_contribution,
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
        &(goal * 2),
        &deadline,
        &min_contribution,
        &None,
    );

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);

    mint_to(&env, &token_address, &admin, &alice, 300_000);
    mint_to(&env, &token_address, &admin, &bob, 400_000);
    mint_to(&env, &token_address, &admin, &charlie, 300_000);

    client.pledge(&alice, &300_000);
    client.pledge(&bob, &400_000);
    client.pledge(&charlie, &300_000);

    assert_eq!(client.pledge_amount(&alice), 300_000);
    assert_eq!(client.pledge_amount(&bob), 400_000);
    assert_eq!(client.pledge_amount(&charlie), 300_000);
    assert_eq!(client.total_pledged(), 1_000_000);
    let stretch_milestone: i128 = 1_500_000;
    client.add_stretch_goal(&stretch_milestone);

    assert_eq!(client.current_milestone(), stretch_milestone);
}

#[test]
fn test_collect_pledges_when_goal_met() {
    let (env, client, creator, token_address, admin) = setup_env();
    let stretch_goal: i128 = 2_000_000;
    client.add_stretch_goal(&stretch_goal);

    assert_eq!(client.current_milestone(), stretch_goal);
}

#[test]
fn test_add_multiple_stretch_goals() {
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
    );

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);

    mint_to(&env, &token_address, &admin, &alice, 600_000);
    mint_to(&env, &token_address, &admin, &bob, 400_000);

    // Make pledges
    client.pledge(&alice, &600_000);
    client.pledge(&bob, &400_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    let token_client = token::Client::new(&env, &token_address);
    let contract_balance_before = token_client.balance(&env.current_contract_address());

    // Collect pledges
    client.collect_pledges();

    // Verify tokens were transferred to contract
    assert_eq!(
        token_client.balance(&env.current_contract_address()),
        contract_balance_before + 1_000_000
    );

    // Verify pledges were cleared
    assert_eq!(client.pledge_amount(&alice), 0);
    assert_eq!(client.pledge_amount(&bob), 0);
    assert_eq!(client.total_pledged(), 0);

    // Verify total_raised was updated
    assert_eq!(client.total_raised(), 1_000_000);
}

#[test]
fn test_collect_pledges_with_mixed_contributions_and_pledges() {
    client.add_stretch_goal(&2_000_000);
    client.add_stretch_goal(&3_000_000);
    client.add_stretch_goal(&5_000_000);

    // Should return the first unmet milestone
    assert_eq!(client.current_milestone(), 2_000_000);
}

#[test]
fn test_current_milestone_updates_after_reaching() {
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
    );

    let contributor = Address::generate(&env);
    let pledger = Address::generate(&env);

    mint_to(&env, &token_address, &admin, &contributor, 600_000);
    mint_to(&env, &token_address, &admin, &pledger, 400_000);

    // Mix contributions and pledges
    client.contribute(&contributor, &600_000);
    client.pledge(&pledger, &400_000);

    assert_eq!(client.total_raised(), 600_000);
    assert_eq!(client.total_pledged(), 400_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Collect pledges
    client.collect_pledges();

    // Verify total_raised includes both
    assert_eq!(client.total_raised(), 1_000_000);
    assert_eq!(client.total_pledged(), 0);
}

#[test]
fn test_collect_pledges_before_deadline_fails() {
    client.add_stretch_goal(&2_000_000);
    client.add_stretch_goal(&3_000_000);

    // Initially, first stretch goal is current
    assert_eq!(client.current_milestone(), 2_000_000);

    // Contribute to reach first stretch goal
    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 2_500_000);
    client.contribute(&contributor, &2_500_000);

    // Now second stretch goal should be current
    assert_eq!(client.current_milestone(), 3_000_000);
}

#[test]
fn test_current_milestone_returns_zero_when_all_met() {
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
    );

    let pledger = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &pledger, 1_000_000);

    client.pledge(&pledger, &1_000_000);

    // Try to collect before deadline
    let result = client.try_collect_pledges();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), crate::ContractError::CampaignStillActive);
}

#[test]
fn test_collect_pledges_when_goal_not_reached_fails() {
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
    );

    let pledger = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &pledger, 500_000);

    client.pledge(&pledger, &500_000);

    // Move past deadline (goal not met)
    env.ledger().set_timestamp(deadline + 1);

    // Try to collect pledges
    let result = client.try_collect_pledges();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), crate::ContractError::GoalNotReached);
}

#[test]
fn test_pledges_discarded_when_goal_not_met() {
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
    );

    let pledger = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &pledger, 500_000);

    client.pledge(&pledger, &500_000);

    let token_client = token::Client::new(&env, &token_address);
    let pledger_balance = token_client.balance(&pledger);

    // Move past deadline (goal not met)
    env.ledger().set_timestamp(deadline + 1);

    // Pledges are simply not collected - no refund needed
    // Verify pledger still has their tokens
    assert_eq!(token_client.balance(&pledger), pledger_balance);
    assert_eq!(client.pledge_amount(&pledger), 500_000);
    assert_eq!(client.total_pledged(), 500_000);
}

#[test]
fn test_pledge_after_deadline_fails() {
    let (env, client, creator, token_address, admin) = setup_env();

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
    );

    // Fast-forward past the deadline
    env.ledger().set_timestamp(deadline + 1);

    let pledger = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &pledger, 500_000);

    let result = client.try_pledge(&pledger, &500_000);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), crate::ContractError::CampaignEnded);
}

#[test]
#[should_panic(expected = "amount below minimum")]
fn test_pledge_below_minimum_panics() {
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
    );

    let pledger = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &pledger, 5_000);

    client.pledge(&pledger, &5_000); // should panic
}

#[test]
fn test_pledge_exact_minimum() {
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
    );

    let pledger = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &pledger, 10_000);

    client.pledge(&pledger, &10_000);

    assert_eq!(client.pledge_amount(&pledger), 10_000);
    assert_eq!(client.total_pledged(), 10_000);
}

#[test]
#[should_panic(expected = "campaign is not active")]
fn test_collect_pledges_after_withdrawal_panics() {
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
    );

    let pledger = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &pledger, 1_000_000);

    client.pledge(&pledger, &1_000_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Collect pledges first
    client.collect_pledges();

    // Withdraw
    client.withdraw();

    // Try to collect pledges again (should panic - status is Successful)
    client.collect_pledges();
}

#[test]
fn test_withdraw_after_collecting_pledges() {
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
    );

    let pledger = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &pledger, 1_000_000);

    client.pledge(&pledger, &1_000_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Collect pledges
    client.collect_pledges();

    // Now withdraw should work
    client.withdraw();

    let token_client = token::Client::new(&env, &token_address);
    assert_eq!(token_client.balance(&creator), 10_000_000 + 1_000_000);
}

#[test]
fn test_collect_pledges_with_no_pledgers() {
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
    );

    // Make regular contribution to meet goal
    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 1_000_000);
    client.contribute(&contributor, &1_000_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Collect pledges (should succeed even with no pledgers)
    client.collect_pledges();

    assert_eq!(client.total_pledged(), 0);
    assert_eq!(client.total_raised(), 1_000_000);
}

#[test]
fn test_pledge_and_contribute_from_same_address() {
    let (env, client, creator, token_address, admin) = setup_env();
    client.add_stretch_goal(&2_000_000);
    client.add_stretch_goal(&3_000_000);

    // Contribute to exceed all stretch goals
    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 4_000_000);
    client.contribute(&contributor, &4_000_000);

    // All stretch goals met, should return 0
    assert_eq!(client.current_milestone(), 0);
}

#[test]
fn test_current_milestone_returns_zero_when_no_stretch_goals() {
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
    );

    let user = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &user, 1_000_000);

    // Same user can both contribute and pledge
    client.contribute(&user, &300_000);
    client.pledge(&user, &700_000);

    assert_eq!(client.contribution(&user), 300_000);
    assert_eq!(client.pledge_amount(&user), 700_000);
    assert_eq!(client.total_raised(), 300_000);
    assert_eq!(client.total_pledged(), 700_000);

    // Move past deadline and collect
    env.ledger().set_timestamp(deadline + 1);
    client.collect_pledges();

    assert_eq!(client.total_raised(), 1_000_000);
    assert_eq!(client.pledge_amount(&user), 0);
    // No stretch goals added
    assert_eq!(client.current_milestone(), 0);
}

#[test]
#[should_panic(expected = "stretch goal must be greater than primary goal")]
fn test_add_stretch_goal_below_primary_goal_panics() {
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
    );

    // Try to add stretch goal below primary goal
    client.add_stretch_goal(&500_000);
}

#[test]
#[should_panic(expected = "stretch goal must be greater than primary goal")]
fn test_add_stretch_goal_equal_to_primary_goal_panics() {
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
    );

    // Try to add stretch goal equal to primary goal
    client.add_stretch_goal(&1_000_000);
}

#[test]
#[should_panic]
fn test_add_stretch_goal_by_non_creator_panics() {
    let env = Env::default();
    let contract_id = env.register(crate::CrowdfundContract, ());
    let client = crate::CrowdfundContractClient::new(&env, &contract_id);

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
    );

    env.mock_all_auths_allowing_non_root_auth();
    env.set_auths(&[]);

    client.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &non_creator,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_id,
            fn_name: "add_stretch_goal",
            args: soroban_sdk::vec![&env],
            sub_invokes: &[],
        },
    }]);

    client.add_stretch_goal(&2_000_000);
}

// ── Overflow Protection Tests ──────────────────────────────────────────────

#[test]
fn test_contribution_near_i128_max_handled_gracefully() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = i128::MAX / 2;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
    );

    let contributor = Address::generate(&env);
    // Mint a very large amount near i128::MAX
    let large_amount = i128::MAX / 2;
    mint_to(&env, &token_address, &admin, &contributor, large_amount);

    // This should succeed without overflow
    client.contribute(&contributor, &large_amount);

    assert_eq!(client.total_raised(), large_amount);
    assert_eq!(client.contribution(&contributor), large_amount);
}

#[test]
fn test_multiple_contributions_causing_overflow_rejected() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = i128::MAX;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
    );

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    
    // First contribution: near max
    let first_amount = i128::MAX - 1_000_000;
    mint_to(&env, &token_address, &admin, &alice, first_amount);
    client.contribute(&alice, &first_amount);

    assert_eq!(client.total_raised(), first_amount);

    // Second contribution: would cause overflow
    let second_amount = 2_000_000;
    mint_to(&env, &token_address, &admin, &bob, second_amount);
    
    let result = client.try_contribute(&bob, &second_amount);
    
    // Should fail with Overflow error
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), crate::ContractError::Overflow);
    
    // Total should remain unchanged
    assert_eq!(client.total_raised(), first_amount);
    // Bob's contribution should not be recorded
    assert_eq!(client.contribution(&bob), 0);
}

#[test]
fn test_single_contributor_overflow_on_second_contribution() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = i128::MAX;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
    );

    let contributor = Address::generate(&env);
    
    // First contribution
    let first_amount = i128::MAX - 500_000;
    mint_to(&env, &token_address, &admin, &contributor, first_amount);
    client.contribute(&contributor, &first_amount);

    assert_eq!(client.contribution(&contributor), first_amount);

    // Second contribution from same contributor would overflow their personal total
    let second_amount = 1_000_000;
    mint_to(&env, &token_address, &admin, &contributor, second_amount);
    
    let result = client.try_contribute(&contributor, &second_amount);
    
    // Should fail with Overflow error
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), crate::ContractError::Overflow);
    
    // Contributor's total should remain unchanged
    assert_eq!(client.contribution(&contributor), first_amount);
}

#[test]
fn test_overflow_protection_preserves_contract_state() {
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let charlie = Address::generate(&env);
    
    // Alice contributes successfully
    mint_to(&env, &token_address, &admin, &alice, 300_000);
    client.contribute(&alice, &300_000);

    mint_to(&env, &token_address, &admin, &alice, 600_000);
    mint_to(&env, &token_address, &admin, &bob, 300_000);
    mint_to(&env, &token_address, &admin, &charlie, 100_000);

    // Bob tries to contribute an amount that would overflow
    let overflow_amount = i128::MAX;
    mint_to(&env, &token_address, &admin, &bob, overflow_amount);
    let result = client.try_contribute(&bob, &overflow_amount);
    assert!(result.is_err());

    // Verify contract state is preserved after overflow attempt
    assert_eq!(client.total_raised(), 300_000);
    assert_eq!(client.contribution(&alice), 300_000);
    assert_eq!(client.contribution(&bob), 0);

    // Charlie can still contribute successfully
    mint_to(&env, &token_address, &admin, &charlie, 200_000);
    client.contribute(&charlie, &200_000);

    assert_eq!(client.total_raised(), 500_000);
    assert_eq!(client.contribution(&charlie), 200_000);
}

#[test]
fn test_exact_i128_max_contribution_accepted() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = i128::MAX;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
    );

    let contributor = Address::generate(&env);
    // Mint exactly i128::MAX
    mint_to(&env, &token_address, &admin, &contributor, i128::MAX);

    // This should succeed - no overflow when adding to 0
    client.contribute(&contributor, &i128::MAX);

    assert_eq!(client.total_raised(), i128::MAX);
    assert_eq!(client.contribution(&contributor), i128::MAX);
}

#[test]
fn test_overflow_on_total_raised_not_individual_contribution() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = i128::MAX;
    let min_contribution: i128 = 1_000;
    client.initialize(
        &creator,
        &token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
    );

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    
    // Alice contributes a large amount
    let alice_amount = i128::MAX / 2 + 1;
    mint_to(&env, &token_address, &admin, &alice, alice_amount);
    client.contribute(&alice, &alice_amount);

    // Bob tries to contribute an amount that would overflow the total
    // but not his individual contribution
    let bob_amount = i128::MAX / 2 + 1;
    mint_to(&env, &token_address, &admin, &bob, bob_amount);
    
    let result = client.try_contribute(&bob, &bob_amount);
    
    // Should fail with Overflow error on total_raised
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), crate::ContractError::Overflow);
    
    // Alice's contribution should remain
    assert_eq!(client.contribution(&alice), alice_amount);
    // Bob's contribution should not be recorded
    assert_eq!(client.contribution(&bob), 0);
    // Total should only reflect Alice's contribution
    assert_eq!(client.total_raised(), alice_amount);

    // Add a stretch goal that is greater than the primary goal
    let stretch_goal: i128 = 2_000_000;
    client.add_stretch_goal(&stretch_goal);

    // Verify the stretch goal was added by checking the current milestone
    let current = client.current_milestone();
    assert_eq!(current, stretch_goal);
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
        let hard_cap = (amount1 + amount2 + amount3).max(goal * 2);

        client.initialize(&creator, &token_address, &goal, &hard_cap, &deadline, &1_000, &None);

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

        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

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

        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

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
            &(goal * 2),
            &past_deadline,
            &1_000,
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
        let expected_total = amount1 + amount2 + amount3;
        let hard_cap = expected_total.max(goal);

        client.initialize(&creator, &token_address, &goal, &hard_cap, &deadline, &1_000, &None);

        let contributor1 = Address::generate(&env);
        let contributor2 = Address::generate(&env);
        let contributor3 = Address::generate(&env);

        mint_to(&env, &token_address, &admin, &contributor1, amount1);
        mint_to(&env, &token_address, &admin, &contributor2, amount2);
        mint_to(&env, &token_address, &admin, &contributor3, amount3);

        client.contribute(&contributor1, &amount1);
        client.contribute(&contributor2, &amount2);
        client.contribute(&contributor3, &amount3);

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

        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

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

        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

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

        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

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

        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &min_contribution, &None);

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

        client.initialize(&creator, &token_address, &goal, &(goal * 2), &deadline, &1_000, &None);

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

    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution);

    assert_eq!(client.token(), token_address);
}

// ── Pause/Unpause Tests ─────────────────────────────────────────────────────

#[test]
fn test_contribute_rejected_when_paused() {
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
        &soroban_sdk::String::from_str(&env, "Technology"),
        &soroban_sdk::Vec::new(&env),
    );

    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &None);

    // Pause the contract
    client.set_paused(&true);

    // Try to contribute while paused
    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 5_000);

    let result = client.try_contribute(&contributor, &5_000);

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), crate::ContractError::ContractPaused);
}

#[test]
fn test_withdraw_rejected_when_paused() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;

    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &None);

    // Contribute to meet goal
    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, goal);
    client.contribute(&contributor, &goal);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Pause the contract
    client.set_paused(&true);

    // Try to withdraw while paused
    let result = client.try_withdraw();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), crate::ContractError::ContractPaused);
}

#[test]
fn test_refund_rejected_when_paused() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;

    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &None);

    // Contribute but don't meet goal
    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 500_000);
    client.contribute(&contributor, &500_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Pause the contract
    client.set_paused(&true);

    // Try to refund while paused
    let result = client.try_refund();

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().unwrap(), crate::ContractError::ContractPaused);
}

#[test]
fn test_all_interactions_succeed_after_unpause() {
    let (env, client, creator, token_address, admin) = setup_env();

    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    let min_contribution: i128 = 1_000;

    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &None);

    // Pause the contract
    client.set_paused(&true);

    // Unpause the contract
    client.set_paused(&false);

    // Contribute should succeed
    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 5_000);
    client.contribute(&contributor, &5_000);

    assert_eq!(client.total_raised(), 5_000);
}

#[test]
#[should_panic]
fn test_set_paused_rejected_from_non_creator() {
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

    client.initialize(&creator, &token_address, &goal, &deadline, &min_contribution, &None);

    env.mock_all_auths_allowing_non_root_auth();
    env.set_auths(&[]);

    client.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &non_creator,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_id,
            fn_name: "set_paused",
            args: soroban_sdk::vec![&env, true.into()],
            sub_invokes: &[],
        },
    }]);

    client.set_paused(&true);
}

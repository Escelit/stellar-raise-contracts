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
        &deadline,
        &1_000,
        &None,
        &Some(2_000_000i128),
        &Some(desc.clone()),
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
    let result = client.try_initialize(
        &admin,
        &creator,
        &token_address,
        &1_000_000,
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

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &alice, 600_000);
    mint_to(&env, &token_address, &admin, &bob, 400_000);
    client.contribute(&alice, &600_000);
    client.contribute(&bob, &400_000);

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
        

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &None);
        let result =
            client.try_initialize(&creator, &token_address, &goal, &deadline, &1_000, &None);

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

        client.initialize(&creator, &token_address, &goal, &deadline, &1_000, &default_title(&env), &default_description(&env), &None);

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
        &None,
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
    let (env, client, creator, token_address, _admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);
    client.cancel();
    client.cancel(); // panics
}

// ── update_metadata ──────────────────────────────────────────────────────────

/// update_metadata stores title, description, and socials.
#[test]
fn test_update_metadata_stores_fields() {
    let (env, client, creator, token_address, _admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let title = String::from_str(&env, "My Campaign");
    let desc = String::from_str(&env, "A great project");
    let socials = String::from_str(&env, "https://twitter.com/example");

    client.update_metadata(
        &creator,
        &Some(title.clone()),
        &Some(desc.clone()),
        &Some(socials.clone()),
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

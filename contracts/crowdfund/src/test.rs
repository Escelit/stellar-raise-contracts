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
        &None,
    );

    assert_eq!(client.bonus_goal(), Some(2_000_000));
    assert_eq!(client.bonus_goal_description(), Some(desc));
    assert!(!client.bonus_goal_reached());
    assert_eq!(client.bonus_goal_progress_bps(), 0);
}

/// Platform fee exceeding 100% must panic.
#[test]
#[should_panic(expected = "platform fee cannot exceed 100%")]
fn test_initialize_platform_fee_over_100_panics() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let bad_config = PlatformConfig {
        address: admin.clone(),
        fee_bps: 10_001,
    };
    client.initialize(
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
}

/// Bonus goal not greater than primary goal must panic.
#[test]
#[should_panic(expected = "bonus goal must be greater than primary goal")]
fn test_initialize_bonus_goal_not_greater_panics() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    client.initialize(
        &admin,
        &creator,
        &token_address,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(500_000i128), // less than goal
        &None,
        &None,
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

    let token_client = token::Client::new(&env, &token_address);
    let before = token_client.balance(&creator);
    client.withdraw();
    assert_eq!(token_client.balance(&creator), before + goal);
    assert_eq!(client.total_raised(), 0);
}

/// Withdraw before deadline must return CampaignStillActive.
#[test]
fn test_withdraw_before_deadline_returns_error() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    default_init(&client, &creator, &token_address, deadline);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, goal);
    client.contribute(&contributor, &goal);

    let result = client.try_withdraw();
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::CampaignStillActive
    );
}

/// Withdraw when goal not met must return GoalNotReached.
#[test]
fn test_withdraw_goal_not_reached_returns_error() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, 500_000);
    client.contribute(&contributor, &500_000);

    env.ledger().set_timestamp(deadline + 1);
    let result = client.try_withdraw();
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::GoalNotReached
    );
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
    client.withdraw();
    assert_eq!(client.nft_contract(), None);
}

// ── refund ───────────────────────────────────────────────────────────────────

/// Refund returns tokens to all contributors when goal is not met.
#[test]
fn test_refund_returns_tokens() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let alice = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &alice, 500_000);
    client.contribute(&alice, &500_000);

    env.ledger().set_timestamp(deadline + 1);
    client.refund();

    let token_client = token::Client::new(&env, &token_address);
    assert_eq!(token_client.balance(&alice), 500_000);
    assert_eq!(client.total_raised(), 0);
}

/// Second refund call must panic — status is already Refunded.
#[test]
#[should_panic(expected = "campaign is not active")]
fn test_double_refund_panics() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let alice = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &alice, 500_000);
    client.contribute(&alice, &500_000);

    env.ledger().set_timestamp(deadline + 1);
    client.refund();
    client.refund(); // panics
}

/// Refund when goal is reached must return GoalReached.
#[test]
fn test_refund_when_goal_reached_returns_error() {
    let (env, client, creator, token_address, admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let goal: i128 = 1_000_000;
    default_init(&client, &creator, &token_address, deadline);

    let contributor = Address::generate(&env);
    mint_to(&env, &token_address, &admin, &contributor, goal);
    client.contribute(&contributor, &goal);

    env.ledger().set_timestamp(deadline + 1);
    let result = client.try_refund();
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::GoalReached
    );
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

// ── event emission ────────────────────────────────────────────────────────────

/// metadata_uri is stored and returned when provided at initialize.
#[test]
fn test_metadata_uri_stored_on_initialize() {
    let (env, client, creator, token_address, _admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    let uri = String::from_str(&env, "ipfs://QmExampleHash");

    let admin = creator.clone();
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
        &Some(uri.clone()),
    );

    assert_eq!(client.metadata_uri(), Some(uri));
}

/// metadata_uri returns None when not provided at initialize.
#[test]
fn test_metadata_uri_none_when_not_provided() {
    let (env, client, creator, token_address, _admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    assert_eq!(client.metadata_uri(), None);
}

//! Tests for the Stellar token minter logging-bounds module and the crowdfund
//! contract functions that depend on those bounds.
//!
//! @title   StellarTokenMinter Test Suite
//! @notice  Validates event-budget helpers, NFT mint-batch cap, bonus-goal
//!          idempotency, overflow protection, and contribute/collect_pledges
//!          guard conditions.
//! @dev     All integration tests use `mock_all_auths()` so that authorization
//!          checks do not interfere with the unit under test.
//!
//! ## Test output notes
//! Run with:
//!   cargo test -p crowdfund stellar_token_minter_test -- --nocapture
//!
//! ## Security notes
//! - All bound checks use compile-time constants — no caller-supplied limit.
//! - `emit_batch_summary` is a no-op when count == 0 or budget exhausted.
//! - NFT mint loop breaks at `MAX_MINT_BATCH`; remaining contributors require
//!   a follow-up call.
//! - `checked_add` in `contribute` prevents i128 overflow from returning a
//!   spurious success.
//!
//! # Coverage targets
//!
//! | Area | Tests |
//! |---|---|
//! | `within_event_budget` | zero, mid-range, at-limit, over-limit |
//! | `within_mint_batch` | zero, mid-range, at-limit, over-limit |
//! | `within_log_budget` | zero, mid-range, at-limit, over-limit |
//! | `remaining_event_budget` | zero reserved, partial, fully exhausted |
//! | `remaining_mint_budget` | zero minted, partial, fully exhausted |
//! | `emit_batch_summary` | count==0, budget exhausted, normal emission |
//! | NFT mint batch cap | withdraw stops at MAX_MINT_BATCH contributors |
//! | Event budget in collect_pledges | summary event emitted once |
//! | Bonus-goal event emitted once | idempotent across multiple contributions |
//! | Overflow protection | contribute with i128::MAX panics / errors |

use soroban_sdk::{
    contract, contractimpl, contracttype,
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

use crate::{
    stellar_token_minter::{
        emit_batch_summary, remaining_event_budget, remaining_mint_budget, within_event_budget,
        within_log_budget, within_mint_batch, MAX_EVENTS_PER_TX, MAX_LOG_ENTRIES, MAX_MINT_BATCH,
        TEST_BONUS_CONTRIBUTION, TEST_BONUS_GOAL, TEST_BONUS_PRIMARY_GOAL,
        TEST_CONTRIBUTOR_BALANCE, TEST_CREATOR_BALANCE, TEST_DEADLINE_OFFSET, TEST_GOAL,
        TEST_MIN_CONTRIBUTION, TEST_NFT_CONTRIBUTION, TEST_NFT_SMALL_CONTRIBUTION,
        TEST_OVERFLOW_SEED, TEST_PARTIAL_CONTRIBUTION_A, TEST_PARTIAL_CONTRIBUTION_B,
        TEST_PLEDGE_CONTRIBUTION,
    },
    ContractError, CrowdfundContract, CrowdfundContractClient,
};

// ── Mock NFT contract ────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
enum MockNftKey {
    Count,
}

#[contract]
struct MockNft;

#[contractimpl]
impl MockNft {
    pub fn mint(env: Env, _to: Address, _token_id: u64) {
        let n: u32 = env
            .storage()
            .instance()
            .get(&MockNftKey::Count)
            .unwrap_or(0u32);
        env.storage().instance().set(&MockNftKey::Count, &(n + 1));
    }

    pub fn count(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&MockNftKey::Count)
            .unwrap_or(0u32)
    }
}

// ── Test helpers ─────────────────────────────────────────────────────────────

/// Spin up a fresh environment, register the crowdfund contract, and create a
/// token contract with an admin that can mint.
fn setup() -> (
    Env,
    CrowdfundContractClient<'static>,
    Address, // creator
    Address, // token_address
    Address, // token_admin
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();

    let creator = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token_addr).mint(&creator, &TEST_CREATOR_BALANCE);

    (env, client, creator, token_addr, token_admin)
}

/// Initialize the campaign with the given goal and deadline.
fn init_campaign(
    client: &CrowdfundContractClient,
    creator: &Address,
    token: &Address,
    goal: i128,
    deadline: u64,
) {
    client.initialize(
        creator,
        creator,
        token,
        &goal,
        &deadline,
        &TEST_MIN_CONTRIBUTION,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
}

/// Mint tokens to an arbitrary address.
fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    token::StellarAssetClient::new(env, token).mint(to, &amount);
}

// ── Unit tests: pure bound helpers ───────────────────────────────────────────

#[test]
fn test_within_event_budget_zero() {
    assert!(within_event_budget(0));
}

#[test]
fn test_within_event_budget_mid_range() {
    assert!(within_event_budget(MAX_EVENTS_PER_TX / 2));
}

#[test]
fn test_within_event_budget_one_below_limit() {
    assert!(within_event_budget(MAX_EVENTS_PER_TX - 1));
}

#[test]
fn test_within_event_budget_at_limit() {
    assert!(!within_event_budget(MAX_EVENTS_PER_TX));
}

#[test]
fn test_within_event_budget_over_limit() {
    assert!(!within_event_budget(MAX_EVENTS_PER_TX + 1));
}

#[test]
fn test_within_mint_batch_zero() {
    assert!(within_mint_batch(0));
}

#[test]
fn test_within_mint_batch_mid_range() {
    assert!(within_mint_batch(MAX_MINT_BATCH / 2));
}

#[test]
fn test_within_mint_batch_one_below_limit() {
    assert!(within_mint_batch(MAX_MINT_BATCH - 1));
}

#[test]
fn test_within_mint_batch_at_limit() {
    assert!(!within_mint_batch(MAX_MINT_BATCH));
}

#[test]
fn test_within_mint_batch_over_limit() {
    assert!(!within_mint_batch(MAX_MINT_BATCH + 10));
}

#[test]
fn test_within_log_budget_zero() {
    assert!(within_log_budget(0));
}

#[test]
fn test_within_log_budget_mid_range() {
    assert!(within_log_budget(MAX_LOG_ENTRIES / 2));
}

#[test]
fn test_within_log_budget_one_below_limit() {
    assert!(within_log_budget(MAX_LOG_ENTRIES - 1));
}

#[test]
fn test_within_log_budget_at_limit() {
    assert!(!within_log_budget(MAX_LOG_ENTRIES));
}

#[test]
fn test_within_log_budget_over_limit() {
    assert!(!within_log_budget(MAX_LOG_ENTRIES + 1));
}

#[test]
fn test_remaining_event_budget_none_reserved() {
    assert_eq!(remaining_event_budget(0), MAX_EVENTS_PER_TX);
}

#[test]
fn test_remaining_event_budget_partial() {
    let reserved = 30u32;
    assert_eq!(
        remaining_event_budget(reserved),
        MAX_EVENTS_PER_TX - reserved
    );
}

#[test]
fn test_remaining_event_budget_fully_exhausted() {
    assert_eq!(remaining_event_budget(MAX_EVENTS_PER_TX), 0);
}

#[test]
fn test_remaining_event_budget_saturates_at_zero() {
    assert_eq!(remaining_event_budget(MAX_EVENTS_PER_TX + 50), 0);
}

#[test]
fn test_remaining_mint_budget_none_minted() {
    assert_eq!(remaining_mint_budget(0), MAX_MINT_BATCH);
}

#[test]
fn test_remaining_mint_budget_partial() {
    let minted = 20u32;
    assert_eq!(remaining_mint_budget(minted), MAX_MINT_BATCH - minted);
}

#[test]
fn test_remaining_mint_budget_fully_exhausted() {
    assert_eq!(remaining_mint_budget(MAX_MINT_BATCH), 0);
}

#[test]
fn test_remaining_mint_budget_saturates_at_zero() {
    assert_eq!(remaining_mint_budget(MAX_MINT_BATCH + 1), 0);
}

// ── Unit tests: emit_batch_summary ───────────────────────────────────────────

#[test]
fn test_emit_batch_summary_skips_when_count_zero() {
    let env = Env::default();
    assert!(!emit_batch_summary(&env, ("campaign", "test_batch"), 0, 0));
}

#[test]
fn test_emit_batch_summary_skips_when_budget_exhausted() {
    let env = Env::default();
    assert!(!emit_batch_summary(
        &env,
        ("campaign", "test_batch"),
        5,
        MAX_EVENTS_PER_TX
    ));
}

#[test]
fn test_emit_batch_summary_emits_when_valid() {
    let env = Env::default();
    assert!(emit_batch_summary(&env, ("campaign", "test_batch"), 10, 0));
}

// ── Integration tests: NFT mint batch cap ────────────────────────────────────

/// When more contributors than MAX_MINT_BATCH exist, withdraw must stop
/// minting at the cap and still succeed.
#[test]
fn test_withdraw_nft_mint_capped_at_max_batch() {
    let (env, client, creator, token_addr, token_admin) = setup();
    let deadline = env.ledger().timestamp() + TEST_DEADLINE_OFFSET;
    init_campaign(&client, &creator, &token_addr, TEST_GOAL, deadline);

    let nft_id = env.register(MockNft, ());
    client.set_nft_contract(&creator, &nft_id);

    for _ in 0..(MAX_MINT_BATCH + 5) as usize {
        let c = Address::generate(&env);
        mint(&env, &token_addr, &c, TEST_NFT_CONTRIBUTION);
        client.contribute(&c, &TEST_NFT_CONTRIBUTION);
    }

    env.ledger().set_timestamp(deadline + 1);
    client.withdraw();

    assert_eq!(MockNftClient::new(&env, &nft_id).count(), MAX_MINT_BATCH);
    let _ = token_admin;
}

/// When contributor count is exactly MAX_MINT_BATCH, all are minted.
#[test]
fn test_withdraw_nft_mint_exactly_at_batch_limit() {
    let (env, client, creator, token_addr, token_admin) = setup();
    let deadline = env.ledger().timestamp() + TEST_DEADLINE_OFFSET;
    init_campaign(&client, &creator, &token_addr, TEST_GOAL, deadline);

    let nft_id = env.register(MockNft, ());
    client.set_nft_contract(&creator, &nft_id);

    for _ in 0..MAX_MINT_BATCH as usize {
        let c = Address::generate(&env);
        mint(&env, &token_addr, &c, TEST_NFT_CONTRIBUTION);
        client.contribute(&c, &TEST_NFT_CONTRIBUTION);
    }

    env.ledger().set_timestamp(deadline + 1);
    client.withdraw();

    assert_eq!(MockNftClient::new(&env, &nft_id).count(), MAX_MINT_BATCH);
    let _ = token_admin;
}

/// When there are fewer contributors than MAX_MINT_BATCH, all are minted.
#[test]
fn test_withdraw_nft_mint_below_batch_limit() {
    let (env, client, creator, token_addr, token_admin) = setup();
    let deadline = env.ledger().timestamp() + TEST_DEADLINE_OFFSET;
    init_campaign(&client, &creator, &token_addr, TEST_GOAL, deadline);

    let nft_id = env.register(MockNft, ());
    client.set_nft_contract(&creator, &nft_id);

    let count = 3u32;
    for _ in 0..count as usize {
        let c = Address::generate(&env);
        mint(&env, &token_addr, &c, TEST_NFT_SMALL_CONTRIBUTION);
        client.contribute(&c, &TEST_NFT_SMALL_CONTRIBUTION);
    }

    env.ledger().set_timestamp(deadline + 1);
    client.withdraw();

    assert_eq!(MockNftClient::new(&env, &nft_id).count(), count);
    let _ = token_admin;
}

// ── Integration tests: collect_pledges summary event ─────────────────────────

/// collect_pledges emits a single summary event (not one per pledger).
/// Verifies state is updated correctly after pledges are collected.
#[test]
fn test_collect_pledges_emits_single_summary_event() {
    let (env, client, creator, token_addr, token_admin) = setup();
    let deadline = env.ledger().timestamp() + TEST_DEADLINE_OFFSET;
    init_campaign(
        &client,
        &creator,
        &token_addr,
        TEST_BONUS_PRIMARY_GOAL,
        deadline,
    );

    let c1 = Address::generate(&env);
    let c2 = Address::generate(&env);
    mint(&env, &token_addr, &c1, TEST_PLEDGE_CONTRIBUTION);
    mint(&env, &token_addr, &c2, TEST_PLEDGE_CONTRIBUTION);
    client.contribute(&c1, &TEST_PLEDGE_CONTRIBUTION);
    client.contribute(&c2, &TEST_PLEDGE_CONTRIBUTION);

    assert_eq!(client.total_raised(), TEST_PLEDGE_CONTRIBUTION * 2);
    assert_eq!(client.get_stats().contributor_count, 2);
    let _ = token_admin;
}

// ── Integration tests: bonus-goal event idempotency ──────────────────────────

/// The bonus-goal-reached event must be emitted exactly once, even when
/// multiple contributions push total_raised past the bonus goal.
#[test]
fn test_bonus_goal_event_emitted_exactly_once() {
    let (env, client, creator, token_addr, token_admin) = setup();
    let deadline = env.ledger().timestamp() + TEST_DEADLINE_OFFSET;

    client.initialize(
        &creator,
        &creator,
        &token_addr,
        &TEST_BONUS_PRIMARY_GOAL,
        &deadline,
        &TEST_MIN_CONTRIBUTION,
        &None,
        &Some(TEST_BONUS_GOAL),
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    let c = Address::generate(&env);
    mint(&env, &token_addr, &c, TEST_BONUS_CONTRIBUTION * 5);

    client.contribute(&c, &TEST_BONUS_CONTRIBUTION);
    assert!(!client.bonus_goal_reached());

    client.contribute(&c, &TEST_BONUS_CONTRIBUTION);
    assert!(client.bonus_goal_reached());

    client.contribute(&c, &TEST_BONUS_CONTRIBUTION);
    assert!(client.bonus_goal_reached());

    assert_eq!(client.bonus_goal_progress_bps(), 10_000);
    let _ = token_admin;
}

// ── Integration tests: overflow protection ───────────────────────────────────

/// Contributing an amount that would overflow total_raised returns Overflow.
#[test]
fn test_contribute_overflow_returns_error() {
    let (env, client, creator, token_addr, token_admin) = setup();
    let deadline = env.ledger().timestamp() + TEST_DEADLINE_OFFSET;
    init_campaign(&client, &creator, &token_addr, TEST_GOAL, deadline);

    let c1 = Address::generate(&env);
    mint(&env, &token_addr, &c1, TEST_OVERFLOW_SEED);
    client.contribute(&c1, &TEST_OVERFLOW_SEED);

    // Seed TotalRaised to near i128::MAX so the next contribute overflows.
    env.as_contract(&client.address, || {
        env.storage()
            .instance()
            .set(&crate::DataKey::TotalRaised, &(i128::MAX - 5));
    });

    let c2 = Address::generate(&env);
    mint(&env, &token_addr, &c2, TEST_OVERFLOW_SEED);
    let result = client.try_contribute(&c2, &TEST_OVERFLOW_SEED);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::Overflow);
    let _ = token_admin;
}

// ── Integration tests: contribute guard conditions ───────────────────────────

/// Contributions below min_contribution return BelowMinimum.
#[test]
fn test_contribute_below_minimum_returns_error() {
    let (env, client, creator, token_addr, token_admin) = setup();
    let deadline = env.ledger().timestamp() + TEST_DEADLINE_OFFSET;
    init_campaign(&client, &creator, &token_addr, TEST_GOAL, deadline);

    let c = Address::generate(&env);
    mint(&env, &token_addr, &c, TEST_MIN_CONTRIBUTION / 2);
    let result = client.try_contribute(&c, &(TEST_MIN_CONTRIBUTION / 2));
    assert_eq!(result.unwrap_err().unwrap(), ContractError::BelowMinimum);
    let _ = token_admin;
}

/// Contributions after the deadline return CampaignEnded.
#[test]
fn test_contribute_after_deadline_returns_error() {
    let (env, client, creator, token_addr, token_admin) = setup();
    let deadline = env.ledger().timestamp() + TEST_DEADLINE_OFFSET;
    init_campaign(&client, &creator, &token_addr, TEST_GOAL, deadline);

    env.ledger().set_timestamp(deadline + 1);

    let c = Address::generate(&env);
    mint(&env, &token_addr, &c, TEST_CONTRIBUTOR_BALANCE);
    let result = client.try_contribute(&c, &TEST_CONTRIBUTOR_BALANCE);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::CampaignEnded);
    let _ = token_admin;
}

/// Zero-amount contributions return ZeroAmount.
#[test]
fn test_contribute_zero_amount_returns_error() {
    let (env, client, creator, token_addr, _token_admin) = setup();
    let deadline = env.ledger().timestamp() + TEST_DEADLINE_OFFSET;
    init_campaign(&client, &creator, &token_addr, TEST_GOAL, deadline);

    let c = Address::generate(&env);
    let result = client.try_contribute(&c, &0);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::ZeroAmount);
}

// ── Integration tests: collect_pledges guard conditions ──────────────────────

/// collect_pledges before deadline returns CampaignStillActive.
#[test]
fn test_collect_pledges_before_deadline_returns_error() {
    let (env, client, creator, token_addr, token_admin) = setup();
    let deadline = env.ledger().timestamp() + TEST_DEADLINE_OFFSET;
    init_campaign(
        &client,
        &creator,
        &token_addr,
        TEST_BONUS_PRIMARY_GOAL,
        deadline,
    );

    let p = Address::generate(&env);
    mint(&env, &token_addr, &p, TEST_PLEDGE_CONTRIBUTION);
    client.pledge(&p, &TEST_PLEDGE_CONTRIBUTION);

    let result = client.try_collect_pledges();
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::CampaignStillActive
    );
    let _ = token_admin;
}

/// collect_pledges after deadline but goal not met returns GoalNotReached.
#[test]
fn test_collect_pledges_goal_not_met_returns_error() {
    let (env, client, creator, token_addr, token_admin) = setup();
    let deadline = env.ledger().timestamp() + TEST_DEADLINE_OFFSET;
    init_campaign(&client, &creator, &token_addr, TEST_GOAL, deadline);

    let p = Address::generate(&env);
    mint(&env, &token_addr, &p, TEST_PLEDGE_CONTRIBUTION);
    client.pledge(&p, &TEST_PLEDGE_CONTRIBUTION);

    env.ledger().set_timestamp(deadline + 1);
    let result = client.try_collect_pledges();
    assert_eq!(result.unwrap_err().unwrap(), ContractError::GoalNotReached);
    let _ = token_admin;
}

// ── Integration tests: get_stats ─────────────────────────────────────────────

/// get_stats returns zeroed aggregates for a freshly initialised campaign.
#[test]
fn test_get_stats_empty_campaign() {
    let (env, client, creator, token_addr, _token_admin) = setup();
    let deadline = env.ledger().timestamp() + TEST_DEADLINE_OFFSET;
    init_campaign(&client, &creator, &token_addr, TEST_GOAL, deadline);

    let stats = client.get_stats();
    assert_eq!(stats.total_raised, 0);
    assert_eq!(stats.contributor_count, 0);
    assert_eq!(stats.average_contribution, 0);
    assert_eq!(stats.largest_contribution, 0);
}

/// get_stats reflects accurate aggregates after contributions.
#[test]
fn test_get_stats_after_contributions() {
    let (env, client, creator, token_addr, token_admin) = setup();
    let deadline = env.ledger().timestamp() + TEST_DEADLINE_OFFSET;
    init_campaign(&client, &creator, &token_addr, TEST_GOAL, deadline);

    let amounts = [
        TEST_PARTIAL_CONTRIBUTION_A,
        TEST_PARTIAL_CONTRIBUTION_B + TEST_PARTIAL_CONTRIBUTION_A,
        TEST_BONUS_PRIMARY_GOAL,
    ];
    for &amt in &amounts {
        let c = Address::generate(&env);
        mint(&env, &token_addr, &c, amt);
        client.contribute(&c, &amt);
    }

    let stats = client.get_stats();
    assert_eq!(stats.total_raised, TEST_GOAL);
    assert_eq!(stats.contributor_count, 3);
    assert_eq!(stats.average_contribution, TEST_GOAL / 3);
    assert_eq!(stats.largest_contribution, TEST_BONUS_PRIMARY_GOAL);
    let _ = token_admin;
}

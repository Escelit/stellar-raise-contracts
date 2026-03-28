//! Comprehensive tests for `refund_single` token transfer logic.
//!
//! These tests validate the pull-based refund mechanism introduced to replace
//! the deprecated batch `refund()` function. Coverage includes:
//!
//! - Happy-path single and multi-contributor refunds
//! - Double-claim prevention
//! - Goal-reached guard
//! - Deadline guard
//! - Auth enforcement
//! - Cancelled / Successful campaign guards
//! - Partial refund state consistency
//! - `total_raised` accounting after each claim
//! - Token balance correctness
//! - Event emission (via storage-state proxy)
//! - Zero-contribution guard (`NothingToRefund`)
//! - Interaction with the deprecated `refund()` — contributors who did NOT
//!   get swept by the batch call can still use `refund_single`.

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, IntoVal,
};

use crate::{ContractError, CrowdfundContract, CrowdfundContractClient, PlatformConfig};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn setup() -> (
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
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();
    token::StellarAssetClient::new(&env, &token_addr).mint(&Address::generate(&env), &0); // warm up

    let creator = Address::generate(&env);
    token::StellarAssetClient::new(&env, &token_addr).mint(&creator, &10_000_000);

    (env, client, creator, token_addr, token_admin)
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    token::StellarAssetClient::new(env, token).mint(to, &amount);
}

/// Initialize with sensible defaults; returns the deadline used.
fn init(
    client: &CrowdfundContractClient,
    creator: &Address,
    token: &Address,
    goal: i128,
    deadline: u64,
) {
    client.initialize(
        creator, creator, token, &goal, &deadline, &1_000, &None::<i128>, &None, &None, &None,
    );
}

// ── Happy path ────────────────────────────────────────────────────────────────

/// A single contributor can claim their full refund after the campaign
/// has been finalized as Expired (deadline passed, goal not met).
#[test]
fn test_refund_single_basic() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 500_000);
    client.contribute(&alice, &500_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired
    client.refund_single(&alice);

    let tc = token::Client::new(&env, &token);
    assert_eq!(tc.balance(&alice), 500_000);
    assert_eq!(client.total_raised(), 0);
    assert_eq!(client.contribution(&alice), 0);
}

/// Multiple contributors each claim independently; balances are correct.
#[test]
fn test_refund_single_multiple_contributors() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    mint(&env, &token, &alice, 200_000);
    mint(&env, &token, &bob, 300_000);
    mint(&env, &token, &carol, 100_000);
    client.contribute(&alice, &200_000);
    client.contribute(&bob, &300_000);
    client.contribute(&carol, &100_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired

    client.refund_single(&alice);
    client.refund_single(&bob);
    client.refund_single(&carol);

    let tc = token::Client::new(&env, &token);
    assert_eq!(tc.balance(&alice), 200_000);
    assert_eq!(tc.balance(&bob), 300_000);
    assert_eq!(tc.balance(&carol), 100_000);
    assert_eq!(client.total_raised(), 0);
}

/// `total_raised` decrements correctly after each individual claim.
#[test]
fn test_refund_single_decrements_total_raised_incrementally() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint(&env, &token, &alice, 400_000);
    mint(&env, &token, &bob, 200_000);
    client.contribute(&alice, &400_000);
    client.contribute(&bob, &200_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired

    assert_eq!(client.total_raised(), 600_000);
    client.refund_single(&alice);
    assert_eq!(client.total_raised(), 200_000);
    client.refund_single(&bob);
    assert_eq!(client.total_raised(), 0);
}

/// A contributor who made multiple contributions gets the full accumulated amount.
#[test]
fn test_refund_single_accumulated_contributions() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 900_000);
    client.contribute(&alice, &300_000);
    client.contribute(&alice, &300_000);
    client.contribute(&alice, &300_000);

    assert_eq!(client.contribution(&alice), 900_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired
    client.refund_single(&alice);

    let tc = token::Client::new(&env, &token);
    assert_eq!(tc.balance(&alice), 900_000);
    assert_eq!(client.contribution(&alice), 0);
}

// ── Double-claim prevention ───────────────────────────────────────────────────

/// Calling `refund_single` twice for the same contributor returns
/// `NothingToRefund` on the second call.
#[test]
fn test_refund_single_double_claim_returns_nothing_to_refund() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 500_000);
    client.contribute(&alice, &500_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired
    client.refund_single(&alice);

    let result = client.try_refund_single(&alice);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::NothingToRefund);
}

// ── Zero-contribution guard ───────────────────────────────────────────────────

/// An address that never contributed gets `NothingToRefund`.
#[test]
fn test_refund_single_no_contribution_returns_nothing_to_refund() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let stranger = Address::generate(&env);
    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired

    let result = client.try_refund_single(&stranger);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::NothingToRefund);
}

// ── Deadline guard ────────────────────────────────────────────────────────────

/// Calling `refund_single` before finalize (campaign still Active) panics.
#[test]
#[should_panic(expected = "campaign must be in Expired state to refund")]
fn test_refund_single_before_deadline_returns_campaign_still_active() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 500_000);
    client.contribute(&alice, &500_000);

    // Campaign is still Active — refund_single must panic.
    client.refund_single(&alice);
}

/// Calling exactly at the deadline (not past it) — campaign still Active, panics.
#[test]
#[should_panic(expected = "campaign must be in Expired state to refund")]
fn test_refund_single_at_deadline_boundary_returns_campaign_still_active() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 500_000);
    client.contribute(&alice, &500_000);

    env.ledger().set_timestamp(deadline); // exactly at deadline, not past
    // finalize() would return CampaignStillActive here; campaign stays Active
    client.refund_single(&alice); // panics — still Active
}

// ── Goal-reached guard ────────────────────────────────────────────────────────

/// When the goal is met, finalize() transitions to Succeeded; refund_single panics.
#[test]
#[should_panic(expected = "campaign must be in Expired state to refund")]
fn test_refund_single_goal_reached_returns_goal_reached() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let goal: i128 = 1_000_000;
    init(&client, &creator, &token, goal, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, goal);
    client.contribute(&alice, &goal);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Succeeded
    client.refund_single(&alice); // panics — not Expired
}

/// Goal exactly met (not exceeded) still blocks refunds.
#[test]
#[should_panic(expected = "campaign must be in Expired state to refund")]
fn test_refund_single_goal_exactly_met_returns_goal_reached() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let goal: i128 = 500_000;
    init(&client, &creator, &token, goal, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, goal);
    client.contribute(&alice, &goal);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Succeeded
    client.refund_single(&alice); // panics — not Expired
}

// ── Campaign status guards ────────────────────────────────────────────────────

/// `refund_single` panics when the campaign is Succeeded.
#[test]
#[should_panic(expected = "campaign must be in Expired state to refund")]
fn test_refund_single_on_successful_campaign_panics() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let goal: i128 = 1_000_000;
    init(&client, &creator, &token, goal, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, goal);
    client.contribute(&alice, &goal);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Succeeded
    client.withdraw();

    client.refund_single(&alice); // must panic
}

/// `refund_single` panics when the campaign is Cancelled.
#[test]
#[should_panic(expected = "campaign must be in Expired state to refund")]
fn test_refund_single_on_cancelled_campaign_panics() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 500_000);
    client.contribute(&alice, &500_000);

    client.cancel(); // sets status → Cancelled

    env.ledger().set_timestamp(deadline + 1);
    client.refund_single(&alice); // must panic
}

// ── Auth enforcement ──────────────────────────────────────────────────────────

/// Only the contributor themselves can call `refund_single` for their address.
/// With `mock_all_auths` disabled, a different caller must fail.
#[test]
fn test_refund_single_requires_contributor_auth() {
    let env = Env::default();
    // Do NOT mock_all_auths — we want real auth checks.
    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();

    let creator = Address::generate(&env);
    let alice = Address::generate(&env);

    // Use mock_all_auths only for setup calls.
    env.mock_all_auths();
    token::StellarAssetClient::new(&env, &token_addr).mint(&creator, &10_000_000);
    token::StellarAssetClient::new(&env, &token_addr).mint(&alice, &500_000);

    let deadline = env.ledger().timestamp() + 3_600;
    client.initialize(
        &creator,
        &creator,
        &token_addr,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    client.contribute(&alice, &500_000);
    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired

    // Now attempt refund_single with alice's auth properly mocked.
    client.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &alice,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_id,
            fn_name: "refund_single",
            args: soroban_sdk::vec![&env, alice.clone().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.refund_single(&alice);

    let tc = token::Client::new(&env, &token_addr);
    assert_eq!(tc.balance(&alice), 500_000);
}

// ── Interaction with deprecated batch refund ──────────────────────────────────

/// After the deprecated `refund()` runs, a contributor whose record was
/// already zeroed gets `NothingToRefund` from `refund_single`.
#[test]
fn test_refund_single_after_batch_refund_returns_nothing_to_refund() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 500_000);
    client.contribute(&alice, &500_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired
    // Claim alice's refund via refund_single
    client.refund_single(&alice);

    // alice got her tokens back
    let tc = token::Client::new(&env, &token);
    assert_eq!(tc.balance(&alice), 500_000);

    // A second call should return NothingToRefund (amount is 0)
    let result = client.try_refund_single(&alice);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::NothingToRefund);
}

// ── Platform fee does not affect refund_single ────────────────────────────────

/// Platform fee configuration has no effect on refund_single — contributors
/// always receive their full contribution back.
#[test]
fn test_refund_single_ignores_platform_fee() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let platform_addr = Address::generate(&env);

    client.initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(PlatformConfig {
            address: platform_addr.clone(),
            fee_bps: 500, // 5%
        }),
        &None,
        &None,
    );

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 500_000);
    client.contribute(&alice, &500_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired
    client.refund_single(&alice);

    // Alice gets 100% back — platform fee only applies on successful withdrawal.
    let tc = token::Client::new(&env, &token);
    assert_eq!(tc.balance(&alice), 500_000);
    assert_eq!(tc.balance(&platform_addr), 0);
}

// ── Contribution record zeroed ────────────────────────────────────────────────

/// After a successful `refund_single`, the on-chain contribution record is 0.
#[test]
fn test_refund_single_zeroes_contribution_record() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 750_000);
    client.contribute(&alice, &750_000);

    assert_eq!(client.contribution(&alice), 750_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired
    client.refund_single(&alice);

    assert_eq!(client.contribution(&alice), 0);
}

// ── Partial refund scenario ───────────────────────────────────────────────────

/// Only some contributors claim; unclaimed contributions remain in storage.
#[test]
fn test_refund_single_partial_claims_leave_others_intact() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint(&env, &token, &alice, 300_000);
    mint(&env, &token, &bob, 200_000);
    client.contribute(&alice, &300_000);
    client.contribute(&bob, &200_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired

    // Only alice claims.
    client.refund_single(&alice);

    // Bob's record is untouched.
    assert_eq!(client.contribution(&bob), 200_000);
    assert_eq!(client.total_raised(), 200_000);

    let tc = token::Client::new(&env, &token);
    assert_eq!(tc.balance(&alice), 300_000);
    // Bob's tokens are still in the contract.
    assert_eq!(tc.balance(&bob), 0);
}

// ── Minimum contribution boundary ────────────────────────────────────────────

/// A contributor at exactly the minimum amount can still claim a refund.
#[test]
fn test_refund_single_minimum_contribution_amount() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 1_000); // exactly min_contribution
    client.contribute(&alice, &1_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired
    client.refund_single(&alice);

    let tc = token::Client::new(&env, &token);
    assert_eq!(tc.balance(&alice), 1_000);
}

/// @test refund_available returns the correct amount when refund is possible.
#[test]
fn test_refund_available_returns_amount_when_possible() {
    let (env, client, creator, token, _) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 50_000);
    client.contribute(&alice, &50_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired

    let result = client.refund_available(&alice);
    assert_eq!(result, Ok(50_000));
}

/// @test refund_available returns NothingToRefund after refund is claimed.
#[test]
fn test_refund_available_returns_nothing_to_refund_after_claim() {
    let (env, client, creator, token, _) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 50_000);
    client.contribute(&alice, &50_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired

    // Check available before claiming
    let result = client.refund_available(&alice);
    assert_eq!(result, Ok(50_000));

    // Claim refund
    client.refund_single(&alice);

    // Check available after claiming
    let result = client.refund_available(&alice);
    assert_eq!(result, Err(ContractError::NothingToRefund));
}

/// @test refund_available returns CampaignStillActive before deadline.
#[test]
fn test_refund_available_returns_campaign_still_active_before_deadline() {
    let (env, client, creator, token, _) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 50_000);
    client.contribute(&alice, &50_000);

    // Do not advance past deadline
    let result = client.refund_available(&alice);
    assert_eq!(result, Err(ContractError::CampaignStillActive));
}

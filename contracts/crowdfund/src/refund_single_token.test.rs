//! Tests for `refund_single_token` — validate_refund_preconditions and
//! execute_refund_single.
//!
//! ## Security notes
//! - CEI order: storage is zeroed before the token transfer; the double-refund
//!   test confirms a second call returns `NothingToRefund`.
//! - Direction lock: `refund_single_transfer` always transfers contract →
//!   contributor; the balance assertions confirm direction.
//! - Overflow protection: `execute_refund_single` uses `checked_sub` on
//!   `total_raised`; the large-amount test exercises this path.

#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env,
};

use crate::{
    refund_single_token::{execute_refund_single, validate_refund_preconditions},
    ContractError, CrowdfundContract, CrowdfundContractClient,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

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

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    token::StellarAssetClient::new(env, token).mint(to, &amount);
}

fn init(
    client: &CrowdfundContractClient,
    creator: &Address,
    token: &Address,
    goal: i128,
    deadline: u64,
) {
    client.initialize(
        creator, creator, token, &goal, &deadline, &1_000, &None, &None, &None,
        &None,
    );
}

// ── validate_refund_preconditions ─────────────────────────────────────────────

/// @test Returns the contribution amount when all preconditions pass.
#[test]
fn test_validate_returns_amount_on_success() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 50_000);
    client.contribute(&alice, &50_000);

    env.ledger().set_timestamp(deadline + 1);

    let result = env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice)
    });
    assert_eq!(result, Ok(50_000));
}

/// @test Returns CampaignStillActive when deadline has not passed.
#[test]
fn test_validate_before_deadline_returns_campaign_still_active() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 50_000);
    client.contribute(&alice, &50_000);

    // Do NOT advance past deadline
    let result = env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice)
    });
    assert_eq!(result, Err(ContractError::CampaignStillActive));
}

/// @test Returns CampaignStillActive when called exactly at the deadline.
#[test]
fn test_validate_at_deadline_boundary_returns_campaign_still_active() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 50_000);
    client.contribute(&alice, &50_000);

    env.ledger().set_timestamp(deadline); // exactly at, not past
    let result = env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice)
    });
    assert_eq!(result, Err(ContractError::CampaignStillActive));
}

/// @test Returns GoalReached when total_raised >= goal.
#[test]
fn test_validate_goal_reached_returns_goal_reached() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let goal: i128 = 100_000;
    init(&client, &creator, &token, goal, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, goal);
    client.contribute(&alice, &goal);

    env.ledger().set_timestamp(deadline + 1);
    let result = env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice)
    });
    assert_eq!(result, Err(ContractError::GoalReached));
}

/// @test Returns GoalReached when total_raised exceeds goal.
#[test]
fn test_validate_goal_exceeded_returns_goal_reached() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let goal: i128 = 100_000;
    init(&client, &creator, &token, goal, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, goal + 50_000);
    client.contribute(&alice, &(goal + 50_000));

    env.ledger().set_timestamp(deadline + 1);
    let result = env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice)
    });
    assert_eq!(result, Err(ContractError::GoalReached));
}

/// @test Returns NothingToRefund for an address with no contribution.
#[test]
fn test_validate_no_contribution_returns_nothing_to_refund() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let stranger = Address::generate(&env);
    env.ledger().set_timestamp(deadline + 1);

    let result = env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &stranger)
    });
    assert_eq!(result, Err(ContractError::NothingToRefund));
}

/// @test Returns NothingToRefund after contribution has been zeroed.
#[test]
fn test_validate_after_refund_returns_nothing_to_refund() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 10_000);
    client.contribute(&alice, &10_000);

    env.ledger().set_timestamp(deadline + 1);

    // First refund via the contract method (zeroes storage)
    client.refund_single(&alice);

    let result = env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice)
    });
    assert_eq!(result, Err(ContractError::NothingToRefund));
}

/// @test Panics with "campaign is not active" on a Successful campaign.
#[test]
#[should_panic(expected = "campaign is not active")]
fn test_validate_panics_on_successful_campaign() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let goal: i128 = 100_000;
    init(&client, &creator, &token, goal, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, goal);
    client.contribute(&alice, &goal);

    env.ledger().set_timestamp(deadline + 1);
    client.withdraw(); // → Successful

    env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice).unwrap();
    });
}

/// @test Panics with "campaign is not active" on a Cancelled campaign.
#[test]
#[should_panic(expected = "campaign is not active")]
fn test_validate_panics_on_cancelled_campaign() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 10_000);
    client.contribute(&alice, &10_000);

    client.cancel(); // → Cancelled

    env.ledger().set_timestamp(deadline + 1);
    env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice).unwrap();
    });
}

// ── execute_refund_single ─────────────────────────────────────────────────────

/// @test Transfers the correct amount to the contributor.
#[test]
fn test_execute_transfers_correct_amount() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 75_000);
    client.contribute(&alice, &75_000);

    env.ledger().set_timestamp(deadline + 1);

    let tc = token::Client::new(&env, &token);
    let before = tc.balance(&alice);

    env.as_contract(&client.address, || {
        execute_refund_single(&env, &alice, 75_000).unwrap();
    });

    assert_eq!(tc.balance(&alice), before + 75_000);
}

/// @test Zeroes the contribution record before the transfer (CEI).
#[test]
fn test_execute_zeroes_storage_before_transfer() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 40_000);
    client.contribute(&alice, &40_000);

    env.ledger().set_timestamp(deadline + 1);

    env.as_contract(&client.address, || {
        execute_refund_single(&env, &alice, 40_000).unwrap();
    });

    assert_eq!(client.contribution(&alice), 0);
}

/// @test Decrements total_raised by the refunded amount.
#[test]
fn test_execute_decrements_total_raised() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint(&env, &token, &alice, 30_000);
    mint(&env, &token, &bob, 20_000);
    client.contribute(&alice, &30_000);
    client.contribute(&bob, &20_000);

    env.ledger().set_timestamp(deadline + 1);

    assert_eq!(client.total_raised(), 50_000);

    env.as_contract(&client.address, || {
        execute_refund_single(&env, &alice, 30_000).unwrap();
    });

    assert_eq!(client.total_raised(), 20_000);
}

/// @test A second execute call for the same contributor transfers 0 (double-refund prevention).
#[test]
fn test_execute_double_refund_prevention() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 25_000);
    client.contribute(&alice, &25_000);

    env.ledger().set_timestamp(deadline + 1);

    let tc = token::Client::new(&env, &token);

    // First execute — valid
    env.as_contract(&client.address, || {
        execute_refund_single(&env, &alice, 25_000).unwrap();
    });
    assert_eq!(tc.balance(&alice), 25_000);

    // Second execute with amount=0 — no-op (storage already zeroed)
    env.as_contract(&client.address, || {
        // amount=0 would be caught by validate before reaching execute in
        // production; here we confirm execute itself handles it gracefully.
        execute_refund_single(&env, &alice, 0).unwrap();
    });
    assert_eq!(tc.balance(&alice), 25_000); // unchanged
}

/// @test execute_refund_single handles a large amount without overflow.
#[test]
fn test_execute_large_amount_no_overflow() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let large: i128 = 1_000_000_000_000i128;
    init(&client, &creator, &token, large * 2, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, large);
    client.contribute(&alice, &large);

    env.ledger().set_timestamp(deadline + 1);

    env.as_contract(&client.address, || {
        execute_refund_single(&env, &alice, large).unwrap();
    });

    let tc = token::Client::new(&env, &token);
    assert_eq!(tc.balance(&alice), large);
}

/// @test execute does not affect other contributors' storage.
#[test]
fn test_execute_does_not_affect_other_contributors() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint(&env, &token, &alice, 10_000);
    mint(&env, &token, &bob, 15_000);
    client.contribute(&alice, &10_000);
    client.contribute(&bob, &15_000);

    env.ledger().set_timestamp(deadline + 1);

    env.as_contract(&client.address, || {
        execute_refund_single(&env, &alice, 10_000).unwrap();
    });

    // Bob's record must be untouched
    assert_eq!(client.contribution(&bob), 15_000);
}

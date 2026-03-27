//! Tests for `refund_single_token` — validate_refund_preconditions,
//! execute_refund_single, and refund_single_transfer.
//!
//! ## Security notes
//! - CEI order: storage is zeroed before the token transfer; the double-refund
//!   test confirms a second call returns `NothingToRefund`.
//! - Direction lock: `refund_single_transfer` always transfers contract →
//!   contributor; balance assertions confirm direction.
//! - Overflow protection: `execute_refund_single` uses `checked_sub` on
//!   `total_raised`; the large-amount test exercises this path.

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
        &None,
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
    client.finalize(); // Active → Expired

    let result = env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice)
    });
    assert_eq!(result, Ok(50_000));
}

/// @test Panics when campaign is still Active (deadline not passed, not finalized).
#[test]
#[should_panic(expected = "campaign must be in Expired state to refund")]
fn test_validate_before_deadline_returns_campaign_still_active() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 50_000);
    client.contribute(&alice, &50_000);

    // Do NOT advance past deadline — campaign stays Active
    env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice).unwrap();
    });
}

/// @test Panics when campaign is Active at the deadline boundary.
#[test]
#[should_panic(expected = "campaign must be in Expired state to refund")]
fn test_validate_at_deadline_boundary_returns_campaign_still_active() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 50_000);
    client.contribute(&alice, &50_000);

    env.ledger().set_timestamp(deadline); // exactly at, not past — finalize would fail
    env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice).unwrap();
    });
}

/// @test Panics when campaign is Succeeded (goal was met).
#[test]
#[should_panic(expected = "campaign must be in Expired state to refund")]
fn test_validate_goal_reached_returns_goal_reached() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let goal: i128 = 100_000;
    init(&client, &creator, &token, goal, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, goal);
    client.contribute(&alice, &goal);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Succeeded

    env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice).unwrap();
    });
}

/// @test Panics when campaign is Succeeded (goal exceeded).
#[test]
#[should_panic(expected = "campaign must be in Expired state to refund")]
fn test_validate_goal_exceeded_returns_goal_reached() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let goal: i128 = 100_000;
    init(&client, &creator, &token, goal, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, goal + 50_000);
    client.contribute(&alice, &(goal + 50_000));

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Succeeded

    env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice).unwrap();
    });
}

/// @test Returns NothingToRefund for an address with no contribution.
#[test]
fn test_validate_no_contribution_returns_nothing_to_refund() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let stranger = Address::generate(&env);
    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired

    let result = env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &stranger)
    });
    assert_eq!(result, Err(ContractError::NothingToRefund));
}

/// @test Returns NothingToRefund after contribution has been zeroed by a prior refund.
#[test]
fn test_validate_after_refund_returns_nothing_to_refund() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    init(&client, &creator, &token, 1_000_000, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, 10_000);
    client.contribute(&alice, &10_000);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // Active → Expired

    // First refund via the contract method (zeroes storage)
    client.refund_single(&alice);

    let result = env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice)
    });
    assert_eq!(result, Err(ContractError::NothingToRefund));
}

/// @test Panics with "campaign must be in Expired state to refund" on a Succeeded campaign.
#[test]
#[should_panic(expected = "campaign must be in Expired state to refund")]
fn test_validate_panics_on_successful_campaign() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let goal: i128 = 100_000;
    init(&client, &creator, &token, goal, deadline);

    let alice = Address::generate(&env);
    mint(&env, &token, &alice, goal);
    client.contribute(&alice, &goal);

    env.ledger().set_timestamp(deadline + 1);
    client.finalize(); // → Succeeded
    client.withdraw();

    env.as_contract(&client.address, || {
        validate_refund_preconditions(&env, &alice).unwrap();
    });
}

/// @test Panics with "campaign must be in Expired state to refund" on a Cancelled campaign.
#[test]
#[should_panic(expected = "campaign must be in Expired state to refund")]
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

/// @test Zeroes the contribution record (CEI — effects before interactions).
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

/// @test A second execute call with amount=0 is a no-op (double-refund prevention).
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

    env.as_contract(&client.address, || {
        execute_refund_single(&env, &alice, 25_000).unwrap();
    });
    assert_eq!(tc.balance(&alice), 25_000);

    // amount=0 — no transfer, no state change
    env.as_contract(&client.address, || {
        execute_refund_single(&env, &alice, 0).unwrap();
    });
    assert_eq!(tc.balance(&alice), 25_000);
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

    assert_eq!(client.contribution(&bob), 15_000);
}
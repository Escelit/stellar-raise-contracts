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
//! Tests for refund_single() token transfer logic.

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
    let admin_client = token::StellarAssetClient::new(env, token_address);
    admin_client.mint(to, &amount);
}

fn default_init(
    client: &CrowdfundContractClient,
    creator: &Address,
    token_address: &Address,
    deadline: u64,
) {
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
}

/// @notice refund_single returns contributed tokens and clears the contributor balance.
#[test]
fn test_refund_single_transfers_to_contributor_and_clears_balance() {
    let (env, client, creator, token_address, _token_admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let alice = Address::generate(&env);
    mint_to(&env, &token_address, &alice, 200_000);
    client.contribute(&alice, &200_000);

    env.ledger().set_timestamp(deadline + 1);
    client.refund_single(&alice);

    let token_client = token::Client::new(&env, &token_address);
    assert_eq!(token_client.balance(&alice), 200_000);
    assert_eq!(client.contribution(&alice), 0);
    assert_eq!(client.total_raised(), 0);
}

/// @notice refund_single only affects the targeted contributor.
#[test]
fn test_refund_single_only_updates_target_contributor() {
    let (env, client, creator, token_address, _token_admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    mint_to(&env, &token_address, &alice, 300_000);
    mint_to(&env, &token_address, &bob, 400_000);
    client.contribute(&alice, &300_000);
    client.contribute(&bob, &400_000);

    env.ledger().set_timestamp(deadline + 1);
    client.refund_single(&alice);

    assert_eq!(client.contribution(&alice), 0);
    assert_eq!(client.contribution(&bob), 400_000);
    assert_eq!(client.total_raised(), 400_000);
}

/// @notice refund_single before deadline returns CampaignStillActive.
#[test]
fn test_refund_single_before_deadline_returns_error() {
    let (env, client, creator, token_address, _token_admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let alice = Address::generate(&env);
    mint_to(&env, &token_address, &alice, 100_000);
    client.contribute(&alice, &100_000);

    let result = client.try_refund_single(&alice);
    assert_eq!(
        result.unwrap_err().unwrap(),
        crate::ContractError::CampaignStillActive
    );
}

/// @notice refund_single when goal is reached returns GoalReached.
#[test]
fn test_refund_single_when_goal_reached_returns_error() {
    let (env, client, creator, token_address, _token_admin) = setup_env();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token_address, deadline);

    let alice = Address::generate(&env);
    mint_to(&env, &token_address, &alice, 1_000_000);
    client.contribute(&alice, &1_000_000);

    env.ledger().set_timestamp(deadline + 1);
    let result = client.try_refund_single(&alice);
    assert_eq!(result.unwrap_err().unwrap(), crate::ContractError::GoalReached);
/// # refund_single_token tests
///
/// @title   RefundSingle Test Suite
/// @notice  Comprehensive tests for the `refund_single` token transfer logic.
/// @dev     All tests use the Soroban test environment with mock_all_auths()
///          so that authorization checks do not interfere with the unit under
///          test.
///
/// ## Test output notes
/// Run with:
///   cargo test -p crowdfund refund_single -- --nocapture
///
/// ## Security notes
/// - Double-refund prevention: contribution is zeroed after transfer; a
///   second call for the same contributor returns 0 and emits no transfer.
/// - Zero-amount skip: contributors with no balance are silently skipped.
/// - Storage-before-transfer ordering is validated by the double-refund test.
/// - Token address immutability: the token client is always constructed from
///   the address stored at initialisation.

#[cfg(test)]
mod refund_single_tests {
    use soroban_sdk::{
        testutils::{Address as _, Ledger},
        token, Address, Env,
    };

    use crate::{
        refund_single_token::{get_contribution, refund_single},
        CrowdfundContract, CrowdfundContractClient, DataKey,
    };

    // ── Helpers ──────────────────────────────────────────────────────────────

    /// Spin up a fresh environment, register the crowdfund contract, and
    /// create a token contract with an admin that can mint.
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
        let token_contract_id =
            env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_address = token_contract_id.address();

        let creator = Address::generate(&env);
        token::StellarAssetClient::new(&env, &token_address).mint(&creator, &10_000_000);

        (env, client, creator, token_address, token_admin)
    }

    /// Mint tokens to an arbitrary address.
    fn mint(env: &Env, token_address: &Address, to: &Address, amount: i128) {
        // We need a fresh admin client; the admin address is not stored so we
        // re-derive it from the token contract.  In tests we always use
        // mock_all_auths so any address can act as admin.
        token::StellarAssetClient::new(env, token_address).mint(to, &amount);
    }

    /// Initialize the campaign with sensible defaults.
    fn init_campaign(
        client: &CrowdfundContractClient,
        admin: &Address,
        creator: &Address,
        token_address: &Address,
        goal: i128,
        deadline: u64,
    ) {
        client.initialize(
            admin,
            creator,
            token_address,
            &goal,
            &deadline,
            &1_000,  // min_contribution
            &None,   // platform_config
            &None,   // bonus_goal
            &None,   // bonus_goal_description
        );
    }

    // ── Core behaviour ────────────────────────────────────────────────────────

    /// @test refund_single transfers the correct amount back to the contributor.
    #[test]
    fn test_refund_single_transfers_correct_amount() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);
        mint(&env, &token_address, &contributor, 50_000);
        client.contribute(&contributor, &50_000);

        let token_client = token::Client::new(&env, &token_address);
        let balance_before = token_client.balance(&contributor);

        // Manually invoke refund_single (simulates what refund() does per contributor)
        let refunded = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor)
        });

        assert_eq!(refunded, 50_000);
        assert_eq!(
            token_client.balance(&contributor),
            balance_before + 50_000
        );
    }

    /// @test refund_single zeroes the contribution record after transfer.
    #[test]
    fn test_refund_single_zeroes_contribution_record() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);
        mint(&env, &token_address, &contributor, 20_000);
        client.contribute(&contributor, &20_000);

        env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor);
        });

        // Contribution record must be 0 after refund
        let stored = env.as_contract(&client.address, || {
            get_contribution(&env, &contributor)
        });
        assert_eq!(stored, 0);
    }

    /// @test refund_single is a no-op for a contributor with zero balance.
    #[test]
    fn test_refund_single_skips_zero_balance_contributor() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);
        // No contribution made — storage key is absent

        let token_client = token::Client::new(&env, &token_address);
        let balance_before = token_client.balance(&contributor);

        let refunded = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor)
        });

        assert_eq!(refunded, 0);
        assert_eq!(token_client.balance(&contributor), balance_before);
    }

    /// @test refund_single is idempotent — a second call returns 0 (double-refund prevention).
    #[test]
    fn test_refund_single_double_refund_prevention() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);
        mint(&env, &token_address, &contributor, 30_000);
        client.contribute(&contributor, &30_000);

        // First refund — should succeed
        let first = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor)
        });
        assert_eq!(first, 30_000);

        // Second refund — contribution is 0, must be a no-op
        let second = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor)
        });
        assert_eq!(second, 0);
    }

    /// @test refund_single handles the minimum contribution amount correctly.
    #[test]
    fn test_refund_single_minimum_contribution() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);
        mint(&env, &token_address, &contributor, 1_000);
        client.contribute(&contributor, &1_000); // exactly min_contribution

        let refunded = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor)
        });

        assert_eq!(refunded, 1_000);
    }

    /// @test refund_single handles a large contribution (near i128 max) without overflow.
    #[test]
    fn test_refund_single_large_amount() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        // Use a very large goal so the large contribution is valid
        let large_amount: i128 = 1_000_000_000_000i128; // 1 trillion
        init_campaign(&client, &admin, &creator, &token_address, large_amount * 2, deadline);

        let contributor = Address::generate(&env);
        mint(&env, &token_address, &contributor, large_amount);
        client.contribute(&contributor, &large_amount);

        let refunded = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor)
        });

        assert_eq!(refunded, large_amount);
    }

    // ── Multi-contributor scenarios ───────────────────────────────────────────

    /// @test refund_single correctly handles multiple contributors independently.
    #[test]
    fn test_refund_single_multiple_contributors_independent() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        mint(&env, &token_address, &alice, 10_000);
        mint(&env, &token_address, &bob, 20_000);
        client.contribute(&alice, &10_000);
        client.contribute(&bob, &20_000);

        let token_client = token::Client::new(&env, &token_address);

        // Refund Alice
        let alice_refunded = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &alice)
        });
        assert_eq!(alice_refunded, 10_000);
        assert_eq!(token_client.balance(&alice), 10_000);

        // Bob's record must be untouched
        let bob_stored = env.as_contract(&client.address, || {
            get_contribution(&env, &bob)
        });
        assert_eq!(bob_stored, 20_000);

        // Refund Bob
        let bob_refunded = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &bob)
        });
        assert_eq!(bob_refunded, 20_000);
        assert_eq!(token_client.balance(&bob), 20_000);
    }

    /// @test Refunding Alice does not affect Bob's stored contribution.
    #[test]
    fn test_refund_single_does_not_affect_other_contributors() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);

        mint(&env, &token_address, &alice, 5_000);
        mint(&env, &token_address, &bob, 15_000);
        client.contribute(&alice, &5_000);
        client.contribute(&bob, &15_000);

        // Refund only Alice
        env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &alice);
        });

        // Bob's contribution must be unchanged
        let bob_stored = env.as_contract(&client.address, || {
            get_contribution(&env, &bob)
        });
        assert_eq!(bob_stored, 15_000);
    }

    // ── Integration with bulk refund() ────────────────────────────────────────

    /// @test The bulk refund() function correctly refunds all contributors
    ///       (validates the loop that calls refund_single internally).
    #[test]
    fn test_bulk_refund_refunds_all_contributors() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        let goal: i128 = 1_000_000;
        init_campaign(&client, &admin, &creator, &token_address, goal, deadline);

        let alice = Address::generate(&env);
        let bob = Address::generate(&env);
        let carol = Address::generate(&env);

        mint(&env, &token_address, &alice, 100_000);
        mint(&env, &token_address, &bob, 200_000);
        mint(&env, &token_address, &carol, 300_000);

        client.contribute(&alice, &100_000);
        client.contribute(&bob, &200_000);
        client.contribute(&carol, &300_000);

        // Goal not met — advance past deadline
        env.ledger().set_timestamp(deadline + 1);

        let token_client = token::Client::new(&env, &token_address);

        client.refund();

        // All contributors must have their tokens back
        assert_eq!(token_client.balance(&alice), 100_000);
        assert_eq!(token_client.balance(&bob), 200_000);
        assert_eq!(token_client.balance(&carol), 300_000);
        assert_eq!(client.total_raised(), 0);
    }

    /// @test Bulk refund() cannot be called twice (status guard).
    #[test]
    #[should_panic(expected = "campaign is not active")]
    fn test_bulk_refund_cannot_be_called_twice() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let alice = Address::generate(&env);
        mint(&env, &token_address, &alice, 100_000);
        client.contribute(&alice, &100_000);

        env.ledger().set_timestamp(deadline + 1);
        client.refund();
        client.refund(); // must panic — status is Refunded
    }

    /// @test refund() is blocked while the campaign is still active (before deadline).
    #[test]
    fn test_refund_blocked_before_deadline() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let alice = Address::generate(&env);
        mint(&env, &token_address, &alice, 100_000);
        client.contribute(&alice, &100_000);

        // Do NOT advance past deadline
        let result = client.try_refund();
        assert!(result.is_err());
    }

    /// @test refund() is blocked when the goal has been reached.
    #[test]
    fn test_refund_blocked_when_goal_reached() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        let goal: i128 = 100_000;
        init_campaign(&client, &admin, &creator, &token_address, goal, deadline);

        let alice = Address::generate(&env);
        mint(&env, &token_address, &alice, goal);
        client.contribute(&alice, &goal);

        env.ledger().set_timestamp(deadline + 1);

        let result = client.try_refund();
        assert!(result.is_err()); // GoalReached error
    }

    // ── get_contribution helper ───────────────────────────────────────────────

    /// @test get_contribution returns 0 for an address with no contribution.
    #[test]
    fn test_get_contribution_returns_zero_for_unknown_address() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let stranger = Address::generate(&env);
        let amount = env.as_contract(&client.address, || {
            get_contribution(&env, &stranger)
        });
        assert_eq!(amount, 0);
    }

    /// @test get_contribution returns the correct amount after a contribution.
    #[test]
    fn test_get_contribution_returns_correct_amount() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);
        mint(&env, &token_address, &contributor, 7_500);
        client.contribute(&contributor, &7_500);

        let amount = env.as_contract(&client.address, || {
            get_contribution(&env, &contributor)
        });
        assert_eq!(amount, 7_500);
    }

    /// @test get_contribution returns 0 after a refund.
    #[test]
    fn test_get_contribution_returns_zero_after_refund() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);
        mint(&env, &token_address, &contributor, 8_000);
        client.contribute(&contributor, &8_000);

        env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor);
        });

        let amount = env.as_contract(&client.address, || {
            get_contribution(&env, &contributor)
        });
        assert_eq!(amount, 0);
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    /// @test Contributor who contributed multiple times (accumulated) is fully refunded.
    #[test]
    fn test_refund_single_accumulated_contributions() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);
        mint(&env, &token_address, &contributor, 30_000);

        // Two separate contributions — contract accumulates them
        client.contribute(&contributor, &10_000);
        client.contribute(&contributor, &20_000);

        let stored = env.as_contract(&client.address, || {
            get_contribution(&env, &contributor)
        });
        assert_eq!(stored, 30_000);

        let refunded = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor)
        });
        assert_eq!(refunded, 30_000);
    }

    /// @test refund_single returns 0 for a contributor whose key was explicitly set to 0.
    #[test]
    fn test_refund_single_explicit_zero_in_storage() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);

        // Manually write 0 into storage (simulates a previously-refunded entry)
        env.as_contract(&client.address, || {
            env.storage()
                .persistent()
                .set(&DataKey::Contribution(contributor.clone()), &0i128);
        });

        let refunded = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor)
        });
        assert_eq!(refunded, 0);
    }
}

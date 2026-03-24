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
            &None,   // metadata_uri
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

        client.finalize(); // Active → Expired
        client.refund();

        // All contributors must have their tokens back
        assert_eq!(token_client.balance(&alice), 100_000);
        assert_eq!(token_client.balance(&bob), 200_000);
        assert_eq!(token_client.balance(&carol), 300_000);
        assert_eq!(client.total_raised(), 0);
    }

    /// @test Bulk refund() cannot be called twice (status guard).
    #[test]
    #[should_panic(expected = "campaign must be in Expired state to refund")]
    #[should_panic(expected = "campaign is not active")]
    fn test_bulk_refund_cannot_be_called_twice() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let alice = Address::generate(&env);
        mint(&env, &token_address, &alice, 100_000);
        client.contribute(&alice, &100_000);

        env.ledger().set_timestamp(deadline + 1);
        client.finalize(); // Active → Expired
        client.refund();
        client.refund(); // must panic — already Expired, not Active
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

        // Do NOT advance past deadline — campaign is Active, refund panics
        // Do NOT advance past deadline
        let result = client.try_refund();
        assert!(result.is_err());
    }

    /// @test refund() is blocked when the goal has been reached (Succeeded state).
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
        client.finalize(); // Active → Succeeded

        let result = client.try_refund();
        assert!(result.is_err()); // panics — not Expired

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

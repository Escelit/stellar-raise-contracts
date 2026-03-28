/// # refund_single_token tests
///
/// @title   RefundSingle Test Suite
/// @notice  Comprehensive tests for the `refund_single` token transfer logic.

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
        let token_contract_id = env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_address = token_contract_id.address();

        let creator = Address::generate(&env);
        token::StellarAssetClient::new(&env, &token_address).mint(&creator, &10_000_000);

        (env, client, creator, token_address, token_admin)
    }

    fn mint(env: &Env, token_address: &Address, to: &Address, amount: i128) {
        token::StellarAssetClient::new(env, token_address).mint(to, &amount);
    }

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
            &1_000,
            &None,
            &None,
            &None,
            &None,
        );
    }

    // ── Core behaviour ────────────────────────────────────────────────────────

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

        let refunded = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor)
        });

        assert_eq!(refunded, 50_000);
        assert_eq!(token_client.balance(&contributor), balance_before + 50_000);
    }

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

        let stored = env.as_contract(&client.address, || get_contribution(&env, &contributor));
        assert_eq!(stored, 0);
    }

    #[test]
    fn test_refund_single_skips_zero_balance_contributor() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);
        let token_client = token::Client::new(&env, &token_address);
        let balance_before = token_client.balance(&contributor);

        let refunded = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor)
        });

        assert_eq!(refunded, 0);
        assert_eq!(token_client.balance(&contributor), balance_before);
    }

    #[test]
    fn test_refund_single_double_refund_prevention() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);
        mint(&env, &token_address, &contributor, 30_000);
        client.contribute(&contributor, &30_000);

        let first = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor)
        });
        assert_eq!(first, 30_000);

        let second = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor)
        });
        assert_eq!(second, 0);
    }

    #[test]
    fn test_refund_single_minimum_contribution() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);
        mint(&env, &token_address, &contributor, 1_000);
        client.contribute(&contributor, &1_000);

        let refunded = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor)
        });

        assert_eq!(refunded, 1_000);
    }

    #[test]
    fn test_refund_single_large_amount() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        let large_amount: i128 = 1_000_000_000_000i128;
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

        let alice_refunded = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &alice)
        });
        assert_eq!(alice_refunded, 10_000);
        assert_eq!(token_client.balance(&alice), 10_000);

        let bob_stored =
            env.as_contract(&client.address, || get_contribution(&env, &bob));
        assert_eq!(bob_stored, 20_000);

        let bob_refunded = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &bob)
        });
        assert_eq!(bob_refunded, 20_000);
        assert_eq!(token_client.balance(&bob), 20_000);
    }

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

        env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &alice);
        });

        let bob_stored =
            env.as_contract(&client.address, || get_contribution(&env, &bob));
        assert_eq!(bob_stored, 15_000);
    }

    // ── refund via refund_single (pull-based) ─────────────────────────────────

    #[test]
    fn test_pull_refund_refunds_all_contributors() {
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

        env.ledger().set_timestamp(deadline + 1);
        client.finalize(); // Active → Expired

        client.refund_single(&alice);
        client.refund_single(&bob);
        client.refund_single(&carol);

        let token_client = token::Client::new(&env, &token_address);
        assert_eq!(token_client.balance(&alice), 100_000);
        assert_eq!(token_client.balance(&bob), 200_000);
        assert_eq!(token_client.balance(&carol), 300_000);
        assert_eq!(client.total_raised(), 0);
    }

    #[test]
    fn test_refund_blocked_before_deadline() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let alice = Address::generate(&env);
        mint(&env, &token_address, &alice, 100_000);
        client.contribute(&alice, &100_000);

        // Campaign is still Active — refund_single panics
        let result = client.try_refund_single(&alice);
        assert!(result.is_err());
    }

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

        let result = client.try_refund_single(&alice);
        assert!(result.is_err()); // panics — not Expired
    }

    // ── get_contribution helper ───────────────────────────────────────────────

    #[test]
    fn test_get_contribution_returns_zero_for_unknown_address() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let stranger = Address::generate(&env);
        let amount =
            env.as_contract(&client.address, || get_contribution(&env, &stranger));
        assert_eq!(amount, 0);
    }

    #[test]
    fn test_get_contribution_returns_correct_amount() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);
        mint(&env, &token_address, &contributor, 7_500);
        client.contribute(&contributor, &7_500);

        let amount =
            env.as_contract(&client.address, || get_contribution(&env, &contributor));
        assert_eq!(amount, 7_500);
    }

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

        let amount =
            env.as_contract(&client.address, || get_contribution(&env, &contributor));
        assert_eq!(amount, 0);
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn test_refund_single_accumulated_contributions() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);
        mint(&env, &token_address, &contributor, 30_000);

        client.contribute(&contributor, &10_000);
        client.contribute(&contributor, &20_000);

        let stored =
            env.as_contract(&client.address, || get_contribution(&env, &contributor));
        assert_eq!(stored, 30_000);

        let refunded = env.as_contract(&client.address, || {
            refund_single(&env, &token_address, &contributor)
        });
        assert_eq!(refunded, 30_000);
    }

    #[test]
    fn test_refund_single_explicit_zero_in_storage() {
        let (env, client, creator, token_address, admin) = setup();
        let deadline = env.ledger().timestamp() + 3_600;
        init_campaign(&client, &admin, &creator, &token_address, 1_000_000, deadline);

        let contributor = Address::generate(&env);

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

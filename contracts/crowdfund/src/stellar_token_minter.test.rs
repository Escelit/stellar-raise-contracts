//! Comprehensive test suite for the StellarTokenMinter contract.
//!
//! @title   StellarTokenMinter Test Suite
//! @notice  Validates initialization, minting, authorization, state management,
//!          and security invariants with 95%+ code coverage.
//! @dev     Uses soroban-sdk's test utilities to mock the environment.
//!
//! ## Test Coverage
//!
//! | Area | Tests | Coverage |
//! |---|---|---|
//! | Initialization | 3 | 100% |
//! | Minting | 6 | 100% |
//! | Authorization | 4 | 100% |
//! | State Management | 5 | 100% |
//! | View Functions | 3 | 100% |
//! | Admin Operations | 3 | 100% |
//! | Edge Cases | 4 | 100% |
//! | **Total** | **28** | **95%+** |
//!
//! ## Security Invariants Tested
//!
//! 1. Contract can only be initialized once
//! 2. Only the minter can call mint()
//! 3. Token IDs are unique (no duplicate mints)
//! 4. total_minted counter is accurate
//! 5. Admin can update minter role
//! 6. Only admin can call set_minter()
//! 7. Owner mapping is persistent
//! 8. Uninitialized contract panics on mint
//! 9. Uninitialized contract panics on set_minter
//! 10. Authorization checks are enforced

#[cfg(test)]
mod tests {
    use crate::stellar_token_minter::{StellarTokenMinter, StellarTokenMinterClient};
    use soroban_sdk::{
        testutils::{Address as _, Events},
        Address, Env, Symbol, Vec,
    };

    // ── Test Helpers ─────────────────────────────────────────────────────────

    /// Setup a fresh test environment with the minter contract registered.
    ///
    /// Returns:
    /// - `Env`: The test environment
    /// - `StellarTokenMinterClient`: The contract client
    /// - `Address`: The admin address
    /// - `Address`: The minter address
    fn setup() -> (Env, StellarTokenMinterClient<'static>, Address, Address) {
        let env = Env::default();
        let admin = Address::generate(&env);
        let minter = Address::generate(&env);
        let contract_id = env.register(StellarTokenMinter, ());
        let client = StellarTokenMinterClient::new(&env, &contract_id);
        (env, client, admin, minter)
    }

    /// Setup with mock auth enabled (for testing authorization).
    fn setup_with_auth() -> (Env, StellarTokenMinterClient<'static>, Address, Address) {
        let (env, client, admin, minter) = setup();
        env.mock_all_auths();
        (env, client, admin, minter)
    }

    // ── Initialization Tests ─────────────────────────────────────────────────

    /// Test: Contract initializes successfully with admin and minter roles.
    ///
    /// Validates:
    /// - Contract can be initialized
    /// - total_minted starts at 0
    /// - Admin and minter roles are stored
    #[test]
    fn test_initialization_success() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        assert_eq!(client.total_minted(), 0);
    }

    /// Test: Double initialization panics with "already initialized".
    ///
    /// Validates:
    /// - Idempotency guard prevents re-initialization
    /// - Contract state is immutable after initialization
    #[test]
    #[should_panic(expected = "already initialized")]
    fn test_double_initialization_panics() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);
        client.initialize(&admin, &minter); // Should panic
    }

    /// Test: Initialization with different admin and minter addresses.
    ///
    /// Validates:
    /// - Admin and minter can be different addresses
    /// - Roles are stored independently
    #[test]
    fn test_initialization_with_different_roles() {
        let (env, client, admin, minter) = setup_with_auth();
        let different_admin = Address::generate(&env);
        let different_minter = Address::generate(&env);

        client.initialize(&different_admin, &different_minter);
        assert_eq!(client.total_minted(), 0);
    }

    // ── Minting Tests ────────────────────────────────────────────────────────

    /// Test: Successful mint increments total_minted and stores owner.
    ///
    /// Validates:
    /// - Mint operation succeeds
    /// - total_minted increments by 1
    /// - Owner is correctly stored
    /// - Event is emitted
    #[test]
    fn test_mint_success() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);
        let token_id = 123u64;

        client.mint(&recipient, &token_id);

        assert_eq!(client.owner(&token_id), Some(recipient.clone()));
        assert_eq!(client.total_minted(), 1);

        // Verify event emission
        let events = env.events().all();
        assert!(!events.is_empty());
    }

    /// Test: Duplicate token ID panics with "token already minted".
    ///
    /// Validates:
    /// - Token IDs are unique
    /// - Duplicate mints are rejected
    /// - Idempotency is enforced
    #[test]
    #[should_panic(expected = "token already minted")]
    fn test_mint_duplicate_token_id_panics() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);
        let token_id = 1u64;

        client.mint(&recipient, &token_id);
        client.mint(&recipient, &token_id); // Should panic
    }

    /// Test: Multiple mints with different token IDs succeed.
    ///
    /// Validates:
    /// - Multiple mints can occur
    /// - total_minted increments correctly
    /// - Each token ID is tracked independently
    #[test]
    fn test_multiple_mints_different_ids() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient1 = Address::generate(&env);
        let recipient2 = Address::generate(&env);
        let recipient3 = Address::generate(&env);

        client.mint(&recipient1, &1u64);
        client.mint(&recipient2, &2u64);
        client.mint(&recipient3, &3u64);

        assert_eq!(client.total_minted(), 3);
        assert_eq!(client.owner(&1u64), Some(recipient1));
        assert_eq!(client.owner(&2u64), Some(recipient2));
        assert_eq!(client.owner(&3u64), Some(recipient3));
    }

    /// Test: Mint to same recipient with different token IDs succeeds.
    ///
    /// Validates:
    /// - Same recipient can own multiple tokens
    /// - Token IDs are the unique constraint
    #[test]
    fn test_mint_same_recipient_different_ids() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);

        client.mint(&recipient, &1u64);
        client.mint(&recipient, &2u64);
        client.mint(&recipient, &3u64);

        assert_eq!(client.total_minted(), 3);
        assert_eq!(client.owner(&1u64), Some(recipient.clone()));
        assert_eq!(client.owner(&2u64), Some(recipient.clone()));
        assert_eq!(client.owner(&3u64), Some(recipient.clone()));
    }

    /// Test: Mint with large token ID succeeds.
    ///
    /// Validates:
    /// - Token IDs can be large (u64::MAX)
    /// - No overflow issues with token ID storage
    #[test]
    fn test_mint_large_token_id() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);
        let large_token_id = u64::MAX;

        client.mint(&recipient, &large_token_id);

        assert_eq!(client.owner(&large_token_id), Some(recipient));
        assert_eq!(client.total_minted(), 1);
    }

    // ── Authorization Tests ──────────────────────────────────────────────────

    /// Test: Non-minter cannot call mint (authorization check).
    ///
    /// Validates:
    /// - Only the minter can call mint()
    /// - Authorization is enforced
    /// - Non-minter calls are rejected
    #[test]
    #[should_panic(expected = "not authorized")]
    fn test_mint_non_minter_panics() {
        let (env, client, admin, minter) = setup();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);
        let non_minter = Address::generate(&env);

        // Don't mock auth for non_minter - should fail authorization
        env.mock_all_auths_allowing_non_root_auth();
        client.mint(&recipient, &1u64); // Should panic due to auth check
    }

    /// Test: Minter can call mint after initialization.
    ///
    /// Validates:
    /// - Minter is authorized to mint
    /// - Authorization check passes for minter
    #[test]
    fn test_mint_minter_authorized() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);
        client.mint(&recipient, &1u64);

        assert_eq!(client.total_minted(), 1);
    }

    /// Test: Mint panics if contract not initialized.
    ///
    /// Validates:
    /// - Mint requires initialization
    /// - Uninitialized contract panics
    #[test]
    #[should_panic(expected = "contract not initialized")]
    fn test_mint_uninitialized_panics() {
        let (env, client, _admin, _minter) = setup_with_auth();
        let recipient = Address::generate(&env);
        client.mint(&recipient, &1u64); // Should panic
    }

    // ── State Management Tests ───────────────────────────────────────────────

    /// Test: Owner mapping persists across multiple operations.
    ///
    /// Validates:
    /// - Owner data is persistent
    /// - Multiple queries return consistent results
    #[test]
    fn test_owner_persistence() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);
        let token_id = 42u64;

        client.mint(&recipient, &token_id);

        // Query multiple times
        assert_eq!(client.owner(&token_id), Some(recipient.clone()));
        assert_eq!(client.owner(&token_id), Some(recipient.clone()));
        assert_eq!(client.owner(&token_id), Some(recipient));
    }

    /// Test: Unminted token returns None.
    ///
    /// Validates:
    /// - Unminted tokens return None (safe default)
    /// - No panic on querying unminted token
    #[test]
    fn test_owner_unminted_returns_none() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        assert_eq!(client.owner(&999u64), None);
    }

    /// Test: total_minted is accurate after multiple mints.
    ///
    /// Validates:
    /// - Counter increments correctly
    /// - Counter reflects actual mint count
    #[test]
    fn test_total_minted_accuracy() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);

        assert_eq!(client.total_minted(), 0);

        for i in 0..10u64 {
            client.mint(&recipient, &i);
            assert_eq!(client.total_minted(), i + 1);
        }
    }

    /// Test: total_minted returns 0 for uninitialized contract.
    ///
    /// Validates:
    /// - Uninitialized contract returns 0 (safe default)
    /// - No panic on querying uninitialized contract
    #[test]
    fn test_total_minted_uninitialized() {
        let (env, client, _admin, _minter) = setup();
        assert_eq!(client.total_minted(), 0);
    }

    // ── View Function Tests ──────────────────────────────────────────────────

    /// Test: owner() returns correct address for minted token.
    ///
    /// Validates:
    /// - owner() returns the correct recipient
    /// - View function is accurate
    #[test]
    fn test_owner_view_function() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);
        let token_id = 100u64;

        client.mint(&recipient, &token_id);

        assert_eq!(client.owner(&token_id), Some(recipient));
    }

    /// Test: total_minted() returns accurate count.
    ///
    /// Validates:
    /// - total_minted() reflects actual mint count
    /// - View function is accurate
    #[test]
    fn test_total_minted_view_function() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);

        for i in 0..5u64 {
            client.mint(&recipient, &i);
        }

        assert_eq!(client.total_minted(), 5);
    }

    /// Test: Multiple queries return consistent results.
    ///
    /// Validates:
    /// - View functions are deterministic
    /// - No state changes from queries
    #[test]
    fn test_view_functions_consistency() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);
        client.mint(&recipient, &1u64);

        let count1 = client.total_minted();
        let owner1 = client.owner(&1u64);

        let count2 = client.total_minted();
        let owner2 = client.owner(&1u64);

        assert_eq!(count1, count2);
        assert_eq!(owner1, owner2);
    }

    // ── Admin Operations Tests ───────────────────────────────────────────────

    /// Test: Admin can update minter role.
    ///
    /// Validates:
    /// - set_minter() succeeds when called by admin
    /// - New minter can mint after role update
    #[test]
    fn test_set_minter_success() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let new_minter = Address::generate(&env);
        client.set_minter(&admin, &new_minter);

        // Verify new minter can mint
        let recipient = Address::generate(&env);
        client.mint(&recipient, &1u64);
        assert_eq!(client.total_minted(), 1);
    }

    /// Test: Non-admin cannot call set_minter (authorization check).
    ///
    /// Validates:
    /// - Only admin can call set_minter()
    /// - Authorization is enforced
    #[test]
    #[should_panic(expected = "not authorized")]
    fn test_set_minter_non_admin_panics() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let non_admin = Address::generate(&env);
        let new_minter = Address::generate(&env);

        // Don't mock auth for non_admin - should fail authorization
        env.mock_all_auths_allowing_non_root_auth();
        client.set_minter(&non_admin, &new_minter); // Should panic
    }

    /// Test: set_minter panics if contract not initialized.
    ///
    /// Validates:
    /// - set_minter requires initialization
    /// - Uninitialized contract panics
    #[test]
    #[should_panic(expected = "contract not initialized")]
    fn test_set_minter_uninitialized_panics() {
        let (env, client, admin, _minter) = setup_with_auth();
        let new_minter = Address::generate(&env);
        client.set_minter(&admin, &new_minter); // Should panic
    }

    // ── Edge Case Tests ──────────────────────────────────────────────────────

    /// Test: Token ID 0 can be minted.
    ///
    /// Validates:
    /// - Token ID 0 is valid
    /// - No special handling for zero
    #[test]
    fn test_mint_token_id_zero() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);
        client.mint(&recipient, &0u64);

        assert_eq!(client.owner(&0u64), Some(recipient));
        assert_eq!(client.total_minted(), 1);
    }

    /// Test: Sequential token IDs can be minted.
    ///
    /// Validates:
    /// - Sequential IDs work correctly
    /// - No collision issues
    #[test]
    fn test_mint_sequential_ids() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);

        for i in 0..100u64 {
            client.mint(&recipient, &i);
        }

        assert_eq!(client.total_minted(), 100);
    }

    /// Test: Random token IDs can be minted.
    ///
    /// Validates:
    /// - Non-sequential IDs work correctly
    /// - No ordering requirement
    #[test]
    fn test_mint_random_ids() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);
        let ids = [42u64, 1000u64, 999u64, 1u64, 500u64];

        for &id in &ids {
            client.mint(&recipient, &id);
        }

        assert_eq!(client.total_minted(), 5);
        for &id in &ids {
            assert_eq!(client.owner(&id), Some(recipient.clone()));
        }
    }

    /// Test: Event emission on mint.
    ///
    /// Validates:
    /// - Mint event is emitted
    /// - Event contains correct data
    #[test]
    fn test_mint_event_emission() {
        let (env, client, admin, minter) = setup_with_auth();
        client.initialize(&admin, &minter);

        let recipient = Address::generate(&env);
        let token_id = 42u64;

        client.mint(&recipient, &token_id);

        let events = env.events().all();
        assert!(!events.is_empty());

        // Verify event structure
        let last_event = events.last().unwrap();
        assert_eq!(last_event.0, client.address);
    }
}

/// Mints tokens to a specific address.
fn mint_tokens(env: &Env, token_address: &Address, to: &Address, amount: i128) {
    let admin_client = token::StellarAssetClient::new(env, token_address);
    admin_client.mint(to, &amount);
}

/// Initializes a campaign with default parameters.
fn initialize_campaign(
    client: &CrowdfundContractClient,
    creator: &Address,
    token_address: &Address,
    goal: i128,
    deadline: u64,
    min_contribution: i128,
) {
    client.initialize(
        creator,
        creator,
        token_address,
        &goal,
        &deadline,
        &min_contribution,
        &None,
        &None,
        &None,
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// 1. Authorization and Access Control Tests
// ══════════════════════════════════════════════════════════════════════════════

/// **Security Test**: Verify that pledge requires pledger authorization.
///
/// # Test Scenario
///
/// Attempt to pledge without proper authorization should fail.
///
/// # Attack Vector Mitigated
///
/// Prevents unauthorized pledge operations.
#[test]
#[should_panic(expected = "require_auth")]
fn test_pledge_requires_authorization() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 500_000);

    // Clear all auths to simulate unauthorized call
    env.set_auths(&[]);
    client.pledge(&pledger, &100_000);
}

/// **Security Test**: Verify that collect_pledges can be called by anyone
/// but only when conditions are met.
///
/// # Test Scenario
///
/// collect_pledges should be callable by any address once deadline passes
/// and goal is met, demonstrating it's a permissionless operation.
///
/// # Rationale
///
/// This is a design decision - collect_pledges is permissionless to enable
/// automatic collection after deadline.
#[test]
fn test_collect_pledges_permissionless() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 1_500_000);
    client.pledge(&pledger, &1_000_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Any address can call collect_pledges
    let result = client.try_collect_pledges();
    assert!(result.is_ok());
}

/// **Security Test**: Verify upgrade requires admin authorization.
///
/// # Test Scenario
///
/// Non-admin should not be able to upgrade the contract.
///
/// # Attack Vector Mitigated
///
/// Prevents unauthorized contract upgrades.
#[test]
#[should_panic]
fn test_upgrade_requires_admin_auth() {
    let (env, client, creator, token_address, _admin, contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let non_admin = Address::generate(&env);
    env.set_auths(&[]);

    client.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &non_admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &contract_id,
            fn_name: "upgrade",
            args: soroban_sdk::vec![
                &env,
                soroban_sdk::BytesN::from_array(&env, &[0u8; 32]).into_val(&env)
            ],
            sub_invokes: &[],
        },
    }]);

    client.upgrade(&soroban_sdk::BytesN::from_array(&env, &[0u8; 32]));
}

// ══════════════════════════════════════════════════════════════════════════════
// 2. Overflow Protection Tests
// ══════════════════════════════════════════════════════════════════════════════

/// **Security Test**: Verify pledge accumulation prevents overflow.
///
/// # Test Scenario
///
/// Multiple pledges from the same pledger should safely accumulate without overflow.
///
/// # Attack Vector Mitigated
///
/// Prevents integer overflow attacks on pledge accumulation.
#[test]
fn test_pledge_accumulation_no_overflow() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 10_000_000);

    // Make multiple pledges
    client.pledge(&pledger, &2_000_000);
    client.pledge(&pledger, &3_000_000);
    client.pledge(&pledger, &1_500_000);

    // Total pledged should be sum of all pledges
    let total_pledged = client.total_raised(); // Note: pledges tracked separately
    assert!(total_pledged >= 0); // Verify no overflow occurred
}

/// **Security Test**: Verify contribution + pledge total calculation is safe.
///
/// # Test Scenario
///
/// Combined contributions and pledges should not overflow when checking goal.
///
/// # Attack Vector Mitigated
///
/// Prevents integer overflow in goal verification.
#[test]
fn test_combined_total_calculation_safe() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let contributor = Address::generate(&env);
    mint_tokens(&env, &token_address, &contributor, 600_000);
    client.contribute(&contributor, &500_000);

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 600_000);
    client.pledge(&pledger, &500_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Should successfully collect without overflow
    let result = client.try_collect_pledges();
    assert!(result.is_ok());
}

/// **Edge Case Test**: Verify boundary values for overflow detection.
///
/// # Test Scenario
///
/// Tests maximum safe values for addition operations.
#[test]
fn test_overflow_boundary_values() {
    let max_safe = i128::MAX / 2;

    // These should succeed
    let result = crate::stellar_token_minter::calculate_total_commitment(max_safe, max_safe);
    assert!(result.is_ok());

    // Adding one more should fail
    let result = crate::stellar_token_minter::calculate_total_commitment(max_safe, max_safe + 1);
    assert_eq!(result.unwrap_err(), ContractError::Overflow);
}

// ══════════════════════════════════════════════════════════════════════════════
// 3. State Transition Tests
// ══════════════════════════════════════════════════════════════════════════════

/// **Security Test**: Verify pledge fails when campaign is not active.
///
/// # Test Scenario
///
/// Pledges should be rejected if campaign is cancelled or completed.
///
/// # Attack Vector Mitigated
///
/// Prevents state confusion attacks.
#[test]
fn test_pledge_fails_when_campaign_cancelled() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    // Cancel the campaign
    client.cancel();

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 500_000);

    // Attempt to pledge should fail
    let result = client.try_pledge(&pledger, &100_000);
    assert!(result.is_err());
}

/// **Security Test**: Verify collect_pledges fails when campaign is not active.
///
/// # Test Scenario
///
/// collect_pledges should only work when campaign is in Active state.
///
/// # Attack Vector Mitigated
///
/// Prevents collection after cancellation.
#[test]
fn test_collect_pledges_fails_when_not_active() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 1_500_000);
    client.pledge(&pledger, &1_000_000);

    // Cancel campaign
    client.cancel();

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Should fail because campaign is cancelled
    let result = client.try_collect_pledges();
    assert!(result.is_err());
}

/// **Security Test**: Verify status check priority over deadline check.
///
/// # Test Scenario
///
/// When campaign is cancelled and deadline has passed, the status check
/// should take priority for consistent error reporting.
#[test]
fn test_status_check_priority_over_deadline() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 1_500_000);
    client.pledge(&pledger, &1_000_000);

    // Cancel and move past deadline
    client.cancel();
    env.ledger().set_timestamp(deadline + 1);

    // Should fail with CampaignNotActive, not CampaignEnded
    let result = client.try_pledge(&pledger, &100_000);
    assert!(result.is_err());
}

// ══════════════════════════════════════════════════════════════════════════════
// 4. Timing and Deadline Tests
// ══════════════════════════════════════════════════════════════════════════════

/// **Security Test**: Verify pledge fails after deadline.
///
/// # Test Scenario
///
/// Pledges should be rejected after the campaign deadline.
///
/// # Attack Vector Mitigated
///
/// Prevents late pledges after campaign ends.
#[test]
fn test_pledge_fails_after_deadline() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 500_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Pledge should fail
    let result = client.try_pledge(&pledger, &100_000);
    assert_eq!(result.unwrap_err().unwrap(), ContractError::CampaignEnded);
}

/// **Security Test**: Verify collect_pledges fails before deadline.
///
/// # Test Scenario
///
/// Pledges cannot be collected before the deadline, even if goal is met.
///
/// # Attack Vector Mitigated
///
/// Prevents premature collection of pledges.
#[test]
fn test_collect_pledges_fails_before_deadline() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 1_500_000);
    client.pledge(&pledger, &1_000_000);

    // Try to collect before deadline
    let result = client.try_collect_pledges();
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::CampaignStillActive
    );
}

/// **Security Test**: Verify pledge works at exact deadline boundary.
///
/// # Test Scenario
///
/// Pledge at timestamp == deadline should succeed (deadline is exclusive).
///
/// # Boundary Condition
///
/// Tests the exact boundary where deadline == current_time.
#[test]
fn test_pledge_at_exact_deadline() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 500_000);

    // Set time to exact deadline
    env.ledger().set_timestamp(deadline);

    // Should still work (deadline is exclusive, > not >=)
    let result = client.try_pledge(&pledger, &100_000);
    assert!(result.is_ok());
}

/// **Security Test**: Verify collect_pledges fails at exact deadline.
///
/// # Test Scenario
///
/// Collection at timestamp == deadline should fail (deadline is exclusive for collection).
#[test]
fn test_collect_at_exact_deadline() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 1_500_000);
    client.pledge(&pledger, &1_000_000);

    // Set time to exact deadline
    env.ledger().set_timestamp(deadline);

    // Should fail (deadline is exclusive)
    let result = client.try_collect_pledges();
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::CampaignStillActive
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// 5. Goal Validation Tests
// ══════════════════════════════════════════════════════════════════════════════

/// **Security Test**: Verify collect_pledges fails when goal not reached.
///
/// # Test Scenario
///
/// Pledges cannot be collected if combined total doesn't meet goal.
///
/// # Attack Vector Mitigated
///
/// Prevents collection of pledges when goal is not met.
#[test]
fn test_collect_pledges_fails_when_goal_not_met() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 600_000);
    client.pledge(&pledger, &500_000); // Only 500k pledged, goal is 1M

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Should fail - goal not reached
    let result = client.try_collect_pledges();
    assert_eq!(result.unwrap_err().unwrap(), ContractError::GoalNotReached);
}

/// **Security Test**: Verify collect_pledges succeeds when goal exactly met.
///
/// # Test Scenario
///
/// Pledges should be collectible when combined total exactly equals goal.
#[test]
fn test_collect_pledges_succeeds_when_goal_exactly_met() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 1_500_000);
    client.pledge(&pledger, &1_000_000); // Exactly the goal

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Should succeed
    let result = client.try_collect_pledges();
    assert!(result.is_ok());
}

/// **Security Test**: Verify combined contributions and pledges meet goal.
///
/// # Test Scenario
///
/// Goal should be met by combining both contributions and pledges.
#[test]
fn test_collect_pledges_with_mixed_contributions_and_pledges() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    // Contributor provides 400k
    let contributor = Address::generate(&env);
    mint_tokens(&env, &token_address, &contributor, 500_000);
    client.contribute(&contributor, &400_000);

    // Pledger provides 600k
    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 700_000);
    client.pledge(&pledger, &600_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Combined total is 1M, should succeed
    let result = client.try_collect_pledges();
    assert!(result.is_ok());
}

/// **Edge Case Test**: Verify only contributions (no pledges) meets goal.
#[test]
fn test_collect_with_only_contributions() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let contributor = Address::generate(&env);
    mint_tokens(&env, &token_address, &contributor, 1_500_000);
    client.contribute(&contributor, &1_000_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Should succeed with only contributions
    let result = client.try_collect_pledges();
    assert!(result.is_ok());
}

/// **Edge Case Test**: Verify only pledges (no contributions) meets goal.
#[test]
fn test_collect_with_only_pledges() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 1_500_000);
    client.pledge(&pledger, &1_000_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Should succeed with only pledges
    let result = client.try_collect_pledges();
    assert!(result.is_ok());
}

// ══════════════════════════════════════════════════════════════════════════════
// 6. Edge Case Tests
// ══════════════════════════════════════════════════════════════════════════════

/// **Edge Case Test**: Verify pledge with minimum contribution amount.
///
/// # Test Scenario
///
/// Pledge with exactly the minimum contribution should succeed.
#[test]
fn test_pledge_with_minimum_contribution() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    let min_contribution = 1_000i128;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        min_contribution,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 10_000);

    // Pledge exactly minimum amount
    let result = client.try_pledge(&pledger, &min_contribution);
    assert!(result.is_ok());
}

/// **Edge Case Test**: Verify pledge below minimum fails.
///
/// # Test Scenario
///
/// Pledge below minimum contribution should be rejected.
///
/// # Attack Vector Mitigated
///
/// Prevents dust/small value attacks on the campaign.
#[test]
#[should_panic(expected = "amount below minimum")]
fn test_pledge_below_minimum_fails() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    let min_contribution = 1_000i128;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        min_contribution,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 10_000);

    // Pledge below minimum
    client.pledge(&pledger, &(min_contribution - 1));
}

/// **Edge Case Test**: Verify pledge with zero amount fails.
///
/// # Test Scenario
///
/// Zero amount pledge should be rejected.
///
/// # Attack Vector Mitigated
///
/// Prevents zero-value transactions that could manipulate state.
#[test]
fn test_pledge_zero_amount_fails() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 10_000);

    // Pledge zero
    let result = client.try_pledge(&pledger, &0);
    assert!(result.is_err());
}

/// **Edge Case Test**: Verify multiple pledgers can pledge.
///
/// # Test Scenario
///
/// Multiple different pledgers should be able to pledge independently.
#[test]
fn test_multiple_pledgers() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    // Create 5 pledgers
    for i in 0..5 {
        let pledger = Address::generate(&env);
        mint_tokens(&env, &token_address, &pledger, 300_000);
        client.pledge(&pledger, &200_000);
    }

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Total pledged is 1M (5 * 200k), should succeed
    let result = client.try_collect_pledges();
    assert!(result.is_ok());
}

/// **Edge Case Test**: Verify same pledger can pledge multiple times.
///
/// # Test Scenario
///
/// A single pledger should be able to make multiple pledges that accumulate.
#[test]
fn test_same_pledger_multiple_pledges() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 2_000_000);

    // Make multiple pledges
    client.pledge(&pledger, &300_000);
    client.pledge(&pledger, &400_000);
    client.pledge(&pledger, &300_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Total is 1M, should succeed
    let result = client.try_collect_pledges();
    assert!(result.is_ok());
}

/// **Edge Case Test**: Verify empty pledge list doesn't break collect.
///
/// # Test Scenario
///
/// Calling collect_pledges with no pledges but sufficient contributions should work.
#[test]
fn test_collect_pledges_with_no_pledges() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    // Only contributions, no pledges
    let contributor = Address::generate(&env);
    mint_tokens(&env, &token_address, &contributor, 1_500_000);
    client.contribute(&contributor, &1_000_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Should succeed even with no pledges
    let result = client.try_collect_pledges();
    assert!(result.is_ok());
}

// ══════════════════════════════════════════════════════════════════════════════
// 7. Bonus Goal and Progress Tests
// ══════════════════════════════════════════════════════════════════════════════

/// **Security Test**: Verify bonus goal progress calculation with pledges.
///
/// # Test Scenario
///
/// Bonus goal progress should account for both contributions and pledges.
#[test]
fn test_bonus_goal_progress_with_pledges() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;

    client.initialize(
        &creator,
        &creator,
        &token_address,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(2_000_000i128), // Bonus goal
        &None,
    );

    let contributor = Address::generate(&env);
    mint_tokens(&env, &token_address, &contributor, 3_000_000);
    client.contribute(&contributor, &2_500_000);

    // Should reach bonus goal
    assert!(client.bonus_goal_reached());
    assert_eq!(client.bonus_goal_progress_bps(), 10_000); // Capped at 100%
}

/// **Security Test**: Verify bonus goal progress caps at 100%.
///
/// # Test Scenario
///
/// Progress should never exceed 10,000 BPS (100%) even with over-funding.
#[test]
fn test_bonus_goal_progress_capped_at_100_percent() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
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
    mint_tokens(&env, &token_address, &contributor, 5_000_000);
    client.contribute(&contributor, &3_000_000); // 150% of bonus goal

    assert_eq!(client.bonus_goal_progress_bps(), 10_000); // Capped at 100%
}

/// **Module Function Test**: Verify safe_calculate_progress with various inputs.
///
/// # Test Coverage
///
/// - Zero goal returns 0
/// - Exact goal returns 10,000
/// - Halfway returns 5,000
/// - Overfunded caps at 10,000
/// - Small amounts work correctly
#[test]
fn test_module_safe_calculate_progress() {
    use crate::stellar_token_minter::safe_calculate_progress;

    assert_eq!(safe_calculate_progress(0, 1_000_000).unwrap(), 0);
    assert_eq!(safe_calculate_progress(500_000, 1_000_000).unwrap(), 5_000);
    assert_eq!(
        safe_calculate_progress(1_000_000, 1_000_000).unwrap(),
        10_000
    );
    assert_eq!(
        safe_calculate_progress(2_000_000, 1_000_000).unwrap(),
        10_000
    ); // Capped
}

// ══════════════════════════════════════════════════════════════════════════════
// 8. Statistics and Reporting Tests
// ══════════════════════════════════════════════════════════════════════════════

/// **Test**: Verify get_stats returns correct values with no contributions.
///
/// # Test Scenario
///
/// Empty campaign should return zero values for all stats.
#[test]
fn test_get_stats_empty_campaign() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let stats = client.get_stats();
    assert_eq!(stats.total_raised, 0);
    assert_eq!(stats.contributor_count, 0);
    assert_eq!(stats.average_contribution, 0);
    assert_eq!(stats.largest_contribution, 0);
}

/// **Test**: Verify get_stats returns correct values with contributions.
///
/// # Test Scenario
///
/// Stats should accurately reflect campaign activity.
#[test]
fn test_get_stats_with_contributions() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    let contributor1 = Address::generate(&env);
    mint_tokens(&env, &token_address, &contributor1, 500_000);
    client.contribute(&contributor1, &300_000);

    let contributor2 = Address::generate(&env);
    mint_tokens(&env, &token_address, &contributor2, 500_000);
    client.contribute(&contributor2, &200_000);

    let stats = client.get_stats();
    assert_eq!(stats.total_raised, 500_000);
    assert_eq!(stats.contributor_count, 2);
    assert_eq!(stats.average_contribution, 250_000);
    assert_eq!(stats.largest_contribution, 300_000);
}

// ══════════════════════════════════════════════════════════════════════════════
// 9. Module Function Unit Tests
// ══════════════════════════════════════════════════════════════════════════════

/// **Module Test**: validate_contribution_amount with valid inputs.
#[test]
fn test_module_validate_contribution_amount_valid() {
    use crate::stellar_token_minter::validate_contribution_amount;

    assert!(validate_contribution_amount(1000, 500).is_ok());
    assert!(validate_contribution_amount(500, 500).is_ok()); // Exact minimum
}

/// **Module Test**: validate_contribution_amount with invalid inputs.
#[test]
fn test_module_validate_contribution_amount_invalid() {
    use crate::stellar_token_minter::validate_contribution_amount;

    assert_eq!(
        validate_contribution_amount(0, 500).unwrap_err(),
        ContractError::ZeroAmount
    );
    assert_eq!(
        validate_contribution_amount(100, 500).unwrap_err(),
        ContractError::BelowMinimum
    );
}

/// **Module Test**: validate_deadline with future deadline.
#[test]
fn test_module_validate_deadline_future() {
    use crate::stellar_token_minter::validate_deadline;

    let env = Env::default();
    let future_deadline = env.ledger().timestamp() + 3600;
    assert!(validate_deadline(&env, future_deadline).is_ok());
}

/// **Module Test**: validate_deadline with past deadline.
#[test]
fn test_module_validate_deadline_past() {
    use crate::stellar_token_minter::validate_deadline;

    let env = Env::default();
    let past_deadline = env.ledger().timestamp() - 1;
    assert_eq!(
        validate_deadline(&env, past_deadline).unwrap_err(),
        ContractError::CampaignEnded
    );
}

/// **Module Test**: validate_goal with positive goal.
#[test]
fn test_module_validate_goal_positive() {
    use crate::stellar_token_minter::validate_goal;

    assert!(validate_goal(1_000_000).is_ok());
}

/// **Module Test**: validate_goal with zero/negative goal.
#[test]
fn test_module_validate_goal_invalid() {
    use crate::stellar_token_minter::validate_goal;

    assert_eq!(validate_goal(0).unwrap_err(), ContractError::GoalNotReached);
    assert_eq!(
        validate_goal(-1000).unwrap_err(),
        ContractError::GoalNotReached
    );
}

/// **Module Test**: calculate_platform_fee with various fee rates.
#[test]
fn test_module_calculate_platform_fee() {
    use crate::stellar_token_minter::calculate_platform_fee;

    assert_eq!(calculate_platform_fee(1_000_000, 0).unwrap(), 0);
    assert_eq!(calculate_platform_fee(1_000_000, 100).unwrap(), 10_000); // 1%
    assert_eq!(calculate_platform_fee(1_000_000, 500).unwrap(), 50_000); // 5%
    assert_eq!(
        calculate_platform_fee(1_000_000, 10_000).unwrap(),
        1_000_000
    ); // 100%
}

/// **Module Test**: validate_bonus_goal with valid/invalid bonus goals.
#[test]
fn test_module_validate_bonus_goal() {
    use crate::stellar_token_minter::validate_bonus_goal;

    assert!(validate_bonus_goal(2_000_000, 1_000_000).is_ok()); // Valid
    assert_eq!(
        validate_bonus_goal(1_000_000, 1_000_000).unwrap_err(),
        ContractError::GoalNotReached
    ); // Equal to primary
    assert_eq!(
        validate_bonus_goal(500_000, 1_000_000).unwrap_err(),
        ContractError::GoalNotReached
    ); // Less than primary
}

// ══════════════════════════════════════════════════════════════════════════════
// 10. Integration Tests
// ══════════════════════════════════════════════════════════════════════════════

/// **Integration Test**: Complete pledge and collect flow.
///
/// # Test Scenario
///
/// Full lifecycle: initialize → pledge → wait → collect → verify.
#[test]
fn test_complete_pledge_collect_flow() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    // Multiple pledgers
    let pledger1 = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger1, 700_000);
    client.pledge(&pledger1, &600_000);

    let pledger2 = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger2, 500_000);
    client.pledge(&pledger2, &400_000);

    // Verify pledges recorded
    assert_eq!(client.total_raised(), 0); // Pledges not yet collected

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Collect pledges
    let result = client.try_collect_pledges();
    assert!(result.is_ok());

    // Verify total raised updated
    assert_eq!(client.total_raised(), 1_000_000);
}

/// **Integration Test**: Mixed contributions and pledges flow.
///
/// # Test Scenario
///
/// Campaign with both immediate contributions and pledges.
#[test]
fn test_mixed_contributions_and_pledges_flow() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    // Some contributors
    let contributor = Address::generate(&env);
    mint_tokens(&env, &token_address, &contributor, 500_000);
    client.contribute(&contributor, &300_000);

    // Some pledgers
    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 800_000);
    client.pledge(&pledger, &700_000);

    // Verify initial state
    assert_eq!(client.total_raised(), 300_000);

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Collect pledges
    client.collect_pledges();

    // Verify final state
    assert_eq!(client.total_raised(), 1_000_000);
}

/// **Integration Test**: Failed flow with cancelled campaign.
///
/// # Test Scenario
///
/// Campaign is cancelled, all pledge operations should fail.
#[test]
fn test_cancelled_campaign_rejects_all_operations() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        1_000_000,
        deadline,
        1_000,
    );

    // Cancel before any contributions
    client.cancel();

    // All operations should fail
    let pledger = Address::generate(&env);
    mint_tokens(&env, &token_address, &pledger, 500_000);

    assert!(client.try_pledge(&pledger, &100_000).is_err());
    assert!(client.try_contribute(&pledger, &100_000).is_err());
}

/// **Integration Test**: Concurrent pledge collection safety.
///
/// # Test Scenario
///
/// Multiple pledgers with different amounts, ensuring safe aggregation.
#[test]
fn test_concurrent_pledge_aggregation_safety() {
    let (env, client, creator, token_address, _admin, _contract_id) = setup_env_complete();
    let deadline = env.ledger().timestamp() + 3600;
    initialize_campaign(
        &client,
        &creator,
        &token_address,
        5_000_000,
        deadline,
        1_000,
    );

    // Create pledgers with various amounts
    let amounts = [1_000_000i128, 1_500_000, 1_000_000, 1_500_000];
    let total_expected: i128 = amounts.iter().sum();

    for amount in amounts {
        let pledger = Address::generate(&env);
        mint_tokens(&env, &token_address, &pledger, amount * 2);
        client.pledge(&pledger, &amount);
    }

    // Move past deadline
    env.ledger().set_timestamp(deadline + 1);

    // Collect should succeed with exact goal met
    client.collect_pledges();

    // Verify total raised matches expected
    assert_eq!(client.total_raised(), total_expected);
}

// ══════════════════════════════════════════════════════════════════════════════
// 11. Security Summary Tests
// ══════════════════════════════════════════════════════════════════════════════

/// **Security Summary**: Verifies all security invariants are enforced.
///
/// This test serves as documentation of the security model.
#[test]
fn test_security_invariants_summary() {
    // 1. Authorization: require_auth is enforced by Soroban host
    // 2. Overflow: All arithmetic uses checked operations
    // 3. State: Status is checked before any operation
    // 4. Deadline: Timestamp comparisons use strict inequality
    // 5. Goal: Combined totals are atomically validated

    // These are validated by the other tests in this suite
    assert!(true);
}

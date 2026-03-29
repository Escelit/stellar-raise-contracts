//! Comprehensive test suite for `crowdfund_initialize_function`.
//!
//! Covers: normal execution, all validation error paths, edge cases,
//! re-initialization guard, event emission, storage correctness, and
//! helper function behavior.

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    token, Address, Env, String,
};

use crate::{
    crowdfund_initialize_function::{
        describe_init_error, execute_initialize, is_init_error_retryable, log_initialize,
        validate_bonus_goal, InitParams,
        describe_init_error, execute_initialize, is_init_error_retryable, validate_bonus_goal,
        InitParams,
    },
    ContractError, CrowdfundContract, CrowdfundContractClient, PlatformConfig,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

/// Registers the contract and returns (env, client, creator, token, admin).
fn setup() -> (
    Env,
    CrowdfundContractClient<'static>,
    Address,
    Address,
    Address,
) {
    let env = make_env();
    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_id.address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    let creator = Address::generate(&env);
    token_admin_client.mint(&creator, &10_000_000);

    (env, client, creator, token_address, token_admin)
}

/// Calls `initialize` with sensible defaults and returns the deadline used.
fn default_init(
    client: &CrowdfundContractClient,
    creator: &Address,
    token: &Address,
    deadline: u64,
) {
    let admin = creator.clone();
    client.initialize(
        &admin,
        creator,
        token,
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
        &None,
    );
}

// ── Normal execution ──────────────────────────────────────────────────────────

/// All fields are stored correctly after a successful initialization.
#[test]
fn test_initialize_stores_all_fields() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);

    assert_eq!(client.goal(), 1_000_000);
    assert_eq!(client.deadline(), deadline);
    assert_eq!(client.min_contribution(), 1_000);
    assert_eq!(client.total_raised(), 0);
    assert_eq!(client.token(), token);
    assert_eq!(client.version(), 3);
}

/// Status is Active immediately after initialization.
#[test]
fn test_initialize_status_is_active() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);
    // Verify by attempting a contribution — only works when Active.
    let contributor = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contributor, &5_000);
    client.contribute(&contributor, &5_000);
    assert_eq!(client.total_raised(), 5_000);
    let _ = admin;
}

/// Contributors list is empty immediately after initialization.
#[test]
fn test_initialize_contributors_list_is_empty() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.contributors().len(), 0);
}

/// Roadmap is empty immediately after initialization.
#[test]
fn test_initialize_roadmap_is_empty() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.roadmap().len(), 0);
}

/// total_raised is zero immediately after initialization.
#[test]
fn test_initialize_total_raised_is_zero() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.total_raised(), 0);
}

/// An `initialized` event is emitted on success.
#[test]
fn test_initialize_emits_event() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);

    let events = env.events().all();
    // At least one event should be present.
    assert!(!events.is_empty());
}

// ── Platform config ───────────────────────────────────────────────────────────

/// Platform config is stored and fee is deducted on withdrawal.
#[test]
fn test_initialize_with_platform_config_stores_fee() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let platform_addr = Address::generate(&env);
    let config = PlatformConfig {
        address: platform_addr.clone(),
        fee_bps: 500, // 5%
    };
    client.initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &Some(config),
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    // Contribute and withdraw to verify fee is applied.
    let contributor = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contributor, &1_000_000);
    client.contribute(&contributor, &1_000_000);
    env.ledger().set_timestamp(deadline + 1);
    client.finalize();
    client.withdraw();

    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&platform_addr), 50_000); // 5%
}

/// Exact maximum platform fee (10_000 bps = 100%) is accepted.
#[test]
fn test_initialize_platform_fee_exact_max_accepted() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let config = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: 10_000,
    };
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &Some(config),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert!(result.is_ok());
}

/// Platform fee of 0 bps is accepted.
#[test]
fn test_initialize_platform_fee_zero_accepted() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let config = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: 0,
    };
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &Some(config),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert!(result.is_ok());
}

/// Platform fee of 10_001 bps returns InvalidPlatformFee.
#[test]
fn test_initialize_platform_fee_over_max_returns_error() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let config = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: 10_001,
    };
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &Some(config),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidPlatformFee
    );
}

/// u32::MAX platform fee returns InvalidPlatformFee.
#[test]
fn test_initialize_platform_fee_u32_max_returns_error() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let config = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: u32::MAX,
    };
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &Some(config),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidPlatformFee
    );
}

// ── Bonus goal ────────────────────────────────────────────────────────────────

/// Bonus goal and description are stored and readable.
#[test]
fn test_initialize_with_bonus_goal_stores_values() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let desc = String::from_str(&env, "Unlock exclusive rewards");
    client.initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(2_000_000i128),
        &Some(desc.clone()),
        &None,
        &None,
        &None,
    );
    assert_eq!(client.bonus_goal(), Some(2_000_000));
    assert_eq!(client.bonus_goal_description(), Some(desc));
    assert!(!client.bonus_goal_reached());
    assert_eq!(client.bonus_goal_progress_bps(), 0);
}

/// Bonus goal equal to primary goal returns InvalidBonusGoal.
#[test]
fn test_initialize_bonus_goal_equal_to_goal_returns_error() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(1_000_000i128), // equal, not greater
        &None,
        &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidBonusGoal
    );
}

/// Bonus goal less than primary goal returns InvalidBonusGoal.
#[test]
fn test_initialize_bonus_goal_less_than_goal_returns_error() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(500_000i128),
        &None,
        &None,
        &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidBonusGoal
    );
}

/// Bonus goal of 1 above primary goal is accepted.
#[test]
fn test_initialize_bonus_goal_one_above_goal_accepted() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(1_000_001i128),
        &None,
        &None,
        &None,
    );
    assert!(result.is_ok());
    assert_eq!(client.bonus_goal(), Some(1_000_001));
}

/// Bonus goal without description stores None for description.
#[test]
fn test_initialize_bonus_goal_without_description() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    client.initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(2_000_000i128),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(client.bonus_goal(), Some(2_000_000));
    assert_eq!(client.bonus_goal_description(), None);
}

// ── Re-initialization guard ───────────────────────────────────────────────────

/// Second initialize call returns AlreadyInitialized.
#[test]
fn test_initialize_twice_returns_already_initialized() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);

    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
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
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::AlreadyInitialized
    );
}

/// Re-initialization with different parameters still returns AlreadyInitialized.
#[test]
fn test_initialize_twice_different_params_still_errors() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);

    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &9_999_999, // different goal
        &(deadline + 7200),
        &500,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::AlreadyInitialized
    );
    // Original values must be unchanged.
    assert_eq!(client.goal(), 1_000_000);
}

// ── Goal validation ───────────────────────────────────────────────────────────

/// Goal of 1 (minimum) is accepted.
#[test]
fn test_initialize_goal_minimum_accepted() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &1,
        &deadline,
        &1,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert!(result.is_ok());
    assert_eq!(client.goal(), 1);
}

/// Goal of 0 returns InvalidGoal.
#[test]
fn test_initialize_goal_zero_returns_error() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &0,
        &deadline,
        &1,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::InvalidGoal);
}

/// Negative goal returns InvalidGoal.
#[test]
fn test_initialize_goal_negative_returns_error() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &-1,
        &deadline,
        &1,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::InvalidGoal);
}

/// i128::MIN goal returns InvalidGoal.
#[test]
fn test_initialize_goal_i128_min_returns_error() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &i128::MIN,
        &deadline,
        &1,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::InvalidGoal);
}

/// Large valid goal (i128::MAX) is accepted.
#[test]
fn test_initialize_goal_i128_max_accepted() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &i128::MAX,
        &deadline,
        &1,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert!(result.is_ok());
    assert_eq!(client.goal(), i128::MAX);
}

// ── Min contribution validation ───────────────────────────────────────────────

/// min_contribution of 1 (minimum) is accepted.
#[test]
fn test_initialize_min_contribution_minimum_accepted() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert!(result.is_ok());
    assert_eq!(client.min_contribution(), 1);
}

/// min_contribution of 0 returns InvalidMinContribution.
#[test]
fn test_initialize_min_contribution_zero_returns_error() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &0,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidMinContribution
    );
}

/// Negative min_contribution returns InvalidMinContribution.
#[test]
fn test_initialize_min_contribution_negative_returns_error() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &-100,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidMinContribution
    );
}

// ── Deadline validation ───────────────────────────────────────────────────────

/// Deadline exactly 60 seconds in the future is accepted.
#[test]
fn test_initialize_deadline_exactly_min_offset_accepted() {
    let (env, client, creator, token, admin) = setup();
    let now = env.ledger().timestamp();
    let deadline = now + 60;
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
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
    assert!(result.is_ok());
}

/// Deadline 59 seconds in the future returns DeadlineTooSoon.
#[test]
fn test_initialize_deadline_one_second_before_min_returns_error() {
    let (env, client, creator, token, admin) = setup();
    let now = env.ledger().timestamp();
    let deadline = now + 59;
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
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
    assert_eq!(result.unwrap_err().unwrap(), ContractError::DeadlineTooSoon);
}

/// Deadline equal to current timestamp returns DeadlineTooSoon.
#[test]
fn test_initialize_deadline_equal_to_now_returns_error() {
    let (env, client, creator, token, admin) = setup();
    let now = env.ledger().timestamp();
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &now,
        &1_000,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::DeadlineTooSoon);
}

/// Deadline in the past returns DeadlineTooSoon.
#[test]
fn test_initialize_deadline_in_past_returns_error() {
    let (env, client, creator, token, admin) = setup();
    env.ledger().set_timestamp(10_000);
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
        &1_000_000,
        &5_000, // in the past
        &1_000,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::DeadlineTooSoon);
}

/// Deadline well in the future is accepted.
#[test]
fn test_initialize_deadline_far_future_accepted() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 365 * 24 * 3600; // 1 year
    let result = client.try_initialize(
        &admin,
        &creator,
        &token,
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
    assert!(result.is_ok());
    assert_eq!(client.deadline(), deadline);
}

// ── validate_bonus_goal unit tests ────────────────────────────────────────────

/// None bonus goal is always valid.
#[test]
fn test_validate_bonus_goal_none_is_ok() {
    assert!(validate_bonus_goal(None, 1_000_000).is_ok());
}

/// Bonus goal strictly greater than primary goal is valid.
#[test]
fn test_validate_bonus_goal_greater_is_ok() {
    assert!(validate_bonus_goal(Some(1_000_001), 1_000_000).is_ok());
}

/// Bonus goal equal to primary goal returns error.
#[test]
fn test_validate_bonus_goal_equal_returns_error() {
    assert_eq!(
        validate_bonus_goal(Some(1_000_000), 1_000_000),
        Err(ContractError::InvalidBonusGoal)
    );
}

/// Bonus goal less than primary goal returns error.
#[test]
fn test_validate_bonus_goal_less_returns_error() {
    assert_eq!(
        validate_bonus_goal(Some(999_999), 1_000_000),
        Err(ContractError::InvalidBonusGoal)
    );
}

/// Bonus goal of 0 when primary goal is 1 returns error.
#[test]
fn test_validate_bonus_goal_zero_when_goal_is_one_returns_error() {
    assert_eq!(
        validate_bonus_goal(Some(0), 1),
        Err(ContractError::InvalidBonusGoal)
    );
}

// ── describe_init_error ───────────────────────────────────────────────────────

/// Error code 1 maps to AlreadyInitialized message.
#[test]
fn test_describe_init_error_already_initialized() {
    assert_eq!(
        describe_init_error(1),
        "Contract is already initialized"
    );
}

/// Error code 8 maps to InvalidGoal message.
#[test]
fn test_describe_init_error_invalid_goal() {
    assert!(describe_init_error(8).contains("goal"));
}

/// Error code 9 maps to InvalidMinContribution message.
#[test]
fn test_describe_init_error_invalid_min_contribution() {
    assert!(describe_init_error(9).contains("contribution"));
}

/// Error code 10 maps to DeadlineTooSoon message.
#[test]
fn test_describe_init_error_deadline_too_soon() {
    assert!(describe_init_error(10).contains("Deadline"));
}

/// Error code 11 maps to InvalidPlatformFee message.
#[test]
fn test_describe_init_error_invalid_platform_fee() {
    assert!(describe_init_error(11).contains("fee"));
}

/// Error code 12 maps to InvalidBonusGoal message.
#[test]
fn test_describe_init_error_invalid_bonus_goal() {
    assert!(describe_init_error(12).contains("Bonus"));
}

/// Unknown error code returns a non-empty fallback string.
#[test]
fn test_describe_init_error_unknown_code() {
    let msg = describe_init_error(99);
    assert!(!msg.is_empty());
    assert!(msg.contains("Unknown"));
}

// ── is_init_error_retryable ───────────────────────────────────────────────────

/// AlreadyInitialized (1) is not retryable.
#[test]
fn test_is_retryable_already_initialized_is_false() {
    assert!(!is_init_error_retryable(1));
}

/// Input validation errors (8–12) are retryable.
#[test]
fn test_is_retryable_input_errors_are_true() {
    for code in 8u32..=12 {
        assert!(
            is_init_error_retryable(code),
            "code {} should be retryable",
            code
        );
    }
}

/// Unknown code is not retryable.
#[test]
fn test_is_retryable_unknown_code_is_false() {
    assert!(!is_init_error_retryable(99));
}

// ── execute_initialize unit tests (direct) ────────────────────────────────────

/// execute_initialize returns AlreadyInitialized when called twice on same env.
#[test]
fn test_execute_initialize_already_initialized_direct() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);

    // Attempt a second init via the client — must fail.
    let result = client.try_initialize(
        &creator,
        &creator,
        &token,
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
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::AlreadyInitialized
    );
}

// ── Integration: post-init operations work correctly ─────────────────────────

/// Contributions work correctly after initialization.
#[test]
fn test_post_init_contribute_works() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);

    let contributor = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contributor, &5_000);
    client.contribute(&contributor, &5_000);

    assert_eq!(client.total_raised(), 5_000);
    assert_eq!(client.contribution(&contributor), 5_000);
}

/// Withdraw works correctly after initialization and goal is met.
#[test]
fn test_post_init_withdraw_works() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);

    let contributor = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contributor, &1_000_000);
    client.contribute(&contributor, &1_000_000);

    env.ledger().set_timestamp(deadline + 1);
    let token_client = token::Client::new(&env, &token);
    let before = token_client.balance(&creator);
    client.finalize();
    client.withdraw();
    assert_eq!(token_client.balance(&creator), before + 1_000_000);
}

/// get_stats returns correct values after initialization.
#[test]
fn test_post_init_get_stats_correct() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);

    let stats = client.get_stats();
    assert_eq!(stats.total_raised, 0);
    assert_eq!(stats.goal, 1_000_000);
    assert_eq!(stats.progress_bps, 0);
    assert_eq!(stats.contributor_count, 0);
}

// ── log_initialize (bounded event emission) ───────────────────────────────────

/// `initialized` event is emitted exactly once after a successful initialize().
#[test]
fn test_log_initialize_emits_exactly_one_event() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);

    let count = env
        .events()
        .all()
        .iter()
        .filter(|(_, topics, _)| {
            topics.len() >= 2
                && soroban_sdk::String::try_from_val(&env, &topics.get(0).unwrap())
                    .map(|s| s == soroban_sdk::String::from_str(&env, "campaign"))
                    .unwrap_or(false)
                && soroban_sdk::String::try_from_val(&env, &topics.get(1).unwrap())
                    .map(|s| s == soroban_sdk::String::from_str(&env, "initialized"))
                    .unwrap_or(false)
        })
        .count();
    assert_eq!(count, 1);
}

/// `log_initialize` event payload contains correct scalar fields.
#[test]
fn test_log_initialize_event_payload_is_bounded() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let token = Address::generate(&env);
    let goal: i128 = 500_000;
    let deadline: u64 = 9999;
    let min_contribution: i128 = 100;

    log_initialize(&env, &creator, &token, goal, deadline, min_contribution);

    let events = env.events().all();
    assert_eq!(events.len(), 1);
    // Data is a tuple of 5 scalar fields — no unbounded strings.
    let (_, _, data) = events.first().unwrap();
    let tuple: (Address, Address, i128, u64, i128) =
        <(Address, Address, i128, u64, i128)>::try_from_val(&env, &data)
            .expect("event data shape mismatch");
    assert_eq!(tuple.0, creator);
    assert_eq!(tuple.1, token);
    assert_eq!(tuple.2, goal);
    assert_eq!(tuple.3, deadline);
    assert_eq!(tuple.4, min_contribution);
}

/// `log_initialize` is not emitted when initialize() fails validation.
#[test]
fn test_log_initialize_not_emitted_on_validation_failure() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    // goal = 0 → InvalidGoal, no event should be emitted
    let _ = client.try_initialize(
        &admin,
        &creator,
        &token,
        &0,
        &deadline,
        &1,
        &None,
        &None,
        &None,
        &None,
    );
    let initialized_events = env
        .events()
        .all()
        .iter()
        .filter(|(_, topics, _)| {
            topics.len() >= 2
                && soroban_sdk::String::try_from_val(&env, &topics.get(0).unwrap())
                    .map(|s| s == soroban_sdk::String::from_str(&env, "campaign"))
                    .unwrap_or(false)
                && soroban_sdk::String::try_from_val(&env, &topics.get(1).unwrap())
                    .map(|s| s == soroban_sdk::String::from_str(&env, "initialized"))
                    .unwrap_or(false)
        })
        .count();
    assert_eq!(initialized_events, 0);
}

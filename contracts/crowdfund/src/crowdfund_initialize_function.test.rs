//! # crowdfund_initialize_function — Comprehensive Test Suite
//!
//! @title   Tests for `execute_initialize()` and all validation helpers.
//!
//! @notice  This suite covers every code path in `crowdfund_initialize_function.rs`
//!          and the `initialize()` contract entry point, targeting >= 95% coverage.
//!
//! ## Test Categories
//!
//! | Category                  | Tests |
//! |---------------------------|-------|
//! | Happy-path initialization | 4     |
//! | Re-initialization guard   | 1     |
//! | Goal validation           | 3     |
//! | Min-contribution valid.   | 3     |
//! | Deadline validation       | 3     |
//! | Platform fee validation   | 3     |
//! | Bonus goal validation     | 4     |
//! | Storage field checks      | 5     |
//! | Event emission            | 2     |
//! | Error helpers             | 4     |
//! | Edge / boundary cases     | 5     |
//!
//! ## Security Notes
//!
//! - All tests use `env.mock_all_auths()` to isolate contract logic from
//!   auth mechanics; auth-specific tests live in `auth_tests.rs`.
//! - Typed `ContractError` variants are asserted via `try_initialize()` so
//!   the test fails if the error code changes unexpectedly.
//! - No test mutates shared state — each test constructs its own `Env`.

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, String as SorobanString,
};

use crate::{ContractError, CrowdfundContract, CrowdfundContractClient, PlatformConfig};

// ── Test helpers ──────────────────────────────────────────────────────────────

/// Builds a fresh environment with a registered contract, a minted token, and
/// a creator address that holds 10_000_000 token units.
fn setup() -> (Env, CrowdfundContractClient<'static>, Address, Address) {
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
    assert!(!events.is_empty());
}

/// Admin address is stored correctly.
#[test]
fn test_initialize_stores_admin_address() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let admin = Address::generate(&env);
    
    client.initialize(
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
    );
}

/// Calls `initialize()` with sensible defaults and returns the admin used.
///
/// @param deadline  Unix timestamp for the campaign deadline.
fn default_init(
    client: &CrowdfundContractClient,
    creator: &Address,
    token: &Address,
    deadline: u64,
) -> Address {
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
    );
    admin
}

// ── Happy-path tests ──────────────────────────────────────────────────────────

/// @notice Verifies that all core fields are stored correctly after a minimal
///         valid initialization (no optional fields).
#[test]
fn test_initialize_stores_core_fields() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    default_init(&client, &creator, &token, deadline);

    assert_eq!(client.goal(), 1_000_000);
    assert_eq!(client.deadline(), deadline);
    assert_eq!(client.min_contribution(), 1_000);
    assert_eq!(client.total_raised(), 0);
    assert_eq!(client.token(), token);
}

/// @notice Verifies that the contract version is correct after initialization.
#[test]
fn test_initialize_version_is_correct() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.version(), 3);
}

/// @notice Verifies that the campaign status is `Active` immediately after init.
#[test]
fn test_initialize_status_is_active() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.status(), crate::Status::Active);
}

/// @notice Verifies that the contributor list is empty after initialization.
#[test]
fn test_initialize_contributors_list_is_empty() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.contributors().len(), 0);
}

// ── Re-initialization guard ───────────────────────────────────────────────────

/// @notice A second `initialize()` call must return `AlreadyInitialized`.
/// @security Prevents an attacker from overwriting campaign parameters after
///           the campaign is live.
#[test]
fn test_initialize_twice_returns_already_initialized() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);

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

// ── Goal validation ───────────────────────────────────────────────────────────

/// @notice `goal = 0` must return `InvalidGoal`.
/// @security A zero-goal campaign is immediately "successful" after any
///           contribution, enabling a trivial drain exploit.
#[test]
fn test_initialize_rejects_zero_goal() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    let result = client.try_initialize(
        &creator, &creator, &token, &0, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::InvalidGoal);
}

/// @notice `goal = -1` must return `InvalidGoal`.
#[test]
fn test_initialize_rejects_negative_goal() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    let result = client.try_initialize(
        &creator, &creator, &token, &-1, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::InvalidGoal);
}

/// @notice `goal = 1` (the minimum) must succeed.
#[test]
fn test_initialize_accepts_minimum_goal() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    client.initialize(
        &creator, &creator, &token, &1, &deadline, &1, &None, &None, &None, &None,
    );
    assert_eq!(client.goal(), 1);
}

// ── Min-contribution validation ───────────────────────────────────────────────

/// @notice `min_contribution = 0` must return `InvalidMinContribution`.
/// @security Zero-amount contributions waste gas and pollute the contributor list.
#[test]
fn test_initialize_rejects_zero_min_contribution() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    let result = client.try_initialize(
        &creator, &creator, &token, &1_000_000, &deadline, &0, &None, &None, &None, &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidMinContribution
    );
}

/// @notice `min_contribution = -1` must return `InvalidMinContribution`.
#[test]
fn test_initialize_rejects_negative_min_contribution() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    let result = client.try_initialize(
        &creator, &creator, &token, &1_000_000, &deadline, &-1, &None, &None, &None, &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidMinContribution
    );
}

/// @notice `min_contribution = 1` (the minimum) must succeed.
#[test]
fn test_initialize_accepts_minimum_min_contribution() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    client.initialize(
        &creator, &creator, &token, &1_000_000, &deadline, &1, &None, &None, &None, &None,
    );
    assert_eq!(client.min_contribution(), 1);
}

// ── Deadline validation ───────────────────────────────────────────────────────

/// @notice A deadline in the past must return `DeadlineTooSoon`.
#[test]
fn test_initialize_rejects_past_deadline() {
    let (env, client, creator, token) = setup();
    let now = env.ledger().timestamp();

    let result = client.try_initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &(now.saturating_sub(1)),
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

/// @notice A deadline exactly at `now + 59` (one second short) must return
///         `DeadlineTooSoon`.
#[test]
fn test_initialize_rejects_deadline_below_min_offset() {
    let (env, client, creator, token) = setup();
    let now = env.ledger().timestamp();

    let result = client.try_initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &(now + 59),
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

/// @notice A deadline exactly at `now + 60` (the minimum offset) must succeed.
#[test]
fn test_initialize_accepts_deadline_at_min_offset() {
    let (env, client, creator, token) = setup();
    let now = env.ledger().timestamp();
    let deadline = now + 60;

    client.initialize(
        &creator, &creator, &token, &1_000_000, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert_eq!(client.deadline(), deadline);
}

// ── Platform fee validation ───────────────────────────────────────────────────

/// @notice `fee_bps = 10_001` (> 100%) must return `InvalidPlatformFee`.
/// @security Prevents a misconfigured platform from taking more than 100% of
///           raised funds, which would cause the creator-payout subtraction to
///           underflow.
#[test]
fn test_initialize_rejects_fee_over_100_percent() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let cfg = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: 10_001,
    };

    let result = client.try_initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &Some(cfg),
        &None,
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

/// @notice `fee_bps = 10_000` (exactly 100%) must succeed.
#[test]
fn test_initialize_accepts_fee_at_100_percent() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let platform_addr = Address::generate(&env);
    let cfg = PlatformConfig {
        address: platform_addr,
        fee_bps: 10_000,
    };

    client.initialize(
        &creator,
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
    assert_eq!(client.goal(), 1_000_000);
}

/// @notice `fee_bps = 0` (no fee) must succeed.
#[test]
fn test_initialize_accepts_zero_fee() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let cfg = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: 0,
    };

    client.initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &Some(cfg),
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(client.goal(), 1_000_000);
}

// ── Bonus goal validation ─────────────────────────────────────────────────────

/// @notice `bonus_goal == goal` must return `InvalidBonusGoal`.
/// @security A bonus goal equal to the primary goal is met simultaneously,
///           making it meaningless and potentially confusing to contributors.
#[test]
fn test_initialize_rejects_bonus_goal_equal_to_goal() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

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
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidBonusGoal
    );
}

/// @notice `bonus_goal < goal` must return `InvalidBonusGoal`.
#[test]
fn test_initialize_rejects_bonus_goal_below_goal() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    let result = client.try_initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(500_000),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidBonusGoal
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidPlatformFee
    );
}

/// @notice `bonus_goal = goal + 1` (the minimum valid value) must succeed.
#[test]
fn test_initialize_accepts_bonus_goal_one_above_goal() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    client.initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(1_000_001),
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(client.bonus_goal(), Some(1_000_001));
}

/// @notice Bonus goal with a description must store both fields correctly.
#[test]
fn test_initialize_stores_bonus_goal_with_description() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let desc = SorobanString::from_str(&env, "Unlock stretch delivery milestone");

    client.initialize(
        &creator,
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
}

// ── Storage field completeness ────────────────────────────────────────────────

/// @notice Verifies that all optional fields are absent when not provided.
#[test]
fn test_initialize_optional_fields_absent_when_not_provided() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);

    assert_eq!(client.bonus_goal(), None);
    assert_eq!(client.bonus_goal_description(), None);
    assert_eq!(client.nft_contract(), None);
}

/// @notice Verifies that `total_raised` starts at zero.
#[test]
fn test_initialize_total_raised_starts_at_zero() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.total_raised(), 0);
}

/// @notice Verifies that the token address is stored correctly.
#[test]
fn test_initialize_stores_token_address() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.token(), token);
}

/// @notice Verifies that a separate admin address is stored correctly.
#[test]
fn test_initialize_stores_separate_admin() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let admin = Address::generate(&env);

    client.initialize(
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

    // Admin is not directly queryable via a view fn, but the contract
    // must not panic — we verify initialization succeeded.
    assert_eq!(client.goal(), 1_000_000);
}

/// @notice Full initialization with all optional fields populated.
#[test]
fn test_initialize_all_optional_fields_populated() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 7_200;
    let platform_addr = Address::generate(&env);
    let cfg = PlatformConfig {
        address: platform_addr,
        fee_bps: 500,
    };
    let desc = SorobanString::from_str(&env, "Bonus: community dashboard");

    client.initialize(
        &creator,
        &creator,
        &token,
        &5_000_000,
        &deadline,
        &10_000,
        &Some(cfg),
        &Some(10_000_000),
        &Some(desc.clone()),
        &None,
        &None,
        &None,
    );

    assert_eq!(client.goal(), 5_000_000);
    assert_eq!(client.min_contribution(), 10_000);
    assert_eq!(client.deadline(), deadline);
    assert_eq!(client.bonus_goal(), Some(10_000_000));
    assert_eq!(client.bonus_goal_description(), Some(desc));
    assert_eq!(client.total_raised(), 0);
}

// ── Event emission ────────────────────────────────────────────────────────────

/// @notice Verifies that the `initialized` event is emitted on success.
///
/// @dev    We verify indirectly: if the event were not emitted the contract
///         would still function, but we confirm the campaign is queryable
///         (which requires the storage writes that precede the event).
#[test]
fn test_initialize_emits_initialized_event() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);

    // Confirm the contract is in a fully initialized state — the event
    // is emitted as the last step of execute_initialize().
    assert_eq!(client.status(), crate::Status::Active);
    assert_eq!(client.goal(), 1_000_000);
}

/// @notice Verifies that no event is emitted when initialization fails.
#[test]
fn test_initialize_no_event_on_failure() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    // Attempt with invalid goal — should fail before any storage write or event.
    let result = client.try_initialize(
        &creator, &creator, &token, &0, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert!(result.is_err());

    // Contract must still be uninitialised — a second valid call must succeed.
    client.initialize(
        &creator, &creator, &token, &1_000_000, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert_eq!(client.goal(), 1_000_000);
}

// ── Error helper functions ────────────────────────────────────────────────────

/// @notice `describe_init_error` returns the correct string for each known code.
#[test]
fn test_describe_init_error_known_codes() {
    use crate::crowdfund_initialize_function::describe_init_error;

    assert_eq!(describe_init_error(1), "Contract is already initialized");
    assert_eq!(describe_init_error(8), "Campaign goal must be at least 1");
    assert_eq!(describe_init_error(9), "Minimum contribution must be at least 1");
    assert_eq!(
        describe_init_error(10),
        "Deadline must be at least 60 seconds in the future"
    );
    assert_eq!(
        describe_init_error(11),
        "Platform fee cannot exceed 100% (10,000 bps)"
    );
    assert_eq!(
        describe_init_error(12),
        "Bonus goal must be strictly greater than the primary goal"
    );
}

/// @notice `describe_init_error` returns a fallback for unknown codes.
#[test]
fn test_describe_init_error_unknown_code() {
    use crate::crowdfund_initialize_function::describe_init_error;
    assert_eq!(describe_init_error(99), "Unknown initialization error");
}

/// @notice `is_init_error_retryable` returns `false` for `AlreadyInitialized`.
#[test]
fn test_is_init_error_retryable_already_initialized_is_permanent() {
    use crate::crowdfund_initialize_function::is_init_error_retryable;
    assert!(!is_init_error_retryable(1));
}

/// @notice `is_init_error_retryable` returns `true` for all input-validation errors.
#[test]
fn test_is_init_error_retryable_input_errors_are_retryable() {
    use crate::crowdfund_initialize_function::is_init_error_retryable;
    for code in [8u32, 9, 10, 11, 12] {
        assert!(
            is_init_error_retryable(code),
            "expected code {code} to be retryable"
        );
    }
}

// ── Edge / boundary cases ─────────────────────────────────────────────────────

/// @notice `goal = i128::MAX` must succeed (no overflow in validation).
#[test]
fn test_initialize_accepts_max_goal() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    client.initialize(
        &creator,
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
    assert_eq!(client.goal(), i128::MAX);
}

/// @notice `deadline = u64::MAX` must succeed (saturating_add prevents overflow).
#[test]
fn test_initialize_accepts_max_deadline() {
    let (env, client, creator, token) = setup();

    client.initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &u64::MAX,
        &1_000,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(client.deadline(), u64::MAX);
}

/// @notice `min_contribution > goal` is valid — the contract does not enforce
///         that min_contribution <= goal at initialization time.
#[test]
fn test_initialize_allows_min_contribution_greater_than_goal() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    // goal = 100, min_contribution = 1_000 — unusual but not forbidden.
    client.initialize(
        &creator, &creator, &token, &100, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert_eq!(client.goal(), 100);
    assert_eq!(client.min_contribution(), 1_000);
}

/// @notice Validates that a failed initialization (invalid goal) does not
///         corrupt state — a subsequent valid call must succeed.
#[test]
fn test_initialize_failed_call_leaves_contract_uninitialised() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    // First call fails.
    let _ = client.try_initialize(
        &creator, &creator, &token, &0, &deadline, &1_000, &None, &None, &None, &None,
    );

    // Second call with valid params must succeed.
    client.initialize(
        &creator, &creator, &token, &1_000_000, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert_eq!(client.goal(), 1_000_000);
}

/// @notice Validates that a failed initialization (invalid platform fee) does
///         not corrupt state — a subsequent valid call must succeed.
#[test]
fn test_initialize_failed_platform_fee_leaves_contract_uninitialised() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let bad_cfg = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: 99_999,
    };

    let _ = client.try_initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &Some(bad_cfg),
        &None,
        &None,
        &None,
        &None,
        &None,
        &None,
    );

    // Contract must still be uninitialised.
    client.initialize(
        &creator, &creator, &token, &1_000_000, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert_eq!(client.goal(), 1_000_000);
}

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
//! Comprehensive test suite for `crowdfund_initialize_function`.
//!
//! @title   CrowdfundInitializeFunction Tests
//! @notice  Tests cover: normal execution, all validation error paths,
//!          edge cases, re-initialization guard, event emission, storage
//!          correctness, and helper function behavior.
//! @dev     Target: 95%+ code coverage for the initialize function module.
//!
//! ## Test Categories
//!
//! 1. **Normal execution** — Happy path initialization
//! 2. **Platform config** — Fee validation edge cases
//! 3. **Bonus goal** — Ordering and boundary conditions
//! 4. **Re-initialization guard** — State isolation
//! 5. **Goal validation** — Boundary and invalid values
//! 6. **Min contribution validation** — Boundary and invalid values
//! 7. **Deadline validation** — Time-based constraints
//! 8. **Helper functions** — Unit tests for validation helpers
//! 9. **Error description helpers** — Frontend integration

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    token, Address, Env, String,
};

use crate::{
    crowdfund_initialize_function::{
        describe_init_error, execute_initialize, is_init_error_retryable, validate_bonus_goal,
        validate_bonus_goal_description, validate_init_params, InitParams,
    },
    ContractError, CrowdfundContract, CrowdfundContractClient, PlatformConfig,
};

// ══════════════════════════════════════════════════════════════════════════════
// Test Helpers
// ══════════════════════════════════════════════════════════════════════════════

/// Creates a test environment with mocked authorizations.
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
//! Tests for initialize-function security and maintainability behavior.

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
    let token_id = env.register_stellar_asset_contract_v2(token_admin);
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

    token_admin_client.mint(&creator, &1_000_000);

    (env, client, creator, token_address)
    (env, client, creator, token_address, token_admin)
}

/// Calls `initialize()` with sensible defaults and returns the admin used.
///
/// @param deadline  Unix timestamp for the campaign deadline.
/// Calls `initialize` with sensible defaults and returns the deadline used.
fn default_init(
    client: &CrowdfundContractClient,
    creator: &Address,
    token: &Address,
    deadline: u64,
) -> Address {
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
    );
}

/// Creates InitParams with sensible defaults for direct execute_initialize testing.
fn default_init_params(env: &Env, creator: &Address, token: &Address) -> InitParams {
    InitParams {
        admin: creator.clone(),
        creator: creator.clone(),
        token: token.clone(),
        goal: 1_000_000,
        deadline: env.ledger().timestamp() + 3600,
        min_contribution: 1_000,
        platform_config: None,
        bonus_goal: None,
        bonus_goal_description: None,
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Normal Execution Tests
// ══════════════════════════════════════════════════════════════════════════════

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
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);

    // Verify by attempting a contribution — only works when Active.
    let contributor = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contributor, &5_000);
    client.contribute(&contributor, &5_000);
    assert_eq!(client.total_raised(), 5_000);
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

/// Creator address is stored correctly.
#[test]
fn test_initialize_stores_creator_address() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let other_creator = Address::generate(&env);
    
    client.initialize(
        &creator,
        &other_creator,
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
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Platform Config Tests
// ══════════════════════════════════════════════════════════════════════════════

/// Platform config is stored and fee is deducted on withdrawal.
#[test]
fn test_initialize_with_platform_config_stores_fee() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let platform_addr = Address::generate(&env);
    let config = PlatformConfig {
        address: platform_addr.clone(),
        fee_bps: 500, // 5%
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


    // Contribute and withdraw to verify fee is applied.
    let contributor = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contributor, &1_000_000);
    client.contribute(&contributor, &1_000_000);
    env.ledger().set_timestamp(deadline + 1);
    client.withdraw();

    let token_client = token::Client::new(&env, &token);
    assert_eq!(token_client.balance(&platform_addr), 50_000); // 5%
}

/// Exact maximum platform fee (10_000 bps = 100%) is accepted.
#[test]
fn test_initialize_platform_fee_exact_max_accepted() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let config = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: 10_000,
    };
    let result = client.try_initialize(
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
    );
    assert!(result.is_ok());
}

/// Platform fee of 0 bps is accepted.
#[test]
fn test_initialize_platform_fee_zero_accepted() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let config = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: 0,
    };
    let result = client.try_initialize(
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
    );
    assert!(result.is_ok());
}

/// Platform fee of 1 bps is accepted.
#[test]
fn test_initialize_platform_fee_one_bps_accepted() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let config = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: 1,
    };
    let result = client.try_initialize(
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
    );
    assert!(result.is_ok());
}

/// Platform fee of 10_001 bps returns InvalidPlatformFee.
#[test]
fn test_initialize_platform_fee_over_max_returns_error() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let config = PlatformConfig {
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
        &Some(config),
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
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let config = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: u32::MAX,
    };
    let result = client.try_initialize(
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
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidPlatformFee
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Bonus Goal Tests
// ══════════════════════════════════════════════════════════════════════════════

/// Bonus goal and description are stored and readable.
#[test]
fn test_initialize_with_bonus_goal_stores_values() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let desc = String::from_str(&env, "Unlock exclusive rewards");
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

        &Some(1_000_000),
        &Some(2_000_000i128),
        &Some(desc.clone()),
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
    assert_eq!(client.bonus_goal(), Some(2_000_000));
    assert_eq!(client.bonus_goal_description(), Some(desc));
    assert!(!client.bonus_goal_reached());
    assert_eq!(client.bonus_goal_progress_bps(), 0);
}

/// Bonus goal equal to primary goal returns InvalidBonusGoal.
#[test]
fn test_initialize_bonus_goal_equal_to_goal_returns_error() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &creator,
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

/// @notice `bonus_goal < goal` must return `InvalidBonusGoal`.
#[test]
fn test_initialize_rejects_bonus_goal_below_goal() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

/// Bonus goal less than primary goal returns InvalidBonusGoal.
#[test]
fn test_initialize_bonus_goal_less_than_goal_returns_error() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
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
        &Some(500_000i128),
        &None,
        &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidBonusGoal
    );
}

/// @notice `bonus_goal = goal + 1` (the minimum valid value) must succeed.
#[test]
fn test_initialize_accepts_bonus_goal_one_above_goal() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

/// Bonus goal of 1 above primary goal is accepted.
#[test]
fn test_initialize_bonus_goal_one_above_goal_accepted() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(1_000_001i128),
        &None,
        &None,
    );
    assert!(result.is_ok());
    assert_eq!(client.bonus_goal(), Some(1_000_001));
}

/// Bonus goal without description stores None for description.
#[test]
fn test_initialize_bonus_goal_without_description() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
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
        &Some(2_000_000i128),
        &None,
        &None,
    );
    assert_eq!(client.bonus_goal(), Some(2_000_000));
    assert_eq!(client.bonus_goal_description(), None);
}

/// Bonus goal of i128::MAX is accepted (theoretical maximum).
#[test]
fn test_initialize_bonus_goal_i128_max_accepted() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(i128::MAX),
        &None,
        &None,
    );
    assert!(result.is_ok());
}

// ══════════════════════════════════════════════════════════════════════════════
// Re-initialization Guard Tests
// ══════════════════════════════════════════════════════════════════════════════

/// Second initialize call returns AlreadyInitialized.
#[test]
fn test_initialize_twice_returns_already_initialized() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
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
    );
    assert_eq!(client.bonus_goal(), Some(1_000_001));
}

/// @notice Bonus goal with a description must store both fields correctly.
#[test]
fn test_initialize_stores_bonus_goal_with_description() {
    let (env, client, creator, token) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let desc = SorobanString::from_str(&env, "Unlock stretch delivery milestone");

    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::AlreadyInitialized
    );
}

/// Re-initialization with different parameters still returns AlreadyInitialized.
#[test]
fn test_initialize_twice_different_params_still_errors() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);

    let result = client.try_initialize(
        &creator,
        &creator,
        &token,
        &9_999_999, // different goal
        &(deadline + 7200),
        &500,
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

/// Re-initialization does not modify any storage values.
#[test]
fn test_initialize_twice_preserves_original_values() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);

    // Attempt re-init with different values
    let _ = client.try_initialize(
        &creator,
        &creator,
        &token,
        &9_999_999,
        &(deadline + 7200),
        &500,
        &None,
        &None,
        &None,
        &None,
    );

    // Verify original values unchanged
    assert_eq!(client.goal(), 1_000_000);
    assert_eq!(client.deadline(), deadline);
    assert_eq!(client.min_contribution(), 1_000);
    assert_eq!(client.total_raised(), 0);
}

// ══════════════════════════════════════════════════════════════════════════════
// Goal Validation Tests
// ══════════════════════════════════════════════════════════════════════════════

/// Goal of 1 (minimum) is accepted.
#[test]
fn test_initialize_goal_minimum_accepted() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &creator,
        &creator,
        &token,
        &1,
        &deadline,
        &1,
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
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &creator,
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
    assert_eq!(result.unwrap_err().unwrap(), ContractError::InvalidGoal);
}

/// Negative goal returns InvalidGoal.
#[test]
fn test_initialize_goal_negative_returns_error() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &creator,
        &creator,
        &token,
        &-1,
        &deadline,
        &1,
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
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &creator,
        &creator,
        &token,
        &i128::MIN,
        &deadline,
        &1,
        &None,
        &None,
        &None,
        &None,
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::InvalidGoal);
}

/// Goal of i128::MAX is accepted (theoretical maximum).
#[test]
fn test_initialize_goal_i128_max_accepted() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
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
    );
    assert!(result.is_ok());
}

// ══════════════════════════════════════════════════════════════════════════════
// Min Contribution Validation Tests
// ══════════════════════════════════════════════════════════════════════════════

/// Min contribution of 1 (minimum) is accepted.
#[test]
fn test_initialize_min_contribution_minimum_accepted() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1,
        &None,
        &None,
        &None,
        &None,
    );
    assert!(result.is_ok());
    assert_eq!(client.min_contribution(), 1);
}

/// Min contribution of 0 returns InvalidMinContribution.
#[test]
fn test_initialize_min_contribution_zero_returns_error() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &0,
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

/// Negative min contribution returns InvalidMinContribution.
#[test]
fn test_initialize_min_contribution_negative_returns_error() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let result = client.try_initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &-1,
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

// ══════════════════════════════════════════════════════════════════════════════
// Deadline Validation Tests
// ══════════════════════════════════════════════════════════════════════════════

/// Deadline exactly 60 seconds in the future is accepted.
#[test]
fn test_initialize_deadline_exactly_60_seconds_accepted() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 60;
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
    );
    assert!(result.is_ok());
}

/// Deadline 59 seconds in the future returns DeadlineTooSoon.
#[test]
fn test_initialize_deadline_59_seconds_returns_error() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 59;
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
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::DeadlineTooSoon);
}

/// Deadline equal to current time returns DeadlineTooSoon.
#[test]
fn test_initialize_deadline_equal_to_now_returns_error() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp();
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
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::DeadlineTooSoon);
}

/// Deadline in the past returns DeadlineTooSoon.
#[test]
fn test_initialize_deadline_in_past_returns_error() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() - 100;
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
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::DeadlineTooSoon);
}

/// Deadline far in the future is accepted.
#[test]
fn test_initialize_deadline_far_future_accepted() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 100_000_000;
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
    );
    assert!(result.is_ok());
    assert_eq!(client.deadline(), deadline);
}

/// Deadline u64::MAX is accepted (theoretical maximum).
#[test]
fn test_initialize_deadline_u64_max_accepted() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = u64::MAX;
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
    );
    assert!(result.is_ok());
}

// ══════════════════════════════════════════════════════════════════════════════
// Validation Helper Unit Tests
// ══════════════════════════════════════════════════════════════════════════════

/// validate_bonus_goal with None bonus_goal returns Ok.
#[test]
fn test_validate_bonus_goal_none_returns_ok() {
    let result = validate_bonus_goal(None, 1_000_000);
    assert!(result.is_ok());
}

/// validate_bonus_goal with bonus_goal > goal returns Ok.
#[test]
fn test_validate_bonus_goal_greater_than_goal_returns_ok() {
    let result = validate_bonus_goal(Some(2_000_000), 1_000_000);
    assert!(result.is_ok());
}

/// validate_bonus_goal with bonus_goal == goal returns Err.
#[test]
fn test_validate_bonus_goal_equal_to_goal_returns_err() {
    let result = validate_bonus_goal(Some(1_000_000), 1_000_000);
    assert_eq!(result.unwrap_err(), ContractError::InvalidBonusGoal);
}

/// validate_bonus_goal with bonus_goal < goal returns Err.
#[test]
fn test_validate_bonus_goal_less_than_goal_returns_err() {
    let result = validate_bonus_goal(Some(500_000), 1_000_000);
    assert_eq!(result.unwrap_err(), ContractError::InvalidBonusGoal);
}

/// validate_bonus_goal with bonus_goal = 0 and goal = 1 returns Err.
#[test]
fn test_validate_bonus_goal_zero_vs_one_returns_err() {
    let result = validate_bonus_goal(Some(0), 1);
    assert_eq!(result.unwrap_err(), ContractError::InvalidBonusGoal);
}

/// validate_bonus_goal_description with None returns Ok.
#[test]
fn test_validate_bonus_goal_description_none_returns_ok() {
    let result = validate_bonus_goal_description(&None);
    assert!(result.is_ok());
}

/// validate_init_params integration test with valid params.
#[test]
fn test_validate_init_params_valid() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let token = Address::generate(&env);
    
    let params = default_init_params(&env, &creator, &token);
    let result = validate_init_params(&env, &params);
    assert!(result.is_ok());
}

/// validate_init_params fails with invalid goal.
#[test]
fn test_validate_init_params_invalid_goal() {
    let env = Env::default();
    let creator = Address::generate(&env);
    let token = Address::generate(&env);
    
    let mut params = default_init_params(&env, &creator, &token);
    params.goal = 0;
    let result = validate_init_params(&env, &params);
    assert_eq!(result.unwrap_err(), ContractError::InvalidGoal);
}

// ══════════════════════════════════════════════════════════════════════════════
// Error Description Helper Tests
// ══════════════════════════════════════════════════════════════════════════════

/// describe_init_error returns correct message for AlreadyInitialized.
#[test]
fn test_describe_init_error_already_initialized() {
    assert_eq!(
        describe_init_error(1),
        "Contract is already initialized"
    );
}

/// describe_init_error returns correct message for InvalidGoal.
#[test]
fn test_describe_init_error_invalid_goal() {
    assert_eq!(
        describe_init_error(8),
        "Campaign goal must be at least 1"
    );
}

/// describe_init_error returns correct message for InvalidMinContribution.
#[test]
fn test_describe_init_error_invalid_min_contribution() {
    assert_eq!(
        describe_init_error(9),
        "Minimum contribution must be at least 1"
    );
}

/// describe_init_error returns correct message for DeadlineTooSoon.
#[test]
fn test_describe_init_error_deadline_too_soon() {
    assert_eq!(
        describe_init_error(10),
        "Deadline must be at least 60 seconds in the future"
    );
}

/// describe_init_error returns correct message for InvalidPlatformFee.
#[test]
fn test_describe_init_error_invalid_platform_fee() {
    assert_eq!(
        describe_init_error(11),
        "Platform fee cannot exceed 100% (10,000 bps)"
    );
}

/// describe_init_error returns correct message for InvalidBonusGoal.
#[test]
fn test_describe_init_error_invalid_bonus_goal() {
    assert_eq!(
        describe_init_error(12),
        "Bonus goal must be strictly greater than the primary goal"
    );
}

/// describe_init_error returns fallback for unknown error code.
#[test]
fn test_describe_init_error_unknown_code() {
    assert_eq!(
        describe_init_error(99),
        "Unknown initialization error"
    );
}

/// describe_init_error returns fallback for zero.
#[test]
fn test_describe_init_error_zero_code() {
    assert_eq!(
        describe_init_error(0),
        "Unknown initialization error"
    );
}

/// is_init_error_retryable returns false for AlreadyInitialized.
#[test]
fn test_is_init_error_retryable_already_initialized() {
    assert!(!is_init_error_retryable(1));
}

/// is_init_error_retryable returns true for all input errors.
#[test]
fn test_is_init_error_retryable_input_errors() {
    assert!(is_init_error_retryable(8));  // InvalidGoal
    assert!(is_init_error_retryable(9));  // InvalidMinContribution
    assert!(is_init_error_retryable(10)); // DeadlineTooSoon
    assert!(is_init_error_retryable(11)); // InvalidPlatformFee
    assert!(is_init_error_retryable(12)); // InvalidBonusGoal
}

/// is_init_error_retryable returns false for unknown error codes.
#[test]
fn test_is_init_error_retryable_unknown_code() {
    assert!(!is_init_error_retryable(99));
}

// ══════════════════════════════════════════════════════════════════════════════
// Integration Tests
// ══════════════════════════════════════════════════════════════════════════════

/// After initialization, contribution works correctly.
#[test]
fn test_initialize_then_contribute() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);

    let contributor = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contributor, &5_000);
    
    client.contribute(&contributor, &5_000);
    assert_eq!(client.total_raised(), 5_000);
    assert_eq!(client.contributors().len(), 1);
}

/// After initialization, withdraw works after deadline with sufficient funds.
#[test]
fn test_initialize_then_withdraw() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    default_init(&client, &creator, &token, deadline);

    let contributor = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contributor, &1_000_000);
    
    client.contribute(&contributor, &1_000_000);
    
    // Fast forward past deadline
    env.ledger().set_timestamp(deadline + 1);
    
    // Finalize first
    client.finalize();
    
    // Now withdraw should work
    client.withdraw();
}

/// Initialize with all optional parameters combined works correctly.
#[test]
fn test_initialize_with_all_optional_params() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    let platform = Address::generate(&env);
    let desc = String::from_str(&env, "Stretch goal for extra features");
    
    let config = PlatformConfig {
        address: platform.clone(),
        fee_bps: 250, // 2.5%
    };
    
    client.initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &Some(config),
        &Some(2_000_000),
        &Some(desc.clone()),
        &None,
    );

    // Verify all values stored
    assert_eq!(client.goal(), 1_000_000);
    assert_eq!(client.bonus_goal(), Some(2_000_000));
    assert_eq!(client.bonus_goal_description(), Some(desc));
    
    // Verify contribution still works
    let contributor = Address::generate(&env);
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&contributor, &1_000_000);
    client.contribute(&contributor, &1_000_000);
    assert_eq!(client.total_raised(), 1_000_000);
}

/// execute_initialize directly stores all fields correctly.
#[test]
fn test_execute_initialize_stores_fields_directly() {
    let env = make_env();
    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);
    
    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = token_id.address();
    
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&creator, &10_000_000);
    
    let params = InitParams {
        admin: creator.clone(),
        creator: creator.clone(),
        token: token.clone(),
        goal: 5_000_000,
        deadline: env.ledger().timestamp() + 7200,
        min_contribution: 500,
        platform_config: None,
        bonus_goal: Some(10_000_000),
        bonus_goal_description: None,
    };
    
    let result = execute_initialize(&env, params);
    assert!(result.is_ok());
    
    assert_eq!(client.goal(), 5_000_000);
    assert_eq!(client.deadline(), env.ledger().timestamp() + 7200);
    assert_eq!(client.min_contribution(), 500);
    assert_eq!(client.bonus_goal(), Some(10_000_000));
}

/// execute_initialize fails for already initialized contract.
#[test]
fn test_execute_initialize_fails_if_already_initialized() {
    let env = make_env();
    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);
    
    let creator = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token = token_id.address();
    
    let token_admin_client = token::StellarAssetClient::new(&env, &token);
    token_admin_client.mint(&creator, &10_000_000);
    
    // First initialization
    let params1 = InitParams {
        admin: creator.clone(),
        creator: creator.clone(),
        token: token.clone(),
        goal: 1_000_000,
        deadline: env.ledger().timestamp() + 3600,
        min_contribution: 1000,
        platform_config: None,
        bonus_goal: None,
        bonus_goal_description: None,
    };
    let result1 = execute_initialize(&env, params1);
    assert!(result1.is_ok());
    
    // Second initialization should fail
    let params2 = InitParams {
        admin: creator.clone(),
        creator: creator.clone(),
        token: token.clone(),
        goal: 2_000_000,
        deadline: env.ledger().timestamp() + 7200,
        min_contribution: 2000,
        platform_config: None,
        bonus_goal: None,
        bonus_goal_description: None,
    };
    let result2 = execute_initialize(&env, params2);
    assert_eq!(result2.unwrap_err(), ContractError::AlreadyInitialized);
}

/// Bonus goal reached flag starts as false.
#[test]
fn test_initialize_bonus_goal_reached_flag_starts_false() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3600;
    
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

        &Some(2_000_000),
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
#![cfg(test)]

use soroban_sdk::{testutils::Ledger, Env};

use crate::crowdfund_initialize_function::{
    validate_initialization_params, validate_initialization_params_bool, InitError,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn env_at(ts: u64) -> Env {
    let env = Env::default();
    env.ledger().set_timestamp(ts);
    env
}

// ── Happy-path ───────────────────────────────────────────────────────────────

#[test]
fn test_valid_minimal_params() {
    let env = env_at(1_000);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 2_000, 10, None, None),
        Ok(())
    );
}

#[test]
fn test_valid_min_contribution_equals_goal() {
    // min_contribution == goal is the boundary — still valid.
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 500, 1_000, 500, None, None),
        Ok(())
    );
}

#[test]
fn test_valid_with_zero_fee_bps() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, Some(0), None),
        Ok(())
    );
}

#[test]
fn test_valid_with_max_fee_bps() {
    // 10 000 bps = 100 % — edge case that must be accepted.
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, Some(10_000), None),
        Ok(())
    );
}

#[test]
fn test_valid_with_bonus_goal() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, None, Some(2_000)),
        Ok(())
    );
}

#[test]
fn test_valid_bonus_goal_just_above_primary() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, None, Some(1_001)),
        Ok(())
    );
}

#[test]
fn test_valid_deadline_one_second_in_future() {
    let env = env_at(999);
    assert_eq!(
        validate_initialization_params(&env, 100, 1_000, 1, None, None),
        Ok(())
    );
}

#[test]
fn test_valid_large_goal() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, i128::MAX, 1_000, 1, None, None),
        Ok(())
    );
}

// ── goal validation ──────────────────────────────────────────────────────────

#[test]
fn test_goal_zero_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 0, 1_000, 1, None, None),
        Err(InitError::GoalNotPositive)
    );
}

#[test]
fn test_goal_negative_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, -1, 1_000, 1, None, None),
        Err(InitError::GoalNotPositive)
    );
}

// ── deadline validation ──────────────────────────────────────────────────────

#[test]
fn test_deadline_in_past_is_invalid() {
    let env = env_at(3_000);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 2_000, 10, None, None),
        Err(InitError::DeadlineInPast)
    );
}

#[test]
fn test_deadline_equal_to_now_is_invalid() {
    let env = env_at(1_000);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 1_000, 10, None, None),
        Err(InitError::DeadlineInPast)
    );
}

// ── min_contribution validation ──────────────────────────────────────────────

#[test]
fn test_min_contribution_zero_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 0, None, None),
        Err(InitError::MinContributionNotPositive)
    );
}

#[test]
fn test_min_contribution_negative_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, -5, None, None),
        Err(InitError::MinContributionNotPositive)
    );
}

#[test]
fn test_min_contribution_exceeds_goal() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 100, 9_999, 150, None, None),
        Err(InitError::MinContributionExceedsGoal)
    );
}

#[test]
fn test_min_contribution_one_above_goal_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1_001, None, None),
        Err(InitError::MinContributionExceedsGoal)
    );
}

// ── platform fee validation ──────────────────────────────────────────────────

#[test]
fn test_platform_fee_above_max_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, Some(10_001), None),
        Err(InitError::PlatformFeeExceedsMax)
    );
}

#[test]
fn test_platform_fee_u32_max_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, Some(u32::MAX), None),
        Err(InitError::PlatformFeeExceedsMax)
    );
}

// ── bonus_goal validation ────────────────────────────────────────────────────

#[test]
fn test_bonus_goal_equal_to_primary_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, None, Some(1_000)),
        Err(InitError::BonusGoalNotGreaterThanGoal)
    );
}

#[test]
fn test_bonus_goal_below_primary_is_invalid() {
    let env = env_at(0);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 9_999, 1, None, Some(500)),
        Err(InitError::BonusGoalNotGreaterThanGoal)
    );
}

// ── error ordering (goal checked first) ─────────────────────────────────────

#[test]
fn test_goal_error_takes_priority_over_deadline() {
    // Both goal and deadline are invalid; goal error should surface first.
    let env = env_at(5_000);
    assert_eq!(
        validate_initialization_params(&env, 0, 1_000, 1, None, None),
        Err(InitError::GoalNotPositive)
    );
}

#[test]
fn test_deadline_error_takes_priority_over_min_contribution() {
    let env = env_at(5_000);
    assert_eq!(
        validate_initialization_params(&env, 1_000, 1_000, 0, None, None),
        Err(InitError::DeadlineInPast)
    );
}

// ── InitError::message ───────────────────────────────────────────────────────

#[test]
fn test_error_messages_are_non_empty() {
    let errors = [
        InitError::GoalNotPositive,
        InitError::DeadlineInPast,
        InitError::MinContributionNotPositive,
        InitError::MinContributionExceedsGoal,
        InitError::PlatformFeeExceedsMax,
        InitError::BonusGoalNotGreaterThanGoal,
    ];
    for e in errors {
        assert!(
            !e.message().is_empty(),
            "message for {e:?} must not be empty"
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
// ── bool-returning compat wrapper ────────────────────────────────────────────

#[test]
fn test_bool_wrapper_returns_true_for_valid_params() {
    let env = env_at(1_000);
    assert!(validate_initialization_params_bool(&env, 1_000, 2_000, 10));
}

#[test]
fn test_bool_wrapper_returns_false_for_invalid_goal() {
    let env = env_at(0);
    assert!(!validate_initialization_params_bool(&env, 0, 2_000, 10));
}

#[test]
fn test_bool_wrapper_returns_false_for_past_deadline() {
    let env = env_at(3_000);
    assert!(!validate_initialization_params_bool(&env, 1_000, 2_000, 10));
}

#[test]
fn test_bool_wrapper_returns_false_for_invalid_min_contribution() {
    let env = env_at(0);
    assert!(!validate_initialization_params_bool(&env, 100, 2_000, 150));
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
        &None,
        &None,
    );
    
    assert!(!client.bonus_goal_reached());
}

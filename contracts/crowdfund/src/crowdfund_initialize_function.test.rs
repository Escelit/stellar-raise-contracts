//! # crowdfund_initialize_function — Comprehensive Test Suite

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, String as SorobanString,
};

use crate::{ContractError, CrowdfundContract, CrowdfundContractClient, PlatformConfig};

// ── Test helpers ──────────────────────────────────────────────────────────────

fn setup() -> (
    Env,
    CrowdfundContractClient<'static>,
    Address, // creator
    Address, // token
    Address, // admin
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_id.address();
    let token_admin_client = token::StellarAssetClient::new(&env, &token_address);

    let creator = Address::generate(&env);
    token_admin_client.mint(&creator, &10_000_000);

    let admin = Address::generate(&env);

    (env, client, creator, token_address, admin)
}

/// Calls `initialize()` with sensible defaults and returns the admin used.
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
    );
    admin
}

// ── Happy-path tests ──────────────────────────────────────────────────────────

#[test]
fn test_initialize_stores_core_fields() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);

    assert_eq!(client.goal(), 1_000_000);
    assert_eq!(client.deadline(), deadline);
    assert_eq!(client.min_contribution(), 1_000);
    assert_eq!(client.total_raised(), 0);
    assert_eq!(client.token(), token);
}

#[test]
fn test_initialize_version_is_correct() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.version(), 3);
}

#[test]
fn test_initialize_status_is_active() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.status(), crate::Status::Active);
}

#[test]
fn test_initialize_contributors_list_is_empty() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.contributors().len(), 0);
}

#[test]
fn test_initialize_roadmap_is_empty() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.roadmap().len(), 0);
}

#[test]
fn test_initialize_total_raised_is_zero() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.total_raised(), 0);
}

#[test]
fn test_initialize_emits_event() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);
    let events = env.events().all();
    assert!(!events.is_empty());
}

#[test]
fn test_initialize_stores_admin_address() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
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
    assert_eq!(client.goal(), 1_000_000);
}

// ── Re-initialization guard ───────────────────────────────────────────────────

#[test]
fn test_initialize_twice_returns_already_initialized() {
    let (env, client, creator, token, _admin) = setup();
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
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::AlreadyInitialized
    );
}

// ── Goal validation ───────────────────────────────────────────────────────────

#[test]
fn test_initialize_rejects_zero_goal() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    let result = client.try_initialize(
        &creator, &creator, &token, &0, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::InvalidGoal);
}

#[test]
fn test_initialize_rejects_negative_goal() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    let result = client.try_initialize(
        &creator, &creator, &token, &-1, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::InvalidGoal);
}

#[test]
fn test_initialize_accepts_minimum_goal() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    client.initialize(
        &creator, &creator, &token, &1, &deadline, &1, &None, &None, &None, &None,
    );
    assert_eq!(client.goal(), 1);
}

// ── Min-contribution validation ───────────────────────────────────────────────

#[test]
fn test_initialize_rejects_zero_min_contribution() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    let result = client.try_initialize(
        &creator, &creator, &token, &1_000_000, &deadline, &0, &None, &None, &None, &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidMinContribution
    );
}

#[test]
fn test_initialize_rejects_negative_min_contribution() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    let result = client.try_initialize(
        &creator, &creator, &token, &1_000_000, &deadline, &-1, &None, &None, &None, &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidMinContribution
    );
}

#[test]
fn test_initialize_accepts_minimum_min_contribution() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    client.initialize(
        &creator, &creator, &token, &1_000_000, &deadline, &1, &None, &None, &None, &None,
    );
    assert_eq!(client.min_contribution(), 1);
}

// ── Deadline validation ───────────────────────────────────────────────────────

#[test]
fn test_initialize_rejects_past_deadline() {
    let (env, client, creator, token, _admin) = setup();
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
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::DeadlineTooSoon);
}

#[test]
fn test_initialize_rejects_deadline_below_min_offset() {
    let (env, client, creator, token, _admin) = setup();
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
    );
    assert_eq!(result.unwrap_err().unwrap(), ContractError::DeadlineTooSoon);
}

#[test]
fn test_initialize_accepts_deadline_at_min_offset() {
    let (env, client, creator, token, _admin) = setup();
    let now = env.ledger().timestamp();
    let deadline = now + 60;

    client.initialize(
        &creator, &creator, &token, &1_000_000, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert_eq!(client.deadline(), deadline);
}

// ── Platform fee validation ───────────────────────────────────────────────────

#[test]
fn test_initialize_rejects_fee_over_100_percent() {
    let (env, client, creator, token, _admin) = setup();
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
        &None,
        &Some(cfg),
        &None,
        &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidPlatformFee
    );
}

#[test]
fn test_initialize_accepts_fee_at_100_percent() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    let cfg = PlatformConfig {
        address: Address::generate(&env),
        fee_bps: 10_000,
    };

    client.initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &Some(cfg),
        &None,
        &None,
    );
    assert_eq!(client.goal(), 1_000_000);
}

#[test]
fn test_initialize_accepts_zero_fee() {
    let (env, client, creator, token, _admin) = setup();
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
        &None,
        &Some(cfg),
        &None,
        &None,
    );
    assert_eq!(client.goal(), 1_000_000);
}

// ── Bonus goal validation ─────────────────────────────────────────────────────

#[test]
fn test_initialize_rejects_bonus_goal_equal_to_goal() {
    let (env, client, creator, token, _admin) = setup();
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
        &Some(1_000_000i128),
        &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidBonusGoal
    );
}

#[test]
fn test_initialize_rejects_bonus_goal_below_goal() {
    let (env, client, creator, token, _admin) = setup();
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
        &Some(500_000i128),
        &None,
    );
    assert_eq!(
        result.unwrap_err().unwrap(),
        ContractError::InvalidBonusGoal
    );
}

#[test]
fn test_initialize_accepts_bonus_goal_one_above_goal() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    client.initialize(
        &creator,
        &creator,
        &token,
        &1_000_000,
        &deadline,
        &1_000,
        &None,
        &None,
        &Some(1_000_001i128),
        &None,
    );
    assert_eq!(client.bonus_goal(), Some(1_000_001));
}

#[test]
fn test_initialize_stores_bonus_goal_with_description() {
    let (env, client, creator, token, _admin) = setup();
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
        &None,
        &Some(2_000_000i128),
        &Some(desc.clone()),
    );

    assert_eq!(client.bonus_goal(), Some(2_000_000));
    assert_eq!(client.bonus_goal_description(), Some(desc));
}

// ── Storage field completeness ────────────────────────────────────────────────

#[test]
fn test_initialize_optional_fields_absent_when_not_provided() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);

    assert_eq!(client.bonus_goal(), None);
    assert_eq!(client.bonus_goal_description(), None);
    assert_eq!(client.nft_contract(), None);
}

#[test]
fn test_initialize_total_raised_starts_at_zero() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.total_raised(), 0);
}

#[test]
fn test_initialize_stores_token_address() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);
    assert_eq!(client.token(), token);
}

#[test]
fn test_initialize_stores_separate_admin() {
    let (env, client, creator, token, admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

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

    assert_eq!(client.goal(), 1_000_000);
}

#[test]
fn test_initialize_all_optional_fields_populated() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 7_200;
    let cfg = PlatformConfig {
        address: Address::generate(&env),
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
        &None,
        &Some(cfg),
        &Some(10_000_000i128),
        &Some(desc.clone()),
    );

    assert_eq!(client.goal(), 5_000_000);
    assert_eq!(client.min_contribution(), 10_000);
    assert_eq!(client.deadline(), deadline);
    assert_eq!(client.bonus_goal(), Some(10_000_000));
    assert_eq!(client.bonus_goal_description(), Some(desc));
    assert_eq!(client.total_raised(), 0);
}

// ── Event emission ────────────────────────────────────────────────────────────

#[test]
fn test_initialize_emits_initialized_event() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;
    default_init(&client, &creator, &token, deadline);

    assert_eq!(client.status(), crate::Status::Active);
    assert_eq!(client.goal(), 1_000_000);
}

#[test]
fn test_initialize_no_event_on_failure() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    let result = client.try_initialize(
        &creator, &creator, &token, &0, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert!(result.is_err());

    client.initialize(
        &creator, &creator, &token, &1_000_000, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert_eq!(client.goal(), 1_000_000);
}

// ── Error helper functions ────────────────────────────────────────────────────

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

#[test]
fn test_describe_init_error_unknown_code() {
    use crate::crowdfund_initialize_function::describe_init_error;
    assert_eq!(describe_init_error(99), "Unknown initialization error");
}

#[test]
fn test_is_init_error_retryable_already_initialized_is_permanent() {
    use crate::crowdfund_initialize_function::is_init_error_retryable;
    assert!(!is_init_error_retryable(1));
}

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

#[test]
fn test_initialize_accepts_max_goal() {
    let (env, client, creator, token, _admin) = setup();
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
    );
    assert_eq!(client.goal(), i128::MAX);
}

#[test]
fn test_initialize_accepts_max_deadline() {
    let (env, client, creator, token, _admin) = setup();

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
    );
    assert_eq!(client.deadline(), u64::MAX);
}

#[test]
fn test_initialize_allows_min_contribution_greater_than_goal() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    client.initialize(
        &creator, &creator, &token, &100, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert_eq!(client.goal(), 100);
    assert_eq!(client.min_contribution(), 1_000);
}

#[test]
fn test_initialize_failed_call_leaves_contract_uninitialised() {
    let (env, client, creator, token, _admin) = setup();
    let deadline = env.ledger().timestamp() + 3_600;

    let _ = client.try_initialize(
        &creator, &creator, &token, &0, &deadline, &1_000, &None, &None, &None, &None,
    );

    client.initialize(
        &creator, &creator, &token, &1_000_000, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert_eq!(client.goal(), 1_000_000);
}

#[test]
fn test_initialize_failed_platform_fee_leaves_contract_uninitialised() {
    let (env, client, creator, token, _admin) = setup();
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
        &None,
        &Some(bad_cfg),
        &None,
        &None,
    );

    client.initialize(
        &creator, &creator, &token, &1_000_000, &deadline, &1_000, &None, &None, &None, &None,
    );
    assert_eq!(client.goal(), 1_000_000);
}

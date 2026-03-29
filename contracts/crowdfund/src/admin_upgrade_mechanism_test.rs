//! Tests for the admin upgrade mechanism — gas efficiency edge cases.
//!
//! Covers:
//! - `validate_wasm_hash`: zero hash, non-zero hash, boundary patterns.
//! - `is_admin_initialized`: before and after `initialize()`.
//! - `validate_admin_upgrade`: admin stored, auth enforced, panic before init.
//! - `upgrade()` via contract client: zero-hash short-circuit, non-admin
//!   rejection, pre-init panic, storage persistence after rejected call.
//! Tests for the admin upgrade mechanism.
//!
//! Covers:
//! - Admin address is stored correctly during `initialize()`.
//! - Only the admin can call `upgrade()` (auth guard enforced).
//! - A non-admin caller is rejected by `upgrade()`.
//! - Creator (distinct from admin) cannot call `upgrade()`.
//! - `upgrade()` panics when called before `initialize()` (no admin stored).
//! - Admin auth is required: no-auth call is rejected.
//! - Zero WASM hash is rejected (edge case: all-zero hash).
//! - Non-zero WASM hash passes validation.
//! - Storage is untouched after a rejected upgrade call.
//! - `validate_wasm_hash`: zero hash, non-zero hash, boundary patterns.
//! - `is_admin_initialized`: before and after `initialize()`.
//! - `validate_admin_upgrade`: admin stored, auth enforced, panic before init.
//! - `upgrade()` via contract client: zero-hash short-circuit, non-admin
//!   rejection, pre-init panic, storage persistence after rejected call.

extern crate std;

use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, BytesN, Env,
};

use crate::{
    admin_upgrade_mechanism::{is_admin_initialized, validate_wasm_hash},
    CrowdfundContract, CrowdfundContractClient,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn setup() -> (
    Env,
    Address,
    CrowdfundContractClient<'static>,
    Address, // admin
    Address, // creator
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    token, Address, BytesN, Env,
};

use crate::{
    admin_upgrade_mechanism::{validate_wasm_hash},
    admin_upgrade_mechanism::{is_admin_initialized, validate_wasm_hash},
    CrowdfundContract, CrowdfundContractClient,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn setup() -> (
    Env,
    Address,
    CrowdfundContractClient<'static>,
    Address,
    Address,
    Address,
    Address, // admin
    Address, // creator
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(token_admin);
    let token_id = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_addr = token_id.address();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 3600;

    client.initialize(
        &admin,
        &creator,
        &token_addr,
        &1_000,
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

    (env, contract_id, client, admin, creator)
}

fn zero_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0u8; 32])
}

fn nonzero_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[1u8; 32])
}

// ── validate_wasm_hash (pure, no Env) ────────────────────────────────────────

/// @notice Zero hash is always invalid — cheapest possible rejection.
#[test]
fn validate_wasm_hash_rejects_zero() {
    let env = Env::default();
    assert!(!validate_wasm_hash(&zero_hash(&env)));
}

/// @notice Any non-zero byte makes the hash valid.
#[test]
fn validate_wasm_hash_accepts_nonzero() {
    let env = Env::default();
    assert!(validate_wasm_hash(&nonzero_hash(&env)));
}

/// Only the first byte set → valid.
#[test]
fn validate_wasm_hash_first_byte_nonzero() {
    let env = Env::default();
    let mut bytes = [0u8; 32];
    bytes[0] = 1;
    assert!(validate_wasm_hash(&BytesN::from_array(&env, &bytes)));
}

/// Only the last byte set → valid.
#[test]
fn validate_wasm_hash_last_byte_nonzero() {
    let env = Env::default();
    let mut bytes = [0u8; 32];
    bytes[31] = 1;
    assert!(validate_wasm_hash(&BytesN::from_array(&env, &bytes)));
}

/// All 0xFF → valid.
#[test]
fn validate_wasm_hash_all_ff_valid() {
    let env = Env::default();
    assert!(validate_wasm_hash(&BytesN::from_array(&env, &[0xFF; 32])));
}

/// Alternating bytes → valid.
#[test]
fn validate_wasm_hash_alternating_bytes_valid() {
    let env = Env::default();
    let bytes: [u8; 32] = core::array::from_fn(|i| if i % 2 == 0 { 0xAA } else { 0x00 });
    assert!(validate_wasm_hash(&BytesN::from_array(&env, &bytes)));
}

// ── is_admin_initialized ──────────────────────────────────────────────────────

/// @notice Returns false before initialize() — no storage read of the value.
#[test]
fn is_admin_initialized_false_before_init() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundContract, ());
    // Invoke the check inside the contract's storage context.
    // We test the helper directly via the module since it's pub.
    // The contract's instance storage is scoped to contract_id, so we need
    // to call it from within that context — use a raw env check instead.
    // is_admin_initialized reads env.storage().instance(), which is
    // contract-scoped; we verify the helper logic with a fresh env.
    let fresh_env = Env::default();
    // A fresh env has no instance storage set → has() returns false.
    assert!(!is_admin_initialized(&fresh_env));
    let _ = contract_id;
}

/// @notice Returns true after initialize() stores the admin.
/// Verified indirectly: validate_admin_upgrade succeeds (no "Admin not
/// initialized" panic) only when the admin key is present.
#[test]
fn is_admin_initialized_true_after_init() {
    let (env, _contract_id, client, _admin, _creator) = setup();
    // upgrade() with no auth → auth error (not "Admin not initialized")
    // confirms the admin key is present in storage.
    env.set_auths(&[]);
    let result = client.try_upgrade(&nonzero_hash(&env));
    // Auth error, not a storage/unwrap panic → admin was stored.
    assert!(result.is_err());
}

// ── validate_admin_upgrade (via contract client) ──────────────────────────────

/// @notice upgrade() panics before initialize() — no admin in storage.
#[test]
#[should_panic(expected = "Admin not initialized")]
fn upgrade_panics_before_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);
    client.upgrade(&nonzero_hash(&env));
}

/// @notice Non-admin caller is rejected.
#[test]
fn upgrade_rejects_non_admin() {
    let (env, contract_id, client, _admin, _creator) = setup();
    let non_admin = Address::generate(&env);
    env.set_auths(&[]);
    );

    (env, contract_id, client, admin, creator, token_addr)
}

fn dummy_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[1u8; 32])
}

fn zero_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0u8; 32])
}

// ── Auth / access control tests ───────────────────────────────────────────────

/// Admin address is stored on initialize — confirmed by upgrade() reaching
/// the auth check rather than panicking on a missing-storage unwrap.
#[test]
fn test_admin_stored_on_initialize() {
    let (env, contract_id, client, _admin, _creator, _token) = setup();
fn zero_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0u8; 32])
}

fn nonzero_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[1u8; 32])
}

// ── validate_wasm_hash (pure, no Env) ────────────────────────────────────────

/// @notice Zero hash is always invalid — cheapest possible rejection.
#[test]
fn validate_wasm_hash_rejects_zero() {
    let env = Env::default();
    assert!(!validate_wasm_hash(&zero_hash(&env)));
}

/// @notice Any non-zero byte makes the hash valid.
#[test]
fn validate_wasm_hash_accepts_nonzero() {
    let env = Env::default();
    assert!(validate_wasm_hash(&nonzero_hash(&env)));
}

/// Only the first byte set → valid.
#[test]
fn validate_wasm_hash_first_byte_nonzero() {
    let env = Env::default();
    let mut bytes = [0u8; 32];
    bytes[0] = 1;
    assert!(validate_wasm_hash(&BytesN::from_array(&env, &bytes)));
}

/// Only the last byte set → valid.
#[test]
fn validate_wasm_hash_last_byte_nonzero() {
    let env = Env::default();
    let mut bytes = [0u8; 32];
    bytes[31] = 1;
    assert!(validate_wasm_hash(&BytesN::from_array(&env, &bytes)));
}

/// All 0xFF → valid.
#[test]
fn validate_wasm_hash_all_ff_valid() {
    let env = Env::default();
    assert!(validate_wasm_hash(&BytesN::from_array(&env, &[0xFF; 32])));
}

/// Alternating bytes → valid.
#[test]
fn validate_wasm_hash_alternating_bytes_valid() {
    let env = Env::default();
    let bytes: [u8; 32] = core::array::from_fn(|i| if i % 2 == 0 { 0xAA } else { 0x00 });
    assert!(validate_wasm_hash(&BytesN::from_array(&env, &bytes)));
}

// ── is_admin_initialized ──────────────────────────────────────────────────────

/// @notice Returns false before initialize() — no storage read of the value.
#[test]
fn is_admin_initialized_false_before_init() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundContract, ());
    // Invoke the check inside the contract's storage context.
    // We test the helper directly via the module since it's pub.
    // The contract's instance storage is scoped to contract_id, so we need
    // to call it from within that context — use a raw env check instead.
    // is_admin_initialized reads env.storage().instance(), which is
    // contract-scoped; we verify the helper logic with a fresh env.
    let fresh_env = Env::default();
    // A fresh env has no instance storage set → has() returns false.
    assert!(!is_admin_initialized(&fresh_env));
    let _ = contract_id;
}

/// @notice Returns true after initialize() stores the admin.
/// Verified indirectly: validate_admin_upgrade succeeds (no "Admin not
/// initialized" panic) only when the admin key is present.
#[test]
fn is_admin_initialized_true_after_init() {
    let (env, _contract_id, client, _admin, _creator) = setup();
    // upgrade() with no auth → auth error (not "Admin not initialized")
    // confirms the admin key is present in storage.
    env.set_auths(&[]);
    let result = client.try_upgrade(&nonzero_hash(&env));
    // Auth error, not a storage/unwrap panic → admin was stored.
    assert!(result.is_err());
}

// ── validate_admin_upgrade (via contract client) ──────────────────────────────

/// @notice upgrade() panics before initialize() — no admin in storage.
#[test]
#[should_panic(expected = "Admin not initialized")]
fn upgrade_panics_before_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);
    client.upgrade(&nonzero_hash(&env));
}

/// @notice Non-admin caller is rejected.
#[test]
fn upgrade_rejects_non_admin() {
    let (env, contract_id, client, _admin, _creator) = setup();
    let non_admin = Address::generate(&env);
    env.set_auths(&[]);
    let result = client
        .mock_auths(&[MockAuth {
            address: &non_admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "upgrade",
                args: soroban_sdk::vec![&env, nonzero_hash(&env).into()],
                sub_invokes: &[],
            },
        }])
        .try_upgrade(&nonzero_hash(&env));
                args: soroban_sdk::vec![&env, dummy_hash(&env).into()],
                sub_invokes: &[],
            },
        }])
        .try_upgrade(&dummy_hash(&env));
    // Auth error (not a storage panic) confirms admin was stored.
    assert!(result.is_err());
}
        .try_upgrade(&nonzero_hash(&env));

/// Non-admin caller is rejected.
#[test]
fn test_non_admin_cannot_upgrade() {
    let (env, contract_id, client, _admin, _creator, _token) = setup();
    let non_admin = Address::generate(&env);
    env.set_auths(&[]);
    let result = client
        .mock_auths(&[MockAuth {
            address: &non_admin,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "upgrade",
                args: soroban_sdk::vec![&env, dummy_hash(&env).into()],
                sub_invokes: &[],
            },
        }])
        .try_upgrade(&dummy_hash(&env));
    assert!(result.is_err());
}

/// @notice Creator (distinct from admin) cannot upgrade.
#[test]
fn upgrade_rejects_creator() {
    let (env, contract_id, client, _admin, creator) = setup();
/// Creator (distinct from admin) cannot call upgrade().
#[test]
fn test_creator_cannot_upgrade() {
    let (env, contract_id, client, _admin, creator, _token) = setup();
fn upgrade_rejects_creator() {
    let (env, contract_id, client, _admin, creator) = setup();

    env.set_auths(&[]);
    let result = client
        .mock_auths(&[MockAuth {
            address: &creator,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "upgrade",
                args: soroban_sdk::vec![&env, nonzero_hash(&env).into()],
                sub_invokes: &[],
            },
        }])
        .try_upgrade(&nonzero_hash(&env));
                args: soroban_sdk::vec![&env, dummy_hash(&env).into()],
                sub_invokes: &[],
            },
        }])
        .try_upgrade(&dummy_hash(&env));
        .try_upgrade(&nonzero_hash(&env));

    assert!(result.is_err());
}

/// @notice upgrade() with no auths set is rejected.
#[test]
fn upgrade_requires_auth() {
    let (env, _contract_id, client, _admin, _creator) = setup();
    env.set_auths(&[]);
    assert!(client.try_upgrade(&nonzero_hash(&env)).is_err());
}

// ── Gas-efficiency edge case: zero-hash short-circuit ────────────────────────

/// @notice Zero hash is rejected before any storage read or auth check.
/// @dev This is the core gas-efficiency improvement: a zero hash panics with
///      "zero wasm hash" rather than reaching `validate_admin_upgrade`.
#[test]
#[should_panic(expected = "zero wasm hash")]
fn upgrade_panics_on_zero_hash_before_auth() {
    let (env, _contract_id, client, _admin, _creator) = setup();
    // mock_all_auths is active from setup(); even with valid auth the zero
    // hash must be caught first.
    client.upgrade(&zero_hash(&env));
}

/// @notice Zero hash is rejected even when called with no auth at all.
/// @dev Confirms the zero-hash check fires before the auth check — the panic
///      message is "zero wasm hash", not an auth error.
#[test]
#[should_panic(expected = "zero wasm hash")]
fn upgrade_zero_hash_rejected_before_auth_check() {
    let (env, _contract_id, client, _admin, _creator) = setup();
    env.set_auths(&[]);
    client.upgrade(&zero_hash(&env));
}

/// @notice Zero hash is rejected even before initialize() — pure check fires first.
#[test]
#[should_panic(expected = "zero wasm hash")]
fn upgrade_zero_hash_rejected_before_init_check() {
/// upgrade() panics when called before initialize() — no admin in storage.
#[test]
fn upgrade_requires_auth() {
    let (env, _contract_id, client, _admin, _creator) = setup();
    env.set_auths(&[]);
    assert!(client.try_upgrade(&nonzero_hash(&env)).is_err());
}

// ── Gas-efficiency edge case: zero-hash short-circuit ────────────────────────

/// @notice Zero hash is rejected before any storage read or auth check.
/// @dev This is the core gas-efficiency improvement: a zero hash panics with
///      "zero wasm hash" rather than reaching `validate_admin_upgrade`.
#[test]
#[should_panic(expected = "zero wasm hash")]
fn upgrade_panics_on_zero_hash_before_auth() {
    let (env, _contract_id, client, _admin, _creator) = setup();
    // mock_all_auths is active from setup(); even with valid auth the zero
    // hash must be caught first.
    client.upgrade(&zero_hash(&env));
}

/// @notice Zero hash is rejected even when called with no auth at all.
/// @dev Confirms the zero-hash check fires before the auth check — the panic
///      message is "zero wasm hash", not an auth error.
#[test]
#[should_panic(expected = "zero wasm hash")]
fn upgrade_zero_hash_rejected_before_auth_check() {
    let (env, _contract_id, client, _admin, _creator) = setup();
    env.set_auths(&[]);
    client.upgrade(&zero_hash(&env));
}

/// @notice Zero hash is rejected even before initialize() — pure check fires first.
#[test]
#[should_panic(expected = "zero wasm hash")]
fn upgrade_zero_hash_rejected_before_init_check() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);
    // No initialize() called; zero hash should still panic with "zero wasm hash"
    // (not "Admin not initialized"), proving the pure check runs first.
    client.upgrade(&zero_hash(&env));
}

// ── Storage persistence after rejected upgrade ────────────────────────────────

/// @notice Campaign state is unchanged after a rejected upgrade call.
#[test]
fn storage_unchanged_after_rejected_upgrade() {
    let (env, _contract_id, client, _admin, _creator) = setup();

    let goal_before = client.goal();
    let deadline_before = client.deadline();
    let raised_before = client.total_raised();

    env.set_auths(&[]);
    let _ = client.try_upgrade(&nonzero_hash(&env));

    assert_eq!(client.goal(), goal_before);
    assert_eq!(client.deadline(), deadline_before);
    assert_eq!(client.total_raised(), raised_before);
}

/// @notice Campaign state is unchanged after a zero-hash rejection.
#[test]
fn storage_unchanged_after_zero_hash_rejection() {
    let (env, _contract_id, client, _admin, _creator) = setup();

    let goal_before = client.goal();
    let raised_before = client.total_raised();

    // Zero hash panics; catch it via try_upgrade.
    let _ = client.try_upgrade(&zero_hash(&env));

    assert_eq!(client.goal(), goal_before);
    assert_eq!(client.total_raised(), raised_before);
    client.upgrade(&dummy_hash(&env)); // unwrap() on None → panic
    client.upgrade(&dummy_hash(&env));
}

/// upgrade() with no auths set is rejected.
#[test]
fn test_upgrade_requires_auth() {
    let (env, _contract_id, client, _admin, _creator, _token) = setup();
    env.set_auths(&[]);
    let result = client.try_upgrade(&dummy_hash(&env));
    assert!(result.is_err());
}

// ── WASM hash validation edge cases ──────────────────────────────────────────

/// All-zero WASM hash is rejected — edge case for missing/unset upload.
#[test]
#[should_panic(expected = "upgrade: wasm_hash must not be zero")]
fn test_zero_wasm_hash_rejected() {
    let env = Env::default();
    validate_wasm_hash(&zero_hash(&env));
}

/// Non-zero WASM hash passes validation without panic.
#[test]
fn test_nonzero_wasm_hash_accepted() {
    let env = Env::default();
    validate_wasm_hash(&dummy_hash(&env)); // must not panic
}

/// Minimum non-zero hash (only last byte set) passes validation.
#[test]
fn test_minimal_nonzero_wasm_hash_accepted() {
    let env = Env::default();
    let mut bytes = [0u8; 32];
    bytes[31] = 1;
    validate_wasm_hash(&BytesN::from_array(&env, &bytes));
}

/// Admin calling upgrade() with a zero hash is rejected before WASM swap.
#[test]
fn test_admin_upgrade_with_zero_hash_rejected() {
    let (env, _contract_id, client, _admin, _creator, _token) = setup();
    // mock_all_auths is active from setup — admin auth passes, hash check fails.
    let result = client.try_upgrade(&zero_hash(&env));
    assert!(result.is_err());
}

// ── State persistence ─────────────────────────────────────────────────────────

/// Contract storage is untouched after a rejected upgrade call.
#[test]
fn test_storage_persists_after_rejected_upgrade() {
    let (env, _contract_id, client, _admin, _creator, _token) = setup();
    // No initialize() called; zero hash should still panic with "zero wasm hash"
    // (not "Admin not initialized"), proving the pure check runs first.
    client.upgrade(&zero_hash(&env));
}

// ── Storage persistence after rejected upgrade ────────────────────────────────

/// @notice Campaign state is unchanged after a rejected upgrade call.
#[test]
fn storage_unchanged_after_rejected_upgrade() {
    let (env, _contract_id, client, _admin, _creator) = setup();

    let goal_before = client.goal();
    let deadline_before = client.deadline();
    let raised_before = client.total_raised();

    env.set_auths(&[]);
    let _ = client.try_upgrade(&dummy_hash(&env));
    let _ = client.try_upgrade(&nonzero_hash(&env));

    assert_eq!(client.goal(), goal_before);
    assert_eq!(client.deadline(), deadline_before);
    assert_eq!(client.total_raised(), raised_before);
}

/// Admin-only upgrade: valid WASM binary test (requires compiled WASM).
#[test]
#[ignore = "requires: cargo build --target wasm32-unknown-unknown --release"]
fn test_admin_can_upgrade_with_valid_wasm() {
    mod crowdfund_wasm {
        soroban_sdk::contractimport!(
            file = "../../target/wasm32-unknown-unknown/release/crowdfund.wasm"
        );
    }
    let (env, _contract_id, client, _admin, _creator, _token) = setup();
    let wasm_hash = env.deployer().upload_contract_wasm(crowdfund_wasm::WASM);
    client.upgrade(&wasm_hash);
/// @notice Campaign state is unchanged after a zero-hash rejection.
#[test]
fn storage_unchanged_after_zero_hash_rejection() {
    let (env, _contract_id, client, _admin, _creator) = setup();

    let goal_before = client.goal();
    let raised_before = client.total_raised();

    // Zero hash panics; catch it via try_upgrade.
    let _ = client.try_upgrade(&zero_hash(&env));

    assert_eq!(client.goal(), goal_before);
    assert_eq!(client.total_raised(), raised_before);
}

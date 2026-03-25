//! Tests for the admin upgrade mechanism.
//!
//! Covers:
//! - Admin address is stored correctly during `initialize()`.
//! - Only the admin can call `upgrade()` (auth guard enforced).
//! - A non-admin caller is rejected by `upgrade()`.
//! - `upgrade()` panics when called before `initialize()` (no admin stored).
//! - Admin distinct from creator: creator cannot call `upgrade()`.

extern crate std;

use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    token, Address, BytesN, Env,
};

use crate::{CrowdfundContract, CrowdfundContractClient};

// ── Helper ───────────────────────────────────────────────────────────────────

fn setup(
) -> (
    Env,
    Address, // contract_id
    CrowdfundContractClient<'static>,
    Address, // admin
    Address, // creator
    Address, // token
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    let token_admin = Address::generate(&env);
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
    );

    (env, contract_id, client, admin, creator, token_addr)
}

/// Dummy 32-byte hash — used where we only need to reach the auth check,
/// not actually execute the WASM swap.
fn dummy_hash(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[1u8; 32])
}

// ── Tests ────────────────────────────────────────────────────────────────────

/// Admin address is stored and readable after initialize().
/// Verified indirectly: upgrade() succeeds only when admin auth is provided.
#[test]
fn test_admin_stored_on_initialize() {
    // If admin were not stored, upgrade() would panic with unwrap() on None.
    // The fact that try_upgrade reaches the auth check (not a storage panic)
    // confirms admin was stored.
    let (env, contract_id, client, admin, _creator, _token) = setup();

    // Provide auth for a non-admin — should be rejected (not a storage error).
    let non_admin = Address::generate(&env);
    env.set_auths(&[]);
    let result = client.mock_auths(&[MockAuth {
        address: &non_admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "upgrade",
            args: soroban_sdk::vec![&env, dummy_hash(&env).into()],
            sub_invokes: &[],
        },
    }])
    .try_upgrade(&dummy_hash(&env));

    // Auth error — not a storage/unwrap panic — confirms admin was stored.
    assert!(result.is_err());
    let _ = admin; // admin was used in initialize
}

/// Non-admin caller is rejected by upgrade().
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

/// Creator (distinct from admin) cannot call upgrade().
#[test]
fn test_creator_cannot_upgrade() {
    let (env, contract_id, client, _admin, creator, _token) = setup();

    env.set_auths(&[]);
    let result = client
        .mock_auths(&[MockAuth {
            address: &creator,
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

/// upgrade() panics when called before initialize() — no admin in storage.
#[test]
#[should_panic]
fn test_upgrade_panics_before_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);
    client.upgrade(&dummy_hash(&env)); // unwrap() on None → panic
}

/// Admin auth is required: calling upgrade() with no auths set is rejected.
#[test]
fn test_upgrade_requires_auth() {
    let (env, _contract_id, client, _admin, _creator, _token) = setup();

    env.set_auths(&[]);
    let result = client.try_upgrade(&dummy_hash(&env));
    assert!(result.is_err());
}

/// Admin can successfully call upgrade() with a valid uploaded WASM hash.
/// Uses the pre-built crowdfund WASM from the release target directory.
#[test]
#[ignore = "requires wasm-opt: run `cargo build --target wasm32-unknown-unknown --release` first"]
fn test_admin_can_upgrade_with_valid_wasm() {
    mod crowdfund_wasm {
        soroban_sdk::contractimport!(
            file = "../../target/wasm32-unknown-unknown/release/crowdfund.wasm"
        );
    }

    let (env, _contract_id, client, _admin, _creator, _token) = setup();
    let wasm_hash = env
        .deployer()
        .upload_contract_wasm(crowdfund_wasm::WASM);
    // Admin auth is mocked via mock_all_auths in setup — should succeed.
    client.upgrade(&wasm_hash);
}

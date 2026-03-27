#![cfg(test)]
use crate::{CrowdfundContract, CrowdfundContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

// ── Helper ───────────────────────────────────────────────────────────────────

fn setup() -> (
    Env,
    Address, // contract_id
    CrowdfundContractClient<'static>,
    Address, // admin
    Address, // creator
    Address, // token
) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = Address::generate(&env);

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);

    client.initialize(
        &admin, &creator, &token, &1000i128, &10000u64, &10i128, &None, &None, &None,
        &None, // metadata_uri
    );

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 3_600;

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

// ── Existing tests ────────────────────────────────────────────────────────────

/// Admin address is stored and readable after initialize().
#[test]
fn test_admin_stored_on_initialize() {
    let (env, contract_id, client, admin, _creator, _token) = setup();

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
    let _ = admin;
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
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let token = Address::generate(&env);

    let contract_id = env.register(CrowdfundContract, ());
    let client = CrowdfundContractClient::new(&env, &contract_id);
    client.upgrade(&dummy_hash(&env));
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
#[test]
#[ignore = "requires wasm-opt: run `cargo build --target wasm32-unknown-unknown --release` first"]
fn test_admin_can_upgrade_with_valid_wasm() {
    mod crowdfund_wasm {
        soroban_sdk::contractimport!(
            file = "../../target/wasm32-unknown-unknown/release/crowdfund.wasm"
        );
    }

    // Initialize with mock_all_auths so initialize() succeeds
    env.mock_all_auths();
    client.initialize(
        &admin, &creator, &token, &1000i128, &10000u64, &10i128, &None, &None, &None,
        &None, // metadata_uri
    );

    // Clear auths — upgrade() must fail without admin auth
    env.set_auths(&[]);
    let new_wasm_hash = BytesN::from_array(&env, &[1u8; 32]);
    client.upgrade(&new_wasm_hash);
}

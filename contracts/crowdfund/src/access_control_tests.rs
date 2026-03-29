#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{
    access_control::{
        assert_not_paused, get_default_admin, get_governance, get_pauser, is_paused, pause,
        set_platform_fee, transfer_default_admin, transfer_pauser, unpause,
        assert_not_paused, get_default_admin, get_governance, get_pauser, is_paused,
        pause, set_platform_fee, transfer_default_admin, transfer_pauser, unpause,
        assert_not_paused, get_default_admin, get_pauser, is_paused, pause, set_platform_fee,
        transfer_default_admin, transfer_pauser, unpause,
    },
    DataKey, PlatformConfig,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn setup() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let pauser = Address::generate(&env);
    let governance = Address::generate(&env);

    // Seed storage directly — mirrors what persist_initialize_state will do.
    env.storage().instance().set(&DataKey::DefaultAdmin, &admin);
    env.storage().instance().set(&DataKey::Pauser, &pauser);
    env.storage()
        .instance()
        .set(&DataKey::GovernanceAddress, &governance);
    env.storage().instance().set(&DataKey::Paused, &false);

    (env, admin, pauser, governance)
}

fn dummy_config(env: &Env, fee_bps: u32) -> PlatformConfig {
    PlatformConfig {
        address: Address::generate(env),
        fee_bps,
    }
}

// ── Pause / unpause ──────────────────────────────────────────────────────────

#[test]
fn pauser_can_pause() {
    let (env, _admin, pauser, _gov) = setup();
    pause(&env, &pauser);
    assert!(is_paused(&env));
}

#[test]
fn admin_can_pause() {
    let (env, admin, _pauser, _gov) = setup();
    pause(&env, &admin);
    assert!(is_paused(&env));
}

#[test]
#[should_panic(expected = "not authorized to pause")]
fn non_pauser_cannot_pause() {
    let (env, _admin, _pauser, _gov) = setup();
    let rando = Address::generate(&env);
    pause(&env, &rando);
}

#[test]
#[should_panic(expected = "not authorized to pause")]
fn creator_cannot_pause() {
    let (env, _admin, _pauser, _gov) = setup();
    let creator = Address::generate(&env);
    // Creator is not stored as admin or pauser — must be rejected.
    pause(&env, &creator);
}

#[test]
fn admin_can_unpause() {
    let (env, admin, pauser, _gov) = setup();
    pause(&env, &pauser);
    assert!(is_paused(&env));
    unpause(&env, &admin);
    assert!(!is_paused(&env));
}

#[test]
#[should_panic(expected = "only DEFAULT_ADMIN_ROLE can unpause")]
fn pauser_cannot_unpause() {
    let (env, _admin, pauser, _gov) = setup();
    pause(&env, &pauser);
    // Pauser can freeze but cannot unfreeze — asymmetric by design.
    unpause(&env, &pauser);
}

#[test]
#[should_panic(expected = "only DEFAULT_ADMIN_ROLE can unpause")]
fn non_admin_cannot_unpause() {
    let (env, _admin, pauser, _gov) = setup();
    pause(&env, &pauser);
    let rando = Address::generate(&env);
    unpause(&env, &rando);
}

// ── assert_not_paused ────────────────────────────────────────────────────────

#[test]
fn assert_not_paused_passes_when_unpaused() {
    let (env, _admin, _pauser, _gov) = setup();
    // Should not panic.
    assert_not_paused(&env);
}

#[test]
#[should_panic(expected = "contract is paused")]
fn assert_not_paused_panics_when_paused() {
    let (env, _admin, pauser, _gov) = setup();
    pause(&env, &pauser);
    assert_not_paused(&env);
}

// ── set_platform_fee ─────────────────────────────────────────────────────────

#[test]
fn governance_can_set_fee() {
    let (env, _admin, _pauser, governance) = setup();
    let config = dummy_config(&env, 500);
    let result = set_platform_fee(&env, &governance, config);
    assert!(result.is_ok());
}

#[test]
#[should_panic(expected = "only GovernanceAddress can set platform fee")]
fn non_governance_cannot_set_fee() {
    let (env, _admin, _pauser, _gov) = setup();
    let rando = Address::generate(&env);
    let _ = set_platform_fee(&env, &rando, dummy_config(&env, 500));
}

#[test]
#[should_panic(expected = "only GovernanceAddress can set platform fee")]
fn creator_cannot_set_fee() {
    let (env, _admin, _pauser, _gov) = setup();
    let creator = Address::generate(&env);
    let _ = set_platform_fee(&env, &creator, dummy_config(&env, 500));
}

#[test]
#[should_panic(expected = "only GovernanceAddress can set platform fee")]
fn admin_cannot_set_fee_directly() {
    // Admin manages roles, not fees — fee changes go through governance only.
    let (env, admin, _pauser, _gov) = setup();
    let _ = set_platform_fee(&env, &admin, dummy_config(&env, 500));
}

#[test]
fn set_fee_rejects_over_100_percent() {
    let (env, _admin, _pauser, governance) = setup();
    let result = set_platform_fee(&env, &governance, dummy_config(&env, 10_001));
    assert!(result.is_err());
}

#[test]
fn set_fee_accepts_boundary_100_percent() {
    let (env, _admin, _pauser, governance) = setup();
    let result = set_platform_fee(&env, &governance, dummy_config(&env, 10_000));
    assert!(result.is_ok());
}

// ── Role transfer ────────────────────────────────────────────────────────────

#[test]
fn admin_can_transfer_admin_role() {
    let (env, admin, _pauser, _gov) = setup();
    let new_admin = Address::generate(&env);
    transfer_default_admin(&env, &admin, &new_admin);
    assert_eq!(get_default_admin(&env), new_admin);
}

#[test]
#[should_panic(expected = "only DEFAULT_ADMIN_ROLE can transfer admin role")]
fn non_admin_cannot_transfer_admin_role() {
    let (env, _admin, _pauser, _gov) = setup();
    let rando = Address::generate(&env);
    let new_admin = Address::generate(&env);
    transfer_default_admin(&env, &rando, &new_admin);
}

#[test]
fn admin_can_transfer_pauser_role() {
    let (env, admin, _pauser, _gov) = setup();
    let new_pauser = Address::generate(&env);
    transfer_pauser(&env, &admin, &new_pauser);
    assert_eq!(get_pauser(&env), new_pauser);
}

#[test]
#[should_panic(expected = "only DEFAULT_ADMIN_ROLE can transfer pauser role")]
fn non_admin_cannot_transfer_pauser_role() {
    let (env, _admin, pauser, _gov) = setup();
    let new_pauser = Address::generate(&env);
    // Pauser cannot reassign their own role.
    transfer_pauser(&env, &pauser, &new_pauser);
}

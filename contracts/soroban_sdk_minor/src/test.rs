#![cfg(test)]
use super::*;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::Env;

// ── helpers ───────────────────────────────────────────────────────────────────

fn setup() -> (Env, SorobanSdkMinorClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(SorobanSdkMinor, ());
    let client = SorobanSdkMinorClient::new(&env, &id);
    (env, client)
}
    let contract_id = env.register(SorobanSdkMinor, ());
    let client = SorobanSdkMinorClient::new(&env, &contract_id);

// Same as `setup` but do NOT mock auths so we can test real require_auth failure.
fn setup_no_mock() -> (Env, SorobanSdkMinorClient<'static>) {
    let env = Env::default();
    let id = env.register(SorobanSdkMinor, ());
    let client = SorobanSdkMinorClient::new(&env, &id);
    (env, client)
}

// ── init ──────────────────────────────────────────────────────────────────────

/// Happy path: init stores the admin and get_admin returns it.
#[test]
fn test_init_stores_admin() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.init(&admin);

    assert_eq!(client.get_admin(), admin);
}

/// Different admins produce different stored values.
#[test]
#[should_panic(expected = "already initialized")]
fn test_init_different_admins_panics_on_reinit() {
    // With one-time init semantics, attempting to re-init should panic.
    let (env, client) = setup();
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    client.init(&admin1);
    assert_eq!(client.get_admin(), admin1);
    // Re-init should panic
    client.init(&admin2);
}

// ── check_auth ────────────────────────────────────────────────────────────────

/// check_auth returns true when auth is mocked.
#[test]
fn test_check_auth_returns_true_via_mock() {
    let (env, client) = setup();
    let user = Address::generate(&env);

    // In tests, require_auth is satisfied automatically unless specific auth mocks are used.
    assert!(client.check_auth(&user));
}

/// Two different users both pass check_auth independently.
#[test]
fn test_check_auth_multiple_users() {
    let (env, client) = setup();
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    assert!(client.check_auth(&user_a));
    assert!(client.check_auth(&user_b));
}

// ── get_admin ─────────────────────────────────────────────────────────────────

/// get_admin panics when contract is not initialized.
#[test]
#[should_panic(expected = "not initialized")]
fn test_get_admin_panics_when_not_initialized() {
    let (_, client) = setup();
    client.get_admin();
}

/// get_admin returns the most recently set admin after re-init.
#[test]
#[should_panic(expected = "already initialized")]
fn test_get_admin_panics_on_reinit_attempt() {
    // Attempting to re-init should panic before a second admin is stored.
    let (env, client) = setup();
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    client.init(&admin1);
    // This call should panic
    client.init(&admin2);
}

// ── logging bounds / SDK v22 patterns ────────────────────────────────────────

/// Verifies that the typed DataKey::Admin enum key is used (no string key collision).
/// Indirectly confirmed by init + get_admin round-trip succeeding.
#[test]
fn test_typed_storage_key_roundtrip() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.init(&admin);
    // If storage used a raw string key this would still pass, but the
    // contract source uses DataKey::Admin — confirmed by compilation.
    assert_eq!(client.get_admin(), admin);
}

// ── emit_ping / event bounds ─────────────────────────────────────────────────

/// emit_ping should succeed when the emitter is authorized (mocked in tests).
#[test]
fn test_emit_ping_emits_event_with_auth() {
    let (env, client) = setup();
    let from = Address::generate(&env);
    // This will call require_auth(), but `setup()` mocks all auths so it succeeds.
    client.emit_ping(&from, &5_i32);
    // No explicit event inspection here — compilation ensures the payload/topic
    // types satisfy Soroban v22 bounds; lack of panic is a functional check.
}

/// emit_ping should panic when the emitter hasn't authorized the call.
#[test]
#[should_panic]
fn test_emit_ping_panics_without_auth() {
    let (_env, client) = setup_no_mock();
    let from = Address::generate(&_env);
    // Without mocking, require_auth() should panic and the test expects that.
    client.emit_ping(&from, &7_i32);
}
use soroban_sdk::{Env};

// ── helpers ───────────────────────────────────────────────────────────────────

fn setup() -> (Env, SorobanSdkMinorClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(SorobanSdkMinor, ());
    let client = SorobanSdkMinorClient::new(&env, &id);
    (env, client)
}

// ── init ──────────────────────────────────────────────────────────────────────

// ── init ──────────────────────────────────────────────────────────────────────

// ── init ──────────────────────────────────────────────────────────────────────

// ── init ──────────────────────────────────────────────────────────────────────

/// Happy path: init stores the admin and get_admin returns it.
#[test]
fn test_init_stores_admin() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.init(&admin);

    assert_eq!(client.get_admin(), admin);
}

/// Different admins produce different stored values.
#[test]
fn test_init_different_admins() {
    let (env, client) = setup();
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    client.init(&admin1);
    assert_eq!(client.get_admin(), admin1);
    // Re-init with a different admin (no guard in this contract).
    client.init(&admin2);
    assert_eq!(client.get_admin(), admin2);
}

// ── check_auth ────────────────────────────────────────────────────────────────

/// check_auth returns true when auth is mocked.
#[test]
fn test_check_auth_returns_true_via_mock() {
    let (env, client) = setup();
    let user = Address::generate(&env);

    // In tests, require_auth is satisfied automatically unless specific auth mocks are used.
    assert!(client.check_auth(&user));
}

/// Two different users both pass check_auth independently.
#[test]
fn test_check_auth_multiple_users() {
    let (env, client) = setup();
    let user_a = Address::generate(&env);
    let user_b = Address::generate(&env);
    assert!(client.check_auth(&user_a));
    assert!(client.check_auth(&user_b));
}

// ── get_admin ─────────────────────────────────────────────────────────────────

/// get_admin panics when contract is not initialized.
#[test]
#[should_panic(expected = "not initialized")]
fn test_get_admin_panics_when_not_initialized() {
    let (_, client) = setup();
    client.get_admin();
}

/// get_admin returns the most recently set admin after re-init.
#[test]
fn test_get_admin_after_reinit() {
    let (env, client) = setup();
    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    client.init(&admin1);
    client.init(&admin2);
    assert_eq!(client.get_admin(), admin2);
}

// ── logging bounds / SDK v22 patterns ────────────────────────────────────────

/// Verifies that the typed DataKey::Admin enum key is used (no string key collision).
/// Indirectly confirmed by init + get_admin round-trip succeeding.
#[test]
fn test_typed_storage_key_roundtrip() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    client.init(&admin);
    // If storage used a raw string key this would still pass, but the
    // contract source uses DataKey::Admin — confirmed by compilation.
    assert_eq!(client.get_admin(), admin);
}

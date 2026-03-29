//! Comprehensive tests for the Soroban SDK minor version bump review module.
//!
//! Covers: compatibility assessment, WASM hash validation, audit event
//! emission, version parsing edge cases, and security boundary conditions.

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

use crate::soroban_sdk_minor::{
    assess_compatibility, emit_upgrade_audit_event, validate_wasm_hash, CompatibilityStatus,
    SDK_VERSION_BASELINE, SDK_VERSION_TARGET,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

fn make_wasm_hash(env: &Env, bytes: [u8; 32]) -> BytesN<32> {
    BytesN::from_array(env, &bytes)
}

// ── Version constant tests ────────────────────────────────────────────────────

/// SDK_VERSION_BASELINE must be a valid semver string.
#[test]
fn test_sdk_version_baseline_is_semver() {
    extern crate std;
    let parts: std::vec::Vec<&str> = SDK_VERSION_BASELINE.split('.').collect();
    assert_eq!(parts.len(), 3, "baseline must have three semver components");
    for part in &parts {
        let _: u32 = part
            .parse()
            .expect("each semver component must be a non-negative integer");
    }
}

/// SDK_VERSION_TARGET must be non-empty.
#[test]
fn test_sdk_version_target_is_non_empty() {
    assert!(!SDK_VERSION_TARGET.is_empty());
}

// ── assess_compatibility tests ────────────────────────────────────────────────

/// Same major version → Compatible.
#[test]
fn test_same_major_is_compatible() {
    let env = make_env();
    let result = assess_compatibility(&env, "22.0.0", "22.1.0");
    assert_eq!(result, CompatibilityStatus::Compatible);
}

/// Same major, same minor, different patch → Compatible.
#[test]
fn test_same_major_minor_patch_bump_is_compatible() {
    let env = make_env();
    let result = assess_compatibility(&env, "22.0.0", "22.0.5");
    assert_eq!(result, CompatibilityStatus::Compatible);
}

/// Identical versions → Compatible.
#[test]
fn test_identical_versions_are_compatible() {
    let env = make_env();
    let result = assess_compatibility(&env, "22.0.0", "22.0.0");
    assert_eq!(result, CompatibilityStatus::Compatible);
}

/// Different major versions → RequiresMigration.
#[test]
fn test_different_major_requires_migration() {
    let env = make_env();
    let result = assess_compatibility(&env, "22.0.0", "23.0.0");
    assert_eq!(result, CompatibilityStatus::RequiresMigration);
}

/// Downgrade across major → RequiresMigration.
#[test]
fn test_major_downgrade_requires_migration() {
    let env = make_env();
    let result = assess_compatibility(&env, "23.0.0", "22.0.0");
    assert_eq!(result, CompatibilityStatus::RequiresMigration);
}

/// Malformed version string (no dots) → treated as major 0, compatible with
/// another major-0 string.
#[test]
fn test_malformed_version_parses_as_zero() {
    let env = make_env();
    // Both parse to major 0 → Compatible.
    let result = assess_compatibility(&env, "invalid", "also-invalid");
    assert_eq!(result, CompatibilityStatus::Compatible);
}

/// Empty version string → major 0, compatible with another empty string.
#[test]
fn test_empty_version_string() {
    let env = make_env();
    let result = assess_compatibility(&env, "", "");
    assert_eq!(result, CompatibilityStatus::Compatible);
}

/// Large version numbers are handled without overflow.
#[test]
fn test_large_version_numbers() {
    let env = make_env();
    let result = assess_compatibility(&env, "4294967295.0.0", "4294967295.99.0");
    assert_eq!(result, CompatibilityStatus::Compatible);
}

// ── validate_wasm_hash tests ──────────────────────────────────────────────────

/// A non-zero hash is valid.
#[test]
fn test_valid_wasm_hash_accepted() {
    let env = make_env();
    let mut bytes = [0u8; 32];
    bytes[0] = 0xde;
    bytes[31] = 0xad;
    let hash = make_wasm_hash(&env, bytes);
    assert!(validate_wasm_hash(&hash));
}

/// A fully-zeroed hash is rejected (security boundary).
#[test]
fn test_zero_wasm_hash_rejected() {
    let env = make_env();
    let hash = make_wasm_hash(&env, [0u8; 32]);
    assert!(!validate_wasm_hash(&hash));
}

/// A hash with only the last byte set is valid.
#[test]
fn test_wasm_hash_last_byte_nonzero_is_valid() {
    let env = make_env();
    let mut bytes = [0u8; 32];
    bytes[31] = 1;
    let hash = make_wasm_hash(&env, bytes);
    assert!(validate_wasm_hash(&hash));
}

/// A hash with only the first byte set is valid.
#[test]
fn test_wasm_hash_first_byte_nonzero_is_valid() {
    let env = make_env();
    let mut bytes = [0u8; 32];
    bytes[0] = 1;
    let hash = make_wasm_hash(&env, bytes);
    assert!(validate_wasm_hash(&hash));
}

/// A fully-set (0xFF) hash is valid.
#[test]
fn test_wasm_hash_all_ff_is_valid() {
    let env = make_env();
    let hash = make_wasm_hash(&env, [0xFF; 32]);
    assert!(validate_wasm_hash(&hash));
}

/// Validate that two distinct non-zero hashes are both accepted.
#[test]
fn test_two_distinct_valid_hashes() {
    let env = make_env();
    let mut a = [0u8; 32];
    a[0] = 1;
    let mut b = [0u8; 32];
    b[15] = 42;
    assert!(validate_wasm_hash(&make_wasm_hash(&env, a)));
    assert!(validate_wasm_hash(&make_wasm_hash(&env, b)));
}

// ── emit_upgrade_audit_event tests ───────────────────────────────────────────

/// Emitting an audit event does not panic and completes without error.
#[test]
fn test_audit_event_does_not_panic() {
    let env = make_env();
    let reviewer = Address::generate(&env);

    // Should complete without panicking.
    emit_upgrade_audit_event(
        &env,
        String::from_str(&env, "22.0.0"),
        String::from_str(&env, "22.1.0"),
        reviewer,
    );
}

/// Emitting an audit event with the same version (no-op upgrade) does not panic.
#[test]
fn test_audit_event_same_version_does_not_panic() {
    let env = make_env();
    let reviewer = Address::generate(&env);

    emit_upgrade_audit_event(
        &env,
        String::from_str(&env, "22.0.0"),
        String::from_str(&env, "22.0.0"),
        reviewer,
    );
}

/// Multiple audit event calls with different reviewers do not panic.
#[test]
fn test_multiple_audit_events_do_not_panic() {
    let env = make_env();
    let reviewer1 = Address::generate(&env);
    let reviewer2 = Address::generate(&env);

    emit_upgrade_audit_event(
        &env,
        String::from_str(&env, "22.0.0"),
        String::from_str(&env, "22.1.0"),
        reviewer1,
    );
    emit_upgrade_audit_event(
        &env,
        String::from_str(&env, "22.1.0"),
        String::from_str(&env, "22.2.0"),
        reviewer2,
    );
}

// ── Integration: compatibility + hash validation ──────────────────────────────

/// A safe upgrade path: same major, valid hash → both checks pass.
#[test]
fn test_safe_upgrade_path() {
    let env = make_env();
    let mut hash_bytes = [0u8; 32];
    hash_bytes[0] = 0xAB;
    let hash = make_wasm_hash(&env, hash_bytes);

    let compat = assess_compatibility(&env, "22.0.0", "22.1.0");
    let hash_ok = validate_wasm_hash(&hash);

    assert_eq!(compat, CompatibilityStatus::Compatible);
    assert!(hash_ok);
}

/// An unsafe upgrade path: cross-major + zero hash → both checks fail.
#[test]
fn test_unsafe_upgrade_path() {
    let env = make_env();
    let hash = make_wasm_hash(&env, [0u8; 32]);

    let compat = assess_compatibility(&env, "22.0.0", "23.0.0");
    let hash_ok = validate_wasm_hash(&hash);

    assert_eq!(compat, CompatibilityStatus::RequiresMigration);
    assert!(!hash_ok);
}

/// Partial failure: compatible versions but zero hash → hash check fails.
#[test]
fn test_compatible_versions_but_zero_hash_fails() {
    let env = make_env();
    let hash = make_wasm_hash(&env, [0u8; 32]);

    let compat = assess_compatibility(&env, "22.0.0", "22.1.0");
    let hash_ok = validate_wasm_hash(&hash);

    assert_eq!(compat, CompatibilityStatus::Compatible);
    assert!(!hash_ok, "zero hash must be rejected even when versions are compatible");
}

/// Partial failure: valid hash but cross-major → compatibility check fails.
#[test]
fn test_valid_hash_but_cross_major_fails() {
    let env = make_env();
    let mut bytes = [0u8; 32];
    bytes[0] = 1;
    let hash = make_wasm_hash(&env, bytes);

    let compat = assess_compatibility(&env, "22.0.0", "23.0.0");
    let hash_ok = validate_wasm_hash(&hash);

    assert_eq!(compat, CompatibilityStatus::RequiresMigration);
    assert!(hash_ok);
}

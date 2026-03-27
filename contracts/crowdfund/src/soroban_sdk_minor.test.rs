//! Tests for `soroban_sdk_minor` frontend/scalability helpers.
//!
//! Covers: compatibility assessment, minor-bump detection, WASM hash
//! validation, pagination bounds, upgrade-note validation, audit event
//! emission, SdkChangeRecord construction, emit_ping_event auth enforcement,
//! and all edge cases for the v22 minor bump.

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

use crate::soroban_sdk_minor::{
    assess_compatibility, build_sdk_change_record, clamp_page_size, emit_ping_event,
    emit_upgrade_audit_event, emit_upgrade_audit_event_with_note, is_minor_bump,
    pagination_window, parse_minor, validate_upgrade_note, validate_wasm_hash,
    CompatibilityStatus, FRONTEND_PAGE_SIZE_MAX, FRONTEND_PAGE_SIZE_MIN, SDK_VERSION_BASELINE,
    SDK_VERSION_TARGET, UPGRADE_NOTE_MAX_LEN,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn make_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env
}

/// Build a Soroban String of `len` bytes filled with `byte`.
fn make_string(env: &Env, len: u32, byte: u8) -> String {
    let bytes = [byte; 512];
    String::from_bytes(env, &bytes[..len as usize])
}

// ── Version constants ─────────────────────────────────────────────────────────

#[test]
fn version_constants_are_non_empty() {
    assert!(!SDK_VERSION_BASELINE.is_empty());
    assert!(!SDK_VERSION_TARGET.is_empty());
}

// ── assess_compatibility ──────────────────────────────────────────────────────

/// Same-major minor bump → Compatible.
#[test]
fn compatibility_same_major_is_compatible() {
    let env = make_env();
    assert_eq!(
        assess_compatibility(&env, "22.0.0", "22.1.0"),
        CompatibilityStatus::Compatible
    );
}

/// Identical versions → Compatible.
#[test]
fn compatibility_identical_versions_is_compatible() {
    let env = make_env();
    assert_eq!(
        assess_compatibility(&env, "22.0.0", "22.0.0"),
        CompatibilityStatus::Compatible
    );
}

/// Same major, patch-only bump → Compatible.
#[test]
fn compatibility_patch_bump_is_compatible() {
    let env = make_env();
    assert_eq!(
        assess_compatibility(&env, "22.0.0", "22.0.5"),
        CompatibilityStatus::Compatible
    );
}

/// Cross-major upgrade → RequiresMigration.
#[test]
fn compatibility_cross_major_requires_migration() {
    let env = make_env();
    assert_eq!(
        assess_compatibility(&env, "22.1.0", "23.0.0"),
        CompatibilityStatus::RequiresMigration
    );
}

/// Major downgrade → RequiresMigration.
#[test]
fn compatibility_major_downgrade_requires_migration() {
    let env = make_env();
    assert_eq!(
        assess_compatibility(&env, "23.0.0", "22.0.0"),
        CompatibilityStatus::RequiresMigration
    );
}

/// Malformed (no dots) but non-empty → both parse to major 0 → Compatible.
#[test]
fn compatibility_malformed_non_empty_parses_as_zero_major() {
    let env = make_env();
    assert_eq!(
        assess_compatibility(&env, "invalid", "also-invalid"),
        CompatibilityStatus::Compatible
    );
}

/// One valid, one malformed → major mismatch → RequiresMigration.
#[test]
fn compatibility_one_malformed_requires_migration() {
    let env = make_env();
    assert_eq!(
        assess_compatibility(&env, "22.0.0", "invalid"),
        CompatibilityStatus::RequiresMigration
    );
}

/// Empty `from_version` → Incompatible (not silently major-0).
#[test]
fn compatibility_empty_from_version_is_incompatible() {
    let env = make_env();
    assert_eq!(
        assess_compatibility(&env, "", "22.0.0"),
        CompatibilityStatus::Incompatible
    );
}

/// Empty `to_version` → Incompatible.
#[test]
fn compatibility_empty_to_version_is_incompatible() {
    let env = make_env();
    assert_eq!(
        assess_compatibility(&env, "22.0.0", ""),
        CompatibilityStatus::Incompatible
    );
}

/// Both empty → Incompatible.
#[test]
fn compatibility_both_empty_is_incompatible() {
    let env = make_env();
    assert_eq!(
        assess_compatibility(&env, "", ""),
        CompatibilityStatus::Incompatible
    );
}

/// Large version numbers do not overflow.
#[test]
fn compatibility_large_version_numbers() {
    let env = make_env();
    assert_eq!(
        assess_compatibility(&env, "4294967295.0.0", "4294967295.99.0"),
        CompatibilityStatus::Compatible
    );
}

// ── parse_minor ───────────────────────────────────────────────────────────────

#[test]
fn parse_minor_standard_semver() {
    assert_eq!(parse_minor("22.3.0"), 3);
    assert_eq!(parse_minor("22.0.0"), 0);
    assert_eq!(parse_minor("22.99.1"), 99);
}

/// Only major present → minor is 0.
#[test]
fn parse_minor_no_minor_component() {
    assert_eq!(parse_minor("22"), 0);
}

/// Trailing dot with no minor value → 0.
#[test]
fn parse_minor_trailing_dot() {
    assert_eq!(parse_minor("22."), 0);
}

/// Non-numeric minor → 0.
#[test]
fn parse_minor_non_numeric() {
    assert_eq!(parse_minor("22.x.0"), 0);
}

/// Empty string → 0.
#[test]
fn parse_minor_empty_string() {
    assert_eq!(parse_minor(""), 0);
}

// ── is_minor_bump ─────────────────────────────────────────────────────────────

#[test]
fn is_minor_bump_detects_forward_minor() {
    assert!(is_minor_bump("22.0.0", "22.1.0"));
    assert!(is_minor_bump("22.1.0", "22.2.0"));
}

/// Same minor → not a minor bump.
#[test]
fn is_minor_bump_same_minor_is_false() {
    assert!(!is_minor_bump("22.1.0", "22.1.0"));
}

/// Patch-only change → not a minor bump.
#[test]
fn is_minor_bump_patch_only_is_false() {
    assert!(!is_minor_bump("22.1.0", "22.1.5"));
}

/// Minor downgrade → not a minor bump.
#[test]
fn is_minor_bump_downgrade_is_false() {
    assert!(!is_minor_bump("22.2.0", "22.1.0"));
}

/// Cross-major → not a minor bump.
#[test]
fn is_minor_bump_cross_major_is_false() {
    assert!(!is_minor_bump("22.0.0", "23.1.0"));
}

// ── validate_wasm_hash ────────────────────────────────────────────────────────

#[test]
fn validate_wasm_hash_rejects_zero() {
    let env = make_env();
    let hash = BytesN::from_array(&env, &[0u8; 32]);
    assert!(!validate_wasm_hash(&hash));
}

#[test]
fn validate_wasm_hash_accepts_non_zero() {
    let env = make_env();
    let mut bytes = [0u8; 32];
    bytes[0] = 1;
    let hash = BytesN::from_array(&env, &bytes);
    assert!(validate_wasm_hash(&hash));
}

/// Only the last byte set → valid.
#[test]
fn validate_wasm_hash_last_byte_nonzero() {
    let env = make_env();
    let mut bytes = [0u8; 32];
    bytes[31] = 1;
    assert!(validate_wasm_hash(&BytesN::from_array(&env, &bytes)));
}

/// All 0xFF → valid.
#[test]
fn validate_wasm_hash_all_ff() {
    let env = make_env();
    assert!(validate_wasm_hash(&BytesN::from_array(&env, &[0xFF; 32])));
}

// ── clamp_page_size ───────────────────────────────────────────────────────────

#[test]
fn clamp_page_size_enforces_bounds() {
    assert_eq!(clamp_page_size(0), FRONTEND_PAGE_SIZE_MIN);
    assert_eq!(clamp_page_size(1), 1);
    assert_eq!(clamp_page_size(50), 50);
    assert_eq!(clamp_page_size(100), FRONTEND_PAGE_SIZE_MAX);
    assert_eq!(clamp_page_size(FRONTEND_PAGE_SIZE_MAX + 1), FRONTEND_PAGE_SIZE_MAX);
    assert_eq!(clamp_page_size(u32::MAX), FRONTEND_PAGE_SIZE_MAX);
}

// ── pagination_window ─────────────────────────────────────────────────────────

#[test]
fn pagination_window_uses_clamped_limit() {
    let window = pagination_window(20, 1_000);
    assert_eq!(window.start, 20);
    assert_eq!(window.limit, FRONTEND_PAGE_SIZE_MAX);
}

#[test]
fn pagination_window_zero_offset() {
    let window = pagination_window(0, 10);
    assert_eq!(window.start, 0);
    assert_eq!(window.limit, 10);
}

/// Edge case: offset at u32::MAX — saturating add must not overflow.
#[test]
fn pagination_window_offset_at_max_does_not_overflow() {
    let window = pagination_window(u32::MAX, 50);
    assert_eq!(window.start, u32::MAX);
    assert_eq!(window.limit, 50);
    assert_eq!(window.start.saturating_add(window.limit), u32::MAX);
}

/// Zero requested limit → clamped to FRONTEND_PAGE_SIZE_MIN.
#[test]
fn pagination_window_zero_limit_clamped_to_min() {
    let window = pagination_window(5, 0);
    assert_eq!(window.limit, FRONTEND_PAGE_SIZE_MIN);
}

// ── validate_upgrade_note ─────────────────────────────────────────────────────

#[test]
fn upgrade_note_validation_accepts_short_note() {
    let env = make_env();
    let ok = String::from_str(&env, "ok");
    assert!(validate_upgrade_note(&ok));
}

/// Exact boundary (len == UPGRADE_NOTE_MAX_LEN) → valid.
#[test]
fn upgrade_note_validation_exact_boundary_is_valid() {
    let env = make_env();
    let exact = make_string(&env, UPGRADE_NOTE_MAX_LEN, b'a');
    assert!(validate_upgrade_note(&exact));
}

/// One byte over the boundary → invalid.
#[test]
fn upgrade_note_validation_one_over_boundary_is_invalid() {
    let env = make_env();
    let long = make_string(&env, UPGRADE_NOTE_MAX_LEN + 1, b'a');
    assert!(!validate_upgrade_note(&long));
}

// ── build_sdk_change_record ───────────────────────────────────────────────────

/// Non-breaking record stores correct fields.
#[test]
fn build_sdk_change_record_non_breaking() {
    let env = make_env();
    let desc = String::from_str(&env, "register API changed");
    let record = build_sdk_change_record(&env, "register_api", false, desc.clone());
    assert!(!record.is_breaking);
    assert_eq!(record.description, desc);
}

/// Breaking record stores correct fields.
#[test]
fn build_sdk_change_record_breaking() {
    let env = make_env();
    let desc = String::from_str(&env, "storage key type changed");
    let record = build_sdk_change_record(&env, "storage_key", true, desc.clone());
    assert!(record.is_breaking);
    assert_eq!(record.description, desc);
}

/// Two records with different ids are distinct.
#[test]
fn build_sdk_change_record_distinct_ids() {
    let env = make_env();
    let desc_a = String::from_str(&env, "change A");
    let desc_b = String::from_str(&env, "change B");
    let rec_a = build_sdk_change_record(&env, "change_a", false, desc_a);
    let rec_b = build_sdk_change_record(&env, "change_b", true, desc_b);
    // Symbols are equal only when the string is the same.
    assert_ne!(rec_a.id, rec_b.id);
    assert_ne!(rec_a.is_breaking, rec_b.is_breaking);
}

// ── emit_upgrade_audit_event ──────────────────────────────────────────────────

#[test]
fn emit_audit_event_does_not_panic() {
    let env = make_env();
    emit_upgrade_audit_event(
        &env,
        String::from_str(&env, "22.0.0"),
        String::from_str(&env, "22.0.1"),
        Address::generate(&env),
    );
}

// ── emit_upgrade_audit_event_with_note ───────────────────────────────────────

#[test]
fn emit_audit_event_with_note_does_not_panic_for_valid_note() {
    let env = make_env();
    emit_upgrade_audit_event_with_note(
        &env,
        String::from_str(&env, "22.0.0"),
        String::from_str(&env, "22.0.1"),
        Address::generate(&env),
        String::from_str(&env, "frontend and indexer verified"),
    );
}

#[test]
#[should_panic(expected = "upgrade note exceeds UPGRADE_NOTE_MAX_LEN")]
fn emit_audit_event_with_note_panics_when_note_too_long() {
    let env = make_env();
    let long = make_string(&env, UPGRADE_NOTE_MAX_LEN + 1, b'x');
    emit_upgrade_audit_event_with_note(
        &env,
        String::from_str(&env, "22.0.0"),
        String::from_str(&env, "22.0.1"),
        Address::generate(&env),
        long,
    );
}

/// Exact-boundary note (len == max) must not panic.
#[test]
fn emit_audit_event_with_note_exact_boundary_does_not_panic() {
    let env = make_env();
    let exact = make_string(&env, UPGRADE_NOTE_MAX_LEN, b'n');
    emit_upgrade_audit_event_with_note(
        &env,
        String::from_str(&env, "22.0.0"),
        String::from_str(&env, "22.1.0"),
        Address::generate(&env),
        exact,
    );
}

// ── emit_ping_event ───────────────────────────────────────────────────────────

/// emit_ping_event succeeds when auth is mocked.
#[test]
fn emit_ping_event_succeeds_with_mocked_auth() {
    let env = make_env();
    let from = Address::generate(&env);
    // mock_all_auths satisfies require_auth() — no panic expected.
    emit_ping_event(&env, from, 42_i32);
}

/// emit_ping_event with value 0 succeeds.
#[test]
fn emit_ping_event_zero_value() {
    let env = make_env();
    let from = Address::generate(&env);
    emit_ping_event(&env, from, 0_i32);
}

/// emit_ping_event with negative value succeeds (i32 is signed).
#[test]
fn emit_ping_event_negative_value() {
    let env = make_env();
    let from = Address::generate(&env);
    emit_ping_event(&env, from, -1_i32);
}

/// emit_ping_event with i32::MAX succeeds.
#[test]
fn emit_ping_event_max_value() {
    let env = make_env();
    let from = Address::generate(&env);
    emit_ping_event(&env, from, i32::MAX);
}

/// emit_ping_event panics without auth (no mock).
#[test]
#[should_panic]
fn emit_ping_event_panics_without_auth() {
    let env = Env::default(); // no mock_all_auths
    let from = Address::generate(&env);
    emit_ping_event(&env, from, 1_i32);
}

/// Two different callers can each emit independently.
#[test]
fn emit_ping_event_multiple_callers() {
    let env = make_env();
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    emit_ping_event(&env, a, 1_i32);
    emit_ping_event(&env, b, 2_i32);
}

// ── Integration: safe vs unsafe upgrade paths ─────────────────────────────────

/// Safe path: same major, valid hash, is a minor bump.
#[test]
fn safe_upgrade_path_all_checks_pass() {
    let env = make_env();
    let mut bytes = [0u8; 32];
    bytes[0] = 0xAB;
    let hash = BytesN::from_array(&env, &bytes);

    assert_eq!(
        assess_compatibility(&env, "22.0.0", "22.1.0"),
        CompatibilityStatus::Compatible
    );
    assert!(validate_wasm_hash(&hash));
    assert!(is_minor_bump("22.0.0", "22.1.0"));
}

/// Unsafe path: cross-major + zero hash.
#[test]
fn unsafe_upgrade_path_all_checks_fail() {
    let env = make_env();
    let hash = BytesN::from_array(&env, &[0u8; 32]);

    assert_eq!(
        assess_compatibility(&env, "22.0.0", "23.0.0"),
        CompatibilityStatus::RequiresMigration
    );
    assert!(!validate_wasm_hash(&hash));
    assert!(!is_minor_bump("22.0.0", "23.0.0"));
}

/// Partial failure: compatible versions but zero hash.
#[test]
fn compatible_versions_but_zero_hash_fails() {
    let env = make_env();
    let hash = BytesN::from_array(&env, &[0u8; 32]);

    assert_eq!(
        assess_compatibility(&env, "22.0.0", "22.1.0"),
        CompatibilityStatus::Compatible
    );
    assert!(!validate_wasm_hash(&hash));
}

/// Partial failure: valid hash but empty version string.
#[test]
fn valid_hash_but_empty_version_is_incompatible() {
    let env = make_env();
    let mut bytes = [0u8; 32];
    bytes[0] = 1;
    let hash = BytesN::from_array(&env, &bytes);

    assert_eq!(
        assess_compatibility(&env, "", "22.1.0"),
        CompatibilityStatus::Incompatible
    );
    assert!(validate_wasm_hash(&hash));
}

/// Full safe-upgrade flow: build record, assess, validate hash, emit event.
#[test]
fn full_safe_upgrade_flow() {
    let env = make_env();
    let reviewer = Address::generate(&env);

    // 1. Build a change record for the register API change.
    let record = build_sdk_change_record(
        &env,
        "register_api",
        false,
        String::from_str(&env, "env.register(Contract, ()) replaces register_contract"),
    );
    assert!(!record.is_breaking);

    // 2. Assess compatibility.
    assert_eq!(
        assess_compatibility(&env, "22.0.0", "22.1.0"),
        CompatibilityStatus::Compatible
    );

    // 3. Validate WASM hash.
    let mut bytes = [0u8; 32];
    bytes[0] = 0xDE;
    let hash = BytesN::from_array(&env, &bytes);
    assert!(validate_wasm_hash(&hash));

    // 4. Emit audit event with note.
    emit_upgrade_audit_event_with_note(
        &env,
        String::from_str(&env, "22.0.0"),
        String::from_str(&env, "22.1.0"),
        reviewer,
        String::from_str(&env, "all checks passed"),
    );
}

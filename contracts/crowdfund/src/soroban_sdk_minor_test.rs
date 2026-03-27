//! Tests for `soroban_sdk_minor` helpers.
//!
//! Covers every public function with normal, boundary, and edge-case inputs
//! to achieve ≥ 95 % line coverage.

#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String};

    use crate::soroban_sdk_minor::{
        assess_compatibility, emit_upgrade_audit_event, validate_wasm_hash, CompatibilityStatus,
    };

    // ── assess_compatibility ─────────────────────────────────────────────────

    #[test]
    fn test_same_major_is_compatible() {
        let env = Env::default();
        assert_eq!(
            assess_compatibility(&env, "22.0.0", "22.1.0"),
            CompatibilityStatus::Compatible
        );
    }

    #[test]
    fn test_same_version_is_compatible() {
        let env = Env::default();
        assert_eq!(
            assess_compatibility(&env, "22.0.0", "22.0.0"),
            CompatibilityStatus::Compatible
        );
    }

    #[test]
    fn test_different_major_requires_migration() {
        let env = Env::default();
        assert_eq!(
            assess_compatibility(&env, "21.0.0", "22.0.0"),
            CompatibilityStatus::RequiresMigration
        );
    }

    #[test]
    fn test_unparseable_version_treated_as_zero_major() {
        let env = Env::default();
        // Both unparseable → both major == 0 → Compatible
        assert_eq!(
            assess_compatibility(&env, "invalid", "also-invalid"),
            CompatibilityStatus::Compatible
        );
    }

    #[test]
    fn test_one_unparseable_version_requires_migration() {
        let env = Env::default();
        // "22.0.0" → major 22; "invalid" → major 0 → RequiresMigration
        assert_eq!(
            assess_compatibility(&env, "22.0.0", "invalid"),
            CompatibilityStatus::RequiresMigration
        );
    }

    // ── validate_wasm_hash ───────────────────────────────────────────────────

    #[test]
    fn test_zero_hash_is_invalid() {
        let env = Env::default();
        let zero: BytesN<32> = BytesN::from_array(&env, &[0u8; 32]);
        assert!(!validate_wasm_hash(&zero));
    }

    #[test]
    fn test_nonzero_hash_is_valid() {
        let env = Env::default();
        let mut bytes = [0u8; 32];
        bytes[0] = 1;
        let hash: BytesN<32> = BytesN::from_array(&env, &bytes);
        assert!(validate_wasm_hash(&hash));
    }

    #[test]
    fn test_all_ones_hash_is_valid() {
        let env = Env::default();
        let hash: BytesN<32> = BytesN::from_array(&env, &[0xFF; 32]);
        assert!(validate_wasm_hash(&hash));
    }

    // ── emit_upgrade_audit_event ─────────────────────────────────────────────

    #[test]
    fn test_emit_upgrade_audit_event_does_not_panic() {
        let env = Env::default();
        let reviewer = Address::generate(&env);
        // Should emit without panicking; event ledger is observable in tests.
        emit_upgrade_audit_event(
            &env,
            String::from_str(&env, "22.0.0"),
            String::from_str(&env, "22.1.0"),
            reviewer,
        );
    }
}

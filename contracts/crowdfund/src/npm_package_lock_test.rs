//! Comprehensive test suite for the `npm_package_lock` module.
//!
//! ## Coverage targets (≥95%)
//! - `parse_semver`              — valid, edge-case, and invalid inputs
//! - `is_version_gte`            — boundary comparisons
//! - `validate_integrity`        — sha512 presence and format
//! - `audit_package`             — pass/fail scenarios per advisory
//! - `audit_all`                 — batch audit correctness
//! - `audit_all_bounded`         — bounds enforcement and error messages
//! - `failing_results`           — filter helper
//! - `validate_lockfile_version` — supported/unsupported versions
//!
//! ## Security notes
//! - Tests explicitly cover GHSA-xpqw-6gx7-v673 (svgo Billion Laughs DoS).
//! - Boundary tests ensure off-by-one errors in version comparisons are caught.
//! - Integrity tests guard against tampered or incomplete lockfile entries.

#[cfg(test)]
mod tests {
    extern crate std;
    use std::collections::HashMap;
    use std::format;
    use std::vec;

    use crate::npm_package_lock::{
        audit_all, audit_all_bounded, audit_package, failing_results, is_version_gte,
        parse_semver, validate_integrity, validate_lockfile_version, AuditResult, PackageEntry,
        MAX_PACKAGES,
    };

    // ── Helpers ──────────────────────────────────────────────────────────────

    /// Build the standard advisory map used across most tests.
    /// Contains the GHSA-xpqw-6gx7-v673 entry: svgo < 3.3.3 is vulnerable.
    fn safe_versions() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("svgo".to_string(), "3.3.3".to_string());
        m
    }

    fn make_entry(name: &str, version: &str, integrity: &str, dev: bool) -> PackageEntry {
        PackageEntry {
            name: name.to_string(),
            version: version.to_string(),
            integrity: integrity.to_string(),
            dev,
        }
    }

    /// A realistic sha512 integrity hash used as the "valid" value in tests.
    const VALID_HASH: &str =
        "sha512-OoohrmuUlBs8B8o6MB2Aevn+pRIH9zDALSR+6hhqVfa6fRwG/Qw9VUMSMW9VNg2CFc/MTIfabtdOVl9ODIJjpw==";

    // ── parse_semver ─────────────────────────────────────────────────────────

    #[test]
    fn test_parse_semver_standard() {
        assert_eq!(parse_semver("3.3.3"), Some((3, 3, 3)));
    }

    #[test]
    fn test_parse_semver_with_v_prefix() {
        assert_eq!(parse_semver("v1.2.3"), Some((1, 2, 3)));
    }

    #[test]
    fn test_parse_semver_with_prerelease() {
        // Pre-release suffix is stripped; numeric base is used
        assert_eq!(parse_semver("3.3.3-beta.1"), Some((3, 3, 3)));
    }

    #[test]
    fn test_parse_semver_zeros() {
        assert_eq!(parse_semver("0.0.0"), Some((0, 0, 0)));
    }

    #[test]
    fn test_parse_semver_large_numbers() {
        assert_eq!(parse_semver("100.200.300"), Some((100, 200, 300)));
    }

    #[test]
    fn test_parse_semver_missing_patch() {
        // Only two components — not valid semver
        assert_eq!(parse_semver("3.3"), None);
    }

    #[test]
    fn test_parse_semver_empty_string() {
        assert_eq!(parse_semver(""), None);
    }

    #[test]
    fn test_parse_semver_non_numeric() {
        assert_eq!(parse_semver("a.b.c"), None);
    }

    #[test]
    fn test_parse_semver_partial_numeric() {
        assert_eq!(parse_semver("1.x.0"), None);
    }

    // ── is_version_gte ───────────────────────────────────────────────────────

    #[test]
    fn test_version_gte_equal() {
        assert!(is_version_gte("3.3.3", "3.3.3"));
    }

    #[test]
    fn test_version_gte_greater_patch() {
        assert!(is_version_gte("3.3.4", "3.3.3"));
    }

    #[test]
    fn test_version_gte_greater_minor() {
        assert!(is_version_gte("3.4.0", "3.3.3"));
    }

    #[test]
    fn test_version_gte_greater_major() {
        assert!(is_version_gte("4.0.0", "3.3.3"));
    }

    #[test]
    fn test_version_gte_less_patch() {
        // 3.3.2 is the last vulnerable svgo version (GHSA-xpqw-6gx7-v673)
        assert!(!is_version_gte("3.3.2", "3.3.3"));
    }

    #[test]
    fn test_version_gte_less_minor() {
        assert!(!is_version_gte("3.2.9", "3.3.3"));
    }

    #[test]
    fn test_version_gte_less_major() {
        assert!(!is_version_gte("2.9.9", "3.3.3"));
    }

    #[test]
    fn test_version_gte_invalid_version() {
        // Unparseable version conservatively returns false
        assert!(!is_version_gte("invalid", "3.3.3"));
    }

    #[test]
    fn test_version_gte_invalid_min() {
        assert!(!is_version_gte("3.3.3", "invalid"));
    }

    // ── validate_integrity ───────────────────────────────────────────────────

    #[test]
    fn test_integrity_valid_sha512() {
        assert!(validate_integrity(VALID_HASH));
    }

    #[test]
    fn test_integrity_empty_string() {
        assert!(!validate_integrity(""));
    }

    #[test]
    fn test_integrity_wrong_algorithm_sha256() {
        assert!(!validate_integrity("sha256-abc123"));
    }

    #[test]
    fn test_integrity_wrong_algorithm_sha1() {
        assert!(!validate_integrity("sha1-abc123"));
    }

    #[test]
    fn test_integrity_sha512_prefix_only() {
        // Prefix present but no actual hash — still passes prefix check
        assert!(validate_integrity("sha512-"));
    }

    #[test]
    fn test_integrity_no_prefix() {
        assert!(!validate_integrity("abc123def456"));
    }

    // ── audit_package — GHSA-xpqw-6gx7-v673 (svgo Billion Laughs DoS) ──────

    #[test]
    fn test_audit_svgo_vulnerable_version_fails() {
        // svgo 3.3.2 is in the vulnerable range (>=3.0.0 <3.3.3)
        let entry = make_entry("svgo", "3.3.2", VALID_HASH, true);
        let result = audit_package(&entry, &safe_versions());
        assert!(!result.passed);
        assert!(result.issues.iter().any(|i| i.contains("3.3.2")));
        assert!(result.issues.iter().any(|i| i.contains("3.3.3")));
    }

    #[test]
    fn test_audit_svgo_patched_version_passes() {
        // svgo 3.3.3 is the first patched release
        let entry = make_entry("svgo", "3.3.3", VALID_HASH, true);
        let result = audit_package(&entry, &safe_versions());
        assert!(result.passed);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_audit_svgo_newer_version_passes() {
        let entry = make_entry("svgo", "3.4.0", VALID_HASH, true);
        let result = audit_package(&entry, &safe_versions());
        assert!(result.passed);
    }

    #[test]
    fn test_audit_svgo_oldest_vulnerable_version_fails() {
        // 3.0.0 is the start of the vulnerable range
        let entry = make_entry("svgo", "3.0.0", VALID_HASH, true);
        let result = audit_package(&entry, &safe_versions());
        assert!(!result.passed);
    }

    #[test]
    fn test_audit_invalid_integrity_fails() {
        let entry = make_entry("svgo", "3.3.3", "", true);
        let result = audit_package(&entry, &safe_versions());
        assert!(!result.passed);
        assert!(result.issues.iter().any(|i| i.contains("integrity")));
    }

    #[test]
    fn test_audit_both_version_and_integrity_fail() {
        // Both checks fail — issues list should have exactly 2 entries
        let entry = make_entry("svgo", "3.3.2", "", true);
        let result = audit_package(&entry, &safe_versions());
        assert!(!result.passed);
        assert_eq!(result.issues.len(), 2);
    }

    #[test]
    fn test_audit_unknown_package_passes_version_check() {
        // Package not in the advisory map — no version constraint applied
        let entry = make_entry("some-unknown-pkg", "1.0.0", VALID_HASH, false);
        let result = audit_package(&entry, &safe_versions());
        assert!(result.passed);
    }

    #[test]
    fn test_audit_result_contains_package_name() {
        let entry = make_entry("svgo", "3.3.2", VALID_HASH, true);
        let result = audit_package(&entry, &safe_versions());
        assert_eq!(result.package_name, "svgo");
    }

    #[test]
    fn test_audit_dev_dependency_audited_same_as_prod() {
        // dev flag does not affect audit logic
        let prod = make_entry("svgo", "3.3.2", VALID_HASH, false);
        let dev = make_entry("svgo", "3.3.2", VALID_HASH, true);
        assert_eq!(
            audit_package(&prod, &safe_versions()).passed,
            audit_package(&dev, &safe_versions()).passed
        );
    }

    // ── audit_all ────────────────────────────────────────────────────────────

    #[test]
    fn test_audit_all_mixed_results() {
        let packages = vec![
            make_entry("svgo", "3.3.2", VALID_HASH, true),  // fails
            make_entry("svgo", "3.3.3", VALID_HASH, true),  // passes
            make_entry("jest", "30.3.0", VALID_HASH, true), // passes (not in map)
        ];
        let results = audit_all(&packages, &safe_versions());
        assert_eq!(results.len(), 3);
        assert!(!results[0].passed);
        assert!(results[1].passed);
        assert!(results[2].passed);
    }

    #[test]
    fn test_audit_all_empty_input() {
        let results = audit_all(&[], &safe_versions());
        assert!(results.is_empty());
    }

    #[test]
    fn test_audit_all_all_pass() {
        let packages = vec![
            make_entry("svgo", "3.3.3", VALID_HASH, true),
            make_entry("jest", "30.3.0", VALID_HASH, true),
        ];
        let results = audit_all(&packages, &safe_versions());
        assert!(results.iter().all(|r| r.passed));
    }

    // ── failing_results ──────────────────────────────────────────────────────

    #[test]
    fn test_failing_results_filters_correctly() {
        let packages = vec![
            make_entry("svgo", "3.3.2", VALID_HASH, true),
            make_entry("svgo", "3.3.3", VALID_HASH, true),
        ];
        let results = audit_all(&packages, &safe_versions());
        let failures = failing_results(&results);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].package_name, "svgo");
    }

    #[test]
    fn test_failing_results_empty_when_all_pass() {
        let packages = vec![make_entry("svgo", "3.3.3", VALID_HASH, true)];
        let results = audit_all(&packages, &safe_versions());
        assert!(failing_results(&results).is_empty());
    }

    // ── validate_lockfile_version ────────────────────────────────────────────

    #[test]
    fn test_lockfile_version_2_valid() {
        assert!(validate_lockfile_version(2));
    }

    #[test]
    fn test_lockfile_version_3_valid() {
        assert!(validate_lockfile_version(3));
    }

    #[test]
    fn test_lockfile_version_1_invalid() {
        // npm <7 lockfiles lack integrity hashes — rejected
        assert!(!validate_lockfile_version(1));
    }

    #[test]
    fn test_lockfile_version_0_invalid() {
        assert!(!validate_lockfile_version(0));
    }

    #[test]
    fn test_lockfile_version_4_invalid() {
        assert!(!validate_lockfile_version(4));
    }

    // ── audit_all_bounded ────────────────────────────────────────────────────

    #[test]
    fn test_bounded_within_limit_returns_ok() {
        let packages = vec![make_entry("svgo", "3.3.3", VALID_HASH, true)];
        assert!(audit_all_bounded(&packages, &safe_versions()).is_ok());
    }

    #[test]
    fn test_bounded_empty_input_returns_ok() {
        assert!(audit_all_bounded(&[], &safe_versions()).is_ok());
    }

    #[test]
    fn test_bounded_results_match_audit_all() {
        let packages = vec![
            make_entry("svgo", "3.3.2", VALID_HASH, true),
            make_entry("svgo", "3.3.3", VALID_HASH, true),
        ];
        let bounded = audit_all_bounded(&packages, &safe_versions()).unwrap();
        let unbounded = audit_all(&packages, &safe_versions());
        assert_eq!(bounded, unbounded);
    }

    #[test]
    fn test_bounded_exactly_at_limit_returns_ok() {
        let packages: Vec<_> = (0..MAX_PACKAGES)
            .map(|i| make_entry(&format!("pkg-{}", i), "1.0.0", VALID_HASH, false))
            .collect();
        assert!(audit_all_bounded(&packages, &safe_versions()).is_ok());
    }

    #[test]
    fn test_bounded_one_over_limit_returns_err() {
        let packages: Vec<_> = (0..=MAX_PACKAGES)
            .map(|i| make_entry(&format!("pkg-{}", i), "1.0.0", VALID_HASH, false))
            .collect();
        let err = audit_all_bounded(&packages, &safe_versions()).unwrap_err();
        assert!(err.contains("MAX_PACKAGES"));
        assert!(err.contains(&MAX_PACKAGES.to_string()));
    }

    #[test]
    fn test_bounded_error_message_contains_actual_count() {
        let count = MAX_PACKAGES + 10;
        let packages: Vec<_> = (0..count)
            .map(|i| make_entry(&format!("pkg-{}", i), "1.0.0", VALID_HASH, false))
            .collect();
        let err = audit_all_bounded(&packages, &safe_versions()).unwrap_err();
        assert!(err.contains(&count.to_string()));
    }

    #[test]
    fn test_max_packages_constant_is_positive() {
        assert!(MAX_PACKAGES > 0);
    }

    // ── audit_all_bounded Tests ──────────────────────────────────────────────────

    #[test]
    fn test_audit_all_bounded_within_limit_returns_ok() {
        let env = Env::default();
        let mut packages = Vec::new(&env);
        packages.push_back(create_entry("svgo", "3.3.3", "sha512-abc123", true));
        let advisories = create_advisory_map(vec![("svgo", "3.3.3")]);
        assert!(audit_all_bounded(&packages, &advisories).is_ok());
    }

    #[test]
    fn test_audit_all_bounded_empty_input_returns_ok() {
        let env = Env::default();
        let packages = Vec::new(&env);
        let advisories = create_advisory_map(vec![]);
        assert!(audit_all_bounded(&packages, &advisories).is_ok());
    }

    #[test]
    fn test_audit_all_bounded_results_match_audit_all() {
        let env = Env::default();
        let mut packages = Vec::new(&env);
        packages.push_back(create_entry("svgo", "3.3.2", "sha512-abc123", true));
        packages.push_back(create_entry("svgo", "3.3.3", "sha512-abc123", true));
        let advisories = create_advisory_map(vec![("svgo", "3.3.3")]);

        let bounded = audit_all_bounded(&packages, &advisories).unwrap();
        let unbounded = audit_all(&packages, &advisories);
        assert_eq!(bounded.len(), unbounded.len());
        for i in 0..bounded.len() {
            assert_eq!(
                bounded.get(i).unwrap().passed,
                unbounded.get(i).unwrap().passed
            );
        }
    }

    #[test]
    fn test_audit_all_bounded_over_limit_returns_err() {
        let env = Env::default();
        let mut packages = Vec::new(&env);
        // MAX_PACKAGES is 500; push 501 entries
        for i in 0u32..501 {
            packages.push_back(create_entry(
                &format!("pkg-{}", i),
                "1.0.0",
                "sha512-abc123",
                false,
            ));
        }
        let advisories = create_advisory_map(vec![]);
        assert!(audit_all_bounded(&packages, &advisories).is_err());
    }

    #[test]
    fn test_audit_all_bounded_error_message_is_descriptive() {
        let env = Env::default();
        let mut packages = Vec::new(&env);
        for i in 0u32..501 {
            packages.push_back(create_entry(
                &format!("pkg-{}", i),
                "1.0.0",
                "sha512-abc123",
                false,
            ));
        }
        let advisories = create_advisory_map(vec![]);
        let err = audit_all_bounded(&packages, &advisories).unwrap_err();
        assert!(err.contains("MAX_PACKAGES"));
    }
}

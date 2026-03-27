/// # soroban_sdk_minor tests
///
/// @title   SorobanSdkMinorAuditor — Test Suite
/// @notice  Comprehensive tests for the script-layer SDK v22 migration auditor.
/// @dev     Self-contained: includes the contract source via `#[path]`.
///          Run: `rustc --test scripts/soroban_sdk_minor.test.rs`
///
/// ## Coverage targets
/// - `parse_semver`           — valid, v-prefix, pre-release, edge cases
/// - `is_version_gte`         — boundary comparisons
/// - `is_sdk_v22_compatible`  — version gate
/// - `scan_source`            — clean file, single/multiple findings, line numbers
/// - `scan_all`               — batch scan, deterministic order
/// - `dirty_results`          — filter helper
///
/// ## Security notes
/// - Tests confirm that both deprecated patterns are detected independently
///   and together on the same line.
/// - Boundary tests ensure off-by-one errors in version comparison are caught.

#[path = "soroban_sdk_minor.rs"]
mod soroban_sdk_minor;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use soroban_sdk_minor::{
        dirty_results, is_sdk_v22_compatible, is_version_gte, parse_semver, scan_all,
        scan_source, MIN_SDK_VERSION,
    };

    // -----------------------------------------------------------------------
    // parse_semver
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_semver_standard() {
        assert_eq!(parse_semver("22.0.11"), Some((22, 0, 11)));
    }

    #[test]
    fn test_parse_semver_v_prefix() {
        assert_eq!(parse_semver("v22.0.0"), Some((22, 0, 0)));
    }

    #[test]
    fn test_parse_semver_prerelease_stripped() {
        assert_eq!(parse_semver("22.0.0-beta.1"), Some((22, 0, 0)));
    }

    #[test]
    fn test_parse_semver_zeros() {
        assert_eq!(parse_semver("0.0.0"), Some((0, 0, 0)));
    }

    #[test]
    fn test_parse_semver_missing_patch() {
        assert_eq!(parse_semver("22.0"), None);
    }

    #[test]
    fn test_parse_semver_empty() {
        assert_eq!(parse_semver(""), None);
    }

    #[test]
    fn test_parse_semver_non_numeric() {
        assert_eq!(parse_semver("a.b.c"), None);
    }

    // -----------------------------------------------------------------------
    // is_version_gte
    // -----------------------------------------------------------------------

    #[test]
    fn test_gte_equal() {
        assert!(is_version_gte("22.0.0", "22.0.0"));
    }

    #[test]
    fn test_gte_greater_patch() {
        assert!(is_version_gte("22.0.11", "22.0.0"));
    }

    #[test]
    fn test_gte_greater_minor() {
        assert!(is_version_gte("22.1.0", "22.0.11"));
    }

    #[test]
    fn test_gte_greater_major() {
        assert!(is_version_gte("23.0.0", "22.0.11"));
    }

    #[test]
    fn test_gte_less_patch() {
        assert!(!is_version_gte("21.9.9", "22.0.0"));
    }

    #[test]
    fn test_gte_invalid_version() {
        assert!(!is_version_gte("invalid", "22.0.0"));
    }

    #[test]
    fn test_gte_invalid_min() {
        assert!(!is_version_gte("22.0.0", "invalid"));
    }

    // -----------------------------------------------------------------------
    // is_sdk_v22_compatible
    // -----------------------------------------------------------------------

    #[test]
    fn test_sdk_v22_exact_min_passes() {
        assert!(is_sdk_v22_compatible(MIN_SDK_VERSION));
    }

    #[test]
    fn test_sdk_v22_newer_passes() {
        assert!(is_sdk_v22_compatible("22.0.11"));
    }

    #[test]
    fn test_sdk_v21_fails() {
        assert!(!is_sdk_v22_compatible("21.9.9"));
    }

    #[test]
    fn test_sdk_invalid_fails() {
        assert!(!is_sdk_v22_compatible("not-a-version"));
    }

    // -----------------------------------------------------------------------
    // scan_source — clean file
    // -----------------------------------------------------------------------

    #[test]
    fn test_scan_clean_source_is_clean() {
        let src = r#"
            env.register(MyContract, ());
            admin.require_auth();
        "#;
        let result = scan_source("lib.rs", src);
        assert!(result.clean);
        assert!(result.findings.is_empty());
        assert_eq!(result.file, "lib.rs");
    }

    // -----------------------------------------------------------------------
    // scan_source — deprecated register_contract
    // -----------------------------------------------------------------------

    #[test]
    fn test_scan_detects_register_contract() {
        let src = "let id = env.register_contract(None, MyContract);";
        let result = scan_source("test.rs", src);
        assert!(!result.clean);
        assert_eq!(result.findings.len(), 1);
        assert!(result.findings[0].1.contains("env.register"));
    }

    #[test]
    fn test_scan_register_contract_line_number_correct() {
        let src = "// line 1\nenv.register_contract(None, X);\n// line 3";
        let result = scan_source("test.rs", src);
        assert_eq!(result.findings[0].0, 2);
    }

    // -----------------------------------------------------------------------
    // scan_source — deprecated Symbol::new
    // -----------------------------------------------------------------------

    #[test]
    fn test_scan_detects_symbol_new() {
        let src = r#"let key = Symbol::new(&env, "admin");"#;
        let result = scan_source("lib.rs", src);
        assert!(!result.clean);
        assert_eq!(result.findings.len(), 1);
        assert!(result.findings[0].1.contains("contracttype"));
    }

    #[test]
    fn test_scan_symbol_new_line_number_correct() {
        let src = "// ok\n// ok\nlet k = Symbol::new(&env, \"x\");";
        let result = scan_source("lib.rs", src);
        assert_eq!(result.findings[0].0, 3);
    }

    // -----------------------------------------------------------------------
    // scan_source — multiple findings
    // -----------------------------------------------------------------------

    #[test]
    fn test_scan_multiple_deprecated_patterns() {
        let src = "env.register_contract(None, C);\nSymbol::new(&env, \"k\");";
        let result = scan_source("lib.rs", src);
        assert!(!result.clean);
        assert_eq!(result.findings.len(), 2);
    }

    #[test]
    fn test_scan_both_patterns_on_same_line() {
        // Contrived but ensures both patterns are checked per line.
        let src = r#"env.register_contract(None, C); let k = Symbol::new(&env, "x");"#;
        let result = scan_source("lib.rs", src);
        assert_eq!(result.findings.len(), 2);
    }

    #[test]
    fn test_scan_empty_source_is_clean() {
        let result = scan_source("empty.rs", "");
        assert!(result.clean);
    }

    // -----------------------------------------------------------------------
    // scan_all
    // -----------------------------------------------------------------------

    #[test]
    fn test_scan_all_returns_one_result_per_file() {
        let mut sources = HashMap::new();
        sources.insert("a.rs".to_string(), "env.register(C, ());".to_string());
        sources.insert("b.rs".to_string(), "env.register_contract(None, C);".to_string());
        let results = scan_all(&sources);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_scan_all_sorted_by_filename() {
        let mut sources = HashMap::new();
        sources.insert("z.rs".to_string(), "".to_string());
        sources.insert("a.rs".to_string(), "".to_string());
        sources.insert("m.rs".to_string(), "".to_string());
        let results = scan_all(&sources);
        assert_eq!(results[0].file, "a.rs");
        assert_eq!(results[1].file, "m.rs");
        assert_eq!(results[2].file, "z.rs");
    }

    #[test]
    fn test_scan_all_empty_map() {
        let results = scan_all(&HashMap::new());
        assert!(results.is_empty());
    }

    // -----------------------------------------------------------------------
    // dirty_results
    // -----------------------------------------------------------------------

    #[test]
    fn test_dirty_results_filters_correctly() {
        let mut sources = HashMap::new();
        sources.insert("clean.rs".to_string(), "env.register(C, ());".to_string());
        sources.insert("dirty.rs".to_string(), "env.register_contract(None, C);".to_string());
        let results = scan_all(&sources);
        let dirty = dirty_results(&results);
        assert_eq!(dirty.len(), 1);
        assert_eq!(dirty[0].file, "dirty.rs");
    }

    #[test]
    fn test_dirty_results_empty_when_all_clean() {
        let mut sources = HashMap::new();
        sources.insert("a.rs".to_string(), "env.register(C, ());".to_string());
        let results = scan_all(&sources);
        assert!(dirty_results(&results).is_empty());
    }

    #[test]
    fn test_dirty_results_all_dirty() {
        let mut sources = HashMap::new();
        sources.insert("a.rs".to_string(), "env.register_contract(None, C);".to_string());
        sources.insert("b.rs".to_string(), "Symbol::new(&env, \"k\");".to_string());
        let results = scan_all(&sources);
        assert_eq!(dirty_results(&results).len(), 2);
    }

    // -----------------------------------------------------------------------
    // MIN_SDK_VERSION constant
    // -----------------------------------------------------------------------

    #[test]
    fn test_min_sdk_version_is_parseable() {
        assert!(parse_semver(MIN_SDK_VERSION).is_some());
    }

    #[test]
    fn test_min_sdk_version_is_v22() {
        let (major, _, _) = parse_semver(MIN_SDK_VERSION).unwrap();
        assert_eq!(major, 22);
    }
}

/// # soroban_sdk_minor
///
/// @title   SorobanSdkMinorAuditor
/// @notice  Script-layer utility for detecting deprecated Soroban SDK v21
///          patterns and validating that contracts have migrated to v22.
/// @dev     Pure functions — no Soroban runtime dependency. Designed for use
///          in CI scripts and pre-commit hooks to enforce SDK migration.
///
/// ## What changed in v22
/// | Area              | Deprecated (v21)                          | Required (v22)                  |
/// |-------------------|-------------------------------------------|---------------------------------|
/// | Test registration | `env.register_contract(None, Contract)`   | `env.register(Contract, ())`    |
/// | Storage keys      | Raw `String` / `Symbol::new` keys         | `#[contracttype]` enum keys     |
/// | Auth pattern      | Manual auth checks                        | `address.require_auth()`        |
///
/// ## Security Assumptions
/// - Deprecated patterns are detected by source-text heuristics; a clean
///   scan does not guarantee absence of all anti-patterns.
/// - Version comparison uses semver tuple ordering — no external crates.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// @notice Minimum Soroban SDK version that supports all v22 patterns.
pub const MIN_SDK_VERSION: &str = "22.0.0";

/// @notice Source patterns that indicate deprecated v21 usage.
/// @dev    Each entry is a (pattern, human-readable description) pair.
pub const DEPRECATED_PATTERNS: &[(&str, &str)] = &[
    (
        "register_contract(",
        "use `env.register(Contract, ())` instead of `env.register_contract`",
    ),
    (
        "Symbol::new(",
        "use `Symbol::short` or a `#[contracttype]` key instead of `Symbol::new`",
    ),
];

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// @notice Result of scanning a single source file for deprecated patterns.
#[derive(Debug, Clone, PartialEq)]
pub struct ScanResult {
    /// File path or identifier that was scanned.
    pub file: String,
    /// True when no deprecated patterns were found.
    pub clean: bool,
    /// List of (line_number, description) for each finding.
    pub findings: Vec<(usize, String)>,
}

/// @notice Parsed semver tuple.
pub type Semver = (u64, u64, u64);

// ---------------------------------------------------------------------------
// Semver helpers (duplicated from npm_package_lock for script independence)
// ---------------------------------------------------------------------------

/// @notice Parses a semver string into (major, minor, patch).
/// @param  version  e.g. "22.0.11"
/// @return Some((major, minor, patch)) or None on parse failure.
pub fn parse_semver(version: &str) -> Option<Semver> {
    let v = version.trim_start_matches('v');
    let base = v.split('-').next().unwrap_or(v);
    let parts: Vec<&str> = base.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

/// @notice Returns true if `version` >= `min_version`.
pub fn is_version_gte(version: &str, min_version: &str) -> bool {
    match (parse_semver(version), parse_semver(min_version)) {
        (Some(v), Some(m)) => v >= m,
        _ => false,
    }
}

/// @notice Returns true if the given SDK version satisfies the v22 minimum.
/// @param  sdk_version  The resolved soroban-sdk version string.
pub fn is_sdk_v22_compatible(sdk_version: &str) -> bool {
    is_version_gte(sdk_version, MIN_SDK_VERSION)
}

// ---------------------------------------------------------------------------
// Deprecated-pattern scanner
// ---------------------------------------------------------------------------

/// @notice Scans `source` line-by-line for deprecated v21 patterns.
/// @param  file    Identifier for the source (used in the result).
/// @param  source  Full source text to scan.
/// @return ScanResult with all findings.
pub fn scan_source(file: &str, source: &str) -> ScanResult {
    let mut findings: Vec<(usize, String)> = Vec::new();

    for (idx, line) in source.lines().enumerate() {
        for (pattern, description) in DEPRECATED_PATTERNS {
            if line.contains(pattern) {
                findings.push((idx + 1, description.to_string()));
            }
        }
    }

    ScanResult {
        file: file.to_string(),
        clean: findings.is_empty(),
        findings,
    }
}

/// @notice Scans multiple files and returns one ScanResult per file.
/// @param  sources  Map of file-name -> source-text.
pub fn scan_all(sources: &HashMap<String, String>) -> Vec<ScanResult> {
    let mut results: Vec<ScanResult> = sources
        .iter()
        .map(|(file, src)| scan_source(file, src))
        .collect();
    // Sort by file name for deterministic output in CI logs.
    results.sort_by(|a, b| a.file.cmp(&b.file));
    results
}

/// @notice Returns only the results that contain findings.
pub fn dirty_results(results: &[ScanResult]) -> Vec<&ScanResult> {
    results.iter().filter(|r| !r.clean).collect()
}

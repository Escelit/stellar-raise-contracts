//! # npm_package_lock
//!
//! @title   NpmPackageLockAuditor — Vulnerability audit module for package-lock.json entries.
//! @title   NPMPackageLock — Vulnerability audit module for package-lock.json entries.
//!
//! @notice  Audits `package-lock.json` dependency entries for known security
//!          vulnerabilities, version constraint violations, and integrity hash validity.
//!
//!          Introduced to address **GHSA-xpqw-6gx7-v673** — a high-severity
//!          Denial-of-Service vulnerability in `svgo` versions `>=3.0.0 <3.3.3`
//!          caused by unconstrained XML entity expansion (Billion Laughs attack)
//!          when processing SVG files containing a malicious `DOCTYPE` declaration.
//!          caused by unconstrained XML entity expansion (Billion Laughs attack).
//!
//! ## Security Assumptions
//!
//! 1. `sha512` integrity hashes are the only accepted algorithm; `sha1` and
//!    `sha256` are rejected as insufficient.
//! 2. `lockfileVersion` must be 2 or 3 (npm >=7). Version 1 lacks integrity
//!    hashes for all entries and is considered insecure.
//! 3. The advisory map (`min_safe_versions`) must be kept up to date as new
//!    CVEs are published. This module does not perform live advisory lookups.
//! 4. This module audits resolved versions only. Ranges in `package.json`
//!    should be reviewed separately to prevent future resolution of vulnerable
//!    versions.
//! 5. `audit_all_bounded` enforces a hard cap on input size to prevent
//!    unbounded processing (gas efficiency / DoS protection).
//!
//! @dev     All checks are pure functions operating on parsed data structs.

#![allow(dead_code)]

#![cfg(test)]
extern crate std;
use std::collections::HashMap;
use std::format;
use std::string::String;
use std::vec::Vec;

// ── Bounds ───────────────────────────────────────────────────────────────────

/// @notice Hard cap on the number of packages processed by `audit_all_bounded`.
/// @dev    Prevents unbounded iteration; mirrors gas-limit patterns in
///         on-chain contracts. Adjust upward only with a documented rationale.
pub const MAX_PACKAGES: usize = 500;

// ── Data Types ───────────────────────────────────────────────────────────────

/// Represents a single resolved package entry from `package-lock.json`.
///
/// @param name       Package name (e.g. "svgo")
/// @param version    Resolved semver string (e.g. "3.3.3")
/// @param integrity  sha512 hash string from the lockfile (e.g. "sha512-...")
/// @param dev        Whether the package is a devDependency
#[derive(Debug, Clone, PartialEq)]
pub struct PackageEntry {
    pub name: String,
    pub version: String,
    pub integrity: String,
    pub dev: bool,
}

/// Audit result for a single package entry.
///
/// @param package_name  Name of the audited package
/// @param passed        True if no issues were found
/// @param issues        List of human-readable issue descriptions (empty if passed)
#[derive(Debug, Clone, PartialEq)]
pub struct AuditResult {
    pub package_name: String,
    pub passed: bool,

#![allow(dead_code)]

use std::collections::HashMap;

// ── Bounds ───────────────────────────────────────────────────────────────────

/// @notice Hard cap on the number of packages processed by `audit_all_bounded`.
/// @dev    Prevents unbounded iteration — mirrors gas-limit patterns used in
///         on-chain contracts. Adjust upward only with a documented rationale.
pub const MAX_PACKAGES: u32 = 500;

// ── Constants ────────────────────────────────────────────────────────────────

/// @notice Hard cap on the number of packages processed by `audit_all_bounded`.
/// @dev    Prevents unbounded iteration; mirrors gas-limit patterns in
///         on-chain contracts. Adjust upward only with a documented rationale.
pub const MAX_PACKAGES: usize = 500;

// ── Data Types ───────────────────────────────────────────────────────────────

/// Represents a single resolved package entry from `package-lock.json`.
///
/// @param name       Package name (e.g. "svgo")
/// @param version    Resolved semver string (e.g. "3.3.3")
/// @param integrity  sha512 hash string from the lockfile (e.g. "sha512-...")
/// @param dev        Whether the package is a devDependency
#[derive(Debug, Clone, PartialEq)]
pub struct PackageEntry {
    pub name: String,
    pub version: String,
    pub integrity: String,
    pub dev: bool,
}

/// Audit result for a single package entry.
///
/// @param package_name  Name of the audited package
/// @param passed        True if no issues were found
/// @param issues        List of human-readable issue descriptions (empty if passed)
#[derive(Debug, Clone, PartialEq)]
pub struct AuditResult {
    pub package_name: String,
    pub passed: bool,
    pub issues: Vec<String>,
}

// ── Semver Parsing ───────────────────────────────────────────────────────────

/// @notice Parse a semantic version string into (major, minor, patch) tuple.
///
/// @dev    Strips an optional leading "v" prefix and any pre-release suffix
///         (everything after the first "-"). Returns None on parse failure to
///         allow graceful degradation rather than panicking.
///
/// # Arguments
/// * `version` – A semver string (e.g. "3.3.3", "v1.2.0", "1.2.0-alpha").
///
/// # Returns
/// `Some((major, minor, patch))` or `None` on parse failure.
pub fn parse_semver(version: &str) -> Option<(u64, u64, u64)> {
    // Strip any leading 'v' prefix
    let v = version.trim_start_matches('v');
    // Take only the numeric part before any pre-release suffix
    let base = v.split('-').next().unwrap_or(v);
    let parts: Vec<&str> = base.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    let major = parts[0].parse::<u64>().ok()?;
    let minor = parts[1].parse::<u64>().ok()?;
    let patch = parts[2].parse::<u64>().ok()?;
    Some((major, minor, patch))
/// @dev    Handles optional "v" prefix, pre-release suffixes, and missing patch.
///         Returns (0, 0, 0) on parse failure to allow graceful degradation.
///
/// # Arguments
/// * `version` – A semver string (e.g. "3.3.3", "v1.2.0", "1.2.0-alpha").
///
/// # Returns
/// `Some((major, minor, patch))` or `None` on parse failure.
pub fn parse_semver(version: &str) -> Option<(u64, u64, u64)> {
    // Strip any leading 'v' prefix
    let v = version.trim_start_matches('v');
    // Take only the numeric part before any pre-release suffix
    let base = v.split('-').next().unwrap_or(v);
    let parts: Vec<&str> = base.split('.').collect();
    if parts.len() < 3 {
        return None;
    }
    let major = parts[0].parse::<u64>().ok()?;
    let minor = parts[1].parse::<u64>().ok()?;
    let patch = parts[2].parse::<u64>().ok()?;
    Some((major, minor, patch))
}

/// @notice Check if `version >= min_version` using semantic versioning rules.
///
/// @dev    Compares tuples lexicographically: major first, then minor, then patch.
///         Returns false if either version string cannot be parsed.
/// @dev    Compares major, then minor, then patch in order.
///
/// # Arguments
/// * `version`     – The version to check.
/// * `min_version` – The minimum acceptable version.
///
/// # Returns
/// `true` if `version >= min_version`, `false` otherwise.
pub fn is_version_gte(version: &str, min_version: &str) -> bool {
    match (parse_semver(version), parse_semver(min_version)) {
        (Some(v), Some(m)) => v >= m,
        // If either version is unparseable, conservatively return false
        _ => false,
    }
pub fn is_version_gte(version: &String, min_version: &String) -> bool {
    let (v_major, v_minor, v_patch) = parse_semver(version);
    let (m_major, m_minor, m_patch) = parse_semver(min_version);

    if v_major != m_major {
        return v_major > m_major;
    }
}

// ── Integrity Validation ─────────────────────────────────────────────────────

/// @notice Validate that an integrity hash is present and uses sha512.
///
/// @dev    Rejects sha1 and sha256 as insufficient. Requires "sha512-" prefix.
///         An empty or malformed integrity string indicates a tampered or
///         incomplete lockfile entry.
///
/// # Arguments
/// * `integrity` – The integrity hash string (e.g. "sha512-...").
///
/// # Returns
/// `true` if valid sha512 hash, `false` otherwise.
pub fn validate_integrity(integrity: &str) -> bool {
    !integrity.is_empty() && integrity.starts_with("sha512-")
///
/// # Arguments
/// * `integrity` – The integrity hash string (e.g. "sha512-...").
///
/// # Returns
/// `true` if valid sha512 hash, `false` otherwise.
pub fn validate_integrity(integrity: &str) -> bool {
    !integrity.is_empty() && integrity.starts_with("sha512-")
}

// ── Package Auditing ─────────────────────────────────────────────────────────

/// @notice Audit a single package entry against known vulnerabilities.
///
/// @dev    Checks integrity hash validity and version constraints.
///         Known vulnerable packages must appear in `min_safe_versions`.
///         If a package is not in the map it is considered unconstrained
///         (only the integrity check applies).
///
/// # Arguments
/// * `entry`             – The package entry to audit.
/// * `min_safe_versions` – Map of package name -> minimum safe version string.
/// @dev    Checks version constraints and integrity hash validity.
///         Returns a typed `AuditResult` with pass/fail status and issues.
///
/// # Arguments
/// * `entry`             – The package entry to audit.
/// * `min_safe_versions` – Map of package name -> minimum safe version string.
///
/// # Returns
/// An `AuditResult` with `passed=true` if all checks pass, `false` otherwise.
pub fn audit_package(
    entry: &PackageEntry,
    min_safe_versions: &HashMap<String, String>,
) -> AuditResult {
    let mut issues: Vec<String> = Vec::new();

    // Integrity check — reject missing or non-sha512 hashes
    if !validate_integrity(&entry.integrity) {
        issues.push(format!(
            "Invalid or missing sha512 integrity hash for '{}'",
            entry.name
        ));
    }

    // Version constraint check — only applied if package is in the advisory map
    if let Some(min_ver) = min_safe_versions.get(&entry.name) {
        if !is_version_gte(&entry.version, min_ver) {
            issues.push(format!(
                "Package '{}' version '{}' is below minimum safe version '{}'",
                entry.name, entry.version, min_ver
            ));
        }
    }

    AuditResult {
        package_name: entry.name.clone(),
        passed: issues.is_empty(),
    min_safe_versions: &soroban_sdk::Map<String, String>,
) -> AuditResult {
    let mut issues: Vec<String> = Vec::new();

    // Integrity check — reject missing or non-sha512 hashes
    if !validate_integrity(&entry.integrity) {
        issues.push(format!(
            "Invalid or missing sha512 integrity hash for '{}'",
            entry.name
        ));
    }

    // Version constraint check — only applied if package is in the advisory map
    if let Some(min_ver) = min_safe_versions.get(&entry.name) {
        if !is_version_gte(&entry.version, min_ver) {
            issues.push(format!(
                "Package '{}' version '{}' is below minimum safe version '{}'",
                entry.name, entry.version, min_ver
            ));
        }
    }

    AuditResult {
        package_name: entry.name.clone(),
        passed: issues.is_empty(),
        issues,
    }
}

/// @notice Audit all packages in a lockfile snapshot.
///
/// @dev    Iterates over all entries and collects results. For inputs of
///         unknown size, prefer `audit_all_bounded` to cap processing.
///
/// # Arguments
/// * `packages`          – Slice of all package entries to audit.
/// * `min_safe_versions` – Map of package name -> minimum safe version string.
///
/// # Returns
/// A `Vec<AuditResult>`, one per package, in the same order as `packages`.
pub fn audit_all(
    packages: &[PackageEntry],
    min_safe_versions: &HashMap<String, String>,
) -> Vec<AuditResult> {
    packages
        .iter()
        .map(|p| audit_package(p, min_safe_versions))
        .collect()
}

/// @notice Bounded variant of `audit_all` — rejects inputs exceeding `MAX_PACKAGES`.
///
/// @notice Use this in place of `audit_all` wherever input size is not
///         statically known, to prevent unbounded processing and ensure
///         predictable execution time (gas efficiency / reliability).
///
/// # Arguments
/// * `packages`          – Slice of all package entries to audit.
/// * `min_safe_versions` – Map of package name -> minimum safe version string.
///
/// # Returns
/// `Ok(Vec<AuditResult>)` or `Err(String)` if the input exceeds `MAX_PACKAGES`.
pub fn audit_all_bounded(
    packages: &[PackageEntry],
    min_safe_versions: &HashMap<String, String>,
) -> Result<Vec<AuditResult>, String> {
    if packages.len() > MAX_PACKAGES {
        return Err(format!(
            "Input exceeds MAX_PACKAGES limit ({} > {}). Split into smaller batches.",
            packages.len(),
            MAX_PACKAGES
        ));
    }
    Ok(audit_all(packages, min_safe_versions))
/// @dev    Iterates over all entries and collects results.
///
/// # Arguments
/// * `packages`          – Slice of all package entries to audit.
/// * `min_safe_versions` – Map of package name -> minimum safe version string.
///
/// # Returns
/// A `Vec<AuditResult>`, one per package, in the same order as `packages`.
pub fn audit_all(
    packages: &[PackageEntry],
    min_safe_versions: &HashMap<String, String>,
) -> Vec<AuditResult> {
    packages
        .iter()
        .map(|p| audit_package(p, min_safe_versions))
        .collect()
}

/// @notice Bounded variant of `audit_all` — rejects inputs exceeding `MAX_PACKAGES`.
///
/// @notice Use this in place of `audit_all` wherever input size is not
///         statically known, to prevent unbounded processing and ensure
///         predictable execution time (gas efficiency / reliability).
///
/// # Arguments
/// * `packages`          – Slice of all package entries to audit.
/// * `min_safe_versions` – Map of package name -> minimum safe version string.
///
/// # Returns
/// `Ok(Vec<AuditResult>)` or `Err(String)` if the input exceeds `MAX_PACKAGES`.
pub fn audit_all_bounded(
    packages: &[PackageEntry],
    min_safe_versions: &HashMap<String, String>,
) -> Result<Vec<AuditResult>, String> {
    if packages.len() > MAX_PACKAGES {
        return Err(format!(
            "Input exceeds MAX_PACKAGES limit ({} > {}). Split into smaller batches.",
            packages.len(),
            MAX_PACKAGES
        ));
    }
    Ok(audit_all(packages, min_safe_versions))
}

/// @notice Filter audit results to only those that failed.
///
/// @dev    Returns references into the original slice to avoid cloning.
///
/// # Arguments
/// * `results` – Slice of audit results.
///
/// # Returns
/// A `Vec` of references to results where `passed == false`.
pub fn failing_results(results: &[AuditResult]) -> Vec<&AuditResult> {
    results.iter().filter(|r| !r.passed).collect()
/// @dev    Returns a new vector containing only failed results.
///
/// # Arguments
/// * `results` – Slice of audit results.
///
/// # Returns
/// A `Vec` of references to results where `passed == false`.
pub fn failing_results(results: &[AuditResult]) -> Vec<&AuditResult> {
    results.iter().filter(|r| !r.passed).collect()
}

/// @notice Validate the lockfile version.
///
/// @dev    Only versions 2 and 3 (npm >=7) are accepted.
///         Version 1 (npm <7) lacks integrity hashes for all entries and is
///         considered insecure. Versions 0 and 4+ are unsupported.
///
/// # Arguments
/// * `version` – The `lockfileVersion` integer from `package-lock.json`.
///         Version 1 lacks integrity hashes and is considered insecure.
///
/// # Arguments
/// * `version` – The `lockfileVersion` integer from `package-lock.json`.
///
/// # Returns
/// `true` if version is 2 or 3, `false` otherwise.
pub fn validate_lockfile_version(version: u32) -> bool {
    version == 2 || version == 3
    version >= MIN_LOCKFILE_VERSION && version <= MAX_LOCKFILE_VERSION
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// @notice Bounded variant of `audit_all` — rejects inputs exceeding `MAX_PACKAGES`.
///
/// @dev    Use this in place of `audit_all` wherever input size is not
///         statically known, to prevent unbounded processing and ensure
///         predictable execution time (DoS protection / gas efficiency).
///
/// @param packages          Vector of package entries to audit.
/// @param min_safe_versions Map of package names to minimum safe versions.
///
/// @return Ok(Vec<AuditResult>) or Err with a descriptive message.
pub fn audit_all_bounded(
    packages: &Vec<PackageEntry>,
    min_safe_versions: &soroban_sdk::Map<String, String>,
) -> Result<Vec<AuditResult>, &'static str> {
    if packages.len() > MAX_PACKAGES {
        return Err("Input exceeds MAX_PACKAGES limit. Split into smaller batches.");
    }
    Ok(audit_all(packages, min_safe_versions))
}

/// @notice Check if any audit results failed.
///
/// @dev    Convenience function for quick validation.
///
/// # Arguments
/// * `results` – Vector of audit results.
///
/// # Returns
/// `true` if any result failed, `false` if all passed.
pub fn has_failures(results: &Vec<AuditResult>) -> bool {
    for i in 0..results.len() {
        if let Some(result) = results.get(i) {
            if !result.passed {
                return true;
            }
        }
    }
    false
}

/// @notice Count the number of failed audits.
///
/// @dev    Useful for reporting and metrics.
///
/// # Arguments
/// * `results` – Vector of audit results.
///
/// # Returns
/// The count of failed audits.
pub fn count_failures(results: &Vec<AuditResult>) -> u32 {
    let mut count = 0u32;
    for i in 0..results.len() {
        if let Some(result) = results.get(i) {
            if !result.passed {
                count += 1;
            }
        }
    }
    count
    version == 2 || version == 3
}

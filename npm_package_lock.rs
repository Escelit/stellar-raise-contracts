/// # npm_package_lock
///
/// Contract module for auditing and validating NPM `package-lock.json`
/// dependency integrity, version constraints, and known vulnerability
/// advisories.
///
/// ## Security Assumptions
/// - Resolved package versions are compared against a known-safe minimum
///   version map to detect vulnerable ranges.
/// - Integrity hashes (sha512) are validated to be non-empty, ensuring
///   the lockfile was not tampered with.
/// - Only `lockfileVersion` 2 and 3 are considered valid (npm >=7).
/// - `audit_all_bounded` enforces a hard cap on input size to prevent
///   unbounded processing (gas efficiency / DoS protection).
///
/// ## NatSpec-style Annotations
/// @title   NpmPackageLockAuditor
/// @notice  Validates package-lock.json entries for security and integrity.
/// @dev     All checks are pure functions operating on parsed data structs.

use std::collections::HashMap;

/// @notice Hard cap on the number of packages processed by `audit_all_bounded`.
/// @dev    Prevents unbounded iteration; mirrors gas-limit patterns in
///         on-chain contracts. Adjust upward only with a documented rationale.
pub const MAX_PACKAGES: usize = 500;

/// Represents a single resolved package entry from `package-lock.json`.
///
/// @param name       Package name (e.g. "svgo")
/// @param version    Resolved semver string (e.g. "3.3.3")
/// @param integrity  sha512 hash string from the lockfile
/// @param dev        Whether the package is a devDependency
#[derive(Debug, Clone, PartialEq)]
pub struct PackageEntry {
    pub name: String,
    pub version: String,
    pub integrity: String,
    pub dev: bool,
}

/// Audit result for a single package.
///
/// @param package_name  Name of the audited package
/// @param passed        True if no issues were found
/// @param issues        List of human-readable issue descriptions
#[derive(Debug, Clone, PartialEq)]
pub struct AuditResult {
    pub package_name: String,
    pub passed: bool,
    pub issues: Vec<String>,
}

/// Parses a semver string into (major, minor, patch) tuple.
///
/// @notice Returns None if the string is not valid semver.
/// @param version  Semver string like "3.3.2"
/// @return         Option<(u64, u64, u64)>
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

/// Returns true if `version` is >= `min_version`.
///
/// @param version      The version to check
/// @param min_version  The minimum acceptable version
pub fn is_version_gte(version: &str, min_version: &str) -> bool {
    match (parse_semver(version), parse_semver(min_version)) {
        (Some(v), Some(m)) => v >= m,
        _ => false,
    }
}

/// Validates that a package's integrity field is non-empty and uses sha512.
///
/// @notice An empty or malformed integrity string indicates a tampered or
///         incomplete lockfile entry.
/// @param integrity  The integrity string from the lockfile entry
pub fn validate_integrity(integrity: &str) -> bool {
    !integrity.is_empty() && integrity.starts_with("sha512-")
}

/// Audits a single `PackageEntry` against a map of minimum safe versions.
///
/// @notice Known vulnerable packages must appear in `min_safe_versions`.
///         If a package is not in the map it is considered unconstrained.
/// @param entry             The package entry to audit
/// @param min_safe_versions Map of package name -> minimum safe version
/// @return                  AuditResult with pass/fail and issue list
pub fn audit_package(
    entry: &PackageEntry,
    min_safe_versions: &HashMap<String, String>,
) -> AuditResult {
    let mut issues: Vec<String> = Vec::new();

    // Integrity check
    if !validate_integrity(&entry.integrity) {
        issues.push(format!(
            "Invalid or missing sha512 integrity hash for '{}'",
            entry.name
        ));
    }

    // Version constraint check
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

/// Audits all packages in a lockfile snapshot.
///
/// @param packages          Slice of all package entries
/// @param min_safe_versions Map of package name -> minimum safe version
/// @return                  Vec of AuditResult, one per package
pub fn audit_all(
    packages: &[PackageEntry],
    min_safe_versions: &HashMap<String, String>,
) -> Vec<AuditResult> {
    packages
        .iter()
        .map(|p| audit_package(p, min_safe_versions))
        .collect()
}

/// Bounded variant of `audit_all` — rejects inputs exceeding `MAX_PACKAGES`.
///
/// @notice Use this in place of `audit_all` wherever input size is not
///         statically known, to prevent unbounded processing and ensure
///         predictable execution time (gas efficiency / reliability).
/// @param packages          Slice of all package entries
/// @param min_safe_versions Map of package name -> minimum safe version
/// @return                  Ok(Vec<AuditResult>) or Err with a message
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

/// Returns the subset of audit results that failed.
///
/// @param results  Full audit result list
/// @return         Only the failing results
pub fn failing_results(results: &[AuditResult]) -> Vec<&AuditResult> {
    results.iter().filter(|r| !r.passed).collect()
}

/// Validates that `lockfileVersion` is 2 or 3 (npm >=7 format).
///
/// @param version  The lockfileVersion integer from package-lock.json
/// @return         true if the version is supported
pub fn validate_lockfile_version(version: u32) -> bool {
    version == 2 || version == 3
}

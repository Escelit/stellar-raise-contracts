//! # multi_signature_execution
//!
//! @notice  Multi-signature execution utilities for the Stellar Raise
//!          crowdfunding contract.  Provides threshold-based approval
//!          tracking, signer-set validation, and execution guards that
//!          enforce M-of-N consensus before any privileged operation
//!          (upgrade, fee change, emergency pause) is allowed to proceed.
//!
//! @dev     All functions are pure or accept raw values so they can be
//!          composed freely in property-based tests without a running
//!          contract instance.  Storage integration is the caller's
//!          responsibility; this module only validates state snapshots.
//!
//! @custom:security-note  Single-key admin operations are the most common
//!          attack vector against on-chain governance.  This module enforces
//!          that no privileged action executes unless the required number of
//!          distinct, authorised signers have approved it.

#![allow(dead_code)]

use soroban_sdk::{contracttype, Address, Env, Vec};

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Minimum number of signers in any multi-sig configuration.
/// @dev     A threshold of 1 with 1 signer degenerates to single-key auth;
///          callers should enforce a higher minimum for production governance.
/// @custom:security-note  Enforced at configuration time to prevent trivially
///          bypassable multi-sig setups.
pub const MIN_SIGNERS: u32 = 1;

/// @notice  Maximum number of signers supported.
/// @dev     Bounded to prevent unbounded iteration in approval checks.
/// @custom:security-note  Exceeding this limit is rejected at configuration
///          time to prevent storage bloat and DoS via large signer sets.
pub const MAX_SIGNERS: u32 = 20;

/// @notice  Approval window in ledger seconds.
/// @dev     Approvals older than this are considered expired and must not
///          count toward the threshold.
/// @custom:security-note  Without an expiry window, a stale approval from a
///          compromised key could be replayed indefinitely.
pub const APPROVAL_EXPIRY_SECONDS: u64 = 86_400; // 24 hours

// ─────────────────────────────────────────────────────────────────────────────
// Types
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Outcome of a multi-sig validation check.
/// @dev     `Approved` means the threshold was met and execution may proceed.
///          `Pending` means more approvals are needed.
///          `Rejected` means a structural violation was detected (duplicate
///          signer, unauthorised signer, expired approval, etc.).
#[derive(Clone, PartialEq, Debug)]
pub enum MultiSigResult {
    /// Threshold met — execution is authorised.
    Approved,
    /// Threshold not yet met; `needed` more approvals required.
    Pending { needed: u32 },
    /// Structural violation; `reason` describes the problem.
    Rejected { reason: &'static str },
}

impl MultiSigResult {
    /// @notice  Returns `true` when execution is authorised.
    pub fn is_approved(&self) -> bool {
        matches!(self, MultiSigResult::Approved)
    }

    /// @notice  Returns `true` when the proposal is still collecting approvals.
    pub fn is_pending(&self) -> bool {
        matches!(self, MultiSigResult::Pending { .. })
    }

    /// @notice  Returns `true` when a structural violation was detected.
    pub fn is_rejected(&self) -> bool {
        matches!(self, MultiSigResult::Rejected { .. })
    }

    /// @notice  Returns the rejection reason, or `""` for non-rejected results.
    pub fn reason(&self) -> &'static str {
        match self {
            MultiSigResult::Rejected { reason } => reason,
            _ => "",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MultiSigConfig
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Immutable configuration for a multi-sig signer set.
/// @dev     Stored once at initialisation; changing it requires a separate
///          governance proposal that itself passes multi-sig validation.
///
/// @param   signers    Ordered list of authorised signer addresses.
/// @param   threshold  Minimum number of distinct approvals required.
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct MultiSigConfig {
    pub signers: Vec<Address>,
    pub threshold: u32,
}

/// @notice  A single signer's approval record.
/// @dev     `timestamp` is the ledger timestamp at approval time and is used
///          to enforce `APPROVAL_EXPIRY_SECONDS`.
///
/// @param   signer     The approving address.
/// @param   timestamp  Ledger timestamp when the approval was recorded.
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct Approval {
    pub signer: Address,
    pub timestamp: u64,
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. CONFIGURATION VALIDATION
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Validates a `MultiSigConfig` before it is stored.
/// @dev     Checks:
///          - `signers` count is within `[MIN_SIGNERS, MAX_SIGNERS]`.
///          - `threshold` is within `[1, signers.len()]`.
///          - No duplicate addresses in `signers`.
/// @custom:security-note  An invalid config (threshold > signers, duplicates)
///          would make the multi-sig permanently unexecutable or trivially
///          bypassable.  Validation must run before any config is persisted.
/// @param   config  The configuration to validate.
/// @return  `Ok(())` if valid, `Err(&'static str)` describing the violation.
pub fn validate_config(config: &MultiSigConfig) -> Result<(), &'static str> {
    let n = config.signers.len();

    if n < MIN_SIGNERS {
        return Err("signer count below MIN_SIGNERS");
    }
    if n > MAX_SIGNERS {
        return Err("signer count exceeds MAX_SIGNERS");
    }
    if config.threshold < 1 {
        return Err("threshold must be at least 1");
    }
    if config.threshold > n {
        return Err("threshold exceeds signer count");
    }
    if has_duplicate_signers(&config.signers) {
        return Err("duplicate signer address detected");
    }

    Ok(())
}

/// @notice  Returns `true` when `signers` contains at least one duplicate.
/// @dev     O(n²) — acceptable for `n <= MAX_SIGNERS` (20).
/// @custom:security-note  Duplicate signers would allow one key to satisfy
///          multiple approval slots, reducing the effective threshold.
fn has_duplicate_signers(signers: &Vec<Address>) -> bool {
    let len = signers.len();
    for i in 0..len {
        for j in (i + 1)..len {
            if signers.get(i) == signers.get(j) {
                return true;
            }
        }
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. SIGNER AUTHORISATION
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Returns `true` when `signer` is present in `config.signers`.
/// @dev     Linear scan — acceptable for `n <= MAX_SIGNERS`.
/// @custom:security-note  Only addresses in the registered signer set may
///          submit approvals.  Unauthorised approvals must be rejected before
///          they are counted toward the threshold.
/// @param   config  The active multi-sig configuration.
/// @param   signer  The address to check.
pub fn is_authorised_signer(config: &MultiSigConfig, signer: &Address) -> bool {
    for i in 0..config.signers.len() {
        if let Some(s) = config.signers.get(i) {
            if s == *signer {
                return true;
            }
        }
    }
    false
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. APPROVAL VALIDATION
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Validates a new approval before it is recorded.
/// @dev     Checks:
///          - `signer` is in `config.signers`.
///          - `signer` has not already approved (no double-vote).
///          - `now` is a plausible timestamp (> 0).
/// @custom:security-note  Double-voting would allow a single compromised key
///          to satisfy multiple approval slots.  This check must run before
///          any approval is appended to the pending list.
/// @param   config     Active multi-sig configuration.
/// @param   approvals  Current list of recorded approvals for this proposal.
/// @param   signer     The address attempting to approve.
/// @param   now        Current ledger timestamp.
/// @return  `Ok(())` if the approval is valid, `Err` otherwise.
pub fn validate_approval(
    config: &MultiSigConfig,
    approvals: &Vec<Approval>,
    signer: &Address,
    now: u64,
) -> Result<(), &'static str> {
    if !is_authorised_signer(config, signer) {
        return Err("signer is not in the authorised signer set");
    }

    for i in 0..approvals.len() {
        if let Some(a) = approvals.get(i) {
            if a.signer == *signer {
                return Err("signer has already approved this proposal");
            }
        }
    }

    if now == 0 {
        return Err("approval timestamp must be non-zero");
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. EXPIRY FILTERING
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Returns the count of approvals that are still within the expiry
///          window at ledger time `now`.
/// @dev     An approval is valid when `now - approval.timestamp <= APPROVAL_EXPIRY_SECONDS`.
///          Uses saturating subtraction to handle the edge case where
///          `approval.timestamp > now` (clock skew).
/// @custom:security-note  Expired approvals must not count toward the
///          threshold.  Without expiry, a stale approval from a compromised
///          key could be replayed indefinitely to satisfy the threshold.
/// @param   approvals  All recorded approvals for this proposal.
/// @param   now        Current ledger timestamp.
/// @return  Number of non-expired approvals.
pub fn count_valid_approvals(approvals: &Vec<Approval>, now: u64) -> u32 {
    let mut count: u32 = 0;
    for i in 0..approvals.len() {
        if let Some(a) = approvals.get(i) {
            let age = now.saturating_sub(a.timestamp);
            if age <= APPROVAL_EXPIRY_SECONDS {
                count = count.saturating_add(1);
            }
        }
    }
    count
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. EXECUTION GUARD
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Evaluates whether a proposal has reached execution threshold.
/// @dev     Counts only non-expired approvals from authorised signers.
///          Returns `Approved` when `valid_count >= config.threshold`,
///          `Pending` when more approvals are needed, or `Rejected` when a
///          structural violation is detected.
/// @custom:security-note  This is the single gate that must be checked before
///          any privileged operation executes.  Callers must not bypass it.
/// @param   config     Active multi-sig configuration.
/// @param   approvals  All recorded approvals for this proposal.
/// @param   now        Current ledger timestamp.
/// @return  `MultiSigResult` indicating whether execution may proceed.
pub fn check_execution_threshold(
    config: &MultiSigConfig,
    approvals: &Vec<Approval>,
    now: u64,
) -> MultiSigResult {
    // Structural check: config must be valid.
    if let Err(reason) = validate_config(config) {
        return MultiSigResult::Rejected { reason };
    }

    // Verify every recorded approval is from an authorised signer.
    for i in 0..approvals.len() {
        if let Some(a) = approvals.get(i) {
            if !is_authorised_signer(config, &a.signer) {
                return MultiSigResult::Rejected {
                    reason: "approval from unauthorised signer detected",
                };
            }
        }
    }

    let valid_count = count_valid_approvals(approvals, now);

    if valid_count >= config.threshold {
        MultiSigResult::Approved
    } else {
        MultiSigResult::Pending {
            needed: config.threshold.saturating_sub(valid_count),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. REVOCATION
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Validates that a signer may revoke their approval.
/// @dev     A revocation is valid when:
///          - `signer` is in the authorised signer set.
///          - `signer` has a recorded approval in `approvals`.
/// @custom:security-note  Allowing revocation ensures a signer can withdraw
///          consent if they detect a malicious proposal before threshold is
///          reached.  Without revocation, a compromised key that approved
///          early cannot be neutralised.
/// @param   config     Active multi-sig configuration.
/// @param   approvals  Current approval list.
/// @param   signer     The address attempting to revoke.
/// @return  `Ok(index)` — the index of the approval to remove — or `Err`.
pub fn validate_revocation(
    config: &MultiSigConfig,
    approvals: &Vec<Approval>,
    signer: &Address,
) -> Result<u32, &'static str> {
    if !is_authorised_signer(config, signer) {
        return Err("signer is not in the authorised signer set");
    }

    for i in 0..approvals.len() {
        if let Some(a) = approvals.get(i) {
            if a.signer == *signer {
                return Ok(i);
            }
        }
    }

    Err("no approval found for this signer")
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. EVENT EMISSION
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Emits a `multisig/approval_recorded` event when a new approval
///          is successfully validated and recorded.
/// @dev     Callers invoke this after appending the approval to storage.
/// @custom:security-note  Every approval must produce an on-chain event so
///          off-chain monitors can track proposal progress and detect
///          unexpected approvals from keys that should be inactive.
/// @param   env        Soroban environment.
/// @param   signer     The approving address.
/// @param   valid_count  Current count of valid approvals after this one.
/// @param   threshold  Required threshold for execution.
pub fn emit_approval_event(env: &Env, signer: &Address, valid_count: u32, threshold: u32) {
    env.events().publish(
        (
            soroban_sdk::Symbol::new(env, "multisig"),
            soroban_sdk::Symbol::new(env, "approval_recorded"),
        ),
        (signer.clone(), valid_count, threshold),
    );
}

/// @notice  Emits a `multisig/executed` event when a proposal reaches
///          threshold and execution is authorised.
/// @dev     Callers invoke this immediately before the privileged operation.
/// @custom:security-note  The execution event provides a tamper-evident
///          record that the threshold was met at a specific ledger timestamp.
/// @param   env        Soroban environment.
/// @param   threshold  The threshold that was satisfied.
/// @param   now        Ledger timestamp at execution time.
pub fn emit_execution_event(env: &Env, threshold: u32, now: u64) {
    env.events().publish(
        (
            soroban_sdk::Symbol::new(env, "multisig"),
            soroban_sdk::Symbol::new(env, "executed"),
        ),
        (threshold, now),
    );
}

/// @notice  Emits a `multisig/revoked` event when a signer revokes their
///          approval.
/// @dev     Callers invoke this after removing the approval from storage.
/// @custom:security-note  Revocation events allow monitors to detect when a
///          proposal loses quorum, which may indicate a key compromise.
/// @param   env     Soroban environment.
/// @param   signer  The revoking address.
pub fn emit_revocation_event(env: &Env, signer: &Address) {
    env.events().publish(
        (
            soroban_sdk::Symbol::new(env, "multisig"),
            soroban_sdk::Symbol::new(env, "revoked"),
        ),
        signer.clone(),
    );
}

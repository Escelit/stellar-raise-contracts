//! # multi_signature_execution.test.rs
//!
//! @notice  Comprehensive test suite for `multi_signature_execution.rs`.
//!          Covers configuration validation, signer authorisation, approval
//!          validation, expiry filtering, execution threshold, revocation,
//!          event helpers, and property-based fuzz tests.
//!
//! @dev     Tests are grouped into nine sections:
//!          1. `MultiSigResult` helper methods
//!          2. `validate_config`
//!          3. `is_authorised_signer`
//!          4. `validate_approval`
//!          5. `count_valid_approvals`
//!          6. `check_execution_threshold`
//!          7. `validate_revocation`
//!          8. Event helpers (smoke tests)
//!          9. Property-based / fuzz tests
//!
//! @custom:security-note  Every failure path asserts the exact variant and
//!          reason string so regressions in validation logic are caught
//!          immediately.  Targets ≥ 95 % line coverage.

#![cfg(test)]

use proptest::prelude::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Vec};

use crate::multi_signature_execution::{
    check_execution_threshold, count_valid_approvals, emit_approval_event, emit_execution_event,
    emit_revocation_event, is_authorised_signer, validate_approval, validate_config,
    validate_revocation, Approval, MultiSigConfig, MultiSigResult, APPROVAL_EXPIRY_SECONDS,
    MAX_SIGNERS, MIN_SIGNERS,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn env() -> Env {
    let e = Env::default();
    e.mock_all_auths();
    e
}

/// Build a `MultiSigConfig` with `n` fresh addresses and the given threshold.
fn make_config(env: &Env, n: u32, threshold: u32) -> MultiSigConfig {
    let mut signers = Vec::new(env);
    for _ in 0..n {
        signers.push_back(Address::generate(env));
    }
    MultiSigConfig { signers, threshold }
}

/// Build an `Approval` for `signer` at `timestamp`.
fn make_approval(signer: Address, timestamp: u64) -> Approval {
    Approval { signer, timestamp }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. MultiSigResult helper methods
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_result_approved_helpers() {
    let r = MultiSigResult::Approved;
    assert!(r.is_approved());
    assert!(!r.is_pending());
    assert!(!r.is_rejected());
    assert_eq!(r.reason(), "");
}

#[test]
fn test_result_pending_helpers() {
    let r = MultiSigResult::Pending { needed: 2 };
    assert!(!r.is_approved());
    assert!(r.is_pending());
    assert!(!r.is_rejected());
    assert_eq!(r.reason(), "");
}

#[test]
fn test_result_rejected_helpers() {
    let r = MultiSigResult::Rejected { reason: "bad config" };
    assert!(!r.is_approved());
    assert!(!r.is_pending());
    assert!(r.is_rejected());
    assert_eq!(r.reason(), "bad config");
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. validate_config
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Happy path: 2-of-3 config is valid.
#[test]
fn test_validate_config_2_of_3_ok() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    assert!(validate_config(&cfg).is_ok());
}

/// @notice  Happy path: 1-of-1 (minimum valid config).
#[test]
fn test_validate_config_1_of_1_ok() {
    let env = env();
    let cfg = make_config(&env, 1, 1);
    assert!(validate_config(&cfg).is_ok());
}

/// @notice  Happy path: threshold == signer count (unanimous).
#[test]
fn test_validate_config_unanimous_ok() {
    let env = env();
    let cfg = make_config(&env, 5, 5);
    assert!(validate_config(&cfg).is_ok());
}

/// @notice  Failure: zero signers rejected.
/// @custom:security-note  Empty signer set makes multi-sig permanently unexecutable.
#[test]
fn test_validate_config_zero_signers_fail() {
    let env = env();
    let cfg = MultiSigConfig {
        signers: Vec::new(&env),
        threshold: 1,
    };
    let err = validate_config(&cfg).unwrap_err();
    assert!(err.contains("MIN_SIGNERS"));
}

/// @notice  Failure: threshold zero rejected.
#[test]
fn test_validate_config_zero_threshold_fail() {
    let env = env();
    let cfg = make_config(&env, 3, 0);
    let err = validate_config(&cfg).unwrap_err();
    assert!(err.contains("threshold must be at least 1"));
}

/// @notice  Failure: threshold exceeds signer count.
/// @custom:security-note  Unreachable threshold makes multi-sig permanently locked.
#[test]
fn test_validate_config_threshold_exceeds_signers_fail() {
    let env = env();
    let cfg = make_config(&env, 2, 3);
    let err = validate_config(&cfg).unwrap_err();
    assert!(err.contains("threshold exceeds signer count"));
}

/// @notice  Failure: signer count exceeds MAX_SIGNERS.
/// @custom:security-note  Unbounded signer sets are a DoS vector.
#[test]
fn test_validate_config_too_many_signers_fail() {
    let env = env();
    let cfg = make_config(&env, MAX_SIGNERS + 1, 1);
    let err = validate_config(&cfg).unwrap_err();
    assert!(err.contains("MAX_SIGNERS"));
}

/// @notice  Failure: duplicate signer address.
/// @custom:security-note  Duplicates allow one key to satisfy multiple slots.
#[test]
fn test_validate_config_duplicate_signer_fail() {
    let env = env();
    let dup = Address::generate(&env);
    let mut signers = Vec::new(&env);
    signers.push_back(dup.clone());
    signers.push_back(Address::generate(&env));
    signers.push_back(dup.clone());
    let cfg = MultiSigConfig { signers, threshold: 2 };
    let err = validate_config(&cfg).unwrap_err();
    assert!(err.contains("duplicate signer"));
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. is_authorised_signer
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Happy path: signer in set returns true.
#[test]
fn test_is_authorised_signer_present() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let signer = cfg.signers.get(1).unwrap();
    assert!(is_authorised_signer(&cfg, &signer));
}

/// @notice  Failure: address not in set returns false.
#[test]
fn test_is_authorised_signer_absent() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let outsider = Address::generate(&env);
    assert!(!is_authorised_signer(&cfg, &outsider));
}

/// @notice  Edge case: empty signer set always returns false.
#[test]
fn test_is_authorised_signer_empty_set() {
    let env = env();
    let cfg = MultiSigConfig {
        signers: Vec::new(&env),
        threshold: 0,
    };
    let addr = Address::generate(&env);
    assert!(!is_authorised_signer(&cfg, &addr));
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. validate_approval
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Happy path: first approval from authorised signer.
#[test]
fn test_validate_approval_first_ok() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let signer = cfg.signers.get(0).unwrap();
    let approvals: Vec<Approval> = Vec::new(&env);
    assert!(validate_approval(&cfg, &approvals, &signer, 1_000).is_ok());
}

/// @notice  Failure: signer not in authorised set.
/// @custom:security-note  Unauthorised approvals must be rejected before counting.
#[test]
fn test_validate_approval_unauthorised_signer_fail() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let outsider = Address::generate(&env);
    let approvals: Vec<Approval> = Vec::new(&env);
    let err = validate_approval(&cfg, &approvals, &outsider, 1_000).unwrap_err();
    assert!(err.contains("not in the authorised signer set"));
}

/// @notice  Failure: signer has already approved (double-vote).
/// @custom:security-note  Double-voting reduces effective threshold.
#[test]
fn test_validate_approval_double_vote_fail() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let signer = cfg.signers.get(0).unwrap();
    let mut approvals: Vec<Approval> = Vec::new(&env);
    approvals.push_back(make_approval(signer.clone(), 1_000));
    let err = validate_approval(&cfg, &approvals, &signer, 2_000).unwrap_err();
    assert!(err.contains("already approved"));
}

/// @notice  Failure: zero timestamp rejected.
/// @custom:security-note  Zero timestamp indicates an uninitialised ledger value.
#[test]
fn test_validate_approval_zero_timestamp_fail() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let signer = cfg.signers.get(0).unwrap();
    let approvals: Vec<Approval> = Vec::new(&env);
    let err = validate_approval(&cfg, &approvals, &signer, 0).unwrap_err();
    assert!(err.contains("timestamp must be non-zero"));
}

/// @notice  Happy path: second signer approves after first — both valid.
#[test]
fn test_validate_approval_two_distinct_signers_ok() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let s0 = cfg.signers.get(0).unwrap();
    let s1 = cfg.signers.get(1).unwrap();
    let mut approvals: Vec<Approval> = Vec::new(&env);
    approvals.push_back(make_approval(s0.clone(), 1_000));
    assert!(validate_approval(&cfg, &approvals, &s1, 2_000).is_ok());
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. count_valid_approvals
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Happy path: all approvals within window are counted.
#[test]
fn test_count_valid_approvals_all_fresh() {
    let env = env();
    let now: u64 = 10_000;
    let mut approvals: Vec<Approval> = Vec::new(&env);
    approvals.push_back(make_approval(Address::generate(&env), now - 100));
    approvals.push_back(make_approval(Address::generate(&env), now - 200));
    approvals.push_back(make_approval(Address::generate(&env), now - 300));
    assert_eq!(count_valid_approvals(&approvals, now), 3);
}

/// @notice  Failure: expired approvals are not counted.
/// @custom:security-note  Stale approvals from compromised keys must not count.
#[test]
fn test_count_valid_approvals_expired_excluded() {
    let env = env();
    let now: u64 = APPROVAL_EXPIRY_SECONDS + 10_000;
    let mut approvals: Vec<Approval> = Vec::new(&env);
    // Fresh approval.
    approvals.push_back(make_approval(Address::generate(&env), now - 100));
    // Expired approval (older than APPROVAL_EXPIRY_SECONDS).
    approvals.push_back(make_approval(Address::generate(&env), now - APPROVAL_EXPIRY_SECONDS - 1));
    assert_eq!(count_valid_approvals(&approvals, now), 1);
}

/// @notice  Edge case: approval exactly at expiry boundary is still valid.
#[test]
fn test_count_valid_approvals_at_expiry_boundary() {
    let env = env();
    let now: u64 = APPROVAL_EXPIRY_SECONDS + 1_000;
    let mut approvals: Vec<Approval> = Vec::new(&env);
    approvals.push_back(make_approval(Address::generate(&env), now - APPROVAL_EXPIRY_SECONDS));
    assert_eq!(count_valid_approvals(&approvals, now), 1);
}

/// @notice  Edge case: empty approval list returns zero.
#[test]
fn test_count_valid_approvals_empty() {
    let env = env();
    let approvals: Vec<Approval> = Vec::new(&env);
    assert_eq!(count_valid_approvals(&approvals, 1_000), 0);
}

/// @notice  Edge case: approval timestamp in the future (clock skew) — age
///          saturates to 0, so the approval is counted as valid.
#[test]
fn test_count_valid_approvals_future_timestamp_counted() {
    let env = env();
    let now: u64 = 1_000;
    let mut approvals: Vec<Approval> = Vec::new(&env);
    // timestamp > now: saturating_sub gives 0, which is <= APPROVAL_EXPIRY_SECONDS.
    approvals.push_back(make_approval(Address::generate(&env), now + 500));
    assert_eq!(count_valid_approvals(&approvals, now), 1);
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. check_execution_threshold
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Happy path: threshold met with all fresh approvals → Approved.
#[test]
fn test_check_execution_threshold_approved() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let now: u64 = 10_000;
    let mut approvals: Vec<Approval> = Vec::new(&env);
    approvals.push_back(make_approval(cfg.signers.get(0).unwrap(), now - 100));
    approvals.push_back(make_approval(cfg.signers.get(1).unwrap(), now - 200));
    assert_eq!(check_execution_threshold(&cfg, &approvals, now), MultiSigResult::Approved);
}

/// @notice  Pending: only one of two required approvals present.
#[test]
fn test_check_execution_threshold_pending_one_of_two() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let now: u64 = 10_000;
    let mut approvals: Vec<Approval> = Vec::new(&env);
    approvals.push_back(make_approval(cfg.signers.get(0).unwrap(), now - 100));
    let result = check_execution_threshold(&cfg, &approvals, now);
    assert_eq!(result, MultiSigResult::Pending { needed: 1 });
}

/// @notice  Pending: zero approvals, threshold 3.
#[test]
fn test_check_execution_threshold_pending_zero_approvals() {
    let env = env();
    let cfg = make_config(&env, 3, 3);
    let approvals: Vec<Approval> = Vec::new(&env);
    let result = check_execution_threshold(&cfg, &approvals, 10_000);
    assert_eq!(result, MultiSigResult::Pending { needed: 3 });
}

/// @notice  Rejected: invalid config (threshold > signers).
/// @custom:security-note  Execution must be blocked when config is invalid.
#[test]
fn test_check_execution_threshold_rejected_invalid_config() {
    let env = env();
    let cfg = make_config(&env, 2, 5); // threshold > signers
    let approvals: Vec<Approval> = Vec::new(&env);
    let result = check_execution_threshold(&cfg, &approvals, 10_000);
    assert!(result.is_rejected());
}

/// @notice  Rejected: approval from unauthorised signer in the list.
/// @custom:security-note  Poisoned approval list must block execution.
#[test]
fn test_check_execution_threshold_rejected_unauthorised_approval() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let now: u64 = 10_000;
    let outsider = Address::generate(&env);
    let mut approvals: Vec<Approval> = Vec::new(&env);
    approvals.push_back(make_approval(cfg.signers.get(0).unwrap(), now - 100));
    approvals.push_back(make_approval(outsider, now - 200)); // not in signer set
    let result = check_execution_threshold(&cfg, &approvals, now);
    assert!(result.is_rejected());
    assert!(result.reason().contains("unauthorised signer"));
}

/// @notice  Pending: all approvals expired — threshold not met.
/// @custom:security-note  Expired approvals must not satisfy the threshold.
#[test]
fn test_check_execution_threshold_all_expired_pending() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let now: u64 = APPROVAL_EXPIRY_SECONDS + 50_000;
    let mut approvals: Vec<Approval> = Vec::new(&env);
    // Both approvals are expired.
    approvals.push_back(make_approval(cfg.signers.get(0).unwrap(), 1));
    approvals.push_back(make_approval(cfg.signers.get(1).unwrap(), 2));
    let result = check_execution_threshold(&cfg, &approvals, now);
    assert!(result.is_pending());
}

/// @notice  Approved: unanimous 3-of-3 with all fresh approvals.
#[test]
fn test_check_execution_threshold_unanimous_approved() {
    let env = env();
    let cfg = make_config(&env, 3, 3);
    let now: u64 = 10_000;
    let mut approvals: Vec<Approval> = Vec::new(&env);
    for i in 0..3 {
        approvals.push_back(make_approval(cfg.signers.get(i).unwrap(), now - 100));
    }
    assert_eq!(check_execution_threshold(&cfg, &approvals, now), MultiSigResult::Approved);
}

/// @notice  Approved: extra approvals beyond threshold still approved.
#[test]
fn test_check_execution_threshold_extra_approvals_still_approved() {
    let env = env();
    let cfg = make_config(&env, 5, 2);
    let now: u64 = 10_000;
    let mut approvals: Vec<Approval> = Vec::new(&env);
    for i in 0..5 {
        approvals.push_back(make_approval(cfg.signers.get(i).unwrap(), now - 50));
    }
    assert_eq!(check_execution_threshold(&cfg, &approvals, now), MultiSigResult::Approved);
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. validate_revocation
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Happy path: signer revokes their own approval.
#[test]
fn test_validate_revocation_ok() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let signer = cfg.signers.get(1).unwrap();
    let mut approvals: Vec<Approval> = Vec::new(&env);
    approvals.push_back(make_approval(cfg.signers.get(0).unwrap(), 1_000));
    approvals.push_back(make_approval(signer.clone(), 2_000));
    let idx = validate_revocation(&cfg, &approvals, &signer).unwrap();
    assert_eq!(idx, 1);
}

/// @notice  Failure: signer not in authorised set.
#[test]
fn test_validate_revocation_unauthorised_fail() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let outsider = Address::generate(&env);
    let approvals: Vec<Approval> = Vec::new(&env);
    let err = validate_revocation(&cfg, &approvals, &outsider).unwrap_err();
    assert!(err.contains("not in the authorised signer set"));
}

/// @notice  Failure: signer has not approved — nothing to revoke.
/// @custom:security-note  Prevents spurious revocation events.
#[test]
fn test_validate_revocation_no_approval_fail() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let signer = cfg.signers.get(0).unwrap();
    let approvals: Vec<Approval> = Vec::new(&env);
    let err = validate_revocation(&cfg, &approvals, &signer).unwrap_err();
    assert!(err.contains("no approval found"));
}

/// @notice  Edge case: revoke first approval in a list of three.
#[test]
fn test_validate_revocation_first_in_list() {
    let env = env();
    let cfg = make_config(&env, 3, 2);
    let s0 = cfg.signers.get(0).unwrap();
    let mut approvals: Vec<Approval> = Vec::new(&env);
    approvals.push_back(make_approval(s0.clone(), 1_000));
    approvals.push_back(make_approval(cfg.signers.get(1).unwrap(), 2_000));
    approvals.push_back(make_approval(cfg.signers.get(2).unwrap(), 3_000));
    let idx = validate_revocation(&cfg, &approvals, &s0).unwrap();
    assert_eq!(idx, 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. Event helpers (smoke tests)
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  emit_approval_event does not panic.
#[test]
fn test_emit_approval_event_no_panic() {
    let env = env();
    let signer = Address::generate(&env);
    emit_approval_event(&env, &signer, 1, 3);
}

/// @notice  emit_execution_event does not panic.
#[test]
fn test_emit_execution_event_no_panic() {
    let env = env();
    emit_execution_event(&env, 3, 10_000);
}

/// @notice  emit_revocation_event does not panic.
#[test]
fn test_emit_revocation_event_no_panic() {
    let env = env();
    let signer = Address::generate(&env);
    emit_revocation_event(&env, &signer);
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. Property-based / fuzz tests
// ─────────────────────────────────────────────────────────────────────────────

proptest! {
    /// @notice  Property: threshold > signer count always fails validate_config.
    #[test]
    fn prop_threshold_exceeds_signers_always_fails(
        n in MIN_SIGNERS..=MAX_SIGNERS,
        extra in 1u32..=10u32,
    ) {
        let env = env();
        let cfg = make_config(&env, n, n + extra);
        prop_assert!(validate_config(&cfg).is_err());
    }

    /// @notice  Property: valid M-of-N config (1 <= M <= N <= MAX) always passes.
    #[test]
    fn prop_valid_config_always_passes(
        n in MIN_SIGNERS..=MAX_SIGNERS,
        threshold in 1u32..=20u32,
    ) {
        prop_assume!(threshold <= n);
        let env = env();
        let cfg = make_config(&env, n, threshold);
        prop_assert!(validate_config(&cfg).is_ok());
    }

    /// @notice  Property: any approval older than APPROVAL_EXPIRY_SECONDS is
    ///          not counted.
    #[test]
    fn prop_expired_approval_never_counted(
        age_over in 1u64..=1_000_000u64,
    ) {
        let env = env();
        let now = APPROVAL_EXPIRY_SECONDS.saturating_add(age_over).saturating_add(1_000);
        let ts = now.saturating_sub(APPROVAL_EXPIRY_SECONDS).saturating_sub(age_over);
        let mut approvals: Vec<Approval> = Vec::new(&env);
        approvals.push_back(make_approval(Address::generate(&env), ts));
        prop_assert_eq!(count_valid_approvals(&approvals, now), 0);
    }

    /// @notice  Property: any approval within the window is always counted.
    #[test]
    fn prop_fresh_approval_always_counted(
        age in 0u64..=APPROVAL_EXPIRY_SECONDS,
    ) {
        let env = env();
        let now = APPROVAL_EXPIRY_SECONDS + 10_000;
        let ts = now.saturating_sub(age);
        let mut approvals: Vec<Approval> = Vec::new(&env);
        approvals.push_back(make_approval(Address::generate(&env), ts));
        prop_assert_eq!(count_valid_approvals(&approvals, now), 1);
    }

    /// @notice  Property: check_execution_threshold with zero approvals is
    ///          always Pending for any valid config.
    #[test]
    fn prop_zero_approvals_always_pending(
        n in MIN_SIGNERS..=5u32,
        threshold in 1u32..=5u32,
    ) {
        prop_assume!(threshold <= n);
        let env = env();
        let cfg = make_config(&env, n, threshold);
        let approvals: Vec<Approval> = Vec::new(&env);
        let result = check_execution_threshold(&cfg, &approvals, 10_000);
        prop_assert!(result.is_pending());
    }
}

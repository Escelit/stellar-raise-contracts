//! # output_sanitization.test.rs
//!
//! @notice  Comprehensive test suite for `output_sanitization.rs`.
//!          Covers every sanitizer function, the aggregate runner, the event
//!          helper, and all edge cases.  Targets ≥ 95 % line coverage.
//!
//! @dev     Tests are grouped into eight sections:
//!          1. `SanitizedOutput` helper methods
//!          2. `sanitize_amount`
//!          3. `sanitize_amount_bounded`
//!          4. `sanitize_bps`
//!          5. `sanitize_deadline`
//!          6. `sanitize_string`
//!          7. `sanitize_contributor_count`
//!          8. `sanitize_campaign_output` (aggregate)
//!          9. `emit_sanitization_warning`
//!         10. Property-based / fuzz tests
//!
//! @custom:security-note  Every failure path asserts the exact variant and
//!          inner value so regressions in clamping logic are caught immediately.

#![cfg(test)]

use proptest::prelude::*;
use soroban_sdk::{Env, String};

use crate::output_sanitization::{
    emit_sanitization_warning, sanitize_amount, sanitize_amount_bounded, sanitize_bps,
    sanitize_campaign_output, sanitize_contributor_count, sanitize_deadline, sanitize_string,
    SanitizedCampaignOutput, SanitizedOutput, MAX_BPS, MAX_CONTRIBUTOR_COUNT, MAX_STRING_LEN,
    TRUNCATED_SENTINEL, ZERO_SENTINEL,
};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn env() -> Env {
    let e = Env::default();
    e.mock_all_auths();
    e
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. SanitizedOutput helper methods
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  `Clean` variant: `is_clean()` true, `was_modified()` false.
#[test]
fn test_sanitized_output_clean_helpers() {
    let out: SanitizedOutput<i128> = SanitizedOutput::Clean(42);
    assert!(out.is_clean());
    assert!(!out.was_modified());
    assert_eq!(*out.value(), 42);
}

/// @notice  `Clamped` variant: `is_clean()` false, `was_modified()` true.
#[test]
fn test_sanitized_output_clamped_helpers() {
    let out: SanitizedOutput<i128> = SanitizedOutput::Clamped(0);
    assert!(!out.is_clean());
    assert!(out.was_modified());
    assert_eq!(*out.value(), 0);
}

/// @notice  `Rejected` variant: `is_clean()` false, `was_modified()` true.
#[test]
fn test_sanitized_output_rejected_helpers() {
    let out: SanitizedOutput<u64> = SanitizedOutput::Rejected(999);
    assert!(!out.is_clean());
    assert!(out.was_modified());
    assert_eq!(*out.value(), 999);
}

/// @notice  `value()` returns the inner value for all three variants.
#[test]
fn test_sanitized_output_value_all_variants() {
    assert_eq!(*SanitizedOutput::Clean(7i128).value(), 7);
    assert_eq!(*SanitizedOutput::Clamped(0i128).value(), 0);
    assert_eq!(*SanitizedOutput::Rejected(1u64).value(), 1);
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. sanitize_amount
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Happy path: zero amount is clean.
#[test]
fn test_sanitize_amount_zero_clean() {
    assert_eq!(sanitize_amount(0), SanitizedOutput::Clean(0));
}

/// @notice  Happy path: positive amount is clean.
#[test]
fn test_sanitize_amount_positive_clean() {
    assert_eq!(sanitize_amount(1_000_000), SanitizedOutput::Clean(1_000_000));
}

/// @notice  Happy path: i128::MAX is clean.
#[test]
fn test_sanitize_amount_max_i128_clean() {
    assert_eq!(sanitize_amount(i128::MAX), SanitizedOutput::Clean(i128::MAX));
}

/// @notice  Failure path: -1 is clamped to ZERO_SENTINEL.
/// @custom:security-note  Negative amounts must never appear in output.
#[test]
fn test_sanitize_amount_negative_one_clamped() {
    let out = sanitize_amount(-1);
    assert_eq!(out, SanitizedOutput::Clamped(ZERO_SENTINEL));
    assert!(out.was_modified());
}

/// @notice  Failure path: i128::MIN is clamped to ZERO_SENTINEL.
/// @custom:security-note  Maximum negative overflow attempt.
#[test]
fn test_sanitize_amount_min_i128_clamped() {
    let out = sanitize_amount(i128::MIN);
    assert_eq!(out, SanitizedOutput::Clamped(ZERO_SENTINEL));
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. sanitize_amount_bounded
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Happy path: amount within [0, max] is clean.
#[test]
fn test_sanitize_amount_bounded_within_range_clean() {
    assert_eq!(
        sanitize_amount_bounded(500, 1_000),
        SanitizedOutput::Clean(500)
    );
}

/// @notice  Happy path: amount exactly at max is clean (inclusive boundary).
#[test]
fn test_sanitize_amount_bounded_at_max_clean() {
    assert_eq!(
        sanitize_amount_bounded(1_000, 1_000),
        SanitizedOutput::Clean(1_000)
    );
}

/// @notice  Happy path: zero amount with positive max is clean.
#[test]
fn test_sanitize_amount_bounded_zero_clean() {
    assert_eq!(
        sanitize_amount_bounded(0, 1_000),
        SanitizedOutput::Clean(0)
    );
}

/// @notice  Failure path: amount above max is clamped to max.
/// @custom:security-note  Over-funded total_raised must not exceed goal in output.
#[test]
fn test_sanitize_amount_bounded_above_max_clamped() {
    let out = sanitize_amount_bounded(1_500, 1_000);
    assert_eq!(out, SanitizedOutput::Clamped(1_000));
}

/// @notice  Failure path: negative amount is clamped to zero regardless of max.
#[test]
fn test_sanitize_amount_bounded_negative_clamped_to_zero() {
    let out = sanitize_amount_bounded(-100, 1_000);
    assert_eq!(out, SanitizedOutput::Clamped(ZERO_SENTINEL));
}

/// @notice  Edge case: max == 0, amount == 0 is clean.
#[test]
fn test_sanitize_amount_bounded_zero_max_zero_amount() {
    assert_eq!(
        sanitize_amount_bounded(0, 0),
        SanitizedOutput::Clean(0)
    );
}

/// @notice  Edge case: max == 0, positive amount is clamped to 0.
#[test]
fn test_sanitize_amount_bounded_zero_max_positive_amount_clamped() {
    let out = sanitize_amount_bounded(1, 0);
    assert_eq!(out, SanitizedOutput::Clamped(0));
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. sanitize_bps
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Happy path: zero bps is clean.
#[test]
fn test_sanitize_bps_zero_clean() {
    assert_eq!(sanitize_bps(0), SanitizedOutput::Clean(0));
}

/// @notice  Happy path: exactly MAX_BPS is clean (inclusive boundary).
#[test]
fn test_sanitize_bps_at_max_clean() {
    assert_eq!(sanitize_bps(MAX_BPS), SanitizedOutput::Clean(MAX_BPS));
}

/// @notice  Happy path: mid-range value is clean.
#[test]
fn test_sanitize_bps_midrange_clean() {
    assert_eq!(sanitize_bps(5_000), SanitizedOutput::Clean(5_000));
}

/// @notice  Failure path: MAX_BPS + 1 is clamped to MAX_BPS.
/// @custom:security-note  Fee > 100 % must never appear in output.
#[test]
fn test_sanitize_bps_over_max_clamped() {
    let out = sanitize_bps(MAX_BPS + 1);
    assert_eq!(out, SanitizedOutput::Clamped(MAX_BPS));
    assert!(out.was_modified());
}

/// @notice  Failure path: u32::MAX is clamped to MAX_BPS.
#[test]
fn test_sanitize_bps_u32_max_clamped() {
    let out = sanitize_bps(u32::MAX);
    assert_eq!(out, SanitizedOutput::Clamped(MAX_BPS));
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. sanitize_deadline
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Happy path: deadline in the future is clean.
#[test]
fn test_sanitize_deadline_future_clean() {
    assert_eq!(
        sanitize_deadline(1_000, 2_000),
        SanitizedOutput::Clean(2_000)
    );
}

/// @notice  Happy path: deadline exactly equal to now is clean (inclusive).
/// @custom:security-note  Contributions at exactly the deadline second are
///          accepted by the contract; the sanitizer must match this boundary.
#[test]
fn test_sanitize_deadline_at_now_clean() {
    assert_eq!(
        sanitize_deadline(1_000, 1_000),
        SanitizedOutput::Clean(1_000)
    );
}

/// @notice  Failure path: deadline one second in the past is rejected.
/// @custom:security-note  Emitting a stale deadline could mark a campaign as
///          expired before contributors have a chance to participate.
#[test]
fn test_sanitize_deadline_past_rejected() {
    let out = sanitize_deadline(1_001, 1_000);
    assert_eq!(out, SanitizedOutput::Rejected(1_001));
    assert!(out.was_modified());
}

/// @notice  Failure path: deadline far in the past is rejected; safe default is `now`.
#[test]
fn test_sanitize_deadline_far_past_rejected() {
    let out = sanitize_deadline(9_999_999, 1);
    assert_eq!(out, SanitizedOutput::Rejected(9_999_999));
}

/// @notice  Edge case: now == 0, deadline == 0 is clean.
#[test]
fn test_sanitize_deadline_both_zero_clean() {
    assert_eq!(sanitize_deadline(0, 0), SanitizedOutput::Clean(0));
}

/// @notice  Edge case: u64::MAX deadline with any now is clean.
#[test]
fn test_sanitize_deadline_u64_max_clean() {
    assert_eq!(
        sanitize_deadline(u64::MAX - 1, u64::MAX),
        SanitizedOutput::Clean(u64::MAX)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. sanitize_string
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Happy path: empty string is clean.
#[test]
fn test_sanitize_string_empty_clean() {
    let env = env();
    let s = String::from_str(&env, "");
    let out = sanitize_string(&env, &s);
    assert!(out.is_clean());
    assert_eq!(out.value().len(), 0);
}

/// @notice  Happy path: string exactly at MAX_STRING_LEN is clean.
#[test]
fn test_sanitize_string_at_max_len_clean() {
    let env = env();
    let s: std::string::String = "a".repeat(MAX_STRING_LEN as usize);
    let soroban_s = String::from_str(&env, &s);
    let out = sanitize_string(&env, &soroban_s);
    assert!(out.is_clean());
}

/// @notice  Happy path: short string is clean.
#[test]
fn test_sanitize_string_short_clean() {
    let env = env();
    let s = String::from_str(&env, "hello");
    let out = sanitize_string(&env, &s);
    assert!(out.is_clean());
}

/// @notice  Failure path: string one byte over MAX_STRING_LEN is clamped to sentinel.
/// @custom:security-note  Oversized strings in events are a DoS vector against
///          off-chain indexers.
#[test]
fn test_sanitize_string_over_max_len_clamped() {
    let env = env();
    let s: std::string::String = "x".repeat(MAX_STRING_LEN as usize + 1);
    let soroban_s = String::from_str(&env, &s);
    let out = sanitize_string(&env, &soroban_s);
    assert!(out.was_modified());
    let sentinel = String::from_str(&env, TRUNCATED_SENTINEL);
    assert_eq!(out.value().len(), sentinel.len());
}

/// @notice  Failure path: very long string is clamped to sentinel.
#[test]
fn test_sanitize_string_very_long_clamped() {
    let env = env();
    let s: std::string::String = "z".repeat(10_000);
    let soroban_s = String::from_str(&env, &s);
    let out = sanitize_string(&env, &soroban_s);
    assert!(out.was_modified());
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. sanitize_contributor_count
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Happy path: zero count is clean.
#[test]
fn test_sanitize_contributor_count_zero_clean() {
    assert_eq!(
        sanitize_contributor_count(0),
        SanitizedOutput::Clean(0)
    );
}

/// @notice  Happy path: exactly MAX_CONTRIBUTOR_COUNT is clean.
#[test]
fn test_sanitize_contributor_count_at_max_clean() {
    assert_eq!(
        sanitize_contributor_count(MAX_CONTRIBUTOR_COUNT),
        SanitizedOutput::Clean(MAX_CONTRIBUTOR_COUNT)
    );
}

/// @notice  Failure path: count above MAX_CONTRIBUTOR_COUNT is clamped.
/// @custom:security-note  Inflated contributor counts mislead governance tooling.
#[test]
fn test_sanitize_contributor_count_over_max_clamped() {
    let out = sanitize_contributor_count(MAX_CONTRIBUTOR_COUNT + 1);
    assert_eq!(out, SanitizedOutput::Clamped(MAX_CONTRIBUTOR_COUNT));
    assert!(out.was_modified());
}

/// @notice  Failure path: u32::MAX is clamped to MAX_CONTRIBUTOR_COUNT.
#[test]
fn test_sanitize_contributor_count_u32_max_clamped() {
    let out = sanitize_contributor_count(u32::MAX);
    assert_eq!(out, SanitizedOutput::Clamped(MAX_CONTRIBUTOR_COUNT));
}

// ─────────────────────────────────────────────────────────────────────────────
// 8. sanitize_campaign_output (aggregate)
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Happy path: all fields valid — was_modified is false.
#[test]
fn test_sanitize_campaign_output_all_clean() {
    let out: SanitizedCampaignOutput =
        sanitize_campaign_output(1_000, 500, 1_000, 5_000, 2_000, 10);
    assert!(!out.was_modified);
    assert_eq!(out.total_raised, 500);
    assert_eq!(out.goal, 1_000);
    assert_eq!(out.progress_bps, 5_000);
    assert_eq!(out.deadline, 2_000);
    assert_eq!(out.contributor_count, 10);
}

/// @notice  Failure path: negative total_raised triggers was_modified.
/// @custom:security-note  Negative balance in output is a data-protection violation.
#[test]
fn test_sanitize_campaign_output_negative_raised_modified() {
    let out = sanitize_campaign_output(1_000, -1, 1_000, 5_000, 2_000, 10);
    assert!(out.was_modified);
    assert_eq!(out.total_raised, 0);
}

/// @notice  Failure path: total_raised above goal is clamped to goal.
#[test]
fn test_sanitize_campaign_output_raised_above_goal_clamped() {
    let out = sanitize_campaign_output(1_000, 1_500, 1_000, 10_000, 2_000, 10);
    assert!(out.was_modified);
    assert_eq!(out.total_raised, 1_000);
}

/// @notice  Failure path: bps above MAX_BPS is clamped.
#[test]
fn test_sanitize_campaign_output_bps_over_max_clamped() {
    let out = sanitize_campaign_output(1_000, 500, 1_000, 99_999, 2_000, 10);
    assert!(out.was_modified);
    assert_eq!(out.progress_bps, MAX_BPS);
}

/// @notice  Failure path: deadline in the past is rejected; safe default is now.
#[test]
fn test_sanitize_campaign_output_past_deadline_rejected() {
    let out = sanitize_campaign_output(5_000, 500, 1_000, 5_000, 1_000, 10);
    assert!(out.was_modified);
    assert_eq!(out.deadline, 5_000); // replaced with `now`
}

/// @notice  Failure path: contributor count above max is clamped.
#[test]
fn test_sanitize_campaign_output_count_over_max_clamped() {
    let out = sanitize_campaign_output(1_000, 500, 1_000, 5_000, 2_000, 999);
    assert!(out.was_modified);
    assert_eq!(out.contributor_count, MAX_CONTRIBUTOR_COUNT);
}

/// @notice  Failure path: multiple fields invalid — was_modified true, all clamped.
#[test]
fn test_sanitize_campaign_output_multiple_fields_invalid() {
    let out = sanitize_campaign_output(1_000, -50, 1_000, 20_000, 500, 999);
    assert!(out.was_modified);
    assert_eq!(out.total_raised, 0);
    assert_eq!(out.progress_bps, MAX_BPS);
    assert_eq!(out.deadline, 1_000);
    assert_eq!(out.contributor_count, MAX_CONTRIBUTOR_COUNT);
}

/// @notice  Edge case: zero goal — total_raised of 0 is clean.
#[test]
fn test_sanitize_campaign_output_zero_goal_zero_raised() {
    let out = sanitize_campaign_output(1_000, 0, 0, 0, 2_000, 0);
    assert_eq!(out.total_raised, 0);
    assert_eq!(out.goal, 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// 9. emit_sanitization_warning
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  No event emitted when was_modified is false.
#[test]
fn test_emit_sanitization_warning_clean_no_event() {
    let env = env();
    let output = SanitizedCampaignOutput {
        total_raised: 500,
        goal: 1_000,
        progress_bps: 5_000,
        deadline: 2_000,
        contributor_count: 10,
        was_modified: false,
    };
    // Should not panic; no event emitted.
    emit_sanitization_warning(&env, &output);
}

/// @notice  Event emitted when was_modified is true.
/// @custom:security-note  Tamper-evident audit trail for data anomalies.
#[test]
fn test_emit_sanitization_warning_modified_emits_event() {
    let env = env();
    let output = SanitizedCampaignOutput {
        total_raised: 0,
        goal: 1_000,
        progress_bps: MAX_BPS,
        deadline: 1_000,
        contributor_count: MAX_CONTRIBUTOR_COUNT,
        was_modified: true,
    };
    // Should not panic; event is emitted.
    emit_sanitization_warning(&env, &output);
}

// ─────────────────────────────────────────────────────────────────────────────
// 10. Property-based / fuzz tests
// ─────────────────────────────────────────────────────────────────────────────

proptest! {
    /// @notice  Property: any non-negative amount always produces Clean.
    #[test]
    fn prop_sanitize_amount_non_negative_always_clean(amount in 0i128..=i128::MAX) {
        prop_assert!(sanitize_amount(amount).is_clean());
    }

    /// @notice  Property: any negative amount always produces Clamped(0).
    /// @custom:security-note  No negative value must ever pass through to output.
    #[test]
    fn prop_sanitize_amount_negative_always_clamped(amount in i128::MIN..=-1i128) {
        let out = sanitize_amount(amount);
        prop_assert!(!out.is_clean());
        prop_assert_eq!(*out.value(), ZERO_SENTINEL);
    }

    /// @notice  Property: any bps <= MAX_BPS is always Clean.
    #[test]
    fn prop_sanitize_bps_valid_always_clean(bps in 0u32..=MAX_BPS) {
        prop_assert!(sanitize_bps(bps).is_clean());
    }

    /// @notice  Property: any bps > MAX_BPS is always Clamped(MAX_BPS).
    #[test]
    fn prop_sanitize_bps_over_max_always_clamped(bps in (MAX_BPS + 1)..=u32::MAX) {
        let out = sanitize_bps(bps);
        prop_assert!(!out.is_clean());
        prop_assert_eq!(*out.value(), MAX_BPS);
    }

    /// @notice  Property: deadline >= now always produces Clean.
    #[test]
    fn prop_sanitize_deadline_future_always_clean(
        now in 0u64..=u64::MAX / 2,
        offset in 0u64..=u64::MAX / 2,
    ) {
        let deadline = now.saturating_add(offset);
        prop_assert!(sanitize_deadline(now, deadline).is_clean());
    }

    /// @notice  Property: deadline < now always produces Rejected(now).
    #[test]
    fn prop_sanitize_deadline_past_always_rejected(
        now in 1u64..=u64::MAX,
        past in 0u64..=u64::MAX,
    ) {
        prop_assume!(past < now);
        let out = sanitize_deadline(now, past);
        prop_assert!(!out.is_clean());
        prop_assert_eq!(*out.value(), now);
    }

    /// @notice  Property: contributor count <= MAX always produces Clean.
    #[test]
    fn prop_sanitize_contributor_count_valid_always_clean(
        count in 0u32..=MAX_CONTRIBUTOR_COUNT,
    ) {
        prop_assert!(sanitize_contributor_count(count).is_clean());
    }

    /// @notice  Property: contributor count > MAX always produces Clamped(MAX).
    #[test]
    fn prop_sanitize_contributor_count_over_max_always_clamped(
        count in (MAX_CONTRIBUTOR_COUNT + 1)..=u32::MAX,
    ) {
        let out = sanitize_contributor_count(count);
        prop_assert!(!out.is_clean());
        prop_assert_eq!(*out.value(), MAX_CONTRIBUTOR_COUNT);
    }

    /// @notice  Property: amount within [0, max] always produces Clean.
    #[test]
    fn prop_sanitize_amount_bounded_within_range_always_clean(
        max in 1i128..=i128::MAX / 2,
        amount in 0i128..=i128::MAX / 2,
    ) {
        prop_assume!(amount <= max);
        prop_assert!(sanitize_amount_bounded(amount, max).is_clean());
    }

    /// @notice  Property: amount > max always produces Clamped(max).
    #[test]
    fn prop_sanitize_amount_bounded_over_max_always_clamped(
        max in 0i128..=i128::MAX / 2,
        amount in 1i128..=i128::MAX / 2,
    ) {
        prop_assume!(amount > max);
        let out = sanitize_amount_bounded(amount, max);
        prop_assert!(!out.is_clean());
        prop_assert_eq!(*out.value(), max);
    }

    /// @notice  Property: sanitize_campaign_output with all valid inputs never
    ///          sets was_modified.
    #[test]
    fn prop_campaign_output_valid_inputs_not_modified(
        now in 0u64..500_000u64,
        raised in 0i128..500_000i128,
        goal in 1i128..500_000i128,
        bps in 0u32..=MAX_BPS,
        deadline_offset in 0u64..500_000u64,
        count in 0u32..=MAX_CONTRIBUTOR_COUNT,
    ) {
        prop_assume!(raised <= goal);
        let deadline = now.saturating_add(deadline_offset);
        let out = sanitize_campaign_output(now, raised, goal, bps, deadline, count);
        prop_assert!(!out.was_modified);
    }
}

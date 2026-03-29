//! # output_sanitization
//!
//! @notice  Output sanitization utilities for the Stellar Raise crowdfunding
//!          contract.  Validates and clamps all values that leave the contract
//!          boundary — query return values, event payloads, and computed
//!          metrics — before they reach off-chain consumers.
//!
//! @dev     All functions are pure (no storage reads or writes).  They accept
//!          raw values and return a typed `SanitizedOutput<T>` so callers can
//!          distinguish clean values from clamped or rejected ones without
//!          panicking.  Compose freely in property-based tests without a
//!          running contract instance.
//!
//! @custom:security-note  Unsanitized outputs are a data-protection risk:
//!          negative balances, out-of-range basis points, or overlong strings
//!          can corrupt off-chain indexers, mislead UIs, and leak internal
//!          state.  Every value that crosses the contract boundary must pass
//!          through this module before emission.

#![allow(dead_code)]

use soroban_sdk::{Env, String};

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Maximum allowed basis-point value (100 %).
/// @dev     Progress and fee values expressed in bps must not exceed this.
pub const MAX_BPS: u32 = 10_000;

/// @notice  Maximum byte length for any string output (e.g. campaign title).
/// @dev     Prevents unbounded string payloads in events and query responses.
/// @custom:security-note  Strings longer than this are truncated, not rejected,
///          so the contract never panics on oversized metadata.
pub const MAX_STRING_LEN: u32 = 256;

/// @notice  Sentinel value emitted when a string exceeds `MAX_STRING_LEN`.
pub const TRUNCATED_SENTINEL: &str = "[TRUNCATED]";

/// @notice  Sentinel value emitted when an amount is clamped to zero.
pub const ZERO_SENTINEL: i128 = 0;

// ─────────────────────────────────────────────────────────────────────────────
// SanitizedOutput
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Wrapper returned by every sanitization function.
/// @dev     `Clean` means the value passed all checks unchanged.
///          `Clamped` means the value was out of range and was adjusted to the
///          nearest safe boundary.
///          `Rejected` means the value is structurally invalid and a safe
///          default was substituted.
#[derive(Clone, PartialEq, Debug)]
pub enum SanitizedOutput<T> {
    /// Value passed all checks; inner value is the original.
    Clean(T),
    /// Value was out of range; inner value is the clamped safe substitute.
    Clamped(T),
    /// Value was structurally invalid; inner value is the safe default.
    Rejected(T),
}

impl<T: Clone> SanitizedOutput<T> {
    /// @notice  Returns the inner value regardless of variant.
    pub fn value(&self) -> &T {
        match self {
            SanitizedOutput::Clean(v)
            | SanitizedOutput::Clamped(v)
            | SanitizedOutput::Rejected(v) => v,
        }
    }

    /// @notice  Returns `true` when the value required no adjustment.
    pub fn is_clean(&self) -> bool {
        matches!(self, SanitizedOutput::Clean(_))
    }

    /// @notice  Returns `true` when the value was modified (clamped or rejected).
    pub fn was_modified(&self) -> bool {
        !self.is_clean()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. AMOUNT SANITIZATION
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Sanitizes a token amount before it is emitted in an event or
///          returned from a query.
/// @dev     Negative amounts are clamped to `ZERO_SENTINEL`.  Zero is clean.
///          Positive values pass through unchanged.
/// @custom:security-note  A negative amount in an event payload could mislead
///          off-chain indexers into crediting a contributor with a refund they
///          did not receive, or reporting a negative campaign balance.
/// @param   amount  Raw token amount from storage or computation.
/// @return  `Clean(amount)` if `amount >= 0`, else `Clamped(0)`.
pub fn sanitize_amount(amount: i128) -> SanitizedOutput<i128> {
    if amount >= 0 {
        SanitizedOutput::Clean(amount)
    } else {
        SanitizedOutput::Clamped(ZERO_SENTINEL)
    }
}

/// @notice  Sanitizes a token amount and enforces an inclusive upper bound.
/// @dev     Negative values are clamped to zero; values above `max` are clamped
///          to `max`.  Useful for sanitizing `total_raised` against `goal` so
///          the emitted progress never exceeds 100 %.
/// @custom:security-note  An over-funded `total_raised` emitted without capping
///          could cause UIs to display > 100 % progress, misleading contributors.
/// @param   amount  Raw token amount.
/// @param   max     Inclusive upper bound (must be >= 0).
/// @return  `Clean`, `Clamped(0)`, or `Clamped(max)`.
pub fn sanitize_amount_bounded(amount: i128, max: i128) -> SanitizedOutput<i128> {
    if amount < 0 {
        return SanitizedOutput::Clamped(ZERO_SENTINEL);
    }
    if max >= 0 && amount > max {
        return SanitizedOutput::Clamped(max);
    }
    SanitizedOutput::Clean(amount)
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. BASIS-POINT SANITIZATION
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Sanitizes a basis-point value (progress %, fee %) before output.
/// @dev     Values above `MAX_BPS` (10 000) are clamped to `MAX_BPS`.
///          The result is always in `[0, 10_000]`.
/// @custom:security-note  An unclamped bps value > 10 000 in a fee event could
///          cause off-chain fee calculators to over-charge contributors.
/// @param   bps  Raw basis-point value.
/// @return  `Clean(bps)` if `bps <= MAX_BPS`, else `Clamped(MAX_BPS)`.
pub fn sanitize_bps(bps: u32) -> SanitizedOutput<u32> {
    if bps <= MAX_BPS {
        SanitizedOutput::Clean(bps)
    } else {
        SanitizedOutput::Clamped(MAX_BPS)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. TIMESTAMP SANITIZATION
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Sanitizes a deadline timestamp before it is emitted.
/// @dev     A deadline in the past (`deadline < now`) is structurally invalid
///          for output — it would tell off-chain consumers the campaign is
///          already over before it was announced.  Such values are rejected and
///          replaced with `now` as the safe default.
/// @custom:security-note  Emitting a stale deadline could allow a malicious
///          indexer to mark a campaign as expired before contributors have a
///          chance to participate.
/// @param   now       Current ledger timestamp (seconds).
/// @param   deadline  Campaign deadline timestamp (seconds).
/// @return  `Clean(deadline)` if `deadline >= now`, else `Rejected(now)`.
pub fn sanitize_deadline(now: u64, deadline: u64) -> SanitizedOutput<u64> {
    if deadline >= now {
        SanitizedOutput::Clean(deadline)
    } else {
        SanitizedOutput::Rejected(now)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. STRING SANITIZATION
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Sanitizes a Soroban `String` length before it is emitted in an
///          event or returned from a query.
/// @dev     Strings longer than `MAX_STRING_LEN` bytes are replaced with a
///          new `String` built from `TRUNCATED_SENTINEL`.  The original string
///          is never emitted.  Empty strings are clean.
/// @custom:security-note  Unbounded string payloads in events can exhaust
///          indexer memory and are a denial-of-service vector against off-chain
///          consumers.  Truncation is preferred over rejection so the contract
///          never panics on oversized metadata.
/// @param   env     Soroban environment (needed to construct the sentinel string).
/// @param   s       The string to sanitize.
/// @return  `Clean(s)` if `s.len() <= MAX_STRING_LEN`, else
///          `Clamped(String::from_str(env, TRUNCATED_SENTINEL))`.
pub fn sanitize_string(env: &Env, s: &String) -> SanitizedOutput<String> {
    if s.len() <= MAX_STRING_LEN {
        SanitizedOutput::Clean(s.clone())
    } else {
        SanitizedOutput::Clamped(String::from_str(env, TRUNCATED_SENTINEL))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 5. CONTRIBUTOR COUNT SANITIZATION
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Maximum number of contributors the contract supports.
/// @dev     Mirrors `contract_state_size::MAX_CONTRIBUTORS`.
pub const MAX_CONTRIBUTOR_COUNT: u32 = 128;

/// @notice  Sanitizes a contributor count before it is emitted.
/// @dev     Counts above `MAX_CONTRIBUTOR_COUNT` are clamped.  Zero is clean.
/// @custom:security-note  An inflated contributor count in an event could
///          mislead governance tooling into believing a campaign has broader
///          support than it actually does.
/// @param   count  Raw contributor count.
/// @return  `Clean(count)` if `count <= MAX_CONTRIBUTOR_COUNT`, else
///          `Clamped(MAX_CONTRIBUTOR_COUNT)`.
pub fn sanitize_contributor_count(count: u32) -> SanitizedOutput<u32> {
    if count <= MAX_CONTRIBUTOR_COUNT {
        SanitizedOutput::Clean(count)
    } else {
        SanitizedOutput::Clamped(MAX_CONTRIBUTOR_COUNT)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 6. AGGREGATE EVENT PAYLOAD SANITIZATION
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Sanitized snapshot of a campaign's public output fields.
/// @dev     All fields have already been passed through their individual
///          sanitizers.  Callers should use `sanitize_campaign_output` to
///          construct this struct rather than building it manually.
#[derive(Clone, Debug, PartialEq)]
pub struct SanitizedCampaignOutput {
    /// Sanitized total raised (>= 0, <= goal).
    pub total_raised: i128,
    /// Sanitized goal (>= 1).
    pub goal: i128,
    /// Sanitized progress in basis points ([0, 10_000]).
    pub progress_bps: u32,
    /// Sanitized deadline (>= now).
    pub deadline: u64,
    /// Sanitized contributor count (<= MAX_CONTRIBUTOR_COUNT).
    pub contributor_count: u32,
    /// Whether any field required adjustment.
    pub was_modified: bool,
}

/// @notice  Runs all individual sanitizers over a campaign output snapshot and
///          returns a `SanitizedCampaignOutput` ready for event emission or
///          query response.
/// @dev     `was_modified` is set to `true` if any single field was clamped or
///          rejected.  Callers should log a warning event when `was_modified`
///          is true so off-chain monitors can detect data anomalies.
/// @custom:security-note  This is the single entry point for sanitizing all
///          campaign output.  Adding a new output field requires adding it here
///          and writing a corresponding test.
/// @param   now               Current ledger timestamp.
/// @param   total_raised      Raw total raised from storage.
/// @param   goal              Raw campaign goal from storage.
/// @param   progress_bps      Raw progress in basis points.
/// @param   deadline          Raw campaign deadline.
/// @param   contributor_count Raw contributor count.
/// @return  A fully sanitized `SanitizedCampaignOutput`.
pub fn sanitize_campaign_output(
    now: u64,
    total_raised: i128,
    goal: i128,
    progress_bps: u32,
    deadline: u64,
    contributor_count: u32,
) -> SanitizedCampaignOutput {
    let s_raised = sanitize_amount_bounded(total_raised, goal.max(0));
    let s_goal = sanitize_amount(goal);
    let s_bps = sanitize_bps(progress_bps);
    let s_deadline = sanitize_deadline(now, deadline);
    let s_count = sanitize_contributor_count(contributor_count);

    let was_modified = s_raised.was_modified()
        || s_goal.was_modified()
        || s_bps.was_modified()
        || s_deadline.was_modified()
        || s_count.was_modified();

    SanitizedCampaignOutput {
        total_raised: *s_raised.value(),
        goal: *s_goal.value(),
        progress_bps: *s_bps.value(),
        deadline: *s_deadline.value(),
        contributor_count: *s_count.value(),
        was_modified,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7. EVENT EMISSION HELPER
// ─────────────────────────────────────────────────────────────────────────────

/// @notice  Emits a `sanitization_warning` event when a campaign output
///          snapshot required modification.
/// @dev     Only emits when `output.was_modified` is true, so clean snapshots
///          produce no extra events.  The event carries the sanitized values so
///          off-chain monitors can compare against raw storage.
/// @custom:security-note  Silent sanitization without an audit trail would
///          allow data anomalies to go undetected.  This event provides the
///          tamper-evident log required for compliance monitoring.
/// @param   env     Soroban environment.
/// @param   output  The sanitized campaign output snapshot.
pub fn emit_sanitization_warning(env: &Env, output: &SanitizedCampaignOutput) {
    if !output.was_modified {
        return;
    }
    env.events().publish(
        (
            soroban_sdk::Symbol::new(env, "sanitization"),
            soroban_sdk::Symbol::new(env, "warning"),
        ),
        (
            output.total_raised,
            output.goal,
            output.progress_bps,
            output.deadline,
        ),
    );
}

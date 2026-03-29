//! # state_compression
//!
//! @title   StateCompression — Packed storage layout for gas-efficient reads/writes.
//!
//! @notice  Reduces ledger I/O costs by packing the four most-frequently-read
//!          campaign scalars (goal, deadline, min_contribution, total_raised)
//!          into a single `CompressedState` struct stored under one instance-
//!          storage key.
//!
//!          Key properties:
//!          - **4-in-1 reads** — one `get` instead of four separate reads.
//!          - **Atomic writes** — all four fields are updated together, preventing
//!            partial-write inconsistency.
//!          - **Checked arithmetic** — all mutations use `checked_*` to prevent
//!            silent overflow.
//!          - **Lazy initialisation** — `load_or_init` returns a zero-value struct
//!            when no state exists yet, avoiding a panic on first access.
//!
//! ## Security Assumptions
//!
//! 1. **No auth here** — this module is a pure storage helper; callers in
//!    `lib.rs` are responsible for authentication and campaign-status checks.
//! 2. **Overflow-safe** — `apply_contribution` and `apply_refund` return
//!    `None` on overflow/underflow; callers must handle the `None` case.
//! 3. **Non-negative invariant** — `total_raised` is never allowed to go below
//!    zero; `apply_refund` returns `None` if the subtraction would underflow.
//! 4. **Single key** — all four scalars share one instance-storage slot, so a
//!    concurrent upgrade that adds a new field must migrate existing data.

#![allow(dead_code)]

use soroban_sdk::{contracttype, Env};

// ── Storage key ───────────────────────────────────────────────────────────────

/// Instance-storage key for the compressed campaign state.
///
/// @dev Using a dedicated enum variant (rather than reusing `DataKey`) keeps
///      this module self-contained and avoids coupling to the main key enum.
#[derive(Clone)]
#[contracttype]
pub enum CompressedKey {
    /// The single slot that holds all four packed scalars.
    State,
}

// ── Packed struct ─────────────────────────────────────────────────────────────

/// @notice Packs the four most-read campaign scalars into one ledger entry.
///
/// @dev    Storing these together means a single `env.storage().instance().get`
///         fetches all four values, cutting read costs by ~75 % compared with
///         four individual `DataKey` lookups.
///
/// @param goal              Campaign funding goal in token units (must be > 0).
/// @param deadline          Campaign deadline as a Unix timestamp (seconds).
/// @param min_contribution  Minimum single-contribution amount (must be ≥ 1).
/// @param total_raised      Running total of tokens raised (must be ≥ 0).
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct CompressedState {
    pub goal: i128,
    pub deadline: u64,
    pub min_contribution: i128,
    pub total_raised: i128,
}

// ── Load / store ──────────────────────────────────────────────────────────────

/// @notice Loads the compressed state from instance storage.
/// @return `Some(CompressedState)` if initialised, `None` otherwise.
pub fn load(env: &Env) -> Option<CompressedState> {
    env.storage().instance().get(&CompressedKey::State)
}

/// @notice Loads the compressed state, returning a zero-value struct when absent.
///
/// @dev    Useful during `initialize` to avoid a panic on first access.
pub fn load_or_init(env: &Env) -> CompressedState {
    load(env).unwrap_or(CompressedState {
        goal: 0,
        deadline: 0,
        min_contribution: 0,
        total_raised: 0,
    })
}

/// @notice Persists the compressed state to instance storage.
///
/// @dev    Callers must ensure all four fields are consistent before calling.
///         In particular, `total_raised` must be ≤ `goal` is NOT enforced here
///         (over-funding is allowed by the campaign logic).
pub fn store(env: &Env, state: &CompressedState) {
    env.storage().instance().set(&CompressedKey::State, state);
}

// ── Atomic mutation helpers ───────────────────────────────────────────────────

/// @notice Adds `amount` to `total_raised` and persists the updated state.
///
/// @dev    Returns `None` (without writing) if the addition would overflow.
///         Callers must treat `None` as a hard error and revert.
///
/// @param env    Soroban environment.
/// @param amount Contribution amount (must be > 0; validated by caller).
/// @return Updated `CompressedState` on success, `None` on overflow.
pub fn apply_contribution(env: &Env, amount: i128) -> Option<CompressedState> {
    let mut state = load_or_init(env);
    state.total_raised = state.total_raised.checked_add(amount)?;
    store(env, &state);
    Some(state)
}

/// @notice Subtracts `amount` from `total_raised` and persists the updated state.
///
/// @dev    Returns `None` (without writing) if the subtraction would underflow
///         or if `total_raised` would go negative.  Callers must treat `None`
///         as a hard error and revert.
///
/// @param env    Soroban environment.
/// @param amount Refund amount (must be > 0; validated by caller).
/// @return Updated `CompressedState` on success, `None` on underflow.
pub fn apply_refund(env: &Env, amount: i128) -> Option<CompressedState> {
    let mut state = load_or_init(env);
    let new_total = state.total_raised.checked_sub(amount)?;
    if new_total < 0 {
        return None;
    }
    state.total_raised = new_total;
    store(env, &state);
    Some(state)
}

// ── Read-only helpers ─────────────────────────────────────────────────────────

/// @notice Returns `true` when the campaign goal has been reached.
///
/// @dev    Returns `false` when no state exists (uninitialised campaign).
pub fn is_goal_reached(env: &Env) -> bool {
    load(env).map_or(false, |s| s.total_raised >= s.goal)
}

/// @notice Returns `true` when the current ledger timestamp is past the deadline.
///
/// @dev    Returns `false` when no state exists.
pub fn is_expired(env: &Env) -> bool {
    load(env).map_or(false, |s| env.ledger().timestamp() > s.deadline)
}

/// @notice Returns the progress toward the goal in basis points (0–10 000).
///
/// @dev    Returns `0` when goal is zero or state is absent.
///         Saturates at 10 000 bps (100 %) when over-funded.
pub fn progress_bps(env: &Env) -> u32 {
    let Some(state) = load(env) else { return 0 };
    if state.goal <= 0 {
        return 0;
    }
    let bps = state
        .total_raised
        .checked_mul(10_000)
        .and_then(|v| v.checked_div(state.goal))
        .unwrap_or(0);
    bps.min(10_000) as u32
}

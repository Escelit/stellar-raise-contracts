//! # crowdfund_initialize_function
//!
//! @title   CrowdfundInitializeFunction — Validated, auditable, and
//!          frontend-ready initialization logic for the crowdfund contract.
//!
//! @notice  This module is the single authoritative location for all
//!          `initialize()` logic.  It provides:
//!
//!          - `InitParams` — a named struct that replaces nine positional
//!            arguments, eliminating silent parameter-order bugs.
//!          - Pure validation helpers for every parameter, each returning a
//!            typed `ContractError` so the frontend can map error codes to
//!            user-facing messages without string matching.
//!          - `execute_initialize()` — a deterministic, single-pass flow with
//!            a strict checks → effects → storage write ordering.
//!          - `describe_init_error()` / `is_init_error_retryable()` — helpers
//!            for off-chain scripts and frontend error handling.
//!
//! ## Performance Optimizations
//!
//! 1. **Early validation exit** — Uses `?` operator for short-circuit error
//!    propagation instead of nested `if let Err` blocks.
//!
//! Panics are opaque to the frontend — the SDK surfaces them as a generic host
//! error with no numeric code.  Typed `ContractError` variants let the frontend
//! display a specific message (e.g. "Platform fee exceeds 100%") without
//! parsing error strings.
//!
//! 3. **Batched validation** — All parameter checks run in a single
//!    `validate_init_params()` call, reducing function call overhead.
//!
//! 4. **Storage write batching** — All required storage writes are grouped
//!    together with only necessary conditional writes for optional fields.
//!
//! 5. **Optimized re-initialization guard** — Uses a single `has()` check on
//!    `DataKey::Creator` as the initialization sentinel, avoiding extra
//!    storage lookups.
//!
//! Interleaving validation and storage writes risks leaving the contract in a
//! partially-initialized state if a later check fails.  This module validates
//! all parameters first, then writes atomically.
//!
//! ## Security Assumptions
//!
//! 1. **Re-initialization guard** — `DataKey::Creator` is used as the
//!    initialization sentinel. The check is the very first operation so no
//!    state can be written before it.
//!
//! 2. **Creator authentication** — `creator.require_auth()` is called before
//!    any storage write. The Soroban host rejects the transaction if the
//!    creator's signature is absent or invalid.
//!
//! 3. **Goal floor** — `goal >= MIN_GOAL_AMOUNT (1)` prevents zero-goal
//!    campaigns that could be immediately drained by the creator.
//!
//! 4. **Minimum contribution floor** — `min_contribution >= MIN_CONTRIBUTION_AMOUNT (1)`
//!    prevents zero-amount contributions that waste gas and pollute storage.
//!
//! 5. **Deadline offset** — `deadline >= now + MIN_DEADLINE_OFFSET (60s)` ensures
//!    the campaign is live for at least one ledger close interval.
//!
//! 6. **Platform fee cap** — `fee_bps <= MAX_PLATFORM_FEE_BPS (10_000)` ensures
//!    the platform can never be configured to take more than 100% of raised funds.
//!
//! 7. **Bonus goal ordering** — `bonus_goal > goal` prevents a bonus goal that
//!    is already met at launch, which would immediately emit a bonus event.
//!
//! 8. **Atomic write ordering** — All validations complete before the first
//!    `env.storage().instance().set()` call. A failed validation leaves the
//!    contract in its pre-initialization state.
//!
//! ## Validation Flow
//!
//! ```text
//! execute_initialize(env, params)
//!        │
//!        ├─► re-initialization guard     → AlreadyInitialized
//!        ├─► creator.require_auth()
//!        ├─► validate_goal(goal)         → InvalidGoal
//!        ├─► validate_min_contribution() → InvalidMinContribution
//!        ├─► validate_deadline(now, dl)  → DeadlineTooSoon
//!        ├─► validate_platform_fee(bps)  → InvalidPlatformFee
//!        ├─► validate_bonus_goal(bg, g)  → InvalidBonusGoal
//!        │
//!        └─► [all checks passed] write storage → emit event → Ok(())
//! ```
//!
//! ## Frontend Integration
//!
//! 1. Call `initialize()` with a fully-populated parameter set.
//! 2. On success, listen for the `("campaign", "initialized")` event to
//!    confirm the campaign is live and cache the emitted parameters.
//! 3. On failure, map the returned `ContractError` code to a user message
//!    using `describe_init_error()` exported from this module.
//!
//! ## Scalability
//!
//! - `initialize()` is a one-shot O(1) function regardless of future campaign size.
//! - `Contributors` and `Roadmap` are seeded as empty vectors; their TTL is
//!   managed by `contribute()` and `add_roadmap_item()`.
//! - The `initialized` event payload is bounded to scalar values only.

#[allow(dead_code)]
use soroban_sdk::{Address, Env, String, Symbol, Vec};

use crate::campaign_goal_minimum::{
    validate_deadline, validate_goal, validate_min_contribution, validate_platform_fee,
};
use crate::{contract_state_size, ContractError, DataKey, PlatformConfig, RoadmapItem, Status};

// ── InitParams ────────────────────────────────────────────────────────────────

/// All parameters required to initialize a crowdfund campaign.
///
/// @dev Using a named struct instead of positional arguments prevents silent
///      parameter-order bugs (e.g. swapping two `i128` fields compiles but
///      produces incorrect state).
///
/// # Type Parameters
/// * `T` - Any type that implements the required trait bounds for Address
#[derive(Clone)]
pub struct InitParams {
    /// The admin address authorized to upgrade the contract.
    ///
    /// @notice The admin is separate from the creator so that a platform
    ///         operator can retain upgrade rights without being the campaign
    ///         creator.
    pub admin: Address,

    /// The campaign creator's address.
    ///
    /// @notice Must authorize the `initialize()` call. Stored as the
    ///         re-initialization sentinel.
    pub creator: Address,

    /// The Stellar asset contract address used for contributions.
    ///
    /// @notice Must be a valid token contract implementing the SEP-41 interface.
    pub token: Address,

    /// The funding goal in the token's smallest unit (e.g. stroops for XLM).
    ///
    /// @notice Must be >= `MIN_GOAL_AMOUNT` (1).
    pub goal: i128,

    /// The campaign deadline as a Unix timestamp (seconds since epoch).
    ///
    /// @notice Must be at least `MIN_DEADLINE_OFFSET` (60) seconds after the
    ///         current ledger timestamp.
    pub deadline: u64,

    /// The minimum contribution amount in the token's smallest unit.
    ///
    /// @notice Must be >= `MIN_CONTRIBUTION_AMOUNT` (1). Setting this to a
    ///         meaningful value (e.g. 1_000 stroops) prevents dust attacks.
    pub min_contribution: i128,

    /// Optional platform fee configuration.
    ///
    /// @notice When `Some`, the platform address receives `fee_bps / 10_000`
    ///         of the total raised on a successful withdrawal.
    ///         `fee_bps` must be <= `MAX_PLATFORM_FEE_BPS` (10_000 = 100%).
    pub platform_config: Option<PlatformConfig>,

    /// Optional secondary bonus goal threshold.
    ///
    /// @notice When `Some`, must be strictly greater than `goal`. Reaching
    ///         this threshold emits a `bonus_goal_reached` event exactly once.
    pub bonus_goal: Option<i128>,

    /// Optional human-readable description for the bonus goal.
    ///
    /// @notice Stored as-is.  The frontend should enforce a reasonable display limit.
    pub bonus_goal_description: Option<String>,
}

// ── Validation helpers ───────────────────────────────────────────────────────

/// Validates that `bonus_goal`, when present, is strictly greater than `goal`.
///
/// @param  bonus_goal  The optional bonus goal value.
/// @param  goal        The primary campaign goal.
/// @return             `Ok(())` if valid or absent; `Err(ContractError::InvalidBonusGoal)` otherwise.
///
/// @dev    A bonus goal equal to the primary goal would be met simultaneously,
///         making it meaningless.  A bonus goal below the primary goal would be
///         met before the campaign succeeds, which is logically inconsistent.
#[inline]
pub fn validate_bonus_goal(bonus_goal: Option<i128>, goal: i128) -> Result<(), ContractError> {
    if let Some(bg) = bonus_goal {
        if bg <= goal {
            return Err(ContractError::InvalidBonusGoal);
        }
    }
    Ok(())
}

/// Validates the bonus goal description length if present.
///
/// @param  description  The optional bonus goal description.
/// @return              `Ok(())` if valid or absent; `Err(ContractError::InvalidBonusGoalDescription)` otherwise.
///
/// @dev    Description length validation prevents unbounded state growth
///         that could increase storage costs and impact contract performance.
#[inline]
pub fn validate_bonus_goal_description(description: &Option<String>) -> Result<(), ContractError> {
    if let Some(desc) = description {
        if let Err(err) = contract_state_size::validate_bonus_goal_description(desc) {
            return Err(ContractError::InvalidBonusGoalDescription);
        }
    }
    Ok(())
}

/// Validates all `InitParams` fields in a single pass.
///
/// @param  env     The Soroban execution environment (used for ledger timestamp).
/// @param  params  The initialization parameters to validate.
/// @return         `Ok(())` if all fields are valid; the first `ContractError` encountered otherwise.
///
/// @dev    Validation order matches the storage write order in `execute_initialize()`
///         so that error codes are predictable and auditable.
pub fn validate_init_params(env: &Env, params: &InitParams) -> Result<(), ContractError> {
    validate_goal(params.goal)?;
    validate_min_contribution(params.min_contribution)?;
    validate_deadline(env.ledger().timestamp(), params.deadline)?;

    if let Some(ref config) = params.platform_config {
        validate_platform_fee(config.fee_bps)?;
    }

    validate_bonus_goal(params.bonus_goal, params.goal)?;
    validate_bonus_goal_description(&params.bonus_goal_description)?;

    Ok(())
}

// ── Core initialization logic ─────────────────────────────────────────────────

/// Executes the full campaign initialization flow.
///
/// @notice This is the single authoritative implementation of campaign
///         initialization. `CrowdfundContract::initialize()` in `lib.rs`
///         delegates to this function after constructing `InitParams`.
///
/// @param  env     The Soroban execution environment.
/// @param  params  Fully-populated initialization parameters.
/// @return         `Ok(())` on success; a typed `ContractError` on failure.
///
/// @dev    Strict ordering guarantee:
///         1. Re-initialization guard (read-only check, no state mutation).
///         2. Creator authentication (`require_auth`).
///         3. Full parameter validation (no storage writes yet).
///         4. Storage writes (all-or-nothing within the transaction).
///         5. Event emission.
///
/// @security  The re-initialization guard uses `DataKey::Creator` as the
///            sentinel because it is always written during initialization and
///            is never deleted.  A failed validation at step 3 leaves the
///            contract in its pre-initialization state — no partial writes.
pub fn execute_initialize(env: &Env, params: InitParams) -> Result<(), ContractError> {
    // ── 1. Re-initialization guard ────────────────────────────────────────
    // Must be the very first check so no state can be written before it.
    if env.storage().instance().has(&DataKey::Creator) {
        return Err(ContractError::AlreadyInitialized);
    }

    // ── 2. Creator authentication ─────────────────────────────────────────
    // Called before any state mutation so an unauthorized call cannot leave
    // partial state.
    params.creator.require_auth();

    // ── 3. Parameter validation ───────────────────────────────────────────
    // All checks run before the first storage write. A failed check leaves
    // the contract in its pre-initialization state.
    validate_init_params(env, &params)?;

    // ── 4. Storage writes ─────────────────────────────────────────────────
    // Admin stored first so upgrade authorization is available immediately.
    env.storage().instance().set(&DataKey::Admin, &params.admin);

    // Core campaign fields.
    env.storage()
        .instance()
        .set(&DataKey::Creator, &params.creator);
    env.storage().instance().set(&DataKey::Token, &params.token);
    env.storage().instance().set(&DataKey::Goal, &params.goal);
    env.storage()
        .instance()
        .set(&DataKey::Deadline, &params.deadline);
    env.storage()
        .instance()
        .set(&DataKey::MinContribution, &params.min_contribution);

    // Counters and status — always initialized to known-good defaults.
    env.storage().instance().set(&DataKey::TotalRaised, &0i128);
    env.storage()
        .instance()
        .set(&DataKey::BonusGoalReachedEmitted, &false);
    env.storage()
        .instance()
        .set(&DataKey::Status, &Status::Active);

    // Optional platform configuration.
    if let Some(ref config) = params.platform_config {
        env.storage()
            .instance()
            .set(&DataKey::PlatformConfig, config);
    }

    // Optional bonus goal.
    if let Some(bg) = params.bonus_goal {
        env.storage().instance().set(&DataKey::BonusGoal, &bg);
    }
    if let Some(ref bg_desc) = params.bonus_goal_description {
        env.storage()
            .instance()
            .set(&DataKey::BonusGoalDescription, bg_desc);
    }

    // Seed empty collections in persistent storage.
    let empty_contributors: Vec<Address> = Vec::new(env);
    env.storage()
        .persistent()
        .set(&DataKey::Contributors, &empty_contributors);

    let empty_roadmap: Vec<RoadmapItem> = Vec::new(env);
    env.storage()
        .instance()
        .set(&DataKey::Roadmap, &empty_roadmap);

    // ── 5. Event emission ─────────────────────────────────────────────────
    // Emit a bounded event so off-chain indexers can reconstruct campaign
    // state from the event stream without polling individual storage keys.
    // Only scalar fields are included — no optional strings — to keep the
    // payload size O(1) regardless of bonus_goal_description length.
    log_initialize(
        env,
        &params.creator,
        &params.token,
        params.goal,
        params.deadline,
        params.min_contribution,
    );

    Ok(())
}

// ── Bounded initialization event ──────────────────────────────────────────────

/// Emits a single bounded `("campaign", "initialized")` event.
///
/// @notice Only scalar fields are included in the payload. Optional strings
///         (e.g. `bonus_goal_description`) are intentionally excluded to keep
///         event size O(1) and prevent unbounded gas consumption when long
///         descriptions are provided.
///
/// @param  env              The Soroban execution environment.
/// @param  creator          The campaign creator address.
/// @param  token            The token contract address.
/// @param  goal             The funding goal.
/// @param  deadline         The campaign deadline timestamp.
/// @param  min_contribution The minimum contribution amount.
///
/// @dev    Callers must not pass unbounded data (e.g. raw description strings)
///         to this function. All string fields must be omitted or pre-truncated
///         before calling.
pub fn log_initialize(
    env: &Env,
    creator: &Address,
    token: &Address,
    goal: i128,
    deadline: u64,
    min_contribution: i128,
) {
    env.events().publish(
        (
            Symbol::new(env, "campaign"),
            Symbol::new(env, "initialized"),
        ),
        (
            creator.clone(),
            token.clone(),
            goal,
            deadline,
            min_contribution,
        ),
    );
}

// ── Error description helpers ─────────────────────────────────────────────────

/// Returns a human-readable description for an `initialize()`-related error code.
///
/// @param  code  The numeric `ContractError` repr value (e.g. `error as u32`).
/// @return       A static string suitable for display in a frontend error message.
///
/// @dev    Off-chain scripts and frontends should use this instead of hardcoding
///         strings so that a future code change only requires updating this function.
pub fn describe_init_error(code: u32) -> &'static str {
    match code {
        1 => "Contract is already initialized",
        8 => "Campaign goal must be at least 1",
        9 => "Minimum contribution must be at least 1",
        10 => "Deadline must be at least 60 seconds in the future",
        11 => "Platform fee cannot exceed 100% (10,000 bps)",
        12 => "Bonus goal must be strictly greater than the primary goal",
        _ => "Unknown initialization error",
    }
}

/// Returns `true` if the error code corresponds to a client-side input error
/// that can be corrected and retried.
///
/// @param  code  The numeric `ContractError` repr value.
/// @return       `true` for correctable input errors; `false` for permanent failures.
///
/// @dev    `AlreadyInitialized` (1) is permanent — the contract cannot be
///         re-initialized.  All other init errors are input validation failures
///         that the caller can fix and retry.
pub fn is_init_error_retryable(code: u32) -> bool {
    matches!(code, 8 | 9 | 10 | 11 | 12)
}

// ── Minimum goal re-export ────────────────────────────────────────────────────

/// Re-exports `MIN_GOAL_AMOUNT` for callers that only import this module.
pub use crate::campaign_goal_minimum::MIN_GOAL_AMOUNT as INIT_MIN_GOAL_AMOUNT;

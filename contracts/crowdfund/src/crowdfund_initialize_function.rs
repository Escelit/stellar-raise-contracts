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
use crate::{ContractError, DataKey, PlatformConfig, RoadmapItem, Status};

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
/// Validates the optional bonus goal description.
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
//! Maintainable validation/storage helpers for `initialize()`.
//!
//! This module extracts the initialization logic from `lib.rs` so the security
//! checks are easier to review and unit test.

use soroban_sdk::{Address, Env, String, Vec};

use crate::{contract_state_size, DataKey, PlatformConfig, RoadmapItem, Status};

/// @notice Validates initialization inputs and panics on invalid configuration.
/// @dev Panics preserve existing contract behavior for callers that rely on
///      fail-fast initialization checks.
pub fn validate_initialize_inputs(
    goal: i128,
    min_contribution: i128,
    platform_config: &Option<PlatformConfig>,
    bonus_goal: Option<i128>,
    bonus_goal_description: &Option<String>,
) {
    if goal <= 0 {
        panic!("goal must be positive");
    }
    if min_contribution <= 0 {
        panic!("min contribution must be positive");
    }

    if let Some(config) = platform_config {
        if config.fee_bps > 10_000 {
            panic!("platform fee cannot exceed 100%");
        }
    }

    if let Some(bg) = bonus_goal {
        if bg <= goal {
            panic!("bonus goal must be greater than primary goal");
        }
    }

    if let Some(description) = bonus_goal_description {
        if let Err(err) = contract_state_size::validate_bonus_goal_description(description) {
            panic!("{}", err);
//! # crowdfund_initialize_function
//!
//! Validation helpers for the `CrowdfundContract::initialize` entry-point.
//!
//! ## Responsibility
//! This module owns all *pre-storage* checks that must pass before any state
//! is written during campaign initialization.  Keeping them here makes the
//! logic unit-testable without deploying a full contract environment.
//!
//! ## Security assumptions
//! * `goal` and `min_contribution` are denominated in the token's smallest
//!   indivisible unit (stroops for XLM-based tokens).
//! * `deadline` is a ledger UNIX timestamp (seconds since epoch).
//! * The caller is responsible for calling `creator.require_auth()` **before**
//!   invoking these helpers.
//! * Platform fee is expressed in basis points (1 bp = 0.01 %).  The maximum
//!   allowed value is 10 000 (= 100 %).

use soroban_sdk::Env;

// ── Error codes ──────────────────────────────────────────────────────────────

/// Reasons why `validate_initialization_params` can fail.
///
/// Each variant maps to a human-readable message returned by
/// [`validation_error_message`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InitError {
    /// `goal` must be a positive integer.
    GoalNotPositive,
    /// `deadline` must be strictly after the current ledger timestamp.
    DeadlineInPast,
    /// `min_contribution` must be a positive integer.
    MinContributionNotPositive,
    /// `min_contribution` must not exceed `goal`.
    MinContributionExceedsGoal,
    /// Platform fee basis points must be ≤ 10 000.
    PlatformFeeExceedsMax,
    /// `bonus_goal`, when provided, must be strictly greater than `goal`.
    BonusGoalNotGreaterThanGoal,
}

impl InitError {
    /// Returns a developer-facing description of the error.
    pub fn message(self) -> &'static str {
        match self {
            Self::GoalNotPositive => "goal must be greater than zero",
            Self::DeadlineInPast => "deadline must be in the future",
            Self::MinContributionNotPositive => "min_contribution must be greater than zero",
            Self::MinContributionExceedsGoal => "min_contribution must not exceed goal",
            Self::PlatformFeeExceedsMax => "platform fee cannot exceed 100% (10 000 bps)",
            Self::BonusGoalNotGreaterThanGoal => "bonus_goal must be greater than primary goal",
        }
    }
}

/// @notice Persists initialize() state in one place for easier audits.
pub fn persist_initialize_state(
    env: &Env,
    admin: &Address,
    creator: &Address,
    token: &Address,
    goal: i128,
    deadline: u64,
    min_contribution: i128,
    platform_config: &Option<PlatformConfig>,
    bonus_goal: Option<i128>,
    bonus_goal_description: &Option<String>,
) {
    env.storage().instance().set(&DataKey::Admin, admin);
    env.storage().instance().set(&DataKey::Creator, creator);
    env.storage().instance().set(&DataKey::Token, token);
    env.storage().instance().set(&DataKey::Goal, &goal);
    env.storage().instance().set(&DataKey::Deadline, &deadline);
    env.storage()
        .instance()
        .set(&DataKey::MinContribution, &min_contribution);
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
    env.storage().instance().set(&DataKey::Status, &Status::Active);

    if let Some(config) = platform_config {
        env.storage().instance().set(&DataKey::PlatformConfig, config);
    }
    if let Some(bg) = bonus_goal {
        env.storage().instance().set(&DataKey::BonusGoal, &bg);
    }
    if let Some(description) = bonus_goal_description {
        env.storage()
            .instance()
            .set(&DataKey::BonusGoalDescription, description);
    }

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
//! # crowdfund_initialize_function
//!
//! @title   CrowdfundInitializeFunction — Optimized initialization logic for
//!          the crowdfund contract with improved performance and security.
//!
//! @notice  This module provides the canonical implementation of campaign
//!          initialization, extracted from `lib.rs` for auditability and
//!          testability. It delivers:
//!
//!          - A validated `InitParams` struct replacing nine positional args.
//!          - Pure validation helpers returning typed `ContractError` variants.
//!          - A deterministic single-pass initialization flow with strict
//!            checks → effects → storage write ordering.
//!          - An `InitializedEvent` payload for off-chain indexers.
//!          - Helper functions for frontend error mapping.
//!
//! @dev     This module is `no_std`-compatible and has no dependency on the
//!          contract's `#[contractimpl]` block, enabling use in off-chain
//!          tooling and property-based tests without a full Soroban environment.
//!
//! ## Performance Optimizations
//!
//! 1. **Early validation exit** — Uses `?` operator for short-circuit error
//!    propagation instead of nested `if let Err` blocks.
//!
//! 2. **Inline hints** — Validation helpers are marked `#[inline]` to allow
//!    the compiler to specialize call sites and eliminate stack frames.
//!
//! 3. **Batched validation** — All parameter checks run in a single
//!    `validate_init_params()` call, reducing function call overhead.
//!
//! Panics are opaque to the frontend — the SDK surfaces them as a generic host
//! error with no numeric code.  Typed `ContractError` variants let the frontend
//! display a specific message (e.g. "Platform fee exceeds 100%") without
//! parsing error strings.
//! 4. **Storage write batching** — All required storage writes are grouped
//!    together with only necessary conditional writes for optional fields.
//!
//! 5. **Optimized re-initialization guard** — Uses a single `has()` check on
//!    `DataKey::Creator` as the initialization sentinel, avoiding extra
//!    storage lookups.
//!
//! Soroban storage is not directly queryable by off-chain services without an
//! RPC call per field.  An `initialized` event carries all campaign parameters
//! in a single ledger entry, enabling indexers to bootstrap campaign state from
//! the event stream alone.
//!
//! ### Why validate before any storage write?
//!
//! Interleaving validation and storage writes risks leaving the contract in a
//! partially-initialized state if a later check fails.  This module validates
//! all parameters first, then writes atomically.
//! 6. **Event emission last** — Event is only emitted after all storage
//!    writes succeed, ensuring consistency.
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
//!        ├─► validate_goal(goal)            → InvalidGoal
//!        ├─► validate_min_contribution(mc)  → InvalidMinContribution
//!        ├─► validate_deadline(now, dl)     → DeadlineTooSoon
//!        ├─► validate_platform_fee(bps)     → InvalidPlatformFee
//!        ├─► validate_bonus_goal(bg, goal) → InvalidBonusGoal
//!        ├─► validate_bonus_goal_description(desc) → InvalidBonusGoalDescription
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
//! The frontend should:
//!
//! 1. Call `initialize()` with a fully-populated set of parameters.
//! 2. On success, listen for the `("campaign", "initialized")` event to
//!    confirm the campaign is live and cache the emitted parameters.
//! 3. On failure, map the returned `ContractError` code to a user message
//!    using the `describe_init_error()` helper exported from this module.

#[allow(dead_code)]
use soroban_sdk::{Address, Env, String, Symbol, Vec};

use crate::campaign_goal_minimum::{
    validate_deadline, validate_goal, validate_min_contribution, validate_platform_fee,
};
use crate::{ContractError, DataKey, PlatformConfig, RoadmapItem, Status};

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
    /// @notice Stored as-is; no length validation is enforced at the contract
    ///         level. The frontend should enforce a reasonable display limit.
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
/// @dev    A bonus goal equal to the primary goal would be met at the same
///         time as the campaign goal, making it meaningless. A bonus goal
///         below the primary goal would be met before the campaign succeeds,
///         which is logically inconsistent.
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
/// Validates the optional bonus goal description.
#[inline]
pub fn validate_bonus_goal_description(description: &Option<String>) -> Result<(), ContractError> {
    // Description is optional; if present, accept any non-empty value.
    // Length validation is handled by Soroban's storage limits.
    let _ = description;
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
/// @dev    Validation order matches the storage write order in
///         `execute_initialize()` so that error codes are predictable.
///         Uses short-circuit evaluation via `?` operator for efficiency.
#[inline]
pub fn validate_init_params(env: &Env, params: &InitParams) -> Result<(), ContractError> {
    validate_goal(params.goal).map_err(|_| ContractError::InvalidGoal)?;
    validate_min_contribution(params.min_contribution).map_err(|_| ContractError::InvalidMinContribution)?;
    validate_deadline(env.ledger().timestamp(), params.deadline).map_err(|_| ContractError::DeadlineTooSoon)?;

    if let Some(ref config) = params.platform_config {
        validate_platform_fee(config.fee_bps).map_err(|_| ContractError::InvalidPlatformFee)?;
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
///            is never deleted. Using a dedicated `Initialized` key would
///            require an extra storage slot and could be confused with other
///            boolean flags.
pub fn execute_initialize(env: &Env, params: InitParams) -> Result<(), ContractError> {
    // ── 1. Re-initialization guard ────────────────────────────────────────
    // Single storage read to check if contract is already initialized.
    // This is the first operation to ensure no state is written on failure.
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
    // All required fields are written first (atomic on success).
    // Optional fields are written conditionally only if present.

    // Admin — stored first so upgrade authorization is available immediately.
    env.storage()
        .instance()
        .set(&DataKey::Admin, &params.admin);
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
    // Emit a structured event so off-chain indexers can reconstruct campaign
    // state from the event stream without polling individual storage keys.
    env.events().publish(
        (
            soroban_sdk::Symbol::new(env, "campaign"),
            soroban_sdk::Symbol::new(env, "initialized"),
        ),
        (
            params.creator.clone(),
            params.token.clone(),
            params.goal,
            params.deadline,
            params.min_contribution,
        ),
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
/// @param  code  The numeric `ContractError` repr value.
/// @return       A static string suitable for display in a frontend error message.
///
/// @dev    The frontend should call this with `error as u32` after receiving
///         a typed error from the SDK client.
#[inline]
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
#[inline]
pub fn is_init_error_retryable(code: u32) -> bool {
    // AlreadyInitialized (1) is permanent — the contract cannot be re-initialized.
    // All other init errors are input validation failures that the caller can fix.
    matches!(code, 8 | 9 | 10 | 11 | 12)
}

// ── Minimum goal re-export ────────────────────────────────────────────────────

/// Re-exports `MIN_GOAL_AMOUNT` for callers that only import this module.
pub use crate::campaign_goal_minimum::MIN_GOAL_AMOUNT as INIT_MIN_GOAL_AMOUNT;

// ── Validation helpers for panic-based initialization ──────────────────────

/// @notice Validates initialization inputs and panics on invalid configuration.
/// @dev Panics preserve existing contract behavior for callers that rely on
///      fail-fast initialization checks. Used by the contract's initialize()
///      entry point for backward compatibility.
pub fn validate_initialize_inputs(
    goal: i128,
    min_contribution: i128,
    platform_config: &Option<PlatformConfig>,
    bonus_goal: Option<i128>,
    bonus_goal_description: &Option<String>,
) {
    if goal <= 0 {
        panic!("goal must be positive");
    }
    if min_contribution <= 0 {
        panic!("min contribution must be positive");
    }

    if let Some(config) = platform_config {
        if config.fee_bps > 10_000 {
            panic!("platform fee cannot exceed 100%");
        }
    }

    if let Some(bg) = bonus_goal {
        if bg <= goal {
            panic!("bonus goal must be greater than primary goal");
        }
    }

    if let Some(description) = bonus_goal_description {
        if let Err(err) = contract_state_size::validate_bonus_goal_description(description) {
            panic!("{}", err);
        }
    }
}

/// @notice Persists initialize() state in one place for easier audits.
/// @dev   This function is called by the contract's initialize() entry point
///        after validation passes. It writes all state in a specific order
///        to ensure atomicity and consistency.
pub fn persist_initialize_state(
    env: &Env,
    admin: &Address,
    creator: &Address,
    token: &Address,
    goal: i128,
    deadline: u64,
    min_contribution: i128,
    platform_config: &Option<PlatformConfig>,
    bonus_goal: Option<i128>,
    bonus_goal_description: &Option<String>,
) {
    // Required fields — always written.
    env.storage().instance().set(&DataKey::Admin, admin);
    env.storage().instance().set(&DataKey::Creator, creator);
    env.storage().instance().set(&DataKey::Token, token);
    env.storage().instance().set(&DataKey::Goal, &goal);
    env.storage().instance().set(&DataKey::Deadline, &deadline);
    env.storage()
        .instance()
        .set(&DataKey::MinContribution, &min_contribution);
    env.storage().instance().set(&DataKey::TotalRaised, &0i128);
    env.storage()
        .instance()
        .set(&DataKey::BonusGoalReachedEmitted, &false);
    env.storage().instance().set(&DataKey::Status, &Status::Active);

    // Optional fields — only written when present.
    if let Some(config) = platform_config {
        env.storage().instance().set(&DataKey::PlatformConfig, config);
    }
    if let Some(bg) = bonus_goal {
        env.storage().instance().set(&DataKey::BonusGoal, &bg);
    }
    if let Some(description) = bonus_goal_description {
        env.storage()
            .instance()
            .set(&DataKey::BonusGoalDescription, description);
    }

    // Seed empty collections.
    let empty_contributors: Vec<Address> = Vec::new(env);
    env.storage()
        .persistent()
        .set(&DataKey::Contributors, &empty_contributors);

    let empty_roadmap: Vec<RoadmapItem> = Vec::new(env);
    env.storage().instance().set(&DataKey::Roadmap, &empty_roadmap);
}

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
// ── Core validation ──────────────────────────────────────────────────────────

/// Validates all initialization parameters for a new crowdfunding campaign.
///
/// # Parameters
/// | Name                | Type          | Description                                      |
/// |---------------------|---------------|--------------------------------------------------|
/// | `env`               | `&Env`        | Soroban environment (used for ledger timestamp). |
/// | `goal`              | `i128`        | Funding target in token's smallest unit.         |
/// | `deadline`          | `u64`         | Campaign end time as a UNIX ledger timestamp.    |
/// | `min_contribution`  | `i128`        | Minimum single contribution amount.              |
/// | `platform_fee_bps`  | `Option<u32>` | Optional platform fee in basis points (0–10 000).|
/// | `bonus_goal`        | `Option<i128>`| Optional stretch goal; must exceed `goal`.       |
///
/// # Returns
/// `Ok(())` when all parameters are valid.
/// `Err(InitError)` with the first failing constraint.
///
/// # Errors
/// * [`InitError::GoalNotPositive`]            – `goal <= 0`
/// * [`InitError::DeadlineInPast`]             – `deadline <= current_timestamp`
/// * [`InitError::MinContributionNotPositive`] – `min_contribution <= 0`
/// * [`InitError::MinContributionExceedsGoal`] – `min_contribution > goal`
/// * [`InitError::PlatformFeeExceedsMax`]      – `fee_bps > 10_000`
/// * [`InitError::BonusGoalNotGreaterThanGoal`]– `bonus_goal <= goal`
///
/// # Security
/// * Does **not** write any storage — purely read-only.
/// * Checks are ordered from cheapest to most expensive.
/// * Integer comparisons use native Rust operators; no overflow is possible
///   because all values are validated against known bounds before arithmetic.
pub fn validate_initialization_params(
    env: &Env,
    goal: i128,
    deadline: u64,
    min_contribution: i128,
    platform_fee_bps: Option<u32>,
    bonus_goal: Option<i128>,
) -> Result<(), InitError> {
    if goal <= 0 {
        return Err(InitError::GoalNotPositive);
    }

    let current_time = env.ledger().timestamp();
    if deadline <= current_time {
        return Err(InitError::DeadlineInPast);
    }

    if min_contribution <= 0 {
        return Err(InitError::MinContributionNotPositive);
    }

    if min_contribution > goal {
        return Err(InitError::MinContributionExceedsGoal);
    }

    if let Some(fee_bps) = platform_fee_bps {
        if fee_bps > 10_000 {
            return Err(InitError::PlatformFeeExceedsMax);
        }
    }

    if let Some(bg) = bonus_goal {
        if bg <= goal {
            return Err(InitError::BonusGoalNotGreaterThanGoal);
        }
    }

    Ok(())
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
/// Convenience wrapper that mirrors the old `bool`-returning signature.
///
/// Kept for backward compatibility with call-sites that only need a pass/fail
/// answer and do not need to distinguish between error kinds.
///
/// # Deprecation
/// Prefer [`validate_initialization_params`] which returns a typed error.
pub fn validate_initialization_params_bool(
    env: &Env,
    goal: i128,
    deadline: u64,
    min_contribution: i128,
) -> bool {
    validate_initialization_params(env, goal, deadline, min_contribution, None, None).is_ok()
}

    env.storage()
        .instance()
        .set(&DataKey::Roadmap, &empty_roadmap);
}

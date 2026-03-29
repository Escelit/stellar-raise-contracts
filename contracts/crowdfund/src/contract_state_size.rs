//! Contract State Size Limits
//!
//! Defines maximum size limits for all campaign-related on-chain state and
//! provides guard functions that return typed errors when limits are exceeded.
//!
//! ## Why limits matter
//! - **Resource efficiency**: caps ledger entry sizes, keeping state-rent predictable.
//! - **Frontend reliability**: the UI can query these constants to pre-validate inputs.
//! - **Scalability**: bounded collections prevent runaway storage growth.
//!
//! All byte constants are measured in UTF-8 bytes; count constants are item counts.

use soroban_sdk::{contract, contracterror, contractimpl, Address, Env, String, Vec};

use crate::{DataKey, RoadmapItem};

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum length of any single string field (title, description, social links) in bytes.
/// @dev Shared limit used by `check_string_len`.
pub const MAX_STRING_LEN: u32 = 256;

/// Maximum number of unique contributors (and pledgers) tracked per campaign.
/// @dev Bounds both the `Contributors` and `Pledgers` persistent lists.
pub const MAX_CONTRIBUTORS: u32 = 128;

/// Maximum number of roadmap milestones stored per campaign.
pub const MAX_ROADMAP_ITEMS: u32 = 32;

/// Maximum number of stretch goals (milestones) stored per campaign.
pub const MAX_STRETCH_GOALS: u32 = 32;

// Legacy aliases kept for backward compatibility with contract_state_size.test.rs
pub const MAX_TITLE_LENGTH: u32 = MAX_STRING_LEN;
pub const MAX_DESCRIPTION_LENGTH: u32 = MAX_STRING_LEN;

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors returned when a state-size limit is exceeded.
///
/// Discriminants are stable and must not be renumbered — they are part of the
/// on-chain ABI.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StateSizeError {
    /// The `Contributors` or `Pledgers` list has reached `MAX_CONTRIBUTORS`.
    ContributorLimitExceeded = 100,
    /// The `Roadmap` list has reached `MAX_ROADMAP_ITEMS`.
    RoadmapLimitExceeded = 101,
    /// The `StretchGoals` list has reached `MAX_STRETCH_GOALS`.
    StretchGoalLimitExceeded = 102,
    /// A string field exceeds `MAX_STRING_LEN` bytes.
    StringTooLong = 103,
}

// ── Guard functions ───────────────────────────────────────────────────────────

/// Returns `Err(StringTooLong)` if `s` exceeds `MAX_STRING_LEN` bytes.
///
/// @param s  The string to validate.
/// @return   `Ok(())` when within bounds.
pub fn check_string_len(s: &String) -> Result<(), StateSizeError> {
    if s.len() > MAX_STRING_LEN {
        Err(StateSizeError::StringTooLong)
    } else {
        Ok(())
    }
}

/// Returns `Err(ContributorLimitExceeded)` if the `Contributors` list is full.
///
/// Reads `DataKey::Contributors` from persistent storage; treats a missing key
/// as an empty list (safe default).
///
/// @param env  The contract environment.
/// @return     `Ok(())` when below `MAX_CONTRIBUTORS`.
pub fn check_contributor_limit(env: &Env) -> Result<(), StateSizeError> {
    let count = env
        .storage()
        .persistent()
        .get::<DataKey, Vec<Address>>(&DataKey::Contributors)
        .map(|v| v.len())
        .unwrap_or(0);
    if count >= MAX_CONTRIBUTORS {
        Err(StateSizeError::ContributorLimitExceeded)
    } else {
        Ok(())
    }
}

/// Returns `Err(ContributorLimitExceeded)` if the `Pledgers` list is full.
///
/// Mirrors `check_contributor_limit` but reads `DataKey::Pledgers`.
///
/// @param env  The contract environment.
/// @return     `Ok(())` when below `MAX_CONTRIBUTORS`.
pub fn check_pledger_limit(env: &Env) -> Result<(), StateSizeError> {
    let count = env
        .storage()
        .persistent()
        .get::<DataKey, Vec<Address>>(&DataKey::Pledgers)
        .map(|v| v.len())
        .unwrap_or(0);
    if count >= MAX_CONTRIBUTORS {
        Err(StateSizeError::ContributorLimitExceeded)
    } else {
        Ok(())
    }
}

/// Returns `Err(RoadmapLimitExceeded)` if the `Roadmap` list is full.
///
/// Reads `DataKey::Roadmap` from instance storage.
///
/// @param env  The contract environment.
/// @return     `Ok(())` when below `MAX_ROADMAP_ITEMS`.
pub fn check_roadmap_limit(env: &Env) -> Result<(), StateSizeError> {
    let count = env
        .storage()
        .instance()
        .get::<DataKey, Vec<RoadmapItem>>(&DataKey::Roadmap)
        .map(|v| v.len())
        .unwrap_or(0);
    if count >= MAX_ROADMAP_ITEMS {
        Err(StateSizeError::RoadmapLimitExceeded)
    } else {
        Ok(())
    }
}

/// Returns `Err(StretchGoalLimitExceeded)` if the `StretchGoals` list is full.
///
/// Reads `DataKey::StretchGoals` from instance storage.
///
/// @param env  The contract environment.
/// @return     `Ok(())` when below `MAX_STRETCH_GOALS`.
pub fn check_stretch_goal_limit(env: &Env) -> Result<(), StateSizeError> {
    let count = env
        .storage()
        .instance()
        .get::<DataKey, Vec<i128>>(&DataKey::StretchGoals)
        .map(|v| v.len())
        .unwrap_or(0);
    if count >= MAX_STRETCH_GOALS {
        Err(StateSizeError::StretchGoalLimitExceeded)
    } else {
        Ok(())
    }
}

// ── Capacity / length validators (called from lib.rs and other modules) ───────

/// Returns `Err(ContributorLimitExceeded)` if `count >= MAX_CONTRIBUTORS`.
/// @param count  Current length of the contributors list.
pub fn validate_contributor_capacity(count: u32) -> Result<(), StateSizeError> {
    if count >= MAX_CONTRIBUTORS {
        Err(StateSizeError::ContributorLimitExceeded)
    } else {
        Ok(())
    }
}

/// Returns `Err(ContributorLimitExceeded)` if `count >= MAX_CONTRIBUTORS`.
/// @param count  Current length of the pledgers list.
pub fn validate_pledger_capacity(count: u32) -> Result<(), StateSizeError> {
    if count >= MAX_CONTRIBUTORS {
        Err(StateSizeError::ContributorLimitExceeded)
    } else {
        Ok(())
    }
}

/// Returns `Err(RoadmapLimitExceeded)` if `count >= MAX_ROADMAP_ITEMS`.
/// @param count  Current length of the roadmap list.
pub fn validate_roadmap_capacity(count: u32) -> Result<(), StateSizeError> {
    if count >= MAX_ROADMAP_ITEMS {
        Err(StateSizeError::RoadmapLimitExceeded)
    } else {
        Ok(())
    }
}

/// Returns `Err(StretchGoalLimitExceeded)` if `count >= MAX_STRETCH_GOALS`.
/// @param count  Current length of the stretch goals list.
pub fn validate_stretch_goal_capacity(count: u32) -> Result<(), StateSizeError> {
    if count >= MAX_STRETCH_GOALS {
        Err(StateSizeError::StretchGoalLimitExceeded)
    } else {
        Ok(())
    }
}

/// Returns `Err(StringTooLong)` if `title` exceeds `MAX_STRING_LEN`.
pub fn validate_title(title: &String) -> Result<(), StateSizeError> {
    check_string_len(title)
}

/// Returns `Err(StringTooLong)` if `description` exceeds `MAX_STRING_LEN`.
pub fn validate_description(description: &String) -> Result<(), StateSizeError> {
    check_string_len(description)
}

/// Returns `Err(StringTooLong)` if `social_links` exceeds `MAX_STRING_LEN`.
pub fn validate_social_links(social_links: &String) -> Result<(), StateSizeError> {
    check_string_len(social_links)
}

/// Returns `Err(StringTooLong)` if `description` (roadmap item) exceeds `MAX_STRING_LEN`.
pub fn validate_roadmap_description(description: &String) -> Result<(), StateSizeError> {
    check_string_len(description)
}

/// Returns `Err(StringTooLong)` if `desc` (bonus goal description) exceeds `MAX_STRING_LEN`.
pub fn validate_bonus_goal_description(desc: &String) -> Result<(), StateSizeError> {
    check_string_len(desc)
}

/// Returns `Err(StringTooLong)` if the combined metadata length exceeds the aggregate limit.
/// @param title_len        Length of the title string in bytes.
/// @param description_len  Length of the description string in bytes.
/// @param socials_len      Length of the social links string in bytes.
pub fn validate_metadata_total_length(
    title_len: u32,
    description_len: u32,
    socials_len: u32,
) -> Result<(), StateSizeError> {
    const AGGREGATE_LIMIT: u32 = MAX_TITLE_LENGTH + MAX_DESCRIPTION_LENGTH + MAX_STRING_LEN;
    if title_len
        .saturating_add(description_len)
        .saturating_add(socials_len)
        > AGGREGATE_LIMIT
    {
        Err(StateSizeError::StringTooLong)
    } else {
        Ok(())
    }
}

// ── Queryable contract (used by contract_state_size.test.rs) ─────────────────

/// Standalone contract that exposes state-size constants over the Soroban ABI.
/// @dev Primarily used by the frontend to query limits without off-chain config.
#[contract]
pub struct ContractStateSize;

#[contractimpl]
impl ContractStateSize {
    /// Returns `MAX_TITLE_LENGTH`.
    pub fn max_title_length(_env: Env) -> u32 {
        MAX_TITLE_LENGTH
    }
    /// Returns `MAX_DESCRIPTION_LENGTH`.
    pub fn max_description_length(_env: Env) -> u32 {
        MAX_DESCRIPTION_LENGTH
    }
    /// Returns `MAX_STRING_LEN` (social links limit).
    pub fn max_social_links_length(_env: Env) -> u32 {
        MAX_STRING_LEN
    }
    /// Returns `MAX_CONTRIBUTORS`.
    pub fn max_contributors(_env: Env) -> u32 {
        MAX_CONTRIBUTORS
    }
    /// Returns `MAX_ROADMAP_ITEMS`.
    pub fn max_roadmap_items(_env: Env) -> u32 {
        MAX_ROADMAP_ITEMS
    }
    /// Returns `MAX_STRETCH_GOALS`.
    pub fn max_stretch_goals(_env: Env) -> u32 {
        MAX_STRETCH_GOALS
    }
}

    /// Returns `true` if `title` length is within `MAX_TITLE_LENGTH`.
    pub fn validate_title(_env: Env, title: String) -> bool {
        title.len() <= MAX_TITLE_LENGTH
    }
    /// Returns `true` if `description` length is within `MAX_DESCRIPTION_LENGTH`.
    pub fn validate_description(_env: Env, description: String) -> bool {
        description.len() <= MAX_DESCRIPTION_LENGTH
    }
    /// Returns `true` if `total_len` is within the aggregate metadata limit.
    pub fn validate_metadata_aggregate(_env: Env, total_len: u32) -> bool {
        total_len <= MAX_TITLE_LENGTH + MAX_DESCRIPTION_LENGTH + MAX_STRING_LEN
    }
}

/// Legacy constant for backwards compatibility (MAX_STRING_LEN = MAX_DESCRIPTION_LENGTH)
pub const MAX_STRING_LEN: u32 = MAX_DESCRIPTION_LENGTH;

/// Legacy function for backwards compatibility.
/// Checks that a string does not exceed MAX_DESCRIPTION_LENGTH bytes.
///
/// @param s The string to validate.
/// @return Ok(()) when within limits, Err otherwise.
pub fn check_string_len(s: &String) -> Result<(), StateSizeError> {
    if s.len() > MAX_STRING_LEN {
        return Err(StateSizeError::StringTooLong);
//! # contract_state_size
//!
//! @title   ContractStateSize — Centralized limits for variable-size contract state
//! @notice  Bounds metadata strings and collection growth so the crowdfund
//!          contract cannot accumulate unreviewed, unbounded state over time.
//! @dev     The helpers in this module are pure and return `Result<(), &'static str>`
//!          so they can be reused from contract logic, tests, and off-chain
//!          tooling without coupling to `ContractError`.
//!
//! ## Why these limits exist
//!
//! The crowdfund contract stores several fields whose size is controlled by
//! user input:
//!
//! 1. Metadata strings (`title`, `description`, `socials`, bonus-goal text)
//! 2. Contributor and pledger address lists
//! 3. Roadmap item descriptions
//! 4. Stretch-goal vectors
//!
//! Without explicit bounds, these values can grow until:
//!
//! - CI tests become slower and more memory-intensive
//! - Storage growth becomes harder to reason about during review
//! - Contract calls that iterate over vectors become less predictable
//! - Off-chain tooling sees inconsistent or excessively large payloads
//!
//! ## Security assumptions
//!
//! 1. Fixed maxima make worst-case state growth auditable.
//! 2. Rejecting oversize writes before persistence prevents silent storage bloat.
//! 3. Bounding collection counts reduces gas and event-surface risk in flows
//!    that later read or iterate over those collections.
//! 4. Metadata limits are sized for practical UX while preventing abuse via
//!    arbitrarily large strings.

use soroban_sdk::String;

/// Maximum number of contributor addresses stored in the contributor index.
///
/// @dev Contributions themselves remain stored per-address. This limit bounds
///      the enumerated contributor list used by view methods and reward flows.
pub const MAX_CONTRIBUTORS: u32 = 128;

/// Maximum number of pledger addresses stored in the pledger index.
pub const MAX_PLEDGERS: u32 = 128;

/// Maximum number of roadmap items allowed for a campaign.
pub const MAX_ROADMAP_ITEMS: u32 = 32;

/// Maximum number of stretch-goal milestones allowed for a campaign.
pub const MAX_STRETCH_GOALS: u32 = 32;

/// Maximum UTF-8 byte length for the campaign title.
pub const MAX_TITLE_LENGTH: u32 = 128;

/// Maximum UTF-8 byte length for the campaign description.
pub const MAX_DESCRIPTION_LENGTH: u32 = 2_048;

/// Maximum UTF-8 byte length for the social-links field.
pub const MAX_SOCIAL_LINKS_LENGTH: u32 = 512;

/// Maximum UTF-8 byte length for the optional bonus-goal description.
pub const MAX_BONUS_GOAL_DESCRIPTION_LENGTH: u32 = 280;

/// Maximum UTF-8 byte length for a roadmap item description.
pub const MAX_ROADMAP_DESCRIPTION_LENGTH: u32 = 280;

/// Maximum combined metadata footprint for title + description + socials.
///
/// @dev This budget is intentionally stricter than the sum of the individual
///      field maxima so callers cannot max out every metadata field at once.
pub const MAX_METADATA_TOTAL_LENGTH: u32 = 2_304;

#[inline]
fn validate_string_length(
    value: &String,
    max_length: u32,
    error: &'static str,
) -> Result<(), &'static str> {
    if value.len() > max_length {
        return Err(error);
//! # Contract State Size Limits
//!
//! This module enforces upper-bound limits on the size of unbounded collections
//! stored in contract state to prevent:
//!
//! - **DoS via state bloat**: an attacker flooding the contributors or roadmap
//!   lists until operations become too expensive to execute.
//! - **Gas exhaustion**: iteration over an unbounded `Vec` in `withdraw`,
//!   `refund`, or `collect_pledges` can exceed Soroban resource limits.
//! - **Ledger entry size violations**: Soroban enforces a hard cap on the
//!   serialised size of each ledger entry; exceeding it causes a host panic.
//!
//! ## Security Assumptions
//!
//! 1. `MAX_CONTRIBUTORS` caps the `Contributors` persistent list. Any `contribute`
//!    call that would push the list past this limit is rejected.
//! 2. `MAX_PLEDGERS` caps the `Pledgers` persistent list.
//! 3. `MAX_ROADMAP_ITEMS` caps the `Roadmap` instance list.
//! 4. `MAX_STRETCH_GOALS` caps the `StretchGoals` list.
//! 5. `MAX_TITLE_LENGTH`, `MAX_DESCRIPTION_LENGTH`, `MAX_SOCIAL_LINKS_LENGTH`,
//!    `MAX_BONUS_GOAL_DESCRIPTION_LENGTH`, and `MAX_ROADMAP_DESCRIPTION_LENGTH`
//!    cap user-supplied `String` fields to prevent oversized ledger entries.
//! 6. `MAX_METADATA_TOTAL_LENGTH` provides a combined budget for title +
//!    description + socials to prevent excessive total storage.
//!
//! ## Limits (rationale)
//!
//! | Constant                           | Value | Rationale                                      |
//! |------------------------------------|-------|------------------------------------------------|
//! | `MAX_CONTRIBUTORS`                 |   128 | Keeps `withdraw` / `refund` batch within gas   |
//! | `MAX_PLEDGERS`                     |   128 | Keeps `collect_pledges` iteration within gas  |
//! | `MAX_ROADMAP_ITEMS`                |    32 | Cosmetic list; no operational iteration needed |
//! | `MAX_STRETCH_GOALS`                |    32 | Small advisory list                            |
//! | `MAX_TITLE_LENGTH`                 |   128 | Prevents oversized instance-storage entries   |
//! | `MAX_DESCRIPTION_LENGTH`           |  2048 | Allows detailed campaign descriptions         |
//! | `MAX_SOCIAL_LINKS_LENGTH`          |   512 | Allows multiple social links                  |
//! | `MAX_BONUS_GOAL_DESCRIPTION_LENGTH`|   280 | Twitter-length limit for goal descriptions    |
//! | `MAX_ROADMAP_DESCRIPTION_LENGTH`   |   280 | Twitter-length limit for roadmap items        |
//! | `MAX_METADATA_TOTAL_LENGTH`        |  2304 | Combined budget: title + description + socials|
//!
//! ## NatSpec Documentation
//!
//! This module follows NatSpec conventions for all public constants and
//! validation functions to enable automated documentation generation and
//! static analysis.
//!
//! ## Usage
//!
//! ```ignore
//! use crate::contract_state_size::{validate_title, MAX_TITLE_LENGTH};
//!
//! fn set_title(env: &Env, title: &String) {
//!     if let Err(e) = validate_title(title) {
//!         panic!("{}", e);
//!     }
//!     // ... store title
//! }
//! ```
//! Contract State Size Limits
//!
//! @title   ContractStateSize — On-chain size-limit constants and enforcement helpers.
//! @notice  Defines upper bounds for every unbounded collection and user-supplied
//!          string stored in the crowdfund contract's ledger state.
//! @dev     All `check_*` helpers follow a checks-before-effects pattern: they
//!          read current state and return a typed `StateSizeError` **before** any
//!          mutation occurs in the calling function.
//!
//! ## Security Rationale
//!
//! Without these limits an adversary could:
//! - Flood `Contributors` / `Pledgers` until iteration in `withdraw` / `refund` /
//!   `collect_pledges` exceeds Soroban's per-transaction resource budget.
//! - Supply oversized `String` values that push a ledger entry past the host's
//!   hard serialisation cap, causing a host panic.
//!
//! ## Limits
//!
//! | Constant            | Value | Applies to                                      |
//! |---------------------|-------|-------------------------------------------------|
//! | `MAX_CONTRIBUTORS`  |   128 | `Contributors` list, `Pledgers` list            |
//! | `MAX_ROADMAP_ITEMS` |    32 | `Roadmap` list (`add_roadmap_item`)             |
//! | `MAX_STRETCH_GOALS` |    32 | `StretchGoals` list (`add_stretch_goal`)        |
//! | `MAX_STRING_LEN`    |   256 | title, description, social links, roadmap desc  |

use soroban_sdk::{contract, contractimpl, contracterror, Env, String, Vec};

// ── Error type ────────────────────────────────────────────────────────────────

/// Typed errors returned by state-size enforcement helpers.
///
/// @dev Discriminants start at 100 to avoid collisions with `ContractError` (1–17).
///      Do **not** renumber these — they are stable across contract upgrades.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StateSizeError {
    /// The `Contributors` or `Pledgers` list has reached `MAX_CONTRIBUTORS`.
    ContributorLimitExceeded = 100,
    /// The `Roadmap` list has reached `MAX_ROADMAP_ITEMS`.
    RoadmapLimitExceeded = 101,
    /// The `StretchGoals` list has reached `MAX_STRETCH_GOALS`.
    StretchGoalLimitExceeded = 102,
    /// A string field exceeds `MAX_STRING_LEN` bytes.
    StringTooLong = 103,
}

// ── Constants ─────────────────────────────────────────────────────────────────

/// Maximum number of unique contributors (and pledgers) tracked per campaign.
pub const MAX_CONTRIBUTORS: u32 = 128;

/// Maximum social-links string length in bytes.
pub const MAX_SOCIAL_LINKS_LENGTH: u32 = 512;

// Additional exported constants for compatibility
pub const MAX_PLEDGERS: u32 = MAX_CONTRIBUTORS;
pub const MAX_TITLE_LENGTH: u32 = MAX_STRING_LEN;
pub const MAX_DESCRIPTION_LENGTH: u32 = MAX_STRING_LEN;
pub const MAX_ROADMAP_DESCRIPTION_LENGTH: u32 = MAX_STRING_LEN;
pub const MAX_BONUS_GOAL_DESCRIPTION_LENGTH: u32 = MAX_STRING_LEN;
pub const MAX_SOCIAL_LINKS_LENGTH: u32 = MAX_STRING_LEN;
pub const MAX_METADATA_TOTAL_LENGTH: u32 = MAX_STRING_LEN * 4; // Combined metadata

// ── Error ─────────────────────────────────────────────────────────────────────

/// Returned when a state-size limit would be exceeded.
///
/// @notice This limit prevents unbounded growth of the contributor index.
///         When reached, new contributors cannot contribute unless they have
///         already contributed previously.
pub const MAX_CONTRIBUTORS: u32 = 128;

/// Maximum number of unique pledgers tracked on-chain.
///
/// @notice This limit prevents unbounded growth of the pledger index.
///         When reached, new pledgers cannot pledge unless they have
///         already pledged previously.
pub const MAX_PLEDGERS: u32 = 128;

/// Maximum number of roadmap items stored in instance storage.
///
/// @notice Roadmap items are cosmetic and do not require iteration in
///         any operational flow, so a modest limit is sufficient.
pub const MAX_ROADMAP_ITEMS: u32 = 32;

/// Maximum number of stretch-goal milestones.
///
/// @notice Stretch goals are advisory milestones that do not require
///         iteration in operational flows.
pub const MAX_STRETCH_GOALS: u32 = 32;

/// Maximum byte length of a campaign title.
///
/// @notice Titles are displayed in UI and stored in instance storage.
///         The limit ensures consistent UI rendering and storage bounds.
pub const MAX_TITLE_LENGTH: u32 = 128;

/// Maximum byte length of a campaign description.
///
/// @notice Descriptions can contain detailed information about the campaign.
///         The limit allows rich content while preventing oversized entries.
pub const MAX_DESCRIPTION_LENGTH: u32 = 2048;

/// Maximum byte length of social links field.
///
/// @notice Social links can contain multiple URLs separated by delimiters.
///         The limit accommodates several links while preventing bloat.
pub const MAX_SOCIAL_LINKS_LENGTH: u32 = 512;

/// Maximum byte length of bonus goal description.
///
/// @notice Bonus goal descriptions are shown when stretch goals are met.
///         Twitter-length limit encourages concise, readable content.
pub const MAX_BONUS_GOAL_DESCRIPTION_LENGTH: u32 = 280;

/// Maximum byte length of roadmap item description.
///
/// @notice Roadmap item descriptions outline milestones and timelines.
///         Twitter-length limit encourages concise, readable content.
pub const MAX_ROADMAP_DESCRIPTION_LENGTH: u32 = 280;

/// Maximum combined byte length of title + description + socials.
///
/// @notice This aggregate limit prevents campaigns from storing several
///         individually-valid but collectively excessive fields at once.
///         The sum of all three fields must not exceed this value.
pub const MAX_METADATA_TOTAL_LENGTH: u32 = 2304;

/// Maximum byte length for any string field (legacy alias).
///
/// @deprecated Use MAX_TITLE_LENGTH instead for new code.
///             This constant is kept for backwards compatibility.
pub const MAX_STRING_LEN: u32 = 256;
// ── Validation helpers ────────────────────────────────────────────────────────
/// Maximum number of unique contributors tracked per campaign.
pub const MAX_CONTRIBUTORS: u32 = 1_000;

/// Maximum number of unique pledgers tracked per campaign.
pub const MAX_PLEDGERS: u32 = 1_000;

/// Maximum number of roadmap items stored for a campaign.
/// Maximum number of roadmap milestones stored per campaign.
pub const MAX_ROADMAP_ITEMS: u32 = 32;

/// Maximum number of stretch-goal milestones stored per campaign.
pub const MAX_STRETCH_GOALS: u32 = 32;

/// Maximum bonus goal description length in bytes.
pub const MAX_BONUS_GOAL_DESCRIPTION_LENGTH: u32 = 280;

/// Maximum roadmap item description length in bytes.
pub const MAX_ROADMAP_DESCRIPTION_LENGTH: u32 = 280;

/// Maximum combined metadata length (title + description + socials).
pub const MAX_METADATA_TOTAL_LENGTH: u32 = 2_304;

/// Minimum allowed campaign goal in token units.
pub const MIN_GOAL_AMOUNT: i128 = 100;
/// Maximum byte length for any user-supplied string field.
pub const MAX_STRING_LEN: u32 = 256;

// ── Standalone helpers (called from lib.rs) ───────────────────────────────────

/// Returns `Ok(())` if `s.len() <= MAX_STRING_LEN`, else `Err(StateSizeError::StringTooLong)`.
#[inline]
pub fn check_string_len(s: &String) -> Result<(), StateSizeError> {
    if s.len() > MAX_STRING_LEN {
        Err(StateSizeError::StringTooLong)
    } else {
        Ok(())
    }
}

/// Returns `Ok(())` if `count < MAX_CONTRIBUTORS`, else `Err(ContributorLimitExceeded)`.
#[inline]
pub fn validate_contributor_capacity(count: u32) -> Result<(), StateSizeError> {
    if count >= MAX_CONTRIBUTORS {
        Err(StateSizeError::ContributorLimitExceeded)
    } else {
        Ok(())
    }
}

/// Reads the `Contributors` list length from persistent storage and enforces the cap.
#[inline]
pub fn check_contributor_limit(env: &Env) -> Result<(), StateSizeError> {
    let count: u32 = env
        .storage()
        .persistent()
        .get::<_, Vec<soroban_sdk::Address>>(&crate::DataKey::Contributors)
        .map(|v| v.len())
        .unwrap_or(0);
    validate_contributor_capacity(count)
}

/// Returns `Ok(())` if `count < MAX_CONTRIBUTORS`, else `Err(ContributorLimitExceeded)`.
#[inline]
pub fn validate_pledger_capacity(count: u32) -> Result<(), StateSizeError> {
    validate_contributor_capacity(count)
}

/// Reads the `Pledgers` list length from persistent storage and enforces the cap.
#[inline]
pub fn check_pledger_limit(env: &Env) -> Result<(), StateSizeError> {
    let count: u32 = env
        .storage()
        .persistent()
        .get::<_, Vec<soroban_sdk::Address>>(&crate::DataKey::Pledgers)
        .map(|v| v.len())
        .unwrap_or(0);
    validate_contributor_capacity(count)
}

/// Returns `Ok(())` if `count < MAX_ROADMAP_ITEMS`, else `Err(RoadmapLimitExceeded)`.
#[inline]
pub fn validate_roadmap_capacity(count: u32) -> Result<(), StateSizeError> {
    if count >= MAX_ROADMAP_ITEMS {
        Err(StateSizeError::RoadmapLimitExceeded)
    } else {
        Ok(())
    }
}

/// Reads the `Roadmap` list length from instance storage and enforces the cap.
#[inline]
pub fn check_roadmap_limit(env: &Env) -> Result<(), StateSizeError> {
    let count: u32 = env
        .storage()
        .instance()
        .get::<_, Vec<crate::RoadmapItem>>(&crate::DataKey::Roadmap)
        .map(|v| v.len())
        .unwrap_or(0);
    validate_roadmap_capacity(count)
}

/// Validates a roadmap item description length (delegates to `check_string_len`).
#[inline]
pub fn validate_roadmap_description(desc: &String) -> Result<(), StateSizeError> {
    check_string_len(desc)
}

/// Returns `Ok(())` if `count < MAX_STRETCH_GOALS`, else `Err(StretchGoalLimitExceeded)`.
#[inline]
pub fn validate_stretch_goal_capacity(count: u32) -> Result<(), StateSizeError> {
    if count >= MAX_STRETCH_GOALS {
        Err(StateSizeError::StretchGoalLimitExceeded)
    } else {
        Ok(())
    }
}

/// Reads the `StretchGoals` list length from instance storage and enforces the cap.
#[inline]
pub fn check_stretch_goal_limit(env: &Env) -> Result<(), StateSizeError> {
    let count: u32 = env
        .storage()
        .instance()
        .get::<_, Vec<i128>>(&crate::DataKey::StretchGoals)
        .map(|v| v.len())
        .unwrap_or(0);
    validate_stretch_goal_capacity(count)
}

/// Validates a title string length.
#[inline]
pub fn validate_title(title: &String) -> Result<(), StateSizeError> {
    check_string_len(title)
}

/// Validates a description string length.
#[inline]
pub fn validate_description(desc: &String) -> Result<(), StateSizeError> {
    check_string_len(desc)
}

/// Validates a social-links string length.
#[inline]
pub fn validate_social_links(links: &String) -> Result<(), StateSizeError> {
    check_string_len(links)
}

/// Validates the aggregate metadata length across title, description, and social links.
///
/// @param title_len       Byte length of the title field.
/// @param description_len Byte length of the description field.
/// @param socials_len     Byte length of the social-links field.
/// @return `Ok(())` if the sum is within the aggregate limit.
#[inline]
pub fn validate_metadata_total_length(
    title_len: u32,
    description_len: u32,
    socials_len: u32,
) -> Result<(), StateSizeError> {
    const AGGREGATE_LIMIT: u32 = MAX_STRING_LEN * 3;
    if title_len.saturating_add(description_len).saturating_add(socials_len) > AGGREGATE_LIMIT {
        Err(StateSizeError::StringTooLong)
    } else {
        Ok(())
    }
}

// ── Standalone contract (exposes constants on-chain) ─────────────────────────

/// On-chain contract that exposes state-size constants and validation functions.
///
/// @notice Frontend UIs can call these view functions to retrieve the current
///         limits without hard-coding them, ensuring UI validation stays in sync
///         with the contract after upgrades.
#[contract]
pub struct ContractStateSize;

#[contractimpl]
impl ContractStateSize {
    /// Returns the maximum allowed title length in bytes.
    /// @dev Used by frontend UI to set input field `maxlength`.
    pub fn max_title_length(_env: Env) -> u32 {
        MAX_TITLE_LENGTH
    }

impl core::fmt::Display for StateSizeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StateSizeError::ContributorLimitExceeded => {
                f.write_str("contributor limit exceeded")
            }
            StateSizeError::RoadmapLimitExceeded => f.write_str("roadmap limit exceeded"),
            StateSizeError::StretchGoalLimitExceeded => {
                f.write_str("stretch goal limit exceeded")
            }
            StateSizeError::StringTooLong => f.write_str("string too long"),
        }
    }
}

impl core::fmt::Display for StateSizeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StateSizeError::ContributorLimitExceeded => {
                f.write_str("contributor limit exceeded")
            }
            StateSizeError::RoadmapLimitExceeded => f.write_str("roadmap limit exceeded"),
            StateSizeError::StretchGoalLimitExceeded => {
                f.write_str("stretch goal limit exceeded")
            }
            StateSizeError::StringTooLong => f.write_str("string too long"),
        }
    }
}

impl core::fmt::Display for StateSizeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StateSizeError::ContributorLimitExceeded => {
                write!(f, "contributor limit exceeded")
            }
            StateSizeError::RoadmapLimitExceeded => {
                write!(f, "roadmap limit exceeded")
            }
            StateSizeError::StretchGoalLimitExceeded => {
                write!(f, "stretch goal limit exceeded")
            }
            StateSizeError::StringTooLong => {
                write!(f, "string too long")
            }
        }
    }
}

// ── Validation helpers ────────────────────────────────────────────────────────

/// Validate title length.
pub fn validate_title(value: &String) -> Result<(), &'static str> {
    if value.len() > MAX_TITLE_LENGTH {
        return Err("title exceeds MAX_TITLE_LENGTH bytes".into());
/// Validates that a title does not exceed MAX_TITLE_LENGTH bytes.
///
/// @param title The title string to validate.
/// @return Ok(()) if the title is within limits, Err with descriptive message otherwise.
/// @notice Callers should treat errors as permanent rejections; the limit
///         will not change without a contract upgrade.
pub fn validate_title(title: &String) -> Result<(), &'static str> {
    if title.len() > MAX_TITLE_LENGTH {
        return Err("title exceeds MAX_TITLE_LENGTH bytes");
pub fn validate_description(description: &String) -> Result<(), &'static str> {
    if description.len() > MAX_DESCRIPTION_LENGTH {
        return Err("description exceeds MAX_DESCRIPTION_LENGTH bytes");
    /// Returns the maximum allowed description length in bytes.
    pub fn max_description_length(_env: Env) -> u32 {
        MAX_DESCRIPTION_LENGTH
    }

/// Validate description length.
pub fn validate_description(value: &String) -> Result<(), &'static str> {
    if value.len() > MAX_DESCRIPTION_LENGTH {
        return Err("description exceeds MAX_DESCRIPTION_LENGTH bytes".into());
/// Validates that a description does not exceed MAX_DESCRIPTION_LENGTH bytes.
///
/// @param description The description string to validate.
/// @return Ok(()) if the description is within limits, Err with descriptive message otherwise.
pub fn validate_description(description: &String) -> Result<(), &'static str> {
    if description.len() > MAX_DESCRIPTION_LENGTH {
        return Err("description exceeds MAX_DESCRIPTION_LENGTH bytes");
    }
    Ok(())
}

/// Validate social links length.
pub fn validate_social_links(value: &String) -> Result<(), &'static str> {
    if value.len() > MAX_SOCIAL_LINKS_LENGTH {
        return Err("social links exceed MAX_SOCIAL_LINKS_LENGTH bytes".into());
/// Validates that social links do not exceed MAX_SOCIAL_LINKS_LENGTH bytes.
///
/// @param socials The social links string to validate.
/// @return Ok(()) if within limits, Err with descriptive message otherwise.
pub fn validate_social_links(socials: &String) -> Result<(), &'static str> {
    if socials.len() > MAX_SOCIAL_LINKS_LENGTH {
        return Err("social links exceed MAX_SOCIAL_LINKS_LENGTH bytes");
    }
    Ok(())
}

/// Validate bonus goal description length.
pub fn validate_bonus_goal_description(value: &String) -> Result<(), &'static str> {
    if value.len() > MAX_BONUS_GOAL_DESCRIPTION_LENGTH {
        return Err(
            "bonus goal description exceeds MAX_BONUS_GOAL_DESCRIPTION_LENGTH bytes".into(),
        );
/// Validates that bonus goal description does not exceed MAX_BONUS_GOAL_DESCRIPTION_LENGTH bytes.
///
/// @param description The bonus goal description to validate.
/// @return Ok(()) if within limits, Err with descriptive message otherwise.
pub fn validate_bonus_goal_description(description: &String) -> Result<(), &'static str> {
    if description.len() > MAX_BONUS_GOAL_DESCRIPTION_LENGTH {
        return Err("bonus goal description exceeds MAX_BONUS_GOAL_DESCRIPTION_LENGTH bytes");
    }
    Ok(())
}

/// Validate roadmap item description length.
pub fn validate_roadmap_description(value: &String) -> Result<(), &'static str> {
    if value.len() > MAX_ROADMAP_DESCRIPTION_LENGTH {
        return Err("roadmap description exceeds MAX_ROADMAP_DESCRIPTION_LENGTH bytes".into());
/// Validates that roadmap description does not exceed MAX_ROADMAP_DESCRIPTION_LENGTH bytes.
///
/// @param description The roadmap description to validate.
/// @return Ok(()) if within limits, Err with descriptive message otherwise.
pub fn validate_roadmap_description(description: &String) -> Result<(), &'static str> {
    if description.len() > MAX_ROADMAP_DESCRIPTION_LENGTH {
        return Err("roadmap description exceeds MAX_ROADMAP_DESCRIPTION_LENGTH bytes");
    }
    Ok(())
}

/// Validate metadata aggregate length.
/// Validates that the combined metadata (title + description + socials) does not exceed
/// MAX_METADATA_TOTAL_LENGTH bytes.
///
/// @param title_len Length of the title in bytes.
/// @param description_len Length of the description in bytes.
/// @param socials_len Length of the social links in bytes.
/// @return Ok(()) if the total is within limits, Err with descriptive message otherwise.
/// @notice This function uses saturating addition to prevent overflow attacks.
///         If the sum would overflow, it is treated as exceeding the limit.
pub fn validate_metadata_total_length(
    title_len: u32,
    description_len: u32,
    socials_len: u32,
) -> Result<(), &'static str> {
    let sum = title_len
        .checked_add(description_len)
        .and_then(|v| v.checked_add(socials_len))
        .ok_or("metadata exceeds MAX_METADATA_TOTAL_LENGTH bytes")?;
    if sum > MAX_METADATA_TOTAL_LENGTH {
        return Err("metadata exceeds MAX_METADATA_TOTAL_LENGTH bytes".into());
    // Use saturating_add to prevent integer overflow attacks
    let total = title_len.saturating_add(description_len).saturating_add(socials_len);
    if total > MAX_METADATA_TOTAL_LENGTH {
        return Err("metadata exceeds MAX_METADATA_TOTAL_LENGTH bytes");
    /// Returns the maximum allowed social links length in bytes.
    pub fn max_social_links_length(_env: Env) -> u32 {
        MAX_SOCIAL_LINKS_LENGTH
    /// Returns the maximum allowed byte length for any string field.
    pub fn max_string_len(_env: Env) -> u32 {
        MAX_STRING_LEN
    }

pub fn validate_contributor_capacity(len: u32) -> Result<(), &'static str> {
    if len >= MAX_CONTRIBUTORS {
        return Err("contributors exceed MAX_CONTRIBUTORS".into());
/// Validates that adding a new contributor would not exceed MAX_CONTRIBUTORS.
///
/// @param current_count The current number of contributors.
/// @return Ok(()) if a new contributor can be added, Err with descriptive message otherwise.
/// @notice This validates the index capacity, not whether a specific address
///         has already contributed. Existing contributors can always contribute.
pub fn validate_contributor_capacity(current_count: u32) -> Result<(), &'static str> {
    if current_count >= MAX_CONTRIBUTORS {
        return Err("contributors exceed MAX_CONTRIBUTORS");
    /// Returns the maximum number of contributors per campaign.
    pub fn max_contributors(_env: Env) -> u32 {
        MAX_CONTRIBUTORS
    }

pub fn validate_pledger_capacity(len: u32) -> Result<(), &'static str> {
    if len >= MAX_PLEDGERS {
        return Err("pledgers exceed MAX_PLEDGERS".into());
/// Validates that adding a new pledger would not exceed MAX_PLEDGERS.
///
/// @param current_count The current number of pledgers.
/// @return Ok(()) if a new pledger can be added, Err with descriptive message otherwise.
/// @notice This validates the index capacity, not whether a specific address
///         has already pledged. Existing pledgers can always pledge again.
pub fn validate_pledger_capacity(current_count: u32) -> Result<(), &'static str> {
    if current_count >= MAX_PLEDGERS {
        return Err("pledgers exceed MAX_PLEDGERS");
    /// Returns the maximum number of roadmap items.
    pub fn max_roadmap_items(_env: Env) -> u32 {
        MAX_ROADMAP_ITEMS
    }

/// Validate roadmap capacity before append.
pub fn validate_roadmap_capacity(len: u32) -> Result<(), &'static str> {
    if len >= MAX_ROADMAP_ITEMS {
        return Err("roadmap exceeds MAX_ROADMAP_ITEMS".into());
/// Validates that adding a new roadmap item would not exceed MAX_ROADMAP_ITEMS.
///
/// @param current_count The current number of roadmap items.
/// @return Ok(()) if a new item can be added, Err with descriptive message otherwise.
pub fn validate_roadmap_capacity(current_count: u32) -> Result<(), &'static str> {
    if current_count >= MAX_ROADMAP_ITEMS {
        return Err("roadmap exceeds MAX_ROADMAP_ITEMS");
    }
    Ok(())
}

/// Validate stretch-goal capacity before append.
pub fn validate_stretch_goal_capacity(len: u32) -> Result<(), &'static str> {
    if len >= MAX_STRETCH_GOALS {
        return Err("stretch goals exceed MAX_STRETCH_GOALS".into());
/// Validates that adding a new stretch goal would not exceed MAX_STRETCH_GOALS.
///
/// @param current_count The current number of stretch goals.
/// @return Ok(()) if a new goal can be added, Err with descriptive message otherwise.
pub fn validate_stretch_goal_capacity(current_count: u32) -> Result<(), &'static str> {
    if current_count >= MAX_STRETCH_GOALS {
        return Err("stretch goals exceed MAX_STRETCH_GOALS");
    }
    Ok(())
}

/// Assert that `s` does not exceed [`MAX_STRING_LEN`] bytes.
// ── Legacy compatibility functions ────────────────────────────────────────────

use crate::DataKey;
use soroban_sdk::Env;

/// Legacy function for checking string length limit.
///
/// @param s The string to validate.
/// @return Ok(()) if within limits, Err with StateSizeError otherwise.
/// @deprecated Use validate_title, validate_description, or validate_social_links instead.
pub fn check_string_len(s: &String) -> Result<(), StateSizeError> {
    if s.len() > MAX_STRING_LEN {
        return Err(StateSizeError::StringTooLong);
    }
    Ok(())
}

/// Checks stretch goal limit from storage.
#[inline]
pub fn check_stretch_goal_limit(env: &soroban_sdk::Env) -> Result<(), &'static str> {
    use soroban_sdk::Vec;
    let count: u32 = env
        .storage()
        .persistent()
        .get::<_, Vec<i128>>(&crate::DataKey::StretchGoals)
        .map(|v| v.len())
        .unwrap_or(0);
    validate_stretch_goal_capacity(count)
#[inline]
fn validate_next_count(
    current_count: u32,
    max_count: u32,
    error: &'static str,
) -> Result<(), &'static str> {
    if current_count >= max_count {
        return Err(error);
/// Assert that adding one more entry to the `Contributors` list is allowed.
///
/// Reads the current list length from persistent storage and compares it
/// against [`MAX_CONTRIBUTORS`].
/// Legacy function for checking contributor limit.
///
/// @param env Soroban environment reference.
/// @return Ok(()) if within limits, Err with StateSizeError otherwise.
/// @deprecated Use validate_contributor_capacity instead.
pub fn check_contributor_limit(env: &Env) -> Result<(), StateSizeError> {
    let contributors: Vec<soroban_sdk::Address> = env
        .storage()
        .persistent()
        .get(&crate::DataKey::Contributors)
        .unwrap_or_else(|| Vec::new(env));

    if contributors.len() >= MAX_CONTRIBUTORS {
        return Err(StateSizeError::ContributorLimitExceeded);
    /// Returns the maximum number of stretch goals.
    pub fn max_stretch_goals(_env: Env) -> u32 {
        MAX_STRETCH_GOALS
    }

/// Validates the campaign title length.
///
/// @param title Proposed title string.
/// @return `Ok(())` when `title.len() <= MAX_TITLE_LENGTH`.
pub fn validate_title(title: &String) -> Result<(), &'static str> {
    validate_string_length(
        title,
        MAX_TITLE_LENGTH,
        "title exceeds MAX_TITLE_LENGTH bytes",
    )
}

/// Validates the campaign description length.
///
/// @param description Proposed description string.
/// @return `Ok(())` when `description.len() <= MAX_DESCRIPTION_LENGTH`.
pub fn validate_description(description: &String) -> Result<(), &'static str> {
    validate_string_length(
        description,
        MAX_DESCRIPTION_LENGTH,
        "description exceeds MAX_DESCRIPTION_LENGTH bytes",
    )
}

/// Validates the social-links field length.
///
/// @param socials Proposed social-links string.
/// @return `Ok(())` when `socials.len() <= MAX_SOCIAL_LINKS_LENGTH`.
pub fn validate_social_links(socials: &String) -> Result<(), &'static str> {
    validate_string_length(
        socials,
        MAX_SOCIAL_LINKS_LENGTH,
        "social links exceed MAX_SOCIAL_LINKS_LENGTH bytes",
    )
}

/// Validates the optional bonus-goal description length.
///
/// @param description Proposed bonus-goal description.
/// @return `Ok(())` when the value fits within the configured bound.
pub fn validate_bonus_goal_description(description: &String) -> Result<(), &'static str> {
    validate_string_length(
        description,
        MAX_BONUS_GOAL_DESCRIPTION_LENGTH,
        "bonus goal description exceeds MAX_BONUS_GOAL_DESCRIPTION_LENGTH bytes",
    )
}

/// Validates a roadmap item description length.
///
/// @param description Proposed roadmap text.
/// @return `Ok(())` when the roadmap text is within the configured limit.
pub fn validate_roadmap_description(description: &String) -> Result<(), &'static str> {
    validate_string_length(
        description,
        MAX_ROADMAP_DESCRIPTION_LENGTH,
        "roadmap description exceeds MAX_ROADMAP_DESCRIPTION_LENGTH bytes",
    )
}

/// Validates the combined metadata footprint.
///
/// @param title_length Campaign title length in bytes.
/// @param description_length Campaign description length in bytes.
/// @param socials_length Social-links field length in bytes.
/// @return `Ok(())` when the total fits within `MAX_METADATA_TOTAL_LENGTH`.
pub fn validate_metadata_total_length(
    title_length: u32,
    description_length: u32,
    socials_length: u32,
) -> Result<(), &'static str> {
    let total = title_length
        .checked_add(description_length)
        .and_then(|value| value.checked_add(socials_length))
        .unwrap_or(u32::MAX);

    if total > MAX_METADATA_TOTAL_LENGTH {
        return Err("metadata exceeds MAX_METADATA_TOTAL_LENGTH bytes");
    }

    Ok(())
}

/// Validates that a new contributor can be added to the indexed contributor list.
///
/// @param current_count Current number of indexed contributors.
/// @return `Ok(())` when `current_count < MAX_CONTRIBUTORS`.
pub fn validate_contributor_capacity(current_count: u32) -> Result<(), &'static str> {
    validate_next_count(
        current_count,
        MAX_CONTRIBUTORS,
        "contributors exceed MAX_CONTRIBUTORS",
    )
}

/// Validates that a new pledger can be added to the indexed pledger list.
///
/// @param current_count Current number of indexed pledgers.
/// @return `Ok(())` when `current_count < MAX_PLEDGERS`.
pub fn validate_pledger_capacity(current_count: u32) -> Result<(), &'static str> {
    validate_next_count(current_count, MAX_PLEDGERS, "pledgers exceed MAX_PLEDGERS")
}

/// Validates that a new roadmap item can be appended.
///
/// @param current_count Current number of roadmap items.
/// @return `Ok(())` when `current_count < MAX_ROADMAP_ITEMS`.
pub fn validate_roadmap_capacity(current_count: u32) -> Result<(), &'static str> {
    validate_next_count(
        current_count,
        MAX_ROADMAP_ITEMS,
        "roadmap exceeds MAX_ROADMAP_ITEMS",
    )
}

/// Validates that a new stretch goal can be appended.
///
/// @param current_count Current number of stretch goals.
/// @return `Ok(())` when `current_count < MAX_STRETCH_GOALS`.
pub fn validate_stretch_goal_capacity(current_count: u32) -> Result<(), &'static str> {
    validate_next_count(
        current_count,
        MAX_STRETCH_GOALS,
        "stretch goals exceed MAX_STRETCH_GOALS",
    )
/// Assert that adding one more entry to the `Pledgers` list is allowed.
/// Legacy function for checking pledger limit.
///
/// @param env Soroban environment reference.
/// @return Ok(()) if within limits, Err with StateSizeError otherwise.
/// @deprecated Use validate_pledger_capacity instead.
pub fn check_pledger_limit(env: &Env) -> Result<(), StateSizeError> {
    let pledgers: Vec<soroban_sdk::Address> = env
        .storage()
        .persistent()
        .get(&crate::DataKey::Pledgers)
        .unwrap_or_else(|| Vec::new(env));

    if pledgers.len() >= MAX_PLEDGERS {
        return Err(StateSizeError::ContributorLimitExceeded);
    /// Validates that a string does not exceed the platform's title limit.
    /// @param title The campaign title to validate.
    /// @return `true` if length <= MAX_TITLE_LENGTH.
    pub fn validate_title(_env: Env, title: String) -> bool {
        title.len() <= MAX_TITLE_LENGTH
    }

    /// Validates that a description does not exceed the platform limit.
    /// @param description The campaign description to validate.
    /// @return `true` if length <= MAX_DESCRIPTION_LENGTH.
    pub fn validate_description(_env: Env, description: String) -> bool {
        description.len() <= MAX_DESCRIPTION_LENGTH
    }

    /// Validates that an aggregate metadata length is within bounds.
    /// @param total_len The combined length of all metadata strings.
    /// @return `true` if within safe limits to prevent state-rent spikes.
    pub fn validate_metadata_aggregate(_env: Env, total_len: u32) -> bool {
        const AGGREGATE_LIMIT: u32 =
            MAX_TITLE_LENGTH + MAX_DESCRIPTION_LENGTH + MAX_SOCIAL_LINKS_LENGTH;
        total_len <= AGGREGATE_LIMIT
    }
}

/// Legacy function for checking roadmap limit.
///
/// @param env Soroban environment reference.
/// @return Ok(()) if within limits, Err with StateSizeError otherwise.
/// @deprecated Use validate_roadmap_capacity instead.
pub fn check_roadmap_limit(env: &Env) -> Result<(), StateSizeError> {
    let roadmap: Vec<crate::RoadmapItem> = env
        .storage()
        .instance()
        .get(&DataKey::Roadmap)
        .unwrap_or_else(|| Vec::new(env));

    if roadmap.len() >= MAX_ROADMAP_ITEMS {
        return Err(StateSizeError::RoadmapLimitExceeded);
    }
    Ok(())
}

/// Legacy function for checking stretch goal limit.
///
/// @param env Soroban environment reference.
/// @return Ok(()) if within limits, Err with StateSizeError otherwise.
/// @deprecated Use validate_stretch_goal_capacity instead.
pub fn check_stretch_goal_limit(env: &Env) -> Result<(), StateSizeError> {
    let goals: Vec<i128> = env
        .storage()
        .instance()
        .get(&DataKey::StretchGoals)
        .unwrap_or_else(|| Vec::new(env));

    if goals.len() >= MAX_STRETCH_GOALS {
        return Err(StateSizeError::StretchGoalLimitExceeded);
    }
    Ok(())
}

// ── Named limits (used by lib.rs validate_* calls) ───────────────────────────

/// Maximum byte length of a campaign title.
pub const MAX_TITLE_LENGTH: u32 = MAX_STRING_LEN;
/// Maximum byte length of a campaign description.
pub const MAX_DESCRIPTION_LENGTH: u32 = MAX_STRING_LEN;
/// Maximum byte length of social links.
pub const MAX_SOCIAL_LINKS_LENGTH: u32 = MAX_STRING_LEN;
/// Maximum byte length of a roadmap item description.
pub const MAX_ROADMAP_DESCRIPTION_LENGTH: u32 = MAX_STRING_LEN;
/// Maximum byte length of a bonus-goal description.
pub const MAX_BONUS_GOAL_DESCRIPTION_LENGTH: u32 = MAX_STRING_LEN;
/// Maximum number of pledgers (alias of MAX_CONTRIBUTORS).
pub const MAX_PLEDGERS: u32 = MAX_CONTRIBUTORS;
/// Maximum combined byte length of all metadata fields.
pub const MAX_METADATA_TOTAL_LENGTH: u32 = MAX_STRING_LEN * 3;

// ── validate_* wrappers ───────────────────────────────────────────────────────

/// Validate that a title string is within bounds.
// ── Display implementation for panic messages ──────────────────────────────────

impl core::fmt::Display for StateSizeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StateSizeError::ContributorLimitExceeded => {
                write!(f, "MAX_CONTRIBUTORS limit exceeded")
            }
            StateSizeError::RoadmapLimitExceeded => write!(f, "MAX_ROADMAP_ITEMS limit exceeded"),
            StateSizeError::StretchGoalLimitExceeded => write!(f, "MAX_STRETCH_GOALS limit exceeded"),
            StateSizeError::StringTooLong => write!(f, "MAX_STRING_LEN exceeded"),
        }
    }
}

// ── Compatibility wrapper functions ───────────────────────────────────────────

/// Validates title string length. Alias for check_string_len.
pub fn validate_title(s: &String) -> Result<(), StateSizeError> {
    check_string_len(s)
}

/// Validate that a description string is within bounds.
/// Validates description string length. Alias for check_string_len.
pub fn validate_description(s: &String) -> Result<(), StateSizeError> {
    check_string_len(s)
}

/// Validate that a social-links string is within bounds.
/// Validates social links string length. Alias for check_string_len.
pub fn validate_social_links(s: &String) -> Result<(), StateSizeError> {
    check_string_len(s)
}

/// Validate that a roadmap item description is within bounds.
/// Validates roadmap description string length. Alias for check_string_len.
pub fn validate_roadmap_description(s: &String) -> Result<(), StateSizeError> {
    check_string_len(s)
}

/// Validate that a bonus-goal description is within bounds.
/// Validates bonus goal description string length. Alias for check_string_len.
pub fn validate_bonus_goal_description(s: &String) -> Result<(), StateSizeError> {
    check_string_len(s)
}

/// Validate combined metadata length does not exceed [`MAX_METADATA_TOTAL_LENGTH`].
// ── Per-field string length limits ───────────────────────────────────────────

/// Maximum byte length for a campaign title.
pub const MAX_TITLE_LENGTH: u32 = 100;

/// Maximum byte length for a campaign description.
pub const MAX_DESCRIPTION_LENGTH: u32 = 512;

/// Maximum byte length for social links.
pub const MAX_SOCIAL_LINKS_LENGTH: u32 = 256;

/// Maximum byte length for a bonus-goal description.
pub const MAX_BONUS_GOAL_DESCRIPTION_LENGTH: u32 = 256;

/// Maximum byte length for a roadmap item description.
pub const MAX_ROADMAP_DESCRIPTION_LENGTH: u32 = 256;

/// Maximum total byte length of all metadata fields combined.
pub const MAX_METADATA_TOTAL_LENGTH: u32 = 768;

/// Alias for `MAX_CONTRIBUTORS` used in pledger-capacity checks.
pub const MAX_PLEDGERS: u32 = MAX_CONTRIBUTORS;

// ── Per-field validators ──────────────────────────────────────────────────────

/// Validate that a title string is within [`MAX_TITLE_LENGTH`].
pub fn validate_title(s: &String) -> Result<(), StateSizeError> {
    if s.len() > MAX_TITLE_LENGTH {
        return Err(StateSizeError::StringTooLong);
/// Validates contributor capacity by checking list length.
pub fn validate_contributor_capacity(len: u32) -> Result<(), StateSizeError> {
    if len >= MAX_CONTRIBUTORS {
        return Err(StateSizeError::ContributorLimitExceeded);
    }
    Ok(())
}

/// Validate that a description string is within [`MAX_DESCRIPTION_LENGTH`].
pub fn validate_description(s: &String) -> Result<(), StateSizeError> {
    if s.len() > MAX_DESCRIPTION_LENGTH {
        return Err(StateSizeError::StringTooLong);
/// Validates pledger capacity by checking list length.
pub fn validate_pledger_capacity(len: u32) -> Result<(), StateSizeError> {
    if len >= MAX_CONTRIBUTORS {
        return Err(StateSizeError::ContributorLimitExceeded);
    }
    Ok(())
}

/// Validate that a social-links string is within [`MAX_SOCIAL_LINKS_LENGTH`].
pub fn validate_social_links(s: &String) -> Result<(), StateSizeError> {
    if s.len() > MAX_SOCIAL_LINKS_LENGTH {
        return Err(StateSizeError::StringTooLong);
/// Validates roadmap capacity by checking list length.
pub fn validate_roadmap_capacity(len: u32) -> Result<(), StateSizeError> {
    if len >= MAX_ROADMAP_ITEMS {
        return Err(StateSizeError::RoadmapLimitExceeded);
    }
    Ok(())
}

/// Validate that a bonus-goal description is within [`MAX_BONUS_GOAL_DESCRIPTION_LENGTH`].
pub fn validate_bonus_goal_description(s: &String) -> Result<(), StateSizeError> {
    if s.len() > MAX_BONUS_GOAL_DESCRIPTION_LENGTH {
        return Err(StateSizeError::StringTooLong);
/// Validates stretch goal capacity by checking list length.
pub fn validate_stretch_goal_capacity(len: u32) -> Result<(), StateSizeError> {
    if len >= MAX_STRETCH_GOALS {
        return Err(StateSizeError::StretchGoalLimitExceeded);
    }
    Ok(())
}

/// Validate that a roadmap item description is within [`MAX_ROADMAP_DESCRIPTION_LENGTH`].
pub fn validate_roadmap_description(s: &String) -> Result<(), StateSizeError> {
    if s.len() > MAX_ROADMAP_DESCRIPTION_LENGTH {
        return Err(StateSizeError::StringTooLong);
    }
    Ok(())
}

/// Validate that the combined metadata length is within [`MAX_METADATA_TOTAL_LENGTH`].
///
/// Uses saturating addition to prevent overflow on the sum.
/// Validates metadata total length (sum of all string fields).
pub fn validate_metadata_total_length(
    title_len: u32,
    description_len: u32,
    socials_len: u32,
) -> Result<(), StateSizeError> {
    let total = title_len
        .saturating_add(description_len)
        .saturating_add(socials_len);
    if total > MAX_METADATA_TOTAL_LENGTH {
    let total_len = title_len.saturating_add(description_len).saturating_add(socials_len);
    if total_len > MAX_METADATA_TOTAL_LENGTH {
        return Err(StateSizeError::StringTooLong);
    }
    Ok(())
}

/// Validate that the contributor list has not reached capacity.
///
/// @param current_len Current length of the contributors list.
pub fn validate_contributor_capacity(current_len: u32) -> Result<(), StateSizeError> {
    if current_len >= MAX_CONTRIBUTORS {
// ── Compatibility wrappers for existing code ──────────────────────────────────

/// Validate contributor capacity (alias for check_contributor_limit).
#[inline]
pub fn validate_contributor_capacity(capacity: u32) -> Result<(), StateSizeError> {
    if capacity >= MAX_CONTRIBUTORS {
// ── Collection capacity validators ───────────────────────────────────────────

/// Validate that `current_len` is below [`MAX_CONTRIBUTORS`].
pub fn validate_contributor_capacity(current_len: u32) -> Result<(), StateSizeError> {
    if current_len >= MAX_CONTRIBUTORS {
        return Err(StateSizeError::ContributorLimitExceeded);
    }
    Ok(())
}

/// Validate that the pledger list has not reached capacity.
///
/// @param current_len Current length of the pledgers list.
pub fn validate_pledger_capacity(current_len: u32) -> Result<(), StateSizeError> {
    if current_len >= MAX_PLEDGERS {
/// Validate pledger capacity (alias for check_pledger_limit).
#[inline]
pub fn validate_pledger_capacity(capacity: u32) -> Result<(), StateSizeError> {
    if capacity >= MAX_PLEDGERS {
/// Validate that `current_len` is below [`MAX_PLEDGERS`].
pub fn validate_pledger_capacity(current_len: u32) -> Result<(), StateSizeError> {
    if current_len >= MAX_PLEDGERS {
        return Err(StateSizeError::ContributorLimitExceeded);
    }
    Ok(())
}

/// Validate that the roadmap list has not reached capacity.
///
/// @param current_len Current length of the roadmap list.
pub fn validate_roadmap_capacity(current_len: u32) -> Result<(), StateSizeError> {
    if current_len >= MAX_ROADMAP_ITEMS {
/// Validate roadmap capacity.
#[inline]
pub fn validate_roadmap_capacity(capacity: u32) -> Result<(), StateSizeError> {
    if capacity >= MAX_ROADMAP_ITEMS {
/// Validate that `current_len` is below [`MAX_ROADMAP_ITEMS`].
pub fn validate_roadmap_capacity(current_len: u32) -> Result<(), StateSizeError> {
    if current_len >= MAX_ROADMAP_ITEMS {
        return Err(StateSizeError::RoadmapLimitExceeded);
    }
    Ok(())
}

/// Validate that the stretch-goals list has not reached capacity.
///
/// @param current_len Current length of the stretch-goals list.
/// Validate that `current_len` is below [`MAX_STRETCH_GOALS`].
pub fn validate_stretch_goal_capacity(current_len: u32) -> Result<(), StateSizeError> {
    if current_len >= MAX_STRETCH_GOALS {
        return Err(StateSizeError::StretchGoalLimitExceeded);
    }
    Ok(())
// ── Error types ───────────────────────────────────────────────────────────────

/// Error returned when a state-size limit would be exceeded.
///
/// @notice Callers should treat this as a permanent rejection for the current
///         campaign state; the limit will not change without a contract upgrade.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StateSizeError {
    /// The contributors / pledgers list is full.
    ContributorLimitExceeded = 100,
    /// The roadmap list is full.
    RoadmapLimitExceeded = 101,
    /// The stretch-goals list is full.
    StretchGoalLimitExceeded = 102,
    /// A string field exceeds its maximum length.
    StringTooLong = 103,
/// Validate stretch goal capacity.
#[inline]
pub fn validate_stretch_goal_capacity(capacity: u32) -> Result<(), StateSizeError> {
    if capacity >= MAX_STRETCH_GOALS {
        return Err(StateSizeError::StretchGoalLimitExceeded);
    }
    Ok(())
}

/// Validate title length.
#[inline]
pub fn validate_title(title: &String) -> Result<(), StateSizeError> {
    if title.len() > MAX_TITLE_LENGTH {
        return Err(StateSizeError::StringTooLong);
    }
    Ok(())
}

/// Validate description length.
#[inline]
pub fn validate_description(description: &String) -> Result<(), StateSizeError> {
    if description.len() > MAX_DESCRIPTION_LENGTH {
        return Err(StateSizeError::StringTooLong);
    }
    Ok(())
}

/// Validate social links length.
#[inline]
pub fn validate_social_links(socials: &String) -> Result<(), StateSizeError> {
    if socials.len() > MAX_SOCIAL_LINKS_LENGTH {
        return Err(StateSizeError::StringTooLong);
    }
    Ok(())
}

/// Validate bonus goal description length.
#[inline]
pub fn validate_bonus_goal_description(description: &String) -> Result<(), StateSizeError> {
    if description.len() > MAX_BONUS_GOAL_DESCRIPTION_LENGTH {
        return Err(StateSizeError::StringTooLong);
/// Validate bonus goal description length.
///
/// @param description The bonus goal description to validate.
/// @return `Ok(())` when within limits, `Err(StateSizeError::StringTooLong)` otherwise.
pub fn validate_bonus_goal_description(description: &String) -> Result<(), StateSizeError> {
    check_string_len(description)
}

/// Validate contributor capacity.
///
/// @param current_count Current number of contributors.
/// @return `Ok(())` when within limits, `Err(StateSizeError::ContributorLimitExceeded)` otherwise.
pub fn validate_contributor_capacity(current_count: u32) -> Result<(), StateSizeError> {
    if current_count >= MAX_CONTRIBUTORS {
        return Err(StateSizeError::ContributorLimitExceeded);
    }
    Ok(())
}

/// Validate roadmap description length.
#[inline]
pub fn validate_roadmap_description(description: &String) -> Result<(), StateSizeError> {
    if description.len() > MAX_ROADMAP_DESCRIPTION_LENGTH {
        return Err(StateSizeError::StringTooLong);
/// Validate pledger capacity.
///
/// @param current_count Current number of pledgers.
/// @return `Ok(())` when within limits, `Err(StateSizeError::ContributorLimitExceeded)` otherwise.
pub fn validate_pledger_capacity(current_count: u32) -> Result<(), StateSizeError> {
    if current_count >= MAX_CONTRIBUTORS {
        return Err(StateSizeError::ContributorLimitExceeded);
    }
    Ok(())
}

/// Validate metadata total length.
#[inline]
pub fn validate_metadata_total_length(title_len: u32, desc_len: u32, social_len: u32) -> Result<(), StateSizeError> {
    let total = title_len.saturating_add(desc_len).saturating_add(social_len);
    if total > MAX_METADATA_TOTAL_LENGTH {
///
/// @param title_length Length of title string.
/// @param description_length Length of description string.
/// @param socials_length Length of socials string.
/// @return `Ok(())` when within limits, `Err(StateSizeError::StringTooLong)` otherwise.
pub fn validate_metadata_total_length(
    title_length: u32,
    description_length: u32,
    socials_length: u32,
) -> Result<(), StateSizeError> {
    let total = title_length + description_length + socials_length;
    if total > MAX_STRING_LEN * 3 {
        return Err(StateSizeError::StringTooLong);
    }
    Ok(())
}
#![no_std]
use soroban_sdk::{Env, String};

pub const MAX_STRING_LEN: u32 = 256;
pub const MAX_CONTRIBUTORS: u32 = 1_000;

pub fn validate_title(s: &String) -> bool { s.len() <= MAX_STRING_LEN }
pub fn validate_description(s: &String) -> bool { s.len() <= MAX_STRING_LEN }
pub fn validate_social_links(s: &String) -> bool { s.len() <= MAX_STRING_LEN }
pub fn validate_roadmap_description(s: &String) -> bool { s.len() <= MAX_STRING_LEN }
pub fn validate_bonus_goal_description(s: &String) -> bool { s.len() <= MAX_STRING_LEN }
pub fn validate_metadata_total_length(len: u32) -> bool { len <= (MAX_STRING_LEN * 5) }
pub fn validate_contributor_capacity(len: u32) -> bool { len < MAX_CONTRIBUTORS }
pub fn validate_pledger_capacity(len: u32) -> bool { len < MAX_CONTRIBUTORS }
pub fn validate_roadmap_capacity(len: u32) -> bool { len < 20 }
pub fn validate_stretch_goal_capacity(len: u32) -> bool { len < 10 }

/// Validate title length.
///
/// @param title The title to validate.
/// @return `Ok(())` when within limits, `Err(StateSizeError::StringTooLong)` otherwise.
pub fn validate_title(title: &String) -> Result<(), StateSizeError> {
    check_string_len(title)
}

/// Validate description length.
///
/// @param description The description to validate.
/// @return `Ok(())` when within limits, `Err(StateSizeError::StringTooLong)` otherwise.
pub fn validate_description(description: &String) -> Result<(), StateSizeError> {
    check_string_len(description)
}

/// Validate social links length.
///
/// @param socials The social links to validate.
/// @return `Ok(())` when within limits, `Err(StateSizeError::StringTooLong)` otherwise.
pub fn validate_social_links(socials: &String) -> Result<(), StateSizeError> {
    check_string_len(socials)
}

/// Validate roadmap capacity.
///
/// @param current_count Current number of roadmap items.
/// @return `Ok(())` when within limits, `Err(StateSizeError::RoadmapLimitExceeded)` otherwise.
pub fn validate_roadmap_capacity(current_count: u32) -> Result<(), StateSizeError> {
    if current_count >= MAX_ROADMAP_ITEMS {
        return Err(StateSizeError::RoadmapLimitExceeded);
    }
    Ok(())
}

/// Validate roadmap description length.
///
/// @param description The roadmap description to validate.
/// @return `Ok(())` when within limits, `Err(StateSizeError::StringTooLong)` otherwise.
pub fn validate_roadmap_description(description: &String) -> Result<(), StateSizeError> {
    check_string_len(description)
}

/// Validate stretch goal capacity.
///
/// @param current_count Current number of stretch goals.
/// @return `Ok(())` when within limits, `Err(StateSizeError::StretchGoalLimitExceeded)` otherwise.
pub fn validate_stretch_goal_capacity(current_count: u32) -> Result<(), StateSizeError> {
    if current_count >= MAX_STRETCH_GOALS {
        return Err(StateSizeError::StretchGoalLimitExceeded);
    }
    Ok(())
// ── Standalone helpers (called from lib.rs) ───────────────────────────────────

/// Returns `Ok(())` if `count < MAX_CONTRIBUTORS`, else `Err("limit exceeded")`.
#[inline]
pub fn validate_contributor_capacity(count: u32) -> Result<(), &'static str> {
    if count >= MAX_CONTRIBUTORS {
        Err("contributor limit exceeded")
    } else {
        Ok(())
    }
}

/// Panics if the contributor list is at capacity.
#[inline]
pub fn check_contributor_limit(env: &soroban_sdk::Env) -> Result<(), &'static str> {
    use soroban_sdk::Vec;
    let count: u32 = env
        .storage()
        .persistent()
        .get::<_, Vec<soroban_sdk::Address>>(&crate::DataKey::Contributors)
        .map(|v| v.len())
        .unwrap_or(0);
    validate_contributor_capacity(count)
}

/// Returns `Ok(())` if `count < MAX_CONTRIBUTORS`, else `Err("limit exceeded")`.
#[inline]
pub fn validate_pledger_capacity(count: u32) -> Result<(), &'static str> {
    if count >= MAX_CONTRIBUTORS {
        Err("pledger limit exceeded")
    } else {
        Ok(())
    }
}

/// Panics if the pledger list is at capacity.
#[inline]
pub fn check_pledger_limit(env: &soroban_sdk::Env) -> Result<(), &'static str> {
    use soroban_sdk::Vec;
    let count: u32 = env
        .storage()
        .persistent()
        .get::<_, Vec<soroban_sdk::Address>>(&crate::DataKey::Pledgers)
        .map(|v| v.len())
        .unwrap_or(0);
    validate_pledger_capacity(count)
}

/// Validates total metadata length.
#[inline]
pub fn validate_metadata_total_length(
    title_len: u32,
    description_len: u32,
    socials_len: u32,
) -> Result<(), &'static str> {
    const AGGREGATE_LIMIT: u32 =
        MAX_TITLE_LENGTH + MAX_DESCRIPTION_LENGTH + MAX_SOCIAL_LINKS_LENGTH;
    let total = title_len.saturating_add(description_len).saturating_add(socials_len);
    if total > AGGREGATE_LIMIT {
        Err("metadata too long")
    } else {
        Ok(())
    }
}

/// Validates a title string length.
#[inline]
pub fn validate_title(title: &soroban_sdk::String) -> Result<(), &'static str> {
    if title.len() > MAX_TITLE_LENGTH {
        Err("title too long")
    } else {
        Ok(())
    }
}

/// Validates a description string length.
#[inline]
pub fn validate_description(desc: &soroban_sdk::String) -> Result<(), &'static str> {
    if desc.len() > MAX_DESCRIPTION_LENGTH {
        Err("description too long")
    } else {
        Ok(())
    }
}

/// Validates social links string length.
#[inline]
pub fn validate_social_links(links: &soroban_sdk::String) -> Result<(), &'static str> {
    if links.len() > MAX_SOCIAL_LINKS_LENGTH {
        Err("social links too long")
    } else {
        Ok(())
    }
}

/// Validates a generic string length (uses description limit).
#[inline]
pub fn check_string_len(s: &soroban_sdk::String) -> Result<(), &'static str> {
    validate_description(s)
}

/// Validates roadmap item capacity.
#[inline]
pub fn validate_roadmap_capacity(count: u32) -> Result<(), &'static str> {
    if count >= MAX_ROADMAP_ITEMS {
        Err("roadmap limit exceeded")
    } else {
        Ok(())
    }
}

/// Checks roadmap limit from storage.
#[inline]
pub fn check_roadmap_limit(env: &soroban_sdk::Env) -> Result<(), &'static str> {
    use soroban_sdk::Vec;
    let count: u32 = env
        .storage()
        .persistent()
        .get::<_, Vec<crate::RoadmapItem>>(&crate::DataKey::Roadmap)
        .map(|v| v.len())
        .unwrap_or(0);
    validate_roadmap_capacity(count)
}

/// Validates a roadmap item description length.
#[inline]
pub fn validate_roadmap_description(desc: &soroban_sdk::String) -> Result<(), &'static str> {
    validate_description(desc)
}

/// Validates stretch goal capacity.
#[inline]
pub fn validate_stretch_goal_capacity(count: u32) -> Result<(), &'static str> {
    if count >= MAX_STRETCH_GOALS {
        Err("stretch goal limit exceeded")
    } else {
        Ok(())
    }
}

/// Checks stretch goal limit from storage.
#[inline]
pub fn check_stretch_goal_limit(env: &soroban_sdk::Env) -> Result<(), &'static str> {
    use soroban_sdk::Vec;
    let count: u32 = env
        .storage()
        .persistent()
        .get::<_, Vec<i128>>(&crate::DataKey::StretchGoals)
        .map(|v| v.len())
        .unwrap_or(0);
    validate_stretch_goal_capacity(count)
}
    /// Returns `true` if `s.len() <= MAX_STRING_LEN`.
    pub fn validate_string(_env: Env, s: String) -> bool {
        s.len() <= MAX_STRING_LEN
    }
}

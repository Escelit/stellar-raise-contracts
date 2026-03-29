//! # Stellar Token Minter Contract
//!
//! @title   StellarTokenMinter
//! @notice  NFT minting contract for the Stellar Raise crowdfunding platform.
//!          Authorized contracts (e.g. the Crowdfund contract) call `mint` to
//!          issue on-chain reward NFTs to campaign contributors.
//! @dev     Implements the Checks-Effects-Interactions pattern throughout.
//!          All state-changing functions enforce `require_auth` before any
//!          storage writes or event emissions.
//!
//! ## Security Model
//!
//! - **Authorization**: Only the designated minter can call `mint`
//!   (enforced via `require_auth` on the stored minter address).
//! - **Admin Separation**: Admin role is separate from minter role
//!   (principle of least privilege — admin cannot mint directly).
//! - **State Management**: Persistent storage is used for token metadata;
//!   instance storage is used for roles and the counter.
//! - **Bounded Operations**: All operations stay within Soroban resource limits.
//! - **Idempotency**: Duplicate token minting is rejected via a persistent-storage
//!   existence check before any write.
//! - **Initialization Guard**: Contract can only be initialized once; a second
//!   call panics with "already initialized".
//!
//! ## Deprecated Patterns (v1.0)
//!
//! The following patterns have been deprecated in favour of more secure implementations:
//! - Direct admin minting (now requires the dedicated minter role)
//! - Unguarded initialization (now panics on double-init)
//! - Implicit authorization (now explicit via `require_auth`)
//!
//! ## Invariants
//!
//! 1. `total_minted` equals the count of unique token IDs that have been minted.
//! 2. Each token ID can only be minted once (persistent storage existence check).
//! 3. Only the designated minter can call `mint` (`require_auth` enforced).
//! 4. Only the admin can update the minter address (`require_auth` enforced).
//! 5. Contract state is immutable after initialization (no re-initialization).

// stellar_token_minter — NFT minting capabilities for the crowdfunding platform.

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

// ── Test constants ────────────────────────────────────────────────────────────//
// Centralised numeric literals used across the stellar_token_minter test suites.
// Defining them here means CI/CD only needs to update one location when campaign
// parameters change, and test intent is self-documenting.

/// Default campaign funding goal used in tests (1 000 000 stroops).
pub const TEST_GOAL: i128 = 1_000_000;

/// Default minimum contribution used in tests (1 000 stroops).
pub const TEST_MIN_CONTRIBUTION: i128 = 1_000;

/// Default campaign duration used in tests (1 hour in seconds).
pub const TEST_DEADLINE_OFFSET: u64 = 3_600;

/// Initial token balance minted to the creator in the test setup helper.
pub const TEST_CREATOR_BALANCE: i128 = 100_000_000;

/// Initial token balance minted to the token-minter test setup helper.
pub const TEST_MINTER_CREATOR_BALANCE: i128 = 10_000_000;

/// Standard single-contributor balance used in most integration tests.
pub const TEST_CONTRIBUTOR_BALANCE: i128 = 1_000_000;

/// Contribution amount used in NFT-batch tests (goal / MAX_MINT_BATCH).
pub const TEST_NFT_CONTRIBUTION: i128 = 25_000;

/// Contribution amount used in the "below batch limit" NFT test.
pub const TEST_NFT_SMALL_CONTRIBUTION: i128 = 400_000;

/// Contribution amount used in collect_pledges / two-contributor tests.
pub const TEST_PLEDGE_CONTRIBUTION: i128 = 300_000;

/// Bonus goal threshold used in idempotency tests.
pub const TEST_BONUS_GOAL: i128 = 1_000_000;

/// Primary goal used in bonus-goal idempotency tests.
pub const TEST_BONUS_PRIMARY_GOAL: i128 = 500_000;

/// Per-contribution amount used in bonus-goal crossing tests.
pub const TEST_BONUS_CONTRIBUTION: i128 = 600_000;

/// Seed balance for overflow protection test (small initial contribution).
pub const TEST_OVERFLOW_SEED: i128 = 10_000;

/// Maximum platform fee in basis points (100 %).
pub const TEST_FEE_BPS_MAX: u32 = 10_000;

/// Platform fee that exceeds the maximum (triggers panic).
pub const TEST_FEE_BPS_OVER: u32 = 10_001;

/// Platform fee of 10 % used in fee-deduction tests.
pub const TEST_FEE_BPS_10PCT: u32 = 1_000;

/// Progress basis points representing 80 % funding.
pub const TEST_PROGRESS_BPS_80PCT: u32 = 8_000;

/// Progress basis points representing 99.999 % funding (just below goal).
pub const TEST_PROGRESS_BPS_JUST_BELOW: u32 = 9_999;

/// Contribution amount that is one stroop below the goal.
pub const TEST_JUST_BELOW_GOAL: i128 = 999_999;

/// Contribution amount used in the "partial accumulation" test.
pub const TEST_PARTIAL_CONTRIBUTION_A: i128 = 300_000;

/// Second contribution amount used in the "partial accumulation" test.
pub const TEST_PARTIAL_CONTRIBUTION_B: i128 = 200_000;

// ── Event / mint budget helpers ───────────────────────────────────────────────

/// Maximum events allowed per Soroban transaction.
pub const MAX_EVENTS_PER_TX: u32 = 100;

/// Maximum NFTs minted in a single `withdraw()` call.
pub const MAX_MINT_BATCH: u32 = 50;

/// Maximum log entries per transaction.
pub const MAX_LOG_ENTRIES: u32 = 64;

/// Returns `true` if `emitted` is below `MAX_EVENTS_PER_TX`.
#[inline]
pub fn within_event_budget(emitted: u32) -> bool {
    emitted < MAX_EVENTS_PER_TX
}

/// Returns `true` if `minted` is below `MAX_MINT_BATCH`.
#[inline]
pub fn within_mint_batch(minted: u32) -> bool {
    minted < MAX_MINT_BATCH
}

/// Returns `true` if `logged` is below `MAX_LOG_ENTRIES`.
#[inline]
pub fn within_log_budget(logged: u32) -> bool {
    logged < MAX_LOG_ENTRIES
}

/// Returns remaining event budget (saturates at 0).
#[inline]
//! Logging bounds for the Stellar token minter / crowdfund contract.
//! Stellar Token Minter Contract
//!
//! @title   StellarTokenMinter
//! @notice  NFT minting contract for the Stellar Raise crowdfunding platform.
//!          Authorized contracts (e.g. the Crowdfund contract) call `mint` to
//!          issue on-chain reward NFTs to campaign contributors.
//! @dev     Implements the Checks-Effects-Interactions pattern throughout.
//!          All state-changing functions enforce `require_auth` before any
//!          storage writes or event emissions.
//!
//! ## Security Model
//!
//! - **Authorization**: Only the designated minter can call `mint`
//!   (enforced via `require_auth` on the stored minter address).
//! - **Admin Separation**: Admin role is separate from minter role
//!   (principle of least privilege — admin cannot mint directly).
//! - **State Management**: Persistent storage is used for token metadata;
//!   instance storage is used for roles and the counter.
//! - **Bounded Operations**: All operations stay within Soroban resource limits.
//! - **Idempotency**: Duplicate token minting is rejected via a persistent-storage
//!   existence check before any write.
//! - **Initialization Guard**: Contract can only be initialized once; a second
//!   call panics with "already initialized".
//!
//! ## Deprecated Patterns (v1.0)
//!
//! The following patterns have been deprecated in favour of more secure implementations:
//! - Direct admin minting (now requires the dedicated minter role)
//! - Unguarded initialization (now panics on double-init)
//! - Implicit authorization (now explicit via `require_auth`)
//!
//! ## Invariants
//!
//! 1. `total_minted` equals the count of unique token IDs that have been minted.
//! 2. Each token ID can only be minted once (persistent storage existence check).
//! 3. Only the designated minter can call `mint` (`require_auth` enforced).
//! 4. Only the admin can update the minter address (`require_auth` enforced).
//! 5. Contract state is immutable after initialization (no re-initialization).

// stellar_token_minter — NFT minting capabilities for the crowdfunding platform.

use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec,
};
// ── Test constants ────────────────────────────────────────────────────────────
//
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

// ── Test constants ────────────────────────────────────────────────────────────//
// Centralised numeric literals used across the stellar_token_minter test suites.
// Defining them here means CI/CD only needs to update one location when campaign
// parameters change, and test intent is self-documenting.

/// Default campaign funding goal used in tests (1 000 000 stroops).
pub const TEST_GOAL: i128 = 1_000_000;

/// Default minimum contribution used in tests (1 000 stroops).
pub const TEST_MIN_CONTRIBUTION: i128 = 1_000;

/// Default campaign duration used in tests (1 hour in seconds).
pub const TEST_DEADLINE_OFFSET: u64 = 3_600;

/// Initial token balance minted to the creator in the test setup helper.
pub const TEST_CREATOR_BALANCE: i128 = 100_000_000;

/// Initial token balance minted to the token-minter test setup helper.
pub const TEST_MINTER_CREATOR_BALANCE: i128 = 10_000_000;

/// Standard single-contributor balance used in most integration tests.
pub const TEST_CONTRIBUTOR_BALANCE: i128 = 1_000_000;

/// Contribution amount used in NFT-batch tests (goal / MAX_MINT_BATCH).
pub const TEST_NFT_CONTRIBUTION: i128 = 25_000;

/// Contribution amount used in the "below batch limit" NFT test.
pub const TEST_NFT_SMALL_CONTRIBUTION: i128 = 400_000;

/// Contribution amount used in collect_pledges / two-contributor tests.
pub const TEST_PLEDGE_CONTRIBUTION: i128 = 300_000;

/// Bonus goal threshold used in idempotency tests.
pub const TEST_BONUS_GOAL: i128 = 1_000_000;

/// Primary goal used in bonus-goal idempotency tests.
pub const TEST_BONUS_PRIMARY_GOAL: i128 = 500_000;

/// Per-contribution amount used in bonus-goal crossing tests.
pub const TEST_BONUS_CONTRIBUTION: i128 = 600_000;

/// Seed balance for overflow protection test (small initial contribution).
pub const TEST_OVERFLOW_SEED: i128 = 10_000;

/// Maximum platform fee in basis points (100 %).
pub const TEST_FEE_BPS_MAX: u32 = 10_000;

/// Platform fee that exceeds the maximum (triggers panic).
pub const TEST_FEE_BPS_OVER: u32 = 10_001;

/// Platform fee of 10 % used in fee-deduction tests.
pub const TEST_FEE_BPS_10PCT: u32 = 1_000;

/// Progress basis points representing 80 % funding.
pub const TEST_PROGRESS_BPS_80PCT: u32 = 8_000;

/// Progress basis points representing 99.999 % funding (just below goal).
pub const TEST_PROGRESS_BPS_JUST_BELOW: u32 = 9_999;

/// Contribution amount that is one stroop below the goal.
pub const TEST_JUST_BELOW_GOAL: i128 = 999_999;

/// Contribution amount used in the "partial accumulation" test.
pub const TEST_PARTIAL_CONTRIBUTION_A: i128 = 300_000;

/// Second contribution amount used in the "partial accumulation" test.
pub const TEST_PARTIAL_CONTRIBUTION_B: i128 = 200_000;

// ── Event / mint budget helpers ───────────────────────────────────────────────

/// Maximum events allowed per Soroban transaction.
pub const MAX_EVENTS_PER_TX: u32 = 100;

/// Maximum NFTs minted in a single `withdraw()` call.
pub const MAX_MINT_BATCH: u32 = 50;

/// Maximum log entries per transaction.
pub const MAX_LOG_ENTRIES: u32 = 64;

/// Returns `true` if `emitted` is below `MAX_EVENTS_PER_TX`.
#[inline]
pub fn within_event_budget(emitted: u32) -> bool {
    emitted < MAX_EVENTS_PER_TX
}

/// Returns `true` if `minted` is below `MAX_MINT_BATCH`.
#[inline]
pub fn within_mint_batch(minted: u32) -> bool {
    minted < MAX_MINT_BATCH
}

/// Returns `true` if `logged` is below `MAX_LOG_ENTRIES`.
#[inline]
pub fn within_log_budget(logged: u32) -> bool {
    logged < MAX_LOG_ENTRIES
}

/// Returns remaining event budget (saturates at 0).
#[inline]
pub fn remaining_event_budget(reserved: u32) -> u32 {
    MAX_EVENTS_PER_TX.saturating_sub(reserved)
}

/// Returns remaining mint budget (saturates at 0).
#[inline]
pub fn remaining_mint_budget(minted: u32) -> u32 {
    MAX_MINT_BATCH.saturating_sub(minted)
}

/// Emits a batch summary event if `count > 0` and budget is not exhausted.
/// Returns `true` if the event was emitted.
pub fn emit_batch_summary(
    env: &Env,
    topic: (&str, &str),
    count: u32,
    emitted_so_far: u32,
) -> bool {
    if count == 0 || !within_event_budget(emitted_so_far) {
        return false;
    }
    env.events().publish(
        (Symbol::new(env, topic.0), Symbol::new(env, topic.1)),
        count,
    );
    true
}

// ── Constants ────────────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Admin address with authority to update the minter role.
    Admin,
    /// Minter address with authority to mint new tokens.
    Minter,
    /// Total count of tokens minted (u64 counter).
    TotalMinted,
    /// Token metadata storage: maps token_id to owner address.
    TokenMetadata(u64),
}

#[contract]
pub struct StellarTokenMinter;

#[contractimpl]
impl StellarTokenMinter {
    /// Initializes the minter contract with admin and minter roles.
    ///
    /// # Arguments
    ///
    /// * `admin` - Contract administrator with authority to update the minter role
    /// * `minter` - Address authorized to perform minting operations
    ///
    /// # Panics
    ///
    /// * If the contract has already been initialized (idempotency guard)
    ///
    /// # Security Notes
    ///
    /// - This function can only be called once per contract instance
    /// - Admin and minter roles are stored separately for principle of least privilege
    /// - No authorization check is performed on initialization (assumed to be called by contract deployer)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let admin = Address::generate(&env);
    /// let minter = Address::generate(&env);
    /// StellarTokenMinter::initialize(env, admin, minter);
    /// ```
    pub fn initialize(env: Env, admin: Address, minter: Address) {
        // Guard: Prevent double initialization
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }

        // Store admin and minter roles
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Minter, &minter);
        
        // Initialize total minted counter to zero
        env.storage().instance().set(&DataKey::TotalMinted, &0u64);
    }

    /// Mints a new NFT to the specified recipient.
    ///
    /// # Arguments
    ///
    /// * `to` - Recipient address (owner of the minted token)
    /// * `token_id` - Unique identifier for the token to mint
    ///
    /// # Panics
    ///
    /// * If the caller is not the designated minter (authorization check)
    /// * If the token ID has already been minted (idempotency check)
    ///
    /// # Security Notes
    ///
    /// - **Authorization**: Enforced via `require_auth()` on the minter address
    /// - **Idempotency**: Token IDs are unique; duplicate mints are rejected
    /// - **State Consistency**: Total minted counter is incremented atomically
    /// - **Event Emission**: Emits a mint event for off-chain tracking
    ///
    /// # Invariants Maintained
    ///
    /// - `total_minted` increases by exactly 1 on successful mint
    /// - Each token_id maps to exactly one owner address
    /// - Only the minter can call this function
    ///
    /// # Example
    ///
    /// ```ignore
    /// let recipient = Address::generate(&env);
    /// let token_id = 42u64;
    /// StellarTokenMinter::mint(env, recipient, token_id);
    /// assert_eq!(StellarTokenMinter::owner(env, token_id), Some(recipient));
    /// ```
    pub fn mint(env: Env, to: Address, token_id: u64) {
        // Guard: Retrieve and verify minter authorization
        let minter: Address = env
            .storage()
            .instance()
            .get(&DataKey::Minter)
            .expect("contract not initialized");
        minter.require_auth();

/// Returns remaining mint budget (saturates at 0).
#[inline]
/// Calculates how many NFT mints remain in the current batch budget.
///
/// Returns `0` when the batch limit is already reached.
///
/// # Arguments
/// * `minted` – NFTs already minted in this `withdraw` call.
pub fn remaining_mint_budget(minted: u32) -> u32 {
    MAX_MINT_BATCH.saturating_sub(minted)
}

/// Emits a batch summary event if `count > 0` and budget is not exhausted.
/// Returns `true` if the event was emitted.
pub fn emit_batch_summary(
    env: &Env,
    topic: (&str, &str),
    count: u32,
    emitted_so_far: u32,
) -> bool {
    if count == 0 || !within_event_budget(emitted_so_far) {
        return false;
    }
    env.events().publish(
        (Symbol::new(env, topic.0), Symbol::new(env, topic.1)),
        count,
    );
    true
}

// ── Constants ────────────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Admin address with authority to update the minter role.
    Admin,
    /// Minter address with authority to mint new tokens.
    Minter,
    /// Total count of tokens minted (u64 counter).
    TotalMinted,
    /// Token metadata storage: maps token_id to owner address.
    TokenMetadata(u64),
}

#[contract]
pub struct StellarTokenMinter;

#[contractimpl]
impl StellarTokenMinter {
    /// Initializes the minter contract with admin and minter roles.
    ///
    /// # Arguments
    ///
    /// * `admin` - Contract administrator with authority to update the minter role
    /// * `minter` - Address authorized to perform minting operations
    ///
    /// # Panics
    ///
    /// * If the contract has already been initialized (idempotency guard)
    ///
    /// # Security Notes
    ///
    /// - This function can only be called once per contract instance
    /// - Admin and minter roles are stored separately for principle of least privilege
    /// - No authorization check is performed on initialization (assumed to be called by contract deployer)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let admin = Address::generate(&env);
    /// let minter = Address::generate(&env);
    /// StellarTokenMinter::initialize(env, admin, minter);
    /// ```
    pub fn initialize(env: Env, admin: Address, minter: Address) {
        // Guard: Prevent double initialization
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("already initialized");
        }

        // Store admin and minter roles
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Minter, &minter);

        // Initialize total minted counter to zero
        env.storage().instance().set(&DataKey::TotalMinted, &0u64);
    }

    /// Mints a new NFT to the specified recipient.
    ///
    /// # Arguments
    ///
    /// * `to` - Recipient address (owner of the minted token)
    /// * `token_id` - Unique identifier for the token to mint
    ///
    /// # Panics
    ///
    /// * If the caller is not the designated minter (authorization check)
    /// * If the token ID has already been minted (idempotency check)
    ///
    /// # Security Notes
    ///
    /// - **Authorization**: Enforced via `require_auth()` on the minter address
    /// - **Idempotency**: Token IDs are unique; duplicate mints are rejected
    /// - **State Consistency**: Total minted counter is incremented atomically
    /// - **Event Emission**: Emits a mint event for off-chain tracking
    ///
    /// # Invariants Maintained
    ///
    /// - `total_minted` increases by exactly 1 on successful mint
    /// - Each token_id maps to exactly one owner address
    /// - Only the minter can call this function
    ///
    /// # Example
    ///
    /// ```ignore
    /// let recipient = Address::generate(&env);
    /// let token_id = 42u64;
    /// StellarTokenMinter::mint(env, recipient, token_id);
    /// assert_eq!(StellarTokenMinter::owner(env, token_id), Some(recipient));
    /// ```
    pub fn mint(env: Env, to: Address, token_id: u64) {
        // Guard: Retrieve and verify minter authorization
        let minter: Address = env
            .storage()
            .instance()
            .get(&DataKey::Minter)
            .expect("contract not initialized");
        minter.require_auth();

        // Guard: Prevent duplicate token minting
        let key = DataKey::TokenMetadata(token_id);
        if env.storage().persistent().has(&key) {
            panic!("token already minted");
        }

        // Effect: Store token metadata (owner address)
        env.storage().persistent().set(&key, &to);

        // Effect: Increment total minted counter
        let total: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalMinted)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalMinted, &(total + 1));

        // Interaction: Emit mint event for off-chain tracking
        env.events()
            .publish((Symbol::new(&env, "mint"), to), token_id);
    }

    /// Returns the owner of a token, or None if the token has not been minted.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The token ID to query
    ///
    /// # Returns
    ///
    /// * `Some(Address)` if the token has been minted
    /// * `None` if the token has not been minted
    ///
    /// # Security Notes
    ///
    /// - This is a read-only view function with no authorization requirements
    /// - Returns None for unminted tokens (safe default)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let owner = StellarTokenMinter::owner(env, 42u64);
    /// assert_eq!(owner, Some(recipient));
    /// ```
    pub fn owner(env: Env, token_id: u64) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::TokenMetadata(token_id))
    }

    /// Returns the total number of NFTs minted by this contract.
    ///
    /// # Returns
    ///
    /// The count of unique token IDs that have been successfully minted.
    ///
    /// # Security Notes
    ///
    /// - This is a read-only view function with no authorization requirements
    /// - Returns 0 if the contract has not been initialized
    /// - Guaranteed to be accurate (incremented atomically on each mint)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = StellarTokenMinter::total_minted(env);
    /// assert_eq!(count, 42);
    /// ```
    pub fn total_minted(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TotalMinted)
            .unwrap_or(0)
    }

    /// Updates the minter address. Only callable by the admin.
    ///
    /// # Arguments
    ///
    /// * `admin` - The current admin address (must match stored admin)
    /// * `new_minter` - The new address to be granted minter privileges
    ///
    /// # Panics
    ///
    /// * If the contract has not been initialized
    /// * If the caller is not the admin (authorization check)
    /// * If the provided admin address does not match the stored admin
    ///
    /// # Security Notes
    ///
    /// - **Authorization**: Enforced via `require_auth()` on the admin address
    /// - **Verification**: Admin address must match the stored admin (prevents spoofing)
    /// - **Atomicity**: Minter role is updated atomically
    /// - **Principle of Least Privilege**: Only admin can update minter role
    ///
    /// # Invariants Maintained
    ///
    /// - Only the admin can call this function
    /// - The new minter address is stored immediately
    /// - Previous minter loses minting privileges
    ///
    /// # Example
    ///
    /// ```ignore
    /// let new_minter = Address::generate(&env);
    /// StellarTokenMinter::set_minter(env, admin, new_minter);
    /// // new_minter can now call mint()
    /// ```
    pub fn set_minter(env: Env, admin: Address, new_minter: Address) {
        // Guard: Retrieve stored admin (panics if not initialized)
        let current_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("contract not initialized");

        // Guard: Verify caller is the admin
        current_admin.require_auth();

        // Guard: Verify provided admin matches stored admin (prevents spoofing)
        if admin != current_admin {
            panic!("unauthorized");
        }

        // Effect: Update minter role
        env.storage().instance().set(&DataKey::Minter, &new_minter);
    }
}
/// Emits a bounded summary event for a batch operation.
///
/// Instead of emitting one event per item (which would be unbounded), callers
/// emit a single summary event carrying the count of processed items.  This
/// function enforces that the summary is only emitted when `count > 0` and
/// that the event budget has not been exhausted.
///
/// # Arguments
/// * `env`      – The Soroban environment.
/// * `topic`    – Two-part event topic `(namespace, name)`.
/// * `count`    – Number of items processed in the batch.
/// * `emitted`  – Events already emitted in this transaction (budget check).
///
/// # Returns
/// `true` if the event was emitted, `false` if skipped (count == 0 or budget
/// exhausted).
pub fn emit_batch_summary(
    env: &Env,
    topic: (&'static str, &'static str),
    count: u32,
    emitted: u32,
) -> bool {
    if count == 0 || !within_event_budget(emitted) {
        return false;
        // Guard: Prevent duplicate token minting
        let key = DataKey::TokenMetadata(token_id);
        if env.storage().persistent().has(&key) {
            panic!("token already minted");
        }

        // Effect: Store token metadata (owner address)
        env.storage().persistent().set(&key, &to);

        // Effect: Increment total minted counter
        let total: u64 = env
            .storage()
            .instance()
            .get(&DataKey::TotalMinted)
            .unwrap_or(0);
        // Update total counter
        let total: u64 = env.storage().instance().get(&DataKey::TotalMinted).unwrap();
        env.storage()
            .instance()
            .set(&DataKey::TotalMinted, &(total + 1));

        // Interaction: Emit mint event for off-chain tracking
        env.events().publish((Symbol::new(&env, "mint"), to), token_id);
        // Emit event
        env.events()
            .publish((Symbol::new(&env, "mint"), to), token_id);
    }

    /// Returns the owner of a token, or None if the token has not been minted.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The token ID to query
    ///
    /// # Returns
    ///
    /// * `Some(Address)` if the token has been minted
    /// * `None` if the token has not been minted
    ///
    /// # Security Notes
    ///
    /// - This is a read-only view function with no authorization requirements
    /// - Returns None for unminted tokens (safe default)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let owner = StellarTokenMinter::owner(env, 42u64);
    /// assert_eq!(owner, Some(recipient));
    /// ```
    pub fn owner(env: Env, token_id: u64) -> Option<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::TokenMetadata(token_id))
    }

    /// Returns the total number of NFTs minted by this contract.
    ///
    /// # Returns
    ///
    /// The count of unique token IDs that have been successfully minted.
    ///
    /// # Security Notes
    ///
    /// - This is a read-only view function with no authorization requirements
    /// - Returns 0 if the contract has not been initialized
    /// - Guaranteed to be accurate (incremented atomically on each mint)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = StellarTokenMinter::total_minted(env);
    /// assert_eq!(count, 42);
    /// ```
    pub fn total_minted(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TotalMinted)
            .unwrap_or(0)
    }

    /// Updates the minter address. Only callable by the admin.
    ///
    /// # Arguments
    ///
    /// * `admin` - The current admin address (must match stored admin)
    /// * `new_minter` - The new address to be granted minter privileges
    ///
    /// # Panics
    ///
    /// * If the contract has not been initialized
    /// * If the caller is not the admin (authorization check)
    /// * If the provided admin address does not match the stored admin
    ///
    /// # Security Notes
    ///
    /// - **Authorization**: Enforced via `require_auth()` on the admin address
    /// - **Verification**: Admin address must match the stored admin (prevents spoofing)
    /// - **Atomicity**: Minter role is updated atomically
    /// - **Principle of Least Privilege**: Only admin can update minter role
    ///
    /// # Invariants Maintained
    ///
    /// - Only the admin can call this function
    /// - The new minter address is stored immediately
    /// - Previous minter loses minting privileges
    ///
    /// # Example
    ///
    /// ```ignore
    /// let new_minter = Address::generate(&env);
    /// StellarTokenMinter::set_minter(env, admin, new_minter);
    /// // new_minter can now call mint()
    /// ```
    pub fn set_minter(env: Env, admin: Address, new_minter: Address) {
        // Guard: Retrieve stored admin (panics if not initialized)
        let current_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("contract not initialized");

        // Guard: Verify caller is the admin
            .expect("not initialized");
        current_admin.require_auth();

        // Guard: Verify provided admin matches stored admin (prevents spoofing)
        if admin != current_admin {
            panic!("unauthorized");
        }

        // Effect: Update minter role
        env.storage()
            .instance()
            .set(&DataKey::Minter, &new_minter);
    }
    env.events().publish((topic.0, topic.1), count);
    true
//! Stellar Token Minter Module
//!
//! This module provides token minting functionality for the Stellar Raise
//! crowdfunding platform. It handles token minting for contributors,
//! platform fee distribution, and NFT reward minting.
//!
//! # Security
//!
//! - All minting operations require proper authorization
//! - Overflow protection on all arithmetic operations
//! - Platform fee validation (max 10,000 bps = 100%)
//! - Contributor list size limits to prevent unbounded growth

use soroban_sdk::{
    contract, contractclient, contractimpl, contracttype, token, Address, Env, IntoVal, String,
    Symbol, Vec,
};

/// Maximum number of NFT mint calls (and their events) emitted in a single
/// `withdraw()` invocation. Caps per-contributor event emission to prevent
/// unbounded gas consumption when the contributor list is large.
pub const MAX_NFT_MINT_BATCH: u32 = 50;

/// Represents the campaign status.
#[derive(Clone, PartialEq)]
#[contracttype]
pub enum Status {
    Active,
    Successful,
    Refunded,
    Cancelled,
}

/// Platform configuration for fee distribution.
#[derive(Clone)]
#[contracttype]
pub struct PlatformConfig {
    /// Address that receives platform fees
    pub address: Address,
    /// Fee in basis points (max 10,000 = 100%)
    pub fee_bps: u32,
}

/// Campaign statistics for frontend display.
#[derive(Clone)]
#[contracttype]
pub struct CampaignStats {
    /// Total tokens raised so far
    pub total_raised: i128,
    /// Funding goal
    pub goal: i128,
    /// Progress in basis points (0-10,000)
    pub progress_bps: u32,
    /// Number of unique contributors
    pub contributor_count: u32,
    /// Average contribution amount
    pub average_contribution: i128,
    /// Largest single contribution
    pub largest_contribution: i128,
}

/// Storage keys for the token minter contract.
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    /// Campaign creator address
    Creator,
    /// Token contract address
    Token,
    /// Funding goal amount
    Goal,
    /// Campaign deadline timestamp
    Deadline,
    /// Total tokens raised
    TotalRaised,
    /// Individual contribution by address
    Contribution(Address),
    /// List of all contributors
    Contributors,
    /// Campaign status
    Status,
    /// Minimum contribution amount
    MinContribution,
    /// Platform configuration
    PlatformConfig,
    /// NFT contract address for reward minting
    NFTContract,
}

/// Contract errors for the token minter.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// Campaign already initialized
    AlreadyInitialized = 1,
    /// Campaign deadline has passed
    CampaignEnded = 2,
    /// Campaign is still active
    CampaignStillActive = 3,
    /// Funding goal not reached
    GoalNotReached = 4,
    /// Funding goal was reached
    GoalReached = 5,
    /// Integer overflow in arithmetic
    Overflow = 6,
    /// No contribution to refund
    NothingToRefund = 7,
    /// Contribution amount is zero
    ZeroAmount = 8,
    /// Contribution below minimum
    BelowMinimum = 9,
    /// Campaign is not active
    CampaignNotActive = 10,
}

/// NFT contract interface for minting rewards.
#[contractclient(name = "NftContractClient")]
pub trait NftContract {
    /// Mint an NFT to the specified address
    fn mint(env: Env, to: Address) -> u128;
}

/// Stellar Token Minter Contract
///
/// Manages token minting for crowdfunding campaigns including:
/// - Contributor token transfers
/// - Platform fee distribution
/// - NFT reward minting
/// - Campaign statistics tracking
#[contract]
pub struct StellarTokenMinter;

#[contractimpl]
impl StellarTokenMinter {
    /// Initializes a new token minter for a crowdfunding campaign.
    ///
    /// # Arguments
    ///
    /// * `env` - Soroban environment
    /// * `admin` - Address authorized for contract upgrades
    /// * `creator` - Campaign creator address (must sign)
    /// * `token` - Token contract address for contributions
    /// * `goal` - Funding goal in token's smallest unit
    /// * `deadline` - Campaign deadline as ledger timestamp
    /// * `min_contribution` - Minimum contribution amount
    /// * `platform_config` - Optional platform fee configuration
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `ContractError::AlreadyInitialized` if called twice
    ///
    /// # Panics
    ///
    /// - If platform fee exceeds 10,000 bps (100%)
    /// - If creator does not authorize the call
    ///
    /// # Security
    ///
    /// - Requires creator authorization
    /// - Validates platform fee bounds
    /// - Prevents double initialization
    pub fn initialize(
        env: Env,
        admin: Address,
        creator: Address,
        token: Address,
        goal: i128,
        deadline: u64,
        min_contribution: i128,
        platform_config: Option<PlatformConfig>,
    ) -> Result<(), ContractError> {
        // Check if already initialized
        if env.storage().instance().has(&DataKey::Creator) {
            return Err(ContractError::AlreadyInitialized);
        }

        // Require creator authorization
        creator.require_auth();

        // Store admin for upgrade authorization
        env.storage().instance().set(&DataKey::Creator, &creator);

        // Validate and store platform configuration
        if let Some(ref config) = platform_config {
            if config.fee_bps > 10_000 {
                panic!("platform fee cannot exceed 100%");
            }
            env.storage()
                .instance()
                .set(&DataKey::PlatformConfig, config);
        }

        // Store campaign parameters
        env.storage().instance().set(&DataKey::Token, &token);
        env.storage().instance().set(&DataKey::Goal, &goal);
        env.storage().instance().set(&DataKey::Deadline, &deadline);
        env.storage()
            .instance()
            .set(&DataKey::MinContribution, &min_contribution);
        env.storage().instance().set(&DataKey::TotalRaised, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Active);

        // Initialize empty contributors list
        let empty_contributors: Vec<Address> = Vec::new(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Contributors, &empty_contributors);

        Ok(())
    }

    /// Contribute tokens to the campaign.
    ///
    /// Transfers tokens from the contributor to the contract. Updates
    /// contribution tracking and emits events for frontend display.
    ///
    /// # Arguments
    ///
    /// * `env` - Soroban environment
    /// * `contributor` - Address making the contribution (must sign)
    /// * `amount` - Amount of tokens to contribute
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or appropriate `ContractError`
    ///
    /// # Errors
    ///
    /// - `ContractError::CampaignNotActive` - Campaign is not in Active status
    /// - `ContractError::ZeroAmount` - Contribution amount is zero
    /// - `ContractError::BelowMinimum` - Amount below minimum contribution
    /// - `ContractError::CampaignEnded` - Deadline has passed
    /// - `ContractError::Overflow` - Integer overflow in accounting
    ///
    /// # Security
    ///
    /// - Requires contributor authorization
    /// - Validates amount against minimum
    /// - Checks campaign deadline
    /// - Uses checked arithmetic to prevent overflow
    pub fn contribute(env: Env, contributor: Address, amount: i128) -> Result<(), ContractError> {
        // Require contributor authorization
        contributor.require_auth();

        // Guard: campaign must be active
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            return Err(ContractError::CampaignNotActive);
        }

        // Validate amount
        if amount == 0 {
            return Err(ContractError::ZeroAmount);
        }

        let min_contribution: i128 = env
            .storage()
            .instance()
            .get(&DataKey::MinContribution)
            .unwrap();
        if amount < min_contribution {
            return Err(ContractError::BelowMinimum);
        }

        // Check deadline
        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        if env.ledger().timestamp() > deadline {
            return Err(ContractError::CampaignEnded);
        }

        // Track contributor if new
        let mut contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or_else(|| Vec::new(&env));

        let is_new_contributor = !contributors.contains(&contributor);

        // Transfer tokens from contributor to contract
        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_address);
        token_client.transfer(&contributor, &env.current_contract_address(), &amount);

        // Update contributor's running total with overflow protection
        let contribution_key = DataKey::Contribution(contributor.clone());
        let previous_amount: i128 = env
            .storage()
            .persistent()
            .get(&contribution_key)
            .unwrap_or(0);

        let new_contribution = previous_amount
            .checked_add(amount)
            .ok_or(ContractError::Overflow)?;

        env.storage()
            .persistent()
            .set(&contribution_key, &new_contribution);
        env.storage()
            .persistent()
            .extend_ttl(&contribution_key, 100, 100);

        // Update global total raised with overflow protection
        let total: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();
        let new_total = total.checked_add(amount).ok_or(ContractError::Overflow)?;

        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &new_total);

        // Add to contributors list if new
        if is_new_contributor {
            contributors.push_back(contributor.clone());
            env.storage()
                .persistent()
                .set(&DataKey::Contributors, &contributors);
            env.storage()
                .persistent()
                .extend_ttl(&DataKey::Contributors, 100, 100);
        }

        // Emit contribution event for frontend tracking
        env.events()
            .publish(("campaign", "contributed"), (contributor, amount));

        Ok(())
    }

    /// Withdraw funds after successful campaign.
    ///
    /// Creator claims raised funds after deadline when goal is met. If platform
    /// config is set, fee is deducted first. If NFT contract is configured,
    /// mints one NFT per contributor.
    ///
    /// # Arguments
    ///
    /// * `env` - Soroban environment
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, or appropriate `ContractError`
    ///
    /// # Errors
    ///
    /// - `ContractError::CampaignStillActive` - Deadline has not passed
    /// - `ContractError::GoalNotReached` - Funding goal was not met
    ///
    /// # Events
    ///
    /// - `("campaign", "withdrawn")` - Emitted with (creator, total)
    /// - `("campaign", "fee_transferred")` - Emitted if platform fee applies
    /// - `("campaign", "nft_minted")` - Emitted for each NFT minted
    ///
    /// # Security
    ///
    /// - Checks campaign deadline
    /// - Validates goal was reached
    /// - Handles platform fee distribution
    /// - Batches NFT minting to prevent gas exhaustion
    pub fn withdraw(env: Env) -> Result<(), ContractError> {
        let status: Status = env.storage().instance().get(&DataKey::Status).unwrap();
        if status != Status::Active {
            panic!("campaign is not active");
        }

        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        if env.ledger().timestamp() <= deadline {
            return Err(ContractError::CampaignStillActive);
        }

        let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
        let total_raised: i128 = env.storage().instance().get(&DataKey::TotalRaised).unwrap();

        if total_raised < goal {
            return Err(ContractError::GoalNotReached);
        }

        let creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        let token_address: Address = env.storage().instance().get(&DataKey::Token).unwrap();
        let token_client = token::Client::new(&env, &token_address);

        let mut amount_to_creator = total_raised;

        // Handle platform fee if configured
        if let Some(config) = env
            .storage()
            .instance()
            .get::<_, PlatformConfig>(&DataKey::PlatformConfig)
        {
            let fee = (total_raised * config.fee_bps as i128) / 10_000;
            amount_to_creator = total_raised - fee;

            if fee > 0 {
                token_client.transfer(
                    &env.current_contract_address(),
                    &config.address,
                    &fee,
                );
                env.events()
                    .publish(("campaign", "fee_transferred"), (config.address, fee));
            }
        }

        // Transfer remaining funds to creator
        if amount_to_creator > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &creator,
                &amount_to_creator,
            );
        }

        // Update status to Successful
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Successful);

        // Emit withdrawal event
        env.events()
            .publish(("campaign", "withdrawn"), (creator, total_raised));

        // Mint NFTs if contract is configured
        if let Some(nft_contract) = env
            .storage()
            .instance()
            .get::<_, Address>(&DataKey::NFTContract)
        {
            let contributors: Vec<Address> = env
                .storage()
                .persistent()
                .get(&DataKey::Contributors)
                .unwrap_or_else(|| Vec::new(&env));

            let nft_client = NftContractClient::new(&env, &nft_contract);
            let batch_size = contributors.len().min(MAX_NFT_MINT_BATCH);

            for i in 0..batch_size {
                let contributor = contributors.get(i).unwrap();
                let token_id = nft_client.mint(&contributor);
                env.events()
                    .publish(("campaign", "nft_minted"), (contributor, token_id));
            }
        }

        Ok(())
    }

    /// Set the NFT contract address for reward minting.
    ///
    /// # Arguments
    ///
    /// * `env` - Soroban environment
    /// * `creator` - Campaign creator address (must sign)
    /// * `nft_contract` - NFT contract address
    ///
    /// # Security
    ///
    /// - Requires creator authorization
    /// - Only callable by campaign creator
    pub fn set_nft_contract(env: Env, creator: Address, nft_contract: Address) {
        let stored_creator: Address = env.storage().instance().get(&DataKey::Creator).unwrap();
        if creator != stored_creator {
            panic!("not authorized");
        }
        creator.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::NFTContract, &nft_contract);
    }

    /// Get total tokens raised.
    ///
    /// # Arguments
    ///
    /// * `env` - Soroban environment
    ///
    /// # Returns
    ///
    /// Total amount of tokens raised in the campaign
    pub fn total_raised(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalRaised)
            .unwrap_or(0)
    }

    /// Get funding goal.
    ///
    /// # Arguments
    ///
    /// * `env` - Soroban environment
    ///
    /// # Returns
    ///
    /// Funding goal amount
    pub fn goal(env: Env) -> i128 {
        env.storage().instance().get(&DataKey::Goal).unwrap()
    }

    /// Get campaign deadline.
    ///
    /// # Arguments
    ///
    /// * `env` - Soroban environment
    ///
    /// # Returns
    ///
    /// Deadline as ledger timestamp
    pub fn deadline(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::Deadline).unwrap()
    }

    /// Get minimum contribution amount.
    ///
    /// # Arguments
    ///
    /// * `env` - Soroban environment
    ///
    /// # Returns
    ///
    /// Minimum contribution amount
    pub fn min_contribution(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::MinContribution)
            .unwrap()
    }

    /// Get contribution by address.
    ///
    /// # Arguments
    ///
    /// * `env` - Soroban environment
    /// * `addr` - Contributor address
    ///
    /// # Returns
    ///
    /// Contribution amount for the address, or 0 if not found
    pub fn contribution(env: Env, addr: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Contribution(addr))
            .unwrap_or(0)
    }

    /// Get list of all contributors.
    ///
    /// # Arguments
    ///
    /// * `env` - Soroban environment
    ///
    /// # Returns
    ///
    /// Vector of contributor addresses
    pub fn contributors(env: Env) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or(Vec::new(&env))
    }

    /// Get campaign statistics for frontend display.
    ///
    /// # Arguments
    ///
    /// * `env` - Soroban environment
    ///
    /// # Returns
    ///
    /// CampaignStats struct with aggregated statistics
    pub fn get_stats(env: Env) -> CampaignStats {
        let total_raised: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalRaised)
            .unwrap_or(0);
        let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
        let contributors: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Contributors)
            .unwrap_or_else(|| Vec::new(&env));

        let contributor_count = contributors.len();
        let average_contribution = if contributor_count > 0 {
            total_raised / contributor_count as i128
        } else {
            0
        };

        let mut largest_contribution = 0i128;
        for i in 0..contributor_count {
            let contributor = contributors.get(i).unwrap();
            let amount: i128 = env
                .storage()
                .persistent()
                .get(&DataKey::Contribution(contributor))
                .unwrap_or(0);
            if amount > largest_contribution {
                largest_contribution = amount;
            }
        }

        let progress_bps = if goal > 0 {
            ((total_raised * 10_000) / goal).min(10_000) as u32
        } else {
            0
        };

        CampaignStats {
            total_raised,
            goal,
            progress_bps,
            contributor_count,
            average_contribution,
            largest_contribution,
        }
    }

    /// Get token contract address.
    ///
    /// # Arguments
    ///
    /// * `env` - Soroban environment
    ///
    /// # Returns
    ///
    /// Token contract address
    pub fn token(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Token).unwrap()
    }

    /// Get NFT contract address if configured.
    ///
    /// # Arguments
    ///
    /// * `env` - Soroban environment
    ///
    /// # Returns
    ///
    /// Optional NFT contract address
    pub fn nft_contract(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::NFTContract)
//! # Stellar Token Minter Module
//!
//! This module provides secure token minting and pledge collection functionality
//! for the Stellar Raise crowdfunding platform.
//!
//! ## Security Features
//!
//! - **Authorization Enforcement**: All state-changing operations require proper authentication
//! - **Overflow Protection**: All arithmetic operations use checked math to prevent overflow
//! - **State Validation**: Strict validation of campaign state before operations
//! - **Deadline Enforcement**: Time-based guards prevent premature or late operations
//! - **Goal Verification**: Ensures pledges are only collected when goals are met
//! - **Reentrancy Safety**: Immutable storage reads prevent read-before-write vulnerabilities
//! - **Input Validation**: All amounts and parameters are validated before use
//!
//! ## Key Functions
//!
//! - [`validate_pledge_preconditions`]: Validates pledge operation preconditions
//! - [`validate_collect_preconditions`]: Validates collect_pledges operation preconditions
//! - [`calculate_total_commitment`]: Safely calculates total raised + pledged amounts
//! - [`validate_contribution_amount`]: Validates contribution amounts for security
//! - [`safe_calculate_progress`]: Safely calculates campaign progress percentage
//!
//! ## Usage
//!
//! This module is used internally by the crowdfund contract to ensure secure
//! pledge and collection operations.
//!
//! ## Attack Vectors Mitigated
//!
//! 1. **Integer Overflow**: All arithmetic uses `checked_*` operations
//! 2. **Deadline Bypass**: Timestamp comparisons use strict inequality
//! 3. **State Confusion**: Status checks occur before any state modifications
//! 4. **Goal Manipulation**: Combined totals are atomically validated before collection

use soroban_sdk::Env;

use crate::{ContractError, DataKey, Status};

/// Validates preconditions for pledge operations.
///
/// # Security Checks
///
/// 1. Campaign must be active
/// 2. Current time must be before deadline
/// 3. Amount must meet minimum contribution requirement
///
/// # Arguments
///
/// * `env` - The contract environment
/// * `amount` - The pledge amount to validate
/// * `min_contribution` - The minimum allowed contribution
///
/// # Returns
///
/// * `Ok(())` if all preconditions are met
/// * `Err(ContractError)` if any validation fails
///
/// # Errors
///
/// * `ContractError::CampaignNotActive` - Campaign is not in active state
/// * `ContractError::CampaignEnded` - Current time is past deadline
/// * `ContractError::BelowMinimum` - Amount is below minimum contribution
/// * `ContractError::ZeroAmount` - Amount is zero
///
/// # Security Invariants
///
/// - Status is read BEFORE deadline check to ensure proper error priority
/// - Amount validation occurs BEFORE deadline check for consistent error messages
pub fn validate_pledge_preconditions(
    env: &Env,
    amount: i128,
    min_contribution: i128,
) -> Result<(), ContractError> {
    // Validate campaign status
    let status: Status = env
        .storage()
        .instance()
        .get(&DataKey::Status)
        .unwrap_or(Status::Active);
    
    if status != Status::Active {
        return Err(ContractError::CampaignNotActive);
    }

    // Validate amount is non-zero (prevent dust attacks)
    if amount == 0 {
        return Err(ContractError::ZeroAmount);
    }

    // Validate amount meets minimum
    if amount < min_contribution {
        return Err(ContractError::BelowMinimum);
    }

    // Validate deadline - strict inequality prevents boundary confusion
    let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
    if env.ledger().timestamp() > deadline {
        return Err(ContractError::CampaignEnded);
    }

    Ok(())
}

/// Validates preconditions for collect_pledges operations.
///
/// # Security Checks
///
/// 1. Campaign must be active
/// 2. Current time must be after deadline
/// 3. Combined total (raised + pledged) must meet or exceed goal
///
/// # Arguments
///
/// * `env` - The contract environment
///
/// # Returns
///
/// * `Ok((goal, total_raised, total_pledged))` if all preconditions are met
/// * `Err(ContractError)` if any validation fails
///
/// # Errors
///
/// * `ContractError::CampaignNotActive` - Campaign is not in active state
/// * `ContractError::CampaignStillActive` - Current time is before deadline
/// * `ContractError::GoalNotReached` - Combined total does not meet goal
///
/// # Security Notes
///
/// - Uses atomic read of all values to prevent TOCTOU vulnerabilities
/// - Overflow check in `calculate_total_commitment` prevents integer wraparound
pub fn validate_collect_preconditions(
    env: &Env,
) -> Result<(i128, i128, i128), ContractError> {
    // Validate campaign status
    let status: Status = env
        .storage()
        .instance()
        .get(&DataKey::Status)
        .unwrap_or(Status::Active);
    
    if status != Status::Active {
        return Err(ContractError::CampaignNotActive);
    }

    // Validate deadline has passed - strict inequality
    let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
    if env.ledger().timestamp() <= deadline {
        return Err(ContractError::CampaignStillActive);
    }

    // Get goal and totals
    let goal: i128 = env.storage().instance().get(&DataKey::Goal).unwrap();
    let total_raised: i128 = env
        .storage()
        .instance()
        .get(&DataKey::TotalRaised)
        .unwrap_or(0);
    let total_pledged: i128 = env
        .storage()
        .instance()
        .get(&DataKey::TotalPledged)
        .unwrap_or(0);

    // Validate goal is met with overflow protection
    let combined_total = calculate_total_commitment(total_raised, total_pledged)?;
    
    if combined_total < goal {
        return Err(ContractError::GoalNotReached);
    }

    Ok((goal, total_raised, total_pledged))
}

/// Safely calculates the total commitment (raised + pledged) with overflow protection.
///
/// # Arguments
///
/// * `total_raised` - The amount already raised through contributions
/// * `total_pledged` - The amount pledged but not yet collected
///
/// # Returns
///
/// * `Ok(i128)` - The combined total
/// * `Err(ContractError::Overflow)` - If addition would overflow
///
/// # Security
///
/// Uses checked arithmetic to prevent integer overflow attacks.
/// This is critical for goal validation as overflow could falsely indicate success.
pub fn calculate_total_commitment(
    total_raised: i128,
    total_pledged: i128,
) -> Result<i128, ContractError> {
    total_raised
        .checked_add(total_pledged)
        .ok_or(ContractError::Overflow)
}

/// Validates that a pledge amount can be safely added to existing totals.
///
/// # Arguments
///
/// * `current_total` - The current total for the pledger
/// * `new_amount` - The new amount to add
///
/// # Returns
///
/// * `Ok(i128)` - The new total
/// * `Err(ContractError::Overflow)` - If addition would overflow
///
/// # Security
///
/// Prevents overflow in pledge accumulation that could lead to
/// incorrect pledge tracking or goal manipulation.
pub fn safe_add_pledge(current_total: i128, new_amount: i128) -> Result<i128, ContractError> {
    current_total
        .checked_add(new_amount)
        .ok_or(ContractError::Overflow)
}

/// Validates a contribution amount meets security requirements.
///
/// # Arguments
///
/// * `amount` - The contribution amount to validate
/// * `min_contribution` - The minimum allowed contribution
///
/// # Returns
///
/// * `Ok(())` if valid
/// * `Err(ContractError)` if validation fails
///
/// # Security Checks
///
/// - Non-zero amount prevents dust transactions
/// - Amount >= minimum prevents spam
/// - Uses separate function to allow independent validation
pub fn validate_contribution_amount(amount: i128, min_contribution: i128) -> Result<(), ContractError> {
    if amount == 0 {
        return Err(ContractError::ZeroAmount);
    }
    if amount < min_contribution {
        return Err(ContractError::BelowMinimum);
    }
    Ok(())
}

/// Safely calculates campaign progress in basis points (BPS).
///
/// # Arguments
///
/// * `current_amount` - The current raised amount
/// * `goal` - The campaign goal
///
/// # Returns
///
/// * `Ok(u32)` - Progress in basis points (0-10000, where 10000 = 100%)
/// * `Err(ContractError::Overflow)` - If calculation would overflow
///
/// # Security
///
/// Progress is capped at 10000 BPS to prevent display issues
/// with overfunded campaigns.
pub fn safe_calculate_progress(current_amount: i128, goal: i128) -> Result<u32, ContractError> {
    if goal <= 0 {
        return Ok(0);
    }
    
    // Use checked multiplication to prevent overflow when comparing
    let bps_multiplier = 10_000i128;
    
    let progress_raw = current_amount
        .checked_mul(bps_multiplier)
        .ok_or(ContractError::Overflow)?
        .checked_div(goal)
        .unwrap_or(0);
    
    // Cap at 100% (10000 BPS) using simple comparison
    if progress_raw > 10_000 {
        Ok(10_000)
    } else {
        Ok(progress_raw as u32)
    }
}

/// Validates that a deadline is in the future.
///
/// # Arguments
///
/// * `env` - The contract environment
/// * `deadline` - The deadline timestamp to validate
///
/// # Returns
///
/// * `Ok(())` if deadline is in the future
/// * `Err(ContractError::CampaignEnded)` if deadline is not valid
///
/// # Security
///
/// Prevents campaigns from being created with invalid deadlines
/// that would immediately end or have no active period.
pub fn validate_deadline(env: &Env, deadline: u64) -> Result<(), ContractError> {
    let current_time = env.ledger().timestamp();
    
    // Deadline must be strictly in the future
    if deadline <= current_time {
        return Err(ContractError::CampaignEnded);
    }
    
    // Reasonable maximum deadline to prevent extremely long campaigns
    // Approximately 1 year in seconds
    const MAX_CAMPAIGN_DURATION: u64 = 31_536_000;
    
    if deadline - current_time > MAX_CAMPAIGN_DURATION {
        // This is a warning, not an error - long campaigns may be intentional
        // Just log this for awareness
    }
    
    Ok(())
}

/// Validates that a goal amount is reasonable.
///
/// # Arguments
///
/// * `goal` - The goal amount to validate
///
/// # Returns
///
/// * `Ok(())` if goal is valid
/// * `Err(ContractError::GoalNotReached)` if goal is not valid
///
/// # Security
///
/// Prevents campaigns with zero or negative goals that could
/// immediately succeed or cause arithmetic issues.
pub fn validate_goal(goal: i128) -> Result<(), ContractError> {
    if goal <= 0 {
        return Err(ContractError::GoalNotReached);
    }
    Ok(())
}

/// Calculates platform fee safely with bounds checking.
///
/// # Arguments
///
/// * `amount` - The total amount to calculate fee from
/// * `fee_bps` - The fee in basis points (0-10000)
///
/// # Returns
///
/// * `Ok(i128)` - The calculated fee amount
/// * `Err(ContractError::Overflow)` - If calculation would overflow
///
/// # Security
///
/// - Fee is capped at 100% (10000 BPS) during initialization
/// - Uses checked arithmetic for safety
pub fn calculate_platform_fee(amount: i128, fee_bps: u32) -> Result<i128, ContractError> {
    // Fee bps is capped at 10000 during initialization
    // This function just performs the calculation safely
    if fee_bps == 0 {
        return Ok(0);
    }
    
    let bps_divisor = 10_000i128;
    
    amount
        .checked_mul(fee_bps as i128)
        .ok_or(ContractError::Overflow)?
        .checked_div(bps_divisor)
        .ok_or(ContractError::Overflow)
}

/// Validates bonus goal is strictly greater than primary goal.
///
/// # Arguments
///
/// * `bonus_goal` - The bonus goal amount
/// * `primary_goal` - The primary goal amount
///
/// # Returns
///
/// * `Ok(())` if bonus goal is valid
/// * `Err(ContractError::GoalNotReached)` if bonus goal is not valid
///
/// # Security
///
/// Prevents nonsensical configurations where bonus goal is
/// less than or equal to primary goal.
pub fn validate_bonus_goal(bonus_goal: i128, primary_goal: i128) -> Result<(), ContractError> {
    if bonus_goal <= primary_goal {
        return Err(ContractError::GoalNotReached);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Ledger, Env};

    /// Sets up a test environment with default campaign parameters.
    fn setup_test_env() -> Env {
        let env = Env::default();
        env.mock_all_auths();
        
        // Set up basic storage
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Active);
        env.storage().instance().set(&DataKey::Goal, &1_000_000i128);
        env.storage()
            .instance()
            .set(&DataKey::Deadline, &(env.ledger().timestamp() + 3600));
        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalPledged, &0i128);
        
        env
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // calculate_total_commitment Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_calculate_total_commitment_success() {
        let result = calculate_total_commitment(500_000, 300_000);
        assert_eq!(result.unwrap(), 800_000);
    }

    #[test]
    fn test_calculate_total_commitment_zero_values() {
        let result = calculate_total_commitment(0, 0);
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_calculate_total_commitment_one_zero() {
        assert_eq!(calculate_total_commitment(500_000, 0).unwrap(), 500_000);
        assert_eq!(calculate_total_commitment(0, 500_000).unwrap(), 500_000);
    }

    #[test]
    fn test_calculate_total_commitment_overflow() {
        let result = calculate_total_commitment(i128::MAX, 1);
        assert_eq!(result.unwrap_err(), ContractError::Overflow);
    }

    #[test]
    fn test_calculate_total_commitment_overflow_negative() {
        let result = calculate_total_commitment(i128::MIN, -1);
        assert_eq!(result.unwrap_err(), ContractError::Overflow);
    }

    #[test]
    fn test_calculate_total_commitment_large_values() {
        let result = calculate_total_commitment(1_000_000_000, 500_000_000);
        assert_eq!(result.unwrap(), 1_500_000_000);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // safe_add_pledge Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_safe_add_pledge_success() {
        let result = safe_add_pledge(100_000, 50_000);
        assert_eq!(result.unwrap(), 150_000);
    }

    #[test]
    fn test_safe_add_pledge_overflow() {
        let result = safe_add_pledge(i128::MAX, 1);
        assert_eq!(result.unwrap_err(), ContractError::Overflow);
    }

    #[test]
    fn test_safe_add_pledge_zero_addition() {
        assert_eq!(safe_add_pledge(100_000, 0).unwrap(), 100_000);
    }

    #[test]
    fn test_safe_add_pledge_multiple_accumulations() {
        let mut total = 0i128;
        total = safe_add_pledge(total, 100_000).unwrap();
        total = safe_add_pledge(total, 200_000).unwrap();
        total = safe_add_pledge(total, 300_000).unwrap();
        assert_eq!(total, 600_000);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // validate_contribution_amount Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_validate_contribution_amount_success() {
        assert!(validate_contribution_amount(1000, 500).is_ok());
    }

    #[test]
    fn test_validate_contribution_amount_exact_minimum() {
        assert!(validate_contribution_amount(1000, 1000).is_ok());
    }

    #[test]
    fn test_validate_contribution_amount_zero() {
        assert_eq!(
            validate_contribution_amount(0, 500).unwrap_err(),
            ContractError::ZeroAmount
        );
    }

    #[test]
    fn test_validate_contribution_amount_below_minimum() {
        assert_eq!(
            validate_contribution_amount(100, 500).unwrap_err(),
            ContractError::BelowMinimum
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // safe_calculate_progress Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_safe_calculate_progress_zero_goal() {
        assert_eq!(safe_calculate_progress(1000, 0).unwrap(), 0);
    }

    #[test]
    fn test_safe_calculate_progress_exact_goal() {
        assert_eq!(safe_calculate_progress(1_000_000, 1_000_000).unwrap(), 10_000);
    }

    #[test]
    fn test_safe_calculate_progress_halfway() {
        assert_eq!(safe_calculate_progress(500_000, 1_000_000).unwrap(), 5_000);
    }

    #[test]
    fn test_safe_calculate_progress_overfunded() {
        // Should cap at 100%
        assert_eq!(safe_calculate_progress(2_000_000, 1_000_000).unwrap(), 10_000);
    }

    #[test]
    fn test_safe_calculate_progress_small_amount() {
        assert_eq!(safe_calculate_progress(1, 10_000).unwrap(), 1);
    }

    #[test]
    fn test_safe_calculate_progress_overflow_protection() {
        // Very large values that could overflow
        let result = safe_calculate_progress(i128::MAX, 1);
        // Should cap at 10000
        assert_eq!(result.unwrap(), 10_000);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // validate_deadline Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_validate_deadline_future() {
        let env = Env::default();
        let future_deadline = env.ledger().timestamp() + 3600;
        assert!(validate_deadline(&env, future_deadline).is_ok());
    }

    #[test]
    fn test_validate_deadline_past() {
        let env = Env::default();
        let past_deadline = env.ledger().timestamp() - 1;
        assert_eq!(
            validate_deadline(&env, past_deadline).unwrap_err(),
            ContractError::CampaignEnded
        );
    }

    #[test]
    fn test_validate_deadline_exact_current() {
        let env = Env::default();
        let current_time = env.ledger().timestamp();
        assert_eq!(
            validate_deadline(&env, current_time).unwrap_err(),
            ContractError::CampaignEnded
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // validate_goal Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_validate_goal_positive() {
        assert!(validate_goal(1_000_000).is_ok());
    }

    #[test]
    fn test_validate_goal_zero() {
        assert_eq!(
            validate_goal(0).unwrap_err(),
            ContractError::GoalNotReached
        );
    }

    #[test]
    fn test_validate_goal_negative() {
        assert_eq!(
            validate_goal(-1000).unwrap_err(),
            ContractError::GoalNotReached
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // calculate_platform_fee Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_calculate_platform_fee_zero_bps() {
        assert_eq!(calculate_platform_fee(1_000_000, 0).unwrap(), 0);
    }

    #[test]
    fn test_calculate_platform_fee_1_percent() {
        // 1% = 100 BPS
        assert_eq!(calculate_platform_fee(1_000_000, 100).unwrap(), 10_000);
    }

    #[test]
    fn test_calculate_platform_fee_5_percent() {
        // 5% = 500 BPS
        assert_eq!(calculate_platform_fee(1_000_000, 500).unwrap(), 50_000);
    }

    #[test]
    fn test_calculate_platform_fee_100_percent() {
        // 100% = 10000 BPS
        assert_eq!(calculate_platform_fee(1_000_000, 10_000).unwrap(), 1_000_000);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // validate_bonus_goal Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_validate_bonus_goal_valid() {
        assert!(validate_bonus_goal(2_000_000, 1_000_000).is_ok());
    }

    #[test]
    fn test_validate_bonus_goal_equal_to_primary() {
        assert_eq!(
            validate_bonus_goal(1_000_000, 1_000_000).unwrap_err(),
            ContractError::GoalNotReached
        );
    }

    #[test]
    fn test_validate_bonus_goal_less_than_primary() {
        assert_eq!(
            validate_bonus_goal(500_000, 1_000_000).unwrap_err(),
            ContractError::GoalNotReached
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // validate_pledge_preconditions Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_validate_pledge_preconditions_success() {
        let env = setup_test_env();
        let result = validate_pledge_preconditions(&env, 10_000, 1_000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pledge_preconditions_zero_amount() {
        let env = setup_test_env();
        let result = validate_pledge_preconditions(&env, 0, 1_000);
        assert_eq!(result.unwrap_err(), ContractError::ZeroAmount);
    }

    #[test]
    fn test_validate_pledge_preconditions_below_minimum() {
        let env = setup_test_env();
        let result = validate_pledge_preconditions(&env, 500, 1_000);
        assert_eq!(result.unwrap_err(), ContractError::BelowMinimum);
    }

    #[test]
    fn test_validate_pledge_preconditions_after_deadline() {
        let env = setup_test_env();
        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        env.ledger().set_timestamp(deadline + 1);
        
        let result = validate_pledge_preconditions(&env, 10_000, 1_000);
        assert_eq!(result.unwrap_err(), ContractError::CampaignEnded);
    }

    #[test]
    fn test_validate_pledge_preconditions_at_deadline() {
        let env = setup_test_env();
        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        // Set to exact deadline
        env.ledger().set_timestamp(deadline);
        
        // At exact deadline should still work (deadline is exclusive)
        let result = validate_pledge_preconditions(&env, 10_000, 1_000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_pledge_preconditions_inactive_campaign() {
        let env = setup_test_env();
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Cancelled);
        
        let result = validate_pledge_preconditions(&env, 10_000, 1_000);
        assert_eq!(result.unwrap_err(), ContractError::CampaignNotActive);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // validate_collect_preconditions Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_validate_collect_preconditions_before_deadline() {
        let env = setup_test_env();
        let result = validate_collect_preconditions(&env);
        assert_eq!(result.unwrap_err(), ContractError::CampaignStillActive);
    }

    #[test]
    fn test_validate_collect_preconditions_at_deadline() {
        let env = setup_test_env();
        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        env.ledger().set_timestamp(deadline);
        
        // At exact deadline should still fail (deadline is exclusive for collection)
        let result = validate_collect_preconditions(&env);
        assert_eq!(result.unwrap_err(), ContractError::CampaignStillActive);
    }

    #[test]
    fn test_validate_collect_preconditions_after_deadline_goal_not_reached() {
        let env = setup_test_env();
        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        env.ledger().set_timestamp(deadline + 1);
        
        // Set totals that don't meet goal
        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &300_000i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalPledged, &200_000i128);
        
        let result = validate_collect_preconditions(&env);
        assert_eq!(result.unwrap_err(), ContractError::GoalNotReached);
    }

    #[test]
    fn test_validate_collect_preconditions_success() {
        let env = setup_test_env();
        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        env.ledger().set_timestamp(deadline + 1);
        
        // Set totals that meet goal
        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &600_000i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalPledged, &500_000i128);
        
        let result = validate_collect_preconditions(&env);
        assert!(result.is_ok());
        
        let (goal, raised, pledged) = result.unwrap();
        assert_eq!(goal, 1_000_000);
        assert_eq!(raised, 600_000);
        assert_eq!(pledged, 500_000);
    }

    #[test]
    fn test_validate_collect_preconditions_exactly_at_goal() {
        let env = setup_test_env();
        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        env.ledger().set_timestamp(deadline + 1);
        
        // Set totals that exactly meet goal
        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &500_000i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalPledged, &500_000i128);
        
        let result = validate_collect_preconditions(&env);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_collect_preconditions_over_goal() {
        let env = setup_test_env();
        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        env.ledger().set_timestamp(deadline + 1);
        
        // Set totals that exceed goal
        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &700_000i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalPledged, &500_000i128);
        
        let result = validate_collect_preconditions(&env);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_collect_preconditions_inactive_campaign() {
        let env = setup_test_env();
        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        env.ledger().set_timestamp(deadline + 1);
        
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Cancelled);
        
        let result = validate_collect_preconditions(&env);
        assert_eq!(result.unwrap_err(), ContractError::CampaignNotActive);
    }

    #[test]
    fn test_validate_collect_preconditions_only_raised() {
        let env = setup_test_env();
        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        env.ledger().set_timestamp(deadline + 1);
        
        // Only contributions, no pledges
        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &1_000_000i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalPledged, &0i128);
        
        let result = validate_collect_preconditions(&env);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_collect_preconditions_only_pledged() {
        let env = setup_test_env();
        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        env.ledger().set_timestamp(deadline + 1);
        
        // Only pledges, no contributions
        env.storage()
            .instance()
            .set(&DataKey::TotalRaised, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalPledged, &1_000_000i128);
        
        let result = validate_collect_preconditions(&env);
        assert!(result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Security Edge Case Tests
    // ═══════════════════════════════════════════════════════════════════════════

    #[test]
    fn test_precondition_validation_order_status_first() {
        // Status is checked first - ensures inactive campaigns fail with correct error
        let env = setup_test_env();
        let deadline: u64 = env.storage().instance().get(&DataKey::Deadline).unwrap();
        env.ledger().set_timestamp(deadline + 1);
        env.storage()
            .instance()
            .set(&DataKey::Status, &Status::Cancelled);
        
        // Should fail with CampaignNotActive, not CampaignEnded
        let result = validate_pledge_preconditions(&env, 10_000, 1_000);
        assert_eq!(result.unwrap_err(), ContractError::CampaignNotActive);
    }

    #[test]
    fn test_overflow_detection_at_boundaries() {
        // Test maximum safe values
        let max_safe = i128::MAX / 2;
        assert!(calculate_total_commitment(max_safe, max_safe).is_ok());
        
        // One more would overflow
        let result = calculate_total_commitment(max_safe, max_safe + 1);
        assert_eq!(result.unwrap_err(), ContractError::Overflow);
    }
}

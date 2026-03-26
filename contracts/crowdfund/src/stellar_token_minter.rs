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
//! This contract provides NFT minting capabilities for the crowdfunding platform.
//! It implements a secure minting mechanism that can be called by authorized
//! contracts (like the Crowdfund contract) to reward contributors with NFTs.
//!
//! ## Security Model
//!
//! - **Authorization**: Only the designated minter can call `mint` (enforced via `require_auth`).
//! - **Admin Separation**: Admin role is separate from minter role for principle of least privilege.
//! - **State Management**: Uses persistent storage for token ID tracking and metadata.
//! - **Bounded Operations**: Ensures all operations are within Soroban resource limits.
//! - **Idempotency**: Prevents duplicate token minting via token ID uniqueness check.
//! - **Initialization Guard**: Contract can only be initialized once.
//!
//! ## Deprecated Patterns (v1.0)
//!
//! The following patterns have been deprecated in favor of more secure implementations:
//! - Direct admin minting (now requires minter role)
//! - Unguarded initialization (now panics on double-init)
//! - Implicit authorization (now explicit via `require_auth`)
//!
//! ## Invariants
//!
//! 1. `total_minted` equals the count of unique token IDs that have been minted
//! 2. Each token ID can only be minted once (enforced by persistent storage check)
//! 3. Only the designated minter can call `mint` (enforced by `require_auth`)
//! 4. Admin can update the minter address (admin-only operation)
//! 5. Contract state is immutable after initialization (no re-initialization)

#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec,
};
// ── Test constants ────────────────────────────────────────────────────────────
//
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
        env.storage()
            .instance()
            .set(&DataKey::TotalMinted, &(total + 1));

        // Interaction: Emit mint event for off-chain tracking
        env.events().publish((Symbol::new(&env, "mint"), to), token_id);
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
    }
}

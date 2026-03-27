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

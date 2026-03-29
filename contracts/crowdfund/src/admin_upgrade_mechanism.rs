use crate::DataKey;
use soroban_sdk::{Address, BytesN, Env};

// ── Constants ─────────────────────────────────────────────────────────────────

/// A zeroed 32-byte hash is never a valid WASM hash.
/// Rejecting it before any storage read or deployer call saves gas.
const ZERO_HASH: [u8; 32] = [0u8; 32];

// ── Pure helpers (no Env required) ───────────────────────────────────────────

/// @title validate_wasm_hash
/// @notice Returns `true` when `wasm_hash` is non-zero.
/// @dev Pure function — no storage reads, no auth, minimal gas cost.
///      Called before `validate_admin_upgrade` so an invalid hash is rejected
///      at the cheapest possible point in the call stack.
/// @security Prevents upgrade calls with a zeroed hash, which would brick
///           the contract by replacing its executable code with nothing.
pub fn validate_wasm_hash(wasm_hash: &BytesN<32>) -> bool {
    wasm_hash.to_array() != ZERO_HASH
}

// ── Storage helpers ───────────────────────────────────────────────────────────

/// @title is_admin_initialized
/// @notice Returns `true` when an admin address has been stored.
/// @dev Uses `has()` — a single existence check — rather than `get()` + unwrap,
///      which avoids deserializing the stored value when only presence matters.
///      Callers that only need to gate on initialization should prefer this over
///      `validate_admin_upgrade` to avoid the unnecessary `require_auth()` cost.
/// @security Read-only; no state mutations.
pub fn is_admin_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

/// @title validate_admin_upgrade
/// @notice Loads the stored admin address and enforces authorization.
/// @dev Panics with "Admin not initialized" when no admin is stored, and
///      delegates auth enforcement to Soroban's `require_auth()`.
///      Callers MUST call `validate_wasm_hash` before this function to
///      short-circuit on a zero hash before paying the storage-read cost.
/// @security `require_auth()` ensures the transaction is signed by the admin
///           address stored during initialization.
pub fn validate_admin_upgrade(env: &Env) -> Address {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("Admin not initialized");
    admin.require_auth();
    admin
}

/// @title perform_upgrade
/// @notice Executes the WASM swap via the Soroban deployer.
/// @dev Must only be called after both `validate_wasm_hash` and
///      `validate_admin_upgrade` have succeeded.  Separating validation from
///      execution keeps each function single-responsibility and testable in
///      isolation.
pub fn perform_upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
    env.deployer().update_current_contract_wasm(new_wasm_hash);
//! # Admin Upgrade Mechanism
//!
//! This module provides a secure and auditable mechanism for upgrading smart contract
//! WASM code. It ensures that only authorized administrators can perform upgrades,
//! while maintaining full transparency and audit capabilities through event emissions.
//!
//! ## Security Features
//!
//! - **Authentication**: All upgrade operations require admin authentication via `require_auth()`
//! - **Atomic Operations**: Upgrades are atomic - they either succeed completely or fail
//! - **Event Audit Trail**: All upgrade operations emit events for off-chain monitoring
//! - **Admin Verification**: Contract prevents upgrades before initialization
//! - **Separation of Concerns**: Admin role is distinct from campaign creator role
//!
//! @author Stellar Crowdfund Protocol
//! @version 1.0.0

#![allow(unused)]

/// Storage keys for admin upgrade mechanism.
#[derive(Clone)]
pub enum DataKey {
    /// The current admin address authorized to perform upgrades.
    Admin,
    /// The current WASM hash of the deployed contract.
    CurrentWasmHash,
    /// Upgrade history for audit purposes.
    UpgradeHistory,
}

/// Errors that can occur during admin upgrade operations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum UpgradeError {
    /// The contract has not been initialized yet (no admin set).
    NotInitialized = 1,
    /// The caller is not the authorized admin.
    NotAuthorized = 2,
    /// The provided WASM hash is invalid (e.g., zero bytes).
    InvalidWasmHash = 3,
    /// The new WASM hash is the same as the current one.
    SameWasmHash = 4,
    /// The new admin address is the same as the current admin.
    SameAdmin = 5,
    /// The new admin address is invalid (e.g., zero address).
    InvalidAdminAddress = 6,
}

/// Helper struct for admin upgrade operations.
pub struct AdminUpgradeHelper;

impl AdminUpgradeHelper {
    /// Validate a WASM hash.
    ///
    /// A valid WASM hash must not be all zeros.
    ///
    /// # Arguments
    /// * `wasm_hash` - The WASM hash to validate
    ///
    /// # Returns
    /// * `Result<(), UpgradeError>` - Ok if valid, error otherwise
    pub fn validate_wasm_hash(wasm_hash: &soroban_sdk::BytesN<32>) -> Result<(), UpgradeError> {
        // Check for zero hash (all zeros)
        let hash_array = wasm_hash.to_array();
        let is_zero = hash_array.iter().all(|&b| b == 0);
        if is_zero {
            return Err(UpgradeError::InvalidWasmHash);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::BytesN;

    use super::AdminUpgradeHelper;

    /// Test that valid WASM hash passes validation.
    #[test]
    fn test_validate_wasm_hash_valid() {
        let env = soroban_sdk::Env::default();
        let valid_hash = BytesN::from_array(&env, &[0xAB; 32]);
        assert!(AdminUpgradeHelper::validate_wasm_hash(&valid_hash).is_ok());
    }

    /// Test that zero WASM hash is rejected.
    #[test]
    fn test_validate_wasm_hash_zero_rejected() {
        let env = soroban_sdk::Env::default();
        let zero_hash = BytesN::from_array(&env, &[0u8; 32]);
        assert_eq!(
            AdminUpgradeHelper::validate_wasm_hash(&zero_hash),
            Err(super::UpgradeError::InvalidWasmHash)
        );
    }

    /// Test that max value WASM hash is valid.
    #[test]
    fn test_max_value_wasm_hash_valid() {
        let env = soroban_sdk::Env::default();
        let max_value = BytesN::from_array(&env, &[0xFF; 32]);
        assert!(AdminUpgradeHelper::validate_wasm_hash(&max_value).is_ok());
    }
}
use soroban_sdk::{Address, Env, BytesN};
use soroban_sdk::{Address, BytesN, Env};

use soroban_sdk::{Address, BytesN, Env};
use crate::DataKey;

// ── Constants ────────────────────────────────────────────────────────────────

/// A WASM hash of all-zero bytes is treated as unset / invalid.
/// Callers must upload a real WASM binary before calling `upgrade()`.
const ZERO_HASH: [u8; 32] = [0u8; 32];

// ── Public API ───────────────────────────────────────────────────────────────

/// Validates that the caller is the authorized admin for contract upgrades.
///
/// ### Security Note
/// Uses `require_auth()` to ensure the transaction is signed by the admin
/// address stored during initialization.
///
/// ### Panics
/// - If no admin is stored (contract not yet initialized).
/// @notice Retrieves the admin address stored during `initialize()` and calls
///         `require_auth()` on it. Panics if no admin has been set.
/// @dev    Uses `require_auth()` which ensures the transaction is signed by
///         the admin address stored during initialization.
/// @return The admin `Address` that was authenticated.
///
/// @custom:security Never call this function without subsequently performing
///                  the upgrade — the auth check is not idempotent.
use soroban_sdk::{Address, BytesN, Env};
use crate::DataKey;
use soroban_sdk::{Address, BytesN, Env};

// ── Constants ─────────────────────────────────────────────────────────────────

/// A zeroed 32-byte hash is never a valid WASM hash.
/// Rejecting it before any storage read or deployer call saves gas.
const ZERO_HASH: [u8; 32] = [0u8; 32];

// ── Pure helpers (no Env required) ───────────────────────────────────────────

/// @title validate_wasm_hash
/// @notice Returns `true` when `wasm_hash` is non-zero.
/// @dev Pure function — no storage reads, no auth, minimal gas cost.
///      Called before `validate_admin_upgrade` so an invalid hash is rejected
///      at the cheapest possible point in the call stack.
/// @security Prevents upgrade calls with a zeroed hash, which would brick
///           the contract by replacing its executable code with nothing.
pub fn validate_wasm_hash(wasm_hash: &BytesN<32>) -> bool {
    wasm_hash.to_array() != ZERO_HASH
}

// ── Storage helpers ───────────────────────────────────────────────────────────

/// @title is_admin_initialized
/// @notice Returns `true` when an admin address has been stored.
/// @dev Uses `has()` — a single existence check — rather than `get()` + unwrap,
///      which avoids deserializing the stored value when only presence matters.
///      Callers that only need to gate on initialization should prefer this over
///      `validate_admin_upgrade` to avoid the unnecessary `require_auth()` cost.
/// @security Read-only; no state mutations.
pub fn is_admin_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

/// @title validate_admin_upgrade
/// @notice Loads the stored admin address and enforces authorization.
/// @dev Panics with "Admin not initialized" when no admin is stored, and
///      delegates auth enforcement to Soroban's `require_auth()`.
///      Callers MUST call `validate_wasm_hash` before this function to
///      short-circuit on a zero hash before paying the storage-read cost.
/// @security `require_auth()` ensures the transaction is signed by the admin
///           address stored during initialization.
pub fn validate_admin_upgrade(env: &Env) -> Address {
    let admin: Address = env
        .storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("Admin not initialized");

    admin.require_auth();
    admin
}

/// Validates that the WASM hash is non-zero (all-zero hash is invalid).
///
/// An all-zero hash is the default/unset value and would indicate a missing
/// or malformed upload. Rejecting it prevents accidental no-op upgrades.
///
/// ### Panics
/// - If `new_wasm_hash` is all zeros.
pub fn validate_wasm_hash(new_wasm_hash: &BytesN<32>) {
    assert!(
        new_wasm_hash.to_array() != [0u8; 32],
        "upgrade: wasm_hash must not be zero"
    );
/// Validates that `new_wasm_hash` is not the all-zero sentinel value.
///
/// @notice An all-zero hash indicates the WASM was never uploaded.
///         Rejecting it prevents a no-op upgrade that would still consume gas
///         and emit a misleading audit event.
/// @param  new_wasm_hash  The 32-byte SHA-256 hash to validate.
///
/// @custom:security This is a best-effort guard. The Soroban host will also
///                  reject an unregistered hash at execution time; this check
///                  surfaces the error earlier with a clear message.
pub fn validate_wasm_hash(new_wasm_hash: &BytesN<32>) -> bool {
    new_wasm_hash.to_array() != ZERO_HASH
}

/// Executes the WASM update.
///
/// @notice Replaces the running contract WASM with the binary identified by
///         `new_wasm_hash`. The contract address and all storage are preserved.
/// @dev    Must only be called after `validate_admin_upgrade()` and
///         `validate_wasm_hash()` have both passed.
/// @param  new_wasm_hash  SHA-256 hash of the new WASM binary (already uploaded).
///
/// @custom:security This operation is irreversible within the same transaction.
///                  Ensure the new WASM is thoroughly tested before calling.
/// @title perform_upgrade
/// @notice Executes the WASM swap via the Soroban deployer.
/// @dev Must only be called after both `validate_wasm_hash` and
///      `validate_admin_upgrade` have succeeded.  Separating validation from
///      execution keeps each function single-responsibility and testable in
///      isolation.
pub fn perform_upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
//! # admin_upgrade_mechanism
//!
//! @title   AdminUpgradeMechanism — Restricted WASM upgrade logic for the
//!          crowdfund contract.
//!
//! @notice  This module exposes a single entry point, `upgrade()`, that
//!          replaces the contract's on-chain WASM binary with a new version
//!          identified by its SHA-256 hash.  The call is gated behind a strict
//!          `admin.require_auth()` check — only the address stored as `Admin`
//!          during `initialize()` may invoke it.
//!
//! @dev     ## Centralized upgradeability — risks and mitigations
//!
//!          Upgradeable contracts introduce a trust assumption: whoever controls
//!          the admin key controls the contract logic.  Risks include:
//!
//!          1. **Key compromise** — If the admin's private key is stolen, an
//!             attacker can deploy arbitrary WASM, potentially draining all
//!             contributor funds or redirecting withdrawals.
//!             *Mitigation*: use a multisig or governance contract as admin,
//!             never a plain EOA.
//!
//!          2. **Malicious upgrade** — A rogue admin (or compromised multisig
//!             signer) could push WASM that removes refund logic or changes
//!             the withdrawal recipient.
//!             *Mitigation*: time-lock upgrades and require off-chain review
//!             before execution; publish the WASM source and verify the hash.
//!
//!          3. **Irreversibility** — Once `update_current_contract_wasm` is
//!             called the old WASM is gone.  There is no built-in rollback.
//!             *Mitigation*: test the new WASM on testnet, verify storage
//!             compatibility, and keep the old WASM hash for reference.
//!
//!          4. **Storage layout drift** — A new WASM that changes `DataKey`
//!             variants or storage types can corrupt existing state.
//!             *Mitigation*: treat storage layout as a public API; add new
//!             keys rather than changing existing ones.
//!
//! ## Upgrade flow
//!
//! ```text
//! 1. Build new WASM  →  cargo build --release --target wasm32-unknown-unknown
//! 2. Upload WASM     →  stellar contract install --wasm <file> --network testnet
//!                        returns <WASM_HASH> (32-byte hex)
//! 3. Call upgrade()  →  stellar contract invoke --id <CONTRACT> -- upgrade
//!                        --new_wasm_hash <WASM_HASH>
//!                        (must be signed by the admin key)
//! ```

use soroban_sdk::{BytesN, Env};

use crate::DataKey;

/// Upgrades the contract to a new WASM implementation — admin-only.
///
/// @notice  Replaces the running WASM binary in-place.  All contract storage
///          and the contract address are preserved across the upgrade.
///
/// @dev     Reads the admin address from instance storage (set during
///          `initialize()`).  Panics with an `unwrap()` failure if called
///          before `initialize()` — this is intentional: an uninitialized
///          contract has no admin, so no upgrade should be possible.
///
/// @param  env           The Soroban execution environment.
/// @param  new_wasm_hash SHA-256 hash of the new WASM binary, exactly 32 bytes.
///                       The binary must already be uploaded to the ledger via
///                       `stellar contract install` before this call.
///
/// ## Security assumptions
///
/// - `BytesN<32>` is enforced by the Soroban type system — the host rejects
///   any invocation that supplies a hash of the wrong length at the ABI layer,
///   before this function body is reached.
/// - The hash is opaque to this function; validity (i.e. the WASM exists on
///   the ledger) is checked by `update_current_contract_wasm` in the host.
///   An unknown hash causes a host-level trap, not a Rust panic.
/// - No integer arithmetic is performed, so overflow is impossible.
pub fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
    let admin: soroban_sdk::Address =
        env.storage().instance().get(&DataKey::Admin).unwrap();
    admin.require_auth();
    env.deployer().update_current_contract_wasm(new_wasm_hash);
}

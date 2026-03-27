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
}

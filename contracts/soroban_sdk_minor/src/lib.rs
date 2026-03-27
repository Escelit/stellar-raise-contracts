#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

/// Storage key for the admin address.
#[contracttype]
pub enum DataKey {
    Admin,
}

/// # SorobanSdkMinor
///
/// @title   SorobanSdkMinor — Soroban SDK v22 patterns demonstration
/// @notice  Showcases updated Address, Auth, and storage patterns introduced
///          in the Soroban SDK v22 minor version bump.
/// @dev     Uses `contracttype` enum keys (not raw strings) for storage,
///          `require_auth()` for authorization, and `env.register` in tests.
///
/// ## Security Assumptions
/// 1. Only the admin can call `init` — enforced via `require_auth()`.
/// 2. Only the caller themselves can satisfy `check_auth` — enforced via `require_auth()`.
/// 3. Storage keys are typed enums, preventing key-collision bugs.
#[contract]
pub struct SorobanSdkMinor;

#[contractimpl]
impl SorobanSdkMinor {
    /// @notice Initializes the contract by storing the admin address.
    /// @dev    Requires the admin to authorize this call (Soroban v22 auth pattern).
    /// @param  admin The administrator address to store.
    pub fn init(env: Env, admin: Address) {
        admin.require_auth();
        // Prevent re-initialization: if Admin is already present, refuse to overwrite.
        // This enforces a one-time init semantics and closes a potential
        // accidental takeover/reinitialization attack vector.
        if env
            .storage()
            .instance()
            .get::<Address>(&DataKey::Admin)
            .is_some()
        {
            panic!("already initialized");
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
    }

    /// @notice Verifies that `user` has authorized the current call.
    /// @param  user The address whose authorization is being checked.
    /// @return `true` if authorization succeeds (panics otherwise).
    pub fn check_auth(_env: Env, user: Address) -> bool {
        user.require_auth();
        true
    }

    /// @notice Emit a small, typed event (topic = `ping`) that demonstrates
    ///         the v22 logging/event bounds using a typed payload.
    /// @dev    The emitter must authorize the call via `require_auth()`.
    /// @param  from  The address which emits the event (must authorize).
    /// @param  value A small integer payload included in the event.
    pub fn emit_ping(env: Env, from: Address, value: i32) {
        // enforce the new auth pattern
        from.require_auth();
        // use a short Symbol topic and a primitive payload which satisfy
        // the Soroban v22 bounds for events
        env.events().publish((Symbol::short("ping"),), value);
    }

    /// @notice Returns the stored admin address.
    /// @dev    Panics if `init` has not been called yet.
    /// @return The admin `Address`.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .expect("not initialized")
    }
}

mod test;

#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, String};

#[contract]
pub struct SorobanSdkMinor;

#[contractimpl]
impl SorobanSdkMinor {
    /// @title Initialize Utility
    /// @notice Demonstrates updated Address and Auth patterns in Soroban SDK v22.
    /// @param env The Soroban environment.
    /// @param admin The administrator address.
    pub fn init(env: Env, admin: Address) {
        admin.require_auth();
        env.storage().instance().set(&String::from_str(&env, "admin"), &admin);
    }

    /// @notice Demonstrates v22 Address handling and cross-contract call patterns.
    /// @dev In v22, Address objects are more robust and require_auth is the preferred pattern for authorization.
    pub fn check_auth(env: Env, user: Address) -> bool {
        // NatSpec: require_auth() verifies that the 'user' has authorized this call.
        // This is a core pattern in the Soroban security model.
        user.require_auth();
        true
    }

    /// @notice Returns the footprint reduction logic example.
    /// @dev v22 optimizations allow for more efficient storage access.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&String::from_str(&env, "admin"))
            .expect("not initialized")
    }
}

mod test;

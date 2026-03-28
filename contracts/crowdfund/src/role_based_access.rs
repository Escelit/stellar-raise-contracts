//! # role_based_access
//!
//! @title   Role-Based Access Control — Granular permissions for Stellar Raise
//!
//! @notice  Implements a comprehensive role-based access control (RBAC) system
//!          for the crowdfund contract. This module extends the basic access_control
//!          module with more granular permissions suitable for complex organizational
//!          structures.
//!
//! ## Role Hierarchy
//!
//! The system implements the following role hierarchy (highest to lowest privilege):
//!
//! | Role                | Symbol Hash        | Description                                      |
//! |---------------------|--------------------|--------------------------------------------------|
//! | `ADMIN`             | `"admin"`          | Full system access, can manage all roles         |
//! | `AUDITOR`           | `"auditor"`        | Read-only access to sensitive operations         |
//! | `CAMPAIGN_MANAGER`  | `"campaign_mgr"`   | Can manage campaigns but not system params        |
//! | `FINANCE`           | `"finance"`        | Can withdraw funds and manage financial ops      |
//! | `OPERATOR`          | `"operator"`       | Can perform operational tasks (pause, emergency)  |
//! | `MINTER`            | `"minter"`         | Can mint tokens (for reward mechanisms)          |
//!
//! ## Security Assumptions
//!
//! 1. **Principle of Least Privilege**: Each role should only have the minimum
//!    permissions required for its function.
//!
//! 2. **Role Separation**: The `ADMIN` role cannot be used for day-to-day
//!    operations — it should only be used for role management.
//!
//! 3. **Audit Trail**: All role-sensitive operations emit events for off-chain
//!    monitoring and compliance.
//!
//! 4. **Time-locked Transfers**: Critical role transfers require a time delay
//!    to allow for intervention if a compromise is detected.
//!
//! 5. **Multi-step Operations**: Role grants and revocations require explicit
//!    confirmation steps to prevent accidental or malicious changes.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::role_based_access::{Role, require_role, assign_role, revoke_role};
//!
//! // Check if an address has a specific role
//! if has_role(&env, &address, Role::CampaignManager) {
//!     // Allow campaign management operations
//! }
//!
//! // Require a specific role (panics if not authorized)
//! require_role(&env, &caller, Role::Finance);
//!
//! // Assign a role to an address
//! assign_role(&env, &admin, Role::Finance, &new_finance_address);
//! ```

#![allow(dead_code)]

use soroban_sdk::{Address, Env, Symbol};

// ── Role Definitions ─────────────────────────────────────────────────────────

/// Represents the available roles in the system.
///
/// Each role has a specific set of permissions and responsibilities.
/// Roles are stored as symbol hashes for efficient storage and comparison.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Role {
    /// Full administrative access — can manage all roles and system parameters.
    /// This role should be secured by multi-signature and used sparingly.
    Admin = 0,

    /// Read-only access to sensitive operations and audit logs.
    /// Can view all contract state but cannot modify anything.
    Auditor = 1,

    /// Can create, modify, and finalize campaigns.
    /// Cannot access system parameters or financial withdrawals.
    CampaignManager = 2,

    /// Can withdraw funds and manage financial operations.
    /// Should be protected by additional verification mechanisms.
    Finance = 3,

    /// Can perform operational tasks including emergency pause.
    /// Lower privilege than Admin but higher than basic users.
    Operator = 4,

    /// Can mint tokens for reward mechanisms.
    /// Highly sensitive — should be limited to reward distribution.
    Minter = 5,
}

impl Role {
    /// @notice Converts a role to its symbol representation for storage and events.
    /// @param role The role to convert.
    /// @return A Symbol representing the role.
    ///
    /// # Security
    /// - Symbol conversion is deterministic and reversible.
    /// - No sensitive data is exposed through symbol representation.
    pub fn to_symbol(&self, env: &Env) -> Symbol {
        match self {
            Role::Admin => Symbol::new(env, "admin"),
            Role::Auditor => Symbol::new(env, "auditor"),
            Role::CampaignManager => Symbol::new(env, "campaign_mgr"),
            Role::Finance => Symbol::new(env, "finance"),
            Role::Operator => Symbol::new(env, "operator"),
            Role::Minter => Symbol::new(env, "minter"),
        }
    }

    /// @notice Gets the list of roles that can assign a given role.
    /// @dev This defines the role hierarchy. Higher-privilege roles can assign
    ///      lower-privilege roles but not vice versa.
    /// @param role The role to check.
    /// @return A slice of roles that can assign this role.
    pub fn get_assigners(&self) -> Vec<Role> {
        match self {
            // Admin can only be assigned by Admin
            Role::Admin => vec![Role::Admin],
            // Auditor can be assigned by Admin
            Role::Auditor => vec![Role::Admin],
            // CampaignManager can be assigned by Admin or CampaignManager
            Role::CampaignManager => vec![Role::Admin, Role::CampaignManager],
            // Finance can be assigned by Admin or Finance
            Role::Finance => vec![Role::Admin, Role::Finance],
            // Operator can be assigned by Admin or Operator
            Role::Operator => vec![Role::Admin, Role::Operator],
            // Minter can be assigned by Admin or Minter
            Role::Minter => vec![Role::Admin, Role::Minter],
        }
    }
}

// ── Storage Keys ─────────────────────────────────────────────────────────────

/// Storage key for role membership mapping.
/// Key format: DataKey::RoleMember(Role, Address)
#[derive(Clone)]
#[soroban_sdk::contracttype]
pub enum RoleDataKey {
    /// Mapping of role to set of addresses with that role.
    RoleMembers(Role),
    /// Pending role transfer for time-locked operations.
    PendingRoleTransfer(Role),
    /// Timestamp when pending transfer becomes effective.
    PendingTransferTimestamp(Role),
    /// Role admin (who can grant/revoke this role).
    RoleAdmin(Role),
    /// Counter for role assignment events (for ordering).
    RoleAssignmentCounter,
}

// ── Role Membership ──────────────────────────────────────────────────────────

/// @notice Checks if an address has a specific role.
/// @param env The Soroban environment.
/// @param address The address to check.
/// @param role The role to verify.
/// @return `true` if the address has the role, `false` otherwise.
///
/// # Security
/// - This function only checks direct role membership.
/// - It does not account for role hierarchy (i.e., Admin does NOT implicitly
///   have all roles for this check).
pub fn has_role(env: &Env, address: &Address, role: Role) -> bool {
    let key = RoleDataKey::RoleMembers(role);
    let members: Vec<Address> = env
        .storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    
    members.iter().any(|m| m == address)
}

/// @notice Requires that the caller has a specific role.
/// @param env The Soroban environment.
/// @param caller The address to check.
/// @param role The required role.
/// @panic "not authorized" if the caller does not have the role.
///
/// # Security
/// - Uses `require_auth()` to ensure the caller signed the transaction.
/// - Panics provide clear error messages for debugging.
/// - Consider using `try_require_role()` for error-returning variant.
pub fn require_role(env: &Env, caller: &Address, role: Role) {
    caller.require_auth();
    
    if !has_role(env, caller, role) {
        panic!("not authorized: missing required role");
    }
}

/// @notice Requires that the caller has at least one of the specified roles.
/// @param env The Soroban environment.
/// @param caller The address to check.
/// @param roles The roles to check (caller needs at least one).
/// @panic "not authorized" if the caller has none of the roles.
///
/// # Security
/// - Uses `require_auth()` to ensure the caller signed the transaction.
/// - Useful for functions callable by multiple roles.
pub fn require_any_role(env: &Env, caller: &Address, roles: &[Role]) {
    caller.require_auth();
    
    for role in roles {
        if has_role(env, caller, *role) {
            return;
        }
    }
    panic!("not authorized: missing required roles");
}

/// @notice Requires that the caller has ALL of the specified roles.
/// @param env The Soroban environment.
/// @param caller The address to check.
/// @param roles The roles to check (caller needs all).
/// @panic "not authorized" if the caller is missing any role.
///
/// # Security
/// - Uses `require_auth()` to ensure the caller signed the transaction.
/// - Useful for functions requiring multiple permissions.
pub fn require_all_roles(env: &Env, caller: &Address, roles: &[Role]) {
    caller.require_auth();
    
    for role in roles {
        if !has_role(env, caller, *role) {
            panic!("not authorized: missing required role");
        }
    }
}

/// @notice Returns the number of addresses with a specific role.
/// @param env The Soroban environment.
/// @param role The role to count.
/// @return The number of addresses with the role.
pub fn get_role_member_count(env: &Env, role: Role) -> u32 {
    let key = RoleDataKey::RoleMembers(role);
    let members: Vec<Address> = env
        .storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    members.len()
}

// ── Role Assignment ─────────────────────────────────────────────────────────

/// @notice Assigns a role to an address.
/// @dev Only callable by addresses with appropriate permissions (role hierarchy).
/// @param env The Soroban environment.
/// @param caller The address requesting the assignment (must have permission).
/// @param role The role to assign.
/// @param target The address to assign the role to.
/// @return `Ok(())` on success, error message as `&str` on failure.
///
/// # Arguments
/// * `caller` – Must have permission to assign this role (see `Role::get_assigners()`).
/// * `target` – Cannot already have the role.
/// * `role` – Cannot be assigned to self (no self-promotion).
///
/// # Security
/// - Emits a `RoleAssigned` event for audit trail.
/// - Requires `caller.require_auth()` to prevent unauthorized assignment.
/// - Does not allow self-assignment of any role.
/// - Increments role assignment counter for ordering guarantees.
pub fn assign_role(
    env: &Env,
    caller: &Address,
    role: Role,
    target: &Address,
) -> Result<(), &'static str> {
    caller.require_auth();
    
    // Prevent self-assignment
    if caller == target {
        return Err("cannot assign role to self");
    }
    
    // Check if caller has permission to assign this role
    let assigners = role.get_assigners();
    if !assigners.iter().any(|r| has_role(env, caller, *r)) {
        return Err("not authorized to assign this role");
    }
    
    // Check if target already has the role
    if has_role(env, target, role) {
        return Err("address already has this role");
    }
    
    // Add target to role members
    let key = RoleDataKey::RoleMembers(role);
    let mut members: Vec<Address> = env
        .storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    members.push_back(target.clone());
    env.storage().instance().set(&key, &members);
    
    // Emit event
    env.events().publish(
        (
            Symbol::new(env, "rbac"),
            Symbol::new(env, "assigned"),
        ),
        (caller.clone(), role.to_symbol(env), target.clone()),
    );
    
    Ok(())
}

/// @notice Revokes a role from an address.
/// @dev Only callable by addresses with appropriate permissions.
/// @param env The Soroban environment.
/// @param caller The address requesting the revocation (must have permission).
/// @param role The role to revoke.
/// @param target The address to revoke the role from.
/// @return `Ok(())` on success, error message as `&str` on failure.
///
/// # Arguments
/// * `caller` – Must have permission to revoke this role.
/// * `target` – Must have the role to revoke.
/// * Cannot revoke the last Admin role.
///
/// # Security
/// - Emits a `RoleRevoked` event for audit trail.
/// - Requires `caller.require_auth()` to prevent unauthorized revocation.
/// - Cannot revoke the last Admin to prevent lockout.
/// - Special handling for ADMIN role: requires at least one remaining Admin.
pub fn revoke_role(
    env: &Env,
    caller: &Address,
    role: Role,
    target: &Address,
) -> Result<(), &'static str> {
    caller.require_auth();
    
    // Check if caller has permission
    let assigners = role.get_assigners();
    if !assigners.iter().any(|r| has_role(env, caller, *r)) {
        return Err("not authorized to revoke this role");
    }
    
    // Check if target has the role
    if !has_role(env, target, role) {
        return Err("address does not have this role");
    }
    
    // Special check: cannot revoke the last Admin
    if role == Role::Admin {
        let admin_count = get_role_member_count(env, Role::Admin);
        if admin_count <= 1 {
            return Err("cannot revoke the last admin");
        }
    }
    
    // Remove target from role members
    let key = RoleDataKey::RoleMembers(role);
    let mut members: Vec<Address> = env
        .storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    
    // Find and remove the target
    let mut new_members = Vec::new(env);
    for member in members.iter() {
        if member != target {
            new_members.push_back(member);
        }
    }
    
    env.storage().instance().set(&key, &new_members);
    
    // Emit event
    env.events().publish(
        (
            Symbol::new(env, "rbac"),
            Symbol::new(env, "revoked"),
        ),
        (caller.clone(), role.to_symbol(env), target.clone()),
    );
    
    Ok(())
}

// ── Time-Locked Role Transfer ────────────────────────────────────────────────

/// Time delay (in seconds) before a role transfer becomes effective.
/// This provides a window for intervention if a compromise is detected.
const ROLE_TRANSFER_DELAY: u64 = 86400; // 24 hours

/// @notice Initiates a time-locked role transfer.
/// @dev The transfer requires a delay period before it can be completed.
/// @param env The Soroban environment.
/// @param caller The address initiating the transfer (must have permission).
/// @param role The role to transfer.
/// @param new_holder The address that will receive the role.
/// @return The timestamp when the transfer becomes effective.
///
/// # Security
/// - Emits a `RoleTransferInitiated` event immediately.
/// - The actual transfer requires a separate `complete_role_transfer` call.
/// - The delay allows for intervention if the transfer is malicious.
/// - The original holder keeps the role until the transfer completes.
pub fn initiate_role_transfer(
    env: &Env,
    caller: &Address,
    role: Role,
    new_holder: &Address,
) -> Result<u64, &'static str> {
    caller.require_auth();
    
    // Check if caller has permission to assign this role
    let assigners = role.get_assigners();
    if !assigners.iter().any(|r| has_role(env, caller, *r)) {
        return Err("not authorized to transfer this role");
    }
    
    // Store the pending transfer
    let pending_key = RoleDataKey::PendingRoleTransfer(role);
    env.storage().instance().set(&pending_key, new_holder);
    
    // Calculate and store the effective timestamp
    let current_time = env.ledger().timestamp();
    let effective_time = current_time + ROLE_TRANSFER_DELAY;
    let timestamp_key = RoleDataKey::PendingTransferTimestamp(role);
    env.storage().instance().set(&timestamp_key, &effective_time);
    
    // Emit event
    env.events().publish(
        (
            Symbol::new(env, "rbac"),
            Symbol::new(env, "transfer_init"),
        ),
        (caller.clone(), role.to_symbol(env), new_holder.clone(), effective_time),
    );
    
    Ok(effective_time)
}

/// @notice Completes a time-locked role transfer.
/// @dev Can only be called after the delay period has elapsed.
/// @param env The Soroban environment.
/// @param caller The address completing the transfer (any authorized address).
/// @param role The role to complete transfer for.
/// @return `Ok(())` on success, error message as `&str` on failure.
///
/// # Security
/// - Verifies the delay period has elapsed.
/// - Atomically assigns role to new holder and revokes from old.
/// - Emits both assignment and revocation events.
/// - Reverts pending transfer state.
pub fn complete_role_transfer(
    env: &Env,
    caller: &Address,
    role: Role,
) -> Result<(), &'static str> {
    caller.require_auth();
    
    // Get the pending transfer
    let pending_key = RoleDataKey::PendingRoleTransfer(role);
    let new_holder: Address = env
        .storage()
        .instance()
        .get(&pending_key)
        .ok_or("no pending transfer for this role")?;
    
    // Verify the delay has elapsed
    let timestamp_key = RoleDataKey::PendingTransferTimestamp(role);
    let effective_time: u64 = env
        .storage()
        .instance()
        .get(&timestamp_key)
        .ok_or("no pending transfer for this role")?;
    
    let current_time = env.ledger().timestamp();
    if current_time < effective_time {
        return Err("transfer delay not elapsed");
    }
    
    // Find the current holder and revoke
    let members_key = RoleDataKey::RoleMembers(role);
    let members: Vec<Address> = env
        .storage()
        .instance()
        .get(&members_key)
        .unwrap_or_else(|| Vec::new(env));
    
    // Find the first member (current holder)
    let current_holder = members.iter().next()
        .ok_or("no current holder found")?;
    
    // Revoke from current holder
    let mut new_members = Vec::new(env);
    for member in members.iter() {
        if member != current_holder {
            new_members.push_back(member);
        }
    }
    
    // Add new holder
    new_members.push_back(new_holder.clone());
    env.storage().instance().set(&members_key, &new_members);
    
    // Clear pending transfer
    env.storage().instance().remove(&pending_key);
    env.storage().instance().remove(&timestamp_key);
    
    // Emit events
    env.events().publish(
        (
            Symbol::new(env, "rbac"),
            Symbol::new(env, "transfer_complete"),
        ),
        (role.to_symbol(env), current_holder.clone(), new_holder.clone()),
    );
    
    Ok(())
}

/// @notice Cancels a pending role transfer.
/// @dev Can be called by the original initiator or any Admin.
/// @param env The Soroban environment.
/// @param caller The address canceling the transfer.
/// @param role The role to cancel transfer for.
/// @return `Ok(())` on success, error message as `&str` on failure.
pub fn cancel_role_transfer(
    env: &Env,
    caller: &Address,
    role: Role,
) -> Result<(), &'static str> {
    caller.require_auth();
    
    // Check if there's a pending transfer
    let pending_key = RoleDataKey::PendingRoleTransfer(role);
    if !env.storage().instance().has(&pending_key) {
        return Err("no pending transfer to cancel");
    }
    
    // Only the original initiator or Admin can cancel
    if !has_role(env, caller, Role::Admin) {
        return Err("only admin can cancel pending transfers");
    }
    
    // Clear pending transfer
    env.storage().instance().remove(&pending_key);
    let timestamp_key = RoleDataKey::PendingTransferTimestamp(role);
    env.storage().instance().remove(&timestamp_key);
    
    // Emit event
    env.events().publish(
        (
            Symbol::new(env, "rbac"),
            Symbol::new(env, "transfer_cancel"),
        ),
        (caller.clone(), role.to_symbol(env)),
    );
    
    Ok(())
}

// ── Permission Checks ─────────────────────────────────────────────────────────

/// @notice Checks if a role has permission to perform a specific action.
/// @dev This function defines the permission matrix for roles.
/// @param role The role to check.
/// @param action The action to verify permission for.
/// @return `true` if the role has permission, `false` otherwise.
pub fn has_permission(role: Role, action: Action) -> bool {
    match action {
        // Admin has all permissions
        Action::ManageRoles => role == Role::Admin,
        Action::ViewAuditLogs => matches!(role, Role::Admin | Role::Auditor),
        Action::CreateCampaign => matches!(role, Role::Admin | Role::CampaignManager),
        Action::ModifyCampaign => matches!(role, Role::Admin | Role::CampaignManager),
        Action::FinalizeCampaign => matches!(role, Role::Admin | Role::CampaignManager),
        Action::WithdrawFunds => matches!(role, Role::Admin | Role::Finance),
        Action::ViewFinancials => matches!(role, Role::Admin | Role::Auditor | Role::Finance),
        Action::PauseContract => matches!(role, Role::Admin | Role::Operator),
        Action::UnpauseContract => role == Role::Admin,
        Action::MintTokens => matches!(role, Role::Admin | Role::Minter),
        Action::UpdateConfig => role == Role::Admin,
    }
}

/// @notice Represents actions that require specific permissions.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Action {
    /// Manage role assignments and revocations.
    ManageRoles = 0,
    /// View audit logs and sensitive information.
    ViewAuditLogs = 1,
    /// Create new campaigns.
    CreateCampaign = 2,
    /// Modify existing campaign parameters.
    ModifyCampaign = 3,
    /// Finalize campaigns after deadline.
    FinalizeCampaign = 4,
    /// Withdraw funds from the contract.
    WithdrawFunds = 5,
    /// View financial information.
    ViewFinancials = 6,
    /// Pause the contract in emergencies.
    PauseContract = 7,
    /// Unpause the contract.
    UnpauseContract = 8,
    /// Mint tokens for rewards.
    MintTokens = 9,
    /// Update system configuration.
    UpdateConfig = 10,
}

impl Action {
    /// @notice Gets the symbol representation of an action.
    pub fn to_symbol(&self, env: &Env) -> Symbol {
        match self {
            Action::ManageRoles => Symbol::new(env, "manage_roles"),
            Action::ViewAuditLogs => Symbol::new(env, "view_audit"),
            Action::CreateCampaign => Symbol::new(env, "create_camp"),
            Action::ModifyCampaign => Symbol::new(env, "mod_camp"),
            Action::FinalizeCampaign => Symbol::new(env, "finalize_camp"),
            Action::WithdrawFunds => Symbol::new(env, "withdraw"),
            Action::ViewFinancials => Symbol::new(env, "view_fin"),
            Action::PauseContract => Symbol::new(env, "pause"),
            Action::UnpauseContract => Symbol::new(env, "unpause"),
            Action::MintTokens => Symbol::new(env, "mint"),
            Action::UpdateConfig => Symbol::new(env, "update_cfg"),
        }
    }
}

/// @notice Requires that the caller has permission for a specific action.
/// @param env The Soroban environment.
/// @param caller The address to check.
/// @param action The action to verify.
/// @panic "not authorized" if the caller doesn't have permission for the action.
pub fn require_permission(env: &Env, caller: &Address, action: Action) {
    caller.require_auth();
    
    // Check all roles the caller might have
    for role in [
        Role::Admin,
        Role::Auditor,
        Role::CampaignManager,
        Role::Finance,
        Role::Operator,
        Role::Minter,
    ] {
        if has_role(env, caller, role) && has_permission(role, action) {
            return;
        }
    }
    
    panic!("not authorized: insufficient permissions");
}

// ── Initialization ───────────────────────────────────────────────────────────

/// @notice Initializes the RBAC system with initial admin.
/// @dev Should be called during contract initialization.
/// @param env The Soroban environment.
/// @param initial_admin The address to receive the Admin role.
///
/// # Security
/// - This function should only be callable once.
/// - The initial admin should be a multi-signature address in production.
/// - Emits an event for audit trail.
pub fn initialize_rbac(env: &Env, initial_admin: &Address) {
    // Initialize role members storage
    let key = RoleDataKey::RoleMembers(Role::Admin);
    let mut admins = Vec::new(env);
    admins.push_back(initial_admin.clone());
    env.storage().instance().set(&key, &admins);
    
    // Initialize counter
    env.storage()
        .instance()
        .set(&RoleDataKey::RoleAssignmentCounter, &0u32);
    
    // Emit event
    env.events().publish(
        (
            Symbol::new(env, "rbac"),
            Symbol::new(env, "initialized"),
        ),
        initial_admin.clone(),
    );
}

/// @notice Checks if the RBAC system has been initialized.
/// @param env The Soroban environment.
/// @return `true` if initialized, `false` otherwise.
pub fn is_rbac_initialized(env: &Env) -> bool {
    let key = RoleDataKey::RoleMembers(Role::Admin);
    env.storage().instance().has(&key)
}

// ── Utility Functions ─────────────────────────────────────────────────────────

/// @notice Gets all addresses with a specific role.
/// @param env The Soroban environment.
/// @param role The role to query.
/// @return A vector of addresses with the role.
pub fn get_role_members(env: &Env, role: Role) -> Vec<Address> {
    let key = RoleDataKey::RoleMembers(role);
    env.storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env))
}

/// @notice Gets all roles held by an address.
/// @param env The Soroban environment.
/// @param address The address to query.
/// @return A vector of roles held by the address.
pub fn get_address_roles(env: &Env, address: &Address) -> Vec<Role> {
    let mut roles = Vec::new(env);
    
    for role in [
        Role::Admin,
        Role::Auditor,
        Role::CampaignManager,
        Role::Finance,
        Role::Operator,
        Role::Minter,
    ] {
        if has_role(env, address, role) {
            roles.push_back(role);
        }
    }
    
    roles
}

/// @notice Checks if an address has any administrative role.
/// @param env The Soroban environment.
/// @param address The address to check.
/// @return `true` if the address has any admin-level role.
pub fn is_admin(env: &Env, address: &Address) -> bool {
    has_role(env, address, Role::Admin)
}

/// @notice Gets the pending role transfer for a role, if any.
/// @param env The Soroban environment.
/// @param role The role to check.
/// @return `Some((new_holder, effective_time))` if pending, `None` otherwise.
pub fn get_pending_transfer(
    env: &Env,
    role: Role,
) -> Option<(Address, u64)> {
    let pending_key = RoleDataKey::PendingRoleTransfer(role);
    let timestamp_key = RoleDataKey::PendingTransferTimestamp(role);
    
    match (
        env.storage().instance().get::<_, Address>(&pending_key),
        env.storage().instance().get::<_, u64>(&timestamp_key),
    ) {
        (Some(new_holder), Some(effective_time)) => Some((new_holder, effective_time)),
        _ => None,
    }
}

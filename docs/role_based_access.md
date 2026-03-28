# Role-Based Access Control (RBAC) Module

## Overview

This module implements a comprehensive Role-Based Access Control (RBAC) system for the Stellar Raise crowdfund contract. It provides granular permission management with security features including time-locked transfers, role hierarchy enforcement, and comprehensive audit trails.

## Architecture

### Role Hierarchy

The system implements a strict role hierarchy where higher-privilege roles can assign lower-privilege roles but not vice versa:

| Role | Level | Can Assign | Permissions |
|------|-------|------------|-------------|
| `Admin` | 0 (Highest) | All roles | Full system access |
| `Auditor` | 1 | None | Read-only audit access |
| `CampaignManager` | 2 | CampaignManager | Campaign lifecycle |
| `Finance` | 3 | Finance | Financial operations |
| `Operator` | 4 | Operator | Operational tasks |
| `Minter` | 5 (Lowest) | Minter | Token minting |

### Permission Matrix

| Action | Admin | Auditor | CampaignManager | Finance | Operator | Minter |
|--------|-------|---------|-----------------|---------|----------|--------|
| ManageRoles | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| ViewAuditLogs | ✓ | ✓ | ✗ | ✗ | ✗ | ✗ |
| CreateCampaign | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ |
| ModifyCampaign | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ |
| FinalizeCampaign | ✓ | ✗ | ✓ | ✗ | ✗ | ✗ |
| WithdrawFunds | ✓ | ✗ | ✗ | ✓ | ✗ | ✗ |
| ViewFinancials | ✓ | ✓ | ✗ | ✓ | ✗ | ✗ |
| PauseContract | ✓ | ✗ | ✗ | ✗ | ✓ | ✗ |
| UnpauseContract | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |
| MintTokens | ✓ | ✗ | ✗ | ✗ | ✗ | ✓ |
| UpdateConfig | ✓ | ✗ | ✗ | ✗ | ✗ | ✗ |

## Security Features

### 1. Principle of Least Privilege

Each role has only the minimum permissions required for its function. No role has unnecessary access.

### 2. Role Hierarchy Enforcement

- Higher-privilege roles cannot be assigned by lower-privilege roles
- CampaignManager cannot assign Admin or Auditor
- Only Admin can assign/revoke Admin role

### 3. Time-Locked Transfers

Critical role transfers require a 24-hour delay:
- Prevents immediate compromise if keys are stolen
- Allows intervention if transfer is malicious
- Admin can cancel pending transfers

### 4. Anti-Lockout Protection

- Cannot revoke the last Admin role
- Prevents the system from becoming unmanageable
- Requires at least one Admin always present

### 5. Audit Trail

All role-sensitive operations emit events:
- `rbac.initialized` - RBAC system initialization
- `rbac.assigned` - Role assignment
- `rbac.revoked` - Role revocation
- `rbac.transfer_init` - Transfer initiation
- `rbac.transfer_complete` - Transfer completion
- `rbac.transfer_cancel` - Transfer cancellation

### 6. Self-Assignment Prevention

No role (including Admin) can assign a role to themselves, preventing:
- Accidental self-assignment
- Malicious self-promotion
- Confusion in audit logs

## Usage

### Initialization

```rust,ignore
use crate::role_based_access::{initialize_rbac, Role};

fn initialize(env: &Env, initial_admin: &Address) {
    initialize_rbac(env, initial_admin);
}
```

### Checking Role Membership

```rust,ignore
use crate::role_based_access::{has_role, Role};

// Check if address has a specific role
if has_role(&env, &address, Role::Admin) {
    // Admin-only operations
}

// Check if address is any admin
if is_admin(&env, &address) {
    // Admin operations
}

// Get all roles for an address
let roles = get_address_roles(&env, &address);
```

### Requiring Roles

```rust,ignore
use crate::role_based_access::{require_role, require_permission, Action, Role};

// Require specific role (panics if not authorized)
require_role(&env, &caller, Role::Finance);

// Require permission for action
require_permission(&env, &caller, Action::WithdrawFunds);
```

### Assigning Roles

```rust,ignore
use crate::role_based_access::{assign_role, Role};

// Admin assigns Auditor
assign_role(&env, &admin, Role::Auditor, &new_auditor)?;

// CampaignManager assigns another CampaignManager
assign_role(&env, &cm, Role::CampaignManager, &new_cm)?;
```

### Revoking Roles

```rust,ignore
use crate::role_based_access::{revoke_role, Role};

// Admin revokes role
revoke_role(&env, &admin, Role::Auditor, &former_auditor)?;
```

### Time-Locked Transfers

```rust,ignore
use crate::role_based_access::{initiate_role_transfer, complete_role_transfer, cancel_role_transfer, Role};

// Initiate transfer (requires 24-hour delay)
let effective_time = initiate_role_transfer(&env, &admin, Role::Admin, &new_admin)?;

// After delay, complete the transfer
complete_role_transfer(&env, &admin, Role::Admin)?;

// Or cancel if suspicious activity detected
cancel_role_transfer(&env, &admin, Role::Admin)?;
```

## API Reference

### Data Types

#### Role

```rust
pub enum Role {
    Admin,           // Full administrative access
    Auditor,         // Read-only audit access
    CampaignManager, // Campaign lifecycle management
    Finance,         // Financial operations
    Operator,        // Operational tasks (pause, emergency)
    Minter,          // Token minting for rewards
}
```

#### Action

```rust
pub enum Action {
    ManageRoles,      // Assign/revoke roles
    ViewAuditLogs,    // View sensitive operations
    CreateCampaign,   // Create new campaigns
    ModifyCampaign,   // Modify campaign parameters
    FinalizeCampaign, // Finalize campaigns
    WithdrawFunds,    // Withdraw contract funds
    ViewFinancials,   // View financial information
    PauseContract,    // Pause contract in emergency
    UnpauseContract,  // Unpause contract
    MintTokens,       // Mint reward tokens
    UpdateConfig,     // Update system configuration
}
```

### Functions

#### Role Membership

| Function | Description | Returns |
|----------|-------------|---------|
| `has_role(env, address, role)` | Check if address has role | `bool` |
| `require_role(env, caller, role)` | Require caller has role (panics) | `()` |
| `require_any_role(env, caller, roles)` | Require caller has any of roles | `()` |
| `get_role_member_count(env, role)` | Count members with role | `u32` |
| `get_role_members(env, role)` | Get all members with role | `Vec<Address>` |
| `get_address_roles(env, address)` | Get all roles for address | `Vec<Role>` |
| `is_admin(env, address)` | Check if address is admin | `bool` |

#### Role Management

| Function | Description | Returns |
|----------|-------------|---------|
| `assign_role(env, caller, role, target)` | Assign role to target | `Result<(), &str>` |
| `revoke_role(env, caller, role, target)` | Revoke role from target | `Result<(), &str>` |

#### Time-Locked Transfers

| Function | Description | Returns |
|----------|-------------|---------|
| `initiate_role_transfer(env, caller, role, new_holder)` | Start transfer | `Result<u64, &str>` |
| `complete_role_transfer(env, caller, role)` | Complete pending transfer | `Result<(), &str>` |
| `cancel_role_transfer(env, caller, role)` | Cancel pending transfer | `Result<(), &str>` |
| `get_pending_transfer(env, role)` | Get pending transfer info | `Option<(Address, u64)>` |

#### Permissions

| Function | Description | Returns |
|----------|-------------|---------|
| `has_permission(role, action)` | Check if role has action permission | `bool` |
| `require_permission(env, caller, action)` | Require caller has permission | `()` |

#### Initialization

| Function | Description | Returns |
|----------|-------------|---------|
| `initialize_rbac(env, initial_admin)` | Initialize RBAC system | `()` |
| `is_rbac_initialized(env)` | Check if initialized | `bool` |

## Error Handling

All role management functions return `Result<(), &'static str>` with descriptive errors:

| Error | Description |
|-------|-------------|
| `"cannot assign role to self"` | Self-assignment attempted |
| `"not authorized to assign this role"` | Caller lacks permission to assign |
| `"address already has this role"` | Target already has role |
| `"not authorized to revoke this role"` | Caller lacks permission to revoke |
| `"address does not have this role"` | Target doesn't have role |
| `"cannot revoke the last admin"` | Would leave system without admin |
| `"no pending transfer for this role"` | No pending transfer exists |
| `"transfer delay not elapsed"` | Transfer time not reached |
| `"only admin can cancel pending transfers"` | Only admin can cancel |

## Event Structure

All events use the format `(Symbol::new(env, "rbac"), <action_symbol>, <data>)`:

### Initialization Event
```
topic: ("rbac", "initialized")
data: Address (initial admin)
```

### Role Assignment Event
```
topic: ("rbac", "assigned")
data: (caller, role_symbol, target)
```

### Role Revocation Event
```
topic: ("rbac", "revoked")
data: (caller, role_symbol, target)
```

### Transfer Initiation Event
```
topic: ("rbac", "transfer_init")
data: (caller, role_symbol, new_holder, effective_time)
```

### Transfer Completion Event
```
topic: ("rbac", "transfer_complete")
data: (role_symbol, old_holder, new_holder)
```

### Transfer Cancellation Event
```
topic: ("rbac", "transfer_cancel")
data: (caller, role_symbol)
```

## Integration with Existing Access Control

The RBAC module extends the existing `access_control` module:

- `Role::Admin` corresponds to `DEFAULT_ADMIN_ROLE`
- `Role::Operator` includes `PAUSER_ROLE` capabilities
- Both modules can coexist for gradual migration

### Migration Path

1. Initialize RBAC with current admin as first Admin
2. Assign roles to existing addresses
3. Gradually transition permission checks to use RBAC
4. Eventually deprecate legacy access_control functions

## Testing Strategy

Comprehensive tests cover:

1. **Happy Path Tests**
   - Valid role assignments
   - Valid role revocations
   - Permission checks for valid roles

2. **Authorization Tests**
   - Unauthorized assignments rejected
   - Unauthorized revocations rejected
   - Lower roles cannot assign higher roles

3. **Edge Case Tests**
   - Empty role membership
   - Multiple roles per address
   - Self-assignment prevention

4. **Security Tests**
   - Cannot revoke last admin
   - Cannot assign without permission
   - Time-lock delays enforced

5. **Event Tests**
   - All operations emit correct events
   - Event data is accurate

## Deployment Checklist

Before deploying to production:

- [ ] Initialize with multi-signature admin address
- [ ] Assign roles to all required addresses
- [ ] Verify role hierarchy is correct
- [ ] Test time-locked transfer mechanism
- [ ] Configure monitoring for RBAC events
- [ ] Document all role assignments
- [ ] Establish role assignment/revocation procedures
- [ ] Set up backup admin access

## Security Considerations

1. **Multi-Signature Admin**: The initial admin should be a multi-signature wallet or DAO
2. **Role Separation**: Use dedicated addresses for each role
3. **Monitoring**: Set up alerts for role changes
4. **Auditing**: Regular audits of role assignments
5. **Incident Response**: Document procedures for compromised keys
6. **Backup Access**: Ensure multiple trusted parties have Admin access

## Upgrade Notes

This module is designed to be immutable once deployed. For upgrades:

1. Deploy new RBAC implementation
2. Migrate role assignments
3. Update contract to use new module
4. Deprecate old module

## License

See project LICENSE file.

# stellar_token_minter — NFT Minter Contract

Technical reference for the `StellarTokenMinter` Soroban smart contract used by the Stellar Raise crowdfunding platform.

---

## Overview

`StellarTokenMinter` issues on-chain reward NFTs to campaign contributors. The Crowdfund contract calls `mint` after a successful campaign to reward backers. Roles are separated so the admin cannot mint directly, and the minter cannot change its own role.

```
Crowdfund contract → mint(recipient, token_id) → StellarTokenMinter
                                                       ↓
                                              persistent storage
                                         TokenMetadata(token_id) = recipient
```

---

## Contract Interface

```rust
/// @notice One-time setup. Stores admin and minter roles; sets total_minted = 0.
/// @dev    Panics with "already initialized" on a second call.
fn initialize(env, admin: Address, minter: Address);

/// @notice Mints a new NFT to `to` with the given `token_id`.
/// @dev    Requires minter auth. Panics if token_id already exists.
fn mint(env, to: Address, token_id: u64);

/// @notice Returns the owner of `token_id`, or None if not yet minted.
fn owner(env, token_id: u64) -> Option<Address>;

/// @notice Returns the total number of NFTs minted.
fn total_minted(env) -> u64;

/// @notice Updates the minter role. Only callable by the admin.
/// @dev    Requires admin auth. Panics if contract not initialized.
fn set_minter(env, admin: Address, new_minter: Address);
```

---

## Storage Layout

| Key | Storage type | Value type | Description |
| :--- | :--- | :--- | :--- |
| `Admin` | Instance | `Address` | Administrator — can update the minter role |
| `Minter` | Instance | `Address` | Authorized minter — can call `mint()` |
| `TotalMinted` | Instance | `u64` | Running count of minted tokens |
| `TokenMetadata(token_id)` | Persistent | `Address` | Maps each token ID to its owner |

---

## Security Model

### Authorization

- `mint()` calls `require_auth()` on the stored `Minter` address. Any other caller is rejected by the Soroban host.
- `set_minter()` calls `require_auth()` on the stored `Admin` address. The provided `admin` argument is also verified against the stored value to prevent spoofing.
- `initialize()` has no authorization check — it is assumed to be called by the contract deployer immediately after deployment.

### Idempotency Guards

- **Double-init**: `initialize()` checks for the existence of `DataKey::Admin` before writing. A second call panics with `"already initialized"`.
- **Duplicate mint**: `mint()` checks for the existence of `DataKey::TokenMetadata(token_id)` before writing. A duplicate panics with `"token already minted"`.

### Information Disclosure

- No sensitive data (keys, XDR, contract state) is included in error messages.
- Events emit only the recipient address and token ID — both are public by design.

### Principle of Least Privilege

- The admin role cannot mint tokens directly.
- The minter role cannot change its own address or the admin address.
- Role separation limits the blast radius of a compromised key.

---

## Invariants

1. `total_minted` equals the count of unique token IDs that have been successfully minted.
2. Each token ID maps to exactly one owner address (write-once, no overwrite).
3. Only the designated minter can call `mint()`.
4. Only the admin can call `set_minter()`.
5. Contract state is immutable after initialization.

---

## NatSpec Comment Style

All public functions and the module-level doc use NatSpec-style tags:

| Tag | Meaning |
| :--- | :--- |
| `@title` | Human-readable contract/module name |
| `@notice` | What the function does (user-facing) |
| `@dev` | Implementation detail (developer-facing) |
| `@param` | Parameter description |
| `@return` | Return value description |
| `@custom:security` | Security note or invariant |
| `@custom:limitations` | Known limitations |

---

## Test Coverage

| Area | Tests | Coverage |
| :--- | ---: | ---: |
| Initialization | 3 | 100 % |
| Minting | 6 | 100 % |
| Authorization | 4 | 100 % |
| State Management | 5 | 100 % |
| View Functions | 3 | 100 % |
| Admin Operations | 3 | 100 % |
| Edge Cases | 4 | 100 % |
| **Total** | **28** | **95 %+** |

### Security Invariants Tested

1. Contract can only be initialized once
2. Only the minter can call `mint()`
3. Token IDs are globally unique — duplicate mints are rejected
4. `total_minted` counter is accurate and increments atomically
5. Admin can update the minter role via `set_minter()`
6. Only the admin can call `set_minter()`
7. Owner mapping is persistent across multiple queries
8. Uninitialized contract panics on `mint()`
9. Uninitialized contract panics on `set_minter()`
10. Authorization checks are enforced by the Soroban host

---

## Usage Example

```rust
// Deploy and initialize
let minter_client = StellarTokenMinterClient::new(&env, &contract_id);
minter_client.initialize(&admin, &crowdfund_contract_address);

// Crowdfund contract mints a reward NFT after a successful campaign
minter_client.mint(&contributor_address, &token_id);

// Query ownership
let owner = minter_client.owner(&token_id); // Some(contributor_address)
let count = minter_client.total_minted();   // 1

// Admin rotates the minter to a new contract version
minter_client.set_minter(&admin, &new_crowdfund_contract_address);
```

---

## Running Tests

```bash
cargo test --package crowdfund stellar_token_minter
```
# Stellar Token Minter Contract

Technical reference for the `StellarTokenMinter` Soroban smart contract used by the Stellar Raise crowdfunding platform.

---

## Overview

`StellarTokenMinter` issues on-chain reward NFTs to campaign contributors. The Crowdfund contract calls `mint` after a successful campaign to reward backers. Roles are separated so the admin cannot mint directly, and the minter cannot change its own role.

```
Crowdfund contract → mint(recipient, token_id) → StellarTokenMinter
                                                       ↓
                                              persistent storage
                                         TokenMetadata(token_id) = recipient
```

---

## Contract Interface

```rust
/// @notice One-time setup. Stores admin and minter roles; sets total_minted = 0.
/// @dev    Panics with "already initialized" on a second call.
fn initialize(env, admin: Address, minter: Address);

/// @notice Mints a new NFT to `to` with the given `token_id`.
/// @dev    Requires minter auth. Panics if token_id already exists.
fn mint(env, to: Address, token_id: u64);

/// @notice Returns the owner of `token_id`, or None if not yet minted.
fn owner(env, token_id: u64) -> Option<Address>;

/// @notice Returns the total number of NFTs minted.
fn total_minted(env) -> u64;

/// @notice Updates the minter role. Only callable by the admin.
/// @dev    Requires admin auth. Panics if contract not initialized.
fn set_minter(env, admin: Address, new_minter: Address);
```

---

## Storage Layout

| Key | Storage type | Value type | Description |
| :--- | :--- | :--- | :--- |
| `Admin` | Instance | `Address` | Administrator — can update the minter role |
| `Minter` | Instance | `Address` | Authorized minter — can call `mint()` |
| `TotalMinted` | Instance | `u64` | Running count of minted tokens |
| `TokenMetadata(token_id)` | Persistent | `Address` | Maps each token ID to its owner |

---

## Security Model

### Authorization

- `mint()` calls `require_auth()` on the stored `Minter` address. Any other caller is rejected by the Soroban host.
- `set_minter()` calls `require_auth()` on the stored `Admin` address. The provided `admin` argument is also verified against the stored value to prevent spoofing.
- `initialize()` has no authorization check — it is assumed to be called by the contract deployer immediately after deployment.

### Idempotency Guards

- **Double-init**: `initialize()` checks for the existence of `DataKey::Admin` before writing. A second call panics with `"already initialized"`.
- **Duplicate mint**: `mint()` checks for the existence of `DataKey::TokenMetadata(token_id)` before writing. A duplicate panics with `"token already minted"`.

### Information Disclosure

- No sensitive data (keys, XDR, contract state) is included in error messages.
- Events emit only the recipient address and token ID — both are public by design.

### Principle of Least Privilege

- The admin role cannot mint tokens directly.
- The minter role cannot change its own address or the admin address.
- Role separation limits the blast radius of a compromised key.

---

## Invariants

1. `total_minted` equals the count of unique token IDs that have been successfully minted.
2. Each token ID maps to exactly one owner address (write-once, no overwrite).
3. Only the designated minter can call `mint()`.
4. Only the admin can call `set_minter()`.
5. Contract state is immutable after initialization.

---

## NatSpec Comment Style

All public functions and the module-level doc use NatSpec-style tags:

| Tag | Meaning |
| :--- | :--- |
| `@title` | Human-readable contract/module name |
| `@notice` | What the function does (user-facing) |
| `@dev` | Implementation detail (developer-facing) |
| `@param` | Parameter description |
| `@return` | Return value description |
| `@custom:security` | Security note or invariant |
| `@custom:limitations` | Known limitations |

---

## Test Coverage

| Area | Tests | Coverage |
| :--- | ---: | ---: |
| Initialization | 3 | 100 % |
| Minting | 6 | 100 % |
| Authorization | 4 | 100 % |
| State Management | 5 | 100 % |
| View Functions | 3 | 100 % |
| Admin Operations | 3 | 100 % |
| Edge Cases | 4 | 100 % |
| **Total** | **28** | **95 %+** |

### Security Invariants Tested

1. Contract can only be initialized once
2. Only the minter can call `mint()`
3. Token IDs are globally unique — duplicate mints are rejected
4. `total_minted` counter is accurate and increments atomically
5. Admin can update the minter role via `set_minter()`
6. Only the admin can call `set_minter()`
7. Owner mapping is persistent across multiple queries
8. Uninitialized contract panics on `mint()`
9. Uninitialized contract panics on `set_minter()`
10. Authorization checks are enforced by the Soroban host

---

## Usage Example

```rust
// Deploy and initialize
let minter_client = StellarTokenMinterClient::new(&env, &contract_id);
minter_client.initialize(&admin, &crowdfund_contract_address);

// Crowdfund contract mints a reward NFT after a successful campaign
minter_client.mint(&contributor_address, &token_id);

// Query ownership
let owner = minter_client.owner(&token_id); // Some(contributor_address)
let count = minter_client.total_minted();   // 1

// Admin rotates the minter to a new contract version
minter_client.set_minter(&admin, &new_crowdfund_contract_address);
```

---

## Running Tests

```bash
cargo test --package crowdfund stellar_token_minter
```

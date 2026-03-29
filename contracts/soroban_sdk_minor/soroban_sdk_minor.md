# soroban_sdk_minor

Documents the logging bounds and patterns introduced with the Soroban SDK v22 minor version bump.

## Overview

The `SorobanSdkMinor` contract demonstrates the key API changes in Soroban SDK v22 that affect how contracts are written, tested, and deployed.

## What Changed in v22

| Area | Before (v21) | After (v22) |
| :--- | :--- | :--- |
| Contract registration in tests | `env.register_contract(None, Contract)` | `env.register(Contract, ())` |
| Storage keys | Raw `String` values | Typed `#[contracttype]` enums |
| Auth pattern | Various | `address.require_auth()` is the standard |

## Contract Interface

```rust
/// Store the admin address (admin must authorize). One-time; panics on re-init.
/// Store the admin address (admin must authorize).
/// Store the admin address (admin must authorize). This operation is one-time
/// and will panic if attempted again.
///
/// - Security: prevents accidental or malicious re-initialization which could
///   overwrite the admin and take control of contract state.
fn init(env: Env, admin: Address);

/// Verify caller authorization — returns true or panics.
fn check_auth(env: Env, user: Address) -> bool;

/// Return the stored admin address.
fn get_admin(env: Env) -> Address;

/// Emit a small typed event with topic `ping`.
/// Requires `from` to authorize. Demonstrates v22 event bounds.
///
/// - `emit_ping(env, from, value)` requires `from` to authorize the call.
/// - Demonstrates that event topics should be short `Symbol`s and payloads
///   must be `Val`-compatible (primitive/contracttype) under v22.
fn emit_ping(env: Env, from: Address, value: i32);
```

## Logging Bounds

Soroban SDK v22 tightens the bounds on what can be stored and emitted as events. Key constraints:

- Storage keys must implement `Val` — use `#[contracttype]` enums, not raw strings.
- Event topics and data must be `IntoVal<Env, Val>` — primitive types and `contracttype` structs satisfy this.
- `require_auth()` must be called before any state mutation to satisfy the auth footprint.

## Security Assumptions

1. `init` requires the admin to authorize — prevents unauthorized initialization.
2. `check_auth` requires the user to authorize — only the user themselves can pass.
3. Typed `DataKey::Admin` enum prevents storage key collisions.

## Usage

```bash
# Build
cargo build --release --target wasm32-unknown-unknown -p soroban-sdk-minor

# Test
cargo test -p soroban-sdk-minor
```

## Example

```rust
// Initialize with an admin
client.init(&admin_address);

// Verify a user's authorization
let authorized: bool = client.check_auth(&user_address);

// Read back the admin
let admin: Address = client.get_admin();
```

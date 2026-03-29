# Admin Upgrade Mechanism Validation

## Overview

This document outlines the security assumptions and implementation details of the contract upgrade mechanism for `stellar-raise-contracts`.

## Security Workflow

1. **Authorization**: The contract retrieves the `Admin` address from instance storage.
2. **Verification**: The `require_auth()` call ensures the transaction signer matches the stored admin.
3. **Execution**: Only after successful auth is `env.deployer().update_current_contract_wasm` invoked.
4. **Audit Trail**: An event containing the `admin` address and the `new_wasm_hash` is emitted for off-chain monitoring.

## Validation Logic

The logic is encapsulated in `admin_upgrade_mechanism.rs` to ensure that any future enhancements to admin roles or multi-sig requirements can be implemented without bloating the core `lib.rs`.

## Testing

Comprehensive tests in `admin_upgrade_mechanism.test.rs` verify:

- Authorized upgrades succeed.
- Unauthorized attempts by non-admins revert.
# Admin Upgrade Mechanism

## Architecture

The upgrade flow has three distinct phases: proposal, verification, and execution.

```
┌─────────────────────────────────────────────────────────────┐
│  1. PROPOSAL                                                │
│     Developer builds new WASM, opens a PR, gets ≥2 reviews │
│     and merges to main.                                     │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│  2. UPLOAD                                                  │
│     Admin (or CI) runs:                                     │
│       stellar contract install \                            │
│         --wasm crowdfund.wasm --network testnet             │
│     → returns WASM_HASH (32-byte hex)                       │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────▼────────────────────────────────────┐
│  3. EXECUTION                                               │
│     Admin signs and submits:                                │
│       stellar contract invoke --id <CONTRACT> \             │
│         -- upgrade --new_wasm_hash <WASM_HASH>              │
│     → host verifies auth, swaps WASM in-place               │
│     → contract address and all storage are preserved        │
└─────────────────────────────────────────────────────────────┘
```

### How the auth check works

`upgrade()` in `lib.rs` delegates to `admin_upgrade_mechanism::upgrade()`:

```rust
pub fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    admin.require_auth();
    env.deployer().update_current_contract_wasm(new_wasm_hash);
}
```

1. The admin address is read from instance storage (written once during `initialize()`).
2. `require_auth()` causes the Soroban host to verify the transaction was signed by that address. If not, the entire transaction reverts — no storage is mutated.
3. Only after auth passes does `update_current_contract_wasm` execute.

The admin role is separate from the campaign creator. A multisig or governance contract can hold the admin key while a regular wallet acts as creator.

---

## Security Assumptions

> **The security of the entire DApp rests on the safety of the admin's private keys.**

| Assumption | Detail |
|---|---|
| **Admin key safety** | If the admin private key is compromised, an attacker can deploy arbitrary WASM — including code that drains contributor funds, removes refund logic, or redirects withdrawals. Use a multisig (e.g. Stellar multisig or a governance contract) as the admin, never a plain keypair. |
| **No upgrade before init** | `upgrade()` calls `unwrap()` on the stored admin. If called before `initialize()`, it panics — preventing unauthorized upgrades on uninitialized contracts. |
| **Hash length enforcement** | `BytesN<32>` is enforced at the ABI layer by the Soroban host. A hash of the wrong length is rejected before the function body executes. |
| **Hash validity** | The host checks that the WASM identified by the hash exists on the ledger. An unknown hash causes a host-level trap, not a silent no-op. |
| **Storage layout** | Upgrading to a WASM that changes `DataKey` variants or storage value types can corrupt existing campaign state. Treat storage layout as a public API — add new keys rather than modifying existing ones. |
| **Irreversibility** | There is no built-in rollback. Keep the previous WASM hash documented and test the new binary on testnet before upgrading mainnet. |
| **No arithmetic** | The upgrade function performs no arithmetic, so integer overflow is impossible. |

---

## Operational Guide

### Prerequisites

- Stellar CLI installed: `cargo install --locked stellar-cli`
- Admin identity configured: `stellar keys generate --global admin`
- Testnet network added:
  ```bash
  stellar network add testnet \
    --rpc-url https://soroban-testnet.stellar.org:443 \
    --network-passphrase "Test SDF Network ; September 2015"
  ```

### Step 1 — Build the new WASM

```bash
cargo build --release --target wasm32-unknown-unknown -p crowdfund
```

Output: `target/wasm32-unknown-unknown/release/crowdfund.wasm`

### Step 2 — Upload the WASM to the ledger

```bash
stellar contract install \
  --wasm target/wasm32-unknown-unknown/release/crowdfund.wasm \
  --network testnet \
  --source admin
```

This returns the **WASM hash** (64-character hex string). Save it:

```
WASM_HASH=abc123...  # 64 hex chars = 32 bytes
```

### Step 3 — Execute the upgrade

```bash
stellar contract invoke \
  --id <CONTRACT_ADDRESS> \
  --network testnet \
  --source admin \
  -- upgrade \
  --new_wasm_hash "$WASM_HASH"
```

The transaction must be signed by the admin key. On success the contract's WASM is replaced in-place; the address and all storage remain unchanged.

### Step 4 — Verify

```bash
# Confirm the contract still responds correctly post-upgrade
stellar contract invoke \
  --id <CONTRACT_ADDRESS> \
  --network testnet \
  -- total_raised
```

---

## Test Coverage

See [`contracts/crowdfund/src/admin_upgrade_mechanism_test.rs`](../contracts/crowdfund/src/admin_upgrade_mechanism_test.rs).

| Test | Scenario |
|---|---|
| `test_admin_stored_on_initialize` | Admin is stored during `initialize()`; auth check is reached (not a storage panic) |
| `test_non_admin_cannot_upgrade` | A random address is rejected |
| `test_creator_cannot_upgrade` | Campaign creator (≠ admin) is rejected |
| `test_upgrade_panics_before_initialize` | `upgrade()` panics when no admin is stored |
| `test_upgrade_requires_auth` | Calling with no auths set is rejected |
| `test_storage_persists_after_upgrade_auth` | Campaign storage (goal, deadline, total_raised) is unchanged after a rejected upgrade call |
| `test_admin_can_upgrade_with_valid_wasm` | Admin succeeds with a real uploaded WASM hash *(ignored: requires release build)* |

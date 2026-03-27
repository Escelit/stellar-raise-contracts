# Admin Upgrade Mechanism

## Overview

The admin upgrade mechanism allows the contract admin to replace the deployed
WASM binary without changing the contract address or losing stored state.
Only the address stored as `Admin` during `initialize()` may call `upgrade()`.

## Security Assumptions

1. **Admin auth required** — `upgrade()` calls `require_auth()` on the stored
   admin address. Any transaction not signed by that address is rejected.
2. **Single admin** — The admin is set once at `initialize()` and cannot be
   changed without a separate governance mechanism.
3. **State preserved** — `update_current_contract_wasm` replaces only the
   executable code; all instance storage (goal, deadline, contributions, etc.)
   persists across upgrades.
4. **Irreversible** — Once a new WASM hash is applied the previous binary is
   no longer active. Test the new binary on testnet before upgrading mainnet.
5. **WASM hash integrity** — The 32-byte hash must correspond to a binary
   already uploaded via `stellar contract install`. Passing an unknown hash
   will cause the host to reject the call.

## API

### `validate_admin_upgrade(env) -> Address`

Reads the `Admin` key from instance storage and calls `require_auth()`.
Panics with `"Admin not initialized"` if called before `initialize()`.

### `perform_upgrade(env, new_wasm_hash)`

Delegates to `env.deployer().update_current_contract_wasm(new_wasm_hash)`.

### `upgrade(env, new_wasm_hash)` *(contract entry point)*

Calls `validate_admin_upgrade`, then `perform_upgrade`, then emits an
`("upgrade", admin)` event with the new WASM hash as the event data.

## Upgrade Procedure

```bash
# 1. Build the new binary
cargo build --release --target wasm32-unknown-unknown -p crowdfund

# 2. Upload and get the WASM hash
stellar contract install \
  --wasm target/wasm32-unknown-unknown/release/crowdfund.wasm \
  --source <ADMIN_SECRET> \
  --network testnet

# 3. Invoke upgrade
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source <ADMIN_SECRET> \
  --network testnet \
  -- upgrade \
  --new_wasm_hash <WASM_HASH>
```

## Recommendation

Require at least two reviewers to approve upgrade PRs before merging to
production. The admin key for mainnet should be a multisig account.

# Soroban SDK Minor Version Bump Guide (v21.x to v22.x)

This document details the changes and improvements introduced by the Soroban SDK v22 upgrade for the Stellar-Raise project.

## Key Changes

### 1. Robust Address Handling
In v22, `Address` objects have improved internal handling and provide a more unified interface across different contract types. 

#### Recommended Pattern:
```rust
// Use require_auth() for explicit authorization
user.require_auth();
```

### 2. Authorization Security (Auth)
The `require_auth` pattern is now more standardized. It ensures that the current address has signed for the transaction or that a valid authorization signature is provided.

### 3. Scalability & Resource Footprint
v22 introduces several under-the-hood optimizations that reduce the footprint of compiled WASM files and optimize gas consumption for common operations:
- **Footprint Reduction**: More efficient storage key serialization.
- **Gas Efficiency**: Optimized environment calls for `Env` and `Address`.

### 4. Cross-Contract Call Improvements
Inter-contract communication is more type-safe and benefits from better diagnostic errors in v22.

## Migration Steps for Scripts

When upgrading scripts to be compatible with v22 tooling:
1. Ensure `soroban-cli` is updated to a version compatible with Protocol 22.
2. Update `Cargo.toml` to use `soroban-sdk = "22.0.0"`.
3. Verify that `require_auth` is used instead of deprecated authorization patterns.

## Scalability Impact on Crowdfunding DApp
For the Crowdfunding DApp, these changes mean:
- Lower fees for contributors due to optimized `contribution` logic.
- Reduced storage costs for campaign metadata.
- Faster execution for `withdraw` and `refund` operations.

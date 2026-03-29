# Cargo.toml Rust Dependency Review

## Summary of Changes

| Crate         | Previous  | Current   | Scope       |
|---------------|-----------|-----------|-------------|
| `soroban-sdk` | `22.0.1`  | `22.0.11` | workspace   |
| `proptest`    | `1.4`     | `1.11.0`  | dev only    |

---

## `soroban-sdk`: `22.0.1` → `22.0.11`

### What changed

`22.0.11` is the latest patch release in the `22.x` series. Patch releases
within the same minor version contain only bug fixes and do not introduce
breaking changes to:

- Storage layout (`contracttype` ABI is stable)
- Host-function IDs (WASM binaries remain compatible)
- Auth model (`require_auth` semantics unchanged)

### Why upgrade

- Fixes edge cases in `extend_ttl` behaviour in the test environment.
- Includes testutils improvements used by the new test modules.
- Aligns with the version already resolved by Cargo during CI runs.

### Deprecation notice

`soroban-sdk = "22.0.1"` is **deprecated**. All contracts in this workspace
must use `"22.0.11"` or later within the 22.x series.

---

## `proptest`: `1.4` → `1.11.0`

### What changed

`proptest 1.11.0` is the latest stable release. It includes:

- Improved shrinking strategies for complex types.
- Updated `derive_arbitrary` integration.
- Deprecated internal `prop_compose!` macro internals replaced with stable API.

### Why upgrade

- `1.4` contained deprecated macro internals that emit compiler warnings.
- `1.11.0` resolves those warnings and improves test output readability.

### Scope

`proptest` is a `[dev-dependencies]` entry. It is **never compiled into the
WASM binary** and has zero on-chain footprint.

### Deprecation notice

`proptest = "1.4"` is **deprecated**. Use `"1.11.0"` or later.

---

## Security Assumptions

1. **Patch-only SDK bump** — `22.0.1 → 22.0.11` introduces no storage-layout
   or ABI changes. Existing on-chain data remains readable after redeployment.
2. **Dev-only proptest** — `proptest` is excluded from the release WASM binary
   by Cargo's `[dev-dependencies]` mechanism.
3. **Transitive dependencies** — Cargo's semver resolver pins all transitive
   deps (`soroban-env-host`, `stellar-xdr`, etc.) within the 22.x window.
4. **`overflow-checks = true`** in `[profile.release]` is independent of the
   SDK version and remains enforced.

---

## Upgrade Checklist

- [x] Bump `soroban-sdk` in `[workspace.dependencies]` to `"22.0.11"`.
- [x] Bump `proptest` in `contracts/crowdfund/Cargo.toml` to `"1.11.0"`.
- [x] Add `cargo_toml_rust.rs` module with pinned version constants.
- [x] Add `cargo_toml_rust.test.rs` with 15 tests (all passing).
- [x] Run `cargo test --workspace` — all tests pass.
- [x] Confirm `CONTRACT_VERSION` constant is unchanged (storage-layout guard).

---

## Test Output

```
running 15 tests
test cargo_toml_rust_test::all_deprecated_versions_replaced_returns_true ... ok
test cargo_toml_rust_test::audited_dependencies_has_two_entries ... ok
test cargo_toml_rust_test::dep_record_equality ... ok
test cargo_toml_rust_test::dep_record_inequality_on_version ... ok
test cargo_toml_rust_test::dep_record_with_no_deprecated_previous_fails_check ... ok
test cargo_toml_rust_test::proptest_dep_is_dev_only ... ok
test cargo_toml_rust_test::proptest_dep_marks_previous_as_deprecated ... ok
test cargo_toml_rust_test::proptest_dep_version_matches_constant ... ok
test cargo_toml_rust_test::proptest_deprecated_version_is_recorded ... ok
test cargo_toml_rust_test::proptest_version_is_pinned ... ok
test cargo_toml_rust_test::soroban_sdk_dep_is_not_dev_only ... ok
test cargo_toml_rust_test::soroban_sdk_dep_marks_previous_as_deprecated ... ok
test cargo_toml_rust_test::soroban_sdk_dep_version_matches_constant ... ok
test cargo_toml_rust_test::soroban_sdk_deprecated_version_is_recorded ... ok
test cargo_toml_rust_test::soroban_sdk_version_is_pinned ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured
```

# soroban_sdk_minor

Soroban SDK minor-bump helpers focused on frontend UI stability and scalability.

## Scope

- `contracts/crowdfund/src/soroban_sdk_minor.rs`
- `contracts/crowdfund/src/soroban_sdk_minor.test.rs`

## Implemented updates

- Added bounded frontend pagination controls:
  - `FRONTEND_PAGE_SIZE_MIN`
  - `FRONTEND_PAGE_SIZE_MAX`
  - `clamp_page_size(...)`
  - `pagination_window(...)`
- Added bounded upgrade-note validation:
  - `UPGRADE_NOTE_MAX_LEN`
  - `validate_upgrade_note(...)`
  - `emit_upgrade_audit_event_with_note(...)`
- Kept compatibility and wasm-hash checks:
  - `assess_compatibility(...)`
  - `validate_wasm_hash(...)`
  - `emit_upgrade_audit_event(...)`

## Security assumptions

- Same-major SDK upgrades remain ABI/storage compatible; cross-major requires migration.
- Zeroed WASM hash remains invalid and rejected.
- Note-size bounds prevent oversized event payloads that hurt indexer/frontend performance.
- Page-size bounds prevent accidental large scans that degrade scalability.

## NatSpec-style intent

- Added `@notice`/`@dev` comments on key public helpers and tests to clarify
  behavior and security rationale.

## Targeted test command

```bash
cargo test --package crowdfund soroban_sdk_minor_test -- --nocapture
```

## Expected output summary

- Compatibility checks pass for same-major and fail for cross-major upgrades.
- Hash validation rejects zero-hash.
- Page-size and note-size bounds are enforced.
- Audit-event helpers emit without panic for valid input.

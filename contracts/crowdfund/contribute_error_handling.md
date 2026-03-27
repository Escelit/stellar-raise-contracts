# contribute() Error Handling — Logging Bounds

## Overview

`contribute()` returns typed `ContractError` variants for every failure path.
In addition, each error path now emits a structured **diagnostic event** so
that off-chain scripts and monitoring tools can observe failures without
parsing host-level error codes.

## Error taxonomy

| Code | Variant             | Trigger                                     |
|------|---------------------|---------------------------------------------|
|  2   | `CampaignEnded`     | `ledger.timestamp > deadline`               |
|  6   | `Overflow`          | checked arithmetic overflowed               |
|  8   | `ZeroAmount`        | `amount == 0`                               |
|  9   | `BelowMinimum`      | `amount < min_contribution`                 |
| 10   | `CampaignNotActive` | campaign status is not `Active`             |

## Diagnostic event schema

Every error path in `contribute()` emits one event before returning:

| Field   | Value                                  |
|---------|----------------------------------------|
| topic 0 | `Symbol("contribute_error")`           |
| topic 1 | `Symbol(<VariantName>)`                |
| data    | `u32` error code (matches table above) |

### Example (Stellar CLI)

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  --source <KEY> \
  -- contribute \
  --contributor <ADDR> \
  --amount 0
# Emits: contribute_error / ZeroAmount / 8
```

### Filtering events in a script

```bash
stellar events \
  --contract-id <CONTRACT_ID> \
  --network testnet \
  --filter-topics "contribute_error,*"
```

## Implementation

`contribute_error_handling::log_contribute_error(&env, error)` is called
immediately before each `return Err(...)` in `contribute()`. It is a
pure event-emit with no state mutation.

The helper lives in `contracts/crowdfund/src/contribute_error_handling.rs`
and is tested in `contracts/crowdfund/src/contribute_error_handling_tests.rs`.

## Security notes

- `log_contribute_error` is not callable externally — it is a private helper.
- It emits read-only data; it cannot mutate contract state.
- No sensitive data (keys, balances) is included in the event payload.
- `contributor.require_auth()` is called before any logging or state access.

## Running tests

```bash
cargo test -p crowdfund contribute_error_handling
```

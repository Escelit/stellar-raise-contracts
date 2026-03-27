# contribute_error_handling

Typed error codes and diagnostic helpers for the `contribute()` entry point.

## Overview

This module replaces ad-hoc panic strings with typed `ContractError` variants and provides:

- **`error_codes`** — numeric constants matching `ContractError`'s `#[repr(u32)]` values for off-chain use.
- **`describe_error(code)`** — human-readable message for any error code.
- **`is_retryable(code)`** — distinguishes input errors (caller can fix and retry) from permanent state errors.
- **`log_contribute_error(env, error)`** — emits a structured diagnostic event before each error return.

## Error Reference

| Code | Variant            | Trigger                              | Retryable |
|------|--------------------|--------------------------------------|-----------|
| 2    | `CampaignEnded`    | `ledger.timestamp > deadline`        | No        |
| 6    | `Overflow`         | checked_add would overflow           | No        |
| 8    | `ZeroAmount`       | `amount == 0`                        | Yes       |
| 9    | `BelowMinimum`     | `amount < min_contribution`          | Yes       |
| 10   | `CampaignNotActive`| campaign status ≠ `Active`           | No        |
| 11   | `NegativeAmount`   | `amount < 0`                         | Yes       |

## Validation Order in `contribute()`

```
1. status != Active       → CampaignNotActive  (checked first — fast exit)
2. amount < 0             → NegativeAmount
3. amount == 0            → ZeroAmount
4. amount < min           → BelowMinimum
5. timestamp > deadline   → CampaignEnded
6. checked_add overflows  → Overflow
```

## Diagnostic Events

Each error path emits a `contribute_error` event before returning:

| Topic 0            | Topic 1                  | Data   |
|--------------------|--------------------------|--------|
| `contribute_error` | `Symbol(<VariantName>)`  | `u32`  |

Off-chain indexers can subscribe to `contribute_error` to observe failures without parsing host-level error codes.

## Security Considerations

- `contributor.require_auth()` is called before any validation — auth failure is always the first gate.
- Negative amounts are rejected before zero/minimum checks to prevent unexpected token-level behaviour.
- The deadline check uses strict `>`: contributions at exactly the deadline timestamp are accepted.
- `log_contribute_error` is read-only and cannot be called externally.

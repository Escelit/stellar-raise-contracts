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
# contribute() Error Handling

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

match client.try_contribute(&contributor, &amount) {
    Ok(_) => println!("contributed"),
    Err(Ok(e)) => eprintln!("contract error {}: {}", e as u32, describe_error(e as u32)),
    Err(Err(e)) => eprintln!("host error: {:?}", e),
}
```
Documents every typed error path in `contribute()`, provides off-chain helper
utilities for scripts, and records the security assumptions for the contribution
flow.

## Error Reference

| Code | Variant          | Trigger                                          | Retryable |
| :--- | :--------------- | :----------------------------------------------- | :-------- |
| 2    | `CampaignEnded`  | `ledger.timestamp > deadline`                    | No        |
| 6    | `Overflow`       | `checked_add` would wrap on contribution totals  | No        |
| 9    | `AmountTooLow`   | `amount < min_contribution`                      | Yes       |
| 10   | `ZeroAmount`     | `amount == 0`                                    | Yes       |

All error codes map directly to the `#[repr(u32)]` values of `ContractError`
in `lib.rs`. Off-chain scripts can compare the raw `u32` code against the
constants in `contribute_error_handling::error_codes`.

## How Off-Chain Scripts Should Interpret Error Codes

```rust
use crowdfund::contribute_error_handling::{describe_error, error_codes, is_retryable};

match client.try_contribute(&contributor, &amount) {
    Ok(_) => println!("contributed successfully"),
    Err(Ok(e)) => {
        let code = e as u32;
        eprintln!("contract error {}: {}", code, describe_error(code));
        if is_retryable(code) {
            // Error Code 9 (AmountTooLow) or 10 (ZeroAmount):
            // prompt the user to enter a higher amount and retry.
        }
    }
    Err(Err(e)) => eprintln!("host error: {:?}", e),
}
```

### Error Code Quick Reference

| Code | Meaning                              | Script Action                          |
| :--- | :----------------------------------- | :------------------------------------- |
| 2    | Campaign has ended                   | Do not retry; campaign is closed       |
| 6    | Arithmetic overflow                  | Do not retry; amount is unreasonably large |
| 9    | Amount below campaign minimum        | Prompt user for a higher amount        |
| 10   | Amount is zero                       | Prompt user for a non-zero amount      |

## Security Assumptions

- `contributor.require_auth()` is called **before** any state mutation.
  Unauthenticated callers are rejected at the host level.
- The token transfer happens **before** storage writes. If the transfer
  fails, the transaction rolls back atomically — no partial state is
  persisted.
- Overflow is caught with `checked_add` on both the per-contributor running
  total and `total_raised`, returning `ContractError::Overflow` (code 6)
  rather than wrapping silently.
- The deadline check uses strict `>`, so a contribution submitted at exactly
  the deadline timestamp is **accepted**. Scripts should account for this
  boundary when computing whether a campaign is still open.
- A zero-amount contribution is rejected with `ZeroAmount` (code 10) before
  the minimum check, preventing gas waste and polluted contributor lists.

## Constants

All numeric thresholds used in `contribute()` are stored in instance storage
under `DataKey::MinContribution` and set at initialization time via
`initialize(..., min_contribution, ...)`. There are no hardcoded numeric
thresholds in the contribution logic itself.
| Topic 0            | Topic 1                  | Data   |
|--------------------|--------------------------|--------|
| `contribute_error` | `Symbol(<VariantName>)`  | `u32`  |

Off-chain indexers can subscribe to `contribute_error` to observe failures without parsing host-level error codes.

## Security Considerations

## Tests

`contracts/crowdfund/src/contribute_error_handling_tests.rs`

22 tests — all passing:

```
contribute_happy_path                                    ok
contribute_accumulates_multiple_contributions            ok
contribute_after_deadline_returns_campaign_ended         ok
contribute_exactly_at_deadline_is_accepted               ok
contribute_below_minimum_returns_typed_error             ok
contribute_one_below_minimum_returns_below_minimum       ok
contribute_zero_amount_returns_typed_error               ok
contribute_to_cancelled_campaign_returns_not_active      ok
contribute_to_successful_campaign_returns_not_active     ok
contribute_negative_amount_returns_typed_error           ok
contribute_large_negative_amount_returns_typed_error     ok
overflow_error_code_is_correct                           ok
negative_amount_error_code_is_correct                    ok
describe_error_campaign_ended                            ok
describe_error_overflow                                  ok
describe_error_zero_amount                               ok
describe_error_below_minimum                             ok
describe_error_campaign_not_active                       ok
describe_error_negative_amount                           ok
describe_error_unknown                                   ok
is_retryable_returns_false_for_all_known_errors          ok
16 tests — all passing:

```
contribute_happy_path                                    ok
contribute_zero_amount_returns_zero_amount_error         ok
contribute_below_minimum_returns_amount_too_low          ok
contribute_one_below_minimum_returns_amount_too_low      ok
contribute_after_deadline_returns_campaign_ended         ok
contribute_exactly_at_deadline_is_accepted               ok
overflow_error_code_matches_enum_repr                    ok
error_code_constants_match_enum_reprs                    ok
describe_error_campaign_ended                            ok
describe_error_overflow                                  ok
describe_error_amount_too_low                            ok
describe_error_zero_amount                               ok
describe_error_unknown                                   ok
is_retryable_amount_too_low_and_zero_amount_are_retryable  ok
is_retryable_campaign_ended_and_overflow_are_not_retryable ok
is_retryable_returns_false_for_negative_amount           ok
```
- `contributor.require_auth()` is called before any validation — auth failure is always the first gate.
- Negative amounts are rejected before zero/minimum checks to prevent unexpected token-level behaviour.
- The deadline check uses strict `>`: contributions at exactly the deadline timestamp are accepted.
- `log_contribute_error` is read-only and cannot be called externally.

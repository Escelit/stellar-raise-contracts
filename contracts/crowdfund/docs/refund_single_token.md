# `refund_single_token` — pull-based refund and token transfer

## Purpose

This module implements the **single-contributor refund** path for expired campaigns that did not reach their goal:

1. **`validate_refund_preconditions`** — read-only guard (`Status::Expired`, contribution > 0).
2. **`execute_refund_single`** — CEI: zero contribution, decrement `total_raised`, `token::transfer`, emit event.
3. **`refund_single_transfer`** — shared token primitive (contract → contributor); no-op if `amount <= 0`.
4. **`get_contribution` / `refund_single`** — storage read and legacy helper used in tests / batch-style flows.

Public entry: `CrowdfundContract::refund_single` (`require_auth` → validate → execute).

## CEI (Checks–Effects–Interactions)

| Phase | What happens |
|-------|----------------|
| Effects | `Contribution(contributor)` set to `0`, TTL extended, `TotalRaised` decreased via `checked_sub` |
| Interactions | Stellar token `transfer(contract, contributor, amount)` |

Zeroing **before** transfer mitigates re-entrancy double-claims if the token hooks the crowdfund again.

## Security assumptions

| Topic | Assumption |
|-------|------------|
| Auth | Only the public `refund_single` entrypoint calls `require_auth`; module functions trust the caller. |
| Amount | `execute_refund_single` must receive the same `amount` as stored for the contributor; mismatches can corrupt totals (contribution is cleared before `total_raised` is updated). `ContractError::Overflow` is only returned when `checked_sub` truly overflows `i128`. |
| Token | Standard Soroban token client; campaign must hold balance; malicious ERC-20-style behaviour is out of scope. |
| Status | Refunds are allowed only in `Expired`; other statuses panic in `validate_refund_preconditions`. |

## Dependencies (audit checklist)

- `DataKey::Contribution`, `DataKey::TotalRaised`, `DataKey::Token`, `DataKey::Status`
- `ContractError::NothingToRefund`, `ContractError::Overflow`
- `soroban_sdk::token::Client`

## Tests

```bash
cargo test -p crowdfund refund_single_token_test
```

Source: `src/refund_single_token.test.rs` (included from `lib.rs` under `cfg(test)`).

## Observability

- Debug event: `("debug", "refund_transfer_attempt")` before token transfer in `refund_single_transfer`.
- Campaign event: `("campaign", "refund_single")` with `(contributor, amount)` after a successful `execute_refund_single`.

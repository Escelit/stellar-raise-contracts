# State Compression

## Overview

`state_compression` packs the four most-frequently-read campaign scalars into a
single instance-storage slot, reducing ledger I/O costs by roughly 75 % compared
with four individual key lookups.

## Motivation

Every Soroban ledger read incurs a fee. The crowdfund contract reads `Goal`,
`Deadline`, `MinContribution`, and `TotalRaised` on almost every entry point
(`contribute`, `withdraw`, `refund_single`, view functions). Storing them
together under one key (`CompressedKey::State`) means a single `get` fetches
all four values atomically.

## Public API

| Function | Description |
|---|---|
| `load(env)` | Returns `Option<CompressedState>` — `None` when uninitialised. |
| `load_or_init(env)` | Returns `CompressedState` with zero values when absent. |
| `store(env, state)` | Persists all four fields atomically. |
| `apply_contribution(env, amount)` | Adds `amount` to `total_raised`; returns `None` on overflow. |
| `apply_refund(env, amount)` | Subtracts `amount` from `total_raised`; returns `None` on underflow or negative result. |
| `is_goal_reached(env)` | Returns `true` when `total_raised >= goal`. |
| `is_expired(env)` | Returns `true` when ledger timestamp > deadline. |
| `progress_bps(env)` | Returns progress toward goal in basis points (0–10 000), saturating at 10 000. |

## Security Notes

- **No auth enforced here.** All callers in `lib.rs` must authenticate before
  invoking mutation helpers.
- **Overflow-safe.** `apply_contribution` and `apply_refund` use `checked_add` /
  `checked_sub` and return `None` without writing on overflow or underflow.
- **Non-negative invariant.** `apply_refund` explicitly rejects results < 0,
  preventing `total_raised` from going negative even if `checked_sub` would
  technically succeed for negative inputs.
- **Atomic writes.** All four fields are written together; there is no window
  where only some fields are updated.

## Usage Example

```rust
use crate::state_compression::{self, CompressedState};

// During initialize:
state_compression::store(&env, &CompressedState {
    goal,
    deadline,
    min_contribution,
    total_raised: 0,
});

// During contribute:
state_compression::apply_contribution(&env, amount)
    .ok_or(ContractError::Overflow)?;

// During refund_single:
state_compression::apply_refund(&env, amount)
    .ok_or(ContractError::Underflow)?;

// View helper:
let bps = state_compression::progress_bps(&env); // 0–10_000
```

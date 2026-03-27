# Withdraw Event Emission

## Overview

Centralises all event publishing for the `withdraw()` function in three
validated helper functions. Replaces scattered inline `env.events().publish()`
calls and eliminates the O(n) per-contributor event loop.

## Problem

The original `withdraw()` had two issues:

1. **Unbounded per-contributor events** — the NFT minting loop emitted one
   `nft_minted` event per contributor with no upper bound. A campaign with
   thousands of contributors would emit thousands of events in a single
   transaction, causing unpredictable gas consumption.

2. **No payload validation** — event amounts were published inline with no
   assertions, making it possible to silently emit a zero-fee or zero-payout
   event that would mislead off-chain indexers.

## Solution

### Three validated emit helpers

| Function                | Event topic 2      | Validation                      |
|-------------------------|--------------------|---------------------------------|
| `emit_fee_transferred`  | `fee_transferred`  | Panics if `fee <= 0`            |
| `emit_nft_batch_minted` | `nft_batch_minted` | Panics if `minted_count == 0`   |
| `emit_withdrawn`        | `withdrawn`        | Panics if `creator_payout <= 0` |

### O(1) batch event

The per-contributor `nft_minted` loop is replaced by `mint_nfts_in_batch`,
which processes at most `MAX_NFT_MINT_BATCH` (50) contributors and emits a
single `nft_batch_minted` summary event carrying the total count.

## Events Reference

| Topic 1    | Topic 2            | Data                   | Condition                              |
|------------|--------------------|------------------------|----------------------------------------|
| `campaign` | `fee_transferred`  | `(Address, i128)`      | Platform fee configured and fee > 0   |
| `campaign` | `nft_batch_minted` | `u32`                  | NFT contract set, at least 1 minted   |
| `campaign` | `withdrawn`        | `(Address, i128, u32)` | Always on successful `withdraw()`     |

### `withdrawn` data tuple

```
(creator: Address, creator_payout: i128, nft_minted_count: u32)
```

> **Breaking change for indexers:** The old two-field tuple `(Address, i128)`
> is now a three-field tuple `(Address, i128, u32)`. Off-chain indexers must
> be updated to decode the new shape.

## Security Assumptions

- **Positive-only amounts** — `emit_fee_transferred` and `emit_withdrawn`
  assert their amount arguments are strictly positive. A zero or negative value
  indicates a logic error upstream and will panic rather than silently emit a
  misleading event.
- **Non-empty batch** — `emit_nft_batch_minted` asserts `minted_count > 0`.
  The caller guards this with `if minted > 0` before calling the helper.
- **State-before-event ordering** — `emit_withdrawn` is called after
  `TotalRaised` is zeroed and `Status` is set to `Succeeded`. Off-chain
  indexers that read contract state on event receipt will observe the final
  consistent state.
- **Single emission** — `emit_withdrawn` is called exactly once per
  `withdraw()` invocation. The status guard prevents a second call from
  reaching the emission point.
- **Cap does not skip contributors permanently** — `MAX_NFT_MINT_BATCH` limits
  a single `withdraw()` call only.

## Test Coverage

File: `contracts/crowdfund/src/withdraw_event_emission_test.rs`

### NFT minting cap (4 tests)

| Test | Verifies |
|------|----------|
| `test_withdraw_mints_all_when_within_cap` | All minted when count < cap |
| `test_withdraw_caps_minting_at_max_batch` | Only cap minted when count > cap |
| `test_withdraw_mints_exactly_at_cap_boundary` | Exact boundary mints exactly cap |
| `test_withdraw_mints_single_contributor` | Single contributor minted |

### `nft_batch_minted` event (4 tests)

| Test | Verifies |
|------|----------|
| `test_withdraw_emits_single_batch_event` | Exactly one event (not one per contributor) |
| `test_withdraw_no_batch_event_without_nft_contract` | No event without NFT contract |
| `test_withdraw_batch_event_data_equals_minted_count` | Data equals actual mint count |
| `test_withdraw_batch_event_data_capped_at_max` | Data equals cap when count > cap |

### `withdrawn` event (5 tests)

| Test | Verifies |
|------|----------|
| `test_withdraw_emits_withdrawn_event_once` | Emitted exactly once |
| `test_withdraw_emits_withdrawn_event_without_nft` | Emitted without NFT contract |
| `test_withdrawn_event_nft_count_zero_without_nft_contract` | nft_count is 0 without NFT |
| `test_withdrawn_event_payout_equals_total_raised_no_fee` | Payout equals total raised |
| `test_withdrawn_event_payout_reflects_fee_deduction` | Payout is net of platform fee |

### `fee_transferred` event (2 tests)

| Test | Verifies |
|------|----------|
| `test_withdraw_emits_fee_transferred_event` | Emitted with platform fee |
| `test_withdraw_no_fee_event_without_platform_config` | Not emitted without config |

### Double-withdraw guard (1 test)

| Test | Verifies |
|------|----------|
| `test_double_withdraw_panics` | Second call panics |

### Security unit tests — emit helpers (9 tests)

| Test | Verifies |
|------|----------|
| `test_emit_fee_transferred_panics_on_zero_fee` | Panics on fee == 0 |
| `test_emit_fee_transferred_panics_on_negative_fee` | Panics on fee < 0 |
| `test_emit_fee_transferred_succeeds_with_positive_fee` | Succeeds with fee > 0 |
| `test_emit_nft_batch_minted_panics_on_zero_count` | Panics on count == 0 |
| `test_emit_nft_batch_minted_succeeds_with_positive_count` | Succeeds with count > 0 |
| `test_emit_withdrawn_panics_on_zero_payout` | Panics on payout == 0 |
| `test_emit_withdrawn_panics_on_negative_payout` | Panics on payout < 0 |
| `test_emit_withdrawn_succeeds_with_valid_args` | Succeeds with valid args |
| `test_emit_withdrawn_allows_zero_nft_count` | nft_count of 0 is valid |

**Total: 25 tests**

## Running the tests

```bash
cargo test --package crowdfund withdraw_event_emission
```

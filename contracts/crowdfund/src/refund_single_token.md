# `refund_single` — Pull-Based Token Refund

## Overview

`refund_single` is the preferred refund mechanism for the crowdfund contract.
It replaces the deprecated batch `refund()` function with a pull-based model
where each contributor independently claims their own refund.

## Why pull-based?

The old `refund()` iterated over every contributor in a single transaction.
On a campaign with many contributors this is unsafe:

- **Unbounded gas**: iteration cost grows linearly with contributor count.
- **Denial of service**: a single bad actor could bloat the contributors list
  to make the batch refund prohibitively expensive.
- **Poor composability**: scripts and automation cannot easily retry partial
  failures.

`refund_single` processes exactly one contributor per call, so gas costs are
constant and predictable regardless of campaign size.

## Function Signature

```rust
pub fn refund_single(env: Env, contributor: Address) -> Result<(), ContractError>
```

### Arguments

| Parameter     | Type      | Description                                      |
|---------------|-----------|--------------------------------------------------|
| `contributor` | `Address` | The address claiming the refund (must be caller) |

### Return value

`Ok(())` on success, or one of the errors below.

### Errors

| Error                          | Condition                                                    |
|--------------------------------|--------------------------------------------------------------|
| `ContractError::CampaignStillActive` | Deadline has not yet passed                            |
| `ContractError::GoalReached`   | Campaign goal was met — no refunds available                 |
| `ContractError::NothingToRefund` | Caller has no contribution on record (or already claimed)  |

### Panics

- `"campaign is not active"` — campaign status is `Successful` or `Cancelled`.

## Security Model

1. **Authentication** — `contributor.require_auth()` is called first. Only the
   contributor themselves can trigger their own refund.

2. **Direction Lock** — The token transfer explicitly uses the contract's address
   as the sender and the contributor as the recipient. This prevents parameter-order
   typos and ensures the direction cannot be reversed by a caller.

2. **Direction Lock** — The token transfer explicitly uses the contract's address
   as the sender and the contributor as the recipient. This prevents parameter-order
   typos and ensures the direction cannot be reversed by a caller.

3. **Checks-Effects-Interactions** — The contribution record is zeroed in
   storage *before* the token transfer is executed. This prevents re-entrancy
   and double-claim attacks even if the token contract calls back into the
   crowdfund contract.

4. **Overflow protection** — `total_raised` is decremented with `checked_sub`,
   panicking on underflow rather than silently wrapping.

5. **Status guard** — `Successful` and `Cancelled` campaigns are explicitly
   rejected. A `Refunded` campaign (set by the deprecated batch path) is
   allowed so that any contributor not swept by the batch can still claim.

## Events

On success, the following event is emitted:

```
topic:  ("campaign", "refund_single")
data:   (contributor: Address, amount: i128)
```

Off-chain indexers and scripts should listen for this event to track refund
activity without polling storage.

## Deprecation of `refund()`

The batch `refund()` function is **deprecated** as of contract v3. It remains
callable for backward compatibility but will be removed in a future upgrade.

Migration checklist for scripts and frontends:

- [ ] Remove any call to `refund()`.
- [ ] For each contributor, call `refund_single(contributor)` instead.
- [ ] Handle `NothingToRefund` gracefully (contributor already claimed or
      was never a contributor).
- [ ] Listen for `("campaign", "refund_single")` events instead of
      `("campaign", "refunded")`.

## CLI Usage

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --network testnet \
  --source <CONTRIBUTOR_SECRET_KEY> \
  -- refund_single \
  --contributor <CONTRIBUTOR_ADDRESS>
```

## Script Example (TypeScript / Stellar SDK)

```typescript
import { Contract, SorobanRpc, TransactionBuilder, Networks } from "@stellar/stellar-sdk";

async function claimRefund(
  contractId: string,
  contributorKeypair: Keypair,
  server: SorobanRpc.Server
) {
  const account = await server.getAccount(contributorKeypair.publicKey());
  const contract = new Contract(contractId);

  const tx = new TransactionBuilder(account, { fee: "100", networkPassphrase: Networks.TESTNET })
    .addOperation(
      contract.call("refund_single", contributorKeypair.publicKey())
    )
    .setTimeout(30)
    .build();

  const prepared = await server.prepareTransaction(tx);
  prepared.sign(contributorKeypair);
  const result = await server.sendTransaction(prepared);
  return result;
}
```

## Storage Layout

| Key                          | Storage    | Type    | Description                          |
|------------------------------|------------|---------|--------------------------------------|
| `DataKey::Contribution(addr)`| Persistent | `i128`  | Per-contributor balance; zeroed on claim |
| `DataKey::TotalRaised`       | Instance   | `i128`  | Global total; decremented on each claim  |

## Test Coverage

### `refund_single_token.test.rs` — unit tests for module internals

Tests `validate_refund_preconditions` and `execute_refund_single` directly
via `env.as_contract`, covering:

| Test | What it validates |
|------|-------------------|
| `test_validate_returns_amount_on_success` | Happy path — returns contribution amount |
| `test_validate_before_deadline_returns_campaign_still_active` | Deadline guard |
| `test_validate_at_deadline_boundary_returns_campaign_still_active` | Strict `>` boundary |
| `test_validate_goal_reached_returns_goal_reached` | Goal exactly met |
| `test_validate_goal_exceeded_returns_goal_reached` | Goal exceeded |
| `test_validate_no_contribution_returns_nothing_to_refund` | Unknown address |
| `test_validate_after_refund_returns_nothing_to_refund` | Already-claimed address |
| `test_validate_panics_on_successful_campaign` | Status guard — Successful |
| `test_validate_panics_on_cancelled_campaign` | Status guard — Cancelled |
| `test_execute_transfers_correct_amount` | Token balance after transfer |
| `test_execute_zeroes_storage_before_transfer` | CEI order |
| `test_execute_decrements_total_raised` | Global accounting |
| `test_execute_double_refund_prevention` | amount=0 is a no-op |
| `test_execute_large_amount_no_overflow` | `checked_sub` on large values |
| `test_execute_does_not_affect_other_contributors` | Isolation |

### `refund_single_token_tests.rs` — integration tests via contract client

Tests the full `refund_single` contract method end-to-end, covering:
basic refund, multi-contributor, accumulated contributions, double-claim,
zero-contribution, deadline boundary, goal-reached, campaign status guards,
auth enforcement, interaction with deprecated `refund()`, platform fee
isolation, contribution record zeroing, partial claims, and minimum amount.

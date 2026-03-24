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

2. **Checks-Effects-Interactions** — The contribution record is zeroed in
   storage *before* the token transfer is executed. This prevents re-entrancy
   and double-claim attacks even if the token contract calls back into the
   crowdfund contract.

3. **Overflow protection** — `total_raised` is decremented with `checked_sub`,
   panicking on underflow rather than silently wrapping.

4. **Status guard** — `Successful` and `Cancelled` campaigns are explicitly
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

See [`refund_single_token_tests.rs`](./refund_single_token_tests.rs) for the
full test suite. Tests cover:

- Basic single-contributor refund
- Multi-contributor independent claims
- Incremental `total_raised` accounting
- Accumulated contributions (multiple `contribute` calls)
- Double-claim prevention (`NothingToRefund`)
- Zero-contribution guard
- Deadline boundary (at deadline vs. past deadline)
- Goal-reached guard (exact and exceeded)
- Campaign status guards (`Successful`, `Cancelled`)
- Auth enforcement
- Interaction with deprecated batch `refund()`
- Platform fee isolation (fee does not affect refund amount)
- Contribution record zeroed after claim
- Partial claims (other contributors unaffected)
- Minimum contribution boundary

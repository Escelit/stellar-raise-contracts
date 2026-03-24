# stellar_token_minter — Crowdfund Contract

Technical reference for the Stellar Raise crowdfund smart contract built with Soroban SDK 22.

---

## Overview

The crowdfund contract manages a single campaign lifecycle:

```
Active → Successful  (goal met, creator withdraws)
Active → Refunded    (deadline passed, goal not met)
Active → Cancelled   (creator cancels early)
```

All token amounts are in the token's smallest unit (stroops for XLM).

---

## Contract Functions

### `initialize`

```rust
fn initialize(
    env: Env,
    admin: Address,
    creator: Address,
    token: Address,
    goal: i128,
    deadline: u64,
    min_contribution: i128,
    platform_config: Option<PlatformConfig>,
    bonus_goal: Option<i128>,
    bonus_goal_description: Option<String>,
) -> Result<(), ContractError>
```

Creates a new campaign. Can only be called once.

- `admin` — stored for `upgrade` authorization.
- `creator` — must sign the transaction (`require_auth`).
- `platform_config` — optional fee recipient; `fee_bps` must be ≤ 10,000.
- `bonus_goal` — must be strictly greater than `goal`.

**Errors:** `AlreadyInitialized`  
**Panics:** platform fee > 100%, bonus goal ≤ primary goal

---

### `contribute`

```rust
fn contribute(env: Env, contributor: Address, amount: i128) -> Result<(), ContractError>
```

Transfers `amount` tokens from `contributor` to the contract. Contributor must sign.

- Rejects amounts below `min_contribution`.
- Rejects contributions after `deadline`.
- Emits `("campaign", "contributed")` event.
- Fires `("campaign", "bonus_goal_reached")` once when `total_raised` crosses `bonus_goal`.

**Errors:** `CampaignEnded`, `Overflow`  
**Panics:** amount below minimum

---

### `pledge`

```rust
fn pledge(env: Env, pledger: Address, amount: i128) -> Result<(), ContractError>
```

Records a pledge without transferring tokens. Tokens are collected later via `collect_pledges`.

**Errors:** `CampaignEnded`

---

### `collect_pledges`

```rust
fn collect_pledges(env: Env) -> Result<(), ContractError>
```

Pulls tokens from all pledgers after the deadline when the combined total meets the goal. Each pledger must have pre-authorized the transfer.

**Errors:** `CampaignStillActive`, `GoalNotReached`

---

### `withdraw`

```rust
fn withdraw(env: Env) -> Result<(), ContractError>
```

Creator claims raised funds after deadline when goal is met. If a `PlatformConfig` is set, the fee is deducted first. If an NFT contract is configured, mints one NFT per contributor.

- Sets status to `Successful`.
- Emits `("campaign", "withdrawn")` and optionally `("campaign", "fee_transferred")`.

**Errors:** `CampaignStillActive`, `GoalNotReached`

---

### `refund`

```rust
fn refund(env: Env) -> Result<(), ContractError>
```

Returns all contributions when the deadline has passed and the goal was not met. Callable by anyone.

- Sets status to `Refunded`.

**Errors:** `CampaignStillActive`, `GoalReached`

---

### `cancel`

```rust
fn cancel(env: Env)
```

Creator cancels the campaign early. Sets status to `Cancelled`. Does not automatically refund contributors — they must be refunded separately.

**Panics:** not active, not authorized

---

### `upgrade`

```rust
fn upgrade(env: Env, new_wasm_hash: BytesN<32>)
```

Replaces the contract WASM without changing its address or storage. Only the `admin` set at initialization can call this.

**Security note:** Test the new WASM thoroughly before upgrading — it is irreversible.

---

### `update_metadata`

```rust
fn update_metadata(
    env: Env,
    creator: Address,
    title: Option<String>,
    description: Option<String>,
    socials: Option<String>,
)
```

Updates campaign metadata fields. Only callable by the creator while the campaign is `Active`. Pass `None` to leave a field unchanged.

---

### `set_nft_contract`

```rust
fn set_nft_contract(env: Env, creator: Address, nft_contract: Address)
```

Configures the NFT contract used for contributor reward minting on successful withdrawal. Only the creator can call this.

---

### `add_stretch_goal`

```rust
fn add_stretch_goal(env: Env, milestone: i128)
```

Adds a stretch goal milestone. Must be greater than the primary goal. Only the creator can call this.

---

### `add_roadmap_item`

```rust
fn add_roadmap_item(env: Env, date: u64, description: String)
```

Appends a roadmap item. `date` must be in the future; `description` must be non-empty. Only the creator can call this.

---

## View Functions

| Function | Returns | Description |
|---|---|---|
| `total_raised` | `i128` | Total tokens contributed so far |
| `goal` | `i128` | Primary funding goal |
| `deadline` | `u64` | Campaign end timestamp |
| `min_contribution` | `i128` | Minimum contribution amount |
| `contribution(addr)` | `i128` | Contribution by a specific address |
| `contributors` | `Vec<Address>` | All contributor addresses |
| `bonus_goal` | `Option<i128>` | Optional bonus goal threshold |
| `bonus_goal_description` | `Option<String>` | Bonus goal description |
| `bonus_goal_reached` | `bool` | Whether bonus goal has been met |
| `bonus_goal_progress_bps` | `u32` | Bonus goal progress in basis points (0–10,000) |
| `current_milestone` | `i128` | Next unmet stretch goal (0 if none) |
| `get_stats` | `CampaignStats` | Aggregate stats (see below) |
| `title` | `String` | Campaign title |
| `description` | `String` | Campaign description |
| `socials` | `String` | Social links |
| `roadmap` | `Vec<RoadmapItem>` | Roadmap items |
| `token` | `Address` | Token contract address |
| `nft_contract` | `Option<Address>` | NFT contract address |
| `version` | `u32` | Contract version (currently 3) |

---

## Data Types

### `CampaignStats`

```rust
pub struct CampaignStats {
    pub total_raised: i128,
    pub goal: i128,
    pub progress_bps: u32,        // 0–10,000 (basis points)
    pub contributor_count: u32,
    pub average_contribution: i128,
    pub largest_contribution: i128,
}
```

### `PlatformConfig`

```rust
pub struct PlatformConfig {
    pub address: Address,   // fee recipient
    pub fee_bps: u32,       // fee in basis points (max 10,000 = 100%)
}
```

### `RoadmapItem`

```rust
pub struct RoadmapItem {
    pub date: u64,
    pub description: String,
}
```

### `ContractError`

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | `initialize` called more than once |
| 2 | `CampaignEnded` | Action attempted after deadline |
| 3 | `CampaignStillActive` | Action requires deadline to have passed |
| 4 | `GoalNotReached` | Withdraw/collect attempted when goal not met |
| 5 | `GoalReached` | Refund attempted when goal was met |

## Testing and Security Notes

- Test coverage is designed for 95%+ lines in the crowdfund module.
- Critical code paths covered:
  - `initialize`: repeated init, platform fee bounds, bonus goal guard.
  - `contribute`: minimum amount guard, deadline guard, aggregation, overflow protection.
  - `pledge` / `collect_pledges`: state transition and transfer effect.
  - `withdraw`: deadline, goal check, platform fee, NFT mint flow.
  - `refund`, `cancel`, `add_roadmap_item`, `add_stretch_goal`, `current_milestone`, `get_stats`, `bonus_goal`.
  - `upgrade`: admin-only authorization.

### Security assumptions

1. `creator.require_auth()` and `admin.require_auth()` provide access control in relevant calls.
2. `platform fee <= 10_000` ensures no more than 100% fees are taken.
3. `bonus_goal` strict comparison (`> goal`) prevents invalid secondary goal loops.
4. `contribute` and `collect_pledges` use `checked_add`/`checked_mul` to avoid overflow in numeric operations.
5. `status` checks in state-transition functions prevent replay / double accounting.

| 6 | `Overflow` | Integer overflow in contribution accounting |

---

## Security Assumptions

- **Auth enforcement**: `creator.require_auth()` and `contributor.require_auth()` are called on every state-changing function. The Soroban host enforces these at the protocol level.
- **Overflow protection**: All addition to `total_raised` and per-contributor balances uses `checked_add`, returning `ContractError::Overflow` on failure.
- **Platform fee cap**: Fee is validated ≤ 10,000 bps (100%) at initialization.
- **Bonus goal ordering**: Bonus goal must exceed primary goal, preventing nonsensical configurations.
- **Upgrade access control**: Only the `admin` stored at initialization can call `upgrade`. The admin address is immutable after initialization.
- **Pull-based refund**: Refunds are pull-based — each contributor calls `refund` (or the creator calls the batch `refund`). This avoids gas exhaustion from large contributor lists in a single transaction.
- **No reentrancy surface**: Soroban's execution model does not support reentrancy; token transfers are atomic host calls.

---

## Events

| Topic | Data | Emitted by |
|---|---|---|
| `("campaign", "contributed")` | `(contributor, amount)` | `contribute` |
| `("campaign", "pledged")` | `(pledger, amount)` | `pledge` |
| `("campaign", "pledges_collected")` | `total_pledged` | `collect_pledges` |
| `("campaign", "bonus_goal_reached")` | `bonus_goal` | `contribute` (once) |
| `("campaign", "withdrawn")` | `(creator, total)` | `withdraw` |
| `("campaign", "fee_transferred")` | `(platform_addr, fee)` | `withdraw` |
| `("campaign", "nft_minted")` | `(contributor, token_id)` | `withdraw` |
| `("campaign", "roadmap_item_added")` | `(date, description)` | `add_roadmap_item` |
| `("metadata_updated", creator)` | `Vec<Symbol>` of updated fields | `update_metadata` |

---

## Test Coverage

Tests live in `contracts/crowdfund/src/test.rs` (functional), `contracts/crowdfund/src/auth_tests.rs` (authorization), and `contracts/crowdfund/src/stellar_token_minter_test.rs` (minter-focused edge cases).

| Area | Tests |
|---|---|
| initialize | fields stored, double-init error, bonus goal, bad fee, bad bonus goal |
| contribute | basic, accumulation, after deadline, below minimum, contributors list |
| withdraw | success, before deadline, goal not met, platform fee, NFT minting, no NFT |
| refund | returns tokens, double refund panic, goal reached error |
| cancel | no contributions, non-creator panic, double cancel panic |
| update_metadata | stores fields, inactive campaign panic |
| pledge | records amount, after deadline error |
| collect_pledges | before deadline error, goal not met error |
| stretch goals | current milestone, no goals |
| bonus goal | reached after contribution, progress bps capped at 10,000 |
| get_stats | accurate aggregates, empty campaign |
| roadmap | add and retrieve items |
| auth | initialize, withdraw, contribute auth guards |
| upgrade | admin-only auth guard (non-admin panics) |

Run with:

```bash
cargo test --package crowdfund
```

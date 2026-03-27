# Contract State Size Limits

The `contract_state_size` module defines the maximum size limits for all
campaign-related on-chain state and provides guard functions that return typed
errors when those limits are exceeded.

## Why limits matter

Storing data on the Stellar ledger involves costs based on both the number of
entries and their byte content (state rent). Enforcing these limits at every
write:

- **Prevents ledger bloat** — caps entry sizes so state-rent stays predictable.
- **Enables frontend validation** — the UI can query constants to pre-validate
  inputs before submitting a transaction, reducing reverts.
- **Bounds collection growth** — prevents runaway storage from unbounded
  contributor, roadmap, or stretch-goal lists.

## Constants

| Constant | Value | Purpose |
|---|---|---|
| `MAX_STRING_LEN` | 256 bytes | Shared limit for all string fields |
| `MAX_CONTRIBUTORS` | 128 | Max entries in `Contributors` / `Pledgers` lists |
| `MAX_ROADMAP_ITEMS` | 32 | Max entries in the `Roadmap` list |
| `MAX_STRETCH_GOALS` | 32 | Max entries in the `StretchGoals` list |
| `MAX_TITLE_LENGTH` | 256 bytes | Alias for `MAX_STRING_LEN` |
| `MAX_DESCRIPTION_LENGTH` | 256 bytes | Alias for `MAX_STRING_LEN` |

## Error type

`StateSizeError` is a `#[contracterror]` enum with stable discriminants:

| Variant | Discriminant | Meaning |
|---|---|---|
| `ContributorLimitExceeded` | 100 | Contributors or pledgers list is full |
| `RoadmapLimitExceeded` | 101 | Roadmap list is full |
| `StretchGoalLimitExceeded` | 102 | Stretch goals list is full |
| `StringTooLong` | 103 | A string field exceeds `MAX_STRING_LEN` |

Discriminants are part of the on-chain ABI and must not be renumbered.

## Guard functions

### Storage-aware guards (require `&Env`)

| Function | Storage key | Error on |
|---|---|---|
| `check_contributor_limit(env)` | `DataKey::Contributors` (persistent) | `len >= MAX_CONTRIBUTORS` |
| `check_pledger_limit(env)` | `DataKey::Pledgers` (persistent) | `len >= MAX_CONTRIBUTORS` |
| `check_roadmap_limit(env)` | `DataKey::Roadmap` (instance) | `len >= MAX_ROADMAP_ITEMS` |
| `check_stretch_goal_limit(env)` | `DataKey::StretchGoals` (instance) | `len >= MAX_STRETCH_GOALS` |

### Pure guards (no `Env` required)

| Function | Error on |
|---|---|
| `check_string_len(s)` | `s.len() > MAX_STRING_LEN` |
| `validate_title(s)` | delegates to `check_string_len` |
| `validate_description(s)` | delegates to `check_string_len` |
| `validate_social_links(s)` | delegates to `check_string_len` |
| `validate_roadmap_description(s)` | delegates to `check_string_len` |
| `validate_bonus_goal_description(s)` | delegates to `check_string_len` |
| `validate_contributor_capacity(count)` | `count >= MAX_CONTRIBUTORS` |
| `validate_pledger_capacity(count)` | `count >= MAX_CONTRIBUTORS` |
| `validate_roadmap_capacity(count)` | `count >= MAX_ROADMAP_ITEMS` |
| `validate_stretch_goal_capacity(count)` | `count >= MAX_STRETCH_GOALS` |
| `validate_metadata_total_length(title, desc, socials)` | combined > aggregate limit |

## Queryable contract

`ContractStateSize` is a standalone Soroban contract that exposes the constants
over the ABI. The frontend can call it to retrieve limits without off-chain
configuration.

```bash
stellar contract invoke --id <CONTRACT_ID> --network testnet -- max_title_length
stellar contract invoke --id <CONTRACT_ID> --network testnet -- max_contributors
```

## Security notes

- All limits are enforced at write time in `lib.rs` and
  `crowdfund_initialize_function.rs` — they cannot be bypassed by a caller.
- Error discriminants are stable; changing them would break existing clients
  that pattern-match on the numeric value.
- The `validate_metadata_total_length` guard uses `saturating_add` to prevent
  integer overflow when summing field lengths.
- No secrets or credentials are stored or referenced in this module.

# Resource Management

## Overview

The `resource_management` module provides bounded resource allocation, per-address
rate limiting, and storage quota enforcement for the Stellar Raise crowdfund contract.

Soroban imposes per-transaction CPU and memory limits. Unbounded loops, unchecked
storage growth, and missing rate limits are common attack vectors. This module
introduces three complementary guards without changing observable contract behaviour.

---

## Files

| File | Purpose |
|---|---|
| `contracts/crowdfund/src/resource_management.rs` | Module implementation |
| `contracts/crowdfund/src/resource_management.test.rs` | Comprehensive test suite |
| `docs/resource_management.md` | This document |

---

## Core Types

### `ResourceBudget`

Tracks CPU instruction and memory byte consumption within a single transaction
and enforces configurable hard caps.

```rust
pub struct ResourceBudget {
    pub cpu_cap: u64,
    pub memory_cap: u64,
    pub cpu_used: u64,
    pub memory_used: u64,
}
```

| Method | Description |
|---|---|
| `new(cpu_cap, memory_cap)` | Creates a budget with explicit caps |
| `default_caps()` | Creates a budget with `DEFAULT_CPU_BUDGET` / `DEFAULT_MEMORY_BUDGET` |
| `consume(cpu, memory)` | Records usage; returns `Err(BudgetExceeded)` if over cap |
| `check()` | Returns `Err(BudgetExceeded)` if either cap is exceeded |
| `cpu_remaining()` | Remaining CPU instructions (saturating at 0) |
| `memory_remaining()` | Remaining memory bytes (saturating at 0) |
| `cpu_utilization_pct()` | CPU usage as a percentage (0–100) |
| `memory_utilization_pct()` | Memory usage as a percentage (0–100) |

**Usage:**

```rust
let mut budget = ResourceBudget::default_caps();
budget.consume(50_000, 1_024)?; // returns Err if over cap
```

### `RateLimiter`

Enforces a per-address operation count within a sliding ledger window.

```rust
pub struct RateLimiter {
    pub limit: u32,
    pub window_ledgers: u32,
    pub window_start: u32,
    pub count: u32,
}
```

| Method | Description |
|---|---|
| `new(limit, window_ledgers, current_ledger)` | Creates a limiter anchored to `current_ledger` |
| `default_for_env(env)` | Creates a limiter with default parameters |
| `record(current_ledger)` | Records one operation; resets window if expired; returns `Err(RateLimitExceeded)` if at limit |
| `remaining()` | Operations remaining in the current window |
| `is_exhausted()` | `true` when `count >= limit` |

**Usage:**

```rust
let mut limiter = RateLimiter::default_for_env(&env);
limiter.record(env.ledger().sequence())?; // Err if rate limit hit
```

### `StorageQuota`

Tracks the number of persistent storage entries written per address and enforces
a configurable per-address cap.

```rust
pub struct StorageQuota {
    pub quota: u32,
    pub used: u32,
}
```

| Method | Description |
|---|---|
| `new(quota)` | Creates a quota with the given cap |
| `default_quota()` | Creates a quota with `DEFAULT_STORAGE_QUOTA` |
| `record_write()` | Records one write; returns `Err(StorageQuotaExceeded)` if at quota |
| `remaining()` | Writes remaining before quota is hit |
| `is_exhausted()` | `true` when `used >= quota` |
| `utilization_pct()` | Quota usage as a percentage (0–100) |

---

## Constants

| Constant | Value | Description |
|---|---|---|
| `DEFAULT_CPU_BUDGET` | 100 000 000 | Default CPU instruction cap per transaction |
| `DEFAULT_MEMORY_BUDGET` | 40 000 000 | Default memory byte cap per transaction |
| `DEFAULT_RATE_LIMIT` | 10 | Default max operations per address per window |
| `DEFAULT_WINDOW_LEDGERS` | 100 | Default rate-limit window in ledger sequences |
| `DEFAULT_STORAGE_QUOTA` | 50 | Default max persistent storage entries per address |

---

## Standalone Helpers

```rust
// Returns true when used <= cap.
let ok = within_budget(used, cap);

// Returns usage as a percentage (0–100); 100 when cap == 0.
let pct = utilization_pct(used, cap);

// Estimates writes rejected given total_writes attempts against quota.
let rejected = estimated_rejected_writes(total_writes, quota);
```

---

## Security Assumptions

1. All counters use saturating arithmetic — no overflow panics on extreme inputs.
2. `RateLimiter` resets the window when the ledger sequence advances past the
   window boundary, preventing stale counts from blocking legitimate users.
3. `StorageQuota` returns `Err(StorageQuotaExceeded)` rather than silently
   dropping writes — callers must handle the error explicitly.
4. `ResourceBudget` caps are set at construction time and cannot be mutated
   after creation.
5. All structs are per-invocation — no cross-transaction state is maintained,
   consistent with Soroban's atomic execution model.

---

## Test Coverage

Run the test suite with:

```bash
cargo test -p crowdfund resource_management
```

Expected: all tests pass. Coverage targets ≥ 95% statement/branch/function/line.

Test categories:
- `ResourceBudget`: construction, consume within/at/over cap, check, remaining (saturating), utilisation percentages, zero-cap edge cases, overflow safety
- `RateLimiter`: construction, record within/at limit, window reset at boundary, remaining, exhaustion, saturating window-end arithmetic
- `StorageQuota`: construction, record_write within/at quota, remaining, exhaustion, utilisation percentages, zero-quota edge case
- `within_budget`: less than, equal to, greater than cap, zero
- `utilization_pct`: half, full, over-cap capped at 100, zero cap, zero used
- `estimated_rejected_writes`: none rejected, exact quota, some rejected, zero quota, saturation

## NatSpec-style comments

Rustdoc in `resource_management.rs` uses `@title`, `@notice`, `@param`, `@return`,
`@dev`, and `@custom:security` tags so Solidity/NatSpec-oriented reviewers can map
concepts quickly.

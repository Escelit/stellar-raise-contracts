//! # Resource Management
//!
//! @title  Smart Contract Resource Management
//! @notice Provides bounded resource allocation, per-address rate limiting,
//!         and storage quota enforcement for the Stellar Raise crowdfund contract.
//!
//! # Design Rationale
//!
//! Soroban imposes per-transaction CPU and memory limits. Unbounded loops,
//! unchecked storage growth, and missing rate limits are common attack vectors.
//! This module introduces three complementary guards:
//!
//! 1. **`ResourceBudget`** — tracks CPU instructions and memory bytes consumed
//!    within a transaction and enforces configurable hard caps.
//! 2. **`RateLimiter`** — enforces a per-address operation count within a
//!    sliding ledger window, preventing spam and DoS via repeated calls.
//! 3. **`StorageQuota`** — tracks the number of persistent storage entries
//!    written per address and enforces a per-address cap.
//!
//! # Security Assumptions
//!
//! - All counters use saturating arithmetic to prevent overflow panics.
//! - `RateLimiter` resets the window when the ledger sequence advances past
//!   the window boundary, preventing stale counts from blocking legitimate users.
//! - `StorageQuota` never silently drops writes; it returns an error when the
//!   quota is exceeded so callers can handle it explicitly.
//! - No cross-transaction state is maintained — all structs are per-invocation.

use soroban_sdk::Env;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default CPU instruction budget per transaction (informational cap).
///
/// @notice Soroban's actual limit is enforced by the host; this constant
///         provides an application-level early-exit threshold.
pub const DEFAULT_CPU_BUDGET: u64 = 100_000_000;

/// Default memory budget in bytes per transaction.
pub const DEFAULT_MEMORY_BUDGET: u64 = 40_000_000;

/// Default maximum operations per address per rate-limit window.
pub const DEFAULT_RATE_LIMIT: u32 = 10;

/// Default rate-limit window in ledger sequences.
pub const DEFAULT_WINDOW_LEDGERS: u32 = 100;

/// Default maximum persistent storage entries per address.
pub const DEFAULT_STORAGE_QUOTA: u32 = 50;

// ---------------------------------------------------------------------------
// ResourceBudget
// ---------------------------------------------------------------------------

/// Tracks CPU and memory consumption within a single transaction and enforces
/// configurable hard caps.
///
/// @notice Exceeding either cap causes `check` to return
///         `Err(ResourceError::BudgetExceeded)`.
///
/// @custom:security
///   - All arithmetic is saturating — no overflow panics.
///   - Caps are set at construction time and cannot be mutated.
#[derive(Clone, Debug, PartialEq)]
pub struct ResourceBudget {
    /// Maximum CPU instructions allowed.
    pub cpu_cap: u64,
    /// Maximum memory bytes allowed.
    pub memory_cap: u64,
    /// CPU instructions consumed so far.
    pub cpu_used: u64,
    /// Memory bytes consumed so far.
    pub memory_used: u64,
}

/// Errors returned by resource management helpers.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ResourceError {
    /// The CPU or memory budget has been exceeded.
    BudgetExceeded,
    /// The per-address rate limit has been reached for this window.
    RateLimitExceeded,
    /// The per-address storage quota has been reached.
    StorageQuotaExceeded,
}

impl ResourceBudget {
    /// Creates a `ResourceBudget` with the given caps.
    ///
    /// @param cpu_cap    Maximum CPU instructions.
    /// @param memory_cap Maximum memory bytes.
    pub fn new(cpu_cap: u64, memory_cap: u64) -> Self {
        Self {
            cpu_cap,
            memory_cap,
            cpu_used: 0,
            memory_used: 0,
        }
    }

    /// Creates a `ResourceBudget` with default caps.
    pub fn default_caps() -> Self {
        Self::new(DEFAULT_CPU_BUDGET, DEFAULT_MEMORY_BUDGET)
    }

    /// Records `cpu` instruction units and `memory` bytes consumed.
    ///
    /// @param cpu    CPU instructions to add.
    /// @param memory Memory bytes to add.
    /// @return `Ok(())` if within budget; `Err(ResourceError::BudgetExceeded)` otherwise.
    ///
    /// @dev Uses saturating_add so counters never wrap on extreme inputs.
    pub fn consume(&mut self, cpu: u64, memory: u64) -> Result<(), ResourceError> {
        self.cpu_used = self.cpu_used.saturating_add(cpu);
        self.memory_used = self.memory_used.saturating_add(memory);
        self.check()
    }

    /// Returns `Ok(())` if current usage is within both caps.
    ///
    /// @return `Err(ResourceError::BudgetExceeded)` if either cap is exceeded.
    pub fn check(&self) -> Result<(), ResourceError> {
        if self.cpu_used > self.cpu_cap || self.memory_used > self.memory_cap {
            return Err(ResourceError::BudgetExceeded);
        }
        Ok(())
    }

    /// Returns the remaining CPU budget (saturating at 0).
    pub fn cpu_remaining(&self) -> u64 {
        self.cpu_cap.saturating_sub(self.cpu_used)
    }

    /// Returns the remaining memory budget (saturating at 0).
    pub fn memory_remaining(&self) -> u64 {
        self.memory_cap.saturating_sub(self.memory_used)
    }

    /// Returns CPU utilisation as a percentage (0–100).
    pub fn cpu_utilization_pct(&self) -> u32 {
        if self.cpu_cap == 0 {
            return 100;
        }
        (self.cpu_used.saturating_mul(100) / self.cpu_cap).min(100) as u32
    }

    /// Returns memory utilisation as a percentage (0–100).
    pub fn memory_utilization_pct(&self) -> u32 {
        if self.memory_cap == 0 {
            return 100;
        }
        (self.memory_used.saturating_mul(100) / self.memory_cap).min(100) as u32
    }
}

impl Default for ResourceBudget {
    fn default() -> Self {
        Self::default_caps()
    }
}

// ---------------------------------------------------------------------------
// RateLimiter
// ---------------------------------------------------------------------------

/// Enforces a per-address operation count within a sliding ledger window.
///
/// @notice Each `RateLimiter` instance tracks a single address. Construct one
///         per address per transaction.
///
/// @custom:security
///   - The window resets automatically when the current ledger sequence
///     advances past `window_start + window_ledgers`.
///   - Counts use saturating arithmetic.
#[derive(Clone, Debug, PartialEq)]
pub struct RateLimiter {
    /// Maximum operations allowed within the window.
    pub limit: u32,
    /// Window size in ledger sequences.
    pub window_ledgers: u32,
    /// Ledger sequence at which the current window started.
    pub window_start: u32,
    /// Operations performed in the current window.
    pub count: u32,
}

impl RateLimiter {
    /// Creates a `RateLimiter` anchored to `current_ledger`.
    ///
    /// @param limit          Maximum operations per window.
    /// @param window_ledgers Window size in ledger sequences.
    /// @param current_ledger Current ledger sequence number.
    pub fn new(limit: u32, window_ledgers: u32, current_ledger: u32) -> Self {
        Self {
            limit,
            window_ledgers,
            window_start: current_ledger,
            count: 0,
        }
    }

    /// Creates a `RateLimiter` with default parameters anchored to the
    /// current ledger sequence from `env`.
    pub fn default_for_env(env: &Env) -> Self {
        Self::new(
            DEFAULT_RATE_LIMIT,
            DEFAULT_WINDOW_LEDGERS,
            env.ledger().sequence(),
        )
    }

    /// Records one operation for the given ledger sequence.
    ///
    /// Resets the window if `current_ledger` has advanced past the window
    /// boundary before recording.
    ///
    /// @param current_ledger Current ledger sequence.
    /// @return `Ok(())` if within limit; `Err(ResourceError::RateLimitExceeded)` otherwise.
    pub fn record(&mut self, current_ledger: u32) -> Result<(), ResourceError> {
        // Reset window if expired.
        let window_end = self.window_start.saturating_add(self.window_ledgers);
        if current_ledger >= window_end {
            self.window_start = current_ledger;
            self.count = 0;
        }

        if self.count >= self.limit {
            return Err(ResourceError::RateLimitExceeded);
        }

        self.count = self.count.saturating_add(1);
        Ok(())
    }

    /// Returns the number of remaining operations in the current window.
    pub fn remaining(&self) -> u32 {
        self.limit.saturating_sub(self.count)
    }

    /// Returns `true` if the rate limit has been reached.
    pub fn is_exhausted(&self) -> bool {
        self.count >= self.limit
    }
}

// ---------------------------------------------------------------------------
// StorageQuota
// ---------------------------------------------------------------------------

/// Tracks the number of persistent storage entries written per address and
/// enforces a configurable per-address cap.
///
/// @notice Construct one `StorageQuota` per address per transaction.
///
/// @custom:security
///   - Returns `Err(ResourceError::StorageQuotaExceeded)` rather than
///     silently dropping writes.
///   - Counts use saturating arithmetic.
#[derive(Clone, Debug, PartialEq)]
pub struct StorageQuota {
    /// Maximum storage entries allowed.
    pub quota: u32,
    /// Storage entries written so far.
    pub used: u32,
}

impl StorageQuota {
    /// Creates a `StorageQuota` with the given cap.
    pub fn new(quota: u32) -> Self {
        Self { quota, used: 0 }
    }

    /// Creates a `StorageQuota` with the default cap.
    pub fn default_quota() -> Self {
        Self::new(DEFAULT_STORAGE_QUOTA)
    }

    /// Records one storage write.
    ///
    /// @return `Ok(())` if within quota; `Err(ResourceError::StorageQuotaExceeded)` otherwise.
    pub fn record_write(&mut self) -> Result<(), ResourceError> {
        if self.used >= self.quota {
            return Err(ResourceError::StorageQuotaExceeded);
        }
        self.used = self.used.saturating_add(1);
        Ok(())
    }

    /// Returns the number of remaining writes allowed.
    pub fn remaining(&self) -> u32 {
        self.quota.saturating_sub(self.used)
    }

    /// Returns `true` if the quota has been exhausted.
    pub fn is_exhausted(&self) -> bool {
        self.used >= self.quota
    }

    /// Returns quota utilisation as a percentage (0–100).
    pub fn utilization_pct(&self) -> u32 {
        if self.quota == 0 {
            return 100;
        }
        (self.used.saturating_mul(100) / self.quota).min(100)
    }
}

impl Default for StorageQuota {
    fn default() -> Self {
        Self::default_quota()
    }
}

// ---------------------------------------------------------------------------
// Standalone helpers
// ---------------------------------------------------------------------------

/// Returns `true` when `used` is within `cap` (i.e. `used <= cap`).
///
/// @param used Current usage.
/// @param cap  Maximum allowed.
pub fn within_budget(used: u64, cap: u64) -> bool {
    used <= cap
}

/// Computes utilisation as a percentage (0–100), saturating.
///
/// @param used Current usage.
/// @param cap  Maximum allowed (returns 100 when 0).
pub fn utilization_pct(used: u64, cap: u64) -> u32 {
    if cap == 0 {
        return 100;
    }
    (used.saturating_mul(100) / cap).min(100) as u32
}

/// Estimates the number of storage writes that would be rejected given
/// `total_writes` attempts against a quota of `quota`.
///
/// @param total_writes Total write attempts.
/// @param quota        Per-address storage quota.
/// @return Number of writes that would be rejected (saturating).
pub fn estimated_rejected_writes(total_writes: u32, quota: u32) -> u32 {
    total_writes.saturating_sub(quota)
}

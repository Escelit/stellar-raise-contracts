//! Comprehensive tests for the `resource_management` module.
//!
//! Covers:
//! - `ResourceBudget`: construction, consume, check, remaining, utilisation
//! - `RateLimiter`: record, window reset, exhaustion, remaining
//! - `StorageQuota`: record_write, exhaustion, remaining, utilisation
//! - Standalone helpers: `within_budget`, `utilization_pct`, `estimated_rejected_writes`
//!
//! @custom:test-output
//!   Run: `cargo test -p crowdfund resource_management`
//!   Expected: all tests pass.
//!
//! @custom:security-notes
//!   - Overflow tests confirm saturating arithmetic prevents panics.
//!   - Window-reset tests confirm stale counts do not block legitimate users.
//!   - Quota-exceeded tests confirm writes are rejected, not silently dropped.

#![cfg(test)]

use soroban_sdk::{testutils::Ledger, Env};

use crate::resource_management::{
    estimated_rejected_writes, utilization_pct, within_budget, RateLimiter, ResourceBudget,
    ResourceError, StorageQuota, DEFAULT_CPU_BUDGET, DEFAULT_MEMORY_BUDGET, DEFAULT_RATE_LIMIT,
    DEFAULT_STORAGE_QUOTA, DEFAULT_WINDOW_LEDGERS,
};

// ---------------------------------------------------------------------------
// ResourceBudget
// ---------------------------------------------------------------------------

#[test]
fn resource_budget_new_starts_at_zero() {
    let b = ResourceBudget::new(1_000, 2_000);
    assert_eq!(b.cpu_used, 0);
    assert_eq!(b.memory_used, 0);
    assert_eq!(b.cpu_cap, 1_000);
    assert_eq!(b.memory_cap, 2_000);
}

#[test]
fn resource_budget_default_caps_uses_constants() {
    let b = ResourceBudget::default_caps();
    assert_eq!(b.cpu_cap, DEFAULT_CPU_BUDGET);
    assert_eq!(b.memory_cap, DEFAULT_MEMORY_BUDGET);
}

#[test]
fn resource_budget_consume_within_budget_returns_ok() {
    let mut b = ResourceBudget::new(1_000, 1_000);
    assert!(b.consume(500, 500).is_ok());
    assert_eq!(b.cpu_used, 500);
    assert_eq!(b.memory_used, 500);
}

#[test]
fn resource_budget_consume_exact_cap_returns_ok() {
    let mut b = ResourceBudget::new(1_000, 1_000);
    assert!(b.consume(1_000, 1_000).is_ok());
}

#[test]
fn resource_budget_consume_exceeds_cpu_returns_err() {
    let mut b = ResourceBudget::new(1_000, 1_000);
    let result = b.consume(1_001, 0);
    assert_eq!(result, Err(ResourceError::BudgetExceeded));
}

#[test]
fn resource_budget_consume_exceeds_memory_returns_err() {
    let mut b = ResourceBudget::new(1_000, 1_000);
    let result = b.consume(0, 1_001);
    assert_eq!(result, Err(ResourceError::BudgetExceeded));
}

#[test]
fn resource_budget_check_ok_when_within_caps() {
    let b = ResourceBudget::new(1_000, 1_000);
    assert!(b.check().is_ok());
}

#[test]
fn resource_budget_cpu_remaining() {
    let mut b = ResourceBudget::new(1_000, 1_000);
    b.consume(300, 0).unwrap();
    assert_eq!(b.cpu_remaining(), 700);
}

#[test]
fn resource_budget_memory_remaining() {
    let mut b = ResourceBudget::new(1_000, 1_000);
    b.consume(0, 400).unwrap();
    assert_eq!(b.memory_remaining(), 600);
}

#[test]
fn resource_budget_remaining_saturates_at_zero() {
    let mut b = ResourceBudget::new(100, 100);
    // Force over-cap via saturating_add path.
    b.cpu_used = 200;
    assert_eq!(b.cpu_remaining(), 0);
}

#[test]
fn resource_budget_cpu_utilization_pct_half() {
    let mut b = ResourceBudget::new(1_000, 1_000);
    b.consume(500, 0).unwrap();
    assert_eq!(b.cpu_utilization_pct(), 50);
}

#[test]
fn resource_budget_memory_utilization_pct_full() {
    let mut b = ResourceBudget::new(1_000, 1_000);
    b.consume(0, 1_000).unwrap();
    assert_eq!(b.memory_utilization_pct(), 100);
}

#[test]
fn resource_budget_utilization_pct_zero_cap_returns_100() {
    let b = ResourceBudget::new(0, 0);
    assert_eq!(b.cpu_utilization_pct(), 100);
    assert_eq!(b.memory_utilization_pct(), 100);
}

#[test]
fn resource_budget_saturating_add_does_not_panic() {
    let mut b = ResourceBudget::new(u64::MAX, u64::MAX);
    // Should not panic even with extreme values.
    let _ = b.consume(u64::MAX, u64::MAX);
}

#[test]
fn resource_budget_default_impl() {
    let b = ResourceBudget::default();
    assert_eq!(b.cpu_cap, DEFAULT_CPU_BUDGET);
}

// ---------------------------------------------------------------------------
// RateLimiter
// ---------------------------------------------------------------------------

#[test]
fn rate_limiter_new_starts_at_zero() {
    let r = RateLimiter::new(5, 100, 1_000);
    assert_eq!(r.count, 0);
    assert_eq!(r.limit, 5);
    assert_eq!(r.window_start, 1_000);
}

#[test]
fn rate_limiter_default_for_env() {
    let env = Env::default();
    let r = RateLimiter::default_for_env(&env);
    assert_eq!(r.limit, DEFAULT_RATE_LIMIT);
    assert_eq!(r.window_ledgers, DEFAULT_WINDOW_LEDGERS);
}

#[test]
fn rate_limiter_record_within_limit_returns_ok() {
    let mut r = RateLimiter::new(3, 100, 0);
    assert!(r.record(0).is_ok());
    assert!(r.record(0).is_ok());
    assert!(r.record(0).is_ok());
    assert_eq!(r.count, 3);
}

#[test]
fn rate_limiter_record_exceeds_limit_returns_err() {
    let mut r = RateLimiter::new(2, 100, 0);
    r.record(0).unwrap();
    r.record(0).unwrap();
    let result = r.record(0);
    assert_eq!(result, Err(ResourceError::RateLimitExceeded));
}

#[test]
fn rate_limiter_window_reset_clears_count() {
    let mut r = RateLimiter::new(2, 100, 0);
    r.record(0).unwrap();
    r.record(0).unwrap();
    // Advance past window boundary.
    assert!(r.record(100).is_ok());
    assert_eq!(r.count, 1);
    assert_eq!(r.window_start, 100);
}

#[test]
fn rate_limiter_window_reset_at_exact_boundary() {
    let mut r = RateLimiter::new(1, 50, 0);
    r.record(0).unwrap();
    // Ledger 50 is exactly at window_start(0) + window_ledgers(50).
    assert!(r.record(50).is_ok());
}

#[test]
fn rate_limiter_remaining_decrements() {
    let mut r = RateLimiter::new(5, 100, 0);
    assert_eq!(r.remaining(), 5);
    r.record(0).unwrap();
    assert_eq!(r.remaining(), 4);
}

#[test]
fn rate_limiter_is_exhausted_when_at_limit() {
    let mut r = RateLimiter::new(1, 100, 0);
    assert!(!r.is_exhausted());
    r.record(0).unwrap();
    assert!(r.is_exhausted());
}

#[test]
fn rate_limiter_saturating_window_end() {
    // window_start near u32::MAX — saturating_add prevents overflow.
    let mut r = RateLimiter::new(5, u32::MAX, u32::MAX - 1);
    // Should not panic.
    let _ = r.record(u32::MAX);
}

// ---------------------------------------------------------------------------
// StorageQuota
// ---------------------------------------------------------------------------

#[test]
fn storage_quota_new_starts_at_zero() {
    let q = StorageQuota::new(10);
    assert_eq!(q.used, 0);
    assert_eq!(q.quota, 10);
}

#[test]
fn storage_quota_default_uses_constant() {
    let q = StorageQuota::default_quota();
    assert_eq!(q.quota, DEFAULT_STORAGE_QUOTA);
}

#[test]
fn storage_quota_record_write_within_quota_returns_ok() {
    let mut q = StorageQuota::new(3);
    assert!(q.record_write().is_ok());
    assert!(q.record_write().is_ok());
    assert!(q.record_write().is_ok());
    assert_eq!(q.used, 3);
}

#[test]
fn storage_quota_record_write_exceeds_quota_returns_err() {
    let mut q = StorageQuota::new(1);
    q.record_write().unwrap();
    let result = q.record_write();
    assert_eq!(result, Err(ResourceError::StorageQuotaExceeded));
}

#[test]
fn storage_quota_remaining_decrements() {
    let mut q = StorageQuota::new(5);
    assert_eq!(q.remaining(), 5);
    q.record_write().unwrap();
    assert_eq!(q.remaining(), 4);
}

#[test]
fn storage_quota_is_exhausted_when_at_quota() {
    let mut q = StorageQuota::new(1);
    assert!(!q.is_exhausted());
    q.record_write().unwrap();
    assert!(q.is_exhausted());
}

#[test]
fn storage_quota_utilization_pct_half() {
    let mut q = StorageQuota::new(10);
    for _ in 0..5 {
        q.record_write().unwrap();
    }
    assert_eq!(q.utilization_pct(), 50);
}

#[test]
fn storage_quota_utilization_pct_full() {
    let mut q = StorageQuota::new(4);
    for _ in 0..4 {
        q.record_write().unwrap();
    }
    assert_eq!(q.utilization_pct(), 100);
}

#[test]
fn storage_quota_utilization_pct_zero_quota_returns_100() {
    let q = StorageQuota::new(0);
    assert_eq!(q.utilization_pct(), 100);
}

#[test]
fn storage_quota_default_impl() {
    let q = StorageQuota::default();
    assert_eq!(q.quota, DEFAULT_STORAGE_QUOTA);
}

// ---------------------------------------------------------------------------
// within_budget
// ---------------------------------------------------------------------------

#[test]
fn within_budget_used_less_than_cap() {
    assert!(within_budget(50, 100));
}

#[test]
fn within_budget_used_equal_to_cap() {
    assert!(within_budget(100, 100));
}

#[test]
fn within_budget_used_greater_than_cap() {
    assert!(!within_budget(101, 100));
}

#[test]
fn within_budget_zero_used() {
    assert!(within_budget(0, 0));
}

// ---------------------------------------------------------------------------
// utilization_pct
// ---------------------------------------------------------------------------

#[test]
fn utilization_pct_half() {
    assert_eq!(utilization_pct(50, 100), 50);
}

#[test]
fn utilization_pct_full() {
    assert_eq!(utilization_pct(100, 100), 100);
}

#[test]
fn utilization_pct_over_cap_capped_at_100() {
    assert_eq!(utilization_pct(200, 100), 100);
}

#[test]
fn utilization_pct_zero_cap_returns_100() {
    assert_eq!(utilization_pct(0, 0), 100);
    assert_eq!(utilization_pct(999, 0), 100);
}

#[test]
fn utilization_pct_zero_used() {
    assert_eq!(utilization_pct(0, 1_000), 0);
}

// ---------------------------------------------------------------------------
// estimated_rejected_writes
// ---------------------------------------------------------------------------

#[test]
fn estimated_rejected_writes_none_rejected() {
    assert_eq!(estimated_rejected_writes(5, 10), 0);
}

#[test]
fn estimated_rejected_writes_exact_quota() {
    assert_eq!(estimated_rejected_writes(10, 10), 0);
}

#[test]
fn estimated_rejected_writes_some_rejected() {
    assert_eq!(estimated_rejected_writes(15, 10), 5);
}

#[test]
fn estimated_rejected_writes_zero_quota() {
    assert_eq!(estimated_rejected_writes(5, 0), 5);
}

#[test]
fn estimated_rejected_writes_saturates() {
    // Should not panic.
    let _ = estimated_rejected_writes(u32::MAX, 0);
}

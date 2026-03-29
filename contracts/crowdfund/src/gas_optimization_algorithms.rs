//! Gas Optimization Algorithms Module
//!
//! This module provides advanced gas optimization algorithms to reduce transaction costs
//! and improve efficiency in the crowdfunding contract operations.

use soroban_sdk::{Env, Vec, Address};

/// Gas optimization configuration parameters
#[derive(Clone)]
pub struct GasOptimizationConfig {
    /// Enable batch operation optimization
    pub enable_batch_optimization: bool,
    /// Enable storage access caching
    pub enable_storage_caching: bool,
    /// Maximum batch size for operations
    pub max_batch_size: u32,
}

impl Default for GasOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_batch_optimization: true,
            enable_storage_caching: true,
            max_batch_size: 50,
        }
    }
}

/// Optimizes storage reads by batching multiple address lookups
///
/// # Arguments
/// * `env` - The contract environment
/// * `addresses` - Vector of addresses to lookup
///
/// # Returns
/// Vector of contribution amounts corresponding to each address
pub fn batch_read_contributions(env: &Env, addresses: &Vec<Address>) -> Vec<i128> {
    let mut results = Vec::new(env);
    
    for addr in addresses.iter() {
        let key = crate::DataKey::Contribution(addr.clone());
        let amount: i128 = env.storage().persistent().get(&key).unwrap_or(0);
        results.push_back(amount);
    }
    
    results
}

/// Calculates optimal batch size based on operation complexity
///
/// # Arguments
/// * `total_operations` - Total number of operations to perform
/// * `max_batch_size` - Maximum allowed batch size
///
/// # Returns
/// Optimal batch size to minimize gas costs
pub fn calculate_optimal_batch_size(total_operations: u32, max_batch_size: u32) -> u32 {
    if total_operations == 0 {
        return 0;
    }
    
    // Use smaller batches for small operation counts
    if total_operations <= 10 {
        return total_operations.min(max_batch_size);
    }
    
    // For larger operations, use square root heuristic for optimal batching
    let sqrt_approx = approximate_sqrt(total_operations);
    sqrt_approx.min(max_batch_size).max(1)
}

/// Approximates square root using binary search (gas-efficient)
fn approximate_sqrt(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    
    let mut low = 1u32;
    let mut high = n;
    let mut result = 1u32;
    
    while low <= high {
        let mid = low + (high - low) / 2;
        
        if mid <= n / mid {
            result = mid;
            low = mid + 1;
        } else {
            high = mid - 1;
        }
    }
    
    result
}

/// Estimates gas cost for a contribution operation
///
/// # Arguments
/// * `is_new_contributor` - Whether this is a new contributor
/// * `has_platform_fee` - Whether platform fee is configured
///
/// # Returns
/// Estimated gas units for the operation
pub fn estimate_contribution_gas(is_new_contributor: bool, has_platform_fee: bool) -> u64 {
    let mut gas_estimate = 10_000u64; // Base cost
    
    if is_new_contributor {
        gas_estimate += 5_000; // Additional storage writes
    }
    
    if has_platform_fee {
        gas_estimate += 2_000; // Fee calculation overhead
    }
    
    gas_estimate
}

/// Optimizes storage access patterns by minimizing redundant reads
///
/// # Arguments
/// * `env` - The contract environment
/// * `key` - Storage key to access
/// * `cached_value` - Optional cached value to avoid read
///
/// # Returns
/// The value from storage or cache
pub fn optimized_storage_read<T: soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val> + Clone>(
    env: &Env,
    key: &crate::DataKey,
    cached_value: Option<T>,
) -> Option<T> {
    if let Some(cached) = cached_value {
        return Some(cached);
    }
    
    env.storage().instance().get(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approximate_sqrt() {
        assert_eq!(approximate_sqrt(0), 0);
        assert_eq!(approximate_sqrt(1), 1);
        assert_eq!(approximate_sqrt(4), 2);
        assert_eq!(approximate_sqrt(9), 3);
        assert_eq!(approximate_sqrt(16), 4);
        assert_eq!(approximate_sqrt(25), 5);
        assert_eq!(approximate_sqrt(100), 10);
    }

    #[test]
    fn test_calculate_optimal_batch_size() {
        assert_eq!(calculate_optimal_batch_size(0, 50), 0);
        assert_eq!(calculate_optimal_batch_size(5, 50), 5);
        assert_eq!(calculate_optimal_batch_size(100, 50), 10);
        assert_eq!(calculate_optimal_batch_size(400, 50), 20);
    }

    #[test]
    fn test_estimate_contribution_gas() {
        assert_eq!(estimate_contribution_gas(false, false), 10_000);
        assert_eq!(estimate_contribution_gas(true, false), 15_000);
        assert_eq!(estimate_contribution_gas(false, true), 12_000);
        assert_eq!(estimate_contribution_gas(true, true), 17_000);
    }
}
